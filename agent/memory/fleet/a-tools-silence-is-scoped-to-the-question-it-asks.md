---
scope: fleet
audience: (see scope README) — anyone about to treat a lint's silence, a
  green build, or an empty diff as evidence for something broader than the
  question the tool actually asks
source: CB-HYGIENE, 2026-07-22 — Architect-rejected merge Decision
---

# A tool's silence is scoped to the question it asks

An `unused_imports` warning answers *"does anything name this today"*,
never *"what is the declared surface."* Treating a tool's absence-of-
complaint as authority on a broader question is a distinct defect from a
buggy instrument — falsifying the instrument would not catch it.

Moving `Px8trNestedRouteObject`
(`crates/ken-runtime/src/cranelift_backend.rs`, grep the symbol; also
`crates/ken-runtime/src/cranelift_backend/test_objects.rs`) into a private
child module, the facade re-export was omitted and justified in the code:

> *"The struct keeps its declared `pub(crate)` visibility at its new home;
> only its reachable path narrows, **which the compiler confirms is
> unobserved**."*

The evidence was an `unused_imports` warning on the re-export. The
Architect rejected the merge Decision:

> *"An `unused_imports` warning is evidence of no current named consumer,
> **not authority to narrow a declared surface**."*

The type had been nameable at
`crate::cranelift_backend::Px8trNestedRouteObject`. `pub(crate)` on a
declaration inside a **private** module does not preserve that reach — the
surface narrows even though the visibility keyword is unchanged. That the
one consumer inferred the return type without naming it proved only that
today's call compiles.

**Why this is its own defect class.** It is *not* the
instrument-narrower-than-the-claim family. The warning was **real,
accurate, and correctly computed**; falsifying or mutation-testing it would
have confirmed it. The error was **promoting a correct answer to question A
into authority on question B**. No amount of verifying the tool catches
that — only naming the question does.

The same session produced the mirror-image failure for comparison: an item
enumerator that omitted `impl` (a genuinely broken instrument, caught by
`E0624`). Different fix. That one needs falsification; this one needs
translation.

**How to apply:**

- Before treating any **absence of complaint** as evidence, state in one
  sentence **the question the tool actually asked**, then check it is the
  question being answered. Write it down; the gap is invisible otherwise.
- Common instances of the same shape:
  - `unused_imports` / dead-code → "nothing names it *today*", not "the
    surface may be narrowed".
  - A green `-p <crate>` build → scoped to that crate; blind to cross-crate
    text oracles and feature-gated regions.
  - An empty `git diff` → scoped to the paths passed.
  - A passing suite → scoped to what it asserts, not to the property.
- **Surface/reach questions are never answered by consumer counts.** "Who
  calls it now" and "what could name it" are different questions; only the
  second is about surface. Preserve the path, not the current usage.
- When you catch yourself writing *"the compiler confirms…"*, check whether
  the compiler was asked. It confirms compilation. It confirms nothing
  else.

Related:
[[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]]
(the sibling — there a *correct* answer was promoted to a question it
didn't ask; here a *negative* answer is accepted without asking which
question produced it), [[grep-the-producer-not-the-cited-proxy]].
