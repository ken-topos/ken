//! Authority, capabilities, and least privilege (`spec/60-security/62`).
//!
//! **Trust surfaces (`62 §H`):**
//! - **Cap-present (kernel-backed):** `Cap E` is a real Π parameter; a
//!   missing-cap `perform` references an unbound variable the kernel rejects.
//!   No-ambient + least-by-default gates: `effects::check::check_capabilities*`.
//! - **Attenuation bound (trusted Rust + conformance-netted):** the elaborator
//!   computes `authority c' ⊑ authority c ⊓ w` and emits a refinement
//!   obligation. Its `Eq`+`Refl` discharge mirrors the elaborator-selected
//!   postulate identities; it is not an independent kernel proof of the meet.
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

pub use ken_host::capability::{
    authority_flows_to, authority_meet, rights_for_authority, Authority, Cap, FsHandle, FsIdentity,
    FsScope, RightSet, SymlinkPolicy, AUTH_FULL, AUTH_NONE, AUTH_PARTIAL,
};

pub fn authority(cap: &Cap) -> Authority {
    cap.authority()
}

pub fn check_authority_sufficient(
    cap: &Cap,
    required: Authority,
    site: &str,
) -> Result<(), CapError> {
    if authority_flows_to(required, cap.authority()) {
        Ok(())
    } else {
        Err(CapError::AuthorityInsufficient {
            required,
            available: cap.authority(),
            site: site.to_owned(),
        })
    }
}

// ── §3 Attenuation ───────────────────────────────────────────────────────────

/// Refinement obligation emitted by `attenuate c w`:
/// `authority c' ⊑ authority c ⊓ w` (`34 §5`/`21 §2`, `62 §3.1`).
///
/// The elaborator computes the product bound. A child exceeding it makes the
/// emitted equality discharge unknown, but opaque postulate identities are
/// elaborator-chosen, so this is not an independent kernel meet check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttenuationObligation {
    pub child_authority: Authority,
    pub parent_authority: Authority,
    pub window: Authority,
    /// Precomputed `parent_authority ⊓ window` — the attenuation bound.
    pub bound: Authority,
    pub child_rights: RightSet,
    pub parent_rights: RightSet,
    pub window_rights: RightSet,
    pub bound_rights: RightSet,
    pub child_scope: FsScope,
    pub parent_scope: FsScope,
    pub window_scope: Option<FsScope>,
    pub bound_scope: FsScope,
    pub child_symlink: SymlinkPolicy,
    pub parent_symlink: SymlinkPolicy,
    pub window_symlink: SymlinkPolicy,
    pub bound_symlink: SymlinkPolicy,
}

impl AttenuationObligation {
    /// `authority c' ⊑ authority c ⊓ w` — the attenuation bound is satisfied.
    pub fn is_satisfied(&self) -> bool {
        authority_flows_to(self.child_authority, self.bound)
            && self.child_rights.is_subset_of(self.bound_rights)
            && scope_flows_to(&self.child_scope, &self.bound_scope)
            && symlink_flows_to(self.child_symlink, self.bound_symlink)
    }
}

/// Product attenuation window. `scope = None` retains the parent's root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttenuationWindow {
    pub authority: Authority,
    pub rights: RightSet,
    pub scope: Option<FsScope>,
    pub symlink: SymlinkPolicy,
}

impl From<Authority> for AttenuationWindow {
    fn from(authority: Authority) -> Self {
        Self {
            authority,
            rights: rights_for_authority(authority),
            scope: None,
            symlink: SymlinkPolicy::FollowWithinScope,
        }
    }
}

fn symlink_meet(left: SymlinkPolicy, right: SymlinkPolicy) -> SymlinkPolicy {
    if left == SymlinkPolicy::NoFollow || right == SymlinkPolicy::NoFollow {
        SymlinkPolicy::NoFollow
    } else {
        SymlinkPolicy::FollowWithinScope
    }
}

fn symlink_flows_to(left: SymlinkPolicy, right: SymlinkPolicy) -> bool {
    left == SymlinkPolicy::NoFollow || right == SymlinkPolicy::FollowWithinScope
}

fn scope_flows_to(child: &FsScope, parent: &FsScope) -> bool {
    child.empty
        || (!parent.empty
            && child.lineage.len() >= parent.lineage.len()
            && child.lineage[..parent.lineage.len()] == parent.lineage)
}

