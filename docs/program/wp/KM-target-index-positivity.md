# KM-target-index-positivity

**Owner:** Kernel build team.
**Reviewer:** Kernel QA and Architect mandatory.
**Branch:** `wp/KM-target-index-positivity`.
**Status:** Steward frame, queued. This does not block the current GADT work.
**Size:** S/M. **Risk:** high enough for Architect review because this touches
kernel inductive admission.

## 0. Trigger

`SURF-gadt-elaboration` landed a surface-side guard for constructor result
indices after Architect found that the explicit-data lowering path could emit a
constructor target like:

```ken
data Bad : Type -> Type where {
  BadMk : Bad (Bad Int)
}
```

The landed fix rejects same-family occurrences in constructor result-index
terms before declaration install. That was the right bounded fix for the
elaboration WP, but it also exposed a kernel authority gap: the kernel strict
positivity check currently inspects family parameters, family indices, and
constructor argument types, but not `ConstructorDecl.target_indices`.

Pat asked for the non-blocking kernel fork: make the kernel enforce the relevant
positivity/admission rule for constructor target indices, then stage a small
surface extension only after that kernel authority exists.

## 1. Objective

Make kernel inductive admission responsible for constructor result target-index
terms.

Today `ConstructorDecl.target_indices` are real kernel declaration payload:

- they are stored in `crates/ken-kernel/src/env.rs`;
- `InductiveDecl::build_types` uses them to build constructor types;
- eliminator method types use them when forming constructor conclusions;
- but `check_positivity` does not currently scan them.

This WP closes that gap. A constructor target index must not be a place where a
producer can smuggle a same-family occurrence that the kernel never admitted.

This WP does **not** remove or relax the current surface guard in
`SURF-gadt-elaboration`. Surface expressivity is a follow-on after the kernel
gate lands.

## 2. Required D0 Audit

Before implementation, post a short D0 audit naming the exact code paths and
answering:

1. Where are constructor `target_indices` type-checked and installed today?
2. Where does `check_positivity` inspect `params`, `indices`, and constructor
   `args`, and where does it currently skip `target_indices`?
3. Does the existing `14 §8` algorithm already imply a D-free target-index
   guard, because index-embedded occurrences are conservatively rejected?
4. If a broader recursive target-index class is proposed, what exact positivity
   rule admits it, and why is it not nested or negative recursion?
5. Is the implementation kernel-only, or does it require any elaborator/surface
   change?

If D0 concludes that a broader admitted class requires changing the normative
strict-positivity algorithm rather than applying the existing conservative
guard to a previously skipped field, stop and route the scope fork to Steward
and Architect before building it.

## 3. Deliverables

Implement on `wp/KM-target-index-positivity`:

1. Extend kernel inductive admission so every constructor `target_indices` term
   is checked by the kernel before admission succeeds.
2. At minimum, reject a same-family occurrence in any target index, using the
   existing `occurs(d, t)` authority or an equivalent stricter check.
3. Keep constructor argument recursion routed through the existing
   `check_pos_arg` path. Do not weaken W-style, negative, or nested recursion
   behavior.
4. Add a focused kernel regression for a declaration whose constructor
   `target_indices` contain the family being declared, equivalent to:

   ```ken
   data Bad : Type -> Type where {
     BadMk : Bad (Bad Int)
   }
   ```

   The test should build the kernel declaration directly or otherwise prove the
   rejection is kernel-side, not only elaborator-side.
5. Add or preserve a positive indexed-family sanity case, such as a `Vec`-style
   target index that mentions only data parameters and constructor binders.
6. Preserve the landed `SURF-gadt-elaboration` behavior. No surface syntax
   expansion belongs in this WP.

## 4. Acceptance Criteria

AC1. Kernel admission rejects recursive target indices.

- A constructor target-index term containing the family being declared rejects
  during kernel inductive admission.
- The failure is observable even if an upstream producer bypasses the current
  elaborator guard.

AC2. Existing positivity behavior is unchanged.

- Strictly positive direct recursive constructor arguments still pass.
- K1.5 W-style recursive arguments still pass.
- Negative, nested, and mutual-family shapes that failed before still fail.

AC3. Ordinary indexed families still pass.

- A `Vec`-style declaration whose constructor target indices mention only
  parameters and constructor telescope binders admits.
- Constructor `target_indices` remain available for generated constructor types
  and eliminator method conclusions.

AC4. Scope stays kernel-mechanism bounded.

- Expected touched area is `crates/ken-kernel/` tests and implementation.
- No `packages`, `crates/ken-elaborator`, `Cargo.lock`, `spec`, or
  `conformance` movement is expected unless D0 routes and receives approval for
  a scope fork.

AC5. Review proves the authority map.

- Final handoff states exactly which target-index rule the kernel now enforces.
- Architect review confirms whether the rule is the existing conservative
  `occurs` guard applied to a skipped field, or a separately approved broader
  positivity rule.

## 5. Follow-On Surface Extension

After this kernel WP lands, Steward may release a small surface follow-on for
recursive target-index expressivity. That follow-on must be framed separately
and must name the exact kernel-admitted positive examples.

The follow-on may do one of two things, depending on the kernel result:

- If the kernel keeps the conservative D-free target-index rule, the follow-on
  is limited to removing duplicated surface authority where safe and proving
  that the kernel rejection remains the trusted gate.
- If Architect approves a broader kernel positivity rule, the follow-on may
  relax the current `SURF-gadt-elaboration` surface guard for exactly that
  kernel-admitted class, with focused positive and negative tests.

Do not let the surface accept recursive target-index shapes merely because the
parser can spell them. The kernel admission result is the authority.

## 6. Review Routing

Kernel implements and routes to Kernel QA. Architect review is mandatory before
integration.

Language QA is required only if the implementation changes
`crates/ken-elaborator/` or any surface behavior. Conformance-validator is
required only if `conformance/` changes.

## 7. Do Not Reopen

- Do not reopen `SURF-gadt-elaboration`; it already landed.
- Do not implement indexed coverage diagnostics or impossible-arm omission.
- Do not implement dependent match expansion.
- Do not add dependent-constructor syntax beyond what is already staged.
- Do not weaken nested-recursion rejection or W-style admission.
- Do not use `local/refs/`.
