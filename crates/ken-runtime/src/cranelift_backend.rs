//! NC6 Cranelift backend spike for the NC5 runtime IR seed.
//!
//! This module deliberately keeps the native boundary narrow. Cranelift code
//! returns scalar `i64` values directly and aggregate observations through an
//! opaque token table decoded by this Rust layer. Native addresses, object
//! layout, allocation order, ABI details, and Cranelift internals never become
//! Ken-observable meaning.

use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use cranelift_codegen::isa::OwnedTargetIsa;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::{
    fnv1a_64, proof_erasure_boundary_facts_from_program, proof_erasure_witness_error,
    validate_supported_runtime_artifact_certificate, KenCheckedProofErasureBoundaryReport,
    ProofErasureBoundaryWitnessError, ProofErasureBoundaryWitnessStage, RuntimeArtifactCertificate,
    RuntimeArtifactIdentity, RuntimeArtifactValidationError, RuntimeArtifactValidationReport,
    RuntimeDeclaration, RuntimeDeclarationKind, RuntimeEffectBoundary, RuntimeExample, RuntimeExpr,
    RuntimeIrRunReport, RuntimeIrTargetIdentity, RuntimeLowerabilityStatus, RuntimeObservation,
    RuntimeProgram, RuntimeValue,
};

pub(crate) mod compiled;
mod lowering;
pub(crate) mod planning;
pub(crate) mod surface;

use compiled::*;
use lowering::core::*;

// RT-SPLIT slice 5 — temporary `#[cfg(test)]` facade wiring (Architect
// `evt_3tgaw9ws44fqg`). These tests are artifact-subject and move to
// `artifact/tests.rs` in slice 6; until then they stay here and reach the
// now-lowering-owned items by item-level test wiring, which preserves their
// call tokens (AC-3) and costs zero production seams. Slice 6 replaces each
// with an explicit import at the new home and does not touch `lowering`.
#[cfg(test)]
use lowering::require_i64_for_artifact_tests;
#[cfg(test)]
use lowering::verify_cranelift_function_for_artifact_tests as verify_cranelift_function;
#[cfg(test)]
use lowering::{PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE, PX8TR_TRAP_PROVENANCE};

// Facade re-exports preserving PRE-EXISTING reach for the four non-private
// declarations this slice moved into `lowering` (§10.2: "the facade explicitly
// re-exports their pre-existing visibility"). Per §10.4 an explicit re-export of
// an already-exported name is NOT a new widening — AC-7 count is unchanged.
//
// `with_px8ds_retired_flat_order` is bare `pub` and reached cross-CRATE as
// `ken_runtime::with_px8ds_retired_flat_order` through `lib.rs:39`. Moving it
// into the private `lowering` module severs that path, and neither
// `-p ken-runtime` build config can observe the break — only the consumer can.
#[cfg(feature = "px8-ds-test-support")]
pub use lowering::with_px8ds_retired_flat_order;
#[cfg(test)]
pub(crate) use lowering::{
    NativeIntLoweringMutation, Px8trTrapProvenanceEvent, NATIVE_INT_LOWERING_MUTATION,
};
use planning::*;
pub use surface::*;

pub fn run_nc6_seed_examples(
    program: &RuntimeProgram,
) -> Result<Vec<CraneliftRunReport>, CraneliftBackendError> {
    reject_program_blockers(program)?;
    let env = NativeSeedEnvironment::nc5_seed();
    program
        .examples
        .iter()
        .map(|example| run_example_with_seed_observation(example, &env))
        .collect()
}

pub fn run_nc8_validated_seed_examples(
    program: &RuntimeProgram,
    certificate: &RuntimeArtifactCertificate,
) -> Result<Vec<CraneliftRunReport>, ValidatedNativeRunError> {
    let validation = validate_supported_runtime_artifact_certificate(program, certificate)?;
    reject_program_blockers(program)?;
    let env = NativeSeedEnvironment::nc5_seed();
    program
        .examples
        .iter()
        .map(|example| {
            let mut report = run_example_native(
                Some(program),
                example,
                &env,
                NativeFidelity::F0NativeExample,
                NativeRunEvidence::from_program(program),
                Some(validation.clone()),
                None,
            )?;
            if report.observation == example.observation {
                report.trust.fidelity = NativeFidelity::F1SeedObservationAgreement;
            }
            Ok(report)
        })
        .collect()
}

pub fn run_example_with_interpreter_observation(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    oracle: InterpreterOracleObservation,
) -> NativeDifferentialReport {
    run_example_with_interpreter_observation_and_reports(program, example, env, oracle, None, None)
}

pub fn run_validated_example_with_interpreter_observation(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    oracle: InterpreterOracleObservation,
    certificate: &RuntimeArtifactCertificate,
) -> Result<NativeDifferentialReport, RuntimeArtifactValidationError> {
    let validation = validate_supported_runtime_artifact_certificate(program, certificate)?;
    Ok(run_example_with_interpreter_observation_and_validation(
        program,
        example,
        env,
        oracle,
        Some(validation),
    ))
}

pub fn run_ken_checked_proof_erasure_example_with_interpreter_observation(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    oracle: InterpreterOracleObservation,
    proof_erasure_boundary: KenCheckedProofErasureBoundaryReport,
) -> Result<NativeDifferentialReport, ProofErasureBoundaryWitnessError> {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if proof_erasure_boundary.artifact != artifact {
        return Err(proof_erasure_witness_error(
            ProofErasureBoundaryWitnessStage::WitnessIdentity,
            "artifact_identity",
            format!(
                "Ken-checked proof-erasure report identity {:?} does not match RuntimeProgram identity {:?}",
                proof_erasure_boundary.artifact, artifact
            ),
        ));
    }
    let recomputed_facts = proof_erasure_boundary_facts_from_program(program);
    if let Some(lane) =
        proof_erasure_boundary_report_mismatch_lane(&proof_erasure_boundary, &recomputed_facts)
    {
        return Err(proof_erasure_witness_error(
            ProofErasureBoundaryWitnessStage::WitnessMismatch,
            lane,
            "Ken-checked proof-erasure report facts do not match the RuntimeProgram lanes",
        ));
    }

    Ok(run_example_with_interpreter_observation_and_reports(
        program,
        example,
        env,
        oracle,
        None,
        Some(proof_erasure_boundary),
    ))
}

pub fn run_runtime_ir_report_with_cranelift(
    program: &RuntimeProgram,
    run_report: RuntimeIrRunReport,
    env: &NativeSeedEnvironment,
) -> NativeRuntimeIrComparisonReport {
    let artifact = NativeArtifactIdentity::from_program(program);
    let example = match runtime_ir_report_example(program, &run_report) {
        Ok(example) => example,
        Err(err) => {
            return runtime_ir_comparison_error_report(
                artifact,
                run_report,
                err,
                NativeDifferentialStage::BoundaryPreflight,
            );
        }
    };

    if let Err(err) = reject_program_blockers(program) {
        return runtime_ir_comparison_error_report(
            artifact,
            run_report,
            err,
            NativeDifferentialStage::BoundaryPreflight,
        );
    }

    match run_example_native(
        Some(program),
        example,
        env,
        NativeFidelity::F0NativeExample,
        NativeRunEvidence::from_program(program),
        None,
        None,
    ) {
        Ok(mut native) => {
            if native.observation == run_report.observation.observation {
                native.trust.fidelity = NativeFidelity::F1RuntimeIrEvaluatorAgreement;
                NativeRuntimeIrComparisonReport {
                    example: example.name.clone(),
                    artifact,
                    runtime_ir: run_report,
                    native: Some(native),
                    verdict: NativeRuntimeIrComparisonVerdict::RuntimeIrNativeAgreement {
                        stage: NativeDifferentialStage::RuntimeIrNativeCompare,
                    },
                }
            } else {
                NativeRuntimeIrComparisonReport {
                    example: example.name.clone(),
                    artifact,
                    verdict: NativeRuntimeIrComparisonVerdict::Mismatch {
                        stage: NativeDifferentialStage::RuntimeIrNativeCompare,
                        runtime_ir: run_report.observation.observation.clone(),
                        native: native.observation.clone(),
                    },
                    runtime_ir: run_report,
                    native: Some(native),
                }
            }
        }
        Err(err) => runtime_ir_comparison_error_report(
            artifact,
            run_report,
            err,
            NativeDifferentialStage::NativeLoweringOrExecution,
        ),
    }
}

pub fn emit_runtime_ir_object_with_cranelift(
    program: &RuntimeProgram,
    run_report: &RuntimeIrRunReport,
    env: &NativeSeedEnvironment,
    entry_symbol: impl Into<String>,
) -> Result<CraneliftObjectArtifact, CraneliftBackendError> {
    let entry_symbol = entry_symbol.into();
    let example = runtime_ir_report_example(program, run_report)?;
    reject_program_blockers(program)?;

    let compiled = compile_program_expr_object(program, &example.ir, env, &entry_symbol)?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let object_bytes = compiled
        .module
        .finish()
        .emit()
        .map_err(|err| backend_module(err.to_string()))?;
    let object_hash = fnv1a_64(&object_bytes);

    Ok(CraneliftObjectArtifact {
        example: example.name.clone(),
        entry_symbol,
        object_bytes,
        object_hash,
        platform_target: native_platform_target_name(),
        backend_name: "Cranelift object".to_string(),
        verifier_passed,
        assumptions,
        unsupported,
    })
}

#[cfg(test)]
fn test_only_distinguished_root_join_plan() -> crate::NativeJoinPlanV1 {
    let site_id = 0;
    let declaration = "decl:fixture::CheckedRoot::main".to_string();
    let checked_occurrence_path = vec![0];
    let checked_result_type_fingerprint = 0x5058_3854_4152_4f4f;
    crate::NativeJoinPlanV1 {
        representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
        sites: vec![crate::NativeJoinPlanSiteV1 {
            site_id,
            occurrence_binding_fingerprint:
                crate::compiler_private_join_occurrence_binding_fingerprint(
                    site_id,
                    &declaration,
                    &checked_occurrence_path,
                    checked_result_type_fingerprint,
                ),
            declaration,
            checked_occurrence_path,
            checked_result_type_fingerprint,
            runtime_frame_fingerprint: crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1,
            answer_kind: crate::NativeJoinAnswerKindV1::ExitCode,
        }],
    }
}

