# The standard-package catalog

> Status: **DRAFT v0**. Reframed by ES1 from "the stdlib (L8)" into the
> **standard-package tier** of the surface taxonomy (`../30-surface/30 §5`).
> This is the third tier: **ordinary Ken, optional, explicitly imported, out of
> `trusted_base()`** — not the always-present prelude (`30 §4`) and not the
> built-in surface TCB (`30 §3`). The monolithic **L8 dissolves** into this
> catalog; `docs/program/wp/L8-stdlib-core.md` is superseded. Each entry is
> ordinary Ken **with its derivation path from the built-ins stated** — a
> catalog entry with no path is a **hidden built-in** (a spec bug caught with
> CV's derivation-path table, `../../conformance/surface/taxonomy/`). ES4 builds
> these as in-repo packages under `catalog/packages/`.

This file is the spec/index side of the catalog. The source-style and
refinement contract for realized packages lives in
`../../docs/program/07-catalog-style-guide.md`.

The standard packages are **ordinary Ken** (they self-host atop the kernel and
the built-ins), with one discipline that distinguishes them from a typical
standard library: **their core abstractions carry their laws as propositions**
(`../20-verification/`), **proved, not postulated**, so the verification layer
can use them. A `Monoid` is not just `(append, empty)` — it is that plus
*proofs* of associativity and the unit laws.

## 1. What is NOT in this catalog (the line to `30`)

The everyday minimum that is **always in scope** is not a package — it is the
two lower tiers of the surface taxonomy (`../30-surface/30`):

- **Built-in (the surface TCB, `30 §3`)** — primitive types + literals
  (`Int`/`Float`/`Char`/`String`/`Bytes`, `../30-surface/35`, `37`), the audited
  primitive ops (`../10-kernel/14 §5`), the effect/FFI boundary
  (`../30-surface/38`), and the base elaborator syntax.
- **Prelude (Ken-defined, always-present, `30 §4`)** — the **closed** set a
  built-in primitive's signature names: `Bool`, `Char`, `List` (`Ordering` is
  **not** prelude — no primitive returns it; it is a package, `30 §4`) — plus
  the kernel's own logic vocabulary (`Ω`, `⊤`/`⊥`, the derived connectives,
  `Eq`, `Decidable`/`DecEq`, `../10-kernel/15`/`16`), referenced, **not**
  re-declared (`30 §6`: `Equal` is deleted for the kernel's `Eq`).

Everything below is a **package**: imported, derivable, re-checked. Core data
(`Unit`/`Empty`/`Nat`/`Option`/`Result`/`Either`/`Pair`, `../30-surface/34`) and
the core combinators (`id`, `∘`, `const`, `flip`) are packages — Ken `data`/defs
over the built-ins, not prelude (no primitive signature names them).

## 2. Lawful classes (the verification-aware core) — packages

A curated set of **classes with laws** (`../30-surface/33 §5`), each a package
whose derivation bottoms out in the built-ins (the class + instances are Ken;
the `Int` instance of `Num`/`Ord` wraps the audited primitive op, `30 §6` F2).
The **first tranche** — `Eq`/`DecEq`/`Ord`, the pattern-setter — is pinned in
**`51-lawful-classes.md`** (ES4-classes), with the `catalog/packages/` layout
established at `../../catalog/packages/README.md`:

| Class | Operations | Laws (propositions) |
|---|---|---|
| `Eq`/`DecEq` | `eq` | reflexive, symmetric, transitive (decidable equality) |
| `Ord` | `cmp`, `≤` | total order |
| `Semigroup`/`Monoid` | `<>`, `empty` | associativity, unit |
| `Functor`/`Applicative`/`Monad` | `map`, `pure`, `>>=` | functor/monad laws |
| `Foldable`/`Traversable` | `fold`, `traverse` | the fold/naturality laws |
| `Num`/`Integral`/`Fractional` | arithmetic | ring/field-ish laws (per type) |

Instances for the built-in/prelude types are provided **with their law proofs**,
so `fold` over a `Monoid` can be reasoned about, and a generic verified
algorithm may *assume the laws hold* (they are **proved, not postulated** — the
discipline that carries into every package build).

The **constructor-class tranche** — `Semigroup`/`Monoid` (value-level algebra)
and `Functor`/`Foldable` (the first classes over a type constructor
`f : Type → Type`) — is pinned in **`55-lawful-functors.md`** (CAT-1,
`catalog/packages/lawful-functors/`), the reusable template the **effectful
tranche** —
`Applicative`/`Monad`/`Traversable` — extends in **`56-effectful-classes.md`**
(CAT-2, same package): the deep chain **wires** superclass fields (`56 §2`), and
`Traversable.traverse` is the first effect-row-polymorphic `proc` (SURF-1,
`../30-surface/36 §1.5`).

## 3. Collections — packages over built-in/prelude carriers

`List` is a **prelude** type (named by the `String ↔ List Char` conversion,
`30 §4`); its combinators (`map`/`filter`/`fold`/`range`/…) are packages.
`Array` is a **built-in audited runtime type** (`../30-surface/37 §3.2`,
`30 §6`: `declare_primitive` OpaqueType, item-2 audited — the O(1)
content-addressed persistent tree is genuinely runtime). **`Map`/`Set`, by
contrast, are proved package trees over `Ord k` — out of `trusted_base()`, not
primitives** (`52-map.md`; OQ-A retires the earlier opaque `Map`/`Set`
primitive, `30 §6`). This catalog provides `Array`'s combinators and hosts the
proved `Map` module (`52-map.md`). The **verified building blocks** — a `sort`
returning `{ ys | is_sorted ys ∧ Perm ys xs }`, the proved `Map` — are the
canonical demonstrations of the thesis; the sort's predicates
`is_sorted`/`Perm` are **definitions** the prover unfolds (`../30-surface/37 §6`,
ES1), never postulates.

The **collection laws** (length/membership/decomposition + the verified `sort`
above) and the agent-facing **view** abstraction — the Layer-1 unit for
looking at data a different way (projection/lens, refinement, representation,
indexed, quotient-respecting, obligation-producing) — are pinned in
**`57-collections-and-views.md`** (CAT-3). `Perm` is `Ω`-native
**count/multiset-equality**, not a raw inductive (the `Ord.total` soundness
move, `57 §3.1`); the view is a plain `Σ`-record (concrete flavors ship now,
the polymorphic family gated on a bounded multi-param-`class` extension). The
family is named **`view`** (operator's call) — SURF-1's retirement of the `view`
keyword frees the word, and `view` is the standard term for a read projection.

The **Layer-2 keyed-collection laws** — `delete`,
`union`/`intersection`/`difference` (a **combining-function** `union`,
subsuming both biases), `keys`/`values` coherence — plus the **set algebra**
(`∪`/`∩`/`∖`, stated **membership-extensionally**, never `Equal (Set K)`:
same-key-set trees
differ in shape) and the **relations frontier** (composition, converse, the
reflexive/symmetric/transitive predicates, transitive closure) are pinned in
**`58-maps-sets-relations.md`** (CAT-4). `delete` is **rebuild-via-`from_list`**
(reusing the landed `preserves_ordered`); a relation is `Map K (Set K)`
adjacency; and the transitive closure is **bounded-reachability `IsTrue`** —
`Ω`-native, the `Perm` move again, never a raw proof-relevant inductive
(`58 §7`), its faithfulness proof + `size` a designed-and-deferred fast-follow.
The `Nat` carrier (a net-new `Axiom`-free `leq_nat`) is the discriminator floor,
not the `Axiom`-holed `Ord Int`/`Ord Char`.

The **Layer-3 parsing/syntax/diagnostics contract** — source artifacts as byte
identity, half-open byte spans, total parser result values, small package-owned
grammars, parser/printer and formatter laws, and diagnostic primary/secondary
span validity — is pinned in **`59-parsing-syntax-diagnostics.md`** (CAT-5).
It is an ordinary `catalog/packages/parsing/` catalog package: no compiler parser
rewrite, no compiler-internal AST as public API, no full Ken syntax reflection,
and no `.ken.md` implementation work. A derived input such as a blanked
`.ken.md` compiled view is only an offset-preserving view of the original
source artifact; the original artifact owns diagnostic identity.

The **length-indexed vector** `Vec (A : Type) : Nat → Type` — a **fresh
dependent inductive family** (not a built-in/prelude carrier), the canonical
dependent-types totality showcase — is pinned in
**`60-length-indexed-vectors.md`** (DS-5). The length index rules the empty case
out *at the type level*, so `head : Vec A (Suc n) → A` is total with no
`Option`; the family, `vnil`/`vcons`, and `head` are landed (the acceptance
suite `explicit_data_elaboration.rs`), while `tail`/`zip`/`lookup` and the `Fin`
bounded index's *use* are gated on the dependent-`match` refinement enhancement
landing this run as `DS-5b` (Kernel ring). Zero `Axiom`, zero `trusted_base()`
delta — `Vec` is an ordinary inductive with a real eliminator.

## 4. I/O, effects, serialization

- Effect interfaces (`../30-surface/36`):
  `IO`/`FS`/`Net`/`Clock`/`Console`/`Rand` — the **boundary primitive** is
  built-in (`30 §3`); the typed effect surface over it is package Ken.
- Serialization (`../30-surface/38 §1`): a derivable `Encode`/`Decode` with the
  **round-trip law** `decode ∘ encode = Ok` provable, plus Merkle hashing for
  content verification — a package.
- A `Stream`/`Iterator` type for lazy iteration (productivity per
  OQ-coinduction) — a package.

## 5. Tooling-facing (T-stream)

The catalog also seeds the tooling Ken needs to be usable (strategy WS-T): a
test/property framework (`../30-surface/` doc-tests + property tests, T3), and
the pedagogy/reference corpus (T4) — important because a new language has
near-zero training priors, so honest, runnable docs *are* the seed corpus.

## 6. Optional / research packages (not core)

Off to the side, never required by the core language: the **Leech/Golay/Co₀**
facilities in their three separate roles (`../40-runtime/44 §4`, OQ-6); the
**coalgebraic** layer (Store-comonad cells, process coalgebras, profunctor wires
— strategy WS-R); linear/affine types; delimited continuations. These are
harvested back as ordinary packages only if they earn it.

## 7. What WS-L must deliver here — the dissolved L8, as packages

The standard-package catalog (this file) delivered as **in-repo packages under
`catalog/packages/`** (ES4): the lawful classes (with **proved** laws), the
collection combinators + verified building blocks, the effect/IO/serialization
interfaces,
and the test/doc tooling seed — **each with its built-in-derivation path
stated** (the ES1 discipline; a missing path is a hidden-built-in spec bug).
Acceptance contributes to **G6** (a realistic verified component uses the
packages) and **G7** (the agent loop has enough library to work with).
Conformance: `../../conformance/stdlib/` — law proofs for the instances and the
verified `sort`/`Map` building blocks; and the derivation-path table
(`../../conformance/surface/taxonomy/`, ES1) that proves each entry is genuinely
package, not a hidden built-in.
