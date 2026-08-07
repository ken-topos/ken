# RT-SEED-CALL-PORT — retire the `SeedClosureCall` residual

**A `Call` whose callee is the retained non-lexical `Closure` form is a retained
residual, and any retained residual routes the whole object to the monolithic
`RecursiveDescent` root. This node retires that class by applying the
closure-seed → callable-unit machinery at the call site.**

**Owner:** Team Runtime. **Branch:** `wp/RT-SEED-CALL-PORT`. **Size:** M.
**Risk:** low–medium — the machinery is built by [[RT-DECL-CLOSURE-PORT]];
this is its second application point.

**The `M` predates the 2026-08-05 `D1` correction and does not include
building the enumerator.** It was set on the premise that the instrument was
inherited. Treat it as covering `D2`/`D3` only; report the `D1` cost at pickup
and the Steward re-sizes then. **Do not** silently absorb it into `M`.

**Read `docs/program/16-recursive-descent-retirement.md` first.** It carries
the campaign's binding traps and the schedule. This frame does not repeat
them.

**This node may close for free on `D1`, and that is a success, not a failure
to find work.**

---

## 1. Fixed inputs

| path | blob at `origin/main = 14c3c5f7` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `f7bc0d0354d8b8d6f7aa68176846b7b05e5a8514` |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | `f57215905ad715cab67b580781d078a614e20dfd` |

**These blobs will be stale when this node starts** — [[RT-DECL-CLOSURE-PORT]]
rewrites `core.rs` before it. **Re-pin at pickup.** They are recorded so the
*derivation* below can be checked against what changed, not so the numbers can
be trusted. **Do not** re-pin the numbers and call that a re-measurement.

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

- **`D0` — the delta-free regression baseline.** Before applying any delta,
  run the target suite on the base and record which rows are green. That set is
  what `AC-1b` holds you to. **A measurement carrying your own delta cannot
  produce it** — see the campaign doc's **Trap 2**, where this cost
  [[RT-DECL-CLOSURE-PORT]] a candidate. If `D1` closes this node for free, no
  delta is ever applied and `D0` is moot; run it only if you proceed to build.
- **`D1` — Measure before building, and be willing to stop.** Report whether
  `SeedClosureCall` still fires on any measured program.

  **The enumeration is not durable on `main`, and this frame previously said it
  was.** [[RT-DECL-CLOSURE-PORT]]'s `D1` permits "a temporary or permanent
  enumeration"; it was rebuilt at `38b05ac9` in a detached scratch worktree
  against the preserved deltas, so it never entered the candidate lineage.
  Measured at `origin/main = 8558d4e6`, the tree carries only
  `recursive_descent_residual` (`core.rs:151`), which returns
  `Option<RecursiveDescentResidual>` — the short-circuited **first** reason, and
  the exact instrument that node's `D1` ruled inadequate. `control.rs:5080` is a
  per-route classifier-accounting pin, not a program-level set.

  ⇒ **Carrying in or rebuilding the enumerator is inside this node's `D1`, not
  a precondition already met.** Size it that way, and record which you did.
  - **If it does not fire anywhere: post that, stop, and hand the node back to
    the Steward to close** — **but only after `D1a` below.** Do not build a
    port for a class with no population, and do not go looking for a real
    program that would resurrect it.
  - If it fires, `D1` names the exact programs and proceeds.