#[cfg(test)]
#[repr(C)]
struct BorrowedFixtureValue {
    kind: u64,
    tag: u64,
    data: *const std::ffi::c_void,
    len: usize,
}
#[cfg(test)]
#[repr(C)]
struct NativeInvocationFixture {
    process_input: *const BorrowedFixtureValue,
    host_context: *mut std::ffi::c_void,
    capability: u64,
    native_int_arena: *mut crate::NativeIntArenaV1,
}
#[cfg(test)]
fn self_consistent_root_join_site(site_id: u64) -> crate::NativeJoinPlanSiteV1 {
    let declaration = "decl:fixture::PX8H::main".to_string();
    let checked_occurrence_path = vec![0];
    let checked_result_type_fingerprint = 19;
    crate::NativeJoinPlanSiteV1 {
        site_id,
        occurrence_binding_fingerprint: crate::compiler_private_join_occurrence_binding_fingerprint(
            site_id,
            &declaration,
            &checked_occurrence_path,
            checked_result_type_fingerprint,
        ),
        declaration,
        checked_occurrence_path,
        checked_result_type_fingerprint,
        runtime_frame_fingerprint: crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1,
        answer_kind: crate::NativeJoinAnswerKindV1::ExitCode,
    }
}
#[cfg(test)]
fn oriented_test_interface(name: u8) -> crate::CheckedAnswerInterfaceV1 {
    let mut bytes = crate::CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec();
    bytes.push(name);
    crate::CheckedAnswerInterfaceV1::new(bytes).unwrap()
}
#[cfg(test)]
fn oriented_test_frame(
    frame_id: u64,
    semantic_position: u64,
    input: u8,
    output: u8,
    parent: Option<u64>,
) -> crate::OrientedSubcontinuationFramePlanV1 {
    let mut frame = crate::OrientedSubcontinuationFramePlanV1 {
        frame_id,
        segment_site_id: 9,
        declaration: "decl:fixture::oriented".to_string(),
        checked_occurrence_path: vec![frame_id],
        semantic_position,
        input_interface: oriented_test_interface(input),
        output_interface: oriented_test_interface(output),
        runtime_frame_fingerprint: frame_id + 100,
        occurrence_binding_fingerprint: 0,
        control_witness: parent.map_or(
            crate::OrientedControlWitnessV1::DistinguishedRoot,
            crate::OrientedControlWitnessV1::ParentFrame,
        ),
    };
    frame.occurrence_binding_fingerprint =
        crate::compiler_private_oriented_occurrence_binding_fingerprint(&frame);
    frame
}

#[cfg(test)]
pub(crate) fn emit_process_entrypoint_object_with_cranelift(
    entrypoint: &RuntimeExpr,
    entry_symbol: impl Into<String>,
) -> Result<CraneliftObjectArtifact, CraneliftBackendError> {
    let entry_symbol = entry_symbol.into();
    let compiled = compile_expr_into_module(
        new_object_module("ken-runtime-process-entrypoint")?,
        &entry_symbol,
        Linkage::Export,
        entrypoint,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        None,
        Some(test_only_distinguished_root_join_plan()),
        None,
    )?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let object_bytes = compiled
        .module
        .finish()
        .emit()
        .map_err(|err| backend_module(err.to_string()))?;
    let object_hash = fnv1a_64(&object_bytes);
    Ok(CraneliftObjectArtifact {
        example: "native-process-entrypoint".to_string(),
        entry_symbol,
        object_bytes,
        object_hash,
        platform_target: native_platform_target_name(),
        backend_name: "Cranelift process object".to_string(),
        verifier_passed,
        assumptions,
        unsupported,
    })
}

#[cfg(test)]
pub(crate) struct Px8trNestedRouteObject {
    pub artifact: CraneliftObjectArtifact,
    pub provenance: Vec<Px8trTrapProvenanceEvent>,
}

#[cfg(test)]
fn px8tr_test_interface(name: u8) -> crate::CheckedAnswerInterfaceV1 {
    let mut bytes = crate::CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec();
    bytes.push(name);
    crate::CheckedAnswerInterfaceV1::new(bytes).expect("PX8-TR test interface is canonical")
}

#[cfg(test)]
fn px8tr_nested_post_effect_fixture() -> (
    RuntimeExpr,
    RuntimeDeclaration,
    crate::OrientedSubcontinuationPlanV1,
) {
    let declaration = "decl:fixture::PX8TR::main".to_string();
    let ret_constructor = "ctor:fixture::PX8TR::ITree::Ret".to_string();
    let vis_constructor = "ctor:fixture::PX8TR::ITree::Vis".to_string();
    let result_err = "ctor:prelude::Result::Err".to_string();
    let result_ok = "ctor:prelude::Result::Ok".to_string();
    let unit = || RuntimeExpr::Construct {
        constructor: "ctor:prelude::Unit::MkUnit".to_string(),
        args: Vec::new(),
    };
    let default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "PX8-TR checked ITree recursor default".to_string(),
    };
    let ret_body = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: result_err.clone(),
                binders: 1,
                body: RuntimeExpr::Construct {
                    constructor: crate::EXIT_FAILURE_CONSTRUCTOR.to_string(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int(1.into()))],
                },
            },
            crate::RuntimeMatchCase {
                constructor: result_ok.clone(),
                binders: 1,
                body: RuntimeExpr::Construct {
                    constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                    args: Vec::new(),
                },
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-TR Result default".to_string(),
        },
    };
    let terminal_body = RuntimeExpr::Let {
        value: Box::new(RuntimeExpr::Effect {
            family: "Console".to_string(),
            operation: ken_host::HostOpV1::ConsoleFlush,
            capability: None,
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:prelude::Stream::Stdout".to_string(),
                args: Vec::new(),
            }],
        }),
        body: Box::new(RuntimeExpr::Construct {
            constructor: result_ok,
            args: vec![unit()],
        }),
    };
    let recursive_body = RuntimeExpr::Construct {
        constructor: vis_constructor.clone(),
        args: vec![
            unit(),
            RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["response".to_string()],
                body: Box::new(terminal_body),
            },
        ],
    };
    let cases = vec![
        crate::RuntimeComputationalMatchCase {
            constructor: ret_constructor,
            argument_binders: 1,
            recursive_positions: Vec::new(),
            body: ret_body,
        },
        crate::RuntimeComputationalMatchCase {
            constructor: vis_constructor.clone(),
            argument_binders: 2,
            recursive_positions: vec![1],
            body: RuntimeExpr::CheckedComputationalIHSlots {
                slot_template_ids: vec![200],
                checked_occurrence_paths: vec![vec![20]],
                body: Box::new(RuntimeExpr::CheckedComputationalIHInvocation {
                    call_template_id: 100,
                    checked_occurrence_path: vec![30],
                    body: Box::new(RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::Var(0)),
                        args: vec![unit()],
                    }),
                }),
            },
        },
    ];
    let frame_fingerprint =
        crate::compiler_private_computational_match_frame_fingerprint(&cases, &default);
    let body = RuntimeExpr::Closure {
        captures: Vec::new(),
        params: vec!["process_input".to_string(), "program_caps".to_string()],
        body: Box::new(RuntimeExpr::CheckedSubcontinuationFrame {
            frame_id: 7,
            body: Box::new(RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(RuntimeExpr::Construct {
                    constructor: vis_constructor.clone(),
                    args: vec![
                        unit(),
                        RuntimeExpr::LexicalClosure {
                            captures: Vec::new(),
                            params: vec!["response".to_string()],
                            body: Box::new(recursive_body),
                        },
                    ],
                }),
                cases,
                default,
            }),
        }),
    };
    let runtime_declaration = RuntimeDeclaration {
        symbol: declaration.clone(),
        kind: RuntimeDeclarationKind::Transparent { body },
        metadata: crate::RuntimeSymbolMetadata::empty(),
    };
    let mut frame = crate::OrientedSubcontinuationFramePlanV1 {
        frame_id: 7,
        segment_site_id: 9,
        declaration: declaration.clone(),
        checked_occurrence_path: vec![10],
        semantic_position: 0,
        input_interface: px8tr_test_interface(0),
        output_interface: px8tr_test_interface(1),
        runtime_frame_fingerprint: frame_fingerprint,
        occurrence_binding_fingerprint: 0,
        control_witness: crate::OrientedControlWitnessV1::DistinguishedRoot,
    };
    frame.occurrence_binding_fingerprint =
        crate::compiler_private_oriented_occurrence_binding_fingerprint(&frame);
    let mut slot = crate::CheckedComputationalIHSlotTemplateV1 {
        slot_template_id: 200,
        declaration: declaration.clone(),
        checked_match_ordinal: 0,
        checked_occurrence_path: vec![20],
        frame_template_id: 7,
        constructor: vis_constructor,
        recursive_position: 1,
        method_binder_ordinal: 0,
        local_telescope: Vec::new(),
        ih_interface: px8tr_test_interface(0),
        segment_site_id: 9,
        frame_templates: vec![7],
        input_interface: px8tr_test_interface(0),
        output_interface: px8tr_test_interface(1),
        runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
            declaration: declaration.clone(),
            runtime_path: vec![2, 0, 2],
        }],
        occurrence_binding_fingerprint: 0,
    };
    slot.occurrence_binding_fingerprint =
        crate::compiler_private_computational_ih_slot_binding_fingerprint(&slot);
    let mut call = crate::CheckedComputationalIHCallTemplateV1 {
        call_template_id: 100,
        declaration: declaration.clone(),
        checked_occurrence_path: vec![30],
        slot_template_id: 200,
        arity: 1,
        local_telescope: Vec::new(),
        result_interface: px8tr_test_interface(1),
        callee_segment_site_id: 9,
        callee_frame_templates: vec![7],
        parent_frame_template_id: Some(7),
        parent_segment_site_id: Some(9),
        caller_interface: px8tr_test_interface(1),
        runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
            declaration,
            runtime_path: vec![2, 0, 2, 0],
        }],
        occurrence_binding_fingerprint: 0,
    };
    call.occurrence_binding_fingerprint =
        crate::compiler_private_computational_ih_call_binding_fingerprint(&call);
    let entrypoint = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::DeclarationRef {
            symbol: runtime_declaration.symbol.clone(),
        }),
        args: vec![RuntimeExpr::Var(0), RuntimeExpr::Var(1)],
    };
    (
        entrypoint,
        runtime_declaration,
        crate::OrientedSubcontinuationPlanV1 {
            representation_rule_version:
                crate::OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![frame],
            recursive_calls: Vec::new(),
            computational_ih_slots: vec![slot],
            computational_ih_calls: vec![call],
        },
    )
}

#[cfg(test)]
pub(crate) fn emit_px8tr_nested_post_effect_object(
    entry_symbol: impl Into<String>,
    disable_repair: bool,
) -> Result<Px8trNestedRouteObject, CraneliftBackendError> {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE.set(false);
            PX8TR_TRAP_PROVENANCE.with(|trace| trace.borrow_mut().clear());
        }
    }

    let entry_symbol = entry_symbol.into();
    let (entrypoint, declaration, plan) = px8tr_nested_post_effect_fixture();
    let declarations = BTreeMap::from([(declaration.symbol.as_str(), &declaration)]);
    PX8TR_TRAP_PROVENANCE.with(|trace| trace.borrow_mut().clear());
    PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE.set(disable_repair);
    let _reset = Reset;
    let compiled = compile_expr_into_module(
        new_object_module("ken-runtime-px8tr-post-effect")?,
        &entry_symbol,
        Linkage::Export,
        &entrypoint,
        &NativeSeedEnvironment::empty(),
        declarations,
        None,
        true,
        Some(&crate::NativeProcessSymbols::legacy_prelude()),
        Some(test_only_distinguished_root_join_plan()),
        Some(plan),
    )?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let object_bytes = compiled
        .module
        .finish()
        .emit()
        .map_err(|err| backend_module(err.to_string()))?;
    let object_hash = fnv1a_64(&object_bytes);
    let provenance = PX8TR_TRAP_PROVENANCE.with(|trace| trace.borrow().clone());
    Ok(Px8trNestedRouteObject {
        artifact: CraneliftObjectArtifact {
            example: "px8tr-nested-post-effect".to_string(),
            entry_symbol,
            object_bytes,
            object_hash,
            platform_target: native_platform_target_name(),
            backend_name: "Cranelift PX8-TR process object".to_string(),
            verifier_passed,
            assumptions,
            unsupported,
        },
        provenance,
    })
}

