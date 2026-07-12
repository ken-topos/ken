# WP S — canonical-form spec clause (kenfmt) · bundles P0

Owner: **spec enclave** (spec-leader / spec-author / conformance-validator).
**Spec + conformance only — no build** (the build is B1–B4/C). Design source of
truth: `docs/program/kenfmt-canonical-form-review.md` (the 12 rule sections + the
8 semantic gates, on `origin/main`) + `docs/program/wp/kenfmt-work-program.md`.
Program: the kenfmt work program. Size **M**. **Bundles WP P0**
(`docs/program/wp/kenfmt-p0-separator-reconciliation.md` — the EBNF separator
fidelity fix). Base: `origin/main` (re-verify cites at pickup).

## Scope — EXTEND the existing canonical-notation section, don't restart it

`spec/30-surface/31-lexical.md §1` **already** mandates the formatter and specs
the **token/glyph** half of the canonical form (Steward-verified on main):

- **§1a** ("read-optimized canonical Unicode", `OQ-syntax` DECIDED) — principle 3:
  "a single mandated **gofmt-style formatter** normalizes ASCII→Unicode and fixes
  layout on save"; principle 4: keywords stay ASCII; principle 5:
  confusable-resistant (TR39).
- **§1b** — the blessed glyph↔ASCII table (`→`/`->`, `λ`/`\`, `Ω`/`Omega`, …),
  with the **`ℓ` overload (level vs. …) flagged as an OPEN team call** — this is
  the `l`/`level` ambiguity the raw-byte `canonical_unicode` over-fires on.
- **§1c** (BL3 / SURF-1 D3) — "the canonical Unicode surface is lexer **and**
  formatter"; the formatter emits canonical Unicode on save; keywords exempt;
  TR39 confusables normalized/rejected.

So the **token/glyph canonicalization is already normative.** What §1 does **not**
yet specify is the **canonical LAYOUT** — width, indentation, line-breaking,
spacing, and the per-construct block forms. **That gap is what S fills**, plus
three reconciliations. S is additive to §1; it does not rewrite §1a/b/c's
settled principles.

## Fixed inputs — SETTLED, do not reopen

- **88 display columns for code / 80 for prose** (operator, 2026-07-12); soft
  pretty-printer width, deterministic (a group fits or breaks).
- **No escape hatch** (operator) — no `fmt:off`; verbatim regions are semantic
  exceptions, not style escapes.
- **Canonicalize by parsed token kind, never raw text** (locked) — the normative
  rule that resolves the §1b `ℓ` open note (an identifier token `l` prints `l`;
  nothing canonicalizes an identifier to the level glyph).
- **Formatting is not refactoring** — the review's "what the formatter must not
  do" list is normative (no reorder/regroup/desugar/kind-switch/add-remove).
- **The review is the content source of truth** — S **transcribes** its 12 rule
  sections into normative spec text; it does not re-derive or dilute them.
- **P0 bundled** — the separator fidelity fix (EBNF `("," field)*` → `(";" …)*`
  for record/class/instance/law field-decl blocks) rides in **this** candidate;
  the parser + corpus already use semicolons (see the P0 frame's grounding).

## Deliverable (mandated outline; each item ends in a concrete edit)

### Spec — `spec/30-surface/31-lexical.md` (extend §1) + `32-grammar.md` (P0)

1. **New normative subsection: canonical LAYOUT** (place as `§1d` or extend
   `§1c`, enclave's call on chapter structure). Transcribe the review's layout
   rules (its §§1, 3–12) as normative spec:
   - **Width/physical text** (review §1): 88 cols display-width; 2-space indent,
     no tabs; trailing-whitespace removal; LF; final newline; blank-line rules.
   - **Spacing** (review §3): infix/`=`/`:`/match-arrow/`as`/guard-`if`/row-`|`
     spacing; none inside `()`/`[]`/braces; comma spacing; **no vertical
     alignment ever**; **the separator (`;`) is attached to the preceding
     field/arm, no space before** (this is where the corpus's `Bool ;` → `Bool;`
     spacing is settled — the P0 grammar fix set the token; S sets its spacing).
   - **Declarations** (review §4): break by proof-review structure (name/subject,
     one binder group per line, `:` result line, `visits`/`requires`/`ensures`/
     `where` on their own lines, `=` at clause-end, body indented).
   - **Types** (review §5): one domain per line in a broken arrow chain,
     arrow-led continuations; break by argument boundary; mandatory-clarity
     parenthesization; never `Equal`↔`==`.
   - **Applications/projections** (review §6): head on line 1, one argument per
     continuation line; keep projection/attached-proof selector with its head.
   - **Lambdas/`let`/`if`** (review §7): coalesce nested lambdas only when AST-
     identical + no comment; multiline `let`/`if` forms.
   - **Matches** (review §8): `match e {}` empty-eliminator; any ≥2-arm match
     multiline; one arm per line; nested match always compound; `;` between arms,
     none after last; no arrow alignment.
   - **Blocks** (review §9): short nullary sums flat; else one ctor per line with
     leading `|`; nonempty record/class/instance/space/policy/module multiline
     one-per-line; empty `{}`; preserve order; never sort.
   - **Effects/contracts/refinements/FFI** (review §10): row spelling `[FS,
     Console | e]`; `visits`/`requires`/`ensures` one per line, source order;
     refinement `{x : A | φ}` spacing; `foreign` sig-then-body, foreign strings
     verbatim; verbatim bodies untouched.
   - **Comments** (review §11): preserved exactly (bar trailing WS); attachment
     (doc→following decl; leading→above at node indent; EOL inline only if it
     fits 88, else moved above); a comment between tokens forces the group to
     break.
2. **Token-kind canonicalization made normative** (review §2) — state that
   canonicalization is driven by parsed **token kind**, and **close the §1b `ℓ`
   overload open note**: identifiers/keywords print their stored spelling; only
   operator/notation token kinds map to the blessed glyph; **`l`/`level`/`in`/
   `not` are never canonicalized by byte resemblance**. Protected regions
   (strings/raw/multiline/chars/bytes/comments/doc-comments/temporal formula
   text/foreign names) are never rewritten.
3. **`.ken.md` canonical form** (review §12): the four recognized fence roles
   (`ken`/`ken ignore`/`ken reject`/`ken example`); canonical openers/closers;
   format recognized bodies in place; **Markdown prose byte-identical**; the
   **narrow, explicit fence-role exemption** for `ignore`/`reject` (token-aware
   canonicalization only, no structural layout on deliberately-incomplete/
   erroring fragments).
4. **P0 (bundled):** apply the `32-grammar.md` EBNF fix per the P0 frame
   (comma→`;` for record/class/instance/law field-decl blocks; leave the
   genuinely-comma lists untouched; §32 sweep).
5. **S-owned open-point resolutions** (record normatively):
   - **Layout vs braces →** canonical **explicit braces** now (already the
     grammar base; revisit only via a language decision).
   - **Lambda surface →** emit canonical `λ … .`; ASCII forms input-only.
   - **Type application →** **preserve as parsed** (juxtaposition vs brackets is
     under `OQ-syntax`; no forcing until it settles).

### Conformance — `conformance/surface/…`

6. **CV golden for the 8 semantic gates** as laws (extend the CAT-5 AC6
   idempotence law to the real grammar): idempotence (`fmt∘fmt == fmt`),
   parse-preservation (AST-equal mod trivia/sanctioned aliases),
   elaboration-preservation (where a stable comparison exists), whole-catalog
   coverage posture, prose byte-identity, trivia/literal preservation, the 88-col
   width property, and the **ambiguity suite** (match-arrow vs arrow; `:` vs `::`;
   projection vs qualified path; `l`-ident vs level; `in`-keyword vs membership;
   aliases inside every literal form; all four fence roles). Encode these as the
   acceptance oracle B3–C must satisfy; the golden is **red-until-built** where it
   asserts formatter output (mark per the F3b convention).

## Boundary — S is SPEC + GOLDEN only

- **No formatter build** — B1 (lossless layer, Architect-ruled AST+trivia), B2
  (token-kind canon replacing `canonical_unicode`), B3 (document algebra +
  printers), B4 (`.ken.md` splicing), C (capstone) are the build. S writes the
  normative form + the acceptance golden they target.
- **No new language/grammar surface** beyond the P0 separator fidelity fix — S
  canonicalizes the *existing* surface; it does not add forms.
- **Do not touch §1a/b/c's settled principles** — S is additive (layout +
  token-kind normativity + the three resolutions), not a rewrite.

## AC

- §1 carries a normative **canonical layout** subsection transcribing review
  §§1,3–12; the 88-col width is stated as a soft-but-deterministic rule.
- Token-kind canonicalization is normative and the §1b `ℓ` overload open note is
  **closed** (identifier `l` stays `l`).
- `.ken.md` canonical form + the narrow `ignore`/`reject` fence-role exemption are
  specced; prose byte-identity is normative.
- The three S-owned resolutions (braces-now, lambda Unicode+dot, type-app
  preserve) are recorded; type-application is explicitly **not** forced.
- **P0 folded**: `32-grammar.md` field-decl separators read `(";" …)*`; the
  genuinely-comma lists are unchanged; §32 sweep clean.
- The CV golden encodes the 8 gates + ambiguity suite as the build's acceptance
  oracle (red-until-built where it asserts output).
- Doc/spec/conformance-only; **zero** crates / kernel / prelude / Cargo / lock /
  `trusted_base()` delta.

## Review

Enclave gates (spec-leader scope/fidelity + CV conformance) then
**Architect-terminal** review (he authored the B1 ruling and will take the S
candidate — `31 §1a/b/c` + golden + bundled P0 — through the normal flow, per
`evt_wccj5th62wsn`). Hand the SHA to Steward; Steward publishes doc-only.

## Do-not-reopen guardrails

- **No escape hatch, no config** — one canonical form.
- **No literal normalization** — numeric/string/foreign/temporal payloads stay
  as-authored (a later, separate decision).
- **No sorting** of imports/constraints/rows/fields/instances — source order is
  resolution-relevant.
- **Do not re-decide the separator token** — P0 settled it (semicolons); S only
  sets its *spacing*.
- **Do not force type-application spelling** — preserve-as-parsed until
  `OQ-syntax`.
- **B1 architecture is settled** — AST + trivia stream (Accepted); S does not
  spec the representation, only the output form.
