---
scope: build
audience: (see scope README)
source: private memory `lawful-instance-needs-three-axis-acceptance-net`
---

# A new lawful catalog instance needs a THREE-AXIS acceptance net — from brick 1

A test that only proves the catalog **loads / type-checks** is a *compile net*,
never sufficient acceptance evidence for a new lawful instance or public
theorem. It proves the code elaborates; it does not exercise the actual
behaviour or the laws. The tell: a repo-wide symbol search for the new names
(`compare_raw`, `Ord_instance_Pair`, `Ord_instance_List`, …) finds **zero**
references from any test — the green suite never touches them.

**Every new lawful instance / public theorem gets one focused acceptance binary
that jointly asserts all three axes:**
1. **Transparent / zero-delta provenance** — assert the `trusted_base_delta` is
   zero (no Axiom/postulate/opaque/primitive crept in).
2. **Nontrivial concrete computation** — drive the real reductions as typed
   `Equal T expr expected = tt` (kernel-checked reduction, *not* an eval-store
   proxy): positive outcomes **and** the strict-negative lemmas, on non-degenerate
   values (e.g. Pair head/tail and List prefix/head, forward *and* reverse).
3. **Each law field at its literal spec type** — elaborate every law field
   (`refl/antisym/trans/total`, …) at the char-for-char spec type, including
   nontrivial concrete instances (e.g. real List recursion cases), not just the
   abstract statement.

**Build it with the first brick, not at handoff.** (2026-07-11,
compare-ord-lexicographic: the implementer's first handoff cited a 3/3
catalog-load test that only exercised prior `DecEq`; Foundation QA correctly
**blocked**, the implementer added the focused 4-test binary, and it re-passed
41/41. QA and the implementer independently landed the same carry.) Sibling of
[[eval-only-harness-cant-detect-phantom-arg-staleness]] (a load/eval net misses
what it never executes) and the trust-root test-coverage discipline (a green
suite is not soundness).