#[cfg(test)]
fn emit_process_entrypoint_object_with_symbols(
    entrypoint: &RuntimeExpr,
    symbols: &crate::NativeProcessSymbols,
    entry_symbol: &str,
) -> Result<CraneliftObjectArtifact, CraneliftBackendError> {
    let compiled = compile_expr_into_module(
        new_object_module("ken-runtime-process-entrypoint")?,
        entry_symbol,
        Linkage::Export,
        entrypoint,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        Some(symbols),
        Some(test_only_distinguished_root_join_plan()),
        None,
    )?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let object_bytes = compiled
        .module
        .finish()
        .emit()
        .map_err(|err| backend_module(err.to_string()))?;
    let object_hash = fnv1a_64(&object_bytes);
    Ok(CraneliftObjectArtifact {
        example: "native-process-entrypoint".to_string(),
        entry_symbol: entry_symbol.to_string(),
        object_bytes,
        object_hash,
        platform_target: native_platform_target_name(),
        backend_name: "Cranelift process object".to_string(),
        verifier_passed,
        assumptions,
        unsupported,
    })
}

pub(crate) fn emit_bound_process_program_object_with_cranelift(
    program: &RuntimeProgram,
    entrypoint: &RuntimeExpr,
    symbols: &crate::NativeProcessSymbols,
    entry_symbol: impl Into<String>,
) -> Result<CraneliftObjectArtifact, CraneliftBackendError> {
    let entry_symbol = entry_symbol.into();
    reject_program_blockers(program)?;
    let compiled = compile_expr_into_module(
        new_object_module("ken-runtime-bound-process-entrypoint")?,
        &entry_symbol,
        Linkage::Export,
        entrypoint,
        &NativeSeedEnvironment::empty(),
        program
            .declarations
            .iter()
            .map(|declaration| (declaration.symbol.as_str(), declaration))
            .collect(),
        None,
        true,
        Some(symbols),
        native_join_plan_for_program(program)?,
        oriented_subcontinuation_plan_for_program(program)?,
    )?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let object_bytes = compiled
        .module
        .finish()
        .emit()
        .map_err(|err| backend_module(err.to_string()))?;
    let object_hash = fnv1a_64(&object_bytes);
    Ok(CraneliftObjectArtifact {
        example: "checked-native-program".to_string(),
        entry_symbol,
        object_bytes,
        object_hash,
        platform_target: native_platform_target_name(),
        backend_name: "Cranelift checked process object".to_string(),
        verifier_passed,
        assumptions,
        unsupported,
    })
}

fn proof_erasure_boundary_report_mismatch_lane(
    report: &KenCheckedProofErasureBoundaryReport,
    recomputed: &crate::ProofErasureBoundaryFacts,
) -> Option<&'static str> {
    if report.facts.runtime_declaration_targets != recomputed.runtime_declaration_targets {
        return Some("runtime_declaration_targets");
    }
    if report.facts.record_field_statuses != recomputed.record_field_statuses {
        return Some("record_field_statuses");
    }
    if report.facts.checked_core_record_field_statuses
        != recomputed.checked_core_record_field_statuses
    {
        return Some("checked_core_record_field_statuses");
    }
    if report.facts.lowerability != recomputed.lowerability {
        return Some("lowerability");
    }
    if report.facts.unsupported != recomputed.unsupported {
        return Some("unsupported");
    }
    if report.facts.obligations != recomputed.obligations {
        return Some("obligations");
    }
    if report.facts.obligation_metadata != recomputed.obligation_metadata {
        return Some("obligation_metadata");
    }
    if report.facts.assumptions != recomputed.assumptions {
        return Some("assumptions");
    }
    if report.facts.assumption_trust_metadata != recomputed.assumption_trust_metadata {
        return Some("assumption_trust_metadata");
    }
    if report.facts.trusted_base_delta != recomputed.trusted_base_delta {
        return Some("trusted_base_delta");
    }
    None
}

fn run_example_with_interpreter_observation_and_validation(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    oracle: InterpreterOracleObservation,
    artifact_validation: Option<RuntimeArtifactValidationReport>,
) -> NativeDifferentialReport {
    run_example_with_interpreter_observation_and_reports(
        program,
        example,
        env,
        oracle,
        artifact_validation,
        None,
    )
}

fn run_example_with_interpreter_observation_and_reports(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    oracle: InterpreterOracleObservation,
    artifact_validation: Option<RuntimeArtifactValidationReport>,
    ken_checked_proof_erasure_boundary: Option<KenCheckedProofErasureBoundaryReport>,
) -> NativeDifferentialReport {
    let artifact = NativeArtifactIdentity::from_program(program);

    if oracle.artifact != artifact {
        return oracle_identity_mismatch_report(example, artifact, oracle);
    }

    if let Err(err) = reject_program_blockers(program) {
        return differential_error_report(example, artifact, oracle, err, true);
    }

    match run_example_native(
        Some(program),
        example,
        env,
        NativeFidelity::F0NativeExample,
        NativeRunEvidence::from_program(program),
        artifact_validation,
        ken_checked_proof_erasure_boundary,
    ) {
        Ok(mut native) => {
            if native.observation == oracle.observation {
                native.trust.fidelity = NativeFidelity::F1InterpreterDifferentialAgreement;
                NativeDifferentialReport {
                    example: example.name.clone(),
                    artifact,
                    oracle,
                    native: Some(native),
                    verdict: NativeDifferentialVerdict::F1InterpreterAgreement {
                        stage: NativeDifferentialStage::InterpreterNativeCompare,
                    },
                }
            } else {
                NativeDifferentialReport {
                    example: example.name.clone(),
                    artifact,
                    verdict: NativeDifferentialVerdict::Mismatch {
                        stage: NativeDifferentialStage::InterpreterNativeCompare,
                        interpreter: oracle.observation.clone(),
                        native: native.observation.clone(),
                    },
                    oracle,
                    native: Some(native),
                }
            }
        }
        Err(err) => differential_error_report(example, artifact, oracle, err, false),
    }
}

pub fn reject_program_blockers(program: &RuntimeProgram) -> Result<(), CraneliftBackendError> {
    if !program.erased_core.metadata.effects.is_empty() {
        return Err(unsupported(
            "RuntimeProgram",
            "package carries effect metadata outside the NC6 D1 supported subset",
        ));
    }
    if !program.erased_core.metadata.capabilities.is_empty() {
        return Err(unsupported(
            "RuntimeProgram",
            "package carries capability metadata outside the NC6 D1 supported subset",
        ));
    }
    if !program.erased_core.metadata.runtime_checks.is_empty() {
        return Err(unsupported(
            "RuntimeProgram",
            "package carries runtime-check metadata outside the supported native subset",
        ));
    }
    if !program.erased_core.metadata.assumptions.is_empty()
        || !program
            .erased_core
            .metadata
            .assumption_trust_metadata
            .is_empty()
        || !program.erased_core.metadata.trusted_base_delta.is_empty()
    {
        return Err(unsupported(
            "RuntimeProgram",
            "package carries trust metadata outside the supported native subset",
        ));
    }

    for declaration in &program.declarations {
        if declaration.metadata.unsupported.is_some()
            || program
                .erased_core
                .metadata
                .unsupported
                .contains_key(&declaration.symbol)
        {
            return Err(unsupported(
                "RuntimeProgram",
                format!("reachable unsupported entry {}", declaration.symbol),
            ));
        }

        let lowerability = declaration
            .metadata
            .lowerability
            .as_ref()
            .or_else(|| {
                program
                    .erased_core
                    .metadata
                    .lowerability
                    .get(&declaration.symbol)
            })
            .ok_or_else(|| {
                unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} is missing runtime lowerability metadata",
                        declaration.symbol
                    ),
                )
            })?;
        if !matches!(lowerability, RuntimeLowerabilityStatus::Supported) {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} has blocking lowerability metadata: {:?}",
                    declaration.symbol, lowerability
                ),
            ));
        }

        if !declaration.metadata.effects.is_empty() {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries effect metadata outside the NC6 D1 supported subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.capabilities.is_empty() {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries capability metadata outside the NC6 D1 supported subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.runtime_checks.is_empty() {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries runtime-check metadata outside the NC6 D1 supported subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.assumptions.is_empty()
            || !declaration.metadata.assumption_trust_metadata.is_empty()
            || !declaration.metadata.trusted_base_delta.is_empty()
        {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries trust metadata outside the NC6 D1 supported subset",
                    declaration.symbol
                ),
            ));
        }

        if let RuntimeDeclarationKind::EffectBoundary { effects } = &declaration.kind {
            if !effects.is_empty() {
                return Err(unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} declares effects outside the NC6 D1 supported subset",
                        declaration.symbol
                    ),
                ));
            }
        }

        if let Some(effect_meta) = program
            .erased_core
            .metadata
            .checked_core
            .effects_foreign_metadata
            .get(&declaration.symbol)
        {
            if effect_meta.boundary == RuntimeEffectBoundary::Foreign
                || effect_meta.boundary == RuntimeEffectBoundary::Effectful
                || effect_meta.foreign_symbol.is_some()
                || !effect_meta.declared_effects.is_empty()
                || !effect_meta.capabilities.is_empty()
                || !effect_meta.runtime_checks.is_empty()
            {
                return Err(unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} carries effects/foreign metadata outside the NC6 D1 subset",
                        declaration.symbol
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub fn run_example_with_seed_observation(
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
) -> Result<CraneliftRunReport, CraneliftBackendError> {
    let mut report = run_example_native(
        None,
        example,
        env,
        NativeFidelity::F0NativeExample,
        NativeRunEvidence::seed_example(),
        None,
        None,
    )?;
    if report.observation == example.observation {
        report.trust.fidelity = NativeFidelity::F1SeedObservationAgreement;
    }
    Ok(report)
}

/// Execute one runtime expression through the tested native process boundary.
///
/// `staged_process_input` is the byte-accurate argv/environment value bound to
/// `RuntimeExpr::Var(0)` for the in-process validation path. Produced process
/// objects instead bind `Var(0)` to their call-scoped borrowed ingress root.
pub(crate) fn run_process_expr_with_cranelift(
    expr: &RuntimeExpr,
    env: &NativeSeedEnvironment,
    staged_process_input: &RuntimeValue,
) -> Result<CraneliftRunReport, CraneliftBackendError> {
    let compiled = compile_expr_with_declarations_and_process_input(
        expr,
        env,
        BTreeMap::new(),
        Some(staged_process_input),
    )?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let (observation, native_returned) = compiled.run(None)?;
    Ok(CraneliftRunReport {
        example: "native-process-entrypoint".to_string(),
        observation,
        verifier_passed,
        native_returned,
        trust: NativeTrustReport {
            backend: "Cranelift JIT",
            fidelity: NativeFidelity::F0NativeExample,
            verifier_passed,
            artifact_validation: None,
            ken_checked_proof_erasure_boundary: None,
            toolchain: native_toolchain_report(),
            evidence: NativeRunEvidence::seed_example(),
            assumptions,
            unsupported,
        },
    })
}

fn run_example_native(
    program: Option<&RuntimeProgram>,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    fidelity: NativeFidelity,
    evidence: NativeRunEvidence,
    artifact_validation: Option<RuntimeArtifactValidationReport>,
    ken_checked_proof_erasure_boundary: Option<KenCheckedProofErasureBoundaryReport>,
) -> Result<CraneliftRunReport, CraneliftBackendError> {
    let compiled = match program {
        Some(program) => compile_program_expr(program, &example.ir, env)?,
        None => compile_expr(&example.ir, env)?,
    };
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let (observation, native_returned) = compiled.run(None)?;
    Ok(CraneliftRunReport {
        example: example.name.clone(),
        observation,
        verifier_passed,
        native_returned,
        trust: NativeTrustReport {
            backend: "Cranelift JIT",
            fidelity,
            verifier_passed,
            artifact_validation,
            ken_checked_proof_erasure_boundary,
            toolchain: native_toolchain_report(),
            evidence,
            assumptions,
            unsupported,
        },
    })
}

fn runtime_ir_report_example<'a>(
    program: &'a RuntimeProgram,
    run_report: &RuntimeIrRunReport,
) -> Result<&'a RuntimeExample, CraneliftBackendError> {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if run_report.artifact != artifact || run_report.observation.artifact != artifact {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport artifact identity does not match the exact RuntimeProgram",
        ));
    }
    if run_report.observation.target != run_report.target {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport observation target does not match the run target",
        ));
    }
    if run_report.evidence.package_identity != program.package_identity
        || run_report.evidence.core_semantic_hash != program.core_semantic_hash
        || run_report.evidence.runtime_artifact_hash != program.artifact_hash
    {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport evidence identity does not match the exact RuntimeProgram",
        ));
    }
    if run_report.evidence.target_example != run_report.target.example
        || run_report.evidence.checked_core_shape != run_report.target.checked_core_shape
    {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport evidence target does not match the run target",
        ));
    }

    let mut matches = program
        .examples
        .iter()
        .filter(|example| RuntimeIrTargetIdentity::from_example(example) == run_report.target);
    let Some(example) = matches.next() else {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport target is not present in RuntimeProgram.examples",
        ));
    };
    if matches.next().is_some() {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport target identity is ambiguous in RuntimeProgram.examples",
        ));
    }
    Ok(example)
}

