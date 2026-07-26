# The value model

> Status: **Elaborated (X2 contract)**. Normative for the value model,
> extensional equality, the callable boundary, and durable canonical encoding.
> In-process sharing, hashing, allocation, and identity are private runtime
> choices.

## 1. Scalars retain their declared types

A Ken value is **not** a uniform `f64` handle. Scalars are **typed values**,
dispatched by their static type (`../30-surface/35-numbers.md`).
The table gives permitted efficient representations, not observable storage:

| Ken type | Permitted fast path |
|---|---|
| `Int` (small) | machine word `i64` (fast path); promotes to a heap bignum when it outgrows the word (§2) |
| `Int64`/`UInt32`/… | the named machine integer |
| `Bool` | `i1` |
| `Char` | `u32` |
| `Float`/`Float32` | `f64`/`f32` |
| `Decimal` | a small struct (coefficient + exponent) |
| handle / heap reference | an opaque implementation-defined reference |

There is **no semantic decode-from-f64 stratum**; arithmetic and dispatch use
the declared type. A runtime MAY box a scalar or select another private
representation without changing its range, operations, equality, durable
encoding, or observable results. A `section`/handle crossing a boundary *may*
be shuttled by an implementation-defined wire convention, but that is a
transport detail, not the value model (`44`/`../30-surface/38`).

## 2. Runtime values and durable canonical data

Ken separates a value's semantics from its in-process representation.
Structural constructor applications (`data`), records (Σ), `String`, `Bytes`,
`Array`, and big integers are **durably canonicalizable** when every value
reachable from the root is closure-free and has a canonical encoding:

- Equal durably canonicalizable values have identical canonical bytes (§3a).
- Unequal values MUST NOT be merged merely because a private hash, pointer, or
  storage key collides.
- A runtime MAY copy, share, intern, deduplicate, or directly embed such
  values. No program can observe which policy was chosen, and no per-value
  slot, address, allocation order, or physical provenance exists in the Ken
  value model.
- Values are immutable. Updating a persistent structure returns a new value
  and leaves the old value unchanged; physical structural sharing is optional.
  Mutable state remains confined to `space` cells
  (`../30-surface/36 §4`).

The proved `Map`/`Set` package trees are closure-free and durably encodable as
ordinary `data`, but they are not durably canonicalizable in the sense above.
Their extensional equality observes ordered contents, while ordinary `data`
bytes preserve the transparent tree topology. Extensionally equal maps or sets
may therefore have different durable bytes (§3a).

An ordinary executable **closure is different**. It is a callable,
runtime-local opaque value. A closure, and any graph containing one, is outside
the durable canonicalization domain by construction. Such an aggregate may
exist as a runtime-local value, but a durable publication boundary rejects it
before canonical bytes, a content digest, or a storage identity can exist.

### 2.1 The callable boundary

The minimum observable contract for ordinary closures is:

- **Opaque and without equality.** A closure has no Ken-visible structural
  equality, `DecEq`, ordering, canonical hash, slot identity, or provenance.
  Closure equality is absent, not extensional. Generic structural `DecEq` for
  an aggregate is available only when every field supports it; an aggregate
  containing a closure does not acquire equality through a hidden runtime
  identity.
- **Transitively non-persistable.** A persistence, canonical-store,
  Merkle/serialization, or durable-export boundary MUST reject a closure or a
  graph containing one before publication. It MUST NOT replace the closure
  with a pointer, ordinal, digest, or process-local handle.
- **Live-domain invocation only.** Separately compiled artifacts may exchange
  an ordinary closure only within one live runtime domain, while the defining
  owner and artifact remain live. The receiver may invoke it at its checked
  callable type, but may not inspect, serialize, persist, reconstruct, or use
  it as stable identity. A raw code pointer or serialized local handle is not a
  valid cross-artifact value. A program cannot forge the callable; a
  wrong-domain, expired, or forged representation MUST refuse before
  invocation.
- **Static references are distinct and explicit.** An implementation that
  provides a stable serializable callable value MUST expose it as an explicit
  `StaticCallableRef`-class value. Its identity is qualified by package or
  artifact, callable unit or export, and ABI/signature, and it carries no
  dynamic captured environment. It may participate in content addressing only
  through those explicit static fields; this chapter assigns no encoding.
  Empty-capture optimization MUST NOT silently convert an ordinary closure into
  this value.
- **A durable closure would be a separate abstraction.** This specification
  defines no persistable closure. Any future `FrozenClosure`-class facility
  must be specified as a distinct explicit value rather than changing the
  meaning of ordinary `Closure`.

These constraints are mission-derived, not representation-derived:

