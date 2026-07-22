//! ISA/module setup and the private JIT/object compilation and
//! materialization machinery (RT-SPLIT §10.1 row 7).
//!
//! Slice 6 created this module as a namespace scaffold so `artifact::api`
//! could be cut as a DESCENDANT before these internals moved down. Slice 7
//! completes it: the nine private operations below arrived from the residual
//! facade, and the six transitional scaffold imports are DELETED. `api.rs`
//! reaches them through the same `use super::{…}` it already had, so that file
//! is unchanged by this slice (§10.5 — if it changed, the scaffold was wrong).
//!
//! `api` is a child of this module, so it consumes these private operations
//! with ZERO visibility widening. Production seam budget for this slice: 0.

#[cfg(test)]
mod tests;

pub(super) mod api;

use std::collections::BTreeMap;

use cranelift_codegen::isa::OwnedTargetIsa;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::{RuntimeDeclaration, RuntimeExpr, RuntimeProgram, RuntimeValue};

// Owner-named sibling imports (§10.3: `artifact -> compiled, lowering::core,
// planning, surface`). Never through the facade.
use crate::cranelift_backend::compiled::{CompiledExpr, CompiledModule};
use crate::cranelift_backend::lowering::core::compile_expr_into_module;
use crate::cranelift_backend::planning::{
    native_join_plan_for_program, oriented_subcontinuation_plan_for_program,
};
use crate::cranelift_backend::surface::{
    backend, backend_module, BackendFailure, CraneliftBackendError, NativeSeedEnvironment,
};

fn compile_expr(
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_with_declarations(expr, seed_env, BTreeMap::new())
}

fn compile_program_expr(
    program: &RuntimeProgram,
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_with_declarations(
        expr,
        seed_env,
        program
            .declarations
            .iter()
            .map(|declaration| (declaration.symbol.as_str(), declaration))
            .collect(),
    )
}

fn compile_expr_with_declarations<'a>(
    expr: &RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_with_declarations_and_process_input(expr, seed_env, declarations, None)
}

fn compile_expr_with_declarations_and_process_input<'a>(
    expr: &RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_into_module(
        new_jit_module()?,
        "ken_nc6_seed",
        Linkage::Local,
        expr,
        seed_env,
        declarations,
        staged_process_input,
        false,
        None,
        None,
        None,
    )
}

fn compile_program_expr_object(
    program: &RuntimeProgram,
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
    entry_symbol: &str,
) -> Result<CompiledModule<ObjectModule>, CraneliftBackendError> {
    compile_expr_into_module(
        new_object_module("ken-runtime-cranelift-object")?,
        entry_symbol,
        Linkage::Export,
        expr,
        seed_env,
        program
            .declarations
            .iter()
            .map(|declaration| (declaration.symbol.as_str(), declaration))
            .collect(),
        None,
        false,
        None,
        native_join_plan_for_program(program)?,
        oriented_subcontinuation_plan_for_program(program)?,
    )
}

fn native_isa() -> Result<OwnedTargetIsa, CraneliftBackendError> {
    let mut flag_builder = settings::builder();
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    flag_builder
        .set("is_pic", "true")
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    let isa_builder = cranelift_native::builder()
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    isa_builder
        .finish(settings::Flags::new(flag_builder))
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))
}

fn new_jit_module() -> Result<JITModule, CraneliftBackendError> {
    let isa = native_isa()?;
    let builder = JITBuilder::with_isa(isa, default_libcall_names());
    Ok(JITModule::new(builder))
}

fn new_object_module(name: &str) -> Result<ObjectModule, CraneliftBackendError> {
    let isa = native_isa()?;
    let builder = ObjectBuilder::new(isa, name.as_bytes().to_vec(), default_libcall_names())
        .map_err(|err| backend_module(err.to_string()))?;
    Ok(ObjectModule::new(builder))
}

fn native_platform_target_name() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

// ── Owner-adjacent test adapters (§10.5a′) ──────────────────────────────────
//
// One adapter per private operation whose TESTS cross the ownership boundary,
// each beside the private original it exposes. Zero production visibility is
// spent: every body is a single delegating call, and `#[cfg(test)]` keeps them
// out of production builds entirely (§10.2a rule 7 — the predicate is bare
// `cfg(test)`, so "absent from production" is licensed here).
//
// ⛔ THE POPULATION IS THE ITEM-AXIS CENSUS OUTPUT, NOT A FIXED CARDINALITY.
// Re-derived at `7c6e03c8`; it EXTENDS the §10.5a′ table from three artifact-
// owned adapters to five, which that clause explicitly sanctions ("a later
// item-axis census may extend the table without contradicting the rule").
// The two additions are `native_isa` and `native_platform_target_name`, whose
// crossing consumers are facade-scope `#[cfg(test)]` fixtures rather than
// lowering subject tests — a population the cf91ec5a census did not range over
// because those operations had not yet moved below the facade.
//
// The three ruled names are reproduced VERBATIM from the frame. Note that
// `new_object_module_for_lowering_tests` now serves facade fixtures too; the
// ruled name is narrower than its consumer set, and renaming a ruled row is
// not this slice's call.

#[cfg(test)]
pub(super) fn new_jit_module_for_lowering_tests() -> Result<JITModule, CraneliftBackendError> {
    new_jit_module()
}

#[cfg(test)]
pub(super) fn new_object_module_for_lowering_tests(
    name: &str,
) -> Result<ObjectModule, CraneliftBackendError> {
    new_object_module(name)
}

#[cfg(test)]
pub(super) fn compile_expr_for_lowering_tests(
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr(expr, seed_env)
}

#[cfg(test)]
pub(super) fn native_isa_for_facade_fixtures() -> Result<OwnedTargetIsa, CraneliftBackendError> {
    native_isa()
}

#[cfg(test)]
pub(super) fn native_platform_target_name_for_facade_fixtures() -> String {
    native_platform_target_name()
}
