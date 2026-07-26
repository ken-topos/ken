# `RT-SCALE-B` — Boundary B: full emission measurement, analytical model, verdict

**Owner:** Runtime ring (empirical) + **Architect** (analytical) · **Size:** L
**Depends on:** `RT-SCALE-A`, `RT-FNSPLIT-B2F` · **Blocks:** `RT-NATIVE-FNSPLIT`

> ## ⛔ THIS NODE DECIDES WHETHER THE EFFORT WORKED
>
> `RT-NATIVE-FNSPLIT` does not merge until this returns a verdict. Every other
> node in the chain is machinery **built to be measured here**.
>
> ⭐ **The chain was recut precisely because this measurement did not exist.**
> 33 hard-stops of correct, converging work accumulated on a representation that
> provably could not reach the gate. **Every individual ruling was right; the
> thing they were accumulating into was not.**

## Objective

Measure the **completed emission path** at n = 3..7 nested resource brackets,
have the Architect produce a **research-grounded analytical model** of the growth
order, and return **one of the two verdicts the operator specified** — routing
outcome (b) to the operator rather than closing on it.

## Deliverables

### `D1` — the empirical harness, permanent and in-tree

Permanent tests (not a one-shot script) at **n = 3, 4, 5, 6, 7**, each under a
**bounded, fail-safe** harness (`prlimit`) so an over-budget run **terminates and
reports** rather than wedging the box.

