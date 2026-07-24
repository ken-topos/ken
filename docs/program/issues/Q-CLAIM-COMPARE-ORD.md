---
id: Q-CLAIM-COMPARE-ORD
title: "claim-loss in list_instance_routes... (compare_ord) — both routing claims dropped, replacement only instantiates Bool"
status: merged
owner: runtime
size: XS
gate: none
depends_on: []
blocks: []
github: null
origin: split out of Q-CLAIM-CLOSURE (2026-07-22) — the third multi-claim test in that finding's table is in a file outside Q-CLAIM-CLOSURE's scope
---

**Split from `Q-CLAIM-CLOSURE`, not a new discovery.** That WP's generator
finding named three multi-claim tests; two are outside its file scope. This is
one of them, carved out so it is **not silently dropped** — which would be this
exact defect class one level up.

## The defect (from Q-CLAIM-CLOSURE's own analysis)

`list_instance_routes_the_canonical_compare_into_raw_list_compare` in
`crates/ken-elaborator/tests/compare_ord_lexicographic_acceptance.rs` carried
**two** claims; the Q-RESIDUE rework **dropped both**, and the replacement
**instantiates only `Bool`** — so the routing the test name asserts (the `List`
instance routes the canonical compare into `raw_list_compare`) is now checked
by nothing.

★ **The name still asserts the routing claim it no longer tests** — a hardcode
of `list_compare` to `Bool` would pass under it. See
`identifiers-are-claim-artifacts`.

⚠ **Advisory, non-blocking** — same posture as Q-CLAIM-CLOSURE: the production
side was confirmed correct at the time of the adversary report; this is a
**coverage regression** that would land green on a future refactor, not a live
defect.

## Scope

Exactly `crates/ken-elaborator/tests/compare_ord_lexicographic_acceptance.rs`.

1. Enumerate the two claims the original block carried; mark each **restored**
   or **consciously dropped** with reason (the Q-CLAIM-CLOSURE closure rule,
   applied to this block).
2. If restored: the routing claim must be exercised on a **non-`Bool`**
   instantiation, since `Bool` is exactly the case that hides it.

## Acceptance

- The claims the block carries are enumerated and each has a verdict.
- **Mutation proof, per claim:** hardcode `list_compare`'s result and confirm
  the restored assertion goes red. ⛔ A test that passes only post-fix proves
  nothing.
- ⚠ **Review lane:** `compare_ord_lexicographic_acceptance.rs` is in the
  ken-elaborator test area — confirm with the owning leader whether this rides
  with a runtime ring or Language reviews it, before kicking. (Steward
  sequencing call, not resolved in this frame.)

## Sequencing

After `Q-CLAIM-CLOSURE` merges — same claim-enumeration technique, and keeping
them serial avoids two rings applying the same rework pattern to sibling files
at once. Not urgent (advisory).
