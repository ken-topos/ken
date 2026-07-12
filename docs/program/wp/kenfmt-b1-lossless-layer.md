# WP B1 — lossless source representation (kenfmt build, the foundation)

Owner: **Language team**. Architecture: **Accepted** —
`docs/program/kenfmt-b1-source-representation.md` (option (2): the existing AST +
a complete token/trivia stream + deterministic comment-attachment; operator
signed off 2026-07-12). Design context: `docs/program/wp/kenfmt-work-program.md`,
`docs/program/kenfmt-canonical-form-review.md`. Normative target: **WP S**
(`spec/30-surface/31-lexical.md §1d`, landed `b9af8cca`). Size **M**. Base:
`origin/main` (re-verify cites at pickup).

## Objective

Build the **lossless source representation** the whole formatter (B2/B3/B4/C) and
any future LSP/refactor tooling inherit: the existing semantic AST **plus** a
complete ordered token/trivia stream, with a deterministic comment-attachment,
**exposed behind an interface**. B1 **round-trips source without changing
layout** — it is the representation + the round-trip proof, **not** the printer
(B3) and **not** canonicalization (B2).

## Fixed inputs — SETTLED (Accepted B1 ruling), do not reopen

- **Representation = AST + token/trivia stream** (NOT a CST). The ruling is
  grounded in the landed parser: `Parser::new(tokens: Vec<(Token, Span)>, src:
  String)` already consumes a pre-lexed token+span vector and retains `src`, and
  the lexer emits `(Token, Span)` per token (`lexer.rs`). The token stream
  already exists; **trivia = the gap bytes between adjacent token spans in
  `src`**, recoverable by a small lexer change (record the skipped whitespace/
  `-- …` comment spans) or a post-pass over `src`. **No new parser; no second
  grammar.**
- **The typed views ARE the existing semantic AST** — reused for precedence/
  grouping. The AST is **never the only** source representation; the trivia
  stream is co-authoritative.
- **Error recovery is OUT** (deferred) — B1 operates on parseable units; the
  invalid-`ignore`/`reject` fence path is token-aware only (WP S exemption), not
  a recovering CST. A CST is revisited only if a real error-recovering LSP/
  refactor consumer lands (the interface, item 4, keeps that a swap).

## The B1 contract — the four items to deliver

1. **A complete, ordered, gapless trivia stream.** Every byte of `src` is
   accounted for: `⋃ token-spans ∪ ⋃ trivia-spans == src`, **contiguously**.
   This is the losslessness invariant — **assert it** (a unit that fails it is a
   bug). Trivia carries whitespace, line comments, and blank-line structure with
   byte offsets.
2. **A deterministic, total comment-attachment.** Every comment gets **exactly
   one** home by a fixed rule: own-line-before a node → **leading**; same-line-
   after → **trailing**; between structural tokens mid-construct →
   **interstitial** on the enclosing node, keyed by byte offset. **Total** = no
   comment is ever dropped or left homeless. (Placement *at print time* is B3;
   B1 only fixes each comment's unambiguous home.)
3. **Typed views = the existing AST**, reused for precedence/grouping — never the
   only representation; the trivia stream is co-authoritative and span-keyed to
   the one source of truth (byte offsets).
4. **Expose the lossless layer as an INTERFACE, not a concrete type** — an
   abstract "formattable tree + attached trivia," span-keyed. This is the
   door-open clause: a future error-recovering CST backs the **same** interface
   (a swap, not a rewrite of B2/B3/B4). B2/B3 consume the interface, not a
   concrete struct.

## Scope

- `crates/ken-elaborator` — the lexer change to record trivia spans (or a `src`
  post-pass), a new module for the trivia stream + comment-attachment, and the
  interface the layer is exposed behind. Reuse the existing AST + `Parser`.
- A **round-trip harness**: reconstruct `src` byte-exactly from (AST + trivia
  stream) and diff against the original, over the **whole catalog** (every `.ken`
  + every parseable Ken fence), not a sample.

### Out of scope (later B-series WPs)

- **No canonicalization** — token-kind glyph normalization replacing
  `canonical_unicode` is **B2**. B1 preserves source lexemes verbatim.
- **No printer / document algebra / layout change** — **B3**. B1 round-trips
  layout **identically**; it does not reformat.
- **No `.ken.md` splicing** — **B4** (B1 may operate per-fence but the splicing +
  prose-identity machinery is B4).
- **No `format.rs` rewrite** — `canonical_unicode` stays as-is until B2 replaces
  it; do not extend the raw-byte path.

## Acceptance criteria

- **AC1 — losslessness invariant.** `⋃ token-spans ∪ ⋃ trivia-spans == src`
  contiguously, asserted programmatically (not just tested on samples).
- **AC2 — byte-exact round-trip.** Reconstructing `src` from (AST + trivia)
  reproduces the original **byte-for-byte** over the whole catalog + all
  parseable fences. A single mismatch is a hard fail.
- **AC3 — total comment-attachment.** Every comment has exactly one home;
  **zero** comments dropped/homeless across the catalog; leading/trailing/
  interstitial classification is deterministic (assert a comment between every
  pair of structural tokens gets a stable home — the ambiguity-suite class).
- **AC4 — interface, not concrete type.** The lossless layer is consumed through
  an abstract interface (formattable tree + attached trivia, span-keyed);
  demonstrate the seam that a future CST could implement.
- **AC5 — no layout/semantic change.** B1 does not reformat and does not
  canonicalize; the AST is unchanged; parse of reconstructed `src` ≡ parse of
  original.
- **AC6 — build.** `scripts/ken-cargo test -p ken-elaborator` green **and** the
  literal CI `cargo build --workspace --locked && cargo test --workspace
  --locked` green (B1 adds a lexer/representation change — confirm the literal
  locked oracle, not just the wrapper). `git diff --check` clean; scope =
  `crates/ken-elaborator` (+ tests) only; **zero** kernel / prelude / semantics /
  Cargo-dep / lock / `trusted_base()` delta.

## Review

**Architect-terminal** (he authored the B1 contract) — the gapless invariant, the
attachment totality/determinism, the interface seam (that a CST could back it),
and that no canonicalization/layout change leaked in. Team QA runs AC2/AC3 over
the catalog + the literal locked CI as first-class gates (the N2 carry). CV's WP S
golden is the downstream oracle (B1 does not satisfy formatter-output cases — it
is red-until-built infrastructure).

## Do-not-reopen guardrails

- **Representation is settled** — AST + trivia stream; **no CST**, no second
  parser, no parser rewrite that the elaborator/kernel pipeline depends on.
- **No canonicalization (B2), no printer (B3), no `.ken.md` splicing (B4), no
  layout change** — B1 is round-trip-identical lossless infrastructure only.
- **Interface, not concrete type** — B2/B3 must consume the abstract layer so a
  CST swap stays possible.
- **Trivia stream is co-authoritative** — never reconstruct comments/whitespace
  from the AST alone.
