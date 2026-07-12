# Ken canonical formatter: readability review and proposed rules

Date: 2026-07-12

## Executive recommendation

Ken should adopt a syntax-aware, lossless, deterministic formatter with these
headline choices:

1. Use a soft limit of **88 display columns**, with a two-space indent and no
   tabs. A breakable syntactic group must break before it exceeds 88 columns;
   only indivisible literals, identifiers, and verbatim regions may exceed it.
2. Emit the canonical Unicode operator vocabulary required by
   `spec/30-surface/31-lexical.md`, but canonicalize by **parsed token kind**,
   never by raw text substitution.
3. Keep explicit braces as the canonical block representation until the open
   layout-versus-braces decision is settled. Use multiline braces for all
   nontrivial blocks.
4. Break declarations by their proof-review structure: name, telescope,
   result, effects/contracts/constraints, then body. Once a telescope breaks,
   put one binder group on each line.
5. Make any match with more than one arm multiline. Compound arm bodies get
   their own indented line. Never column-align arrows, colons, or equals signs.
6. Break applications by argument boundaries, and broken arrow types by domain
   boundaries. Never wrap a term at arbitrary token positions.
7. Preserve declaration order, binder grouping, comments, literal spelling,
   and every choice that is not explicitly designated as canonical notation.
   Formatting must not become refactoring.
8. For `.ken.md`, change only recognized Ken fence bodies and canonical fence
   markers. Markdown prose must remain byte-for-byte identical.

The implementation should use a lossless concrete syntax tree, or an AST paired
with a complete token/trivia stream, and a Wadler/Prettier-style document
algebra. The current `canonical_unicode` function is a useful migration seed,
not a suitable foundation for the full formatter.

## Sources reviewed

The primary design constraints come from:

- `docs/PRINCIPLES.md`, especially agent-writes/human-reads, predictability,
  honesty, and the rule that required facts live in checked Ken rather than
  comments;
- `spec/30-surface/31-lexical.md` and `32-grammar.md`;
- the declaration, match, effect, verification, and FFI forms in surface and
  verification chapters;
- `spec/90-open-decisions.md`, where the canonical-Unicode and single-formatter
  principles are decided but some exact spellings and layout remain open;
- `docs/program/07-catalog-style-guide.md` and the four literate fence roles;
- every `.ken.md` file under `catalog/`, with closer inspection of the large
  proof developments and the surface/proof guides; and
- the already-framed `docs/program/wp/ken-formatter-canonical.md`.

This proposal fills the WP's canonical-form seam. It does not propose a new
language feature.

## What the catalog says about the problem

The current catalog contains 15,695 lines across 14 `.ken.md` files. Looking
only inside all four kinds of Ken fence gives 11,667 code lines:

| Measure | Count |
|---|---:|
| Lines over 80 columns | 2,037 |
| Lines over 100 columns | 1,039 |
| Lines over 120 columns | 638 |
| Lines over 160 columns | 213 |
| Lines over 200 columns | 80 |
| Maximum observed line | 353 columns |
| Dense one-line multi-arm matches | about 130 |
| Lines containing tabs | 13 |

The hardest material is not ordinary application code. It is precisely the
material a reviewer most needs to inspect carefully: long dependent
telescopes, equality goals, class-law fields, nested congruence/transitivity
proofs, and matches that establish exhaustive cases.

Several good idioms are already visible and should be made uniform:

- two-space indentation;
- a declaration body indented two spaces after `=`;
- top-level blank lines separating definitions;
- multiline `match` blocks for substantive case analysis;
- result types beginning on a continuation line for long proof declarations;
- one proof step per line in many of the better-formatted proofs; and
- narrative in Markdown rather than comments inside the checked fence.

The recurring readability failures are also clear:

- an entire 10--14 binder telescope on one line;
- nested matches compressed into one line;
- high-arity applications with no visible argument boundaries;
- chained lambdas such as `lambda y. lambda z. lambda p. ...` rendered as a
  wall of punctuation;
- ad hoc visual alignment that creates large gaps and churns when a sibling
  name changes; and
- inconsistent indentation, including tabs, in a few complex proofs.

## Canonical rules

### 1. Width, indentation, and physical text

- Target **88 Unicode display columns**. Count display width, not UTF-8 bytes.
  Canonical glyphs such as `lambda`, arrows, and Omega each occupy their normal
  terminal display width.
- Use exactly two ASCII spaces per indentation level. Tabs are forbidden.
- Remove trailing whitespace, use LF line endings, and end every file and Ken
  fence body with one newline.
