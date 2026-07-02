# The content store: capacity and reclamation

> Status: **impl-ready (X2)**. Normative for the *principles* (loud refusal,
> dedup accounting, no-lattice-on-the-hot-path) and the *concrete store design*
> (per-`space` index, arena layout, capacity bound, accounting, reclamation
> mechanics) at team-ready resolution. **`OQ-5`/`OQ-6`/`OQ-gc` DECIDED**
> (operator, 2026-06-27): **engineering-chosen** capacity (wide handles → no
> practical ceiling) with **loud refusal**; the **lattice machinery out of the
> core** (optional research packages only); **reclamation a semantics-invisible
> implementation detail** (manual+region now, automatic GC addable later without
> touching surface or semantics — deferred). Contract for WS-X **X2/X4** and
> Foundation's **K3**. Central stance: capacity and reclamation are
> **engineering** choices for a **systems-adjacent** language (§3); the
> Leech-lattice numbers are *aesthetic*, not load-bearing, and out of the core
> (§4).
>
> **X2 grounding (perishable-frame, K2c-s2 rule).** This elaboration is pinned
> against the **landed** K3 store on `main` (`ken-runtime/src/store.rs`,
> `canonical.rs`, `hash.rs`), not the F4 prose. Where the F4 design and the
> landed code diverge, the **landed code is normative** and the divergence is
> flagged inline (the store is realized **per `space`** with a bare-hash index,
> not a process-wide `(root, hash)` index; reclamation drops page buffers, not
> `madvise`; the index resize is single-writer, not lock-free). One cross-file
> reconcile for `41 §2`/`§3b` is flagged in §1 (not changed here).

## 1. The store

The content-addressed heap (`41 §2`) is the value store. The landed store is
realized **per `space`**: the unit is a **`Space`** owning **one append-mostly
arena** of interned canonical bytes (§1b), **one dedup index** (§1a), an
**accounting** triple (§2), and an optional **soft capacity limit** (§2). The
process default is a single-`space` `Store` (a thin wrapper over one `Space`).

- **Each surface `space` (`../30-surface/36 §4`) is realized at runtime as a
  store `Space`** owning its own arena + index. This is the runtime realization
  that `36 §4.4` explicitly **defers to `40-runtime/`** — so `44` is the
  authority that maps the *semantic* `space` boundary to *physical* arena
  ownership and reclamation (§3). The `space` boundary is therefore the
  **reclamation boundary**.
- **Dedup is per-`space`.** Within a space, identical content ⇒ the same slot ⇒
  **O(1) structural equality** (`41 §4`). **Identity (slot ids) is
  process-global**: a single monotonic counter allocates slot ids across *all*
  spaces, so ids are unique process-wide and **never reused** (even after a
  reset retires them, §3). So *within* a space, equality is a slot-id compare;
  *across* spaces (**shared-nothing**, `36 §4.4`) values are passed by
  re-interning, never by raw slot-id sharing (§3).
- **Addressing is conventional** (`41 §3`): the content key is the **64-bit
  FNV-1a hash** of the canonical byte encoding (`41 §3a`/`§3b`), with
  **`memcmp`** of the full canonical bytes resolving collisions exactly. Slot
  ids are a **monotonic 64-bit counter** (§2). No lattice geometry here (§4).

**Reconcile (perishable-frame).** The F4 prose — and `41 §2`/`§3b` — describe a
*process-wide `(root, hash) → ref` index* giving global dedup "across arenas."
The **landed store realizes partitioning as separate per-`space` indexes keyed
on the bare FNV-1a hash** (no compound `(root, hash)` key, no cross-arena
auto-chaining). "Process-wide global dedup" holds for the **single-`space`
default** (one space = the whole process); under **multi-`space` isolation**
dedup is **per-`space`**. **Flag for `41`:** `41 §2`'s "`(root, hash) → ref`"
and `41 §3b` step 2's "probe the index for `h`" wording should be reconciled to
"a per-`space` index keyed on the FNV-1a hash; partitioning is realized as
separate per-`space` indexes, not a compound key." Raised as a cross-file
erratum to `41` — **not changed on this branch** (scope: `41` is a separately
merged K3 contract).

