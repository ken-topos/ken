---
scope: roles/steward
audience: (see scope README)
source: 2026-07-30, RT-DECL-CLOSURE-PORT — QA blocked on `static_recursor`
  before reaching the D8 "expected failure" claim, and the repair kickoff did
  not restate it
---

# A blocking verdict short-circuits the remaining acceptance claims

A candidate handoff listed its verification runs, one of which was:

> `d8_every_required_join_plan_is_consumed_exactly_once` — **expected retained
> true-base failure only**: `Backend(Module("retained body has no
> graph-derived call target in this unit"))`; candidate no longer changes the
> D8 mechanism.

QA ran `check`, full `d7_`, and `static_recursor`, **blocked on
`static_recursor` 0/6, and stopped.** The leader's repair kickoff then scoped
the repair precisely — and did not restate the D8 claim among the controls to
re-establish.

⇒ **That sentence is now owed by nobody.** It was never checked by anyone but
its author, and the block that should have exposed it instead consumed the
attention that would have.

**A block is not a clean slate for the claims it did not reach.** It feels
like one, because the candidate is dead and a fresh SHA is coming — so every
unexamined claim silently rides into the next round as settled.

**Why this particular claim is the dangerous shape:** it is a **negative
assertion** — *it still fails*. A negative assertion passes for **any** reason,
including a new and different cause that happens to render the same message.
**A message match is not a mechanism match.**

> ### OUTCOME: THE CLAIM WAS FALSE AND THE WP OWNED THE REGRESSION
>
> Measured at the zero-WP merge-base `f7cea8fd`, the control **passes**; the
> candidate **fails** it on the same path with an aggregate-record /
> source-occurrence owner disagreement for a dead nested-join origin. The
> implementer's verdict: *"This atomic WP owns the regression."*
>
> ⇒ **"Expected retained true-base failure" was a live regression wearing a
> baseline's clothes**, and it would have merged as an accepted known-failure.
> This is not a hypothetical hazard — it is the measured one.

**THE ANCHOR IS WHERE THIS CHECK ACTUALLY FAILS — AND IT FAILED TWICE
BEFORE IT WORKED.** Asking for the differential is the easy half. Both anchors
first used already contained the change:

| anchor tried | what it actually was | WP work vs merge-base |
|---|---|---|
| the candidate's parent | the **previous blocked candidate** | all of it |
| the *"preservation-only"* WIP seam | a hard-stop checkpoint, **not** a base | **+16451 / −3845 across 12 files** |

**"Preservation-only, no QA candidate" describes what the commit CLAIMS, not
what it CONTAINS.** It means nobody offered it for review — it is in-flight
state, often deep in it. Reading it as a clean base is the error that made two
rounds of the differential vacuous, and it was **my** error, not the ring's.

⇒ **A baseline anchor must be a commit with ZERO of the WP's changes** —
`origin/main`, or `git merge-base <candidate> origin/main`. **Measure the
anchor's diff before trusting it**, never infer cleanliness from its message.

**How to apply:**

- **At the publish gate, diff the handoff's claim list against what the QA
  verdict actually covered.** The residue is your obligation list. A verdict
  that blocks early has a *shorter* coverage than a verdict that passes, and it
  is the early block that reads as thorough.
- **Route the residue to the LEADER before the fresh candidate**, as an
  *acceptance* obligation, explicitly adding no scope to the repair. Folding it
  into the next review request is far cheaper than holding a publish later.
- **Discriminate an "expected failure" against a ZERO-WP anchor by cause, not by
  string** — `origin/main` or the exact merge-base, never the parent commit
  and never a preservation seam. Confirm the test exists at that anchor, then
  require the verdict to **record the anchor SHA**, so no later round
  re-litigates it.
- **"Unchanged from base" is a claim about two runs, and you were shown
  one.**

Sibling of [[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]]
— there the defect was in the control's design; here the control was fine and
**nobody ran it**, because a block upstream ended the pass.
