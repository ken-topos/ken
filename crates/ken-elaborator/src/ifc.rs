//! Information-flow control — IFC by typing (`spec/60-security/61`).
//!
//! **Two trusted surfaces** — the kernel does NOT backstop either:
//! - **N1 (by-typing flow rules):** Labels are *erased* before the kernel
//!   (`§3`). A flow bug (wrong `⊑` in `L-SINK`, dropped `pc`-join,
//!   label-dropping `bind`/`incl`) emits a well-typed core the kernel accepts
//!   while NI is violated. The sole net: discriminating flip cases {A1–A4,C1,F1}.
//! - **N2 (by-proof reduction faithfulness):** The kernel re-checks the cert
//!   for the obligation it is *handed*, not its faithfulness to 2-safety. A
//!   wrong reduction (too-weak `Φ_post`, dropped `coterminates_ζ`) yields a
//!   kernel-valid cert for a non-NI claim. Sole net: D5 (interfering→`disproved`).

use crate::extract::ObligationTriple;
use crate::prover::{attempt_obligation, ProverResult, Verdict};

// ─── §2 Label lattice (DLM instance) ─────────────────────────────────────────

/// A security label — scalar `⊥=0 ≤ 1 ≤ ⊤=2`.
///
/// Confidentiality: `Public(0) ⊑ Internal(1) ⊑ Secret(2)`.
/// Integrity (dual): `Trusted(0) ⊑ Untrusted(2)` — `Trusted=⊥`, `Untrusted=⊤`.
/// Labels are **erased** before the kernel (§3); no kernel primitive introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(pub u8);

// DLM standard lattice — named constants (§2.1).
pub const PUBLIC: Label = Label(0);    // ⊥_conf (readable by all)
pub const INTERNAL: Label = Label(1);  // intermediate confidentiality
pub const SECRET: Label = Label(2);    // ⊤_conf (readable by few)

pub const TRUSTED: Label = Label(0);   // ⊥_integ (most trustworthy source)
pub const UNTRUSTED: Label = Label(2); // ⊤_integ (attacker-influenced)

pub const BOTTOM: Label = Label(0);    // ⊥ (pure context, no taint)
pub const TOP: Label = Label(2);       // ⊤

/// `ℓ ⊔ κ` — join (least upper bound); raises to the more sensitive.
/// Conf: `⊔ = ∩` (fewer readers); Integ: `⊔ = ∪` (any taint poisons).
/// Both correspond to `max` on the scalar representation.
pub fn join(a: Label, b: Label) -> Label {
    Label(a.0.max(b.0))
}

/// `ℓ ⊓ κ` — meet (greatest lower bound).
pub fn meet(a: Label, b: Label) -> Label {
    Label(a.0.min(b.0))
}

/// `ℓ ⊑ κ` — "data at level ℓ may flow to a sink with clearance κ".
/// True iff `ℓ ≤ κ` in the scalar order.
/// - Conf: `Public ⊑ Secret` ✓, `Secret ⊑ Public` ✗.
/// - Integ: `Trusted ⊑ Untrusted` ✓, `Untrusted ⊑ Trusted` ✗.
pub fn flows_to(label: Label, clearance: Label) -> bool {
    label.0 <= clearance.0
}

// ─── §5a @ct label — separate opt-in axis ─────────────────────────────────

/// The constant-time label — a separate axis from confidentiality/integrity.
/// A `@ct`-marked value must never steer a leakage-relevant effect sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CtLabel(pub bool);

/// Leakage-relevant effect sinks (§5a) — `@ct` values are barred from these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeakageSink {
    BranchGuard,
    MemoryIndex,
    VarTimePrimitive,
}

/// A `@ct` hook — carries the label and the deferred reify-trigger for timing.
/// Ken provides the source-level precondition `Q`; timing enforcement itself
/// is deferred to `[Sec1ct]`/`[Ward]` (honest limits, §5a/§H).
#[derive(Debug, Clone)]
pub struct CtHook {
    pub ct_label: CtLabel,
    /// `[Sec1ct]`/`[Ward]` reify-trigger — always present for `@ct` values; never
    /// silent (`61 §5a`/`§H`, LP-2).
    pub deferred_timing: Option<&'static str>,
}

