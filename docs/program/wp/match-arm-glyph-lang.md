# WP: match-arm separator `⇒`→`↦` — Language lane (lexer/parser/formatter + catalog/prelude/test migration)

**Owner:** Language (lexer/parser/formatter + catalog/prelude/test). **Size:** M
(one small precise lexer/parser/formatter change + a large but mechanical,
fence-scoped source sweep). **Risk:** low — **pure surface syntax; same AST,
same elaboration, zero `ken-kernel`/TCB delta.** **Base:** `origin/main @
900c135f`. **Review:** Architect surface + fidelity + Language build; **no
soundness gate** (Architect ruling `evt_5m0aq3ddrxb7s`).

## Fixed inputs (settled — do NOT reopen)

Operator-decided design, Architect-ruled mechanism (`evt_5m0aq3ddrxb7s`). The
**match-arm separator** changes from `⇒` (ASCII `=>`) to **`↦` "maps to"**
(ASCII **`|->`**). Rationale: `⇒` is the conventional glyph for *implication*,
so `Zero ⇒ Proved` mis-reads as "Zero implies Proved" when it means "yields";
`↦` is the honest symbol for a case→value mapping.

- **The function/implication arrow `→` (ASCII `->`) is DELIBERATELY unchanged.**
  Implication and function-type are the same `Π` former under
  propositions-as-types; the operator explicitly rejected splitting one former
  across two glyphs. This WP touches **only** the match-arm punctuation.
- **Migrate-then-remove, no deprecation window** — see Phase structure. A
  lingering `⇒` defeats the "one honest surface" goal.
- **This is a semantic, not textual, migration.** `⇒`/`=>` map to the match
  token **only in Ken source**. The SAME glyph `⇒` is *implication-in-prose*
  (spec, `.ken.md` prose, Rust doc-comments) and `=>` is **Rust's own match
  arm** in `.rs` test files. A blind sweep would corrupt both. Every edit MUST
  be scoped to Ken-source positions (see Scope).

## Mechanism (ruled — implement as specified)

**1. Lexer (`crates/ken-elaborator/src/lexer.rs`).**
- `↦` — add a trivial single-codepoint arm → the match token (mirror the
  existing `'⇒'` arm at `lexer.rs:258`).
- `|->` — on `'|'` (currently `lexer.rs:206`, emits `Token::Pipe` with no
  lookahead), do a **2-char lookahead**: emit the match token only on exact
  `|->`, else fall through to `Token::Pipe` (mirror the existing `=`→`=>` /
  `-`→`->` peeks). **Maximal-munch is safe — grounded:** `Token::Pipe`'s three
  parser uses (data-ctor separator `parser.rs:976`, row/effect head-list
  `:1326`, refinement `{x:A | φ}` `:1424`) are each followed by a
  ConId/ident/expression, **none of which can begin with the infix `->`**, so
  `|->` never occurs in a valid program (confirmed: `git grep '|->'` = 0 hits).
- **Token rename:** `Token::FatArrow` → `Token::MapsTo` (the token enum +
  lexer emit sites + the parser consume site). **The parser match-arm production
  is otherwise unchanged**, so the AST and elaboration are byte-identical.

