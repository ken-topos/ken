//! V4 proof-failure diagnostics — `24 §1`–§7.
//!
//! Turns a non-`proved` V3 verdict (`23 §1.2`) into a **structured,
//! machine-readable diagnostic** an agent can act on. V4 **never re-decides the
//! verdict** — the cardinal rule (`24` preamble): the `false`/`unknown` tag is a
//! **projection** of V3's verdict field, not an independent reading of the
//! evidence.
//!
//! **A V4 bug is infidelity (advisory-UX), never unsoundness (`24 §6`).** The
//! kernel already settled `proved`/not via the certificate (`23 §1.3`); V4 only
//! *renders* the outcome. So the load-bearing guard here is **fidelity**:
//! mislabeling `unknown` as `false`, or emitting a diagnostic for a `proved`
//! goal, or offering an `unknown`-only action on a refuted goal.
//!
//! ## Mechanisms at V4 (per `24 §7`)
//!
//! 1. **Kripke countermodels** (`24 §1`) with the `false`-vs-`unknown`
//!    discriminator read faithfully from V3's verdict. A checked V3
//!    refutation is rendered as a two-stage monotone model; later solver
//!    backends may supply larger models without changing the verdict boundary.
//! 2. **Typed holes** (`24 §2`) as opaque postulates in `trusted_base()`, with
//!    **`unknown` runtime propagation** per the Kleene/Heyting table (`41 §6`).
//!    The `ThirdValue` type and its Kleene operations are the concrete runtime
//!    face.
//! 3. **Three-region Heyting decomposition** (`24 §3`): `proved → S_φ`,
//!    `disproved → S_{¬φ}`, `unknown → unknown` — keyed to V3's verdict, never
//!    independently computed from `{¬¬φ}`.
//! 4. **Slice / missing-hypothesis** (`24 §4`), `unknown`-only: `add_precondition
//!    ψ` when `Γ, ψ ⊢ φ` flips from `unknown` → `proved`. Slice search is
//!    `[placeholder — reifies in V4-backend]`; the action is emitted structurally.
//!
//! **Glivenko / cross-case metatheory invariant** (`24 §3` / `23 §5`): a
//! **classically-valid** goal (e.g. `p ∨ ¬p`, `¬¬p ⇒ p`) is **never** tagged
//! `false` / placed in `S_{¬φ}` — `¬φ` is unprovable, so no world forces it,
//! so the verdict is `unknown`, not `disproved`. This invariant is asserted
//! cross-case by the conformance consistency sweep.
//!
//! `[placeholder — reifies in V4-backend]` marks deferred backend work.

use ken_kernel::{GlobalId, Term};

use crate::extract::{ObligationId, ObligationTriple, Provenance};
use crate::prover::{Countermodel, FormulaPath, FormulaStep, ProverResult, Verdict};

// ─── Kripke model types (24 §1) ──────────────────────────────────────────────

/// Opaque identifier for a Kripke world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldId(pub String);

/// Structural identity for an atomic proposition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomId {
    /// An atom identified by its structural path in the obligation formula.
    Formula(FormRef),
    /// Uninterpreted display text. Validation rejects this variant; it exists
    /// only so wire/display data cannot masquerade as semantic evidence.
    Named(String),
}

/// A reference to a proposition occurrence in φ.
///
/// The empty path inherits proposition role from [`ObligationTriple::phi`].
/// Child steps through binders are preserved as wire identities but rejected
/// by semantic resolution until V4 has typed valuations and substitution.
/// Raw term children such as equality operands are never atoms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormRef(pub FormulaPath);

/// A formula query interpreted by the Kripke forcing relation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KripkeFormula {
    Node(FormRef),
    Not(FormRef),
}

/// The world + subformula witnessing where φ fails in the Kripke model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureWitness {
    pub world: WorldId,
    pub subformula: FormRef,
}

/// A Kripke countermodel for φ (`24 §1`).
///
/// The `verdict` field is **copied from V3** — the cardinal rule.  The
/// `worlds`/`order`/`forcing`/`failure` fields are the model structure.
///
/// **No independent `is_false` flag** (`24 §1`): the `false`/`unknown` tag *is*
/// the `verdict` field, projected from V3.  `disproved` holds iff some world
/// forces `¬φ`; otherwise `unknown`.
///
/// Checked V3 refutations carry a minimal two-stage model. A future solver
/// backend may replace that model with a larger one without changing the
/// verdict boundary.
#[derive(Debug, Clone)]
pub struct KripkeCountermodel {
    /// Copied from V3's verdict (the cardinal rule — never recomputed from
    /// `worlds`/`forcing`).
    pub verdict: DiagnosticTag,
    /// Finite world set; w0 the root.
    pub worlds: Vec<WorldId>,
    /// The ≤ preorder on worlds.
    pub order: Vec<(WorldId, WorldId)>,
    /// Which atoms hold where (monotone in ≤).
    pub forcing: Vec<(WorldId, AtomId)>,
    /// World + subformula where φ breaks.
    pub failure: Option<FailureWitness>,
}