impl NativeArtifactIdentity {
    fn from_program(program: &RuntimeProgram) -> Self {
        Self {
            package_identity: program.package_identity.clone(),
            core_semantic_hash: program.core_semantic_hash,
            runtime_artifact_hash: program.artifact_hash,
        }
    }
}

impl NativeRunEvidence {
    fn seed_example() -> Self {
        let mut evidence = Self::default();
        evidence.unavailable.insert(
            "package/core/runtime artifact identity unavailable for standalone seed example"
                .to_string(),
        );
        evidence.evidence_sources.insert(
            "backend".to_string(),
            "compiled Cranelift JIT run".to_string(),
        );
        evidence
    }

    fn from_program(program: &RuntimeProgram) -> Self {
        let mut evidence = Self {
            package_identity: Some(program.package_identity.clone()),
            core_semantic_hash: Some(program.core_semantic_hash),
            runtime_artifact_hash: Some(program.artifact_hash),
            evidence_sources: BTreeMap::new(),
            unavailable: BTreeSet::new(),
        };
        evidence.evidence_sources.insert(
            "package_identity".to_string(),
            "RuntimeProgram.package_identity from the exact runtime artifact".to_string(),
        );
        evidence.evidence_sources.insert(
            "core_semantic_hash".to_string(),
            "RuntimeProgram.core_semantic_hash from the exact runtime artifact".to_string(),
        );
        evidence.evidence_sources.insert(
            "runtime_artifact_hash".to_string(),
            "RuntimeProgram.artifact_hash from the exact runtime artifact".to_string(),
        );
        evidence.evidence_sources.insert(
            "backend".to_string(),
            "compiled Cranelift JIT run".to_string(),
        );
        evidence
    }
}

fn native_toolchain_report() -> NativeToolchainReport {
    NativeToolchainReport {
        cranelift: NativeEvidenceFact::Unavailable {
            reason: "Cranelift package/version fact is not captured from the exact run yet"
                .to_string(),
        },
        linker: NativeEvidenceFact::Unavailable {
            reason: "linker/finalizer fact is not captured from the exact run yet".to_string(),
        },
        runtime: NativeEvidenceFact::Available {
            value: format!("ken-runtime {}", env!("CARGO_PKG_VERSION")),
            evidence_source: "compiled ken-runtime crate version embedded by Cargo for this binary"
                .to_string(),
        },
    }
}

fn oracle_identity_mismatch_report(
    example: &RuntimeExample,
    artifact: NativeArtifactIdentity,
    oracle: InterpreterOracleObservation,
) -> NativeDifferentialReport {
    let reason = format!(
        "oracle artifact identity {:?} does not match runtime artifact identity {:?}",
        oracle.artifact, artifact
    );
    NativeDifferentialReport {
        example: example.name.clone(),
        artifact,
        oracle,
        native: None,
        verdict: NativeDifferentialVerdict::Unsupported {
            stage: NativeDifferentialStage::BoundaryPreflight,
            construct: "InterpreterOracleObservation",
            reason,
        },
    }
}

fn differential_error_report(
    example: &RuntimeExample,
    artifact: NativeArtifactIdentity,
    oracle: InterpreterOracleObservation,
    err: CraneliftBackendError,
    preflight: bool,
) -> NativeDifferentialReport {
    let verdict = match err {
        CraneliftBackendError::Unsupported(err) => NativeDifferentialVerdict::Unsupported {
            stage: if preflight {
                NativeDifferentialStage::BoundaryPreflight
            } else {
                NativeDifferentialStage::NativeLoweringOrExecution
            },
            construct: err.construct,
            reason: err.reason,
        },
        CraneliftBackendError::Backend(err) => NativeDifferentialVerdict::BackendFailure {
            stage: NativeDifferentialStage::NativeLoweringOrExecution,
            reason: err.to_string(),
        },
    };
    NativeDifferentialReport {
        example: example.name.clone(),
        artifact,
        oracle,
        native: None,
        verdict,
    }
}

fn runtime_ir_comparison_error_report(
    artifact: NativeArtifactIdentity,
    run_report: RuntimeIrRunReport,
    err: CraneliftBackendError,
    stage: NativeDifferentialStage,
) -> NativeRuntimeIrComparisonReport {
    let example = run_report.target.example.clone();
    let verdict = match err {
        CraneliftBackendError::Unsupported(err) => NativeRuntimeIrComparisonVerdict::Unsupported {
            stage,
            construct: err.construct,
            reason: err.reason,
        },
        CraneliftBackendError::Backend(err) => NativeRuntimeIrComparisonVerdict::BackendFailure {
            stage: NativeDifferentialStage::NativeLoweringOrExecution,
            reason: err.to_string(),
        },
    };
    NativeRuntimeIrComparisonReport {
        example,
        artifact,
        runtime_ir: run_report,
        native: None,
        verdict,
    }
}

fn compile_expr(
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_with_declarations(expr, seed_env, BTreeMap::new())
}

fn compile_program_expr(
    program: &RuntimeProgram,
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_with_declarations(
        expr,
        seed_env,
        program
            .declarations
            .iter()
            .map(|declaration| (declaration.symbol.as_str(), declaration))
            .collect(),
    )
}

fn compile_expr_with_declarations<'a>(
    expr: &RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_with_declarations_and_process_input(expr, seed_env, declarations, None)
}

fn compile_expr_with_declarations_and_process_input<'a>(
    expr: &RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_into_module(
        new_jit_module()?,
        "ken_nc6_seed",
        Linkage::Local,
        expr,
        seed_env,
        declarations,
        staged_process_input,
        false,
        None,
        None,
        None,
    )
}

fn compile_program_expr_object(
    program: &RuntimeProgram,
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
    entry_symbol: &str,
) -> Result<CompiledModule<ObjectModule>, CraneliftBackendError> {
    compile_expr_into_module(
        new_object_module("ken-runtime-cranelift-object")?,
        entry_symbol,
        Linkage::Export,
        expr,
        seed_env,
        program
            .declarations
            .iter()
            .map(|declaration| (declaration.symbol.as_str(), declaration))
            .collect(),
        None,
        false,
        None,
        native_join_plan_for_program(program)?,
        oriented_subcontinuation_plan_for_program(program)?,
    )
}

fn native_isa() -> Result<OwnedTargetIsa, CraneliftBackendError> {
    let mut flag_builder = settings::builder();
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    flag_builder
        .set("is_pic", "true")
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    let isa_builder = cranelift_native::builder()
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    isa_builder
        .finish(settings::Flags::new(flag_builder))
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))
}

fn new_jit_module() -> Result<JITModule, CraneliftBackendError> {
    let isa = native_isa()?;
    let builder = JITBuilder::with_isa(isa, default_libcall_names());
    Ok(JITModule::new(builder))
}

fn new_object_module(name: &str) -> Result<ObjectModule, CraneliftBackendError> {
    let isa = native_isa()?;
    let builder = ObjectBuilder::new(isa, name.as_bytes().to_vec(), default_libcall_names())
        .map_err(|err| backend_module(err.to_string()))?;
    Ok(ObjectModule::new(builder))
}

fn native_platform_target_name() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

