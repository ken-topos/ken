//! Pre-emission planning for the Cranelift backend: native-join and
//! oriented-subcontinuation plan extraction from checked-package metadata,
//! the checked-marker census, and transport validation.
//!
//! RT-SPLIT slice 2 of 7. Pure move out of the flat `cranelift_backend`
//! module; no logic, signature, or rename changes. No CLIF emission lives
//! here. Depends only on `surface`.

use std::collections::{BTreeMap, BTreeSet};

use super::surface::{unsupported, CraneliftBackendError};
use crate::{RuntimeDeclaration, RuntimeDeclarationKind, RuntimeExpr, RuntimeProgram};

pub(super) fn native_join_plan_for_program(
    program: &RuntimeProgram,
) -> Result<Option<crate::NativeJoinPlanV1>, CraneliftBackendError> {
    let candidates = program
        .erased_core
        .metadata
        .checked_core
        .metadata
        .values()
        .filter(|bytes| bytes.starts_with(crate::NATIVE_JOIN_PLAN_V1_HEADER))
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [bytes] => crate::NativeJoinPlanV1::decode(bytes)
            .map(Some)
            .map_err(|reason| unsupported("NativeJoinPlanV1", reason)),
        _ => Err(unsupported(
            "NativeJoinPlanV1",
            "checked package contains multiple native join plans",
        )),
    }
}

pub(super) fn oriented_subcontinuation_plan_for_program(
    program: &RuntimeProgram,
) -> Result<Option<crate::OrientedSubcontinuationPlanV1>, CraneliftBackendError> {
    let candidates = program
        .erased_core
        .metadata
        .checked_core
        .metadata
        .values()
        .filter(|bytes| bytes.starts_with(crate::ORIENTED_SUBCONTINUATION_PLAN_V1_HEADER))
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [bytes] => crate::OrientedSubcontinuationPlanV1::decode(bytes)
            .map(Some)
            .map_err(|reason| unsupported("OrientedSubcontinuationPlanV1", reason)),
        _ => Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            "checked package contains multiple oriented subcontinuation plans",
        )),
    }
}

/// Coverage for the two plan extractors that bridge checked-package metadata
/// to the planning validators.
///
/// Measured before the RT-SPLIT slice-2 move: neutering either extractor to
/// return `Ok(None)` unconditionally left all 293 lib tests green. The
/// validators downstream *are* covered -- neutering
/// `validate_oriented_subcontinuation_transport`,
/// `require_exact_marker_locations`, `planned_marker_locations_for_declaration`
/// or any of the three collectors each turns the suite red -- but they are
/// exercised by tests that hand-build a plan and call them directly. Nothing
/// covered the step that *produces* that plan from a program's metadata, so
/// the metadata-to-plan wiring was unverified in both directions: a program
/// carrying a plan could be read as carrying none, and the multi-plan and
/// decode-failure rejections could stop firing, all without a red test.
///
/// The round-trip assertions are what make this more than a smoke test: each
/// extractor must return *the plan that was encoded*, not merely some plan, so
/// `Ok(None)` and "return a default" are both excluded.
#[cfg(test)]
mod plan_extraction_tests {
    use super::*;
    use crate::{ErasedExecutableCore, RuntimeMetadata};

