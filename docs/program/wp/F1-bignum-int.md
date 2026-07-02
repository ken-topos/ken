# WP F1 — Genuine arbitrary-precision `Int` (retire the i128 ceiling)

## Objective

Make `Int` genuinely arbitrary-precision in the interpreter so `add_int` /
`sub_int` / `mul_int` are **total** — no i128 wrap, no debug panic. F1 is the
**dependency root** of the ratified Phase-2 tranche (`18a §4.1`): the `Decimal`
and `Char` demotes, `Float.toDecimal` exactness, and the `checked_*` /
`saturating_*` derivations all ride a real bignum `Int`.

This is a **wrong-value fix in the tested-not-trusted interpreter ring** — the
kernel is intact (`ken-kernel/src/obs.rs:84` keeps `Eq` at a primitive type
neutral; there is no evaluator dependency in the kernel), so today's i128 wrap
is a wrong *value*, never a false proof. The correctness AC delivered here is
the **precondition for the eventual K3 trusted-base promotion** of these ops:
a reduction that can produce a wrong value cannot be promoted to
kernel-executed, so "`add_int` NATIVE iff bignum" is what makes that promotion
admissible at all.

## Fixed inputs — settled; do NOT reopen

- **Boundary ratified (Pat, 2026-07-02).** `add/sub/mul/eq/leq_int` stay
  **NATIVE iff bignum** (`spec/10-kernel/18a §5`); this WP delivers the
  "iff bignum" half. Do not relitigate native-vs-derived.
- **Arithmetic source = CURATE, not construct (ADR 0009 tier-a).** Source the
  production arithmetic from a **battle-tested, pure-safe-Rust,
  permissively-licensed, actively-maintained** arbitrary-precision integer
  crate — the ADR-0009 "select an industry-trusted component" move for an
  outer-ring capability (rubric step 1). Default candidate **`num-bigint`**; if
  it carries un-audited `unsafe`, use a `#![forbid(unsafe_code)]` equivalent
  (`ibig`, `dashu`). Vendor + pin + record the dependency delta per
  `spec/60-security/63` + ADR 0009. **Do NOT construct a bignum in-tree for
  production** — curation reaches the trust an oracle-netted outer-ring
  evaluator needs; in-tree/proved construction is the *deferred K3 question*
  (ADR 0009 tier-c migration), not this WP.
- **Reuse the existing store representation.** `ken-runtime` already carries
  `Value::BigInt { sign, limbs }` (content-addressed, `canonical.rs`,
  `minimal_limbs` invariant). The evaluator's bignum must convert **to/from**
  that representation losslessly at the store boundary — reuse it, do not
  fork a second encoding.
- **The reduction interface is FROZEN.** The symbol-keyed `prim_reduce` arms,
  their arity, and their names, and the `PrimReduction::Op` registrations in
  `ken-elaborator`, do **not** change. F1 replaces the *representation and the
  arithmetic behind* the arms, not the surface.
- **Interp-local; NOT a trusted-base promotion.** F1 promotes nothing into
  `trusted_base()` and touches **no `ken-kernel` file**. The K3 promotion
  (kernel-executed reductions) is a separate, later WP that re-decides the
  *trusted* representation behind this same frozen interface.

## Scope

**IN:**

- Replace `EvalVal::BigInt(i128)` (`ken-interp/src/eval.rs:140`, perishable
  anchor) with the curated arbitrary-precision type (or a thin newtype over it).
- Rewrite `exact_int_binop` and the `add_int` / `sub_int` / `mul_int` arms
  (`eval.rs:673-675`, perishable) plus the `eq_int` comparison over the new
  type — **total, no i128 intermediate** anywhere on the arithmetic path.
- The lossless eval ↔ `Value::BigInt { sign, limbs }` conversion at the store
  boundary.
- The small-int fast path (`EvalVal::Int(i64)` for in-range values) MAY stay as
  a representation optimization **iff** it lives entirely behind the frozen
  interface and is **never** the arithmetic path that wraps (promote to the
  bignum type before any op that could overflow i64/i128).

**OUT (sibling tranche WPs, each their own frame):**

- **F5** — `leq_int` registered-but-unreduced. Its reduce arm must be
  bignum-correct across the F1 boundary, but it lands as its own WP; F1 only
  guarantees the representation `leq_int` will compare over.
