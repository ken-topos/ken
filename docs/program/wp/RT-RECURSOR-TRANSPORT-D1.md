# RT-RECURSOR-TRANSPORT — `D0` re-census, `D1` activation probe, `D2` repair

**Base, pinned: `f4212c2cc8a990410f6a58c48ff482b681f2e706`.** Frame blob
`80311af5`, node blob `ed829abd`, both verified present at that base before any
edit. **`D0` and `D1`'s measurements were taken on that tree; `D2`'s were taken
at the `D2` evidence checkpoint `392e883a`**, a different object — **every
count in this document names the object it was measured on, never a role.**

**`D1`'s answer is asymmetric.** One position closes for free; the other does
not and is routed as a refusal. That is the input to the Steward's re-size, and
it is stated here rather than folded into `D2`.

Citations are by grep-able phrase. Line numbers for these two variants have gone
stale twice already in this campaign.

---

## D0 — the re-census

### The delta-free baseline, taken before any edit

```
git rev-parse HEAD    -> f4212c2cc8a990410f6a58c48ff482b681f2e706
git status --porcelain -> empty          (delta-free: no edit of mine exists yet)
scripts/ken-cargo build -p ken-runtime   -> Finished, 50 warnings
df -h /workspaces                        -> 11G available
git rev-parse HEAD    -> f4212c2c...     (re-quoted before the suite)
scripts/ken-cargo test -p ken-runtime --lib
```

**812 passed, 0 failed, 4 ignored, at `f4212c2c`.** The anchor was quoted three
times in the one shell and the worktree was empty at the time, so this is a
baseline of that object and not of my change.

### Both variants are live and selected

The `RecursiveDescentResidual` enum carries exactly two live variants, with the
three retired siblings present only as comments recording their retirement:

| variant | doc phrase it is declared under |
|---|---|
| `MatchScrutineeRecursor` | *"An ordinary match consuming an active computational recursor."* |
| `LexicalCallArgumentRecursor` | *"A lexical unit call whose argument is an active computational recursor."* |

Classified in `recursive_descent_residual`, by shape rather than by name:

- **`MatchScrutineeRecursor`** — a `RuntimeExpr::Match` whose scrutinee is a
  `ComputationalMatch` with any case whose `recursive_positions` is non-empty.
- **`LexicalCallArgumentRecursor`** — a `RuntimeExpr::Call` whose callee is a
  `LexicalClosure` and whose arguments include such a `ComputationalMatch`.

**Census over the whole of `lowering/`:** `MatchScrutineeRecursor` at 10 sites,
`LexicalCallArgumentRecursor` at 7, and **no third variant name anywhere**.

### The exhaustive selector and enumerator are preserved

Neither walker has a wildcard arm — checked across the whole classifier and
enumerator span. A new `RuntimeExpr` form still cannot compile until someone
classifies it, which is the property that keeps the instrument from silently
under-reporting as the IR grows.

`BoundaryUse`: **zero hits in `crates/`**, re-derived at this base.

---

## D1 — the activation probe

### The mechanism, and why it is built on the enumerator

A test-only per-variant selector exclusion,
`set_selector_variant_exclusion(Some(variant))`, sitting in
`select_body_emission_authority`. It enumerates the **full** residual set using
the landed `enumerate_recursive_descent_residuals`, removes exactly the one
variant under test, and lets the remainder decide the lane.

**It is not a second walker, and the reason is not tidiness.** The selector's
own classifier short-circuits at the first residual it finds. Subtracting one
variant from *that* answer would silently also drop every variant it never
reached, so the probe would read "nothing retains this" from an instrument that
had stopped looking. Enumerating first and removing one member is the only
subtraction that means what it says.

The mechanism is `#[cfg(test)]`; with no exclusion set the selector takes its
original path, and in a production build the branch does not exist. Both
witnesses assert their program is **single-variant** before probing, so the
remainder after removal is empty by construction rather than by luck.

### Position A — `MatchScrutineeRecursor`: does NOT close for free

⚠ **This section is the `D1` finding, measured at `2e5e6a8b`. It is HISTORY.**
`D2` repairs this position and both lanes now execute — see the `D2` section.
The refusal below is what `D1` measured, not what the current object does.

| lane | outcome |
|---|---|
| retained (no exclusion) | **executes**, `Returned(Int(Small(7)))` |
| functionized (excluded) | **compile-time refusal**, never executes |

