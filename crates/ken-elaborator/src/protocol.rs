//! T1 machine-readable diagnostic protocol (`spec/20-verification/25-protocol.md`).
//!
//! Serializes V3 prover verdicts + V4 diagnostics to structured, versioned,
//! schema-valid JSON documents. **The cardinal rule**: every status string,
//! every `verdict` tag, and every diagnostic `kind` on the wire is a
//! **projection** of the one V3 verdict — copied at serialization, never
//! recomputed from the JSON evidence.
//!
//! `(oracle)` note per `25` preamble: exact JSON field *names* are a
//! reference finalized with the agent-team software. What is normative here:
//! value-sets (`status ∈ {proved,disproved,incomplete}`, etc.), cross-field
//! agreement invariants, id/`hole_id` stability semantics, and
//! `trusted_base_delta` emptiness.

use serde_json::{json, Value};

use crate::diagnostics::{
    Diagnostic, DiagnosticTag, KripkeCountermodel, Region, SuggestedAction, TypedHole,
};
use crate::extract::{ObligationId, ObligationTriple};
use crate::prover::Verdict;

// ─── Cardinal-rule cross-walk types (§wire-contract) ─────────────────────────

/// Per-obligation projection of V3's verdict (`25 §4`).
/// `Discharged` ⇐ `Proved`, `Refuted` ⇐ `Disproved`, `Open` ⇐ `Unknown`.
/// The cardinal rule: this is a **projection**, not an independent decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationStatus {
    Discharged,
    Refuted,
    Open,
}

/// Document rollup — total function with precedence `refuted ⊐ open ⊐
/// discharged` (`25 §3`): `disproved` if any `refuted`, else `incomplete` if
/// any `open`, else `proved`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocStatus {
    Proved,
    Disproved,
    Incomplete,
}

/// The `false`/`unknown` wire discriminator — the `countermodel.verdict` tag,
/// copied verbatim from V3's verdict, **never** recomputed from JSON evidence
/// (`25 §wire-contract`, `24 §1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireVerdict {
    False,
    Unknown,
}

impl ObligationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Discharged => "discharged",
            Self::Refuted => "refuted",
            Self::Open => "open",
        }
    }
}

impl DocStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proved => "proved",
            Self::Disproved => "disproved",
            Self::Incomplete => "incomplete",
        }
    }
}

impl WireVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::False => "false",
            Self::Unknown => "unknown",
        }
    }
}

// ─── Cardinal-rule projections ────────────────────────────────────────────────

/// Project a V3 verdict to an obligation status (the cross-walk, `25 §4`).
pub fn project_obligation_status(verdict: &Verdict) -> ObligationStatus {
    match verdict {
        Verdict::Proved { .. } => ObligationStatus::Discharged,
        Verdict::Disproved { .. } => ObligationStatus::Refuted,
        Verdict::Unknown { .. } => ObligationStatus::Open,
    }
}

/// Project a V3 verdict to the wire verdict tag.
/// `Proved → None` (no countermodel), `Disproved → Some(False)`,
/// `Unknown → Some(Unknown)`.
pub fn project_wire_verdict(verdict: &Verdict) -> Option<WireVerdict> {
    match verdict {
        Verdict::Proved { .. } => None,
        Verdict::Disproved { .. } => Some(WireVerdict::False),
        Verdict::Unknown { .. } => Some(WireVerdict::Unknown),
    }
}

/// Document-level rollup: precedence `refuted ⊐ open ⊐ discharged` (`25 §3`).
pub fn rollup_doc_status(statuses: &[ObligationStatus]) -> DocStatus {
    if statuses.iter().any(|s| *s == ObligationStatus::Refuted) {
        DocStatus::Disproved
    } else if statuses.iter().any(|s| *s == ObligationStatus::Open) {
        DocStatus::Incomplete
    } else {
        DocStatus::Proved
    }
}

// ─── Id derivation (stable, §4/§6, `22 §1`, `24 §6`) ────────────────────────

/// Stable obligation `id` string — a function of **program structure** (the
/// clause path), not run order or allocation order (`22 §1`, `24 §6`).
pub fn obligation_id_string(id: &ObligationId) -> String {
    id.0.clone()
}

/// Stable `hole_id` string — a function of the obligation provenance/clause
/// path, not allocation order (`25 §4`, `24 §6`).
pub fn hole_id_string(obligation_id: &ObligationId) -> String {
    format!("?h:{}", obligation_id.0)
}

// ─── Action serialization ─────────────────────────────────────────────────────

