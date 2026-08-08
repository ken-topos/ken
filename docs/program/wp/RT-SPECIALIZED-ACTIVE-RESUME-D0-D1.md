# RT-SPECIALIZED-ACTIVE-RESUME — `D0`/`D1` census and partition

Base: `origin/main` `dcd6d84c604f07fdd19af2752d0d2d847a203c52`.
Frame blob `284029ac`, node blob `e92b5c38`, both read from the worktree at that
base rather than from the release post.

This document is the `D0`/`D1` checkpoint. **No production code changed.** The
census instrument was reverted before this record was written, and every
coordinate below was re-derived by name at the clean tree.

## 1. Method, and why it is placed where it is

One instrument at the **entry** of `lower_computational_match_value_composed`
(`core.rs:3761`), recording every arrival before any guard can return —
including the zero-eliminator refusal. Every arrival then emits exactly one
**disposition** naming the exit it took.

Placing the instrument above the destructure alone would have been wrong here.
`BoundedNat` and `StructuralNat` are routed away *before* the destructure, and
`specialized_at` can refuse before either. An instrument below those exits
cannot distinguish "this class has no members" from "this class left earlier",
which is the whole property the census exists to establish.

Two things about the instrument are load-bearing, and both were failures first:

- **The record key is `(pid, seq)`, not `seq`.** Several `ken-runtime` tests
  spawn subprocesses, which inherit the census environment variable and link the
  same library, so their counters restart at zero. Measured: **six** writer
  processes, with `seq=0` reused six times carrying different content. Keyed on
  `seq` alone the first run silently merged distinct arrivals and under-reported
  the denominator as 482 against a true 507.
- **One `write_all` per record.** `writeln!` issues a syscall per format
  fragment, so concurrent libtest threads interleave mid-row. Zero malformed
  records in every run reported here.

**Instrument validation, per run, before any number below was read:** arrivals
equal dispositions; every arrival has exactly one disposition; no disposition
without an arrival; no duplicate key; no malformed row.

| run | arrivals | dispositions | 0-disp | >1-disp | orphan | malformed |
|---|---|---|---|---|---|---|
| retained | 507 | 507 | 0 | 0 | 0 | 0 |
| A-only exclusion | 497 | 497 | 0 | 0 | 0 | 0 |

## 2. Both configurations were censused, because the lane changes the variant

The A rows only reach the fifth wall under **A-only exclusion**. A retained-run
census alone would not have measured this node's population at all.

Arming the exclusion suite-wide took one correction worth recording. The first
probe applied the exclusion unconditionally and fired the committed
`debug_assert` in `select_body_emission_authority` on roughly a hundred
programs, aborting the run after 100 records. **That assertion is correct** —
its own message says a blanket exclusion "would then measure an ordinary
functionized program rather than this position". The probe was narrowed to arm
only where `MatchScrutineeRecursor` is genuinely a residual, since removing an
absent member is a no-op by definition. A census over a run that aborted at
100 of ~1000 records is not a measurement, and was not read as one.

Under exclusion the suite is **812 passed / 6 failed / 4 ignored**. That is a
measurement configuration, not a suite verdict: it deliberately reds the rows
that depend on the retained lane.

## 3. `D0` — the denominator, closed at the production boundary

Every arrival, by operand phase and first eliminator frame. `LoweringOperand`
has exactly two arms and `EliminatorFrame` exactly five, so this table is
closed by construction, not by enumeration.

| phase x first frame | retained | A-only exclusion |
|---|---|---|
| `Carried` x `Computational` | 244 | 245 |
| `Specialized` x `Computational` | 207 | 200 |
| `Specialized` x `Ordinary` | 34 | 26 |
| `Carried` x `Ordinary` | 8 | 10 |
| **`Specialized` x `Active`** | **6** | **4** |
| `Carried` x `InvocationReturn` | 4 | 6 |
| `Carried` x `Active` | 4 | 6 |
| `Specialized` x `InvocationReturn` | 0 | 0 |
| any x `PendingLet` | 0 | 0 |
| any x no eliminator | 0 | 0 |
| **total** | **507** | **497** |

