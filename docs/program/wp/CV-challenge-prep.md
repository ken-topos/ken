# CV-challenge-prep — PREPARE the challenging Ken-specific conformance exercises

**Steward frame → conformance-validator (direct, operator-mandated).** A
**PREPARE-only** authoring task, not a merge WP: no gate, no `/spec` change, no
Integrator. Owner: **conformance-validator** (the durable independent checker).
This aligns with CV's already-tracked task *"Verified-showcase examples: propose
selection, then author."* Findings / questions → **Steward**.

## Why — depth after breadth
VAL2 (Rosetta pangram, just closed: 10 PASS / 6 KNOWN-GAP) validated the
surface **breadth** — output, loops, recursion, strings, ADTs, HOFs, effects —
and surfaced 11 findings (5 genuine capability gaps + 1 elaborator bug + fixes).
It deliberately did **not** reach the **depth** of Ken's *distinctive* value:
the properties that make Ken Ken. This task prepares the **blind-spot
instrument** — challenging exercises that probe that depth, where Ken's
conformance story is either thin, unexercised, or genuinely novel.

## The axes to probe (CV proposes the final selection)
These are the distinctive-feature axes VAL2's breadth pangram didn't reach —
suggestions, not a mandate; **you own the selection** (propose it first, per your
own tracked task):
- **Lawful typeclasses carrying real law proofs** — `Ord`/`Eq`/`DecEq`
  instances whose dictionaries carry *actual* total-order / decidability proof
  terms, not stubs (the deferred lawful-classes work; the discriminating case
  must FAIL against a law-less instance). Includes the canonical-carrier
  soundness edge (`DecEq` deliverable over a canonical carrier, false over a
  non-canonical one).
- **Codata / coinduction under totality** — streams / infinite structures that
  are productive-not-terminating; how Ken expresses guarded corecursion without
  breaking the totality guarantee.
- **Dependent types / indexed families / dependent elimination** — length-
  indexed vectors, proof-carrying `match`, motive-driven elimination beyond the
  simple `data`+`match` VAL2 used (adjacent to the ≥2-recursive-field bug
  finding #5, which is a live elaborator gap — probe around it, don't require it).
- **Proof-carrying / verified programs** — a verified-sort (sortedness +
  permutation evidence), where the `Perm` relation must be expressed at the
  right universe (`‖Perm‖` truncation or count-equality, NOT a proof-relevant
  inductive at Ω — the known kernel constraint).
- **The Ω / strict-prop boundary** — programs that exercise proof-irrelevance
  and where it bites (sub-singleton vs. proof-relevant).

Ground the selection in VAL2's findings ledger: the 5 capability gaps (Map,
`[FS]`, `[State]`, mutual-recursion, ≥2-rec-field-match) mark *known* thin
spots — the challenge suite should stress **adjacent** depth and the axes VAL2
didn't touch **at all**, not re-probe the already-documented gaps.

## Deliverable — PREPARED, not run
1. **Propose the selection first** (your tracked "propose selection, then
   author" step) — a short list of the exercises with, for each: the axis it
   probes, why it's a meaningful blind-spot test, and its *expected* conformance
   behavior (should-pass / should-reject / known-gap-with-reason).
2. **Author the prepared suite** — the exercise sources + expected-behavior
   specs + a one-line "what this discriminates" per exercise. Staged, ready to
   run.

## ⛔ HARD CONSTRAINT — DO NOT RUN. STOP AT PREPARED.
**Prepare and stage the exercises; do NOT execute them against the
implementation.** This is an explicit operator instruction: the operator wants
to be present when the challenge suite is first run, because the results may
surface hard conformance failures or design-level questions that are the
operator's call. Author + expected-behavior + staged = done. **Then stop** and
report the prepared suite to the Steward for the operator's return (~11:00 UTC).

## Guardrails
- **Clean-room:** you may read the *permissive* references to sharpen an
  exercise's design (Spec-enclave privilege), never copy them in. No AGPLv3
  `yon` contact. Copyleft refs approach-only under the leakage recheck.
- **PREPARE only** — no execution, no conformance-corpus wiring into CI, no
  `/spec` change. This is a staged instrument, not a landed WP.
- Propose-before-author — don't sink effort into authoring before the selection
  is sound; a quick Steward round-trip on the proposed list is cheap insurance.