The refusal, verbatim:

```
Unsupported(UnsupportedLowering {
  construct: "ComputationalMatch",
  reason: "scrutinee is not a constructor value after ordinary expression lowering" })
```

**This is routed as a refusal and is NOT recorded as an outcome**, exactly as
the frame requires: a compile-time refusal that never executes is not a
behavioural result.

**It is the position's refusal, not the fixture's, and that is measured.** The
control is the same ordinary-`Match`-over-`ComputationalMatch` program with
**no recursive position**: it is not a residual at all, takes the functionized
lane with no exclusion set, and **executes**, returning `Int(Small(7))`. The
only difference between the two programs is the recursive position, so that is
the discriminator.

Without this control the red would have been equally consistent with *"an
ordinary match cannot consume a computational match at all"* — a different and
much larger claim, and one that would have mis-scoped `D2`.

### Position B — `LexicalCallArgumentRecursor`: CLOSES FOR FREE

| lane | outcome |
|---|---|
| retained (no exclusion) | executes, `Returned(Constructor "ctor:fixture::rt::Leaf")` |
| functionized (excluded) | **executes**, `Returned(Constructor "ctor:fixture::rt::Leaf")` |

**The decoded observations are identical.** The landed continuation machinery
already carries this position; there is nothing to build for it.

**One difference deliberately not compared, and why.** The raw native result
token differs — `0` on the retained lane, `517` on the functionized one. It is
a **pre-decode, lane-internal** value: `run` returns it alongside the
observation, and the two lanes encode the same result differently before
decoding. Comparing tokens across lanes would manufacture a divergence where
the semantics agree. The decoded `RuntimeObservation` is what the lanes are
required to agree on, so that is what the witness compares. Stated rather than
silently dropped, because a reader running the probe by hand will see the
difference.

### What the witnesses are

Both are **closed and executable**. The landed `d1_*_witness` fixtures
scrutinise a free `Var(0)`: excellent for asking the classifier a question,
impossible to compile or run. `D1` needs an executed outcome, so the probe uses
the same shapes closed over a real constructor.

---

## Consequences for sizing

- **Position B needs no `D2`.** Any `D2` is for position A alone.
- **Position A's refusal is at lowering**, on the shape of the value an ordinary
  match receives from a computational match with a recursive position. Whether
  the narrow binding that fixes it is available over the existing continuation
  machinery, or requires a new planner/ABI population — hard stop 2 — is not
  answered by `D1` and is not guessed at here.
- **Hard stop 1 is not triggered by this evidence, and I am not claiming it is
  cleared either.** The two positions demonstrably differ in *outcome*, but a
  difference in outcome is not the same as *"materially different transports"* —
  one of them needs no transport at all. Whether the remaining position's
  transport is materially different from anything is a `D2` question.

**Suite at the `D1` checkpoint `2e5e6a8b`: 814 passed, 0 failed, 4 ignored** —
the 812 delta-free baseline plus exactly the two `D1` witnesses
(`rt_d1_position_a_...`, `rt_d1_position_b_...`).

⛔ **That figure is bound to `2e5e6a8b` and is not a current-candidate count.**
`D2` adds controls, so the population moves; the count for whatever object you
are reading is in that deliverable's own section, never here. A count that says
"this candidate" is true only until the next commit, which is the defect this
line previously had.

---

## D2 — position A closes, by propagating the backedge protocol marker

Architect disposition `evt_bqg3gjwkp350`: hard stop 2 **not triggered**, node
stays `M`, and the repair is a narrow protocol-marker propagation.

**`Lowered::RecursiveBackedge` is not a value.** It records that a
tail-recursive edge has already been emitted as a CFG jump and the current
block is predecessor-free, so every enclosing combinator must propagate it.
`lower_carried_computational_match` was right to return it; the defect was the
next consumer. `resume_active_continuation` saw a non-empty pending suffix and
unconditionally handed the marker to the outer ordinary eliminator, which asked
a protocol token to be a constructor.

The repair sits in `resume_active_continuation`, after the pending suffix is
known and **before** cursor minting, the successor `Active` frame, and any
eliminator work, so the predecessor-free path emits no suffix block,
allocation, call, claim or occurrence-plan consumption. It matches
`Specialized(RecursiveBackedge)` only.