## 4. The population, and the two members that are not evidence

`Specialized x first-Active` under A-only exclusion is **4 arrivals**, and they
are uniform on every recorded axis:

- variant `ProcessExitStatus`, all four;
- `pending_len = 0`, `pending_kinds` empty, all four;
- `activation = ContinuationActivationId(0)`, `cursor = ContinuationCursorId(3)`,
  `parent = true`, `selected_ancestry = 1`, `source_lineage = 0`,
  `source_selected_cursor = None`, `selected_scope = false`;
- `route = DirectScrutinee`, `owner = Some(Predeclared(PredeclaredFunctionId(0)))`;
- disposition `ConstructorRefusal`, all four — the fifth wall.

**Attributed by fixture, which is what makes the exclusion possible:**

| fixture | committed control? |
|---|---|
| `d8d_the_composed_binding_site_is_live_and_neither_landed_population_installs_a_target` | no |
| `px8j_all_three_producer_paths_reach_real_consumers` | no |
| `ccr_d3_the_active_carried_route_is_taken_and_the_continuation_refusal_is_gone` | **yes** |
| `coc_d3_the_trailing_suffix_is_continued_and_the_mutation_restores_the_refusal` | **yes** |

⇒ **The independent population is two.** The other two are this chain's own
`D3` controls, which arm the exclusion hook internally and are therefore members
of the population they observe. Quoting four would inflate the perimeter with
artifacts of the measurement.

This measurement **closes** the frame's floor rather than merely re-finding it:
the two known rows are not a floor under a wider population, they are the whole
independent population at this base.

## 5. `D1` — the partition, with every zero stated

All five classes the frame names, measured under A-only exclusion, plus the
retained reading where it differs.

| class | independent members | arrivals | disposition |
|---|---|---|---|
| ordinary-live (`ProcessExitStatus`) | **2** | 4 | `ConstructorRefusal` |
| `Constructor` | 0 | 0 | — |
| `BoundedNat` (routed control) | 0 | 0 | route never taken |
| `StructuralNat` (routed control) | 0 | 0 | route never taken |
| `RecursiveBackedge` | 0 | 0 | — |
| `Trap` | 0 | 0 | — |
| any other `Lowered` variant | 0 | 0 | — |

**Exactly one cell has members.**

Three of those zeros are stronger than "not observed with an `Active` frame":

- **`BoundedNat` and `StructuralNat` are never reached from this function at
  all**, in either configuration, with any first frame. The
  `lower_bounded_nat_computational` routes above the destructure took **zero**
  arrivals out of 1004 across both runs.
- **`specialized_at` refused zero times** and the zero-eliminator arm was reached
  **zero times**.
- **Only three of the twenty-two `Lowered` variants ever reach this boundary**:
  `Constructor` (470 arrivals across both runs, all accepted), `ProcessExitStatus`
  (6, all refused), `RecursiveBackedge` (1, refused). The variant axis is taken
  from `lowered_value_kind`, which is a committed wildcard-free dispatch table,
  so a new variant cannot be silently absorbed into an existing bucket.

### The discriminating fact, which is about the lane and not the frame

The **same two programs** reach this boundary in both configurations and arrive
with a **different variant**:

| | retained | A-only exclusion |
|---|---|---|
| `d8d`, `px8j` variant | `Constructor` | `ProcessExitStatus` |
| first frame | `Active` | `Active` |
| `pending_len` | 1 (`Ordinary`) | 0 |
| disposition | `ConstructorAccepted` | `ConstructorRefusal` |

The `Active` frame is constant across the two; the operand's variant and the
frame's pending suffix are what move. ⇒ **The wall is not about the `Active`
frame.** It is that the activated lane delivers an ordinary live value where the
retained lane delivered a constructor, and the destructure demands constructor
shape from both before it dispatches either.

That is the first missing static fact, stated per class: for the ordinary-live
cell there is **no missing fact**. Nothing the destructure extracts is consumed
on the path the `Active` frame would take — which is exactly the Architect's
ruling, now with a measured population under it rather than an argument.

