# Lexical structure

> Status: **DRAFT v0**. **`OQ-syntax` principles DECIDED** (operator,
> 2026-06-27, §1a); the concrete *token table* below is a **starter** that
> iterates with the team, now *governed by* those principles. The literal forms
> feeding `35-numbers.md` are the part that most matters for downstream
> chapters.

## 1. Source text

- Source is **UTF-8**. Ken is Unicode-aware: identifiers and operators may use a
  **curated** set of mathematical symbols (so `→`, `×`, `∧`, `Ω`, `≤`, `≠`, `⊑`
  appear in source, matching the spec's notation). An ASCII spelling exists for
  every such symbol so no program *requires* a special keyboard (§1a).
- Files use the extension `.ken`. Line endings are LF (CRLF tolerated).

## 1a. Notation: read-optimized canonical Unicode (`OQ-syntax` DECIDED)

Ken is **written by agents and read by humans**, so *writing* is cheap and
*reading* is dear — which **inverts** the usual ASCII-because-humans-type
tradeoff. Ken optimizes its **canonical form for reading**: the typability tax
that binds mainstream languages does not bind a language whose writers are
agents. Five principles (decided; the §2–§6 spellings are a starter under them):

1. **Match established CS/Math notation; never invent.** The legibility win
   comes from the reader's *existing training* — admit a glyph only if a
   type-theory/ CS-educated reader already knows it (`→ × ∀ ∃ λ Σ Π Ω ⊢ ⊑ ⊔ ⊓ ¬
   ∧ ∨ ∈ ≤ ≠ ≡ ℓ`). Decorative or novel glyphs are rejected — they *cost*
   legibility with no convention to amortize.
2. **A total ASCII transliteration.** Every notation token has a typeable ASCII
   form (§1b). A human may write either; the glyph carries **zero** extra
   information (round-trippable), so reading the ASCII loses nothing — the
   exploration/self-learning affordance.
3. **Formatter-canonicalized.** A **single mandated formatter** (gofmt-style)
   normalizes ASCII → canonical Unicode and fixes layout on save. Because humans
   read and agents write, **one canonical format** means the reader always sees
   consistent notation — no style variance to parse. (No formatting latitude.)
4. **Keywords stay ASCII words.** `const fn proc data record match space visits
   requires ensures prop proof lemma prove law` are *names* — legibility beats
   symbol density, and they are already typeable. Notation is reserved for
   *operators*, where a symbol carries established meaning; Unicode-ifying
   keywords would be decoration. (So the purity keywords `const`/`fn`/`proc`,
   `36 §1.6`, are ASCII words, not glyphs.)
5. **Curated and confusable-resistant (a security property, not only
   legibility).** The blessed set is **bounded** (a fixed table, not "any
   Unicode"), and the lexer **normalizes/rejects Unicode confusables** (the TR39
   security profile: `⊔`/`U`, `∨`/`v`, `×`/`x`, `ℓ`/`l`, Cyrillic look- alikes).
   A reviewer must read **exactly** what the kernel checks — no homoglyph can
   smuggle code past a human reader (`../60-security/64`). This makes the rich
   notation *safe*, part of the "human reviews, kernel checks" integrity story.

## 1b. Starter notation table (iterates with the team)

Canonical glyph ↔ ASCII input, drawn from the notation the spec already uses.
**Starter, not final** — the team tunes spellings against real code; the
*principles* (§1a) are fixed. The ASCII fallback prefers an established TeX/CS
digraph where one is unambiguous, else the spelled-out name.

| Glyph | ASCII | Role |
|---|---|---|
| `→` | `->` | function type / arrow |
| `λ` | `\` | anonymous function (named is `fn`/`proc`) |
| `∀` | `forall` | universal quantifier (propositions) |
| `∃` | `exists` | existential quantifier |
| `Σ` `Π` | `Sigma` `Pi` | dependent sum / product (binders) |
| `Ω` | `Omega` | strict-prop universe (`../10-kernel/12`) |
| `≡` | `===` | propositional equality (`Eq`, `../10-kernel/15`) † |
| `≤` `≥` `≠` | `<=` `>=` `/=` | comparison |
| `¬` `∧` `∨` | `not` `/\` `\/` | logical connectives |
| `∈` | `in` | membership |
| `⊑` `⊔` `⊓` | `<:` `\/` `/\` | IFC lattice flows-to / join / meet (`../60-security/61`) ‡ |
| `×` | `><` | product type |
| `ℓ` | `level` / `l` | universe level / label (context-disambiguated) ‡ |

† Equality notation is the load-bearing fine choice: `≡` propositional vs. `==`
boolean `DecEq` (`33 §5`) must stay distinct (Lean/Agda convention); `=` is
**binding only**. The exact ASCII for `≡` (`===` vs. a named form) is a team
call. ‡ The lattice-op ASCII (`⊑`/`⊔`/`⊓`) and the `ℓ` overload (level vs.
label) are the other genuinely-contested cells — flagged for the team, not fixed
here.

## 1c. BL3 — the canonical Unicode surface is lexer *and* formatter (SURF-1 D3)

> Status: **resolved** — a **direct consequence of §1a**, made explicit here for
> the BL3 build. The question "is the Unicode surface a lexer change or a
> formatting convention?" is answered **both**, exactly as §1a principles 2–4
> already decide; SURF-1 D3 does not add a new decision, only pins the division
> of labour and confirms **ASCII stays accepted**.

- **The lexer accepts both spellings as the *same token* (principle 2).** A
  curated Unicode glyph and its ASCII transliteration (`→`/`->`, `λ`/`\`, `∀`/
  `forall`, `Σ`/`Sigma`, `Ω`/`Omega`, `⊑`/`<:`, …, §1b) lex to the **identical**
  token — the §1b/§8 "the two are the same token" rule generalized across the
  blessed table. So the glyph carries **zero** extra information and **ASCII
  spellings remain accepted forever** (no program ever *requires* a special
  keyboard). This is genuinely a **lexer** capability, not only a convention.
- **The formatter emits canonical Unicode on save (principle 3).** The single
  mandated formatter normalizes accepted ASCII input to canonical Unicode
  glyph (and fixes layout), so the reader always sees consistent notation. This
  is the **convention** half — but it is *downstream* of the lexer, applied to
  already-accepted source, never a parse gate.
- **Keywords are exempt — they stay ASCII words (principle 4).** The Unicode
  surface is for **operators/symbols** only; `const`/`fn`/`proc` and every other
  keyword (`31 §4`) stay ASCII words. BL3 Unicode-ifies the *operator* surface,
  not the keyword surface.
- **Confusable-resistance is a hard lexer gate (principle 5).** The blessed set
  is bounded; the lexer normalizes/rejects TR39 confusables (`⊔`/`U`, `∨`/`v`,
  `×`/`x`, `ℓ`/`l`, Cyrillic look-alikes) so a reviewer reads exactly what the
  kernel checks (`../60-security/64`).

**Build scope (BL3 / D4).** The build realizes the lexer's accept-both +
same-token behaviour and the formatter's Unicode normalization, then **runs the
formatter over the corpus** (prelude, `packages/*`, `examples/rosetta/*`) to
convert ASCII digraphs to canonical Unicode — landed together with the `view →
const`/`fn`/`proc` migration (D4) as one workspace-green unit. A Unicode-surface
`.ken` and its ASCII twin **elaborate identically** (acceptance 7), because they
lex to the same tokens.

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
- **`punct`** — `( ) [ ] { } , . ; : :: | = → @ ⟨ ⟩` and the spec brace
  `{ … | … }`.

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

- **Critical rule (exact numerics at the lexer):** a bare integer literal is
  **`Int`**, never a float. `2` is `Int`; `2.0` is `Float`; `2.0d` is `Decimal`.
  Integers and floats are *different tokens with different default types*; there
  is no universal `f64` carrier at Ken's lexer. Numeric literals are
  **polymorphic** over the numeric classes via the elaborator (`35 §4`, `39`),
  defaulting as above when unconstrained.
- Underscores are digit separators and are ignored.

## 4. Keywords (proposal)

```
const fn proc let def data record module import use space
match if then else where requires ensures prop proof lemma prove law
visits foreign forall exists in as mut class instance
becomes declassify policy temporal assume test
```

Reserved but spelling-revisable (OQ-syntax), **except** the purity keywords
`const`/`fn`/`proc` (`36 §1.6`), whose spellings are **fixed** by the operator
ruling (SURF-1); `view` is **retired**. `let` remains reserved for the local
`let … in …` expression (`32 §3`). `type` is **reserved but not a declaration
keyword** — it named the definition/refinement construct before
`SURF-def-refinement` (`33 §1`) renamed it to `def`; `type` stays rejected as a
free identifier to preserve future optionality. Contextual keywords
(`infixl`, `derive`, …) are not globally reserved. The decided post-freeze
surface tokens are also lexed here (all spellings OQ-syntax):

- the wrapping-arithmetic operator `+%` (and `wrapping_add`, …) in the operator
  set (`35 §3`, OQ-1a);
- the type-level identifiers `Lazy` (OQ-eval-order) and `Wrapping` (OQ-1a,
  `Wrapping[T]`);
- an annotation token `annotation ::= "@" ident`, with `@ct` a named attribute
  (`../60-security/61 §5a`), distinct from any binary use of `@`.

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

## 8. V0 minimal lexer (the G1 slice)

V0 (the minimal elaborator, `../30-surface/39-elaboration.md §5`) lexes **only**
the token subset below — just enough to write a trivial dependently-typed
program. The full token set (§2–§4) is for the complete surface; V0 recognises
none of the rest (no literals, no operators, no layout, no annotations).

- **Keywords:** `fn`, `const`, `let`, `in`, `Type`. (V0 is pure-only, needing
  `fn`/`const` and never `proc`; `36`. `Type` lexes as a keyword in V0, not a
  `conid` — it is the universe former, `../10-kernel/12`. The landed V0 lexer
  still spells `view`/`let` until the D4 migration; the surface here is the
  target.)
- **Punctuation:** `(`, `)`, `:`, `=`, `.`, and the arrow `->` (canonical `→`;
  the two are the same token, §1b).
- **Lambda:** ASCII `\` (canonical `λ`; same token, §1b).
- **Identifiers:** the §2 case distinction is load-bearing in V0 —
  **lowercase-initial** `ident` is a term variable; **uppercase-initial**
  `conid` is a base type (`Nat`, `Bool`) or other type constructor. Name
  resolution (`39 §5.3`) and type-position parsing (`39 §5.2`) rely on it.
- **Level digits:** bare non-negative integers (`0`, `1`, …) appear **only** as
  the optional explicit level after `Type` (`Type 0`); V0 has no other numeric
  literals (§3 is out of V0).

Whitespace separates tokens; line comments `-- …` (§5) are skipped. Everything
else in §2–§4 — block comments, doc comments, operators, the literal forms of
§3, layout (§6) — is **out of V0** and lexes only under the full lexer.
