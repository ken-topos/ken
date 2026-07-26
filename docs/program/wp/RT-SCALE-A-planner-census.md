# `RT-SCALE-A` — Boundary A: the planner census for n = 3..7

**Owner:** Runtime ring · **Size:** M · **Depends on:** `RT-FNSPLIT-B2F`
**Blocks:** `RT-SCALE-B` — and through it, `RT-NATIVE-FNSPLIT`'s merge.

> ## ⛔ THIS IS ONE HALF OF A MERGE CONDITION, NOT A REPORT
>
> `RT-NATIVE-FNSPLIT` **does not merge** without both boundaries. The operator
> ruled 2026-07-23 (`evt_4btfhwqhah1ye`) that *"SP tests complete under a
> timeout" is NOT acceptance* — a 4-bracket program costing ~103 CPU-s / ~4 GB
> to compile is unacceptable **without understanding the scaling law**.
>
> ⛔ **Neither boundary may stand in for the other**, and ⛔ **a post-failure
> prefix cannot substitute for any boundary.**

## Objective

Build the **new factored static transition graph** at **n = 3, 4, 5, 6, 7**
nested resource brackets and report the Boundary A metric list — **on the
completed factored representation**, superseding the provisional outer-planner
census landed at `647a2e5b`.

## ⛔⛔ THE PROVISIONAL CENSUS IS NOT A BASELINE — READ THIS BEFORE MEASURING

A Boundary A census already landed at **`647a2e5b`**. The recut frame states it
is **true only for the OUTER planner** and **PROVISIONAL for the completed
representation**.

⛔ **Do not cite `87 / 115 / 143 / 171 / 199`, `K = 8`, or widths `12 / 32 / 16`
as settled.** ⛔ **Do not open by trying to reproduce them.** Measure the
completed object; then, separately, report whether the new numbers agree with
the provisional ones and say plainly what changed. ⚠ **Agreement is not
confirmation** — if the completed representation reproduces the outer-planner
census exactly, that is a *finding to explain*, not a validation, because the
provisional run could not see the inner planner at all.

> ## ⛔⛔ RE-CUT 2026-07-26 — THIS NODE NO LONGER WAITS ON `B2F`. RUN IT NOW.
>
> **`depends_on` was `[RT-FNSPLIT-B2F]`. It is now `[RT-FNSPLIT-B2A-S]`, which is
> MERGED.** The Steward wrote the old dependency and it was wrong.
>
> ⭐ **Why this is the operator's call and not a tidy-up.** The effort's goal is a
> **growth property**. `B2F` sits behind `B2V`, which has absorbed **six
> Architect production blocks**. So the old edge sequenced **the earliest
> available growth signal behind the two hardest remaining nodes** — and across
> **43 hard-stops on two chains there is still not one measured number describing
> growth in `n`.**
>
> ⛔ **That is the exact structural flaw the recut was supposed to fix.** The
> recut exists because *"33 hard-stops of correct, converging work accumulated on
> a representation that provably could not reach the gate."* The recut repaired
> the **representation**; it did **not** repair *measurement-comes-last*. This
> edge is that repair.
>
> **What makes it runnable today:** this node censuses the **planner** — static
> nodes, edges, planned helpers, `K`, and the helper-**identity** invariants. The
> planner landed at `B2A-C`/`B2A-S`, both merged, and `647a2e5b` already produced
> a provisional census — ⇒ **runnability is demonstrated, not assumed.**
>
> ### ⛔ DEFERRED ROWS — report them, never omit them
>
> Some rows genuinely cannot be measured until `B2V` defines the value word and
> `B2F` emits. ⛔ **Report each as `DEFERRED — NEEDS B2V/B2F`, in that exact
> spelling.** A deferred row is **not** `could_not_determine` (a *failure*) and
> is **not** a silent omission — those two already read identically in a green
> verdict, which is the failure `AC-A1` exists to prevent, and a third state that
> is neither must be spelled or it will be read as one of them.
>
> | row | why deferred |
> |---|---|
> | constant **node payload width** (`D3` invariant 2, width half) | the payload is `B2V`'s value word |
> | **frame / store-node schema** width (`D2`) | same |
>
> ⛔ **Everything else is due now** — including `D3` invariants **1, 3 and 4**,
> the **ID** half of invariant 2, `D4`, and `D5`'s `k`.
>
> ⚠ **THE DEFERRAL BOUNDARY IS A QUESTION FOR THE ARCHITECT, NOT A RULING.** The
> Steward drew it from what the planner produces. ⭐ **If the Architect moves it,
> it can only move rows OUT of deferral and INTO this node's scope** — so the
> boundary being wrong delays nothing. ⛔ **Do not use an unresolved boundary as
> a reason to wait; report what you measured and name what you deferred.**

