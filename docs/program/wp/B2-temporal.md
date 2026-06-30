# WP B2 — `Temporal` obligations as data

**Owner:** Team Kernel (WS-B). **Branch:** `wp/B2-temporal` (cut from
`origin/main`). **Stream / gate:** WS-B → feeds the Ward seam (B1 export carries
`Temporal` as the `T` channel; B3's monitor projects from it). **Depends on:**
L2
(inductive data + `match` — `Temporal` IS ordinary `data`) — **merged**; B1 (the
export `T` channel) — **merged**. **Spec source:**
`spec/70-behavioral/72-temporal.md` (+ `71 §2` alphabet `Σ`, `14` inductives,
the
4-way epistemic status).

> **Steward *frame*** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails; the spec enclave elaborates `72` to team-ready rigor +
> conformance before Team Kernel builds. **Perishable:** `Temporal` is built on
> the **landed L2 inductive machinery** + flows into the **landed B1 export `T`
> channel** — pin against the code, not this line.

## 1. Objective (one line)

Deliver `Temporal` — a **deeply-embedded LTL/μ-calculus inductive datatype**
over
the event alphabet `Σ`, with `temporal{}` surface notation, tagged `delegated`,
flowing into the B1 export for **Ward** to discharge — **with NO modal layer
added to Ken's kernel** (Ken reasons *about* temporal formulas as data, never
*with* `▷` modalities).

## 2. Settled inputs — FIXED, do not reopen

Per `72` (a **durable** decision, not a v1 expedient):

1. **`Temporal`-as-DATA, never kernel modalities (§1).** Ken **states** a
   temporal
   property as an ordinary **deeply-embedded inductive value**; it is
   **exported**
   (`71`) and discharged by **Ward** (model-checking/monitoring), **not** in
   Ken.
   Ken gains **NO** guarded/`▷`-modal layer — no "later" modality, tick
   variables,
   Löb rule, or clock structure (that is TCB growth + new metatheory, the ADR
   0005
   rejection). The consistent application of
   *deep-embed-and-reason-reflectively*
   (OTT `Eq`-as-data, reflective `decide`, `Temporal`-as-data — the same move).
2. **Reason ABOUT formulas, not WITH modalities (§2).** Because `Temporal` is
   ordinary data, Ken's existing static logic **may** prove properties *about*
   formulas (an LTL→NNF rewrite preserves semantics; `simplify` is sound +
   terminating; an obligation is well-formed/closed) but **may not** discharge
   `□(req → ◇resp)`, decide liveness/fairness, or quantify over infinite traces
   (that needs the temporal model — Ward). The proofs Ken does here are
   **unremarkable static proofs about an inductive type — no new kernel power.**
3. **The datatype is over `Σ`, inert to the kernel (§3).** `Temporal Σ` is a
   deeply-embedded logic over the **effect/event alphabet `Σ`** (the
   perform-node
   signatures, `71 §2` — the same atoms Ward monitors). DRAFT core:
   `atom`/`not`/
   `and`/`or`/`next`/`until` + μ-calculus `mu`/`nu`/`var`; `◇φ := until true φ`,
   `□φ := not (until true (not φ))`. It is a **normal inductive type** (`14`):
   elimination is ordinary `elim`; **nothing touches conversion or judgmental
   structure.**
4. **Encoding is DEFERRED (§3, with Ward).** The **exact constructor set, the
   `Pred Σ` atom language, the fixpoint-variable discipline, and the ITF-facing
   serialization** are an encoding pass with Ward — **NOT fixed here.** Pin the
   *concept + the LTL/μ core + the inert-data property*; `(oracle)`-tag the
   exact
   spelling (defer-spelling-not-concept).
5. **Surface `temporal{}` → `delegated` (§4).** A `temporal{}` block (or a
   `delegated`/`assume` clause) — `always`/`eventually`/`until`/`next`/`leadsto`
   (`p ~> q := always (p → eventually q)`) — elaborates to the constructors, is
   tagged **`delegated`** in the 4-way epistemic status, is **human-visible** in
   source, and **flows into the export** (`71`). Concrete keywords settle with
   `OQ-syntax` (oracle-tag).
6. **Unbounded liveness is NOT a kernel modality (§5).** If unbounded liveness
   ever becomes load-bearing, the response is a **contained reflective model**
   (the `OQ-12` move), **never** kernel modalities. Out of scope here.

## 3. Mandated deliverable outline (each ends in an implementable choice)

