# The content store: capacity, reclamation, and the lattice

> Status: **Elaborated (F4)**. Normative for the *principles* (loud refusal,
> dedup accounting, no-lattice-on-the-hot-path) and now the *concrete store
> design* (index data structure, arena layout, reclamation mechanics).
> **`OQ-5`/`OQ-6`/`OQ-gc` DECIDED** (operator, 2026-06-27):
> **engineering-chosen** capacity (wide handles → no practical ceiling) with
> **loud refusal**; the **lattice machinery out of the core** (optional
> research packages only); **reclamation a semantics-invisible implementation
> detail** (manual+region now, automatic GC addable later without touching
> surface or semantics — deferred). Contract for WS-X **X2/X4** and Foundation's
> **K3**. Its central stance: the Leech-lattice numbers are *aesthetic*, not
> load-bearing.

## 1. The store

The content-addressed heap (`41 §2`) is the process's value store: an
append-mostly arena of interned values, addressed by hash+memcmp (`41 §3`), with
a process-wide `(root, hash) → ref` index giving global dedup and auto-chaining
across arenas.

### 1a. Store index design (F4-elaborated)

The store index is an **open-addressing hash table** keyed on the 64-bit
FNV-1a hash, with linear probing. Each bucket carries:

```
bucket {
    hash      : UInt64       // the full 64-bit FNV-1a hash
    slot_id   : UInt64       // monotonic slot id (0 = empty/tombstone)
    canon_len : UInt32       // byte length of canonical encoding
    canon_ptr : Pointer       // into the arena (canonical bytes)
}
```

**Design rationale.** Open-addressing with linear probing suits the
append-mostly workload (inserts dominate; deletes are batch/region-scoped).
A full bucket has `slot_id ≠ 0`. A tombstone (deleted bucket) is marked
`slot_id = 0, hash = 0`; probes skip it but the bucket remains occupied
so linear probing continues past it.

**Resize.** When load factor exceeds a threshold (recommend 70%), the index
is migrated to a doubled table. Migration is lock-free for readers (RCU-style:
a new table is allocated, buckets are copied, and a single atomic pointer
swing publishes the new table; readers see either the old or new index, both
valid). The resize threshold is an X2 tuning constant.

**Index partitioning.** The `arena_root` parameter in the index key enables
partitioning by arena (e.g. per-`space` arenas). The process may have multiple
index partitions, each covering one arena chain. Arena-local partitioning
means a `space` reset (§3) only needs to clear its own index partition,
not the whole process index.

### 1b. Arena page layout (F4-elaborated)

The heap is an **append-mostly arena** organized as a chain of fixed-size
pages. Recommended page size: **4 MiB** (matches huge-page alignment; good
TLB behaviour; large enough to amortize allocation overhead, small enough
that reclaiming a page is cheap).

```
page {
    page_id   : UInt32        // monotonic page counter
    used      : AtomicUInt32  // bytes consumed (bump-pointer advances this)
    data      : [Byte; PAGE_SIZE]  // the actual storage
}
```

Allocations within a page are **bump-pointer**: atomically advance `used`
by the allocation size, store the canonical bytes first (for `memcmp`) and
then the in-memory value representation, return pointers. When a page fills,
a new page is allocated and linked to the chain.

The arena root (a pointer to the head page + the page chain) is the
address-space identifier for the index (§1a's `arena_root`).

## 2. Capacity is an engineering choice, not numerology (OQ-5)

A capacity scheme tied to the Leech lattice would fix **196,560 slots/heap** —
the kissing number of Λ₂₄ — and 256 heaps/chain (an 8-bit heap id + 24-bit
slot), giving ~50.3M distinct contents and ~17–18 GB/process. But the **196,560
is aesthetic**: a 24-bit slot field holds ~16M; nothing in addressing needs the
Leech kissing number.

**Ken's stance:** choose any capacity bound on **engineering grounds** (encoding
width, arena sizing, target scale), *not* lattice numerology.

- **`OQ-5` DECIDED:** keep the **"loud refusal over silent degradation"**
  philosophy — at a limit, fail loudly with a clear error; never silently drop,
  alias, or corrupt — with **no practical ceiling** (wide handles: 64-bit slot
  ids → billions+, sized to the hardware). The 64-bit width is fixed by F4
  (`41 §3b`); the *stance* — deliberate, loud, no Leech ceiling (strategy
  G5-perf) — is permanent.
- Dedup means real consumption is **one slot per distinct value**, not per
  occurrence — capacity is in *distinct* values, an important accounting point
  for any bound chosen.