    fn program_with_checked_metadata(entries: &[(&str, Vec<u8>)]) -> RuntimeProgram {
        let mut metadata = RuntimeMetadata::default();
        for (symbol, bytes) in entries {
            metadata
                .checked_core
                .metadata
                .insert((*symbol).to_string(), bytes.clone());
        }
        RuntimeProgram {
            package_identity: "module:fixture::planning".to_string(),
            core_semantic_hash: 1,
            artifact_hash: 2,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::new(),
                metadata,
            },
            declarations: Vec::new(),
            examples: Vec::new(),
        }
    }

    fn native_join_plan() -> crate::NativeJoinPlanV1 {
        let site_id = 7;
        let declaration = "decl:fixture::Main::main".to_string();
        let checked_occurrence_path = vec![1, 2];
        let checked_result_type_fingerprint = 11;
        crate::NativeJoinPlanV1 {
            representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
            sites: vec![crate::NativeJoinPlanSiteV1 {
                site_id,
                declaration: declaration.clone(),
                checked_occurrence_path: checked_occurrence_path.clone(),
                checked_result_type_fingerprint,
                // Derived, not a literal: `decode` rejects a site whose
                // binding fingerprint is not exactly this function of the
                // other four fields, so a hand-picked constant makes the
                // fixture undecodable rather than merely unrealistic.
                occurrence_binding_fingerprint:
                    crate::compiler_private_join_occurrence_binding_fingerprint(
                        site_id,
                        &declaration,
                        &checked_occurrence_path,
                        checked_result_type_fingerprint,
                    ),
                runtime_frame_fingerprint: 17,
                answer_kind: crate::NativeJoinAnswerKindV1::Int,
            }],
        }
    }

    fn oriented_plan() -> crate::OrientedSubcontinuationPlanV1 {
        crate::OrientedSubcontinuationPlanV1 {
            representation_rule_version:
                crate::OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: Vec::new(),
            recursive_calls: Vec::new(),
            computational_ih_slots: Vec::new(),
            computational_ih_calls: Vec::new(),
        }
    }

    #[test]
    fn native_join_plan_absent_when_no_metadata_carries_the_header() {
        // The discriminating half: unrelated metadata must not be mistaken for
        // a plan, so this is not vacuously None.
        let program = program_with_checked_metadata(&[(
            "decl:fixture::Other",
            b"SomeOtherMetadataV1\0payload".to_vec(),
        )]);
        assert_eq!(native_join_plan_for_program(&program).unwrap(), None);
    }

    #[test]
    fn native_join_plan_round_trips_the_encoded_plan() {
        let plan = native_join_plan();
        let program =
            program_with_checked_metadata(&[("decl:fixture::Main", plan.canonical_bytes())]);
        assert_eq!(
            native_join_plan_for_program(&program).unwrap(),
            Some(plan),
            "the extractor must return the plan that was encoded, not merely some plan"
        );
    }

    #[test]
    fn native_join_plan_rejects_two_plans_in_one_package() {
        let bytes = native_join_plan().canonical_bytes();
        let program = program_with_checked_metadata(&[
            ("decl:fixture::A", bytes.clone()),
            ("decl:fixture::B", bytes),
        ]);
        let err = native_join_plan_for_program(&program).unwrap_err();
        assert_eq!(
            err,
            unsupported(
                "NativeJoinPlanV1",
                "checked package contains multiple native join plans"
            )
        );
    }

    #[test]
    fn native_join_plan_surfaces_a_decode_failure_rather_than_dropping_it() {
        // Header present so the entry is selected, payload truncated so decode
        // must fail: this is the arm that distinguishes "no plan" from
        // "unreadable plan", and conflating them would silently disable native
        // join lowering.
        let mut bytes = crate::NATIVE_JOIN_PLAN_V1_HEADER.to_vec();
        bytes.extend_from_slice(&[0x00, 0x01]);
        let program = program_with_checked_metadata(&[("decl:fixture::Main", bytes)]);
        let err = native_join_plan_for_program(&program).unwrap_err();
        assert!(
            matches!(&err, CraneliftBackendError::Unsupported(u) if u.construct == "NativeJoinPlanV1"),
            "expected an Unsupported(NativeJoinPlanV1) decode failure, got {err:?}"
        );
    }

    #[test]
    fn oriented_plan_absent_when_no_metadata_carries_the_header() {
        let program = program_with_checked_metadata(&[(
            "decl:fixture::Other",
            b"SomeOtherMetadataV1\0payload".to_vec(),
        )]);
        assert_eq!(
            oriented_subcontinuation_plan_for_program(&program).unwrap(),
            None
        );
    }

    #[test]
    fn oriented_plan_round_trips_the_encoded_plan() {
        let plan = oriented_plan();
        let program =
            program_with_checked_metadata(&[("decl:fixture::Main", plan.canonical_bytes())]);
        assert_eq!(
            oriented_subcontinuation_plan_for_program(&program).unwrap(),
            Some(plan),
            "the extractor must return the plan that was encoded, not merely some plan"
        );
    }

    #[test]
    fn oriented_plan_rejects_two_plans_in_one_package() {
        let bytes = oriented_plan().canonical_bytes();
        let program = program_with_checked_metadata(&[
            ("decl:fixture::A", bytes.clone()),
            ("decl:fixture::B", bytes),
        ]);
        let err = oriented_subcontinuation_plan_for_program(&program).unwrap_err();
        assert_eq!(
            err,
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked package contains multiple oriented subcontinuation plans"
            )
        );
    }

    #[test]
    fn oriented_plan_surfaces_a_decode_failure_rather_than_dropping_it() {
        let mut bytes = crate::ORIENTED_SUBCONTINUATION_PLAN_V1_HEADER.to_vec();
        bytes.extend_from_slice(&[0x00, 0x01]);
        let program = program_with_checked_metadata(&[("decl:fixture::Main", bytes)]);
        let err = oriented_subcontinuation_plan_for_program(&program).unwrap_err();
        assert!(
            matches!(&err, CraneliftBackendError::Unsupported(u) if u.construct == "OrientedSubcontinuationPlanV1"),
            "expected an Unsupported(OrientedSubcontinuationPlanV1) decode failure, got {err:?}"
        );
    }
}

