# Runtime resource conformance — seed cases

Format: `../../README.md`. These cases pin the observable resource contract in
`spec/40-runtime/44-capacity.md`: declared limits fail loudly, reclamation is
semantics-invisible, logical `space` isolation holds, and storage mechanisms
remain private.

They do not require global interning, same-slot results, FNV-1a, a probing
policy, a load factor, page or arena geometry, slot retirement, or a collector
policy. A runtime may expose optional aggregate diagnostics through a declared
profile, but no case observes a per-value runtime identity.

## Reading disciplines

- **Exercise a declared profile.** A resource-limit case first declares the
  resource and limit it is testing. It does not infer a capacity unit from an
  implementation's current store.
- **Observe values and typed outcomes.** Tests compare extensional values,
  durable canonical bytes where publication is requested, and named errors.
  Allocation counters are admissible only when a profile declares them.
- **No hidden representation oracle.** A pointer, slot, table bucket, page,
  arena, allocation order, or collector schedule is not a Ken observation.
- **Keep the closure boundary stronger.** Making storage private does not grant
  ordinary closures slot identity, canonical bytes, equality, or persistence
  (`41 §2.1`).

---

## runtime/capacity/declared-resource-profile-accounting
- spec: `spec/40-runtime/44-capacity.md §2,§2a` (supporting)
- given: a runtime profile declaring a finite resource unit and an optional
  aggregate usage counter; run two workloads whose declared resource usage is
  known to differ
- expect: the counter follows the profile's declared unit and both workloads
  stay within the declared bound. The case does not assume that occurrences,
  distinct values, slots, or bytes are the unit unless the profile says so.
- why: resource accounting must be testable against the profile that promises
  it; the spec does not choose a universal storage metric.

## runtime/capacity/loud-at-declared-limit-raises-not-silent (soundness)
- spec: `spec/40-runtime/44-capacity.md §2`, `43 §2` (AC1)
- given: a runtime profile with a declared finite limit; consume exactly the
  limit, then perform one operation whose declared resource cost exceeds it
- expect: the operation returns or raises the profile's typed
  `CapacityExhausted` outcome. All earlier values retain their extensional
  contents. The runtime does not silently drop, fabricate, merge, alias, or
  corrupt a value.
- why: `OQ-5` retains loud refusal independently of the representation or
  accounting unit. A silent success/drop and the typed refusal are the
  discriminating pair.

## runtime/capacity/storage-policy-independent-observations
- spec: `spec/40-runtime/44-capacity.md §1` (AC2)
- given: run the same closure-free value and equality corpus through one
  configuration that copies equal values and another that shares or interns
  them
- expect: both configurations produce the same extensional values, equality
  results, and durable canonical bytes. Neither exposes a representation
  identity.
- why: copying and sharing are private. A result that changes with the storage
  policy would make an implementation choice semantic.

## runtime/capacity/reclamation-is-semantics-invisible
- spec: `spec/40-runtime/44-capacity.md §3` (AC3)
- given: compute a closure-free canonical value, retain one live reference,
  discard unrelated values, and allow or request reclamation
- expect: the live value's extensional observation and durable canonical bytes
  are unchanged. No stale reference aliases a different value.
- why: reclamation may release any private storage, but it cannot change value
  semantics. The case deliberately does not require pages to be dropped or an
  aggregate byte counter to reach zero.

## runtime/capacity/space-reset-is-isolated
- spec: `spec/40-runtime/44-capacity.md §3`, `36 §4.3–§4.4` (AC4)
- given: two logical spaces A and B with non-aliased mutable cells; reset or
  terminate A, then observe B through its existing typed operations
- expect: A's local mutable state is unavailable after its lifetime ends; B's
  state and values are unchanged. No operation in A gains mutable authority
  over B.
- why: the logical `space` isolation and no-shared-mutable-authority contract
  survives; per-space indexes, arenas, and reset mechanics are private.

## runtime/capacity/escape-survives-sender-reset
- spec: `spec/40-runtime/44-capacity.md §3`, `36 §4.4` (AC4)
- given: publish a closure-free immutable value from space A to space B through
  the message boundary, then reset or terminate A
- expect: B retains the same extensional value and, if it republishes the value,
  produces the same durable canonical bytes. A closure-containing value is
  refused before publication and is not a positive witness for this case.
