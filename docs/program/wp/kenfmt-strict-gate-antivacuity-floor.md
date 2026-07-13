# WP — kenfmt-C strict-gate anti-vacuity floor (evolution-robust corpus count)

Owner: **Language** (their kenfmt gate). Reviewer: **Architect**
(C-gate design; ruling `evt_5x7cgmtz7ygt9`). Size: **XS** (test-only,
~4 assertions). Base: `origin/main @ bd9e6c4b` (post-I-1). Deps: I-1 merged
(done — it retired the frame oracle these tests used to carry).

## Why (settled — do not reopen)

The kenfmt-C strict frozen-corpus gates already **enumerate the corpus
dynamically** (glob `catalog/**/*.ken.md`, `examples/rosetta/**/*.ken`, + the
boundary `.ken`) and run `ken fmt --check` / per-file `format(file)==file` +
zero-indent over whatever the glob finds — so new catalog packages are *already*
gated clean. The **only** frozen residue is the exact-count assertions, which
break on every authorized corpus addition (CC1's two new packages made
catalog 14→16; every CC WP repeats this). Per-WP count migration (14→16→18…) is
the proliferation anti-pattern rejected on the frame oracle.

But the count asserts have a real job — **anti-vacuity**: a dynamically-globbed
gate passes *vacuously* if the glob silently breaks and matches zero/too-few
files (`format(file)==file` over an empty set is trivially green → the gate is
disabled but looks green). So the fix is **not** to delete the count — it is to
replace the exact pin with a **lower-bound floor** at today's baseline.

## Deliverable (mandated — exact edits, test-only)

Replace each exact-count `assert_eq!` with a floor `assert!(… >= N)` at the
current baseline; change **nothing else** (keep the single `collect`/glob
enumeration as the one source of truth, the per-file fixed-point + zero-indent
checks, and the `ken fmt --check` CLI invocation byte-identical):

- `crates/ken-cli/tests/ken_fmt.rs` (`strict_frozen_corpus_gate_is_green`):
  - `assert_eq!(catalog.len(), 14)` → `assert!(catalog.len() >= 14, …)`
  - `assert_eq!(rosetta.len(), 16)` → `assert!(rosetta.len() >= 16, …)`
- `crates/ken-elaborator/tests/kenfmt_c_capstone.rs`
  (`canonical_frozen_corpus_is_a_31_file_fixed_point`, P1):
  - `assert_eq!(literate.len(), 14)` → `assert!(literate.len() >= 14, …)`
  - `assert_eq!(plain.len(), 17)` → `assert!(plain.len() >= 17, …)`

Keep each assert's failure message naming the floor + observed count (drift is
still legible). **Do not** touch `PLAIN_FRAME_ORACLE`/`LITERATE_FRAME_ORACLE`
(retired by I-1 — gone post-`bd9e6c4b`). **Do not** dynamicize
`canonical_reformat_has_no_pathological_line_expansion` — the Architect ruled it
a scoped historical backstop that does not trip new files; leave as-is.

## Acceptance criteria

- **AC1** — the four `assert_eq!(len,N)` become `assert!(len>=N)`; nothing else
  in either test moves (per-file fixed-point, zero-indent, CLI `--check`
  invocation byte-identical). Test-only.
- **AC2 — non-vacuous.** The gate still **fails** on a de-canonicalized fixture,
  and would fail if the glob dropped below the floor (broken enumeration /
  deleted corpus file). Show it (or reason it structurally for the Architect).
- **AC3** — `scripts/ken-cargo build --workspace --locked && … test --workspace
  --locked` green on the exact SHA (with the pre-CC1 corpus, counts are exactly
  at the floor → pass; the gate is unblocked for CC1's rebase).
- **AC4 — scope.** Only the two named test files; no production/formatter/
  corpus/kernel/`Cargo.lock`/`spec`/`conformance`/`.github` delta.

## Review & close

Language build → Language QA → **Architect** re-confirm (his 3 checks:
each `assert_eq!`→`assert!(>=)`, per-file/CLI checks unchanged, still fails on a
de-canonicalized fixture) → `git_request` to Steward → honesty-gate + CI-poll
publish. **On merge, signal @foundation-leader to rebase CC1
(`wp/cc1-nonempty-validation @ 852d65ba`) onto the new main** and re-run the
locked workspace gate → CC1 unblocked. Unblocks the whole CC1→CC9 chain from the
count treadmill.
