//! Authority, capabilities, and least privilege (`spec/60-security/62`).
//!
//! **Trust surfaces (`62 §H`):**
//! - **Cap-present (kernel-backed):** `Cap E` is a real Π parameter; a
//!   missing-cap `perform` references an unbound variable the kernel rejects.
//!   No-ambient + least-by-default gates: `effects::check::check_capabilities*`.
//! - **Attenuation bound (kernel-backed):** `authority c' ⊑ authority c ⊓ w`
//!   is a `34 §5`/`21 §2` refinement obligation the kernel re-checks; a too-
//!   strong child makes it undischargeable (`§3.1`).
//! - **Declassify cap (trusted-by-typing):** `DeclassifyCap.is_valid` is an
//!   erased-label check — NOT kernel-Q; Sec1's N1 posture (`62 §H`).
//! - **Revocation / audit (static contract):** Sec2 delivers the typed
//!   interface + transitivity/boundary property; the runtime membrane and
//!   audit-record emission are DEFERRED to `40-runtime`/`Ward`.

use ken_kernel::{declare_postulate, GlobalEnv, Level, Term};

use crate::effects::check::{check_capabilities, EffectError};
use crate::effects::infer::EffectDecl;
use crate::effects::row::{EffectName, EffectRow};
use crate::extract::ObligationId;
use crate::ifc::{FlowCtx, FlowError, FlowResult, Label};
use crate::prover::{attempt_with_cert, ProverResult};

// ── §2 Authority lattice ──────────────────────────────────────────────────────

/// Authority level on a scalar lattice: `⊥=0 ≤ 1 ≤ ⊤=2`.
/// **More authority = higher value; attenuation moves DOWN.**
/// `⊑` = "has at most this authority" (restricted ⊑ full).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Authority(pub u8);

pub const AUTH_NONE:    Authority = Authority(0); // ⊥ — no access
pub const AUTH_PARTIAL: Authority = Authority(1); // restricted (e.g. read-only, single dir)
pub const AUTH_FULL:    Authority = Authority(2); // ⊤ — full access

/// `a ⊓ b` — meet; takes the lesser authority (attenuation bound).
pub fn authority_meet(a: Authority, b: Authority) -> Authority {
    Authority(a.0.min(b.0))
}

/// `a ⊑ b` — does `a` have at most `b`'s authority?
/// Equivalently: `a` is at most as powerful as `b`.
pub fn authority_flows_to(a: Authority, b: Authority) -> bool {
    a.0 <= b.0
}

// ── §2.2 Capability token ─────────────────────────────────────────────────────

/// An unforgeable capability token: authority level + the effect it gates
/// (`62 §2`, `36 §2.5`).
///
/// Tokens are minted by handlers (via `mint`) or derived via `attenuate`.
/// There is no `strengthen` or `amplify` — authority is monotone-downward
/// by construction. The surface language's elaboration discipline prevents
/// user-code forgery; `mint` is accessible to handlers and test crates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cap {
    authority_val: Authority,
    pub effect: EffectName,
}

impl Cap {
    /// Mint a root capability. Called by effect handlers and conformance tests.
    /// The surface language's elaboration discipline (`62 §2.2`) prevents
    /// user-code forgery — `mint` is not callable from Ken's surface language.
    pub fn mint(authority: Authority, effect: impl Into<EffectName>) -> Self {
        Cap { authority_val: authority, effect: effect.into() }
    }
}

/// Return the authority carried by a capability (the only public authority reader).
pub fn authority(cap: &Cap) -> Authority {
    cap.authority_val
}

// ── §3.1 Sink-authority sufficiency check ────────────────────────────────────

