# Values conformance — elaborated corpus (F4)

Format: `../README.md`. These pin the value model at implementation resolution:
content addressing and O(1) equality for closure-free canonical data, the
runtime-local opaque callable boundary, canonical-encoding determinism, the
immediate-vs-interned-vs-local boundary, `Int` promotion, and `unknown`
propagation.

**Realization status.** This revision is a spec/conformance boundary, not its
runtime implementation. The current `ken-foundation` and `ken-runtime`
`values.rs`/`canonical.rs` still encode `Value::Closure`, and
`ken-runtime/src/store.rs` still carries
`closure_content_addressed`/`closure_distinct_env_no_collision`; the interpreter
still assigns closure slots. Those controls assert the retired contract and are
not retained anchors. The callable/aggregate/publication cases below are
explicitly RED-UNTIL the implementation follow-on removes that route.

## runtime/values/dedup-shares-slot
- spec: `spec/40-runtime/41-values.md §2,§3b,§4`
- given: two independently-constructed structurally-equal **closure-free
  canonical** compound values (same record type, same field values, constructed
  separately)
- expect: they occupy the **same slot** (global dedup); `==` is O(1)
  (slot-id comparison, not structural traversal)
- why: content addressing gives closure-free canonical data sharing + O(1)
  structural equality.

## runtime/values/equality-is-slot-id
- spec: `spec/40-runtime/41-values.md §4`
- given: a closure-free canonical compound value `v` with two references `a`
  and `b` (both bound to `v`)
- expect: `a == b` is true and the comparison is a single integer
  comparison (slot id compare), verified by measuring comparison cost
  as independent of value depth (e.g. a deeply-nested record vs. a
  shallow one — both are slot-id compares)
- why: the headline property — deep traversal happened once, at intern
  time.

## runtime/values/canonical-encoding-map-ordering
- spec: `spec/40-runtime/41-values.md §3a`
- given: a `Map` constructed by inserting entries `{k2→v2, k1→v1, k3→v3}`
  and the same `Map` constructed by inserting `{k1→v1, k2→v2, k3→v3}`
- expect: both produce **identical canonical bytes** (entries sorted
  lexicographically by canonical key bytes) and thus **share a slot**
- why: canonical ordering makes Map encoding deterministic.

## runtime/values/canonical-encoding-set-ordering
- spec: `spec/40-runtime/41-values.md §3a`
- given: a `Set` built by inserting `{c, a, b}` and the same `Set` built
  by inserting `{a, b, c}`
- expect: both produce identical canonical bytes (elements sorted
  lexicographically by canonical element bytes) and share a slot
- why: canonical ordering makes Set encoding deterministic.

## runtime/values/canonical-encoding-record-field-order
- spec: `spec/40-runtime/41-values.md §3a`
- given: a record `{x=1, y="hello"}` and a value constructed with fields
  in a different order (e.g. named-field syntax `{y="hello", x=1}`)
- expect: both encode to identical bytes (field order is declaration
  order, not construction order) and share a slot
- why: records encode fields in declaration order — deterministic.

## runtime/values/scalars-are-typed-immediates
- spec: `spec/40-runtime/41-values.md §1`
- given: an `Int`, a `Bool`, a `Float`
- expect: each is an **unboxed typed immediate** (machine word / `Bool` /
  `f64`), not routed through a uniform `f64` nor a heap slot
- why: the "every value is an f64" model is not Ken's.

## runtime/values/int-small-to-bignum
- spec: `spec/40-runtime/41-values.md §1`, `35 §1`
- given: an `Int` computation that grows past a machine word
- expect: transparent promotion to a heap bignum; value stays exact;
  the promoted bignum is content-addressed (equal large ints share a
  slot)
- why: arbitrary-precision `Int` with a small-int fast path.

## runtime/values/immediate-vs-interned-boundary
- spec: `spec/40-runtime/41-values.md §5`
- given: a `Bool`, an `Int64`, a `Float`, a small `Int` (within i64),
  a small `Decimal`, and a closure-free `String`, record, `Array`, and bignum
  (overflowed `Int`), plus an ordinary closure
- expect: the scalars are **immediate** (no slot id, no heap allocation);
  closure-free canonical compounds are **content-addressed** (each has a slot
  id; equal values share a slot); the closure is **runtime-local and opaque**,
  with no slot id or canonical encoding
- why: the OQ-7 boundary is concrete — cheap scalars are immediate, canonical
  data is interned, and ordinary callables stay outside both identities.

## runtime/values/closure-callable-observed-by-application (oracle)
- spec: `spec/40-runtime/41-values.md §2.1`, `42 §3.5`
- given: two ordinary closures that each capture `n = 2` and compute
  `\x. x + n`, constructed independently, then each applied to `40`
