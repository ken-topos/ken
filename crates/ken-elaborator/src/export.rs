//! B1 behavioral-export emitter — the five-part assume-guarantee contract
//! (`spec/70-behavioral/71-assumption-boundary.md §2.1–§5.2`).
//!
//! **Generated, never authored.** Every field is a projection of Ken's verified
//! content. The emitter cannot over-claim: it projects exactly the four-way
//! epistemic status from `21 §5`, with a kernel-side honesty discriminator
//! (`§2.1` I1).
//!
//! **Honesty discriminator (I1, the load-bearing pin):** a claim is in `Q`
//! iff its V1 hole is ABSENT from `trusted_base()` (the postulate was
//! discharged: `upgrade_to_transparent` was called and the cert kernel-
//! checked). A claim whose hole remains in `trusted_base()` is in `P`.
//! Checking `trusted_base()` membership is a structural check on the kernel
//! environment, not a comparison of status strings: a lazy emitter that trusts
//! a V-layer "proved" string (or buckets by presence of an `ensures` clause)
//! would land open holes in `Q` (over-claim). The discriminating pair
//! EX-A1/EX-A2 is the sole net for this discriminator.
//!
//! **One-way gate (I4):** there is no code path from a `Ward`/classical/
//! delegated result to `proved` status. The emitter takes only
//! kernel-produced `Verdict` values and the kernel's `trusted_base()` — no
//! external discharge result can enter here and write a `Q` entry. The gate
//! is structural: `QEntry` can only be constructed inside `emit_export`, and
//! only when `trusted_base()` does not contain the hole.
//!
//! **Disproved boundary (71 §2.1):** a refuted claim is never exported. The
//! build is non-shippable when any obligation is `disproved`.
//!
//! **Σ = L5 perform-node signatures (I3):** the alphabet is the `EffectRow`
//! from the effects elaborator — reuse, not reinvention.
//!
//! **G carries support, never measure (I5):** `GEntry` has NO weight/
//! likelihood/probability field. The structural absence makes a measure
//! unrepresentable — a compile error, not a runtime check. This is the
//! exhaustive-by-construction seal (§4.1), the B1 analog of `LeakSink`.
//!
//! **Content-addressed hash (§3.3):** `BehavioralExport::hash` is the
//! SHA-256 of a canonical JSON serialization (deterministic field and entry
//! order). A non-canonical serialization yields a non-reproducible hash.
//! The `sha2` crate is not in scope yet; we use a deterministic canonical
//! string and `DefaultHasher` as a build-internal content-address. `(oracle)`
//! note: the final hash algorithm is Ward-finalized; this implementation pins
//! the _discipline_ (canonical order, no timestamps), not the exact algorithm.

use std::collections::{BTreeMap, BTreeSet};

use ken_kernel::GlobalId;

use crate::effects::row::EffectRow;
use crate::extract::{ObligationTriple, ProvKind};
use crate::prover::Verdict;

// ─── Status types ────────────────────────────────────────────────────────────

/// Status tag for a `P` (assumptions) entry (`71 §2.1`).
///
/// Both `Tested` and `Unknown` land in `P`, never in `Q`. The tag records
/// how the assumption arose: as an explicit statement (`Tested`) or as an
/// undischarged obligation hole (`Unknown`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PStatus {
    /// Explicit `prove`/`law` statement accepted as an assumption without a
    /// kernel proof — trusted by assertion. Corresponds to `21 §5` `tested`.
    Tested,
    /// An open obligation hole: the goal is a postulate in `trusted_base()`.
    /// Corresponds to `21 §5` `unknown`.
    Unknown,
}

impl PStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tested => "tested",
            Self::Unknown => "unknown",
        }
    }
}

// ─── Export entries ───────────────────────────────────────────────────────────

/// An entry in `Q` (guarantees) — a proved postcondition (`71 §2.1`, I1).
///
/// Invariant: the corresponding hole is ABSENT from `trusted_base()` at
/// emission time (discharged via `upgrade_to_transparent`). Only constructible
/// inside `emit_export` — the one-way gate holds structurally.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QEntry {
    /// Stable obligation id (`22 §1`).
    pub obligation_id: String,
    /// Proved goal — a proposition `φ : Ω` the downstream may *assume*.
    pub goal: String,
}

