# kenfmt B2 — where notation-role lives (lexer / parser / formatter)

- **Status:** **Accepted** (Architect ruling, 2026-07-12). Grounded entirely in
  the already-approved `spec/30-surface/31-lexical.md` §1a–§1d (WP S); needs no
  operator sign-off — it honors every locked constraint and only pins the layer
  each canonicalization belongs to.
- **Deciders:** the Architect. Framed against a grounded B2 seam question from
  the Language implementer.
- **Scope:** B2 (the formatter's token-kind canonicalization), the BL3/D4 lexer
  capability, and the §1d protected-word semantics. Tool-internal — **zero
  `trusted_base()` delta**.

## The question

B1 faithfully exposes `SourceToken.kind: lexer::Token`. But the **landed lexer
does not distinguish notation-role from identifier-role for word forms**:
`forall`/`exists`/`not` lex as ordinary `Ident`; `Omega`/`Sigma`/`Pi` as
ordinary `ConId`; `l`, `level`, and `ℓ` are indistinguishable at the token
level. So a formatter reading the B1 stream *by token kind* (the locked AC —
never by raw text) cannot both (a) preserve an identifier named `l`/`level`/
`not` and (b) glyph the same ASCII spelling *as notation* — the token kind is
identical in both roles. The implementer asked whether B2 should add a
parser-produced token-role overlay, or canonicalize only the already-distinct
tokens and defer the word aliases.

## Ruling: defer word-notation to the lexer (§1c BL3), do **not** add a B2 overlay

The spec already assigns the layer. Three families, three homes:

**Family A — operator/symbol digraphs, already distinct tokens today**
(`->`/`→`, `|->`/`↦`, `\`/`λ`, `<=`/`≤`, `>=`/`≥`, `/=`/`≠`, `===`/`≡`,
`><`/`×`, `<:`/`⊑`, `/\`, `\/`). These lex to distinct **operator** tokens now
— they are not identifiers. **B2 canonicalizes these by token kind
immediately.** This is the shippable subset.

**Family B — reserved notation *words*** (`forall`/`∀`, `exists`/`∃`,
`Sigma`/`Σ`, `Pi`/`Π`, `Omega`/`Ω`). §1c BL3 is explicit that accept-both /
same-token for these is **"genuinely a lexer capability, not only a
convention"** — its build scope (BL3/D4) is the lexer producing the notation
token, after which glyph-vs-spelling falls out of token kind. That capability
is **specified but not landed** for the word forms. **The fix is to land BL3/D4
in the lexer**, not to reconstruct role in B2. Once the lexer emits a notation
token for `forall`, B2's existing "print the token's blessed §1b glyph" covers
Family B for free.

**Family C — genuinely contextual, protected** (`l`/`level`→`ℓ`, `in`→`∈`,
`not`→`¬` in identifier/prose position). §1d resolves these conservatively:
they are **"never rewritten."** The identifier/keyword reading always wins the
ASCII spelling; the glyph forms are accepted as input and print as themselves.
**B2 simply never touches them** — no parser role machinery is required for the
formatter, because the spec chose protection over role-resolution here.

### Why not a B2 parser role-overlay (the rejected option)

- **Wrong layer.** §1c *normatively* makes notation-word recognition a lexer
  capability. A B2-side overlay does the lexer's specified job in the parser,
  duplicating role logic and diverging from the spec's stated division of
  labour (subsume-don't-proliferate).
- **Speculative.** Family A ships without it; Family C needs no role at all.
  Building the overlay now extends ahead of need (reflect-don't-extend) when the
  BL3/D4 lexer WP is the specified path.
- **B1 stays correct.** The enrichment is the lexer's `Token` enum gaining
  notation variants (BL3); B1's stream carries the richer kinds unchanged. No
  B1 rework, and B1's interface door (contract item 4) accommodates it.

## The sequencing correction to surface

The locked AC — *canonicalize by parsed token kind, never raw text* — is
**correct but presupposes §1c BL3**: notation words must lex to distinct tokens
for "by token kind" to reach them. BL3/D4's word-form lexing is unbuilt, so the
AC is simply not yet *reachable* for Family B — it is not wrong, its
precondition is unmet. Path to full notation canonicalization:

1. **B2 now** — canonicalize Family A by token kind; leave Families B and C
   untouched. Deterministic and idempotent (an untouched `forall` is stable).
   Conformance word→glyph cases for Family B stay **RED-UNTIL** the lexer lands.
2. **BL3/D4 lexer WP** (Language) — realize accept-both / same-token for the
   Family B words (`forall`/`exists`/`Sigma`/`Pi`/`Omega`), per §1c. `l`/`level`
   are explicitly *excluded* from the lexer same-token rule (Family C, §1d).
3. **Thin B2 follow-on** — with Family B now token-distinct, the existing
   glyph-for-notation-token rule covers them; no new formatter mechanism.

This is staged and non-blocking: each WP is independently shippable and green.
It touches no locked constraint (B2 canonicalizes only by token kind, never raw
text, and cannot over-fire on an identifier — the over-fire the whole token-kind
discipline exists to prevent). The only cross-team dependency is that **full
word-notation canonicalization is gated on the BL3/D4 lexer WP** — routed to the
Steward for scoping and the Language lead for the build.

## Revisit if

The team decides any Family C word (`in`, `not`) should be *reserved* notation
rather than protected — that would move it from Family C (§1d protect) to
Family B (§1c lexer token), and is a spec/language call, not a B2 call.