- **Loud-refusal mechanics (F4-elaborated).** When a limit is reached (arena
  `mmap` fails, index resize fails, or a configurable soft cap is hit), the
  runtime produces a **typed error** — `CapacityExhausted { arena: ArenaId,
  limit: UInt64, current: UInt64 }` — catchable at a `space` boundary. It MUST
  NOT silently drop values, alias slots, or corrupt the index. The exact
  soft-cap configuration mechanism (environment variable / CLI flag /
  `space`-level parameter) is an X2 convenience detail — the loud-refusal
  principle is fixed.

## 3. Reclamation

- **No automatic GC / compaction** in F4: the append-mostly store keeps
  slots stable for fast ids and O(1) equality.
- **Manual reclamation exists** — `clear`/`reset`/`strip`-style operations
  release an arena's pages (e.g. `madvise(MADV_DONTNEED)`); a `space`/region
  boundary (`../30-surface/36 §4`) bounds a working set's lifetime.
- **Automatic reclamation is a deferred implementation detail (`OQ-gc`
  DECIDED).** Start **manual + region-scoped** (suits the immutable, dedup'd
  model). Adding automatic refcount/GC later is a **well-demonstrated benefit at
  modest cost** and — crucially — **invisible to the language surface and
  semantics** (values are immutable and identity is content; reclaiming an
  unreachable slot changes nothing observable). So it collapses to an
  implementation choice the runtime may adopt when working sets demand it,
  **without a language fork**.

- **Reclamation mechanics (F4-elaborated).** Foundation implements:
  1. **Arena-level `reset`:** given an arena, iterate its page chain and
     release each page (`munmap` for dedicated mappings; `madvise(MADV_DONTNEED)`
     for carve-outs from a reserved region), clear the index partition, and
     retire all slot ids in that arena (ids are not reused).
  2. **`space`-bounded working sets:** each `space` (`../30-surface/36 §4`)
     owns zero or more arenas. When a `space` terminates, its arenas are
     reclaimed. A `space` that processes a bounded unit of work (e.g. one
     request) can `reset` its arena at the end of each unit, bounding memory.
  3. **No compaction, no moving GC:** slots remain at stable addresses for
     their lifetime. Fragmentation is an X2 concern (the 4 MiB pages and
     append-mostly allocation keep fragmentation manageable at F4 scale).
  The interface is designed so that automatic GC can be added later without
  changing the language surface or semantics (`OQ-gc`).

## 4. The lattice machinery — three separate optional roles (OQ-6)

The Leech/Golay/Co₀ math is **not** the allocator and decomposes into **three
distinct, code-separate** roles. If Ken includes any of it, it is scoped to
these and **never** the hot path:

| Role | What it is | Where it'd live |
|---|---|---|
| **Golay(24,12,8) error-correction** | a `VoyagerList`-style EC-coded sequence | an optional stdlib data type |
| **Kissing-number bitmap (XSet)** | the 196,560-bit set as a fixed-size set/MPHF domain | an optional fixed-domain `Set` impl (`../30-surface/37 §2`) |
| **Co₀/M24 orbit canonicalization** | canonicalizing representatives under symmetry | an optional, separate utility |

- **None is required** for the core language; all are opt-in libraries
  (`../50-`).
- The claim of specific Co₀ orbit cardinalities (98280/8386560/8292375) is
  **not** carried as fact; if such a facility is built, those are to be
  computed/verified, not assumed.
- **`OQ-6` DECIDED:** the lattice machinery is **not in the core** — available
  only as research/optional packages (strategy WS-R), **never on the allocation
  hot path**. Capacity rests on engineering choices, never on lattice
  aesthetics.

## 5. Scale and limits validation (X4)

When the native backend lands (X3), **X4** validates scale: the chosen capacity
bound holds under load, dedup behaves, reclamation reclaims, and any boundary is
**loud**. The interpreter + native backend are differential-tested (`42 §5`) so
the value model is identical across them; capacity behaviour is documented, not
discovered in production.

## 6. What WS-X must deliver here (X2, X4) and Foundation (K3)

The content store with conventional addressing + global dedup (X2); a
**deliberate, loud** capacity bound chosen on engineering grounds (OQ-5), with
dedup-aware accounting; manual/region reclamation (§3 mechanics); and (X4)
scale/limits validation. Foundation implements the store index (§1a), arena
layout (§1b), loud-refusal error path (§2), and reclamation mechanics (§3) from
the elaborated design; K3 builds the production store on this contract. The
lattice machinery, if any, is **optional and off the hot path** (OQ-6).

Conformance + scale: `../../conformance/runtime/capacity/` — dedup
accounting, a loud at-limit failure (not silent), reclamation releasing
pages, and no-lattice-on-hot-path (oracle tag).