pub(super) fn collect_checked_subcontinuation_frames(
    expr: &RuntimeExpr,
    frames: &mut BTreeMap<u64, u64>,
) -> Result<(), CraneliftBackendError> {
    match expr {
        RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
            let RuntimeExpr::ComputationalMatch { cases, default, .. } = body.as_ref() else {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "checked subcontinuation marker does not wrap a computational frame",
                ));
            };
            let fingerprint =
                crate::compiler_private_computational_match_frame_fingerprint(cases, default);
            if frames.insert(*frame_id, fingerprint).is_some() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "Runtime IR repeats a checked subcontinuation frame marker",
                ));
            }
            collect_checked_subcontinuation_frames(body, frames)
        }
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. }
        | RuntimeExpr::Project { record: body, .. }
        | RuntimeExpr::Closure { body, .. } => collect_checked_subcontinuation_frames(body, frames),
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            for capture in captures {
                collect_checked_subcontinuation_frames(capture, frames)?;
            }
            collect_checked_subcontinuation_frames(body, frames)
        }
        RuntimeExpr::Let { value, body } => {
            collect_checked_subcontinuation_frames(value, frames)?;
            collect_checked_subcontinuation_frames(body, frames)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            collect_checked_subcontinuation_frames(scrutinee, frames)?;
            collect_checked_subcontinuation_frames(then_expr, frames)?;
            collect_checked_subcontinuation_frames(else_expr, frames)
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for arg in args {
                collect_checked_subcontinuation_frames(arg, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            collect_checked_subcontinuation_frames(scrutinee, frames)?;
            for case in cases {
                collect_checked_subcontinuation_frames(&case.body, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            collect_checked_subcontinuation_frames(scrutinee, frames)?;
            for case in cases {
                collect_checked_subcontinuation_frames(&case.body, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::Record { fields } => {
            for (_, value) in fields {
                collect_checked_subcontinuation_frames(value, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::Call { callee, args } => {
            collect_checked_subcontinuation_frames(callee, frames)?;
            for arg in args {
                collect_checked_subcontinuation_frames(arg, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability {
                collect_checked_subcontinuation_frames(&capability.value, frames)?;
            }
            for arg in args {
                collect_checked_subcontinuation_frames(arg, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => Ok(()),
    }
}

#[derive(Default)]
pub(super) struct CheckedOrientedMarkerSets {
    pub(super) recursive_calls: BTreeMap<(u64, Vec<u64>), BTreeSet<Vec<u64>>>,
    pub(super) computational_ih_slots: BTreeMap<(u64, Vec<u64>), BTreeSet<Vec<u64>>>,
    pub(super) computational_ih_calls: BTreeMap<(u64, Vec<u64>), BTreeSet<Vec<u64>>>,
}

impl CheckedOrientedMarkerSets {
    fn extend_from(&mut self, other: &Self) {
        for (key, paths) in &other.recursive_calls {
            self.recursive_calls
                .entry(key.clone())
                .or_default()
                .extend(paths.iter().cloned());
        }
        for (key, paths) in &other.computational_ih_slots {
            self.computational_ih_slots
                .entry(key.clone())
                .or_default()
                .extend(paths.iter().cloned());
        }
        for (key, paths) in &other.computational_ih_calls {
            self.computational_ih_calls
                .entry(key.clone())
                .or_default()
                .extend(paths.iter().cloned());
        }
    }
}

pub(super) fn collect_checked_oriented_markers(
    expr: &RuntimeExpr,
    markers: &mut CheckedOrientedMarkerSets,
    root: &str,
    runtime_path: &mut Vec<u64>,
) -> Result<(), CraneliftBackendError> {
    match expr {
        RuntimeExpr::CheckedRecursiveInvocation {
            call_template_id,
            checked_occurrence_path,
            body,
        } => {
            if !markers
                .recursive_calls
                .entry((*call_template_id, checked_occurrence_path.clone()))
                .or_default()
                .insert(runtime_path.clone())
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    format!(
                        "Runtime IR root {root} repeats checked recursive-call marker {call_template_id} at the same structural path {runtime_path:?}"
                    ),
                ));
            }
            collect_checked_oriented_child(body, markers, root, runtime_path, 0)
        }
        RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids,
            checked_occurrence_paths,
            body,
        } => {
            if slot_template_ids.len() != checked_occurrence_paths.len() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "computational IH slot marker identity/location arity differs",
                ));
            }
            for (slot_template_id, checked_occurrence_path) in
                slot_template_ids.iter().zip(checked_occurrence_paths)
            {
                if !markers
                    .computational_ih_slots
                    .entry((*slot_template_id, checked_occurrence_path.clone()))
                    .or_default()
                    .insert(runtime_path.clone())
                {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        format!(
                            "Runtime IR root {root} repeats checked computational-IH slot marker {slot_template_id} at the same structural path {runtime_path:?}"
                        ),
                    ));
                }
            }
            collect_checked_oriented_child(body, markers, root, runtime_path, 0)
        }
        RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id,
            checked_occurrence_path,
            body,
        } => {
            if !markers
                .computational_ih_calls
                .entry((*call_template_id, checked_occurrence_path.clone()))
                .or_default()
                .insert(runtime_path.clone())
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    format!(
                        "Runtime IR root {root} repeats checked computational-IH call marker {call_template_id} at the same structural path {runtime_path:?}"
                    ),
                ));
            }
            collect_checked_oriented_child(body, markers, root, runtime_path, 0)
        }
        RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedJoinSite { body, .. } => {
            collect_checked_oriented_child(body, markers, root, runtime_path, 0)
        }
        RuntimeExpr::Project { record, .. } => {
            collect_checked_oriented_child(record, markers, root, runtime_path, 1)
        }
        RuntimeExpr::Closure { body, .. } => {
            collect_checked_oriented_child(body, markers, root, runtime_path, 2)
        }
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            for (index, capture) in captures.iter().enumerate() {
                collect_checked_oriented_child(
                    capture,
                    markers,
                    root,
                    runtime_path,
                    10 + index as u64,
                )?;
            }
            collect_checked_oriented_child(body, markers, root, runtime_path, 3)
        }
        RuntimeExpr::Let { value, body } => {
            collect_checked_oriented_child(value, markers, root, runtime_path, 0)?;
            collect_checked_oriented_child(body, markers, root, runtime_path, 1)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            collect_checked_oriented_child(scrutinee, markers, root, runtime_path, 0)?;
            collect_checked_oriented_child(then_expr, markers, root, runtime_path, 1)?;
            collect_checked_oriented_child(else_expr, markers, root, runtime_path, 2)
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for (index, arg) in args.iter().enumerate() {
                collect_checked_oriented_child(arg, markers, root, runtime_path, index as u64)?;
            }
            Ok(())
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            collect_checked_oriented_child(scrutinee, markers, root, runtime_path, 0)?;
            for (index, case) in cases.iter().enumerate() {
                collect_checked_oriented_child(
                    &case.body,
                    markers,
                    root,
                    runtime_path,
                    1 + index as u64,
                )?;
            }
            Ok(())
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            collect_checked_oriented_child(scrutinee, markers, root, runtime_path, 0)?;
            for (index, case) in cases.iter().enumerate() {
                collect_checked_oriented_child(
                    &case.body,
                    markers,
                    root,
                    runtime_path,
                    1 + index as u64,
                )?;
            }
            Ok(())
        }
        RuntimeExpr::Record { fields } => {
            for (index, (_, value)) in fields.iter().enumerate() {
                collect_checked_oriented_child(value, markers, root, runtime_path, index as u64)?;
            }
            Ok(())
        }
        RuntimeExpr::Call { callee, args } => {
            collect_checked_oriented_child(callee, markers, root, runtime_path, 0)?;
            for (index, arg) in args.iter().enumerate() {
                collect_checked_oriented_child(arg, markers, root, runtime_path, 1 + index as u64)?;
            }
            Ok(())
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability {
                collect_checked_oriented_child(&capability.value, markers, root, runtime_path, 0)?;
            }
            for (index, arg) in args.iter().enumerate() {
                collect_checked_oriented_child(arg, markers, root, runtime_path, 1 + index as u64)?;
            }
            Ok(())
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => Ok(()),
    }
}