// RT-SPLIT slice 5: shared test helpers whose final users span the
// lowering subject subtree AND the facade's residual artifact/api tests.
// Final-user LCA is the facade, so they sit at facade FILE SCOPE under
// item-level `#[cfg(test)]` -- ancestor-private, reachable by descendants
// with zero widening. A sibling `mod tests` could not be reached at all.
#[cfg(test)]
const PX8N_SHORT_WROTE: u64 = 0;
#[cfg(test)]
const PX8N_ZERO_WRITE: u64 = 1;
#[cfg(test)]
fn run_px8n_write_arm_fixture(scenario: u64) -> (i64, Px8nHostReplyFixture) {
    run_px8n_arm_fixture(scenario, px8n_write_arm_fixture)
}
#[cfg(test)]
fn host_result_computational_fixture(
    ok_binders: usize,
    include_ok: bool,
    mismatched_result_kind: bool,
) -> RuntimeExpr {
    let result_ok = "ctor:prelude::Result::Ok".to_string();
    let result_err = "ctor:prelude::Result::Err".to_string();
    let scalar_tree = "ctor:fixture::Tree::Scalar".to_string();
    let exit_tree = "ctor:fixture::Tree::Exit".to_string();
    let mut producer_cases = vec![RuntimeMatchCase {
        constructor: result_err,
        binders: 1,
        body: RuntimeExpr::Construct {
            constructor: if mismatched_result_kind {
                exit_tree.clone()
            } else {
                scalar_tree.clone()
            },
            args: if mismatched_result_kind {
                Vec::new()
            } else {
                vec![RuntimeExpr::Value(RuntimeValue::Int((9).into()))]
            },
        },
    }];
    if include_ok {
        producer_cases.push(RuntimeMatchCase {
            constructor: result_ok,
            binders: ok_binders,
            body: RuntimeExpr::Construct {
                constructor: scalar_tree.clone(),
                args: vec![RuntimeExpr::Value(RuntimeValue::Int((7).into()))],
            },
        });
    }
    RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Effect {
                family: "Console".to_string(),
                operation: ken_host::HostOpV1::ConsoleWrite,
                capability: None,
                args: vec![
                    RuntimeExpr::Construct {
                        constructor: "ctor:prelude::Stream::Stdout".to_string(),
                        args: Vec::new(),
                    },
                    RuntimeExpr::Value(RuntimeValue::Bytes(b"probe".to_vec())),
                ],
            }),
            cases: producer_cases,
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "dynamic Result producer default".to_string(),
            },
        }),
        cases: vec![
            crate::RuntimeComputationalMatchCase {
                constructor: scalar_tree,
                argument_binders: 1,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Var(0),
            },
            crate::RuntimeComputationalMatchCase {
                constructor: exit_tree,
                argument_binders: 0,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Construct {
                    constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                    args: Vec::new(),
                },
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "computational tree default".to_string(),
        },
    }
}
#[cfg(test)]
fn constructor_field_aggregate() -> RuntimeExpr {
    RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Effect {
            family: "Console".to_string(),
            operation: ken_host::HostOpV1::ConsoleIsTerminal,
            capability: None,
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:prelude::Stream::Stdout".to_string(),
                args: Vec::new(),
            }],
        }),
        cases: [
            ("ctor:prelude::Bool::True", "ctor:prelude::Result::Ok", 7),
            ("ctor:prelude::Bool::False", "ctor:prelude::Result::Err", 9),
        ]
        .into_iter()
        .map(|(constructor, result, payload)| RuntimeMatchCase {
            constructor: constructor.to_string(),
            binders: 0,
            body: RuntimeExpr::Construct {
                constructor: result.to_string(),
                args: vec![RuntimeExpr::Value(RuntimeValue::Int((payload).into()))],
            },
        })
        .collect(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7p aggregate producer default".to_string(),
        },
    }
}
#[cfg(test)]
fn host_result_closure_match(argument: RuntimeExpr) -> RuntimeExpr {
    let exit_success = || RuntimeExpr::Construct {
        constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
        args: Vec::new(),
    };
    RuntimeExpr::Call {
        callee: Box::new(ordinary_match_closure(
            vec![
                RuntimeMatchCase {
                    constructor: "ctor:prelude::Result::Err".to_string(),
                    binders: 1,
                    body: exit_success(),
                },
                RuntimeMatchCase {
                    constructor: "ctor:prelude::Result::Ok".to_string(),
                    binders: 1,
                    body: exit_success(),
                },
            ],
            RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "direct HostResult default".to_string(),
            },
        )),
        args: vec![argument],
    }
}
#[cfg(test)]
fn recursive_computational_result_depth(depth: usize, leaf_body: RuntimeExpr) -> RuntimeExpr {
    let node = "ctor:fixture::RecursiveTree::Node";
    let leaf = "ctor:fixture::RecursiveTree::Leaf";
    fn child(depth: usize, node: &str, leaf: &str) -> RuntimeExpr {
        RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["unit".to_string()],
            body: Box::new(if depth == 0 {
                RuntimeExpr::Construct {
                    constructor: leaf.to_string(),
                    args: Vec::new(),
                }
            } else {
                RuntimeExpr::Construct {
                    constructor: node.to_string(),
                    args: vec![child(depth - 1, node, leaf)],
                }
            }),
        }
    }
    let recursive_child = child(depth, node, leaf);
    RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: node.to_string(),
            args: vec![recursive_child],
        }),
        cases: vec![
            crate::RuntimeComputationalMatchCase {
                constructor: node.to_string(),
                argument_binders: 1,
                recursive_positions: vec![0],
                body: RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::Var(0)),
                    args: vec![RuntimeExpr::Construct {
                        constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                        args: Vec::new(),
                    }],
                },
            },
            crate::RuntimeComputationalMatchCase {
                constructor: leaf.to_string(),
                argument_binders: 0,
                recursive_positions: Vec::new(),
                body: leaf_body,
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "recursive tree default".to_string(),
        },
    }
}
#[cfg(test)]
fn self_consistent_join_site(
    site_id: u64,
    runtime_frame_fingerprint: u64,
) -> crate::NativeJoinPlanSiteV1 {
    let declaration = "decl:fixture::PX8H::main".to_string();
    let checked_occurrence_path = vec![1, site_id];
    let checked_result_type_fingerprint = 17;
    crate::NativeJoinPlanSiteV1 {
        site_id,
        occurrence_binding_fingerprint: crate::compiler_private_join_occurrence_binding_fingerprint(
            site_id,
            &declaration,
            &checked_occurrence_path,
            checked_result_type_fingerprint,
        ),
        declaration,
        checked_occurrence_path,
        checked_result_type_fingerprint,
        runtime_frame_fingerprint,
        answer_kind: crate::NativeJoinAnswerKindV1::Int,
    }
}
#[cfg(test)]
fn total_primitive(symbol: &str, args: Vec<RuntimeExpr>) -> RuntimeExpr {
    RuntimeExpr::PrimitiveCall {
        primitive: RuntimePrimitive {
            symbol: symbol.to_string(),
            partiality: RuntimePartiality::Total,
        },
        args,
    }
}
#[cfg(test)]
fn big(sign: crate::Sign, limbs: &[u64]) -> RuntimeExpr {
    RuntimeExpr::Value(RuntimeValue::Int(crate::RuntimeIntV1::Big {
        sign,
        limbs: limbs.to_vec(),
    }))
}

#[cfg(test)]
#[repr(C)]
struct Px8nHostReplyFixture {
    scenario: u64,
    call_index: u64,
    malformed_request: u64,
}
#[cfg(test)]
fn px8n_write_arm_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
    px8n_write_arm_fixture_with_start(symbols, RuntimeExpr::Value(RuntimeValue::Int((7).into())))
}
#[cfg(test)]
fn run_px8n_arm_fixture(
    scenario: u64,
    expression: fn(&crate::NativeProcessSymbols) -> RuntimeExpr,
) -> (i64, Px8nHostReplyFixture) {
    let isa = native_isa().unwrap();
    let mut builder = JITBuilder::with_isa(isa, default_libcall_names());
    builder.symbol(
        "ken_host_dispatch_v1",
        px8n_scripted_host_dispatch as *const u8,
    );
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let compiled = compile_expr_into_module(
        JITModule::new(builder),
        "px8n_fs_write_at",
        Linkage::Local,
        &expression(&symbols),
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        Some(&symbols),
        Some(test_only_distinguished_root_join_plan()),
        None,
    )
    .unwrap();
    let input = BorrowedFixtureValue {
        kind: 1,
        tag: 0,
        data: std::ptr::null(),
        len: 0,
    };
    let mut fixture = Px8nHostReplyFixture {
        scenario,
        call_index: 0,
        malformed_request: 0,
    };
    let mut native_int_arena = crate::NativeIntArenaV1::default();
    let invocation = NativeInvocationFixture {
        process_input: &input,
        host_context: (&mut fixture as *mut Px8nHostReplyFixture).cast(),
        capability: 0,
        native_int_arena: &mut native_int_arena,
    };
    let (_, result) = compiled
        .run(Some((&invocation as *const NativeInvocationFixture).cast()))
        .unwrap();
    (result.unwrap(), fixture)
}
#[cfg(test)]
fn ordinary_match_closure(cases: Vec<RuntimeMatchCase>, default: RuntimeTrap) -> RuntimeExpr {
    RuntimeExpr::LexicalClosure {
        captures: Vec::new(),
        params: vec!["value".to_string()],
        body: Box::new(RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases,
            default,
        }),
    }
}

#[cfg(test)]
extern "C" fn px8n_scripted_host_dispatch(
    invocation: *const std::ffi::c_void,
    operation: i64,
    request: *const std::ffi::c_void,
    request_size: i64,
    reply: *mut std::ffi::c_void,
) -> i64 {
    // SAFETY: this symbol is installed only into the test JIT below, which
    // supplies these exact call-scoped fixtures for one synchronous call.
    let invocation = unsafe { &*(invocation.cast::<NativeInvocationFixture>()) };
    // SAFETY: `host_context` points to the live fixture for the duration of
    // the compiled call and is never retained by the dispatcher.
    let fixture = unsafe { &mut *(invocation.host_context.cast::<Px8nHostReplyFixture>()) };
    let expected = if fixture.call_index == 0
        || (fixture.call_index == 1 && fixture.scenario != PX8I_METADATA_BIG)
    {
        ken_host::HostOpV1::BufferAllocate
    } else if fixture.scenario == PX8I_METADATA_BIG {
        ken_host::HostOpV1::FsHandleMetadata
    } else if fixture.scenario == PX8I_WRAPPING_WRITE_START {
        ken_host::HostOpV1::FsWriteAt
    } else if fixture.scenario >= PX8N_SHORT_READ {
        ken_host::HostOpV1::FsReadAt
    } else {
        ken_host::HostOpV1::FsWriteAt
    };
    if operation != expected as i64 {
        fixture.malformed_request = 1;
        return -1;
    }
    let wire = ken_host::host_effect_wire_layout_v1(expected)
        .expect("PX8-N scripted operation has a generated wire layout");
    if request_size != i64::from(wire.request_size) {
        fixture.malformed_request = 2;
        return -1;
    }
    let load = |offset: u32| {
        // SAFETY: each offset is generated from the target-C layout for
        // this exact request record and the lowering supplied its size.
        unsafe { *(request.cast::<u8>().add(offset as usize).cast::<u64>()) }
    };
    if expected == ken_host::HostOpV1::BufferAllocate {
        if load(wire.request_offsets[0]) != 8 {
            fixture.malformed_request = 3;
            return -1;
        }
    } else if expected == ken_host::HostOpV1::FsHandleMetadata {
        if load(wire.request_offsets[0]) != 11 {
            fixture.malformed_request = 5;
            return -1;
        }
    } else if [
        load(wire.request_offsets[0]),
        load(wire.request_offsets[1]),
        load(wire.request_offsets[2]),
        load(wire.request_offsets[3]),
        load(wire.request_offsets[4]),
    ] != [
        11,
        22,
        0,
        match fixture.scenario {
            PX8I_BIG_READ_START => PX8I_BIG_U64,
            PX8I_WRAPPING_WRITE_START => u64::MAX - 1,
            _ => 7,
        },
        4,
    ] {
        fixture.malformed_request = 4;
        return -1;
    }
    // SAFETY: the reply pointer names the target-C-sized stack record
    // supplied by the compiled caller for this exact operation.
    unsafe { std::ptr::write_bytes(reply.cast::<u8>(), 0, wire.reply_size as usize) };
    let store = |offset: u32, value: u64| {
        // SAFETY: generated offsets are aligned u64 fields within the
        // zeroed reply record above.
        unsafe {
            *(reply.cast::<u8>().add(offset as usize).cast::<u64>()) = value;
        }
    };
    if expected == ken_host::HostOpV1::BufferAllocate {
        store(wire.reply_tag_offset, wire.reply_resource_tag);
        store(
            wire.reply_detail_offset,
            if fixture.call_index == 0 { 11 } else { 22 },
        );
    } else if expected == ken_host::HostOpV1::FsHandleMetadata {
        store(wire.reply_tag_offset, wire.reply_metadata_tag);
        store(wire.reply_detail_offset, PX8I_BIG_U64);
    } else {
        match fixture.scenario {
            PX8N_SHORT_WROTE | PX8I_WRAPPING_WRITE_START => {
                store(wire.reply_tag_offset, wire.reply_write_progress_tag);
                store(wire.reply_detail_offset, 1);
            }
            PX8N_ZERO_WRITE => {
                store(wire.reply_tag_offset, wire.reply_resource_error_tag);
                store(wire.reply_detail_offset, wire.resource_error_no_progress);
            }
            PX8N_OVER_BOUND_WRITE => {
                store(wire.reply_tag_offset, wire.reply_write_progress_tag);
                store(wire.reply_detail_offset, 5);
            }
            PX8N_SHORT_READ => {
                store(wire.reply_tag_offset, wire.reply_read_progress_tag);
                store(wire.reply_detail_offset, 1);
                store(wire.reply_bytes_len_offset, 7);
            }
            PX8N_READ_EOF => {
                store(wire.reply_tag_offset, wire.reply_read_progress_tag);
            }
            PX8N_OVER_BOUND_READ => {
                store(wire.reply_tag_offset, wire.reply_read_progress_tag);
                store(wire.reply_detail_offset, 5);
                store(wire.reply_bytes_len_offset, 7);
            }
            PX8I_BIG_READ_START => {
                store(wire.reply_tag_offset, wire.reply_read_progress_tag);
                store(wire.reply_detail_offset, 1);
                store(wire.reply_bytes_len_offset, PX8I_BIG_U64);
            }
            _ => return -1,
        }
    }
    fixture.call_index += 1;
    0
}
#[cfg(test)]
fn px8n_write_arm_fixture_with_start(
    symbols: &crate::NativeProcessSymbols,
    start: RuntimeExpr,
) -> RuntimeExpr {
    let trap = || RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "PX8-N checked result default".to_string(),
    };
    let allocate = || RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::BufferAllocate,
        capability: None,
        args: vec![RuntimeExpr::Value(RuntimeValue::Int((8).into()))],
    };
    let write = RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::FsWriteAt,
        capability: None,
        args: vec![
            RuntimeExpr::Var(1),
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            RuntimeExpr::Var(0),
            start,
            RuntimeExpr::Value(RuntimeValue::Int((4).into())),
        ],
    };
    let transfer_observation = px8n_exact_nat(
        symbols,
        RuntimeExpr::Var(0),
        0,
        px8n_exact_nat(
            symbols,
            RuntimeExpr::Var(1),
            3,
            RuntimeExpr::Value(RuntimeValue::Int((3).into())),
        ),
    );
    let success = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![crate::RuntimeMatchCase {
            constructor: symbols.wrote.clone(),
            binders: 1,
            body: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(0)),
                cases: vec![crate::RuntimeMatchCase {
                    constructor: symbols.private_transfer_count.clone(),
                    binders: 2,
                    body: px8n_failure(symbols, transfer_observation),
                }],
                default: trap(),
            },
        }],
        default: trap(),
    };
    let error = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![crate::RuntimeMatchCase {
            constructor: symbols.resource_no_progress.clone(),
            binders: 0,
            body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((70).into()))),
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-N expected exact NoProgress".to_string(),
        },
    };
    let write_result = RuntimeExpr::Match {
        scrutinee: Box::new(write),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: error,
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: success,
            },
        ],
        default: trap(),
    };
    let second = RuntimeExpr::Match {
        scrutinee: Box::new(allocate()),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((81).into()))),
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: write_result,
            },
        ],
        default: trap(),
    };
    RuntimeExpr::Match {
        scrutinee: Box::new(allocate()),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((80).into()))),
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: second,
            },
        ],
        default: trap(),
    }
}

