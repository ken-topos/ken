use std::path::PathBuf;
use std::process::Command;

fn ken_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ken"))
}

#[test]
fn ken_run_accepts_exact_ken_md_fence() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("literate_success.ken.md");
    std::fs::write(
        &path,
        r#"A literate Ken file.

```ken
proc main (_input : ProcessInput) (_caps : ProgramCaps) : HostIO ExitCode visits [Console] = host_program (print_line "literate ok")
```
"#,
    )
    .expect("write fixture");

    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("spawn ken run");

    assert!(
        output.status.success(),
        "ken run should accept .ken.md, got {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "literate ok\n");
}

#[test]
fn ken_run_reports_original_offset_for_fenced_error() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("literate_error.ken.md");
    let fixture = "Prose before the program.\n```ken\nproc main : IO Unit visits [Console] = print_line missing_name\n```\n";
    std::fs::write(&path, fixture).expect("write fixture");
    let expected_offset = fixture
        .find("missing_name")
        .expect("fixture contains missing_name");

    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("spawn ken run");

    assert!(
        !output.status.success(),
        "invalid fenced program must reject"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing_name"),
        "stderr should name missing binding: {stderr}"
    );
    assert!(
        stderr.contains(&format!("start: {expected_offset}")),
        "stderr should use original .ken.md byte offset {expected_offset}: {stderr}"
    );
}

#[test]
fn ken_ignore_fence_is_prose_only_for_cli() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("literate_ignore.ken.md");
    std::fs::write(
        &path,
        r#"```ken ignore
proc main (_input : ProcessInput) (_caps : ProgramCaps) : HostIO ExitCode visits [Console] = host_program (print_line "ignored")
```
"#,
    )
    .expect("write fixture");

    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("spawn ken run");

    assert!(
        !output.status.success(),
        "ken ignore must not compile as Ken"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing entrypoint 'main'"),
        "only non-compiled fences should leave no entrypoint; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn ken_run_accepts_a_correctly_failing_reject_block() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("literate_reject_ok.ken.md");
    std::fs::write(
        &path,
        r#"```ken
proc main (_input : ProcessInput) (_caps : ProgramCaps) : HostIO ExitCode visits [Console] = host_program (print_line "reject ok")
```
```ken reject
const bad : Nat = undefinedName
```
"#,
    )
    .expect("write fixture");

    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("spawn ken run");

    assert!(
        output.status.success(),
        "a reject block that correctly fails to elaborate must not fail the run, got {:?}; \
         stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "reject ok\n");
}

#[test]
fn ken_run_rejects_a_stale_reject_block() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("literate_reject_stale.ken.md");
    let fixture = r#"```ken
proc main (_input : ProcessInput) (_caps : ProgramCaps) : HostIO ExitCode visits [Console] = host_program (print_line "unreachable")
```
```ken reject
const stale : Nat = Zero
```
"#;
    std::fs::write(&path, fixture).expect("write fixture");

    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("spawn ken run");

    assert!(
        !output.status.success(),
        "a reject block that unexpectedly elaborates must fail the run"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("reject"),
        "stderr should name the stale block's role: {stderr}"
    );
}

#[test]
fn ken_run_treats_unrecognized_ken_role_as_a_hard_error() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("literate_bogus_role.ken.md");
    std::fs::write(
        &path,
        r#"```ken bogus
const whatever : Nat = Zero
```
"#,
    )
    .expect("write fixture");

    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("spawn ken run");

    assert!(
        !output.status.success(),
        "an unrecognized 'ken' fence role must hard-error, not silently fall back to prose"
    );
}

#[test]
fn plain_ken_run_path_still_executes() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("plain_success.ken");
    std::fs::write(
        &path,
        r#"proc main (_input : ProcessInput) (_caps : ProgramCaps) : HostIO ExitCode visits [Console] = host_program (print_line "plain ok")
"#,
    )
    .expect("write fixture");

    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("spawn ken run");

    assert!(
        output.status.success(),
        "plain .ken path should still execute, got {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "plain ok\n");
}
