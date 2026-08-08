# RT-SEED-CALL-PORT — retire the `SeedClosureCall` residual

**A `Call` whose callee is the retained non-lexical `Closure` form is a retained
residual, and any retained residual routes the whole object to the monolithic
`RecursiveDescent` root. This node retires that class by applying the
closure-seed → callable-unit machinery at the call site.**

**Owner:** Team Runtime. **Branch:** `wp/RT-SEED-CALL-PORT`. **Size:** M.
**Risk:** low–medium — the machinery is built by [[RT-DECL-CLOSURE-PORT]];
this is its second application point.

**The `M` stands, and the re-size announced on 2026-08-05 is WITHDRAWN
(2026-08-08).** That note told the ring `D1` included building the enumerator
and to report its cost for a re-size. That premise was false: the instrument
is on `main` — see `D1`. `D1` is cheap, `M` now over-covers rather than
under-covers, and there is no build cost to report.

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

  **THE ENUMERATOR IS DURABLE ON `main`. DO NOT BUILD ONE.** Measured at
  `origin/main = 606efa93` (2026-08-08):

  - `enumerate_recursive_descent_residuals` — `core.rs:598`, the entry point,
    over the expression plus the declaration map.
  - `collect_recursive_descent_residuals` — `core.rs:616`, the
    non-short-circuiting twin: accumulates into a `BTreeSet`, visits every child
    regardless of what a sibling produced, exhaustive `match` with no wildcard
    arm. It classifies `SeedClosureCall` at the `Call` arm, `core.rs:707`.

  It landed in `7ca5cfc0` ([[RT-SRCBODY-BIND-ORDER]]), and its own doc comment
  names this node as a reuser.

  **This frame asserted the opposite until 2026-08-08, and the error is worth
  naming because it is the reusable one.** [[RT-DECL-CLOSURE-PORT]]'s
  enumerator genuinely never entered the candidate lineage — true when that node
  closed, and it stayed in this frame as an inherited premise. A *later,
  different* node landed a durable one. **A fact about the tree decays; re-derive
  it against current `main` at each use rather than carrying it forward.**

  ⇒ **`D1` is the corpus measurement only.** The instrument is a precondition
  already met.
  - **If it does not fire anywhere: post that, stop, and hand the node back to
    the Steward to close** — **but only after `D1a` below.** Do not build a
    port for a class with no population, and do not go looking for a real
    program that would resurrect it.
  - If it fires, `D1` names the exact programs and proceeds.
- **`D1a` — THE FREE-CLOSE GATE. An EXACT-SET positive control on the
  instrument, on THIS tree, required before any close-on-absence.**

  **THIS CONTROL IS ALREADY COMMITTED. Do not construct a temporary program and
  do not plan a byte-identical restore.**
  `d1_the_enumerator_reports_every_variant_not_the_first`
  (`core/tests/control.rs:10849`) is `#[test]` and not `#[ignore]`d. It builds a
  compound firing four variants and asserts `assert_eq!` over a `BTreeSet` — an
  exact-set assertion, not membership — with `SeedClosureCall` among them and its
  firing witness at `control.rs:10723`. It then contrasts the short-circuiting
  selector's one-reason answer on the same program.

  **What remains is to RUN it by name on this tree**, per the instrument's own
  doc: re-prove cheaply at each point of use, because later deliverables rewrite
  that file underneath it. **Presence is not greenness.** Only then is a "fires
  nowhere" result admissible as a close.

  **The committed witness is not a population.**
  `d1_seed_closure_call_witness()` is a hand-built `RuntimeExpr` in a test, not
  a Ken program. It shows the enumerator can *see* the class; it says nothing
  about whether the class *has* a live population. Report the instrument's
  provability and the corpus's population as two separate findings.

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
- **`D2` — Callee-position seed units. THE MECHANISM IS RULED; BUILD TO IT.**
  Architect ruling `evt_7p8dmg1rez02c`, 2026-08-08, after `D1` established a
  fixture-only population. **`D2` is a PORT of the published capability — not a
  deletion, not a deferral.** The ruling's grounds, so nobody re-opens it:
  `RuntimeExpr::Closure` and `LexicalClosure` are **different capture-authority
  forms** — seed symbols resolved from the explicit seed environment versus
  expressions evaluated in the lexical environment. The seed form is a live
  member of the public backend-neutral IR, has evaluator semantics, and is
  exercised green through the native backend today. **Deleting only
  `Call { callee: Closure }` would make the same closure value callable after a
  `DeclarationRef` but unlawful written directly, purely by source position.**

  The six binding points:

  1. For a `FunctionizedUnits` `Call` whose **exact callee occurrence** is
     `RuntimeExpr::Closure`, use **that callee's planner-owned body unit**.
  2. Check `params.len() == args.len()` **before emission**.
  3. Lower and carry arguments from the call's child origins **in parameter
     order**. Resolve seed symbols **only** through `lower_seed_capture`, retain
     **capture source order**, and pass exactly **`Parameter ++ Capture`** to the
     existing typed `call_declared_unit` path.
  4. **Do NOT route through `DeclarationClosure` identity or checked-recursion
     validation.** A literal callee has no declaration-reference occurrence, no
     symbol identity, and no checked-call template to validate. Reuse the **unit
     transport**, not the declaration-specific identity join.
  5. **No `Constructor`/`Record` capture environment is needed. Trap 5 is
     VACUOUS — record it as such and mint no token.** If an aggregate, or a
     second codec or transport, appears: hard-stop.
  6. **Hard-stop** if the planner does not already own the exact callee-body
     unit, or the existing call path cannot accept the seed captures. Do not
     create a parallel mechanism.
