# Canonical formatter acceptance seed — WP S

These cases are the black-box acceptance oracle for the canonical formatter
specified by `31-lexical.md §1`. They cover the eight semantic gates that B3
through C must make green. They do not prescribe a formatter representation or
add grammar. B1's lossless source layer and B2's token-kind printer are
prerequisites; B3, B4, and C own the observable outputs below.

**Status.** Every case that invokes `ken fmt`, compares formatter bytes, or
classifies formatted line width is **RED-UNTIL-BUILT (B3/B4/C)**. Parse and
elaboration controls that do not invoke the formatter remain live. An
unparseable `ken ignore` or `ken reject` fence is deliberately exempt only from
structural layout: B2 token-kind canonicalization still applies where lexing
succeeds, and the original body layout otherwise remains byte-identical.

The formatter is syntax-aware. Assertions compare parsed token roles, ASTs,
surface-to-core results, protected payload bytes, and masked prose bytes; raw
substring replacement is never an acceptable witness.

---

## FMT1 — byte fixed point (gate 1)

### surface/formatting/canonical-form-is-idempotent (property)

- spec: `31 §1a` (one canonical form), `31 §1d` (deterministic grouping and
  layout), WP S gate 1
- given: each fixture in FMT2–FMT8, including a long declaration, a broken
  arrow chain and application, nested matches, every protected literal form,
  interstitial comments, and all four literate fence roles
- expect: **RED-UNTIL-BUILT (B3/B4/C)** — byte-for-byte
  `fmt(fmt(source)) == fmt(source)`. The comparison includes final newline,
  blank lines, comment placement, fence markers, and Markdown outside fences.
- why: a formatter that oscillates between flat and broken groups, relocates a
  comment on each pass, or repeatedly rewrites a fence marker can satisfy
  parse preservation while failing to define one canonical form. Byte identity
  is the non-degenerate observable.

---

## FMT2 — parse preservation (gate 2)

### surface/formatting/layout-preserves-parsed-program (property)

- spec: `31 §1a`/`§1c` (accepted aliases are the same token), `31 §1d`
  (formatting is not refactoring), WP S gate 2
- given: one parseable unit containing declarations, grouped binders, an open
  effect row, contracts, refinements, a record/class/instance block, a
  two-arm nested match, a lambda, `let`, `if`, a projection, a qualified path,
  and an attached-proof selector; format it once
- expect: **RED-UNTIL-BUILT (B3/C)** — parsing before and after yields equal
  ASTs after erasing spans and trivia and identifying only the sanctioned
  ASCII/Unicode aliases. Declaration and arm order, binder grouping,
  parentheses required by precedence, literal lexemes, type-application form,
  and attached-proof spelling are unchanged.
- why: the equality excludes exactly trivia and same-token notation. A printer
  that sorts, regroups, desugars, changes `Equal` to `==`, changes bracketed
  type application, or switches proof-reference form changes the compared AST
  and fails even when both outputs parse.

### surface/formatting/parse-control-same-ast-different-layout (control)

- spec: `31 §1c`, `32` (existing grammar)
- given: two unformatted parseable sources differing only in whitespace and
  accepted ASCII/Unicode aliases
- expect: both parse now and their ASTs are equal under the same comparison
  used above; this control is **LIVE** and does not invoke `ken fmt`
- why: separates a formatter failure from an absent parser capability and
  pins the sanctioned equivalence relation used by FMT2.

---

## FMT3 — elaboration preservation (gate 3)

### surface/formatting/layout-preserves-elaborated-core (property)

- spec: `31 §1d` (semantic preservation), `39` (surface-to-core
  elaboration), WP S gate 3
- given: a closed module whose name resolution depends on source order and
  contains a qualified import, a local binding, an instance constraint, a
  projection, and an attached-proof reference; elaborate the original and its
  formatted output under the same roots and entry unit
- expect: **RED-UNTIL-BUILT (B3/C)** — both elaborate successfully to the
  byte-identical stable core serialization and identical `trusted_base()`;
  resolved `GlobalId`s and declaration order are identical
- why: AST equality alone can miss a resolution, fixity, or source-order bug.
  The stable core result is structural: a formatter that reorders imports,
  fields, instances, or declarations cannot pass merely because both sources
  remain well-typed.

