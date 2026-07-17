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
//! delegated result to `proved` status. The checked emitter takes only
//! kernel-produced `Verdict` values and the kernel's `trusted_base()` — no
//! external discharge result can enter here and write a `Q` entry. The gate
//! is structural: `QEntry` can only be constructed inside the checked-target
//! export transaction, and
//! only when `trusted_base()` does not contain the hole.
//!
//! **Disproved boundary (71 §2.1):** a refuted claim is never exported. The
//! build is non-shippable when any obligation is `disproved`.
//!
//! **Σ = L5 perform-node signatures (I3):** the alphabet is derived from the
//! selected checked target's closed, target-neutral denotation graph.  The
//! declared effect row remains an admission upper bound and is never an
//! alphabet source or fallback.
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

use crate::compiler_driver::{CheckedPerformNodeV1, CheckedTargetDenotationV1};
use crate::extract::{ObligationTriple, ProvKind};
use crate::prover::Verdict;
use ken_kernel::GlobalId;

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
/// inside the checked-target export transaction — the one-way gate holds structurally.
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
/// [`emit_checked_target_export`]; the `temporal` parameter flows straight into `T`.
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
                require_no_live_resource_on: ["NormalReturn", "ReturnedError", "ControlledTrap"],
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

// ─── Exact perform-node inventory ───────────────────────────────────────────

/// A typed perform identity recovered from the closed checked denotation.
///
/// `Host` is identity-checked against the closed `HostOpV1` catalog. `L5`
/// reserves the general non-host operation form rather than pretending the
/// host catalog is the whole language. Current lowering fails closed before
/// export when such a node has not yet reached a typed runtime representation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerformNodeSignatureV1 {
    Host {
        family_symbol: String,
        operation: ken_host::HostOpV1,
    },
    L5 {
        family_symbol: String,
        operation_symbol: String,
    },
}

/// Canonical exact inventory, bound to one target and one checked semantic
/// artifact. Its fields and constructor are producer-private; callers cannot
/// hand the exporter a provenance-free set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformNodeInventoryV1 {
    target_symbol: String,
    package_identity: String,
    core_semantic_hash: u64,
    artifact_hash: u64,
    closure_identity: u64,
    nodes: BTreeSet<PerformNodeSignatureV1>,
}

impl PerformNodeInventoryV1 {
    pub fn target_symbol(&self) -> &str {
        &self.target_symbol
    }

    pub fn nodes(&self) -> &BTreeSet<PerformNodeSignatureV1> {
        &self.nodes
    }
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
    /// The correlated resource body is not the exact Spec-owned V1 schema.
    /// Validation happens before either a `T` wire entry or a content hash is
    /// emitted, so an independent-event lookalike cannot become shippable.
    InvalidResourceLifetimeObligation,
    /// The selected checked target did not yield a finite, exhaustive graph.
    /// No declared-row or whole-family widening is permitted.
    NonClosedPerformInventory { reason: String },
    /// The producer-created inventory does not belong to this exact target and
    /// semantic artifact, or its node set was changed after derivation.
    PerformInventoryBindingMismatch,
    /// A structural perform node is not covered by the separately retained
    /// declared-row upper bound (`ρ_inf ⊆ ρ_decl`).
    PerformedEffectOutsideDeclaredRow { effect: String },
    /// A `T` or correlated-resource symbol is not in the exact B1 alphabet.
    TemporalSymbolOutsideAlphabet { symbol: String },
}

// ─── The emitter (the sole constructor for Q entries) ────────────────────────