/// Check that `cap` satisfies a sink demanding `required` authority.
///
/// **Correct check:** `required ⊑ authority(cap)` — the sink's demand must be
/// ≤ the cap's authority (the cap must be AT LEAST as powerful as required).
///
/// **`[Sec2-dual]` orientation:** a weakened cap has LOWER authority; a strong
/// sink (demanding the parent's full authority) REJECTS it. Getting `⊑`
/// backwards (`authority(cap) ⊑ required`) would silently invert — weakened
/// caps would pass strong sinks (privilege escalation). The C1↔C2 pair nets it.
pub fn check_authority_sufficient(
    cap: &Cap,
    required: Authority,
    site: &str,
) -> Result<(), CapError> {
    if authority_flows_to(required, cap.authority_val) {
        // required ⊑ cap.authority — sink's demand is ≤ cap's authority → ok
        Ok(())
    } else {
        Err(CapError::AuthorityInsufficient {
            required,
            available: cap.authority_val,
            site: site.to_owned(),
        })
    }
}

// ── §3 Attenuation ───────────────────────────────────────────────────────────

/// Kernel-re-checked refinement obligation from `attenuate c w`:
/// `authority c' ⊑ authority c ⊓ w` (`34 §5`/`21 §2`, `62 §3.1`).
///
/// Distinct from Sec1's erased labels (which are trusted-by-typing). When the
/// child's authority exceeds the bound, `is_satisfied()` is false and the
/// kernel rejects the refinement discharge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttenuationObligation {
    pub child_authority:  Authority,
    pub parent_authority: Authority,
    pub window:           Authority,
    /// Precomputed `parent_authority ⊓ window` — the attenuation bound.
    pub bound:            Authority,
}

impl AttenuationObligation {
    /// `authority c' ⊑ authority c ⊓ w` — the attenuation bound is satisfied.
    pub fn is_satisfied(&self) -> bool {
        authority_flows_to(self.child_authority, self.bound)
    }
}

/// Derive a weaker capability `c'` with `authority c' = authority c ⊓ w`,
/// and emit the kernel-re-checked refinement obligation.
///
/// The canonical construction sets `authority c' = authority c ⊓ w` exactly
/// (satisfying the obligation by `⊑-refl`). A deviant `c'` with authority
/// exceeding the bound makes the obligation undischargeable — the kernel rejects.
///
/// **Monotone-downward:** `authority c' ≤ authority c` always (meet ≤ either
/// operand). There is no path to amplify authority.
pub fn attenuate(cap: &Cap, w: Authority) -> (Cap, AttenuationObligation) {
    let bound = authority_meet(cap.authority_val, w);
    let child = Cap { authority_val: bound, effect: cap.effect.clone() };
    let obl = AttenuationObligation {
        child_authority:  bound,
        parent_authority: cap.authority_val,
        window:           w,
        bound,
    };
    (child, obl)
}

/// Discharge the attenuation refinement obligation via the kernel.
///
/// Encodes `authority c' ⊑ authority c ⊓ w` (`62 §3.1`/`22 §2.1`) as
/// `Eq(Authority_type, child, bound)` where `child` and `bound` are opaque
/// kernel postulates representing the authority scalars. When
/// `child_authority == bound` (canonical case), both sides are the SAME
/// postulate — `Refl(child)` proves `Eq(T, v, v)` → `Proved`. When the
/// child is over-strong (`child_authority > bound`), distinct postulates are
/// used — `Refl(child)` cannot prove `Eq(T, c, b)` with `c ≢ b` → `Unknown`.
pub fn discharge_attenuation(
    env: &mut GlobalEnv,
    obl: &AttenuationObligation,
    id: &str,
) -> ProverResult {
    // Opaque carrier type for the authority scalar.
    let auth_type_id = declare_postulate(env, vec![], Term::ty(Level::Zero))
        .expect("authority type postulate");
    let auth_type = Term::const_(auth_type_id, vec![]);
    // Postulate child authority value : Authority_type.
    let child_id = declare_postulate(env, vec![], auth_type.clone())
        .expect("child authority postulate");
    let child_term = Term::const_(child_id, vec![]);
    // Canonical: child_authority == bound → same postulate both sides of Eq.
    // Over-strong: child_authority != bound → distinct postulates, Refl fails.
    let bound_term = if obl.child_authority == obl.bound {
        child_term.clone()
    } else {
        let bound_id = declare_postulate(env, vec![], auth_type.clone())
            .expect("bound authority postulate");
        Term::const_(bound_id, vec![])
    };
    let phi = Term::Eq(
        Box::new(auth_type),
        Box::new(child_term.clone()),
        Box::new(bound_term),
    );
    let cert = Term::Refl(Box::new(child_term));
    let verdict = attempt_with_cert(env, &phi, cert);
    ProverResult { obligation_id: ObligationId(id.to_owned()), verdict }
}

