//! The local context Γ and the global environment Σ (`11 §3`, §4).
//!
//! - A [`Context`] is an ordered telescope of term-variable types (de Bruijn:
//!   the most-recently pushed variable is index `0`). There are no
//!   interval/cofibration entries (ADR 0005).
//! - A [`GlobalEnv`] records top-level declarations in dependency order. It is
//!   **append-only and acyclic** (`11 §4`): a declaration may reference only
//!   earlier ones, which is what makes δ-unfolding well-founded.
//!
//! Admission *checks* (signature type-checking, strict positivity, universe
//! checks) live in [`crate::check`] / [`crate::inductive`]; this module is the
//! pure data structure, lookup, and the type-former/constructor type generation
//! that makes `infer` O(1).

use std::collections::HashMap;

use crate::term::{GlobalId, Level, LevelVar, Term};

/// The local context Γ — a telescope of term-variable types (`11 §3`).
///
/// `types[len-1]` is the type of de Bruijn variable `0`; `types[0]` is the
/// type of variable `len-1`. Pushing `x : A` appends `A`.
#[derive(Clone, Default)]
pub struct Context {
    pub types: Vec<Term>,
}

impl Context {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }

    /// Number of bindings (the de Bruijn depth).
    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Extend with `x : ty` (ty is in the current context).
    pub fn push(&mut self, ty: Term) {
        self.types.push(ty);
    }

    pub fn pop(&mut self) -> Option<Term> {
        self.types.pop()
    }

    /// Type of de Bruijn variable `i`, or `None` if out of range (raw-wf fail).
    pub fn lookup(&self, i: usize) -> Option<&Term> {
        let n = self.types.len();
        if i < n {
            Some(&self.types[n - 1 - i])
        } else {
            None
        }
    }

    /// Extend with a telescope whose entries are stored relative to the
    /// preceding entry (`tel[i]` is in the context of `tel[0..i]` on top of the
    /// current context). The current context must match the telescope's base.
    pub fn extend_tel(&mut self, tel: &[Term]) {
        for t in tel {
            self.types.push(t.clone());
        }
    }
}

/// Build `Π (x₁:A₁)…(xₙ:Aₙ). body` as a right-nested `Pi`-chain.
///
/// `tel[i]` is in the context of `tel[0..i]`; `body` is in the context of the
/// whole telescope. This is the canonical telescope-to-Π fold used to build
/// type-former and constructor types.
pub fn telescope_to_pi(tel: &[Term], body: Term) -> Term {
    tel.iter()
        .rev()
        .fold(body, |acc, a| Term::pi(a.clone(), acc))
}

/// A primitive reduction rule — the *interface* a primitive registers (`14
/// §5`). K1 defines only the interface; the value model (K3) and kernel API
/// (K-api) elaborate the registered computation. Primitives are opaque
/// constants in K1. Checked surface literals carry separate accounting status:
/// they are values, not primitive operations or assumed postulates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrimReduction {
    /// An opaque primitive type (e.g. `Int : Type 0`) — no reduction.
    OpaqueType,
    /// A checked surface literal whose value is stored by the elaborator side
    /// table. This is not a trusted primitive assumption.
    Literal,
    /// A primitive operation awaiting its registered reduction (K3).
    Op {
        /// Symbolic name, for `trusted_base()` enumeration.
        symbol: &'static str,
    },
}

/// A registered decidable-equality certificate for an opaque primitive type
/// (`docs/adr/0013-int-decidable-equality-kernel-posture.md` Layer 1) — the
/// kernel-audited assumption that `eq_op`
/// decides propositional equality at the registered primitive, split into a
/// sound direction (`eq_op` true ⇒ propositionally equal) and a complete
/// direction (propositionally equal ⇒ `eq_op` true). Both `sound` and
/// `complete` are ordinary [`Decl::Opaque`] postulates admitted via
/// [`crate::check::declare_postulate`] (so their types are kernel-checked
/// before being trusted); this struct is only the registry record tying them
/// to the primitive and its equality op.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecEqCert {
    pub eq_op: GlobalId,
    pub sound: GlobalId,
    pub complete: GlobalId,
}