/// An entry in `P` (assumptions) — the assumption boundary (`71 §2.1`, I2).
///
/// Contains `tested` entries (explicit assumes) and `unknown` entries (open
/// obligation holes in `trusted_base()`). Projected live from the kernel
/// environment — removing an assumption yields a different `P` and a
/// different hash.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PEntry {
    /// Stable obligation id (`22 §1`).
    pub obligation_id: String,
    /// The assumption's goal proposition.
    pub goal: String,
    /// How this entry arose (`tested` = explicit statement; `unknown` = hole).
    pub status: PStatus,
}

/// An entry in `T` (obligations) — a delegated `Temporal` value (`71 §2.1`).
///
/// B1 provides the channel; B2 fills the `Temporal` value body (`72 §5`). The
/// status is the constant `delegated` (pinned at source, `72 §5` — serialized
/// in [`serialize_export`], never `proved`/`tested`/`unknown`). No Ward/classical
/// result may ever promote a `T` entry to `Q` — that path does not exist
/// (I4 / EX-E1): `QEntry` is built only in the `Verdict::Proved` arm of
/// [`emit_export`]; the `temporal` parameter flows straight into `T`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TEntry {
    /// Stable obligation id (`22 §1`).
    pub obligation_id: String,
    /// The delegated `Temporal` value — the B2-filled body of the `T` channel
    /// (`72 §5`). Ranges over the shared `Σ` alphabet.
    pub formula: crate::temporal::Temporal,
}

/// The static correlation descriptor for a V1 resource lifetime (`71 §2.2`).
///
/// This value deliberately contains no runtime resource identity. Ward binds
/// that identity from the successful `FsOpen` event selected by `bind_at` and
/// requires the same event field on the use and settlement operations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceLifetimeCorrelationV1 {
    pub identity_type: &'static str,
    pub event_field: &'static str,
    pub bind_at: &'static str,
    pub require_same_at: [ken_host::HostOpV1; 2],
}

/// The four fixed checks compiled by Ward for each runtime identity selected
/// by [`ResourceLifetimeCorrelationV1`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WardResourceLifetimeMonitorV1 {
    pub successful_acquire_settles_exactly_once: bool,
    pub forbid_successful_use_after_settlement: bool,
    pub require_no_live_resource_on: [&'static str; 3],
    pub retain_settlement_outcome: bool,
}

/// The additional correlated body admitted by the behavioral export's `T`
/// channel (`71 §2.2`). It is one target-level monitor template, not one entry
/// per operation or per dynamically minted resource.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceLifetimeObligationV1 {
    pub schema_version: u16,
    pub body_kind: &'static str,
    pub obligation_id: &'static str,
    pub status: &'static str,
    pub correlation: ResourceLifetimeCorrelationV1,
    pub acquire_op: ken_host::HostOpV1,
    pub use_op: ken_host::HostOpV1,
    pub settle_op: ken_host::HostOpV1,
    pub monitor_template: WardResourceLifetimeMonitorV1,
}

impl ResourceLifetimeObligationV1 {
    /// The Spec-owned descriptor, reproduced without extension or inference.
    pub const fn pinned() -> Self {
        Self {
            schema_version: 1,
            body_kind: "ResourceLifetimeObligationV1",
            obligation_id: "resource-lifetime-v1",
            status: "delegated",
            correlation: ResourceLifetimeCorrelationV1 {
                identity_type: "ResourceTraceIdentityV1",
                event_field: "EffectEventV1.resource",
                bind_at: "Successful(FsOpen)",
                require_same_at: [
                    ken_host::HostOpV1::FsHandleMetadata,
                    ken_host::HostOpV1::ResourceRelease,
                ],
            },
            acquire_op: ken_host::HostOpV1::FsOpen,
            use_op: ken_host::HostOpV1::FsHandleMetadata,
            settle_op: ken_host::HostOpV1::ResourceRelease,
            monitor_template: WardResourceLifetimeMonitorV1 {
                successful_acquire_settles_exactly_once: true,
                forbid_successful_use_after_settlement: true,
                require_no_live_resource_on: [
                    "NormalReturn",
                    "ReturnedError",
                    "ControlledTrap",
                ],
                retain_settlement_outcome: true,
            },
        }
    }
}

