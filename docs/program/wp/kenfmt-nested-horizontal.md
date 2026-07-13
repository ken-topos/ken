# WP · kenfmt batch-2a — horizontalize nested applications (then re-sweep + re-arm)

**Owner:** Language (kenfmt owner) · **Reviewer:** QA · **Architect:**
informational (layout = tool/style, not kernel semantics) · **Size:** M ·
**Base:** `origin/main @ 431e36ea` (batch-1 landed). **Status: READY** —
operator-directed 2026-07-13 (Pat, after the batch-2 catalog-wide sweep exposed
the gap via AC5: "fix the formatter, then re-sweep").

## Why (AC5 finding on the batch-2 candidate `7bfd142b`)

The landed horizontal-first formatter horizontalizes **signatures and
arrow-returns** (R1/R1a) but **leaves nested application / proof-term arguments
splayed one token per line.** Concrete: in `Core/LawfulClasses.ken.md`
`pair_ord_head_sound`, the subexpression `(pair_fst a b x)` renders **inline in
the signature** (`: Equal Bool (ord_leq_at a da (pair_fst a b x) (pair_fst a b
y)) True =`, ~70 cols, fits 96) but is **splayed to four lines in the proof
body**:
```
      Equal
      OrdResult
      (compare
      a
      da
      (pair_fst
      a
      b
      x)
      ...
```
This is the same splay flavor the operator flagged on OrdNat (R1a fixed it only
for arrow-chain returns). It is **pervasive** — Map's 27k-line diff, Collections,
Parsing all carry it — and pre-existing (old capstone-C); the current formatter
does not collapse it. `7bfd142b` is **parked** on `wp/kenfmt-catalog-wide` for
reference; the catalog sweep is redone here with the fixed formatter.

## Objective

Extend the horizontal-first collapse so **every** application / parenthesized
subexpression that fits the 96-column budget at its indentation stays on **one
line** — in nested proof-term and λ-body positions, not just top-level
signatures. Then re-run the catalog-wide reformat and re-arm the strict gate.

## Fixed inputs (settled — do not reopen)

- **Generalize R3/R5, don't invent a new style.** R3 (short parenthesized
  expressions never split) and R5 (small expressions stay on one line) are
  already operator-approved; the gap is that they fire at top level but a
  deeply-nested application inside a λ-body/proof term falls through to a
  one-token-per-line printer. The fix makes fit-grouping **recursive**: every
  application/paren node is a fit group — inline if it fits at its column, break
  **minimally** (and recurse into the arguments) only when it exceeds 96. Same
  "prefer horizontal, split minimally" principle as batch-1, applied uniformly.
- **Width budget = 96** (unchanged). **Layout-only, semantics-preserving** —
  whitespace/line-breaks only; never reorder/rename/insert/drop a token (the hard
  invariant; token-kind stream + AST unchanged, mechanically gated).
- **R1/R1a signature ladder and R2 atomic proof selectors are unchanged** — this
  WP only adds nested-application collapse; it must not regress the batch-1
  signature/return rendering (OrdNat stays as approved).

## Mandated deliverable outline

1. **Formatter fix (`layout.rs`).** Make application/paren fit-grouping recursive
   so short nested applications (e.g. `(pair_fst a b x)`, `(compare a da …)`)
   collapse to one line when they fit, and larger ones break minimally at
   argument boundaries with each fitting argument kept horizontal (never
   one-token-per-line). No change to R1/R1a/R2.
2. **Representative-file checkpoint (operator review).** Apply the fixed formatter
   to **one** representative deep-proof file — **`Core/LawfulClasses.ken.md`**
   (the `pair_ord_*` proofs are the worst case) — and hand the Steward the diff.
   The Steward presents it to the operator for approval **before** any
   catalog-wide re-sweep (same one-file-first discipline that worked in batch-1).
3. **Catalog-wide re-sweep + re-arm (after operator approval).** Run the fixed
   `ken fmt` over the whole frozen corpus (all catalog `.ken.md` + the
   `examples/rosetta/*.ken` the gate covers — 19 package sources + examples,
   NOT the stale "15" from the batch-2 frame); regenerate the frozen-corpus
   fixture from the formatter output; remove the three `#[ignore]` markers to
   re-arm the strict gates (`strict_frozen_corpus_gate_is_green`,
   `canonical_frozen_corpus_is_a_31_file_fixed_point`,
   `canonical_reformat_has_no_pathological_line_expansion`).

## Acceptance criteria

- **AC1 — nested applications horizontal.** Short applications in proof-term /
  λ-body positions stay on one line when they fit 96 cols; `git grep` shows no
  one-token-per-line splay of a fitting application (e.g. no bare `(pair_fst`
  followed by `a`/`b`/`x` on separate lines). Golden tests pin the
  nested-collapse rule alongside the batch-1 R1–R5/R1a goldens.
- **AC2 — no batch-1 regression.** `Core/OrdNat.ken.md` renders exactly as
  approved at `431e36ea` (R1 ladder + R1a inline returns unchanged); R2 proof
  selectors stay atomic.
- **AC3 — whole-corpus idempotent fixed point.** `ken fmt --check` green on every
  frozen-corpus source; format-of-format = format.
- **AC4 — the three strict gates RE-ARMED and GREEN** against the regenerated
  canonical corpus; `git grep 'kenfmt strict frozen-corpus gate paused'` returns
  nothing.
- **AC5 — semantics-preserving, whole corpus** — token-kind stream + AST
  unchanged per file (the hard invariant, mechanically gated).
- **AC6 — validate LOCALLY TARGETED only** (operator hard rule, COORDINATION §12:
  NO local `cargo test --workspace` — the box OOMs). Formatter/layout goldens +
  the re-armed gate tests + affected crates via `scripts/ken-cargo -p <crate>` /
  `--test <name>`; CI runs the full `--locked` gate at merge.

## Sequencing (Steward)

Two-stage, gated on operator review between stages (the batch-1 pattern that
worked): **(1)** formatter fix + `LawfulClasses` representative → **Steward
presents to operator** → approve → **(2)** catalog-wide re-sweep + gate re-arm →
QA → Steward AC5 spot-check → publish. Handoff-Gate compact the Language ring
(retros for batch-1 already in; batch-2 candidate parked, no obligation) then
kick. Closes the kenfmt line. Supersedes the batch-2 catalog sweep (#61 gated on
this).
