---
scope: fleet
audience: (see scope README) — anyone who accepts an intermediate checkpoint,
  approves at a QA or merge gate, publishes on a Decision, or writes a
  handoff that reports test status as a delta
source: RT-FNSPLIT-RECUR-PORT, 2026-07-28 — six "inherited" reds turned out
  to be zero inherited and seven caused, measured only when the gate asked
  for the absolute reading
---

# "No new red" is measured against the LAST CHECKPOINT, not the merge-base

⛔ **A regression introduced at a WP's first checkpoint becomes "inherited
debt" by its second, and is never re-measured again.** Every later checkpoint
compares itself to the previous one, so *"no new red"* is **true at every
step** while the object silently diverges from its merge-base. The chain of
honest deltas sums to a false claim about the object.

## What happened

`RT-FNSPLIT-RECUR-PORT` ran five checkpoints on one branch. At `D8` the wider
control module read **83 pass / 6 red**, and those six were accepted by the
leader as *"preserved checkpoint debt"*. Each of the four checkpoints after it
reported **"no new red"** — accurately. Runtime QA then approved the full
candidate, listing the targeted controls that passed.

The gate asked one question: *state the reds **at the exact object**, and say
whether they are inherited.* The leader ran the full module on **both**
objects:

| object | result |
|---|---|
| merge-base `b4491297` | **83 passed / 0 failed** |
| candidate `e0fe7a81` | **84 passed / 7 failed** |

⭐ **Zero inherited. Seven caused** — and a **seventh** nobody had ever named,
`exactly_one_plan_origin_to_expression_lookup_exists`. The candidate could not
truthfully carry any of them as preserved debt.

## Two traps inside the first one

⛔ **A mechanism argument was used to excuse a red, and it was false.** The
`lower_expr` population oracle was dismissed with *"`D8` adds zero `lower_expr`
calls."* That is a **prediction**, and the counter-measurement is cheap: the
census had moved `65 → 68`. ⇒ **An explanation for why a red does not matter is
a claim to be tested, not a disposition.**

⚠ **Do not retire an oracle on class grounds while its finding is
unexplained.** The Steward proposed retiring that census as a source-line-count
oracle, which the operator has ruled against as a class (*"test oracles that
assert facts about source code, catalog, or documentation lines are an
invitation for failure and delay"*). ⭐ **That disposition was conditional on
the reds being inherited — they were not.** A devalued oracle class still
caught a real regression that a mechanism claim denied. Retire the oracle
*after* the finding is explained, never as the explanation.

## How to apply

1. ⭐ **At any gate, require the ABSOLUTE reading at the candidate AND at the
   merge-base.** Two numbers, one command each. ⛔ Never accept a delta chain
   as evidence about an object — *"no new red"*, *"no regression"*, *"same as
   the last checkpoint"* are all claims about a **step**, not the artifact you
   are about to merge.
2. ⛔ **"Inherited" / "pre-existing" / "preserved debt" is a claim about the
   BASE.** It can only be discharged by measuring the base. A memory of an
   earlier checkpoint's reading is not a base measurement — the earlier
   checkpoint is inside the WP.
3. **Write handoffs so this cannot hide.** Report `N passed / M failed at
   <sha>`, not *"no new failures"*. If you name debt as inherited, name the
   base sha you measured it at in the same sentence.
4. ⚠ **The QA approval is not where this gets caught.** QA binds the exact
   object and runs the targeted controls; the wider module's absolute reading
   is a different question, and it goes unasked unless someone asks it.

Related: [[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]].
⭐ Two neighbouring shapes worth holding alongside it: **state a diff claim
against the anchor your READER holds**, not the one you happen to be standing
on; and an aggregate differential can pass while exactly one of its N
contributors defects, because the sum hides the arm.