**What X2 adds vs. what K3 shipped.** K3 shipped the working store — arena,
index, intern algorithm, dedup, slot ids, `StoreStats` (`41`). X2 **hardens**
that landed store; it builds **no** new addressing. X2 pins: the **loud capacity
bound** and **dedup-aware accounting contract** (§2), and the **manual +
region reclamation** mapped to the `space` boundary (§3) — *extend, don't
rebuild* (the landed `Space`/`Arena`/`Index` are the substrate).

### 1a. Store index design

The store index is an **open-addressing hash table** with **linear probing**,
keyed on the **64-bit FNV-1a hash** (`41 §3b`). Capacity is a **power of two**
(probe mask `idx = hash & (cap − 1)`); the landed initial capacity is **2¹⁶
buckets**. Each bucket carries:

```
bucket {
    hash       : UInt64     // the full 64-bit FNV-1a hash
    slot_id    : UInt64     // monotonic slot id (0 = empty / null sentinel)
    page_idx   : UInt        // arena page holding the canonical bytes
    offset     : UInt        // byte offset within that page
    canon_len  : UInt32      // byte length of the canonical encoding
    tombstone  : Bool        // reserved (see below) — never set in X2
}
```

**The canonical bytes live in the arena, not the bucket** — the bucket holds an
arena locator `(page_idx, offset, canon_len)`. A bucket is **occupied** iff
`slot_id ≠ 0 ∧ ¬tombstone`.

**Probe / hit.** On `intern`, probe from `hash & (cap−1)`, stepping `+1` (mod
`cap`). An empty bucket (`slot_id = 0`) ends the probe as a **miss** (new
value). A potential **hit** requires `¬tombstone ∧ bucket.hash = hash ∧
bucket.canon_len = len`; then `memcmp` the stored arena bytes against the
candidate canonical bytes — equal ⇒ **`Hit(slot_id)`**; hash-equal but
bytes-differ ⇒ a **true hash collision**, continue probing. (`memcmp` on the
full canonical form is what makes "equal slot ⇒ structurally equal" exact,
`41 §3b`.)

**Resize.** When the next insert would exceed load factor **0.70**, the index
**doubles** and rehashes every occupied bucket into the new table (slot ids and
arena locators are preserved; a previously-interned value still deduplicates to
its original slot id after a resize). The resize threshold and initial capacity
are X2 tuning constants.

- **Concurrency (reconcile / staging).** The landed resize is a
  **single-writer** rehash (`&mut`), *not* the F4 "lock-free RCU-style
  migration." A lock-free / concurrent index is a **future hardening**
  (native-backend / X4 era), **out of X2 scope** — do not present it as shipped.
- **Tombstones (reconcile).** The `tombstone` flag is **reserved machinery** —
  the landed store has **no per-entry deletion path**, so it is **never set**.
  The only deletion is whole-index `clear()` at a `space` reset (§3). Per-value
  free is **not** in X2 (no GC, `OQ-gc`).
- **Partitioning** is realized as **separate per-`space` indexes** (each `Space`
  owns one), not an `arena_root` key field (reconcile from the F4 §1a prose).

### 1b. Arena page layout

The arena is an **append-mostly chain of fixed-size pages**. Landed page size:
**4 MiB** (good TLB behaviour; large enough to amortize allocation, small enough
that dropping a page on reset is cheap). A page is a growable byte buffer of up
to `PAGE_SIZE`:

```
page  { data : [Byte]  (≤ PAGE_SIZE) }
arena { pages : [page] }              // append-mostly chain
```

Allocation is **bump (append)** into the **tail** page. When a value does not
fit the tail page's remaining space, a **fresh page** is allocated. An
**oversized value** (`> PAGE_SIZE`) is placed in its **own dedicated page**,
then a fresh empty page is pushed so the "tail page is the bump target"
invariant holds for subsequent appends. `append(bytes)` returns
`(page_idx, offset)` (stored in the bucket); `get(page_idx, offset, len)`
borrows the bytes for `memcmp`.

