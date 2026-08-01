---
scope: build
audience: (see scope README)
source: RT-NATIVE-FNSPLIT Boundary A, 2026-07-24 — three instances in one
  candidate (Steward on 92cac774; Architect on e70bb2a5, evt_158mxvyp2qfvn),
  plus a fourth the Steward committed in a WP guard an hour after writing this
  file. Retros (evt_2y7q11tz5jk99) confirmed a SINGLE common cause across all
  three, reached independently by implementer, QA, and leader.
---

# A check that measures a **proxy** passes for the wrong reason — and looks identical to passing for the right one

A check can have both arms, reach its gate, and still be worthless, because it
compares something **correlated with** the property instead of **the property**.
This is not the vacuous-negative-arm failure
([[discriminator-negative-arm-must-be-expressible-and-reaching]]) — the inputs
are producible and the gate is reached. The defect is that **the thing being
measured is not the thing being claimed.**

> A proxy check is green when the property holds **and** green when it does
> not. Its greenness carries no information, and nothing about its source
> reveals that.

## The three instances, all in one candidate

**1. Comparing a value to itself (identity as a proxy for derivation).**

```rust
let mut changed = wrapper.frame;
changed.environment = PersistentNodeId(u32::MAX);
assert_eq!(wrapper.key, wrapper.key);   // tautology: x == x
assert_ne!(wrapper.frame, changed);     // asserts the test's OWN setup
```

The claim was *"a dynamic activation cannot change static helper identity."*
Neither line involves an activation flowing into a key derivation. **A second
defect hid under the first**: the mutation `PersistentNodeId(u32::MAX)` would
be rejected outright as an unclosed activation, so it could never have reached
the mechanism even had the assertion been written correctly — see
[[mutation-proof-injection-point-is-a-reachability-tell]]. Fixing the assertion
alone would have produced a *second* proxy.

**2. Counting only the labelled members (a label as a proxy for the relation).**
The `CompletedTail` validator counted incoming edges **already labelled**
`CompleteProducerTail`. An incoming edge of any other kind was invisible to a
check whose claim was about *incoming edges*. One correctly-labelled edge
existing does not exclude bypasses.

**3. Cardinality as a proxy for set equality.** Reachability compared
`reachable.len()` with `nodes.len()`. Replacing the sole entry with an
out-of-range `StaticNodeId(u32::MAX)` made the reachable set
`{bad-id, Terminal, TrapTerminal}` — **same count, wrong set** — so validation
returned `Ok(())` while the real node was unreachable. The count balanced.

**4. Path overlap as a proxy for content replay (the Steward, one hour after
writing this file).** A WP guard said: *"if `git diff --stat origin/main...<B
tip>` still shows Boundary A's files, the re-anchor was done wrong."* But
Boundary B's whole job was to make A's plan load-bearing, so it **must** edit
A's files. The guard was **unsatisfiable by construction** — a correct B could
never pass it. The property was *"A's content was not re-introduced"*; the
evidence that settled it was content, not paths: A's signature tokens appearing
**exactly once** (`B=1, main=1`) and A's file at `main`'s size plus B's net
delta rather than doubled.

**That fourth instance is the most useful one in this file**: the lesson was
already written down and indexed, and it did not fire. Recording a discipline
does not install it — only a step in a checklist does.

## The shared shape

Each check measured a quantity that **agrees with the property on the cases the
author had in mind** and diverges elsewhere: self-identity for derivation,
label-membership for edge-membership, cardinality for set equality. In every
case the honest check was available and barely more expensive.

**The tell is grammatical.** Write the claim as a sentence and check that the
code's operands are its nouns. *"A dynamic activation cannot change static
helper identity"* has **two** operands — an activation and a derived identity.
Code containing neither an activation nor a derivation is not testing that
sentence, whatever it is named.

## The one test that catches all of them — build the discriminating pair

The Boundary A ring converged on this independently, from three seats, and it is
sharper than any advice below:

> **Before accepting a check, construct a case that PRESERVES the proxy and
> VIOLATES the property.** If you can build one, the check is broken. If you
> genuinely cannot, the proxy is sound *for that claim*.

It falls out immediately in every instance: a second frame leaves `x == x` true
(1); an unlabelled incoming edge leaves the label-count true (2); an
out-of-range root leaves the cardinality true (3); an incremental edit to A's
file leaves the path present (4). **Each pair takes under a minute to imagine
and none of them were.**

**Arithmetic corroborates but cannot close.** A census that comes out affine,
a count that balances, a size that matches — these are consistent with the
property and also consistent with its violation. Do not let a satisfying number
substitute for the pair.

⇒ **Frame authors and leaders: this is a plan-time obligation, not a review-time
one.** The Boundary A leader's carry: *pair every structural claim with its
exact counterexample class — alternate edge, alternate activation, alternate
entry — before implementation begins.* Naming the counterexample class in the
frame is what stops a proxy-first test from being authored at all.

## How to apply

- **Recompute through the production function, from real inputs.** The repair
  that worked: `helper_key_for_activation(id, frame_a)` vs
  `helper_key_for_activation(id, frame_b)` — same static node, **two real
  closed frames**, key derived by the real function each time, then asserted to
  collapse. Add a field to the key and it fails. That is the property.
- **Take the operands from the real population, not from sentinels.** Instance 1
  needed `other_activation` to be a genuine closed frame *from another node in
  the plan*; a fabricated one was rejected before it could discriminate.
- **Prefer set equality to existence and to counting.** "The correct edge
  exists" and "the counts match" are both proxies. "The incoming set is exactly
  `{X}`" and "the reachable set equals the closed node-ID set" are properties.
- **Ask what else would make this green.** If you can name any state where the
  check passes and the claim is false, you have a proxy. This is the cheapest
  step and it caught all three.
- **Best outcome: move it into production.** Instance 1's real fix was
  `validate()` rejecting *every* node whose activation-derived key differs from
  its fixed key — converting a tested property into a structural one, the same
  move as [[a-claimed-executable-inventory-needs-a-reversible-deletion-proof]].
- **Reviewers: verify the mechanism, not the description.** All three passed
  review-by-reading, because each *reads* like the claim it names. Run the
  operands, not the prose.
