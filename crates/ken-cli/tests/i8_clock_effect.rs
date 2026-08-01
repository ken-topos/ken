//! I-8 wall-clock effect acceptance through the public injected runner.

use ken_cli::{run_program, ProgramOutcome, SourceFormat};
use ken_interp::{CaptureHost, ClockTrace, HostHandler};

const CLOCK_PACKAGE: &str = include_str!("../../../catalog/packages/Capability/Time/WallClock.ken.md");

const CLOCK_BODY: &str = r#"
  bind (Coproduct (FSOp APartial) AmbientOp)
       (resp_coproduct (FSOp APartial) AmbientOp
         (fs_resp APartial) ambient_resp)
       Instant ExitCode
    (host_clock APartial Instant wall_now)
    (\first .
      match first {
        MkInstant _firstNanoseconds |->
          bind (Coproduct (FSOp APartial) AmbientOp)
               (resp_coproduct (FSOp APartial) AmbientOp
                 (fs_resp APartial) ambient_resp)
               Instant ExitCode
            (host_clock APartial Instant wall_now)
            (\second .
              match second {
                MkInstant _secondNanoseconds |-> host_exit APartial Success
              })
      })
"#;

fn clock_program(visits: &str) -> String {
    format!(
        "program capabilities FS APartial\n\
         proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)\n\
           : HostIO APartial ExitCode {visits} =\n{CLOCK_BODY}"
    )
}

fn run_once() -> (ProgramOutcome, Vec<ClockTrace>, Vec<u8>) {
    let mut host = CaptureHost::new(Vec::new());
    host.set_clock_script([1_700_000_000_000_000_123, 1_700_000_000_000_000_456]);
    let outcome = run_program(
        &clock_program("visits [Clock]"),
        SourceFormat::Ken,
        &[],
        &[],
        b"",
        &mut host,
    )
    .expect("Clock program runs through the public injected driver");
    let trace = host.clock_trace().to_vec();
    let snapshot = format!("{outcome:?}|{trace:?}").into_bytes();
    (outcome, trace, snapshot)
}

#[test]
fn scripted_wall_clock_is_byte_identical_and_reaches_the_injected_host() {
    let first = run_once();
    let second = run_once();

    assert_eq!(first.0, ProgramOutcome { exit_status: 0 });
    assert_eq!(
        first.2, second.2,
        "independent snapshots must match bytewise"
    );
    assert_eq!(
        first.1,
        vec![
            ClockTrace::WallNow {
                nanoseconds: 1_700_000_000_000_000_123_i128.into(),
            },
            ClockTrace::WallNow {
                nanoseconds: 1_700_000_000_000_000_456_i128.into(),
            },
        ],
        "both reads must reach CaptureHost in program order"
    );
}

#[test]
fn fixed_clock_repeats_one_value_and_traces_each_read() {
    let mut host = CaptureHost::new(Vec::new());
    host.set_fixed_clock(-42);
    assert_eq!(host.clock_wall_now(), (-42_i128).into());
    assert_eq!(host.clock_wall_now(), (-42_i128).into());
    assert_eq!(
        host.clock_trace(),
        &[
            ClockTrace::WallNow {
                nanoseconds: (-42_i128).into(),
            },
            ClockTrace::WallNow {
                nanoseconds: (-42_i128).into(),
            },
        ]
    );
}

#[test]
fn undeclared_clock_row_is_a_named_effect_escape() {
    let mut env = ken_elaborator::ElabEnv::new().expect("prelude");
    let error = env
        .elaborate_file(&clock_program(""))
        .expect_err("calling wall_now without Clock in the row must reject");
    match error {
        ken_elaborator::ElabError::TypeMismatch { reason, .. } => {
            assert!(
                reason.contains("EffectEscapes"),
                "wrong rejection: {reason}"
            );
            assert!(reason.contains("Clock"), "missing Clock witness: {reason}");
        }
        other => panic!("expected named EffectEscapes rejection, got {other:?}"),
    }
}

#[test]
fn clock_package_is_structural_zero_trust_and_declares_no_ordering_law() {
    let extracted = ken_elaborator::literate::extract_ken_md(CLOCK_PACKAGE)
        .expect("Capability.Time.WallClock package must extract");
    for forbidden in [
        "Axiom",
        "postulate",
        "primitive",
        "opaque",
        "monotonic",
        "leq",
        "≤",
    ] {
        assert!(
            !extracted.source.contains(forbidden),
            "checked Clock package must not declare {forbidden}"
        );
    }
    assert!(
        CLOCK_PACKAGE.contains("declares no ordering or monotonicity law"),
        "the wall-clock absence of an ordering law must be stated explicitly"
    );

    let mut env = ken_elaborator::ElabEnv::new().expect("prelude");
    let before = env.env.trusted_base();
    env.elaborate_ken_md_file(CLOCK_PACKAGE)
        .expect("ordinary Capability.Time.WallClock package elaborates");
    assert_eq!(before, env.env.trusted_base());
    for name in [
        "Instant",
        "MkInstant",
        "ClockOp",
        "WallNow",
        "clock_resp",
        "wall_now",
        "instant_nanoseconds",
        "replace_instant_nanoseconds",
    ] {
        let id = env.globals[name];
        assert!(
            !env.env.trusted_base().contains(&id),
            "{name} must be ordinary kernel-checked Ken"
        );
    }
}
