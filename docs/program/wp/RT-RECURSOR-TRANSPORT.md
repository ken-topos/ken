# RT-RECURSOR-TRANSPORT — carry an active recursor across a unit boundary

**An active computational recursor's result carries invocation-local
scope/return-hole state. Two residual classes exist solely because that state
cannot cross a functionized unit boundary. This node builds that transport — or
proves the state need not cross — and retires both classes.**

**Owner:** Team Runtime. **Branch:** `wp/RT-RECURSOR-TRANSPORT`. **Size:** L.
**Risk:** ⭐ **highest in the campaign.** This is the mechanism the B2F migration
stopped at.

⛔ **Read `docs/program/16-recursive-descent-retirement.md` first.** It carries
the campaign's three binding traps and the schedule. This frame does not repeat
them.

---

## 1. Fixed inputs

| path | blob at `origin/main = 14c3c5f7` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `f7bc0d0354d8b8d6f7aa68176846b7b05e5a8514` |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | `f57215905ad715cab67b580781d078a614e20dfd` |
| `crates/ken-runtime/src/cranelift_backend/lowering/mod.rs` | `b924db34df3be74421fa773132fe476a53503ecc` |

⚠ **Stale by pickup** — three nodes rewrite `core.rs` first. **Re-pin at
pickup.** ⛔ The pin exists so the derivation can be re-checked, not so the
numbers can be trusted.

## 2. The two classes are one mechanism

**`MatchScrutineeRecursor`** — `core.rs:96-105`:

```rust
matches!(scrutinee.as_ref(),
    RuntimeExpr::ComputationalMatch { cases, .. }
        if cases.iter().any(|case| !case.recursive_positions.is_empty()))
    .then_some(RecursiveDescentResidual::MatchScrutineeRecursor)
```

**`LexicalCallArgumentRecursor`** — `core.rs:125-136`: a `Call` whose callee is
a `LexicalClosure` and whose **argument** matches that same
non-empty-`recursive_positions` shape.

⭐ **The code states the shared mechanism itself**, in
`LexicalCallArgumentRecursor`'s doc comment (`core.rs:47-52`):

> *"The recursive result still carries invocation-local scope/return-hole state.
> Passing it through a separately declared lexical unit is not one of the
> completed functionized ports, so the established recursive descent lane
> retains the whole call."*

⇒ Same predicate, same carried state, two syntactic positions: match scrutinee
and lexical call argument. ⛔ **Retiring one without the other builds the
transport twice.**

## 3. ⭐⭐ `D1` IS A FEASIBILITY PROBE AND IT CAN BE PULLED FORWARD

This node is **sixth** in a seven-node campaign. If the mechanism is infeasible
as scoped, we learn it after five nodes of investment — and *half-migrated* is
the state the operator's directive rules out.

⭐ **`D1` needs no code change, no branch, and no queue position.** It can be run
during [[RT-DECL-CLOSURE-PORT]] or any later node, by anyone, at any time. **If
the Architect or the ring wants this risk retired early, run it early and hand
the result to the Steward to re-cut the schedule.**

⛔ **Do not reorder the *build* work to chase it.** The transport machinery built
earlier in the campaign is real preparation for this node, and starting here
first would discard that.

## 4. Deliverables

- **⭐ `D0` — the delta-free regression baseline.** Before applying any delta,
  run the target suite on the base and record which rows are green. That set is
  what `AC-1b` holds you to. ⛔ A measurement carrying your own delta cannot
  produce it — see the campaign doc's **Trap 2**, where this cost
  [[RT-DECL-CLOSURE-PORT]] a candidate. ⚠ This node exposes the campaign's
  largest newly reachable population, so its baseline is the one most likely to
  move.
- **`D1` — The feasibility answer, and it is a question about the state, not the
  code.** For the invocation-local scope/return-hole state a recursive result
  carries, determine and report which holds:
  - **(a)** the state can cross a unit boundary under a typed transport —
    name the transport's shape;
  - **(b)** the state **need not cross**, because it can be shown dead at the
    boundary for the real population — name the proof obligation;
  - **(c)** neither, as scoped. ⛔ **Then stop and report.** (c) is a genuine
    outcome, it re-cuts the campaign, and ⛔ it must not be softened into (a) by
    describing an unbuilt mechanism as buildable.

  ⭐ Report (a)/(b)/(c) **explicitly**, in those terms. ⛔ A `D1` that describes
  the problem without landing on one of the three has not been delivered.
- **`D2` — Full-residual enumeration** across the measured programs, including
  the population unmasked by [[RT-PRODUCER-MATCH-PORT]]'s `D4`. ⚠ Scope this
  node against **that** number, not against the pre-campaign one.
- **`D3` — The transport**, per `D1`'s answer, covering **both** syntactic
  positions.
- **`D4` — Remove both `MatchScrutineeRecursor` and
  `LexicalCallArgumentRecursor`**, and only then re-run `AC-1a` and `AC-1b`.

## 5. Acceptance criteria

- **`AC-1a` — the ceiling moved.** The selector reports
  `authority=FunctionizedUnits` / `residuals=none` on every program `D2` named
  as firing either class.
- **`AC-1b` — the objects still build.** Those programs **compile and pass**
  their existing suites, **and every row green in `D0` is still green**. ⛔ Not
  "the residuals are gone" — the objects build. ⚠ `AC-1a` does **not** discharge
  this: it quantifies over the firing set, and the regression population is its
  complement (campaign doc, Trap 2).
- **`AC-2`.** `D1` lands on **(a)**, **(b)** or **(c)** explicitly and in the
  tree. Under (b), the proof obligation is discharged by a control, ⛔ not by
  prose.
- **`AC-3` — both positions, not one.** A control exercises the recursor in
  **match-scrutinee** position and another in **lexical-call-argument**
  position, and a mutation defeating the transport **reds** each. ⭐ One control
  covering one position would let the fold silently become a half-fold.
- **`AC-4` (no-regression).** Workspace green **in CI** — ⛔ never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-5`.** The exhaustive-match fail-closed property at `core.rs:59-65` is
  preserved. ⛔ No wildcard arm.
- **`AC-6`.** Emitted function count and per-function code-size distribution
  recorded for the affected programs, as at `RT-DECL-CLOSURE-PORT.AC-6`.
  ⛔ Report; do not tune, do not pin a threshold.

## 6. ⛔ Banned scope

- ⛔ **Retiring only one of the two classes.** They are folded for a stated
  mechanism reason; half is a worse state than neither, because it hides that
  the transport is incomplete.
- ⛔ **Softening a `(c)` into an `(a)`.** If the mechanism is not buildable as
  scoped, that is the deliverable.
- ⛔ **Deleting the selector or the `RecursiveDescent` lane** — that is
  [[RT-DESCENT-RETIRE]], gated on this node landing.
- ⛔ **Weakening the `recursive_positions` predicate** to shrink the population.

## 7. Hard stop

Stop and report on `D1 = (c)`, or if the transport lands and `AC-1b` still fails,
or if the two positions turn out to need different transports — ⭐ that last one
falsifies this node's fold and is the Steward's re-cut, not the ring's to
absorb. ⚠ Per Trap 2, this node exposes the largest newly reachable population
in the campaign; **expect a fail-closed invariant to fire and route it as its own
node.**
