# WP DS-6a — Int decidable-equality certificate (required core)

**Owner:** Kernel team. **Operator-ruled** (2026-07-10, "do the right thing");
**Architect-designed** ([ADR 0013](../adr/0013-int-decidable-equality-kernel-posture.md),
`evt_36tfyha8nrrd0`). Buildable immediately; independent of DS-6b.

## Goal

Retire the per-catalog `Axiom` posture for `DecEq Int` / `Eq Int` by
relocating the trust into **one named kernel decidable-equality certificate**,
admitted as trusted kernel vocabulary and counted honestly in `trusted_base()`.
Read ADR 0013 first — it is the durable design; this WP is the build unit.

## Scope

1. **General opt-in certificate mechanism.** Add a kernel registration by which an
   **opaque primitive type MAY carry a decidable-equality certificate** — its `eq`
   op plus the `sound`/`complete` certificate pair. Registration is per-primitive
   and opt-in; **unregistered primitives stay neutral exactly as today**
   (`obs.rs:84`, the fail-safe default). Do NOT special-case `Int` in the
   mechanism — `Int` is the first *registrant*, not the rule.
2. **Register `Int`** with `eq_int` (already a `trusted_base()` primitive).
3. **Rewire `instance DecEq Int` / `Eq Int`** so their `sound`/`complete` law
   fields point at the certificate instead of `Axiom`. Remove the catalog `Axiom`s.
4. **`Char` instance rides free** — write the trivial `instance DecEq Char` (none
   exists today); it bottoms out at `Eq Int` through the carrier-lowered
   refinement, zero Char-specific kernel work.

## Boundary / constraints

- **TCB delta is EXACTLY one line item:** *"the kernel trusts `eq_int`
  decides propositional `Int` equality, both directions"* (split sound +
  complete). **No new primitive** (`eq_int` already trusted); **admits
  nothing more** (only `Int`, only equality). The `trusted_base()`
  accounting must show exactly this and no more — the Architect greps
  it at the gate.
- **Zero new catalog `Axiom`; the existing `DecEq Int`/`Eq Int` `Axiom`s are
  REMOVED** (net catalog-Axiom delta negative). No `sorry`.
- The certificate makes the **universal** laws (`∀ abstract x y`) trusted-but-named
  — it does **not** make concrete `Equal Int 5 5` *compute* (that is DS-6b). Do not
  over-claim computation here.

## Acceptance bar (soundness — the Architect gates the exact rule at pseudocode)

- **Soundness / over-equate:** the certificate does not admit a proof of a false
  equality — a proof of `Equal Int 5 6` is kernel-**REJECTED** (assert the specific
  `KernelRejected`/`TypeMismatch`, not bare `is_err`).
- **Neutral preserved:** `Eq Int x y` for abstract `x, y` stays **neutral**;
  unregistered primitives' `Eq` stays neutral (no regression to `obs.rs` behavior).
- **Zero-Axiom-delta (DS-2-style before/after set-diff):** elaborating the new
  instances adds exactly the certificate line(s) to `trusted_base()` and **no**
  catalog `Axiom`.
- Home tests: kernel-side beside `k7_eq_at_inductive_whnf.rs`; catalog-side beside
  `ds1_empty_dec_acceptance.rs`. Targeted builds only (`-p <crate> <test>`);
  full-suite green is proven in CI at merge.

## Gate

Kernel ring (kernel-leader → kernel-implementer → kernel-qa) → **@architect
soundness gate** (gates the exact certificate rule + `trusted_base` accounting;
confirms only `Int`-equality is admitted and the general mechanism is fail-safe) →
**conformance** (the discriminating arms above) → `git_request` to Steward →
CI-gated merge. Prominent kernel/TCB land — logged for the operator. Own retro;
flag every judgment call. No WP-token identifiers in production source.