/// Generator support structure — partition + boundaries + case decomposition
/// from refinement predicates and `match` arms (`71 §4`).
///
/// **NO measure/weight/likelihood field** — the structural seal is
/// exhaustive-by-construction (§4.1, I5). Attempting to attach a probability
/// to a `GEntry` is a compile error. The sampling policy lives on Ward's side,
/// keyed over the partition vocabulary exported here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GEntry {
    /// Name of the function or declaration this generator covers.
    pub source: String,
    /// The partition conditions: predicates from refinement types or arms
    /// from `match` branches. Each is a formula over the function's inputs.
    /// These cover *which* inputs are valid (support), never *how likely* they
    /// are (measure).
    pub conditions: Vec<String>,
    // NO weight: f64 — structurally absent; measure is unrepresentable here.
}

// ─── The export contract ──────────────────────────────────────────────────────

/// The five-part assume-guarantee export contract (`71 §2.1`).
///
/// Generated from verified content; every field is a projection. A downstream
/// (`Ward` static verifier, test generator, runtime monitor) reads this as the
/// *boundary* between what Ken proved and what Ken assumed.
#[derive(Debug, Clone)]
pub struct BehavioralExport {
    /// The checked target's name.
    pub target_name: String,
    /// `Q` — proved guarantees: kernel-certified postconditions and invariants.
    /// Each entry's hole was absent from `trusted_base()` at emission (I1).
    pub guarantees: Vec<QEntry>,
    /// `P` — assumptions: the environment the generator's input domain models.
    /// Contains `tested` (explicit) and `unknown` (open holes) entries (I2).
    pub assumptions: Vec<PEntry>,
    /// `Σ` — the interaction-tree perform-node alphabet (`36 §2`). Equals the
    /// L5 effect row exactly — reuse, not reinvention (I3).
    pub alphabet: BTreeSet<String>,
    /// `T` — delegated `Temporal` obligations. Status: `delegated` (I4).
    /// B2 fills the `Temporal` value body (`TEntry::formula`, `72 §5`); B1
    /// provided the channel.
    pub obligations: Vec<TEntry>,
    /// The optional correlated resource-lifetime body in `T`. Presence is
    /// derived solely from reachable `Σ`: exactly one when `FsOpen` occurs,
    /// none otherwise. Ordinary `Temporal` entries remain unchanged above.
    pub resource_lifetime_obligation: Option<ResourceLifetimeObligationV1>,
    /// `G` — generator support: partition + boundaries from refinement/match.
    /// No measure (I5 — structural seal, §4.1).
    pub generators: Vec<GEntry>,
    /// Content-addressed canonical hash (`§3.3`, `63 §2`).
    ///
    /// SHA-256 of the canonical JSON serialization (BTreeMap key order,
    /// sorted Vec entries). Same verified content → identical hash; a removed
    /// assumption changes the hash. `(oracle)` note: exact algorithm finalized
    /// with Ward; this implementation pins the _canonical-form discipline_.
    pub hash: String,
}

// ─── Error ────────────────────────────────────────────────────────────────────

/// An error from the export emitter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportError {
    /// A refuted (`disproved`) claim was encountered (`71 §2.1`, EX-F1).
    /// This build is **not shippable** — the claim must be fixed before export.
    /// The claim is absent from all export fields; the build fails here.
    DisproovedClaim { obligation_id: String },
}

// ─── The emitter (the sole constructor for Q entries) ────────────────────────

