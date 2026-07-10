//! `effect-composition` D5 §3 leg (2) — the SECOND distinct `(g,h)` pairing
//! through the general `resp_coproduct` (AC3's generality face), driving the real
//! `ken` CLI binary (subprocess) against a real elaborated program. No test
//! in this file hand-constructs a `Coproduct`/`InL`/`InR` value.
//!
//! Distinct from the {FS,Console} terminal-driver peel (`run_io`'s D3
//! mechanism, `fs_read_file_lines_flip_e2e.rs`): this exercises the
//! **pure-handler** role instead — `run_state` (a kernel-re-checked
//! `declare_def` fold, COEXIST-preserved, BV4) peels `StateOp Nat` out of
//! `Coproduct (StateOp Nat) ConsoleOp` via the now-general `resp_coproduct`, threads the
//! state, and re-emits the untouched `ConsoleOp` residual tree, which
//! `run_io` then runs directly (no `Coproduct` wrapper left to peel). Two
//! genuinely different coproduct-execution mechanisms, both resting on the
//! same general `resp_coproduct` (D1) — the executable ≥2-distinct-pairings
//! discriminator (`effect-composition-conformance.md` §3 leg (2)).

use std::path::PathBuf;
use std::process::Command;

fn ken_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ken"))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `get` (State) sequenced with `inject_r (print_line ...)` (Console) under
/// `Coproduct (StateOp Nat) ConsoleOp`, run through `run_state` then `run_io`.
/// `Pair Unit Nat` is a surface-nameable alias for `run_state`'s real return
/// type `Sigma Unit Nat` (`prelude.rs`'s `Pair := \a b. Sigma a b`).
const PROG: &str = r#"
const prog : ITree ConsoleOp console_resp (Pair Unit Nat) =
  run_state Nat ConsoleOp console_resp Unit Zero
    (bind (Coproduct (StateOp Nat) ConsoleOp) (resp_coproduct (StateOp Nat) ConsoleOp (resp_state Nat) console_resp) Nat Unit
      (get Nat ConsoleOp console_resp MkUnit)
      (\n . inject_r (StateOp Nat) ConsoleOp (resp_state Nat) console_resp Unit (print_line "state-console-pairing")))
"#;

#[test]
fn state_console_pairing_runs_through_run_state_then_run_io() {
    let tmp_dir = std::env::temp_dir().join("ken-effect-composition-e2e");
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let path = tmp_dir.join("state_console.ken");
    std::fs::write(&path, PROG).expect("write program");

    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&path)
        .current_dir(workspace_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn `ken run`: {e}"));

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "must succeed; stderr: {stderr}");
    assert_eq!(stdout, "state-console-pairing\n", "the Console residual must print exactly once");
}