## 6. `AC-5`'s discriminator is disjoint from the repair cell, by measurement

The committed full-equality suppression control is
`rt_d2_exact_counts_and_the_suppression_ab` (`control.rs:11345`), pinning the
fifth refusal in full equality.

**Its arrival at this boundary is `Specialized(RecursiveBackedge)` with first
frame `Ordinary`** — one arrival, retained configuration, disposition
`ConstructorRefusal`.

⇒ It is outside the repair cell **on both axes at once**: different variant and
different first frame. A repair keyed on `Specialized(ProcessExitStatus)` x
first-`Active` cannot reach it, so its suppressed `RecursiveBackedge` path still
lands on the exact constructor refusal afterwards. `AC-5` is discharged by
disjointness rather than by re-running the control and hoping.

Stated as its own sentence, because a measured property can be true and about
something else:

- **MEASURED:** the `AC-5` control's only arrival at this boundary carries
  variant `RecursiveBackedge` and first frame `Ordinary`.
- **CLAIMED:** a repair scoped to `ProcessExitStatus` x first-`Active` leaves it
  reaching the exact refusal.
- **THE GAP:** none for the scope above, but the claim is **conditional on that
  scope**. Any widening of the repair's key — to all of first-`Active`, or to any
  non-constructor variant — reintroduces the risk, and that widening is precisely
  the hoist the frame's constraint 1 forbids. If `D2`'s key ever widens, this
  disjointness must be re-measured, not carried forward.

## 7. What `D2` looks like from here, and the interface question

`resume_active_continuation` (`core.rs:2059`) takes a **`LoweringOperand`**, not
a `Lowered`. So a `Specialized(ProcessExitStatus)` operand is expressible at that
entry **by signature**. No interface widens.

Its first statement is:

```rust
let Some((head, tail)) = active.pending.split_first() else {
    return Ok(value);
};
```

**Every measured member has `pending_len = 0`**, so for the whole measured
population the resume is the identity on the operand. That is worth stating
plainly rather than discovering during `D2`: the measured cell's repair is a
routing decision, and the thing it routes to returns the value unchanged.

It also bounds what this checkpoint may claim. A resume over a **non-empty**
`pending` is unexercised by every member of this population, so `D2` must not
present the empty case as evidence for the general one.

## 8. Hard stops: none fired

| stop | result |
|---|---|
| 1. classes need materially distinct mechanisms | **no** — one cell has members |
| 2. repair needs a planner or ABI population | **no** — no planner or ABI surface reached |
| 3. routing requires widening an interface | **no** — `resume_active_continuation` already takes a `LoweringOperand` |
| 4. census finds a variant with no cell | **no** — all three observed variants have cells |
| 5. a sixth wall | **not reached** — `D2` has not run |

Stop 5 is the one this checkpoint cannot answer, and it is not evidence of
absence.

## 9. Re-size input

**One cell, two independent members, uniform on every axis, routed to an entry
that already accepts the operand type and that returns the identity for the
measured `pending_len = 0`.** No planner, no ABI, no interface change, and the
`AC-5` discriminator measured disjoint from the repair key.

That is the same evidence shape that re-sized the predecessor from `M` to `S`.
The re-size is the Steward's call, not this document's.

## 10. What this checkpoint does NOT establish

- **Trap 1 is unchanged and this census does not close it.** Both independent
  members are hand-built `RuntimeExpr` values. The instrument was written
  environment-gated rather than `#[cfg(test)]` specifically so it could also run
  inside `ken-cli`'s and `ken-verify`'s integration binaries, where `ken-runtime`
  is built without `cfg(test)` and a `cfg(test)` instrument is structurally
  blind. **That cross-crate run was not performed at this checkpoint**, so
  nothing here establishes that a real Ken program exhibits the population. The
  capability now exists; the measurement has not been taken.
- The zeros are **over this corpus at this base**. They are what makes the empty
  cells safe to leave fail-closed; they are not unreachability.
- The retained-configuration reading is not this node's population. It is
  recorded because the variant difference between the two lanes is the finding.
