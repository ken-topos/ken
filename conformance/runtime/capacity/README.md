# Capacity conformance — elaborated corpus (F4)

Format: `../README.md`. These pin the store-capacity commitments:
loud refusal, dedup-aware accounting, reclamation, and the lattice
non-dependency.

## runtime/capacity/loud-refusal-not-silent
- spec: `spec/40-runtime/44-capacity.md §2`
- given: exhausting the store's capacity bound (either by filling the
  arena to a configurable soft cap, or by exhausting OS memory)
- expect: a **loud, clear failure** — a typed `CapacityExhausted` error
  with arena id, limit, and current count; NEVER silent drop, alias, or
  corruption
- why: loud refusal over silent degradation, decoupled from Leech
  numerology (OQ-5).

## runtime/capacity/dedup-aware-accounting
- spec: `spec/40-runtime/44-capacity.md §2`, `41 §2`
- given: intern the same value 1000 times, and intern 1000 distinct
  values
- expect: the first consumes **1 slot** (not 1000); the second consumes
  **1000 slots**. Capacity accounting tracks distinct values, not total
  `intern()` calls. The witness (`41 §7`) reports `total_slots`
  accurately.
- why: dedup means one slot per distinct value — capacity is in distinct
  values, not occurrences (OQ-5 accounting point).

## runtime/capacity/reclamation-releases-pages
- spec: `spec/40-runtime/44-capacity.md §3`
- given: a `space`-bounded arena populated with values; then `reset`
  the arena
- expect: the arena's pages are released to the OS (RSS drops); the
  index partition is cleared; retired slot ids are not reused (a new
  value gets a fresh, larger slot id); subsequent `intern()` calls
  allocate from the reset arena
- why: manual + region-scoped reclamation works — arena reset is
  observable via OS metrics and via fresh slot ids.

## runtime/capacity/reset-retires-slot-ids
- spec: `spec/40-runtime/44-capacity.md §3`, `41 §3b`
- given: an arena with slot ids up to N; reset the arena; intern a
  new value
- expect: the new value gets slot id > N (slot ids are monotonic and
  never reused, even after reset)
- why: retired slot ids are permanently retired — a slot id is a
  value's permanent identity for the process lifetime.

## runtime/capacity/space-bounded-reclamation
- spec: `spec/40-runtime/44-capacity.md §3`
- given: two `space`s each with their own arena; terminate one `space`
- expect: only the terminated space's arena is reclaimed; the other
  space's values remain live and reachable
- why: `space` boundaries scope reclamation — one space's reset does
  not affect another.

## runtime/capacity/no-lattice-on-hot-path
- spec: `spec/40-runtime/41-values.md §3`, `44 §4`
- given: the allocation/addressing path (intern a value)
- expect: addressing is **hash (FNV-1a) + `memcmp`**, slot ids are a
  monotonic counter; **no** Leech quantizer / Co₀ canonicalization /
  Golay encoding on the path; no `mmgroup` dependency in the build
  graph for the value-model crate
- why: the lattice is not load-bearing for addressing; Ken takes no
  `mmgroup` dependency on the value-model path (OQ-6).
- oracle: true

## runtime/capacity/auto-gc-not-present
- spec: `spec/40-runtime/44-capacity.md §3`
- given: a long-running process that creates and discards values over
  time, without explicit `reset` calls
- expect: memory grows monotonically until `reset` is called (no
  automatic GC); values remain reachable at stable slot ids until
  their arena is explicitly reclaimed
- why: automatic GC is deferred (OQ-gc); the current model is manual +
  region-scoped. This test documents the F4 baseline — it should be
  updated when auto-GC is added.

## runtime/capacity/index-resize-preserves-slots
- spec: `spec/40-runtime/44-capacity.md §1a`
- given: intern enough distinct values to trigger index resize
  (exceed initial capacity × 0.7 load factor)
- expect: all previously-interned values remain reachable at their
  original slot ids; subsequent intern hits (re-intern an existing
  value) still return the original slot id; no slot id reassignment
- why: index resize is a transparent implementation detail — slot ids
  are stable across resize.

## runtime/capacity/arena-page-chaining
- spec: `spec/40-runtime/44-capacity.md §1b`
- given: intern values until one 4 MiB arena page fills; intern more
  values
- expect: a new page is allocated and chained; values span pages; the
  intern algorithm handles cross-page allocation transparently
- why: the arena chain scales beyond a single page.