> ### ⛔ WORKERS RUN ON THE PRODUCT'S STACK — 8 MiB / `ulimit -s`
>
> ⛔ **NOT** `crates/ken-cli/tests/`'s `stack_size(256 * 1024 * 1024)`. **Six
> pre-existing 256 MiB sites are already blind to stack growth**, and that is
> how a real regression hid: `B2A-C` shifted the **total** minimum stack ~128 KiB
> and reddened CI with `fatal runtime error: stack overflow` (PR #940); bisect at
> 64 KiB showed base `70bd2c74` cleared libtest's 2 MiB default by **< 64 KiB**.
> The remedy wrapped that test at 256 MiB — correct, and it means **that test can
> never detect stack growth again.**
>
> ⇒ A measurement taken on a 256 MiB stack **cannot observe the property this
> gate exists to measure.**

### `D2` — the B2 metric list, every row, for each n

- **compile wall-time** · **peak RSS**
- **distinct interned semantic states** · **defined helpers** · **emitted
  helpers**
- **CLIF instructions** · **CLIF bytes**
- **descriptor construction / comparison work**
- **total DFG / instructions / blocks**
- the same **structural counts** Boundary A reports, on the completed object

⛔ **Every metric, every n. A missing metric is a failed AC, not a footnote.**
⛔ **Do not borrow Boundary A's metric list as a substitute for this one.**

### `D3` — the differential suite

The exact **normal / abrupt / trap / join / affine** differential suite, re-run
on the completed representation.

### `D4` — the analytical model (**Architect**, research-grounded)

Predicted **order of growth vs. n**, and the specific question the operator
asked: **is ~103 s / ~4 GB at n=4 bad constants on an O(n) mechanism, or residual
super-linearity?** Super-linearity ⇒ **a further mechanism gap**, not a tuning
problem. Must be grounded in the research dispatch `evt_62fqpe7pfvym4`, not
derived solely from this node's own table.

⚠ **The model consumes `k`** — the recursive lowering frame count from
`RT-SCALE-A`'s `D5`. ⛔ **If `k` is still unmeasured, say so and treat the
stack axis as carrying no weight**, rather than substituting an inferred
per-frame figure.

### `D5` — the verdict, in the operator's two shapes

- **(a)** empirically **and** analytically **linear O(n)** — plus a **plan to
  reduce the constants**; or
- **(b)** a **research-supported** reason growth is inherently super-linear —
  plus an **explicit operator ceiling / acceptability decision.**

⛔ **Outcome (b) is NOT closable by the ring.** It routes to the **operator**
through the Steward. ⛔ **Do not close this node on a verdict only the ring has
read**, and do not soften (b) into (a) by describing super-linear growth as
"acceptable constants".

## ⛔⛔ BASELINE HONESTY — THERE IS NO BASELINE

**No apples-to-apples wall/RSS baseline exists at `b077eb7a`** — it cannot
complete even the depth-2 public control (Phase 1 returned
`could_not_determine`).

- ✅ **Report the recut's ABSOLUTE n=3..7 values.** They stand on their own.
- ⛔ **The historic n=4 `1,482 states / 1,525 edges` comparison is
  NON-COMPARABLE** unless it came from the **identical source, phase boundary,
  and metric schema.** It did not. Label it so wherever it appears.
- ★ **The operator decides the constants from the new complete measurements. The
  verdict must NOT inherit a fabricated baseline.** Writing a comparison against
  a number never produced by the same measurement is the failure this clause
  exists to prevent.

## Acceptance criteria

**`AC-B1` — fail-closed.** A run that cannot complete reports
**`could_not_determine` as a THIRD OUTCOME THAT FAILS** — never a silent pass,
never an omitted row.

**`AC-B2` — complete metric coverage.** Every `D2` metric, every n. Missing one
is a **failed AC**.

**`AC-B3` — first AND second finite differences**, not ratios. ⛔ **A single
ratio, or a fitted curve alone, discharges nothing.**

**`AC-B4` — no exponent claimed from few points.** `370n`, `93n²` and a product
switching on at n=5 **all pass through the historic n=4 datum**. The
**structural invariants discriminate**; the table corroborates. State that
sentence in the verdict.

**`AC-B5` — the differential suite is green on the completed representation**,
with its baseline recipe **in the tree**: base SHA, probe function names, and the
`git worktree add --detach <sha>` + test invocation.

> ⭐ **Why the recipe is required, not the results:** the asserted property is
> **equality against committed constants**, so a post-change re-capture produces
> byte-identical values — **no observation distinguishes a genuine pre-change
> baseline from a re-recorded one.** Demonstrate the binding; do not testify to
> it.

**`AC-B6` — the analytical model is present, research-grounded, and answers the
operator's binary** (bad constants on O(n) vs. residual super-linearity). ⛔ A
model derived only from this node's own table does not discharge it.

**`AC-B7` — the verdict is (a) or (b) explicitly**, and **(b) is routed to the
operator, not closed.**

**`AC-B8` — no fabricated baseline.** The absolute values stand alone, and any
appearance of the historic n=4 figure is labelled **NON-COMPARABLE**.

**`AC-B9` — the `AC` → control map.** One row per `AC` in this frame, naming the
control and the evidence; an `AC` with no control is recorded
**`NO CONTROL — open residual`**, in that spelling.

> ⭐ **This chain's own evidence for why that row must exist:** on
> `RT-FNSPLIT-B2V`, `fd4e7f08`'s map was **complete, honest, mutation-proved**
> (`NODE_LIMB_COUNT` → `NODE_FIELD_COUNT` reddened exactly at limb count) and
> `ken-runtime` was **398/0** — and **three production defects sat outside it**,
> because no `AC` asked the layout-closure question, so no row was missing.
> ⇒ **A green `AC`→control map is coverage of the questions the `AC` set knows
> how to ask, and nothing wider.**

## ⛔ Do-not-reopen guardrails

1. ⛔ **Do not re-ask the hold-falsified / hold-confirmed question.** It is
   **CLOSED**: Phase 1 returned `could_not_determine`, and the hold rests on
   **code inspection rejecting an O(n) proof, not on curve-fitting.**
2. ⛔ **Neither boundary may stand in for the other**, and **a post-failure
   prefix cannot substitute for any boundary.**
3. ⛔ **`B2F`'s `AC-G0` does not discharge any part of this node.** That is a
   Θ(1)-per-module helper-count invariant (6 definitions / 8 declarations),
   already answered — **not** the n=3..7 empirical table.
4. ⛔ **Interning is necessary, not sufficient.** It shares equal subterms; it
   **cannot** merge two distinct tuples merely because their components overlap.
   **Calling the vectors "interned" does not reduce the product-state count.**

## Standing

- ⛔ **Local builds/tests are TARGETED ONLY** — `scripts/ken-cargo -p
  ken-runtime`, or `--test <name>`. **Never `--workspace`** (`COORDINATION §12`,
  operator hard rule). Workspace-green and `--locked` mean **green in CI**.
  ⚠ **This node measures compile cost deliberately** — that is exactly why it
  runs under `prlimit` and only ever on scoped targets.
- ⛔ **Never `git stash`** — `refs/stash` is shared across ~70 worktrees.
- Read `agent/playbooks/tools/pin-a-property.md` before writing any assertion.
- ⚠ **Report the measurement the artifact states, and do not generalize past
  it.** `AC-B3`, `AC-B4` and `AC-B8` each guard one way that has already gone
  wrong on this chain.
