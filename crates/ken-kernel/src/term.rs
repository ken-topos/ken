//! Core syntax — terms, universe levels, and the global-name identifiers.
//!
//! This is the kernel's term language (`spec/10-kernel/11-syntax.md`): a small,
//! fully-explicit, de Bruijn-indexed core calculus. The kernel only ever sees
//! this language; elaboration (outside the kernel) produces it.
//!
//! ## Representation
//!
//! - **de Bruijn indices** are the reference representation (`11 §2`): a bound
//!   variable is a `usize` counting binders outward (`0` = nearest enclosing
//!   binder). This makes α-equivalence syntactic identity and substitution
//!   capture-free.
//! - The grammar is **fixed from K1** (`11 §1`): the `[K2]`-tagged formers
//!   (`Ω`, `Eq`, `refl`, `cast`, `J`, quotients, truncation) are carried as
//!   term variants now so the core-term type does not change shape between K1
//!   and K2. K1 implements *none* of their typing/computation — `check`/`infer`
//!   reject them as unrecognised (`11 §6`, `12 §5`); only raw well-formedness
//!   (scoping) is checked for them.
//!
//! ## Levels
//!
//! Universe levels (`12 §1`) form their own category `ℓ ::= 0 | suc ℓ | max ℓ₁
//! ℓ₂ | u` (a level variable). Levels are *not* terms; level polymorphism
//! (`12 §4`) abstracts top-level declarations over level variables and
//! instantiates each use with explicit level arguments — it lives "outside" the
//! term calculus.

use std::fmt;

/// A universe-level variable. Stable across substitution (an index into a
/// declaration's level-parameter list, see [`crate::env`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelVar(pub u32);

/// Universe levels `ℓ ::= 0 | suc ℓ | max ℓ ℓ | u` (`11 §1`, `12 §1`).
///
/// The semilattice equations (`max` associative/commutative/idempotent,
/// `max ℓ 0 = ℓ`, `suc (max ℓ₁ ℓ₂) = max (suc ℓ₁) (suc ℓ₂)`) are handled by
/// [`Level::normalize`]/[`Level::eq`], not by the constructors.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Level {
    /// `0`, the base level.
    Zero,
    /// `suc ℓ` — the next level up.
    Suc(Box<Level>),
    /// `max ℓ₁ ℓ₂` — the join (predicative `max` at Π/Σ formation, `12 §2`).
    Max(Box<Level>, Box<Level>),
    /// `u` — a level variable (`12 §4`), atomic under level equality.
    Var(LevelVar),
}

impl Level {
    /// `0`.
    pub const fn zero() -> Level {
        Level::Zero
    }

    /// `suc ℓ`.
    pub fn suc(self) -> Level {
        Level::Suc(Box::new(self))
    }

    /// `max a b`.
    pub fn max(self, other: Level) -> Level {
        Level::Max(Box::new(self), Box::new(other))
    }

    /// Decidable level equality under the semilattice equations (`12 §1`, §6.1).
    ///
    /// Two levels are equal iff they share a semilattice normal form, with
    /// [`Level::Var`] treated atomically (a variable is equal only to itself;
    /// the kernel never guesses a level, `12 §4`). Conservative: if equality
    /// cannot be decided structurally it returns `false`, never a false `true`.
    /// Named `equiv` to distinguish it from the derived structural `PartialEq`.
    pub fn equiv(&self, other: &Level) -> bool {
        self.normalize() == other.normalize()
    }

    /// Reduce a level to its semilattice normal form.
    ///
    /// Normal form: a sorted, deduplicated `max`-tree of atoms (`Zero`/`Suc` of
    /// atoms/`Var`), with `Zero` absorbed and `suc` pushed through `max`. This
    /// realises `max` a/comm/idemp, `max ℓ 0 = ℓ`, `suc (max a b) = max (suc a)
    /// (suc b)`.
    pub fn normalize(&self) -> Level {
        match self {
            Level::Zero => Level::Zero,
            Level::Var(v) => Level::Var(*v),
            Level::Suc(a) => a.normalize().suc(),
            Level::Max(a, b) => {
                let mut atoms = Vec::new();
                collect_max(&a.normalize(), &mut atoms);
                collect_max(&b.normalize(), &mut atoms);
                normalize_max_atoms(atoms)
            }
        }
    }
}