/// A constructor of an inductive family (`14 §1`).
///
/// `cₖ : (Δₖ) → D Δ_p t̄ₖ`. The `args` telescope `Δₖ` and the `target_indices`
/// `t̄ₖ` are stored relative to `Δ_p` (and `t̄ₖ` may additionally reference the
/// args, e.g. `Vec`'s `suc n` index). [`ConstructorDecl::type_`] is the fully
/// expanded `Π Δ_p. Π Δₖ. D Δ_p t̄ₖ`, generated at admission for O(1) `infer`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructorDecl {
    pub id: GlobalId,
    /// `Δₖ` — constructor argument telescope, relative to `Δ_p`.
    pub args: Vec<Term>,
    /// `t̄ₖ` — the index instance the constructor targets, relative to
    /// `Δ_p + Δₖ`.
    pub target_indices: Vec<Term>,
    /// `Π Δ_p. Π Δₖ. D Δ_p t̄ₖ` — the constructor's full type (level-polymorphic
    /// in the family's level params).
    pub type_: Term,
    /// Positions `j` in `args` whose type is a recursive occurrence of `D`
    /// (used to insert induction hypotheses in ι, `14 §7.3`).
    pub recursive_positions: Vec<usize>,
}

/// An inductive family declaration (`14 §1`).
///
/// `data D (Δ_p) : (Δ_i) → Type ℓ where cₖ : (Δₖ) → D Δ_p t̄ₖ`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InductiveDecl {
    pub id: GlobalId,
    /// Level parameters abstracted by the family (`12 §4`); empty if mono-level.
    pub level_params: Vec<LevelVar>,
    /// `Δ_p` — parameters, fixed across the family. Relative to the empty
    /// term context (only level params are in scope).
    pub params: Vec<Term>,
    /// `Δ_i` — indices, may vary per constructor. Relative to `Δ_p`.
    pub indices: Vec<Term>,
    /// `ℓ` — the family's universe level (may mention `level_params`).
    pub level: Level,
    /// Constructors in declaration order.
    pub constructors: Vec<ConstructorDecl>,
    /// `Π Δ_p. Π Δ_i. Type ℓ` — the type former's full type (level-polymorphic).
    pub former_type: Term,
}

/// A top-level declaration in `Σ` (`11 §4`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decl {
    /// `c : A := t` — transparent definition, δ-unfoldable. Non-recursive in
    /// K1 (acyclic env); general recursive δ is K2c (`11 §4`).
    Transparent {
        id: GlobalId,
        level_params: Vec<LevelVar>,
        ty: Term,
        body: Term,
    },
    /// `c : A` — opaque constant / postulate; blocks δ (`11 §4`).
    Opaque {
        id: GlobalId,
        /// Required human-readable audit label. This is provenance metadata,
        /// not declaration identity, and no kernel judgment may inspect it.
        name: String,
        level_params: Vec<LevelVar>,
        ty: Term,
    },
    /// `data D …` — an inductive family; carries its constructors and the
    /// generated former type.
    Inductive(InductiveDecl),
    /// `c : A := prim p` — a primitive type/operation, opaque + registered
    /// reduction (`14 §5`). K1: interface only.
    Primitive {
        id: GlobalId,
        level_params: Vec<LevelVar>,
        ty: Term,
        reduction: PrimReduction,
    },
}

/// One readable entry in the unchecked-assumption ledger (`18 §4.2`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrustedBaseEntry {
    pub id: GlobalId,
    pub name: String,
}

/// The readable trusted-base ledger.
///
/// Owned iteration intentionally yields the historical `GlobalId` stream so
/// existing trust-delta set/count checks keep their exact semantics. Audit
/// clients use [`TrustedBase::entries`] to read the required labels.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrustedBase {
    entries: Vec<TrustedBaseEntry>,
}