**2. Formatter (`crates/ken-elaborator/src/format.rs`) — ORDERING IS
LOAD-BEARING.** Add `rest.starts_with("|->") => ("↦", 3)` as an arm **BEFORE**
the `rest.starts_with("->") => ("→", 2)` arm at `format.rs:78` — otherwise
`|->` beautifies to `|→` (it hits `->` first). (The `=>` → `⇒` arm at
`format.rs:80` is dropped in Phase 2's removal — see below.)

**3. Additive first.** Phases 1's lexer/formatter change is **additive**: both
`⇒`/`=>` (old) and `↦`/`|->` (new) parse, and the formatter beautifies `|->`.
This lets the source sweep land while old spellings still work.

## Phase structure (this WP = two PRs; a third, gated, closes the migration)

- **PR1 — additive lexer/parser/formatter + Language-lane source sweep.**
  Ship the Mechanism §1–§2 change (additive), then migrate **every Ken-source
  match arm** in this lane to `↦`/`|->`. Both old and new parse, so the
  workspace stays green throughout.
- **PR2 (a separate, LATER PR — gated by the Steward, do NOT open until told).**
  Remove `⇒`/`=>` as match spellings from the lexer + the `=>`→`⇒` arm from
  `format.rs`. This is gated on **both** PR1 **and** the Spec-enclave lane
  (`match-arm-glyph-spec`: grammar production + conformance fixtures) being
  merged — otherwise removal breaks un-migrated spec/conformance sites. The
  Steward releases PR2 when both are in.

## Scope / deliverables (PR1)

**Migrate `⇒` AND `=>` (both are the match token) to `↦`/`|->`, Ken-source
positions ONLY:**
1. **`catalog/**/*.ken.md`** — inside ```` ```ken ```` (and `ken-repl` /
   `ken-error`) fenced blocks only. **~1,076 `⇒` + ~123 `=>` ≈ 1,200 sites.**
   Prose `⇒` **outside** fences (implication) stays. **Use a fence-aware
   script**, not hand edits (see Method).
2. **`crates/ken-elaborator/src/prelude.rs`** — the Rust-emitted Ken strings
   only (e.g. the `match d { Yes p ⇒ … }` emission near `prelude.rs:839`). ~2
   sites. **Do NOT touch Rust `match … =>` arms in this file.**
3. **`crates/**/tests/**/*.rs` and any `src` Ken snippets** — `⇒` inside raw
   Ken string literals (`r#"…"#` / inline). **~104 `⇒` sites.** A `⇒` in a
   `.rs` file is always inside a Ken string (Rust uses `=>`), so `⇒`→`↦` there
   is safe; but **`=>` in `.rs` is overwhelmingly Rust's own match arm — do NOT
   touch `=>` in `.rs` except inside a Ken string literal.** Migrate `=>`→`|->`
   only where it is demonstrably inside a Ken snippet.

**Out of scope (other lane):** `spec/**` grammar + examples, `conformance/**`
fixtures — those are `match-arm-glyph-spec` (Spec enclave). Do not touch them.

**Never touch:** implication-in-prose `⇒` (`.ken.md` prose outside fences, Rust
doc-comments like `¬¬φ ⇒ φ`), Rust `match … =>` arms, the `→`/`->` arrow.

## Method (mechanical, auditable)

- Write a **fence-aware / string-literal-aware** substitution script (extract
  ```` ```ken ```` fence bodies and Ken raw-string spans, substitute `⇒`→`↦`
  and `=>`→`|->` **within those spans only**, splice back). Do not run a blind
  `sed` over whole files.
- **Audit the diff:** `git diff` must show changes only inside fences / Ken
  string literals; **zero** changes to prose or Rust match arms. Spot-check a
  sample of each file class. A `git grep '⇒'` / `git grep '=>'` **outside**
  Ken-source positions after the sweep must equal the pre-sweep prose/Rust
  count (nothing migrated that shouldn't be).

## Acceptance criteria (testable, PR1)

1. Lexer accepts `↦`, `|->`, **and** (still) `⇒`, `=>` as the match separator;
   `Token::FatArrow` renamed `MapsTo`; parser arm + AST unchanged.
2. **Parse regression net:** `data X = A | B`, `{x : A | φ}`, and the
   row/effect head-list still parse; `Zero|->v`, `Zero |-> v`, and `Zero ↦ v`
   all parse as a match arm (add focused lexer/parser tests for each).
3. Formatter beautifies `|->`→`↦` (before `->`); add a formatter test.
4. Every Ken-source match arm in the lane migrated to `↦`/`|->`; **zero** prose
   `⇒` or Rust `=>` changed (diff audit + grep parity).
5. **Full `cargo test --workspace` green** (`scripts/ken-cargo` targeted for the
    elaborator; workspace in CI). Catalog acceptance nets green.
6. **Zero trust delta:** no `ken-kernel` change; `git diff --check` clean; no
   `Axiom`/`postulate`/`trusted_base` touched.

## Do-not-reopen guardrails

- Design is operator-decided, mechanism Architect-ruled. Don't relitigate the
  glyph, the ASCII form, or migrate-then-remove.
- `→`/`->` is **unchanged** — do not "unify" or touch the function arrow.
- **Do NOT open PR2 (removal)** until the Steward releases it (gated on the
  Spec lane).
- No compatibility alias survives past PR2 — the end state is a single surface.
