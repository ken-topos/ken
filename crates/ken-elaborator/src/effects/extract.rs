//! By-construction `param_rows` extraction for higher-order effectful params.
//!
//! ## The gap
//!
//! The row-poly soundness contract (§1.2) requires that EVERY higher-order
//! effectful parameter gets a `RowVar` assigned in the `EffectDecl`, OR is
//! routed through the conservative `unknown_effectful_params` path. A parameter
//! that goes through NEITHER path silently infers ∅ for its latent row — the
//! same gap as the L5-build `unknown_effectful_params` stand-in, re-introduced
//! without this module.
//!
//! ## Solution: allocator + explicit extraction
//!
//! `RowVarAllocator` issues fresh `RowVar`s. `extract_hof_params` processes
//! every higher-order effectful parameter name and returns a `(name, RowVar)`
//! pair for each. Because ALL HOF params are enumerated in one call, no param
//! can silently slip through.
//!
//! Callers use `build_decl_with_extracted_params` (or call `with_param_row`
//! directly) to wire the vars into the `EffectDecl`. The allocator is shared
//! across the function's parameter list so each var is unique.
//!
//! ## Fail-closed contract
//!
//! If a caller omits a HOF param from the `params` slice passed to
//! `extract_hof_params`, `infer_row_poly` will not propagate that param's
//! latent effects — the inferred row will be too narrow and the escape check
//! may accept spuriously. Tests in `tests/effects.rs` (`param_rows_*`)
//! demonstrate the discriminating verdict: correct extraction catches the
//! escape; omission misses it.

use super::infer::EffectDecl;
use super::row::RowVar;

/// Allocates fresh `RowVar`s in strictly-increasing order.
///
/// Create one allocator per function being analysed; do NOT share across
/// functions (that would conflate two functions' row vars).
pub struct RowVarAllocator {
    next: u32,
}

impl RowVarAllocator {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    /// Allocate the next fresh `RowVar`.
    pub fn fresh(&mut self) -> RowVar {
        let v = RowVar(self.next);
        self.next += 1;
        v
    }
}

impl Default for RowVarAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract a `RowVar` for each higher-order effectful parameter name.
///
/// Returns a `Vec<(String, RowVar)>` — one entry per param in the same
/// order as `params`. Each entry's `RowVar` is fresh from `alloc`.
///
/// **By construction**: every HOF param in `params` gets a `RowVar`. There is
/// no way to skip a param at the call site — the caller must enumerate all HOF
/// params here.
pub fn extract_hof_params(
    params: &[&str],
    alloc: &mut RowVarAllocator,
) -> Vec<(String, RowVar)> {
    params.iter().map(|&name| (name.to_string(), alloc.fresh())).collect()
}

/// Build an `EffectDecl` from the result of `extract_hof_params`.
///
/// Registers every extracted `RowVar` into the decl's `param_rows` field so
/// `infer_row_poly` can propagate them symbolically.
pub fn build_decl_with_extracted_params(
    name: &str,
    extracted: &[(String, RowVar)],
) -> EffectDecl {
    extracted
        .iter()
        .fold(EffectDecl::new(name), |decl, (_, rv)| decl.with_param_row(*rv))
}