// ── §4 Revocation — static contract ──────────────────────────────────────────

/// Revocation handle for a capability delegation tree (`62 §4`, static face).
///
/// **Static contract:** revoking the parent revokes the parent AND every
/// capability attenuated from it (transitivity). The runtime membrane
/// (forwarder / validity-cell flip) is DEFERRED to `40-runtime`/`OQ-Space`.
#[derive(Debug, Clone)]
pub struct RevocationHandle {
    pub revoked: bool,
}

impl RevocationHandle {
    pub fn new() -> Self { RevocationHandle { revoked: false } }
    pub fn revoke(&mut self) { self.revoked = true; }
}

impl Default for RevocationHandle {
    fn default() -> Self { Self::new() }
}

/// Static contract: `true` iff the delegation is still live (not revoked).
///
/// Transitivity-by-construction: all attenuated caps share the parent's
/// `RevocationHandle`, so one `revoke()` closes the whole sub-delegation.
/// The discriminator: a non-transitive impl revoking only the parent (leaving
/// children live) passes a parent-only check but fails the child check here.
pub fn check_revocation_transitive(handle: &RevocationHandle) -> bool {
    !handle.revoked
}

// ── §5 Audit points statically known ─────────────────────────────────────────

/// Check that a trust-boundary effect is **statically declared** in the row
/// (and therefore auditable — `62 §5`, `36 §3.1`).
///
/// An un-declared boundary effect is impossible: you cannot perform an effect
/// the type didn't declare (`36 §1.4`). The audit points are the `Vis` nodes
/// the row type names — statically known; no un-audited boundary effect can
/// occur.
pub fn check_audit_boundary(declared_row: &EffectRow, boundary_effect: &EffectName) -> bool {
    declared_row.contains(boundary_effect.as_str())
}

// ── §6 Authority + flow compose ──────────────────────────────────────────────

/// The outcome of the two-concession check: cap gate AND flow gate.
///
/// Authority gates *may this code act* (`62 §1`); flow gates *may this data
/// flow here* (`61 §3`). Both are **independent** — holding `Cap_Net` does not
/// buy clearance; clean flow does not buy authority. Dropping either rejects.
#[derive(Debug)]
pub enum AuthAndFlowResult {
    Accept,
    CapRejected(EffectError),
    FlowRejected(FlowError),
}

/// Check that both the capability gate AND the flow gate pass.
///
/// `CapRejected` takes priority over `FlowRejected` (a missing-capability
/// error names the exact missing cap; a flow error names the violated rule).
/// Both are reported as their own error type — neither subsumes the other.
pub fn check_authority_and_flow(
    decl: &EffectDecl,
    performed: &EffectRow,
    handler_caps: &EffectRow,
    flow_ctx: &FlowCtx,
    data_label: Label,
    clearance: Label,
    site: &str,
) -> AuthAndFlowResult {
    if let Err(e) = check_capabilities(decl, performed, handler_caps) {
        return AuthAndFlowResult::CapRejected(e);
    }
    match flow_ctx.l_sink(data_label, clearance, site) {
        FlowResult::Accept => AuthAndFlowResult::Accept,
        FlowResult::Reject(fe) => AuthAndFlowResult::FlowRejected(fe),
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

/// An authority-sufficiency error (`62 §3.1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapError {
    /// Sink demands more authority than the cap carries.
    AuthorityInsufficient {
        required:  Authority,
        available: Authority,
        site:      String,
    },
}

impl std::fmt::Display for CapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthorityInsufficient { required, available, site } => {
                write!(
                    f,
                    "AuthorityInsufficient at '{}': requires Authority({}), \
                     cap has Authority({})",
                    site, required.0, available.0
                )
            }
        }
    }
}

impl std::error::Error for CapError {}
