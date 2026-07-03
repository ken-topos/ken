//! `ken run` Console-execution regression test (`wp/console-harvest-fix`,
//! AC3). Runs the real `ken` binary as a subprocess against a Console/IO
//! program and asserts stdout — the exact path that regressed silently
//! (the Console-ID harvest looked up dotted/two-param names that were
//! never registered, so every `ken run` on an IO program hard-failed
//! with exit 2 before running anything). Reuses the canonical
//! `examples/rosetta/hello-world` fixture rather than a throwaway one.

use std::path::PathBuf;
use std::process::Command;

fn ken_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ken"))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn ken_run_executes_console_program_and_prints_stdout() {
    let example = workspace_root().join("examples/rosetta/hello-world/hello-world.ken");
    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&example)
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
        "Hello, World!\n",
        "ken run should print the Console program's output exactly"
    );
}