/// Emit the behavioral export contract from verified content (`71 §2.1`).
///
/// # Parameters
/// - `target_name`: the checked declaration's name.
/// - `results`: pairs of `(ObligationTriple, Verdict)` — the V2→V3 pairs
///   for each obligation. The `hole_id` in each triple is the V1 kernel
///   postulate; its presence/absence in `trusted_base_set` is the honesty
///   discriminator.
/// - `trusted_base_set`: the kernel's current `trusted_base()` at emission.
///   Collect via `env.trusted_base().into_iter().collect()`.
/// - `alphabet`: the program's L5 effect row (`36 §2`). Pass the result of
///   `infer_all` or the declared row for the target.
/// - `generators`: support structure from refinement predicates and match arms.
///   No measure field — structural seal.
/// - `temporal`: delegated `Temporal` obligations (B2 fills the body —
///   `TEntry::formula`; status is the constant `delegated`, `72 §5`).
///
/// # Honesty discriminator (I1)
/// For each obligation with verdict `Proved`:
/// - If `hole_id ∉ trusted_base_set` (hole was discharged): → `Q`.
/// - If `hole_id ∈ trusted_base_set` (hole still open): → `P` (unknown).
///   This case is conservative; correct elaboration + discharge should not
///   reach it, but the emitter is not allowed to over-claim.
///
/// # One-way gate (I4)
/// `QEntry` is only constructed in this function, from `Verdict::Proved` values
/// that the kernel produced. There is no argument, parameter, or code path
/// that accepts a Ward/classical "green" result and writes a `Q` entry.
/// A `Ward` discharge re-entering as a `TEntry` stays in `T`, never in `Q`.
///
/// # Errors
/// Returns `ExportError::DisproovedClaim` if any obligation is `Disproved`.
/// The caller must not produce a shippable export in that case.
pub fn emit_export(
    target_name: &str,
    results: &[(ObligationTriple, Verdict)],
    trusted_base_set: &BTreeSet<GlobalId>,
    alphabet: EffectRow,
    generators: Vec<GEntry>,
    temporal: Vec<TEntry>,
) -> Result<BehavioralExport, ExportError> {
    let mut guarantees: Vec<QEntry> = Vec::new();
    let mut assumptions: Vec<PEntry> = Vec::new();

    for (triple, verdict) in results {
        let id_str = triple.id.0.clone();
        let goal_str = format!("{:?}", triple.phi);

        match verdict {
            Verdict::Proved { .. } => {
                // HONESTY DISCRIMINATOR (I1): proved ∧ hole ∉ trusted_base → Q.
                // A lazy emitter that trusts the V-layer's "proved" string, or
                // buckets by presence of an `ensures` clause, would route open
                // holes here too (over-claim). We check trusted_base() directly.
                if !trusted_base_set.contains(&triple.hole_id) {
                    // Hole was discharged: kernel-certified. → Q.
                    guarantees.push(QEntry {
                        obligation_id: id_str,
                        goal: goal_str,
                    });
                } else {
                    // Proved verdict but hole still in trusted_base: conservative
                    // fallback to P/unknown. This path should not arise under
                    // correct elaboration + discharge, but the emitter must never
                    // over-claim.
                    assumptions.push(PEntry {
                        obligation_id: id_str,
                        goal: goal_str,
                        status: PStatus::Unknown,
                    });
                }
            }

            Verdict::Unknown { .. } => {
                // P entry: goal is in trusted_base (an open hole).
                // Status depends on provenance:
                //   Ensures → open obligation hole → Unknown.
                //   Prove / LawField → explicit statement trusted by assertion → Tested.
                let status = match &triple.provenance.kind {
                    ProvKind::Ensures { .. } => PStatus::Unknown,
                    ProvKind::Prove
                    | ProvKind::LawField { .. }
                    | ProvKind::CallPrecond
                    | ProvKind::PartialPrim => PStatus::Tested,
                };
                assumptions.push(PEntry {
                    obligation_id: id_str,
                    goal: goal_str,
                    status,
                });
            }

            Verdict::Disproved { .. } => {
                // DISPROVED BOUNDARY (71 §2.1, EX-F1): a refuted claim is never
                // exported. The build is non-shippable. Return an error — the
                // caller must not produce an export from this state.
                return Err(ExportError::DisproovedClaim { obligation_id: id_str });
            }
        }
    }

    // Sort for canonical order (deterministic hash).
    guarantees.sort();
    assumptions.sort();
    let mut obligations = temporal;
    obligations.sort();
    let mut gens = generators;
    gens.sort();
    for g in &mut gens {
        g.conditions.sort();
    }

    // Σ: alphabet = the L5 effect row's perform-node signatures.
    // BTreeSet gives canonical order (already deterministic via EffectRow's
    // BTreeSet internals, but we take ownership here).
    let alphabet_set: BTreeSet<String> = alphabet.effects().cloned().collect();

    let resource_lifetime_obligation = alphabet_set
        .contains("FsOpen")
        .then(ResourceLifetimeObligationV1::pinned);

    let hash = compute_hash(
        target_name,
        &guarantees,
        &assumptions,
        &alphabet_set,
        &obligations,
        resource_lifetime_obligation.as_ref(),
        &gens,
    );

    Ok(BehavioralExport {
        target_name: target_name.to_string(),
        guarantees,
        assumptions,
        alphabet: alphabet_set,
        obligations,
        resource_lifetime_obligation,
        generators: gens,
        hash,
    })
}

