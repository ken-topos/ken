//! Effect-row lattice (`36 §1.1`) and row-polymorphism types (`§1.2` row-poly).
//!
//! An effect row `ρ = {E₁, …, Eₙ}` is a **finite set** of named effects.
//! The lattice structure is set inclusion: join = union, bottom = ∅.
//! A latent-effect arrow `A →[ρ] B` is represented by a `ρ`-annotated
//! declaration in the effect environment.
//!
//! `RowVar` and `RowType` extend the first-order `EffectRow` to track **row
//! variables** for higher-order parameters (K1.5-denotation, `§1.2` `f a`
//! clause where `f : A →[ρ] B` and `ρ` is unknown at definition site).

use std::collections::{BTreeSet, HashMap};

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

// ---------------------------------------------------------------------------
// Row polymorphism — RowVar and RowType (§1.2 row-poly, K1.5-denotation)
// ---------------------------------------------------------------------------

/// A row variable — represents the unknown latent effect row of a higher-order
/// parameter `f : A →[ρ] B` (§1.2 `f a` clause).
///
/// Assigned once per higher-order parameter at the declaration site; the
/// call-graph analysis propagates them symbolically rather than approximating
/// them as ∅ (the L5-build conservative guard approach).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RowVar(pub u32);

/// A row type: a concrete effect set, a row variable, or their join.
///
/// Used when row inference must track symbolic row variables (higher-order
/// params). `Concrete` is the first-order case; `Var` is the row-poly case.
///
/// Defined here (in `row.rs`) so both `infer.rs` and `row_poly.rs` can use it
/// without a circular dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowType {
    /// A fully-known finite set of effects.
    Concrete(EffectRow),
    /// A row variable (from a higher-order parameter's latent row).
    Var(RowVar),
    /// Symbolic join. Reduces to `Concrete` when both arms are concrete;
    /// stays symbolic when either arm is a variable.
    Join(Box<RowType>, Box<RowType>),
}

/// A row substitution: maps row variables to row types.
pub type RowSubst = HashMap<RowVar, RowType>;

impl RowType {
    pub fn empty() -> Self {
        Self::Concrete(EffectRow::empty())
    }

    pub fn singleton(e: impl Into<EffectName>) -> Self {
        Self::Concrete(EffectRow::singleton(e))
    }

    pub fn var(v: RowVar) -> Self {
        Self::Var(v)
    }

    pub fn concrete(row: EffectRow) -> Self {
        Self::Concrete(row)
    }

    /// Lattice join (set-union at concrete level, symbolic otherwise).
    ///
    /// Identity simplification: `∅ ⊕ x = x` and `x ⊕ ∅ = x` — avoids
    /// accumulating `Join(Concrete(∅), ...)` noise in the normal form.
    /// Idempotence/subsumption simplification keeps recursive row-polymorphic
    /// fixpoints finite: `e ∪ e = e`, and `e ∪ (E ∪ e) = E ∪ e`.
    pub fn join(self, other: Self) -> Self {
        if self == other {
            return self;
        }
        if self.is_subset_of(&other) {
            return other;
        }
        if other.is_subset_of(&self) {
            return self;
        }
        match (self, other) {
            (Self::Concrete(r1), Self::Concrete(r2)) => Self::Concrete(r1.join(&r2)),
            (Self::Concrete(r), other) if r.is_empty() => other,
            (this, Self::Concrete(r)) if r.is_empty() => this,
            (l, r) => Self::Join(Box::new(l), Box::new(r)),
        }
    }

    /// Concrete effects reachable in this row type (row variables contribute ∅).
    pub fn concrete_effects(&self) -> EffectRow {
        match self {
            Self::Concrete(r) => r.clone(),
            Self::Var(_) => EffectRow::empty(),
            Self::Join(l, r) => l.concrete_effects().join(&r.concrete_effects()),
        }
    }

    /// All row variables referenced in this type.
    pub fn row_vars(&self) -> Vec<RowVar> {
        let mut vs: Vec<RowVar> = match self {
            Self::Concrete(_) => vec![],
            Self::Var(v) => vec![*v],
            Self::Join(l, r) => {
                let mut v = l.row_vars();
                v.extend(r.row_vars());
                v
            }
        };
        vs.sort();
        vs.dedup();
        vs
    }

    /// `self ⊆ other` (§1.4 escape test, row-poly version).
    ///
    /// - `Concrete(s1) ⊆ Concrete(s2)` iff `s1 ⊆ s2`.
    /// - `Var(x) ⊆ Var(y)` iff `x == y`.
    /// - `Join(a,b) ⊆ r` iff `a ⊆ r && b ⊆ r`.
    /// - `Concrete(∅) ⊆ anything` (empty row is bottom).
    /// - `x ⊆ Join(l,r)` iff `x ⊆ l || x ⊆ r` (x fits entirely in one arm).
    ///   Note: `x ⊆ Join(l,r)` does NOT require `x ⊆ l && x ⊆ r`; it suffices
    ///   that one arm covers `x` (conservative: we don't split `x` across arms
    ///   without unification, but single-arm containment is always sound).
    /// - Everything else: conservative `false`.
    pub fn is_subset_of(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Concrete(s1), Self::Concrete(s2)) => s1.is_subset_of(s2),
            (Self::Var(x), Self::Var(y)) => x == y,
            (Self::Join(l, r), other) => l.is_subset_of(other) && r.is_subset_of(other),
            (Self::Concrete(s), _) if s.is_empty() => true,
            // x ⊆ Join(l, r): check if x is entirely covered by one arm.
            (lhs, Self::Join(l, r)) => lhs.is_subset_of(l) || lhs.is_subset_of(r),
            _ => false,
        }
    }

    /// Effects in `self` not coverable by `other`.
    ///
    /// Returns `(concrete escaping effects, uncovered row vars)`.
    pub fn escaping_from(&self, other: &Self) -> (EffectRow, Vec<RowVar>) {
        match (self, other) {
            (Self::Concrete(s1), Self::Concrete(s2)) => (s1.minus(s2), vec![]),
            (Self::Var(x), Self::Var(y)) if x == y => (EffectRow::empty(), vec![]),
            (Self::Var(x), _) => (EffectRow::empty(), vec![*x]),
            (Self::Join(l, r), other) => {
                let (ce1, cv1) = l.escaping_from(other);
                let (ce2, cv2) = r.escaping_from(other);
                let mut cvs = cv1;
                cvs.extend(cv2);
                cvs.sort();
                cvs.dedup();
                (ce1.join(&ce2), cvs)
            }
            (Self::Concrete(s), _) if s.is_empty() => (EffectRow::empty(), vec![]),
            _ => (self.concrete_effects(), self.row_vars()),
        }
    }

    /// Apply a row substitution: replace `Var(v)` with `subst[v]`.
    pub fn apply_subst(&self, subst: &RowSubst) -> Self {
        match self {
            Self::Concrete(r) => Self::Concrete(r.clone()),
            Self::Var(v) => subst.get(v).cloned().unwrap_or(Self::Var(*v)),
            Self::Join(l, r) => l.apply_subst(subst).join(r.apply_subst(subst)),
        }
    }
}
