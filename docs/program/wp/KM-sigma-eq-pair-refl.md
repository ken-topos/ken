# KM-sigma-eq-pair-refl

**Owner:** Mechanism lane, initial owner Language for elaborator/conversion
audit and build. Kernel becomes co-owner only if D0 proves the trusted checker
or kernel conversion is the rejecting layer.
**Reviewer:** Architect mandatory. Language QA mandatory for an
elaborator-only candidate; Kernel QA mandatory if any kernel path changes.
**Branch:** `wp/KM-sigma-eq-pair-refl`.
**Status:** Steward frame. Blocks CAT-3 D3.
**Size:** M. **Risk:** medium/high until D0 classifies whether the fix is
elaborator-only or TCB-touching.

## 0. Trigger

CAT-3 D3 is blocked at leader review on
`wp/CAT-3-build @ b007e0598d0d6bfc3cb4fda841d9d549ab569770`
(`b007e05`, `CAT-3 D3 projection records`).

The exact blocker is in the concrete first-component lens over
`Pair Bool Bool`. The merged CAT-3 spec and build wrapper require the public
coherence laws as full pair equalities:

```ken
set-get : (s : Pair Bool Bool) ->
  Equal (Pair Bool Bool) (set s (get s)) s

set-set : (s : Pair Bool Bool) -> (b c : Bool) ->
  Equal (Pair Bool Bool) (set (set s b) c) (set s c)
```

The D3 candidate instead changes the class fields and exported concrete laws to
componentwise `And` proofs over `pairFst` / `pairSnd`. That is a public
law-shape weakening, not an internal helper.

Architect ruled at `evt_491wqvjz2mnd2` that the pinned contract stands:
`b007e05` must not proceed to QA with componentwise `And` as the public
replacement for `set-get` / `set-set`.

## 1. Objective

Make the accepted CAT-3 D3 full-pair lens laws build without weakening their
public type, or prove a grounded no-fix ruling and name the smaller follow-on.

The desired public surface remains:

- `Lens` fields for `set_get` and `set_set` returning
  `Equal (Pair Bool Bool) ...`, not componentwise `And`;
- exported concrete laws `fstLensSetGet` and `fstLensSetSet` returning the same
  full pair equality;
- componentwise lemmas may remain only as internal helpers if the final proof
  route uses them.

This WP is a mechanism fix, not a CAT-3 rescope.

## 2. Grounding

Use only Ken-owned artifacts:

- landed CAT-3 spec: `spec/50-stdlib/57-collections-and-views.md:350-367`;
- active CAT-3 build wrapper on `wp/CAT-3-build`:
  `docs/program/wp/CAT-3-build.md:104-115`;
- D3 candidate: `wp/CAT-3-build @ b007e05`;
- language-leader blocker: `evt_7sdjz0s99k3g6`;
- Architect ruling: `evt_491wqvjz2mnd2`.

Architect reproduced the smaller mechanism issue on exact `b007e05`:

- direct full-shape `Refl` probes for `set-get` and `set-set` fail with
  `Refl expects an Eq-shaped goal`;
- a smaller identity repro also fails:

```ken
fn pair_refl_fn (s : Pair Bool Bool) : Equal (Pair Bool Bool) s s =
  Refl
```

- the component proof cannot currently be used at the full equality type
  either: a proof term with the componentwise type is rejected when checked
  against `Equal (Pair Bool Bool) ...`.

The likely boundary, per Architect, is that `Pair` is the prelude transparent
Sigma alias and kernel conversion has Sigma eta, but the current checking path
reduces `Eq` at a Sigma carrier to a Sigma/component-equality proposition before
`Refl` can check the original Eq-shaped goal. The current checker/conversion
path also does not accept the component proof term against the unreduced full
`Equal (Pair ...)` surface.

## 3. Required D0 Audit

Before implementation, post an audit in the CAT-3 thread that answers:

1. Does the rejection occur before kernel checking, while elaborating/checking
   `Refl` against an `Equal (Pair Bool Bool) ...` target?
2. Does whnf/conversion reduce the equality proposition at Sigma carrier before
   the `Refl` introduction rule sees the original Eq-shaped goal?
