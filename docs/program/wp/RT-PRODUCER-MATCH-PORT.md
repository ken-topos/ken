# RT-PRODUCER-MATCH-PORT — retire the `ProducerMatchCall` residual

**An ordinary producer `Match` whose scrutinee is directly a `Call` is a
retained residual, and any retained residual routes the whole object to the
monolithic `RecursiveDescent` root. This node makes a producer call in scrutinee
position deliver its result across a callable-unit boundary into the match.**

**Owner:** Team Runtime. **Branch:** `wp/RT-PRODUCER-MATCH-PORT`. **Size:** M.
**Risk:** medium.

 **Read `docs/program/16-recursive-descent-retirement.md` first.** It carries
the campaign's binding traps and the schedule. This frame does not repeat
them.

---

## 1. Fixed inputs

| path | blob at `origin/main = 14c3c5f7` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `f7bc0d0354d8b8d6f7aa68176846b7b05e5a8514` |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | `f57215905ad715cab67b580781d078a614e20dfd` |

 **Stale by pickup** — two nodes rewrite `core.rs` before this one. **Re-pin at
pickup.** The pin here is so the derivation below can be re-checked against
what changed, not so the numbers can be trusted.

## 2. The mechanism, at exact anchors

`core.rs:92-95`, the first test in the `RuntimeExpr::Match` arm:

```rust
RuntimeExpr::Match { scrutinee, cases, .. } =>
    matches!(scrutinee.as_ref(), RuntimeExpr::Call { .. })
        .then_some(RecursiveDescentResidual::ProducerMatchCall)
        .or_else(|| /* MatchScrutineeRecursor */ )
        .or_else(|| recursive_descent_residual(scrutinee))
        .or_else(|| /* case bodies */ )
```

**The gap:** a producer `Call` in scrutinee position must deliver its result
across a callable-unit boundary *into* the match, rather than being recursively
lowered into the generated root together with the match.

## 3. THIS CLASS IS TESTED FIRST, SO IT MASKS THE NEXT NODE'S POPULATION

`ProducerMatchCall` short-circuits **before** `MatchScrutineeRecursor`, before
the recursion into the scrutinee, and before the case bodies. ⇒ While this class
fires on a program, [[RT-RECURSOR-TRANSPORT]]'s classes may also be present on
that same program and be **completely unreported**.

**Two consequences, both binding:**

- **`D1` must enumerate all residuals** on every measured program, not the
  reported first. This is the campaign's Trap 1 and it is not optional here.
- **Retiring this class will make [[RT-RECURSOR-TRANSPORT]]'s population
  visibly larger.** That is a *measurement improving*, not a regression, and
  it does not mean this node broke something. **Do NOT record a before/after
  delta for it** — see `D4`: the before-side operand is destroyed by `D3`, and
  the successor measures its own population rather than inheriting a number.

## 4. Deliverables

- ** `D0` — the delta-free regression baseline.** Before applying any delta,
  run the target suite on the base and record which rows are green. That set is
  what `AC-1b` holds you to. A measurement carrying your own delta cannot
  produce it — see the campaign doc's **Trap 2**, where this cost
  [[RT-DECL-CLOSURE-PORT]] a candidate.
- **`D1` — Full-residual enumeration** across the measured programs, recording
  for each: which classes fire, and which were previously masked by this one.
   Post before building.
- **`D2` — Producer-call callable unit.** The scrutinee call becomes a
  separately owned callable unit whose result crosses a typed boundary into the
  match. Reuse the transport built by [[RT-DECL-CLOSURE-PORT]] where it applies;
   report rather than duplicate it if it does not.
   **If this unit synthesizes a `Constructor`/`Record` capture environment, the
  campaign doc's Trap 5 binds that site before its first allocation** — and if it
  allocates no aggregate, record that fact rather than minting a token.
- **`D3` — Remove `ProducerMatchCall`** from `RecursiveDescentResidual`, and only
  then re-run `AC-1a` and `AC-1b`.
