# RT-DESCENT-RETIRE — delete the selector, the enum, the authority, and the lane

**With all five residual classes retired, the migration selector still exists,
still evaluates on every compilation, and the `RecursiveDescent` emission lane is
still compiled in — dead. This node deletes it. That residue is the tech debt the
directive names, so this is a required node, not a tidy-up.**

**Owner:** Team Runtime. **Branch:** `wp/RT-DESCENT-RETIRE`. **Size:** M.
**Risk:** medium — a wide deletion across five production files, with a
**one-shot** oracle.

⛔ **Read `docs/program/16-recursive-descent-retirement.md` first.**

**Gated on five nodes, not four.** Do not start until
[[RT-DECL-CLOSURE-PORT]], [[RT-SEED-CALL-PORT]], [[RT-PRODUCER-MATCH-PORT]],
[[RT-RECURSOR-TRANSPORT]] **and [[RT-FNUNIT-RESULT-TOKEN]]** have merged.

**The fifth is not a migration node and its gate is not "the class is
retired."** `RT-FNUNIT-RESULT-TOKEN` owns `nc22`, the only program exercising a
shape that **only the lane you are deleting supports**. It is `#[ignore]`d under
that node's quarantine — so if you delete the lane first, the capability
disappears and **the single row that would have caught it is already
suppressed**. Its gate is `nc22` running **green on the `FunctionizedUnits`
lane**, not the skip being tidied. Added 2026-08-08 by the Steward; that node
was filed after this frame was written.

---

## 1. Fixed inputs

| path | blob at `origin/main = 14c3c5f7` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `f7bc0d0354d8b8d6f7aa68176846b7b05e5a8514` |
| `crates/ken-runtime/src/cranelift_backend/lowering/mod.rs` | `b924db34df3be74421fa773132fe476a53503ecc` |
| `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs` | `f9d7fc1025bfa80cb5eaf66284252d3bdd59c28c` |
| `crates/ken-runtime/src/object_linker_packaging.rs` | `59d2940576894f516494c28c5b8d66a8260337f8` |

⚠ **Every one of these is stale by pickup** — four nodes run first. **Re-pin at
pickup.** ⛔ These are recorded to bound the *surface*, not to be trusted as
values.

## 2. The surface

At `origin/main = 14c3c5f7`, `BodyEmissionAuthority` / `RecursiveDescent`
occurrences span **five production files plus three test modules**:

| file | occurrences |
|---|---|
| `lowering/core.rs` | 22 |
| `lowering/core/tests/control.rs` | 16 |
| `lowering/mod.rs` | 4 |
| `planning/static_transition.rs` | 3 |
| `object_linker_packaging.rs` | 1 |

Plus `core/tests/constructors.rs` and `core/tests/effects.rs`.

⚠ **A deletion that misses a file leaves a dead branch that still compiles.**
The count above is the pre-campaign surface and will have moved; ⛔ **re-derive
it, do not re-pin it.**

## 3. ⛔⛔ THE ORACLE IS SPENT BY THE COMMIT THAT CLEARS IT

Once the last residual class is retired, **nothing in the tree can distinguish
"the lane is unreachable" from "the lane was deleted."** The evidence that the
lane is genuinely dead exists **only before this node lands**, and this node
destroys it.

⇒ ⭐ **`D1` captures that evidence first, while it is still capturable.**
⛔ Do not begin deleting and then attempt to prove the lane was dead — by then
the proof is unavailable and any argument for it is circular.

## 4. Deliverables

- **`D1` — Capture the spent-oracle evidence, BEFORE any deletion.** On the
  pre-deletion tree, with all five classes retired: run the full-residual
  enumeration over every measured program and the whole test corpus, and record
  that **no residual fires anywhere** and **no program selects
  `BodyEmissionAuthority::RecursiveDescent`**. ⛔ Post this before `D2`.
- **`D2` — A positive control on `D1`'s instrument.** Reintroduce one residual
  temporarily and confirm the enumeration **reports it** and the authority
  **flips to `RecursiveDescent`**. Restore byte-identically.
  ⭐ **Without this, `D1` is a negative check that passes for any reason** —
  including a broken instrument.
- **`D3` — Delete the classifiers**: `recursive_descent_residual`,
  `declaration_recursive_descent_residual`, `RecursiveDescentResidual`, and
  `select_body_emission_authority`.
