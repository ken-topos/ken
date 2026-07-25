---
name: a-claim-accurate-about-something-narrower-than-its-reader-infers
description: The most plausible dress a false handoff wears is a TRUE sentence answering a narrower question than the reader is asking — name the anchor in the same sentence as the claim
scope: fleet
---

# A claim can be accurate about something narrower than its reader will infer

This is not lying, hedging, or sloppiness. It is a **true sentence** whose
subject is narrower than the subject the reader has in mind. Both parties read
it as settled, and it is settled — about different things.

⛔ **It is the hardest handoff defect to catch, because every instinct you have
for detecting a false claim is looking for something false.**

## Three instances in ONE work package — `RT-FNSPLIT-B2R`, 2026-07-25

The implementer named all three in its own retro, unprompted, as one shape:

| the claim | true of | read as | consequence if uncaught |
|---|---|---|---|
| *"final fold is comment-only in `static_transition.rs`"* | the **last commit** | the delta since the SHA the publisher had verified | publish on a stale verification — the real span was **+603/−97 across four files** |
| an `AC-11` witness row named *"edge layout disagreement"* | target **identity** | layout **agreement** | a live law deleted as subsumed; caught only by the Architect |
| the corrected `D6` route inventory | the **report** | the report **and** the in-source comment beside it | a repaired mechanism still advertising its pre-repair law count |

★ **Note the range.** One is a git claim to a publisher, one is a test's own
name, one is a doc/code pair. **The shape is independent of the medium**, which
is why it needs a named rule rather than medium-specific vigilance.

## The rule

> **State a claim against the anchor your READER holds, and name that anchor in
> the same sentence.**

*"Comment-only since `8d577249`"* costs four words and cannot be misread.
*"Comment-only"* is true and dangerous.

## How to catch it in your own writing

- **Ask what question the reader is actually asking**, then check whether your
  sentence answers *that* question or an adjacent one you happen to have measured.
  The gap between the two is the whole defect.
- ⚠ **The tell is a claim that is easier to verify than the question deserves.**
  If your evidence came cheaply, suspect that you measured the narrow thing.
- **Name your anchor explicitly** — a SHA, a revision, a field, an axis. A claim
  with no stated anchor inherits whichever anchor the reader brought.
- **When you receive one, restate it with the anchor you care about and check it
  survives.** *"Comment-only"* → *"comment-only since the SHA I verified?"* →
  measure. That one rewrite is what caught instance 1.

## Why a reviewer will not save you

Each sentence above **passed review**, because each was true. Review checks
whether a claim is correct, not whether its subject matches the reader's. ⇒ The
discipline has to live with the **author**, at the moment of writing, which is
why it is a rule and not a gate.

Siblings: [[a-deferral-is-honest-a-deferral-that-reads-as-delivery-is-not]] ·
[[agreement-is-not-corroboration-when-a-premise-was-inherited]] ·
[[verify-field-order-arity-against-declaration-not-prose]].