/// Emit the behavioral export contract from one selected checked target
/// (`71 §2.1`, Architect ruling `evt_21ads3za1z4a9`).
///
/// # Parameters
/// - `denotation`: producer-created, identity-bound closed target denotation.
/// - `results`: pairs of `(ObligationTriple, Verdict)` — the V2→V3 pairs
///   for each obligation. The `hole_id` in each triple is the V1 kernel
///   postulate; its presence/absence in `trusted_base_set` is the honesty
///   discriminator.
/// - `trusted_base_set`: the kernel's current `trusted_base()` at emission.
///   Collect via `env.trusted_base().into_iter().collect()`.
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
pub fn emit_checked_target_export(
    denotation: &CheckedTargetDenotationV1,
    results: &[(ObligationTriple, Verdict)],
    trusted_base_set: &BTreeSet<GlobalId>,
    generators: Vec<GEntry>,
    temporal: Vec<TEntry>,
) -> Result<BehavioralExport, ExportError> {
    let inventory = derive_perform_inventory(denotation)?;
    assemble_checked_export(
        denotation,
        &inventory,
        results,
        trusted_base_set,
        generators,
        temporal,
    )
}

fn assemble_checked_export(
    denotation: &CheckedTargetDenotationV1,
    inventory: &PerformNodeInventoryV1,
    results: &[(ObligationTriple, Verdict)],
    trusted_base_set: &BTreeSet<GlobalId>,
    generators: Vec<GEntry>,
    temporal: Vec<TEntry>,
) -> Result<BehavioralExport, ExportError> {
    if inventory != &derive_perform_inventory(denotation)? {
        return Err(ExportError::PerformInventoryBindingMismatch);
    }
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
                return Err(ExportError::DisproovedClaim {
                    obligation_id: id_str,
                });
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

    // Σ is solely the canonical wire projection of the producer-created exact
    // perform inventory. The declared row has already been checked separately;
    // it is never read here as a source or fallback.
    let alphabet_set = project_inventory_alphabet(inventory)?;

    for obligation in &obligations {
        validate_temporal_signature_atoms(&obligation.formula, &alphabet_set)?;
    }

    let resource_lifetime_obligation = alphabet_set
        .contains("FsOpen")
        .then(ResourceLifetimeObligationV1::pinned);

    if let Some(resource) = resource_lifetime_obligation.as_ref() {
        validate_resource_lifetime_obligation(resource)?;
        for operation in [resource.acquire_op, resource.use_op, resource.settle_op] {
            let symbol = canonical_host_perform_signature_v1(operation).to_string();
            if !alphabet_set.contains(&symbol) {
                return Err(ExportError::TemporalSymbolOutsideAlphabet { symbol });
            }
        }
    }

    let hash = compute_hash(
        denotation.target_name(),
        &guarantees,
        &assumptions,
        &alphabet_set,
        &obligations,
        resource_lifetime_obligation.as_ref(),
        &gens,
    );

    Ok(BehavioralExport {
        target_name: denotation.target_name().to_string(),
        guarantees,
        assumptions,
        alphabet: alphabet_set,
        obligations,
        resource_lifetime_obligation,
        generators: gens,
        hash,
    })
}

fn derive_perform_inventory(
    denotation: &CheckedTargetDenotationV1,
) -> Result<PerformNodeInventoryV1, ExportError> {
    if denotation.package.header.package_identity != denotation.package_identity
        || denotation.package.core_semantic_hash != denotation.core_semantic_hash
        || denotation.package.artifact_hash != denotation.artifact_hash
    {
        return Err(ExportError::PerformInventoryBindingMismatch);
    }
    let nodes = denotation
        .perform_nodes
        .iter()
        .map(|node| match node {
            CheckedPerformNodeV1::Host {
                family_symbol,
                operation,
            } => PerformNodeSignatureV1::Host {
                family_symbol: family_symbol.to_string(),
                operation: *operation,
            },
            CheckedPerformNodeV1::L5 {
                family_symbol,
                operation_symbol,
            } => PerformNodeSignatureV1::L5 {
                family_symbol: family_symbol.to_string(),
                operation_symbol: operation_symbol.to_string(),
            },
        })
        .collect::<BTreeSet<_>>();

    for node in &nodes {
        let effect = match node {
            PerformNodeSignatureV1::Host {
                family_symbol,
                operation,
            } => {
                if canonical_symbol_tail(family_symbol) != host_operation_family(*operation).0 {
                    return Err(ExportError::NonClosedPerformInventory {
                        reason: format!(
                            "typed host operation {operation:?} is bound to the wrong family {family_symbol}"
                        ),
                    });
                }
                host_operation_family(*operation).1.to_string()
            }
            PerformNodeSignatureV1::L5 { family_symbol, .. } => {
                effect_label_for_family(family_symbol)
            }
        };
        if !denotation.declared_effects.contains(&effect) {
            return Err(ExportError::PerformedEffectOutsideDeclaredRow { effect });
        }
    }

    Ok(PerformNodeInventoryV1 {
        target_symbol: denotation.target_symbol.to_string(),
        package_identity: denotation.package_identity.to_string(),
        core_semantic_hash: denotation.core_semantic_hash,
        artifact_hash: denotation.artifact_hash,
        closure_identity: denotation.closure_identity,
        nodes,
    })
}