impl TrustedBase {
    pub fn entries(&self) -> &[TrustedBaseEntry] {
        &self.entries
    }

    /// Iterate over opaque ids for compatibility with the historical
    /// set-shaped trust audit API. Use [`Self::entries`] when labels matter.
    pub fn iter(&self) -> impl Iterator<Item = &GlobalId> {
        self.entries.iter().map(|entry| &entry.id)
    }

    pub fn contains(&self, id: &GlobalId) -> bool {
        self.entries.iter().any(|entry| entry.id == *id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl IntoIterator for TrustedBase {
    type Item = GlobalId;
    type IntoIter = std::vec::IntoIter<GlobalId>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries
            .into_iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl Decl {
    pub fn id(&self) -> GlobalId {
        match self {
            Decl::Transparent { id, .. } | Decl::Opaque { id, .. } | Decl::Primitive { id, .. } => {
                *id
            }
            Decl::Inductive(d) => d.id,
        }
    }

    /// The level parameters abstracted by this declaration.
    pub fn level_params(&self) -> &[LevelVar] {
        match self {
            Decl::Transparent { level_params, .. }
            | Decl::Opaque { level_params, .. }
            | Decl::Primitive { level_params, .. } => level_params,
            Decl::Inductive(d) => &d.level_params,
        }
    }

    /// Is this a transparent (δ-unfoldable) definition?
    pub fn is_transparent(&self) -> bool {
        matches!(self, Decl::Transparent { .. })
    }
}

/// The global environment `Σ` — append-only, acyclic (`11 §4`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GlobalEnv {
    decls: Vec<Decl>,
    by_id: HashMap<GlobalId, usize>,
    /// Constructor id → (index into `decls`, index into the inductive's
    /// constructors).
    ctor_index: HashMap<GlobalId, (usize, usize)>,
    next_id: u32,
    /// The prelude `Top : Ω_0` constant (`16 §1.3`) — the truth proposition,
    /// produced by Eq-by-type at `Trunc` (`Eq ‖A‖ _ _ ⇝ Top`) and the canonical
    /// "trivial proof" target. Set by [`GlobalEnv::new`].
    top_id: Option<GlobalId>,
    /// The prelude `Bottom : Ω_0` constant (`16 §1.3`) — the falsity
    /// proposition, produced by Eq-by-type's different-constructor case
    /// (`Eq (D …) (c_k …) (c_l …) ⇝ Bottom`). Set by [`GlobalEnv::new`].
    bottom_id: Option<GlobalId>,
    /// The prelude `tt : Top` constant (`16 §1.3`, K5) — `Top`'s sole
    /// introduction, the canonical proof of a goal that reduced to `Top`.
    /// Set by [`GlobalEnv::new`].
    tt_id: Option<GlobalId>,
    /// Registered decidable-equality certificates
    /// (`docs/adr/0013-int-decidable-equality-kernel-posture.md` Layer 1),
    /// keyed by the primitive type's [`GlobalId`]. General, opt-in,
    /// per-primitive — an unregistered primitive has no entry here and its
    /// `Eq` stays neutral exactly as before (`obs.rs`'s fail-safe default is
    /// untouched by this registry).
    deceq_certs: HashMap<GlobalId, DecEqCert>,
    /// The primitive type `Term::IntLit` terms belong to
    /// (`docs/adr/0013-int-decidable-equality-kernel-posture.md` Layer 2),
    /// set once by [`GlobalEnv::register_int_lit_type`]. `None` until
    /// registered — bookkeeping metadata (like `deceq_certs`), never a
    /// `trusted_base()` member.
    int_lit_ty: Option<GlobalId>,
}

impl GlobalEnv {
    pub fn new() -> Self {
        let mut env = Self::default();
        // K2 prelude — the truth/falsity propositions as direct `Ω_0`
        // constants (`16 §1.3`; the unsound general `Up : Type → Ω` coercion is
        // dropped, so these are standalone declarations, not wrappings). They
        // are kernel vocabulary (like `Type`/`Ω`), kept out of `trusted_base`.
        env.bottom_id =
            Some(env.declare_prelude_const("Bottom", Term::Omega(Level::zero())));
        env.top_id = Some(env.declare_prelude_const("Top", Term::Omega(Level::zero())));
        // K5: `tt : Top` — `Top`'s sole inhabitant, a genuine sub-singleton
        // admissible in Ω (`16 §1.1`). Typed at `Top` itself (not `Ω_0`), so
        // this must come after `top_id` is set.
        let top = Term::Const {
            id: env.top_id.expect("top_id just set"),
            level_args: Vec::new(),
        };
        env.tt_id = Some(env.declare_prelude_const("tt", top));
        env
    }

    /// Declare a prelude constant `c : ty` (opaque, no δ). Used only by
    /// [`new`] for `Top`/`Bottom` (`ty = Ω_0`) and `tt` (`ty = Top`). The
    /// caller is responsible for `ty` being well-formed without running the
    /// check pipeline (both uses here are, by the `Omega`-formation and
    /// sub-singleton-in-Ω rules, `16 §1.1`).
    fn declare_prelude_const(&mut self, name: &str, ty: Term) -> GlobalId {
        let id = self.fresh_id();
        self.add_decl(Decl::Opaque {
            id,
            name: name.to_string(),
            level_params: Vec::new(),
            ty,
        });
        id
    }

    /// The prelude `Top : Ω_0` constant id (`16 §1.3`); always present after
    /// [`GlobalEnv::new`].
    pub fn top_id(&self) -> GlobalId {
        self.top_id
            .expect("prelude Top is declared in GlobalEnv::new")
    }

    /// The prelude `Bottom : Ω_0` constant id (`16 §1.3`); always present after
    /// [`GlobalEnv::new`].
    pub fn bottom_id(&self) -> GlobalId {
        self.bottom_id
            .expect("prelude Bottom is declared in GlobalEnv::new")
    }

    /// The prelude `tt : Top` constant id (`16 §1.3`, K5); always present
    /// after [`GlobalEnv::new`].
    pub fn tt_id(&self) -> GlobalId {
        self.tt_id
            .expect("prelude tt is declared in GlobalEnv::new")
    }

    /// Is `id` one of the prelude `Top`/`Bottom`/`tt` constants?
    fn is_prelude(&self, id: GlobalId) -> bool {
        self.top_id == Some(id) || self.bottom_id == Some(id) || self.tt_id == Some(id)
    }

    /// Allocate a fresh, unused [`GlobalId`]. Used during admission so a
    /// family's constructors can reference the family before it is committed.
    pub fn fresh_id(&mut self) -> GlobalId {
        let id = GlobalId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Commit an already-checked declaration. The caller is responsible for
    /// having run the admission checks (`crate::check`).
    pub fn add_decl(&mut self, decl: Decl) -> GlobalId {
        let id = decl.id();
        let idx = self.decls.len();
        self.decls.push(decl);
        self.by_id.insert(id, idx);
        if let Decl::Inductive(ind) = &self.decls[idx] {
            for (ci, c) in ind.constructors.iter().enumerate() {
                self.ctor_index.insert(c.id, (idx, ci));
            }
        }
        id
    }

    pub fn lookup(&self, id: GlobalId) -> Option<&Decl> {
        self.by_id.get(&id).map(|&i| &self.decls[i])
    }

    /// Remove the most-recently added declaration (provisional admission
    /// rollback: an inductive whose signature fails checking is withdrawn so
    /// its not-yet-finalized id is not left dangling). Reindexes the lookup
    /// maps; the popped [`GlobalId`]s become free for re-use.
    pub fn remove_last(&mut self) -> Option<Decl> {
        let decl = self.decls.pop()?;
        self.by_id.remove(&decl.id());
        if let Decl::Inductive(ind) = &decl {
            for c in &ind.constructors {
                self.ctor_index.remove(&c.id);
            }
        }
        // Declarations receive consecutive ids, and an inductive's family id
        // precedes all of its constructor ids. Rewind to the declaration's id
        // so provisional admission rollback restores the allocator as well as
        // the lookup tables.
        self.next_id = self.next_id.min(decl.id().0);
        Some(decl)
    }

    /// The (level_params, type) of a const/former/primitive use, for `infer`.
    /// For an inductive former this is the `former_type`; for a constructor use
    /// [`GlobalEnv::constructor`] is used instead.
    pub fn const_type(&self, id: GlobalId) -> Option<(&[LevelVar], Term)> {
        let decl = self.lookup(id)?;
        match decl {
            Decl::Transparent {
                level_params, ty, ..
            } => Some((level_params, ty.clone())),
            Decl::Opaque {
                level_params, ty, ..
            } => Some((level_params, ty.clone())),
            Decl::Primitive {
                level_params, ty, ..
            } => Some((level_params, ty.clone())),
            Decl::Inductive(ind) => Some((&ind.level_params, ind.former_type.clone())),
        }
    }

    /// The body of a transparent definition, for δ-unfolding (`11 §4`).
    pub fn transparent_body(&self, id: GlobalId) -> Option<(Vec<LevelVar>, Term)> {
        match self.lookup(id)? {
            Decl::Transparent {
                level_params, body, ..
            } => Some((level_params.clone(), body.clone())),
            _ => None,
        }
    }

    /// The inductive family declaration, if `id` is an inductive former.
    pub fn inductive(&self, id: GlobalId) -> Option<&InductiveDecl> {
        match self.lookup(id)? {
            Decl::Inductive(ind) => Some(ind),
            _ => None,
        }
    }

    /// The parent inductive and constructor index, if `id` is a constructor.
    pub fn constructor(&self, id: GlobalId) -> Option<(&InductiveDecl, usize)> {
        let &(di, ci) = self.ctor_index.get(&id)?;
        let decl = &self.decls[di];
        match decl {
            Decl::Inductive(ind) => Some((ind, ci)),
            _ => None,
        }
    }

    /// Upgrade a pre-admitted `Opaque` declaration in-place to `Transparent`
    /// (with a body) after SCT has approved it (`18 §4`). Returns `false` if
    /// `id` is not present or is not opaque.
    pub fn upgrade_to_transparent(&mut self, id: GlobalId, body: Term) -> bool {
        let Some(&idx) = self.by_id.get(&id) else {
            return false;
        };
        let decl = self.decls[idx].clone();
        if let Decl::Opaque {
            level_params, ty, ..
        } = decl
        {
            self.decls[idx] = Decl::Transparent {
                id,
                level_params,
                ty,
                body,
            };
            true
        } else {
            false
        }
    }

    /// Iterate over all declarations in dependency order (for
    /// `trusted_base()` enumeration, `18 §5`).
    pub fn decls(&self) -> impl Iterator<Item = &Decl> {
        self.decls.iter()
    }

    /// Record a decidable-equality certificate for `prim_ty`. The caller
    /// ([`crate::check::declare_deceq_certificate`]) is responsible for
    /// having admitted `cert.sound`/`cert.complete` as checked postulates
    /// first; this method only records the registry entry.
    pub(crate) fn register_deceq_cert(&mut self, prim_ty: GlobalId, cert: DecEqCert) {
        self.deceq_certs.insert(prim_ty, cert);
    }

    /// The registered decidable-equality certificate for `prim_ty`, if any.
    /// `None` for every unregistered primitive — the fail-safe default is
    /// unchanged.
    pub fn deceq_cert(&self, prim_ty: GlobalId) -> Option<&DecEqCert> {
        self.deceq_certs.get(&prim_ty)
    }

    /// Record that `Term::IntLit` terms have type `id` (ADR 0013 Layer 2).
    /// Set once by the elaborator, alongside the primitive's own
    /// `declare_deceq_certificate` call.
    pub fn register_int_lit_type(&mut self, id: GlobalId) {
        self.int_lit_ty = Some(id);
    }

    /// The primitive type `Term::IntLit` terms belong to, if registered.
    /// `None` before registration — callers (`infer`) must fail closed
    /// rather than assume a default.
    pub fn int_lit_type(&self) -> Option<GlobalId> {
        self.int_lit_ty
    }

    /// The postulates and real primitives in `Σ` — the unchecked assumptions a
    /// program rests on (`18 §5`). The prelude `Top`/`Bottom` constants are
    /// excluded: they are fixed kernel vocabulary (`16 §1.3`), not user
    /// assumptions. Checked surface literals are also excluded: their values
    /// are stored as syntax-derived data, not as primitive operations.
    pub fn trusted_base(&self) -> TrustedBase {
        let entries = self
            .decls
            .iter()
            .filter(|d| match d {
                Decl::Opaque { .. } => true,
                Decl::Primitive { reduction, .. } => *reduction != PrimReduction::Literal,
                _ => false,
            })
            .filter(|d| !self.is_prelude(d.id()))
            .map(|d| match d {
                Decl::Opaque { id, name, .. } => TrustedBaseEntry {
                    id: *id,
                    name: name.clone(),
                },
                Decl::Primitive {
                    id,
                    reduction: PrimReduction::OpaqueType,
                    ..
                } => TrustedBaseEntry {
                    id: *id,
                    name: "opaque primitive type".to_string(),
                },
                Decl::Primitive {
                    id,
                    reduction: PrimReduction::Op { symbol },
                    ..
                } => TrustedBaseEntry {
                    id: *id,
                    name: (*symbol).to_string(),
                },
                Decl::Primitive {
                    reduction: PrimReduction::Literal,
                    ..
                }
                | Decl::Transparent { .. }
                | Decl::Inductive(_) => unreachable!("trusted-base filter mismatch"),
            })
            .collect();
        TrustedBase { entries }
    }
}

impl InductiveDecl {
    /// Build the type former's type `Π Δ_p. Π Δ_i. Type ℓ` and the constructor
    /// types `Π Δ_p. Π Δₖ. D Δ_p t̄ₖ`, populating `former_type` and each
    /// constructor's `type_`. Called at admission after `id`, `level_params`,
    /// `params`, `indices`, `level`, and constructor `args`/`target_indices`
    /// are set.
    ///
    /// `recursive_positions` for each constructor must already be computed
    /// (by the positivity check, [`crate::inductive`]).
    pub fn build_types(&mut self) {
        // Former type: Π Δ_p. Π Δ_i. Type ℓ
        self.former_type = telescope_to_pi(&self.params, {
            telescope_to_pi(&self.indices, Term::Type(self.level.clone()))
        });

        let m = self.params.len();
        let level_args: Vec<Level> = self.level_params.iter().map(|p| Level::Var(*p)).collect();
        let former = Term::IndFormer {
            id: self.id,
            level_args: level_args.clone(),
        };

        for c in &mut self.constructors {
            let n = c.args.len();
            // Body: D Δ_p t̄ₖ, in context Δ_p + Δₖ (depth m + n).
            // Params p₁..pₘ are at de Bruijn indices (n + m - 1) .. n.
            let mut head = former.clone();
            for j in 0..m {
                let idx = n + m - 1 - j;
                head = Term::app(head, Term::var(idx));
            }
            for t in &c.target_indices {
                head = Term::app(head, t.clone());
            }
            c.type_ = telescope_to_pi(&self.params, telescope_to_pi(&c.args, head));
        }
    }
}