| Retained constraint | Mission property that fails without it |
|---|---|
| Opacity and no closure equality | Code-plus-environment identity is not function equality. Exposing it would make an intensional implementation detail a false semantic claim, violating correctness and honesty about the boundary (`docs/PRINCIPLES.md` principles 4 and 8). |
| Transitive rejection before publication | A durable artifact containing a process-local callable cannot reproduce its meaning. Substitution or late failure would be silent degradation at a security boundary (principles 8 and 12). |
| Typed invocation only in a live runtime domain | Forged calls or calls after the defining owner is gone do not preserve the checked program's behavior or safety (principles 4 and 12). |
| Explicit separation of static callable references | If an optimization can add identity or serializability, observable behavior depends on the implementation strategy rather than the program, violating predictability (principle 10). |
| A separate future durable abstraction | Reusing ordinary closures would hide two different contracts behind one value and proliferate accidental mechanisms, violating subsumption and boundary honesty (principles 7 and 8). |

**Constraints removed by this revision.** Ken no longer requires ordinary
closures to be interned or deduplicated; assigns them no canonical kind tag or
`(code_id, captured environment)` byte encoding; gives them no `memcmp`/slot
equality, canonical or Merkle hash, persistence, or durable cross-artifact
identity; and requires no artifact binding for their local representation. An
implementation may use pointers, handles, hashing, memoization, or other local
machinery for dispatch, GC, or optimization only when it cannot affect
program-observable results.

This chapter fixes the observable validity boundary, not its implementation.
It requires no particular handle or trampoline, owner/lifetime encoding,
allocation scheme, GC strategy, or memoization scheme. It also does not require
that `StaticCallableRef` or a future durable higher-order abstraction exist.

## 3. Durable encoding and private in-process addressing

Durable canonical encoding and in-process addressing are separate contracts.
(A cryptographic/Merkle hash is used for
  *serialization/verification*, `../30-surface/38 §1` — a separate concern from
  in-process addressing).

`OQ-hash` is revised accordingly: this specification fixes no in-process hash,
collision strategy, probing policy, load factor, or identifier scheme. A
runtime MAY use FNV-1a, another hash, direct pointers, structural values, or no
index. Any private collision strategy MUST preserve extensional equality and
MUST NOT merge unequal values.

No Leech-lattice or Co₀ machinery is part of value semantics or durable
canonical encoding (`44 §4`). Optional library mathematics and private runtime
data structures remain separate from this contract.

### 3a. Canonical byte encoding (F4-elaborated)

Every durably canonicalizable value has a deterministic canonical byte form.
Closure-free `Map`/`Set` trees also have durable bytes, but those bytes are not
canonical for their extensional equality. This section is the normative
authority for both boundaries. The earlier implementation profile in
`../../docs/design/content-addressing.md §1` is derived; its byte-layout details
apply only where they agree with this chapter, and its interning, slot, and
storage language is superseded.

**Kind tags.** Each encoding is prefixed by a 1-byte kind tag from a single
namespace (see the design doc §1.1 for the full table). Currently assigned:
`data` (`0x02`), record/Σ (`0x03`), `String` (`0x04`), `Bytes` (`0x05`),
`Array` (`0x06`), bignum `Int` (`0x01`), big `Decimal` (`0x0A`).
**Kinds `0x07`/`0x08` (formerly `Map`/`Set` heap primitives) are
retired** under OQ-A: `Map`/`Set` are now proved `data` trees
(`../50-stdlib/52-map.md`) encoding as ordinary `data` (`0x02`); the tags are
held reserved (a later content-addressed fast-map, `52-map §6`, would reclaim
them).

**Determinism rules (the correctness bar):**

- **Records:** fields encode in **declaration order** (the order in the
  `record` definition), never alphabetical or insertion order.
- **`data`:** constructor identified by a **global elaborator-assigned id**
  (not a per-type de Bruijn index); arguments encode in positional order.
- **`String`:** **NFC-normalized** UTF-8. Normalization is performed at
  construction time; the normalized form is stored.
- **Bignums:** sign-magnitude, **minimal-limb** representation (no trailing
  zero limbs) — guarantees a unique encoding for every integer.
- **`Array`:** elements in index order.

These rules guarantee that two equal values in the durable canonicalization
domain encode to identical bytes regardless of construction history.

**`Map`/`Set` residual (OQ-A).** The transparent package carrier encodes through
the ordinary `data` rule above: constructor identity and positional children
preserve its tree topology. The encoding does not sort entries, add a
Map-specific discriminator, or define a separate extensional codec. Each
closure-free tree must round-trip through its own durable bytes, and extensional
equality plus ordered `to_list` observation remain unchanged. However, two
extensionally equal `Map`/`Set` values built with different insertion histories
may have different tree topologies and therefore different durable bytes.
Durable bytes are not a canonical form for their extensional equality, and
content-addressed deduplication of those values is not guaranteed. A later
proved fast-map may supersede this representation (`../50-stdlib/52-map.md
§6`); this chapter does not define it.

