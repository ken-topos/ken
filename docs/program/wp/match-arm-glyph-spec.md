# WP: match-arm separator `⇒`→`↦` — Spec enclave lane (grammar production + conformance fixtures)

**Owner:** Spec enclave — **spec-author** (grammar production) + **CV**
(conformance Ken fixtures); spec-leader coordinates. **Size:** S (~38 mechanical
Ken-source sites + one grammar production). **Risk:** low — pure surface syntax,
same AST, zero TCB. **Base:** `origin/main` **after `match-arm-glyph-lang` PR1
lands** (the additive lexer must accept `↦`/`|->` first). **Review:** Architect
surface + fidelity + Spec/CV; no soundness gate (Architect ruling
`evt_5m0aq3ddrxb7s`, lane split).

## Fixed inputs (settled — do NOT reopen)

Operator-decided, Architect-ruled. The **match-arm separator** becomes `↦`
(ASCII `|->`); `⇒`/`=>` retire. **`→`/`->` (function/implication arrow) is
unchanged.** Semantic, not textual, migration: migrate `⇒`/`=>` **only where it
is the match-arm token in Ken source**; **spec implication-prose `⇒` stays
untouched** (it is the meta-level implication glyph and remains correct — e.g.
Rust-doc / prose `¬¬φ ⇒ φ`). See the sibling `match-arm-glyph-lang.md` for the
full mechanism; this lane is source-only (no lexer/parser/formatter — that ships
in the Language lane's PR1, on which this WP depends).

## Scope / deliverables

1. **Surface-grammar production** — the spec's match-arm grammar rule
   (`spec/**`, the production that currently spells the separator `⇒`/`=>`):
   change the separator to `↦` (ASCII `|->`). Update any inline Ken **examples**
   in the grammar/spec prose that show a match arm. (~4 `⇒` in spec ```` ```ken ````
   fences + the production.)
2. **Conformance Ken fixtures** (`conformance/**`, CV lane) — migrate every
   **match-arm** `⇒`/`=>` in the `.ken` / `.ken.md`-fenced conformance fixtures
   to `↦`/`|->` (~34 sites). These are the executable challenge inputs; they
   must parse under the migrated surface.

**Never touch:** spec **implication-prose** `⇒` (meta-level, stays), any `→`,
any non-Ken text.

## Method

- Fence-/fixture-aware substitution (same discipline as the Language lane):
  migrate `⇒`→`↦`, `=>`→`|->` **only** inside Ken grammar productions,
  ```` ```ken ```` fences, and `.ken` fixtures. Audit the diff — zero
  implication-prose `⇒`
  changed.

## Acceptance criteria (testable)

1. The spec match-arm production reads `↦` (ASCII `|->`); its Ken examples use
   `↦`. Spec implication-prose `⇒` is unchanged (grep parity).
2. Every conformance fixture match arm migrated; **all conformance fixtures
   parse/elaborate** under the migrated surface (run the fixture elaboration
   net).
3. Zero TCB delta; `git diff --check` clean; no `spec/` normative *rule* changed
   other than the separator spelling.

## Sequencing (Steward-owned)

- **Starts only after** `match-arm-glyph-lang` PR1 merges (additive lexer
  accepts `↦`/`|->`) — else the migrated fixtures won't parse.
- This WP + the Language sweep are **both** prerequisites for the Language
  lane's **PR2 (removal of `⇒`/`=>`)**. The Steward gates that removal on both.

## Do-not-reopen guardrails

- Design/mechanism settled; migrate mechanically. `→` unchanged.
- Spec implication-prose `⇒` is **correct as-is** — do not migrate it.
