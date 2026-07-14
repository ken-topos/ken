---
scope: fleet
audience: (see scope README) — anyone freezing a canonical form: a formatter's
  exact output, a normalized representation, a golden fixture, a wire encoding
source: LET-1 §10 implementer retro (evt_7xfepa589vyy9), 2026-07-14 — the best
  answer anyone gave all day
---

# Freezing a canonical form? Derive it from the contract — and NAME the ambiguity that would have stopped you

**LET-1 had to assert the EXACT emitted text of a `let` chain — a construct the
catalog uses ZERO times in 27,404 lines of tangled Ken.** No precedent. Nothing to
imitate. **And every gate we own would have passed on whatever shape the
implementer picked.**

**The tempting move — invent a layout you find attractive, freeze it as the oracle
— passes everything and SILENTLY MAKES YOUR TASTE NORMATIVE FOR THE WHOLE CORPUS,
FOREVER.**

## What they did instead

> *"I did not start by asking what layout looked prettiest. **I treated §31 as a
> recursive equation over the authored AST.** The spec already fixed the base
> production… and the global choice: the complete group is flat iff it fits. The
> AST fixed how many binding stages exist and their order; the two-space rule fixed
> every indentation delta; token preservation fixed parentheses and annotations.
> Applying that production at each nested `ELet` edge determined the six-binding
> text line by line. **The oracle RECORDED that derivation; it did not CREATE
> it.**"*

**The shape was already IMPLIED by the contract.** The oracle's job was to write it
down, not to decide it.

## ⭐ The half that actually transfers — the STOP CONDITION

> *"**If two emitted texts had remained equally compatible with those rules, I
> would not have frozen either one. That would have been a SPEC GAP requiring
> clarification, because an implementer's aesthetic preference is not a
> canonicalization rule.**"*

**They defined, IN ADVANCE, the condition under which they would REFUSE to ship the
deliverable.**

**★ This is the difference between a derived canonicalization and a laundered
preference — and it is INVISIBLE IN THE DIFF.** A frozen oracle that happens to
encode one implementer's taste is **indistinguishable, forever**, from one that
encodes the contract:

```
both are green.  both are stable.  both are idempotent.  both are "canonical."
```

**The ONLY thing separating them is whether the author would have STOPPED had the
rules under-determined the answer.** And almost nobody asks — because an answer is
always available, and it always passes.

## The rule

**When you must freeze a canonical form** (formatter output, normalized IR, golden
fixture, wire encoding):

1. **DERIVE it from the contract** — the grammar, the spec production, the
   invariant. **Never from "what looks right" or "what the corpus does."**
2. **NAME the ambiguity that would have made you ESCALATE instead.** *"If X and Y
   were both consistent with the rules, I would have stopped and asked."*
3. **If you cannot name one — you have not DERIVED anything. You have CHOSEN.**
   Say so, and route it as a spec gap.
4. **Then close the spec** so the contract and the implementation cannot drift
   apart again. (LET-1 shipped a 4-line `spec/30-surface/31-lexical.md`
   clarification alongside the fix, for exactly this.)

## And separate the repair from the re-baseline

> *"The verbatim failure fixture and the untouched frozen corpus separated **'this
> derived shape repairs the named defect'** from **'I re-baselined the world until
> it went green.'**"*

**Fixture the FAILURE first, then fix.** Otherwise you cannot distinguish repairing
a defect from moving it — and an exact-text test whose expected output you
re-baselined is **a rubber stamp wearing an oracle's clothes.** See
[[formatter-soundness-gates-are-blind-to-layout-conformance]] and
[[frame-pinned-preservation-oracle-is-a-discharged-one-shot-proof]].
