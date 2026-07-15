//! Tested native process-entrypoint shell.
//!
//! This module stages byte-accurate process data for the Cranelift entrypoint,
//! maps the returned `ExitCode` through the same fail-closed function used by
//! the interpreter runner, and owns runtime diagnostic init/teardown. These
//! properties are tested and validated; successful native execution is never
//! promoted to a Ken proof.

use std::io::{self, Write};

use crate::{
    run_process_expr_with_cranelift, NativeSeedEnvironment, RuntimeExpr, RuntimeGroundValue,
    RuntimeObservation, RuntimeTrap, RuntimeValue,
};

pub const PROCESS_INPUT_CONSTRUCTOR: &str = "ctor:prelude::ProcessInput::MkProcessInput";
pub const LIST_NIL_CONSTRUCTOR: &str = "ctor:prelude::List::Nil";
pub const LIST_CONS_CONSTRUCTOR: &str = "ctor:prelude::List::Cons";
pub const PROD_CONSTRUCTOR: &str = "ctor:prelude::Prod::MkProd";
pub const EXIT_SUCCESS_CONSTRUCTOR: &str = "ctor:prelude::ExitCode::Success";
pub const EXIT_FAILURE_CONSTRUCTOR: &str = "ctor:prelude::ExitCode::Failure";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeProcessInput {
    pub arguments: Vec<Vec<u8>>,
    pub environment: Vec<(Vec<u8>, Vec<u8>)>,
    pub working_directory: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessExitCode {
    Success,
    Failure(i64),
    MalformedFailure,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessExitMapping {
    pub status: i32,
    pub trap_report: Option<&'static str>,
}

/// The single interpreter/native `ExitCode -> i32` mapping.
///
/// Every malformed or absent result fails closed to status 1 and names the
/// malformed boundary in a trap report. `Failure 0` also fails closed to 1.
pub fn process_exit_status(exit_code: ProcessExitCode) -> ProcessExitMapping {
    match exit_code {
        ProcessExitCode::Success => ProcessExitMapping {
            status: 0,
            trap_report: None,
        },
        ProcessExitCode::Failure(0) => ProcessExitMapping {
            status: 1,
            trap_report: None,
        },
        ProcessExitCode::Failure(code @ 1..=255) => ProcessExitMapping {
            status: code as i32,
            trap_report: None,
        },
        ProcessExitCode::Failure(_) | ProcessExitCode::MalformedFailure => ProcessExitMapping {
            status: 1,
            trap_report: Some("malformed ExitCode::Failure payload"),
        },
        ProcessExitCode::Malformed => ProcessExitMapping {
            status: 1,
            trap_report: Some("entrypoint returned a malformed ExitCode"),
        },
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeProcessOutcome {
    pub exit_status: i32,
    pub observation: Option<RuntimeObservation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeProcessRuntimeSupportFact {
    Available,
}

#[derive(Debug)]
pub struct NativeProcessRuntime {
    pub staged_process_input: RuntimeValue,
    pub trap_reporting: NativeProcessRuntimeSupportFact,
}

/// Initialize the native process runtime before the entrypoint is lowered.
pub fn initialize_native_process_runtime(input: &NativeProcessInput) -> NativeProcessRuntime {
    NativeProcessRuntime {
        staged_process_input: native_process_input_value(input),
        trap_reporting: NativeProcessRuntimeSupportFact::Available,
    }
}

/// Tear down the native process runtime by flushing its diagnostic streams.
pub fn teardown_native_process_runtime(stderr: &mut impl Write) {
    flush_diagnostics(stderr);
}

/// Construct the post-erasure runtime value corresponding to
/// `MkProcessInput (List Bytes) (List (Prod Bytes Bytes)) Bytes`.
pub fn native_process_input_value(input: &NativeProcessInput) -> RuntimeValue {
    let arguments = list_value(input.arguments.iter().cloned().map(RuntimeValue::Bytes));
    let environment = list_value(input.environment.iter().cloned().map(|(key, value)| {
        RuntimeValue::Constructor {
            constructor: PROD_CONSTRUCTOR.to_string(),
            args: vec![RuntimeValue::Bytes(key), RuntimeValue::Bytes(value)],
        }
    }));
    RuntimeValue::Constructor {
        constructor: PROCESS_INPUT_CONSTRUCTOR.to_string(),
        args: vec![
            arguments,
            environment,
            RuntimeValue::Bytes(input.working_directory.clone()),
        ],
    }
}

fn list_value(values: impl IntoIterator<Item = RuntimeValue>) -> RuntimeValue {
    values
        .into_iter()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .fold(
            RuntimeValue::Constructor {
                constructor: LIST_NIL_CONSTRUCTOR.to_string(),
                args: vec![],
            },
            |tail, head| RuntimeValue::Constructor {
                constructor: LIST_CONS_CONSTRUCTOR.to_string(),
                args: vec![head, tail],
            },
        )
}

/// Initialize, execute, report, and tear down one native process entrypoint.
///
/// `entrypoint` consumes the staged `ProcessInput` as `RuntimeExpr::Var(0)`.
/// This is the runtime-init staging route: the unchanged entry expression is
/// lowered against the current invocation's value, and raw process bytes are
/// never coerced through UTF-8.
pub fn run_native_process_entrypoint(
    input: &NativeProcessInput,
    entrypoint: &RuntimeExpr,
) -> NativeProcessOutcome {
    let stderr = io::stderr();
    run_native_process_entrypoint_with_stderr(input, entrypoint, &mut stderr.lock())
}

pub fn run_native_process_entrypoint_with_stderr<W>(
    input: &NativeProcessInput,
    entrypoint: &RuntimeExpr,
    stderr: &mut W,
) -> NativeProcessOutcome
where
    W: Write,
{
    let runtime = initialize_native_process_runtime(input);

    let report = match run_process_expr_with_cranelift(
        entrypoint,
        &NativeSeedEnvironment::empty(),
        &runtime.staged_process_input,
    ) {
        Ok(report) => report,
        Err(error) => {
            report_trap(stderr, &format!("native entrypoint failed: {error}"));
            teardown_native_process_runtime(stderr);
            return NativeProcessOutcome {
                exit_status: 1,
                observation: None,
            };
        }
    };

    let mapping = match &report.observation {
        RuntimeObservation::Returned(value) => process_exit_status(runtime_exit_code(value)),
        RuntimeObservation::Trapped(trap) => {
            report_runtime_trap(stderr, trap);
            ProcessExitMapping {
                status: 1,
                trap_report: None,
            }
        }
    };
    if let Some(message) = mapping.trap_report {
        report_trap(stderr, message);
    }
    teardown_native_process_runtime(stderr);
    NativeProcessOutcome {
        exit_status: mapping.status,
        observation: Some(report.observation),
    }
}

fn runtime_exit_code(value: &RuntimeGroundValue) -> ProcessExitCode {
    match value {
        RuntimeGroundValue::Constructor { constructor, .. }
            if constructor == EXIT_SUCCESS_CONSTRUCTOR =>
        {
            ProcessExitCode::Success
        }
        RuntimeGroundValue::Constructor { constructor, args }
            if constructor == EXIT_FAILURE_CONSTRUCTOR =>
        {
            match args.as_slice() {
                [RuntimeGroundValue::Int(code)] => ProcessExitCode::Failure(*code),
                _ => ProcessExitCode::MalformedFailure,
            }
        }
        _ => ProcessExitCode::Malformed,
    }
}

fn report_runtime_trap(stderr: &mut impl Write, trap: &RuntimeTrap) {
    report_trap(stderr, &format!("{:?}: {}", trap.code, trap.message));
}

fn report_trap(stderr: &mut impl Write, message: &str) {
    let _ = writeln!(stderr, "ken native trap: {message}");
}

fn flush_diagnostics(stderr: &mut impl Write) {
    let _ = io::stdout().flush();
    let _ = stderr.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RuntimeTrapCode, RuntimeValue};

    fn failure(code: i64) -> RuntimeExpr {
        failure_expr(RuntimeExpr::Value(RuntimeValue::Int(code)))
    }

    fn failure_expr(code: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::Construct {
            constructor: EXIT_FAILURE_CONSTRUCTOR.to_string(),
            args: vec![code],
        }
    }

    fn success() -> RuntimeExpr {
        RuntimeExpr::Construct {
            constructor: EXIT_SUCCESS_CONSTRUCTOR.to_string(),
            args: vec![],
        }
    }

    fn process_entry(body: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![crate::RuntimeMatchCase {
                constructor: PROCESS_INPUT_CONSTRUCTOR.to_string(),
                binders: 3,
                body,
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "native process input is malformed".to_string(),
            },
        }
    }

    fn argc_entrypoint() -> RuntimeExpr {
        process_entry(RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![crate::RuntimeMatchCase {
                constructor: LIST_CONS_CONSTRUCTOR.to_string(),
                binders: 2,
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(1)),
                    cases: vec![
                        crate::RuntimeMatchCase {
                            constructor: LIST_NIL_CONSTRUCTOR.to_string(),
                            binders: 0,
                            body: failure(1),
                        },
                        crate::RuntimeMatchCase {
                            constructor: LIST_CONS_CONSTRUCTOR.to_string(),
                            binders: 2,
                            body: failure(3),
                        },
                    ],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "native argv tail is malformed".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "native argv is empty or malformed".to_string(),
            },
        })
    }

    fn environment_key_length_entrypoint() -> RuntimeExpr {
        process_entry(RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(1)),
            cases: vec![crate::RuntimeMatchCase {
                constructor: LIST_CONS_CONSTRUCTOR.to_string(),
                binders: 2,
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(0)),
                    cases: vec![crate::RuntimeMatchCase {
                        constructor: PROD_CONSTRUCTOR.to_string(),
                        binders: 2,
                        body: failure_expr(RuntimeExpr::PrimitiveCall {
                            primitive: crate::RuntimePrimitive {
                                symbol: "bytes_length".to_string(),
                                partiality: crate::RuntimePartiality::Total,
                            },
                            args: vec![RuntimeExpr::Var(0)],
                        }),
                    }],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "native environment pair is malformed".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "native environment is empty or malformed".to_string(),
            },
        })
    }

    fn list_len(value: &RuntimeValue) -> usize {
        match value {
            RuntimeValue::Constructor { constructor, args }
                if constructor == LIST_NIL_CONSTRUCTOR && args.is_empty() =>
            {
                0
            }
            RuntimeValue::Constructor { constructor, args }
                if constructor == LIST_CONS_CONSTRUCTOR && args.len() == 2 =>
            {
                1 + list_len(&args[1])
            }
            other => panic!("malformed staged list: {other:?}"),
        }
    }

    fn process_parts(value: &RuntimeValue) -> (&RuntimeValue, &RuntimeValue, &[u8]) {
        let RuntimeValue::Constructor { constructor, args } = value else {
            panic!("ProcessInput is not a constructor")
        };
        assert_eq!(constructor, PROCESS_INPUT_CONSTRUCTOR);
        let [arguments, environment, RuntimeValue::Bytes(cwd)] = args.as_slice() else {
            panic!("ProcessInput has the wrong field shape")
        };
        (arguments, environment, cwd)
    }

    #[test]
    fn two_distinct_argv_vectors_reach_the_native_entry() {
        let entrypoint = argc_entrypoint();
        for (arguments, expected) in [
            (vec![b"ken".to_vec()], 1),
            (vec![b"ken".to_vec(), b"one".to_vec(), vec![0xff, 0x00]], 3),
        ] {
            let input = NativeProcessInput {
                arguments,
                environment: vec![],
                working_directory: b"/tmp".to_vec(),
            };
            let mut stderr = Vec::new();
            let outcome =
                run_native_process_entrypoint_with_stderr(&input, &entrypoint, &mut stderr);
            assert_eq!(outcome.exit_status, expected);
            assert!(stderr.is_empty());
        }
    }

    #[test]
    fn argv_environment_and_cwd_are_byte_accurate_and_field_ordered() {
        let input = NativeProcessInput {
            arguments: vec![vec![0xff, b'a'], vec![0x00]],
            environment: vec![(vec![0xfe], vec![0xfd, 0x00])],
            working_directory: vec![b'/', 0xfc],
        };
        let staged = native_process_input_value(&input);
        let (arguments, environment, cwd) = process_parts(&staged);
        assert_eq!(list_len(arguments), 2);
        assert_eq!(list_len(environment), 1);
        assert_eq!(cwd, &[b'/', 0xfc]);

        let RuntimeValue::Constructor { args, .. } = arguments else {
            unreachable!()
        };
        assert_eq!(args[0], RuntimeValue::Bytes(vec![0xff, b'a']));
        let RuntimeValue::Constructor { args, .. } = environment else {
            unreachable!()
        };
        let RuntimeValue::Constructor {
            constructor,
            args: pair,
        } = &args[0]
        else {
            panic!("environment head is not Prod")
        };
        assert_eq!(constructor, PROD_CONSTRUCTOR);
        assert_eq!(pair[0], RuntimeValue::Bytes(vec![0xfe]));
        assert_eq!(pair[1], RuntimeValue::Bytes(vec![0xfd, 0x00]));
    }

    #[test]
    fn byte_environment_reaches_the_native_entry() {
        let input = NativeProcessInput {
            arguments: vec![b"ken".to_vec()],
            environment: vec![(vec![0xff, b'K'], vec![0x00, 0xfe])],
            working_directory: vec![],
        };
        let mut stderr = Vec::new();
        let entrypoint = environment_key_length_entrypoint();
        let outcome = run_native_process_entrypoint_with_stderr(&input, &entrypoint, &mut stderr);
        assert_eq!(outcome.exit_status, 2);
        assert!(stderr.is_empty());
    }

    #[test]
    fn runtime_init_makes_trap_reporting_available_before_entry() {
        let input = NativeProcessInput {
            arguments: vec![b"ken".to_vec()],
            environment: vec![],
            working_directory: b"/tmp".to_vec(),
        };
        let runtime = initialize_native_process_runtime(&input);
        assert_eq!(
            runtime.trap_reporting,
            NativeProcessRuntimeSupportFact::Available
        );
        assert_eq!(
            runtime.staged_process_input,
            native_process_input_value(&input)
        );
    }

    #[test]
    fn exit_status_mapping_is_shared_and_fails_closed() {
        assert_eq!(process_exit_status(ProcessExitCode::Success).status, 0);
        assert_eq!(process_exit_status(ProcessExitCode::Failure(0)).status, 1);
        assert_eq!(
            process_exit_status(ProcessExitCode::Failure(255)).status,
            255
        );

        let input = NativeProcessInput {
            arguments: vec![],
            environment: vec![],
            working_directory: vec![],
        };
        let mut stderr = Vec::new();
        let malformed_entrypoint = RuntimeExpr::Value(RuntimeValue::Int(0));
        let malformed =
            run_native_process_entrypoint_with_stderr(&input, &malformed_entrypoint, &mut stderr);
        assert_eq!(malformed.exit_status, 1);
        assert!(String::from_utf8(stderr)
            .unwrap()
            .contains("entrypoint returned a malformed ExitCode"));

        let mut malformed_failure_stderr = Vec::new();
        let malformed_failure_entrypoint = RuntimeExpr::Value(RuntimeValue::Constructor {
            constructor: EXIT_FAILURE_CONSTRUCTOR.to_string(),
            args: vec![],
        });
        let malformed_failure = run_native_process_entrypoint_with_stderr(
            &input,
            &malformed_failure_entrypoint,
            &mut malformed_failure_stderr,
        );
        assert_eq!(malformed_failure.exit_status, 1);
        assert!(String::from_utf8(malformed_failure_stderr)
            .unwrap()
            .contains("malformed ExitCode::Failure payload"));
    }

    #[test]
    fn trap_reports_to_runtime_stderr_and_clean_run_does_not() {
        let input = NativeProcessInput {
            arguments: vec![],
            environment: vec![],
            working_directory: vec![],
        };
        let mut clean_stderr = Vec::new();
        let clean =
            run_native_process_entrypoint_with_stderr(&input, &success(), &mut clean_stderr);
        assert_eq!(clean.exit_status, 0);
        assert!(clean_stderr.is_empty());

        let mut trap_stderr = Vec::new();
        let trap_entrypoint = RuntimeExpr::Trap(RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "fixture trap".to_string(),
        });
        let trapped =
            run_native_process_entrypoint_with_stderr(&input, &trap_entrypoint, &mut trap_stderr);
        assert_eq!(trapped.exit_status, 1);
        let report = String::from_utf8(trap_stderr).unwrap();
        assert!(report.contains("ken native trap"));
        assert!(report.contains("ExplicitTrap: fixture trap"));
    }

    #[test]
    fn host_effect_execution_remains_a_named_unavailable_lane() {
        let input = NativeProcessInput {
            arguments: vec![],
            environment: vec![],
            working_directory: vec![],
        };
        let mut stderr = Vec::new();
        let effect_entrypoint = RuntimeExpr::Effect {
            family: "Console".to_string(),
            operation: ken_host::HostOpV1::ConsoleRead,
            capability: None,
            args: vec![],
        };
        let outcome =
            run_native_process_entrypoint_with_stderr(&input, &effect_entrypoint, &mut stderr);
        assert_eq!(outcome.exit_status, 1);
        let report = String::from_utf8(stderr).unwrap();
        assert!(report.contains("unsupported runtime-IR lowering: Effect"));
        assert!(report.contains("Console.257"));
    }
}
