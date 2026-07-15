# VAL2-rosetta-revalidate — re-run the full corpus to zero KNOWN-GAP (wave 3)

**Steward frame → Team Language. Light-gated re-validation mini-WP** (same class
as `VAL2-rosetta-pangram.md`: **no `§2c` pipeline, no `/spec` elaboration, no
Architect/CV gate** unless a genuinely-new capability gap surfaces). Owner:
**language-implementer** (re-author examples + oracles) → **language-qa** (run +
verify + confirm idiomatic). Gate: **light** — Steward-approved, Integrator
merges. Findings → **Steward**.

## Why
Operator directive (Pat): "retry the val2 examples; fix any issues you find;
repeat until all val2 examples are successful." The VAL2 pangram
(`examples/rosetta/`, runner `crates/ken-cli/tests/rosetta.rs`) shipped **16
programs: 10 PASS + 6 KNOWN-GAP**. Since then, **every one of the 6 gaps'
blocking capability has landed** — so this wave re-authors those 6 (plus makes
`rpn-calculator` idiomatic) and drives the corpus to **zero KNOWN-GAP**.

## The 6 KNOWN-GAP examples — capability now landed, re-author + re-run

Each: rewrite `<slug>.ken` to the intended program (the commented-out target is
in the file / `KNOWN-GAP.md`), add the `expected` oracle, **delete `KNOWN-GAP.md`**,
confirm the differential runner passes. **Reuse landed packages** — do not
re-derive list/string/sort ops.

1. **`tree-traversal`** — the ≥2-recursive-field `match` bug is fixed (#5,
   `07d167f`). Uncomment the BST + in-order traversal over `Char`, `"PASS"`/
   `"FAIL"` oracle (matches `palindrome`/`closures` style, steers around the
   separate `natToDecimal` blowup).
2. **`mutual-recursion`** — surface mutual recursion landed (#3, `83f728a`).
   **Auto-grouping: just write the two `view`s (`isEven`/`isOdd`) adjacent — NO
   `mutual` keyword, no forward-decl.** Folded to a `"PASS"`/`"FAIL"` oracle.
3. **`ackermann`** — SCT now does lexicographic/multi-measure descent (`e889284`,
   #256). Uncomment the standard `A(m,n)`; pin **`A(3,2) = 29`** (feasible oracle
   per its `KNOWN-GAP.md`; `A(3,4)` is too many calls at runtime).
4. **`letter-frequency`** — `Map` ops landed as a package (#8). Fold over the
   input's `List Char` threading a `Map Char Nat` (insert-or-increment), then
   `toList` + `catalog/packages/Data/Collections/Derived.ken.md` compare for a sorted report. Reuse the
   landed `Map` — do not hand-roll association lists.
5. **`accumulator-factory`** — `[State]` effect landed (#10, `5626038`). Use the
   **real hidden-state** form (closure over a `[State]` cell). **Do NOT** fake it
   with an explicitly-threaded `Acc` param — the `KNOWN-GAP.md` explicitly warns
   that misrepresents what the task probes (language-qa confirms idiomatic).
6. **`read-file-lines`** — `read_bytes` reduction landed (#9, FS/L6). Read bytes
   → `bytes_decode` → split on newlines → print each line. **Likely needs a small
   `lines`/`splitOn` helper** riding the `catalog/packages/Data/Collections/Derived.ken.md` floor — if that
   helper is a non-trivial sub-build, flag it to Steward as a sub-task rather than
   forcing it in.

## Also: make `rpn-calculator` idiomatic
It already PASSES via prefix `sub_int`/`mul_int`. Now that infix `-`/`*` landed
(#11), **rewrite it to idiomatic infix** and keep it green (the oracle is
unchanged — this is an ergonomics rewrite, verify the value is identical).

## Acceptance criteria
- **AC1 — zero KNOWN-GAP.** No `KNOWN-GAP.md` remains under `examples/rosetta/`;
  all 16 (now 17-with-`closures`? — whatever the corpus count is) have an
  `expected` oracle and pass the differential runner.
- **AC2 — runner green.** `cargo test -p ken-cli --test rosetta` passes for the
  whole corpus; `cargo test --workspace` stays green.
- **AC3 — idiomatic, not faked.** Each re-authored example expresses the task the
  way it's meant to be written (esp. accumulator-factory's real hidden state);
  language-qa confirms — a threaded/prefix workaround that misrepresents the task
  is a **fail**, not a pass.
- **AC4 — new gaps are findings.** If re-authoring surfaces a genuinely-new
  missing capability or a fresh bug, that is a **finding → Steward** (its own WP);
  note it, keep the rest moving, and the corpus repeats until clean.

## Guardrails
- Examples are **surface programs** — kernel/`trusted_base` untouched; reuse
  landed `catalog/packages/Data/Collections/Derived.ken.md` + `catalog/packages/Core/Classes/LawfulClasses.ken.md`, no re-derivation.
- Keep the `natToDecimal` exponential-blowup steer-around (PASS/FAIL oracle
  strings) where the landed examples already use it.
- **Lane:** Language. **After Phase 1** (`83f728a`) — done. Branch off
  `origin/main`.