- **F2** — bare fixed-width `*_intN` obligation emission. **F3** — retire
  the legacy wrapping arms.
- `div_int` / `mod_int` partiality (the div-by-zero face-(b) obligation) —
  the partiality discipline (`18a §2`), not re-opened here; F1's representation
  must *support* correct bignum div/mod when that WP lands.
- The `Decimal` / `Char` demotes and any K3 trusted-base promotion.

## Acceptance (testable — the spec-leader's conformance seeds must cover)

1. **No-wrap totality.** `mul_int(2^127, 2)`, `mul_int(2^64, 2^64)`,
   `add_int(2^128 - 1, 1)`, and a product chain exceeding 2^1000 all reduce to
   the **exact** value — no panic, no wrap. (These are the exact F1 failure
   operands.)
2. **Independent differential oracle — NOT green-vs-green (`18a §3`).** The
   corpus reference is an **independent** arbitrary-precision source (golden
   vectors from an external tool, or a distinct crate / small schoolbook
   reference), **never the production crate on both sides** — `18a §3`'s
   "same wrong code on both sides" is the explicit anti-pattern. Boundary
   operands: across 2^63, 2^64, 2^127, 2^128; negatives; zero; mixed-sign
   `mul`. The reference grounds the floor ops (`add_int`, `mul_int`, `eq_int`)
   that have no lower Ken op to defer to.
3. **Store round-trip.** Every evaluator bignum converts to
   `Value::BigInt { sign, limbs }` and back **byte-identically**;
   content-addressing (`canonical.rs`) stays stable and the `minimal_limbs`
   (trailing-zero-strip, one-zero-limb-for-zero) invariant is preserved.
4. **Workspace-green, not crate-green (the K7 lesson).** `cargo test
   --workspace` is the no-regression gate. F1 changes reduction *values*; any
   `packages/` / `.ken` / golden-test artifact riding the old i128 behavior
   **migrates land-together in the one workspace-green unit**. Do **not** assert
   a "`ken-interp`-only diff."
5. **Dependency delta recorded.** The curated crate is vendored, version-pinned,
   licensed-clean (permissive, non-copyleft — clean-room-compatible), and its
   addition is documented per §63 + ADR 0009, with `unsafe`-status verified
   (none, or audited-and-noted).

## Guardrails (do-not-reopen)

- **No `ken-kernel` file changes.** F1 is interp-local; the kernel does not
  reduce these ops (`obs.rs:84`). If a kernel change appears necessary, **STOP
  and escalate to the Steward** — it is a scope error, not an F1 task.
- **Do not change the reduction interface** — symbol names, arity, or the
  `PrimReduction::Op` registrations.
- **Do not add to `trusted_base()`** — no `declare_primitive` /
  `declare_postulate`; F1 changes behavior behind existing registrations only.
- **One production type + one *independent* oracle reference** — no third
  encoding, and the oracle reference is never the production crate.
- **Current-state claims are perishable** — the `eval.rs:140` / `:673-675`
  line anchors verify against the landed code at pickup, not against this brief.

## The K7 lesson — load-bearing here (COORDINATION §7)

F1 is a **reduction-value change**, so its blast radius is wider than the file
it touches. The *soundness* surface is narrow (interpreter-only, kernel
untouched — legitimately asserted), but the **landing unit is
workspace-wide**: downstream proof terms and golden vectors that rode the i128
behavior migrate in the same green unit. Distinguish them in the diff, and make
the no-regression AC `cargo test --workspace`, never `-p ken-interp`.

## Logistics

- **Team:** Runtime (owns `ken-interp` + `ken-runtime`).
- **Branch:** `wp/F1-bignum-int` off `origin/main`.
- **Depends:** none — F1 is the tranche root. **Blocks:** `Decimal` / `Char`
  demote, `checked_*` / `saturating_*` derivation, `Float.toDecimal` exactness,
  F5 correctness across the boundary.
- **Gates:** Architect soundness (interp-local, kernel-untouched, oracle nets
  rather than green-vs-green) + conformance-validator oracle/burden (the
  independent-reference discipline, the boundary corpus) — both hard ACs.
- **Pipeline:** Steward frame (this doc) → spec-leader elaboration →
  Integrator merge to `main` → Runtime team kickoff. Cites **ADR 0009** (the
  curate-not-construct decision is the first Phase-2 instance of the ratified
  supply strategy).