- expect: both applications produce the closure-free ground observation `42`;
  the case performs **no** closure equality, ordering, hash, provenance, or
  slot comparison
- why: callability is observable; code-plus-environment identity is not
  function equality. Higher-order agreement is selected application to a
  closure-free observation, never comparison of closure representation.

## runtime/values/closure-containing-aggregate-has-no-deceq (oracle)
- spec: `spec/40-runtime/41-values.md §2.1`
- given: derive/use structural `DecEq` for (a) a record containing only `Int`
  fields and (b) the same outer record shape with one field of type
  `Int -> Int`
- expect: (a) accepts and compares structurally; (b) is rejected because the
  callable field has no `DecEq`. It MUST NOT compare a pointer, slot, code id,
  captured environment, or other hidden runtime identity.
- status: **RED-UNTIL runtime value implementation is reconciled to `41 §2.1`**
- why: closure equality is absent, not extensional. An aggregate cannot acquire
  equality from an intensional implementation identity.

## runtime/values/closure-publication-rejected-transitively (oracle)
- spec: `spec/40-runtime/41-values.md §2.1`, `36 §4.4`, `44 §3`
- given: attempt canonical-store, serialization, durable-export, and
  cross-space message publication of (a) a closure-free canonical record and
  (b) an ordinary closure directly and nested as a record field, data
  constructor argument, array element, and map value
- expect: (a) succeeds; every (b) attempt rejects **before** canonical bytes,
  hash, slot, provenance, or publication are produced. Rejection is transitive
  and MUST NOT substitute a pointer, ordinal, digest, or local handle.
- status: **RED-UNTIL runtime publication is reconciled to `41 §2.1`**
- why: the positive control proves that the boundary, rather than the carrier
  shape, causes refusal; checking every reachable edge prevents a nested
  callable from silently becoming durable.

## runtime/values/empty-capture-closure-is-not-static-reference (oracle)
- spec: `spec/40-runtime/41-values.md §2.1`
- given: an ordinary empty-capture closure `\x. x + 1`, and, only if the
  implementation provides one, an explicit capture-free static callable
  reference qualified by package/artifact, export/callable unit, and
  ABI/signature
- expect: the ordinary closure remains runtime-local and publication rejects
  it exactly as above; an optimization MUST NOT silently promote it. The
  explicit static reference may be serialized only when that separate optional
  facility exists; this case pins no type spelling, layout, or encoding.
- status: **RED-UNTIL empty-capture publication follows `41 §2.1`**
- why: optimization cannot add equality, identity, or persistence that the
  program did not explicitly select.

## runtime/values/live-domain-closure-invocation-if-supported (oracle)
- spec: `spec/40-runtime/41-values.md §2.1`
- given: only if cross-artifact ordinary-closure exchange is provided, pass a
  typed closure to another artifact in the same live runtime domain; pair its
  valid call with wrong-domain, expired-owner, and forged representations
- expect: the valid call may run and yields only its ground result; each invalid
  representation refuses before invocation. None can be inspected, serialized,
  persisted, reconstructed, or used as stable identity. If the implementation
  provides no such exchange facility, this case is not applicable.
- why: the spec permits live-domain invocation under observable safety
  conditions; it does not require a handle/trampoline representation or require
  the optional facility to exist.

## runtime/values/bignum-minimal-limb-encoding
- spec: `spec/40-runtime/41-values.md §3a`
- given: a bignum `0` and a bignum representing `2^64`
- expect: `0` encodes as `sign=0, limbs=[0]` (minimal — one limb, not
  zero limbs); `2^64` encodes as `sign=0, limbs=[0, 1]` (two limbs, no
  trailing zeros). Two separate constructions of the same large integer
  produce identical canonical bytes and share a slot.
- why: sign-magnitude minimal-limb guarantees unique bignum encoding.

## runtime/values/unknown-propagates
- spec: `spec/40-runtime/41-values.md §6`, `42 §4`
- given: a value depending on an open verification hole, combined via
  `∧`/`∨`
- expect: `unknown ∧ false = false`, `unknown ∨ true = true`, else
  `unknown`; the program **runs**
- why: partial verification runs and marks where the gap bites.

## runtime/values/dedup-across-kinds
- spec: `spec/40-runtime/41-values.md §3a` (kind tags)
- given: a `String` `"42"` and a `Bytes` `[0x34, 0x32]` (the ASCII
  encoding of `"42"`) — different kinds, same raw byte content
- expect: they occupy **different** slots (the 1-byte kind tag
  disambiguates them)
- why: kind tags prevent cross-kind collisions.