fn canonical_symbol_tail(symbol: &str) -> &str {
    symbol.rsplit("::").next().unwrap_or(symbol)
}

fn effect_label_for_family(symbol: &str) -> String {
    canonical_symbol_tail(symbol)
        .strip_suffix("Op")
        .unwrap_or_else(|| canonical_symbol_tail(symbol))
        .to_string()
}

fn host_operation_family(operation: ken_host::HostOpV1) -> (&'static str, &'static str) {
    match operation {
        ken_host::HostOpV1::ConsoleRead
        | ken_host::HostOpV1::ConsoleWrite
        | ken_host::HostOpV1::ConsoleFlush
        | ken_host::HostOpV1::ConsoleIsTerminal => ("ConsoleOp", "Console"),
        ken_host::HostOpV1::ClockWallNow => ("ClockOp", "Clock"),
        ken_host::HostOpV1::FsReadFile
        | ken_host::HostOpV1::FsWriteFile
        | ken_host::HostOpV1::FsAppendFile
        | ken_host::HostOpV1::FsMetadata
        | ken_host::HostOpV1::FsReadDirectory
        | ken_host::HostOpV1::FsCreateDirectory
        | ken_host::HostOpV1::FsRemoveFile
        | ken_host::HostOpV1::FsRemoveDirectory
        | ken_host::HostOpV1::FsRename
        | ken_host::HostOpV1::FsChangeMode
        | ken_host::HostOpV1::FsOpen
        | ken_host::HostOpV1::FsHandleMetadata
        | ken_host::HostOpV1::ResourceRelease => ("FSOp", "FS"),
    }
}

/// Injective canonical wire spelling for one typed perform identity.
///
/// Host operations retain their pinned V1 names. Non-host L5 identities carry
/// both stable family and constructor symbols with length prefixes, so neither
/// namespace separators nor same-family siblings can collide.
pub fn canonical_perform_node_signature_v1(
    node: &PerformNodeSignatureV1,
) -> Result<String, ExportError> {
    match node {
        PerformNodeSignatureV1::Host {
            family_symbol,
            operation,
        } => {
            if canonical_symbol_tail(family_symbol) != host_operation_family(*operation).0 {
                return Err(ExportError::NonClosedPerformInventory {
                    reason: format!(
                        "typed host operation {operation:?} is bound to the wrong family {family_symbol}"
                    ),
                });
            }
            Ok(canonical_host_perform_signature_v1(*operation).to_string())
        }
        PerformNodeSignatureV1::L5 {
            family_symbol,
            operation_symbol,
        } => Ok(canonical_l5_perform_signature_v1(
            family_symbol,
            operation_symbol,
        )),
    }
}