---

## FMT4 — whole-catalog posture (gate 4)

### surface/formatting/whole-catalog-preservation-and-fixed-point (property)

- spec: `31 §1a` (mandated formatter), `31 §1d`, WP S gate 4
- given: every `.ken` file and every parseable recognized Ken fence in the
  repository catalog, with no sampling and no allow-list of known hard files
- expect: **RED-UNTIL-BUILT (C)** — each unit passes FMT1, FMT2, FMT3 wherever
  stable core comparison is available, FMT6, and FMT7. Every parseable `ken`,
  `ken example`, `ken ignore`, and `ken reject` body is included according to
  its role; deliberately invalid bodies use the narrow FMT8 exemption.
- why: the catalog's long telescopes, nested proofs, comments, and literals are
  the formatter's real domain. A representative sample can be green while an
  unvisited production silently loses syntax or trivia.

---

## FMT5 — literate prose identity (gate 5)

### surface/formatting/literate-prose-is-byte-identical (property)

- spec: `31 §1d` (literate canonical form), WP S gate 5
- given: a `.ken.md` document with non-ASCII prose, trailing prose spaces,
  blank lines, inline code containing `->`, `|->`, `l`, `level`, and `in`, an
  unrecognized fenced language, and each of the four recognized Ken roles
- expect: **RED-UNTIL-BUILT (B4/C)** — mask each recognized fence from opener
  through closer, concatenate the remaining byte ranges, and compare them with
  the corresponding input ranges: they are byte-identical. Only recognized
  fence markers and bodies may differ. Adjacent fences are not joined or
  moved, and roles do not change.
- why: ordinary prose contains bytes that resemble Ken aliases. Comparing the
  masked byte ranges catches a raw global canonicalizer or Markdown reflow;
  comparing only rendered text would not.

---

## FMT6 — comments and protected payloads (gate 6)

### surface/formatting/comments-retain-text-and-attachment (property)

- spec: `31 §1d` (comment preservation and attachment), WP S gate 6
- given: a doc comment before a declaration, a leading comment before a match,
  an end-of-line comment that fits, one that cannot fit, and a distinct comment
  between every adjacent pair of structural token classes across the fixture
- expect: **RED-UNTIL-BUILT (B3/C)** — every comment's text bytes are
  identical except trailing horizontal whitespace; doc and leading comments
  remain attached to the same AST node; a fitting EOL comment stays inline and
  the non-fitting one moves immediately above its same node. Each interstitial
  comment forces its containing group to break and crosses no token boundary.
- why: comment presence alone is green-vs-green under misattachment. Node
  identity plus exact text and relative token interval make relocation
  observable.

### surface/formatting/all-literal-lexemes-are-verbatim (property)

- spec: `31 §1b`/`§1d` (token-kind canonicalization and protected regions),
  `31 §3` (literal forms), WP S gate 6
- given: one parseable fixture containing every literal category and spelling
  distinction: integers `1_000`, `0xFF`, `0b1010`, `0o17`; decimal
  `1_000.00d`; floats `1e-9`, `0x1p-3`; ordinary string, raw/multiline string,
  char, escaped char, `b"..."`, and `0x[...]` bytes; and both booleans. Every
  text-capable payload contains as many complete alias byte sequences as its
  grammar admits, including `->`, `|->`, `\\`, `forall`, `exists`, `Sigma`,
  `Pi`, `Omega`, `===`, `<=`, `>=`, `/=`, `not`, `/\\`, `\\/`, `in`, `<:`,
  `><`, `level`, and `l`. Foreign symbol/library strings, temporal formula
  text, line/block/doc comments, and Markdown prose carry the same alias set.
- expect: **RED-UNTIL-BUILT (B2/B3/B4/C)** — every literal and verbatim payload
  source lexeme is byte-identical after formatting: base, separators, suffix,
  exponent, delimiter, escape spelling, and payload all survive. Alias-looking
  bytes inside any protected region are not converted to glyphs.
- why: this exercises each literal form independently. Testing only an ordinary
  string would leave raw/multiline strings, chars, bytes, numeric spellings,
  comments, foreign names, and temporal payloads unguarded.

