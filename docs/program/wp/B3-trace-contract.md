# WP B3 — trace/instrumentation contract (Ken's half of the Ward runtime seam)

**Owner:** Team Kernel (WS-B). **Branch:** `wp/B3-trace-contract` (cut from
`origin/main`). **Stream / gate:** WS-B → **G-Ward-seam** (B1 export + B3 trace
*complete* the seam). **Depends on:** B1 (the export — `Σ`/`Q`/`P`/`T` source) —
**merged**; X1 (interpreter/effect runtime) — **merged**; L5
space/message-passing
(`36 §4`, correlation) — **merged**. **Spec source:**
`spec/70-behavioral/73-conformance.md`.

> **Steward *frame*** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails; the spec enclave elaborates `73` to team-ready rigor +
> conformance before Team Kernel builds. **Perishable:** B3 *projects from the
> landed B1 export* (`71`/`export.rs`) + instruments the landed `36 §2` perform
> points — pin against the landed code, not this line (K2c-series-2 trap).

## 1. Objective (one line)

Emit the **trace/instrumentation contract** — a *generated* companion to the B1
export that makes a running program **observable in the model's own vocabulary
`Σ`**: a concrete `Σ`-event schema at the effect boundary, correlation keys for
multi-`space` traces, runtime `Q`/`P` assertion points, and the monitor
projected
from `T` — so a downstream engine can check refinement, **without Ken doing the
checking**.

## 2. Settled inputs — FIXED, do not reopen

Per `73` (consumer-agnostic; ADR 0006 "design as one system, build as two"):

1. **Ken's responsibility is OBSERVABILITY, not conformance (`73 §1`).** Ken
   makes
   the system observable in `Σ` and supplies the accepting monitor; it does
   **NOT** check conformance, choose where the check runs, decide the failure
   response, or run the engine — those are the **downstream consumer's** (Ward,
   `§4`). Ken stops at the contract.
2. **The contract is GENERATED, not hand-authored (`73 §2`)** — same
   no-overclaim
   property as the B1 export (projects verified/runtime content, cannot
   over-claim). Almost everything is **already in the B1 export** (`Σ` alphabet,
   `T` monitors, `Q`/`P` invariants); B3 adds the **one** thing offline
   consumers
   didn't need: a **concretization of `Σ` for a live system**.
3. **Instrumentation sits ONLY at the effect boundary (`73 §2`)** — the
   interaction-tree **perform points** (`OQ-8`, `36 §2`), an already-existing,
   small, well-defined set. So overhead is **instrumentation-dominated and
   bounded**, never pervasive code rewriting.
4. **The monitor is PROJECTED from `T`, never re-authored (`73 §2`)** — the
   delegated temporal obligations (`72`) synthesized into monitors (LTL→Büchi);
   it derives from the export, not a separate model. **No authoring drift.**
5. **ITF-compatible serialization (`73 §2`, `71 §3`)** — the same format spans
   B1's counterexamples and B3's live traces. Reuse, don't reinvent.
6. **One-way flow holds (B1 §5):** Ken emits the contract; the engine checks;
   **no
   monitor verdict re-enters Ken as `proved`** (the G-Ward-seam invariant).

## 3. Mandated deliverable outline (each item ends in an implementable choice)

Deliver the trace contract emitter (`ken-elaborator` alongside the B1 export +
`ken-interp` instrumentation) + spec `73`:

1. **The concrete `Σ`-event schema** — the **field-level record per
   perform-node**
   (the new artifact). Pin: which perform points emit, what fields each event
   carries, generated from the landed `36 §2` perform-node signatures (the B1
   `Σ`
   alphabet) — a 1:1 concretization, no second alphabet.
2. **Correlation / identity keys** — `space` identity + message provenance
   (`36 §4`) so a monitor can reconstruct a coherent global/per-space trace. Pin
   the correlation-key set; offline model-checking glossed this, a live monitor
   can't.