impl CtHook {
    pub fn new(is_ct: bool) -> Self {
        CtHook {
            ct_label: CtLabel(is_ct),
            deferred_timing: if is_ct { Some(TRIGGER_SEC1CT) } else { None },
        }
    }
    /// True iff the @ct label is set and the reify-trigger is present.
    pub fn has_reify_trigger(&self) -> bool {
        self.ct_label.0 && self.deferred_timing.is_some()
    }
    /// True iff the label has been carried (survives into the denotation).
    pub fn label_carries(&self) -> bool {
        self.ct_label.0
    }
}

// ─── Flow error + result ──────────────────────────────────────────────────────

/// A flow typing rejection — names the violated rule, labels, and sink site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowError {
    pub rule: &'static str,
    pub data_label: Label,
    pub pc_label: Label,
    pub sink_clearance: Label,
    pub site: String,
}

impl FlowError {
    pub fn new(rule: &'static str, data: Label, pc: Label, clearance: Label, site: &str) -> Self {
        FlowError {
            rule,
            data_label: data,
            pc_label: pc,
            sink_clearance: clearance,
            site: site.to_owned(),
        }
    }
}

/// Result of the flow-typing pass — accept or reject with error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowResult {
    Accept,
    Reject(FlowError),
}

impl FlowResult {
    pub fn is_accept(&self) -> bool { matches!(self, Self::Accept) }
    pub fn is_reject(&self) -> bool { matches!(self, Self::Reject(_)) }
    pub fn error(&self) -> Option<&FlowError> {
        match self { Self::Reject(e) => Some(e), _ => None }
    }
}

// ─── §3 Flow-typing context and the four rules ───────────────────────────────

/// Flow-typing context — holds the current program-counter label `pc`.
///
/// All four flow rules (§3) operate through this context. `pc` tracks which
/// secret branches we are inside (the implicit-flow discipline).
#[derive(Debug, Clone)]
pub struct FlowCtx {
    pub pc: Label,
}

impl FlowCtx {
    /// Start with `pc = ⊥` — no taint in the initial context.
    pub fn new() -> Self { FlowCtx { pc: BOTTOM } }

    pub fn with_pc(pc: Label) -> Self { FlowCtx { pc } }

    /// **L-PURE**: a pure value at any label — no flow constraint, always accepts.
    pub fn l_pure(&self) -> FlowResult {
        FlowResult::Accept
    }

    /// **L-COMBINE** (`61 §3.1`): computing on labeled data joins the labels.
    /// Returns the label of the result: `ℓ₁ ⊔ ℓ₂ ⊔ pc`.
    /// A bug that took `ℓ₁ ⊓ ℓ₂` or only `ℓ₂` would lower the result label,
    /// letting a combined Secret+Public value masquerade as Public (A4 target).
    pub fn l_combine(&self, l1: Label, l2: Label) -> Label {
        join(join(l1, l2), self.pc)
    }

    /// **L-OBSERVE** (`61 §3.1`): branching on a value `@ ℓ` raises `pc` to
    /// `pc ⊔ ℓ`. Returns the new context (the caller enters the branch in it).
    /// A bug that drops the `pc`-raise lets a Secret-keyed branch output to a
    /// Public sink (A3 target — the implicit-flow discriminator).
    pub fn l_observe(&self, value_label: Label) -> FlowCtx {
        FlowCtx { pc: join(self.pc, value_label) }
    }

    /// **L-SINK** (`61 §3.1`): write data `@ ℓ` to a sink with clearance `κ`.
    /// Requires `(ℓ ⊔ pc) ⊑ κ`. The `pc`-join catches implicit flows.
    pub fn l_sink(&self, value_label: Label, clearance: Label, site: &str) -> FlowResult {
        let combined = join(value_label, self.pc);
        if flows_to(combined, clearance) {
            FlowResult::Accept
        } else {
            FlowResult::Reject(FlowError::new("L-SINK", value_label, self.pc, clearance, site))
        }
    }