- Use one blank line between top-level declarations. Do not insert blank lines
  between arms or fields of one syntactic block.
- Preserve at most one intentionally separated blank line around an attached
  comment. The formatter should otherwise own vertical whitespace.
- There is no formatter-disable directive. A mandated format with escape
  hatches is no longer one canonical format. Verbatim literals and explicitly
  verbatim language constructs are semantic exceptions, not style escapes.

Why 88 rather than 80: the repository's prose target is 80, but dependent type
applications contain more unavoidable punctuation and explicit type arguments
than prose. Eight extra columns materially reduce vertical explosion while
remaining comfortable in a side-by-side review. A 100- or 120-column target
would leave much of the observed proof density intact. The 88-column rule is a
soft width in the pretty-printer sense, but deterministic: a group either fits
or it breaks.

### 2. Canonical tokens and protected regions

- Emit the blessed Unicode spelling for operators and mathematical notation:
  for example, `->` becomes the canonical arrow, `|->` becomes the canonical
  match arrow, and ASCII lambda input becomes the canonical lambda glyph.
- Keep keywords as ASCII words: `const`, `fn`, `proc`, `data`, `match`,
  `requires`, `ensures`, `proof`, and so on.
- Canonicalize only after lexing and parsing has established token role. An
  identifier spelled `l`, the keyword `in`, and ordinary prose containing
  `not` must never be converted merely because their bytes resemble an alias.
- Do not rewrite inside strings, raw or multiline strings, chars, bytes,
  comments, doc comments, temporal formula text, foreign symbol/library names,
  or any other verbatim payload.
- Reject unblessed confusable identifier characters at the lexer boundary.
  Do not silently "repair" an unknown identifier into a different binding.
- Preserve numeric base, digit separators, suffixes, string delimiters, and
  escapes. Literal normalization should be a later, separately justified
  decision; it is not needed for layout.

The current `canonical_unicode` implementation works on raw bytes. It skips
line comments and ordinary strings, but not the entire specified literal and
comment surface, and its `canonical_ident` maps `l` and `level` to the level
glyph in every identifier position. That documented over-fire demonstrates why
token-role-driven canonicalization is required.

### 3. Spacing

- One space on each side of infix operators, binding `=`, type `:`, match
  arrows, `as`, guard `if`, and row-tail `|`.
- No space inside `()`, `[]`, or record/refinement braces.
- One space after commas. No space before commas or semicolons.
- No spaces around projection `.`, attached-proof `::`, or qualified-path `.`.
- A semicolon is attached to the preceding arm or field. In a multiline block,
  use it between siblings and omit it after the last sibling.
- Do not vertically align sibling arrows, colons, equals signs, or bodies.
  Indentation expresses structure; alignment creates unrelated diff churn.

Thus prefer:

```ken ignore
match x {
  None -> d;
  Some v -> v
}
```

over padding `None` to make its arrow line up with `Some v`. In actual output,
the two arrows above are emitted using the canonical match glyph.

### 4. Top-level declarations

A declaration remains flat only when its complete header and a simple body fit
comfortably on one line. A block body (`match`, multiline lambda, `let`, `if`,
or another compound form) always starts on the following line.

When a declaration header breaks:

1. keep the declaration keyword, name, and attached subject on the first line;
2. place each existing binder group on its own line, indented two spaces;
3. put the result type on a line beginning with `:`;
4. put `visits`, each `requires`, each `ensures`, and a broken `where` clause on
   separate lines in grammar order;
5. keep `=` at the end of the final signature/clause line; and
6. indent the body two spaces.

For example:

```ken ignore
lemma list_traverse_composition_cons
  (g : Type -> Type)
  (h : Type -> Type)
  (apg : Applicative g)
  (aph : Applicative h)
  (a : Type)
  (b : Type)
  (c : Type)
  (t1 : a -> g b)
  (t2 : b -> h c)
  (hd : a)
  (xs : List a)
  : Equal (Compose g h (List c)) lhs rhs =
  proof_body
```

The formatter emits canonical Unicode for the ASCII notation shown in these
illustrative `ignore` blocks.

Do not combine adjacent `(x : A) (y : A)` into `(x y : A)`, split a grouped
`(x y : A)`, infer implicit binders, or reorder constraints. Those are source
and API choices, not formatting.

For long result types, break their internal type structure using the type rules
below before considering any exceptional width overflow.

### 5. Function, dependent-function, and equality types

- Keep a short arrow chain flat.
- In a broken arrow chain, put one domain per line and lead continuation lines
  with the arrow. This makes the telescope visually scannable:

