# EFF-SPACE-ENSURES-PRESTATE — `old` must stop meaning nothing

**Node:** [`EFF-SPACE-ENSURES-PRESTATE`](../issues/EFF-SPACE-ENSURES-PRESTATE.md)
· **Owner:** Language · **Size:** M · **Gate:** none

**Fixed inputs, measured at `origin/main = 0031dd6a`. ⛔ Re-derive at point of
use — these are current-state claims and they perish.**

| input | pin |
|---|---|
| the defect | `crates/ken-elaborator/src/elab.rs` blob **`7029765b`** — `RExpr::ROld(inner, span) => check(cx, inner, expected, span)` at **`:614`**, and `RExpr::ROld(e, _) => infer(cx, e)` at **`:2199`** |
| the contract | `spec/30-surface/36-effects.md §4.3` — bare `cᵢ` is post-state, `old(cᵢ)` is pre-state, "well-defined because the denotation *names* the pre-state" |
| the surface that already works | `parser.rs:2184` blob `e3fc6620` (`EOld`); `resolve.rs:1473` blob `f05c7535` (gated to `PropCtx::SpaceOpEnsures`) |
| ⛔ the false green | `tests/effects.rs` blob **`373e7cb2`**, `space_old_scoped_to_ensures_type_level` — asserts pre/post structure in its docstring, constructs no `ensures` and no `old` |
| the obligation path | `elab.rs:5434–5452` collects `ensures`; `extract.rs:118` lifts to `ObligationTriple` `<def>.ensures.<n>` |

## 1. The problem in one line

`old(n)` and `n` elaborate to the **same core term**, so `ensures n == old(n) +
1` — §4.3's own worked example — becomes the obligation `n == n + 1`.

⭐ **The severity is not "a feature is missing." It is that a wrong obligation is
indistinguishable from a real one.** `old` parses, passes its scope gate, and
produces an `ObligationTriple` with a plausible id. Every surface signal says the
contract was recorded. Nothing downstream can tell that the predicate lost its
meaning between resolve and elaborate.

## 2. Deliverable

**Make `old` mean the pre-state, or make it fail.** ⛔ **What it may not do is
stay transparent.**

Two acceptable end states. **Determine which is reachable, and report before
building** — this is the one judgment I am not front-loading, because it depends
on whether a pre-state binding can be produced at this stage:

**Shape A — the spec's model (preferred).** A space operation's `ensures`
elaborates against its state-transformer denotation `S → R × S` (§4.2). `old(e)`
resolves cell references against `s_pre`; a bare cell resolves against `s_post`.

**Shape B — fail closed (acceptable fallback).** If the pre-state binding
genuinely requires the §4.1 space-block machinery this node **excludes**, then
`old` must raise a hard, specific elaboration error — *`old` is not yet
supported* — instead of silently elaborating to the post-state.

⭐ **Why B is acceptable and the status quo is not:** B leaves the feature
unavailable and *says so*. The status quo leaves it available and wrong. A
visible unsupported-feature error is strictly better than a silently false
obligation, and it is honest about the boundary (`docs/PRINCIPLES.md`).

⚠ **If you find a third shape, report it.** The frame's mechanism claim is a
hypothesis until someone implements against it.

## 3. Acceptance criteria

| AC | claim | control |
|---|---|---|
| `AC-E1` | ⭐⭐ **`old(e)` and `e` no longer produce the same core term** in a space-op `ensures` (Shape A), **or** `old` is rejected with a specific error (Shape B). | ⛔ **The control must compare the two elaborations, not assert one of them.** Elaborate `ensures n == old(n) + 1` and `ensures n == n + 1` and show the emitted obligations **differ** (A), or that the first is rejected and the second is not (B). ⚠ A test that only checks the `old` case passes under transparency |
| `AC-E2` | ⭐⭐ **The `36 §4.3` worked example behaves as the spec says.** `inc`'s `ensures n == old(n) + 1` yields the obligation §4.3 states, and it discharges. | Under Shape B this AC is **explicitly not discharged** — say so, and say which error fires. ⛔ Do not mark it green by reinterpreting it |
| `AC-E3` | ⭐⭐ **The false green is dead.** | ⛔ `space_old_scoped_to_ensures_type_level` must either construct a real `ensures` containing `old`, or **lose the `old` claim from its docstring**. A test whose name and comment assert a property its body cannot observe is worse than no test — it reads as coverage. **Say which you did.** |
| `AC-E4` | **The scope gate still holds.** `old` outside `PropCtx::SpaceOpEnsures` is still `UnboundName`. | This is existing behavior at `resolve.rs:1473`; the control is that a `old` in a *pure* view's `ensures` is still rejected after your change. ⚠ Shape A touches this path — a widened context is the plausible regression |
| `AC-E5` | No regression. | Targeted only: `scripts/ken-cargo test -p ken-elaborator`. Workspace / `--locked` / conformance run **in CI** |
| `AC-E6` | Trusted-base delta is **zero**. | The elaborator is **not** in the TCB (`36 §7`); the kernel re-checks the emitted term. No `trusted_base()` change, no new primitive. ⭐ If your fix appears to need one, the premise has failed — stop and report |

## 4. Scope

**IN:** `old`'s elaboration; the space-op `ensures` context if Shape A; the test
repair in `AC-E3`.

⛔ **OUT:**
- ⛔ **`becomes` and the `space` cell-block surface.** Zero hits for
  `becomes`/`KwBecomes` in `lexer.rs`/`parser.rs`; `space` parses only as `space
  proc` (`parser.rs:355`). That is a separate, larger node. **If Shape A turns
  out to require it, that is Shape B's trigger — report it, do not build it.**
- ⛔ **The rest of `effects/`.** The row lattice, inference, `ITree`,
  capabilities, `run_state`, and tail-resumptive handlers are **built and live**
  — 2614 lines consumed by nine production modules. Do not revisit them.
- ⛔ **`conformance/surface/effects/`.** It holds only `seed-effects.md`; filling
  it is not this WP.

## 5. Contention check

**Measured at `0031dd6a`.** No open WP branch names `elab.rs`, `resolve.rs`, or
`tests/effects.rs`. Language's last node (`STR-NFC-CONSTRUCTION`) is merged and
the ring is closed.

⚠ `elab.rs` is a large shared file. ⭐ **The licence is that this WP changes two
match arms on one `RExpr` variant** — not that no other node ever edits `elab.rs`.
If a candidate finds itself restructuring the ensures pipeline broadly, the
premise has failed and it comes back to me.

## 6. Validation — ⛔ TARGETED ONLY

⛔ **NEVER `--workspace`** (operator, `COORDINATION.md §12`). `scripts/ken-cargo
test -p ken-elaborator`. Workspace, `--locked` and conformance run **in CI**.

⚠ `ken-cargo` is a machine-wide `flock` with one slot and several rings are live
on it — coordinate the turn **in-thread**; ⛔ never sample `ps` to decide it is
free.

## 7. Reporting

Return exact SHA/tree/base, and specifically: **which shape you built and why the
other was not reachable**; **the `AC-E1` comparison** (the two obligations, or
the rejection); whether `AC-E2` is discharged or explicitly not; **what you did to
the false-green test and what it would now catch**; and whether the scope gate
regressed.
