---
scope: fleet
audience: (see scope README)
source: CC3 red CI, 2026-07-14 (PR #620)
---

# Adding a file to a globbed corpus trips oracles you did not enumerate

A WP that **adds a file** to a directory some test **globs** (`catalog/`,
`examples/rosetta/`, `conformance/`) does not just have to make its *own* tests
pass — it must satisfy **every corpus-wide oracle**, and those live in crates
you never touched. **Targeted per-crate validation cannot see them**, so they
surface as **red CI at publish**, after review and after the merge Decision —
the most expensive place to find them.

CC3 added two `catalog/packages/Parsing/*.ken.md` files. It passed its own
acceptance harness, the CAT-5 regression suite, QA, Architect terminal review,
and the Steward honesty gate — then went red on
`crates/ken-elaborator/tests/kenfmt_c_capstone.rs`, an oracle in a crate CC3
had no reason to think about. The WP frame had named **one** catalog gate
(`crates/ken-cli/tests/ken_fmt.rs`) and missed the second.

**Rule: when a WP adds to (or renames within) a globbed corpus, grep for EVERY
test that enumerates that directory** — `rg 'collect\(.*catalog|examples/rosetta'
crates/*/tests/` — and run each one targeted before release. Name them in the
frame's acceptance criteria. "The formatter gate" is rarely the only one.

## The deeper trap: a frame-pinned oracle silently becomes a rubber stamp

`kenfmt_c_capstone.rs` holds `FRAME_LINE_COUNTS` — a hardcoded table of
**pre-capstone** line counts — and asserts (a) the table **equals** the live
corpus, and (b) per file, `canonical_lines * 2 <= frame_lines * 9` (an
expansion bound of today's file against its own pre-reformat form).

**Assertion (a) becomes false the moment the catalog legitimately grows**, and
the "obvious fix" — add a row for the new file — is a **fabricated baseline**: a
file created *after* the frame has **no pre-frame form**. Whatever number you
write compares the file to itself (ratio 1.0), so its expansion check is
**vacuous forever**. The gate converts itself from a proof into a ledger, one WP
at a time.

**This had already happened twice before anyone noticed** — the table carries
CC1's and CC2's post-capstone files (`NonEmpty`, `Validation`, `Codec`,
`Numeric`, `StringKeys`, `StringBijection`), all with fabricated baselines and
all vacuous.

**The fix is to re-scope, never to re-baseline** (see
[[frame-pinned-preservation-oracle-is-a-discharged-one-shot-proof]]): a
frame-pinned oracle is a **discharged one-shot migration proof**. Assert only
that its historical paths **still exist** (catching deletion/rename), and let a
**live-anchored** property cover new files — here,
`canonical_frozen_corpus_is_a_39_file_fixed_point` already *enumerates the live
corpus* and checks every file is a `ken fmt` fixed point, so it covered the new
files correctly and stayed green. **Verify the live-anchored net exists before
narrowing the frozen one**, or you trade a rubber stamp for a hole.

**Tell for the reviewer:** if a WP's diff adds rows to a frozen baseline table,
ask what the baseline *means* for a file that did not exist when the baseline
was taken. If the answer is "its current value," the check is vacuous and the
row should not exist.