    /// **L-SINK(@ct)** (`61 §5a`): a `@ct`-marked value flowing to a
    /// leakage-relevant sink is a type error — the source-level CT precondition `Q`.
    /// Ken never proves the timing guarantee itself (that is `[Ward]`).
    pub fn l_ct_sink(&self, ct: CtLabel, _sink: &LeakageSink, site: &str) -> FlowResult {
        if ct.0 {
            FlowResult::Reject(FlowError::new(
                "L-SINK(ct)", TOP, self.pc, BOTTOM, site,
            ))
        } else {
            FlowResult::Accept
        }
    }
}

impl Default for FlowCtx {
    fn default() -> Self { Self::new() }
}

// ─── §3.2 No-laundering through effect routing ───────────────────────────────

/// Check that a `Vis` node's IFC label is preserved through `bind`/`incl`.
///
/// `bind (Vis e f) k = Vis e (λr. …)` reconstructs the **same** `Vis e` node
/// (`36 §2.2`), and `incl` re-tags only the effect tag (`36 §2.4`) — neither
/// touches the label index. So after correct routing, the label is unchanged.
///
/// The targeted bug: a `bind`/`incl`/handler that **drops** the label index
/// (emitting `PUBLIC` instead of `SECRET`), producing a well-typed core the
/// kernel accepts (labels erased — N1). This function is `true` on the correct
/// implementation and `false` on the label-dropping bug.
pub fn check_no_laundering(original_vis_label: Label, after_routing_label: Label) -> bool {
    original_vis_label == after_routing_label
}

// ─── §4 Declassification ─────────────────────────────────────────────────────

/// A declassification capability: `Cap_declassify[ℓ→ℓ']` where `ℓ' ⊑ ℓ`.
/// Capability-gated and audited (`62 §5`); the only way a label moves down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclassifyCap {
    pub from: Label,
    pub to: Label,
}

impl DeclassifyCap {
    pub fn new(from: Label, to: Label) -> Self { DeclassifyCap { from, to } }
    /// `to ⊑ from` and strictly lower (a genuine downgrade).
    pub fn is_valid(&self) -> bool {
        flows_to(self.to, self.from) && self.to != self.from
    }
}

/// Result of a declassification attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclassifyResult {
    Accept { downgraded_label: Label },
    Reject { reason: &'static str },
}

/// Attempt a declassification `ℓ → ℓ'`.
/// Requires a valid `Cap_declassify[ℓ→ℓ']` in scope.
pub fn check_declassify(
    cap: Option<&DeclassifyCap>,
    value_label: Label,
    target_from: Label,
    target_to: Label,
) -> DeclassifyResult {
    match cap {
        None => DeclassifyResult::Reject { reason: "missing Cap_declassify" },
        Some(c) if c.from != target_from || c.to != target_to => {
            DeclassifyResult::Reject { reason: "capability edge mismatch" }
        }
        Some(c) if !c.is_valid() => {
            DeclassifyResult::Reject { reason: "invalid capability: to ⊋ from" }
        }
        Some(c) if value_label != c.from => {
            DeclassifyResult::Reject { reason: "value label does not match capability from-level" }
        }
        Some(c) => DeclassifyResult::Accept { downgraded_label: c.to },
    }
}

/// Check that a declassification authority is present in the `trusted_base_delta`.
///
/// Completeness of the delta is the **sole backstop** for hidden downgrades
/// (`25 §3`, `18 §5`). A package that omits the authority from the delta
/// silently hides the downgrade — B3 names this exact guard.
pub fn check_declassify_in_delta(authority_id: &str, delta: &[String]) -> bool {
    delta.iter().any(|id| id == authority_id)
}