- **Reconcile.** The F4 "huge-page alignment / `mmap`" and the
  `page_id`/atomic-`used` page struct are **not** landed — a page is a plain
  growable byte buffer and bump is **single-writer**. `mmap`-backed pages, huge
  pages, and atomic bump are **future** optimizations (native backend / X4), not
  the X2 contract. `arena_bytes` (§2) is the sum of page lengths.

## 2. Capacity is an engineering choice, not numerology (OQ-5)

**Ken's stance:** the capacity bound is chosen on **engineering grounds**
(encoding width, arena sizing, target scale) — **wide handles** so there is no
practical ceiling, **loud refusal** over silent degradation at any bound, and
**dedup-aware accounting** (capacity is measured in *distinct* values). No
lattice numerology enters.

> A Leech-lattice capacity scheme was **considered and rejected as aesthetic**.
> It would fix **196,560 slots/heap** — the kissing number of Λ₂₄ — and 256
> heaps/chain (an 8-bit heap id + 24-bit slot), giving ~50.3M distinct contents
> and ~17–18 GB/process. But that 196,560 is a flourish: a 24-bit slot field
> already holds ~16M, and nothing in addressing needs the kissing number. The
> number is not load-bearing — this note records the rejection so it is not
> re-proposed.

- **`OQ-5` DECIDED:** keep the **"loud refusal over silent degradation"**
  philosophy — at a limit, fail loudly with a clear error; never silently drop,
  alias, or corrupt — with **no practical ceiling** (wide handles).
- **Slot-field width: 64-bit — DECIDED by the landed store.** Slot ids are
  `UInt64` from a single process-wide monotonic counter (`41 §3b`), starting at
  1 (slot 0 = null sentinel), billions+ and sized past any practical working
  set. The F4 "48 or 64" choice is **settled at 64**; there is **no Leech
  196,560 ceiling**.
- Dedup means real consumption is **one slot per distinct value**, not per
  occurrence — capacity is in **distinct** values (the accounting point below).

**Loud-refusal mechanics.** A `space` carries an **opt-in soft capacity limit**
(default = none / unbounded). On a **new distinct** slot (a probe miss), if
`total_slots ≥ limit`, `intern` returns the typed loud result:

```
CapacityExhausted { limit : UInt64, current : UInt64 }
```

(Reconcile: the landed shape is `{ limit, current }` — there is **no** `arena`
field; an `ArenaId`/space label is a **future** addition once spaces are named,
not in the landed error.) It **MUST NOT** silently drop values, alias slots, or
corrupt the index. The intern result is a **closed** sum `New | Hit |
CapacityExhausted` — there is **no "drop" arm** by construction.

- **Dedup-aware by construction:** the limit counts **distinct** slots (only a
  `New` increments `total_slots`); a dedup **`Hit` never trips it**. N
  occurrences of one value consume **one** slot against the bound (AC1). *(This
  is the AC2 fixed-point trap, §6: the over-limit witness must be a **distinct**
  value — a repeated value hits the dedup path and never reaches the limit
  check.)*

**Limit-site enumeration — every site fails loud, never silent:**

1. **Soft cap reached** → `CapacityExhausted { limit, current }`: a **typed,
   catchable** error surfaced at the `space` boundary — the headline refusal.
2. **True memory exhaustion** (an index-resize allocation or a new arena-page
   allocation fails under real OS OOM) → a **loud fatal** failure (the process
   aborts loudly), **never** a silent drop/alias/corrupt. The soft cap is
   *catchable*; hardware OOM is *loud-fatal* — both satisfy loud-never-silent
   (`OQ-5`); the honest distinction is catchable-vs-fatal, not loud-vs-silent.

**Registered in the runtime-fault taxonomy (`43 §2`, 5th class).**
`CapacityExhausted` is a **loud resource-limit fault** — a class **distinct**
from the *obligation-generating* partial primitives (`43 §2` case 2:
div-by-zero, fixed-width overflow). Ken does **not** generate a static
obligation "this program never exhausts the store" (unprovable in general; the
stance is **detect-and-fail-loud**, not prove-absence). It is always loud,
never silent,
and surfaced at the `space` boundary — honesty-about-the-boundary (`43 §3`), not
a pre-discharged proof obligation.