impl KripkeCountermodel {
    /// Build a two-stage model from a V3 refutation.
    ///
    /// V3 supplies the structural formula reference carried beside its
    /// kernel-checked `q : ¬φ`. Atomic forcing is empty for the present unequal
    /// equality: recursive negation holds because no accessible world forces
    /// that equality atom.
    pub fn from_v3(tag: DiagnosticTag, v3: &Countermodel, formula: &Term) -> Result<Self, String> {
        let root = WorldId("w0".to_owned());
        let extension = WorldId("w1".to_owned());
        let model = Self {
            verdict: tag,
            worlds: vec![root.clone(), extension.clone()],
            order: vec![
                (root.clone(), root.clone()),
                (root.clone(), extension.clone()),
                (extension.clone(), extension),
            ],
            forcing: vec![],
            failure: Some(FailureWitness {
                world: root,
                subformula: FormRef(v3.refutation.formula.clone()),
            }),
        };
        model.validate(formula)?;
        Ok(model)
    }

    /// Evaluate a supported formula under the model's Kripke semantics.
    ///
    /// The evaluator is side-effect free and advisory. Its result is never
    /// consulted to choose or alter [`Self::verdict`].
    pub fn forces(
        &self,
        world: &WorldId,
        query: &KripkeFormula,
        root: &Term,
    ) -> Result<bool, String> {
        if !self.worlds.contains(world) {
            return Err(format!("unknown Kripke world {}", world.0));
        }
        match query {
            KripkeFormula::Not(formula) => {
                formula.resolve(root)?;
                for accessible in self.accessible_from(world) {
                    if self.forces(accessible, &KripkeFormula::Node(formula.clone()), root)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            KripkeFormula::Node(formula) => match formula.resolve(root)? {
                Term::Eq(_, _, _) => Ok(self
                    .forcing
                    .contains(&(world.clone(), AtomId::Formula(formula.clone())))),
                Term::Pi(_, _) => Err(
                    "unsupported Kripke formula node: Pi requires a typed valuation, \
                     domain, and binder substitution"
                        .to_owned(),
                ),
                Term::Sigma(_, _) => Err(
                    "unsupported Kripke formula node: Sigma requires a typed domain \
                     and existential witness"
                        .to_owned(),
                ),
                _ => unreachable!("nested FormRef paths accept only proposition nodes"),
            },
        }
    }

    /// Check the structural Kripke invariants rendered by V4.
    ///
    /// This is an advisory-data validator only. Its result must never be used
    /// to change [`Self::verdict`].
    pub fn validate(&self, root: &Term) -> Result<(), String> {
        if self.worlds.is_empty() {
            return Err("Kripke model has no worlds".to_owned());
        }
        for (index, world) in self.worlds.iter().enumerate() {
            if self.worlds[..index].contains(world) {
                return Err(format!("duplicate Kripke world {}", world.0));
            }
            if !self.order.contains(&(world.clone(), world.clone())) {
                return Err(format!("Kripke preorder is not reflexive at {}", world.0));
            }
        }
        for (lower, upper) in &self.order {
            if !self.worlds.contains(lower) || !self.worlds.contains(upper) {
                return Err("Kripke order references a world outside the model".to_owned());
            }
        }
        for (lower, middle) in &self.order {
            for (same_middle, upper) in &self.order {
                if middle == same_middle && !self.order.contains(&(lower.clone(), upper.clone())) {
                    return Err(format!(
                        "Kripke preorder is not transitive: {} <= {} <= {}",
                        lower.0, middle.0, upper.0
                    ));
                }
            }
        }
        for (world, atom) in &self.forcing {
            if !self.worlds.contains(world) {
                return Err(format!(
                    "Kripke forcing references unknown world {}",
                    world.0
                ));
            }
            match atom {
                AtomId::Formula(formula) => {
                    formula.resolve(root)?;
                }
                AtomId::Named(name) => {
                    return Err(format!("Kripke forcing atom is not structural: {name}"));
                }
            }
        }
        for (lower, upper) in &self.order {
            for (_, atom) in self.forcing.iter().filter(|(world, _)| world == lower) {
                if !self.forcing.contains(&(upper.clone(), atom.clone())) {
                    return Err(format!(
                        "Kripke forcing is not monotone: {} forces {:?} but {} does not",
                        lower.0, atom, upper.0
                    ));
                }
            }
        }

        let failure = self
            .failure
            .as_ref()
            .ok_or_else(|| "Kripke refutation has no failure witness".to_owned())?;
        if !self.worlds.contains(&failure.world) {
            return Err("Kripke failure witness references an unknown world".to_owned());
        }
        failure.subformula.resolve(root)?;
        if !self.forces(
            &failure.world,
            &KripkeFormula::Not(failure.subformula.clone()),
            root,
        )? {
            return Err("Kripke witness world does not force the negated formula".to_owned());
        }
        Ok(())
    }

    fn accessible_from<'a>(&'a self, world: &'a WorldId) -> impl Iterator<Item = &'a WorldId> {
        self.order
            .iter()
            .filter_map(move |(lower, upper)| (lower == world).then_some(upper))
    }
}

impl FormRef {
    pub fn root() -> Self {
        Self(FormulaPath::root())
    }

    pub fn child(&self, step: FormulaStep) -> Self {
        let mut path = self.0.clone();
        path.0.push(step);
        Self(path)
    }

    fn resolve<'a>(&self, root: &'a Term) -> Result<&'a Term, String> {
        for step in &self.0 .0 {
            match (step, root) {
                (FormulaStep::PiDomain, Term::Pi(_, _))
                | (FormulaStep::PiCodomain, Term::Pi(_, _))
                | (FormulaStep::SigmaFirst, Term::Sigma(_, _))
                | (FormulaStep::SigmaSecond, Term::Sigma(_, _)) => {
                    return Err(format!(
                        "formula path enters unsupported binder role {step:?}"
                    ));
                }
                (FormulaStep::AppFunction, Term::App(_, _))
                | (FormulaStep::AppArgument, Term::App(_, _))
                | (FormulaStep::EqType, Term::Eq(_, _, _))
                | (FormulaStep::EqLeft, Term::Eq(_, _, _))
                | (FormulaStep::EqRight, Term::Eq(_, _, _)) => {
                    return Err(format!("formula path enters non-proposition role {step:?}"));
                }
                _ => {
                    return Err(format!(
                        "invalid formula path {:?} at term {root:?}",
                        self.0
                    ));
                }
            }
        }
        if !is_proposition_node(root) {
            return Err(format!(
                "formula path {:?} resolves to a non-proposition term {root:?}",
                self.0
            ));
        }
        Ok(root)
    }
}

