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
// RT-SPLIT slice 7, rule 8: this list was 31 names. The residual facade
// fixtures whose final-user LCA is this subtree have MOVED here (see the
// 37-item ledger), so they are now local declarations rather than imports.
// What remains is exactly the fixtures whose final-user LCA genuinely IS the
// facade, and which therefore have no lawful lower home:
//
//   emit_process_entrypoint_object_with_cranelift  -- also consumed by
//       `object_linker_packaging.rs:659`, ABOVE the facade, and its exact
//       declaration text is pinned by a cross-crate oracle in
//       `ken-cli/tests/px4b_native_production.rs:752`
//   total_primitive  -- users span `artifact/api/tests.rs` and this subtree
//
// `run_example_with_seed_observation` is a facade re-export of an
// `artifact::api` entrypoint, not one of the 37 residual fixtures.
pub(in crate::cranelift_backend) use super::super::super::{
    emit_process_entrypoint_object_with_cranelift, run_example_with_seed_observation,
};

// Crate-root items the subject tests assert against.
pub(in crate::cranelift_backend) use crate::{
    CraneliftObjectArtifact, NativeFidelity, RuntimeExample, RuntimeLowerabilityStatus,
    RuntimeObservation, UnsupportedLowering,
};

// Ruled test module: a `use` is permitted here (AC-8 class 2).
pub(in crate::cranelift_backend) use crate::cranelift_backend::test_support::{
    test_only_distinguished_root_join_plan, total_primitive,
};

// RT-SPLIT slice 7 (§10.5a′) — the three artifact privates these subject tests
// reach across the ownership boundary now arrive through owner-adjacent
// adapters instead of the facade, because the originals moved down into
// `artifact` and a sibling subtree cannot see them. Aliasing back to the
// original names keeps every leaf-test call token unchanged, so this is an
// IMPORT-ONLY edit: no subject test body changes and no production item is
// widened. §10.2 places these tests by the behavior they DISCRIMINATE, which
// is lowering — a setup callee living in `artifact` does not reassign them.
pub(in crate::cranelift_backend) use crate::cranelift_backend::artifact::{
    compile_expr_for_lowering_tests as compile_expr,
    new_jit_module_for_lowering_tests as new_jit_module,
    new_object_module_for_lowering_tests as new_object_module,
};

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
// RT-SPLIT slice 7, rule 8 correction: this module's OWN code calls
// `recursive_computational_result_depth`, so its final-user LCA is HERE, not
// `control.rs`. My first ledger pass omitted `tests/mod.rs` from the user
// scan -- a parent cannot see a child's privates, so placing it in `control`
// broke this caller. One row, same window defect as the other two.
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

fn recursive_computational_result(leaf_body: RuntimeExpr) -> RuntimeExpr {
    recursive_computational_result_depth(0, leaf_body)
}

// ── RT-SPLIT slice 7, rule 8 finalization ─────────────────────────────────
// Residual facade test fixtures whose final-user LCA is this module. Facade
// file scope was a TRANSITIONAL zero-widening holding position, never final
// ownership (Architect `evt_h69xwchqqxmj`); slice 7 discharges it. Moved
// verbatim -- ordered item-level identity, no body edits.

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
fn big(sign: crate::Sign, limbs: &[u64]) -> RuntimeExpr {
    RuntimeExpr::Value(RuntimeValue::Int(crate::RuntimeIntV1::Big {
        sign,
        limbs: limbs.to_vec(),
    }))
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
