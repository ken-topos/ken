# WP B3 — document algebra + layout printer (kenfmt build)

Owner: **Language team**. Single build lane (**build**; the layout spec already
exists — WP S `§31 §1d`). A **CV/enclave conformance companion** (the per-axis
both-orientation output oracles) is authored in parallel — see "Conformance
companion" below. Consumes **B1** (`FormattableSource`, `9df1f465`) and **B2**
(token-kind canon, `8945e887`). Design source of truth: **Architect ruling**
`docs/program/kenfmt-b3-doc-algebra-and-layout.md` + WP S
`spec/30-surface/31-lexical.md §1d` (the complete normative layout). Size **L**.
Base: `origin/main` (re-verify cites at pickup).

## Objective

Build the **layout engine**: a Wadler/Leijen document algebra + one printer per
grammar production over B1's typed AST, producing the **single canonical
layout** — line-breaking, indentation, the **88-column** display-width fill —
that WP S `§1d` already mandates. B3 consumes B1's lossless token/trivia stream +
comment attachments and B2's canonical token spellings, and emits final
canonical text. B3 is the **build to `§1d`**, not a new design: every layout axis
is already normatively ruled there; B3 implements it.

## Fixed inputs — SETTLED (WP S `§1d` + Architect ruling), do NOT reopen

- **Algebra = Wadler/Leijen `Doc`** (`text` / soft `line` / hard `line` /
  `group` / `nest` / `flatten`), **display-width** measurement (canonical glyphs
  `λ`/`→`/`Ω` count at terminal width — `§1d`'s "88 Unicode display columns"),
  **one printer per grammar production** over `FormattableSource`'s typed AST.
  **NOT a Prettier-style fill model** — fill *packs* a sequence maximally per
  line, exactly what Ken forbids (one `match` arm per line, always).
- **`group` is a BINARY, deterministic fit decision** — flat iff the flattened
  group fits the remaining width, else broken; a **pure function of `(subtree,
  current column, 88)`** → exactly one layout per input. This is the `§1d`
  "one deterministic canonical form" mandate; fill's packing latitude is the
  non-determinism `§1d` prohibits. Do **not** introduce any width-tie or packing
  choice.
- **Mandatory-structural breaks via hard `line`** (propagates → any `group`
  containing one can never flatten): every `match` with **≥2 arms**, a **nested
  `match`**, a **block body**, a **non-trivial sum** (one constructor per line).
  These are multiline **independent of width**. Width-driven breaks use a soft
  `line` inside a `group`.
- **`nest` is enclosing-relative, exactly two ASCII spaces per level** — never
  aligned to a coincidental visual column where a head token ended. **NO global
  alignment combinators** (a local edit must not reflow unrelated siblings).
  Tabs forbidden.
- **The axes are already ruled in `§1d` — B3 implements, never re-decides:**
  - **Blank-line runs:** 1 between top-level decls; 0 between siblings; around an
    attached comment at most one preserved (`0→0`, `1→1`, `2+→1`); the formatter
    otherwise owns vertical space.
  - **Alignment:** **never** align sibling arrows / colons / equals / bodies —
    single space; indentation alone expresses structure.
  - **Separators:** semicolon **between** declaration-block siblings, **omitted
    after the last**; comma in record literals, patterns, named-arg forms.
  - **Parentheses:** precedence-required + the three mandatory-clarity cases
    kept; **"any other parenthesis is removed"** (the mandate form — the WP-S
    redundant-paren "permission vs mandate" determinism hole is **already
    closed** in `§1d`).
- **Comments = hard-`line`-bearing `Doc` nodes** (lifted from B1's attachments,
  which are total + exactly-one-home, keyed by byte offset so B3 inherits the
  no-relocation guarantee):
  - **Interstitial** (mid-construct) → a **hard `line`** into the enclosing group
    → **forces that group to its broken form**, at the enclosing node's indent,
    **never crossing a syntactic boundary**.
  - **Leading** (own line above) → a hard-`line`-separated line at the node's
    indent; a doc comment binds to the following declaration.
  - **Trailing / EOL** → inline **iff** `code + 2 spaces + comment ≤ 88`, else
    moved to the line immediately above (a determinism axis — oracle **both**
    sides).
  - **Invariant (state once):** any `group` carrying an attached interstitial or
    leading comment **cannot flatten**.
- **B3 is CST-agnostic** — read the `FormattableSource` interface only; a future
  CST backing that interface needs no B3 rework (the B1 door-open clause). Do
  **not** re-lex, reconstruct tokens from raw text, or change B1's lossless
  layer. Consume B2's canonical spellings (do not re-canonicalize).

## Scope

- `crates/ken-elaborator` — the Wadler/Leijen `Doc` algebra + one printer per
  grammar production, driving canonical layout from B1's `FormattableSource`
  typed AST and B2's canonical token spellings. This is the layout half the B2
  pass deliberately deferred (B2 preserved B1's layout; B3 replaces it with the
  canonical layout).

### Out of scope (later kenfmt WPs / capstone)

- **No one-time catalog reformat** — B3 runs its gate **read-only** over the
  whole catalog (parse-preservation + idempotence + width property). The actual
  reformat of the catalog to canonical form is the **capstone C** (atomic,
  strict gate), not B3.
- **No `.ken.md` splicing** — **B4**.
- **No change to B1's lossless layer or B2's token canon** — consume both
  read-only.

## The preservation gate TRANSITIONS at B3 (read carefully)

B1's gate was **byte round-trip** (B1 was layout-neutral). **B3 changes layout**,
so byte round-trip no longer holds. B3's gate is the three-part property, run
**continuously over the whole catalog, read-only**:

1. **Parse-preservation** — `fmt(src)` parses to an AST **equal to** `src`'s
   modulo trivia / spans / sanctioned aliases. (The paren printer is the
   highest-risk axis for this — see AC4.)
2. **Idempotence** — `fmt(fmt(src)) == fmt(src)` **byte-exact**.
3. **88-column width property** — every line `> 88` display columns is
   classified **indivisible / verbatim** (an over-long string literal, a
   `ken ignore` fragment); **no breakable syntax silently overflows**.

## Acceptance criteria

- **AC1 — Wadler/Leijen algebra.** `text`/soft-`line`/hard-`line`/`group`/`nest`/
  `flatten` with **display-width** fit; `group` is the binary pure-function
  fit decision; one printer per production over the typed AST. No fill/packing.
- **AC2 — mandatory-structural breaks.** `match` ≥2 arms, nested `match`, block
  body, non-trivial sum are **always multiline** (hard `line`), independent of
  width; assert a narrow-but-still-broken case (width can't flatten them).
- **AC3 — the `§1d` axes.** Blank-line runs (`2+→1`, sibling→0); **no** sibling
  alignment; semicolon-between / no-trailing decl separators (comma in record/
  pattern/named-arg); **2-space** enclosing-relative indent, no tabs.
- **AC4 — parenthesization (highest-risk).** Redundant parens **removed**,
  precedence-required + the three mandatory-clarity cases **kept**. Gate =
  **parse-preservation over the whole catalog** (AST-equal modulo trivia/spans/
  aliases) **+ elaboration-preservation backstop** (catches a fixity/resolution
  interaction parser-equality alone misses). This is the axis that most needs
  **adversarial catalog coverage** — a wrong paren choice reparses to a
  different AST.
- **AC5 — comments.** Interstitial → hard `line` forcing the enclosing group to
  break, at the node indent, no boundary crossing; leading → line above; trailing
  → inline iff `code+2+comment ≤ 88` else moved above. **Any group carrying an
  attached interstitial/leading comment never flattens** — assert this directly.
- **AC6 — the WP S golden flips GREEN.** The formatter-output cases in
  `conformance/surface/formatting/seed-canonical-format.md` marked
  **red-until-B3** now pass, and the per-axis both-orientation oracles (the CV
  companion) pass. Don't fake a case whose oracle isn't yet authored — coordinate
  with CV.
- **AC7 — the gate + build.** Parse-preservation + idempotence + 88-col width
  property green **read-only over the whole catalog**. `scripts/ken-cargo test -p
  ken-elaborator` green **AND** literal `cargo build --workspace --locked &&
  cargo test --workspace --locked` green. `git diff --check` clean; scope =
  `crates/ken-elaborator` (+ tests) only; **zero** kernel/prelude/semantics/
  Cargo/lock/`trusted_base()` delta (tool-internal formatter).

## Conformance companion (CV / enclave, parallel)

Per the Architect ruling, B3 needs **no spec sub-lane** (`§1d` is the single
normative home; a sub-lane would duplicate it and invite drift). What it needs is
a **conformance extension** owned by CV: for **each axis above**, confirm the
golden carries a **both-orientation output oracle** — an *input-in-alias-form →
canonical* case **and** a *canonical → canonical idempotence* case — extending
`seed-canonical-format.md`'s red-until-B3 cases. This is the determinism
discipline's home (semantic-preservation + idempotence alone cannot prove a
*unique* canonical output; each axis needs a keep/normalize direction pinned both
ways). Steward sequences this alongside the Language build; the build's AC6
depends on it.

## Review

**Architect-terminal** (he owns the kenfmt B-series contracts and authored this
design). Team QA runs the **three-gate property** (parse-preservation +
idempotence + 88-col width) over the whole catalog **and** the literal locked CI
as first-class gates. CV's `seed-canonical-format.md` (+ the per-axis oracle
extension) is the acceptance oracle: B3 is the producer that flips the layout
gate cases red→green.

