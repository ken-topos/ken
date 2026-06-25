# Lexical structure

> Status: **DRAFT v0**. Proposal-level (OQ-syntax). Tokens, identifiers,
> literals, comments, and the layout rule. Spelling is revisable; the *token
> categories* (especially the literal forms feeding `35-numbers.md`) are the
> part that matters for downstream chapters.

## 1. Source text

- Source is **UTF-8**. Ken is Unicode-aware: identifiers and operators may use
  non-ASCII letters and a curated set of mathematical symbols (so `→`, `×`, `∧`,
  `Ω`, `≤`, `≠` can appear in source, matching the spec's notation). An ASCII
  spelling exists for every such symbol (`->`, `*`/`×`, `/\`, `<=`, `/=`) so no
  program *requires* a special keyboard (OQ-syntax: how much Unicode to bless).
- Files use the extension `.ken`. Line endings are LF (CRLF tolerated).

## 2. Tokens

```
token ::= ident | conid | keyword | literal | operator | punct | layout
```

- **`ident`** — value/term names: lowercase-initial, `[a-z_][A-Za-z0-9_']*` plus
  blessed Unicode letters. Primes (`x'`) allowed (math-friendly).
- **`conid`** — constructor / type / module names: uppercase-initial. The case
  distinction (lowercase = term variable, uppercase = constructor/type) is used
  by `match` to tell binders from nullary constructors (`34`).
- **`keyword`** — reserved (§4).
- **`literal`** — numbers, strings, chars, bytes (§3).
- **`operator`** — symbolic, from a fixed set plus user-defined (`33`); fixity
  and precedence are declared (`infixl`/`infixr`/`infix N`).
- **`punct`** — `( ) [ ] { } , . ; : | = → @ ⟨ ⟩` and the spec brace `{ … | …
  }`.

## 3. Literals (the part that matters)

Literal *forms* are fixed even though syntax is otherwise OQ, because they
determine the numeric story (`35`):

| Literal | Examples | Default type |
|---|---|---|
| **integer** | `0`, `42`, `1_000`, `0xFF`, `0b1010`, `0o17` | `Int` (arbitrary precision) |
| **decimal** | `3.14d`, `0.1d`, `1_000.00d` | `Decimal` |
| **float** | `3.14`, `1e-9`, `0x1p-3` | `Float` (IEEE f64) — **only with a `.`/exponent** |
| **string** | `"…"` with escapes, `"""…"""` raw/multiline | `String` (UTF-8) |
| **char** | `'a'`, `'\n'`, `'\u{1F600}'` | `Char` (Unicode scalar) |
| **bytes** | `b"…"`, `0x[deadbeef]` | `Bytes` (`38`) |
| **bool** | `true`, `false` | `Bool` |

- **Critical rule (the f64 correction at the lexer):** a bare integer literal is
  **`Int`**, never a float. `2` is `Int`; `2.0` is `Float`; `2.0d` is `Decimal`.
  The prototype's "every number is `f64`" defect does not even exist at Ken's
  lexer — integers and floats are *different tokens with different default
  types*. Numeric literals are **polymorphic** over the numeric classes via the
  elaborator (`35 §literals`, `39`), defaulting as above when unconstrained.
- Underscores are digit separators and are ignored.

## 4. Keywords (proposal)

```
view let type data record module import use space
match if then else where requires ensures prove law
visits foreign forall exists in as mut
```

Reserved but spelling-revisable (OQ-syntax). Contextual keywords (`infixl`,
`derive`, …) are not globally reserved.

## 5. Comments and documentation

- Line comment `-- …`; block comment `{- … -}` (nestable).
- Doc comment `--- …` (or `{-- … --}`) attaches to the following declaration and
  is consumed by the doc generator and the LSP. Doc comments may contain spec
  fragments and examples that the test framework can run (`../50-stdlib/`,
  strategy T3/T4).

## 6. Layout (indentation) vs. braces

- The DRAFT uses an **offside / layout rule** (significant indentation opens
  blocks: a `where`, a `match`, a `module` body), with explicit `{ ; }`
  available as an unambiguous fallback (the layout rule inserts virtual
  braces/semicolons).
- Whether Ken is layout-sensitive or brace-delimited is **OQ-syntax**; the
  grammar (`32`) is written against the brace form, and layout is sugar
  producing it.

## 7. What WS-L must deliver here

A lexer producing the token stream above with the fixed literal categories (§3 —
especially `Int`-default integers), Unicode + ASCII spellings, comments/doc
comments, and the layout-to-braces translation. Conformance:
`../../conformance/surface/lexical/` — including the regression that `2 : Int`
and `2.0 : Float` are distinct (the f64 non-reproduction at the lexer).
