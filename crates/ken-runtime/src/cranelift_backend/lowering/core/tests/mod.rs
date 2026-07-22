//! Subject-partitioned tests for the lowering SCC (RT-SPLIT §10.1).
//!
//! Slice 4 populates `control`, `effects` and `constructors` -- the subjects
//! whose tests reach `lowering::core`-private items. `values.rs` is populated
//! in slice 5 from the Architect's ruled row list (`evt_3xvn8g7n5rv7m`).

// `super` here is `core`; re-exported so the leaf subject modules inherit
// the same namespace via their own `use super::*`.
pub(in crate::cranelift_backend) use super::*;

// Ruled test module: imports are permitted here (AC-8 class 2). The subject
// modules reach these through this module rather than inheriting them from a
// production glob -- `lowering/mod.rs` must not import through the facade
// (§10.3), so the test subtree names its own fixture dependencies explicitly.
//
// Facade-owned test fixtures. `test_only_distinguished_root_join_plan`,
// `NativeInvocationFixture` and `BorrowedFixtureValue` are the ruled
// facade-LCA fixtures that stay at residual-parent file scope until slice 7
// (§10.2a rule 6); the rest are shared helpers whose final users span this
// subtree and the facade's residual artifact/api tests.
pub(in crate::cranelift_backend) use super::super::super::{
    big, compile_expr, constructor_field_aggregate, emit_process_entrypoint_object_with_cranelift,
    emit_process_entrypoint_object_with_symbols, host_result_closure_match,
    host_result_computational_fixture, new_jit_module, new_object_module, ordinary_match_closure,
    oriented_test_frame, oriented_test_interface, px8n_exact_nat, px8n_failure,
    px8n_write_arm_fixture_with_start, recursive_computational_result_depth,
    run_example_with_seed_observation, run_px8n_arm_fixture, run_px8n_write_arm_fixture,
    self_consistent_join_site, self_consistent_root_join_site, total_primitive,
    BorrowedFixtureValue, NativeInvocationFixture, Px8nHostReplyFixture, PX8I_BIG_READ_START,
    PX8I_BIG_U64, PX8I_METADATA_BIG, PX8I_WRAPPING_WRITE_START, PX8N_OVER_BOUND_READ,
    PX8N_OVER_BOUND_WRITE, PX8N_READ_EOF, PX8N_SHORT_READ, PX8N_SHORT_WROTE, PX8N_ZERO_WRITE,
};

// Crate-root items the subject tests assert against.
pub(in crate::cranelift_backend) use crate::{
    CraneliftObjectArtifact, NativeFidelity, RuntimeExample, RuntimeLowerabilityStatus,
    RuntimeObservation, UnsupportedLowering,
};

// Ruled test module: a `use` is permitted here (AC-8 class 2).
pub(in crate::cranelift_backend) use crate::cranelift_backend::test_support::test_only_distinguished_root_join_plan;

mod constructors;
mod control;
mod effects;
mod values;

// Shared by >1 subject module: §10.2 places a helper at the lowest
// tests/mod.rs ancestor shared by its actual users.
fn console_write_effect() -> RuntimeExpr {
    RuntimeExpr::Effect {
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
    }
}
fn recursive_computational_result(leaf_body: RuntimeExpr) -> RuntimeExpr {
    recursive_computational_result_depth(0, leaf_body)
}
