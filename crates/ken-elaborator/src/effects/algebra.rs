//! Row algebra (`36 §2.3`) — K1-buildable set-level operations.
//!
//! The full `⊕` (coproduct of `Op`/`Resp` signatures into a combined `ITree`)
//! is **K1.5-gated** — it requires `ITree` which the current kernel does not
//! admit yet (`check_no_pi_bound_recursive`, §7.0). This module implements the
//! **K1-buildable part**: the set-level row join `ρ1 ⊕ ρ2 = ρ1 ∪ ρ2` that the
//! row-inference and escape-check passes need.

use super::row::{EffectName, EffectRow};

/// A capability parameter: `using name : Cap E` (§2.5, §3).
///
/// A function of row `ρ` takes one `Cap E` parameter per un-handled effect
/// (§2.5). This records one such parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapParam {
    /// The binding name (e.g. `fs`, `net`).
    pub name: String,
    /// The effect this capability gates (e.g. `"FS"`, `"Net"`).
    pub effect: EffectName,
}

impl CapParam {
    pub fn new(name: impl Into<String>, effect: impl Into<EffectName>) -> Self {
        Self {
            name: name.into(),
            effect: effect.into(),
        }
    }
}

/// Row join at the set level: `ρ1 ⊕ ρ2 = ρ1 ∪ ρ2` (§2.3).
///
/// The full `⊕` extends this to a coproduct of `Op`/`Resp` containers;
/// that extension is K1.5-gated. The set-level join is what the row-inference
/// and escape-check passes need.
pub fn row_join(r1: &EffectRow, r2: &EffectRow) -> EffectRow {
    r1.join(r2)
}

/// Capability-set derivable from a list of `CapParam`s.
///
/// Returns the set of effect names for which a capability parameter exists.
pub fn cap_set(params: &[CapParam]) -> EffectRow {
    EffectRow::from_effects(params.iter().map(|p| p.effect.clone()))
}