Deliver in the kernel/elaborator (as ordinary inductive data + surface
notation):

1. **The `Temporal Σ` datatype** — declared via the **landed L2 `data`
   machinery**
   (`14`/K1) over the `Σ` alphabet (the B1 export's effect-label `Σ`); the LTL/μ
   core (§2.3) with the derived `◇`/`□`/`leadsto`. Pin the concept + value-set;
   `(oracle)`-tag the exact constructor spelling + `Pred Σ` (the Ward encoding
   pass). **No new kernel rule** — ordinary inductive, ordinary `elim`.
2. **Surface `temporal{}` notation** → the constructors; the `delegated` tag
   (4-way
   status); human-visible in source. Pin the elaboration; oracle-tag keywords.
3. **Export flow** — a `Temporal` value flows into the **landed B1 export** as
   the
   `T` (delegated) channel; **never** projected to `Q`/`P` (it is delegated,
   Ward
   discharges). Pin the wiring to `export.rs`.
4. **Reason-about operations (the metatheory of the embedding)** — at least one
   verified `Temporal`-level operation (e.g. an NNF/`simplify` rewrite) proved
   sound by **ordinary static proof** over the inductive type — demonstrating
   reason-*about* without reason-*with*.

## 4. Testable acceptance criteria

- **AC1 (`Temporal` is ordinary inert data — the durable headline, structural)**
  `Temporal Σ` is a normal inductive type with ordinary `elim`; **the kernel
  gains
  NO modal judgment** — assert the **absence** of any `▷`/later/tick/Löb/clock
  construct in the kernel (the grep-for-forbidden-construct net), and that
  conversion/judgmental structure is unchanged.
- **AC2 (derived operators)** `◇`/`□`/`leadsto` elaborate to the `until`-based
  core
  (structural on the elaborated term).
- **AC3 (surface → `delegated`)** A `temporal{}` claim elaborates to the
  constructors and is tagged **`delegated`** (4-way status), human-visible.
- **AC4 (export flow, never `Q` — discriminating)** A `Temporal` value flows
  into
  the export as the **`T`/delegated** channel and is **never** projected to `Q`
  or
  `P` (it is Ward's to discharge) — route through the **real** B1 export; the
  verdict is `delegated`, never `proved` (B1's discriminator; couples to B3's
  monitor projection).
- **AC5 (reason-ABOUT, not WITH)** A static proof **about** a `Temporal` formula
  (e.g. a rewrite preserves a stated property) type-checks (ordinary data); but
  there is **no** way to discharge the temporal obligation itself inside Ken
  (no modality) — assert both faces.
- **Conformance:** `conformance/behavioral/temporal/` — AC1–AC5, the no-kernel-
  modality absence-assertion + the delegated-never-Q discriminator + cross-case
  sweep. **QA gate:** the datatype routes through **real** `elim`; the
  export-flow
  through the **real** B1 emitter; no synthetic `Temporal` literal where a real
  elaboration is asserted.

## 5. Do-not-reopen guardrails

- **NO kernel modality** (§2.1) — no `▷`/later/tick/Löb/clock; nothing touches
  conversion or judgmental structure. This is the **durable** core decision —
  assert the absence structurally.
- **Reason ABOUT, not WITH** (§2.2) — Ken proves about formulas (data); Ward
  discharges the obligations.
- **Encoding deferred** (§2.4) — pin the concept + LTL/μ core + inert-data
  property; `(oracle)`-tag the exact constructor set / `Pred Σ` / serialization
  (the Ward pass).
- **`Temporal` is `delegated`, never `Q`/`P`** (§2.5) — flows into the export,
  Ward
  discharges; never kernel-proved.
- **Built on L2 `data`** — ordinary inductive, ordinary `elim`; no special form.

## 6. Sequencing notes

- B2 **closes the `Temporal` half of the Ward seam** — B1 (export) + B3 (trace +
  monitor projection) already land it; B2 supplies the **datatype** B3's
  `compile : Temporal → Monitor` projects from (B3 oracle-tagged the `Temporal`
  surface pending this WP). Keep the `T`-channel wiring consistent with the
  landed
  B1/B3.
- The exact encoding is a **joint pass with Ward** — pin Ken's side (inert data,
  delegated, exported), defer the wire/constructor spelling.
- Standard §2c: frame → spec-leader elaborates `72` + conformance → merge
  (Architect + conformance-validator) → Team Kernel compacted, then kicked off.
