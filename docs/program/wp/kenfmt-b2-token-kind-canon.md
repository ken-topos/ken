# WP B2 — token-kind canonicalization (kenfmt build; replaces `canonical_unicode`)

Owner: **Language team**. Single lane (**build**; the spec + golden already
exist — WP S). Consumes **B1** (`FormattableSource`, landed `9df1f465`).
Normative target: **WP S** `spec/30-surface/31-lexical.md` — §1b (blessed
glyph↔ASCII table + the `ℓ` token-kind-disambiguation footnote) and §1d
"**Token-kind canonicalization and protected source**" (spec line ~130, landed
`b9af8cca`). Design source: `docs/program/kenfmt-canonical-form-review.md` §2.
Size **M**. Base: `origin/main` (re-verify cites at pickup).

## Objective

Replace the raw-byte `canonical_unicode` normalizer with a **token-kind-driven**
canonicalizer that consumes B1's token stream: canonical notation is chosen from
each token's **parsed kind**, **never** by raw-text substitution. This is the
normative rule WP S §1d already mandates, and it **fixes the `l`/`level` → `ℓ`
over-fire** at the source (the raw-byte pass's central bug). B2 canonicalizes
token **spelling** only; it does **not** do layout (that is B3) — it preserves
B1's layout so the output still round-trips.

## Fixed inputs — SETTLED (WP S normative), do NOT reopen

- **Canonicalize by parsed token KIND, never raw text** (WP S §1d, line ~130).
  Operator/notation token kinds map to the blessed glyph (`->`→`→`, `\`→`λ`, …,
  the §1b table); **identifier and keyword tokens print their stored spelling**
  unchanged. In particular `l`, `level`, `in`, `not` as **identifier** tokens are
  **never** rewritten by byte resemblance.
- **★ Notation-layer scope — three families (Architect ruling, Accepted;
  `docs/program/kenfmt-b2-notation-canonicalization-layers.md`).** The locked
  "by token kind" AC is correct but **presupposes §1c BL3**: a word only
  canonicalizes if it lexes to a distinct token. So B2's shippable scope splits:
  - **Family A — operator/symbol digraphs already distinct today** (`->`/`→`,
    `|->`/`↦`, `\`/`λ`, `<=`/`≤`, `>=`/`≥`, `/=`/`≠`, `===`/`≡`, `><`/`×`,
    `<:`/`⊑`, `/\`, `\/`): these lex to distinct **operator** tokens now — **B2
    canonicalizes these by token kind. THIS is B2's build scope.**
  - **Family B — reserved notation *words*** (`forall`/`∀`, `exists`/`∃`,
    `Sigma`/`Σ`, `Pi`/`Π`, `Omega`/`Ω`): lex as ordinary `Ident`/`ConId` today.
    Word→glyph is **GATED on a separate BL3/D4 lexer WP** (Language) that makes
    them lex to notation tokens — **NOT a B2 parser role-overlay** (wrong layer,
    subsume-don't-proliferate). Once the lexer emits the notation token, B2's
    existing glyph rule covers Family B for free. **B2 leaves Family-B words
    untouched** (a left-alone `forall` is deterministic + idempotent).
  - **Family C — genuinely contextual, protected** (`l`/`level`→`ℓ`, `in`→`∈`,
    `not`→`¬`): §1d rules these **never rewritten**. B2 never touches them; no
    role machinery.
- **Protected source is verbatim** (WP S §1d). String / raw-string / char /
  bytes literals (base, separators, suffixes, delimiters, escapes), comments and
  doc-comments, temporal-formula text, and foreign symbol/library names preserve
  their **source lexeme** — no canonicalization, no literal normalization.
- **`canonical_unicode` is a migration SEED, not the foundation** (review §2).
  **Reuse its glyph table** behind the token-kind dispatch; **retire the
  raw-byte scanning path** (do not extend it).
- **Formatting is not refactoring** — B2 changes only token *spelling* to the
  blessed glyph; it does not reorder, regroup, desugar, or touch layout.
- **B1 is the input** — consume `FormattableSource`'s token/trivia stream; do
  **not** re-lex from raw `src`, and do **not** reconstruct tokens from the AST.

## Scope

- `crates/ken-elaborator` — a token-kind canonicalizer over B1's token stream
  (map each token to its canonical spelling by kind; protected/identifier/
  keyword kinds pass through verbatim; operator/notation kinds → blessed glyph
  from the reused table). **Replace `canonical_unicode`'s raw-byte body**
  (`format.rs:36`) with the token-kind path (or route its callers to the new
  path and retire the scanner). Keep the public entry point's signature stable
  for existing callers where practical, or migrate callers explicitly.

### Out of scope (later B-series WPs)

- **No layout / document algebra / line-breaking / 88-col** — that is **B3**
  (Wadler/Prettier printer). B2 preserves B1's layout exactly; it only
  canonicalizes token spelling.
- **No `.ken.md` splicing** — **B4**.
- **No comment-attachment or lossless-representation change** — that is B1
  (landed); B2 consumes it read-only.

## Acceptance criteria

- **AC1 — token-kind dispatch.** Canonical spelling is chosen from the token
  **kind**, not raw text. Operator/notation kinds → blessed glyph (§1b table);
  identifier/keyword kinds → stored spelling verbatim.
- **AC2 — the over-fire is fixed.** An **identifier** token spelled `l`,
  `level`, `in`, or `not` is **not** rewritten to a glyph; assert this directly
  (the case the raw-byte `canonical_unicode` got wrong). A genuine
  level/notation token still prints `ℓ`/its glyph.
- **AC3 — protected source verbatim.** Strings/raw/char/bytes/comments/
  doc-comments/temporal/foreign lexemes pass through byte-identical; no literal
  normalization.
- **AC4 — no layout change.** B2 preserves B1's layout; a B2 pass that changes
  only token spelling (not whitespace/structure) — verify against a fixture
  whose only diff from input is the ASCII→glyph token spellings.
- **AC5 — raw-byte path retired.** The `canonical_unicode` raw-byte scanner no
  longer drives canonicalization (reused glyph table only); no caller reaches a
  raw-text substitution path.
- **AC6 — WP S token-kind golden: Family-A cases flip GREEN.** The WP S
  conformance golden's **Family-A** operator-digraph / `ℓ`-disambiguation
  (identifier-protected) / protected-payload cases that were RED-UNTIL-BUILT for
  B2 now pass (identify them in
  `conformance/surface/formatting/seed-canonical-format.md`); the ambiguity-suite
  arms for `l`-ident-vs-level and aliases-inside-literals are green.
  **Family-B word→glyph cases (`forall`→`∀`, etc.) stay RED-UNTIL the BL3/D4
  lexer WP lands** — B2 does not flip them (the lexer precondition is unmet). Mark
  that dependency in the golden, don't fake the pass.
- **AC7 — build.** `scripts/ken-cargo test -p ken-elaborator` green **and** the
  literal `cargo build --workspace --locked && cargo test --workspace --locked`
  green. `git diff --check` clean; scope = `crates/ken-elaborator` (+ tests)
  only; **zero** kernel/prelude/semantics/Cargo/lock/`trusted_base()` delta.

## Review

**Architect-terminal** (he owns the kenfmt B-series contracts and the token-kind
locked constraint). Team QA runs AC2/AC3/AC6 + the literal locked CI as
first-class gates (the N2 carry; the B1 carry — preserve the three lossless gates
downstream). CV's WP S golden is the acceptance oracle: B2 is the producer that
flips the token-kind gate cases from red to green.

## Do-not-reopen guardrails

- **Token-kind, never raw text** — the whole point; do not reintroduce a
  byte-resemblance substitution.
- **No layout (B3), no `.ken.md` (B4)** — spelling canonicalization only.
- **Protected source verbatim** — never canonicalize inside a protected region
  or normalize a literal.
- **Consume B1 read-only** — no re-lexing, no AST-reconstructed tokens, no
  change to the lossless layer.
- **Retire the raw-byte scanner** — reuse only its glyph table.