/// Serialize a `SuggestedAction` to wire JSON. Region-tagged so an
/// `unknown`-only action never rides a `false` goal (`24 §5`, `25 §5`).
pub fn serialize_action(action: &SuggestedAction) -> Value {
    match action {
        SuggestedAction::AddPrecondition { hypothesis: _ } => json!({
            "kind": "add_precondition", "region": "unknown",
            "detail": "add a precondition to discharge this obligation",
            "edit": null
        }),
        SuggestedAction::StrengthenRefinement { parameter: _ } => json!({
            "kind": "strengthen_refinement", "region": "unknown",
            "detail": "strengthen the refinement type", "edit": null
        }),
        SuggestedAction::ProvideLemma { statement: _ } => json!({
            "kind": "provide_lemma", "region": "unknown",
            "detail": "provide a supporting lemma", "edit": null
        }),
        SuggestedAction::CaseSplit { expression: _ } => json!({
            "kind": "case_split", "region": "unknown",
            "detail": "split on this expression", "edit": null
        }),
        SuggestedAction::InductOn { variable: _ } => json!({
            "kind": "induct_on", "region": "unknown",
            "detail": "use induction on this variable", "edit": null
        }),
        SuggestedAction::FixCounterexample { description } => json!({
            "kind": "fix_counterexample", "region": "false",
            "detail": description, "edit": null
        }),
    }
}

// ─── Diagnostic serialization (§5 shapes, four `24` mechanisms) ──────────────

/// Serialize a `KripkeCountermodel` to wire JSON (`25 §5`, `24 §1`).
///
/// `verdict` is **copied** from V3's verdict via `DiagnosticTag`, never
/// recomputed from the model structure (cardinal rule).
pub fn serialize_countermodel(cm: &KripkeCountermodel, actions: &[SuggestedAction]) -> Value {
    let verdict_str = match cm.verdict {
        DiagnosticTag::False => "false",
        DiagnosticTag::Unknown => "unknown",
    };
    let worlds: Vec<Value> = cm.worlds.iter().map(|w| json!(w.0)).collect();
    let order: Vec<Value> = cm.order.iter().map(|(a, b)| json!([a.0, b.0])).collect();

    // forcing: BTreeMap<world → [atoms]> (sorted for determinism)
    let mut forcing_map: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for w in &cm.worlds {
        forcing_map.insert(w.0.clone(), vec![]);
    }
    for (world, atom) in &cm.forcing {
        forcing_map
            .entry(world.0.clone())
            .or_default()
            .push(atom.0.clone());
    }
    let forcing = serde_json::to_value(forcing_map).unwrap_or(json!({}));

    let failure = cm
        .failure
        .as_ref()
        .map(|f| {
            json!({
                "world": f.world.0,
                "subformula": f.subformula.0
            })
        })
        .unwrap_or(Value::Null);

    let suggested_actions: Vec<Value> = actions.iter().map(serialize_action).collect();
    json!({
        "kind": "countermodel",
        "verdict": verdict_str,
        "worlds": worlds,
        "order": order,
        "forcing": forcing,
        "failure": failure,
        "suggested_actions": suggested_actions
    })
}

/// Serialize a `TypedHole` to wire JSON (`25 §5`, `24 §2`).
pub fn serialize_hole(
    hole: &TypedHole,
    obligation_id: &ObligationId,
    actions: &[SuggestedAction],
) -> Value {
    let hole_id = hole_id_string(obligation_id);
    let suggested_actions: Vec<Value> = actions.iter().map(serialize_action).collect();
    json!({
        "kind": "hole",
        "hole_id": hole_id,
        "goal": format!("{:?}", hole.goal),
        "context": [],
        "origin": {
            "span": format!("{:?}", hole.origin.span),
            "clause": format!("{:?}", hole.origin.kind)
        },
        "runtime": "unknown",
        "suggested_actions": suggested_actions
    })
}

/// Serialize a three-region decomposition diagnostic (`25 §5`, `24 §3`).
pub fn serialize_decomposition(
    true_region: &str,
    false_region: &str,
    unknown_region: &str,
    actions: &[SuggestedAction],
) -> Value {
    let suggested_actions: Vec<Value> = actions.iter().map(serialize_action).collect();
    json!({
        "kind": "decomposition",
        "true_region": true_region,
        "false_region": false_region,
        "unknown_region": unknown_region,
        "suggested_actions": suggested_actions
    })
}

/// Serialize a slice / missing-hypothesis diagnostic (`25 §5`, `24 §4`).
/// Only valid for `unknown` obligations.
pub fn serialize_slice(
    missing_hypothesis: &str,
    bridge: &str,
    sufficient: bool,
    actions: &[SuggestedAction],
) -> Value {
    let suggested_actions: Vec<Value> = actions.iter().map(serialize_action).collect();
    json!({
        "kind": "slice",
        "missing_hypothesis": missing_hypothesis,
        "bridge": bridge,
        "sufficient": sufficient,
        "suggested_actions": suggested_actions
    })
}

