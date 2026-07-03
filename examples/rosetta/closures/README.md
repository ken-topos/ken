# Closures/Value capture

Build several closures that each capture a distinct value, and show the
captured value survives independently in each closure.

Reference: <https://rosettacode.org/wiki/Closures/Value_capture>

## Status

**Elaborates + evaluates correctly (verified in-process).** `ken run`
end-to-end verification is pending the Runtime console-harvest fix landing
(same status as `palindrome`).

## Implementation notes

- `mkPrefixer p` returns a closure capturing `p`; three closures
  (`"Hello, "`/`"Bye, "`/`"Hey, "`) are built into a list and applied to a
  common name (`"World"`) via a local `map` (a one-off, matching
  `crates/ken-elaborator/tests/l3a_acceptance.rs`'s own local `map` — not in
  the landed `packages/collections` floor). Each closure captures `p`
  independently — the property this task probes (a naive shared-mutable
  capture bug would make all three closures prepend the same, last-bound
  prefix).
- **`GAP-arrow-type-as-type-arg` (non-blocking, worked around).** `A -> B`
  has no term-level expression form in the parser — `Expr`
  (`crates/ken-elaborator/src/parser.rs:1156-1246`) has no arrow/Pi variant;
  arrow types parse only via the separate type-annotation grammar (after
  `:`). Confirmed empirically: `Nil (String -> String)` fails to parse
  (`expected RParen, found Arrow`), both bare and under `(_ : Type)`
  ascription — a generic (`List a`, `Option a`, ...) can never be explicitly
  instantiated at a function type. Worked around by boxing the closure in a
  single-constructor `data StrFn = MkStrFn (String -> String)` (`StrFn` is
  an ordinary named `Type`, so `List StrFn` instantiates fine) — a standard
  closure-boxing idiom, not a hack hiding broken behavior, but forced by a
  real parser gap rather than chosen. Flagged to language-leader; not filed
  as its own blocking `KNOWN-GAP.md` since the example still delivers the
  probed axis faithfully.
- Output steers around the tracked `natToDecimal` exponential-blowup gap:
  the oracle is a PASS/FAIL `String`.

## Oracle

`main` prints `PASS`.