## Deliverables

> ### ⛔ CONTENTION CONSTRAINT — `B2V` IS IN FLIGHT IN THE SAME CRATE
>
> `RT-FNSPLIT-B2V` is **active** and touching ten files in
> `crates/ken-runtime/src/`, **including `lib.rs` and
> `cranelift_backend/lowering/mod.rs`**.
>
> ### ⛔ AMENDED 2026-07-26 — MY INTEGRATION-TEST CONSTRAINT WAS UNSATISFIABLE
>
> **The original text demanded a new integration-test file under
> `crates/ken-runtime/tests/`, no production visibility change, and no private
> duplication. Those three cannot all hold** — @runtime-leader measured it at
> the current tree (`evt_4xcrmeayx1xvp`) and was **right to refuse to build
> around it**.
>
> `plan_static_transition_graph`, `StaticTransitionPlan`, `BoundaryACensus` and
> `StaticTransitionPlan::census()` are **all confined to
> `crate::cranelift_backend`**, and production reaches planning only internally
> via `lowering/core`. ⇒ **An auto-discovered integration test cannot build the
> plan or observe `k` at all.**
>
> ⭐ **My reasoning was over-generalized, and this is the correction:** I wrote
> *"a `#[cfg(test)]` module inside `src/` requires a parent-module edit."* That
> is true for **adding a new module** and **false for extending an existing
> suite**. It is also the `cfg(test)` asymmetry I warned about in the same
> breath — **the warning was right and I applied it in the wrong direction.**
>
> ### ✅ WHERE THE HARNESS GOES
>
> ⛔ **Extend the EXISTING `#[cfg(test)]` suite in
> `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs`** —
> where the planner is legitimately test-visible and **where the current n=3..7
> census already lives.**
>
> **Contention with `B2V` is still empty, and still by construction — verified,
> not assumed:** `static_transition.rs` is **not in `B2V`'s ten-file set**
> (`git diff --name-only aecdb001 9b254fb9`). Extending a suite that already
> exists needs **no parent-module edit**.
>
> ⛔ **DO NOT widen production visibility.** No `pub` export of the planner is
> authorized, and Verify was right to hold rather than widen unilaterally. If
> you conclude a production seam is genuinely required, ⛔ **stop and route it**
> — it would touch files `B2V` is rewriting, which is a sequencing decision, not
> an implementation detail.
>
> ### ✅ RULED — `evt_1dgbawkjfs507`, 2026-07-26. BINDING HARNESS SHAPE.
>
> **`D1` REMAINS AN 8 MiB PRODUCT-STACK MEASUREMENT.** ⛔ It is **not** amended
> down to libtest's ~2 MiB default — **pin the planner worker explicitly
> instead.** The Architect verified this amended frame at exact `368f8706` and
> the production tree at `4427147d`.
>
> ⭐ **Why no public seam:** `StaticTransitionPlan` and
> `plan_static_transition_graph` are `pub(in crate::cranelift_backend)`;
> `BoundaryACensus` and `census()` are test-only; and the existing n=3..7 fixture
> **already invokes the exact production planner** inside `static_transition.rs`.
> ⛔ **A public seam would create a production contract whose only consumer is a
> test — unnecessary surface and the wrong architectural trade.**
>
> 1. Harness stays in `static_transition.rs` and **must call the real
>    `plan_static_transition_graph` and `StaticTransitionPlan::census()`.** ⛔ No
>    copy, no `include!`, no proxy planner, no visibility widening, no new
>    `pub`/`pub(crate)` census API.
> 2. Run the census body on a **deliberately created worker whose requested stack
>    is exactly `8 * 1024 * 1024` bytes.** **Name that nominal stack in every
>    census and verdict.** ⛔ The small libtest dispatch thread must perform **no
>    recursive planning work.**
> 3. ⛔⛔ **PROCESS ISOLATION IS REQUIRED — an in-process `join` ALONE DOES NOT
>    DISCHARGE `AC-A1`.** A Rust worker stack overflow may **abort its process
>    rather than return through `join`**, so the failure you most need to catch
>    is the one `join` cannot report. Use the permanent test as a **supervisor
>    over an isolated worker invocation** (or an equivalently causal mechanism)
>    so that **timeout, signal, nonzero exit, missing/malformed result, and stack
>    failure ALL become the exact third outcome `could_not_determine` and FAIL
>    the test.** The isolated worker then creates the exact 8 MiB planner thread.
> 4. ⛔ **Do not infer product failure from the incidental ~2 MiB libtest stack,
>    and do not use 256 MiB.** The metric is **the explicitly pinned 8 MiB
>    worker**.
> 5. `D2`–`D5`, `AC-A1`–`AC-A8`, the exact deferred spelling, and the
>    `RT-SCALE-B` deferral boundary are **unchanged**.
>
> ⭐ **The framing that dissolves the conflict:** *permanence means committed and
> automatically run — not "must be an integration-test crate."* My original
> constraint confused the two.