/// Serialize a V4 `Diagnostic` to the wire JSON shape (`25 §5`).
/// Routes by countermodel/typed_hole presence; falls to region for backends
/// not yet landed at V4.
pub fn serialize_diagnostic(diag: &Diagnostic, obligation_id: &ObligationId) -> Value {
    let actions = &diag.suggested_actions;
    match (&diag.countermodel, &diag.typed_hole) {
        (Some(cm), _) => serialize_countermodel(cm, actions),
        (None, Some(hole)) => serialize_hole(hole, obligation_id, actions),
        (None, None) => {
            // No backend data; emit based on region (honest placeholder).
            match diag.region {
                Region::Refuted => {
                    serialize_decomposition("", "[countermodel pending V4-backend]", "", actions)
                }
                Region::Unknown => {
                    serialize_slice("[pending V4-backend]", "add as precondition", true, actions)
                }
                Region::Proved => Value::Null,
            }
        }
    }
}

// ─── Obligation + document builders ──────────────────────────────────────────

/// Serialize one obligation to its wire JSON object (`25 §4`).
///
/// `diagnostic` must be `null` iff `verdict` is `Proved` (the cross-walk
/// totality invariant, `25 §4`/§6).
pub fn serialize_obligation(
    verdict: &Verdict,
    diag: Option<&Diagnostic>,
    obligation_id: &ObligationId,
    triple: &ObligationTriple,
) -> Value {
    let ob_status = project_obligation_status(verdict);
    let diag_value = match ob_status {
        ObligationStatus::Discharged => Value::Null,
        _ => diag
            .map(|d| serialize_diagnostic(d, obligation_id))
            .unwrap_or(Value::Null),
    };
    json!({
        "id": obligation_id_string(obligation_id),
        "goal": { "pretty": format!("{:?}", triple.phi) },
        "context": [],
        "provenance": {
            "span": format!("{:?}", triple.provenance.span),
            "clause": format!("{:?}", triple.provenance.kind)
        },
        "status": ob_status.as_str(),
        "diagnostic": diag_value
    })
}

/// Build a `trusted_base_delta` entry for an open hole obligation.
pub fn trusted_base_entry(obligation_id: &ObligationId, triple: &ObligationTriple) -> Value {
    json!({
        "id": hole_id_string(obligation_id),
        "goal": format!("{:?}", triple.phi)
    })
}

/// Serialize a full verdict document (`25 §3`).
///
/// `obligations_data` = list of `(verdict, diag, obligation_id, triple)`
/// for each obligation.
pub fn serialize_document(
    target_name: &str,
    obligations_data: &[(Verdict, Option<Diagnostic>, ObligationId, ObligationTriple)],
) -> Value {
    let mut ob_values: Vec<Value> = vec![];
    let mut statuses: Vec<ObligationStatus> = vec![];
    let mut trusted_base_delta: Vec<Value> = vec![];

    for (verdict, diag, ob_id, triple) in obligations_data {
        let status = project_obligation_status(verdict);
        if status == ObligationStatus::Open {
            trusted_base_delta.push(trusted_base_entry(ob_id, triple));
        }
        statuses.push(status);
        ob_values.push(serialize_obligation(verdict, diag.as_ref(), ob_id, triple));
    }

    let doc_status = rollup_doc_status(&statuses);
    let discharged = statuses
        .iter()
        .filter(|s| **s == ObligationStatus::Discharged)
        .count();
    let open = statuses
        .iter()
        .filter(|s| **s == ObligationStatus::Open)
        .count();

    json!({
        "schema": "ken.verify/v1",
        "target": { "name": target_name },
        "status": doc_status.as_str(),
        "obligations": ob_values,
        "trusted_base_delta": trusted_base_delta,
        "stats": {
            "obligations": obligations_data.len(),
            "discharged": discharged,
            "open": open
        }
    })
}

// ─── JSON round-trip ─────────────────────────────────────────────────────────

/// Serialize a `Value` to a JSON string and parse it back.
///
/// Tests round-trip losslessness: `round_trip(v) == v` (modulo key ordering,
/// which `serde_json` normalises via object deserialization).
pub fn round_trip(doc: &Value) -> Value {
    let s = serde_json::to_string(doc).expect("serialize");
    serde_json::from_str(&s).expect("deserialize")
}

// ─── Reference validator (§8) ─────────────────────────────────────────────────