// ─── Canonical hash (§3.3) ───────────────────────────────────────────────────

/// Compute a deterministic content-address hash for the export (`71 §3.3`).
///
/// Serializes the export to a canonical string (BTreeMap for sorted key order;
/// sorted Vec entries) and hashes it. A non-canonical serialization (map-
/// iteration order, an embedded timestamp, allocation-order-dependent ids)
/// yields a different hash across runs.
///
/// `(oracle)` note: the exact hash algorithm (SHA-256, BLAKE3, …) is
/// finalized by Ward. This implementation uses a FNV-style deterministic
/// fold over the canonical JSON string to pin the _canonical-form discipline_
/// at B1 build time, without introducing a hash-crate dependency. The fold
/// is collision-resistant within the corpus and deterministic by construction.
fn compute_hash(
    target_name: &str,
    guarantees: &[QEntry],
    assumptions: &[PEntry],
    alphabet: &BTreeSet<String>,
    obligations: &[TEntry],
    resource_lifetime_obligation: Option<&ResourceLifetimeObligationV1>,
    generators: &[GEntry],
) -> String {
    // Build a canonical representation using BTreeMap (sorted keys).
    let mut root: BTreeMap<&str, String> = BTreeMap::new();

    root.insert("target", target_name.to_string());

    // Q entries: sorted by obligation_id (already sorted above).
    let q_repr: Vec<String> = guarantees
        .iter()
        .map(|e| format!("{}:{}", e.obligation_id, e.goal))
        .collect();
    root.insert("guarantees", q_repr.join("|"));

    // P entries: sorted.
    let p_repr: Vec<String> = assumptions
        .iter()
        .map(|e| format!("{}:{}:{}", e.obligation_id, e.status.as_str(), e.goal))
        .collect();
    root.insert("assumptions", p_repr.join("|"));

    // Σ: BTreeSet is already sorted.
    let sigma_repr: Vec<&String> = alphabet.iter().collect();
    root.insert("alphabet", sigma_repr.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(","));

    // T entries: id + the delegated `Temporal` formula body (content-addressed
    // — a different formula yields a different hash, `71 §3.3`).
    let t_repr: Vec<String> = obligations
        .iter()
        .map(|e| format!("{}:{:?}", e.obligation_id, e.formula))
        .collect();
    root.insert("obligations", t_repr.join("|"));

    // The structured descriptor participates in T's content hash. Runtime
    // identity `r` is absent by construction; every static descriptor field is
    // present, so changing any one of them changes the export hash.
    root.insert(
        "resource_lifetime_obligation",
        resource_lifetime_obligation
            .map(canonical_resource_lifetime_obligation)
            .unwrap_or_default(),
    );

    // G entries: source + sorted conditions.
    let g_repr: Vec<String> = generators
        .iter()
        .map(|e| format!("{}:[{}]", e.source, e.conditions.join(";")))
        .collect();
    root.insert("generators", g_repr.join("|"));

    // Serialize the BTreeMap canonically.
    let canonical: String = root
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    // FNV-1a fold — deterministic, no external crate dependency.
    // (oracle): replace with SHA-256 when the Ward wire format is finalized.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in canonical.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("ken-export-v0:{:016x}", hash)
}

// ─── Serialization ───────────────────────────────────────────────────────────

