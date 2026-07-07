# The runtime and reference semantics

> Status: **DRAFT v0**. Normative for the value model, equality, and the
> reference-semantics role; capacity/representation choices are flagged OQ.
> Contract for WS-X (X1/X2, later X3/X4). It rests on two design commitments:
> **heterogeneous typed values (not uniform f64)** and **FNV-1a + memcmp content
> addressing (not Leech-lattice geometry)**.

The **interpreter is the reference semantics** (`../00-overview.md §3`): the
meaning of a Ken program *is* its evaluation here. A later native backend (X3)
is correct iff it agrees with the interpreter on a differential corpus; the
interpreter never stops being the oracle (the reference implementation against
which later backends are validated).

## 1. What the runtime provides

1. A **value model** (`41-values.md`): how Ken values are represented —
   heterogeneous typed immediates for scalars, a **content-addressed heap** for
   compound/identity-bearing values, with O(1) structural equality and global
   deduplication.
2. The **operational semantics** (`42-evaluation.md`): how core terms reduce to
   values, how effects act, and how `unknown` propagates through partial
   programs.
3. **Termination/totality at runtime** (`43-termination.md`): what is guaranteed
   total (the kernel-checked core) vs. where partiality/`unknown` can appear.
4. The **content store and its limits** (`44-capacity.md`): addressing, dedup,
   reclamation, and the (deliberately chosen, not numerologically fixed)
   capacity story.
5. The **checked-core package** (`46-checked-core-package.md`): the stable
   post-elaboration, kernel-admitted compiler input, including version,
   semantic-hash, metadata, and trust-coverage rules.
6. The **erasure/runtime-IR boundary** (`47-erasure-runtime-ir.md`): the first
   executable compiler artifact below checked core, including proof erasure,
   runtime IR, loud unsupported-erasure rejection, and interpreter comparison
   observations.

## 2. The two design commitments this section encodes

- **No uniform f64.** Scalars (`Int`, `Bool`, `Float`, handles) are **unboxed,
  typed** machine values, never routed through a float or a heap slot (`41 §1`).
  Each value's static type fixes its representation directly (`35-numbers.md`).
- **Conventional content addressing.** The heap is addressed by a **fast hash
  (FNV-1a-style) + `memcmp`**, with slot ids a monotonic counter — **not** by
  Leech-lattice geometry or Co₀-orbit canonicalization, which are *never* on the
  allocation path (`41 §3`, `44`). The Leech/Golay/Co₀ machinery, if used at
  all, is scoped to three *separate, optional* roles (`44 §4`).

## 3. Design principles

- **Immutability + sharing.** Pure values are immutable; equal values are shared
  (dedup). Identity is *what a value is*, not where it lives (`41 §equality`).
- **Loud refusal over silent degradation.** Resource limits fail loudly, never
  silently corrupt (`44`) — independent of any specific Leech-derived numbers.
- **Reference first, performance behind it.** The interpreter is simple and
  correct; speed is the native backend's job (X3), differential-tested against
  this.

## 4. What WS-X must deliver (ties to X1/X2, G1/G6/G5-perf)

The reference interpreter (X1) that runs the G1 vertical slice and is the
reference semantics for everything after; the content-addressed runtime (X2)
with O(1) equality + dedup and conventional addressing; and (later) the native
backend (X3) + scale validation (X4) with any capacity boundary
*deliberate and loud*. Conformance: `../../conformance/runtime/`.
