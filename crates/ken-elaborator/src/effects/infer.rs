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
use super::row::{EffectName, EffectRow};

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
        }
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