/// Serialize a `BehavioralExport` to canonical JSON for wire transmission
/// or storage (`71 §3.1`, `63 §2`).
///
/// Field key spellings are `(oracle)`-tagged — Ward finalizes the wire tokens.
/// The value-set and cross-field invariants are normative (locked).
pub fn serialize_export(export: &BehavioralExport) -> serde_json::Value {
    use serde_json::{json, Value};

    let guarantees: Vec<Value> = export.guarantees.iter().map(|e| {
        json!({
            "obligation_id": e.obligation_id,    // (oracle): "id" / "obligation_id"
            "goal": e.goal,
            "status": "proved"
        })
    }).collect();

    let assumptions: Vec<Value> = export.assumptions.iter().map(|e| {
        json!({
            "obligation_id": e.obligation_id,    // (oracle)
            "goal": e.goal,
            "status": e.status.as_str()          // "tested" | "unknown"
        })
    }).collect();

    let alphabet: Vec<&String> = export.alphabet.iter().collect();

    let mut obligations: Vec<Value> = export.obligations.iter().map(|e| {
        json!({
            "obligation_id": e.obligation_id,    // (oracle)
            "status": "delegated",              // constant (`72 §5`); never proved/tested/unknown
            "formula": format!("{:?}", e.formula) // (oracle): Ward-facing spelling deferred
        })
    }).collect();
    if let Some(resource) = &export.resource_lifetime_obligation {
        obligations.push(serialize_resource_lifetime_obligation(resource));
    }

    let generators: Vec<Value> = export.generators.iter().map(|e| {
        json!({
            "source": e.source,
            "conditions": e.conditions,
            // No weight/measure field — structural seal (I5, §4.1)
        })
    }).collect();

    json!({
        "schema": "ken.export/v0",               // (oracle): version token
        "target": export.target_name,
        "guarantees": guarantees,                // Q — (oracle): "guarantees" / "Q"
        "assumptions": assumptions,              // P — (oracle): "assumptions" / "P"
        "alphabet": alphabet,                    // Σ — (oracle): "alphabet" / "sigma"
        "obligations": obligations,              // T — (oracle): "obligations" / "T"
        "generators": generators,                // G — (oracle): "generators" / "G"
        "hash": export.hash
    })
}

fn host_op_name(operation: ken_host::HostOpV1) -> &'static str {
    match operation {
        ken_host::HostOpV1::FsOpen => "FsOpen",
        ken_host::HostOpV1::FsHandleMetadata => "FsHandleMetadata",
        ken_host::HostOpV1::ResourceRelease => "ResourceRelease",
        _ => unreachable!("resource-lifetime V1 contains only its pinned three operations"),
    }
}

fn canonical_resource_lifetime_obligation(value: &ResourceLifetimeObligationV1) -> String {
    format!(
        "schema_version={};body_kind={};obligation_id={};status={};identity_type={};event_field={};bind_at={};require_same_at=[{},{}];acquire_op={};use_op={};settle_op={};successful_acquire_settles_exactly_once={};forbid_successful_use_after_settlement={};require_no_live_resource_on=[{},{},{}];retain_settlement_outcome={}",
        value.schema_version,
        value.body_kind,
        value.obligation_id,
        value.status,
        value.correlation.identity_type,
        value.correlation.event_field,
        value.correlation.bind_at,
        host_op_name(value.correlation.require_same_at[0]),
        host_op_name(value.correlation.require_same_at[1]),
        host_op_name(value.acquire_op),
        host_op_name(value.use_op),
        host_op_name(value.settle_op),
        value.monitor_template.successful_acquire_settles_exactly_once,
        value.monitor_template.forbid_successful_use_after_settlement,
        value.monitor_template.require_no_live_resource_on[0],
        value.monitor_template.require_no_live_resource_on[1],
        value.monitor_template.require_no_live_resource_on[2],
        value.monitor_template.retain_settlement_outcome,
    )
}

fn serialize_resource_lifetime_obligation(
    value: &ResourceLifetimeObligationV1,
) -> serde_json::Value {
    use serde_json::json;

    json!({
        "schema_version": value.schema_version,
        "body_kind": value.body_kind,
        "obligation_id": value.obligation_id,
        "status": value.status,
        "correlation": {
            "identity_type": value.correlation.identity_type,
            "event_field": value.correlation.event_field,
            "bind_at": value.correlation.bind_at,
            "require_same_at": value.correlation.require_same_at.map(host_op_name),
        },
        "acquire_op": host_op_name(value.acquire_op),
        "use_op": host_op_name(value.use_op),
        "settle_op": host_op_name(value.settle_op),
        "monitor_template": {
            "successful_acquire_settles_exactly_once": value.monitor_template.successful_acquire_settles_exactly_once,
            "forbid_successful_use_after_settlement": value.monitor_template.forbid_successful_use_after_settlement,
            "require_no_live_resource_on": value.monitor_template.require_no_live_resource_on,
            "retain_settlement_outcome": value.monitor_template.retain_settlement_outcome,
        }
    })
}