/// Flatten a normalized level into its `max` atoms (a `Suc`/`Var`/`Zero`),
/// pushing the `suc` count onto each atom.
fn collect_max(l: &Level, out: &mut Vec<(u32, Atom)>) {
    match l {
        Level::Max(a, b) => {
            collect_max(a, out);
            collect_max(b, out);
        }
        Level::Zero => out.push((0, Atom::Zero)),
        Level::Var(v) => out.push((0, Atom::Var(*v))),
        Level::Suc(a) => {
            // `suc` of a max-tree: push +1 onto each atom (suc(max a b) = max
            // (suc a) (suc b)); `suc` of a single atom increments its offset.
            let mut inner = Vec::new();
            collect_max(a, &mut inner);
            for (n, atom) in inner {
                out.push((n + 1, atom));
            }
        }
    }
}

/// A `max` atom: `Zero` or a level variable. (`Suc` is tracked as an offset.)
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Atom {
    Zero,
    Var(LevelVar),
}

/// Rebuild a level from a flattened, suc-offset atom list, applying the
/// semilattice laws: absorb dominated atoms, deduplicate, sort, and right-nest
/// as `Max`.
///
/// **Domination respects atom identity** (`12 §1`). A level `suc^n a` is
/// dominated (absorbed into a larger `max`) only by an atom that is provably
/// `>=` it:
/// - `Zero@n` (`suc^n 0`) is dominated by any atom at offset `> n`, and by a
///   `Var` at offset `== n` (`max (suc^n v) (suc^n 0) = suc^n v`). (The
///   `max ℓ 0 = ℓ` absorption.)
/// - `Var(a)@n` (`suc^n a`) is dominated **only** by `Var(a)@m` with `m > n`
///   — the *same* variable at a higher offset (`max (suc^m a) (suc^n a) =
///   suc^m a`). **Distinct variables never dominate each other** (their levels
///   are incomparable: `max (suc u) v` is not `suc u`, since `v` may exceed
///   `u`), and `Zero` never dominates a `Var`.
fn normalize_max_atoms(atoms: Vec<(u32, Atom)>) -> Level {
    // Deduplicate identical (offset, atom) pairs.
    let mut pairs: Vec<(u32, Atom)> = atoms;
    pairs.sort();
    pairs.dedup();

    // Keep only the non-dominated atoms (see the domination rules above). An
    // atom is never dominated by itself (offsets use strict `<`, or the
    // same-offset `Zero`-by-`Var` case which requires distinct atoms).
    let mut kept: Vec<(u32, Atom)> = Vec::new();
    for &p in &pairs {
        let dominated = pairs.iter().any(|&q| match (p.1, q.1) {
            // `Zero@n` dominated by any atom at offset > n, or by a Var at == n.
            (Atom::Zero, Atom::Zero) => p.0 < q.0,
            (Atom::Zero, Atom::Var(_)) => p.0 <= q.0,
            // `Var(a)@n` dominated only by the SAME variable at a higher offset.
            (Atom::Var(a), Atom::Var(b)) => a == b && p.0 < q.0,
            // `Var@n` is never dominated by `Zero`.
            (Atom::Var(_), Atom::Zero) => false,
        });
        if !dominated {
            kept.push(p);
        }
    }
    if kept.is_empty() {
        return Level::Zero;
    }
    // Right-nest the kept atoms as Max, rebuilding Suc offsets.
    kept.into_iter()
        .map(|(n, atom)| {
            let base = match atom {
                Atom::Zero => Level::Zero,
                Atom::Var(v) => Level::Var(v),
            };
            (0..n).fold(base, |acc, _| acc.suc())
        })
        .reduce(|a, b| a.max(b))
        .expect("non-empty kept list")
}

