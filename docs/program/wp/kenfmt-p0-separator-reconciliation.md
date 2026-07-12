# WP P0 — field-separator reconciliation (kenfmt prerequisite)

Owner: **spec enclave** (spec-leader / spec-author / conformance-validator).
**Spec-only** — no build, no parser change (see grounding). Program:
`docs/program/wp/kenfmt-work-program.md`. Size **XS/S**. Deps: none. **Sequenced
first** in the kenfmt program — it settles the canonical field separator before
WP **S** writes the canonical-form rules that reference it. Base: `origin/main`
(re-verify cites at pickup).

## Why this is a prerequisite, not a formatting choice

The canonical formatter must emit *one* field separator for multiline
`record` / `class` / `instance` (and `law`) blocks. That separator is a
**grammar** fact, not a formatting preference — so it is settled here, in the
spec, before kenfmt canonicalizes onto it. Deciding it inside the formatter
would smuggle a grammar change into a formatting diff.

## Grounding — the divergence is a STALE EBNF, verified against landed code

Steward-verified on `origin/main`, three ways (do not re-derive from the review
prose — this is the ground truth):

- **EBNF (`spec/30-surface/32-grammar.md`) says COMMAS** for the field lists:
  - L21 `record ConId tyvar* "{" field ("," field)* "}"`
  - L24 `class ConId binder* "{" class_field ("," class_field)* "}"`
  - L25 `instance … "{" field_assign ("," field_assign)* "}"`
  - L232 `law ConId "(" tyvar ")" "{" field ("," field)* "}"`
- **The landed parser (`crates/ken-elaborator/src/parser.rs`) implements
  SEMICOLONS** and does **not** accept commas between fields:
  - `parse_class_decl` L737–739: after each field, `if matches!(peek,
    Token::Semicolon) { advance }` — semicolon-separated, trailing-tolerant.
  - `parse_instance_decl` L766–768: identical semicolon handling.
  - Docstrings L690/L752/L662 all show `{ field … ; … }`.
- **The corpus already uses SEMICOLONS**, no trailing `;` (e.g.
  `catalog/packages/Core/EmptyDec.ken.md`: `eq : a → a → Bool ;` … last field no
  `;`; `instance DecEq Bool { eq = bool_eq ; … }`).
- **The grammar is already internally split:** `data_block` (L47) and
  `prop_block` (L61) use `(";" …)*`. So the record/class/instance comma is the
  lone outlier — semicolons **unify** the grammar with itself *and* with the
  parser + corpus.

**Conclusion:** the parser and corpus agree on semicolons; the EBNF comma is the
single stale artifact. The reconciliation direction is unambiguous — **make the
EBNF semicolons** — and it is a **spec-only** edit (no parser touch, no corpus
migration).

## Fixed inputs — SETTLED, do not reopen

- **Canonical separator = semicolon** between fields/assignments in multiline
  `record` / `class` / `instance` / `law` blocks; **no trailing semicolon** after
  the last field (matches the parser's trailing-tolerant loop and the corpus).
- **Direction = correct the spec to the landed reality.** The parser + corpus are
  the ground truth; the EBNF is wrong and is what changes. Do **not** propose
  moving parser/corpus to commas.
- **Spec-only, zero build.** The parser already implements this; confirm-only
  that no parser edit is needed (it is not, per the grounding). No crates / kernel
  / prelude / Cargo / lock delta.

## Deliverable

### Spec — `spec/30-surface/32-grammar.md`

1. Change the record/class/instance/law field-list productions from
   `("," …)*` to `(";" …)*` (L21, L24, L25, L232 above), matching `data_block`
   / `prop_block`. Trailing-separator tolerance is a parser detail — the EBNF
   states the canonical `(";" field)*` form; note the last field carries no `;`.
2. **Sweep §32 (and any §33/§34 cross-reference) for residual comma-separated
   field-list prose/examples** and reconcile them to the semicolon form. A
   generalization that still says "comma-separated fields" is a fidelity bug.
3. **Do NOT touch the comma-separated lists that are genuinely commas** — record
   *literals* / *patterns* (L158/L218 `field_assign`/`field_pat`), tuples
   (L157), rows (`[FS, Console]` L55), `derive(…)` (L59), `import (a, b)` (L13),
   `where`-constraint lists (L37), `match` scrutinee list (L153). Those are not
   field-*declaration* blocks and stay comma-separated. **Scope is exactly the
   `record`/`class`/`instance`/`law` field-declaration blocks.**

### Conformance — `conformance/surface/…`

4. Confirm no conformance case asserts comma-separated `record`/`class`/
   `instance` fields (grep the surface seeds). If one does, reconcile it to
   semicolons. Expected: none (the corpus is already semicolons).

## Boundary note — what P0 does NOT do

- **No spacing/layout rule** — whether it is `Bool ;` or `Bool;` (the corpus has
  a space before `;`) is **kenfmt's** canonical-spacing decision in WP **S**,
  not P0. P0 fixes only the *separator token* in the grammar.
- **No parser change** — the parser already parses semicolons; touching it is
  out of scope (confirm-only).
- **No corpus reformat** — the corpus is already semicolons; the one-time
  space-normalization rides the kenfmt capstone, not P0.

## AC

- The four EBNF field-declaration productions read `(";" …)*`; the §32 sweep
  finds no residual comma-separated field-declaration prose/example.
- The genuinely-comma lists (literals, patterns, tuples, rows, derive, import,
  constraints, match) are **unchanged**.
- No conformance case depends on comma-separated fields (or is reconciled).
- Spec/conformance-only; **zero** crates / parser / kernel / prelude / Cargo /
  lock / `trusted_base()` delta.

## Review

Enclave gates (spec-leader scope/fidelity + CV conformance) then
**Architect-terminal** grammar-shape review (the EBNF↔parser fidelity call). Hand
the SHA to Steward; Steward publishes doc-only.

## Do-not-reopen guardrails

- Do **not** move the parser/corpus to commas — the spec is what is wrong.
- Do **not** decide separator *spacing* here — that is WP **S** (kenfmt canonical
  spacing).
- Do **not** widen the parser to accept commas — semicolon is the one canonical
  separator; comma between fields stays a syntax error.
- Do **not** touch the genuinely-comma lists enumerated above.
