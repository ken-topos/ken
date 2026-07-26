# The runtime and reference semantics

> Status: **DRAFT v0**. Normative for the value model, equality, callable
> boundary, and reference-semantics role; capacity/representation choices are
> flagged OQ.
> Contract for WS-X (X1/X2, later X3/X4). It rests on two design commitments:
> **heterogeneous typed values (not uniform f64)** and **deterministic durable
> canonical bytes separated from private in-process representation**.

The **interpreter is the reference semantics** (`../00-overview.md §3`): the
meaning of a Ken program *is* its evaluation here. A later native backend (X3)
is correct iff it agrees with the interpreter on a differential corpus; the
interpreter never stops being the oracle (the reference implementation against
which later backends are validated).

## 1. What the runtime provides

1. A **value model** (`41-values.md`): how Ken values are represented —
   heterogeneous typed values, deterministic durable bytes for closure-free
   canonical data, private in-process storage, and runtime-local opaque
   ordinary closures.
2. The **operational semantics** (`42-evaluation.md`): how core terms reduce to
   values, how effects act, and how `unknown` propagates through partial
   programs.
3. **Termination/totality at runtime** (`43-termination.md`): what is guaranteed
   total (the kernel-checked core) vs. where partiality/`unknown` can appear.
4. The **runtime resource contract** (`44-capacity.md`): loud declared limits,
   semantics-invisible reclamation, and private storage choices.
5. The **checked-core package** (`46-checked-core-package.md`): the stable
   post-elaboration, kernel-admitted compiler input, including version,
   semantic-hash, metadata, and trust-coverage rules.
6. The **erasure/runtime-IR boundary** (`47-erasure-runtime-ir.md`): the first
   executable compiler artifact below checked core, including proof erasure,
   runtime IR, loud unsupported-erasure rejection, and interpreter comparison
   observations.
7. The **executable-artifact contract**
   (`48-executable-artifact-contract.md`): the identity/report envelope above
   checked core and runtime IR for closed Ken-only native executable attempts,
   including native artifact facts, toolchain facts, explicit unavailable
   lanes, and no-promotion rules.

## 2. The two design commitments this section encodes

- **No uniform f64.** Scalars (`Int`, `Bool`, `Float`, handles) retain their
  declared types and operations; they are never routed through a semantic
  decode-from-float stratum (`41 §1`). A runtime may use unboxed machine values
  or another private representation without changing the value
  (`35-numbers.md`).
- **Canonical durability, private runtime representation.** Closure-free
  canonical values crossing a durable boundary have deterministic bytes;
  copying, sharing, interning, hashing, indexing, and allocation are private
  runtime choices (`41 §3`, `44`). Leech/Golay/Co₀ machinery is not a semantic
  dependency and, if used at all, stays in optional profiled roles (`44 §4`).

## 3. Design principles

- **Immutability without observable representation.** Pure values are
  immutable; copying or sharing equal data does not change program behavior.
  Ordinary closures are runtime-local opaque callables with no structural or
  persistent identity (`41 §2.1`).
- **Loud refusal over silent degradation.** Resource limits fail loudly, never
  silently corrupt (`44`) — independent of any specific Leech-derived numbers.
- **Reference first, performance behind it.** The interpreter is simple and
  correct; speed is the native backend's job (X3), differential-tested against
  this.

## 4. What WS-X must deliver (ties to X1/X2, G1/G6/G5-perf)

The reference interpreter (X1) that runs the G1 vertical slice and is the
reference semantics for everything after; the canonical-data runtime (X2) with
deterministic durable bytes, private in-process representation, and a
runtime-local opaque callable boundary; and (later) the native backend (X3) +
scale validation (X4) with any declared capacity boundary *deliberate and
loud*.
Conformance: `../../conformance/runtime/`.
