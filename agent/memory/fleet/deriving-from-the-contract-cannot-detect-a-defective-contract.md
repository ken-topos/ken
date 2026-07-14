---
scope: fleet
audience: (see scope README) — anyone freezing an oracle for a HUMAN-VISIBLE
  property (formatter output, error-message text, rendered docs, generated code,
  a UI); anyone whose acceptance criteria are all mechanical
source: LET-1 → LET-1b, 2026-07-14 — the fix shipped, every gate was green, and
  the output was still bad. Sibling and CORRECTION to
  [[freezing-a-canonical-form-name-the-ambiguity-that-would-have-stopped-you]]
---

# Deriving from the contract cannot detect a DEFECTIVE contract

**LET-1's implementer did everything right.** They refused to invent a pretty
layout; they treated spec §31 as a recursive equation over the AST; they froze an
exact-text oracle that *recorded* the derivation rather than creating it. **We
promoted it as the best artifact of the day.** It is still the right discipline.

**And the emitted layout was bad anyway** — a flat three-binding `let` chain
rendered as a right-nested **staircase**, each RHS forced onto its own line even
when the whole binding fit in 58 columns:

```
const chars : List Char =
  let left_chars : List Char =
    string_to_list_char left
  in
    let right_chars : List Char =
      string_to_list_char right
    in
      let joined_chars : List Char =
        append Char left_chars right_chars
      in
        joined_chars          ← 8 columns right of the binding it belongs to
```

**Because `spec/30-surface/31-lexical.md:216-227` SAYS SO.** The derivation was
faithful. **The contract was wrong.**

## ★ The gap in the stop condition

The celebrated stop condition was:

> *"If two emitted texts had remained equally compatible with those rules, I
> would not have frozen either one — that would be a spec gap."*

**That guards against AMBIGUITY** — against laundering your taste into a
canonical form. **It cannot fire here, because §31 determines exactly ONE text.**
There was no fork to stop at.

**There is a third case, and neither the author nor the reviewer had a rule for
it:**

| | the rules say | the discipline |
|---|---|---|
| **1** | nothing | escalate — spec gap |
| **2** | two things | **STOP** — spec gap *(the celebrated rule)* |
| **3** | **exactly one thing, and it is BAD** | **← nothing catches this** |

> **⇒ Deriving from the contract protects you from laundering your PREFERENCE.
> It does NOT protect you from a DEFECTIVE CONTRACT.**

**And it is worse than neutral: a faithful derivation transmits the defect WITH
FULL AUTHORITY, then freezes it in an exact-text oracle — which turns the bug
into EVIDENCE FOR ITSELF and makes it harder to fix.** The next person to
question the layout is arguing against a green, derived, spec-cited test.

## ★★ The tell was in the WP's own title

**LET-1 was called "readable let-chain layout."** Its acceptance criteria
asserted:

```
exact emitted text · AST preservation · token preservation
idempotence · ≤80 columns · zero trusted_base() delta
```

**Six gates. All green. NOT ONE asked whether the output was READABLE.**

**The exact-text oracle FEELS like it closes that gap. It does not.** It pins
**what** the output *is*; it never asks whether the output is **good** — and its
expected value was derived from the same defective production, so **it agrees
with the defect by construction.**

> **We replaced a stability gate with a different stability gate and called it a
> quality gate.**

This is [[formatter-soundness-gates-are-blind-to-layout-conformance]] one level
deeper — *knowing* that stability ≠ quality did not save us, because the remedy
we reached for (freeze the exact text) **is also a stability gate.**

## The rule

**When the deliverable is a HUMAN-VISIBLE artifact, one acceptance criterion
must be dischargeable only by a human-equivalent READER — and it must be
adversarial to the mechanism:**

- **AC: paste the rendered output VERBATIM into the handoff, and state the
  property in reader's terms.** *"The final body is not indented deeper than the
  first binding."* Not *"the oracle passes."*
- **A mechanical oracle CANNOT discharge it.** If your AC can be satisfied by a
  green test, it is not this AC.
- **Ask the question in the WP's TITLE.** If the WP promises *readable*, *clear*,
  *ergonomic*, *fast*, *simple* — **name the gate that measures that exact
  adjective.** If every gate measures something else, you will ship the adjective
  unverified and the gates will all be green.

## And for the SPEC side

**A spec production that has never been exercised is a HYPOTHESIS, not a
contract.** §31's `let` rule sat unexercised because the catalog had **zero**
`let` bindings in 27,404 lines. **The first real corpus of `let` chains was the
thing that revealed it** — LET-2's own teaching guide, which the formatter
promptly staircased.

> **⇒ The first time a production is actually exercised, RE-READ IT AGAINST THE
> OUTPUT.** Not against the code — against the *output*, with your eyes. A rule
> nobody has run is a rule nobody has checked, no matter how long it has been
> "in the spec."