impl fmt::Debug for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Zero => write!(f, "0"),
            Level::Suc(a) => write!(f, "suc {:?}", a),
            Level::Max(a, b) => write!(f, "max {:?} {:?}", a, b),
            Level::Var(v) => write!(f, "u{}", v.0),
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Identifier of a top-level declaration in the global environment `Σ`
/// (`11 §4`): a transparent def, opaque constant, inductive family, or
/// primitive. Stable names resolved in `Σ`, *not* de Bruijn variables.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlobalId(pub u32);

impl fmt::Debug for GlobalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "g{}", self.0)
    }
}

impl fmt::Display for GlobalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// A term in the core language (`11 §1`). One category for terms and types
/// (types are terms of a universe; Ken is a pure type system).
///
/// Binder convention (de Bruijn): in `Pi(A, B)`, `Lam(A, t)`, `Sigma(A, B)`,
/// and `Let { body, .. }`, the bound variable is index `0` inside the second
/// component, shifted up by [`crate::subst::weaken`] when used in a larger
/// context.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Term {
    // --- Universes (`12`) ---
    /// `Type ℓ` — a universe (`12 §1`).
    Type(Level),
    /// `Ω` — `[K2]` strict-proposition universe; reserved in the grammar,
    /// rejected by K1 `check`/`infer` (`12 §5`).
    Omega,

    // --- Variables & constants ---
    /// A de Bruijn-indexed bound variable (`11 §2`).
    Var(usize),
    /// A use of a global constant/definition `c` with explicit level arguments
    /// (`12 §4`). `level_args` is empty for a mono-level declaration.
    Const {
        id: GlobalId,
        level_args: Vec<Level>,
    },

    // --- Inductive formers & constructors (`14`) ---
    /// A use of an inductive type former `D` with explicit level arguments.
    IndFormer {
        id: GlobalId,
        level_args: Vec<Level>,
    },
    /// A use of a constructor `cₖ` of `D` with explicit level arguments.
    Constructor {
        id: GlobalId,
        level_args: Vec<Level>,
    },
    /// `elim_D {ℓ..} p̄ M m̄ i̅ s` — the dependent eliminator (`14 §3`, §7).
    /// `fam` is the inductive family `D`; `level_args` instantiate `D`'s level
    /// params; `params` are the family parameters `p̄` (the first args of
    /// `elim_D`, since `elim_D : Π Δ_p. Π ℓ'. Π M. Π m̄. Π i̅. Π x. M i̅ x`);
    /// `motive` is `M`; `methods` are `m₁…mₙ`; `indices` are `i̅`; `scrut` is
    /// `s`. Params are carried explicitly so the motive's type — which depends
    /// on the param instance — is checkable without inferring the scrutinee.
    Elim {
        fam: GlobalId,
        level_args: Vec<Level>,
        params: Vec<Term>,
        motive: Box<Term>,
        methods: Vec<Term>,
        indices: Vec<Term>,
        scrut: Box<Term>,
    },

    // --- Π: dependent functions (`13 §1`) ---
    /// `(x : A) → B`. `B` binds `x` at index 0.
    Pi(Box<Term>, Box<Term>),
    /// `λ (x : A). t`. `t` binds `x` at index 0. The domain `A` is carried so
    /// the elaborator's fully-explicit form is preserved (`18 §3` check rule).
    Lam(Box<Term>, Box<Term>),
    /// `f u` — application.
    App(Box<Term>, Box<Term>),

    // --- Σ: dependent pairs (`13 §2`) ---
    /// `(x : A) × B`. `B` binds `x` at index 0. **Genuinely dependent** — `B`
    /// may mention `x` (`README §7`); the kernel MUST NOT shortcut this.
    Sigma(Box<Term>, Box<Term>),
    /// `(t , u)` — pair introduction.
    Pair(Box<Term>, Box<Term>),
    /// `p.1` — first projection.
    Proj1(Box<Term>),
    /// `p.2` — second projection; typed at `B[p.1/x]` (`13 §2`).
    Proj2(Box<Term>),

    // --- let & ascription ---
    /// `let x := val : ty in body`. `body` binds `x` at index 0.
    Let {
        ty: Box<Term>,
        val: Box<Term>,
        body: Box<Term>,
    },
    /// `(t : A)` — type ascription, a checking hint erased once `t` checks
    /// against `A` (`11 §1`). Carries no runtime/conversion content.
    Ascript(Box<Term>, Box<Term>),

    // --- [K2]-reserved formers: parse + raw-wf, but check/infer reject ---
    /// `Eq A t u` — `[K2]` observational equality (`15`, `16`).
    Eq(Box<Term>, Box<Term>, Box<Term>),
    /// `refl t` — `[K2]` (`15`).
    Refl(Box<Term>),
    /// `cast A B e t` — `[K2]` cast along `Eq Type A B` (`16`).
    Cast(Box<Term>, Box<Term>, Box<Term>, Box<Term>),
    /// `J M d e` — `[K2]` derived eliminator (`15`).
    J(Box<Term>, Box<Term>, Box<Term>),
    /// `A / R` — `[K2]` set-quotient (`16`).
    Quot(Box<Term>, Box<Term>),
    /// `[t]` — `[K2]` quotient class (`16`).
    QuotClass(Box<Term>),
    /// `elim_/ M f s` — `[K2]` quotient eliminator (`16`).
    QuotElim {
        motive: Box<Term>,
        method: Box<Term>,
        scrut: Box<Term>,
    },
    /// `‖ A ‖` — `[K2]` propositional truncation (`16`).
    Trunc(Box<Term>),
    /// `|t|` — `[K2]` truncation projection (`16`).
    TruncProj(Box<Term>),
}