**Normative: the loud result MUST propagate.** Every layer above the store MUST
surface `CapacityExhausted` (as a catchable `space`-boundary error) and **MUST
NOT** map it to a null/sentinel slot or otherwise swallow it.

> **Flag — landed gap (the AC2 discriminator).** The current interpreter
> (`ken-interp`'s `intern` shim) maps `CapacityExhausted → NULL_SLOT` — a
> **silent drop** (the over-limit value aliases to the null slot). This violates
> `OQ-5`. Team Runtime MUST propagate the loud error; **AC2** (§6) discriminates
> against **exactly this** bug — the conformance must assert the error is
> *raised*, not that the program merely "did not crash" (the silent-NULL_SLOT
> path and a correct-but-uncrashed path are green-vs-green otherwise).

**Dedup-aware accounting (the accounting point, `44 §2`).** The store exposes
occupancy counted in **distinct** values via the `StoreStats` witness (`41 §7`)
plus a `distinct_count()`:

| Field | Meaning |
|---|---|
| `total_slots` | distinct values currently stored (one per `New`) — **the dedup-aware capacity/occupancy measure** |
| `total_interns` | all `intern` calls (including dedup hits) |
| `dedup_hits` | calls that returned an existing slot |
| `arena_bytes` | total arena bytes (sum of page lengths) |
| `index_buckets`, `index_load` | index table size and occupancy fraction |

- **Invariant** (within a `space`, between resets, **absent
  `CapacityExhausted`**): `total_interns = total_slots + dedup_hits`, and
  `distinct_count() = total_slots`. A bound of *K* admits *K* **distinct**
  contents regardless of occurrence count (AC1). **Carve-out (landed):** a
  **refused** `intern` (the at-limit `CapacityExhausted`, §2) increments
  `total_interns` but adds **no** slot and **no** dedup hit, so under refusals
  `total_interns = total_slots + dedup_hits + refused` — the bare equality holds
  only on the no-refusal path.
- **Reset asymmetry (landed — pin it).** A `space` reset (§3) sets `total_slots
  → 0` (occupancy is now zero) but **`total_interns`/`dedup_hits` persist** —
  they are **lifetime witness** counters (`41 §7`), not occupancy. A consumer
  must read `total_slots`/`arena_bytes` as **live occupancy** and
  `total_interns`/`dedup_hits` as **monotone lifetime** stats.
- This accounting is **extensional-safe** (`41 §7`): it reports about the
  *store*, never about individual values' identity or provenance.

## 3. Reclamation and the memory model

**Systems-adjacent positioning (operator ruling, 2026-07-02).** Ken is
**systems-*adjacent***, not a bare-metal systems language: it keeps the
software-engineering / verified aspiration and **yields the true-systems space**
— freestanding, manual memory against the OS kernel — **to Rust**. The
content-addressed managed heap with optional, semantics-invisible reclamation is
the **correct** model for that positioning, not a compromise to apologise for.
The **default is manual + region** reclamation (a `space` reset, below) — a
**systems-native, deterministic** technique (arenas/regions, no mandatory
collector). **Automatic GC is optional, semantics-invisible, and droppable:**
because values are immutable and identity is content (`41 §4`), reclaiming an
unreachable slot changes **nothing observable**, so a collector can be added
later without a language fork and Ken is **not** "a GC language." The model does
**not** chase bare-metal — that space is Rust's. This is settled positioning
(`OQ-systems-target` closed in favour of systems-adjacent), stated here as the
memory-model rationale, not a new fork.

- **No automatic GC / compaction** (`OQ-gc`): the append-mostly store keeps
  slots **stable** for fast ids and O(1) equality. Slots never move; ids are
  never recycled.
- **Manual reclamation = a `space` reset (landed).** `reset` on a `space`:
  **releases the arena's pages** (the page buffers are dropped — memory returns
  to the allocator), **clears the index**, and sets `total_slots → 0`. The
  space's slot ids are **retired, not recycled**: the process-global monotonic
  counter is untouched, so a post-reset id is **strictly greater** than any
  retired id. **Other spaces are untouched.**
  - **Reconcile.** The landed release is a **page-buffer drop** (the arena's
    page chain is cleared, freeing the bytes); the observable contract is
    "`arena_bytes` drops to 0," measured via `StoreStats.arena_bytes` (not RSS,
    which is allocator-dependent). The F4 `madvise(MADV_DONTNEED)`/`munmap` is a
    **future** mechanism for an `mmap`-backed arena (native backend / X4), not
    the X2 contract.
