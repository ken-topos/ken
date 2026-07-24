//! Permanent fail-closed census for RT-NATIVE-FNSPLIT recut Phase 1.
//!
//! This test is ignored in the ordinary suite because it deliberately compiles
//! five large linked-native programs. Run it explicitly with:
//!
//! `scripts/ken-cargo test -p ken-cli --test native_partition_scaling_census
//! -- --ignored --exact native_partition_scaling_census_is_complete_and_bounded`
//!
//! Every depth runs in its own Linux process group under `prlimit`. The parent
//! enforces both an address-space/CPU ceiling and an independent wall deadline,
//! while sampling `/proc/<pid>/status` for the child's peak resident high-water
//! mark. Timeout, resource-limit termination, malformed output, and a missing
//! metric are all the explicit third outcome: `could not determine`, which
//! fails this test.
//!
//! Promise class: durable invariant. The harness asserts completeness,
//! fail-closed resource bounds, and relational finite differences without
//! freezing any current count or growth exponent.

#![cfg(target_os = "linux")]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Read as _;
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const CHILD_DEPTH: &str = "KEN_NATIVE_PARTITION_CENSUS_CHILD_DEPTH";
const CENSUS_PREFIX: &str = "KEN_NATIVE_PARTITION_CENSUS_V1 ";
const ADDRESS_SPACE_BYTES: u64 = 6 * 1024 * 1024 * 1024;
const CPU_SECONDS: u64 = 240;
const WALL_SECONDS: u64 = 300;
const STACK_BYTES: u64 = 512 * 1024 * 1024;

const KINDS: [&str; 7] = [
    "exported_root",
    "arm",
    "producer_kont",
    "source_arm",
    "source_kont",
    "cleanup_step",
    "all",
];

const METRICS: [&str; 16] = [
    "nodes",
    "source_nodes",
    "states_per_source_max",
    "edges",
    "helpers",
    "clif_instructions",
    "clif_bytes",
    "descriptor_bytes_constructed",
    "descriptor_bytes_retained",
    "exact_comparison_bytes",
    "frame_fields_total",
    "frame_fields_max",
    "static_key_bytes_max",
    "env_len_max",
    "pending_len_max",
    "path_len_max",
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct CensusRow {
    depth: usize,
    kind: String,
    values: BTreeMap<String, u64>,
}

#[derive(Debug)]
struct BoundedRun {
    rows: Vec<CensusRow>,
    wall_ms: u64,
    peak_rss_kib: u64,
}

fn output_dir(depth: usize) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ken-native-partition-census-n{depth}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).expect("create census output directory");
    path
}

fn nested_body(remaining: usize) -> String {
    if remaining == 0 {
        return "leaf_body".to_string();
    }
    let child = nested_body(remaining - 1);
    format!(
        r#"(\buffer.
          bind (Coproduct (FSOp APartial) AmbientOp)
            (resp_coproduct (FSOp APartial) AmbientOp
              (fs_resp APartial) ambient_resp)
            (Result ResourceError (ResourceBracketResult Unit Unit))
            (ResourceBodyResult Unit Unit)
            (withBuffer APartial Unit Unit (1 : Int) {child})
            (\outcome. body_result outcome))"#
    )
}

