//! Type-driven `param_rows` extraction — HOF-effectful params selected
//! mechanically from the complete parameter telescope.
//!
//! ## Why type-driven, not name-list-driven
//!
//! The row-poly soundness contract requires that EVERY higher-order effectful
//! parameter gets a `RowVar` in the `EffectDecl`, or is routed through the
//! conservative `unknown_effectful_params` path. A parameter that goes through
//! NEITHER silently infers ∅ for its latent row — the escape check accepts
//! spuriously.
//!
//! The previous API (`extract_hof_params(&[&str])`) accepted a caller-supplied
//! name list. That relocated the gap: the identification step — which params are
//! HOF-effectful — was still manual and unchecked. Omitting a name from the
//! slice bypassed the RowVar assignment with no compile-time error.
//!
//! ## Solution: classify_telescope
//!
//! `classify_telescope` takes the **complete** parameter telescope — one
//! `(name, ParamTy)` entry per parameter, in order — and assigns a fresh
//! `RowVar` to each parameter whose `ParamTy` is `HofEffectful` or `Unknown`.
//! Selection happens by type, not by caller-supplied membership. Omission is
//! structurally impossible: to "drop" a param you must remove it from the
//! telescope, which makes the telescope incomplete (a caller-side bug, not a
//! silent miss at the row-poly layer).
//!
//! ## ParamTy classification
//!
//! The caller classifies each parameter's type into one of four kinds. This
//! classification is the integration point where elaborator type information
//! feeds the row-poly layer. Future wiring (surface-type traversal →
//! `ParamTy`) makes classification automatic; for now callers supply it
//! explicitly but exhaustively for every param.
//!
//! - `Base` — non-function type; no row needed.
//! - `HofPure` — function type with a pure (concrete-∅) codomain.
//! - `HofEffectful` — function type with an effectful codomain; gets a `RowVar`.
//! - `Unknown` — type not yet resolved; conservative: treated as `HofEffectful`.
//!
//! The `Unknown` arm ensures the module is fail-closed even before full
//! surface-type traversal is wired: an unresolved type gets a RowVar rather
//! than silently inferring ∅.

use super::infer::EffectDecl;
use super::row::RowVar;

// ── Allocator ───────────────────────────────────────────────────────────────

/// Allocates fresh `RowVar`s in strictly-increasing order.
///
/// Create one allocator per function; do NOT share across functions (that
/// would conflate distinct functions' row variables).
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

// ── ParamTy ─────────────────────────────────────────────────────────────────

/// Classification of a function parameter's type for row-variable assignment.
///
/// The caller must classify **every** parameter and supply the result as part
/// of the complete telescope passed to `classify_telescope`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamTy {
    /// Non-function base type (e.g., `Nat`, `Bool`, `String`, `ITree R` as a
    /// first-order value). No latent row — no `RowVar` needed.
    Base,
    /// Function type with a **pure** (concrete-∅) codomain.
    /// Example: `Nat → Nat`.  No latent effects — no `RowVar` needed.
    HofPure,
    /// Function type with an **effectful** codomain (carries a latent row).
    /// Example: `Nat → ITree R`, `A → Eff [ρ] B`.
    /// Must receive a fresh `RowVar` so the latent row is tracked.
    HofEffectful,
    /// Type not yet resolved by the elaborator.
    /// Conservative: treated as `HofEffectful` (fail-closed — assigns a
    /// `RowVar` rather than silently inferring ∅).
    Unknown,
}

impl ParamTy {
    /// Returns `true` iff this parameter requires a fresh `RowVar`.
    pub fn is_hof_effectful(&self) -> bool {
        matches!(self, ParamTy::HofEffectful | ParamTy::Unknown)
    }
}

// ── classify_telescope ───────────────────────────────────────────────────────

/// Assign `RowVar`s to HOF-effectful parameters from the complete telescope.
///
/// `telescope` must contain **one entry per parameter** of the function, in
/// order. Each entry whose `ParamTy::is_hof_effectful()` returns `true` gets a
/// fresh `RowVar` from `alloc`; all other params map to `None`.
///
/// The result is a parallel `Vec<(String, Option<RowVar>)>` with the same
/// length and order as the input telescope.
///
/// By construction: every `HofEffectful` or `Unknown` param receives a
/// `RowVar`. Omission is impossible — you would have to remove the param from
/// the telescope, making the telescope incomplete (a structural error).
pub fn classify_telescope(
    telescope: &[(&str, ParamTy)],
    alloc: &mut RowVarAllocator,
) -> Vec<(String, Option<RowVar>)> {
    telescope
        .iter()
        .map(|(name, ty)| {
            let rv = if ty.is_hof_effectful() {
                Some(alloc.fresh())
            } else {
                None
            };
            (name.to_string(), rv)
        })
        .collect()
}

// ── build_decl_from_telescope ────────────────────────────────────────────────

/// Build an `EffectDecl` from the output of `classify_telescope`.
///
/// Only entries with `Some(RowVar)` contribute a `with_param_row` call;
/// first-order and HOF-pure params (mapped to `None`) are skipped.
pub fn build_decl_from_telescope(
    name: &str,
    classified: &[(String, Option<RowVar>)],
) -> EffectDecl {
    classified.iter().fold(EffectDecl::new(name), |decl, (_, rv_opt)| {
        match rv_opt {
            Some(rv) => decl.with_param_row(*rv),
            None => decl,
        }
    })
}
