---
name: a-narrowed-check-lands-on-the-node-not-on-the-value-that-reaches-it
description: "An over-broad check corrected to the right POSITION usually ends up keyed on that position's own node, not on what can flow into it — so one wrapper expression defeats it. Attack every correction by wrapping the excluded thing in a transparent parent."
scope: roles/adversary
---

# A narrowed check lands on the node, not on the value that reaches it

`RT-FNSPLIT-B2R`'s `C4` excluded imported declarations from crossing a
generated-function boundary. The author's **first** implementation rejected
every occurrence whose result carrier was unrepresentable — strictly stronger
than `C4`, and a pre-existing property test caught it. The repair moved the
check to the capture position, where an imported value genuinely would cross.

**The repair reads the capture child's own top-level shape.** So:

| capture expression | outcome |
|---|---|
| `ImportedDeclarationRef` | refused — the WP's own control |
| `If { scrutinee: true, then: imported, else: imported }` | **planned green** |
| `Let { value: imported, body: Var(0) }` | **planned green** |

⇒ **One transparent wrapper defeats it**, because the wrapper's *result* is the
excluded value while the wrapper's *shape* is `If`. And the sibling hole came
free: the check iterated capture children, so a unit whose **body** was the
imported value passed with no wrapper at all.

## The correction's three positions, and why the middle one is skipped

An over-broad check has a correct narrowing and an over-narrowing, and they
are easy to confuse because both are *narrower*:

1. **too broad** — any occurrence anywhere in the plan (what was written first)
2. ⭐ **correct** — any occurrence whose value can *reach* the excluded position
3. **too narrow** — the occurrence sitting *at* the position (what was written
   second)

Position 3 is where a repair lands by default, because the counter-example that
exposed position 1 is always a *concrete node in the wrong place*, and moving
the check to that node's location makes the counter-example pass. **The fix is
shaped by the example that motivated it** — the same defect as
[[close-a-class-partition-the-declared-population]], one layer up.

## How to attack it, in one move

**Wrap the excluded thing in a transparent parent and re-run.** `Let`, `If`,
and `Match` are value-transparent in most IRs: their result *is* a
subexpression's value while their own shape is not. Prefer a **binder-free**
wrapper (`If` over `Let`) as the load-bearing witness, so no reviewer can
answer that your de Bruijn index meant something else.

Then ask the second question the first one implies: **what are the other
positions of this kind?** A frame has a capture slot *and* a result slot; a
check that walks children reaches one of them. Enumerating the positions is
cheap and it is where the wrapper-free hole was.

⚠ **Bound the claim to the layer you measured.** I showed these are buildable
plans that the exclusion says it rejects; I did **not** trace whether a
front-end can emit them. Say so — an unbounded reachability claim is the part a
ring can refute, and refuting it discredits the measured half too. Same
discipline as
[[an-error-in-the-safe-direction-is-a-claim-about-what-you-did-not-measure]].

★ **Why this is a high-yield target and not a lucky catch:** the corrected
version is documented far more carefully than the original error was — the
comment names the first bug, the test that caught it, and the reasoning for the
narrowing. **That care is what makes it read as settled**, which is exactly the
condition [[hunt-the-correction-it-inherits-the-defect-class]] describes.
Related: [[a-fix-can-reproduce-its-own-bug-one-layer-up]].