| run | seat arrivals | backedge matches | propagations | result |
|---|---:|---:|---:|---|
| position A, functionized | 1 | 1 | **1** | `Returned(Int(Small(7)))` |
| position A, retained | 1 | 0 | 0 | `Returned(Int(Small(7)))` |
| position A, propagation suppressed | 1 | **1** | 0 | the exact `D1` refusal, replayed |
| non-recursive control | **0** | 0 | 0 | executes; never reaches this seat |
| position B | **0** | 0 | 0 | executes; never reaches this seat |

**Every figure in this table is asserted exactly by
`rt_d2_exact_counts_and_the_suppression_ab`, not as a lower bound.** A
duplicated seat consumption would satisfy `> 0` while this table claimed one.

**Both lanes now agree on position A's executed answer.**

### Two corrections to my own earlier readings, kept rather than overwritten

- I called an empty generated-context population "the edge". It is the
  planner's documented mixed-population state: a context is minted only for a
  causal call whose emission owner is a `Specialization`, and the trace's step
  `A` records this one's owner as `Predeclared`. Never hard-stop-2 evidence.
- I called `RecursiveBackedge` a mis-presented value. It is a protocol marker,
  and manufacturing a carrier or constructor for it would have duplicated a
  path that had already jumped.

### Two different zeros, and the counter that separates them

A zero propagation count has two possible causes, and they are not the same
finding:

- **the guard declined** — position A's retained lane *arrives* at the seat
  with a non-backedge value, is declined, and still consumes its eliminator.
  This is the genuine same-seat non-backedge control.
- **the seat was never reached** — position B and the non-recursive control
  have arrivals `0`. Their zeros are **scope** evidence and say nothing about
  the guard's behaviour.

⛔ Not every zero is paired with a positive arrival count, and an earlier draft
of this record said it was. Two of the rows are explicitly zero-arrival, which
is the point of showing arrivals at all.

### Why the suppressed run counts MATCHES, not just propagations

The production guard is `!suppress && matches!(..)` and **short-circuits**. Under
suppression the `matches!` is never evaluated, so a zero propagation count is
guaranteed *by construction* — an A/B whose mutated side asserts only that zero
proves nothing about whether the detector would have fired.

The backedge-match counter is incremented **before** the guard, so the
suppressed run shows one arrival and **one match** with zero completed
propagations. That is what makes the suppression a real A/B, and it is why the
mutated arm also asserts exactly one `RT-D2 E COMPOSED-CONSUMER` event carrying
`actual_kind=RecursiveBackedge`: asserting the error message alone would let a
different failure with the same substring pass.

### Validation

**Suite at the `D2` evidence checkpoint `392e883a`: 816 passed, 0 failed, 4
ignored** — the 814 at `2e5e6a8b` plus the two `D2` controls. Targeted, same
shell, anchor quoted before and after; never `--workspace`.

⛔ **`392e883a` is named rather than "the `D2` candidate", and the distinction
is the one this document keeps getting wrong.** A role moves; an object does
not. The child that added the oracle assertion below introduces **no test**, so
the population and therefore this figure are unchanged from `392e883a` — which
is why binding to the parent evidence SHA is stable and avoids a commit trying
to name its own hash.

Every count in this document names the SHA it was measured on, and none is
labelled by role:

| figure | object | what it is |
|---:|---|---|
| 812 / 0 / 4 | `f4212c2c` | the delta-free `D0` baseline, taken with an empty worktree |
| 814 / 0 / 4 | `2e5e6a8b` | the `D1` checkpoint — the baseline plus the two `D1` witnesses |
| 816 / 0 / 4 | `392e883a` | the `D2` evidence checkpoint — plus the two `D2` controls |

A suite figure is destroyed by the next commit that adds a test, so a count
bound to a position rather than to an object is stale the moment it is true.

⛔ **This footer previously grouped `812` and `814` together at `2e5e6a8b`.
`812` was measured at `f4212c2c`, before any edit of mine existed** — that is
the whole point of a delta-free baseline, and mislabelling it attaches the
baseline to a tree that already contains the change it is supposed to be a
baseline of. The error was in the sentence asserting the rule, which is the
form these keep taking: a summary restating measurements loosely enough to be
wrong while reading as a tidy conclusion. The table above replaces the prose
because a table cannot group two figures under one object by accident.