- **Region-scoped lifetime = the `space` boundary.** Each surface `space`
  (`36 §4`) realizes as a store `Space`; when the `space` terminates — or
  `reset`s a bounded unit of work (e.g. one request) — its arena is reclaimed,
  **bounding the working set** (AC4).

**The reclamation / dedup boundary — resolved at the source (the load-bearing X2
ruling).** Because dedup is **per-`space`** (§1) and spaces are
**shared-nothing** (`36 §4.4`), the boundary is well-defined — for any value at
a `space`'s reclamation point there are exactly two outcomes:

- A value that **does not escape** its space is **reclaimed** with the space's
  arena.
- A value that **escapes** — passed to another space, or returned past the
  boundary to a caller — crosses as an **immutable content-addressed value**
  (`36 §4.4`); physically it is **interned into the recipient (surviving)
  space's store** (the physical realization of shared-nothing value passing). It
  lives in a surviving arena and therefore **survives** the sender's reclamation
  (AC4: escaping values survive).

This makes reclamation **content-identity-preserving and observationally
invisible** (AC5): only the dead, non-escaping values of the reset space are
released; every **live** value sits in a surviving store with a **stable** slot
id; the global monotonic, never-reused counter guarantees a retired id is
**never resurrected**, so no live value is renumbered or aliased. Equality
(`41 §4`) is a slot-id compare *within* a store and is unaffected.

- **Staging.** The per-`space` `Space` mechanism + per-space `reset` (with
  isolation and id retirement) is **landed and tested**. *Wiring* a surface
  `space` to a distinct runtime `Space`, and **interning-on-escape** at the
  boundary, is the **X2 build deliverable** (the interpreter currently runs a
  single store). The **escape-detection mechanism** (eager copy at send vs.
  escape analysis) is an implementation tuning — like the small-aggregate
  boundary (`41 §5`) — left to the build; the **observable contract** (escaping
  survives, non-escaping reclaimed, identity preserved) is normative here.
- **No compaction, no moving GC:** slots stay at stable addresses for their
  store's lifetime; fragmentation is managed by the 4 MiB pages + append
  allocation. Automatic refcount/GC stays **deferred and surface-invisible**
  (`OQ-gc`) — values are immutable and identity is content, so reclaiming an
  unreachable slot changes nothing observable; it is addable later **without a
  language fork**. **Do not build it.**

## 4. The lattice machinery — considered, rejected as core (OQ-6)

The Leech/Golay/Co₀ math is **not** the allocator and is **out of the core**
(`OQ-6` DECIDED) — recorded here only as an **optional forward-pointer**, not a
core mechanism. If Ken ever includes any of it, it decomposes into three
**distinct, code-separate** roles, each an opt-in **WS-R** package (`../50-`),
**never** on the allocation hot path:

| Role | What it is | Where it'd live |
|---|---|---|
| **Golay(24,12,8) error-correction** | a `VoyagerList`-style EC-coded sequence | an optional stdlib data type |
| **Kissing-number bitmap (XSet)** | the 196,560-bit set as a fixed-size set/MPHF domain | an optional fixed-domain `Set` impl (`../30-surface/37 §2`) |
| **Co₀/M24 orbit canonicalization** | canonicalizing representatives under symmetry | an optional, separate utility |

- **None is required** for the core language; all are opt-in libraries
  (`../50-`). The specific Co₀ orbit cardinalities (98280/8386560/8292375) are
  **not** carried as fact — if such a facility is built they are to be
  computed/verified, not assumed.
- **`OQ-6` DECIDED:** the lattice machinery is **not in the core** — available
  only as research/optional packages (strategy WS-R), **never on the allocation
  hot path**. Capacity rests on engineering choices, never on lattice
  aesthetics. **Structural guard:** no `mmgroup`/Leech/Co₀ dependency appears on
  the store's build path (a build-time / oracle conformance check, §6).

