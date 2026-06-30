# X2 capacity conformance — seed cases

Format: `../../README.md`. These pin the **X2 store hardening** (`44`): the
**loud** capacity bound (never silent), **dedup-aware accounting** counted in
*distinct* values, and **manual + region reclamation** mapped to the `space`
boundary — all over the **landed** per-`space` store (`crates/ken-runtime/
src/store.rs`; K3 shipped the substrate, X2 hardens it). Each case is grounded
on the elaborated `spec/40-runtime/44-capacity.md §1–§6` and drives **real**
values through the **actual** `Space`/`Store` API.

## Reading disciplines (how these cases are pitched)

- **Observable = a real consequence, not "it didn't crash" or a synthetic
  flag.** AC2 asserts a `CapacityExhausted` value is *raised* by a real
  over-limit `intern`; AC3 asserts a real `StoreStats.arena_bytes` *drop* to 0
  after a real `reset`. A case that merely checks the program survived, or reads
  a counter never wired to the store, guards nothing (QA gate, `44 §6`).
- **The over-limit witness sits off the dedup fixed point (the L1 off-grid-
  witness carry, new domain).** A repeated value takes the dedup `Hit` path and
  **returns before the limit check is ever reached** (`probe_or_insert`: the
  occupied-bucket `memcmp` hit returns first). So an over-limit witness that
  repeats an existing value is *green-vs-green* under both correct code and the
  silent-drop bug. The AC2 witness is therefore a **distinct** value (a `New`),
  and a dedicated partner case (`at-limit-repeat-does-not-trip`) pins that the
  distinctness is load-bearing — exactly the §6 design-time lock-point.
- **Discriminating pairs over one store, two states.** AC2 flips `New` →
  `CapacityExhausted` as a distinct value crosses the bound; AC4 keeps the
  reset space-local (B untouched while A is cleared); AC5 flips `new_id >
  old_id` (retired, never resurrected) against a recycled id. Each pair would go
  green-vs-red under the exact bug it targets, never green-vs-green.
- **Reconciled to the landed shapes, not the F4 prose.** The error is
  `CapacityExhausted { limit, current }` — **no `arena`/`ArenaId` field** (a
  future addition once spaces are named, `44 §2`). The reclamation observable is
  `StoreStats.arena_bytes → 0`, **not RSS** (allocator-dependent, `44 §3`).
  Resize is **single-writer** rehash, not lock-free RCU (staged, `44 §1a`). The
  `tombstone` field is reserved-but-never-set (no per-value free in X2).