fn nested_resource_bracket_source(depth: usize) -> String {
    assert!((3..=7).contains(&depth));
    let mut source = String::from(
        r#"program capabilities FS APartial
fn leaf_body (_buffer : Resource Buffer)
  : HostIO APartial (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp
      (fs_resp APartial) ambient_resp)
    (ResourceBodyResult Unit Unit)
    (ResourceBodyOk Unit Unit MkUnit)

fn body_result
  (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : HostIO APartial (ResourceBodyResult Unit Unit) =
  match outcome {
    Ok (ResourceBracketOk unit) |->
      Ret (Coproduct (FSOp APartial) AmbientOp)
        (resp_coproduct (FSOp APartial) AmbientOp
          (fs_resp APartial) ambient_resp)
        (ResourceBodyResult Unit Unit)
        (ResourceBodyOk Unit Unit MkUnit);
    Ok bracket |->
      Ret (Coproduct (FSOp APartial) AmbientOp)
        (resp_coproduct (FSOp APartial) AmbientOp
          (fs_resp APartial) ambient_resp)
        (ResourceBodyResult Unit Unit)
        (ResourceBodyErr Unit Unit MkUnit);
    Err error |->
      Ret (Coproduct (FSOp APartial) AmbientOp)
        (resp_coproduct (FSOp APartial) AmbientOp
          (fs_resp APartial) ambient_resp)
        (ResourceBodyResult Unit Unit)
        (ResourceBodyErr Unit Unit MkUnit)
  }

"#,
    );
    let root_body = nested_body(depth - 1);
    writeln!(
        source,
        r#"fn finish
  (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : HostIO APartial ExitCode =
  match outcome {{
    Ok (ResourceBracketOk unit) |-> host_exit APartial Success;
    Ok (ResourceBracketBodyError error) |-> host_exit APartial (Failure 81);
    Ok (ResourceBracketReleaseError error) |-> host_exit APartial (Failure 82);
    Ok (ResourceBracketBodyAndReleaseError body_error release_error) |->
      host_exit APartial (Failure 83);
    Err error |-> host_exit APartial (Failure 84)
  }}

proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [FS] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp
      (fs_resp APartial) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) ExitCode
    (withBuffer APartial Unit Unit (1 : Int) {root_body})
    (\outcome. finish outcome)
"#
    )
    .expect("write generated entrypoint");
    source
}

fn compile_child(depth: usize) {
    let root = output_dir(depth);
    let source = nested_resource_bracket_source(depth);
    let compile = std::thread::Builder::new()
        .name(format!("partition-census-n{depth}"))
        .stack_size(256 * 1024 * 1024)
        .spawn({
            let root = root.clone();
            move || {
                ken_cli::build_native_program(
                    &source,
                    ken_cli::SourceFormat::Ken,
                    &format!("native-partition-census-n{depth}"),
                    &root,
                )
            }
        })
        .expect("spawn large-stack census compiler")
        .join()
        .expect("large-stack census compiler did not panic");
    let _output = compile.unwrap_or_else(|error| {
        panic!("depth {depth} linked-native compilation failed: {error:?}")
    });
    std::fs::remove_dir_all(root).expect("remove census output directory");
}

fn vm_hwm_kib(pid: u32) -> Option<u64> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

fn kill_process_group(pid: u32) {
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .status();
}

fn parse_rows(depth: usize, stderr: &str) -> Result<Vec<CensusRow>, String> {
    let mut rows = Vec::new();
    for line in stderr
        .lines()
        .filter(|line| line.starts_with(CENSUS_PREFIX))
    {
        let mut fields = BTreeMap::new();
        for field in line[CENSUS_PREFIX.len()..].split_whitespace() {
            let (name, value) = field
                .split_once('=')
                .ok_or_else(|| format!("malformed census field {field:?}"))?;
            fields.insert(name.to_string(), value.to_string());
        }
        let kind = fields
            .remove("kind")
            .ok_or_else(|| "census row omitted kind".to_string())?;
        let mut values = BTreeMap::new();
        for metric in METRICS {
            let value = fields
                .remove(metric)
                .ok_or_else(|| format!("{kind} omitted metric {metric}"))?
                .parse::<u64>()
                .map_err(|error| format!("{kind}/{metric} is not u64: {error}"))?;
            values.insert(metric.to_string(), value);
        }
        if !fields.is_empty() {
            return Err(format!("{kind} emitted unknown fields {fields:?}"));
        }
        rows.push(CensusRow {
            depth,
            kind,
            values,
        });
    }
    for kind in KINDS {
        let count = rows.iter().filter(|row| row.kind == kind).count();
        if count != 1 {
            return Err(format!("expected one {kind} row, found {count}"));
        }
    }
    if rows.len() != KINDS.len() {
        return Err(format!(
            "expected {} census rows, found {}",
            KINDS.len(),
            rows.len()
        ));
    }
    Ok(rows)
}

fn bounded_run(depth: usize) -> Result<BoundedRun, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("resolve census test executable: {error}"))?;
    let mut child = Command::new("prlimit")
        .args([
            &format!("--as={ADDRESS_SPACE_BYTES}"),
            &format!("--cpu={CPU_SECONDS}"),
            &format!("--stack={STACK_BYTES}"),
            "--",
        ])
        .arg(executable)
        .args([
            "--ignored",
            "--exact",
            "native_partition_scaling_census_is_complete_and_bounded",
            "--nocapture",
        ])
        .env(CHILD_DEPTH, depth.to_string())
        .env("KEN_NATIVE_PARTITION_METRICS", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|error| format!("spawn prlimit child: {error}"))?;
    let pid = child.id();
    let start = Instant::now();
    let deadline = start + Duration::from_secs(WALL_SECONDS);
    let mut peak_rss_kib = 0;
    let status = loop {
        peak_rss_kib = peak_rss_kib.max(vm_hwm_kib(pid).unwrap_or(0));
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("wait for census child: {error}"))?
        {
            break status;
        }
        if Instant::now() >= deadline {
            kill_process_group(pid);
            let _ = child.wait();
            return Err(format!("wall deadline of {WALL_SECONDS}s exceeded"));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let wall_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let mut stdout = String::new();
    let mut stderr = String::new();
    child
        .stdout
        .take()
        .expect("census child stdout is piped")
        .read_to_string(&mut stdout)
        .map_err(|error| format!("read census child stdout: {error}"))?;
    child
        .stderr
        .take()
        .expect("census child stderr is piped")
        .read_to_string(&mut stderr)
        .map_err(|error| format!("read census child stderr: {error}"))?;
    if !status.success() {
        return Err(format!(
            "resource-bounded child exited {status}; stdout={stdout:?}; stderr={stderr:?}"
        ));
    }
    let rows = parse_rows(depth, &stderr)?;
    if peak_rss_kib == 0 {
        return Err("peak RSS was not observable in /proc".to_string());
    }
    Ok(BoundedRun {
        rows,
        wall_ms,
        peak_rss_kib,
    })
}