3. **Runtime `Q`/`P` assertion points** — the proved invariants `Q` (watched
   invariants) + boundary assumptions `P` (monitor confirms held) rendered as
   **runtime-checkable assertions at the points they apply**, projected from the
   B1 export's `Q`/`P`.
4. **The monitor spec from `T`** — the temporal obligations synthesized to
   monitors
   (LTL→Büchi), **projected** from the export's `T`. Pin the synthesis as a
   projection (`compile : Temporal Σ → Monitor`), reusing the `71 §5`
   faithfulness-lemma discipline (B2/B3 own the `compile` lemma — this is its B3
   home for the monitor direction).
5. **ITF-compatible serialization** — the live-trace wire form = the B1 trace
   layer's ITF format. Field spellings `(oracle)`-tagged where Ward finalizes
   them (defer-spelling-not-concept).

## 4. Testable acceptance criteria

- **AC1 (generated, checkable end-to-end)** A running program emits a trace in
  `Σ`
  that a monitor **synthesized from the *same* export** accepts — **no
  separately-authored model**. Structural: the monitor derives from the export's
  `T`, the trace from the export's `Σ`.
- **AC2 (effect-boundary only — bounded)** Instrumentation touches **only** the
  perform points — assert **no** instrumentation outside the `36 §2` effect
  boundary (the bounded-overhead guarantee; a structural absence-assertion).
- **AC3 (multi-space correlation)** Events from two distinct `space`s carry
  correlation keys that let a monitor reconstruct a coherent trace — a
  discriminating case: correlated events link, uncorrelated don't.
- **AC4 (runtime `Q`/`P`)** A proved `Q` emits a watched-invariant assertion at
  its
  point; a boundary `P` emits a confirm-held assertion — projected from the
  export, not re-authored.
- **AC5 (monitor projected, not authored)** The monitor is the projection of the
  export's `T` — assert it changes when `T` changes (a re-authored monitor
  wouldn't); never a hand-written model.
- **AC6 (one-way / G-Ward-seam)** **No monitor/engine verdict re-enters Ken as
  `proved`** — there is no ingest path; the contract is emit-only (the seam
  gate,
  reused from B1).
- **Conformance:** `conformance/behavioral/trace/` — AC1–AC6, per-case
  verdict/structural-flip + cross-case sweep. **QA gate:** route a **real**
  program through the **actual** instrumentation + emitter and observe the real
  `Σ`-events/correlation-keys/assertion-points, never a synthetic trace literal.

## 5. Do-not-reopen guardrails

- **Ken emits, does not check** (`73 §1/§4`) — the engine, the check location,
  the
  failure response, attestation are **Ward's**; do not build them. B3 is the
  *contract*, not the consumer.
- **Generated, never authored** (§2.2) — projects the export; no drift.
- **Effect-boundary instrumentation only** (§2.3) — bounded overhead, no
  pervasive
  rewriting.
- **Monitor projected from `T`** (§2.4) — never a separate model.
- **One-way flow** (§2.6) — no engine verdict becomes `proved` (the
  G-Ward-seam).
- **`Σ` is B1's perform-node alphabet** — concretize it, don't invent a second.

## 6. Sequencing notes

- B3 **completes the Ward seam** with B1 → flips **G-Ward-seam** (Ken emits a
  reproducible export + trace contract a *stub* consumer can read; no Ward
  result
  ever `proved`). After B3, **Ward** (the sibling project) can be stood up
  against
  the stable seam (B1–B3); **B4** (agentic boundary, needs Sec1+Sec2+B3) is the
  remaining WS-B WP.
- The `compile : Temporal → Monitor` faithfulness lemma is B3's home (B1
  deferred
  it to B2/B3) — couples to **B2** (the `Temporal` datatype, needs L2). If B2
  hasn't landed, pin the monitor projection over the **`T` channel B1 fixed** +
  `(oracle)`-tag the `Temporal` surface, defer the full lemma to B2.
- Standard §2c: frame → spec-leader elaborates `73` + conformance → merge
  (Architect + conformance-validator) → Team Kernel compacted, then kicked off.
