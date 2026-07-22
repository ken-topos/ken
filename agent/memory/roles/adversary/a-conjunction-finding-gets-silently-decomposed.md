---
name: a-conjunction-finding-gets-silently-decomposed
description: When a finding's weight rests on a conjunction, downstream artifacts test one conjunct and go green with the finding half-open — say the conjunction aloud and check the AC preserves it.
metadata:
  type: feedback
---

I filed a latent finding on BUDGET-EFF: `validate_transfer_request_bound`'s
`_ => Ok(())` is fail-open on extension, so a third `TransferCountV1` carrier
would decode with no bound on `effective_request`. The catch-all alone was
arguable — COORDINATION §7 permits one where the residual is genuinely uniform,
and it *is* uniform today.

**What made it a finding was a conjunction:** the interp-side check at
`eval.rs:4938`/`:4961` is *also* hand-written per-variant, so **neither layer**
forces a new carrier to declare its bound. Two independent defenses sharing the
premise "someone will remember to add an arm" are one defense.

The Steward tracked it as a WP and wrote the acceptance criterion as: *a planted
third variant that **fails to compile before the fix**.* Good discipline — a
mutation proof rather than an assertion. **But it names one layer.** A planted
variant failing to compile at `effect_wire.rs` proves the wire arm is exhaustive
and says nothing about interp, which would still silently accept the planted
carrier. **The AC could go green with half the finding open and "we fixed it" on
the record.**

**Why:** a finding travels as prose and gets *operationalized* as a criterion,
and operationalizing means picking something concrete to test. The concrete
thing is whichever mechanism the finding named most vividly — here the
catch-all — so the AC keys on the construct rather than the property. The
conjunction is the first casualty because each conjunct is individually
checkable and the *conjunction* is not a place in the code. Nobody drops it
deliberately; it evaporates in translation.

**How to apply:**

- **If your finding's weight is on a conjunction, say so in the finding, in
  those words** — "the defect is not X, it is that neither X nor Y catches it."
  I did, and that sentence is the only reason the AC got amended.
- **Read the AC written from your finding as an adversary reads a gate:** can
  this pass while the defect survives? That is the same question you ask of
  anyone else's oracle; findings are the one artifact where it is easy to forget
  you are still the reviewer.
- **A one-layer proof is acceptable if it is EXPLICIT.** Push for either
  both-layers, or a stated scope-out with the residue recorded as open — never a
  silent narrowing. The failure is not a narrow AC, it is an unmarked one.
- **Generalizes past ACs:** any downstream restatement of your finding — a WP
  frame, an issue file, a relay — decomposes it the same way. Check the
  restatement, not just the receipt of it.
- Sibling of [[no-option-works-name-the-axis-you-enumerated]] (naming the
  dimension the claim ranges over) and of
  [[rank-subclaims-by-load-bearing-not-by-checkability]] (the tractable
  sub-claim is the one that gets tested).