- **`D1a` — THE FREE-CLOSE GATE. An EXACT-SET positive control on the
  instrument, on THIS tree, required before any close-on-absence.** Construct or
  temporarily reintroduce a program that *does* fire `SeedClosureCall`, and show
  the enumeration reports **the exact set of variants that program fires** — not
  merely that `SeedClosureCall` appears somewhere among them. Restore
  byte-identically. **Only then** is a "fires nowhere" result admissible as a
  close.

  **A reachability control does not discharge this, and this frame asked for
  one until 2026-08-05.** [[RT-DECL-CLOSURE-PORT]] measured the difference on
  2026-08-03 at `38b05ac9`: under a short-circuit mutation the exact-set control
  went red while the all-variants-reachable control **stayed green**, because
  each witness fires exactly one variant, so short-circuiting cannot change its
  answer. That frame names this node when it says a later node re-proving only
  reachability has re-proved nothing about short-circuiting.

  **Why this node needs the gate and the others do not.** A free close is this
  node's *predicted* outcome — [[RT-DECL-CLOSURE-PORT]] is expected to subsume
  this class. ⇒ An **empty result is exactly what everyone expects to see**, so a
  broken or mis-aimed instrument produces the anticipated answer and nobody looks
  twice. That is campaign **Trap 3** at its most dangerous: the absence is not
  merely unproved, it is *welcome*.

  **And that is precisely why the shape matters: a short-circuiting
  enumerator is one such break, and a reachability gate passes while it
  short-circuits.** The gate as previously written was blind to the one defect it
  exists to catch.

  **No earlier validation transfers here.** That node rewrites `core.rs` and
  retires a class between then and now, and its instrument never landed. **Prove
  it on this tree, at the point of use.**
- **`D2` — Callee-position seed units.** A `Call` whose callee is a closure seed
  reaches a separately owned callable unit, reusing
  [[RT-DECL-CLOSURE-PORT]]'s transport rather than a parallel one.
  **If a second transport mechanism appears necessary, stop and report** —
  that is a finding about the first one's generality, and it re-cuts this node.
  **If this unit synthesizes a `Constructor`/`Record` capture environment, the
  campaign doc's Trap 5 binds that site before its first allocation** — and if it
  allocates no aggregate, record that fact rather than minting a token.
- **`D3` — Remove `SeedClosureCall`** from `RecursiveDescentResidual`, and only
  then re-run `AC-1a` and `AC-1b`.

## 4. Acceptance criteria

- **`AC-1a` — the ceiling moved.** The selector reports
  `authority=FunctionizedUnits` / `residuals=none` on every program `D1` named
  as firing this class.
- **`AC-1b` — the objects still build.** Those programs **compile and pass**
  their existing suites, **and every row green in `D0` is still green**. Not
  "the residual is gone" — the objects build. `AC-1a` does **not** discharge
  this: it quantifies over the firing set, and the regression population is its
  complement (campaign doc, Trap 2).
- **`AC-2`.** `D1`'s enumeration is recorded in the tree with the class's full
  population named, **and the enumerator itself lands in the candidate lineage,
  not only its output.** Recording the result while the instrument stays in a
  scratch worktree is what left this frame asserting a durable enumeration that
  did not exist; three later nodes in this campaign consume the same instrument,
  so a result-only record reproduces the defect at each of them.

  **If the node closes on an empty population, `D1a`'s positive control is
  recorded too.** A close-on-absence without it is not a
  measurement that the class is retired — it is a measurement that the
  instrument reported nothing.
- **`AC-3` (no-regression).** Workspace green **in CI** — **never** a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-4`.** The exhaustive-match fail-closed property at `core.rs:59-65` is
  preserved: a new `RuntimeExpr` form must still be unable to compile until the
  classifier assigns it. **No wildcard arm.**
- **`AC-5`.** Emitted function count and per-function code-size distribution
  recorded for the affected programs, as at `RT-DECL-CLOSURE-PORT.AC-6`.
  **Report; do not tune, do not pin a threshold.**

## 5. Banned scope

- **Building a second transport** parallel to `RT-DECL-CLOSURE-PORT`'s.
  Report the generality gap instead.
- **Retiring any other residual class.** Each is its own node.
- **Deleting the selector or the `RecursiveDescent` lane** — that is
  [[RT-DESCENT-RETIRE]], and it is gated on all four migrations.
- **Manufacturing a population** to justify the node if `D1` comes back empty.

## 6. Hard stop

Stop and report if `D1` shows the class fires only through shapes that also fire
a class owned by a later node, or if `D2` cannot reuse the existing transport.
Per the campaign's Trap 2, a newly reachable program shape tripping a
fail-closed invariant is **expected** here — route it as its own node; do not
absorb it and do not work around it.
