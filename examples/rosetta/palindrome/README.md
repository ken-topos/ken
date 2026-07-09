# Palindrome detection

Determine whether a string reads the same forwards and backwards.

Reference: <https://rosettacode.org/wiki/Palindrome_detection>

## Status

**Elaborates + evaluates correctly (verified in-process).** `ken run`
end-to-end verification is pending the Runtime console-harvest fix
(`wp/console-harvest-fix`, `72eff0e`) landing on `main` — VAL2's differential
runner will pick this up once that merges; until then this dir's `expected`
is pinned from the in-process evaluation result, not yet runner-confirmed.

## Implementation notes

- Reuses the landed `catalog/packages/collections` string surface: `eq`,
  `string_to_list_char` / `list_char_to_string`. No string op is re-derived.
- `reverse` over `List Char` is not in the landed 7-combinator floor, so it's
  defined locally (`reverseListChar`) — a one-off local helper, not promoted
  to the shared package (nothing else in this wave needs `reverse` yet).
  SCT: the recursive call is on `xs2`, a strict `Cons` sub-term.
- `palindrome s = eq s (reverseString s)`.
- The oracle steers around the tracked `natToDecimal` exponential-blowup
  finding (VAL2, routed to the Architect design fork): output is a `String`
  ("PASS"/"FAIL"), never a printed `Nat`/`Int`.
- Discriminating corpus, folded to one value (no IO-sequencing needed for a
  single-line oracle): `"racecar"` (true palindrome), `"hello"` (same-length
  non-palindrome), `""` (vacuous palindrome), `"a"` (trivial palindrome). Any
  one wrong flips the result to `"FAIL"`.

## Oracle

`main` prints `PASS`.
