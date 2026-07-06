# ken-md-literate

**Owner:** Language, with Architect review for export/hash policy.
**Branch:** `wp/ken-md-literate` when released.
**Status:** Released to Language (2026-07-06). CAT-4/CAT-3 priority work is
closed; start with D0 pins before implementation.
**Size:** S/M. **Risk:** low for read/elaboration; medium for source identity
and future formatter/export seams.

## 0. Source Brief

Operator brief: `local/ken-md-literate-brief.md` (2026-07-05).

The brief proposes a `.ken.md` literate Markdown source form: ordinary Markdown
is prose, and only fenced code blocks whose info string is `ken` are Ken source.
The design relies on Ken's current byte-offset span model and a single `&str`
source path through the lexer/parser/elaborator.

## 1. Objective

Support `.ken.md` input for single-file Ken execution while preserving
diagnostic byte offsets into the original Markdown file.

The v1 implementation must extract executable Ken by **blanking out** Markdown
outside compiled fences, not by concatenating code blocks. The extracted string
must have the same byte length as the original input; every non-newline byte
outside compiled fences becomes ASCII space, and newlines are preserved.

## 2. Required Design Pins

Before implementation, post a short D0 note pinning:

1. **Compiled fence syntax.** Start with exact backtick fences whose info string
   is exactly `ken`. Other languages and variants such as `ken ignore` are
   prose-only for v1 unless the D0 note explicitly extends the grammar.
2. **Declaration boundaries.** Decide whether declarations may straddle two
   `ken` fences. The brief notes blank-out makes this mechanically possible;
   the WP must choose either "allowed" or "rejected for legibility."
3. **Export/source hash policy.** Architect must decide whether any source hash
   for future export/provenance uses the original `.ken.md` artifact or the
   extracted Ken input. V1 must not silently choose a policy at an export seam.
4. **Out-of-scope loader/fmt behavior.** Filesystem module loading and `ken fmt`
   round-trips are deferred. The design should state where the same extractor
   would be reused when those entry points exist.

## 3. Implementation Scope

In scope:

- Add an extractor, likely in `ken-elaborator`, reusable by CLI/tests:
  `extract_ken(md: &str) -> String`.
- Preserve byte offsets by blanking outside compiled fences at the byte level.
- Preserve newlines exactly.
- Handle UTF-8 prose safely.
- Add a CLI branch at the disk-read boundary so `ken run path.ken.md` elaborates
  the extracted source while ordinary `.ken` behavior is unchanged.
- Keep diagnostics/errors using offsets that index the original `.ken.md`.

Out of scope:

- CommonMark-complete fence parsing.
- Tilde fences, indented fences, nested fences, and attributes beyond the D0
  pinned v1 info string.
- Multi-file module loader support.
- Formatter/LSP round-trip behavior.
- Doctest expected-failure modes.
- Kernel, trusted-base, or language-semantics changes.

## 4. Acceptance Criteria

- **AC1 -- offset preservation.** Unit tests prove extracted output length
  equals input length, and a token/error inside a `ken` fence retains the
  original `.ken.md` byte offset.
- **AC2 -- token equivalence.** For supported fences, the token stream matches
  concatenating the Ken fence contents modulo whitespace.
- **AC3 -- fence selection.** `ken` fences compile; non-`ken` fences are blanked
  and do not contribute declarations.
- **AC4 -- UTF-8 safety.** Unicode prose outside Ken fences remains valid after
  blanking, with newlines preserved.
- **AC5 -- CLI e2e.** `ken run` accepts a `.ken.md` file whose Ken fences form a
  valid program and rejects/diagnoses an invalid fenced program at the original
  file offset.
- **AC6 -- plain Ken unchanged.** Existing `.ken` execution and focused parser /
  elaborator tests remain green.
- **AC7 -- seam policy recorded.** The export/hash decision is documented or
  explicitly deferred with Architect approval; no source hash behavior changes
  accidentally.
- **AC8 -- workspace green.** Focused tests and
  `scripts/ken-cargo test --workspace` pass.

## 5. Guardrails

- Do not concatenate code blocks for the compiler input.
- Do not thread a new source-map object through diagnostics for v1.
- Do not alter `lexer.rs`, `parser.rs`, span representation, resolver, or kernel
  behavior unless D0 proves the blank-out route is impossible.
- Keep the original Markdown text available wherever future diagnostic
  rendering needs source context; the extracted text is only lexer input.
- Do not make ordinary `.md` files executable; the extension is `.ken.md`.

## 6. Review Path

Language leader routes implementer/QA. Architect reviews D0 export/hash policy
and any source-identity seam. Kernel review is not expected. Integrator merge
uses the normal build/test, conformance, clean-room, and path-guard gates.

## 7. Downstream

Future WPs may add:

- module-loader extension handling for `.ken.md`;
- formatter/LSP round-trip inside fences only;
- doctest modes such as `ken ignore` or expected-error fences;
- richer CommonMark fence parsing.