// ─── §5.3 By-proof relational path (basic: D1/D2/D5) ────────────────────────

/// Reify triggers for deferred capabilities (§H, honest limits, LP-2).
/// Every deferred case names its trigger — never silent.
pub const TRIGGER_REL_DEFERRED: &str = "[rel-deferred]";
pub const TRIGGER_SEC1CT: &str = "[Sec1ct]";
pub const TRIGGER_WARD: &str = "[Ward]";

/// `[Sec1-dual]`: genuine `(Conf×Integ)` product lattice + lattice-parametric
/// flow rules, with an A2 that flips on the dual-order independently of A1.
/// Currently `UNTRUSTED=Label(2)=SECRET` and `TRUSTED=Label(0)=PUBLIC` — one
/// scalar; a bug specific to the IntegLabel ordering cannot be distinguished
/// from a ConfLabel bug. The real integrity dual (separate carrier, dual `⊑`)
/// must make A2 flip while A1 stays green.
pub const TRIGGER_SEC1_DUAL: &str = "[Sec1-dual]";

/// `[Sec1-launder]`: wire `check_no_laundering` to real `bind`/`incl`/
/// `handler_fold` in `effects::itree`. The C1 test currently checks
/// label-equality over hand-assigned literals — the actual trusted surface
/// (`36 §2.2/§2.4`: `bind (Vis e f) k = Vis e (λr.…)` preserving the label
/// index) is not exercised. A real `Vis`-routed tree must be the discriminant.
pub const TRIGGER_SEC1_LAUNDER: &str = "[Sec1-launder]";

/// `[Sec1-reduce]`: implement `product(c, ζ)` (variable renaming, `lowEq_ζ`,
/// `coterminates_ζ` conjunct) and tie D5 to a genuine product-program
/// reduction. Currently `check_reduction_faithfulness` is a verdict-shape
/// predicate over a synthetic obligation — a too-weak `Φ_post` (the N2 failure
/// mode) cannot be detected because nothing constructs `Φ_post`.
pub const TRIGGER_SEC1_REDUCE: &str = "[Sec1-reduce]";

/// A relational (non-interference) claim, post-reduction to a unary obligation.
///
/// The **product-program construction** (variable renaming, `lowEq_ζ` /
/// `coterminates_ζ` encoding) is the **trusted step (N2)** — the kernel checks
/// the cert for the handed obligation, not its faithfulness to 2-safety. A wrong
/// reduction yields a kernel-valid cert for a non-NI claim (a false `proved`).
/// The sole net for N2: D5 (`interfering → disproved`).
pub struct RelationalClaim {
    pub view_name: String,
    pub observer_level: Label,
    /// The unary obligation produced by the product-program reduction.
    pub obligation: ObligationTriple,
    /// Deferred capability trigger (if any — `[rel-deferred]`, `[Sec1ct]`…).
    pub deferred_trigger: Option<&'static str>,
}

impl RelationalClaim {
    pub fn new(
        view_name: &str,
        observer: Label,
        obligation: ObligationTriple,
        trigger: Option<&'static str>,
    ) -> Self {
        RelationalClaim {
            view_name: view_name.to_owned(),
            observer_level: observer,
            obligation,
            deferred_trigger: trigger,
        }
    }

    /// Check the NI claim via V3 (kernel re-checks the cert).
    pub fn check(&self, env: &mut ken_kernel::GlobalEnv) -> ProverResult {
        attempt_obligation(env, &self.obligation)
    }

    /// True iff this claim carries a non-silent deferred trigger.
    pub fn has_deferred_trigger(&self) -> bool {
        self.deferred_trigger.is_some()
    }
}

/// Check reduction faithfulness (N2 positive-soundness backstop, D5):
/// a known-interfering program MUST produce `disproved`.
/// Returns `true` iff the verdict is `Disproved` — the sole acceptable outcome.
pub fn check_reduction_faithfulness(verdict: &Verdict) -> bool {
    matches!(verdict, Verdict::Disproved { .. })
}
