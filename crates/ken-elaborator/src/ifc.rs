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

/// A security label — a product of three factors (§2.2, §5a.1).
///
/// - `conf`: confidentiality — `Public(0) ⊑ Internal(1) ⊑ Secret(2)`.
/// - `integ`: integrity (scalar; `[Sec1-dual]` defers the true order-dual carrier) —
///   `Trusted(0) ⊑ Untrusted(2)`.
/// - `ct`: constant-time taint (`§5a.1`) — `ct⊥=false` (safe), `ct⊤=true` (@ct,
///   timing-sensitive). **Taint orientation**: sink demands `ct⊥`; `ct⊤ ⋢ ct⊥` → reject.
///   Join (`⊔`) is logical-OR (any `@ct` input ⇒ `@ct` result).
///
/// Ordering is **componentwise** (§2.2 products and §5a.1 CT axis are independent).
/// Labels are **erased** before the kernel (§3); no kernel primitive introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label {
    pub conf: u8,  // confidentiality: 0=PUBLIC/⊥, 1=INTERNAL, 2=SECRET/⊤
    pub integ: u8, // integrity: 0=TRUSTED/⊥, 2=UNTRUSTED/⊤ ([Sec1-dual]: scalar only)
    pub ct: bool,  // CT taint: false=ct⊥ (safe), true=ct⊤ (@ct, timing-sensitive)
}

// DLM standard lattice — named constants (§2.1).
pub const PUBLIC: Label = Label {
    conf: 0,
    integ: 0,
    ct: false,
}; // ⊥_conf (readable by all)
pub const INTERNAL: Label = Label {
    conf: 1,
    integ: 0,
    ct: false,
}; // intermediate confidentiality
pub const SECRET: Label = Label {
    conf: 2,
    integ: 0,
    ct: false,
}; // ⊤_conf (readable by few)

pub const TRUSTED: Label = Label {
    conf: 0,
    integ: 0,
    ct: false,
}; // ⊥_integ (most trustworthy)
pub const UNTRUSTED: Label = Label {
    conf: 0,
    integ: 2,
    ct: false,
}; // ⊤_integ (attacker-influenced)

pub const BOTTOM: Label = Label {
    conf: 0,
    integ: 0,
    ct: false,
}; // ⊥ (pure context, no taint)
pub const TOP: Label = Label {
    conf: 2,
    integ: 2,
    ct: true,
}; // ⊤ (fully secret+untrusted+ct)

/// `ct⊥` — the safe CT level; what a leakage sink demands as its clearance.
pub const CT_BOT: Label = Label {
    conf: 0,
    integ: 0,
    ct: false,
};
/// `ct⊤` — the `@ct` taint; a timing-sensitive value that must not steer a `LeakSink`.
pub const CT_TOP: Label = Label {
    conf: 0,
    integ: 0,
    ct: true,
};

/// `ℓ ⊔ κ` — componentwise join (§2.2 product lattice + §5a.1 CT axis).
/// - conf/integ: `max` (raises to the more sensitive).
/// - ct: logical-OR (any `@ct` input ⇒ `@ct` result; cannot compute `@ct` away).
pub fn join(a: Label, b: Label) -> Label {
    Label {
        conf: a.conf.max(b.conf),
        integ: a.integ.max(b.integ),
        ct: a.ct || b.ct,
    }
}

/// `ℓ ⊓ κ` — componentwise meet (§2.2).
pub fn meet(a: Label, b: Label) -> Label {
    Label {
        conf: a.conf.min(b.conf),
        integ: a.integ.min(b.integ),
        ct: a.ct && b.ct,
    }
}

/// `ℓ ⊑ κ` — componentwise "flows to" (§2.2 product order).
/// True iff every component flows to the corresponding clearance component:
/// - conf: `ℓ.conf ≤ κ.conf`  (`Public ⊑ Secret` ✓, `Secret ⊑ Public` ✗)
/// - integ: `ℓ.integ ≤ κ.integ` (`Trusted ⊑ Untrusted` ✓, reverse ✗)
/// - ct: `!ℓ.ct || κ.ct`  (`ct⊥ ⊑ ct⊥` ✓, `ct⊤ ⊑ ct⊥` ✗ — taint → safe is blocked)
pub fn flows_to(label: Label, clearance: Label) -> bool {
    label.conf <= clearance.conf && label.integ <= clearance.integ && (!label.ct || clearance.ct)
}