- **`D4` — Post-retirement capability evidence.** That both surviving classes
  are visible to the enumeration once this class stops short-circuiting, and
  that the selector walks past the `Match` node.

  **RESTATED 2026-08-08, and the original demand was MINE AND WRONG.** `D4` said
  "the masked-population delta, handed to the Steward for
  [[RT-RECURSOR-TRANSPORT]]'s re-scope." **That measurement cannot be taken from
  this node and is not needed.**

  **Why it cannot be taken.** A before/after delta needs a pre-retirement
  operand, and **`D3` destroys it** — the variant no longer exists to name. The
  first attempt subtracted `{MatchScrutineeRecursor}` from a *post*-retirement
  value and labelled the subtrahend a pre-retirement selector result. Before
  retirement the selector reported `ProducerMatchCall` **and stopped**, so
  **both** survivors were hidden, not one, and the answer understated the
  successor population by half. **Manufacturing the operand from post-retirement
  values IS the defect**; the ring withdrew the claim rather than repairing it,
  which was correct.

  ⇒ **This is [[RT-DESCENT-RETIRE]] §3's "the oracle is spent by the commit that
  clears it," arriving one node early.** That frame writes it down in advance;
  this one did not.

  **Why it is not needed.** [[RT-RECURSOR-TRANSPORT]] `D2` already measures its
  own population — *"including the population unmasked by
  `RT-PRODUCER-MATCH-PORT`'s `D4` **if that node has landed by pickup**... It no
  longer gates this one — scope against what is measured on `c45a59a9`."* **The
  successor measures rather than inherits, so no figure is owed to it.**

  **Also note: a hand-built `RuntimeExpr` could not have supplied it anyway.**
  Campaign Trap 1 says a synthetic witness proves instrument and class
  visibility, never a real-program population.

  ⇒ **Standing question for any before/after claim in this campaign: NAME THE
  BASE EACH SIDE WAS MEASURED ON, BEFORE YOU SUBTRACT.** If one side has no base
  you can name, the subtraction does not exist.

### THE PROSE SWEEP IS PART OF `D3`. Budget for it.

**[[RT-SEED-CALL-PORT]] spent FOUR review rounds on this and the mechanism was
correct in every one of them.** Its implementer's own statement of the shape,
recorded here because this node performs the identical operation — retiring a
variant and changing a counted population:

> **Every claim which counts or names a member of a population my delta changes
> is falsified by that delta, and prose cannot go red.**

Concretely, what that node shipped and had bounced: a comment its own commit
falsified; a counter documented as an emission oracle whose increment sat before
a fallible call; a dozen sentences counting a population the same commit was
changing; then a corrective child claiming a completed sweep that was not one.

**The fourth round is the one to learn from, because it is the sweep itself
failing.** They wrote the grep, then ran it over a narrower domain than the
claim they made for it. Two specific misses:

- **It omitted the NEW count.** A retirement from N to N-1 makes the *new*
  number stale-able too — their own corrected prose had just introduced the word
  and the sweep did not cover it.
- **It never covered string literals** — which is exactly where a fixture's
  user-visible identity lives.

⇒ **Before handing back `D3`, sweep for every place the retired variant is
named OR counted, including string literals and including the number you just
wrote.** State the grep's domain alongside its result, so the claim and the
instrument can be compared. **No test will catch any of this.**

## 5. Acceptance criteria

- **`AC-1a` — the ceiling moved.** The selector reports
  `authority=FunctionizedUnits` / `residuals=none` on every program `D1` named
  as firing this class.
- **`AC-1b` — the objects still build.** Those programs **compile and pass**
  their existing suites, **and every row green in `D0` is still green**. Not
  "the residual is gone" — the objects build. `AC-1a` does **not** discharge
  this: it quantifies over the firing set, and the regression population is its
  complement (campaign doc, Trap 2).
- **`AC-2`.** `D1`'s enumeration and `D4`'s **post-retirement capability
  evidence** are in the tree. **A before/after delta is NOT required and must
  not be reconstructed** — `D3` destroys the before-side operand, and
  manufacturing it from post-retirement values is the defect `D4` records.
- **`AC-3` (no-regression).** Workspace green **in CI** — never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-4`.** The exhaustive-match fail-closed property at `core.rs:59-65` is
  preserved. No wildcard arm.
- **`AC-5`.** Emitted function count and per-function code-size distribution
  recorded for the affected programs, as at `RT-DECL-CLOSURE-PORT.AC-6`.
   Report; do not tune, do not pin a threshold.

## 6. Banned scope

- **Retiring `MatchScrutineeRecursor`** because it sits in the same `Match`
  arm and is now newly visible. It is [[RT-RECURSOR-TRANSPORT]]'s, it is folded
  with a sibling class there for a stated reason, and absorbing it here would
  build that transport twice.
- **Any other residual class.**
- **Deleting the selector or the `RecursiveDescent` lane** — that is
  [[RT-DESCENT-RETIRE]].

## 7. Hard stop

Stop and report if `D2` cannot deliver the call result across the boundary
without invocation-local scope/return-hole transport — **that is
[[RT-RECURSOR-TRANSPORT]]'s mechanism**, and discovering this node needs it is a
re-cut of the schedule, not a licence to widen. Per Trap 2, a newly reachable
shape tripping a fail-closed invariant is expected; route it as its own node.
