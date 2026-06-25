# The content store: capacity, reclamation, and the lattice

> Status: **DRAFT v0**. Normative for the *principles* (loud refusal, dedup
> accounting, no-lattice-on-the-hot-path); the exact capacity bound and whether
> to include the lattice machinery at all are **OQ-5/OQ-6**. Contract for WS-X
> **X2/X4**. Encodes the reality-check's biggest structural correction: the
> Leech-lattice numbers are *aesthetic*, not load-bearing (digest §5).

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

- The DRAFT keeps the **"loud refusal over silent degradation"** *philosophy* —
  at a limit, fail loudly with a clear error; never silently drop, alias, or
  corrupt — while leaving the **actual bound** to X2/X4 (e.g. a 32- or 48-bit
  slot field for billions of values, sized to the hardware). **OQ-5** records
  this; the bound is "deliberate and loud" (strategy G5-perf), whatever its
  value.
- Dedup means real consumption is **one slot per distinct value**, not per
  occurrence — capacity is in *distinct* values, an important accounting point
  for any bound chosen.

## 3. Reclamation

- **No automatic GC / compaction** in the DRAFT (matching the prototype): the
  append-mostly store keeps slots stable for fast ids and O(1) equality.
- **Manual reclamation exists** — `clear`/`reset`/`strip`-style operations
  release an arena's pages (e.g. `madvise(MADV_DONTNEED)`); a `space`/region
  boundary (`../30-surface/36 §4`) bounds a working set's lifetime. Whether Ken
  later adds automatic reclamation (refcount/GC for the content heap) is
  **OQ-gc**; the DRAFT is manual + region-scoped, which suits the immutable,
  dedup'd model.

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
- **OQ-6** records whether to build any of this at all (the DRAFT: not in the
  core; available as research/optional packages, strategy WS-R).

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