---

## FMT7 — deterministic 88-column property (gate 7)

### surface/formatting/breakable-syntax-never-exceeds-88-columns (property)

- spec: `31 §1d` (88 display columns, two-space indentation, deterministic
  group breaking), WP S gate 7
- given: paired fixtures at display widths 88 and 89 for a declaration header,
  arrow chain, application, match arm, effect row, contract, refinement, and
  record/class/instance field; include Unicode glyphs whose UTF-8 byte length
  differs from display width
- expect: **RED-UNTIL-BUILT (B3/C)** — the 88-column form remains flat when its
  group fits; adding the one display-column token makes that same breakable
  group choose its specified multiline layout. Every output line over 88 is
  classified by a span wholly containing one indivisible identifier/literal or
  a specified verbatim region; no line exceeds 88 solely because breakable
  syntax was left flat. Indentation is two ASCII spaces per level, never tabs.
- why: the 88/89 pair fixes both boundary orientation and display-width
  counting. A byte-counting implementation or a vague best-effort wrapper
  flips one arm and fails.

---

## FMT8 — token-role ambiguity and literate boundary (gate 8)

Each pair below holds surrounding syntax fixed and changes only the token role
under test. The expected formatted tokens are structural outputs; merely
accepting both arms is insufficient.

### surface/formatting/function-arrow-and-match-arrow-stay-distinct (ambiguity)

- spec: `31 §1b`/`§1d`, `32` (`type` arrow and `arm` match arrow)
- given: an ASCII function type `A -> B` and a match arm `Some x |-> x`
- expect: **RED-UNTIL-BUILT (B2/B3/C)** — output contains `A → B` and
  `Some x ↦ x`; neither token is printed as the other
- why: longest-token and parsed-role discrimination. A raw `->` pass can
  corrupt `|->` while still producing arrow-looking text.

### surface/formatting/binding-colon-and-attached-selector-stay-distinct (ambiguity)

- spec: `31 §1d`, `32` (`:` and `::`)
- given: `(x : A)` adjacent to the reference `subject::proof_name`
- expect: **RED-UNTIL-BUILT (B3/C)** — binding spaces around `:`, while `::`
  remains attached with no spaces; token count and roles are unchanged
- why: a punctuation pass that handles `:` before `::` changes the selector or
  inserts spaces that split it.

### surface/formatting/projection-and-qualified-path-keep-their-ast-roles (ambiguity)

- spec: `31 §1d`, `32` (`expr . ident` and qualified `path`)
- given: the same spelling `M.value` once resolved as a module-qualified path
  and once as field projection from local `M`, in otherwise identical calls
- expect: **RED-UNTIL-BUILT (B3/C)** — both print without spaces around `.`,
  and parsing the outputs preserves their distinct AST/resolution roles and
  `GlobalId`/projection targets
- why: spelling identity is not role identity. The structural comparison
  catches a printer/parser path that silently reparses every dot as one class.

### surface/formatting/l-identifier-is-not-a-level-token (ambiguity)

- spec: `31 §1b`/`§1d` (token-kind rule closes the `ℓ` overload)
- given: `fn keep_l (l : Nat) : Nat = l` beside a genuine level-token fixture
  using the canonical level role
- expect: **RED-UNTIL-BUILT (B2/B3/C)** — both identifier occurrences remain
  the stored spelling `l`; only the parsed level token prints `ℓ`. Repeat with
  an identifier named `level`, which also remains `level`.
- why: this is the direct raw-byte over-fire discriminator: the buggy
  canonicalizer changes the binding and its use, while the correct token-kind
  printer changes only the level token.

### surface/formatting/in-keyword-and-membership-token-stay-distinct (ambiguity)

- spec: `31 §1b`/`§1d`, `32` (`let ... in ...` and membership `∈`)
- given: a `let x = value in body` expression beside membership written with
  its accepted ASCII alias in an otherwise fixed proposition
- expect: **RED-UNTIL-BUILT (B2/B3/C)** — the keyword remains ASCII `in`; the
  parsed membership operator prints `∈`
