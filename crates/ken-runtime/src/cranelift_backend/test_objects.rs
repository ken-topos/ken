//! Test-only native object-emission entry points and their fixtures.
//!
//! The whole module is gated by its declaration in the parent
//! (`#[cfg(test)] mod test_objects;`), so nothing here is compiled into a
//! library build and no item carries its own `#[cfg(test)]`.
//!
//! Distinct from the sibling `test_support`, and the split is by reach, not by
//! taste:
//!
//! | | `test_support` | here |
//! |---|---|---|
//! | holds | shared fixture *constructors* | object-emission *entry points* + their fixtures |
//! | visibility | `pub(super)`, never re-exported | `pub(crate)`, re-exported by the facade |
//! | consumers | inside `cranelift_backend` only | plus `object_linker_packaging`, a crate-root sibling |
//!
//! `test_support`'s charter forbids both halves of what this module needs — it
//! admits no owner-adjacent boundary adapter, and no re-export may carry a name
//! out of it — so these five items cannot live there.
//!
//! ⚠ `emit_process_entrypoint_object_with_cranelift` and
//! `emit_px8tr_nested_post_effect_object` are reached from
//! `object_linker_packaging` by two DIFFERENT paths: the first at the crate
//! root (`crate::…`, via the `lib.rs` glob re-export of this module's parent),
//! the second module-qualified (`crate::cranelift_backend::…`). Both are
//! `pub(crate)` for that reason, and both depend on the facade's re-export
//! block. Narrowing either visibility, or dropping either re-export, breaks a
//! consumer that this module's own compilation cannot see.
//!
//! ⚠ `ken-cli/tests/px4b_native_production.rs` reads THIS FILE as text and
//! asserts the declaration shape of
//! `emit_process_entrypoint_object_with_cranelift` — it pins the visibility
//! spelling, not the behavior. Renaming it, re-grading it, or reformatting that
//! signature breaks a test in another crate that no `-p ken-runtime` build
//! configuration can observe; it fails only in CI. Grep the workspace for the
//! name as a STRING before touching that declaration.

use std::collections::BTreeMap;

use crate::fnv1a_64;
use crate::{RuntimeDeclaration, RuntimeExpr, RuntimeValue};
use cranelift_module::Linkage;

// Reached through the facade's own re-export, which is how this name was in
// scope while these bodies still lived there.
use super::CraneliftObjectArtifact;

use super::artifact::{
    native_platform_target_name_for_lowering_tests as native_platform_target_name,
    new_object_module_for_lowering_tests as new_object_module,
};
use super::lowering::core::*;
use super::lowering::{PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE, PX8TR_TRAP_PROVENANCE};