// ─── §5a @ct label — separate opt-in axis ─────────────────────────────────

/// The constant-time label — a thin wrapper; the CT state now lives in `Label.ct`.
/// Kept for backward-compat with `CtHook`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CtLabel(pub bool);

/// Leakage-relevant effect sinks (§5a.2) — a **sealed sum**, exactly three members.
/// There is **no `_ => non-sink` catch-all** (COORDINATION §7): a new leaky op
/// must be classified here before it compiles, never silently default to non-sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeakageSink {
    /// The scrutinee of a control-flow branch (if/match) lowered to a machine branch.
    BranchGuard,
    /// A data-dependent memory/array index feeding an indexing `Vis` op.
    MemoryIndex,
    /// A primitive whose run time depends on operand value (flagged `var-time` in effect sig).
    VarTimePrimitive,
}

/// A known `Vis`-op class for the exhaustive leakage-sink classifier.
/// Every potentially-leaky op MUST appear here — adding one outside this set is a
/// compile error, never a silent non-sink (§5a.2, COORDINATION §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisOpClass {
    ControlFlowBranch, // scrutinee of if/match → BranchGuard
    ArrayIndex,        // data-dependent memory index → MemoryIndex
    VarTimePrimitive,  // primitive flagged var-time in effect sig → VarTimePrimitive
    PureOp,            // pure/constant-time — no timing leak
    CtByteEq,          // CT-safe equality primitive — no timing leak
}

