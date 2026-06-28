# Strings and collections

> Status: **DRAFT v0**. Proposal-level. The core data types a programmer reaches
> for daily: `String`, `List`, `Map`, `Set`, and iteration. These are stdlib
> (`../50-stdlib/`), not kernel primitives, but the spec fixes their *shape* and
> their relationship to the content-addressed runtime.

## 1. Strings and text

- **`String`** — an immutable **UTF-8** text value; a primitive type
  (`../10-kernel/14 §5`) at runtime (`../40-runtime/41-values.md`), content-
  addressed like any compound value (so equal strings share storage, and
  equality is O(1)). Ken treats `String` as first-class from the start.
- **`Char`** — a Unicode scalar value (`35 §2`).
- Indexing is by **code point** or explicit byte view; the API distinguishes
  byte-length from character-length to avoid the usual UTF-8 traps. Raw bytes
  are `Bytes` (`38`).
- Strings are **not** `List Char` (that is a separate, convertible view); the
  runtime representation is a packed byte buffer for efficiency.

## 2. The core collections

| Type | Meaning | Notes |
|---|---|---|
| `List a` | immutable singly-linked list | the inductive `data List` (`34 §1`); pattern-matchable |
| `Array a` | immutable contiguous sequence, O(1) index | persistent/structural-sharing under the hood |
| `Map k v` | immutable key→value | needs `DecEq`/`Ord` (or hashable) `k` (`33 §5`) |
| `Set a` | immutable set | as `Map a Unit`; the lattice-bitmap option is `../40-runtime/44` |
| `Option a`, `Result e a` | optionality / fallibility | sum types (`34 §1`); explicit absence/error |

- All core collections are **immutable and persistent** (updates return new
  values, sharing structure). Mutation, where needed, is in a `space` (`36 §4`).
- Because the runtime is content-addressed (`../40-runtime/41`),
  structurally-equal collections are **shared and O(1)-comparable** — Ken
  exposes this as the default semantics of value equality.
- `List` is the canonical *inductive* collection (good for proofs/`match`);
  `Array`/`Map`/`Set` are *abstract* types with proven operations (and, where
  relevant, laws as propositions — e.g. `Map` lookup/insert laws, usable by the
  verification layer).

## 3. Iteration

- **Structural recursion / `match`** is the primitive way to consume `List` and
  other inductives (`34 §3`), and is what the verification layer reasons over.
- **Higher-order combinators** — `map`, `filter`, `fold`/`reduce`, `zip` — are
  ordinary `view`s in the stdlib over the collection types, with laws
  (functor/fold laws) available as propositions.
- **Comprehensions / `for`** (if included) are sugar over the combinators
  (OQ-syntax). The semantic core is the combinators + recursion; no special
  iteration protocol is a kernel concept.
- **Streaming / infinite sequences (`OQ-coinduction` DECIDED — deferred).**
  Ken's core is **inductive and total**; it has **no coinductive types and no
  productivity checker** (that machinery is the dual of SCT and the
  guarded-modal growth `OQ-temporal` already declined — deferred until a
  concrete need the idioms below cannot serve). "Defer coinduction" does **not**
  mean "cannot stream" — infinitude is served three ways, all available now:
  - **Generators** — a finite-step, effectful producer (`view … visits [Yield]`,
    the iterator idiom): each step terminates; the "ongoing" is the consumer's
    loop, not an infinite value.
  - **`Lazy a` streams** — an explicit lazy sequence built on the opt-in `Lazy`
    thunk (`../40-runtime/42 §2`) with a **fuel / depth bound**: `take n`,
    unfold-to-depth. Finite-by-construction at every use.
  - **The behavioral seam** — a genuinely forever-running process is a `space`/
    actor with a **total per-message handler** (`36 §4`, `OQ-Space`); the
    "forever" lives in the runtime loop + Ward's temporal model
    (`../70-behavioral/`, `OQ-temporal`), never in a Ken value. A stdlib
    `Stream`/`Iterator` is thus a **library type over these idioms**, not a
    language primitive.

## 4. Equality and ordering

- Value **equality** is structural and content-addressed by default
  (`../40-runtime/41 §4`); `DecEq` instances (`33 §5`) make it usable in
  constraints, and `Eq` (observational equality, `../10-kernel/15`) is the
  propositional version proofs use.
- **Ordering** (`Ord`) is a lawful class (total order propositions provable),
  underpinning `Map`/`Set` and sorting; sortedness is expressible as a
  refinement (`34 §5`) and provable (the canonical verification example,
  `../20-verification/21 §2`).

## 5. What WS-L must deliver here (L3, L8)

`String` (UTF-8, byte/char views), `List`/`Array`/`Map`/`Set`,
`Option`/`Result`, the core combinators with laws, and the equality/ordering
classes — a curated, **lawful** collections core (part of the stdlib,
`../50-stdlib/`). Conformance: `../../conformance/surface/collections/` — UTF-8
length edge cases, persistent- update sharing, and a verified `sort` producing a
`{ xs | isSorted xs }`.