- why: recipient-visible value lifetime is independent of sender storage.
  Copying, sharing, or re-interning on transfer is not prescribed.

## runtime/capacity/reset-never-aliases-live-values (soundness)
- spec: `spec/40-runtime/44-capacity.md §3`, `41 §4` (supporting)
- given: keep a value live in space B, reset space A, then create a different
  value in A
- expect: the new value does not compare equal to, overwrite, or otherwise
  alias B's live value. B's value remains unchanged.
- why: reset cannot resurrect a stale observable identity or cause a false
  merge. A monotonic slot counter is one possible implementation, not the
  contract.

## runtime/capacity/no-semantic-lattice-dependency (oracle)
- spec: `spec/40-runtime/44-capacity.md §4`, `41 §3b` (OQ-6)
- given: evaluate and durably encode representative closure-free values in a
  build with no Leech/Co₀/Golay runtime facility available
- expect: evaluation, extensional equality, and durable encoding succeed with
  the same observations. No semantic rule or required runtime dependency
  invokes lattice machinery.
- why: lattice machinery is optional and cannot be a semantic dependency. The
  case does not substitute a required FNV/memcmp path for the retired lattice
  path.
- oracle: true

## runtime/capacity/store-growth-preserves-values
- spec: `spec/40-runtime/44-capacity.md §1,§3`
- given: retain early closure-free values while creating enough later values to
  force whatever growth path the implementation uses
- expect: every retained value preserves its extensional observation and
  durable canonical bytes; no value is lost or falsely merged
- why: growth is representation-private. The case does not require a table,
  initial size, load factor, rehash, locator, or stable slot id.

## runtime/capacity/large-values-remain-safe
- spec: `spec/40-runtime/44-capacity.md §1,§2`
- given: construct and retain a value larger than the runtime's ordinary small
  allocation class, followed by a normal value
- expect: both values remain reachable and extensionally correct, or the large
  construction fails through a declared loud resource outcome. Neither value
  corrupts the other.
- why: large-value safety survives without prescribing page size, dedicated
  pages, bump allocation, or a tail-page invariant.

---

## Retired producers

- `runtime/capacity/at-limit-repeat-does-not-trip` is **retired**. Whether a
  repeat consumes capacity depends on the declared resource profile and private
  representation; loud refusal is covered by
  `loud-at-declared-limit-raises-not-silent`.
- `runtime/capacity/no-automatic-gc` is **retired**. Collector scheduling and
  the choice to collect are private; `reclamation-is-semantics-invisible`
  retains the observable safety property.

## Coverage map

- **AC1:** `loud-at-declared-limit-raises-not-silent`.
- **AC2:** `storage-policy-independent-observations`.
- **AC3:** `reclamation-is-semantics-invisible`.
- **AC4:** `space-reset-is-isolated` +
  `escape-survives-sender-reset`.
- **AC5:** `large-values-remain-safe`.
- **Supporting profile/safety cases:** `declared-resource-profile-accounting` +
  `reset-never-aliases-live-values`.
- **OQ-6:** `no-semantic-lattice-dependency`.
- **Private growth mechanism:** `store-growth-preserves-values`.

## Cross-case consistency sweep

- Every limit case uses the resource and accounting unit declared by its
  profile; no case silently promotes a current storage statistic into universal
  semantics.
- Every lifetime case compares extensional values and durable bytes, never
  pointer, slot, page, arena, allocation-order, or collector identity.
- Loud refusal, no false merge, no stale aliasing, escape survival, and
  no-shared-mutable-authority remain positive obligations.
- Ordinary closures remain outside equality, canonicalization, slot identity,
  and publication even if a runtime privately uses slots for other values.

## Subsumed siblings

This file remains the canonical home for runtime resource behavior. The
capacity references in `runtime/seed-runtime.md` point here. Value equality and
durable canonical-byte determinism live in `runtime/values/`; closure
publication refusal lives there and is not duplicated as a resource case.

## Build-sequencing note

The landed runtime may currently expose stores, slots, arenas, indexes, or
statistics. Those are implementation facts, not this conformance contract. A
later runtime WP may reuse, replace, or remove them provided these observable
cases remain true and any advertised performance/resource profile is met.
