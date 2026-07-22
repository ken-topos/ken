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
use cranelift_module::Linkage;

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
    native_platform_target_name_for_lowering_tests as native_platform_target_name,
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
// §10.4 THE FINAL EXPLICIT FACADE CUT (slice 7): the facade uses explicit
// re-export lists ONLY -- no internal `pub use child::*`. This replaces
// `pub use surface::*;`, the last internal glob in the module.
//
// ⛔ This list is the whole point of the cut and it is DERIVED, not authored:
// it is exactly the set of module-level bare-`pub` items in `surface.rs`,
// enumerated mechanically and checked for set equality in BOTH directions.
// A name dropped here vanishes from `ken_runtime::*` via `lib.rs:39` and can
// still compile green across the whole workspace -- slice 6 measured that 6 of
// 10 re-exported names had ZERO in-repo consumers, so the compiler is not a
// net for this edit. `NativeSeedEnvironment::{empty, nc5_seed, insert}` are
// impl METHODS, not module items, and correctly do not appear.
//
// Re-exporting an already-exported name at its pre-existing visibility is not
// a widening (§10.4) -- the AC-7 ledger is unchanged.
// `backend_module` is `pub(crate)` in `surface` and the GLOB was re-exporting
// it: `native_int_clif.rs:14`, a SIBLING of this module, imports it as
// `crate::cranelift_backend::backend_module`. Restricted-visibility items are
// re-exported by a glob too, so enumerating only bare-`pub` declarations drops
// them -- the compiler caught this one because it happens to have an in-crate
// consumer, which is luck, not a net. Re-exported at its PRE-EXISTING
// `pub(crate)`, so this is not a widening (§10.4).
pub(crate) use surface::backend_module;
pub use surface::{
    BackendFailure, CraneliftBackendError, CraneliftObjectArtifact, CraneliftRunReport,
    InterpreterOracleObservation, NativeArtifactIdentity, NativeDifferentialReport,
    NativeDifferentialStage, NativeDifferentialVerdict, NativeEvidenceFact, NativeFidelity,
    NativeRunEvidence, NativeRuntimeIrComparisonReport, NativeRuntimeIrComparisonVerdict,
    NativeSeedEnvironment, NativeToolchainReport, NativeTrustReport, UnsupportedLowering,
    ValidatedNativeRunError,
};

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