- **`D4` — Delete the authority**: `BodyEmissionAuthority::RecursiveDescent`
  and, if the enum is then a single variant, the enum itself and every branch on
  it across all five files.
- **`D5` — Delete the recursive-descent emission lane** it selected.
- **`D6` — Retire or re-home the lane's tests.** ⚠ Tests that exercised the
  `RecursiveDescent` lane are testing deleted code. ⛔ Do not delete a test that
  is actually asserting a *semantic* property reachable on the surviving lane —
  re-home those. ⛔ Do not keep a test green by keeping dead code alive for it.
- **`D6b` — ANSWER THE COVERAGE QUESTION THE ADVERSARY LEFT OPEN.** Folded here
  2026-08-08 rather than filed as its own node: it is a coverage-accounting
  question, `AC-5` already forbids a silent net loss, and this node performs the
  last deletion that can change the answer.

  **The question, in the Adversary's words** (`evt_7fx8em9q24p8h`, on the merged
  `RT-PRODUCER-MATCH-PORT` `D3`): *"after this retirement, does any live row
  still exercise the ported shape, or does it now have zero live coverage in
  either direction?"*

  **It named this as unmeasured on purpose and that is the right disposition.**
  It had taken three instrument errors from rushing population measurements, and
  judged a wrong answer here worse than no answer. **Do not read the absence of
  a figure as evidence either way.**

  **What makes it live rather than academic.** That node's `D3` reverted one row
  to its original program, so the row no longer exercises the ported shape, and
  it re-homed two others that **nobody has verified**. If those three were the
  only coverage, the shape now has none — and the retirement would have removed
  the lane *and* its witnesses in two separate merges, neither of which could see
  the other.

  ⇒ **Answer it from the fixture set, in both directions:** which live rows
  exercise the producer-call-in-scrutinee shape on the surviving lane, and which
  exercise its refusal. **Zero in either direction is a finding to route, not a
  gap for you to fill here.** The Adversary states it is cheap for the ring to
  run; if it is not, say so rather than estimating.

- **`D6a` — SWEEP THE REACHABILITY-PREMISED "CANNOT OCCUR" ARGUMENTS.** Added
  2026-08-08 from a measured falsification, folded here rather than filed as its
  own node because **this node makes the largest reachability change in the
  campaign** and the sweep is worthless before it.

  **The measurement that demands it.** During `RT-PRODUCER-MATCH-PORT` `D2` an
  arm was found refusing with an argument that had become false:

  > *"a deforestable producer is by construction one whose shape was read at
  > compile time. So a carried scrutinee cannot arrive here from today's
  > corpus."*

  **`RT-SEED-CALL-PORT` `D3` falsified it** — `requires_heterogeneous_deforestation`
  classifies on the **source** shape while the callee is now lowered as a
  separately owned unit. Nobody was looking for it; the implementer hit it
  building the next node, and it was **the same implementer who had landed the
  commit that broke it.**

  ⇒ **The campaign's entire purpose is changing which lane a program takes, and
  an in-code argument premised on the old reachability goes false SILENTLY.** A
  stale "cannot occur" is not merely wrong prose — **it is the justification for
  an arm that may now be reachable and wrong.** No test reds.

  **THE 46 IS A CANDIDATE SET, NOT A BOUND — AND THE PHRASE LIST MISSES THE
  CLASS THAT WAS ACTUALLY FALSIFIED.** A grep for
  `cannot (arrive|occur|reach|happen)` / `never reaches` / `unreachable in
  practice` / `by construction` across `cranelift_backend/lowering/*.rs` and
  `planning/*.rs`, excluding tests, returned **46 hits at
  `origin/main = 1699e0a3`.** **Do not treat that as the population.** I
  selected it by phrase while scoping this deliverable by premise; those are not
  the same set, and the gap is not hypothetical.

  **Two verified counterexamples**, both production, both load-bearing
  impossibility premises, **neither matching any of the four patterns**
  (Adversary, on the `D2` merge; coordinates re-derived by me at
  `origin/main = 55d811b8`, since the reported ones had drifted):

  - `lowering/core.rs:11324` — *"planner proved impossible, and no switch could
    instantiate it."*
  - `lowering/core.rs:15447` — *"...already refused every other source, so this
    arm is **unreachable-by-validation** rather than a fallback."*

  ⇒ **The second is the falsified claim's exact structural shape:** an arm whose
  safety is **delegated to a named upstream stage's refusal.** The one that broke
  held precisely that, and broke because the upstream stage classified on the
  **source** shape while the world moved underneath it.

  **And note the trap in the wording:** *"unreachable-by-validation"* is one
  hyphen from *"unreachable in practice"* and means something different — it
  names **which stage is trusted**, which is exactly the premise that can go
  stale.

  ⇒ **Widen the selector before you enumerate.** The at-risk premise *"an
  earlier stage already refused this"* is expressible with none of the four
  phrases. A first cut — `already refused` / `refused every` / `proved
  impossible` / `validated upstream` / `guaranteed by the (planner|validator)` —
  finds **4 more in `core.rs` alone**. **Treat both lists as seeds and state the
  selector you actually ran.**

  **Neither counterexample is claimed to be false.** Their truth was not
  evaluated; they are evidence about the *enumeration's coverage*, nothing more.

  **Scope it by the premise, not by the phrase.** The at-risk class is any claim
  resting on *which values can reach a point* — carried versus compile-time
  shape, which authority a program selects, which lane an arm sits behind.
  A claim resting on a type or a structural invariant is not at risk.

  **Report the classification, not just the fixes.** For each hit: at-risk or
  not, and if at-risk, still-true or falsified. **A hit dismissed without a
  stated reason is not swept.** Re-run the grep at your own base and state its
  domain beside the result — the previous node's sweep failed by running a
  narrower domain than the claim it made.
