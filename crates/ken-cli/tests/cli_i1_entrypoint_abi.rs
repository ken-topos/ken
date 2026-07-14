//! Program-I I-1 acceptance: named entrypoint, raw argv, and total exit map.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn ken_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ken"))
}

fn fixture(name: &str, source: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ken-cli-i1-entrypoint-abi");
    fs::create_dir_all(&dir).expect("create fixture directory");
    let path = dir.join(format!("{name}.ken"));
    fs::write(&path, source).expect("write fixture");
    path
}

fn run(name: &str, source: &str, args: impl IntoIterator<Item = OsString>) -> Output {
    let source = if source.trim_start().starts_with("program") {
        source.to_owned()
    } else {
        format!("program capabilities FS APartial\n{source}")
    };
    let mut command = Command::new(ken_bin());
    command.arg("run").arg(fixture(name, &source));
    command.args(args);
    command.output().expect("run ken fixture")
}

fn abi_main(body: &str) -> String {
    format!(
        "proc main (_input : ProcessInput) (_caps : ProgramCaps APartial) \
         : HostIO APartial ExitCode visits [Console] = {body}\n"
    )
}

#[test]
fn named_main_runs_when_it_is_not_the_last_declaration() {
    let source = format!(
        "{}\nconst afterMain : Nat = Zero\n",
        abi_main("host_program APartial (print_line \"named-main\")")
    );
    let output = run("not-last", &source, []);
    assert_eq!(output.status.code(), Some(0), "stderr: {:?}", output.stderr);
    assert_eq!(output.stdout, b"named-main\n");
}

#[test]
fn missing_and_duplicate_main_report_distinct_named_errors() {
    let missing = run("missing", "const helper : Nat = Zero\n", []);
    assert_eq!(missing.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains("missing entrypoint 'main'"),
        "stderr: {:?}",
        missing.stderr
    );

    let duplicate = run(
        "duplicate",
        &format!(
            "{}{}",
            abi_main("host_exit APartial Success"),
            abi_main("host_exit APartial Success")
        ),
        [],
    );
    assert_eq!(duplicate.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&duplicate.stderr).contains("duplicate entrypoint 'main'"),
        "stderr: {:?}",
        duplicate.stderr
    );
}

#[test]
fn named_main_with_wrong_signature_is_rejected_before_execution() {
    let output = run(
        "wrong-signature",
        "program capabilities FS APartial\n\
         fn main (_input : ProcessInput) (_caps : ProgramCaps APartial) : Nat = Zero\n",
        [],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid entrypoint 'main'"),
        "stderr: {:?}",
        output.stderr
    );
}

#[cfg(unix)]
#[test]
fn argv_after_separator_round_trips_non_utf8_and_pre_separator_is_rejected() {
    use std::os::unix::ffi::OsStringExt;

    let source = r#"program capabilities FS APartial
proc main (input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  match input {
    MkProcessInput arguments _environment _cwd |->
      match arguments {
        Nil |-> host_exit APartial (Failure 2) ;
        Cons bytes _rest |->
          match bytes_at bytes 0 {
            None |-> host_exit APartial (Failure 3) ;
            Some byte |-> match eq_int (uint8_to_int byte) 255 {
              True  |-> host_program APartial (print_line "raw-ok") ;
              False |-> host_exit APartial (Failure 3)
            }
          }
      }
  }
"#;
    let raw = OsString::from_vec(vec![0xff]);
    let output = run("raw-argv", source, [OsString::from("--"), raw]);
    assert_eq!(output.status.code(), Some(0), "stderr: {:?}", output.stderr);
    assert_eq!(output.stdout, b"raw-ok\n");

    let rejected = run(
        "pre-separator",
        &abi_main("host_exit APartial Success"),
        [OsString::from("--unknown")],
    );
    assert_eq!(rejected.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("unexpected argument before '--'"),
        "stderr: {:?}",
        rejected.stderr
    );
}

#[test]
fn exit_code_mapping_covers_every_contract_arm() {
    for (name, value, expected) in [
        ("success", "Success", 0),
        ("failure-three", "Failure 3", 3),
        ("failure-max", "Failure 255", 255),
        ("failure-zero", "Failure 0", 1),
    ] {
        let output = run(
            name,
            &abi_main(&format!("host_exit APartial ({value})")),
            [],
        );
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{name}: stderr {:?}",
            output.stderr
        );
    }
}