/// Convenience constructors (keep call sites readable).
impl Term {
    pub fn var(i: usize) -> Term {
        Term::Var(i)
    }
    pub fn ty(l: Level) -> Term {
        Term::Type(l)
    }
    pub fn pi(a: Term, b: Term) -> Term {
        Term::Pi(Box::new(a), Box::new(b))
    }
    pub fn lam(a: Term, t: Term) -> Term {
        Term::Lam(Box::new(a), Box::new(t))
    }
    pub fn app(f: Term, a: Term) -> Term {
        Term::App(Box::new(f), Box::new(a))
    }
    pub fn sigma(a: Term, b: Term) -> Term {
        Term::Sigma(Box::new(a), Box::new(b))
    }
    pub fn pair(a: Term, b: Term) -> Term {
        Term::Pair(Box::new(a), Box::new(b))
    }
    pub fn proj1(p: Term) -> Term {
        Term::Proj1(Box::new(p))
    }
    pub fn proj2(p: Term) -> Term {
        Term::Proj2(Box::new(p))
    }
    pub fn const_(id: GlobalId, level_args: Vec<Level>) -> Term {
        Term::Const { id, level_args }
    }
    pub fn indformer(id: GlobalId, level_args: Vec<Level>) -> Term {
        Term::IndFormer { id, level_args }
    }
    pub fn constructor(id: GlobalId, level_args: Vec<Level>) -> Term {
        Term::Constructor { id, level_args }
    }

    /// Is this term a `[K2]`-reserved former? K1 `check`/`infer` reject these
    /// as unrecognised (`11 §6`, `12 §5`).
    pub fn is_k2_reserved(&self) -> bool {
        matches!(
            self,
            Term::Omega
                | Term::Eq(..)
                | Term::Refl(_)
                | Term::Cast(..)
                | Term::J(..)
                | Term::Quot(..)
                | Term::QuotClass(_)
                | Term::QuotElim { .. }
                | Term::Trunc(_)
                | Term::TruncProj(_)
        )
    }

