//! Subject-partitioned tests for the lowering SCC (RT-SPLIT §10.1).
//!
//! Slice 4 populates `control`, `effects` and `constructors` -- the subjects
//! whose tests reach `lowering::core`-private items. `values.rs` is populated in slice 5 from the
//! Architect's ruled row list (`evt_3xvn8g7n5rv7m`).

// `super` here is `core`; re-exported so the leaf subject modules inherit
// the same namespace via their own `use super::*`.
pub(in crate::cranelift_backend) use super::*;

// Ruled test module: imports are permitted here (AC-8 class 2), which keeps
// these test-only names out of the production `lowering/mod.rs` namespace.
use super::super::super::test_only_distinguished_root_join_plan;

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
