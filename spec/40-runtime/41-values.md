# The value model

> Status: **Elaborated (F4)**. Normative for the model, equality, and
> canonical encoding. Contract for WS-X **X1/X2** and Foundation's **K3**
> (production store, building on F4's concrete encoding + index design).
> Encodes two design commitments: heterogeneous typed values (not uniform
> f64, §1) and conventional content addressing (FNV-1a + memcmp, not Leech,
> §3). The canonical byte encoding (§3a) and intern algorithm (§3b) are now
> specified at implementation resolution — Foundation implements to these,
> not from first principles.

## 1. Scalars are typed immediates

A Ken value is **not** a uniform `f64` handle. Scalars are **unboxed, typed
machine values**, dispatched by their static type
(`../30-surface/35-numbers.md`):

| Ken type | Runtime immediate |
|---|---|
| `Int` (small) | machine word `i64` (fast path); promotes to a heap bignum when it outgrows the word (§2) |
| `Int64`/`UInt32`/… | the named machine integer |
| `Bool` | `i1` |
| `Char` | `u32` |
| `Float`/`Float32` | `f64`/`f32` |
| `Decimal` | a small struct (coefficient + exponent) |
| handle / heap reference | a tagged pointer / slot id (§3) |

There is **no decode-from-f64 stratum**; the type determines the representation
directly. A `section`/handle crossing a boundary *may* be shuttled as an
integer- in-`f64` **wire convention**, but that is a transport detail, not the
value model (`44`/`../30-surface/38`).

## 2. Compound values and the content-addressed heap

Compound and identity-bearing values — constructor applications (`data`),
records (Σ), `String`, `Bytes`, `Array`/`Map`/`Set`, closures, and big integers
— live in a **content-addressed heap**:

- A value is stored once, keyed by the **hash of its (canonical) content**;
  identical content ⇒ **same slot** ⇒ stored once (global **deduplication**).
- References into the heap are compact **slot ids**; a value is a small
  immediate (scalar) or a slot id (compound).
- The heap is **append-mostly and immutable**: a stored value never changes;
  "updating" a structure allocates the changed spine and **shares** the rest
  (persistent data structures for free). Mutable state is confined to `space`
  cells (`../30-surface/36 §4`), which are *not* content-addressed.

## 3. Addressing: a fast hash + memcmp (NOT lattice geometry)

Content addressing is **conventional**.

- The content key is a **fast non-cryptographic hash (FNV-1a-style)** of the
  canonical byte encoding, with **`memcmp`** to resolve hash collisions exactly.
  Slot ids are a **monotonic counter**. (A cryptographic/Merkle hash is used for
  *serialization/verification*, `../30-surface/38 §1` — a separate concern from
  in-process addressing). **`OQ-hash` DECIDED:** a fast non-cryptographic hash +
  `memcmp` in-process, a cryptographic/Merkle hash for serialization — two
  hashes, two jobs; the exact functions are an X2 constant.
- **No Leech-lattice quantizer, no Co₀-orbit canonicalization on the allocation
  path.** Heap addressing is not lattice geometry; Ken MUST NOT put lattice math
  on the hot path. (The lattice's *legitimate*, optional, separate roles are in
  `44 §4`.)
- **Canonical encoding.** Dedup requires a canonical byte form per value so that
  "same value ⇒ same bytes ⇒ same hash." The canonicalization rules (field
  order, normalization of which values are interned) are part of X2 and must be
  deterministic.

### 3a. Canonical byte encoding (F4-elaborated)

Every content-addressed value has a deterministic canonical byte form.
The encoding is specified in full in `docs/design/content-addressing.md
§1`; this section states the normative rules that Foundation implements.

**Kind tags.** Each encoding is prefixed by a 1-byte kind tag from a single
namespace (see the design doc §1.1 for the full table). Currently assigned:
`data` (`0x02`), record/Σ (`0x03`), `String` (`0x04`), `Bytes` (`0x05`),
`Array` (`0x06`), closure (`0x09`), bignum `Int` (`0x01`), big `Decimal`
(`0x0A`). **Kinds `0x07`/`0x08` (formerly `Map`/`Set` heap primitives) are
retired** under OQ-A: `Map`/`Set` are now proved `data` trees
(`../50-stdlib/52-map.md`) encoding as ordinary `data` (`0x02`); the tags are
held reserved (a later content-addressed fast-map, `52-map §6`, would reclaim
them).

**Determinism rules (the correctness bar):**

- **Records:** fields encode in **declaration order** (the order in the
  `record` definition), never alphabetical or insertion order.
- **`Map`:** entries sorted by the **lexicographic order of the canonical
  byte encoding of each key**. Duplicate keys resolved at construction time
  (last-write-wins).
- **`Set`:** elements sorted by the **lexicographic order of the canonical
  byte encoding of each element**.
- **`data`:** constructor identified by a **global elaborator-assigned id**
  (not a per-type de Bruijn index); arguments encode in positional order.
- **`String`:** **NFC-normalized** UTF-8. Normalization is performed at
  construction time; the normalized form is stored.
- **Bignums:** sign-magnitude, **minimal-limb** representation (no trailing
  zero limbs) — guarantees a unique encoding for every integer.
- **Closures:** encoded as `(code_id, captured environment)` where the
  captured environment is the **full canonical encoding of a record**
  (not a hash digest). Closure addressing is **memcmp-exact** — two
  closures with distinct captured environments are never conflated, so
  the "equal slot ⇒ structurally equal" invariant holds for closures
  the same as every other value kind. (The `env_hash` shortcut would
  break the kernel fast path, `§6` + `../../10-kernel/17 §3`).
- **`Array`:** elements in index order.

These rules guarantee that two structurally-equal values encode to identical
bytes regardless of construction history. In particular: a `Map` or `Set`
built in two different insertion orders encodes identically.

**Constructor and type identity.** The elaborator assigns globally-unique
integer identifiers to constructors (`data`) and record types. These travel
in the encoding so that two values of different types that happen to share
a field layout do not collide.

### 3b. Hashing and the intern algorithm (F4-elaborated)

**Hash function.** The hash of canonical bytes is **FNV-1a 64-bit** with
the FNV-specification constants:

```
offset_basis = 0xcbf29ce484222325
prime        = 0x100000001b3
```

This is non-cryptographic and fast. A cryptographic/Merkle hash (BLAKE3 is
the recommendation, selected at X2) is used for serialization/integrity
(`38 §1`) — a separate hash pipeline, two hashes for two jobs. **`OQ-hash`
DECIDED.**

**Collision resolution.** `memcmp` of the full canonical bytes resolves hash
collisions exactly. The canonical bytes are stored alongside the slot in the
store index so `memcmp` always compares against the canonical form.

**Intern algorithm.** Given canonical bytes `b[0..n)`:

1. Compute `h = FNV-1a(b)`.
2. Probe the index (`44 §1`) for `h`.
3. On hit (hash matches and `memcmp` confirms equal canonical bytes):
   return the existing slot id.
4. On miss: atomically increment the monotonic slot-id counter (`44 §2`),
   bump-allocate space in the arena, copy `b` and construct the in-memory
   value representation, and occupy the probed bucket.
5. On hash collision: continue probing per the index's open-addressing
   discipline (`44 §1`).

**Slot ids** are 64-bit, monotonic (increment-on-insert), starting at 1
(slot 0 is the null/invalid sentinel). The slot id is the permanent
identity of a distinct value for the process lifetime. Slot ids are never
reused (even if the slot is reclaimed — the id is retired).

The full index data structure, arena layout, and intern pseudocode are in
`docs/design/content-addressing.md §3`.

## 4. Structural equality is O(1) {#equality}

Because identical values share a slot, **structural equality of two heap values
is a slot-id comparison — O(1)** (after construction). This is the headline
runtime property of the content-addressed heap:

- `a == b` on heap values is `slot(a) == slot(b)`; on scalars it is the native
  comparison. No deep traversal at comparison time (the traversal happened once,
  at intern time).
- This O(1) structural equality is also a **conversion fast path** for the
  kernel (`../10-kernel/17 §3`): closed terms with equal content hashes are
  definitionally equal.
- It realizes "extensionality made physical" *as an optimization*, while the
  *propositional* equality that proofs use is `Eq` (`../10-kernel/15`, `16`) —
  the two agree on closed first-order data but the proof story does not depend
  on the heap.

## 5. Which values are content-addressed (`OQ-7` DECIDED)

**Decided (operator, 2026-06-27):** **scalars are immediate**;
**compound/identity-bearing values are content-addressed**, with the equality
story per case (slot-equality for interned, native for immediate). The principle
is fixed — cheap things stay immediate; shared/compared things are interned —
and the **exact small-aggregate boundary** (are tiny tuples interned? closures
by code+env hash?) is an **empirical X2 tuning**, not a semantic commitment.

**Concrete starting rule (F4-elaborated).** Foundation implements:

| Category | Values | Treatment |
|---|---|---|
| **Immediate scalars** | `Bool`, `Char`, `Float`/`Float32`, `Int8`–`Int64`, `UInt8`–`UInt64` | Stored inline; never reach the interner |
| **Small `Int`** | Within `i64` range | Inline `i64`; promotes to heap bignum on overflow (`§1`) |
| **Small `Decimal`** | Coefficient fits `i64`, exponent in `i32` range | Inline struct `{ i64 coeff, i32 exp }`; promotes to heap on overflow |
| **Interned compounds** | `data` applications, records, `String`, `Bytes`, `Array`, `Map`, `Set`, closures, bignums (overflowed `Int`), big `Decimal` | Content-addressed via the intern algorithm |

**Tiny-aggregate boundary (X2 tuning).** F4's baseline interns **all**
aggregates — including tiny records — for correctness-by-construction (identity
follows content). X2 may identify a small-aggregate cutoff (e.g. records ≤ 2
machine words with all scalar fields) that is immediate for performance. This is
a **semantics-preserving optimization**: it changes the equality cost model
(immediate aggregates compare by field traversal, not slot-id) but not the
equality *result*. Foundation implements the "intern-everything" baseline.

## 6. The `unknown` value

The runtime has a distinguished **`unknown`** value (the operational residue of
an open verification hole, `../20-verification/24 §2`):

- It is the third truth value at runtime and the "result not determined" marker;
  it **propagates** (Kleene/Heyting: `unknown ∧ false = false`, `unknown ∨ true
  = true`, else `unknown`; strict operators yield `unknown` on an `unknown`
  operand).
- `unknown` lets a **partially-verified program run** and surface *where* an
  unproven property actually affects a result, instead of failing closed
  (`../20-verification/21 §5`). A fully-verified program never produces
  `unknown` from holes (it has none).

## 7. Introspection (extensional-safe) (`OQ-witness` DECIDED)

**Decided (operator, 2026-06-27):** the runtime MAY expose **process-level**
statistics — slots used, dedup rate, arena bytes, a Merkle root — as a
first-class, **extensional-safe** facility (a `witness` surface primitive). It
**MUST NOT** expose **per-value identity or provenance** (which slot a value
occupies, allocation order), as that would break referential transparency. Stats
about the *store*, yes; identity of *values*, no.

**Concrete stat set (F4-elaborated).** The `witness` surface returns a
`StoreStats` record:

```
StoreStats {
    total_slots     : UInt64   // distinct values currently stored
    total_interns   : UInt64   // total intern() calls (includes hits)
    dedup_hits      : UInt64   // intern() calls that returned an existing slot
    arena_bytes     : UInt64   // total arena memory allocated
    index_buckets   : UInt64   // total index buckets across all partitions
    index_load      : Float    // fraction of occupied buckets (0.0–1.0)
    merkle_root     : Option<Bytes>  // root of the Merkle tree over all stored
                                     // values; populated only when Merkle is
                                     // enabled (off by default)
}
```

This is **extensional-safe**: it reports about the *store*, never about
individual values. A program's observable behavior must be identical regardless
of what `witness` reports. The exact `witness` API (function vs. language
primitive) is a surface-design detail for the language team; the stat-set shape
is fixed.

## 8. What WS-X must deliver here (X1/X2) and Foundation (K3)

The value model: typed scalar immediates (no uniform f64); the content-addressed
heap with FNV-1a+memcmp addressing, canonical encoding (per §3a), global dedup,
and **O(1) structural equality**; the immediate-vs-interned boundary (OQ-7, §5
table); the intern algorithm (§3b); the `unknown` value with propagation; and
extensional-safe process introspection (§7). Foundation implements the canonical
byte encoding, hashing, intern algorithm, and the `StoreStats` shape from the
elaborated design; K3 builds the production store on this contract.

Conformance:
- `../../conformance/runtime/values/` — dedup (equal values share a slot), O(1)
  equality, canonical-encoding determinism (Map/Set ordering), `Int`
  small→bignum promotion, and `unknown` propagation.
- `../../conformance/runtime/capacity/` — loud at-limit failure, dedup-aware
  accounting, reclamation page release, and no-lattice-on-hot-path.
