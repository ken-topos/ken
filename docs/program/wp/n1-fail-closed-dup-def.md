# WP N1 — fail-closed duplicate-definition slice (ADR 0014 round 1)

Owner path: **spec enclave (Lane A) → Language team (Lane B)**. Design source of
truth: `docs/adr/0014-cross-package-resolution-and-fail-closed-collision.md`
(Accepted), forks **MRES-5 + MRES-7 + MRES-8**. Program:
`docs/program/wp/adr0014-work-program.md`. Size **S**. Deps: **none** (acts on
one compilation unit's own globals — no loader, no import system).

This is the operator's requested splittable early slice. It runs on the `#29`
template: **Lane A (spec + conformance golden, enclave) merges first; Lane B
(build, Language) implements against the landed spec.**

## Fixed inputs — SETTLED, do not reopen

These are Accepted in ADR 0014; the WP implements them, it does not relitigate:

- **MRES-5.** A second top-level **definition** of the same single-namespace name
  in **one compilation unit** is a **hard error** (not silent last-writer-wins).
- **MRES-7.** Under Ken's single-flat-namespace (types-are-terms, the D8-③
  ruling), a **class and a constructor with the same name** (`class Eq` vs ctor
  `Eq`) is the **same collision** → the **same hard error**. The
  qualified-constructor *escape* is a separable later investment — **out of scope
  here**; N1 only makes the collision an error.
- **MRES-8.** The check lives at the **`resolve_decl` choke-point**, by
  **generalizing the existing `check_no_reserved_sugar_collision` guard**
  (`crates/ken-elaborator/src/resolve.rs`, today a 3-name reserved list:
  Refl/Axiom/absurd) to a **duplicate-definition check over the live globals**,
  **preserving** the arity-gated-sugar exclusion.
- **Arity-gated-sugar coexistence STAYS LEGAL.** The guard's own caveat warns
  that a blanket reject over-rejects legitimate arity-gated `Eq`/`J` coexistence.
  That coexistence is **preserved** — the check must exclude the arity-gated
  sugar cases exactly as the current guard does. This is the load-bearing
  discrimination: **duplicate ordinary definition = error; arity-gated sugar
  pair = still legal.**
- **Scope wall.** Elaborator surface only. **Zero** kernel / prelude / semantics
  / `trusted_base()` / Cargo / lock delta. This is not one of the #8
  disambiguation forks — it is the 4th option (hard error) that complements them.

## Lane A — spec + conformance golden (enclave) · merges FIRST

**Deliverable (mandated outline; each item ends in a concrete choice):**

1. **Normative rule in `spec/30-surface/33-declarations.md`.** State, as
   normative surface behavior: *two top-level definitions of the same name in one
   compilation unit are rejected; a class and a constructor sharing a name are
   the same single-namespace collision and are likewise rejected; the arity-gated
   sugar coexistence (`Eq`/`J`) remains legal.* Place it with the existing
   single-namespace / D8-③ material; cite the ADR fork tags. Keep the wording
   role-blind (it is about *names*, not *kinds*).
2. **Do NOT touch §3.3's shadowing clause.** MRES-6 (local-vs-import clash) is a
   **separate, later WP (N3)** and reverses §3.3 there. N1 is
   **definition-vs-definition in one unit only** — the §3.3 import-shadowing text
   is out of scope and unchanged here.
3. **Conformance golden.** Extend the appropriate `conformance/surface/…` seed
   with, at minimum: (a) a duplicate ordinary top-level definition → **rejected**
   with the specific error; (b) `class Eq` + ctor `Eq` in one unit → **rejected**;
   (c) a **discriminating positive arm** — an arity-gated `Eq`/`J` (or the real
   sugar pair) coexisting → **accepted**. (a)/(b) vs (c) must be a genuine
   **reject vs accept flip** on the collision axis (not green-vs-green): same
   surrounding context, only the collision present/absent.

**AC (Lane A).** spec rule normative and self-consistent; golden encodes the
reject cases + the discriminating accept arm; the golden is red-until-built
(the build has not landed yet) — mark it so, exactly as the F3b conformance
convention. Doc/spec/conformance-only; no crates delta. Hand the SHA to Steward;
Steward publishes; **Lane B unblocks on the landed spec.**

## Lane B — build (Language team) · merges SECOND, on landed Lane A

**Deliverable.** Generalize `check_no_reserved_sugar_collision` at the
`resolve_decl` choke-point to reject a top-level definition whose name already
occupies the unit's globals, **preserving** the arity-gated-sugar exclusion.
Emit a **specific error variant** (name the offending name + that it is a
duplicate/collision), not a generic failure. Wire the class/ctor case through
the same funnel (single-flat-namespace: a class name and a ctor name collide).

**AC (Lane B).**
- Duplicate top-level definition → rejected with the **specific** error variant
  (assert the variant, not merely `is_err`).
- `class Eq` vs ctor `Eq` in one unit → rejected via the same funnel.
- Arity-gated `Eq`/`J` coexistence → **still accepted** (the exclusion holds).
- The Lane A conformance golden flips **red → green**.
- Full `scripts/ken-cargo test --workspace` green; `git diff --check` clean;
  scope = `crates/ken-elaborator` (+ tests) only; **zero** kernel / prelude /
  semantics / Cargo / lock delta.

**Review.** Architect-terminal (surface/soundness of the collision funnel +
the preserved arity-gated exclusion). CV's golden is the acceptance oracle.

## Do-not-reopen guardrails

- The qualified-constructor escape (MRES-7) is **not** built here — do not add
  disambiguation syntax; only make the collision an error.
- The §3.3 import-shadowing reversal is **N3**, not N1 — do not touch it.
- Do not widen the check to shadowing an import/prelude (that is N3/MRES-6) — N1
  is **duplicate definition within one unit** only.
- Preserve the arity-gated-sugar exclusion verbatim in behavior — a regression
  that rejects `Eq`/`J` coexistence is a **fail**, not a stricter-is-better win.
