---
scope: teams/runtime
audience: runtime-leader, runtime-implementer, runtime-qa
source: Architect ruling evt_842spc7t6js1 on RT-FNSPLIT-B2F hard-stop #9
  (2026-07-25); the standing correction carried on
  docs/program/issues/RT-FNSPLIT-B2O.md, restated because successive drafts of
  that same file had it backwards
related: a-structural-pin-that-enumerates-spellings-is-not-a-proof-of-the-property
---

# `ClosureBody` is a body's RETURN SUCCESSOR, not a function-unit head

When you need the set of **function-unit heads** in the static-transition plane,
the ruled answer is:

> **`plan.entries` ∪ every `EdgeKind::StaticBody` *target*.**

**`TransitionKind::ClosureBody` is NOT a head.** It is the transition a body
takes on the way **back out** — a return successor. Any prose describing the unit
heads as *"the root plus the `ClosureBody` heads"* is **wrong**, and that
specific error has appeared in successive drafts of the very issue file that
warns against it.

## Why the mistake is attractive

The name reads like *"the head of a closure body,"* and a closure body genuinely
*is* a unit. Both halves of that intuition are true; the conclusion is still
false, because the thing named `ClosureBody` sits on the **wrong end** of the
body it names. Nothing about the identifier tells you which end, so the reflex
that spells the seed set from memory gets it backwards roughly as often as not.

## What to do

- **Re-derive the seed set from the ruling or the code, never from prose** —
  including prose in a tracker file, a frame, or an earlier message of your own.
  This is a domain fact whose *natural-language rendering* is the unreliable
  part.
- **The two `EdgeKind::StaticBody` producers are the population to check** when
  you need every closure form covered. A fixture exercising only one of them
  leaves the other's behaviour unmeasured — that gap has been measured live,
  with both untested forms turning out correct, which is exactly why it read as
  safe.
- **A seed set is a population, so pin it as one.** Enumerating the spellings you
  remember is the defect this chain has paid for repeatedly; derive the set and
  assert it.
