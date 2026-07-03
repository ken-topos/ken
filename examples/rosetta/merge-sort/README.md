# Sorting algorithms/Merge sort

Sort a sequence via merge sort.

Reference: <https://rosettacode.org/wiki/Sorting_algorithms/Merge_sort>

## Status

**Elaborates + evaluates correctly (verified in-process, ~11s).** `ken run`
end-to-end verification is pending the runner (`ken-cli/tests/rosetta.rs`,
in progress). Note the runner needs a generous per-example timeout for this
one (see below).

## Implementation notes

- Structural recursion + SCT, `leqChar` (`Ord`), parametric polymorphism
  over the list-element type. Sorts `List Char` — steers around the tracked
  `natToDecimal` gap (oracle is a PASS/FAIL `String`).
- `splitAlt` halves a list by alternating elements (no `Nat` division
  needed). `merge`'s recursion decreases exactly one of its two list args
  per call (not the same one every time) — confirmed SCT-acceptable via a
  scratch probe; distinct from Ackermann's blocked lexicographic case since
  each individual call site still has one literal strict-subterm argument.
- `mergeSort` is not directly structurally recursive (its two recursive
  calls operate on `splitAlt`'s *computed* halves, not literal `Cons`-tail
  sub-terms) — same well-founded-recursion situation as the landed
  `gcd.ken`'s `natGcdFueled`: fixed via an explicit `Nat` fuel parameter
  (the list's own length) that decreases structurally while the list
  arguments ride along as computed values.
- **Corpus size, same tracked perf family as `natToDecimal`.** `fuel`
  (`f`) is referenced twice per level (once per recursive child call) — the
  same "bound value referenced twice in one function body" shape already
  identified as an exponential-cost trigger in `ken-interp` (no
  term-sharing in substitution; a Runtime fix is already scoped). Confirmed
  empirically: 4 chars = 221ms, 5 = 1.7s, 6 = 10.5s, 7 = 58s (~8x per +1
  element). Sized the oracle at 6 chars (`"dcbafe"` → `"abcdef"`,
  ~11s) — the same judgment call as `gcd` (4, fast) vs. `factorial`/
  `fibonacci` (120/55, infeasible): pick the largest size still
  comfortably within a generous test timeout, don't force an
  arbitrarily-larger "natural" size through what's now a known-infeasible
  cost regime.

## Oracle

`main` prints `PASS` (~11s to evaluate — the runner needs a generous
per-example timeout for this dir specifically).