fn collect_checked_oriented_child(
    expr: &RuntimeExpr,
    markers: &mut CheckedOrientedMarkerSets,
    root: &str,
    runtime_path: &mut Vec<u64>,
    edge: u64,
) -> Result<(), CraneliftBackendError> {
    runtime_path.push(edge);
    let result = collect_checked_oriented_markers(expr, markers, root, runtime_path);
    runtime_path.pop();
    result
}

fn planned_marker_locations_for_declaration(
    plan: &crate::OrientedSubcontinuationPlanV1,
    declaration: &str,
) -> CheckedOrientedMarkerSets {
    let mut expected = CheckedOrientedMarkerSets::default();
    for call in &plan.recursive_calls {
        if call.declaration == declaration {
            expected.recursive_calls.insert(
                (call.call_template_id, call.checked_occurrence_path.clone()),
                call.runtime_marker_locations
                    .iter()
                    .map(|location| location.runtime_path.clone())
                    .collect(),
            );
        }
    }
    for slot in &plan.computational_ih_slots {
        if slot.declaration == declaration {
            expected.computational_ih_slots.insert(
                (slot.slot_template_id, slot.checked_occurrence_path.clone()),
                slot.runtime_marker_locations
                    .iter()
                    .map(|location| location.runtime_path.clone())
                    .collect(),
            );
        }
    }
    for call in &plan.computational_ih_calls {
        if call.declaration == declaration {
            expected.computational_ih_calls.insert(
                (call.call_template_id, call.checked_occurrence_path.clone()),
                call.runtime_marker_locations
                    .iter()
                    .map(|location| location.runtime_path.clone())
                    .collect(),
            );
        }
    }
    expected
}

