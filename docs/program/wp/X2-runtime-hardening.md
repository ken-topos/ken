# WP X2 — runtime hardening (capacity, dedup accounting, reclamation)

**Owner:** Team Runtime. **Branch:** `wp/X2-runtime-hardening` (cut from
`origin/main`). **Stream / gate:** WS-X → **G6** (feeds X4 → G5-perf).
**Depends on:** K3 (content-addressed value model) — **merged**; X1
(interpreter)
— **merged**. **Spec source:** `spec/40-runtime/44-capacity.md` (+ `41` value
model, `30-surface/36 §4` space/region boundary).

> **Steward *frame*** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails; the spec enclave elaborates `44` to team-ready rigor +
> conformance before Team Runtime builds. **Perishable:** X2 *hardens the
> already-
> landed K3/X1 store* — **verify the current store/arena/dedup-index state in
> the
> code at pickup** (K2c-series-2 stale-frame trap), don't rebuild what K3
> shipped.

## 1. Objective (one line)

Harden the runtime content store: a **deliberate, loud capacity bound** (never
silent degradation), **dedup-aware accounting** (capacity in *distinct* values),
and **manual + region-scoped reclamation** — building on the landed K3 store, no
automatic GC, no lattice machinery on the hot path.

## 2. Settled inputs — FIXED, do not reopen

Decided in `44` (`OQ-5`, `OQ-6`, `OQ-gc` — all DECIDED):

1. **Loud refusal over silent degradation (`OQ-5`).** At a capacity limit, fail
   **loudly** with a clear error — **never silently drop, alias, or corrupt** a
   value. **No practical ceiling:** wide handles (a 48-/64-bit slot field for
   billions+, sized to hardware). The exact width is an **X2 constant** (an
   engineering choice — encoding width, arena sizing, target scale); the
   *stance*
   is permanent. **NOT** the Leech kissing number (196,560) or any lattice
   numerology.
2. **Dedup accounting.** The store is hash+memcmp-addressed with a process-wide
   `(root, hash) → ref` index → real consumption is **one slot per *distinct*
   value**, not per occurrence. Capacity is accounted in **distinct** values.
3. **Reclamation = manual + region-scoped; NO automatic GC (`OQ-gc`).** The
   append-mostly store keeps slots **stable** for fast ids + O(1) equality, so
   no
   GC/compaction in scope. **Manual reclamation:** `clear`/`reset`/`strip`-style
   ops release an arena's pages (e.g. `madvise(MADV_DONTNEED)`); a
   `space`/region
   boundary (`36 §4`) bounds a working set's lifetime. Automatic refcount/GC is
   a
   **deferred** runtime detail — **invisible to the language surface +
   semantics**
   (values immutable, identity = content; reclaiming an unreachable slot changes
   nothing observable) → addable later **without a language fork**. Do not build
   it.
4. **No lattice machinery on the hot path (`OQ-6`).** The Leech/Golay/Co₀ math
   is
   **not** the allocator; if Ken ever includes any, it is opt-in stdlib/research
   packages, **never** the allocation path. X2 builds **none** of it.

## 3. Mandated deliverable outline (each item ends in an implementable choice)

Deliver in the runtime crate (`ken-interp` / the K3 store home — verify at
pickup) + spec `44`:

1. **The hardened content store.** Conventional addressing (hash+memcmp) +
   **global dedup** via the `(root, hash) → ref` index, on the landed K3 arena.
   Pin what X2 *adds* vs. what K3 already shipped (extend, don't rebuild).
2. **The loud capacity bound.** Pin the **slot-field width** (the X2 constant —
   choose 48 or 64 bit on engineering grounds, state the rationale) and the
   **loud at-limit error** (a `CapacityExhausted`-style typed error, the F4
   pattern) on the **exact** path that would otherwise drop/alias — **no silent
   degradation anywhere**. Enumerate the limit sites; each fails loud.
3. **Dedup-aware accounting.** Expose capacity/occupancy counted in **distinct**
   values (a slot is consumed once per distinct content, regardless of
   occurrence
   count). Pin the accounting API + invariant.
4. **Manual + region reclamation.** `clear`/`reset`/`strip` releasing arena
   pages
   (`madvise(MADV_DONTNEED)`); the `space`/region boundary (`36 §4`) bounding a
   working set. Pin that reclamation is **content-identity-preserving** (it
   changes nothing observable — immutable, content-addressed) and that live slot
   ids/equality are unaffected.

## 4. Testable acceptance criteria

- **AC1 (dedup)** N occurrences of the **same** value consume **one** slot —
  assert `slots_used == distinct_count`, not `== occurrence_count` (a structural
  value assertion; the F4/K3 dedup hardened at runtime scale).
- **AC2 (loud at-limit — the headline)** At the capacity bound, the store
  returns
  a **clear `CapacityExhausted`-style error** — and **never** silently drops,
  aliases, or corrupts. *Discriminating:* the over-limit insert must **fail
  loud**
  (right) vs. the silent-degradation bug (wrong) — assert the error is raised,
  not
  that "it didn't crash" (green-vs-green otherwise).
- **AC3 (manual reclamation)** A `clear`/`reset` on an arena/region **releases
  its
  pages** (observably frees memory — e.g. RSS drops / the arena reports
  reclaimed)
  — not a no-op.
- **AC4 (region-scoped lifetime)** A `space`/region boundary bounds a working
  set's lifetime — values created within it are reclaimed at the boundary, while
  values escaping it survive.
- **AC5 (reclamation is invisible to identity)** After manual reclamation of
  unreachable slots, **live** values' ids + O(1) equality are **unchanged** (no
  observable semantic effect — content-identity preserved).
- **Conformance:** `conformance/runtime/capacity/` — dedup accounting, the loud
  at-limit failure (NOT silent), reclamation releasing pages. **QA gate:** drive
  **real** values through the **actual** store/arena and observe the real
  consequence (a real over-limit insert → real error; a real `clear` → real page
  release), never a synthetic counter assertion.

## 5. Do-not-reopen guardrails

- **Loud, never silent** (`OQ-5`) — every limit site fails loud; no drop/alias/
  corrupt fallback. This is the permanent stance.
- **No automatic GC** in X2 (`OQ-gc`) — manual + region only; auto-GC is
  deferred
  and surface-invisible, **do not build it**.
- **No lattice machinery** on the hot path (`OQ-6`) — X2 builds none; capacity
  rests on engineering choices, never Leech aesthetics.
- **Slot ids stay stable** (append-mostly) — fast ids + O(1) equality depend on
  it; reclamation releases pages but does not renumber live slots.
- **Extend the landed K3 store, don't rebuild it** (perishable-frame caveat).

## 6. Sequencing notes

- X2 is in the **breadth wave** (brings Team Runtime online). **X4**
  (scale/limits
  validation, needs X2 + X3) is the downstream follow-on — X2 delivers the
  *mechanism*, X4 *validates it under load*; keep the loud-bound + dedup
  accounting **X4-testable** (a stated bound X4 can stress).
- The capacity bound's width + the loud error couple loosely to **F4**'s
  content-addressing design (`CapacityExhausted`, slot-id equality) — reuse the
  F4 pattern, don't re-derive.
- Standard §2c: frame → spec-leader elaborates `44` + conformance → merge
  (Architect + conformance-validator) → Team Runtime compacted, then kicked off.