```ken ignore
  : (y : Nat)
    -> (z : Nat)
    -> Equal Bool (leq_nat x y) True
    -> Equal Bool (leq_nat y z) True
    -> Equal Bool (leq_nat x z) True =
```

- Apply the same rule to dependent pairs and long proposition combinators.
- Break a long `Equal`/`And`/type-constructor application by argument boundary,
  not in the middle of an argument.
- Print parentheses from precedence and the small set of mandatory clarity
  cases. Do not preserve arbitrary redundant parentheses, but always
  parenthesize an arrow type used as an application argument, an ascription
  used as a subexpression, and any lower-precedence infix operand whose grouping
  would otherwise be visually doubtful.
- Never change `Equal` into Boolean `==`, or vice versa. That distinction is
  semantic and central to review.

### 6. Applications and projections

- Keep a flat application on one line if it fits.
- When it does not fit, keep the function/head on the first line and put one
  syntactic argument per continuation line, indented two spaces.
- A compound argument is itself grouped and nested. Its continuation indent is
  relative to the enclosing syntax, never to the visual column where the head
  happened to end.
- Keep a projection or attached-proof selector with its head when possible;
  break the arguments supplied to the selected term, not between `d` and
  `.field` or between `subject` and `::proof`.

This makes a proof application read as a tree rather than a token stream:

```ken ignore
cong
  (g (h (List c -> List c)))
  (Compose g h (List c))
  lhs
  rhs
  (lambda y. apg.ap arg_type result_type y value)
  equality_proof
```

### 7. Lambdas, `let`, and `if`

- Coalesce immediately nested lambdas into one binder sequence only when the
  parsed AST is identical and no comment intervenes. Prefer `lambda y z p q.`
  over four adjacent lambda prefixes.
- Keep a lambda flat when its body is simple and fits. Otherwise place its body
  on the next line, indented two spaces.
- A `let` with a compound value or body is multiline:

```ken ignore
let x : A =
  value
in
  body
```

- An `if` whose whole expression does not fit, or whose branch is compound, is
  printed as:

```ken ignore
if condition then
  true_branch
else
  false_branch
```

- Do not turn `if` into `match`, introduce or remove a `let`, eta-reduce, or
  perform any other term transformation.

### 8. Matches

- `match e {}` is the canonical empty eliminator form.
- A single-arm match may remain flat only if the pattern and body are atomic,
  there is no guard or `eqn:` modifier, and the entire expression fits.
- Every match with two or more arms is multiline.
- Put one arm per line. A compound or broken arm body begins on the following
  line, indented two spaces past the arm.
- A nested match is always treated as compound, even when its token count would
  fit on the outer arm's line.
- Put semicolons after all but the last arm, with no preceding space.
- Do not align match arrows.

For example:

```ken ignore
match x {
  BTrue ->
    match y {
      BTrue -> True;
      BFalse -> False;
      BNot y1 -> False;
      BAnd yl yr -> False
    };
  BFalse ->
    match y {
      BTrue -> False;
      BFalse -> True;
      BNot y1 -> False;
      BAnd yl yr -> False
    }
}
```

This rule directly addresses the dense one-line nested matches in the current
catalog. It also makes exhaustiveness, arm order, guards, and wildcard coverage
easy to review.

### 9. Data, records, classes, instances, and other blocks

- A short simple sum of nullary constructors may remain on one line:

```ken ignore
data Color = Red | Green | Blue
```

- Otherwise print one constructor per line, with a leading `|` on every
  continuation constructor:

```ken ignore
data BoolExpr =
  BTrue
  | BFalse
  | BNot BoolExpr
  | BAnd BoolExpr BoolExpr
```

- An explicit dependent family always uses a multiline `where { ... }` block,
  one constructor signature per line.
- Nonempty `record`, `class`, `instance`, `space`, `policy`, and `module` blocks
  are multiline, one field/declaration/assignment per line.
- Empty blocks use `{}`.
- Do not visually align field names or types. Long field types break by their
  own arrow/application structure.
- Preserve constructor, field, and declaration order. In Ken, source order is
  relevant to resolution and is also part of the author's explanatory order.
- Never sort imports, constraints, effect rows, fields, or instances as a
  formatter operation.

### 10. Effects, contracts, refinements, and FFI

- Print rows as `[FS, Console]`, open rows as `[FS, Console | e]`, and empty
  rows as `[]`. Preserve row order unless the language later declares a
  source-level canonical order.
