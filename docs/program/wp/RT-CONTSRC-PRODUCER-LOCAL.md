# RT-CONTSRC-PRODUCER-LOCAL — the producer-local continuation source coordinate

**A value created mid-body is a third availability class. Continuation
specialization can name entry values and generated-context captures, and cannot
name a host-effect result or a `Match` case binder — so the environment for
those edges never closes and the specialization is never committed. This node
adds that coordinate domain.**

**Owner:** Team Runtime. **Size:** L.
**Node:** `docs/program/issues/RT-CONTSRC-PRODUCER-LOCAL.md`.
**Risk:** medium-high — it widens a planner/ABI representation, and the
Architect has already ruled the naive shape unlawful.

**Authority:** Steward ruling 2026-08-05 at [[RT-DECL-CLOSURE-PORT]] checkpoint
`1f`, on measurements `evt_5kws532ac99c9` and `evt_5ngh190h9b1k5` and the
Architect representation gate `evt_75k8cydbj5127`.

---

## 1. Base and fixed inputs

**Governing base: exact `179af86350ba7191935fcc9ff902bb166c954339`**, on branch
`wp/RT-DECL-CLOSURE-PORT-typed-units`. **Continue that branch.** It is not on
`main`, so a fresh branch cut from it gains no independent mergeability and
risks losing the proved lineage. `D7` checkpoints 1, `1b` and `1c` are proved
substrate and are preserved byte-identically.

⛔ **Rebase, merge or cherry-pick of `fb8fd881`, `430798bf`, `548682c3`,
`42ccd8ec` remains banned** — competing historical implementations; importing
one reintroduces the role/disposition-derived schema the host-effect ruling
ruled false.

| path | blob at `179af863` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs` | `e66e423c9991a406694a9e1a59d58906f3f94929` |
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `949a4bea2cd53c840ba63f3320dbfc3f2eb5550a` |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | `fc6981119d0f2bb3023d4a4abc6b46fabfcb2771` |
| `crates/ken-cli/tests/rt_parity_native.rs` | `b2df2bbd00644b907cae5d05efa76edd9df1b3f2` |

**Baselines at that base, per-row and not as totals:** `ken-runtime` lib
**718 passed / 2 failed** (two standing reds), `ken-elaborator` lib **108 / 0**,
`D0` parity **1 passed / 6 failed** with `buffer_freeze` the passing row.

## 2. The measured gap

`ContinuationInputSource` (`static_transition.rs:410`) is `Parameter` /
`LexicalCapture` / `SeedCapture`, and its enclosing record requires an
**entry-ABI coordinate**. `continuation_owner_entry_sources` enumerates exactly
`parameters + captures`; every carrier, ownership, storage-owner, affinity and
equality check derives from that exact `AbiSlot`.

A host-effect result or a `Match` case binder is created **after entry**. It is
neither a parameter/capture nor the unit's outgoing `Result` convention slot.
The emission seam agrees independently: its exhaustive two-class resolution
covers an entry value in its root owner and a value captured by a generated
context.

**Closure-edge census at the base:** 34 case-binder-only, 4
effect-result-plus-case-binder, 1 `Construct`-only. The four mixed edges span
all six failing `D0` rows.

## 3. Deliverables

- **`D0` — the delta-free baseline.** Before any delta, record which rows are
  green at `179af863`, per row. ⛔ A measurement carrying your own delta cannot
  produce it.

- **`D1` — the two coordinate domains, separated.** Not a fourth enum arm.
  1. **Entry ABI source** — existing owner + parameter/capture position and its
     slot-derived contract, **unchanged**.
  2. **Producer-local source** — an exact structural binding identity in the
     producer body, with planner-derived carrier / ownership / storage /
     affinity, and an exact emission-time locator into the environment that
     actually contains it.

  The source-position type is a **closed sum**: an entry position must not be
  representable as a local binding, or the reverse. ⛔ No default arm.

- **`D2` — coverage of both binding kinds.** The host-effect result **and** the
  exact `Match` case binder, as **distinct** structural bindings. A later common
  local-binding representation may subsume them; this node does not assume one.

- **`D3` — the three consumers, each handled explicitly.**
  - `validate_continuation_source_slot` re-derives the same arm and contract.
    ⛔ No exemption for the new arm — it is the only exact validator.
  - Generated-context capture lookup compares the **full** source coordinate.
  - The emission resolver handles the local arm **explicitly**. ⛔ No default,
    no fallthrough.

- **`D4` — broad admission.** Every exact producer-local value is represented
  and **all** newly representable candidates may lawfully intern. Report the
  resulting intern/decline census as a **vector over the full required
  environment**.

## 4. Acceptance criteria

- **`AC-1` — the linked row closes.** The `D0` row
  `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` reaches the
  real producer and returns `InvalidBounds` at the exact `264 -> 262 /
  position 1` consumer, with shared-host dispatch count **zero**. Removing the
  carried-capacity arm recreates the refusal at that exact seat.
- **`AC-1b` — per row, never a total.** Every row green in `D0` is still green,
  stated per row. ⛔ A pass/fail count is not evidence: it reads identically
  before and after, and that is what hid two of this campaign's false laws.
- **`AC-2` — the closed sum is enforced by the type, not by convention.** A new
  source kind must be unable to compile until every one of `D3`'s three
  consumers assigns it. ⛔ No wildcard arm.
- **`AC-3` — the 34 newly-interning edges are accounted for individually.** Name
  them, and show for each that interning is lawful. ⛔ An aggregate "no
  regressions" claim does not discharge this — a differential over an aggregate
  passes while one of N contributors defects.
- **`AC-4` (no-regression).** Workspace green **in CI** — ⛔ never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-5` — `1c`'s converse survives.** The interned-to-member law and its four
  mutation controls remain intact and **non-vacuous**: show each still fails
  when its target is mutated. Broad admission changes the interned population,
  which is exactly the condition under which a control can silently go vacuous.