**Constructor and type identity.** The elaborator assigns globally-unique
integer identifiers to constructors (`data`) and record types. These travel
in the encoding so that two values of different types that happen to share
a field layout do not collide.

### 3b. Private addressing profile

An implementation MAY derive a private key from §3a's bytes and MAY use
bytewise comparison to resolve collisions. FNV-1a, linear probing, table
growth, arena allocation, slot numbering, and identifier retirement are
examples of private choices, not conformance requirements. The runtime design
may document a chosen profile, but programs and portable tests cannot observe
it.

## 4. Equality and the optional constant-time profile {#equality}

Comparable values use extensional equality: two values compare equal exactly
when their language-defined structures and primitive components are equal.
Comparable immediates may use native comparison. Ordinary closures have no
equality operation (§2.1).

The core language promises **no universal equality complexity bound**.
`O(1)` equality is instead an optional performance profile:

- A runtime advertising `constant-time-equality` MUST provide worst-case
  `O(1)` equality for every value kind listed by that profile.
- The profile does not prescribe interning, slot identity, hashing, or another
  implementation strategy.
- A runtime that does not advertise the profile may traverse values. Both
  runtimes MUST return the same equality result.

The kernel may use private hashes or sharing as a conversion fast path
(`../10-kernel/17 §3`) only after confirming equality; a collision or
representation difference cannot establish definitional equality.

## 5. Which values are durably canonicalizable (`OQ-7` DECIDED)

**Decided (operator, 2026-06-27; closure boundary revised 2026-07-26):**
**scalars may use immediate representations**, **closure-free canonical
compound data has durable canonical bytes**, and ordinary closures are
runtime-local opaque callables (§2.1). Equality is extensional for comparable
data, native for comparable immediates when chosen, and absent for closures.
The immediate/boxed/shared boundary is private runtime tuning, not semantics.

**Concrete starting rule (F4-elaborated).** Foundation implements:

| Category | Values | Treatment |
|---|---|---|
| **Immediate scalars** | `Bool`, `Char`, `Float`/`Float32`, `Int8`–`Int64`, `UInt8`–`UInt64` | May be stored inline |
| **Small `Int`** | Within `i64` range | May use inline `i64`; arithmetic still promotes without overflow (`§1`) |
| **Small `Decimal`** | Coefficient fits `i64`, exponent in `i32` range | May use an inline coefficient/exponent pair |
| **Canonical compounds** | Closure-free structural `data` applications, records, `String`, `Bytes`, `Array`, bignums, big `Decimal` | Deterministic canonical bytes under §3a; runtime representation private |
| **Durably encodable package trees** | Closure-free proved `Map`/`Set` trees | Ordinary topology-preserving `data` bytes round-trip; extensionally equal values may have different bytes (§3a) |
| **Runtime-local callable** | Ordinary `Closure`, and any aggregate graph containing one | Opaque and non-persistable; refused before durable bytes exist |

An implementation may tune the immediate/boxed/shared boundary without
changing equality, canonical bytes at a durable boundary, or any observable
result.

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

**Decided (operator, 2026-06-27; realization revised by
`SPEC-STORE-SPLIT`):** a runtime MAY expose aggregate, process- or
domain-level resource statistics as an extensional-safe `witness` facility.
It MUST NOT expose per-value identity, provenance, allocation order, or a
representation-dependent equality witness.

The stat set is profile-specific. Slot counts, deduplication rates, arena bytes,
index load, and Merkle roots are permitted diagnostics when the selected
runtime actually has them; none is a portable required field. A program's
semantic result must be independent of the reported statistics.

## 8. What WS-X must deliver here (X1/X2) and Foundation (K3)

The value model: typed scalars (no uniform f64); deterministic durable
canonical bytes for closure-free canonical data (§3a); extensional equality
with an optional constant-time performance profile (§4); the private
immediate/boxed/shared boundary (§5); the callable boundary and transitive
publication refusal (§2.1); the `unknown` value; and extensional-safe aggregate
introspection (§7). Foundation and Runtime may choose in-process storage,
hashing, sharing, and reclamation strategies privately.

Conformance:
- `../../conformance/runtime/values/` — extensional equality independent of
  representation, canonical-encoding determinism, closure opacity and
  transitive publication refusal, `Int` small→bignum promotion, and `unknown`
  propagation.
- `../../conformance/runtime/capacity/` — loud declared-limit failure,
  storage-policy independence, reclamation invisibility, space isolation, and
  large-value safety.
