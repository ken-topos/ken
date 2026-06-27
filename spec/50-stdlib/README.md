# The prelude and core standard library

> Status: **DRAFT v0**. Lowest-resolution section — fixes the *shape and
> principles* of the stdlib, not every signature (those grow with WS-L).
> Contract for **L8** (curated stdlib) + the prelude that the surface chapters
> assume.

The stdlib is **ordinary Ken** (it self-hosts atop the kernel), with one
discipline that distinguishes it from a typical standard library: **its core
abstractions carry their laws as propositions** (`../20-verification/`), so the
verification layer can use them. A `Monoid` is not just `(append, empty)` — it
is that plus *proofs* of associativity and the unit laws.

## 1. The prelude (always in scope)

The implicitly-imported core the surface chapters reference:

- **Primitive scalars & numerics** — `Int`, `Int64`/`UInt32`/…, `Decimal`,
  `Float`/`Float32`, `Bool`, `Char` (`../30-surface/35`), with the numeric
  classes for literal overloading.
- **Core data** — `Unit`, `Empty`, `Bool`, `Nat`, `Option`, `Result`, `Either`,
  `Pair`/tuples (`../30-surface/34`); `Ordering`.
- **Text & bytes** — `String`, `Bytes` (`../30-surface/37`, `../30-surface/38`).
- **Logic & equality** — `Ω`, `⊤`/`⊥`, the Heyting connectives, `Eq`/`Id`
  (observational), `Decidable`, `DecEq` (`../10-kernel/12 §5`,
  `../10-kernel/15`).
- **Core functions** — `id`, `∘` (compose), `const`, `flip`, basic combinators.

## 2. Lawful classes (the verification-aware core)

A curated set of **classes with laws** (`../30-surface/33 §5`):

| Class | Operations | Laws (propositions) |
|---|---|---|
| `Eq` | `eq` | reflexive, symmetric, transitive (or decidable equality) |
| `Ord` | `cmp`, `≤` | total order |
| `Semigroup`/`Monoid` | `<>`, `empty` | associativity, unit |
| `Functor`/`Applicative`/`Monad` | `map`, `pure`, `>>=` | functor/monad laws |
| `Foldable`/`Traversable` | `fold`, `traverse` | the fold/naturality laws |
| `Num`/`Integral`/`Fractional` | arithmetic | ring/field-ish laws (per type) |

Instances for the prelude types are provided **with their law proofs**, so e.g.
`fold` over a `Monoid` can be reasoned about, and a generic verified algorithm
may *assume the laws hold* (they are proved, not postulated).

## 3. Collections

`List`, `Array`, `Map`, `Set` (`../30-surface/37`) with their combinators
(`map`/`filter`/`fold`/…) and the relevant operation laws (e.g. `Map`
lookup-after- insert). Verified building blocks: a `sort` returning `{ xs |
isSorted xs ∧ isPermutationOf xs }`, a verified `Map`, etc. — the canonical
demonstrations of the thesis.

## 4. I/O, effects, serialization

- Effect interfaces (`../30-surface/36`):
  `IO`/`FS`/`Net`/`Clock`/`Console`/`Rand` as capability/effect types.
- Serialization (`../30-surface/38 §1`): a derivable `Encode`/`Decode` with the
  **round-trip law** `decode ∘ encode = Ok` provable, plus Merkle hashing for
  content verification.
- A `Stream`/`Iterator` type for lazy iteration (productivity per
  OQ-coinduction).

## 5. Tooling-facing (T-stream)

The stdlib also seeds the tooling Ken needs to be usable (strategy WS-T): a
test/property framework (`../30-surface/` doc-tests + property tests, T3), and
the pedagogy/reference corpus (T4) — important because a new language has
near-zero training priors, so honest, runnable docs *are* the seed corpus.

## 6. Optional / research packages (not core)

Off to the side, never required by the core language: the **Leech/Golay/Co₀**
facilities in their three separate roles (`../40-runtime/44 §4`, OQ-6); the
**coalgebraic** layer (Store-comonad cells, process coalgebras, profunctor wires
— strategy WS-R); linear/affine types; delimited continuations. These are
harvested back as ordinary packages only if they earn it.

## 7. What WS-L must deliver here (L8)

A curated prelude + core stdlib of **lawful** abstractions (classes with proved
laws), the collection types with verified building blocks, the effect/IO/
serialization interfaces, and the test/doc tooling seed. Acceptance contributes
to **G6** (a realistic verified component uses the stdlib) and **G7** (the agent
loop has enough library to work with). Conformance: `../../conformance/stdlib/`
— law proofs for the prelude instances and the verified `sort`/`Map` building
blocks.