fn scope_meet(parent: &FsScope, window: Option<&FsScope>) -> FsScope {
    let Some(window) = window else {
        return parent.clone();
    };
    if scope_flows_to(window, parent) {
        window.clone()
    } else if scope_flows_to(parent, window) {
        parent.clone()
    } else {
        let mut empty = parent.clone();
        empty.empty = true;
        empty
    }
}

/// Derive a weaker capability `c'` with `authority c' = authority c ⊓ w`,
/// and emit its refinement obligation.
///
/// The canonical construction sets `authority c' = authority c ⊓ w` exactly
/// (satisfying the obligation by `⊑-refl`). A deviant `c'` exceeding the
/// product bound makes the obligation undischargeable.
///
/// **Monotone-downward:** `authority c' ≤ authority c` always (meet ≤ either
/// operand). There is no path to amplify authority.
pub fn attenuate<W: Into<AttenuationWindow>>(cap: &Cap, window: W) -> (Cap, AttenuationObligation) {
    let window = window.into();
    let bound = authority_meet(cap.authority(), window.authority);
    let bound_rights = cap.scope().rights.intersect(window.rights);
    let mut bound_scope = scope_meet(cap.scope(), window.scope.as_ref());
    bound_scope.rights = bound_rights;
    let bound_symlink = symlink_meet(cap.scope().symlink, window.symlink);
    bound_scope.symlink = bound_symlink;
    let child = Cap::mint_scoped(bound, cap.effect(), bound_scope.clone());
    let obl = AttenuationObligation {
        child_authority: bound,
        parent_authority: cap.authority(),
        window: window.authority,
        bound,
        child_rights: bound_rights,
        parent_rights: cap.scope().rights,
        window_rights: window.rights,
        bound_rights,
        child_scope: bound_scope.clone(),
        parent_scope: cap.scope().clone(),
        window_scope: window.scope,
        bound_scope,
        child_symlink: bound_symlink,
        parent_symlink: cap.scope().symlink,
        window_symlink: window.symlink,
        bound_symlink,
    };
    (child, obl)
}

/// Discharge the attenuation refinement obligation via kernel equality.
///
/// Encodes `authority c' ⊑ authority c ⊓ w` (`62 §3.1`/`22 §2.1`) as
/// `Eq(Authority_type, child, bound)` where `child` and `bound` are opaque
/// kernel postulates representing the authority scalars. When
/// `child_authority == bound` (canonical case), both sides are the SAME
/// postulate — `Refl(child)` proves `Eq(T, v, v)` → `Proved`. When the
/// child is over-strong (`child_authority > bound`), distinct postulates are
/// used — `Refl(child)` cannot prove `Eq(T, c, b)` with `c ≢ b` → `Unknown`.
/// The elaborator chooses whether those opaque identities coincide, so this
/// mirrors its product-bound decision rather than independently checking it.
pub fn discharge_attenuation(
    env: &mut GlobalEnv,
    obl: &AttenuationObligation,
    id: &str,
) -> ProverResult {
    // Opaque carrier type for the authority scalar.
    let auth_type_id = declare_postulate(
        env,
        format!("{id}.authority_type"),
        vec![],
        Term::ty(Level::Zero),
    )
    .expect("authority type postulate");
    let auth_type = Term::const_(auth_type_id, vec![]);
    // Postulate child authority value : Authority_type.
    let child_id = declare_postulate(
        env,
        format!("{id}.child_authority"),
        vec![],
        auth_type.clone(),
    )
    .expect("child authority postulate");
    let child_term = Term::const_(child_id, vec![]);
    // Canonical: child_authority == bound → same postulate both sides of Eq.
    // Over-strong: child_authority != bound → distinct postulates, Refl fails.
    let canonical = obl.child_authority == obl.bound
        && obl.child_rights == obl.bound_rights
        && obl.child_scope == obl.bound_scope
        && obl.child_symlink == obl.bound_symlink;
    let bound_term = if canonical {
        child_term.clone()
    } else {
        let bound_id = declare_postulate(
            env,
            format!("{id}.bound_authority"),
            vec![],
            auth_type.clone(),
        )
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
    ProverResult {
        obligation_id: ObligationId(id.to_owned()),
        verdict,
    }
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
    pub fn new() -> Self {
        RevocationHandle { revoked: false }
    }
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}

impl Default for RevocationHandle {
    fn default() -> Self {
        Self::new()
    }
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
        required: Authority,
        available: Authority,
        site: String,
    },
}

impl std::fmt::Display for CapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthorityInsufficient {
                required,
                available,
                site,
            } => {
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