### `D1` — the harness, permanent and in-tree

A **permanent test** (not a one-shot script) that constructs minimal Ken programs
at n = 3..7 nested resource brackets and emits the census below. It runs under a
**bounded, fail-safe harness** (`prlimit`), so an over-budget run **terminates
and reports**, never hangs the box.

> ### ⛔ THE HARNESS RUNS ON THE PRODUCT'S STACK — 8 MiB / `ulimit -s`
>
> ⛔ **NOT** the `crates/ken-cli/tests/` convention of
> `stack_size(256 * 1024 * 1024)`. **Six pre-existing 256 MiB sites are already
> blind to stack growth** — that convention is exactly why a real regression hid.
>
> **Measured, and this is why the rule is here:** `B2A-C`'s correspondence
> threading shifted the **total** minimum stack by ~128 KiB and turned CI red on
> `ken-cli::px8ta_oriented_subcontinuation
> public_two_three_level_brackets_finish_and_release_lifo` with
> `fatal runtime error: stack overflow` (PR #940). Bisected at 64 KiB:
> base `70bd2c74` cleared libtest's 2 MiB default by **< 64 KiB**; candidate
> `08633b3c` did not fit. The remedy wrapped that one test at 256 MiB —
> in-scope and correct, and it means **that test can never detect stack growth
> again.**
>
> ⇒ A census that runs on a 256 MiB stack **cannot observe the property this
> gate exists to measure.**

### `D2` — the census, every row, for each n

- **static nodes** · **edges** · **planned helpers**
- **persistent-store nodes** · **out-of-line evidence records**
- **fixed `K`** per static source/control node — **name the number**
- **fixed key schemas** · **fixed frame / store-node schemas**

⛔ **`AC1.2′`: every metric in *this* list, for every n. A missing metric is a
failed AC, not a footnote.** ⛔ **Do not borrow Boundary B's metrics into A.**

### `D3` — the four structural invariants, by construction where possible

These are what actually discharge the gate; the table corroborates them.

1. **No flattened env / pending / path member in helper identity.**
2. **Constant ID / node payload width.**
3. **Affine total persistent nodes.**
4. **At most affine logical chain depth.**

> ### ⛔ THE WIDTH METRIC WAS WRONG ONCE — DO NOT RESTORE IT
>
> The original Phase-1 list demanded constant maxima for *"env / pending / path
> lengths."* **That is the wrong invariant and would have rejected a CORRECT
> design.** The constant invariants govern maximum **inline identity / frame /
> store-node width**. **Logical persistent-chain DEPTH may grow Θ(n), and that is
> SOUND** — the helper/frame carries **one constant-width ID** into the
> persistent store rather than the chain itself.
>
> ⛔ **Do NOT require the logical chain length to be constant**, and do not
> report it as a violation when it grows.

### `D4` — `#34` present from the first accepted planner shape

The **source-return-owned resume transition** is represented **explicitly**, not
retrofitted. `Terminal` stays **un-overloaded** — it means *no continuation*,
and this state has a live continuation owned indirectly by an exact
source-return descriptor. Option 2's duplicate direct `W` remains **rejected**.

### `D5` — measure `k`, the recursive lowering frame count

⛔ **`k` is currently UNKNOWN, and the analytical model in [[RT-SCALE-B]] cannot
be built without it.**

> ### ⚠ A TOTAL WAS ONCE LAUNDERED INTO A PER-FRAME FIGURE. DO NOT REPEAT IT.
>
> The Steward originally wrote that threading added *"~128 KiB per recursive
> lowering frame."* **That is not what was measured** (adversary N2,
> `evt_7mve56d192pv6`). The bisect measured a ~128 KiB shift in the **TOTAL**
> minimum stack across an **unknown number of frames `k`**. Per-frame growth is
> ≈ `128/k` KiB.
>
> ⛔ **And it does not "err safe."** Extrapolating at 128 KiB/frame is
> pessimistic only if `k > 1`, which is itself a claim about an unmeasured
> quantity. ⇒ **The honest statement is that per-frame growth is UNKNOWN.**
>
> ⚠ The recursion is over the **expression tree**, not the bracket depth — so
> `k` is **not** 2, and **not** 3, and **not** n. Report `k` per n as measured
> data, with the method that produced it.