- **`D3` — Remove `SeedClosureCall`** from `RecursiveDescentResidual`, and only
  then re-run `AC-1a` and `AC-1b`.

  **After `D3`, the exact `D1` firing population must select
  `FunctionizedUnits`, build and run, and enumerate no `SeedClosureCall`.** The
  delta-free `D0` remains the regression complement.

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
  population named. **The "enumerator lands in the candidate lineage" half is
  DISCHARGED BY INHERITANCE** — `7ca5cfc0` landed it (see `D1`). State that it is
  already satisfied; do not re-land it.

  The obligation still binds on anything you *add*: three later nodes in this
  campaign consume the same instrument, so any extension of it lands in the
  lineage rather than in a scratch worktree.

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
- **`AC-6` — `D2`/`D3` evidence, set by the Architect's ruling
  (`evt_7p8dmg1rez02c`). Three controls, and the third is not optional.**
  1. The **canonical explicit-seed-env positive**.
  2. The **missing-capture loud refusal**.
  3. **An ORDER-SENSITIVE `Parameter ++ Capture` control.**

  **Why the third exists, and it is a live blindness in the current corpus, not
  a hypothetical.** The existing seed computes `5 + 2 = 7`. **Addition is
  commutative, so that witness returns 7 whether the port passes
  `Parameter ++ Capture` or `Capture ++ Parameter`** — it cannot see the one
  ordering defect point 3 of `D2` exists to prevent.

  ⇒ **The smallest discriminating witness is a direct seed closure using
  `sub_int` with argument `5` and capture `2`, expecting `3`. A swap yields
  `-3`.** Do not discharge this AC with a commutative operator.

## 5. Banned scope

- **Building a second transport** parallel to `RT-DECL-CLOSURE-PORT`'s.
  Report the generality gap instead.
- **Retiring any other residual class.** Each is its own node.
- **Deleting the selector or the `RecursiveDescent` lane** — that is
  [[RT-DESCENT-RETIRE]], and it is gated on all four migrations.
- **DELETING OR NARROWING the `Call { callee: Closure }` capability.** Ruled out
  by the Architect (`evt_7p8dmg1rez02c`): a lawful delete is a separate
  structural IR redesign that removes seed closures from general expression
  position and recuts evaluator, corpus and public API together — and it would
  additionally owe the **executed** elaborator proof that the PORT branch does
  not. `D1` establishes no intrinsic reason for that redesign.
- **Routing the port through `DeclarationClosure` identity or checked-recursion
  validation** (`D2` point 4).
- **Minting a Trap 5 capture-environment token.** Trap 5 is vacuous here; record
  that it is, rather than discharging it.
- ~~**Manufacturing a population** if `D1` comes back empty.~~ **MOOT — `D1`
  came back non-empty.** `SeedClosureCall` fires; kept only so a reader does not
  think the ban was dropped.

## 6. Hard stop

Stop and report if:

- `D1` shows the class fires only through shapes that also fire a class owned by
  a later node — **not triggered; `D1` measured exactly `{SeedClosureCall}`** on
  the firing seed;
- `D2` cannot reuse the existing transport;
- **the planner does not already own the exact callee-body unit**, or the
  existing call path **cannot accept the seed captures** (`D2` point 6);
- **an aggregate, or a second codec or transport, appears** (`D2` point 5).
Per the campaign's Trap 2, a newly reachable program shape tripping a
fail-closed invariant is **expected** here — route it as its own node; do not
absorb it and do not work around it.
