---
scope: fleet
audience: (see scope README) — anyone who runs a PILOT to calibrate an approach
  before a rollout, and anyone who reviews one
source: LET-3 Phase 2, 2026-07-14 — the pilot passed every gate, the Steward
  declared the approach calibrated, and the rollout immediately hit a hard
  elaborator defect in the one shape the pilot had never emitted
---

# A green pilot is not evidence for a shape the pilot never produced

**A pilot's value is its COVERAGE, not its greenness.** When a pilot passes, the
thing you have learned is *"the shapes this pilot emitted work."* That is
**strictly narrower** than *"the approach works"* — and the gap between the two is
invisible, because a green gate looks identical either way.

## What happened

The LET-3 pilot rewrote two catalog sites with local `let` bindings. It passed QA,
the Architect, and the Steward's own review gate. The approach was declared
calibrated and a 34-lemma rollout was framed on top of it.

**Both pilot sites emitted multi-binding GROUPS.** Neither emitted a
**single-binding plain `let`** — the other half of the very syntax being piloted.

| | Type-0 (`fn`) | **Ω (proof)** |
|---|---|---|
| **grouped `let`** | ✅ pilot (`slice`, 4 bindings) | ✅ pilot (injectivity, 4+) |
| **single `let`** | (pre-existing paths) | ❌ **never exercised** |

The rollout's very first lemma hit `KernelRejected { TypeMismatch { expected:
Type 0, found: Ω0 } }` on a single-binding `let` in a proof body — **a landed
defect in the one cell the pilot never covered.** The Steward's frame had then
pinned *"a single binding stays a plain `let`"* as a **fixed input**, walking the
build team directly into unexplored ground with a green light behind it.

## The rule

> **Before you accept a pilot, draw the matrix of shapes the ROLLOUT will emit,
> and mark which cells the PILOT actually produced. Any empty cell is unpiloted —
> and a fixed input that lands in one is a pin with no evidence under it.**

The matrix axes are whatever the rollout actually varies. Here it was
**{grouped, single} × {data, proof}** — two axes, four cells, **two covered.**
Nobody drew it, so nobody saw that half the space was untested.

**The pilot cannot tell you this. Only the reviewer can**, and only by asking the
one question a green result never prompts: **"what did this NOT do?"**

## Why it hides so well

- **Every gate passes**, because gates test the artifact that exists, not the
  artifact the rollout will produce.
- **The uncovered cell is often the *simpler* one** (a single binding is the easy
  case), so it reads as obviously-fine and nobody thinks to check it. **The
  degenerate case is not the safe case** — it is frequently a *different code
  path*, and here it was the broken one.
- **A corpus with zero prior instances of the feature** (this catalog had ~27,000
  lines and no local bindings at all) means **nothing else exercises it either.**
  A pilot into virgin territory is the *only* coverage there is — so its holes are
  the system's holes.

## The general form

This is the population error one level up.
[[an-enumeration-needs-a-proven-closure-not-a-better-grep]] says a candidate set
needs a **proven closure**, not a convenient list. **A pilot is a *sample*, and a
sample chosen by convenience is not a population.** If you would not accept "the
sites I happened to read" as an enumeration, **do not accept "the shapes the pilot
happened to emit" as coverage** — and the closure you owe is over the shapes the
*rollout* will emit, not the ones the pilot found handy.

Sibling of [[verify-the-report-is-real-before-explaining-it]] and
[[adding-a-file-to-a-globbed-corpus-trips-oracles-you-did-not-enumerate]]. Same
family: **a green result is only as strong as the space it ranged over — and
nothing in the green tells you what that space was.**