fn is_proposition_node(term: &Term) -> bool {
    matches!(term, Term::Eq(_, _, _) | Term::Pi(_, _) | Term::Sigma(_, _))
}

// ─── Typed hole (24 §2) ──────────────────────────────────────────────────────

/// A stable identifier for a typed hole, derived from the obligation's
/// provenance (`22 §1`).  Wraps `GlobalId` from `declare_postulate`.
///
/// **Determinism note** (`24 §6`): hole ids must be stable functions of the
/// provenance, not allocation order.  At V4-build, the id equals the
/// `declare_postulate` result (allocation-order) — provenance-keyed ids are
/// `[placeholder — V4-backend]` pending a deterministic id scheme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoleId(pub GlobalId);

/// A typed hole `?h : φ` in context Γ (`24 §2`).
///
/// The hole is already admitted as an opaque postulate (`declare_postulate`,
/// `18 §4.2`) by V3, enumerated in `trusted_base()`.  V4 surfaces it as a
/// structured value an agent can query and fill.
#[derive(Debug, Clone)]
pub struct TypedHole {
    /// Stable id (derived from provenance `22 §1`).
    pub id: HoleId,
    /// The obligation's goal φ.
    pub goal: Term,
    /// The obligation's open context Γ.
    pub context: Vec<Term>,
    /// Provenance from V2 (`22 §1`): where the obligation originated.
    pub origin: Provenance,
}

// ─── Tag, region, actions ─────────────────────────────────────────────────────

/// The `false`/`unknown` tag on a diagnostic — **projected from V3's verdict**
/// (cardinal rule, `24` preamble).  There is no fourth value.
///
/// `proved` goals emit **no** diagnostic and this tag never appears for them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticTag {
    /// V3 `disproved` → `false`: some world forces `¬φ`; the goal is genuinely
    /// refutable.  *Fix the code or spec.*
    False,
    /// V3 `unknown` → `unknown`: information is absent; the goal is in the
    /// `¬¬φ ⇒ φ` gap.  *Supply more facts.*
    Unknown,
}

