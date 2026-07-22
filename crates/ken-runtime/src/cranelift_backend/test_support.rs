//! Facade-level shared test fixtures (RT-SPLIT §10.2a rule 6).
//!
//! Holds the facade-LCA fixtures the rule-8 whole-residual census emits —
//! currently TWO. A fixture lives here when it is a genuine fixture/setup
//! helper (it constructs shared state, rather than delegating once to a
//! private production operation) AND its final direct users span two or more
//! ruled subject-test subtrees, so its LCA really is the facade:
//!
//!   test_only_distinguished_root_join_plan  — `lowering` + `artifact::api`
//!   total_primitive                         — `artifact/api/tests.rs` + `lowering`
//!
//! ⛔ The membership is the CENSUS OUTPUT, not a fixed count. An earlier
//! "exactly ONE" reading was the output of the narrower grounded-fixture
//! census and was falsified by the whole-residual ledger — the third
//! enumeration in this clause family to go stale that way. State the rule;
//! let the census emit the members.
//!
//! `NativeInvocationFixture` and `BorrowedFixtureValue` are NOT here: their
//! measured direct users are lowering/effects-only, so they are effects-owned
//! (Architect `evt_h69xwchqqxmj`).
//!
//! ⛔ AC-8: no production consumer. Outside ruled `tests` modules every use
//! must be a DIRECT ROOTED `crate::cranelift_backend::test_support::<item>` at
//! the semantic use site, inside an entire item carrying item-level
//! `#[cfg(test)]` — no `use`, alias, or re-export may launder a name out of
//! here. Contains no `#[test]` cases, assertions, subject tests, production
//! policy, or owner-adjacent boundary adapters.

use crate::{RuntimeExpr, RuntimePartiality, RuntimePrimitive};

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

// RT-SPLIT slice 7, rule 8 whole-residual fold (Architect `evt_1s7nxrjje35tk`,
// overturning my provisional facade-file placement). Genuine fixture/setup
// helper -- it constructs a shared `RuntimeExpr::PrimitiveCall` value rather
// than delegating once to a private production operation -- whose final users
// span `artifact/api/tests.rs` and the lowering subtree. That is a real facade
// LCA, so `test_support.rs` is its lawful home under §10.2a rule 2.
pub(super) fn total_primitive(symbol: &str, args: Vec<RuntimeExpr>) -> RuntimeExpr {
    RuntimeExpr::PrimitiveCall {
        primitive: RuntimePrimitive {
            symbol: symbol.to_string(),
            partiality: RuntimePartiality::Total,
        },
        args,
    }
}