/// Classify a `Vis`-op site into its leakage-sink class (or `None` for safe ops).
/// No `_ =>` catch-all: every `VisOpClass` variant is classified explicitly.
pub fn classify_vis_op(op: VisOpClass) -> Option<LeakageSink> {
    match op {
        VisOpClass::ControlFlowBranch => Some(LeakageSink::BranchGuard),
        VisOpClass::ArrayIndex => Some(LeakageSink::MemoryIndex),
        VisOpClass::VarTimePrimitive => Some(LeakageSink::VarTimePrimitive),
        VisOpClass::PureOp => None,
        VisOpClass::CtByteEq => None,
    }
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
        // Sec1ct now lands the @ct discipline; the remaining deferred aspect is
        // the binary timing guarantee, delegated to [Ward] (61 §5a.6/§H).
        CtHook {
            ct_label: CtLabel(is_ct),
            deferred_timing: if is_ct { Some(TRIGGER_WARD) } else { None },
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
    pub fn is_accept(&self) -> bool {
        matches!(self, Self::Accept)
    }
    pub fn is_reject(&self) -> bool {
        matches!(self, Self::Reject(_))
    }
    pub fn error(&self) -> Option<&FlowError> {
        match self {
            Self::Reject(e) => Some(e),
            _ => None,
        }
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
    pub fn new() -> Self {
        FlowCtx { pc: BOTTOM }
    }

    pub fn with_pc(pc: Label) -> Self {
        FlowCtx { pc }
    }

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
    ///
    /// Projected onto **all components** (§5a.3): if `ℓ.ct = ct⊤` (`@ct` scrutinee),
    /// `pc.ct` is raised to `ct⊤` in both branches, closing the implicit CT channel.
    /// A bug that drops the `pc.ct`-raise lets a `@ct`-guarded inner op through
    /// (CT-A4 target — the implicit-flow discriminator).
    pub fn l_observe(&self, value_label: Label) -> FlowCtx {
        FlowCtx {
            pc: join(self.pc, value_label),
        }
    }

    /// **L-SINK** (`61 §3.1`): write data `@ ℓ` to a sink with clearance `κ`.
    /// Requires `(ℓ ⊔ pc) ⊑ κ`. The `pc`-join catches implicit flows.
    pub fn l_sink(&self, value_label: Label, clearance: Label, site: &str) -> FlowResult {
        let combined = join(value_label, self.pc);
        if flows_to(combined, clearance) {
            FlowResult::Accept
        } else {
            FlowResult::Reject(FlowError::new(
                "L-SINK",
                value_label,
                self.pc,
                clearance,
                site,
            ))
        }
    }

    /// **L-CT-SINK** (`61 §5a.3`): a `@ct` value — or any value inside a `@ct`-guarded
    /// branch (`pc.ct = ct⊤`) — reaching a leakage-relevant sink is a type error.
    ///
    /// Checks `(ℓ.ct ⊔ pc.ct) = ct⊥`, i.e. `!(value_label.ct || self.pc.ct)`.
    /// On failure: names `ℓ.ct`, `pc.ct`, and the sink site (per `L-SINK`'s
    /// diagnostic contract, §3.1).
    ///
    /// The `_sink` parameter is accepted (for caller exhaustiveness via `LeakageSink`)
    /// but the rejection condition is purely label-based (`61 §5a.3`).
    ///
    /// The observable is elaboration accept/reject — **never** a V3 verdict (§5a.3).
    /// Ken proves only the source-level precondition `Q`; timing is `[Ward]`'s (§5a.6).
    pub fn l_ct_sink(&self, value_label: &Label, _sink: &LeakageSink, site: &str) -> FlowResult {
        if value_label.ct || self.pc.ct {
            FlowResult::Reject(FlowError::new(
                "L-CT-SINK",
                *value_label,
                self.pc,
                CT_BOT,
                site,
            ))
        } else {
            FlowResult::Accept
        }
    }
}

impl Default for FlowCtx {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn new(from: Label, to: Label) -> Self {
        DeclassifyCap { from, to }
    }
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
        None => DeclassifyResult::Reject {
            reason: "missing Cap_declassify",
        },
        Some(c) if c.from != target_from || c.to != target_to => DeclassifyResult::Reject {
            reason: "capability edge mismatch",
        },
        Some(c) if !c.is_valid() => DeclassifyResult::Reject {
            reason: "invalid capability: to ⊋ from",
        },
        Some(c) if value_label != c.from => DeclassifyResult::Reject {
            reason: "value label does not match capability from-level",
        },
        Some(c) => DeclassifyResult::Accept {
            downgraded_label: c.to,
        },
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

// ─── §5a.4 CT-in-parameter promise + `Q` export ──────────────────────────────

/// A CT-in-parameter signature promise. The concept is locked (§5a.4):
/// constant-time-in-a-named-parameter, source-level, paired 1:1 with a
/// `63 §5a` Ward discharge result. `(oracle)` — the literal field-token
/// spelling is B1/`71`-deferred (defer-spelling-not-concept).
#[derive(Debug, Clone)]
pub struct CtPromise {
    pub param_name: String,
    /// Always `true` — this is a source-level precondition, NOT a timing guarantee.
    pub source_level: bool,
}

/// A source-level CT guarantee clause (`Q`) emitted by a checked CT promise.
/// Rides the `71` `guarantees` (`Q`) channel (`71 §2`). Pairs 1:1 with a
/// `63 §5a` Ward discharge result. `(oracle)` — field-token spelling is B1/`71`-deferred.
#[derive(Debug, Clone)]
pub struct CtGuaranteeQ {
    /// The named parameter the promise covers.
    pub param_name: String,
    /// Always `true` — source-level precondition, NOT a timing guarantee.
    pub source_level: bool,
}

/// Check a CT-in-parameter promise and emit a `Q` clause if the body is clean.
///
/// The body is checked externally (with the named parameter bound at `ct⊤`) and
/// the result passed here. If the body `Accept`s, the promise is discharged and
/// a `Q` clause is emitted. If it `Reject`s (a `LeakSink` op's operand depended
/// on the `@ct` parameter), the promise is broken and the error is returned.
pub fn check_ct_promise(
    promise: &CtPromise,
    body_result: FlowResult,
) -> Result<CtGuaranteeQ, FlowError> {
    match body_result {
        FlowResult::Accept => Ok(CtGuaranteeQ {
            param_name: promise.param_name.clone(),
            source_level: true,
        }),
        FlowResult::Reject(e) => Err(e),
    }
}

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
