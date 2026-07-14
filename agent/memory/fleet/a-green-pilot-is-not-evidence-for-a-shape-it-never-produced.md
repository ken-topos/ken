---
scope: fleet
audience: (see scope README) — anyone who runs a PILOT to calibrate
  an approach before a rollout, and anyone who reviews one
source: LET-3 Phase 2, 2026-07-14 — the pilot passed every gate, the
  Steward declared the approach calibrated, and the rollout
  immediately hit a hard elaborator defect in the one shape the pilot
  had never emitted
---

# A green pilot is not evidence for a shape the pilot never produced

**A pilot's value is its COVERAGE, not its greenness.** When a pilot
passes, the thing you have learned is *"the shapes this pilot emitted
work."* That is **strictly narrower** than *"the approach works"* — and
the gap between the two is invisible, because a green gate looks identical
either way.

## What happened

The LET-3 pilot rewrote two catalog sites with local `let` bindings. It
passed QA, the Architect, and the Steward's own review gate. The approach
was declared calibrated and a 34-lemma rollout was framed on top of it.

The rollout's very first lemma hit `KernelRejected { TypeMismatch
{ expected: Type 0, found: Ω0 } }` — **a landed defect in a shape the
pilot had never produced.** The Steward's frame had then pinned that
shape as a **fixed input**, walking the build team into unexplored
ground with a green light behind it.

## ★★ AND THE FIRST COVERAGE MATRIX I DREW WAS ON THE WRONG AXIS

**This is the sharpest part of the lesson, and I got it wrong before I
got it right.** Looking at which sites failed, the obvious axis was
**binding count**:

| | Type-0 (`fn`) | **Ω (proof)** |
|---|---|---|
| **grouped `let`** | ✅ pilot (`slice`, 4 bindings) | ✅ pilot (injectivity, 4+) |
| **single `let`** | (pre-existing paths) | ❌ never exercised |

**That matrix is WRONG, and it fits the evidence perfectly.** Every
failing site was a single binding; every passing site was a group. The
correlation was total.

**The Architect's grounding found the real axis: the LET BODY'S
ELABORATION MODE.** `check()` has **no `RLet` arm**, so a `let` in
*checking* position falls through to `infer()` and **loses the expected
goal**; `infer_match` then builds a constant `… → Type ℓ` motive and
falls back to `Type 0` when the return classifier is `Ω`. **Binding
count is irrelevant** — a one-binding `let` and a group's outer node
have the *same resolved form*. The grouped bodies passed because they
happened to be **inferable applications**; the singletons failed because
they happened to put a **checked-mode `match`** right behind the `let`.

> **⇒ The axes of your coverage matrix must be the axes the MECHANISM
> turns on, not the ones the SYNTAX makes salient.** Binding count is
> visible in the source and reads like a variable. **Body mode is
> invisible, and it was the variable.** A matrix drawn on the salient
> axis will look complete, explain every observation, and still leave
> the real cell untested — because *it was never one of your cells.*

**The tell:** a proposed axis that **perfectly** separates pass from fail
on a small sample is *evidence of correlation*, not of mechanism. **Before
you trust it, make someone name the code path.** If nobody can point at
the branch where the two cells diverge, you have found a proxy, not a
cause.

## The rule

> **Before you accept a pilot, draw the matrix of shapes the ROLLOUT
> will emit, and mark which cells the PILOT actually produced. Any empty
> cell is unpiloted — and a fixed input that lands in one is a pin with
> no evidence under it.**
>
> **Then check the AXES against the mechanism, not against the syntax.**

The matrix axes are whatever the **mechanism** varies. Here the true axis
was **{inferable body, checked body}** — and **the pilot produced only
inferable bodies.** Nobody drew any matrix at all, so nobody saw that
half the space was untested; and the first matrix drawn *after* the
failure still named the wrong axis.

**The pilot cannot tell you this. Only the reviewer can**, and only by
asking the one question a green result never prompts: **"what did this
NOT do?"**

## Why it hides so well

- **Every gate passes**, because gates test the artifact that exists, not
  the artifact the rollout will produce.
- **The uncovered cell is often the one that looks *degenerate*** — and
  **the degenerate case is not the safe case.** It is frequently a
  *different code path*, and here it was the broken one.
- **The real axis is usually the invisible one.** Anything you can *see*
  in the source (a count, a keyword, a shape) is a candidate axis you
  will reach for first; the axis that actually forks the code (an
  elaboration mode, a calling convention, an inferred-vs-checked
  position) leaves **no mark in the text.**
- **A corpus with zero prior instances of the feature** (this catalog had
  ~27,000 lines and no local bindings at all) means **nothing else
  exercises it either.** A pilot into virgin territory is the *only*
  coverage there is — so its holes are the system's holes.

## The general form

This is the population error one level up.
[[an-enumeration-needs-a-proven-closure-not-a-better-grep]] says a
candidate set needs a **proven closure**, not a convenient list. **A
pilot is a *sample*, and a sample chosen by convenience is not a
population.** If you would not accept "the sites I happened to read" as
an enumeration, **do not accept "the shapes the pilot happened to emit"
as coverage** — and the closure you owe is over the shapes the *rollout*
will emit, not the ones the pilot found handy.

Sibling of [[verify-the-report-is-real-before-explaining-it]] and
[[adding-a-file-to-a-globbed-corpus-trips-oracles-you-did-not-enumerate]].
Same family: **a green result is only as strong as the space it ranged
over — and nothing in the green tells you what that space was.**
