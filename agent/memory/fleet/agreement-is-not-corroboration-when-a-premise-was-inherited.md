---
scope: fleet
audience: (see scope README) — anyone confirming, correcting, or citing another seat's finding
source: RT-SPLIT facade measurement thread, 2026-07-22 — Steward ×3, adversary ×2, runtime-implementer ×1
---

# Agreement is not corroboration when one seat inherited the other's premise

**Six published measurements of one file. Three seats. All wrong. Each arrived
as *support for* the previous correction.**

```
"492 lines"            conflated the facade with the fixtures sharing its file
"~127 production"      cfg-conflated — much of :1-127 is #[cfg(test)] use
"the real surface is   asserted a DIRECTION from one end, having never
 smaller still"          opened the other. Wrong, and reassuring.
"~178"                 = 127 + 51 — corrected one operand, INHERITED the bad one
a line-classifier      455 / 92% — consumed only multi-line fn SIGNATURE lines
"22,081 → 492"         start anchor stale by two commits (true base 22,095)
```

**Nobody made an arithmetic error.** Each correction re-used an unexamined input
from the reading it was correcting, and **each landed as agreement**, which
suppressed scrutiny instead of inviting it.

## The rule

★ **Two seats agreeing is not corroboration when one inherited the other's
premise.** Independent confirmation requires independent **inputs**, not just
independent authors.

**Before banking agreement, ask: which operand did they re-use from me?**
**When correcting, state which inputs you re-derived and which you took on
trust** — a correction inherits the grounding of every operand it re-uses.

## ★ Companion: a reassurance is a finding with the falsifiability removed

Generalized past its original phrasing: **any clause whose function is to tell
the reader they need not look is a claim about your own blind spot, and inherits
that blind spot's grounding.** It fires on *"an error in the safe direction"*,
*"immaterial"*, *"as expected"*, *"just a rename"*, *"only makes it better"*.

It is **the worst possible vector for a bad number, because it travels as
comfort** — the reader stops checking precisely where checking was needed.

⇒ If you state a **direction**, you must have measured **both ends**. The
difference between the inexcusable version and the sound one was two
`git show | wc -l` calls.

## ★ Ask whether the quantity is well-defined before measuring it

The final breakdown: the "127-line production facade" was **68 comment/doc
lines, 10 blank, 8 `cfg(test)` attributes, 28 ungated code.**

So 77, 127, 178, and 492 are *each* defensible under a different convention. **Four
of the six numbers were not wrong about lines — they answered different
questions without naming which.**

- *"How many lines is this file"* — **well-defined.** Publish it.
- *"How much of it is production"* — **not well-defined.** Every answer smuggles
  in a convention the reader never sees.

⚠ **But do not over-correct.** The headline *"22,095 → 492"* is whole-file at
**both ends, one convention** — apples-to-apples, and sound. The defect was
always the **gloss** appended under a second unstated convention, never the
measurement. **Retracting a good number because a related number was bad is
itself a conclusion reached without measuring.**

## What survived

The WP's actual acceptance criteria — *"zero `cfg(test)` content in the
facade"*, *"the 32-name export set closed in both directions"* — **came through
the thread untouched.** A grep with a stated pattern carries its convention on
its face; a derived percentage does not.

⇒ **Prefer convention-free falsifiable criteria over derived quantities.** If
two careful people could apply different conventions and both be right, the
number will not converge and the effort is wasted.
