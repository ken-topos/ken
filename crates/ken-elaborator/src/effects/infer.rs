//! Effect-row inference: `infer_row` and call-graph least-fixpoint (`36 §1.2`,
//! `§1.3`).
//!
//! `infer_row` computes the least effect row for a declaration from:
//!   - its **direct performs** (`perform_E op` nodes in the body, §1.2);
//!   - its **callee rows** (transitive: each call site releases the callee's
//!     latent row `ρ_f`, §1.2 `f a` clause).
//!
//! For **mutually recursive** definitions the §1.3 call-graph least-fixpoint
//! iterates rows from ∅ until stable (Kripke-style).

use std::collections::HashMap;

use super::algebra::CapParam;
use super::row::{EffectName, EffectRow, RowType, RowVar};

/// A declared surface function, as seen by the effect-row analysis.
///
/// Does **not** carry kernel/core terms — the row analysis is a pure static
/// pass over the surface AST, independent of elaboration (§7.1 pipeline).
#[derive(Debug, Clone)]
pub struct EffectDecl {
    /// The function's name.
    pub name: String,
    /// The user-declared row annotation (`visits [E1, …]`), if present.
    ///
    /// `None` means no annotation; the escape check uses ∅ as the default
    /// (`visits` omitted ≡ `ρ_decl = ∅`, §1.4).
    pub declared_row: Option<EffectRow>,
    /// Capability parameters (`using c : Cap E`) in the function's signature.
    pub cap_params: Vec<CapParam>,
    /// Effect names directly introduced by `perform_E op` nodes in the body
    /// (§1.2 `perform_E op` clause: releases `{E}` into the row).
    pub direct_effects: Vec<EffectName>,
    /// Names of functions called in the body, whose latent rows are released
    /// at each call site (§1.2 `f a` clause).
    pub callees: Vec<String>,
    /// For capability check (§2.5): which effects are directly performed
    /// (require a `Cap E` in scope). Typically same as `direct_effects` but
    /// named separately for clarity.
    pub performed_effects: Vec<EffectName>,
    /// Candidate effects from **higher-order parameters** whose latent rows
    /// are unknown (§1.2 `f a` clause for `f : A →[ρ] B` where `ρ` is a
    /// row variable on a parameter, not a named callee in `callees`).
    ///
    /// **L5-build (conservative):** `check_higher_order_guard` rejects if the
    /// declared row doesn't cover all candidates. Still supported for backward
    /// compatibility; prefer `param_rows` for new code.
    pub unknown_effectful_params: Vec<EffectName>,

    // --- L5-denotation row-polymorphism fields (K1.5-gated, now buildable) ---
    /// Row variables assigned to higher-order parameters (§1.2 `f a` clause,
    /// row-poly). Each element is the `RowVar` for one higher-order parameter's
    /// latent row. `infer_row_poly` propagates these symbolically (rather than
    /// approximating them as ∅).
    ///
    /// Example: `apply_twice (f : A →[ρ₀] A)` assigns `RowVar(0)` for `f`.
    pub param_rows: Vec<RowVar>,

    /// The declared row type for row-polymorphic functions (may contain row
    /// variables). When `Some`, overrides `declared_row` in the row-poly
    /// escape check (`check_row_poly_escape`).
    ///
    /// Example: `apply_twice` declares row `RowType::Var(RowVar(0))` (same
    /// variable as the parameter `f`'s latent row).
    pub declared_row_type: Option<RowType>,
}

impl EffectDecl {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            declared_row: None,
            cap_params: Vec::new(),
            direct_effects: Vec::new(),
            callees: Vec::new(),
            performed_effects: Vec::new(),
            unknown_effectful_params: Vec::new(),
            param_rows: Vec::new(),
            declared_row_type: None,
        }
    }

    /// Declare that this function has a higher-order parameter whose latent
    /// row may contain `effect` (conservative L5-build path).
    pub fn with_unknown_param_effect(mut self, e: impl Into<EffectName>) -> Self {
        self.unknown_effectful_params.push(e.into());
        self
    }

    /// Assign a row variable to a higher-order parameter's latent row
    /// (row-poly path, L5-denotation). `infer_row_poly` will propagate `rv`
    /// symbolically rather than approximating it as ∅.
    pub fn with_param_row(mut self, rv: RowVar) -> Self {
        self.param_rows.push(rv);
        self
    }

    /// Set the declared row as a `RowType` (may contain row variables).
    ///
    /// For row-polymorphic functions (e.g. `apply_twice : (A →[ρ₀] A) → A →[ρ₀]
    /// A`), set this to `RowType::Var(RowVar(0))` so the escape check sees the
    /// same row variable on both sides.
    pub fn with_declared_row_type(mut self, rt: RowType) -> Self {
        self.declared_row_type = Some(rt);
        self
    }

    pub fn with_declared_row(mut self, row: EffectRow) -> Self {
        self.declared_row = Some(row);
        self
    }

    pub fn with_cap_param(mut self, p: CapParam) -> Self {
        self.cap_params.push(p);
        self
    }

    pub fn with_cap_params(mut self, ps: Vec<CapParam>) -> Self {
        self.cap_params.extend(ps);
        self
    }

    pub fn with_direct_effect(mut self, e: impl Into<EffectName>) -> Self {
        let e = e.into();
        self.performed_effects.push(e.clone());
        self.direct_effects.push(e);
        self
    }

    pub fn with_callee(mut self, callee: impl Into<String>) -> Self {
        self.callees.push(callee.into());
        self
    }

    pub fn with_callees(mut self, callees: Vec<String>) -> Self {
        self.callees.extend(callees);
        self
    }
}

