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

### 2.4 No new kernel rule — small primitive core, derived surface

`String` attaches as a kernel **primitive** (`14 §5`): an opaque type constant
whose inhabitants are **string literals** and the results of a **small,
audited** set of registered primitive ops. That primitive core is deliberately
minimal — the type constant, literals, the native `String ↔ List Char`
round-trip (`string_to_list_char` / `list_char_to_string`, landed in slice 1
`L3-strings-roundtrip`, `§2.3`), and the byte-buffer reads (`byteLength` over
the packed NFC buffer). A primitive op carries a **registered reduction**
(`41`), so e.g. `byteLength "abc" ≡ 3` holds definitionally and proofs can
compute over string literals.

**The string *surface* — `concat` / `slice` / `charAt` / `eq` / the ordering op
— is `derived`, not primitive (`§2.5`).** These lower to ordinary prelude
`view`s over the `List Char` view (`§2.3`), routed through the native round-trip
— **not** registered native primitives. This is the settled slice-2 approach
(Approach A, Architect ruling `evt_4k1yqah3yvpds`): deriving
trivially-structural ops adds **zero** `trusted_base()` delta, where a native
prim would grow the audited reduction surface for no benefit
(subsume-don't-proliferate). So the
primitive set stays small and audited (`18 §5`), and `String` adds **no**
inductive declaration and **no** conversion rule.

Non-definitional string laws (e.g. `byteLength (s ++ t) ≡ byteLength s +
byteLength t`) are **prelude propositions** (`14 §5`, `35 §6.2`), not kernel
reductions.

### 2.5 The derived string surface (`concat` / `slice` / `charAt` / `eq` / …)

The everyday string operations are **derived** — ordinary prelude `view`s over
the `List Char` view (`§2.3`), routed through the native `string_to_list_char`
(`s2l`) / `list_char_to_string` (`l2s`) round-trip (slice 1, landed). They add
**zero** native primitives and **zero** `trusted_base()` delta; each reduces by
unfolding to the `§4.1` `List Char` combinator floor over the real `elim_List` /
`elim_Nat` (`34 §3`). The mandated bodies (Approach A, `evt_4k1yqah3yvpds` — do
**not** native-ize):

```
concat a b   =  l2s (list_append (s2l a) (s2l b))
slice  i j s =  l2s (take (natSub j i) (drop i (s2l s)))
charAt i s   =  nth i (s2l s)                        -- : Option Char
eq     a b   =  list_eq eqChar (s2l a) (s2l b)       -- : Bool
```

- **`charAt` is total and honest about absence.** `nth` returns `None` on an
  out-of-range index and on the empty string — `charAt i "" ≡ None` and
  `charAt 5 "abc" ≡ None` — so the result type is `Option Char`, never a partial
  index (`34 §1` honest sum, not a sentinel).
- **`slice` clamps by construction.** `drop i` past the end yields `Nil` and
  `take n` past the end stops at the end, so an out-of-range `slice` returns the
  available sub-view, never stuck. The length is `natSub j i` — **saturating**
  `Nat` monus (`§4.1`): when `j < i` it is `0`, so `take 0 _ ≡ Nil` and
  `slice j i s ≡ ""` (an empty slice, never an underflow). Indices are
  **code-point** positions (over the `List Char` view), never byte offsets — a
  byte-offset slice could split a multi-byte scalar (`§2.2`, ADR 0010).
- **`eq` is codepoint-wise and rides the landed `eqChar`.** `eq a b` decides
  equality of the two scalar sequences via `list_eq` threading the landed
  `eqChar : Char → Char → Bool` (`= eq_int` under `Char`'s `Int` erasure,
  `decimal_char.rs`). This is the **normative default** (ADR 0010 §2): because
  `s2l`/`l2s` are a round-trip bijection on scalar sequences, `String` is
  **canonical** w.r.t. `List Char`, so codepoint-wise `eq` is sound. **NFC-
  normalization equality is OUT** (`§6`, ADR 0010 §3): it identifies distinct
  scalar sequences, so over the codepoint carrier it is *non-canonical* — a
  lawful `DecEq` for it would inhabit `Bottom`; if ever wanted it is a
  separately-named `Eq`/setoid in a later WP, **never** a `DecEq`/`Ord` here.
- **`compare` is 3-way, codepoint-wise (`§2.5.1`).** `compare a b : OrdResult`
  (`Lt` / `Eq` / `Gt`) is the lexicographic order of the two scalar sequences
  via `list_compare` threading `compareChar`. The landed `Ord Char` is
  `leq`-only (no `compare` method, no `Ordering`/`OrdResult` type on `main`), so
  `compareChar` **repackages** the landed `leqChar`/`eqChar` into 3-way and
  `OrdResult` is a locally-declared, string-surface-**exported** checked
  inductive — zero-TCB-delta (Architect ruling `evt_1stp9sspm6ag8`).

**Deliverability honesty (trust level).** This WP delivers the value-level
*functions* `eq : String → String → Bool` (and the `§2.5.1` ordering op) —
Boolean/decision operations in the tested-not-trusted interpreter ring ("a
wrong value, never a false proof"). Because `String` is **canonical** w.r.t.
`List Char` (ADR 0010 §2), these are *soundly transportable* to lawful `DecEq
String` / `Ord String` **instances** — the canonicity precondition holds here,
unlike `Decimal` (`../10-kernel/18a §5.6.1(4)`). But that transport
additionally needs a lawful **`DecEq Char`**, which is **not yet landed** (only
the `eqChar` *view* +
`Ord Char`-by-transport are on `main`); so the proof-carrying `DecEq String` /
`Ord String` instances are a **tracked follow-on**, not delivered here. This WP
delivers the *functions*; it does **not** ship the lawful instances — filing the
functions as proof-carrying instances would over-claim the trust level.

#### 2.5.1 String ordering — the 3-way `compare` and `OrdResult`

`compare` is a **3-way** codepoint-wise comparison — the more fundamental
ordering op, subsuming `≤` / `<` / `==` (a `leq`-only interface cannot cheaply
recover 3-way). The landed `Ord Char` is `leq`-valued only (`= leq_int`, no
`compare` method), so the surface declares a **local** result type and
**repackages** the landed Char ops:

```
data OrdResult = Lt | Eq | Gt              -- exported from the string surface

compareChar a b =                          -- 3-way from landed leqChar / eqChar
  match eqChar a b {
    True  => Eq
    False => match leqChar a b { True => Lt ; False => Gt }
  }

compare a b = list_compare compareChar (s2l a) (s2l b)   -- : OrdResult
```

Three normative points (Architect ruling `evt_1stp9sspm6ag8`):

- **Named `OrdResult`, not `Ordering`** — matching the landed `natCmp` precedent
  (`val1_string_literals.rs:334`) and ES2's retired-from-prelude name; do not
  introduce a second name for the concept.
- **Local to the string surface and exported.** ES2 retired `OrdResult` from the
  *prelude* as un-referenced bloat, and explicitly sanctions a *local*
  declaration where genuinely needed (the sanction-comment sits on the `val1`
  precedent); a string `compare` is a genuine 3-way need. It is **not**
  re-promoted to the prelude here — that would reopen ES2's retirement, and one
  consumer does not justify prelude-global (YAGNI). **Forward note (not this
  WP):** when a *second* consumer lands (verified `sort` / `Map`/`Set` ordering
  will want the same type), that WP raises "≥2 consumers → promote `OrdResult`
  to a shared location" as a subsume decision to the Steward.
- **`compareChar` repackages, it does not re-derive.** Settled input #4 forbids
  re-*deriving* a Char comparison; `compareChar` reuses the landed `leqChar` /
  `eqChar` verbatim and only wraps their results 3-way. It is a faithful 3-way
  of the landed total order (`Eq` on `eqChar`; else `Lt` / `Gt` by `Ord Char`'s
  antisymmetry + totality) and a `declare_def` — a bug is a wrong value, never a
  false proof. **Zero-TCB-delta:** `OrdResult` is a **checked inductive**
  (kernel-admitted by positivity, not a postulate/primitive), and `compareChar`
  / `list_compare` are `declare_def`s.

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

### 4.1 The `List Char` combinator floor (derived, zero-TCB-delta)

The `§2.5` string surface bottoms out in a small floor of **generic** `List a` /
`Nat` combinators. Each is a **termination-checked recursive derived def** — a
member of a `declare_recursive_group`, kernel-checked, `sct_check`-certified,
and `declare_def`-registered (upgrading opaque → transparent on SCT success).
They lower to the **real** eliminators: a `match` on the recursive argument
elaborates to a `Term::Elim` over the `List` / `Nat` family (`34 §3`) — **not**
a bespoke reducer, and **not** a registered `elim_List` / `elim_Nat` constant
(there is no such constant; the eliminator is the generic `Term::Elim { fam }`).
Because they are `declare_def`s (checked), the floor adds **zero**
`trusted_base()` delta.

Every recursion shape here already elaborates + SCT-passes on `main` — the
de-risking precedent is in `crates/ken-elaborator/tests/l3a_acceptance.rs` (and,
for `natSub`, `crates/ken-elaborator/tests/val1_string_literals.rs`), per the
Architect capability confirm (`evt_4k1yqah3yvpds`):

| Combinator | Signature | Recursion (decreasing arg) | Landed precedent |
|---|---|---|---|
| `list_append` | `{a} → List a → List a → List a` | 1st list, `Cons` tail | `map` (simpler than) |
| `nth` | `{a} → Nat → List a → Option a` | `Cons` tail + `Suc` pred | `map` |
| `take` | `{a} → Nat → List a → List a` | `Suc` pred (`Nat` fuel) | `unfoldUpTo` |
| `drop` | `{a} → Nat → List a → List a` | `Suc` pred (`Nat` fuel) | `unfoldUpTo` |
| `natSub` | `Nat → Nat → Nat` | `Suc` preds (saturating) | `natSub` (val1) |
| `list_eq` | `{a} → (a→a→Bool) → List a → List a → Bool` | both `Cons` tails | `zip` |
| `list_compare` | `{a} → (a→a→OrdResult) → List a → List a → OrdResult` | both `Cons` tails | `zip` / `insert` |

The frame named **6** combinators; this is **7** — `natSub` (the saturating
`Nat` monus `slice` needs, `§2.5`; the frame assumed a landed `sub` that does
not exist) and `list_compare` over the locally-declared `OrdResult` (`§2.5.1`;
`list_compare` replaces the frame's `list_compare : … → Ordering`, which named a
type that does not exist — Architect ruling `evt_1stp9sspm6ag8`). Both are the
same Approach-A derived-def shape; the delta is a count-note, not a soundness
change.

**Mandated defining equations** (structural — every recursive call is an
**applied** call whose decreasing argument is a **strict subterm** of a matched
argument, `elim`-driven, `34 §3`). Shown with implicit `{a}` and pattern
clauses; the build follows the landed explicit-type-argument, nested-`match`
style (`l3a_acceptance.rs`). `OrdResult`'s `Lt`/`Eq`/`Gt` are from `§2.5.1`:

```
list_append Nil          ys = ys
list_append (Cons x xs)  ys = Cons x (list_append xs ys)

nth _        Nil          = None
nth Zero     (Cons x _)   = Some x
nth (Suc n)  (Cons _ xs)  = nth n xs

take Zero    _            = Nil
take _       Nil          = Nil
take (Suc n) (Cons x xs)  = Cons x (take n xs)

drop Zero    xs           = xs
drop _       Nil          = Nil
drop (Suc n) (Cons _ xs)  = drop n xs

natSub a        Zero      = a
natSub Zero     (Suc _)   = Zero
natSub (Suc m)  (Suc n)   = natSub m n

list_eq eq Nil         Nil         = True
list_eq eq Nil         (Cons _ _)  = False
list_eq eq (Cons _ _)  Nil         = False
list_eq eq (Cons x xs) (Cons y ys) = match eq x y {
  True  => list_eq eq xs ys        -- short-circuits, first mismatch decides
  False => False
}

list_compare cmp Nil         Nil         = Eq
list_compare cmp Nil         (Cons _ _)  = Lt      -- shorter prefix < longer
list_compare cmp (Cons _ _)  Nil         = Gt
list_compare cmp (Cons x xs) (Cons y ys) = match cmp x y {
  Eq => list_compare cmp xs ys     -- first difference decides; else recurse
  Lt => Lt
  Gt => Gt
}
```

- **AC2 — SCT sound-zone (soundness, Architect brief-condition 1).** Confirm
  per-combinator that the recursive call is an **applied** call decreasing on a
  strict subterm — the `Cons` tail (`list_append` / `nth` / `take` / `drop` /
  `list_eq` / `list_compare`) or the `Suc` predecessor (`nth` / `take` / `drop`
  / `natSub`). The floor does **not** lean on the SCT to bless *unapplied*
  self-reference or recursion-through-an-opaque-`Map`, where the SCT
  over-accepts (a bare self-`Const` is modelled all-`Unknown` and **rejected**;
  certification requires an applied call carrying a `Down` argument,
  `ken-kernel/src/sct.rs`).
  None of these combinators need that shape — the check is a cheap, concrete
  per-combinator confirmation, squarely in the SCT's sound zone.
- **AC6 — name hygiene (Architect brief-condition 2).** `list_append` is a
  **distinct** name; it must **not** shadow the `Bytes`-domain `append`
  (FS-effect — `append : Bytes → Bytes → Bytes visits [FS]`,
  `crates/ken-elaborator/src/bytes.rs`). Module-qualify if the surface would
  otherwise resolve the wrong op. The other floor names are currently free —
  `nth` / `take` / `drop` / `natSub` / `list_eq` / `list_compare` do not collide
  with landed globals (grep-verified at authoring; re-verify at build).
- **Totality (AC7).** Each combinator is total on well-typed input — `natSub`
  **saturates** at `0` (never underflows), `nth` totalizes out-of-range to
  `None`, `take` / `drop` totalize out-of-range to `Nil` / the remainder. No
  well-typed application reduces to `Neutral` / stuck.

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

**Ordering — explicit comparator, with the lawful `Ord` class supplying it
(ES2-remainder pin `evt_3cn9v6em54yej`; class realized by ES4-classes,
`../50-stdlib/51-lawful-classes.md`).** The verified `sort` threads an
**explicit comparator** `leq : a → a → Bool` — the minimal mechanism that yields
a real, prover-unfoldable specification. The **lawful `Ord a` structure class**
(its total-order law fields — reflexivity of `≤`, antisymmetry, transitivity,
totality — *proved* not postulated) is defined in `51-lawful-classes.md`; per
`33 §5.4`, `where Ord a` **subsumes** the explicit comparator by supplying the
dictionary's `leq` (`d.leq`) to this **same** `sort` — no second `sort`, no new
mechanism (reflect-don't-extend). It is `Ord`'s `total`/`antisym` laws that let
a verified `sort` *discharge* the sortedness obligation this explicit-comparator
form only *states*. Ordered `Map`/`Set` operations (`§3.3`) likewise use
built-in comparators (the L-classes boundary below).

**The verified `sort` (the canonical verification example).** `sort` takes an
explicit comparator and produces a **refinement-typed** result (`34 §5`):

```
view sort {a} (leq : a → a → Bool) (xs : List a)
    : { ys : List a | isSorted leq ys ∧ Perm ys xs } = …
```

This matches the landed AC6 `sort` surface exactly (`l3a_acceptance.rs`, the
`leq : a → a → Bool` comparator after the ES2 `OrdResult → Bool` migration) — no
`where`-constraint threading, no new surface. The refinement carries **two**
conjuncts, and the second is **load-bearing**:

- `isSorted leq ys` — `ys` is in non-decreasing `leq`-order (a decidable
  refinement predicate, `34 §5`).
- `Perm ys xs` — `ys` is a **permutation** of the input (comparator-free).

`isSorted`-**alone is degenerate**: `sort _ = Nil` satisfies
`{ ys | isSorted leq ys }` (the empty list is vacuously sorted), so a
sortedness-only spec is met by a **constant-`Nil`** implementation that throws
the input away — it guards nothing
(the discriminating-example / refinement-must-not-be-vacuous discipline). The
`Perm ys xs` conjunct is what forces `sort` to actually *be* a sort. The
elaboration **emits the conjoined obligation**
`isSorted leq (sort leq xs) ∧ Perm (sort leq xs) xs` on the result introduction
(`34 §5`, `22 §2.1`); a verified
`sort` discharges it with a bundled proof (AC6 observes the **emitted VC**
structurally — per the untrusted-layer lesson, the obligation must be *emitted*,
not assumed).

**The refinement predicates are definitions, not postulates (ES1).** The
obligation `isSorted leq (sort leq xs) ∧ Perm (sort leq xs) xs` is dischargeable
only if the prover can **unfold** `isSorted` and `Perm` — so both are
**definitions** (`Ω`-valued, re-checked, **out** of `trusted_base()`), never
opaque postulates. As `declare_postulate`s (their current `prelude.rs` form) the
predicates are **undefined**: `isSorted leq (sort leq xs)` cannot reduce, so the
obligation is either **undischargeable** or discharged **circularly** (the proof
assuming the
conclusion), and the flagship verified `sort` would prove **nothing**
(`30 §6`, the surface-minimality invariant; ES2 lands the demotion). The
defining shapes:

- **`isSorted : Π(a : Type). (a → a → Bool) → List a → Ω`** — an `Ω`-valued
  structural recursion over the **explicit comparator**: `isSorted leq Nil = ⊤`,
  `isSorted leq (x :: Nil) = ⊤`, and
  `isSorted leq (x :: y :: r) = IsTrue (leq x y) ∧ isSorted leq (y :: r)` (the
  connective is the derived Ω-conjunction `And`, `16 §1.3`; the recursion
  descends structurally on the list, so it terminates). The comparator is
  `Bool`-valued (matching the landed `sort`), so the order relation enters `Ω`
  through the **bridge** `IsTrue (leq x y) := Eq Bool (leq x y) True : Ω` — a
  proof-irrelevant proposition (both `Bool` as real `data Bool = True | False`
  and `Eq _ : Ω` are landed by ES2). It **must** land in `Ω` (proof-irrelevant);
  a `Type`-sorted "predicate" leaks content into the refinement carrier
  (`13 §4` / `16 §8.2`). (With the lawful `Ord a` class,
  `../50-stdlib/51-lawful-classes.md`, `where Ord a` supplies this **same**
  `leq : a → a → Bool` from the dictionary — the `IsTrue` bridge is
  **unchanged**; the class changes only *where* `leq` comes from, `33 §5.4`.)
- **`Perm : Π(a : Type). List a → List a → Ω`** — a permutation **must** be
  `Ω`-valued, and a bare inductive relation is **not**:
  `data Perm_rel := perm_refl | perm_swap | perm_trans | perm_cons` is
  proof-**relevant** (a proof records *which* permutation) so it lands in
  `Type`, and `16 §1.3` **forbids** a proof-relevant `Type → Ω` directly (it
  would admit `Bool`, collapsing `true ≡ false` by Ω-PI). **Pinned form
  (ES2-remainder ruling `evt_3cn9v6em54yej`, closing ES1's "spec picks one"
  fork) — the truncation:**

  `Perm xs ys := ∥ Perm_rel xs ys ∥ : Ω`

  — propositional truncation of the `Type`-level inductive (the `∨ := ∥+∥` /
  `∃ := ∥Σ∥` pattern, `16 §6`; proof-irrelevant). It is **comparator-free** and
  carries **no `DecEq a` / `Ord a` dependency** — the decisive reason to prefer
  it. The rejected alternative, **count-equality**
  `Perm xs ys := Π (x : a). Eq Nat (count x xs) (count x ys)`, is natively `Ω`
  (a `Π` of `Eq`s) but requires `DecEq a` to `count` occurrences (counting
  compares elements) — dragging in exactly the class dependency this ruling
  defers. `Perm` therefore uses the truncation, **not** count-equality.

  Declaring the bare inductive `: Ω` is the relevance leak `16 §1.3`/`13 §4`
  forbid (CV's table surfaced this fork; CV-Spec blocked on it).

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
- **Refinement `{ ys : List a | isSorted leq ys ∧ Perm ys xs }`** — carrier
  `List a` at its level; the predicate is `Ω`-valued (`12 §5`/`16 §1`),
  discharged as a V3 obligation, **no** universe bump (`34 §5`/§7).

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
- **AC6 (the verified example, structural).** `sort` (threading the explicit
  `leq : a → a → Bool`) produces `{ ys | isSorted leq ys ∧ Perm ys xs }` — the
  **conjoined** refinement obligation is **emitted** and dischargeable; assert
  the **`Perm` conjunct is present** (a sortedness-only obligation is degenerate
  — `const Nil` satisfies it). `isSorted`/`Perm` are the pinned **definitions**
  (`§6`: explicit-comparator `isSorted`, `Perm := ∥Perm_rel∥`), unfoldable — not
  postulates (the demotion is the ES2-remainder follow-on).

**Derived string surface — slice-2 acceptance (`§2.5` / `§2.5.1` / `§4.1`,
impl-ready).** The floor + 5 string ops, mapping the WP frame's AC1–AC7:

- **DS-AC1/AC5 (floor registered, zero-TCB-delta).** All **7** floor combinators
  (`§4.1`) and `compareChar` **producer-grep** as `declare_recursive_group` /
  `declare_def` members over the real `Term::Elim` (not hand-fed, not a bespoke
  reducer); `OrdResult` grep as a **checked inductive** (`data`, not a
  `declare_primitive` / `declare_postulate` / `declare_opaque`). `git diff
  origin/main -- crates/ken-kernel/` is **empty**; `trusted_base()` unchanged.
- **DS-AC2 (SCT sound-zone).** Each combinator's recursive call is an applied
  call on a strict subterm (`§4.1`) — not leaning on the SCT's over-accept zone.
- **DS-AC3 (5 ops reduce correct).** `concat` / `slice` / `charAt` / `eq` /
  `compare` reduce to the **correct value** on a multi-byte corpus (reuse slice
  1's boundary corpus, through the real `s2l`/`l2s`): `charAt` → `None` on
  out-of-range **and** empty; `slice` clamps, incl. `j < i → ""`.
- **DS-AC4 (`eq`/`compare` codepoint-wise, discriminating PAIR).** A
  **non-degenerate pair**: `eq` **accepts** two equal scalar sequences **and
  rejects** a differing pair (incl. a same-length, single-codepoint-differing
  pair); `compare` gives `Lt` / `Eq` / `Gt` correctly on the ordered triple
  `"a" < "ab" < "b"`. Assert the **result value**. A canonically-equivalent but
  codepoint-**distinct** pair (NFC vs NFD of one grapheme) must compare
  **unequal** — pins that NFC-eq was **not** smuggled in (ADR 0010 §3).
- **DS-AC6 (name hygiene).** `list_append` does not shadow the `Bytes` `append`
  (`§4.1`); both resolve to their intended op.
- **DS-AC7 (round-trip / totality).** `concat`+`slice` compose sanely (e.g.
  `slice 0 (charLength a) (concat a b) ≡ a` on scalar-clean `a`); `list_append`
  associativity on a small corpus; every combinator total (no `Neutral`/stuck).

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