- why: the same input bytes occupy opposite token roles. Replacing every `in`
  either corrupts the keyword or fails to canonicalize membership.

### surface/formatting/lambda-and-dependent-arrow-remain-distinct (ambiguity)

- spec: `31 §1b`/`§1d` (canonical lambda `λ ... .`), `32`
- given: ASCII lambda `\\x. x` beside dependent arrow `(x : A) -> B x`
- expect: **RED-UNTIL-BUILT (B2/B3/C)** — `λx. x` and
  `(x : A) → B x`; neither construct is desugared into the other
- why: pins the S-owned lambda resolution and the distinct arrow role.

### surface/formatting/ascription-binder-fixity-and-associativity-survive (ambiguity)

- spec: `31 §1d`, `32` (precedence, binder lookahead, fixity)
- given: an expression ascription and dependent binder sharing `:`, a
  right-associated arrow chain, a left-associated application, arithmetic
  precedence, and a user-declared fixity expression
- expect: **RED-UNTIL-BUILT (B3/C)** — formatted output reparses to the exact
  same tree for each construct, inserting mandatory-clarity parentheses where
  needed but never changing grouping
- why: catches a pretty-printer that preserves tokens yet changes the parse at
  a line break or precedence boundary.

### surface/formatting/four-fence-roles-and-narrow-exemption (ambiguity)

- spec: `31 §1d` (the four literate roles and narrow exemption), WP S gate 8
- given: one `.ken.md` document with (a) parseable `ken`, (b) deliberately
  incomplete `ken ignore`, (c) deliberately syntax-erroring `ken reject`, and
  (d) parseable runnable `ken example`; all openers begin noncanonically but
  retain their exact role, and every body contains an accepted ASCII alias
- expect: **RED-UNTIL-BUILT (B2/B4/C)** — all four openers/closers become the
  canonical markers at column zero without changing role. The parseable `ken`
  and `ken example` bodies receive full structural layout. The unparseable
  `ignore` and `reject` bodies receive token-aware canonicalization only where
  tokens are recognized and otherwise retain body layout byte-for-byte.
  Markdown prose passes FMT5. A parseable `ignore` or `reject` body is formatted
  structurally; the exemption follows actual parse failure plus the eligible
  role, not role alone.
- why: the last sentence is the boundary discriminator. Exempting every
  `ignore`/`reject` fence under-formats valid code; attempting AST layout on an
  invalid body rejects the document. Holding body fixed and varying
  parseability makes either over-broad interpretation observable.

---

## Coverage map

| Gate | Acceptance home | Build gate |
|---|---|---|
| 1. Idempotence | `canonical-form-is-idempotent` | B3/B4/C |
| 2. Parse preservation | `layout-preserves-parsed-program` | B3/C |
| 3. Elaboration preservation | `layout-preserves-elaborated-core` | B3/C |
| 4. Whole catalog | `whole-catalog-preservation-and-fixed-point` | C |
| 5. Prose identity | `literate-prose-is-byte-identical` | B4/C |
| 6. Trivia/literals | `comments-retain-text-and-attachment`, `all-literal-lexemes-are-verbatim` | B2/B3/B4/C |
| 7. Width | `breakable-syntax-never-exceeds-88-columns` | B3/C |
| 8. Ambiguity | all FMT8 cases | B2/B3/B4/C |

## Cross-case consistency

- FMT2 and FMT3 use the same formatted unit: AST preservation and core
  preservation are independent requirements, not substitutes.
- FMT5 and the literal/protected-payload case compare disjoint byte ranges:
  Markdown prose outside fences versus lexemes and trivia inside recognized
  Ken regions. Together they rule out both global and in-language raw-byte
  rewriting.
- FMT7 permits an over-width line only when one classified indivisible or
  verbatim span itself forces it. That exception is semantic, not a formatter
  escape hatch, and cannot exempt surrounding breakable syntax.
- Every FMT8 ambiguity is a controlled pair. Correct and buggy printers produce
  different token kinds, ASTs, resolved targets, or bytes; none is a
  green-vs-green acceptance-only claim.
- The four-fence case is the sole structural-layout exemption. It is gated by
  both eligible role and actual parse failure; it never weakens token-aware
  canonicalization or prose identity.