fn require_exact_marker_locations(
    declaration: &str,
    actual: &CheckedOrientedMarkerSets,
    expected: &CheckedOrientedMarkerSets,
) -> Result<(), CraneliftBackendError> {
    if actual.recursive_calls != expected.recursive_calls {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "checked recursive-call Runtime occurrences differ in declaration {declaration}"
            ),
        ));
    }
    if actual.computational_ih_slots != expected.computational_ih_slots {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "checked computational-IH slot Runtime occurrences differ in declaration {declaration}"
            ),
        ));
    }
    if actual.computational_ih_calls != expected.computational_ih_calls {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "checked computational-IH call Runtime occurrences differ in declaration {declaration}"
            ),
        ));
    }
    Ok(())
}

pub(super) fn validate_oriented_subcontinuation_transport(
    expr: &RuntimeExpr,
    declarations: &BTreeMap<&str, &RuntimeDeclaration>,
    plan: Option<&crate::OrientedSubcontinuationPlanV1>,
) -> Result<(), CraneliftBackendError> {
    let mut markers = BTreeMap::new();
    let mut entry_nonframe_markers = CheckedOrientedMarkerSets::default();
    let mut nonframe_markers = CheckedOrientedMarkerSets::default();
    let mut declaration_nonframe_markers = Vec::new();
    collect_checked_subcontinuation_frames(expr, &mut markers)?;
    collect_checked_oriented_markers(
        expr,
        &mut entry_nonframe_markers,
        "<entry>",
        &mut Vec::new(),
    )?;
    nonframe_markers.extend_from(&entry_nonframe_markers);
    for (symbol, declaration) in declarations.iter() {
        if let RuntimeDeclarationKind::Transparent { body } = &declaration.kind {
            collect_checked_subcontinuation_frames(body, &mut markers)?;
            let mut declaration_markers = CheckedOrientedMarkerSets::default();
            collect_checked_oriented_markers(
                body,
                &mut declaration_markers,
                symbol,
                &mut Vec::new(),
            )?;
            nonframe_markers.extend_from(&declaration_markers);
            declaration_nonframe_markers.push((*symbol, declaration_markers));
        }
    }
    let markers_are_empty = markers.is_empty()
        && nonframe_markers.recursive_calls.is_empty()
        && nonframe_markers.computational_ih_slots.is_empty()
        && nonframe_markers.computational_ih_calls.is_empty();
    match (markers_are_empty, plan) {
        (true, None) => return Ok(()),
        (false, None) => {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked subcontinuation markers have no checked plan metadata",
            ));
        }
        (true, Some(plan))
            if plan.frames.is_empty()
                && plan.recursive_calls.is_empty()
                && plan.computational_ih_slots.is_empty()
                && plan.computational_ih_calls.is_empty() =>
        {
            return Ok(())
        }
        (true, Some(_)) => {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked plan has no Runtime frame markers",
            ));
        }
        (false, Some(_)) => {}
    }
    let plan = plan.expect("nonempty marker set has a plan");
    plan.validate()
        .map_err(|reason| unsupported("OrientedSubcontinuationPlanV1", reason))?;
    if !entry_nonframe_markers.recursive_calls.is_empty()
        || !entry_nonframe_markers.computational_ih_slots.is_empty()
        || !entry_nonframe_markers.computational_ih_calls.is_empty()
    {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            "checked recursive/IH marker escaped its declaration into the entry expression",
        ));
    }
    for (declaration, markers) in &declaration_nonframe_markers {
        let expected = planned_marker_locations_for_declaration(plan, declaration);
        require_exact_marker_locations(declaration, markers, &expected)?;
    }
    if markers.len() != plan.frames.len() {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            "checked plan and Runtime marker sets differ",
        ));
    }
    for frame in &plan.frames {
        let Some(fingerprint) = markers.remove(&frame.frame_id) else {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked plan frame marker is missing or transplanted",
            ));
        };
        if fingerprint != frame.runtime_frame_fingerprint {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked plan frame fingerprint is stale",
            ));
        }
    }
    if !markers.is_empty() {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            "Runtime frame marker has no checked plan entry",
        ));
    }
    let plan_recursive_calls = plan
        .recursive_calls
        .iter()
        .map(|call| (call.call_template_id, call.checked_occurrence_path.clone()))
        .collect::<BTreeSet<_>>();
    let plan_ih_slots = plan
        .computational_ih_slots
        .iter()
        .map(|slot| (slot.slot_template_id, slot.checked_occurrence_path.clone()))
        .collect::<BTreeSet<_>>();
    let plan_ih_calls = plan
        .computational_ih_calls
        .iter()
        .map(|call| (call.call_template_id, call.checked_occurrence_path.clone()))
        .collect::<BTreeSet<_>>();
    let runtime_recursive_calls = nonframe_markers
        .recursive_calls
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let runtime_ih_slots = nonframe_markers
        .computational_ih_slots
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let runtime_ih_calls = nonframe_markers
        .computational_ih_calls
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    if runtime_recursive_calls != plan_recursive_calls
        || runtime_ih_slots != plan_ih_slots
        || runtime_ih_calls != plan_ih_calls
    {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            "checked plan and Runtime recursive/IH marker sets differ",
        ));
    }
    Ok(())
}
