# Parsing/RPN calculator

Evaluate a reverse-Polish-notation expression.

Reference: <https://rosettacode.org/wiki/Parsing/RPN_calculator_algorithm>

## Status

**Elaborates + evaluates correctly (verified in-process, ~ms).** `ken run`
end-to-end verification is pending building the runner's package-prelude
allowlist decision for this dir (it needs no `packages/collections`
symbols — self-contained — so should work once tested through the CLI).

## Implementation notes

- `RpnOp = Push Int | Add | Sub | Mul` — a fixed, pre-tokenized
  `List RpnOp` (no string tokenizer/`split-on-whitespace` exists yet in
  `packages/collections` — not this example's axis to probe, so
  hardcoded rather than rediscovered here).
- **Division omitted.** `ken-interp` has no `div_int`/`mod_int` primitive
  at all (only `add_int`/`sub_int`/`mul_int`) — the same already-known gap
  documented in `packages/collections/collections.ken`'s header and the
  VAL2 `natToDecimal` finding. Not re-filed as a second `KNOWN-GAP.md`;
  this example demonstrates the probed axis (`Option` composition without
  exceptions) over `+`/`-`/`*`, unaffected by it.
- **Two more small syntax gaps found and worked around:** no `-` infix
  operator exists at all (`GAP-subtraction`, `sub_int` called prefix); `*`
  parses (`BinOp::Mul`) but is "not yet supported" at elaboration time
  (`mul_int` called prefix instead).
- `rpnStep`/`rpnEvalList` compose via `Option`, short-circuiting on the
  first stack-underflow (`None`) rather than any exception mechanism —
  the probed axis. Two-`Cons` peels are done via nested match
  *expressions* (flat patterns only), not nested constructor *patterns*
  (`GAP-nested-patterns` still holds).
- Discriminating oracle: `"3 4 + 2 *"` = `14` (success case) AND `"3 +"`
  = stack-underflow (failure case, must actually reject, not produce a
  wrong number) — both folded into one PASS/FAIL check.

## Oracle

`main` prints `PASS`.