    /// The immediate sub-terms (for traversals: substitution, raw-wf, occurs).
    pub fn children(&self) -> Vec<&Term> {
        match self {
            Term::Type(_) | Term::Omega | Term::Var(_) => Vec::new(),
            Term::Const { .. } | Term::IndFormer { .. } | Term::Constructor { .. } => Vec::new(),
            Term::Elim {
                params,
                motive,
                methods,
                indices,
                scrut,
                ..
            } => {
                let mut v: Vec<&Term> = params.iter().collect();
                v.push(motive);
                v.extend(methods);
                v.extend(indices);
                v.push(scrut);
                v
            }
            Term::Pi(a, b) | Term::Lam(a, b) | Term::Sigma(a, b) | Term::Pair(a, b) => vec![a, b],
            Term::App(f, a) | Term::Ascript(f, a) => vec![f, a],
            Term::Proj1(p)
            | Term::Proj2(p)
            | Term::Refl(p)
            | Term::QuotClass(p)
            | Term::Trunc(p)
            | Term::TruncProj(p) => vec![p],
            Term::Let { ty, val, body } => vec![ty, val, body],
            Term::Eq(a, t, u) => vec![a, t, u],
            Term::Cast(a, b, e, t) => vec![a, b, e, t],
            Term::J(m, d, e) => vec![m, d, e],
            Term::Quot(a, r) => vec![a, r],
            Term::QuotElim {
                motive,
                method,
                scrut,
            } => vec![motive, method, scrut],
        }
    }
}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Type(l) => write!(f, "Type {:?}", l),
            Term::Omega => write!(f, "Ω"),
            Term::Var(i) => write!(f, "@{}", i),
            Term::Const { id, level_args } => {
                write!(f, "{:?}", id)?;
                fmt_level_args(f, level_args)
            }
            Term::IndFormer { id, level_args } => {
                write!(f, "D{:?}", id)?;
                fmt_level_args(f, level_args)
            }
            Term::Constructor { id, level_args } => {
                write!(f, "c{:?}", id)?;
                fmt_level_args(f, level_args)
            }
            Term::Elim {
                fam,
                params,
                motive,
                methods,
                indices,
                scrut,
                ..
            } => {
                write!(f, "elim_D{:?}", fam)?;
                if !params.is_empty() {
                    write!(f, " p={:?}", params)?;
                }
                write!(f, " [{:?}] {:?} {:?}", motive, methods, scrut)?;
                if !indices.is_empty() {
                    write!(f, " idx={:?}", indices)?;
                }
                Ok(())
            }
            Term::Pi(a, b) => write!(f, "(Π {:?}. {:?})", a, b),
            Term::Lam(a, t) => write!(f, "(λ {:?}. {:?})", a, t),
            Term::App(g, a) => write!(f, "({:?} {:?})", g, a),
            Term::Sigma(a, b) => write!(f, "(Σ {:?}. {:?})", a, b),
            Term::Pair(a, b) => write!(f, "({:?}, {:?})", a, b),
            Term::Proj1(p) => write!(f, "{:?}.1", p),
            Term::Proj2(p) => write!(f, "{:?}.2", p),
            Term::Let { ty, val, body } => {
                write!(f, "(let : {:?} = {:?} in {:?})", ty, val, body)
            }
            Term::Ascript(t, a) => write!(f, "({:?} : {:?})", t, a),
            Term::Eq(a, t, u) => write!(f, "Eq {:?} {:?} {:?}", a, t, u),
            Term::Refl(t) => write!(f, "refl {:?}", t),
            Term::Cast(a, b, e, t) => write!(f, "cast {:?} {:?} {:?} {:?}", a, b, e, t),
            Term::J(m, d, e) => write!(f, "J {:?} {:?} {:?}", m, d, e),
            Term::Quot(a, r) => write!(f, "{:?}/{:?}", a, r),
            Term::QuotClass(t) => write!(f, "[{:?}]", t),
            Term::QuotElim {
                motive,
                method,
                scrut,
            } => {
                write!(f, "elim_/ {:?} {:?} {:?}", motive, method, scrut)
            }
            Term::Trunc(a) => write!(f, "‖{:?}‖", a),
            Term::TruncProj(t) => write!(f, "|{:?}|", t),
        }
    }
}

fn fmt_level_args(f: &mut fmt::Formatter<'_>, args: &[Level]) -> fmt::Result {
    if args.is_empty() {
        return Ok(());
    }
    write!(f, "{{")?;
    for (i, l) in args.iter().enumerate() {
        if i > 0 {
            write!(f, " ")?;
        }
        write!(f, "{:?}", l)?;
    }
    write!(f, "}}")
}