fn row<'a>(runs: &'a [BoundedRun], depth: usize, kind: &str) -> &'a CensusRow {
    runs.iter()
        .flat_map(|run| &run.rows)
        .find(|row| row.depth == depth && row.kind == kind)
        .unwrap_or_else(|| panic!("missing row n={depth} kind={kind}"))
}

fn print_rows_and_differences(runs: &[BoundedRun]) {
    for run in runs {
        for census in &run.rows {
            let mut line = format!(
                "RT_NATIVE_FNSPLIT_PHASE1_ROW n={} kind={}",
                census.depth, census.kind
            );
            for metric in METRICS {
                write!(line, " {metric}={}", census.values[metric]).expect("write census row");
            }
            if census.kind == "all" {
                write!(
                    line,
                    " compile_wall_ms={} peak_rss_kib={}",
                    run.wall_ms, run.peak_rss_kib
                )
                .expect("write process metrics");
            }
            eprintln!("{line}");
        }
    }
    for kind in KINDS {
        for metric in METRICS {
            let values = (3..=7)
                .map(|depth| i128::from(row(runs, depth, kind).values[metric]))
                .collect::<Vec<_>>();
            let first = values
                .windows(2)
                .map(|pair| pair[1] - pair[0])
                .collect::<Vec<_>>();
            let second = first
                .windows(2)
                .map(|pair| pair[1] - pair[0])
                .collect::<Vec<_>>();
            eprintln!(
                "RT_NATIVE_FNSPLIT_PHASE1_DIFF kind={kind} metric={metric} \
                 first={first:?} second={second:?}"
            );
        }
    }
    for (metric, values) in [
        (
            "compile_wall_ms",
            runs.iter()
                .map(|run| i128::from(run.wall_ms))
                .collect::<Vec<_>>(),
        ),
        (
            "peak_rss_kib",
            runs.iter()
                .map(|run| i128::from(run.peak_rss_kib))
                .collect::<Vec<_>>(),
        ),
    ] {
        let first = values
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect::<Vec<_>>();
        let second = first
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect::<Vec<_>>();
        eprintln!(
            "RT_NATIVE_FNSPLIT_PHASE1_DIFF kind=all metric={metric} \
             first={first:?} second={second:?}"
        );
    }
}

fn hold_is_falsified(runs: &[BoundedRun]) -> bool {
    let constant_width_metrics = [
        "frame_fields_max",
        "static_key_bytes_max",
        "env_len_max",
        "pending_len_max",
        "path_len_max",
        "states_per_source_max",
    ];
    constant_width_metrics.iter().all(|metric| {
        let first = row(runs, 3, "all").values[*metric];
        (4..=7).all(|depth| row(runs, depth, "all").values[*metric] == first)
    }) && [
        "nodes",
        "edges",
        "helpers",
        "clif_instructions",
        "clif_bytes",
        "descriptor_bytes_constructed",
        "descriptor_bytes_retained",
        "exact_comparison_bytes",
        "frame_fields_total",
    ]
    .iter()
    .all(|metric| {
        let values = (3..=7)
            .map(|depth| i128::from(row(runs, depth, "all").values[*metric]))
            .collect::<Vec<_>>();
        let differences = values
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect::<Vec<_>>();
        differences.windows(2).all(|pair| pair[0] == pair[1])
    })
}

#[test]
#[ignore = "explicit 5-depth resource-bounded native scaling census"]
fn native_partition_scaling_census_is_complete_and_bounded() {
    if let Some(depth) = std::env::var(CHILD_DEPTH)
        .ok()
        .and_then(|depth| depth.parse::<usize>().ok())
    {
        compile_child(depth);
        return;
    }

    let mut runs = Vec::new();
    let mut indeterminate = Vec::new();
    for depth in 3..=7 {
        eprintln!("RT_NATIVE_FNSPLIT_PHASE1_START n={depth}");
        match bounded_run(depth) {
            Ok(run) => runs.push(run),
            Err(error) => {
                eprintln!(
                    "RT_NATIVE_FNSPLIT_PHASE1_RESULT n={depth} outcome=could_not_determine \
                     error={error}"
                );
                indeterminate.push((depth, error));
            }
        }
    }
    if !indeterminate.is_empty() {
        panic!("RT-NATIVE-FNSPLIT Phase 1 could not determine every n=3..7 row: {indeterminate:?}");
    }
    print_rows_and_differences(&runs);
    let outcome = if hold_is_falsified(&runs) {
        "hold_falsified"
    } else {
        "hold_confirmed"
    };
    eprintln!("RT_NATIVE_FNSPLIT_PHASE1_OUTCOME {outcome}");
}
