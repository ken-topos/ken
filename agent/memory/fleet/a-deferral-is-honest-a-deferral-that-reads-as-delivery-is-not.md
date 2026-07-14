---
scope: fleet
audience: (see scope README) — anyone deferring a capability, or reading a WP that merged with one deferred
source: V3 FO/Z3 finding + Pat's ruling, 2026-07-14
---

# A deferral is honest. A deferral that reads as a delivery is not.

**V3 (the prover) merged.** Its acceptance suite passes. Its certificates really
are kernel-checked. Its classifier really does route D/FO/HO. **Every one of
those facts is true — and the verification thesis was not delivered.**

`attempt_fo`, the first-order route, in full:

```rust
fn attempt_fo(...) -> Verdict {
    // The Kripke embedding for quantified FO goals is [placeholder — reifies in V4].
    attempt_ipc(env, ctx, phi, phi_closed)
}
```

**It delegates to the propositional tactic.** FO obligations beyond the
intro skeleton return `unknown`. **No test fails — because `unknown` is a LEGAL,
HONEST answer.** A WP-level "merged" roll-up reports the thesis as done.

## The three things that made it invisible

1. **The gap is in what a route can DISCHARGE, not in what it RETURNS.** Every
   value the code produces is correct. **The shortfall is in reach, and reach is
   not a value any test inspects.**
2. **`unknown` is a legal verdict.** A failure mode that is *designed to be
   acceptable* cannot be detected by asking "did anything fail?"
3. **The WP status said `merged`.** Which was **true**.

## ★ The rule

**When a WP merges with a headline deliverable deferred, the deferral must be
written into the SCOPE, not just into a source comment.** A `[placeholder]` in
the code is a note to the next implementer. **It is not a disclosure to the
reader of the spec, the tracker, or the gate.**

**Gate status must be grounded in the gate's own text, cross-referenced against
what LANDED — never rolled up from WP status.** *V3 is `merged` and G3 is `not
met`, simultaneously, and both are correct.* **A roll-up cannot express that, and
so it lies.**

## And the corollary that actually bit

**The deferral was CORRECT** — Pat ruled it stays deferred, because the solver is
an unproven *performance* hypothesis and the gap costs ~nothing today (only 2 of
31 catalog packages use `requires`/`ensures`). **The operator's judgment was
fine. The DOCUMENTATION was not:** the spec advertised an FO capability the code
did not have.

**⇒ The cost of a deferral is not the missing feature. It is the prose that still
promises it.** Deferring is cheap and often right. **Leaving the promise standing
is what turns a known limitation into a latent overclaim** — and the person it
deceives is the one who resumes cold in three weeks and *builds on the promise*.

## How to catch it

**Read the gate's own text, then try to WRITE the sentence that says we met it.**
If the honest sentence needs a "…except when…", **that exception IS the scope
boundary, and it must appear in the spec.** Same instrument as
[[a-dependency-is-met-when-you-can-write-the-obligation]] and
[[never-pin-a-shape-that-cannot-state-its-own-contract]] — *try to write the
claim down and see whether it survives contact with the code.*

**And when you report one of these: expect to be wrong in the details.** I
over-claimed three times (scope of the `unknown`, which WP owned the gap, whether
an embedding must strictly precede a solver) and was corrected by the Verify and
Spec leads on all three. **The finding survived; my framing didn't.** Publish the
correction as loudly as the finding — sibling of
[[verify-the-guard-before-acting-on-it]].