/// Validate a verdict document against the §8 reference schema.
///
/// Checks: required fields present, value-sets correct, `diagnostic:null` iff
/// discharged, countermodel `verdict` ∈ `{false,unknown}` when present. Ignores
/// unknown fields (forward-compatibility, `25 §6`).
///
/// Returns `Ok(())` if valid, `Err(reason)` otherwise.
pub fn validate_document(doc: &Value) -> Result<(), String> {
    // schema required
    doc.get("schema")
        .and_then(Value::as_str)
        .ok_or("missing required field: schema")?;

    // target required
    doc.get("target").ok_or("missing required field: target")?;

    // status required ∈ {proved, disproved, incomplete}
    let status_str = doc
        .get("status")
        .and_then(Value::as_str)
        .ok_or("missing required field: status")?;
    if !["proved", "disproved", "incomplete"].contains(&status_str) {
        return Err(format!(
            "status must be in {{proved,disproved,incomplete}}, got: {}",
            status_str
        ));
    }

    // obligations required array
    let obligations = doc
        .get("obligations")
        .and_then(Value::as_array)
        .ok_or("missing required field: obligations")?;

    for (i, ob) in obligations.iter().enumerate() {
        validate_obligation(ob, i)?;
    }

    // trusted_base_delta: if present, must be an array
    if let Some(delta) = doc.get("trusted_base_delta") {
        if !delta.is_array() {
            return Err("trusted_base_delta must be an array".to_owned());
        }
    }

    Ok(())
}

fn validate_obligation(ob: &Value, idx: usize) -> Result<(), String> {
    // id required (stable field — `25 §6`)
    ob.get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("obligations[{}]: missing required stable field: id", idx))?;

    // status required ∈ {discharged, refuted, open}
    let ob_status = ob
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("obligations[{}]: missing required field: status", idx))?;
    if !["discharged", "refuted", "open"].contains(&ob_status) {
        return Err(format!(
            "obligations[{}]: status must be in {{discharged,refuted,open}}",
            idx
        ));
    }

    // diagnostic: null iff discharged (`25 §4`)
    let diag = ob.get("diagnostic");
    if ob_status == "discharged" {
        if diag.map(|d| !d.is_null()).unwrap_or(false) {
            return Err(format!(
                "obligations[{}]: discharged obligation must have null diagnostic",
                idx
            ));
        }
    } else {
        // non-discharged: diagnostic must be non-null
        if diag.map(|d| d.is_null()).unwrap_or(true) {
            return Err(format!(
                "obligations[{}]: non-discharged obligation must have non-null diagnostic (totality, 25 §4)",
                idx
            ));
        }
        validate_diagnostic(diag.unwrap(), idx)?;
    }

    Ok(())
}

fn validate_diagnostic(diag: &Value, ob_idx: usize) -> Result<(), String> {
    // kind required ∈ {countermodel, hole, decomposition, slice}
    let kind = diag.get("kind").and_then(Value::as_str).ok_or_else(|| {
        format!(
            "obligations[{}].diagnostic: missing required field: kind",
            ob_idx
        )
    })?;
    if !["countermodel", "hole", "decomposition", "slice"].contains(&kind) {
        return Err(format!(
            "obligations[{}].diagnostic: kind must be in \
             {{countermodel,hole,decomposition,slice}}, got: {}",
            ob_idx, kind
        ));
    }

    if kind == "countermodel" {
        // verdict required ∈ {false, unknown} (stable discriminator field)
        let verdict = diag.get("verdict").and_then(Value::as_str).ok_or_else(|| {
            format!(
                "obligations[{}].diagnostic: countermodel missing required stable \
                     field: verdict",
                ob_idx
            )
        })?;
        if !["false", "unknown"].contains(&verdict) {
            return Err(format!(
                "obligations[{}].diagnostic: verdict must be in {{false,unknown}}, got: {}",
                ob_idx, verdict
            ));
        }
    }

    if kind == "hole" {
        // hole_id required (stable field)
        diag.get("hole_id").and_then(Value::as_str).ok_or_else(|| {
            format!(
                "obligations[{}].diagnostic: hole missing required stable field: hole_id",
                ob_idx
            )
        })?;
    }

    // suggested_actions: each entry must have kind + region ∈ {false,unknown}
    if let Some(actions) = diag.get("suggested_actions").and_then(Value::as_array) {
        for (j, action) in actions.iter().enumerate() {
            // unknown action kinds are ACCEPTED (forward-compat, `25 §6`)
            action.get("kind").and_then(Value::as_str).ok_or_else(|| {
                format!(
                    "obligations[{}].diagnostic.suggested_actions[{}]: missing field: kind",
                    ob_idx, j
                )
            })?;
            let region = action
                .get("region")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!(
                        "obligations[{}].diagnostic.suggested_actions[{}]: missing field: region",
                        ob_idx, j
                    )
                })?;
            if !["false", "unknown"].contains(&region) {
                return Err(format!(
                    "obligations[{}].diagnostic.suggested_actions[{}]: \
                     region must be in {{false,unknown}}, got: {}",
                    ob_idx, j, region
                ));
            }
        }
    }

    Ok(())
}
