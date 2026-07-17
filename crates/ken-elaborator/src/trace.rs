//! B3 trace/instrumentation contract — the runtime companion to the B1 export
//! (`spec/70-behavioral/73-conformance.md §2`).
//!
//! **Generated, never authored.** Every field is a projection of already-
//! verified content (Q/P/T from the B1 export) plus runtime instrumentation
//! (Σ-events from the `36 §2` perform points). No new claims.
//!
//! **Instrumentation only at the effect boundary (TC2, §2).** Events are emitted
//! only at `Vis` firings in `drive_h_instrumented` (`ken-interp::eval`). Pure
//! steps (`Ret`, β, ι) emit nothing. Bounded overhead is structural.
//!
//! **One-way / emit-only (TC5, §3).** There is NO ingest path from a monitor
//! verdict back into Ken's epistemic status. A `delegated` T stays `delegated`
//! regardless of monitor outcome. `TraceContract` has no field by which a monitor
//! verdict enters `Q`. The gate is structural — the absence of a code path, not
//! a runtime check.

use std::collections::BTreeMap;

use crate::export::BehavioralExport;

// ─── Σ-event schema (73 §2.1) ────────────────────────────────────────────────

/// One runtime event — emitted once per `Vis` firing at the `36 §2` perform
/// point (`drive_h_instrumented` in `ken-interp::eval`).
///
/// Values are ITF witnesses (`71 §3.2`): they carry runtime data and have
/// **no epistemic status**. An event is never tagged `proved`/`tested`/
/// `delegated`. A green trace is *evidence for* a `delegated` `T`, never a
/// promotion (TC5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    /// Which `Σ` member fired — an effect label from B1's `BTreeSet<String>`.
    /// Locked concept; literal field key `(oracle)`.
    pub effect: String,
    /// The `Op` tag within the effect (`Console.Write`, `State.Get`, …).
    /// Concretizes the per-op signature B1's `Σ` abstracts at label granularity.
    /// Literal key `(oracle)`.
    pub op: String,
    /// The argument value the op carries — an ITF witness, no status.
    /// Literal key `(oracle)`.
    pub op_arg: String,
    /// The runtime response `E.Resp op` returned to the continuation — ITF witness.
    /// Literal key `(oracle)`.
    pub response: String,
    /// The `space` identity for correlation (`36 §4.1`) — present on **every**
    /// event. Literal key `(oracle)`.
    pub space_id: String,
    /// Message content address (`41 §3`) — **only** on cross-space send/receive
    /// events. `None` for events within a single space. Literal key `(oracle)`.
    pub message_provenance: Option<String>,
    /// Per-space monotone sequence index — lets a monitor order per-space events.
    pub sequence_pos: u64,
}

// ─── Runtime Q/P assertion points (73 §2.3) ──────────────────────────────────

/// A runtime-checkable assertion point — projected from B1's Q/P (TC4).
///
/// Ken emits; the downstream engine runs. The proposition is the export entry's
/// own goal — no re-authoring. Changes when the B1 export's Q/P change (TC4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssertionPoint {
    /// From a `Q` (proved) entry: the downstream engine watches this invariant.
    WatchedInvariant {
        obligation_id: String,
        goal: String,
    },
    /// From a `P` (assumption) entry: the engine confirms the assumption held.
    ConfirmHeld {
        obligation_id: String,
        goal: String,
    },
}

// ─── Monitor projection (73 §2.4) ────────────────────────────────────────────

/// Monitor projected from the B1 export's `T` channel (`73 §2.4`, TC4).
///
/// **Projection, not authoring:** the monitor is the image of `T` — it changes
/// when `T` changes. A hand-written monitor ignoring `T` does not (TC4, TR-E).
///
/// **B2-deferred (oracle):** the concrete `Temporal` datatype (constructors,
/// the `Pred Σ` atom language, `72 §3`) and the full `compile : Temporal Σ →
/// Monitor` faithfulness lemma (`71 §5.2`) land in B2. B3 provides the
/// **projection plumbing** over the landed `T` channel (`TEntry`, `export.rs`).
/// The `Temporal` surface and `compile` signature are `(oracle)`-tagged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorProjection {
    /// Obligation ids from the B1 export's `T` channel — the projection source.
    /// A different `T` yields a different set here (TC4 / TR-E discriminator).
    pub delegated_obligations: Vec<String>,
    // (oracle): concrete Temporal formula + compile signature deferred to B2.
    // The monitor's atom predicates range over Σ (the same events TR-A emits).
}

