# Surface conformance — seed cases

Format: `../README.md`. These pin the surface non-reproductions of the
prototype's gaps (the f64 defect; stubbed sum types; no exhaustiveness).

> **The `surface/numbers/*` cases moved.** The numeric model (the f64
> non-reproduction, `Int` exactness above 2⁵³, literal defaulting, the overflow
> obligation, `Decimal`/`Float` honesty, conversions) is now pinned to L1 rigor
> in `numbers/seed-numbers.md`. The two bootstrap cases that lived here —
> `surface/numbers/int-not-float` and `surface/numbers/int-exact-above-2^53` —
> are **subsumed** there (one home per property); see that file for AC1–AC6.

> **The `surface/data-match/*` cases moved.** Sum types, `match`,
> exhaustiveness + reachability, indexed families, and refinement types are now
> pinned to L2 rigor in `data-match/seed-data-match.md`. The three bootstrap
> cases that lived here — `construct-then-eliminate`, `exhaustiveness-required`,
> and `refinement-obligation` — are **subsumed** there (one home per property)
> as AC1, AC3, AC7; see that file for AC1–AC7.

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
