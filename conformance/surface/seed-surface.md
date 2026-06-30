# Surface conformance — seed cases

Format: `../README.md`. These pin the surface non-reproductions of the
prototype's gaps (the f64 defect; stubbed sum types; no exhaustiveness).

> **The `surface/numbers/*` cases moved.** The numeric model (the f64
> non-reproduction, `Int` exactness above 2⁵³, literal defaulting, the overflow
> obligation, `Decimal`/`Float` honesty, conversions) is now pinned to L1 rigor
> in `numbers/seed-numbers.md`. The two bootstrap cases that lived here —
> `surface/numbers/int-not-float` and `surface/numbers/int-exact-above-2^53` —
> are **subsumed** there (one home per property); see that file for AC1–AC6.

## surface/data-match/construct-then-eliminate
- spec: `spec/30-surface/34-data-match.md §1,§3`
- given: `data Option a = None | Some a`; `match (Some 3) { Some x => x; None =>
  0 }`
- expect: **reduces-to** `3` (real constructor + eliminator)
- why: sum types are finished, not lowered to an opaque base with no eliminator
  (the prototype's stub would fail this).

## surface/data-match/exhaustiveness-required
- spec: `spec/30-surface/34-data-match.md §4`
- given: `match (c : Color) { Red => …; Green => … }` (missing `Blue`)
- expect: **rejects** (non-exhaustive match) naming the unmatched pattern `Blue`
- why: exhaustiveness checking the prototype lacks.

## surface/data-match/refinement-obligation
- spec: `spec/30-surface/34-data-match.md §5`, `20-verification/22`
- given: passing a plain `Int` where `{ n : Int | n > 0 }` is expected
- expect: an **obligation** `n > 0` is generated at that point (discharged or a
  hole), not a silent coercion
- why: refinements enforce; using `A` as `{x:A|φ}` costs a proof.

## surface/elaboration/well-typed-output (invariant)
- spec: `spec/30-surface/39-elaboration.md §3`
- given: any program the elaborator **accepts**
- expect: its emitted core term **passes** `kernel.check` (no accepted program
  yields ill-typed core)
- why: the elaborator is untrusted but disciplined; the kernel is the backstop.

## surface/elaboration/ambiguity-is-an-error
- spec: `spec/30-surface/39-elaboration.md §3`
- given: an unresolvable metavariable / ambiguous instance
- expect: a **surface error** with a span (never a silent default, except the
  declared numeric/level defaults)
- why: no guessing past genuine ambiguity.