/// Canonical HostOpV1 signature. The exhaustive match is the wire vocabulary.
pub const fn canonical_host_perform_signature_v1(operation: ken_host::HostOpV1) -> &'static str {
    match operation {
        ken_host::HostOpV1::ConsoleRead => "ConsoleRead",
        ken_host::HostOpV1::ConsoleWrite => "ConsoleWrite",
        ken_host::HostOpV1::ConsoleFlush => "ConsoleFlush",
        ken_host::HostOpV1::ConsoleIsTerminal => "ConsoleIsTerminal",
        ken_host::HostOpV1::ClockWallNow => "ClockWallNow",
        ken_host::HostOpV1::FsReadFile => "FsReadFile",
        ken_host::HostOpV1::FsWriteFile => "FsWriteFile",
        ken_host::HostOpV1::FsAppendFile => "FsAppendFile",
        ken_host::HostOpV1::FsMetadata => "FsMetadata",
        ken_host::HostOpV1::FsReadDirectory => "FsReadDirectory",
        ken_host::HostOpV1::FsCreateDirectory => "FsCreateDirectory",
        ken_host::HostOpV1::FsRemoveFile => "FsRemoveFile",
        ken_host::HostOpV1::FsRemoveDirectory => "FsRemoveDirectory",
        ken_host::HostOpV1::FsRename => "FsRename",
        ken_host::HostOpV1::FsChangeMode => "FsChangeMode",
        ken_host::HostOpV1::FsOpen => "FsOpen",
        ken_host::HostOpV1::FsHandleMetadata => "FsHandleMetadata",
        ken_host::HostOpV1::ResourceRelease => "ResourceRelease",
    }
}

/// Canonical non-host L5 signature with an unambiguous pair encoding.
pub fn canonical_l5_perform_signature_v1(family_symbol: &str, operation_symbol: &str) -> String {
    format!(
        "L5:{}:{}:{}:{}",
        family_symbol.len(),
        family_symbol,
        operation_symbol.len(),
        operation_symbol
    )
}

fn project_inventory_alphabet(
    inventory: &PerformNodeInventoryV1,
) -> Result<BTreeSet<String>, ExportError> {
    let mut alphabet = BTreeSet::new();
    for node in &inventory.nodes {
        alphabet.insert(canonical_perform_node_signature_v1(node)?);
    }
    Ok(alphabet)
}