pub(crate) fn emit_process_entrypoint_object_with_cranelift(
    entrypoint: &RuntimeExpr,
    entry_symbol: impl Into<String>,
) -> Result<CraneliftObjectArtifact, CraneliftBackendError> {
    let entry_symbol = entry_symbol.into();
    let compiled = compile_expr_into_object_module(
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

pub(crate) struct Px8trNestedRouteObject {
    pub artifact: CraneliftObjectArtifact,
    pub provenance: Vec<Px8trTrapProvenanceEvent>,
}

fn px8tr_test_interface(name: u8) -> crate::CheckedAnswerInterfaceV1 {
    let mut bytes = crate::CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec();
    bytes.push(name);
    crate::CheckedAnswerInterfaceV1::new(bytes).expect("PX8-TR test interface is canonical")
}

/// `D5a` localization: the same fixture's planning inputs, so a census can be
/// taken independently of the emission run.
#[cfg(test)]
pub(crate) fn px8tr_nested_post_effect_planning_inputs()
-> (crate::RuntimeExpr, Vec<crate::RuntimeDeclaration>) {
    let (entry, declaration, _plan) = px8tr_nested_post_effect_fixture();
    (entry, vec![declaration])
}

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
                        callee: Box::new(RuntimeExpr::Var(
                            if px8tr_call_selected_recursive_argument() {
                                2
                            } else {
                                0
                            },
                        )),
                        // `D8p` — under test only, nest an ORDINARY call on the
                        // same recursor binder inside the checked application.
                        // The nested call instantiates a semantic IH layer of
                        // its own, and a segment carrying a checked frame
                        // requires every semantic layer to carry a checked
                        // invocation authority. This is the refusal `D8p` must
                        // not weaken: projecting the checked-application seam
                        // onto a second call edge must not make an unauthorised
                        // layer acceptable.
                        args: if px8tr_nest_ordinary_ih_call() {
                            vec![RuntimeExpr::Call {
                                callee: Box::new(RuntimeExpr::Var(0)),
                                args: vec![unit()],
                            }]
                        } else {
                            vec![unit()]
                        },
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

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8p`** — nest an ordinary call on the same
/// recursor binder inside the checked application, under test only.
#[cfg(test)]
thread_local! {
    static PX8TR_NEST_ORDINARY_IH_CALL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn set_px8tr_nest_ordinary_ih_call(armed: bool) {
    PX8TR_NEST_ORDINARY_IH_CALL.with(|cell| cell.set(armed));
}

#[cfg(test)]
fn px8tr_nest_ordinary_ih_call() -> bool {
    PX8TR_NEST_ORDINARY_IH_CALL.with(std::cell::Cell::get)
}

#[cfg(not(test))]
fn px8tr_nest_ordinary_ih_call() -> bool {
    false
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D6b`** — call the SELECTED RECURSIVE
/// ARGUMENT, under test only.
///
/// `Var(2)` is that member: the `Vis` case has `argument_binders = 2` and
/// `recursive_positions = [1]`, so
/// [`continuation_case_binder_run`](crate::cranelift_backend::lowering::units::continuation_case_binder_run)
/// lays the run out as `[IH, ordinary field 0, SelectedRecursiveArgument{1}, ..]`.
/// ⛔ Not `Var(0)`: that is the induction hypothesis, which every existing
/// witness already calls, and calling it is what leaves the argument member
/// proven as a binding and never as a call.
#[cfg(test)]
thread_local! {
    static PX8TR_CALL_SELECTED_RECURSIVE_ARGUMENT: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn set_px8tr_call_selected_recursive_argument(armed: bool) {
    PX8TR_CALL_SELECTED_RECURSIVE_ARGUMENT.with(|cell| cell.set(armed));
}

#[cfg(test)]
fn px8tr_call_selected_recursive_argument() -> bool {
    PX8TR_CALL_SELECTED_RECURSIVE_ARGUMENT.with(std::cell::Cell::get)
}

#[cfg(not(test))]
fn px8tr_call_selected_recursive_argument() -> bool {
    false
}

pub(crate) fn emit_px8tr_nested_post_effect_object(
    entry_symbol: impl Into<String>,
    disable_repair: bool,
) -> Result<Px8trNestedRouteObject, CraneliftBackendError> {
    emit_px8tr_nested_post_effect_object_with_plan(entry_symbol, disable_repair, |_| {})
}

/// **`RT-DECL-CLOSURE-PORT` `D5a`** — the same witness, with one hook: the
/// checked plan is handed to `mutate` before compilation.
///
/// ⭐ The mutation lands on the **plan**, never on the lowering. A control that
/// perturbed the consumer would be asking whether the consumer agrees with
/// itself; perturbing the plan asks the question that matters — whether the
/// consumer is really reading the checked templates it claims to.
pub(crate) fn emit_px8tr_nested_post_effect_object_with_plan(
    entry_symbol: impl Into<String>,
    disable_repair: bool,
    mutate: impl FnOnce(&mut crate::OrientedSubcontinuationPlanV1),
) -> Result<Px8trNestedRouteObject, CraneliftBackendError> {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE.set(false);
            PX8TR_TRAP_PROVENANCE.with(|trace| trace.borrow_mut().clear());
        }
    }

    let entry_symbol = entry_symbol.into();
    let (entrypoint, declaration, mut plan) = px8tr_nested_post_effect_fixture();
    mutate(&mut plan);
    let declarations = BTreeMap::from([(declaration.symbol.as_str(), &declaration)]);
    PX8TR_TRAP_PROVENANCE.with(|trace| trace.borrow_mut().clear());
    PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE.set(disable_repair);
    let _reset = Reset;
    let compiled = compile_expr_into_object_module(
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
