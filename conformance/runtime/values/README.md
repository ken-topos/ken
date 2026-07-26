# Values conformance — elaborated corpus (F4)

Format: `../README.md`. These pin the value model at implementation resolution:
durable canonical bytes and extensional equality for closure-free canonical
data, private in-process representation, the runtime-local opaque callable
boundary, `Int` promotion, and `unknown` propagation.

The proved `Map`/`Set` package trees are the explicit durable-byte exception:
their ordinary topology-preserving `data` bytes round-trip, but are not
canonical for extensional equality (`41 §3a`, OQ-A). No case requires
content-addressed deduplication across extensionally equal Map/Set values.

**Realization status.** This revision is a spec/conformance boundary, not its
runtime implementation. The current `ken-foundation` and `ken-runtime`
`values.rs`/`canonical.rs` still encode `Value::Closure`, and
`ken-runtime/src/store.rs` still carries
`closure_content_addressed`/`closure_distinct_env_no_collision`; the interpreter
still assigns closure slots. Those controls assert the retired contract and are
not retained anchors. The callable/aggregate/publication cases below are
explicitly RED-UNTIL the implementation follow-on removes that route.

## runtime/values/equal-canonical-values-same-durable-bytes
- spec: `spec/40-runtime/41-values.md §2,§3a,§4`
- given: two independently-constructed structurally-equal **closure-free
  canonical** compound values (same record type, same field values, constructed
  separately)
- expect: they compare equal and produce identical durable canonical bytes.
  The case does not assert whether either value is copied, shared, or interned.
- why: durable canonicity and extensional equality survive independently of
  runtime storage.

## runtime/values/extensional-equality-independent-of-representation
- spec: `spec/40-runtime/41-values.md §4`
- given: equal closure-free canonical values constructed through both shared
  and independent evaluation paths, plus one structurally different value
- expect: the equal pair compares true and the different value compares false.
  The result is invariant under runtime representation. Complexity is tested
  only by an implementation advertising `constant-time-equality`.
- why: equality is extensional; a pointer, slot, or construction path cannot
  change it.

## runtime/values/map-extensional-observation-and-durable-roundtrip
- spec: `spec/40-runtime/41-values.md §3a,§4`;
  `spec/50-stdlib/52-map.md §1.1,§5.3`
- given: a `Map` constructed by inserting entries `{k2→v2, k1→v1, k3→v3}`
  and the same `Map` constructed by inserting `{k1→v1, k2→v2, k3→v3}`
- expect: the maps compare equal extensionally and have identical ordered
  `to_list` observations. Each independently round-trips through its own
  topology-preserving ordinary-`data` bytes. The case requires neither equal
  nor unequal byte strings across the two insertion histories.
- why: OQ-A preserves extensional Map behavior and durable round-trip while
  trading away insertion-order-independent byte canonicity.

## runtime/values/set-extensional-observation-and-durable-roundtrip
- spec: `spec/40-runtime/41-values.md §3a,§4`;
  `spec/50-stdlib/52-map.md §1.1,§4.4`
- given: a `Set` built by inserting `{c, a, b}` and the same `Set` built
  by inserting `{a, b, c}`
- expect: the sets compare equal extensionally and have identical ordered
  element observations. Each independently round-trips through its own
  topology-preserving ordinary-`data` bytes. The case requires neither equal
  nor unequal byte strings across the two insertion histories.
- why: `Set a = Map a Unit` inherits OQ-A's extensional behavior and its
  non-canonical durable-byte cost.

## runtime/values/canonical-encoding-record-field-order
- spec: `spec/40-runtime/41-values.md §3a`
- given: a record `{x=1, y="hello"}` and a value constructed with fields
  in a different order (e.g. named-field syntax `{y="hello", x=1}`)
- expect: both encode to identical bytes (field order is declaration
  order, not construction order) and compare equal
- why: records encode fields in declaration order — deterministic.

## runtime/values/scalars-retain-distinct-types
- spec: `spec/40-runtime/41-values.md §1`
- given: an `Int`, a `Bool`, a `Float`
- expect: each retains its declared type and behavior and is not routed through
  a uniform `f64`; boxing or immediacy is not observed
- why: the "every value is an f64" model is not Ken's.

## runtime/values/int-small-to-bignum
- spec: `spec/40-runtime/41-values.md §1`, `35 §1`
- given: an `Int` computation that grows past a machine word
- expect: the value stays exact and preserves its durable canonical bytes
  across the representation boundary; no physical promotion form is observed
- why: arbitrary-precision `Int` permits a private small-integer fast path.

## runtime/values/runtime-representation-does-not-change-value
- spec: `spec/40-runtime/41-values.md §5`
- given: a `Bool`, an `Int64`, a `Float`, a small `Int` (within i64),
  a small `Decimal`, and a closure-free `String`, record, `Array`, and bignum
  (overflowed `Int`), plus an ordinary closure
- expect: every closure-free value retains its type, extensional equality, and,
  where durably published, canonical bytes regardless of runtime
  representation. The closure is **runtime-local and opaque**, with no slot
  identity or canonical encoding.
- why: OQ-7 makes value semantics independent of private representation while
  preserving the stronger closure boundary.

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
  produce identical canonical bytes and compare equal.
- why: sign-magnitude minimal-limb guarantees unique bignum encoding.

## runtime/values/unknown-propagates
- spec: `spec/40-runtime/41-values.md §6`, `42 §4`
- given: a value depending on an open verification hole, combined via
  `∧`/`∨`
- expect: `unknown ∧ false = false`, `unknown ∨ true = true`, else
  `unknown`; the program **runs**
- why: partial verification runs and marks where the gap bites.

## runtime/values/canonical-kind-separation-no-false-merge
- spec: `spec/40-runtime/41-values.md §3a` (kind tags)
- given: a `String` `"42"` and a `Bytes` `[0x34, 0x32]` (the ASCII
  encoding of `"42"`) — different kinds, same raw byte content
- expect: their durable canonical bytes differ in the 1-byte kind tag and no
  equality/canonicalization path treats them as the same typed value
- why: kind tags prevent cross-kind false merge without prescribing storage.
