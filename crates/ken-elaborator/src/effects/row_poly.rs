//! Row polymorphism — row-poly inference and escape check (`36 §1.2`, row-poly).
//!
//! Now buildable: K1.5 admitted ITree's W-style shape. Row variables can unify
//! with the type system (`36 §2.1`, `§3.1`); latent-row propagation for
//! higher-order parameters is no longer a conservative approximation.
//!
//! ## What this replaces
//!
//! L5-build's `unknown_effectful_params` + `check_higher_order_guard` was a
//! conservative safety valve: callers listed *candidate* effects from an opaque
//! higher-order parameter and the guard rejected if the declared row didn't cover
//! them. The guard was sound but imprecise: `apply_twice (f : A →[ρ] A)` couldn't
//! propagate `ρ` — callers had to manually list candidates.
//!
//! This module provides the precise path: a `RowVar` is assigned to each
//! higher-order parameter's latent row; `infer_row_poly` propagates `RowVar`s
//! symbolically; `check_row_poly_escape` checks `ρ_inf ⊆ ρ_decl` where both
//! sides may contain row variables. `apply_twice` infers row `ρ₀`, declares
//! row `ρ₀`, and the check passes — no candidate listing needed.
//!
//! The conservative guard (`check_higher_order_guard`) remains for callers
//! that still use `EffectDecl::unknown_effectful_params`.

use std::collections::HashMap;

use super::row::{EffectName, EffectRow, RowType, RowVar};
use super::check::EffectError;

/// Infer the row type for a declaration with row-polymorphic higher-order
/// parameters (§1.2, row-poly path).
///
/// This is the upgraded form of `infer_row` that tracks row variables:
/// - `direct_effects`: concrete `perform_E op` nodes — added as `Concrete`.
/// - `callees`: named globals — look up concrete row in `env` (as before).
/// - `param_rows`: row variables assigned to higher-order parameters — each
///   `RowVar` is released into the symbolic row.
///
/// Returns a `RowType` that may contain row variables.
pub fn infer_row_poly(
    env: &HashMap<String, EffectRow>,
    direct_effects: &[EffectName],
    callees: &[String],
    param_rows: &[RowVar],
) -> RowType {
    let mut row = RowType::empty();

    for e in direct_effects {
        row = row.join(RowType::singleton(e.clone()));
    }

    for callee in callees {
        if let Some(callee_row) = env.get(callee.as_str()) {
            row = row.join(RowType::concrete(callee_row.clone()));
        }
    }

    for rv in param_rows {
        row = row.join(RowType::Var(*rv));
    }

    row
}

/// Check `ρ_inf ⊆ ρ_decl` for row types containing row variables (§1.4, row-poly).
///
/// - `declared_row_type`: the polymorphic declared row (may contain `RowVar`s).
///   Takes precedence when `Some`.
/// - `declared_row`: the concrete fallback (`EffectRow` from L5-build's
///   `with_declared_row` builder). Used when `declared_row_type` is `None`.
/// - If both are `None`, `ρ_decl = ∅`.
///
/// Escape errors report concrete escaping effects and uncovered row variables.
/// A row variable in the inferred row that is the *same variable* as in the
/// declared row passes (§1.4: `RowVar(x) ⊆ RowVar(x)`).
pub fn check_row_poly_escape(
    decl_name: &str,
    inferred: &RowType,
    declared_row_type: Option<&RowType>,
    declared_row: Option<&EffectRow>,
) -> Result<(), EffectError> {
    let declared: RowType = declared_row_type
        .cloned()
        .unwrap_or_else(|| {
            declared_row
                .map(|r| RowType::Concrete(r.clone()))
                .unwrap_or(RowType::empty())
        });

    if inferred.is_subset_of(&declared) {
        return Ok(());
    }

    let (concrete_esc, var_esc) = inferred.escaping_from(&declared);

    let mut witnesses: Vec<(EffectName, String)> = concrete_esc
        .effects()
        .map(|e| (e.clone(), "<unknown>".to_string()))
        .collect();
    for v in &var_esc {
        witnesses.push((
            format!("ρ_{}", v.0),
            "<row-var-escape>".to_string(),
        ));
    }

    Err(EffectError::EffectEscapes {
        decl_name: decl_name.to_string(),
        witnesses,
    })
}