fn validate_temporal_signature_atoms(
    formula: &crate::temporal::Temporal,
    alphabet: &BTreeSet<String>,
) -> Result<(), ExportError> {
    use crate::temporal::{Pred, Temporal};

    match formula {
        Temporal::Atom(Pred::Event(symbol)) => {
            if alphabet.contains(symbol) {
                Ok(())
            } else {
                Err(ExportError::TemporalSymbolOutsideAlphabet {
                    symbol: symbol.clone(),
                })
            }
        }
        Temporal::Atom(_) | Temporal::Var(_) => Ok(()),
        Temporal::Not(inner) | Temporal::Next(inner) => {
            validate_temporal_signature_atoms(inner, alphabet)
        }
        Temporal::And(left, right) | Temporal::Or(left, right) | Temporal::Until(left, right) => {
            validate_temporal_signature_atoms(left, alphabet)?;
            validate_temporal_signature_atoms(right, alphabet)
        }
        Temporal::Mu { body, .. } | Temporal::Nu { body, .. } => {
            validate_temporal_signature_atoms(body, alphabet)
        }
    }
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
    root.insert(
        "alphabet",
        sigma_repr
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(","),
    );

    // T entries: id + the delegated `Temporal` formula body (content-addressed
    // — a different formula yields a different hash, `71 §3.3`).
    let mut t_repr: Vec<String> = obligations
        .iter()
        .map(|e| format!("{}:{:?}", e.obligation_id, e.formula))
        .collect();
    // The structured descriptor is a member of T, exactly like its wire
    // representation below.  When it is absent we append nothing, preserving
    // the byte-for-byte pre-PX7-F canonical input and hash.
    if let Some(resource) = resource_lifetime_obligation {
        t_repr.push(canonical_resource_lifetime_obligation(resource));
    }
    root.insert("obligations", t_repr.join("|"));

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
pub fn try_serialize_export(export: &BehavioralExport) -> Result<serde_json::Value, ExportError> {
    use serde_json::{json, Value};

    if let Some(resource) = export.resource_lifetime_obligation.as_ref() {
        validate_resource_lifetime_obligation(resource)?;
    }

    let guarantees: Vec<Value> = export
        .guarantees
        .iter()
        .map(|e| {
            json!({
                "obligation_id": e.obligation_id,    // (oracle): "id" / "obligation_id"
                "goal": e.goal,
                "status": "proved"
            })
        })
        .collect();

    let assumptions: Vec<Value> = export
        .assumptions
        .iter()
        .map(|e| {
            json!({
                "obligation_id": e.obligation_id,    // (oracle)
                "goal": e.goal,
                "status": e.status.as_str()          // "tested" | "unknown"
            })
        })
        .collect();

    let alphabet: Vec<&String> = export.alphabet.iter().collect();

    let mut obligations: Vec<Value> = export
        .obligations
        .iter()
        .map(|e| {
            json!({
                "obligation_id": e.obligation_id,    // (oracle)
                "status": "delegated",              // constant (`72 §5`); never proved/tested/unknown
                "formula": format!("{:?}", e.formula) // (oracle): Ward-facing spelling deferred
            })
        })
        .collect();
    if let Some(resource) = &export.resource_lifetime_obligation {
        obligations.push(serialize_resource_lifetime_obligation(resource));
    }

    let generators: Vec<Value> = export
        .generators
        .iter()
        .map(|e| {
            json!({
                "source": e.source,
                "conditions": e.conditions,
                // No weight/measure field — structural seal (I5, §4.1)
            })
        })
        .collect();

    Ok(json!({
        "schema": "ken.export/v0",               // (oracle): version token
        "target": export.target_name,
        "guarantees": guarantees,                // Q — (oracle): "guarantees" / "Q"
        "assumptions": assumptions,              // P — (oracle): "assumptions" / "P"
        "alphabet": alphabet,                    // Σ — (oracle): "alphabet" / "sigma"
        "obligations": obligations,              // T — (oracle): "obligations" / "T"
        "generators": generators,                // G — (oracle): "generators" / "G"
        "hash": export.hash
    }))
}

/// Serialize a schema-valid export using the established infallible API.
/// Malformed correlated resource bodies fail closed before producing a value;
/// callers that need to inspect that rejection use [`try_serialize_export`].
pub fn serialize_export(export: &BehavioralExport) -> serde_json::Value {
    try_serialize_export(export).expect("BehavioralExport contains an invalid resource schema")
}

fn validate_resource_lifetime_obligation(
    value: &ResourceLifetimeObligationV1,
) -> Result<(), ExportError> {
    if value == &ResourceLifetimeObligationV1::pinned() {
        Ok(())
    } else {
        Err(ExportError::InvalidResourceLifetimeObligation)
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
        canonical_host_perform_signature_v1(value.correlation.require_same_at[0]),
        canonical_host_perform_signature_v1(value.correlation.require_same_at[1]),
        canonical_host_perform_signature_v1(value.acquire_op),
        canonical_host_perform_signature_v1(value.use_op),
        canonical_host_perform_signature_v1(value.settle_op),
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
            "require_same_at": value.correlation.require_same_at.map(canonical_host_perform_signature_v1),
        },
        "acquire_op": canonical_host_perform_signature_v1(value.acquire_op),
        "use_op": canonical_host_perform_signature_v1(value.use_op),
        "settle_op": canonical_host_perform_signature_v1(value.settle_op),
        "monitor_template": {
            "successful_acquire_settles_exactly_once": value.monitor_template.successful_acquire_settles_exactly_once,
            "forbid_successful_use_after_settlement": value.monitor_template.forbid_successful_use_after_settlement,
            "require_no_live_resource_on": value.monitor_template.require_no_live_resource_on,
            "retain_settlement_outcome": value.monitor_template.retain_settlement_outcome,
        }
    })
}

#[cfg(test)]
mod resource_lifetime_hash_tests {
    use super::*;

    #[test]
    fn one_descriptor_field_mutation_changes_the_t_hash() {
        let pinned = ResourceLifetimeObligationV1::pinned();
        let mut independent_event_lookalike = pinned.clone();
        independent_event_lookalike.correlation.event_field = "EffectEventV1.operation";
        let alphabet = BTreeSet::from(["FsOpen".to_string()]);

        let hash = |resource: &ResourceLifetimeObligationV1| {
            compute_hash(
                "resource-target",
                &[],
                &[],
                &alphabet,
                &[],
                Some(resource),
                &[],
            )
        };
        assert_ne!(hash(&pinned), hash(&independent_event_lookalike));
    }
}

