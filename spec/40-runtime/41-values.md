# The value model

> Status: **DRAFT v0**. Normative for the model and equality; exact encodings
> are implementation latitude (flagged OQ where a real fork). Contract for WS-X
> **X1/X2**. Encodes the digest's two corrections: heterogeneous typed values
> (not uniform f64, §1) and conventional content addressing (FNV-1a + memcmp,
> not Leech, §3).

## 1. Scalars are typed immediates (the f64 correction at runtime)

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
integer- in-`f64` **wire convention** (the one grain of truth in the analysis),
but that is a transport detail, not the value model (`44`/`../30-surface/38`).

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

The reality-check's key correction: content addressing is **conventional**.

- The content key is a **fast non-cryptographic hash (FNV-1a-style)** of the
  canonical byte encoding, with **`memcmp`** to resolve hash collisions exactly.
  Slot ids are a **monotonic counter**. (A cryptographic/Merkle hash is used for
  *serialization/verification*, `../30-surface/38 §1` — a separate concern from
  in-process addressing; OQ-hash picks the exact functions.)
- **No Leech-lattice quantizer, no Co₀-orbit canonicalization on the allocation
  path.** The analysis's "heap addressing is Leech-lattice geometry" is refuted;
  Ken MUST NOT put lattice math on the hot path. (The lattice's *legitimate*,
  optional, separate roles are in `44 §lattice`.)
- **Canonical encoding.** Dedup requires a canonical byte form per value so that
  "same value ⇒ same bytes ⇒ same hash." The canonicalization rules (field
  order, normalization of which values are interned) are part of X2 and must be
  deterministic.

## 4. Structural equality is O(1)

Because identical values share a slot, **structural equality of two heap values
is a slot-id comparison — O(1)** (after construction). This is the headline
runtime property the prototype's heap provides and Ken keeps:

- `a == b` on heap values is `slot(a) == slot(b)`; on scalars it is the native
  comparison. No deep traversal at comparison time (the traversal happened once,
  at intern time).
- This O(1) structural equality is also a **conversion fast path** for the
  kernel (`../10-kernel/17 §3`): closed terms with equal content hashes are
  definitionally equal.
- It realizes "extensionality made physical" *as an optimization*, while the
  *propositional* equality that proofs use is `Path` (`../10-kernel/15`) — the
  two agree on closed first-order data but the proof story does not depend on
  the heap.

## 5. Which values are content-addressed (OQ-7)

The DRAFT: **scalars are immediate**; **compound/identity-bearing values are
content-addressed**. The exact boundary — e.g. are small tuples interned? are
closures interned by code+env hash? — is **OQ-7** (`../90-open-decisions.md`),
with the equality story stated per case (slot-equality for interned, native for
immediate). The principle is fixed: cheap things stay immediate; shared/compared
things are interned.

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

## 7. Introspection (extensional-safe)

The runtime MAY expose **process-level** statistics — slots used, dedup rate,
arena bytes, a Merkle root — as a first-class, **extensional-safe** facility
(the analysis's `witness`, addressing a real gap: the prototype has C-level
introspection but no surface primitive). It MUST NOT expose **per-value identity
or provenance** (which slot a value occupies, allocation order), as that would
break referential transparency. Stats about the *store*, yes; identity of
*values*, no.

## 8. What WS-X must deliver here (X1/X2)

The value model: typed scalar immediates (no uniform f64); the content-addressed
heap with FNV-1a+memcmp addressing, canonical encoding, global dedup, and **O(1)
structural equality**; the immediate-vs-interned boundary (OQ-7); the `unknown`
value with propagation; and extensional-safe process introspection. Conformance:
`../../conformance/runtime/values/` — dedup (equal values share a slot), O(1)
equality, `Int` small→bignum promotion, and `unknown` propagation.
