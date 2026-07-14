---
scope: fleet
audience: (see scope README) — anyone repairing a "fabricated placeholder" class
  of bug, and anyone reviewing the repair; frame authors writing the ACs for one
source: KTR-2 Architect block, 2026-07-14 — a WP whose entire purpose was to stop
  the elaborator fabricating a placeholder SORT shipped a diagnostic that
  fabricated a placeholder ATTRIBUTION. Caught in terminal review, not by QA,
  not by the six discriminators written to catch exactly this class.
---

# A fix can reproduce its own bug **one layer up** — and its own test suite will not see it

## What happened

**KTR-2's whole reason to exist:** `data.rs` mapped an unknown type name to
`Term::ty(Level::Zero)` — it **fabricated a sort** rather than admit it did not
know. The frame said so in capitals. Six discriminating tests were written
specifically to prove no placeholder term can reach the kernel. All six passed.

**The repaired code then did the same thing to source attribution.** When the new
diagnostic could not localize *which* constructor argument the kernel had
rejected, it filled in:

```rust
UniverseArgument { constructor: "<unknown>", name: None, index: 0, span: decl_span }
```

`index: 0` renders as **constructor argument `#1`**. So a malformed declaration
whose *actual* violating field was `C.payload` at index 1 was reported as
**`<unknown>.#1`** — a **false source position, stated with full confidence**,
alongside the kernel's real (correct) levels.

> **The WP that existed to stop the elaborator inventing a placeholder VALUE
> shipped an elaborator that invented a placeholder LOCATION.** *Same reflex —
> "I don't know, so I'll synthesize something plausible and keep going" — one
> abstraction layer up, in the very code written to cure it.*

## Why every gate missed it

- **The six discriminators tested the layer the frame named** (does a bad term
  reach the kernel?). **They were blind to the layer the fix introduced** (does
  the *error* tell the truth?). **A test suite derived from the frame cannot
  catch a defect the fix invents.**
- **QA approved it.** So did I — I had independently verified the *frame's* three
  guardrails (no kernel change, no mass-replace, six discriminators present) and
  reported the candidate "safe to publish."
- **It only fell out of a probe of a case nobody had scoped**: a *malformed
  family* (`data D (x : Int Int)`), where the universe gate fires **before**
  signature checking, so the localizer's own family probe returns `None`. The
  fallback path was **unreachable from every test anyone thought to write.**

## The rule

**When you fix a "the code fabricates X when it doesn't know" bug, the repair
almost always introduces a new place where the code must say "I don't know" —
and the same reflex fires there.** Ask, before review:

1. **Where does my fix have to admit ignorance?** (localization failed, name
   unresolved, span unavailable, level symbolic). **List those paths.**
2. **On each one, do I SYNTHESIZE a value, or do I PROPAGATE the absence?** A
   sentinel (`"<unknown>"`, `0`, `""`, `-1`, `None`-rendered-as-first-element) is
   **a fabricated placeholder wearing different clothes.**
3. **Can the sentinel be RENDERED?** *That is the tell.* `index: 0` was harmless
   until a formatter turned it into `#1`. **A placeholder that reaches a human as
   a confident claim is the bug, restated.**

**⇒ The honest fallback is structural, not cosmetic:** make the unattributed case
**a different shape that CANNOT carry the fields it does not know** — here, drop
back to the raw `KernelRejected { argument, family }` with no constructor/index
field at all — rather than the same shape filled with lies. **Then the type
system, not your care, enforces the honesty.**

## For reviewers and frame authors

**A diagnostics WP needs an AC for the FAILURE of its own diagnostic.** Ours had
six ACs about correct attribution and **none** about what happens when attribution
is impossible. Add one, and make it a **named-variant** assertion (not
`is_err()`) that the output **cannot name a position it does not know**.

**And the probe that finds it is always a malformed input, not a wrong one** — an
input broken *upstream* of the mechanism you are testing, so your locator runs on
rubble. Those cases sit outside every frame's scope by construction. **Write one
anyway.**

Sibling of [[an-enumeration-needs-a-proven-closure-not-a-better-grep]] (there:
the *population* was derived from the symptom; here: the *tests* were derived from
the frame — in both, the reasoning was sound and ran on the wrong universe, and
**came back clean**).
