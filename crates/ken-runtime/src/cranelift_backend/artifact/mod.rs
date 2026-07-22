//! `artifact` namespace scaffold (RT-SPLIT §10.5, slice 6).
//!
//! Slice 6 creates this module so `artifact::api` can be cut as a DESCENDANT
//! before the artifact internals move down in slice 7. Only `api` lives here
//! today.
//!
//! ⛔ The six imports below are a TRANSITIONAL SCAFFOLD, not ownership: these
//! operations still live in the residual parent and move into this module in
//! slice 7, at which point these imports are DELETED and `api.rs` is unchanged.
//! They are a second scaffold-import population in the sense of AC-9 — slice 5's
//! reconciliation numbers do not apply to them.

pub(super) mod api;

pub(in crate::cranelift_backend) use super::{
    compile_expr, compile_expr_with_declarations_and_process_input, compile_program_expr,
    compile_program_expr_object, native_platform_target_name, new_object_module,
};
