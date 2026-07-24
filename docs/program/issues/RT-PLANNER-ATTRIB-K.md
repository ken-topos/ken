---
id: RT-PLANNER-ATTRIB-K
title: "Boundary A planner: fixed K is a design invariant — move the K-exceeded rejection off the capacity channel"
status: ready
owner: runtime
size: XS
gate: none
depends_on: [RT-NATIVE-FNSPLIT]
blocks: []
github: null
origin: adversary finding J1 on RT-PLANNER-DIAGNOSTIC-K (36dd61f6), side thread thr_2seh2bm1kr5mh evt_3zcz8pz8hd8jp, 2026-07-24 — routed to the Architect as a representation fork; ruled (a) at evt_6091m3nhregch, which names the Steward as next mover. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## The Architect ruled the fork. This WP carries only the ruled half.
>
> **Fixed K is a planner design invariant, not an input capacity.** Exceeding it
> means the planner violated its own construction grammar — a compiler bug — not
> that the user wrote something unsupported. ⛔ **Do not re-open the fork.**

## The ruling, and why it is not a judgment call

`RT-PLANNER-DIAGNOSTIC-K` reclassified 43 self-consistency rejections onto
`planner_error` → `PlannerInvariant`. It left **one** site on
`planner_capacity_error` → `Unsupported` that is **the only input-reachable
rejection in the planner** (`static_transition.rs:861`, verified on `main`):

```rust
if helpers.values().copied().max().unwrap_or(0) > MAX_HELPERS_PER_STATIC_SOURCE {
    return Err(planner_capacity_error(
        "fixed K helpers per static source was exceeded",
    ));
}
```

The other six capacity sites (`:245`, `:256`, `:278`, `:317`, `:331`, `:720`)
are `u32` identity/depth exhaustion — **genuine finite representation limits,
and they stay where they are.**

The Architect settled K by following **ownership through the builder**, which is
a proof rather than another sample:

- `plan_expr` allocates one fresh `StaticSourceId` before its exhaustive,
  no-wildcard `RuntimeExpr` match (`:405-421`);
- `node` contributes one helper to the supplied owner (`:249-267`); `edge`
  contributes one to the owner of its `from` node (`:270-296`);
- every variable-length child collection — arguments, captures, fields, cases,
  recursive bodies — lowers through `plan_expr` / `plan_cases`, which allocate
  **fresh** child/test owners (`:359-402`).

⇒ **Collection size grows the number of sources, not the helper count of one
source.** The source-local budget is closed per match arm: evaluate/sequence
owners use 2; branch/case and closure-related owners at most 3; the unique
saturating `ComputationalMatch` owner constructs exactly four nodes
`{CompletedTail, ProducerTail, ProducerWrapper, SourceReturnResume}` and four
outgoing edges (`:492-509`) = **8**; terminal owners use 1. Local counts are
shape-varying; **their maximum is the input-independent structural constant
`K = 8`.**

★ **No accepted source shape legitimately exhausts K.** A ninth helper means the
planner broke its own closed construction grammar — or a later slice changed the
ruled outer topology.

## ⛔ Coverage — the ruling explicitly declines the "second shape" close

The adversary's second reading was: *if K is shape-dependent, the n=3..7 census
proves nothing, because depth is not the axis that would break it.* The
Architect's answer is that **the n=3..7 row is corroboration along depth only**
— it does not prove universal K — **but a second arbitrary shape row is still
just another sample and is not the right close.**

The proof is the exhaustive source-local construction above **plus** the
production fail-closed max check. Keep the pairwise-constant census and its real
saturating `ComputationalMatch` witness.

⛔ **Do not block this attribution fix on a second program generator.** If a
later durability WP strengthens coverage, the shape is an **exhaustive
per-construction-family budget ledger tied to the builder**, not "one more
shape."

## Deliverables

### D1 — the attribution (the whole point, ~2 lines)

`:860-863`: `planner_capacity_error(...)` → `planner_error(...)`. Nothing else
about the check moves — the threshold, the constant, and the fail-closed
position are unchanged.

### D2 — the committed test at `:1523-1591`

`planner_invariants_and_input_capacity_have_distinct_attribution` currently pins
the **exact wrong rendering** as a durable promise:

