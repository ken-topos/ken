# WP B1 — assumption-boundary export emitter (Ken's half of the Ward seam)

**Owner:** Team Kernel (repurposed to WS-B — operator-confirmed; WS-K complete).
**Branch:** `wp/B1-export-emitter` (cut from `origin/main`).
**Stream / gate:** WS-B (tier-1 seam) → **G-Ward-seam**. **Depends on:** V1
(four-way status + spec syntax) — **merged**; L5 (interaction-tree denotation, Σ
= perform-node signatures) — **merged**. **Spec source:**
`spec/70-behavioral/71-assumption-boundary.md` (whole chapter).

> **Steward *frame*** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails. The **spec enclave elaborates `71` to team-ready rigor
> +
> conformance** before Team Kernel builds. **Any "current state" line is
> perishable — verify against the landed code/spec at pickup, do not build from
> this line** (K2c-series-2). **NEW DOMAIN for Team Kernel** (first WS-B WP) —
> but
> the emitter lives in `ken-elaborator` and *projects already-verified content*,
> so it is elaborator-adjacent, not new theory.

## 1. Objective (one line)

Emit the **behavioral export**: a *generated* (never hand-authored), versioned,
content-addressed **five-part assume-guarantee contract** (`Q`/`P`/`Σ`/`T`/`G`)
plus an ITF trace layer — the stable seam artifact a *family* of `Ward`
consumers
reads, faithful to exactly what Ken **proved / assumed / delegated**,
structurally
unable to over-claim.

## 2. Settled inputs — FIXED, do not reopen

Decided in `71` (+ ADR 0006 "design as one system, build as two"; `OQ-8`,
`OQ-classical-bridge`). Pin them:

1. **Five-part schema (`71 §2`), exactly.** `Q` guarantees (`proved`
   postconditions/space-invariants) · `P` assumptions (`trusted_base_delta` +
   explicit `assume`s + boundary labels, status `tested`) · `Σ` alphabet (the
   interaction-tree **perform-node signatures**, `OQ-8` — *reuse, not reinvent*)
   ·
   `T` obligations (the `Temporal` data values stated in source, status
   `delegated`) · `G` generators (refinement/dependent-type **support**,
   derived).
2. **Generated, never hand-authored (`71 §1`).** Every field is a **projection
   of
   Ken's verified content** — so the export **cannot over-claim** (it states
   exactly the four-way status `21 §5`, no more). The model is a *function of
   the
   code*; this is the structural antidote to model↔code drift.
3. **Two layers (`71 §3`).** Contract layer (`Q`/`P`/`Σ`/`T`/`G` predicates) =
   **Ken-native** (semantically faithful — "one logic, two engines" depends on
   it). Trace/witness layer = **ITF** (Informal Trace Format — Quint/Apalache
   interop, no bespoke format).
4. **Versioned + content-addressed + travels in provenance (`71 §3`, `63 §2`).**
   The export hash links *this model* to *this build* — "this Ward model
   corresponds to this code" is checkable, not asserted.
5. **`G` carries *support*, never *measure* (`71 §4`).** Ken exports the
   equivalence-class **partition / boundaries / case-decomposition** (derivable
   from refinement predicates + `match` arms), claiming nothing about
   likelihood.
   The **sampling policy** (which valid states are likely/risky) is
   **Ward-side**,
   out of Ken source (`OQ-sampling-policy`). Exclusions bifurcate:
   invariant-type →
   rides `P`/`Q`; judgment-type → measure → Ward. **A `no-measure` invariant is
   load-bearing.**
6. **One-way flow, strictly (`71 §5`).** Ken *exports* `T`/`P`; Ward discharges
   by
   classical means; **results NEVER re-enter Ken as proof terms.** A discharged
   obligation stays `delegated`/`tested`, rides `trusted_base_delta`, is **never
   promoted to `proved`** (this is the **G-Ward-seam** gate). Assume-guarantee:
   every Ken theorem is "**given `P`, then `Q`**", kernel-checked regardless of
   how
   `P` is later discharged — no classical strength leaks into the kernel.

## 3. Mandated deliverable outline (each item ends in an implementable choice)

Deliver the **export emitter in `crates/ken-elaborator`** + spec `71`:

1. **The five-part projection.** For each of `Q`/`P`/`Σ`/`T`/`G`, pin the
   **source
   of truth** it projects from and the projection function: `Q` ← proved
   postconditions/invariants (V-status `proved`); `P` ← `trusted_base_delta`
   (`25 §3`) + `assume`s + boundary labels; `Σ` ← L5 perform-node signatures
   (`36 §2`) **verbatim** (no second alphabet); `T` ← `Temporal` source values
   (the *channel*; the `Temporal` **datatype** itself is **B2** — emit what
   exists,
   structure the channel); `G` ← refinement-predicate + `match`-arm partition.
