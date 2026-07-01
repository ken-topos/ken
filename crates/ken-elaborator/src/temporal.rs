//! B2 — `Temporal Σ`: temporal/behavioral logic as **deeply-embedded inductive
//! data** (`spec/70-behavioral/72-temporal.md`, impl-ready B2).
//!
//! Ken **states** a temporal property as an ordinary inductive value and
//! **delegates** its discharge to `Ward`; the kernel gains **no** modal
//! judgment (`OQ-temporal` DECIDED — data-only, ADR 0006). This module is the
//! buildable-now `Temporal` half of the Ward seam: the datatype, the derived
//! operators, the `delegated` export flow into the landed B1 `T` channel, and
//! the reason-*about* metatheorem (`closed`). The `sat`/`compile` semantics +
//! the `WardFormula` wire spelling are the joint Ward encoding pass
//! (`(oracle)`-tagged, `72 §3.1`/§6.3) and are **not** built here.
//!
//! Built from `/spec` (`72`) + `/conformance` (`seed-temporal.md`) only — never
//! from prototype source (`CLEAN-ROOM.md`).
//!
//! # Two representations, one datatype
//!
//! - **The value** — [`Temporal`] (below): a Rust enum, the deeply-embedded
//!   LTL/μ-calculus value the elaborator produces, reasons about (`closed`),
//!   and exports. Ordinary structural recursion over this enum IS the
//!   `elim_Temporal` analog (`72 §6.1`): one branch per constructor.
//! - **The kernel admission** — [`temporal_inductive_spec`]: the same datatype
//!   as a kernel [`InductiveSpec`], so the real `declare_inductive` +
//!   `check_positivity` ground the spec's "admitted by K1" claim against the
//!   kernel that exists now (the TE-A positivity pair). The kernel sees only
//!   the core constructors; derived ops are syntax, not constructors (AC2).

use ken_kernel::{CtorSpec, GlobalId, InductiveSpec, Level, Term};

// ─── Fixpoint variables (first-order) ───────────────────────────────────────

/// A fixpoint variable — **first-order**, not HOAS (`72 §3.1`).
///
/// `mu`/`nu` bind a `Var` and `var X` refers to it; the body is a plain
/// `Temporal Σ`. First-order binding is **load-bearing**: a HOAS encoding
/// (`mu : (Temporal → Temporal) → …`) puts `Temporal` in a negative position
/// and breaks strict positivity (TE-A2). The exact representation (named vs de
/// Bruijn) is `(oracle)`-tagged (`72 §3.1`); a first-order `Name` is the B2
/// buildable-now choice.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Var(pub String);

// ─── The atom language `Pred Σ` ──────────────────────────────────────────────

/// A state/event predicate over the effect alphabet `Σ` (`72 §3`).
///
/// Atoms are exactly the events `Ward` monitors, over the B1 `Σ` (`71 §2`).
/// `Top` is the always-true predicate (`atom ⊤`, used by the derived
/// operators). The full `Pred Σ` language (events only vs events + observable
/// state) is `(oracle)`-tagged (`72 §3.1`); `Top` + a named event predicate is
/// the B2 buildable-now value-set.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pred {
    /// `⊤` — the always-true predicate (`atom ⊤`).
    Top,
    /// A named event predicate over `Σ` (e.g. `settled`, `req`).
    Event(String),
}

// ─── The `Temporal Σ` datatype — LTL/μ core (§3) ─────────────────────────────

/// `Temporal Σ` — a deeply-embedded LTL/μ-calculus value over `Σ` (`72 §3`).
///
/// The **core constructors** the kernel admits (TE-A) and the derived operators
/// elaborate to (TE-C). `◇`/`□`/`leadsto` are **not** constructors — see
/// [`Temporal::eventually`]/[`always`]/[`leadsto`].
///
/// Inert to the kernel (`72 §7`): consumed only by ordinary structural
/// recursion (the `elim_Temporal` analog); introduces no conversion/η rule.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Temporal {
    /// `atom p` — a state/event predicate.
    Atom(Pred),
    /// `not φ`.
    Not(Box<Temporal>),
    /// `and φ ψ`.
    And(Box<Temporal>, Box<Temporal>),
    /// `or φ ψ`.
    Or(Box<Temporal>, Box<Temporal>),
    /// `next φ` — `◯`/`X`.
    Next(Box<Temporal>),
    /// `until φ ψ` — `φ U ψ`.
    Until(Box<Temporal>, Box<Temporal>),
    /// `mu X. φ` — least fixpoint (X first-order, guarded in φ).
    Mu { var: Var, body: Box<Temporal> },
    /// `nu X. φ` — greatest fixpoint.
    Nu { var: Var, body: Box<Temporal> },
    /// `var X` — a fixpoint-variable reference.
    Var(Var),
}

impl Temporal {
    /// `◇φ := until (atom ⊤) φ` — **eventually** (`72 §3`). Derived syntax, not
    /// a constructor: the elaborated head is `until` (TE-C1).
    pub fn eventually(phi: &Temporal) -> Temporal {
        Temporal::Until(Box::new(Temporal::Atom(Pred::Top)), Box::new(phi.clone()))
    }