## Do-not-reopen guardrails

- **Wadler/Leijen, one canonical layout** — no Prettier fill, no packing
  latitude, no width-tie choice; `group` is a binary pure function.
- **No global alignment** — `nest` enclosing-relative 2-space only.
- **Axes are `§1d`'s, not B3's** — implement the settled dispositions; do not
  re-decide blank-lines / separators / parens / alignment.
- **Parenthesis printer is the sharp edge** — parse-preservation + elaboration
  backstop over the whole catalog; adversarial coverage.
- **Comments never relocate across a boundary; a group with an attached comment
  never flattens.**
- **Read-only gate, not the reformat** — the one-time catalog reformat is
  capstone C; B3's gate runs read-only.
- **Consume B1 + B2 read-only** — no re-lex, no re-canonicalize, no lossless-
  layer change.
- **Surface/tool-internal only** — zero TCB delta.

## Notes

- **P0 (field separator) is not a B3 blocker.** `§1d` + grammar
  (`field (";" field)*`) + parser are settled on **semicolon-between /
  no-trailing** for declaration blocks. B3 builds to that; if P0's
  comma-unification ever flips it, that is a **one-token printer constant**, not
  a redesign. (Steward tracks P0's final status.)
- **Revisit-if (from the ruling):** an error-recovering CST landing (B3
  unaffected — targets the interface); `OQ-syntax` settling type-application
  spelling (B3 gains one canonicalization it currently preserves-as-parsed).
