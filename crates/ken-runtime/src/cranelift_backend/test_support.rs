//! Facade-level shared test fixtures (RT-SPLIT §10.2a rule 6).
//!
//! Seeded in slice 6 with the one grounded facade-LCA fixture whose users span
//! `lowering` and `artifact::api`; `NativeInvocationFixture` and
//! `BorrowedFixtureValue` join it in slice 7 when the artifact tests move.
//! An accumulating namespace scaffold, not a production module.
//!
//! ⛔ AC-8: no production consumer. Outside ruled `tests` modules every use
//! must be a DIRECT ROOTED `crate::cranelift_backend::test_support::<item>` at
//! the semantic use site, inside an entire item carrying item-level
//! `#[cfg(test)]` — no `use`, alias, or re-export may launder a name out of
//! here. Contains no `#[test]` cases, assertions, subject tests, production
//! policy, or owner-adjacent boundary adapters.

pub(super) fn test_only_distinguished_root_join_plan() -> crate::NativeJoinPlanV1 {
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