/// The three-region Heyting decomposition (`24 §3`), **keyed to V3's verdict**
/// (cardinal rule) — never independently recomputed from `{¬¬φ}` set membership.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Region {
    /// `proved` → `S_φ`.
    Proved,
    /// `disproved` → `S_{¬φ}` (genuinely refutable).
    Refuted,
    /// `unknown` → the `¬¬φ` gap (characteristic core `{¬¬φ} ∖ S_φ`).
    Unknown,
}

/// A region-tagged suggested action (`24 §5`).
///
/// Actions are **region-tagged**: `unknown`-only actions NEVER appear on a
/// `false`-tagged diagnostic (the §4 fidelity constraint — offering
/// `add_precondition` on a genuinely-refuted goal is a fidelity bug).
#[derive(Debug, Clone)]
pub enum SuggestedAction {
    // ── unknown-region only (24 §4/§5) ──────────────────────────────────
    /// `Γ, h : ψ ⊢ φ` makes V3 return `proved` — add this precondition.
    /// `hypothesis` is `[placeholder — V4-backend]` until slice search lands.
    AddPrecondition { hypothesis: Term },
    /// Narrow an input refinement `{x:A | φ'}`.
    StrengthenRefinement { parameter: Term },
    /// Provide a lemma whose statement closes the hole.
    ProvideLemma { statement: Term },
    /// Case-split on an expression.
    CaseSplit { expression: Term },
    /// Use induction on a variable.
    InductOn { variable: Term },
    // ── false-region only (24 §1/§3) ─────────────────────────────────────
    /// The input class that genuinely fails — fix the code or spec.
    FixCounterexample { description: String },
}

// ─── The diagnostic value ─────────────────────────────────────────────────────

/// A single proof-failure diagnostic (`24 §1`–§6).
///
/// For each verdict that is **not** `proved`, `project_diagnostic` emits one
/// `Diagnostic`.  `proved` → `None` (`24 §7` AC5).
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The obligation this diagnostic explains.
    pub obligation_id: ObligationId,
    /// The tag — **copied from V3's verdict** (cardinal rule, `24` preamble).
    /// This is the `verdict` field of the `KripkeCountermodel` (`24 §1`);
    /// V4 never recomputes it.
    pub tag: DiagnosticTag,
    /// The three-region Heyting decomposition (`24 §3`), keyed to the tag.
    pub region: Region,
    /// Kripke countermodel when the disproved formula is in V4's supported
    /// diagnostic fragment. Unsupported formulas retain the copied false tag
    /// and fall back to an explicit pending explanation. `24 §1`.
    pub countermodel: Option<KripkeCountermodel>,
    /// Typed hole (present for `unknown`).  `24 §2`.
    pub typed_hole: Option<TypedHole>,
    /// Region-tagged suggested actions (`24 §5`).
    pub suggested_actions: Vec<SuggestedAction>,
}

// ─── Cardinal-rule projection ─────────────────────────────────────────────────

/// Project one V3 verdict into a diagnostic value (`24` preamble).
///
/// **Cardinal rule**: `proved` → `None` (no diagnostic); `disproved` → `False`
/// tag; `unknown` → `Unknown` tag.  The tag is **always the copied verdict field**
/// — never recomputed from the model.  V4 never relabels `unknown` as `false`.
///
/// This is the soundness bridge of V4: if this function projects faithfully,
/// no V4 bug can relabel a verdict.
pub fn project_diagnostic(result: &ProverResult, triple: &ObligationTriple) -> Option<Diagnostic> {
    match &result.verdict {
        // proved → no diagnostic (24 §7 AC5, 24 preamble table row 1)
        Verdict::Proved { .. } => None,

        // disproved → false tag, S_{¬φ} region, countermodel (24 §1)
        Verdict::Disproved { countermodel } => {
            let tag = DiagnosticTag::False;
            let cm = KripkeCountermodel::from_v3(tag.clone(), countermodel, &triple.phi).ok();
            Some(Diagnostic {
                obligation_id: result.obligation_id.clone(),
                tag,
                region: Region::Refuted,
                countermodel: cm,
                typed_hole: None,
                // false-region only: fix_counterexample (24 §5)
                // NO add_precondition / strengthen_refinement / etc.
                suggested_actions: vec![SuggestedAction::FixCounterexample {
                    description: countermodel.description.clone(),
                }],
            })
        }

        // unknown → unknown tag, unknown region, typed hole (24 §1/§2)
        Verdict::Unknown { hole_id } => {
            let typed_hole = TypedHole {
                id: HoleId(*hole_id),
                goal: triple.phi.clone(),
                context: triple.context.clone(),
                origin: triple.provenance.clone(),
            };
            Some(Diagnostic {
                obligation_id: result.obligation_id.clone(),
                tag: DiagnosticTag::Unknown,
                region: Region::Unknown,
                countermodel: None,
                typed_hole: Some(typed_hole),
                // unknown-region actions (24 §4/§5).
                // add_precondition ψ: ψ is [placeholder — V4-backend] (slice
                // search finds ψ from §1's unforced countermodel atom).
                // The structural contract (§4 sufficiency MUST) is that adding
                // ψ flips the verdict unknown → proved; the search is deferred.
                suggested_actions: vec![SuggestedAction::AddPrecondition {
                    hypothesis: triple.phi.clone(), // [placeholder — V4-backend]
                }],
            })
        }
    }
}