    /// `□φ := not (until (atom ⊤) (not φ))` — **always** (`¬◇¬φ`, `72 §3`).
    /// Derived: head `not`, inner `until` (TE-C2).
    pub fn always(phi: &Temporal) -> Temporal {
        Temporal::Not(Box::new(Temporal::Until(
            Box::new(Temporal::Atom(Pred::Top)),
            Box::new(Temporal::Not(Box::new(phi.clone()))),
        )))
    }

    /// `p ~> q := □ (not p or ◇ q)` — **leadsto** (`72 §3`). Two layers of
    /// derivation (`□` over `◇`); the elaborated tree is built entirely from
    /// the `until`/`not`/`or`/`atom` core (TE-C3).
    pub fn leadsto(p: &Temporal, q: &Temporal) -> Temporal {
        let inner = Temporal::Or(
            Box::new(Temporal::Not(Box::new(p.clone()))),
            Box::new(Temporal::eventually(q)),
        );
        Temporal::always(&inner)
    }
}

// ─── Reason-*about*: the `closed` metatheorem (§6.1, AC5) ────────────────────

/// `closed : Temporal Σ → Bool` — `true` iff every `var X` occurs under a
/// binding `mu`/`nu X` (`72 §6.1`).
///
/// Ordinary structural recursion over the datatype (the `elim_Temporal`
/// analog) with a binder environment — **no** trace/satisfaction model, **no**
/// new kernel power. The bound/free verdict flip (TE-F1) pins that `closed`
/// actually inspects structure: `var X` under an enclosing `mu X` → `true`; the
/// same `var X` free → `false`.
pub fn closed(phi: &Temporal) -> bool {
    closed_with(phi, &std::collections::HashSet::new())
}

/// `closed` with an accumulated binder environment (the `mu`/`nu` vars in
/// scope). Structural recursion: one branch per constructor.
fn closed_with(phi: &Temporal, env: &std::collections::HashSet<Var>) -> bool {
    match phi {
        Temporal::Atom(_) => true,
        Temporal::Not(a) => closed_with(a, env),
        Temporal::And(a, b) | Temporal::Or(a, b) | Temporal::Until(a, b) => {
            closed_with(a, env) && closed_with(b, env)
        }
        Temporal::Next(a) => closed_with(a, env),
        Temporal::Mu { var, body } | Temporal::Nu { var, body } => {
            let mut env2 = env.clone();
            env2.insert(var.clone());
            closed_with(body, &env2)
        }
        Temporal::Var(x) => env.contains(x),
    }
}

// ─── Surface `temporal{}` notation → constructors (§4, AC3) ─────────────────

/// Surface AST for a `temporal { … }` block (`72 §4`). Keywords are
/// `(oracle)`/`OQ-syntax`; the **elaboration target** (the §3 constructors)
/// and the **`delegated`** status are pinned. Fixpoint surface syntax (`mu`/
/// `nu`/`var`) is deferred to the Ward encoding pass (`72 §3.1`) and is **not**
/// in the surface AST; the core [`Temporal`] enum carries those constructors
/// for the kernel admission + `closed`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemporalExpr {
    /// An atom: a named event predicate, or `top`/`true` for `atom ⊤`.
    Atom(String),
    /// `not φ`.
    Not(Box<TemporalExpr>),
    /// `φ and ψ`.
    And(Box<TemporalExpr>, Box<TemporalExpr>),
    /// `φ or ψ`.
    Or(Box<TemporalExpr>, Box<TemporalExpr>),
    /// `next φ`.
    Next(Box<TemporalExpr>),
    /// `φ until ψ`.
    Until(Box<TemporalExpr>, Box<TemporalExpr>),
    /// `eventually φ` — derived (`◇`).
    Eventually(Box<TemporalExpr>),
    /// `always φ` — derived (`□`).
    Always(Box<TemporalExpr>),
    /// `p leadsto q` — derived (`~>`).
    Leadsto(Box<TemporalExpr>, Box<TemporalExpr>),
}

/// Elaborate a surface [`TemporalExpr`] to a core [`Temporal`] value (`72 §4`),
/// expanding the derived operators to the `until`/`not` core (AC2/TE-C).
///
/// `top`/`true` elaborate to `atom ⊤`; any other atom name is a named event
/// predicate over `Σ`.
pub fn elaborate_temporal_expr(expr: &TemporalExpr) -> Temporal {
    match expr {
        TemporalExpr::Atom(name) => {
            let pred = match name.as_str() {
                "top" | "true" => Pred::Top,
                _ => Pred::Event(name.clone()),
            };
            Temporal::Atom(pred)
        }
        TemporalExpr::Not(a) => Temporal::Not(Box::new(elaborate_temporal_expr(a))),
        TemporalExpr::And(a, b) => Temporal::And(
            Box::new(elaborate_temporal_expr(a)),
            Box::new(elaborate_temporal_expr(b)),
        ),
        TemporalExpr::Or(a, b) => Temporal::Or(
            Box::new(elaborate_temporal_expr(a)),
            Box::new(elaborate_temporal_expr(b)),
        ),
        TemporalExpr::Next(a) => Temporal::Next(Box::new(elaborate_temporal_expr(a))),
        TemporalExpr::Until(a, b) => Temporal::Until(
            Box::new(elaborate_temporal_expr(a)),
            Box::new(elaborate_temporal_expr(b)),
        ),
        TemporalExpr::Eventually(a) => {
            Temporal::eventually(&elaborate_temporal_expr(a))
        }
        TemporalExpr::Always(a) => Temporal::always(&elaborate_temporal_expr(a)),
        TemporalExpr::Leadsto(a, b) => {
            Temporal::leadsto(&elaborate_temporal_expr(a), &elaborate_temporal_expr(b))
        }
    }
}

