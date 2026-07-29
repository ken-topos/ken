# RT-SEED-CALL-PORT — retire the `SeedClosureCall` residual

**A `Call` whose callee is the retained non-lexical `Closure` form is a retained
residual, and any retained residual routes the whole object to the monolithic
`RecursiveDescent` root. This node retires that class by applying the
closure-seed → callable-unit machinery at the call site.**

**Owner:** Team Runtime. **Branch:** `wp/RT-SEED-CALL-PORT`. **Size:** M.
**Risk:** low–medium — the machinery is built by [[RT-DECL-CLOSURE-PORT]];
this is its second application point.

⛔ **Read `docs/program/16-recursive-descent-retirement.md` first.** It carries
the campaign's two binding traps and the schedule. This frame does not repeat
them.

⭐ **This node may close for free on `D1`, and that is a success, not a failure
to find work.**

---

## 1. Fixed inputs

| path | blob at `origin/main = 14c3c5f7` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `f7bc0d0354d8b8d6f7aa68176846b7b05e5a8514` |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | `f57215905ad715cab67b580781d078a614e20dfd` |

⚠ **These blobs will be stale when this node starts** — [[RT-DECL-CLOSURE-PORT]]
rewrites `core.rs` before it. **Re-pin at pickup.** They are recorded so the
*derivation* below can be checked against what changed, not so the numbers can
be trusted. ⛔ Do not re-pin the numbers and call that a re-measurement.

## 2. The mechanism, at exact anchors

`core.rs:118-124`, inside the `RuntimeExpr::Call` arm:

```rust
RuntimeExpr::Call { callee, args } =>
    matches!(callee.as_ref(), RuntimeExpr::Closure { .. })
        .then_some(RecursiveDescentResidual::SeedClosureCall)
```

The callee is a **non-lexical `Closure`** — a closure seed in *call* position.
[[RT-DECL-CLOSURE-PORT]] retires the same seed form in *transparent declaration
body* position and, doing so, builds:

- planner-owned callable units for closure seeds,
- typed capture / parameter / result / trap transport across that boundary,
- `DeclarationRef` calls to those units.

⇒ **This node is that machinery applied one position over.** If it is genuinely
general, little or nothing remains here.

## 3. Deliverables

- **`D1` — Measure before building, and be willing to stop.** Using the
  full-residual enumeration built by [[RT-DECL-CLOSURE-PORT]]'s `D1`, report
  whether `SeedClosureCall` still fires on any measured program.
  - ⭐ **If it does not fire anywhere: post that, stop, and hand the node back to
    the Steward to close.** ⛔ Do not build a port for a class with no
    population, and ⛔ do not go looking for a program that would resurrect it.
  - If it fires, `D1` names the exact programs and proceeds.
- **`D2` — Callee-position seed units.** A `Call` whose callee is a closure seed
  reaches a separately owned callable unit, reusing
  [[RT-DECL-CLOSURE-PORT]]'s transport rather than a parallel one.
  ⛔ **If a second transport mechanism appears necessary, stop and report** —
  that is a finding about the first one's generality, and it re-cuts this node.
- **`D3` — Remove `SeedClosureCall`** from `RecursiveDescentResidual`, and only
  then re-run `AC-1`.

## 4. Acceptance criteria

- **`AC-1`.** Every program `D1` named as firing this class **compiles and
  passes** its existing suite. ⛔ Not "the residual is gone" — the objects build.
- **`AC-2`.** `D1`'s enumeration is recorded in the tree with the class's full
  population named.
- **`AC-3` (no-regression).** Workspace green **in CI** — ⛔ never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-4`.** The exhaustive-match fail-closed property at `core.rs:59-65` is
  preserved: a new `RuntimeExpr` form must still be unable to compile until the
  classifier assigns it. ⛔ No wildcard arm.
- **`AC-5`.** Emitted function count and per-function code-size distribution
  recorded for the affected programs, as at `RT-DECL-CLOSURE-PORT.AC-6`.
  ⛔ Report; do not tune, do not pin a threshold.

## 5. ⛔ Banned scope

- ⛔ **Building a second transport** parallel to `RT-DECL-CLOSURE-PORT`'s.
  Report the generality gap instead.
- ⛔ **Retiring any other residual class.** Each is its own node.
- ⛔ **Deleting the selector or the `RecursiveDescent` lane** — that is
  [[RT-DESCENT-RETIRE]], and it is gated on all four migrations.
- ⛔ **Manufacturing a population** to justify the node if `D1` comes back empty.

## 6. Hard stop

Stop and report if `D1` shows the class fires only through shapes that also fire
a class owned by a later node, or if `D2` cannot reuse the existing transport.
⚠ Per the campaign's Trap 2, a newly reachable program shape tripping a
fail-closed invariant is **expected** here — route it as its own node; ⛔ do not
absorb it and ⛔ do not work around it.
