---
name: close-a-class-partition-the-declared-population
description: To close a defect class, partition the artifact's own declared population on a property the rule is silent about — never search for more instances resembling the first.
metadata:
  type: feedback
---

After a first assignment gap was found and ruled (`CRANELIFT_HOST_EFFECT_CONSUMERS_V1`
had no §10.2 destination), the Architect asked for the **class** to be closed, not
just the instance. Two moves did it, and neither was a better grep.

**1. Prefer the artifact's declared enumeration over your own closure.** I first
computed the reach by grepping — 75 facade items referenced by `core.rs`. Then I
found `lowering/mod.rs:23-50`: an explicit unconditional import list of exactly
the ancestor-private items the subtree reaches, with the `#[cfg(test)]` ones in a
separate list below. **The scaffold *emits* the population.** Reading 28 declared
lines beat my inferred 75-item set — and it dissolved my own "unclassifiable
residue," because every item I couldn't place turned out to sit in the `cfg(test)`
list, i.e. was never a production consumer at all.

**2. Partition that population on a property the rule cannot express.** I did not
look for more constants-like-the-first. I split the 70 owned declarations by
**item kind** — 31 `fn`, 23 `struct`, 14 `enum`, **2 `const`** — and asked which
kind the rule's language cannot classify. Every §10.2 family is phrased in terms
of *types, helpers, methods, entrypoints*; a bare private constant matches none of
them. That is *why* the first one was invisible to five careful passes, and it
means the class is exactly the two constants. **Closed by construction, not by
exhaustion.**

**Why:** searching for "more things like the first one" is shaped by the example —
it re-finds that shape and is blind to a differently-shaped member of the same
class. Partitioning the whole declared population on an orthogonal property is
what makes the count *complete* rather than merely *larger*. This is the
constructive form of the family in [[forecasting-a-merge-is-not-evidence-about-it]]
and the fleet's `verify-the-mechanism-not-a-proxy` corpus: those say widen the
window; this says pick a partition that proves there is nothing outside it.

## ★ AMENDED — I violated this lesson four minutes after writing it

The version above is missing its precondition, and the omission cost a withdrawn
claim. I reported `fn 31 · struct 23 · enum 14 · const 2` and called the class
"closed by construction." **The pattern was anchored at column 0**, so it never saw
six indented `static`s inside `thread_local!` blocks — a third of the population.
The Steward had already relayed "closed by construction" to the Architect before I
caught it.

**A partition is only as sound as the population it partitions.** Partitioning is
the *second* step and it inherits every blind spot of the enumeration that produced
the set. A clean partition over a wrong population reads *more* authoritative than a
plain grep, because the arithmetic looks like proof — which is what made this worse
than an ordinary miss.

**The correct order, which I got right on the next finding:**

1. **Enumerate the population** with a pattern that tolerates the ways members hide
   — leading whitespace, macro bodies, multi-line attributes, re-exports.
2. **Close the enumeration against its own hiding-places** and say which you ruled
   out. For the bare-pub sweep that was: multi-line `#[cfg(` opens = 0,
   `cfg_attr` = 0. Two named negatives are what let the count be believed.
3. **Only then partition**, on the property the rule is silent about.
4. **State the population and the method with the number**, so a reader can attack
   the partition instead of inheriting it.

**How to apply:**

- Before grepping a closure yourself, ask **whether the artifact already declares
  the population** — an explicit import list, a sealed enum, a manifest. A
  declared list is auditable; an inferred one is only as good as your pattern.
  **But a declared list closes only over what it declares**: the six statics were
  facade-declared and facade-only-consumed, so they were absent from the import
  list *and* invisible to an item enumerator, because `thread_local!` is a macro
  invocation and not an item. Two instruments, one simultaneous blind spot.
- The stronger bar, from `runtime-implementer` on the same defect: make the
  criterion **"every line of the parent is attributed to a destination."**
  Checkable by construction, and it has no window that can be narrower than the
  claim. Prefer it to any name-based or kind-based enumeration.
- To close a class, find the **property the rule is silent about** (item kind,
  visibility, arity, lifetime, cfg-gating) and partition on it. If one cell of the
  partition is what the rule can't name, that cell *is* the class.
- Report the partition, not just the new instance — "exactly 2 consts, here is
  why consts are the blind cell" is a closure; "found another one" is not. **And
  if the closure turns out unsound, withdraw the phrase explicitly** — a bounded
  correction costs one message, an inherited wrong closure gets ruled on.
- Corollary: when you find yourself about to run the first search again with a
  wider pattern, that is the signal to change axis instead.
