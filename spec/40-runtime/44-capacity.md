# The content store: capacity, reclamation, and the lattice

> Status: **DRAFT v0**. Normative for the *principles* (loud refusal, dedup
> accounting, no-lattice-on-the-hot-path). **`OQ-5`/`OQ-6`/`OQ-gc` DECIDED**
> (operator, 2026-06-27): **engineering-chosen** capacity (wide handles → no
> practical ceiling) with **loud refusal**; the **lattice machinery out of the
> core** (optional research packages only); **reclamation a semantics-invisible
> implementation detail** (manual+region now, automatic GC addable later without
> touching surface or semantics — deferred). Contract for WS-X **X2/X4**.
> Encodes the reality-check's biggest structural correction: the Leech-lattice
> numbers are *aesthetic*, not load-bearing (digest §5).

## 1. The store

The content-addressed heap (`41 §2`) is the process's value store: an
append-mostly arena of interned values, addressed by hash+memcmp (`41 §3`), with
a process-wide `(root, hash) → ref` index giving global dedup and auto-chaining
across arenas.

## 2. Capacity is an engineering choice, not numerology (OQ-5)

The prototype fixes **196,560 slots/heap** — the kissing number of the Leech
lattice Λ₂₄ — and 256 heaps/chain (an 8-bit heap id + 24-bit slot), giving
~50.3M distinct contents and ~17–18 GB/process. The reality-check confirms these
*numbers* but shows the **196,560 is aesthetic**: a 24-bit slot field holds
~16M; nothing in addressing needs the Leech kissing number.

**Ken's stance:** choose any capacity bound on **engineering grounds** (encoding
width, arena sizing, target scale), *not* lattice numerology.

- **`OQ-5` DECIDED:** keep the **"loud refusal over silent degradation"**
  philosophy — at a limit, fail loudly with a clear error; never silently drop,
  alias, or corrupt — with **no practical ceiling** (wide handles: a 48-/64-bit
  slot field for billions+, sized to the hardware). The exact width is an X2/X4
  constant; the *stance* — deliberate, loud, no Leech ceiling (strategy G5-perf)
  — is permanent.
- Dedup means real consumption is **one slot per distinct value**, not per
  occurrence — capacity is in *distinct* values, an important accounting point
  for any bound chosen.

## 3. Reclamation

- **No automatic GC / compaction** in the DRAFT (matching the prototype): the
  append-mostly store keeps slots stable for fast ids and O(1) equality.
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

## 4. The lattice machinery — three separate optional roles (OQ-6)

The reality-check is explicit that the Leech/Golay/Co₀ math is **not** the
allocator and decomposes into **three distinct, code-separate** roles. If Ken
includes any of it, it is scoped to these and **never** the hot path:

| Role | What it is | Where it'd live |
|---|---|---|
| **Golay(24,12,8) error-correction** | a `VoyagerList`-style EC-coded sequence | an optional stdlib data type |
| **Kissing-number bitmap (XSet)** | the 196,560-bit set as a fixed-size set/MPHF domain | an optional fixed-domain `Set` impl (`../30-surface/37 §2`) |
| **Co₀/M24 orbit canonicalization** | canonicalizing representatives under symmetry | an optional, separate utility |

- **None is required** for the core language; all are opt-in libraries
  (`../50-`).
- The unverified doc claim of specific Co₀ orbit cardinalities
  (98280/8386560/8292375) is **not** carried as fact (digest §5b); if such a
  facility is built, those are to be computed/verified, not assumed.
- **`OQ-6` DECIDED:** the lattice machinery is **not in the core** — available
  only as research/optional packages (strategy WS-R), **never on the allocation
  hot path**. This is the final break with the prototype's central aesthetic
  conceit (the reality-check's biggest structural correction).

## 5. Scale and limits validation (X4)

When the native backend lands (X3), **X4** validates scale: the chosen capacity
bound holds under load, dedup behaves, reclamation reclaims, and any boundary is
**loud**. The interpreter + native backend are differential-tested (`42 §5`) so
the value model is identical across them; capacity behaviour is documented, not
discovered in production.

## 6. What WS-X must deliver here (X2, X4)

The content store with conventional addressing + global dedup (X2); a
**deliberate, loud** capacity bound chosen on engineering grounds (OQ-5), with
dedup-aware accounting; manual/region reclamation; and (X4) scale/limits
validation. The lattice machinery, if any, is **optional and off the hot path**
(OQ-6). Conformance + scale: `../../conformance/runtime/capacity/` — dedup
accounting, a loud at-limit failure (not silent), and reclamation releasing
pages.