## Acceptance criteria

**`AC-A1` — fail-closed, verbatim from `AC1.1′`.** A run that cannot complete
reports **`could_not_determine` as a THIRD OUTCOME THAT FAILS** — never a silent
pass, never an omitted row.

**`AC-A2` — complete metric coverage (`AC1.2′`).** Every `D2` metric, every n.
Missing one is a **failed AC**.

**`AC-A3` — first AND second finite differences (`AC1.3′`).** Report the
differences, **not ratios**. ⛔ **A single ratio, or a fitted curve alone,
discharges nothing.**

**`AC-A4` — the four `D3` invariants are demonstrated BY CONSTRUCTION where
possible.** Fixed-arity key types, no `Vec` / `String` / path members in
identity; grep-able and asserted. ⭐ **Prefer the compiler over a test:** if the
type makes a variable-width member unrepresentable in helper identity, that is a
stronger discharge than any assertion. Name `K` and assert it.

**`AC-A5` — no exponent claimed from few points (`AC1.5′`).** ⛔ `370n`, `93n²`
and a product switching on at n=5 **all pass through the historic n=4 datum.**
The **structural invariants discriminate**; the table corroborates. State that
sentence in the verdict.

**`AC-A6` — `k` is reported per n, with its method**, or the node states
plainly that it could not be measured and why. ⛔ **A derived per-frame figure
inferred from a total is not a measurement** and fails this AC.

**`AC-A7` — the provisional census is addressed explicitly.** State how the new
numbers relate to `647a2e5b`'s `87/115/143/171/199` / `K=8` / `12/32/16`, and
⛔ **do not present agreement as confirmation.**

**`AC-A8` — the `AC` → control map.** One row per `AC` in this frame, naming the
control that discharges it and the evidence. An `AC` with no control is recorded
**`NO CONTROL — open residual`**, in that spelling.

> ⭐ **Why that spelling is mandatory, from this chain's own history:** an `AC`
> with **zero** controls is invisible to a review that examines controls —
> *discharged* and *never asked* read identically in a green verdict. And on
> `RT-FNSPLIT-B2V`, three honestly-recorded residuals turned out to be **the
> predicate's uncovered faces**, which is what triggered the whole recut. **A
> standing residual is a debt, not a disposition.**

## ⛔ This node is a GENUINE STOP — `AC1.4′`

Report and **HOLD for a Steward + Architect read** before anything downstream
consumes the numbers. ⛔ **Do not roll straight into [[RT-SCALE-B]].**

⚠ The binary *hold-falsified / hold-confirmed* question is **CLOSED** — Phase 1
returned `could_not_determine`, and the hold rests on **code inspection
rejecting an O(n) proof, not on curve-fitting.** ⛔ **Do not re-ask it**, and do
not treat this census as re-litigating it.

## Standing

- ⛔ **Local builds/tests are TARGETED ONLY** — `scripts/ken-cargo -p
  ken-runtime`, or `--test <name>`. **Never `--workspace`** (`COORDINATION §12`,
  operator hard rule). Workspace-green and `--locked` mean **green in CI**.
- ⛔ **Never `git stash`** — `refs/stash` is shared across ~70 worktrees.
- Read `agent/playbooks/tools/pin-a-property.md` before writing any assertion.
- ⚠ **Report the measurement the artifact states.** Do not generalize past it —
  that is the specific failure `AC-A5`, `AC-A6` and `AC-A7` each guard.