/// Infer the effect row for a single `EffectDecl` given an environment of
/// already-known rows (§1.2).
///
/// This is the non-fixpoint form — callee rows are looked up in `env`. For
/// the least-fixpoint over a call graph use `infer_all`.
pub fn infer_row(env: &HashMap<String, EffectRow>, decl: &EffectDecl) -> EffectRow {
    let mut row = EffectRow::empty();
    // Direct `perform_E op` nodes (§1.2, `perform_E op` clause).
    for e in &decl.direct_effects {
        row = row.join(&EffectRow::singleton(e.clone()));
    }
    // Transitive: release each callee's latent row at its call site (§1.2,
    // `f a` clause: `infer_row(f a) = ρ_f ∪ infer_row(a)`).
    for callee in &decl.callees {
        if let Some(callee_row) = env.get(callee.as_str()) {
            row = row.join(callee_row);
        }
    }
    row
}

/// Infer the symbolic effect row for a single declaration from an environment
/// whose callee rows may themselves contain row variables (`36 §1.5.5`).
pub fn infer_row_type(env: &HashMap<String, RowType>, decl: &EffectDecl) -> RowType {
    let mut row = RowType::empty();

    for e in &decl.direct_effects {
        row = row.join(RowType::singleton(e.clone()));
    }

    for callee in &decl.callees {
        if let Some(callee_row) = env.get(callee.as_str()) {
            row = row.join(callee_row.clone());
        }
    }

    for rv in &decl.param_rows {
        row = row.join(RowType::Var(*rv));
    }

    row
}

/// Least-fixpoint effect-row inference over a call graph (`36 §1.3`).
///
/// Handles mutual recursion: starts with ∅ for every function, then iterates
/// `infer_row` per function until no row changes. Terminates because rows
/// grow monotonically (only ∪ operations) and the lattice is finite (finite
/// set of named effects).
///
/// Returns a map `name → inferred EffectRow` for every declaration in `decls`,
/// plus any pre-existing entries in `seed` (e.g. leaf primitives whose rows
/// are declared, not inferred).
pub fn infer_all(
    seed: &HashMap<String, EffectRow>,
    decls: &[EffectDecl],
) -> HashMap<String, EffectRow> {
    // Start from the seed (leaf declarations / pre-declared primitives).
    let mut rows: HashMap<String, EffectRow> = seed.clone();

    // Initialise all analysed declarations to ∅ (will grow toward fixpoint).
    for d in decls {
        rows.entry(d.name.clone()).or_insert_with(EffectRow::empty);
    }

    // Iterate until stable (fixpoint).
    loop {
        let mut changed = false;
        for decl in decls {
            let new_row = infer_row(&rows, decl);
            let old_row = rows.get(&decl.name).cloned().unwrap_or_default();
            if new_row != old_row {
                rows.insert(decl.name.clone(), new_row);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    rows
}

/// Least-fixpoint row inference over `RowType` (`36 §1.5.5`).
///
/// This is the row-polymorphic lift of [`infer_all`]. The update operator is
/// still monotone and uses only joins; `RowType::join` is idempotent, so a
/// recursive declaration that releases the same row variable through its own
/// call reaches a stable point instead of building an ever-deeper join tree.
pub fn infer_all_poly(
    seed: &HashMap<String, RowType>,
    decls: &[EffectDecl],
) -> HashMap<String, RowType> {
    let mut rows: HashMap<String, RowType> = seed.clone();

    for d in decls {
        rows.entry(d.name.clone()).or_insert_with(RowType::empty);
    }

    loop {
        let mut changed = false;
        for decl in decls {
            let new_row = infer_row_type(&rows, decl);
            let old_row = rows.get(&decl.name).cloned().unwrap_or_else(RowType::empty);
            if new_row != old_row {
                rows.insert(decl.name.clone(), new_row);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    rows
}