// ─── The elaborated obligation (delegated, never a kernel hole) ──────────────

/// A `temporal{}` claim elaborated to a `Temporal` value + its `delegated`
/// status (`72 §4`/§5, AC3/AC4).
///
/// This is **not** a kernel obligation hole: a delegated property is **exported,
/// not assumed** (`21 §5.2`) — it is never in `trusted_base()` (it is not
/// `unknown`) and never kernel-proved (not `proved`/`Q`). Its sole projection is
/// the B1 `T`/`delegated` channel (TE-E). `source` carries the human-visible
/// formula text (it appears verbatim in source, not erased — `72 §4`).
#[derive(Clone, Debug)]
pub struct TemporalObligation {
    /// Stable obligation id (`22 §1`): `"{decl}.temporal.{idx}"`.
    pub id: String,
    /// The elaborated `Temporal` value (the `TEntry` body, `72 §5`).
    pub formula: Temporal,
    /// Human-visible source text of the formula (`72 §4`).
    pub source: String,
}

// ─── Kernel admission — the real `declare_inductive` spec (TE-A) ─────────────

/// The `Temporal` family as a kernel [`InductiveSpec`] for the real
/// `declare_inductive` + `check_positivity` (TE-A1). First-order `Var` binding:
/// every recursive occurrence of `Temporal` is **direct** (strictly positive),
/// so K1 admits it **without** the K1.5 W-style path (`72 §3.1`).
///
/// `Pred Σ` and `Var` are non-recursive parameters; their exact spelling is
/// `(oracle)`-tagged, so the arg type is a placeholder non-recursive type
/// (`Type 0`). The strict-positivity property is independent of that spelling —
/// it hinges on `Temporal` occurring only in direct (positive) positions.
pub fn temporal_inductive_spec(d_id: GlobalId) -> InductiveSpec {
    let d = Term::indformer(d_id, vec![]);
    let pred = Term::ty(Level::Zero); // (oracle): `Pred Σ` spelling deferred
    let var = Term::ty(Level::Zero); // (oracle): `Var` spelling deferred
    let ctor = |args: Vec<Term>| CtorSpec {
        args,
        target_indices: vec![],
    };
    InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::Zero,
        constructors: vec![
            ctor(vec![pred.clone()]),         // atom : Pred → Temporal
            ctor(vec![d.clone()]),            // not  : Temporal → Temporal
            ctor(vec![d.clone(), d.clone()]), // and  : Temporal → Temporal → Temporal
            ctor(vec![d.clone(), d.clone()]), // or
            ctor(vec![d.clone()]),            // next : Temporal → Temporal
            ctor(vec![d.clone(), d.clone()]), // until : Temporal → Temporal → Temporal
            ctor(vec![var.clone(), d.clone()]), // mu  : Var → Temporal → Temporal
            ctor(vec![var.clone(), d.clone()]), // nu  : Var → Temporal → Temporal
            ctor(vec![var]),                   // var : Var → Temporal
        ],
    }
}

/// The **HOAS variant** of `Temporal` for TE-A2: `mu`/`nu` take
/// `(Temporal → Temporal) → Temporal`, placing `Temporal` in a **negative**
/// position (the domain of the inner arrow). The same `check_positivity` that
/// admits the first-order datatype rejects this — the non-degenerate pair's
/// verdict flips on the structural discriminator (`72 §3.1`).
pub fn temporal_hoas_inductive_spec(d_id: GlobalId) -> InductiveSpec {
    let mut spec = temporal_inductive_spec(d_id);
    let d = Term::indformer(d_id, vec![]);
    // Replace the first-order `mu`/`nu` (`Var → Temporal → Temporal`) with the
    // HOAS shape `(Temporal → Temporal) → Temporal`: a single Pi(Pi(D, D), D)? —
    // no: the constructor arg is the HOAS function space `(Temporal → Temporal)`,
    // i.e. one arg of type `Pi(D, D)`.
    let hoas_arg = Term::pi(d.clone(), d);
    // mu is constructor index 6, nu is 7 in `temporal_inductive_spec`.
    spec.constructors[6] = CtorSpec {
        args: vec![hoas_arg.clone()],
        target_indices: vec![],
    };
    spec.constructors[7] = CtorSpec {
        args: vec![hoas_arg],
        target_indices: vec![],
    };
    spec
}