```
"unsupported runtime-IR lowering: NativeStaticTransitionPlanner: fixed K helpers
 per static source was exceeded"
```

Re-point the synthetic-ninth-helper arm at `BackendFailure::PlannerInvariant`
with the compiler-bug diagnostic, and **rename the test** — its current name
claims a distinction it will no longer draw.

### ⭐ D3 — say what the test stops proving, and decide deliberately

**This is the deliverable most likely to be skipped, so it is called out
separately.** After D1 both arms of that test land on **the same channel**, so
what was a non-degenerate pair (`PlannerInvariant` vs `Unsupported`, closed by
`assert!(!invariant.to_string().contains("unsupported"))`) becomes **two samples
of one channel**. A test whose name promises discrimination it no longer
performs is worse than one that never claimed it.

⇒ Take one of these and **state which, and why**:

- **(i)** keep a genuine non-degenerate pair by sourcing the `Unsupported` arm
  from a real unsupported construct elsewhere in the backend, so the swap-closer
  still fires; or
- **(ii)** retire the discrimination claim explicitly — rename to what it now
  tests (two distinct planner-invariant failure modes, both attributed as
  compiler bugs, with exact diagnostics) and record in the test comment that the
  capacity channel is no longer input-reachable, which is *why* no pair exists.

⛔ **Do not leave a renamed test that silently keeps the old promise in its
comment.** Either option is acceptable; an unstated collapse is not.

## Blast radius — the verdict label DOES change, and that is intended

`artifact/api.rs:753-766` and `:783-793` branch on `Unsupported` vs `Backend`,
mapping to `NativeDifferentialVerdict::Unsupported` vs `::BackendFailure`. So
after D1 a K-exceeded trip reports as **`BackendFailure`** in the differential
harness.

★ **The Architect ruled this the honest and intended mapping, not a regression**
— a planner bug should not read as an unsupported construct. In-repo consumers
are `nc7_differential_trust_report.rs` tests asserting *other* constructs'
verdicts; confirm none regressed rather than assuming.

## Acceptance criteria

- **AC-1 — the site moved, and only that site.** `:861` routes through
  `planner_error`. ⭐ Enumerate **both** channels afterward and show the six
  `u32` exhaustion sites are **still** on `planner_capacity_error`. A count is
  only as good as its window: state the window.
- **AC-2 — the new rendering is committed verbatim.** Quote the exact
  `to_string()` the test now asserts. The old `unsupported runtime-IR lowering:
  …` string must appear **nowhere** for this rejection — grep and show zero.
- **AC-3 — D3 is answered in the handoff**, naming (i) or (ii) and the reasoning.
  If (ii), the test comment states that the capacity channel is not
  input-reachable.
- **AC-4 — the census is untouched.** `fixed_k` stays in the pairwise-equal list
  (`:1485`), the `≤ MAX_HELPERS_PER_STATIC_SOURCE` assertion survives (`:1510`),
  and `MAX_HELPERS_PER_STATIC_SOURCE` is still `8`. ⛔ This WP does not change
  the cap or the coverage.
- **AC-5 — differential verdicts checked, not assumed.** Run the
  `nc7_differential_trust_report` consumers and report the result.
- **AC-6 — no behavioural change.** The set of programs that compile is
  unchanged. `scripts/ken-cargo test -p ken-runtime` green — the **full**
  `-p ken-runtime` suite, not a targeted filter: this file's siblings observe
  minted error shapes.

## ⛔ Contention — sequenced STRICTLY AFTER Boundary B1

Touches `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs`
— **the file Boundary B1 is editing right now.** The runtime ring is a
sequential token-ring and this WP has no independent branch it can safely hold.

⇒ **Do not dispatch until B1 has landed on `main`**, then cut this from the
result. The Architect's *"without stopping B1"* means do not interrupt B1 — it
is not licence to run the two concurrently.

⚠ Verify with **content**, not path overlap: after B1 lands, re-confirm `:861`
still reads `planner_capacity_error` before changing it. If B1 moved or
renumbered the check, re-anchor on the **predicate**
(`> MAX_HELPERS_PER_STATIC_SOURCE`), never on the line number.