- Put a broken `visits` clause on its own signature line.
- Put every `requires` and `ensures` clause on its own line. Multiple clauses
  remain in source order; the formatter must not conjoin or reorder them.
- Format refinements with spaces around the structural punctuation:
  `{x : A | phi}` in ASCII illustration, using canonical notation in output.
- Keep `result` and `old` expressions as ordinary parsed expressions.
- A `foreign` declaration breaks into the Ken type/effect signature first and
  its `symbol`/`library`/`pure` body second. Never change or normalize the
  foreign strings.
- A temporal or other specified verbatim body is indented as a containing
  construct, but its internal bytes are untouched.

### 11. Comments

- Preserve every comment's text exactly except trailing horizontal whitespace.
- A doc comment remains attached to the following declaration.
- A leading comment remains immediately above the syntactic node it precedes,
  at that node's indentation.
- An end-of-line comment may stay inline only if code plus two separating
  spaces plus comment fits within 88 columns. Otherwise put it on the line
  immediately above the node it annotates.
- A comment between tokens forces the surrounding group to break rather than
  being silently relocated across a syntactic boundary.
- The formatter does not enforce the catalog's higher-level recommendation to
  move narrative into Markdown. It preserves source faithfully; a linter may
  flag misplaced narrative separately.

### 12. Literate `.ken.md` files

- Recognize only the four exact roles already defined by the literate format:
  `ken`, `ken ignore`, `ken reject`, and `ken example`.
- Canonical openers are at column zero with exactly one space before a role;
  canonical closers are a bare three-backtick line at column zero.
- Format each recognized fence body in place. Do not join adjacent fences,
  move declarations between fences, or change a fence's role.
- Markdown outside recognized fence bodies must be byte-for-byte identical.
  This should be tested by masking the fence ranges before comparing input and
  output.
- A `ken ignore` block may intentionally be an incomplete fragment, and a
  `ken reject` block may intentionally contain a syntax error. Full AST
  formatting cannot be required for those bodies without either a fragment
  parser or error-recovering CST. Until that exists, use token-aware
  canonicalization for unparseable `ignore`/`reject` bodies and leave their
  layout unchanged. Do not pretend they have a unique structural layout.
- The strict CI rule should distinguish "canonical parseable Ken" from
  "deliberately unparseable teaching fragment." An explicit, narrow fence-role
  exemption is better than a formatter that guesses structure from invalid
  syntax.

## What the formatter must not do

The formatter must not:

- reorder declarations or imports;
- rename identifiers or change casing;
- regroup binders;
- switch between `lemma`, `proof`, `fn`, and `const`;
- switch between an attached-proof selector and another proof spelling;
- desugar `if`, `match`, records, classes, effects, or refinements;
- choose between bracketed and juxtaposed type application while that spelling
  remains an open syntax choice;
- add or remove types, implicit arguments, constraints, or annotations;
- normalize numeric values, strings, foreign names, or temporal text;
- make proof terms shorter or introduce helpers; or
- reflow Markdown prose.

Some of these transformations could improve a program, but they belong to a
linter, refactoring tool, or agent-authored change. Mixing them into formatting
would make every formatting diff semantically suspect.

## Implementation architecture

### Lossless representation first

The current parser builds a semantic AST after the lexer has skipped whitespace
and line comments. An AST-only printer would therefore delete or misplace
comments and lose literal/token choices. That is unacceptable for a source
formatter.

Use one of these equivalent designs:

1. a lossless CST whose leaves include all tokens and trivia, with typed views
   into declarations, types, patterns, and expressions; or
2. the existing AST plus a complete ordered token/trivia stream and source
   spans, with a deterministic attachment algorithm for leading, trailing, and
   interstitial comments.

The first is safer for long-term parser, formatter, LSP, and refactoring reuse.
The formatter may reuse the semantic AST for precedence and grouping, but it
must never rely on that AST as the only source representation.

### Pretty-printing algebra

Implement a small document algebra with at least:

- text;
- hard line;
- soft line;
- concatenation;
- group/flatten;
- nest;
- choice if needed; and
- display-width measurement.

Give each grammar production one printer. Do not build a sequence of regular
expressions or line-based wrapping heuristics. Declaration, binder, type,
application, match-arm, and block documents should compose recursively.

Avoid global alignment combinators. They make a local edit reformat unrelated
sibling lines and can push otherwise short code over the width.

### Token canonicalization

Move Unicode canonicalization behind the lexer:

- accepted ASCII and Unicode aliases produce one token kind;
- the printer chooses the canonical glyph for that token kind;
- identifier and keyword tokens print their stored spelling unless a normative
  rule explicitly says otherwise; and