2. **The serialization — two layers.** The Ken-native contract serialization
   (value-set + cross-field invariants pinned; literal field **spellings**
   `(oracle)`-tagged where not yet locked, per defer-spelling-not-concept — Ward
   finalizes the wire token, Ken locks the concept + stability discipline) + the
   **ITF** trace layer for witnesses.
3. **Content-addressing + provenance.** Hash the export; embed in provenance
   (`63 §2`) so `export_hash ↔ build` is reproducible.
4. **The `no-measure` invariant — exhaustive-by-construction.** Encode the
   support/measure split so a `G` field **cannot** carry a likelihood/weight —
   make
   "measure leaked into the export" a **compile error / type-level
   impossibility**,
   not a convention (COORDINATION §7 exhaustive-by-construction).
5. **The one-way-flow + `delegated`-status discipline.** No code path promotes a
   delegated obligation to `proved`; the emitter only ever *reads* verified
   status
   and *projects* it. (The `compile : Temporal Σ → WardFormula` **faithfulness
   lemma** `§5` is **coupled to B2/B3** — name it as the downstream owner; B1
   fixes
   the `T` channel + `Σ` alphabet it will reuse, not the lemma.)

## 4. Testable acceptance criteria

- **AC1 (reproducible)** Same checked program → **same export hash**
  (deterministic
  projection; structural assertion on the hash, not just "an export is
  produced").
- **AC2 (no over-claim)** The export **never asserts a claim Ken did not prove
  or
  state** — every `Q` entry traces to a `proved` result; every delegated `T`
  carries status `delegated`, never `proved`. *Discriminating:* a program with
  an
  *unproved* postcondition must emit it under `P`/`assume`, **not** `Q` (verdict
  flips between proved and assumed — not green-vs-green).
- **AC3 (assumption visibility)** Removing an `assume` / shrinking the
  `trusted_base_delta` shows up as a **changed `P`** (and a changed export
  hash).
- **AC4 (Σ reuse)** `Σ` equals the L5 perform-node signatures of the program —
  **not** a re-derived alphabet (assert structural equality to the denotation's
  signatures).
- **AC5 (no-measure)** No `G` field can carry a weight/likelihood — attempt must
  be
  rejected (type error / not representable). The exported `G` is partition +
  boundaries only.
- **AC6 (one-way / G-Ward-seam)** **No Ward result is ever recorded as
  `proved`** —
  there is no ingest path; a discharged obligation stays `delegated`/`tested`.
  (This is the gate acceptance; assert there is no code path from a Ward verdict
  to
  a `proved` status.)
- **Conformance:** `conformance/behavioral/export/` — AC1–AC6, per-case
  verdict/structural-flip + a cross-case sweep (the status-projection class:
  proved→Q, tested→P, delegated→T all agree). **QA gate (2-team build-qa
  lesson):**
  each case must **route through the actual emitter projecting real verified
  content**, not *predicate about* a synthetic export struct — a test that
  builds
  an export literal and checks a field guards nothing; drive a real checked
  program
  through the emitter and observe the projected field.

## 5. Do-not-reopen guardrails

- **No new alphabet** — `Σ` IS the L5 perform-node signatures (§2.1).
- **No measure in the export** — support only; sampling policy is Ward-side
  (§2.5).
- **No Ward result re-enters Ken** — one-way; nothing becomes `proved` (§2.6).
  Over-claiming here breaks the seam's entire trust story.
- **No kernel enlargement** — the emitter reads verified content + projects; it
  proves nothing new. The exported predicates are **trusted projections** — the
  kernel does not re-check the *serialization*, so **conformance is the net**
  for
  projection fidelity (enumerate what the kernel does not see: the export
  bytes).
- **`Temporal` datatype + `compile` faithfulness = B2/B3**, not B1 — B1 fixes
  the
  `T` channel + `Σ`, B2 builds `Temporal`, B3 the trace/runtime contract.

## 6. Sequencing notes

- B1 **opens WS-B**; **B2** (Temporal-as-data, needs L2 + B1) and **B3** (trace
  contract, needs B1 + X1) follow on Team Kernel. The `T` channel + `Σ` alphabet
  B1 fixes are what B2/B3 reuse — coordinate via spec, do not hard-bind wire
  spellings cross-WP.
- The CT signature promise `Q` from **Sec1ct** (Team Verify, in elaboration) is
  a
  `Q`/`P` producer for this export — coordinate the boundary-`Q` shape via the
  spec; do not pre-bind field names across the two WPs.
- Standard §2c pipeline: frame → spec-leader elaborates `71` + conformance →
  merge
  (Architect + Spec, touches `spec/`+`conformance/`) → Team Kernel compacted,
  then
  kicked off on this branch.
