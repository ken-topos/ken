# kenfmt B1 — source-representation architecture ruling

- **Status:** **Accepted** (Architect ruling; operator sign-off 2026-07-12).
  B1 releases to build after WP S (the `31 §1a/b/c` canonical-form clause)
  lands — S is what the printers normatively target.
- **Date:** 2026-07-12
- **Deciders:** the Architect (ruling); the operator (sign-off); framed against
  `docs/program/kenfmt-canonical-form-review.md` and
  `docs/program/wp/kenfmt-work-program.md`.
- **Scope:** the lossless source representation B1 builds and B2–B4 (and any
  future LSP/refactor tooling) inherit. Tool-internal — **zero `trusted_base()`
  delta**; the formatter is not kernel TCB.

## The fork

B1 must give the formatter a **lossless** source representation (an AST-only
printer would drop or misplace comments — unacceptable). Two candidates:

1. a **lossless CST** — leaves are all tokens + trivia, with typed views into
   decls/types/patterns/exprs; or
2. the existing **AST + a complete ordered token/trivia stream + a deterministic
   comment-attachment algorithm** (leading/trailing/interstitial).

The readability review leans (1) as "safer for long-term parser/formatter/LSP/
refactoring reuse."

## Ruling: (2) — AST + token/trivia stream + deterministic attachment

I diverge from the review's lean, on cost, risk, and reflect-don't-extend
grounds — grounded in the landed parser, not in the abstract.

**The landed parser is already the machine (2) needs.**
`Parser::new(tokens: Vec<(Token, Span)>, src: String)`
(`crates/ken-elaborator/src/parser.rs`) consumes a **pre-lexed token+span
vector and retains the original source**. So:

- The **complete ordered token stream with spans already exists** — the parser
  is built on it. The lexer emits `(Token, Span)` per token
  (`lexer.rs:next_token`); trivia (whitespace, `-- …` line comments) is the
  **gap bytes between adjacent token spans in the retained `src`** — recoverable
  by a small lexer change (record the skipped spans) or a post-pass over
  `src`. No new parser.
- The **"typed views into decls/types/patterns/exprs" that B1's AC names are the
  existing semantic AST** — reused for free. Under (1) those views are new
  red-tree infrastructure to build.
- `format.rs` today is a **129-line raw-byte `canonical_unicode` normalizer**
  (the seed B2 replaces — and the source of the `l`/`level`→`ℓ` over-fire),
  **not** a large pretty-printer to preserve. B3 builds the document algebra;
  "don't fork CAT-5's printer" means reuse its glyph-choice table behind
  token-kind, which (2) does directly.

**(1) is expensive and risky against exactly this code.** A CST requires either
(a) refactoring the 2074-line parser the **whole elaborator/kernel pipeline
depends on** to emit a CST and derive the AST from it — a regression there
breaks everything downstream — or (b) a **second parser** that emits a CST
alongside the AST parser: two grammars to keep in lockstep forever
(subsume-don't-proliferate violation, divergence risk). Both buy a **long-term
reuse benefit with no current consumer**: there is no Ken LSP or refactoring
engine on the roadmap. *Reflect-don't-extend* says build the general
concrete-tree infrastructure when a real consumer needs it — not speculatively.

**The one genuine CST differentiator — error recovery — is deferred anyway.**
Formatting a syntactically-invalid `ken reject` block needs an error-recovering
parser/CST; the review itself rules "until that exists, use token-aware
formatting." So neither option error-recovers today; that capability, if it
becomes real, is the trigger to build the CST *then*, not now.

**(2)'s fidelity risk is bounded by the mandated gates.** The real hazard of (2)
is comment mis-attachment (an interstitial comment re-emitted in the wrong
place). But losslessness is **verified independently of the representation** by
B1–C's hard gates — idempotence (`fmt∘fmt == fmt`), trivia/literal preservation
(comment bytes identical), whole-catalog round-trip, and the ambiguity suite's
"comments between every pair of structural tokens." A mis-attachment fails one
of these. The gates make (2) safe; they do not depend on a CST. And a CST does
not eliminate comment-*placement* logic — even with comments as tree leaves, the
printer must still decide where to re-emit a comment attached to a node it
breaks across lines. The delta (1) buys here is smaller than it looks.

## The B1 contract (what (2) must deliver)

1. **A complete, ordered, gapless trivia stream.** Every byte of `src` is
   accounted for: `⋃ token-spans ∪ ⋃ trivia-spans == src`, contiguously. This is
   the losslessness invariant; assert it in B1.
2. **A deterministic, total comment-attachment.** Every comment gets **exactly
   one** home by a fixed rule (own-line-before → leading; same-line-after →
   trailing; between structural tokens mid-construct → interstitial on the
   enclosing node, keyed by byte offset). Total = no comment is ever dropped or
   left homeless.
3. **Typed views = the existing AST**, reused for precedence/grouping — never
   the *only* source representation (the trivia stream is co-authoritative).
4. **Expose the lossless layer as an interface**, not a concrete type — the
   printer (B3) consumes an abstract "formattable tree + attached trivia,"
   span-keyed to one source of truth (the byte offsets). This is the door-open
   clause: if a real error-recovering LSP/refactor consumer later justifies a
   CST, it backs the **same interface** — a swap, not a rewrite of B2/B3/B4.

## Fit with the locked constraints

- **Canonicalize by parsed token kind, not raw text** (locked): natural under
  (2) — the stream carries token *kinds* (`Token::MapsTo`, `Token::Colon`, …),
  one kind per accepted ASCII/Unicode alias. The printer chooses the canonical
  glyph per kind; identifiers/keywords print their stored spelling; literals
  print the preserved lexeme. This is exactly what fixes `l`/`level`→`ℓ`: an
  identifier token named `l` prints `l`; nothing canonicalizes an identifier to
  the level glyph.
- **Reuse CAT-5's printer, don't fork it**: B3's document algebra reuses
  `format.rs`'s glyph-choice table (moved behind token kind, B2) rather than
  rewriting it.
- **96-col code / 80-col prose, no escape hatch**: orthogonal — width lives in
  the B3 document algebra, which consumes the same representation either way.

## Revisit if

A concrete error-recovering **LSP or refactoring engine** lands on the roadmap:
that is the consumer whose absence makes (1) speculative today. Because B1
exposes the lossless layer behind an interface (contract item 4), a CST can then
implement it without reworking the printer or the canonicalizer.
