# Values conformance — elaborated corpus (F4)

Format: `../README.md`. These pin the content-addressed value model at
implementation resolution: dedup, O(1) equality, canonical-encoding
determinism, the immediate-vs-interned boundary, `Int` promotion, and
`unknown` propagation.

## runtime/values/dedup-shares-slot
- spec: `spec/40-runtime/41-values.md §2,§3b,§4`
- given: two independently-constructed structurally-equal compound values
  (same record type, same field values, constructed separately)
- expect: they occupy the **same slot** (global dedup); `==` is O(1)
  (slot-id comparison, not structural traversal)
- why: content addressing gives sharing + O(1) structural equality.

## runtime/values/equality-is-slot-id
- spec: `spec/40-runtime/41-values.md §4`
- given: a compound value `v` with two references `a` and `b` (both
  bound to `v`)
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
  a small `Decimal`, and a `String`, a record, an `Array`, a bignum
  (overflowed `Int`)
- expect: the scalars are **immediate** (no slot id, no heap allocation);
  the compounds are **content-addressed** (each has a slot id, equal
  values share a slot)
- why: the OQ-7 boundary is concrete — cheap things immediate,
  shared/compared things interned.

## runtime/values/closure-content-addressed
- spec: `spec/40-runtime/41-values.md §3a`
- given: two closures with the same code and equal captured environments
  (same captured values, in the same capture order), defined at different
  points
- expect: both closures share a slot (the interner `memcmp`s the full
  canonical encoding of the captured environment, not a hash digest)
- why: closures are content-addressed by `(code_id, full captured env
  record)` — memcmp-exact, not hash-only.

## runtime/values/closure-distinct-env-no-collision
- spec: `spec/40-runtime/41-values.md §3a`, design doc §1.9
- given: two closures with the same code but **different** captured
  environments (e.g. one captures `x=1, y="a"`, the other captures
  `x=1, y="b"`)
- expect: the closures occupy **different** slots (the memcmp of the
  canonical encoding detects the distinct environments exactly)
- why: closure addressing is memcmp-exact — the `env_hash` shortcut
  (which would collide probabilistically) is NOT used; "equal slot
  ⇒ structurally equal" holds for closures as for all value kinds.

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