- literal/verbatim tokens print their preserved source lexeme.

This naturally fixes the `l`/`level` context bug and the `in` versus membership
distinction. Longest-token matching, such as match-arrow input before ordinary
arrow input, remains a lexer responsibility.

### Literate splicing

Reuse the literate extractor's ranges, but retain each role and original opener
and closer. Format recognized bodies, then splice replacements from the last
range to the first so earlier byte offsets remain valid. Verify that the prose
segments concatenate to exactly the original bytes.

### Fixed point and semantic gates

The minimum hard gates are:

1. **Idempotence:** `fmt(fmt(source)) == fmt(source)` byte for byte.
2. **Parse preservation:** before and after produce equal ASTs modulo spans,
   trivia, and explicitly sanctioned token aliases.
3. **Elaboration preservation:** both forms elaborate to the same surface-to-
   core result where a stable comparison is available. This is valuable in a
   dependently typed language because parser equality alone may miss a
   resolution/fixity interaction.
4. **Whole-catalog coverage:** run both preservation checks over every `.ken`
   and parseable Ken fence, not a sample.
5. **Prose preservation:** `.ken.md` prose is byte-identical.
6. **Trivia/literal preservation:** comment text and protected literal payloads
   are identical.
7. **Width property:** every line over 88 columns is classified as containing
   an indivisible or verbatim region; no breakable syntax silently exceeds it.
8. **Property tests:** generate ASTs, print them, parse them, and check equality;
   also fuzz whitespace and ASCII/Unicode aliases around a fixed AST.

Maintain a hand-written ambiguity suite for at least:

- match arrow versus function arrow;
- lambda versus dependent arrow;
- `in` keyword versus membership notation;
- `l` as an identifier versus a universe level;
- `:` versus `::`;
- projection versus qualified path;
- ascription versus dependent binder lookahead;
- right-associative arrows and left-associative applications;
- arithmetic precedence and user-declared fixity;
- comments between every pair of structural tokens;
- strings, chars, bytes, raw/multiline strings, and escapes containing every
  ASCII alias; and
- all four literate fence roles, including incomplete `ignore` and
  syntax-rejecting `reject` examples.

## Spec choices to settle before freezing output

The formatter will turn proposal-level spellings into a de facto language
standard, so these points should be resolved explicitly rather than guessed:

1. **Layout versus braces.** The grammar is written against braces, while the
   lexical chapter leaves offside layout open. Recommendation: canonicalize to
   explicit braces now. It is unambiguous, already the grammar's base form, and
   does not make whitespace part of semantics. Revisit only through a language
   decision and corpus migration.
2. **Class/record separators.** The EBNF uses commas for fields, while current
   catalog examples widely use semicolons and the landed parser accepts the
   actual corpus form. Recommendation: choose semicolons between declaration-
   like fields and assignments in multiline blocks, no trailing semicolon, and
   reconcile grammar, parser, and examples together.
3. **Broken sum layout.** Adopt leading continuation pipes as proposed above;
   the current catalog also contains trailing-pipe layouts.
4. **Lambda surface.** The documents show both dot-terminated Unicode lambdas
   and ASCII arrow/dot variants. The canonical printer should always emit the
   Unicode lambda with a dot, leaving ASCII forms input-only.
5. **Type application.** Both juxtaposition and brackets occur in the spec and
   remain under `OQ-syntax`. The formatter should preserve the parsed form until
   one canonical spelling is chosen.
6. **Invalid literate fragments.** Decide whether `ignore` and `reject` are
   structurally formattable fragment grammars or explicit layout exemptions.
   Strict CI needs a precise answer.

## Suggested delivery order

1. Pin the canonical-form section in the surface spec, including the six open
   points above.
2. Add the lossless syntax/trivia layer and round-trip it without changing
   layout.
3. Replace raw Unicode substitution with token-kind printing and close the
   protected-region/context bugs.
4. Add the document algebra and printers production by production, with the
   whole-catalog preservation gate continuously enabled.
5. Add `.ken.md` range splicing and prose-identity tests.
6. Run a preview corpus reformat and review representative ordinary code,
   dependent telescopes, class laws, deeply nested proofs, and all fence roles.
7. Land the tool, full corpus rewrite, and strict check atomically as the
   existing WP requires.

The most important design judgment is that canonical formatting is a
**review interface**, not merely whitespace cleanup. The output should expose
the same boundaries a human checks: telescope inputs, claimed result, effect
and contract boundary, exhaustive cases, and the tree of the proof term.
