# The runtime and reference semantics

> Status: **DRAFT v0**. Normative for the value model, equality, and the
> reference-semantics role; capacity/representation choices are flagged OQ.
> Contract for WS-X (X1/X2, later X3/X4). Grounded in digest §5–§6 — including
> the two big corrections: **heterogeneous typed values (not uniform f64)** and
> **FNV-1a + memcmp content addressing (not Leech-lattice geometry)**.

The **interpreter is the reference semantics** (`../00-overview.md §3`): the
meaning of a Ken program *is* its evaluation here. A later native backend (X3)
is correct iff it agrees with the interpreter on a differential corpus; the
interpreter never stops being the oracle.

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

## 2. The two corrections this section encodes

- **No uniform f64.** Scalars (`Int`, `Bool`, `Float`, handles) are **unboxed,
  typed** machine values, never routed through a float or a heap slot (`41
  §scalars`). The prototype already does this; the analysis's "every value is an
  f64" premise is not Ken's model (`35-numbers.md`).
- **Conventional content addressing.** The heap is addressed by a **fast hash
  (FNV-1a-style) + `memcmp`**, with slot ids a monotonic counter — **not** by
  Leech-lattice geometry or Co₀-orbit canonicalization, which the reality-check
  showed are *never* on the allocation path (`41 §addressing`, `44`). The
  Leech/Golay/Co₀ machinery, if used at all, is scoped to three *separate,
  optional* roles (`44 §lattice`).

## 3. Design principles

- **Immutability + sharing.** Pure values are immutable; equal values are shared
  (dedup). Identity is *what a value is*, not where it lives (`41 §equality`).
- **Loud refusal over silent degradation.** Resource limits fail loudly, never
  silently corrupt (`44`) — a real prototype principle Ken keeps, decoupled from
  the specific Leech-derived numbers.
- **Reference first, performance behind it.** The interpreter is simple and
  correct; speed is the native backend's job (X3), differential-tested against
  this.

## 4. What WS-X must deliver (ties to X1/X2, G1/G6/G5-perf)

The reference interpreter (X1) that runs the G1 vertical slice and is the oracle
for everything after; the content-addressed runtime (X2) with O(1) equality +
dedup, reimplemented cleanly with conventional addressing; and (later) the
native backend (X3) + scale validation (X4) with any capacity boundary
*deliberate and loud*. Conformance: `../../conformance/runtime/`.
