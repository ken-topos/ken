//! NC6 Cranelift backend spike for the NC5 runtime IR seed.
//!
//! This module deliberately keeps the native boundary narrow. Cranelift code
//! returns scalar `i64` values directly and aggregate observations through an
//! opaque token table decoded by this Rust layer. Native addresses, object
//! layout, allocation order, ABI details, and Cranelift internals never become
//! Ken-observable meaning.

use std::collections::{BTreeMap, BTreeSet};

// ⛔ RT-SPLIT slice 7: once the nine internals moved into `artifact`, these
// became unused in the LIB build while the residual `#[cfg(test)]` fixtures
// still need them. The `cfg(test)` build asymmetry is bidirectional, so the
// lib build's unused-import warning is NOT authority to delete them --
// pruning was taken over the INTERSECTION of both configs, never either
// alone. Gated rather than deleted, per this file's existing rule for the
// same situation at `fnv1a_64` below: "gate rather than choose a side."
#[cfg(test)]
use cranelift_jit::{JITBuilder, JITModule};
#[cfg(test)]
use cranelift_module::{default_libcall_names, Linkage};

// `RuntimeProgram` is still used by the lib build; the other three are
// test-only after the move, so this splits rather than gating wholesale.
use crate::RuntimeProgram;
#[cfg(test)]
use crate::{RuntimeDeclaration, RuntimeExpr, RuntimeValue};
#[cfg(test)]
mod test_support;

pub(crate) mod artifact;
pub(crate) mod compiled;
mod lowering;
pub(crate) mod planning;
pub(crate) mod surface;

// §10.1: the facade preserves the exact old `ken_runtime::<name>` surface.
// Re-exporting an already-exported name at its PRE-EXISTING visibility is not
// a widening (§10.4) — the AC-7 ledger is unchanged.
pub(crate) use artifact::api::{
    emit_bound_process_program_object_with_cranelift, run_process_expr_with_cranelift,
};

// `#[cfg(test)]`-only crate-root consumers: the residual omnibus `mod tests`
// uses these; the lib build does not. An UNCONDITIONAL `use` here warns as
// unused in the lib build, and deleting them breaks the test build — the
// asymmetry is bidirectional, so gate rather than choose a side.
#[cfg(test)]
use crate::fnv1a_64;
pub use artifact::api::{
    emit_runtime_ir_object_with_cranelift, reject_program_blockers,
    run_example_with_interpreter_observation, run_example_with_seed_observation,
    run_ken_checked_proof_erasure_example_with_interpreter_observation, run_nc6_seed_examples,
    run_nc8_validated_seed_examples, run_runtime_ir_report_with_cranelift,
    run_validated_example_with_interpreter_observation,
};
// Test-only after slice 7: the production consumers of this glob moved into
// `artifact`. Gated, not deleted -- see the bidirectional-asymmetry note above.
#[cfg(test)]
use lowering::core::*;

// RT-SPLIT slice 5 — `#[cfg(test)]` facade wiring (Architect
// `evt_3tgaw9ws44fqg`) for lowering-owned items the remaining facade fixtures
// still reach. `verify_cranelift_function_for_artifact_tests` left with the
// two `px8i_*` tests in slice 7 and is now imported at their new home,
// `artifact/tests.rs`; `lowering` itself was not touched by either slice.
#[cfg(test)]
use lowering::{PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE, PX8TR_TRAP_PROVENANCE};

// RT-SPLIT slice 7 — owner-adjacent test adapters (§10.5a′) for the three
// artifact privates that the facade's own `#[cfg(test)]` fixtures still reach
// after the internals moved down. Aliasing at the import keeps every call
// token in those fixture bodies byte-identical, so this is an IMPORT-ONLY
// edit and the fixtures stay ordered item-level moves (AC-3) — the same shape
// as the slice-5 wiring above and as §10.5a′'s lowering-test wiring.
// Zero production visibility is spent; the adapters are `#[cfg(test)]`.
#[cfg(test)]
use artifact::{
    native_isa_for_facade_fixtures as native_isa,
    native_platform_target_name_for_facade_fixtures as native_platform_target_name,
    new_object_module_for_lowering_tests as new_object_module,
};

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
pub use surface::*;

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
        Some(crate::cranelift_backend::test_support::test_only_distinguished_root_join_plan()),
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
        Some(crate::cranelift_backend::test_support::test_only_distinguished_root_join_plan()),
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
        Some(crate::cranelift_backend::test_support::test_only_distinguished_root_join_plan()),
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
        Some(crate::cranelift_backend::test_support::test_only_distinguished_root_join_plan()),
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
