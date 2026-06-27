# Trace conformance — Ken's observability contract

> Status: **DRAFT v0**. Normative for **Ken's half** — what Ken emits so a
> downstream engine can check that a *running* system refines the model. The
> conformance engine, its mode (gate / monitor / both), and its failure response
> are **out of scope** (a downstream consumer's policy, §4). **`OQ-conformance`
> DECIDED** (operator, 2026-06-27): Ken provides **observability in the model's
> vocabulary** — a trace/instrumentation contract — and nothing about the
> checking mechanism. ADR 0006.

## 1. The question Ken answers

Translation faithfulness (`71 §5`) reduces "the model means the code" to two
parts: the model has **no authoring drift** (it is *generated*, `71 §1`), and
the running code **stays within it** (trace conformance — do the program's
actual behaviors refine the model's allowed ones?). This chapter is the second
part, but only **Ken's contribution** to it:

> Ken's responsibility is to make the running system **observable in the model's
> own vocabulary** (`Σ`). It does **not** check conformance, choose where the
> check runs, or decide what happens on a violation — those belong to a
> downstream consumer (§4).

Almost everything a conformance engine needs is **already in the `71` export**
(it is consumer-agnostic): `Σ` is the event alphabet, `T` synthesizes the
monitors, `Q`/`P` are the invariants/assumptions to watch. The one thing runtime
conformance needs that the offline consumers did not is a **concretization of
`Σ` for a live system** — the contract below.

## 2. The trace/instrumentation contract

A companion to the `71` export, **generated** from the program (same
no-overclaim property), carrying:

- **`Σ` → wire-level events.** *Which* perform points emit, and *what fields*
  each event carries, so a monitor can read the live stream. Instrumentation
  sits at the **effect boundary** — the interaction-tree perform points (`OQ-8`,
  `../30-surface/36 §2`), an already-existing, well-defined, small set — which
  is why runtime overhead is **instrumentation-dominated and bounded**, not
  pervasive code rewriting. The *concrete event schema* (the field-level record
  per perform-node) is the new artifact.
- **Correlation / identity.** In a multi-`space`, message-passing system
  (`OQ-Space`, `../30-surface/36 §4`), events from different spaces must
  **correlate** — space identity and message provenance — for a monitor to
  reconstruct a coherent global (or per-space) trace. Offline model-checking
  glossed this; a live monitor cannot. The contract carries the correlation
  keys.
- **Runtime forms of `Q`/`P`.** The proved invariants `Q` and boundary
  assumptions `P` rendered as **runtime-checkable assertions** at the points
  they apply (a `Q` is a watched invariant; a `P` is a boundary the monitor
  confirms held).
- **The monitor specification from `T`.** The delegated temporal obligations
  (`72`) synthesized into monitors (LTL → Büchi, `README` L3) — *projected* from
  the export, never re-authored.

The trace serialization is **ITF-compatible** (`71 §3`) so the same format spans
counterexamples and live traces.

## 3. The refinement relation

"The implementation refines the model" means: every emitted trace is **accepted
by the model** — it stays within the behaviors `Q`/`Σ`/`T` permit. For safety
and the temporal obligations this is exactly **monitor acceptance** (the
Büchi/MOP monitor synthesized in §2 does not reject). Ken's contribution is to
make this relation *checkable* — emit traces in `Σ` and supply the accepting
monitor; the *act* of checking is the engine's.

## 4. Out of scope — the consumer (a downstream engine + its policy)

Explicitly **not** Ken's, recorded here to fix the boundary:

- **Where the check runs** — CI **gate** (offline, on generated/sampled traces,
  validating the *assumed* distribution), production **monitor** (online, on
  real traces, validating the *actual* distribution), or **both**. The two catch
  divergence on disjoint input sets (sampled vs. real) and differ on prevent-vs-
  detect; choosing among them is a **per-deployment policy**, not a language
  decision.
- **The engine.** Likely a **distinct engine** from the offline model-checker —
  online, low-latency, colocated with the workload (e.g. a **k8s sidecar**),
  with a different failure model. The `71` export is a **broadcast contract** to
  a *family* of consumers (static verifier, test generator, runtime monitor),
  each applying its own policy to many of the same outputs; the runtime monitor
  consumes the trace contract (§2) with a **conformance policy** (refine-or-
  signal) rather than a **discharge policy**. This sharpens ADR 0006: "two
  engines" is really **Ken + a family of behavioral engines sharing one export
  and one logic.**
- **The failure response** — halt / alert / degrade / roll back — is operational
  policy on the consumer side, per environment.
- **Attestation.** Which conformance was performed (gate-only / gate+monitor,
  coverage) is recorded in the **discharge attestation** (`../60-security/63
  §5a`); a deployment gate may *require* live monitoring for an external
  endpoint while accepting gate-only internally — the same per-deployment
  machinery as the sampling policy (`OQ-sampling-policy`).

## 5. What this area must deliver

The trace/instrumentation contract (§2) emitted alongside the `71` export:
concrete `Σ`-event schema at the effect boundary; correlation/identity keys for
multi-space traces; runtime `Q`/`P` assertion points; the monitor spec from `T`;
ITF-compatible serialization. Acceptance: a running program emits a trace a
monitor synthesized from the *same* export can check, with no
separately-authored model; the instrumentation touches only the effect boundary.
The consuming engine and its policy are validated in the sibling's project, not
here. Conformance: `../../conformance/behavioral/trace/`.
