# Strings and collections

> Status: **impl-ready (L3).** The data types a programmer reaches for daily:
> `String`, `List`/`Array`/`Map`/`Set`, `Option`/`Result`, the core combinators,
> and structural equality/ordering. These are **stdlib** (`../50-stdlib/`) over
> the **landed** content-addressed value model (`../40-runtime/41-values.md`) and
> the **landed** L2 `data` machinery (`34-data-match.md`) — **no new kernel
> rule**. This chapter fixes their *shape*, their lowering to `41`, and their
> laws as propositions.
>
> **L3 scope (this WP).** §1 the decision (state inductively, do not coinduct);
> §2 `String`; §3 the collection types + persistence/sharing; §4 combinators +
> laws; §5 infinitude without coinduction; §6 equality, ordering, and the
> verified `sort`; §7 the no-coinduction structural absence; §8 level-discipline
> + pinned-vs-deferred; §9 deliverables + acceptance. The **exact API spelling**
> (method names, the `Map`/`Set`/`Array` internal representation) is `(oracle)`-
> tagged for the build team; the **concept, the laws, the lowering, and the
> staging** are pinned here.
>
> **Perishable — pin against the landed code, not this prose.** `String` rides
> the landed `41 §3a` NFC-normalized encoding; collections ride `41 §2`
> persistence + the `41 §3a` kind tags; cross-declaration combinator references
> ride the landed `L-resolver-globals` fallback (`c3a3f1d`). Verify each against
> `41`/`ken-elaborator` before building.

## 1. The decision: state collections inductively, do not coinduct

Ken's collections core is **inductive and total**. `List` is an ordinary
inductive `data` (`34 §1`); `Array`/`Map`/`Set` are abstract types over the
content-addressed heap; every operation is a **terminating** function and every
law is an ordinary **proposition**. There is **no coinductive type, no `codata`,
and no productivity/guardedness checker** (`OQ-coinduction` DECIDED — deferred;
the dual of SCT and the declined `OQ-temporal` modal growth, `§7`).

This is the same reflect-don't-extend move the kernel makes elsewhere: "defer
coinduction" does **not** mean "cannot stream." Genuine infinitude is served by
**inductive idioms** — a fuel-bounded unfold, an opt-in `Lazy` thunk, an
effectful generator, or the behavioral seam — none of which add a coinductive
value or a new kernel rule (`§5`). The decision is **durable**, not a v1
expedient: a `Stream`/`Iterator` is a *library type over these idioms*, never a
language primitive.

## 2. Strings and text

`String` is an immutable **UTF-8** text value and a **primitive type** (an
opaque type constant, `../10-kernel/14 §5`); it is **content-addressed** like
any compound value (`41 §2`), so equal strings share one slot and equality is
**O(1)** (`41 §4`). It is **not** `List Char` — that is a separate, convertible
*view* (`§2.3`). The runtime representation is a **packed byte buffer**, NFC-
normalized at construction (`41 §3a`, kind tag `0x04`).

### 2.1 NFC normalization is at the address, not just the surface