3. Can the elaborator preserve the original `Eq A x y` introduction target for
   `Refl` while still using conversion to prove endpoints equal?
4. Alternatively, can a component Sigma/`And` proof soundly inhabit the full
   `Equal (Pair Bool Bool) ...` surface through existing conversion/checking,
   without making all component proofs coercions into equality?
5. Is the fix elaborator-only, or does it require kernel conversion/checking?
6. Does the current rejection remain fail-closed?

The audit must name the exact files/functions touched or ruled out.

## 4. Accepted Fix Families

The implementation must choose one sound route and justify it:

- **Eq-shaped `Refl` route.** `Refl` checks against the original
  `Eq A x y` / `Equal A x y` goal when endpoints convert, even if whnf of the
  proposition would observationally reduce the Sigma equality to component
  obligations.
- **Component-to-full route.** A component Sigma/`And` proof can inhabit the
  full `Equal (Pair Bool Bool) ...` goal through a sound, explicit conversion or
  checking rule.

Do not implement both unless the D0 audit proves they are the same local change.

## 5. Regression Shape

Add focused Ken-owned red-to-green tests. Keep them smaller than the full CAT-3
package candidate.

Required positives:

- `pair_refl_fn`:

```ken
fn pair_refl_fn (s : Pair Bool Bool) : Equal (Pair Bool Bool) s s =
  Refl
```

- full first-component lens `set-get` over `Pair Bool Bool`;
- full first-component lens `set-set` over `Pair Bool Bool`;
- a reconstructed CAT-3 D3 candidate check proving that the public
  `Lens.set_get`, `Lens.set_set`, `fstLensSetGet`, and `fstLensSetSet`
  surfaces return full pair equalities.

Required negatives:

- the existing wrong-endpoint CAT-3 lens negative still rejects;
- a componentwise `And` proof is not silently accepted as an arbitrary full
  equality unless the chosen fix family explicitly and soundly supports that
  exact conversion;
- wrong endpoints for `set-get` or `set-set` still reject at the intended type
  mismatch.

## 6. Acceptance Criteria

- **AC1 -- minimized red-to-green.** The `pair_refl_fn` reproducer fails before
  the fix with the same `Refl expects an Eq-shaped goal` class and checks after
  the fix.
- **AC2 -- CAT-3 lens positives.** Full pair-equality `set-get` and `set-set`
  declarations over the concrete first-component `Pair Bool Bool` lens check.
- **AC3 -- CAT-3 public surface restored.** On `wp/CAT-3-build` rebased after
  this mechanism, the public `Lens` fields and exported concrete laws are full
  `Equal (Pair Bool Bool) ...` surfaces. Componentwise helpers, if any, are not
  counted as the public law.
- **AC4 -- negative precision.** The wrong-endpoint negative still rejects, and
  the fix does not allow arbitrary component proofs to inhabit unrelated full
  equality goals.
- **AC5 -- TCB posture explicit.** If elaborator-only, `crates/ken-kernel`,
  `Cargo.lock`, and trusted-base output remain unchanged. If kernel-touching,
  route Kernel QA and Architect review before merge.
- **AC6 -- workspace green.** Focused tests and
  `scripts/ken-cargo test --workspace` pass.
- **AC7 -- CAT-3 state explicit.** The final review note states whether CAT-3
  D3 may resume from `b007e05` by restoring the full pair laws, or whether a
  smaller follow-on remains.

## 7. Guardrails

- Do not weaken CAT-3's public lens law shape to componentwise `And`.
- Do not add an `Axiom`, postulate, primitive, opaque shortcut, or proposition
  wrapper for the lens laws.
- Do not change CAT-3 package semantics or continue widening D3 while this
  mechanism is unresolved.
- Do not make all `And` component proofs coerce to full equality. Any
  component-to-full rule must be justified by the Sigma equality structure it
  targets.
- Do not use `local/refs/` or reference-derived proof routes.

## 8. Downstream

Until this WP lands or receives an Architect-approved no-fix split, CAT-3 D3
remains held at `b007e05` and must not go to Language QA.

After this WP lands, Language resumes CAT-3 D3 by restoring the full pair
equality public surfaces and rerunning D3 review/QA. The componentwise candidate
may be amended in place only after the mechanism is available and only if the
public contract is restored.
