---
scope: fleet
audience: (see scope README) — anyone who decides WHICH GATE a WP needs; every
  leader arguing a review can be skipped; anyone shipping a CONDITIONAL law
source: CC6b, 2026-07-14 — the gate-skip argument was sound in its conclusion
  and wrong in its reason, which is the half that generalizes
---

# A VACUOUS law has ZERO trust delta — so a gate keyed on "trust movement" is blind to the thing most likely to be wrong

**CC6b's frame said: Architect review is required only on trust movement.** The
ring measured **zero `trusted_base()` delta**, and argued — correctly, as it
happened — that no Architect review was needed.

**The conclusion was right. The REASON does not hold, and the reason is what
gets reused.**

## Why

CC6b's whole content was a **conditional law**, rewritten after the original was
proved false:

```ken
path_parse_render_valid : path_valid p ≡ True → path_parse (path_render p) ≡ p
```

**A conditional law whose hypothesis nothing satisfies is TRUE and WORTHLESS.**
And such a law:

```
type-checks                      ✅
adds no Axiom, no postulate      ✅
has ZERO trusted_base() delta    ✅   ← the gate's trigger never fires
is completely hollow             ⛔
```

> **⇒ Vacuity and trust-surface growth are ORTHOGONAL. A gate that triggers on
> trust movement cannot see vacuity AT ALL.** The hollow WP is *exactly* the one
> that sails through, because it adds nothing — and adding nothing is what the
> gate is looking for.

## What actually saved it

**The reaching lemma** — and it must be **proved**, not postulated:

```ken
lemma path_parse_valid (raw : Bytes) : Eq Bool (path_valid (path_parse raw)) True
  = path_split_preserves_valid (bytes_to_list raw) …   -- structural induction
```

**∀ raw bytes, the parser's output satisfies the hypothesis.** *Now* the
conditional law quantifies over a set that is inhabited by everything real, and
composing the two yields the unconditional identity anyone actually wanted.

## The rules

1. **Ship a conditional law? SHIP ITS REACHING LEMMA IN THE SAME WP.** *"For all
   `p` with `P p` …"* is half a deliverable until you prove **something reaches
   `P`** — and prove it, never postulate it. **An AC that says "the law
   elaborates" is not an AC.** *We swapped a FALSE law for a TRUE one; the trap
   we were one lemma away from was swapping it for a VACUOUS one.*
2. **When you argue a gate can be SKIPPED, state the risk the gate exists to
   catch — then show THAT risk is absent.** "The trigger didn't fire" is not the
   same claim, and the gap between them is where a hollow WP lives. **A sound
   conclusion from a wrong reason will be reused, and the next time the reason
   will carry it somewhere false.**
3. **Frame authors: scope a gate by the RISK, not by a proxy for it.** CC6b's
   frame keyed the Architect on *trust movement*, when the live risk after the
   mid-flight ruling was *vacuity*. **The proxy was measurable and the risk was
   not — which is precisely why the proxy got written down.**

Sibling of [[discriminating-axis-vacuous-until-capability-lands]] and
[[verified-showcase-predicate-must-be-defined-not-postulated]]. Same family as
[[deriving-from-the-contract-cannot-detect-a-defective-contract]]: **every gate
was green and the gates were measuring the wrong thing.**
