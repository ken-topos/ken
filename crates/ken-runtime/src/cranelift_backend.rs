//! Cranelift backend for the runtime IR seed.
//!
//! This module deliberately keeps the native boundary narrow. Cranelift code
//! returns scalar `i64` values directly and aggregate observations through an
//! opaque token table decoded by this Rust layer. Native addresses, object
//! layout, allocation order, ABI details, and Cranelift internals never become
//! Ken-observable meaning.
//!
//! This file holds no test material — that lives in the two gated child
//! modules below. What remains is facade surface (module declarations and
//! re-exports) plus the two inherent `impl` blocks at the foot of the file,
//! which are production code on `surface`'s own types and are reached from
//! `artifact::api`.

use std::collections::{BTreeMap, BTreeSet};

use crate::RuntimeProgram;

pub(crate) mod artifact;
pub(crate) mod compiled;
mod lowering;
pub(crate) mod planning;
pub(crate) mod surface;

#[cfg(test)]
mod test_objects;
#[cfg(test)]
mod test_support;

// The facade preserves the exact pre-existing `ken_runtime::<name>` surface.
// Re-exporting an already-exported name at its established visibility does not
// widen it.
pub(crate) use artifact::api::{
    emit_bound_process_program_object_with_cranelift, run_process_expr_with_cranelift,
};
pub use artifact::api::{
    emit_runtime_ir_object_with_cranelift, reject_program_blockers,
    run_example_with_interpreter_observation, run_example_with_seed_observation,
    run_ken_checked_proof_erasure_example_with_interpreter_observation, run_nc6_seed_examples,
    run_nc8_validated_seed_examples, run_runtime_ir_report_with_cranelift,
    run_validated_example_with_interpreter_observation,
};

// `with_px8ds_retired_flat_order` is bare `pub` and reached cross-CRATE as
// `ken_runtime::with_px8ds_retired_flat_order` through `lib.rs:39`. Moving it
// into the private `lowering` module severs that path, and neither
// `-p ken-runtime` build config can observe the break — only the consumer can.
#[cfg(feature = "px8-ds-test-support")]
pub use lowering::with_px8ds_retired_flat_order;

// ⛔ This list is DERIVED, not authored: it is exactly the set of module-level
// bare-`pub` items in `surface.rs`, enumerated mechanically and checked for set
// equality in BOTH directions. A name dropped here vanishes from
// `ken_runtime::*` via `lib.rs:39` and can still compile green across the whole
// workspace — most of these names have no in-repo consumer at all, so the
// compiler is not a net for this edit. `NativeSeedEnvironment::{empty,
// nc5_seed, insert}` are impl METHODS, not module items, and correctly do not
// appear.
//
// `backend_module` is listed separately because a glob re-exports
// restricted-visibility items too: it is `pub(crate)` in `surface`, so
// enumerating only bare-`pub` declarations silently drops it.
// `native_int_clif.rs:14`, a SIBLING of this module, imports it as
// `crate::cranelift_backend::backend_module` — the compiler caught that one
// only because it happens to have an in-crate consumer, which is luck, not a
// net.
pub(crate) use surface::backend_module;
pub use surface::{
    BackendFailure, CraneliftBackendError, CraneliftObjectArtifact, CraneliftRunReport,
    InterpreterOracleObservation, NativeArtifactIdentity, NativeDifferentialReport,
    NativeDifferentialStage, NativeDifferentialVerdict, NativeEvidenceFact, NativeFidelity,
    NativeRunEvidence, NativeRuntimeIrComparisonReport, NativeRuntimeIrComparisonVerdict,
    NativeSeedEnvironment, NativeToolchainReport, NativeTrustReport, UnsupportedLowering,
    ValidatedNativeRunError,
};

// Test-only facade surface. These re-exports carry names to consumers OUTSIDE
// this module — `object_linker_packaging`, a crate-root sibling, reaches every
// one of them — so they are surface, not test content, and they cannot move
// into the gated child modules that declare them.
//
// ⛔ The `#[cfg(test)]` gate is deliberate, not an artifact to clean up. The
// build asymmetry here is bidirectional: an UNGATED `use` warns as unused in
// the library build, and deleting it breaks the test build. Neither
// configuration's diagnostics are authority on their own, so the decision is
// taken over the INTERSECTION of both — gate rather than choose a side.
#[cfg(test)]
pub(crate) use lowering::{
    NativeIntLoweringMutation, Px8trTrapProvenanceEvent, NATIVE_INT_LOWERING_MUTATION,
};
#[cfg(test)]
pub(crate) use test_objects::{
    emit_process_entrypoint_object_with_cranelift, emit_px8tr_nested_post_effect_object,
};
// `Px8trNestedRouteObject` was nameable at `crate::cranelift_backend::
// Px8trNestedRouteObject` before the move. Spelling the declaration
// `pub(crate)` at its new home does NOT preserve that reach, because the module
// holding it is private — the type surface would narrow even though the
// visibility keyword is unchanged. This re-export preserves the pre-existing
// path. It is kept separate from the block above so the `allow` covers only
// this one intentionally path-preserving import: the absence of a named
// consumer today is evidence that nothing currently reaches it, not authority
// to remove the way anything could.
#[cfg(test)]
#[allow(unused_imports)] // Preserve the pre-existing nameable crate-private type path.
pub(crate) use test_objects::Px8trNestedRouteObject;

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