## 5. Banned scope

- ⛔ **A fourth `ContinuationInputSource` case** while the enclosing record
  still requires an entry-ABI coordinate. The Architect rejected this shape
  explicitly.
- ⛔ **Claiming a mid-body value exists at function entry** — widening
  parameters/captures to seat it, inventing an entry position, or reusing
  `AbiSlotKind::Result` (a different boundary direction).
- ⛔ **Exempting the new arm from `validate_continuation_source_slot`**, or
  using `immediate_slot` alone and discarding root provenance.
- ⛔ **Any route-modality or edge-selection authority.** Broad admission
  dissolves the need. If you find yourself needing one, that is a finding about
  `D4`'s scope — hard-stop and return it.
- ⛔ **Corpus identity, closure identity, first-`Open` reason, or planned-member
  status as a predicate.** All four are forbidden substitutes for a real
  authority, and `member=true` is measured constant across all 612 declines and
  all 489 interns, so it discriminates nothing.
- ⛔ **Special-casing closure `381`** or any named closure.

## 6. The standing methodological requirement

**Validate the full required environment as a vector. First-`Open`
classification is not a population oracle.**

This is not general advice; it is the specific defect that produced a false
minimality ruling on 2026-08-05. "6 effect edges equal the 6 failing rows" was a
pair count short-circuited at the first `Open` position, compared against a 161
that was in a different unit. The effect-result-only population is **zero**.

⇒ Every census this node produces **states its unit** and answers *what does
this edge require*, never *where did it first stop*.

## 7. Hard stop

Stop and report, with the concrete edge, if:

- a lawful producer-local coordinate cannot be expressed without one of the five
  exits the Architect closed;
- broad admission turns out to require an edge-selection authority after all; or
- closing the case-binder binding perturbs a row that `D0` recorded green, in a
  way the per-row evidence cannot account for.

⛔ Do not absorb any of these and do not work around them.