A `String` is interned by the **canonical bytes of its NFC-normalized UTF-8**
(`41 §3a`: "normalization is performed at construction time; the normalized form
is stored"). Two consequences are normative and load-bearing:

- **O(1) equality is NFC-aware.** `s == t` is a slot-id comparison (`41 §4`);
  because both were NFC-normalized before interning, two strings that are
  canonically-equivalent under NFC **share a slot** and compare equal in O(1).
  Equality is decided once, at intern time, never by re-traversal.
- **`byteLength` is over the *stored* (normalized) bytes.** Length-in-bytes
  reports the NFC byte buffer's length, not the pre-normalization source. This
  is the only length a program can observe, and it is deterministic.

### 2.2 Byte-length and char-length are distinct (the UTF-8 trap)

The API **distinguishes** two lengths, and they differ on any string with a
multi-byte code point:

- **`byteLength s`** — the number of bytes in the stored UTF-8 buffer.
- **`charLength s`** — the number of **Unicode scalar values** (code points);
  equal to the length of the `List Char` view (`§2.3`).

For an ASCII string the two coincide; for `"é"` (one code point, NFC `U+00E9`,
two UTF-8 bytes) `byteLength = 2` while `charLength = 1`. Indexing is **by code
point** or by an **explicit byte view** — never an ambiguous "length"/"index"
that silently means one or the other. This is the headline correctness property
of treating `String` as packed UTF-8 rather than `List Char` (AC1).

### 2.3 Convertible views — `String ↔ List Char`, `String ↔ Bytes`

`String` is *not* `List Char`, but the two are inter-convertible; likewise
`String` and `Bytes` (`38`). The conversions and their **totality** are pinned;
the literal method names are `(oracle)`-tagged (`§8`):

| Conversion | Totality | Meaning |
|---|---|---|
| `String → List Char` | total | decode code points (the `charLength`-long view) |
| `List Char → String` | total | encode UTF-8, **then NFC-normalize** + intern |
| `String → Bytes` | total | the stored NFC UTF-8 buffer (`byteLength`-long) |
| `Bytes → String` | **partial → `Result`** | validate UTF-8, NFC-normalize, intern |

`Bytes → String` is the one partial direction: arbitrary bytes need not be valid
UTF-8, so it returns `Result DecodeError String` (`§6` uses `Result`, not a
sentinel). `Char` is a Unicode scalar value (`35 §2.4`: `u32`, range
`U+0000–U+10FFFF` excluding the surrogate block `U+D800–U+DFFF` — a refinement
on the carrier, so `List Char → String` cannot encode an invalid scalar).

### 2.4 No new kernel rule

`String` attaches as a kernel **primitive** (`14 §5`): an opaque type constant
whose inhabitants are literals and the results of registered primitive ops
(concat, slice, `byteLength`, code-point indexing). A primitive op carries a
**registered reduction** (`41`), so e.g. `byteLength "abc" ≡ 3` holds
definitionally and proofs can compute over string literals. Non-definitional
string laws (e.g. `byteLength (s ++ t) ≡ byteLength s + byteLength t`) are
**prelude propositions** (`14 §5`, `35 §6.2`), not kernel reductions. The
primitive set stays small and audited (`18 §5`); `String` adds **no** inductive
declaration and **no** conversion rule.

## 3. The core collection types

| Type | Kind | Lowering | Equality |
|---|---|---|---|
| `List a` | transparent inductive `data` (L2) | `data List a = Nil \| Cons a (List a)` (`34 §1`) | structural, O(1) (`41 §4`) |
| `Array a` | abstract (persistent index tree) | content-addressed, kind `0x06` (`41 §3a`) | structural, O(1) |
| `Map k v` | abstract (`DecEq`-keyed) | content-addressed, kind `0x07`, key-sorted canonical form | structural, O(1) |
| `Set a` | abstract (`DecEq`-keyed) | content-addressed, kind `0x08`, element-sorted canonical form | structural, O(1) |
| `Option a` | transparent inductive `data` (L2) | `data Option a = None \| Some a` (`34 §1`) | structural, O(1) |
| `Result e a` | transparent inductive `data` (L2) | `data Result e a = Err e \| Ok a` (`34 §1`) | structural, O(1) |

All core collections are **immutable and persistent**: an "update" allocates the
changed spine and **shares** the unchanged rest (`41 §2`, append-mostly heap).
Mutation, where genuinely needed, lives only in a `space` (`36 §4`), never in a
collection value.

### 3.1 `List` — the transparent inductive (canonical for proofs)

`List a` is the ordinary inductive `data` of `34 §1`: `Nil`/`Cons`, pattern-
matchable, consumed by `elim_List` (`34 §3`). It is the **canonical** collection
for verification — `match`, structural recursion, and the per-branch refinement
of `34 §3.3` all apply directly. `List` is **transparent**: a program may
`match` on its constructors. (`Option`/`Result` are the same L2 story, reused
here, not re-declared.)

### 3.2 `Array` — the persistent index tree (abstract)

`Array a` is an **abstract type** (`33 §4` module abstraction): an opaque
carrier plus a lawful interface (`get`, `set`, `push`, `length`,
`fromList`/`toList`). The carrier is a **persistent index tree** (a chunked /
radix-balanced tree), each node content-addressed (`41 §3a`, kind `0x06`). This
reconciles the two requirements the frame pins together — they are in tension
for a flat buffer:

- **Effectively-constant index.** Index is a bounded-depth tree descent
  (`O(log_b n)` for a large branching factor `b` — the persistent-vector
  standard), reported as "O(1)" at the interface; a flat `O(1)`-index buffer
  **cannot** also share structure on update, so the tree is the honest
  reconciliation, not a literal `O(1)` claim (honesty-about-the-boundary).
- **Structural sharing on update.** `set i x` allocates only the path from the
  root to the changed chunk and **shares** every other node (`41 §2`); the
  result is a new value O(1)-comparable to siblings via slot-id. The unchanged
  subtrees are *the same slots*.

`Array` is **abstract**, so it is consumed through its interface, never by
`match` on its internal tree; the exact branching factor / representation
(RRB-tree, HAMT-vector, chunk size) is an `(oracle)`/X2 tuning (`§8`), invisible
to the laws.

### 3.3 `Map` and `Set` — `DecEq`-keyed, canonically ordered

`Map k v` and `Set a` are abstract types (`33 §4`) over the content-addressed
heap (kinds `0x07`/`0x08`). Their **canonical form is sorted by the
lexicographic order of the canonical byte encoding of each key/element** (`41
§3a`), so:

- **Insertion-order-independent identity.** A `Map`/`Set` built in two different
  insertion orders **interns to the same slot** (`41 §3a`: "built in two
  different insertion orders encodes identically") ⇒ structural O(1) equality
  for free, no user `Ord` required for *identity*.
- **`DecEq k` is the membership constraint.** `lookup`/`member`/`insert` need
  **decidable key equality** — `Map k v` and `Set a` carry a `where DecEq k`
  (resp. `DecEq a`) constraint (`33 §5`). A key type **without** `DecEq` is a
  **compile error** (the constraint is unsatisfiable), naming the missing
  instance — the AC5 verdict flip.
- **`Ord k` enables ordered operations.** `Ord k` (`§6`) is **not** required for
  the core map (the canonical byte order already totally orders stored keys); it
  is the constraint for *ordered* operations — `minKey`, `maxKey`, range
  queries, ordered fold. Pin the split: `DecEq` for membership, `Ord` for order.

`Set a` is **`Map a Unit`** semantically (the DRAFT's framing); whether it is
literally that or a distinct kind-`0x08` value is an `(oracle)` representation
choice — the laws (`§4`) and equality are identical either way.

### 3.4 Persistence and sharing (the runtime contract)

Persistence is **not** a per-collection reimplementation: it is the content-
addressed heap's append-mostly immutability (`41 §2`). "Updating" any collection
allocates the changed spine and shares the rest; the shared sub-structures are
the **same slots**, so sharing is observable as slot-id equality (the
conformance asserts the *sharing*, `41`-style, not merely a correct result —
AC2). Mutable cells exist only in a `space` (`36 §4`) and are explicitly
**not** content-addressed.

## 4. Iteration and combinators (laws as propositions)

**Structural recursion / `match`** is the primitive way to consume `List` and
other inductives (`34 §3`); it is what the verification layer reasons over. The
higher-order combinators — `map`, `filter`, `fold`/`reduce`, `zip` — are
ordinary **prelude `view`s** over the collection interfaces (`../50-stdlib/`),
**not** a kernel iteration protocol. Comprehensions / `for`, if included, are
**sugar** over the combinators (`OQ-syntax`); the semantic core is combinators +
recursion.

The **laws are propositions** (`14 §5`, `21 §3`), stated and proved in the
prelude, usable by the verification layer — they add **no kernel rule**:

```
-- functor laws (for List, Array, Option, …)
map_id    : map id xs           ≡ xs
map_comp  : map (g ∘ f) xs       ≡ map g (map f xs)
-- fold / fusion
fold_fusion : h (fold f z xs)    ≡ fold f' (h z) xs        -- given the fusion premise
-- Map lookup/insert (the canonical algebraic spec)
lookup_insert_eq  : lookup k (insert k v m)            ≡ Some v
lookup_insert_neq : k ≠ k' → lookup k (insert k' v m)  ≡ lookup k m
```

These are `≡`-propositions over the combinators/operations, dischargeable by the
prover (AC3 observes the **emitted obligation** structurally, not "it
compiles"). A combinator law in one declaration may reference a combinator
defined in **another** declaration (`map_id` references `map`): this
**cross-declaration lowercase reference** is supported by the landed
`L-resolver-globals` fallback (`c3a3f1d`: an `EVar` scope-miss falls through to
a global `RCon` lookup, locals still shadowing globals) — the L2-build resolver
limitation the frame flagged is **resolved**, verified against the on-`main`
elaborator. No resolver sub-WP is required for L3.

## 5. Infinitude without coinduction

Ken serves genuine infinitude with **inductive, total** idioms — *staged by what
they depend on*, so the build team's **mandated** demonstration rests only on
**landed L2** machinery and never on a deferred primitive (a defer-spelling-not-
concept corollary: a buildable-now deliverable must not depend on a deferred
spelling — see the B2 carry). In dependency order:

1. **Fuel-bounded inductive unfold — buildable now, L2 only (the mandated
   demonstration).** A structurally-recursive unfold to a finite prefix:

   ```
   unfoldUpTo : (s → Option (a × s)) → Nat → s → List a
   unfoldUpTo step Zero    s = Nil
   unfoldUpTo step (Suc n) s = match step s {
     None          => Nil
     Some (a , s') => Cons a (unfoldUpTo step n s')
   }
   ```

   This terminates by structural descent on the `Nat` fuel — an **ordinary total
   `List`-producing function** over the landed `34 §1` `data` (`List`, `Option`,
   `Nat`) with **no** coinductive value, **no** `Lazy`, **no** effect. `take n`
   over a generating step is exactly this. **This is the AC4-mandated working
   demonstration** — it rests solely on the pinned/landed concept.
2. **`Lazy a` streams (rides `42 §2`; force/memo may be deferred for G1).** An
   explicit lazy sequence on the opt-in `Lazy` thunk (`42 §2`) with a fuel/depth
   bound (`take n`), finite-by-construction at every use. **Staging flag:** `42
   §2` defers the `Lazy` force/memo primitive for G1; this idiom is therefore
   **not** the buildable-now demonstration — it lands when `Lazy` does. (This is
   precisely why item 1, not this one, is the mandated deliverable.)
3. **Generators (`view … visits [Yield]`, rides L5).** A finite-step effectful
   producer (`36`): each step terminates; the "ongoing" is the consumer's loop,
   not an infinite value. Available once L5 effects are in play.
4. **The behavioral seam (`36 §4`, Ward).** A genuinely forever-running process
   is a `space`/actor with a **total per-message handler**; the "forever" lives
   in the runtime loop + Ward's temporal model (`../70-behavioral/`), never in a
   Ken value.

A `Stream`/`Iterator` is a **library type over these idioms**, not a language
primitive — and crucially, none of the four introduces a coinductive type or a
productivity checker (`§7`).

## 6. Equality, ordering, and the verified `sort`

**Equality** is structural and content-addressed by default (`41 §4`): on heap
values `a == b` is a slot-id comparison (O(1)); `DecEq` (`33 §5`) makes it
usable in constraints (its `eq` returns `Bool` with the `ok` law tying it to the
kernel's `Eq`); `Eq` (observational equality, `15`/`16`) is the propositional
version proofs use. `DecEq` is a **structure class** (`33 §5`): genuinely many
can exist per carrier, so it follows the canonical-instance resolver convention.

**Ordering** `Ord a` is a **lawful structure class** carrying its total-order
law proofs (reflexivity of `≤`, antisymmetry, transitivity, totality) — usable
by the prover, underpinning ordered `Map`/`Set` operations (`§3.3`) and sorting.

**The verified `sort` (the canonical verification example).** `sort` requires
`Ord a` and produces a **refinement-typed** result (`34 §5`):

```
view sort {a} (xs : List a) : { ys : List a | isSorted ys ∧ Perm ys xs }
  where Ord a = …
```

The refinement carries **two** conjuncts, and the second is **load-bearing**:

- `isSorted ys` — `ys` is in non-decreasing `Ord`-order (a decidable refinement
  predicate, `34 §5`).
- `Perm ys xs` — `ys` is a **permutation** of the input.

`isSorted`-**alone is degenerate**: `sort _ = Nil` satisfies `{ ys | isSorted ys
}` (the empty list is vacuously sorted), so a sortedness-only spec is met by a
**constant-`Nil`** implementation that throws the input away — it guards nothing
(the discriminating-example / refinement-must-not-be-vacuous discipline). The
`Perm ys xs` conjunct is what forces `sort` to actually *be* a sort. The
elaboration **emits the conjoined obligation** `isSorted (sort xs) ∧ Perm (sort
xs) xs` on the result introduction (`34 §5`, `22 §2.1`); a verified `sort`
discharges it with a bundled proof (AC6 observes the **emitted VC** structurally
— per the untrusted-layer lesson, the obligation must be *emitted*, not
assumed).

**The refinement predicates are definitions, not postulates (ES1).** The
obligation `isSorted (sort xs) ∧ Perm (sort xs) xs` is dischargeable only if the
prover can **unfold** `isSorted` and `Perm` — so both are **definitions**
(`Ω`-valued, re-checked, **out** of `trusted_base()`), never opaque postulates.
As `declare_postulate`s (their current `prelude.rs` form) the predicates are
**undefined**: `isSorted (sort xs)` cannot reduce, so the obligation is either
**undischargeable** or discharged **circularly** (the proof assuming the
conclusion), and the flagship verified `sort` would prove **nothing**
(`30 §6`, the surface-minimality invariant; ES2 lands the demotion). The
defining shapes:

- **`isSorted : Π{a}. Ord a => List a -> Ω`** — an `Ω`-valued structural
  recursion: `isSorted Nil = ⊤`, `isSorted (x :: Nil) = ⊤`, and
  `isSorted (x :: y :: r) = (x ≤ y) ∧ isSorted (y :: r)` (the connective is the
  derived Ω-conjunction, `16 §1.3`; `≤` is `Ord`'s). It **must** land in `Ω`
  (proof-irrelevant) — a `Type`-sorted "predicate" would leak content into the
  refinement carrier (`13 §4` / `16 §8.2`).
- **`Perm : Π{a}. List a -> List a -> Ω`** — the **inductive relation**
  `data Perm : List a -> List a -> Ω := perm_refl | perm_swap | perm_trans |
  perm_cons`, preferred over the count-equality form
  (`∀ x. count x xs = count x ys`) because it carries **no `DecEq a`
  dependency**. Also in `Ω`.

Neither is prelude — no primitive signature names them (`30 §4`); they are the
verified-`sort` showcase's own definitions.

**L-classes staging boundary (flag, do not resolve).** The collection **types**
and **structural equality** ship in L3 with **built-in** `DecEq`/`Ord` instances
for the primitive and core types (the L1-numerics precedent: built-in now). Full
**user-type instancing** of `DecEq`/`Ord` (a user `instance DecEq MyType`)
depends on **L-classes** (`33 §5`/`39`) and is **gated** there — L3 pins the
boundary, it does not deliver user instancing.

## 7. No coinductive type / no productivity checker (structural absence)

The §1 decision is enforced by a **structural-absence** net — the
grep-for-forbidden-**construct** seal, pinned by the construct's signature, not
a lexeme (the B2/Sec1-N1 absence-discipline; lexemes collide with ordinary
vocabulary). A conforming kernel + surface MUST contain:

- **No coinductive type former** — no `codata` declaration form, no greatest-
  fixpoint type constructor, no `Stream`/`Colist` *kernel* type. (A library
  `Stream` built on `§5`'s idioms is fine — it is a `data`/`Lazy`/effect value,
  not a coinductive former.)
- **No `cofix` / co-pattern / copattern-matching** term former.
- **No productivity or guardedness checker** pass — the kernel's sole structural
  admission gate is **strict positivity** for inductives (`14 §8`) and the
  **SCT** termination measure for recursion (`17 §4`); there is **no** dual
  guardedness analysis.

**Named vocabulary collisions (so the net targets the construct, not the
word):** `Lazy` (`42 §2`) is a **thunk type**, not coinduction; "lazy" WHNF (`42
§2`) is the kernel's *conversion* strategy, not a productivity rule; a stdlib
`Stream` (`§5`) is a *library* type. The net asserts the **absence of the
construct** (`codata`/`cofix`/guardedness-pass AST node or kernel judgment), and
names these three benign homonyms so a build-team grep targets the AST/judgment,
not the string `lazy`/`stream`. This is **durable** (`OQ-coinduction` DECIDED).

## 8. Level-discipline reconcile and pinned-vs-deferred

**Level discipline (editorial — no new formation rule).** Every type here
instantiates a landed kernel rule; per the standing directive each is stated
with its level, and none adds a universe computation:

- **`String`** — a kernel **primitive** at `Type 0` (`14 §5`, `35 §2`); opaque
  constant, no formation rule.
- **`List a` / `Option a` / `Result e a`** — inductive `data` (`14 §1`); for `a
  : Type ℓ`, `List a : Type ℓ` (predicative, `12 §2`); `Result e a` at
  `max(level e, level a)`. The landed `14 §1` formation computes it — **no new
  rule** (`34 §7`).
- **`Array a` / `Set a`** — abstract types at `level a`; `Map k v` at `max(level
  k, level v)`. Abstract carriers over `41`'s heap, no universe bump.
- **Refinement `{ ys : List a | isSorted ys ∧ Perm ys xs }`** — carrier `List a`
  at its level; the predicate is `Ω`-valued (`12 §5`/`16 §1`), discharged as a
  V3 obligation, **no** universe bump (`34 §5`/§7).

**Pinned here (do not reopen).** `String` = content-addressed NFC UTF-8
primitive (not `List Char`); byte-length ≠ char-length; the four
convertible-view totalities (`§2.3`); collections immutable + persistent with
observable sharing; `List`/`Option`/`Result` transparent inductive,
`Array`/`Map`/`Set` abstract; `DecEq` for membership / `Ord` for order; the
combinator law set (`§4`); the fuel-bounded unfold as the buildable-now
infinitude demonstration; `sort`'s **`isSorted ∧ Perm`** refinement; the
no-coinduction absence; the L-classes staging boundary.

**`(oracle)`-deferred to the build team / X2 (spelling, not concept).** The
exact **method names** (`byteLength` vs `lengthBytes`, `get` vs `index`, …); the
**`Array`/`Map`/`Set` internal representation** (RRB-tree / HAMT / bitmap,
branching factor — `41 §5` tiny-aggregate tuning is X2); whether `Set a` is
literally `Map a Unit`; the `DecodeError` payload of `Bytes → String`; the
comprehension/`for` surface (`OQ-syntax`). These are spelling/representation
choices the laws and equality are invariant under.

## 9. What WS-L must deliver here (L3, → L8) and acceptance

Deliver in the surface/elaborator + prelude (lowering to the landed `41`): UTF-8
`String` (byte/char views, the four conversions, `Char`); `List` (L2 `data`),
`Array` (persistent index tree), `Map`/`Set` (`DecEq`-keyed), `Option`/`Result`
(L2); the `map`/`filter`/`fold`/`zip` combinators with their laws as
propositions; the fuel-bounded-unfold infinitude idiom; structural equality +
`DecEq`/`Ord` (built-in instances now); and the verified `sort`. L8 extends this
to the full lawful stdlib; L3 **unblocks T3** (the test/property framework).

**Testable acceptance criteria.**

- **AC1 (`String` UTF-8 primitive, structural).** A `String` is
  content-addressed (NFC-equal strings O(1)-equal, **same slot**) and
  `byteLength ≠ charLength` on a multi-byte string (assert **both**
  view-lengths, not "compiles"); `String` is **not** `List Char` (the
  convertible view is a separate value).
- **AC2 (persistent collections, structural).** `List` pattern-matches (real
  `elim_List`, `34`); an `Array`/`Map` update **returns a new value sharing the
  unchanged structure** — assert the **sharing** (shared sub-structure = same
  slot-id), not merely a correct result.
- **AC3 (lawful combinators, structural).** A functor/fold law (`map_id`, or
  `lookup_insert` for `Map`) is **provable as a proposition** — observe the
  **emitted obligation**, not "it type-checks"; the cross-declaration reference
  resolves (`L-resolver-globals`).
- **AC4 (NO coinduction — the headline guardrail).** Assert **no coinductive
  type former / `cofix` / productivity checker** (the `§7` structural-absence
  net, pinned by construct + the three named homonyms), **AND** a working
  **fuel-bounded `unfoldUpTo … n`** produces a finite `List` prefix —
  infinitude with no coinductive value, on landed L2 only.
- **AC5 (structural equality + `DecEq`, verdict flip).** Structurally-equal
  collections are O(1)-comparable (content-addressed, same slot); a `Map`/`Set`
  with a key type that **has** `DecEq` accepts, while one **lacking** `DecEq`
  **rejects** naming the missing instance — the verdict flips.
- **AC6 (the verified example, structural).** `sort` produces `{ ys | isSorted
  ys ∧ Perm ys xs }` — the **conjoined** refinement obligation is **emitted**
  and dischargeable; assert the **`Perm` conjunct is present** (a
  sortedness-only obligation is degenerate — `const Nil` satisfies it).

**Conformance:** `../../conformance/surface/collections/` — UTF-8
byte/char-length edge cases + the `Bytes → String` partial decode;
persistent-update **sharing** (slot-id); the combinator laws + `Map`
lookup/insert; the verified `sort` with the **`isSorted ∧ Perm`** obligation;
the no-coinduction structural absence + the working `unfoldUpTo`; the
`DecEq`-key verdict flip. Per-case verdict/structural-flip **and** the
cross-case sweep: every collection's equality maps to the content-addressed
slot-id story (`41 §4`); the `DecEq`-membership vs `Ord`-order split is
consistent across `Map` and `Set`.

**QA gate (new-surface WP):** **producer-grep** the `String`/`Array`/`Map`/`Set`
**registration** in `ken-elaborator/src/` (and the `String` primitive in the
kernel primitive set, `18 §5`) **before** counting green — the types must route
through **real** registration, the combinator laws + `sort` through **real**
obligation emission (`22`), the `List` through **real** `elim_List`; **no**
synthetic collection literal or hand-fed obligation where a real elaboration is
asserted (the green-vs-green / hand-fed-deliverable net).