impl MonitorProjection {
    pub fn is_empty(&self) -> bool {
        self.delegated_obligations.is_empty()
    }
}

// ─── The trace contract (73 §2) ──────────────────────────────────────────────

/// The full B3 trace/instrumentation contract — a *generated* companion to the
/// B1 export that makes a running program **observable in the model's vocabulary
/// `Σ`** (`73 §2`, `73 §5`).
///
/// Every field is a **projection** or **instrumentation record**:
/// - `events`: runtime `Σ`-events from `drive_h_instrumented` (§2.1)
/// - `assertion_points`: projected from B1 export's `Q`/`P` (§2.3)
/// - `monitor`: projected from B1 export's `T` (§2.4)
///
/// **Emit-only / no ingest (TC5, §3):** there is NO field by which a monitor
/// verdict enters `Q`. A `delegated` `T` stays `delegated` regardless of monitor
/// outcome. The gate is structural — the absence of a code path.
#[derive(Debug, Clone)]
pub struct TraceContract {
    pub target_name: String,
    /// `Σ`-events from the effect boundary (§2.1). Ordered by emission time.
    pub events: Vec<TraceEvent>,
    /// Runtime `Q`/`P` assertion points — projected from the B1 export (§2.3).
    pub assertion_points: Vec<AssertionPoint>,
    /// Monitor projected from the B1 export's `T` channel (§2.4).
    pub monitor: MonitorProjection,
    /// Content-addressed canonical hash (inherited from `71 §3.3` discipline).
    pub hash: String,
}

/// A runtime event could not be projected through the selected B1 export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceContractError {
    /// The event names no member of the one B1-derived alphabet.  B3 does not
    /// derive or widen an alphabet of its own.
    EventOutsideAlphabet { effect: String },
}

// ─── Emitter ─────────────────────────────────────────────────────────────────

/// Emit the trace contract from runtime events + the B1 export.
///
/// # Projection sources (TC4)
/// - `events`: collected by `drive_h_instrumented` at the effect boundary (TC2).
///   The caller decodes raw (effect_val, response_val) to `TraceEvent` structs
///   with effect labels from B1's `Σ` (TC1 closure check).
/// - `export`: the B1 export — `Q` projects to `WatchedInvariant`, `P` to
///   `ConfirmHeld`, `T` to `MonitorProjection`. The assertion-point set and
///   the monitor change when the export changes (TC4 / TR-D, TR-E).
///
/// # One-way gate (TC5, I4)
/// There is NO parameter, field, or code path here that takes a monitor verdict
/// and writes a `Q` entry or promotes a `T` to `proved`. The monitor projection
/// reads `T` obligation ids and nothing else. A caller cannot inject a monitor-
/// accept result into `guarantees`. The gate is structural: this function has no
/// return path from `on_event` data to `Q`; it only reads `export.obligations`.
pub fn emit_trace_contract(
    target_name: &str,
    events: Vec<TraceEvent>,
    export: &BehavioralExport,
) -> TraceContract {
    try_emit_trace_contract(target_name, events, export)
        .expect("runtime event is outside the checked B1 alphabet")
}

/// Checked form of [`emit_trace_contract`].
///
/// Every B3 event is accepted only as an image of the alphabet already carried
/// by the B1 export.  This is a closure check, not a second derivation rule.
pub fn try_emit_trace_contract(
    target_name: &str,
    events: Vec<TraceEvent>,
    export: &BehavioralExport,
) -> Result<TraceContract, TraceContractError> {
    if let Some(event) = events
        .iter()
        .find(|event| !export.alphabet.contains(&event.effect))
    {
        return Err(TraceContractError::EventOutsideAlphabet {
            effect: event.effect.clone(),
        });
    }

    // Project assertion points from Q (→ WatchedInvariant) and P (→ ConfirmHeld).
    // These change when the export's Q/P change — they are the export's image.
    let mut assertion_points: Vec<AssertionPoint> = Vec::new();
    for q in &export.guarantees {
        assertion_points.push(AssertionPoint::WatchedInvariant {
            obligation_id: q.obligation_id.clone(),
            goal: q.goal.clone(),
        });
    }
    for p in &export.assumptions {
        assertion_points.push(AssertionPoint::ConfirmHeld {
            obligation_id: p.obligation_id.clone(),
            goal: p.goal.clone(),
        });
    }

    // Project monitor from T: the monitor IS the image of T.
    // It changes when T changes (TC4 / TR-E discriminator).
    // (oracle): concrete Temporal formula + compile signature deferred to B2.
    let monitor = MonitorProjection {
        delegated_obligations: export.obligations
            .iter()
            .map(|t| t.obligation_id.clone())
            .collect(),
    };

    let hash = compute_trace_hash(target_name, &events, &assertion_points, &monitor);

    Ok(TraceContract {
        target_name: target_name.to_string(),
        events,
        assertion_points,
        monitor,
        hash,
    })
}

