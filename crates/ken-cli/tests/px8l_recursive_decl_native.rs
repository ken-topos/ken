fn output_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ken-px8l-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn declaration_refs(
    expr: &ken_runtime::RuntimeExpr,
    output: &mut std::collections::BTreeSet<String>,
) {
    use ken_runtime::RuntimeExpr;
    match expr {
        RuntimeExpr::DeclarationRef { symbol } => {
            output.insert(symbol.clone());
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for arg in args {
                declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Let { value, body } => {
            declaration_refs(value, output);
            declaration_refs(body, output);
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            declaration_refs(scrutinee, output);
            declaration_refs(then_expr, output);
            declaration_refs(else_expr, output);
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            declaration_refs(scrutinee, output);
            for case in cases {
                declaration_refs(&case.body, output);
            }
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            declaration_refs(scrutinee, output);
            for case in cases {
                declaration_refs(&case.body, output);
            }
        }
        RuntimeExpr::Record { fields } => {
            for (_, field) in fields {
                declaration_refs(field, output);
            }
        }
        RuntimeExpr::Project { record, .. }
        | RuntimeExpr::Closure { body: record, .. }
        | RuntimeExpr::LexicalClosure { body: record, .. } => declaration_refs(record, output),
        RuntimeExpr::Call { callee, args } => {
            declaration_refs(callee, output);
            for arg in args {
                declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability {
                declaration_refs(&capability.value, output);
            }
            for arg in args {
                declaration_refs(arg, output);
            }
        }
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. } => {
            declaration_refs(body, output)
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => {}
    }
}

const PROGRAM: &str = r#"program capabilities FS APartial
fn walk (fuel : Nat) (state : Bool) : HostIO APartial ExitCode =
  match fuel {
    Zero |-> match state {
      False |-> host_exit APartial (Failure 7);
      True |-> host_exit APartial Success
    };
    Suc smaller |-> walk smaller (match state {
      False |-> True;
      True |-> False
    })
  }

fn seed (input : ProcessInput) : Nat =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> Zero;
      Cons _argv0 rest |-> match rest {
        Nil |-> Zero;
        Cons _argument _tail |-> Suc (Suc (Suc Zero))
      }
    }
  }

fn main (input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode =
  walk (seed input) True
"#;

fn assert_agreement(arguments: &[&str], expected_exit: i32) {
    let dir = output_dir(if arguments.is_empty() {
        "zero"
    } else {
        "multi"
    });
    let output = ken_cli::build_native_program(
        PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px8l-recursive-declaration",
        &dir,
    )
    .expect("admitted recursive declaration compiles through the finite closure");
    assert_eq!(
        output.runtime_program.declarations.len(),
        2,
        "the executable closure is exactly main plus the admitted recursive declaration"
    );
    let recursive = output
        .runtime_program
        .declarations
        .iter()
        .find(|declaration| declaration.symbol.ends_with("::walk"))
        .expect("the finite closure retains walk as a declaration");
    let ken_runtime::RuntimeDeclarationKind::Transparent { body } = &recursive.kind else {
        panic!("walk remains an executable transparent declaration");
    };
    let mut references = std::collections::BTreeSet::new();
    declaration_refs(body, &mut references);
    assert!(
        references.contains(&recursive.symbol),
        "walk retains a finite self-reference instead of being eagerly unfolded"
    );
    let native = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: arguments.iter().map(std::ffi::OsString::from).collect(),
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked recursive artifact runs");

    let mut argv = vec![b"ken".to_vec()];
    argv.extend(
        arguments
            .iter()
            .map(|argument| argument.as_bytes().to_vec()),
    );
    let mut host = ken_interp::CaptureHost::new(Vec::new());
    let interpreted = ken_cli::run_program_effect_observation(
        PROGRAM,
        ken_cli::SourceFormat::Ken,
        &argv,
        &[],
        b"/",
        &mut host,
    )
    .expect("same recursive checked source runs through the interpreter");

    assert_eq!(native, interpreted);
    assert_eq!(native.exit_status, expected_exit);
    assert!(native.effect_trace.is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn dynamic_zero_seed_takes_the_base_case() {
    assert_agreement(&[], 0);
}

#[test]
fn dynamic_multistep_seed_preserves_updated_parameter_order() {
    assert_agreement(&["three"], 7);
}

#[test]
fn nondecreasing_cycle_is_rejected_before_native_lowering() {
    let source = r#"program capabilities FS APartial
fn spin (fuel : Nat) : HostIO APartial ExitCode = spin fuel
fn main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode = spin Zero
"#;
    let dir = output_dir("nondecreasing");
    let error = ken_cli::build_native_program(
        source,
        ken_cli::SourceFormat::Ken,
        "px8l-nondecreasing-cycle",
        &dir,
    )
    .expect_err("the kernel SCT gate must reject a non-decreasing recursive cycle");
    assert!(
        matches!(
            error,
            ken_elaborator::compiler_driver::NativeProgramBuildError::Driver(
                ken_elaborator::compiler_driver::CompilerDriverError::Elaboration(
                    ken_elaborator::ElabError::KernelRejected { .. }
                )
            )
        ),
        "non-decreasing recursion must retain the typed kernel boundary: {error:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}