## 5. Scale and limits validation (X4)

When the native backend lands (X3), **X4** validates scale: the chosen capacity
bound holds under load, dedup behaves, reclamation reclaims, and any boundary is
**loud**. The interpreter + native backend are differential-tested (`42 §5`) so
the value model is identical across them; capacity behaviour is documented, not
discovered in production. X2 delivers the **mechanism** (the loud bound +
dedup-aware accounting + reclamation); X4 **validates it under load** — keep the
bound a stated, stressable number.

## 6. What WS-X must deliver here (X2, X4) and Foundation (K3)

The per-`space` content store with conventional addressing + per-space dedup
(§1, **landed by K3** — X2 extends, does not rebuild); a **deliberate, loud**
capacity bound chosen on engineering grounds (`OQ-5`, §2), with **dedup-aware
accounting** counted in distinct values (§2); **manual + region reclamation**
mapped to the `space` boundary (§3, content-identity-preserving); and (X4)
scale/limits validation (§5). The lattice machinery, if any, is **optional and
off the hot path** (`OQ-6`, §4).

**Acceptance criteria** (each a structural/verdict flip — right vs. the targeted
bug observably differ):

- **AC1 (dedup — distinct, not occurrences).** N occurrences of the **same**
  value consume **one** slot: assert `total_slots == distinct_count()`, **not**
  `== occurrence_count`. (Landed substrate: `dedup_aware_accounting`.)
- **AC2 (loud at-limit — the headline).** An over-limit insert of a **distinct**
  value through the **actual store** **raises** `CapacityExhausted` — and the
  store **never** silently drops/aliases/corrupts. *Discriminator:* assert the
  error is **raised** (right) vs. the **silent `NULL_SLOT` drop** (wrong, the
  current interp shim) — **not** "it didn't crash" (green-vs-green otherwise).
  **Fixed-point trap (the L1 off-grid-witness carry):** the over-limit witness
  MUST be a **distinct** value (a `New`) — a repeated value takes the dedup
  `Hit` path and **never reaches the limit check**, so a same-value witness is
  green-vs-green under both correct and silent-drop code. (Landed substrate:
  `loud_refusal_not_silent` — correctly uses distinct values then a distinct
  over-limit insert.)
- **AC3 (manual reclamation).** A `reset` on a `space`'s arena **releases its
  pages**: assert `arena_bytes` drops to **0** (real release), not a no-op.
  (Landed substrate: `reclamation_releases_pages`.)
- **AC4 (region-scoped lifetime).** A value created in space A is **gone** after
  `A.reset()`, while a value **escaping** to space B (or returned past the
  boundary) **survives** — assert per-space isolation **and** escape-survival.
  (Landed substrate: `space_bounded_reclamation` covers isolation;
  escape-survival is the X2 wiring half.)
- **AC5 (reclamation invisible to identity).** After reclaiming unreachable
  slots, **live** values (in surviving spaces) keep their slot ids + O(1)
  equality, and retired ids are **never resurrected** (a post-reset id is
  strictly greater than any retired one). (Landed substrate:
  `reset_retires_slot_ids`.)

**Conformance:** `../../conformance/runtime/capacity/` — dedup accounting (AC1),
the **loud** at-limit failure (AC2, **NOT silent** — the discriminator against
the `NULL_SLOT` swallow), reclamation page release (AC3), region-scoped lifetime
+ escape-survival (AC4/AC5), and **no-lattice-on-hot-path** (a build-time /
oracle structural check: no `mmgroup`/Leech dependency on the store path).

**QA gate.** Drive **real** values through the **actual** store/arena and
observe the real consequence: a real over-limit insert → a **real raised**
error (never a synthetic flag or counter); a real `reset` → a **real**
`arena_bytes` drop.
Structural witnesses must sit **off the targeted bug's fixed point** (the L1
carry): the AC2 over-limit witness is a **distinct** `New` (a dedup `Hit` never
trips the limit). Foundation/K3 ship the substrate (`41`); X2 hardens it to this
contract; X4 stresses it under load (§5).