- **`D7` — The closing measurement**: emitted function count and per-function
  code-size distribution across the measured programs, against
  `RT-DECL-CLOSURE-PORT.AC-6`'s opening figures.

## 5. Acceptance criteria

- **`AC-1`.** The whole test corpus **compiles and passes** with the lane
  deleted. Workspace green **in CI** — ⛔ never a local `--workspace` run
  (`COORDINATION §12`).
- **`AC-2`.** `D1`'s evidence and `D2`'s positive control are both in the tree.
  ⛔ `D1` without `D2` does not discharge this AC.
- **`AC-3` — the deletion is complete.** No `BodyEmissionAuthority`,
  `RecursiveDescentResidual`, or recursive-descent lane symbol survives in
  `crates/ken-runtime/src/`. ⭐ This is a **review** obligation on the QA seat
  and a compile consequence — ⛔ **not** a grep oracle committed as a test
  (operator: source-text oracles are an invitation for failure and delay).
- **`AC-4`.** `D7`'s closing figures are recorded next to
  `RT-DECL-CLOSURE-PORT.AC-6`'s opening figures. ⛔ Report; do not tune, and
  ⛔ do not pin a threshold — a size number rots at the next merge.
- **`AC-5`.** Every test removed under `D6` is accounted for: retired as
  lane-specific, or re-homed with its semantic property intact.
  ⛔ A silent net loss of coverage fails this.

  **`D6b` is inside this AC, and it extends the accounting backwards.** The
  coverage that can go silently missing is not only what *this* node removes —
  `RT-PRODUCER-MATCH-PORT` `D3` already reverted one row and re-homed two
  unverified ones. **Answering `D6b` in both directions is part of discharging
  `AC-5`**; a coverage claim that only accounts for this node's own deletions
  does not discharge it. **Zero live coverage in either direction is a finding
  to route, not a failure of this AC** — the AC fails when the question is left
  unanswered, not when the answer is unwelcome.

## 6. ⛔ Banned scope

- **Starting before all five gating nodes merge** — the four migration nodes
  and [[RT-FNUNIT-RESULT-TOKEN]]. A partial deletion is strictly worse than
  none: it removes the fallback while a class can still select it, or while a
  shape still has no other lane to run on.
- ⛔ **Keeping the lane "just in case."** That is the half-migrated state the
  directive rules out. If a case still needs it, ⛔ **stop** — the campaign is
  not finished and the missing class is a node, not a retained fallback.
- ⛔ **Deleting a test that asserts a property still reachable** on the
  surviving lane.

## 7. Hard stop

Stop and report if `D1` finds any residual still firing, if `D2`'s positive
control fails to flip the authority, or if the deletion cannot complete without
retaining a `RecursiveDescent` branch. ⭐ **Any of those means the campaign is
not done, and the honest outcome is to name the surviving class and hand it back
to the Steward as a node** — ⛔ not to delete around it.
