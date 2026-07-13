# WP · Disable the kenfmt strict frozen-corpus gate (reversible)

**Owner:** Language · **Reviewer:** QA (locked workspace) · **Architect:**
informational (their gate is paused by operator policy — no design vote) ·
**Size:** XS (test-only) · **Base:** `origin/main @ 1f85ad9a`

## Why (operator directive, 2026-07-13)

The full-catalog kenfmt reformat + strict gate (capstone C) was approved before
the operator reviewed the formatter *output*. The reformat needs **incremental
rework** — one file at a time — and only when the whole catalog is reworked will
the gate be re-enabled. Until then the strict frozen-corpus gate **must not**
force every catalog file to match the current (premature) canonical form or fail
CI, because that blocks the incremental rework.

**This is a policy pause, not a redesign.** Do not touch the formatter, do not
delete the gate, do not revert the catalog. The current reformatted catalog
stays as the rework starting point.

## Fixed inputs — the exact edits (do not improvise)

Add this attribute (verbatim reason) immediately above each of the **three**
named `#[test]` functions:

```rust
#[ignore = "kenfmt strict frozen-corpus gate paused per operator 2026-07-13; catalog reformat is being reworked incrementally — re-enable after rework (see docs/program/IMPLEMENTATION-PROGRESS.md)"]
```

1. `crates/ken-cli/tests/ken_fmt.rs` → `strict_frozen_corpus_gate_is_green`
2. `crates/ken-elaborator/tests/kenfmt_c_capstone.rs` →
   `canonical_frozen_corpus_is_a_31_file_fixed_point`
3. `crates/ken-elaborator/tests/kenfmt_c_capstone.rs` →
   `canonical_reformat_has_no_pathological_line_expansion`

**Do NOT touch** (they validate the `ken fmt` binary still works — the rework
needs them): `fmt_rewrites_plain_and_literate_sources_through_landed_entry_points`,
`fmt_check_names_every_offender_and_never_writes`,
`fmt_fails_loudly_for_unsupported_paths_and_must_parse_bodies`. All are
fixture-based, not corpus gates.

**Do NOT delete** any test, helper (`collect`, `assert_no_zero_indent_continuation`,
`FRAME_LINE_COUNTS`), or `use`. `#[ignore]` keeps them compiled and referenced
(no dead-code warnings) and makes re-enabling a one-line revert. Reversibility is
the whole point.

## Acceptance criteria

- **AC1** — exactly the three named tests carry the `#[ignore]` attribute above;
  no other test, no production/formatter/catalog byte, is changed. `git diff
  --stat` shows only the two test files, `+3/−0` lines each region.
- **AC2** — `scripts/ken-cargo build --workspace --locked && scripts/ken-cargo
  test --workspace --locked` is green, with the three gate tests reported
  **ignored** (not run) and the three kept `ken_fmt.rs` behavior tests still
  **run and pass**.
- **AC3** — reversibility is self-evident: removing the three attributes re-arms
  the gate unchanged (no proof needed; state it in the handoff).
- **AC4** — zero kernel / `Cargo.lock` / `spec` / `conformance` / `.github` /
  `catalog` delta; test-only, two files.

## Do-not-reopen guardrails

- Not a formatter fix, not a gate redesign, not a catalog revert.
- Don't "improve" the disable into deletion — keep it a reversible `#[ignore]`.
- Don't widen scope to the fixture behavior tests or any other suite.
