# WP · kenfmt layout rework — batch 2 (catalog-wide) + re-arm the strict gate

**Owner:** Language (kenfmt owner) · **Reviewer:** QA · **Architect:**
informational (layout = tool/style, not kernel semantics) · **Size:** M ·
**Base:** `origin/main @ 431e36ea` (batch-1 landed: horizontal-first `layout.rs`
+ R1a, `Core/OrdNat.ken.md` reformatted, PR #610). **Status: READY** —
operator-approved 2026-07-13 (Pat green-lit "catalog-wide + re-arm gate #57"
after reviewing and approving the batch-1 `OrdNat` diff).

## Objective

Apply the **landed, operator-approved** horizontal-first formatter across the
**entire catalog**, then **re-arm the three strict frozen-corpus gates** that
task #57 paused. This is the "then catalog" half of the operator's incremental
directive (one file → review → then catalog); the style is now locked, so the
sweep is mechanical.

## Fixed inputs (settled — do not reopen)

- **The formatter is DONE and APPROVED.** The horizontal-first rules R1–R5 **plus
  R1a** (split-return arrow chains split at top-level `→` only; each fitting
  operand stays on one line) are implemented in
  `crates/ken-elaborator/src/layout.rs` as landed at `431e36ea` and were approved
  by the operator on the `OrdNat` representative diff. **Do NOT change the
  formatter.** If a catalog file renders in a way that looks wrong, that is a
  finding to escalate to the Steward — **not** a licence to re-tune `layout.rs`
  in this WP (a formatter change would reopen the approved style and require
  re-review).
- **Width budget = 96 columns** (settled batch-1).
- **Layout-only, semantics-preserving** — the formatter changes
  whitespace/line-breaks only; it never reorders, renames, inserts, or drops a
  token. This is the hard invariant, mechanically gated per file (AC3).
- **Scope = the whole catalog** — every `catalog/packages/**/*.ken.md` (15 files
  at base; `OrdNat` is already in the target form from batch-1 and should be a
  no-op under `ken fmt`).

## Mandated deliverable outline

1. **Catalog-wide reformat.** Run the landed `ken fmt` over every
   `catalog/packages/**/*.ken.md`. Most files still carry the **pre-review
   "premature canonical" form** from the old (capstone-C) formatter; this brings
   them all to the approved horizontal-first form. `OrdNat` is already converged.
   Result: `ken fmt --check catalog/packages/**/*.ken.md` is green (idempotent
   fixed point) across the whole catalog.
2. **Regenerate the frozen-corpus fixture/goldens** to the new canonical form so
   the strict gates assert against the approved output — the frozen-corpus fixed
   point, the line-count fixture (`FRAME_LINE_COUNTS`), and any canonical
   snapshot the three gate tests read. Regenerate from the formatter output; do
   not hand-edit expected values.
3. **Re-arm the three strict gates** — remove the `#[ignore = "kenfmt strict
   frozen-corpus gate paused …"]` attribute (added by #57) from exactly these
   three `#[test]` functions, restoring them to enforced:
   - `crates/ken-cli/tests/ken_fmt.rs` → `strict_frozen_corpus_gate_is_green`
   - `crates/ken-elaborator/tests/kenfmt_c_capstone.rs` →
     `canonical_frozen_corpus_is_a_31_file_fixed_point`
   - `crates/ken-elaborator/tests/kenfmt_c_capstone.rs` →
     `canonical_reformat_has_no_pathological_line_expansion`
   Leave the three fixture tool-behavior tests untouched (they were never paused).

## Acceptance criteria

- **AC1 — whole-catalog idempotent fixed point.** `ken fmt --check` is green on
  **every** `catalog/packages/**/*.ken.md`; a second format is a no-op
  (format-of-format = format) for every file.
- **AC2 — the three strict gates are RE-ARMED and GREEN.** No `#[ignore]` remains
  on the three named tests; all three run and pass against the regenerated
  canonical corpus. `git grep 'kenfmt strict frozen-corpus gate paused'` returns
  **nothing**.
- **AC3 — semantics-preserving, whole catalog.** For every reformatted file the
  token-kind stream **and** AST are unchanged vs the pre-reformat file (only
  whitespace/layout differs) — the hard invariant, mechanically gated by the
  read-only whole-catalog check (AST + token-kind order + idempotence + 96-col
  bounds) already present from batch-1.
- **AC4 — validate LOCALLY TARGETED only** (operator hard rule, COORDINATION §12:
  **NO local `cargo test --workspace`** — the box OOMs). Run the formatter/layout
  goldens + the re-armed gate tests + the affected crates via `scripts/ken-cargo
  -p <crate>` / `--test <name>` (e.g. `-p ken-cli --test ken_fmt`, `-p
  ken-elaborator --test kenfmt_c_capstone --test kenfmt_b3_layout --test
  kenfmt_signature_layout`). The full-workspace `--locked` gate is **CI's** job;
  the publisher polls it at merge.
- **AC5 — Steward representative spot-check before publish.** Before the
  git_request, the Steward eyeballs a handful of reformatted files spanning
  **distinct constructs** OrdNat lacks — a multi-constructor `data` decl, an
  effectful/lawful class, a functor, `Map`/`Collections`, `Validation` — to
  confirm the approved rules generalize. Any surprising rendering is escalated to
  the operator **before** landing (not silently swept). The formatter is not
  changed to satisfy this; a real style gap becomes a new batch-3 finding.

## Do-not-reopen guardrails

- **Not a formatter change.** `layout.rs` stays byte-identical to `431e36ea`
  unless the Steward+operator explicitly reopen the style. A rendering that looks
  off = escalate, don't re-tune.
- **Not a semantics change.** Whitespace/line-breaks only; AC3 gates it.
- **Do not delete or weaken the gates** — re-arm them exactly (remove the
  `#[ignore]`), don't rewrite their assertions beyond regenerating the canonical
  fixture from formatter output.
- **This closes the kenfmt line** (task #57 re-arm + the catalog rework). No
  further kenfmt batches unless AC5 surfaces a real style gap.

## Sequencing (Steward)

Batch-1 is merged and approved; this is the direct continuation. Language ring is
idle with retros in — Handoff-Gate compact (leader + implementer + qa) then kick.
On merge, CI enforces the re-armed strict gate henceforth. Closes task #57 (gate
re-arm) and the kenfmt rework line.