#[cfg(test)]
mod exact_inventory_tests {
    use super::*;
    use crate::compiler_driver::{compile_checked_target_denotation, CompilerSource};

    fn denotation(target: &str) -> CheckedTargetDenotationV1 {
        compile_checked_target_denotation(
            "b1_inventory_binding",
            CompilerSource::new(
                "binding.ken",
                r#"
proc first (_value : Unit)
  : HostIO AFull (Result IOError Unit) visits [Console] =
  host_console AFull (Result IOError Unit) (flush Stdout)

proc second (_value : Unit)
  : HostIO AFull (Result IOError Unit) visits [Console] =
  host_console AFull (Result IOError Unit) (flush Stdout)
"#,
            ),
            target,
        )
        .expect("checked binding fixture")
    }

    fn rejects(denotation: &CheckedTargetDenotationV1, inventory: &PerformNodeInventoryV1) {
        assert!(matches!(
            assemble_checked_export(denotation, inventory, &[], &BTreeSet::new(), vec![], vec![],),
            Err(ExportError::PerformInventoryBindingMismatch)
        ));
    }

    #[test]
    fn target_semantic_and_node_mutations_reject_before_hash() {
        let first = denotation("first");
        let canonical = derive_perform_inventory(&first).expect("canonical inventory");

        let other_target =
            derive_perform_inventory(&denotation("second")).expect("other target inventory");
        rejects(&first, &other_target);

        let mut other_semantic_identity = canonical.clone();
        other_semantic_identity.core_semantic_hash ^= 1;
        rejects(&first, &other_semantic_identity);

        let mut omitted_node = canonical.clone();
        omitted_node.nodes.pop_first();
        rejects(&first, &omitted_node);

        let mut added_node = canonical;
        added_node.nodes.insert(PerformNodeSignatureV1::L5 {
            family_symbol: "fixture::SyntheticOp".to_string(),
            operation_symbol: "fixture::Synthetic".to_string(),
        });
        rejects(&first, &added_node);
    }

    #[test]
    fn exact_fixture_preserves_hash_and_headroom_changes_only_alphabet() {
        let exact = compile_checked_target_denotation(
            "b1_compat_exact",
            CompilerSource::new("exact.ken", "fn target (value : Unit) : Unit = value"),
            "target",
        )
        .expect("exact pure target");
        let exact_export =
            emit_checked_target_export(&exact, &[], &BTreeSet::new(), vec![], vec![])
                .expect("exact export");
        assert_eq!(
            exact_export.hash,
            compute_hash("target", &[], &[], &BTreeSet::new(), &[], None, &[]),
            "an already-exact legacy alphabet retains its canonical hash"
        );

        let headroom = compile_checked_target_denotation(
            "b1_compat_headroom",
            CompilerSource::new(
                "headroom.ken",
                "proc target (value : Unit) : Unit visits [Console] = value",
            ),
            "target",
        )
        .expect("legal headroom target");
        let current = emit_checked_target_export(&headroom, &[], &BTreeSet::new(), vec![], vec![])
            .expect("headroom export");
        let legacy_alphabet = BTreeSet::from(["Console".to_string()]);
        let legacy_hash = compute_hash("target", &[], &[], &legacy_alphabet, &[], None, &[]);
        assert!(current.alphabet.is_empty());
        assert_ne!(current.hash, legacy_hash);

        let mut current_wire = serialize_export(&current);
        let mut legacy_wire = current_wire.clone();
        legacy_wire["alphabet"] = serde_json::json!(["Console"]);
        legacy_wire["hash"] = serde_json::json!(legacy_hash);
        current_wire.as_object_mut().unwrap().remove("alphabet");
        current_wire.as_object_mut().unwrap().remove("hash");
        legacy_wire.as_object_mut().unwrap().remove("alphabet");
        legacy_wire.as_object_mut().unwrap().remove("hash");
        assert_eq!(current_wire, legacy_wire);
    }
}