- **Accounting invariant carve-out.** `44 §2`'s `total_interns = total_slots +
  dedup_hits` holds **in the absence of `CapacityExhausted` attempts**: the
  landed `intern` increments `total_interns` on *every* call including a refused
  one (`41 §7`: "total intern() calls"), while a refusal yields neither a slot
  nor a hit. AC1 asserts the invariant only in the no-exhaustion regime; the
  accounting in AC2's exhaustion context is asserted on `total_slots`/
  `distinct_count` alone. (Raised as a one-line spec carve-out for the merge
  Decision — flag, not a blocker; conformance follows the landed code.)
- **Tags.** `(soundness)` — a load-bearing **loud-never-silent / no-aliasing**
  safety commitment (`OQ-5`) that must never regress (silent drop, slot-id
  resurrection, or false-merge would corrupt). `(oracle)` — confirm against the
  reference interpreter / a build-time structural check once available; until
  then ground on `44` + the landed `store.rs`.

---

## runtime/capacity/dedup-accounting-distinct-not-occurrences
- spec: `spec/40-runtime/44-capacity.md §2`, `41 §7` (AC1)
- given: into one `Space`, `intern` a **single** distinct compound value 1000
  times; into a second `Space`, `intern` 1000 **pairwise-distinct** compounds.
- expect: first space — `stats.total_slots == 1 == distinct_count()`,
  `total_interns == 1000`, `dedup_hits == 999`. Second space —
  `total_slots == 1000 == distinct_count()`, `dedup_hits == 0`. Capacity is
  counted in **distinct** values (one slot per `New`), **not** occurrences.
- why: a bound of *K* admits *K* distinct contents regardless of occurrence
  count; the same-value run is the discriminator (`total_slots` flips 1 vs 1000
  against a count-occurrences bug). The invariant `total_interns = total_slots +
  dedup_hits` holds here (`1000 = 1 + 999`, `1000 = 1000 + 0`) — no exhaustion.

## runtime/capacity/loud-at-limit-raises-not-silent (soundness)
- spec: `spec/40-runtime/44-capacity.md §2`, `43 §2` (AC2 — the headline)
- given: a `Space::with_capacity_limit(3)`; `intern` 3 **distinct** compounds
  (each a `New`); then `intern` a **4th, distinct** compound (a would-be `New`).
- expect: the 4th `intern` **returns** `CapacityExhausted { limit: 3, current:
  3 }` — a **raised**, catchable typed result. The store **MUST NOT** map it to
  `NULL_SLOT`, alias a slot, or corrupt the index: `distinct_count()` stays 3,
  the 3 prior slot ids are unchanged, `total_slots` does not advance past 3. The
  intern result sum `New | Hit | CapacityExhausted` has **no drop arm**.
- why: `OQ-5` loud-refusal-never-silent. **Discriminator:** the error is
  *raised* (right) vs the **silent `CapacityExhausted → NULL_SLOT` drop** (wrong
  — the current `ken-interp` intern shim). Asserting "did not crash" is
  green-vs-green against the silent-drop path; asserting the raised error is the
  flip. **Witness is distinct** (a `New`) — see the partner below.

## runtime/capacity/at-limit-repeat-does-not-trip (soundness)
- spec: `spec/40-runtime/44-capacity.md §2,§6` (AC2 fixed-point partner)
- given: the same `Space` exactly at its limit (`total_slots == 3 == limit`);
  now **re-intern an existing** value (a repeat of one already stored).
- expect: returns `Hit(existing_slot)` — **not** `CapacityExhausted`.
  `total_slots` stays 3. The dedup `Hit` returns before the limit check.
- why: pins that the AC2 witness **must** be distinct: a repeat never reaches
  the limit check, so a same-value over-limit witness cannot discriminate the
  silent-drop bug (green-vs-green). Under a bug that checks the limit *before*
  dedup, this repeat would wrongly trip `CapacityExhausted` → the verdict flips
  `Hit` vs `CapacityExhausted`. (The L1 off-grid-witness discipline in the
  capacity domain; `44 §6` design-time lock-point.)

## runtime/capacity/reclamation-releases-pages
- spec: `spec/40-runtime/44-capacity.md §3` (AC3)
- given: a `Space`; `intern` enough distinct compounds that
  `stats.arena_bytes > 0` (e.g. 100 distinct strings); then call `reset()`.
- expect: `stats.arena_bytes == 0` (the page buffers are dropped — real
  release) **and** `stats.total_slots == 0`. **Not** a no-op (`arena_bytes`
  unchanged). Observed via `StoreStats.arena_bytes`, **not RSS** (which is
  allocator-dependent).
- why: manual reclamation drops the append-mostly page chain; the normative
  observable is `arena_bytes → 0` (the F4 `madvise`/`munmap` is a future
  `mmap`-backed mechanism, not the X2 contract). Verdict flips 0 vs >0 against a
  no-op `reset`.

## runtime/capacity/space-reset-is-isolated
- spec: `spec/40-runtime/44-capacity.md §3`, `36 §4.4` (AC4 — isolation half)
- given: two `Space`s A and B; `intern` `v_a` into A and `v_b` into B; then
  `A.reset()`.
- expect: A is cleared — `A.distinct_count() == 0`, `A.stats().arena_bytes ==
  0`. **B is untouched** — `B.distinct_count() == 1`, `B.stats().arena_bytes >
  0`, and re-interning `v_b` into B still returns `Hit`. One space's reset never
  reaches another's arena or index.
- why: the `space` boundary is the reclamation boundary; spaces are
  shared-nothing (`36 §4.4`). Flips B-untouched (count 1) vs B-collateral-damage
  (count 0) under a process-wide reset bug.

## runtime/capacity/escape-survives-sender-reset
- spec: `spec/40-runtime/44-capacity.md §3` (AC4 — escape-survival half)
- given: a value `v` that **escapes** space A to space B. Shared-nothing value
  passing realizes escape as **re-interning `v` into the recipient (surviving)
  space B** (`44 §3`); model it as `intern(v)` into A (→ `id_a`) **and** into B
  (→ B-local `id_b`). Then `A.reset()`.
- expect: A loses `v` (`A.distinct_count() == 0`). B **retains** `v`: re-
  interning `v` into B returns `Hit(id_b)` and `B.stats().arena_bytes > 0`. The
  escaped value survives in the surviving arena. (Escape preserves **content**
  identity via re-intern; `id_b` is B's own slot id — slot ids are process-
  global, each `New` a fresh id — not the sender's `id_a`.)
- why: AC4 — escaping values survive the sender's reclamation because they live
  in a surviving store; non-escaping values are reclaimed (AC4 isolation).
  The automatic interning-on-escape **wiring** is the X2 *build* deliverable
  (the interpreter runs a single store today); the **observable contract**
  (escaped survives, non-escaping reclaimed) is normative here and is pinned
  against the landed `Space` primitives via an explicit recipient re-intern.

## runtime/capacity/reset-retires-ids-never-resurrected (soundness)
- spec: `spec/40-runtime/44-capacity.md §3`, `41 §3b` (AC5)
- given: `intern` `v1` into a space (→ `old_id`); `reset()`; then `intern` a new
  `v2` into the space (a `New`).
- expect: `new_id > old_id` **strictly** — the retired id is **never** reused
  (the process-global monotonic counter is untouched by `reset`). A **live**
  value sitting in a *surviving* space keeps its slot id and O(1) equality
  across another space's reset (no renumbering, no aliasing).
- why: reclaiming unreachable slots is observationally invisible
  (content-identity-preserving). **Discriminator:** `new_id > old_id` (right)
  vs a recycled/reset-to-`old_id` counter (wrong — a resurrected id would alias
  a live value, a silent corruption). (soundness)

## runtime/capacity/no-lattice-on-hot-path (oracle)
- spec: `spec/40-runtime/44-capacity.md §4`, `41 §3` (OQ-6)
- given: the `intern`/addressing build path (the store crate's dependency
  graph and the `probe_or_insert` body).
- expect: addressing is **FNV-1a hash + `memcmp` + a monotonic `u64`
  counter** — **no** Leech quantizer, Co₀/M24 canonicalization, or Golay
  encoding on the path, and **no `mmgroup` (or Leech/Co₀) dependency** in the
  store crate's build graph.
- why: `OQ-6` — the lattice machinery is optional and never on the allocation
  hot path; capacity rests on engineering choice, not numerology. A build-time /
  oracle structural check (dependency-absence + path inspection).
- oracle: true

## runtime/capacity/index-resize-preserves-slot-ids
- spec: `spec/40-runtime/44-capacity.md §1a` (the resize transparency contract)
- given: `intern` enough distinct values to cross the **0.70** load factor on
  the **2¹⁶** initial table, triggering the single-writer **double + rehash**.
- expect: **every** previously-interned value still dedups to its **original**
  slot id after the resize (re-interning it returns `Hit(original_id)`); **no**
  slot-id reassignment; arena locators preserved.
- why: index resize is a transparent implementation detail — slot ids are stable
  across structural growth (`44 §1a`). The rehash is **single-writer** (`&mut`),
  not the F4 lock-free RCU migration (a future hardening, out of X2 scope).
  Flips original-id-preserved vs reassigned-after-resize.

## runtime/capacity/arena-spans-pages-oversized-safe
- spec: `spec/40-runtime/44-capacity.md §1b` (page chain + oversized handling)
- given: `intern` enough values to overflow one **4 MiB** page (chaining) and
  one **oversized** value `> PAGE_SIZE` (placed in its own dedicated page,
  followed by a fresh tail page); then `intern` a normal value after the
  oversized one.
- expect: all values reachable and dedup to their original slot ids across
  pages; `stats.arena_bytes > PAGE_SIZE`; the oversized value does **not**
  corrupt the bump target — the normal value interned after it gets a fresh slot
  and both still dedup correctly.
- why: the append-mostly page chain scales beyond one page; oversized handling
  preserves the "tail page is the bump target" invariant (`44 §1b`) — the
  `remaining()` underflow this case guards against.

## runtime/capacity/no-automatic-gc (OQ-gc baseline)
- spec: `spec/40-runtime/44-capacity.md §3` (OQ-gc — automatic GC deferred)
- given: a `Space`; create and discard many distinct values **without** any
  `reset` call; an early value `v0` is interned, then many more values follow.
- expect: `arena_bytes` is **non-decreasing** across the interns (nothing is
  reclaimed in the background), and `v0` is **still reachable** — re-interning
  `v0` returns `Hit` at its **original** slot id (it was not silently freed).
- why: `OQ-gc` — automatic GC is deferred; the model is manual + region-scoped,
  values stay at stable ids until an explicit `reset`. Documents the baseline:
  a build that silently added background reclamation would lose `v0` (re-intern
  → a new id) — the case flips against exactly that. **Update when auto-GC
  lands** (the contract then changes).

---

## Coverage map (AC → cases)

- **AC1** (dedup — distinct, not occurrences):
  `dedup-accounting-distinct-not-occurrences`.
- **AC2** (loud at-limit — the headline): `loud-at-limit-raises-not-silent`
  paired with the fixed-point partner `at-limit-repeat-does-not-trip`.
- **AC3** (manual reclamation): `reclamation-releases-pages`.
- **AC4** (region-scoped lifetime): `space-reset-is-isolated` (isolation half)
  + `escape-survives-sender-reset` (escape-survival half).
- **AC5** (reclamation invisible to identity):
  `reset-retires-ids-never-resurrected`.
- **OQ-6** (no lattice on hot path): `no-lattice-on-hot-path` (oracle).
- **§1a / §1b / §3 mechanics**: `index-resize-preserves-slot-ids`,
  `arena-spans-pages-oversized-safe`, `no-automatic-gc`.

## Cross-case sweep (consistency over shared metatheory)

- **The loud-never-silent class agrees** (`OQ-5`): every limit/refusal site
  surfaces a raised, catchable result and never a silent drop/alias —
  `loud-at-limit-raises-not-silent` (soft cap → `CapacityExhausted`),
  `at-limit-repeat-does-not-trip` (dedup short-circuits *before* the cap, so a
  repeat is a `Hit`, not a refusal), and the closed `New | Hit |
  CapacityExhausted` sum (no drop arm). No case admits a silent outcome.
- **The dedup-accounting class agrees**: `total_slots == distinct_count()`
  counts **distinct** values everywhere — under dedup (`dedup-accounting`),
  across a resize (`index-resize-preserves-slot-ids`), and across pages
  (`arena-spans-pages-oversized-safe`). Occurrence count never drives occupancy.
- **The identity-stability class agrees**: slot ids are process-global,
  monotonic, and never reused — preserved across resize (`index-resize`),
  retired-not-recycled across reset (`reset-retires-ids`), unaffected in a
  surviving space by another's reset (`escape-survives-sender-reset`,
  `space-reset-is-isolated`). No case renumbers or resurrects an id.
- **Accounting invariant scope** (the carve-out, above): the
  `total_interns = total_slots + dedup_hits` identity is asserted only where no
  `CapacityExhausted` occurred (`dedup-accounting`); AC2's exhaustion context
  asserts occupancy on `total_slots`/`distinct_count` alone, since the landed
  `intern` counts a refused call in `total_interns`.

## Subsumed siblings (one home per property)

This file is the **canonical X2 home** for the capacity properties (`44`). The
predecessors are retired here to avoid two homes per property:

- `runtime/capacity/README.md` (the F4 elaborated corpus) is replaced by a
  pointer to this file. Its `loud-refusal-not-silent` case carried two details
  the X2 landed-code reconcile **corrected**: the error has **no `arena` id**
  (`{ limit, current }` only), and reclamation is observed via **`arena_bytes`,
  not RSS** — both fixed in the cases above.
- `runtime/seed-runtime.md`'s two capacity cases (`runtime/capacity/loud-
  refusal`, `runtime/addressing/no-lattice-on-hot-path`) are subsumed by
  `loud-at-limit-raises-not-silent` and `no-lattice-on-hot-path` here; that file
  now points to this seed for capacity.
- Dedup **correctness** (the `memcmp` no-false-merge guard, distinct-env
  closures) is pinned in `runtime/values/` and is **not** duplicated here; AC1
  above pins only the dedup **accounting** (distinct-count occupancy).

## Build-sequencing note

K3 (Foundation) shipped the substrate — arena, index, intern, dedup, slot ids,
`StoreStats` (`41`); X2 **hardens** it to this contract and builds **no** new
addressing. Two halves of AC4 differ in landed-ness: the **isolation** half
(`space-reset-is-isolated`) runs against the landed per-`space` `reset` today;
the **escape-survival** half (`escape-survives-sender-reset`) pins the normative
contract via an explicit recipient-side re-intern, since the automatic
**interning-on-escape wiring** (the interpreter runs a single store now) is the
X2 *build* deliverable. X4 stresses the bound under load (`44 §5`).
