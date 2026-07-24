# RT-NATIVE-FNSPLIT — Boundary B recut: **B1, the semantic-IR representation checkpoint**

**Steward recut, 2026-07-24, in response to Architect ruling `evt_49bnspfb74tne`
(fork (b)) and its addendum `evt_3b2a75fcaegja` (zero outer-helper growth).**
Parent frame: `RT-NATIVE-FNSPLIT-recut.md`. This file is **authoritative for
Boundary B** and supersedes that frame's single-slice "Boundary B — FULL
EMISSION" section.

> ## ⛔ THE ONE THING THAT CHANGED: BOUNDARY B IS NOW **TWO** SLICES
>
> The Architect ruled that *"the exact enum and its exhaustive
> source/control-to-IR builder are a **required representation checkpoint before
> the retained body port resumes**"* and that *"Runtime must not resume the
> retained semantic body port until that representation slice has its own review
> gate."*
>
> ⇒ **B1 = the representation** (this file). **B2 = the retained body port +
> full emission census** (parent frame's Boundary B metrics, unchanged).
> **B1 has its own review gate and its own merge unit.** B2 does not start until
> B1 is reviewed and landed.

## Why the split is not bureaucratic

The stopped port at `415b5aa7` is **+21,544 / −2,086 across 12 files**. Landing
the representation decision inside that diff would put the one thing a reviewer
must scrutinize — a closed opcode grammar with **no admissible wildcard arm** —
inside a change no reviewer can isolate. The three prior recut hard-stops were
all representation questions surfacing late. **B1 exists so the fourth one
surfaces at a review gate instead of at implementation.**

## Fixed inputs — SETTLED, do not reopen

These are ruled. Re-deriving them is out of scope; contradicting them is a
hard-stop, not a judgement call.

1. **Fork (b) is chosen.** A small closed semantic-IR arena with explicit
   static-origin preservation. ⛔ **Do NOT enumerate another `TransitionKind`
   for each exposed `SourcePrefix`/`ProducerKont` responsibility.** That is the
   advisory's *rejected middle* — a taxonomy of lowering accidents with neither
   a derivation nor a structural bound.
2. **Boundary A is retained as the outer control/ownership plan** — landed at
   `647a2e5b`. `PlannedHelperKey`, node/edge IDs, fixed activation roots,
   persistent stores, and `R→W→T→CompletedTail` ownership all stand. Its 11
   transition kinds and 9 edge kinds are **scheduling/authority vocabulary** and
   are not to absorb semantic operations.
3. **⛔ ZERO outer planned helpers per static source may be added.**
   `MAX_HELPERS_PER_STATIC_SOURCE = 8` and measured `fixed_k = 8` at every
   n=3..7 — **the outer inventory is FULL, headroom is exactly zero** (adversary
   H1, measured; Architect addendum `evt_3b2a75fcaegja`). The inner arena is
   semantic material referenced by *already planned* body-owning node
   descriptors. **An IR record is not a helper.** If your design needs one more
   outer helper on any static source, **stop** — that is a hard-stop, not a cap
   bump.
4. **`R`/`W`/`T`/`C` keep their ruled ownership** and may not alias unrelated
   return or source-prefix semantics. `SourceKont`, `SourceArm`, `Arm`,
   `ProducerKont` are **inputs to the lowering, not helper identities.**

## Objective

Build, in production, the **origin-preserving closed semantic-IR plane** that
Boundary A's outer plan can reference — and prove it is closed and bounded,
**before** any retained body is ported onto it.

## Deliverables

### D1 — Static origins, allocated before anything can lose them

A **sole** pre-emission construction pass allocates a dense `StaticOriginId` for
every exact source or generated control occurrence, **before any clone, queue,
or dynamic activation exists.**

- Occurrence-distinct but **textually equal** bodies get **distinct** origins.
- A cloned view may **carry or reference** an existing origin. ⛔ It may **not**
  mint, reconstruct, pointer-infer, or semantic-hash-cons one.

★ This is the direct fix for hard-stop #3's third limb — *"`SourceArm` bodies
lose exact occurrence identity before reserve"*. Allocating first is what makes
the loss impossible rather than detectable.

### D2 — One positional semantic program per planned body-owning node

Fixed-width descriptor: `{planned node, StaticOriginId, SemanticProgramId,
CaptureLayoutId, ruled child node/edge IDs}`.

⛔ **Edges remain body-free transfer contracts.** Zero body-owning edges.

### D3 — The closed operation grammar ⭐ THE REVIEW CHECKPOINT

`SemanticProgramId` indexes an arena of **fixed-width IR records**, with
operands and material held **out of line** by dense ranges/IDs.

**The opcode population must be derived once, exhaustively, from the semantic
lowering primitives** — expression evaluation, value/control transfer,
branch/case selection, invocation/resume, return/completion, affine cleanup.

> ⛔ **NOT from `SourceKont`/`ProducerKont` variant names. NOT one case at a
> time. NO wildcard or fallback arm is admissible.**

**The exact enum and its exhaustive source/control-to-IR builder are the
deliverable this slice exists to gate.** Hand them back for review as the
primary artifact, not as a supporting detail.

### D4 — Bounded expansion

Each registered occurrence is **visited once** and creates at most a fixed K of
semantic records, edges, and operand elements. Variable collections are
flattened **once** into counted operand arenas.

⛔ **A fixed-width range pointing at quadratic material FAILS.** ⛔ No post-ID
subtree clone, path-sensitive replay, cross-product duplication, or
emission-time body reconstruction.

### D5 — Activations never define code

Dynamic work is only `{PlannedHelperKey, DynamicActivationFrame, fixed
scalar/authority payload and store handles}`. It may initialize stores and
invoke a **predeclared** function.

`PartitionSemanticStateKey` / `PartitionContinuationInterner` may remain **only**
as activation/evidence machinery. ⛔ They may not allocate a `FuncId`, select a
body/program, schedule a second definition, or affect helper topology/count.

## ⛔ OUT OF SCOPE — this is B2, and folding it in fails the slice

- The retained #24–#33 semantic **body port** (the bulk of `415b5aa7`).
- CLIF instructions/bytes, compile wall-time, peak RSS.
- The normal/abrupt/trap/join/affine **differential suite**.
- Any historic-baseline comparison. ⛔ Absolute values only; there is no
  baseline, and `1,482/1,525` is not one.

## Acceptance criteria

**AC-B1.1 — Fail-closed, retained verbatim.** A run that cannot complete reports
`could_not_determine` as a **third outcome that FAILS**. Never a silent pass.

**AC-B1.2 — Sole builder.** Exactly one exhaustive origin/IR builder exists; **no
production semantic-definition construction anywhere else.** Prove it
structurally (grep the construction site count and name the predicate), not by
hand-enumeration.

**AC-B1.3 — Exact positional correspondence.** node ↔ descriptor ↔ predeclared
function, one-to-one, with **zero body-owning edges**.

**AC-B1.4 — ⭐ Origin preservation across every retained clone seam.** Equal
syntax at distinct occurrences **stays distinct**; repeated activations of one
occurrence **reuse** the same planned helper/program. Both halves demonstrated.

**AC-B1.5 — Order and state independence.** Permuting discovery order, and
mutating dynamic state, cannot change helper IDs, programs, bodies, or topology.

**AC-B1.6 — ⛔ ZERO outer helper growth.** `fixed_k` measured against landed
`647a2e5b` is **unchanged at 8,8,8,8,8** for n=3..7. This is a *merge condition*.

**AC-B1.7 — Constant opcode vocabulary and inline widths**, asserted
**pairwise-equal across n**, not merely affine. ⚠ See the census note below —
this exact distinction already produced a live defect.

**AC-B1.8 — The strengthened census.** Report, with first and second finite
differences:

| metric | expected |
|---|---|
| opcode vocabulary count | **constant** |
| distinct `StaticOriginId`s | affine |
| IR records · edges · helper definitions | affine |
| definitions **per origin** | **constant** (and 1) |
| **all out-of-line operand elements** | affine — ⭐ *this is the one that catches a fixed-width ID over quadratic material* |
| duplicate-origin definitions · clone count | **zero** |
| max definitions for any one origin | **constant** (and 1) |
| max inline widths | **constant** |

**AC-B1.9 — ⭐ A STRUCTURAL O(n) ARGUMENT, not a table.** State the
one-visit/bounded-material argument in prose and bind it to the code.
⛔ **n=3..7 finite differences are corroboration ONLY.** Research grounded this:
pattern-match compilation to decision trees duplicates rows/submatrices and
explodes in code size while every inline width stays constant (Maranget). **A
locally regular lowering whose cross-products determine growth is exactly our
situation.** Zero second differences at n=3..7 do not exclude it.

**AC-B1.10 — Negative controls, each failing at a NAMED artifact.** Not "it went
red" — *"it went red at this assertion."* Required:

1. pointer-based origin recovery;
2. semantic hash-consing of equal-but-distinct occurrences;
3. a second definition for one planned node;
4. post-origin cloning;
5. **a fixed-width operand range backed by deliberately super-linear material.**

⚠ Commit the real work **before** any mutation-proof reset, or the reset eats
it. ⛔ Never `git stash` — the stash stack is shared across ~70 worktrees.

## ⭐ Before you accept any check in this slice — the discriminating pair

Boundary A's retros found **one** cause behind all three of its review folds:
*proxy-first tests accepted without a property-first counterexample.*

> **Before accepting a check, construct a case that PRESERVES the proxy and
> VIOLATES the property.** If you can build one, the check is broken.

This is not advice; it is why AC-B1.7 says *pairwise-equal* rather than *affine*.
The landed planner **carried** this exact defect — `fixed_k` asserted affine but
not constant, so `4,5,6,7,8` would pass while the zero headroom in fixed input 3
goes negative just past the measured window (adversary H3).

⚠ **CORRECTED 2026-07-24 (Architect, `evt_6091m3nhregch`).**
`RT-PLANNER-DIAGNOSTIC-K` landed at `36dd61f6` and **fixed it**: `fixed_k` is now
in the pairwise-equal list (`static_transition.rs:1485`, asserted at `:1504`),
alongside the surviving `≤ MAX_HELPERS_PER_STATIC_SOURCE` bound (`:1510`).
**Do not read this section as describing live code.**

⇒ What still stands, and is the reason the discipline binds you: the `n=3..7`
census varies **depth only**, so it is **corroboration along one axis — not
closure.** Universal K rests on the exhaustive source-local construction
argument, not on the row. ⚠ **Arithmetic corroborates but cannot close.**

## Escalation — when to hard-stop rather than improvise

- **A genuinely new control-transfer TOPOLOGY** the closed IR cannot express ⇒
  stop and amend the **outer planner** explicitly. ⛔ A new semantic *action*
  alone is **not** grounds for a new `TransitionKind`.
- **Any need for a ninth helper on a static source** ⇒ stop (fixed input 3).
- **Any opcode you cannot derive from the six primitives** ⇒ stop. Do not add a
  wildcard arm; that is the failure this slice exists to prevent.

**Cadence: recut hard-stop count is 3; the next Research pull is #6.** A review
fold is not a hard-stop.

## Branch and sequencing

- **B1 is its own branch and its own merge unit.** ⛔ Do not grow it into B2.
- ✅ **`RT-PLANNER-DIAGNOSTIC-K` (S, Runtime) was sequenced ahead of B1 and
  LANDED at `36dd61f6`** — both touch `planning/static_transition.rs`. Cut B1
  from that result, not from an earlier base.
- ⚠ **`RT-PLANNER-ATTRIB-K` (XS, Runtime) is sequenced STRICTLY AFTER B1** —
  same file. It moves the K-exceeded rejection off the capacity channel per the
  Architect's J1 ruling. ⛔ It does **not** change the cap, the census, or any
  B1 constraint — in particular the zero-outer-helper requirement stands exactly
  as written.
- The stopped `415b5aa7` checkpoint is **useful evidence and a semantic oracle**,
  and it is **not** an acceptance path to complete by adding enum cases. Keep it;
  do not build on it.
