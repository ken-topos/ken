---
id: DOC-W4-TOOLCHAIN
title: "Wave 4 slice 1 — the toolchain reference, plus the D0 report on which of Wave 4's generated facts the toolchain can actually produce today"
status: merged
owner: doc
size: M
gate: none
depends_on: [DOC-W3-DEPDATA]
blocks: []
github: null
origin: "Steward 2026-08-01, under section 2a-bis. The depends_on edge is RING-CAPACITY SEQUENCING, not a content dependency — this slice needs nothing from the dependent-data page, but the doc ring runs one candidate at a time and this must not enter the frontier while DOC-W3-DEPDATA is live. Measured at origin/main = 7fa65b20."
---

# Wave 4, slice 1 — the toolchain reference, and what can be generated

Wave 4 produces `library/reference/` across language, verification, toolchain,
runtime, platform, and diagnostics, plus the symbol, keyword, diagnostic, and
glossary indexes. Its exit property: *a reader who knows what they are looking
for can find a complete, current answer without reading the normative spec front
to back.*

## The problem this slice has to solve before any other Wave 4 slice can be cut

The program commits that **exact syntax, CLI, target, and public-declaration
facts are generated.** Measured at `7fa65b20`, nothing can generate them:

- `scripts/` holds exactly three generators — `gen-doc-status.sh`,
  `gen-progress.sh`, `gen-source-attestations.sh`. None extracts a declaration,
  a keyword, a syntax production, or a CLI surface.
- The CLI has **no machine-readable output** — no `--format`, no JSON path
  anywhere in `crates/ken-cli/src/main.rs`. `print_help` writes prose to stdout.
- Ken has **no diagnostic registry.** That was measured for `DOC-W3-HOWTO`: the
  refusal population is order 300+ formatted-message sites with no index. A
  *generated* diagnostic index is therefore not producible today.
- `library/reference/` does not exist.

**Wave 5 has an explicit precondition for exactly this** — the Librarian reports
which facts the checked artifact format can express before that wave is framed,
and a fact we cannot generate gets authored and labelled as authored. **Wave 4
makes the same generation commitment and states no such precondition.** That is
a gap in the program, and this slice closes it rather than discovering it four
pages in.

## Why the toolchain reference is the right first slice

It is the one Wave 4 surface that needs **no generator at all**, and it is small
and closed: seven subcommands, three options in five accepted spellings
(`fmt --check`, `--version` / `-V`, `--help` / `-h`), one positional argument
that is not a flag (`native-build`'s `<output-dir>`), and three exit-status
classes. `DOC-W3-HOWTO` already established the observe-don't-paraphrase
discipline over this same surface and produced five recipes from it.

⚠ **An earlier version of this paragraph called the flag count two and the exit
behaviour uniform. Both were false** — they came from reading `print_help` and
counting one `exit` value rather than running the tool and enumerating. The
frame's fixed-inputs table is authoritative and carries the measured surface
with line numbers; this paragraph only motivates the slice.

So this slice delivers a real reference page set on a surface that is small,
closed, and verifiable by running the tool — while its D0 answers the question
every later Wave 4 slice depends on.

The frame is `docs/program/wp/DOC-W4-TOOLCHAIN.md`.
