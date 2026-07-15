use std::process::Command;

const PURE_PROGRAM: &str = r#"program capabilities FS APartial
fn main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode = host_exit APartial Success
"#;

fn output_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ken-px4b-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn real_source_builds_one_identity_bound_linked_process_artifact() {
    let dir = output_dir("pure");
    let output =
        ken_cli::build_native_program(PURE_PROGRAM, ken_cli::SourceFormat::Ken, "px4b-pure", &dir)
            .expect("checked source reaches native artifact");

    assert_eq!(
        output.package.core_semantic_hash,
        output.runtime_program.core_semantic_hash
    );
    assert_eq!(
        output.package.artifact_hash,
        output.runtime_program.artifact_hash
    );
    assert_eq!(
        output.artifact.runtime_artifact.core_semantic_hash,
        output.package.core_semantic_hash
    );
    assert!(matches!(
        output.report.runtime_lowering,
        ken_elaborator::compiler_driver::ReportFact::Emitted
    ));
    assert!(matches!(
        output.report.native_artifact,
        ken_elaborator::compiler_driver::ReportFact::Emitted
    ));
    assert!(!output.plan.main().to_string().contains("prelude::"));

    let mut stale_plan = output.package.clone();
    let plan_bytes = stale_plan
        .artifact
        .semantic
        .metadata
        .values_mut()
        .find(|bytes| bytes.starts_with(b"NativeEntrypointPlanV1\0"))
        .expect("plan is contained in semantic inputs");
    plan_bytes.push(0xff);
    assert!(matches!(
        ken_elaborator::checked_core::validate_checked_core_package(&stale_plan),
        Err(ken_elaborator::checked_core::CheckedCorePackageError::SemanticHashMismatch { .. })
    ));
    stale_plan.core_semantic_hash =
        ken_elaborator::checked_core::semantic_fingerprint(&stale_plan.artifact.semantic);
    assert!(matches!(
        ken_elaborator::checked_core::validate_checked_core_package(&stale_plan),
        Err(ken_elaborator::checked_core::CheckedCorePackageError::ArtifactHashMismatch { .. })
    ));

    let mut command = Command::new(&output.artifact.executable_path);
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        command.arg(std::ffi::OsString::from_vec(vec![0xff]));
    }
    #[cfg(not(unix))]
    command.arg("raw-argv");
    let ran = command
        .env("KEN_PX4B_RAW", "bytes")
        .current_dir(&dir)
        .output()
        .expect("linked process artifact runs with fresh process data");
    assert_eq!(ran.status.code(), Some(0), "stderr: {:?}", ran.stderr);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn authority_mismatch_fails_before_any_artifact_is_written() {
    let dir = output_dir("mismatch");
    let source = PURE_PROGRAM.replace("ProgramCaps APartial", "ProgramCaps AFull");
    let error =
        ken_cli::build_native_program(&source, ken_cli::SourceFormat::Ken, "px4b-mismatch", &dir)
            .expect_err("declared/type authority mismatch must reject");
    assert!(matches!(
        error,
        ken_elaborator::compiler_driver::NativeProgramBuildError::Admission(
            ken_elaborator::program_admission::ProgramAdmissionError::InvalidMainAbi { .. }
        )
    ));
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn one_vis_reaches_the_named_px5_lane() {
    let dir = output_dir("vis");
    let source = r#"program capabilities FS APartial
proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  host_program APartial (print_line "px5")
"#;
    let error = ken_cli::build_native_program(source, ken_cli::SourceFormat::Ken, "px4b-vis", &dir)
        .expect_err("base producer must stop at the PX5 effect lane");
    assert!(
        matches!(
            error,
            ken_elaborator::compiler_driver::NativeProgramBuildError::Unavailable(ref lane)
                if lane.lane == "host_effect_lowering_unavailable"
        ),
        "unexpected error: {error:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn native_build_subcommand_reaches_the_same_public_producer() {
    let dir = output_dir("cli");
    let source_path = dir.join("main.ken");
    let artifact_dir = dir.join("artifact");
    std::fs::write(&source_path, PURE_PROGRAM).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_ken"))
        .arg("native-build")
        .arg(&source_path)
        .arg(&artifact_dir)
        .output()
        .expect("native-build command runs");
    assert_eq!(built.status.code(), Some(0), "stderr: {:?}", built.stderr);
    let executable = String::from_utf8(built.stdout).unwrap();
    let executable = std::path::PathBuf::from(executable.trim());
    assert!(executable.is_file());
    let ran = Command::new(executable)
        .output()
        .expect("CLI artifact runs");
    assert_eq!(ran.status.code(), Some(0), "stderr: {:?}", ran.stderr);
    let _ = std::fs::remove_dir_all(dir);
}
