//! `ken run` String-op execution regression test (`wp/CLI-string-wiring-fix`,
//! AC2, VAL2 finding #7). Runs the real `ken` binary as a subprocess against
//! a program using `string_to_list_char`/`list_char_to_string` — the exact
//! path that regressed silently: `run_file` built an `EvalStore` but never
//! set `store.list_char_ids`, so String ops degraded to `Neutral`
//! (`ken-interp`'s "never silently wrong" default) instead of erroring,
//! surfacing only as `unhandled effect: Ctor { .. Neutral .. }` at the
//! Console-driver boundary. A self-contained fixture is used (written to a
//! per-test temp file) rather than a checked-in `examples/rosetta/` program,
//! since this WP fixes CLI plumbing, not a Rosetta example — no Language
//! `examples/` fixture using String ops was landed at the time of this fix.

use std::path::PathBuf;
use std::process::Command;

fn ken_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ken"))
}

const FIXTURE: &str = r#"
fn list_append (a : Type) (xs : List a) (ys : List a) : List a =
  match xs { Nil |-> ys ; Cons x xs2 |-> Cons a x (list_append a xs2 ys) }

fn shout (s : String) : String =
  list_char_to_string (list_append Char (string_to_list_char s) (string_to_list_char "!"))

proc main : IO Unit visits [Console] = print_line (shout "hi")
"#;

#[test]
fn ken_run_executes_string_op_program_and_prints_stdout() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let fixture_path = dir.join("string_op_exec_fixture.ken");
    std::fs::write(&fixture_path, FIXTURE).expect("failed to write fixture .ken file");

    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&fixture_path)
        .output()
        .expect("failed to spawn `ken run`");

    assert!(
        output.status.success(),
        "ken run should exit 0, got {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "hi!\n",
        "ken run should print the round-tripped String, not degrade to Neutral/unhandled effect"
    );
}
