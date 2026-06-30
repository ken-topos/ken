//! Effect-row lattice (`36 §1.1`).
//!
//! An effect row `ρ = {E₁, …, Eₙ}` is a **finite set** of named effects.
//! The lattice structure is set inclusion: join = union, bottom = ∅.
//! A latent-effect arrow `A →[ρ] B` is represented by a `ρ`-annotated
//! declaration in the effect environment.

use std::collections::BTreeSet;

/// A named effect: `FS`, `Clock`, `Console`, `Net`, `Rand`, `Counter`, …
pub type EffectName = String;

/// A finite, ordered set of effect names — the row `ρ` (§1.1).
///
/// `BTreeSet` gives a canonical ordering (deterministic display + equality).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectRow(BTreeSet<EffectName>);

impl EffectRow {
    pub fn empty() -> Self {
        Self(BTreeSet::new())
    }

    pub fn singleton(e: impl Into<EffectName>) -> Self {
        let mut s = BTreeSet::new();
        s.insert(e.into());
        Self(s)
    }

    pub fn from_effects(effects: impl IntoIterator<Item = EffectName>) -> Self {
        Self(effects.into_iter().collect())
    }

    /// Lattice join: `ρ1 ∪ ρ2` (§1.1).
    pub fn join(&self, other: &Self) -> Self {
        Self(self.0.union(&other.0).cloned().collect())
    }

    /// Subset: `self ⊆ other` (§1.4 declared-row check: `ρ_inf ⊆ ρ_decl`).
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.0.is_subset(&other.0)
    }

    /// Set difference: effects in `self` but not `other`.
    pub fn minus(&self, other: &Self) -> Self {
        Self(self.0.difference(&other.0).cloned().collect())
    }

    pub fn contains(&self, e: &str) -> bool {
        self.0.contains(e)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn effects(&self) -> impl Iterator<Item = &EffectName> {
        self.0.iter()
    }

    pub fn insert(&mut self, e: impl Into<EffectName>) {
        self.0.insert(e.into());
    }
}

impl std::fmt::Display for EffectRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let mut first = true;
        for e in &self.0 {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", e)?;
            first = false;
        }
        write!(f, "]")
    }
}