// ─── Canonical hash (inherited from 71 §3.3 discipline) ──────────────────────

fn compute_trace_hash(
    target_name: &str,
    events: &[TraceEvent],
    assertion_points: &[AssertionPoint],
    monitor: &MonitorProjection,
) -> String {
    let mut root: BTreeMap<&str, String> = BTreeMap::new();
    root.insert("target", target_name.to_string());

    let events_repr: Vec<String> = events.iter().map(|e| {
        format!(
            "{}:{}:{}:{}:{}:{}:{}",
            e.effect,
            e.op,
            e.op_arg,
            e.response,
            e.space_id,
            e.message_provenance.as_deref().unwrap_or(""),
            e.sequence_pos
        )
    }).collect();
    root.insert("events", events_repr.join("|"));

    let ap_repr: Vec<String> = assertion_points.iter().map(|ap| match ap {
        AssertionPoint::WatchedInvariant { obligation_id, goal } =>
            format!("Q:{}:{}", obligation_id, goal),
        AssertionPoint::ConfirmHeld { obligation_id, goal } =>
            format!("P:{}:{}", obligation_id, goal),
    }).collect();
    root.insert("assertion_points", ap_repr.join("|"));

    root.insert("monitor", monitor.delegated_obligations.join(","));

    let canonical: String = root
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    // FNV-1a fold — same discipline as B1 compute_hash.
    // (oracle): replace with SHA-256 when Ward wire format is finalized.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in canonical.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("ken-trace-v0:{:016x}", hash)
}

// ─── Serialization ───────────────────────────────────────────────────────────

/// Serialize the trace contract to canonical JSON (`73 §2.5`, `71 §3.1`).
///
/// Field key spellings are `(oracle)`-tagged — Ward finalizes the wire tokens.
/// Reuses B1's ITF-compatible wire form discipline (no second format).
pub fn serialize_trace_contract(contract: &TraceContract) -> serde_json::Value {
    use serde_json::{json, Value};

    let events: Vec<Value> = contract.events.iter().map(|e| {
        let mut obj = json!({
            "effect":       e.effect,       // (oracle): field key
            "op":           e.op,           // (oracle): field key
            "op_arg":       e.op_arg,       // (oracle): field key
            "response":     e.response,     // (oracle): field key
            "space_id":     e.space_id,     // (oracle): field key
            "sequence_pos": e.sequence_pos,
        });
        if let Some(prov) = &e.message_provenance {
            // (oracle): field key for message provenance
            obj["message_provenance"] = json!(prov);
        }
        obj
    }).collect();

    let assertion_points: Vec<Value> = contract.assertion_points.iter().map(|ap| {
        match ap {
            AssertionPoint::WatchedInvariant { obligation_id, goal } => json!({
                "kind":           "watched_invariant",  // (oracle): field key
                "obligation_id":  obligation_id,        // (oracle)
                "goal":           goal,
            }),
            AssertionPoint::ConfirmHeld { obligation_id, goal } => json!({
                "kind":           "confirm_held",       // (oracle): field key
                "obligation_id":  obligation_id,        // (oracle)
                "goal":           goal,
            }),
        }
    }).collect();

    json!({
        "schema": "ken.trace/v0",               // (oracle): version token
        "target": contract.target_name,
        "events": events,
        "assertion_points": assertion_points,   // (oracle): field key
        "monitor": {                            // (oracle): field key
            "delegated_obligations": contract.monitor.delegated_obligations,
            // (oracle): concrete Temporal formula body deferred to B2
        },
        "hash": contract.hash,
    })
}