#[cfg(test)]
const PX8N_SHORT_READ: u64 = 3;
#[cfg(test)]
const PX8I_METADATA_BIG: u64 = 6;
#[cfg(test)]
const PX8I_WRAPPING_WRITE_START: u64 = 8;
#[cfg(test)]
const PX8I_BIG_U64: u64 = i64::MAX as u64 + 97;
#[cfg(test)]
fn px8n_exact_nat(
    symbols: &crate::NativeProcessSymbols,
    nat: RuntimeExpr,
    depth: usize,
    exact: RuntimeExpr,
) -> RuntimeExpr {
    let mismatch = RuntimeExpr::Value(RuntimeValue::Int((99).into()));
    let cases = if depth == 0 {
        vec![
            crate::RuntimeMatchCase {
                constructor: symbols.nat_zero.clone(),
                binders: 0,
                body: exact,
            },
            crate::RuntimeMatchCase {
                constructor: symbols.nat_suc.clone(),
                binders: 1,
                body: mismatch,
            },
        ]
    } else {
        vec![
            crate::RuntimeMatchCase {
                constructor: symbols.nat_zero.clone(),
                binders: 0,
                body: mismatch,
            },
            crate::RuntimeMatchCase {
                constructor: symbols.nat_suc.clone(),
                binders: 1,
                body: px8n_exact_nat(symbols, RuntimeExpr::Var(0), depth - 1, exact),
            },
        ]
    };
    RuntimeExpr::Match {
        scrutinee: Box::new(nat),
        cases,
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: format!("PX8-N expected exact structural Nat depth {depth}"),
        },
    }
}
#[cfg(test)]
fn px8n_failure(symbols: &crate::NativeProcessSymbols, code: RuntimeExpr) -> RuntimeExpr {
    RuntimeExpr::Construct {
        constructor: symbols.exit_failure.clone(),
        args: vec![code],
    }
}

