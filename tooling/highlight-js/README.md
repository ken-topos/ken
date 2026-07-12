# `highlight-js/` — a highlight.js grammar for Ken

A [highlight.js](https://highlightjs.org/) language definition for Ken,
registered under the name `ken`. It exists so a Markdown reader (or any other
highlight.js-powered viewer) can colour Ken source — the literate `.ken.md`
fenced ` ```ken ` blocks that make up the catalog and guide, and plain `.ken`
files — without the reader knowing anything about Ken beforehand.

## Files

| File | Purpose |
|---|---|
| `ken.js` | The language definition itself: a UMD module exporting the highlight.js "language definer" function `(hljs) => languageObject`. |
| `sample.ken` | A fixture exercising every token category (all keywords, every literal form, `--` comments, `data`/`match`, `class`/`instance`, named proof claims, the operator/Unicode set) — used to eyeball the highlighting and as a target for manual smoke tests. |
| `README.md` | This file. |

## Registering it

Because highlight.js auto-selects a language from a fenced code block's info
string, registering the `ken` alias is the *only* step needed — every
` ```ken ` fence in a rendered Markdown file then highlights automatically.

**As an ES module / bundler / Node dependency graph:**

```js
import hljs from 'highlight.js/lib/core';
import ken from './ken.js';

hljs.registerLanguage('ken', ken);
hljs.highlightAll();
```

`ken.js` has no bare `export`/`import` statements (it is a UMD module), so
this `import` works via your bundler's CommonJS interop, and so does a plain
`const ken = require('./ken.js')` in Node.

**As a browser `<script>` tag (no bundler):**

```html
<script src="https://cdn.jsdelivr.net/npm/highlight.js@11/lib/core.min.js"></script>
<script src="./ken.js"></script>
<script>
  // ken.js's UMD wrapper exposes the definer as `window.hljsDefineKen`
  // when loaded as a plain global script (no `module`/`define` present).
  hljs.registerLanguage('ken', window.hljsDefineKen);
  hljs.highlightAll();
</script>
```

Either way, a Markdown reader that runs its rendered HTML through highlight.js
(most do, as a post-render pass over `<pre><code class="language-ken">`
blocks) picks up ` ```ken ` fences automatically once `registerLanguage` has
run once at startup — no per-fence configuration.

## Coverage table

| Token category | Highlighted? | How |
|---|---|---|
| `--` line comments | yes | `comment` scope, via `hljs.COMMENT('--', '$')` |
| Keywords (`const fn proc let in Type requires ensures prove law old space data match def type foreign temporal class instance derive where module import use pub prop lemma proof`) | yes | `keyword` scope, via the `keywords` table |
| Uppercase-initial identifiers (`ConId` — types, data constructors, e.g. `Bool`, `Nat`, a user's `Shape`) | yes | `title.class` scope, via the single uppercase-initial rule (no hand-listed type names) |
| `Ω`/`Σ`/`Π` (fold to `Omega`/`Sigma`/`Pi`) | yes | `title.class`, same rule as ConIds |
| `∀`/`∃`/`¬`/`ℓ` (fold to ordinary lowercase identifiers `forall`/`exists`/`not`/`level`) | no (by design) | left as plain text — they're term-level identifiers, not a distinct category |
| Integer literals (`123`) | yes | `number` |
| Float literals (`3.14`, `1.0e-9`) | yes | `number` — a dotted mantissa is required; an exponent alone (`1e-9`, no dot) is not a float to the lexer either |
| Decimal literals (`0.1d`, `5d`) | yes | `number` |
| Float32 literals (`1.5f32`, `5f32`) | yes | `number` |
| String literals (`"..."`) | yes | `string`, single-line, no escape handling (matches the lexer) |
| Arrows (`->`/`→`, `|->`/`↦`) | yes | `operator` |
| `::` `:` `=` `==` `===`/`≡` `\|` `;` `.` `,` | yes | `operator` |
| Lambda (`\`/`λ`), logic (`/\`/`∧`, `\/`/`∨`) | yes | `operator` |
| Comparisons (`<=`/`≤`, `>=`/`≥`, `/=`/`≠`, `<:`/`⊑`, `⊔`, `⊓`, `><`/`×`) | yes | `operator` |
| Arithmetic (`+`, `+%`, `-`, `*`), membership (`∈`) | yes | `operator` |
| Brackets `( ) { } [ ]` | not specially coloured | left as plain text, consistent with most highlight.js grammars |
| `absurd` (bottom-elimination surface sugar) | yes, as a special case | `built_in` — **not** a lexer keyword; called out because it's reserved *sugar*, kept as a tiny, explicitly-commented list in `ken.js` rather than a growing name list |
| A lone `<`, `>`, or `/` | no | these are not valid Ken tokens on their own (the lexer errors unless they start `<=`/`<:`, `>=`/`><`, `/=`/`/\`); the grammar does not invent acceptance for them |

## Deliberate non-coverage

A highlighter categorises tokens **for the eye** — it is not a type checker,
and it is intentionally coarser than the real Ken elaborator:

- It does not distinguish a **bound** occurrence of an identifier from a
  **free** one, or a shadowed local from an unrelated global — every
  lowercase-initial identifier gets the same (absent) styling regardless of
  scope.
- It does not validate a **refinement predicate** (`{ x : A | φ }`) — the
  `|` inside it highlights as the ordinary `Pipe` operator, exactly like the
  `|` in a `data` declaration or a `match` arm; the grammar has no notion of
  "this is inside a refinement."
- It does not check purity (`const`/`fn`/`proc`), exhaustiveness of a
  `match`, effect rows, or any other elaboration-time property — a program
  that is nonsense to the real compiler can still highlight as if it were
  fine, and vice versa (a lex error like a bare `<` just renders unstyled,
  not flagged as an error).
- It does not resolve names — an identifier that doesn't exist anywhere
  looks identical to one that does, as long as its case matches the
  ConId/ordinary-identifier split.

None of this is a bug to fix later; it is the honest boundary of what a
lexical highlighter can and should do.

## Grounding note

Every token category above is derived from
[`crates/ken-elaborator/src/lexer.rs`](../../crates/ken-elaborator/src/lexer.rs),
the single source of truth for what Ken actually lexes — not from a hand-kept
guess. `ken.js` carries the same grounding note in its header comment. To
re-sync after the lexer changes:

1. Re-grep the keyword match arms (the `match s.as_str() { ... }` block) and
   update the `keyword` list in `ken.js` verbatim — do not add or drop a word
   without a corresponding lexer change.
2. Re-grep the single-character Unicode-alias arms (`Ω`, `Σ`, `Π`, `∀`, `∃`,
   `¬`, `ℓ`, and the operator aliases like `→`, `↦`, `≡`, …) and update the
   `title.class`/`operator` regexes.
3. Re-grep `lex_numeric` for the literal suffix set (currently `d`,
   `f32`) and the dot/exponent rule, and update the `number` regex.
4. Re-run this directory's verification commands (below) before shipping the
   update.

See the parent [`tooling/README.md`](../README.md) for the category-wide
grounding discipline this entry follows (why every integration here carries
its own re-sync note, and how this fits alongside the anticipated
`textmate/`, `tree-sitter/`, `lsp/`, and `emacs/` integrations).

## Verification

No dependency is installed — these are dependency-free checks against the
UMD module's shape:

```sh
# Proves ken.js has no stray ESM syntax and parses as plain JS/CommonJS.
node --check tooling/highlight-js/ken.js

# Proves the definer function runs and returns a well-formed language object,
# without a real highlight.js installed (a minimal stub standing in for the
# one hljs helper the definer actually calls, `hljs.COMMENT`).
node -e "const def=require('./tooling/highlight-js/ken.js'); const stub={COMMENT:(a,b)=>({begin:a,end:b}),QUOTE_STRING_MODE:{},C_NUMBER_MODE:{},inherit:(o)=>o}; const g=(typeof def==='function')?def(stub):def; console.log('name=',g.name,'aliases=',g.aliases,'contains?',Array.isArray(g.contains), 'kw?', !!g.keywords);"
```

`highlight.js` itself is **not** a dependency of this repo — this grammar is
plain JavaScript that a *consumer* (the Markdown reader) supplies the real
`hljs` object to at registration time.