/// Project a batch of V3 results into diagnostics, in obligation order.
///
/// `proved` results contribute `None` and are filtered; the returned `Vec`
/// contains only non-proved verdicts (`24 §7` AC5).
pub fn project_all(results: &[ProverResult], triples: &[ObligationTriple]) -> Vec<Diagnostic> {
    results
        .iter()
        .zip(triples.iter())
        .filter_map(|(r, t)| project_diagnostic(r, t))
        .collect()
}

// ─── Kleene/Heyting runtime `unknown` (24 §2, 41 §6) ───────────────────────

/// The runtime third value `unknown` (`41 §6`, `42 §4`).
///
/// `Known(b)` = a definite Boolean; `Unknown` = the runtime face of an open
/// typed hole.  Kleene operations (`tv_and`, `tv_or`, `tv_not`) implement the
/// **verbatim `41 §6` table** — `24 §2` adds no rule `41 §6` omits, so the two
/// files cannot contradict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThirdValue {
    Known(bool),
    Unknown,
}

/// `∧` — absorbing operand is `false` (`41 §6`).
///
/// ```text
/// unknown ∧ false = false   (absorption — absorbing operand decides)
/// unknown ∧ true  = unknown (no decision)
/// known(a) ∧ known(b) = known(a && b)
/// ```
///
/// The connective is **non-strict in the absorbing position** (`42 §4` —
/// the CBV-laziness carry): `unknown ∧ false` does NOT force the hole.
pub fn tv_and(a: ThirdValue, b: ThirdValue) -> ThirdValue {
    match (a, b) {
        (ThirdValue::Known(false), _) | (_, ThirdValue::Known(false)) => ThirdValue::Known(false),
        (ThirdValue::Known(true), ThirdValue::Known(true)) => ThirdValue::Known(true),
        _ => ThirdValue::Unknown,
    }
}

/// `∨` — absorbing operand is `true` (`41 §6`).
///
/// ```text
/// unknown ∨ true  = true    (absorption)
/// unknown ∨ false = unknown (no decision)
/// known(a) ∨ known(b) = known(a || b)
/// ```
pub fn tv_or(a: ThirdValue, b: ThirdValue) -> ThirdValue {
    match (a, b) {
        (ThirdValue::Known(true), _) | (_, ThirdValue::Known(true)) => ThirdValue::Known(true),
        (ThirdValue::Known(false), ThirdValue::Known(false)) => ThirdValue::Known(false),
        _ => ThirdValue::Unknown,
    }
}

/// `¬` — `unknown` is not decidable, so negation propagates (`41 §6`).
///
/// ```text
/// ¬ unknown   = unknown
/// ¬ known(b)  = known(!b)
/// ```
///
/// Note: propositional `⇒` is `Π`/`apply` (`16 §1.3`), covered by the
/// strict-position `apply unknown u = unknown` — it is not a separate row.
pub fn tv_not(a: ThirdValue) -> ThirdValue {
    match a {
        ThirdValue::Known(b) => ThirdValue::Known(!b),
        ThirdValue::Unknown => ThirdValue::Unknown,
    }
}

/// Strict positions propagate `unknown` (`41 §6`, `42 §4`).
///
/// `apply unknown u = unknown`, `elimReduce … unknown = unknown`,
/// `primReduce op (…unknown…) = unknown`, `(a, unknown).2 = unknown`, etc.
///
/// Any strict position with an `Unknown` operand yields `Unknown`.
pub fn tv_strict(v: ThirdValue) -> ThirdValue {
    match v {
        ThirdValue::Unknown => ThirdValue::Unknown,
        other => other, // strict but known — passes through
    }
}