#[cfg(test)]
const PX8N_OVER_BOUND_WRITE: u64 = 2;
#[cfg(test)]
const PX8N_READ_EOF: u64 = 4;
#[cfg(test)]
const PX8N_OVER_BOUND_READ: u64 = 5;
#[cfg(test)]
const PX8I_BIG_READ_START: u64 = 7;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        evaluate_runtime_ir_example, nc5_seed_examples, ErasedExecutableCore,
        RuntimeArtifactValidationStage, RuntimeArtifactValidationTier, RuntimeAssumptionTrustKind,
        RuntimeAssumptionTrustMetadata, RuntimeDeclaration, RuntimeEffectsForeignAuditMetadata,
        RuntimeFieldStatus, RuntimeIrSeedEnvironment, RuntimeMatchCase, RuntimeMetadata,
        RuntimeSymbolMetadata,
    };

    fn seed_program_with_lowerability(status: Option<RuntimeLowerabilityStatus>) -> RuntimeProgram {
        let symbol = "decl:fixture::Main::main".to_string();
        let mut metadata = RuntimeMetadata::default();
        if let Some(status) = status.clone() {
            metadata.lowerability.insert(symbol.clone(), status);
        }
        RuntimeProgram {
            package_identity: "module:fixture::nc6".to_string(),
            core_semantic_hash: 1,
            artifact_hash: 2,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from([symbol.clone()]),
                metadata,
            },
            declarations: vec![RuntimeDeclaration {
                symbol,
                kind: RuntimeDeclarationKind::Record {
                    fields: vec![crate::RuntimeField {
                        name: "value".to_string(),
                        status: RuntimeFieldStatus::Runtime,
                    }],
                },
                metadata: RuntimeSymbolMetadata {
                    lowerability: status,
                    ..RuntimeSymbolMetadata::empty()
                },
            }],
            examples: nc5_seed_examples(),
        }
    }

    fn nc22_program_with_body(
        body: RuntimeExpr,
        observation: RuntimeObservation,
    ) -> RuntimeProgram {
        let symbol = "decl:fixture::Main::main".to_string();
        let mut metadata = RuntimeMetadata::default();
        metadata
            .lowerability
            .insert(symbol.clone(), RuntimeLowerabilityStatus::Supported);
        RuntimeProgram {
            package_identity: "module:fixture::nc22".to_string(),
            core_semantic_hash: 22,
            artifact_hash: 2200,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from([symbol.clone()]),
                metadata,
            },
            declarations: vec![RuntimeDeclaration {
                symbol: symbol.clone(),
                kind: RuntimeDeclarationKind::Transparent { body },
                metadata: RuntimeSymbolMetadata {
                    lowerability: Some(RuntimeLowerabilityStatus::Supported),
                    ..RuntimeSymbolMetadata::empty()
                },
            }],
            examples: vec![RuntimeExample {
                name: "main-entrypoint".to_string(),
                checked_core_shape: "compiler-produced declaration ref".to_string(),
                ir: RuntimeExpr::DeclarationRef { symbol },
                observation,
            }],
        }
    }

    #[test]
    fn program_runner_preflights_metadata_before_backend_lowering() {
        let program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));

        let reports = run_nc6_seed_examples(&program).expect("seed program runs");

        assert_eq!(reports.len(), 5);
        assert!(reports
            .iter()
            .all(|report| report.trust.fidelity == NativeFidelity::F1SeedObservationAgreement));
    }

    #[test]
    fn nc22_cranelift_agrees_with_runtime_ir_report_for_broad_starter_shapes() {
        let body = RuntimeExpr::Let {
            value: Box::new(total_primitive(
                "add_int",
                vec![
                    RuntimeExpr::Value(RuntimeValue::Int((2).into())),
                    RuntimeExpr::Value(RuntimeValue::Int((3).into())),
                ],
            )),
            body: Box::new(RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Closure {
                    captures: Vec::new(),
                    params: vec!["x".to_string()],
                    body: Box::new(RuntimeExpr::Match {
                        scrutinee: Box::new(RuntimeExpr::Construct {
                            constructor: "ctor:fixture::Box::Box".to_string(),
                            args: vec![RuntimeExpr::Var(0)],
                        }),
                        cases: vec![RuntimeMatchCase {
                            constructor: "ctor:fixture::Box::Box".to_string(),
                            binders: 1,
                            body: RuntimeExpr::Record {
                                fields: vec![
                                    (
                                        "ok".to_string(),
                                        RuntimeExpr::If {
                                            scrutinee: Box::new(total_primitive(
                                                "eq_int",
                                                vec![
                                                    RuntimeExpr::Var(0),
                                                    RuntimeExpr::Value(RuntimeValue::Int(
                                                        (5).into(),
                                                    )),
                                                ],
                                            )),
                                            then_expr: Box::new(RuntimeExpr::Value(
                                                RuntimeValue::Bool(true),
                                            )),
                                            else_expr: Box::new(RuntimeExpr::Value(
                                                RuntimeValue::Bool(false),
                                            )),
                                        },
                                    ),
                                    (
                                        "value".to_string(),
                                        total_primitive(
                                            "sub_int",
                                            vec![
                                                total_primitive(
                                                    "mul_int",
                                                    vec![
                                                        RuntimeExpr::Var(0),
                                                        RuntimeExpr::Value(RuntimeValue::Int(
                                                            (2).into(),
                                                        )),
                                                    ],
                                                ),
                                                RuntimeExpr::Value(RuntimeValue::Int((3).into())),
                                            ],
                                        ),
                                    ),
                                ],
                            },
                        }],
                        default: RuntimeTrap {
                            code: RuntimeTrapCode::PatternMatchFailure,
                            message: "unexpected constructor".to_string(),
                        },
                    }),
                }),
                args: vec![RuntimeExpr::Var(0)],
            }),
        };
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Record {
            fields: vec![
                ("ok".to_string(), RuntimeGroundValue::Bool(true)),
                ("value".to_string(), RuntimeGroundValue::Int((7).into())),
            ],
        });
        let program = nc22_program_with_body(body, observation.clone());
        let run_report = evaluate_runtime_ir_example(
            &program,
            &program.examples[0],
            &RuntimeIrSeedEnvironment::empty(),
        )
        .expect("runtime-IR evaluator runs the compiler-produced artifact");

        let report = run_runtime_ir_report_with_cranelift(
            &program,
            run_report,
            &NativeSeedEnvironment::empty(),
        );

        assert_eq!(
            report.verdict,
            NativeRuntimeIrComparisonVerdict::RuntimeIrNativeAgreement {
                stage: NativeDifferentialStage::RuntimeIrNativeCompare,
            }
        );
        let native = report.native.expect("native side ran");
        assert_eq!(native.observation, observation);
        assert_eq!(
            native.trust.fidelity,
            NativeFidelity::F1RuntimeIrEvaluatorAgreement
        );
        assert_eq!(
            native.trust.evidence.runtime_artifact_hash,
            Some(program.artifact_hash)
        );
    }

    #[test]
    fn nc22_imported_dependency_lowers_as_stable_unsupported_native_lane() {
        let symbol = "decl:fixture::Main::main".to_string();
        let dependency = "dep:fixture".to_string();
        let imported = "decl:dep::value".to_string();
        let dependency_hash = "hash:dep".to_string();
        let mut program = nc22_program_with_body(
            RuntimeExpr::ImportedDeclarationRef {
                symbol: imported.clone(),
                dependency: dependency.clone(),
                dependency_semantic_hash: dependency_hash.clone(),
            },
            RuntimeObservation::Returned(RuntimeGroundValue::Int((9).into())),
        );
        program.declarations[0].symbol = symbol.clone();
        program.erased_core.symbols.insert(imported.clone());
        program
            .erased_core
            .metadata
            .lowerability
            .insert(imported.clone(), RuntimeLowerabilityStatus::Supported);
        program
            .erased_core
            .metadata
            .dependency_semantic_hashes
            .insert(dependency.clone(), dependency_hash.clone());
        let mut runtime_env = RuntimeIrSeedEnvironment::empty();
        runtime_env.insert_imported_declaration(
            imported,
            dependency,
            dependency_hash,
            RuntimeGroundValue::Int((9).into()),
        );
        let run_report = evaluate_runtime_ir_example(&program, &program.examples[0], &runtime_env)
            .expect("runtime-IR evaluator can use an exact imported seed binding");

        let report = run_runtime_ir_report_with_cranelift(
            &program,
            run_report,
            &NativeSeedEnvironment::empty(),
        );

        assert!(matches!(
            report.verdict,
            NativeRuntimeIrComparisonVerdict::Unsupported {
                stage: NativeDifferentialStage::NativeLoweringOrExecution,
                construct: "ImportedDeclarationRef",
                ..
            }
        ));
        assert!(report.native.is_none());
    }

    #[test]
    fn nc22_runtime_ir_report_identity_mismatch_rejects_before_native_lowering() {
        let program = nc22_program_with_body(
            RuntimeExpr::Value(RuntimeValue::Int((1).into())),
            RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
        );
        let mut run_report = evaluate_runtime_ir_example(
            &program,
            &program.examples[0],
            &RuntimeIrSeedEnvironment::empty(),
        )
        .expect("runtime-IR evaluator runs");
        run_report.evidence.runtime_artifact_hash = 0xdead_beef;

        let report = run_runtime_ir_report_with_cranelift(
            &program,
            run_report,
            &NativeSeedEnvironment::empty(),
        );

        assert!(matches!(
            report.verdict,
            NativeRuntimeIrComparisonVerdict::Unsupported {
                stage: NativeDifferentialStage::BoundaryPreflight,
                construct: "RuntimeIrRunReport",
                ..
            }
        ));
        assert!(report.native.is_none());
    }

    #[test]
    fn nc22_ambiguous_runtime_ir_report_target_rejects_before_native_lowering() {
        let mut program = nc22_program_with_body(
            RuntimeExpr::Value(RuntimeValue::Int((1).into())),
            RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
        );
        program.examples.push(program.examples[0].clone());
        let mut run_report = evaluate_runtime_ir_example(
            &nc22_program_with_body(
                RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
            ),
            &program.examples[0],
            &RuntimeIrSeedEnvironment::empty(),
        )
        .expect("runtime-IR evaluator runs");
        run_report.artifact = RuntimeArtifactIdentity::from_program(&program);
        run_report.observation.artifact = RuntimeArtifactIdentity::from_program(&program);
        run_report.evidence.package_identity = program.package_identity.clone();
        run_report.evidence.core_semantic_hash = program.core_semantic_hash;
        run_report.evidence.runtime_artifact_hash = program.artifact_hash;

        let report = run_runtime_ir_report_with_cranelift(
            &program,
            run_report,
            &NativeSeedEnvironment::empty(),
        );

        assert!(matches!(
            report.verdict,
            NativeRuntimeIrComparisonVerdict::Unsupported {
                stage: NativeDifferentialStage::BoundaryPreflight,
                construct: "RuntimeIrRunReport",
                ..
            }
        ));
        assert!(report.native.is_none());
    }

    #[test]
    fn nc8_valid_certificate_records_f2_validation_separate_from_f1() {
        let example = nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "closed-scalar-primitive")
            .expect("seed exists");
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![example.clone()];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
        let oracle = InterpreterOracleObservation {
            artifact: NativeArtifactIdentity::from_program(&program),
            observation: example.observation.clone(),
            evidence_source: "test oracle over matching RuntimeProgram identity".to_string(),
        };

        let report = run_validated_example_with_interpreter_observation(
            &program,
            &example,
            &NativeSeedEnvironment::empty(),
            oracle,
            &certificate,
        )
        .expect("certificate validates");

        assert_eq!(
            report.verdict,
            NativeDifferentialVerdict::F1InterpreterAgreement {
                stage: NativeDifferentialStage::InterpreterNativeCompare,
            }
        );
        let native = report.native.expect("native side ran");
        assert_eq!(
            native.trust.fidelity,
            NativeFidelity::F1InterpreterDifferentialAgreement
        );
        let validation = native
            .trust
            .artifact_validation
            .expect("validated artifact fact is report-visible");
        assert_eq!(
            validation.tier,
            RuntimeArtifactValidationTier::F2BoundedRuntimeArtifactValidation
        );
        assert_eq!(
            validation.artifact.package_identity,
            program.package_identity
        );
        assert_eq!(
            validation.artifact.core_semantic_hash,
            program.core_semantic_hash
        );
        assert_eq!(validation.artifact.artifact_hash, program.artifact_hash);
        assert!(validation
            .evidence_source
            .contains("recomputed supported-subset facts"));
    }

    #[test]
    fn nc8_certificate_wrong_identity_rejects_before_native_run() {
        let program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
        certificate.artifact_hash = Some(0xdead_beef);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("wrong artifact identity rejects");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ArtifactIdentity);
        assert_eq!(err.fact, "runtime_artifact_identity");
    }

    #[test]
    fn nc8_certificate_missing_fields_rejects_loudly() {
        let program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
        certificate.core_semantic_hash = None;

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("missing identity field rejects");

        assert_eq!(
            err.stage,
            RuntimeArtifactValidationStage::MalformedCertificate
        );
        assert_eq!(err.fact, "core_semantic_hash");

        let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
        certificate.claim.as_mut().expect("claim exists").facts = None;
        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("missing facts reject");

        assert_eq!(
            err.stage,
            RuntimeArtifactValidationStage::MalformedCertificate
        );
        assert_eq!(err.fact, "facts");
    }

    #[test]
    fn nc8_certificate_contradictory_claim_rejects() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "closed-scalar-primitive")
            .expect("seed exists")];
        let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
        certificate
            .claim
            .as_mut()
            .expect("claim exists")
            .facts
            .as_mut()
            .expect("facts exist")
            .declaration_count = Some(program.declarations.len() + 1);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("contradictory count rejects");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimMismatch);
        assert_eq!(err.fact, "declaration_count");
    }

    #[test]
    fn nc8_certificate_false_supported_claim_rejects_by_recomputation() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Unsupported {
                reason: "not lowerable".to_string(),
            }));
        let symbol = program.declarations[0].symbol.clone();
        program.declarations[0].metadata.lowerability =
            Some(RuntimeLowerabilityStatus::Unsupported {
                reason: "not lowerable".to_string(),
            });
        program
            .erased_core
            .metadata
            .unsupported
            .insert(symbol, b"hidden blocker".to_vec());
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("false supported-subset claim rejects");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert!(matches!(
            err.fact,
            "no_reachable_unsupported_entries" | "all_reachable_lowerability_supported"
        ));
    }

    #[test]
    fn nc8_certificate_rejects_unknown_runtime_value_by_recomputation() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "unknown-runtime-value".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Value(RuntimeValue::Unknown),
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((0).into())),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("unknown runtime values are outside the supported subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_values_supported");
        assert!(err.reason.contains("unknown runtime data"));
    }

    #[test]
    fn nc8_certificate_rejects_let_expression_in_validated_example() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "let-outside-supported-subset".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
                body: Box::new(RuntimeExpr::Var(0)),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("let expressions are outside the NC6 supported subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Let"));
    }

    #[test]
    fn nc8_certificate_rejects_if_expression_in_reachable_transparent_declaration() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::If {
                scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
                then_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
                else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((0).into()))),
            },
        };
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("if expressions are outside the NC6 supported subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("If"));
    }

    #[test]
    fn nc8_certificate_rejects_unsupported_total_primitive_in_validated_example() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "unsupported-total-primitive".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "sub_int".to_string(),
                    partiality: RuntimePartiality::Total,
                },
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Int((2).into())),
                    RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                ],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("unsupported total primitives are outside the NC6 supported subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_primitives_supported");
        assert!(err.reason.contains("sub_int"));
    }

    #[test]
    fn nc8_certificate_rejects_add_int_wrong_arity_in_reachable_transparent_declaration() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "add_int".to_string(),
                    partiality: RuntimePartiality::Total,
                },
                args: vec![RuntimeExpr::Value(RuntimeValue::Int((1).into()))],
            },
        };
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("add_int arity mismatch is outside the NC6 supported subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_primitives_supported");
        assert!(err.reason.contains("arity 1"));
    }

    #[test]
    fn nc8_certificate_rejects_add_int_non_literal_int_operand_shape() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "add-int-non-int-operand".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "add_int".to_string(),
                    partiality: RuntimePartiality::Total,
                },
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Bool(true)),
                    RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                ],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((2).into())),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("add_int non-literal-Int operands are outside the NC8 subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_primitives_supported");
        assert!(err.reason.contains("non-literal-Int operand"));
    }

    #[test]
    fn nc8_certificate_rejects_add_int_var_bound_to_bool_payload() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "add-int-var-bound-to-bool".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Construct {
                    constructor: "ctor:fixture::BoolBox::Box".to_string(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
                }),
                cases: vec![RuntimeMatchCase {
                    constructor: "ctor:fixture::BoolBox::Box".to_string(),
                    binders: 1,
                    body: RuntimeExpr::PrimitiveCall {
                        primitive: RuntimePrimitive {
                            symbol: "add_int".to_string(),
                            partiality: RuntimePartiality::Total,
                        },
                        args: vec![
                            RuntimeExpr::Var(0),
                            RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                        ],
                    },
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "unused default".to_string(),
                },
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((2).into())),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("add_int variable operands are outside the first NC8 validator");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Match"));
    }

    #[test]
    fn nc8_certificate_rejects_top_level_var_example() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "top-level-var".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Var(0),
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((0).into())),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("unbound var is outside the first NC8 validator");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Var"));
    }

    #[test]
    fn nc8_certificate_rejects_project_from_non_record_example() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "project-from-int".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Project {
                record: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
                field: "x".to_string(),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("project is outside the first NC8 validator");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Project"));
    }

    #[test]
    fn nc8_certificate_rejects_top_level_observable_closure() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "top-level-closure".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Closure {
                captures: Vec::new(),
                params: Vec::new(),
                body: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("closure is outside the first NC8 validator");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Closure"));
    }

    #[test]
    fn nc8_certificate_rejects_var_in_reachable_transparent_declaration() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::Var(0),
        };
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("transparent declaration var is outside the first NC8 validator");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Var"));
    }

    #[test]
    fn missing_lowerability_metadata_rejects_before_backend_lowering() {
        let program = seed_program_with_lowerability(None);

        let err = run_nc6_seed_examples(&program).expect_err("missing metadata rejects");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }

    #[test]
    fn reachable_unsupported_metadata_rejects_before_backend_lowering() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let symbol = program.declarations[0].symbol.clone();
        program
            .erased_core
            .metadata
            .unsupported
            .insert(symbol, b"unsupported target".to_vec());

        let err = run_nc6_seed_examples(&program).expect_err("unsupported metadata rejects");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }

    #[test]
    fn reachable_declaration_effect_metadata_rejects_before_backend_lowering() {
        for lane in [
            "effects",
            "capabilities",
            "runtime_checks",
            "assumptions",
            "assumption_trust_metadata",
            "trusted_base_delta",
        ] {
            let mut program =
                seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
            let target = program.declarations[0].symbol.clone();
            match lane {
                "effects" => {
                    program.declarations[0]
                        .metadata
                        .effects
                        .insert("Console".to_string());
                }
                "capabilities" => {
                    program.declarations[0]
                        .metadata
                        .capabilities
                        .insert("cap:Console".to_string());
                }
                "runtime_checks" => {
                    program.declarations[0]
                        .metadata
                        .runtime_checks
                        .insert("check:Console".to_string());
                }
                "assumptions" => {
                    program.declarations[0]
                        .metadata
                        .assumptions
                        .insert("assume:Console".to_string());
                }
                "assumption_trust_metadata" => {
                    program.declarations[0]
                        .metadata
                        .assumption_trust_metadata
                        .insert(
                            "assume:Console".to_string(),
                            RuntimeAssumptionTrustMetadata {
                                kind: RuntimeAssumptionTrustKind::Declassify,
                                target,
                                affects_runtime_meaning: true,
                            },
                        );
                }
                "trusted_base_delta" => {
                    program.declarations[0]
                        .metadata
                        .trusted_base_delta
                        .insert("assume:Console".to_string());
                }
                _ => unreachable!("test lanes are exhaustive"),
            }

            let err = match run_nc6_seed_examples(&program) {
                Ok(_) => panic!("expected {lane} metadata to reject"),
                Err(err) => err,
            };

            assert!(matches!(
                err,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "RuntimeProgram",
                    ..
                })
            ));
        }
    }

    #[test]
    fn reachable_package_effect_metadata_rejects_before_backend_lowering() {
        for lane in ["effects", "capabilities", "runtime_checks"] {
            let mut program =
                seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
            match lane {
                "effects" => {
                    program
                        .erased_core
                        .metadata
                        .effects
                        .insert("Console".to_string());
                }
                "capabilities" => {
                    program
                        .erased_core
                        .metadata
                        .capabilities
                        .insert("cap:Console".to_string());
                }
                "runtime_checks" => {
                    program
                        .erased_core
                        .metadata
                        .runtime_checks
                        .insert("check:Console".to_string());
                }
                _ => unreachable!("test lanes are exhaustive"),
            }

            let err = match run_nc6_seed_examples(&program) {
                Ok(_) => panic!("expected package {lane} metadata to reject"),
                Err(err) => err,
            };

            assert!(matches!(
                err,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "RuntimeProgram",
                    ..
                })
            ));
        }
    }

    #[test]
    fn reachable_effectful_checked_core_metadata_rejects_before_backend_lowering() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let symbol = program.declarations[0].symbol.clone();
        program
            .erased_core
            .metadata
            .checked_core
            .effects_foreign_metadata
            .insert(
                symbol,
                RuntimeEffectsForeignAuditMetadata {
                    declared_effects: BTreeSet::from(["Console".to_string()]),
                    capabilities: BTreeSet::from(["cap:Console".to_string()]),
                    foreign_symbol: None,
                    boundary: RuntimeEffectBoundary::Effectful,
                    runtime_checks: BTreeSet::from(["check:Console".to_string()]),
                    lowerability: RuntimeLowerabilityStatus::Supported,
                },
            );

        let err = run_nc6_seed_examples(&program)
            .expect_err("effectful checked-core metadata must reject");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }

    #[test]
    fn reachable_foreign_checked_core_metadata_rejects_before_backend_lowering() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let symbol = program.declarations[0].symbol.clone();
        program
            .erased_core
            .metadata
            .checked_core
            .effects_foreign_metadata
            .insert(
                symbol,
                RuntimeEffectsForeignAuditMetadata {
                    declared_effects: BTreeSet::new(),
                    capabilities: BTreeSet::new(),
                    foreign_symbol: Some("host.fixture.foreign".to_string()),
                    boundary: RuntimeEffectBoundary::Foreign,
                    runtime_checks: BTreeSet::new(),
                    lowerability: RuntimeLowerabilityStatus::Supported,
                },
            );

        let err =
            run_nc6_seed_examples(&program).expect_err("foreign checked-core metadata must reject");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }

    #[test]
    fn px8i_jit_and_object_construct_identical_local_helper_clif() {
        let mut jit = new_jit_module().expect("JIT module constructs");
        let jit_clif = crate::native_int_clif::capture_native_int_local_graph(&mut jit)
            .expect("JIT local helper graph emits");
        let mut object =
            new_object_module("px8i-local-helper-identity").expect("object module constructs");
        let object_clif = crate::native_int_clif::capture_native_int_local_graph(&mut object)
            .expect("object local helper graph emits");
        assert_eq!(jit_clif, object_clif);
        assert!(!jit_clif.is_empty());
        // Rework (Q-RESIDUE, 2026-07-21): the bare `5` was unverified
        // provenance. Grounded against `emit_native_int_local_graph`, which
        // calls exactly six `define_*` helpers (resolve, intern, compare,
        // narrow, export, binop); `capture_native_int_local_graph` joins
        // their captured CLIF bodies with "-- helper --", so N helpers yield
        // N-1 separators. This is a fixed property of the compiler's own
        // small, deliberately-enumerated local-helper set, not an external or
        // growable corpus -- pinning it here catches a helper silently
        // failing to emit a body.
        const LOCAL_HELPER_COUNT: usize = 6;
        assert_eq!(
            jit_clif.matches("-- helper --").count(),
            LOCAL_HELPER_COUNT - 1,
            "expected all {LOCAL_HELPER_COUNT} native-Int local helpers (resolve, intern, compare, narrow, export, binop) to emit a captured CLIF body"
        );
    }

    #[test]
    fn px8i_local_helpers_reject_invalid_zero_stale_and_wrong_arena_slots() {
        let mut module = new_jit_module().expect("JIT module constructs");
        let helpers = crate::native_int_clif::emit_native_int_local_graph(&mut module, false)
            .expect("local helper graph emits");
        let pointer = module.target_config().pointer_type();

        let mut mint_signature = module.make_signature();
        mint_signature.params.push(AbiParam::new(pointer));
        mint_signature.returns.push(AbiParam::new(types::I64));
        let mint_id = module
            .declare_function("px8i_mint_probe", Linkage::Local, &mint_signature)
            .expect("mint probe declares");
        let mut mint_context = module.make_context();
        mint_context.func =
            Function::with_name_signature(UserFuncName::user(2, mint_id.as_u32()), mint_signature);
        let intern = module.declare_func_in_func(helpers.intern, &mut mint_context.func);
        let mut frontend = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut mint_context.func, &mut frontend);
            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);
            let arena = builder.block_params(entry)[0];
            let limbs = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                16,
                3,
            ));
            let zero = builder.ins().iconst(types::I64, 0);
            let one = builder.ins().iconst(types::I64, 1);
            builder.ins().stack_store(zero, limbs, 0);
            builder.ins().stack_store(one, limbs, 8);
            let output = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                16,
                3,
            ));
            let limbs = builder.ins().stack_addr(pointer, limbs, 0);
            let output_pointer = builder.ins().stack_addr(pointer, output, 0);
            let two = builder.ins().iconst(types::I64, 2);
            let call = builder
                .ins()
                .call(intern, &[arena, zero, limbs, two, output_pointer]);
            let status = builder.inst_results(call)[0];
            require_i64_for_artifact_tests(&mut builder, status, 0);
            let slot = builder.ins().stack_load(types::I64, output, 8);
            builder.ins().return_(&[slot]);
            builder.seal_all_blocks();
            builder.finalize();
        }
        verify_cranelift_function(&mint_context.func, module.isa()).expect("mint verifies");
        module
            .define_function(mint_id, &mut mint_context)
            .expect("mint defines");

        let mut check_signature = module.make_signature();
        check_signature.params.push(AbiParam::new(pointer));
        check_signature.params.push(AbiParam::new(types::I64));
        check_signature.params.push(AbiParam::new(types::I64));
        check_signature.returns.push(AbiParam::new(types::I64));
        let check_id = module
            .declare_function("px8i_slot_probe", Linkage::Local, &check_signature)
            .expect("slot probe declares");
        let mut check_context = module.make_context();
        check_context.func = Function::with_name_signature(
            UserFuncName::user(2, check_id.as_u32()),
            check_signature,
        );
        let compare = module.declare_func_in_func(helpers.compare, &mut check_context.func);
        let mut frontend = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut check_context.func, &mut frontend);
            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);
            let params = builder.block_params(entry).to_vec();
            let eq = builder.ins().iconst(types::I64, 0);
            let call = builder.ins().call(
                compare,
                &[params[0], eq, params[1], params[2], params[1], params[2]],
            );
            let status = builder.inst_results(call)[0];
            builder.ins().return_(&[status]);
            builder.seal_all_blocks();
            builder.finalize();
        }
        verify_cranelift_function(&check_context.func, module.isa()).expect("check verifies");
        module
            .define_function(check_id, &mut check_context)
            .expect("check defines");
        module
            .finalize_definitions()
            .expect("probe module finalizes");

        let mint = module.get_finalized_function(mint_id);
        let check = module.get_finalized_function(check_id);
        let mint = unsafe {
            mem::transmute::<_, extern "C" fn(*mut crate::NativeIntArenaV1) -> u64>(mint)
        };
        let check = unsafe {
            mem::transmute::<_, extern "C" fn(*mut crate::NativeIntArenaV1, u64, u64) -> i64>(check)
        };
        let mut first = crate::NativeIntArenaV1::default();
        let mut second = crate::NativeIntArenaV1::default();
        let slot = mint(&mut first);
        assert_ne!(slot, 0);
        assert_eq!(check(&mut first, crate::NATIVE_INT_BIG_TAG_V1, slot), 1);
        assert_eq!(check(&mut first, crate::NATIVE_INT_BIG_TAG_V1, 0), -1);
        assert_eq!(check(&mut second, crate::NATIVE_INT_BIG_TAG_V1, slot), -1);
        assert_eq!(check(&mut first, 9, slot), -1);
    }
}
