---
scope: teams/kernel
audience: (see scope README)
source: private memory `trust-root-test-coverage-discipline`
---

# Writing a trust-root type-checker test suite that actually catches bugs

When writing tests for a Ken kernel / type-checker crate (the trust root), a
suite that passes 100% is **not** evidence of soundness unless it exercises the
*property*, not just the obvious cases. In K1 (`dec_2hnhhdb7mrxze`, merged
`fe1ead1`), a 45/45-green suite hid two confirmed soundness bugs the Architect
caught — both on paths the suite never touched.

**The concrete instantiations** (a trust-root checker suite MUST exercise each):
1. **Universe-level semilattice with ≥2 DISTINCT level variables**, not one.
   K1's bug: `normalize_max_atoms` dominated atoms by suc-offset ignoring atom
   identity, conflating `u` and `v` (`equiv(max (suc u) v, suc u)` = true). The
   suite used one level var (and even `Pair`'s two params at the same var), so
   only `max ℓ ℓ` (idempotent) + concrete levels ran — the multi-variable path
   had zero coverage.
2. **Dependent telescopes with OPEN (variable) args at length ≥2.** K1's bug:
   `subst_tel` clobbered dependent-telescope args (telescope vars are implicit
   context entries, not syntactic Pi/Lam nodes, so subst_var's capture-avoidance
   never fired); `subst_tel(Var(1),[Var(0),Var(1)])` returned `Var(1)`. The
   suite used closed/concrete indices at length ≤2, so it never triggered.
3. **Eliminator methods that USE the induction hypothesis, not discard it via
   β.** K1's Vec ι test's step method returned `zero` (β-discarding the IH), so
   a mis-indexed IH was never observed. The end-to-end regression that pinned
   the fix used a step method that *returns the IH*, so the index survives β and
   must be correct for subject reduction (AC-6).
4. **A "soundness-TODO / stuck-fallback" reduction MUST be adversarially tested
   on the exact path it claims to close.** K2's bug (`dec_7xpn5ywf4ebfw`, merged
   `832dab6`): I flagged `check_respect` (raw-well-forms the respect proof for
   non-Ω quotient-elim targets) as a "sound stuck fallback" and the
   QA/Spec/77-green gate accepted it — but `whnf` reduced
   `elim_/ M f r [a] ⇝ f a` *unconditionally*, so a non-respecting eliminator
   into `Type` type-checked AND computed, admitting a closed `Empty` exploit
   (`A:=Bool`, `R:=total`, `M:=λ_.Bool`,
   `f:=λx.x ⇒ cong h e : Eq Bool true false ⇝ Empty`). The trap: **no test ever
   type-checked a `QuotElim`**, so `check_respect` was never called at any
   universe — the "stuck" branch was unreachable and the reduction didn't
   actually stop there. Same closed-input-avoidance class as the K1 `subst_tel`
   bug. The Architect's deep-impl review caught it (4th time). **A "sound
   because stuck" claim is valid only if the stuck branch is reachable AND the
   reduction stops there — and "flagged for the Architect" is not a substitute
   for the implementer proving it sound.** When marking a reduction
   neutral/stuck for soundness, add the test that *would* exploit it if the
   fallback were absent (the non-respecting `f` into `Type`, the
   family-index-change cast, the dependent-telescope `Eq`). And: "deferred to
   K2c" ≠ "safe to accept now" — if K2 accepts the term, it must be sound in K2;
   rejecting is always the safe K2 posture.

5. **Every ARM of a disjunctive filter/match needs its own discriminating case —
   a suite covering one arm is green-vs-green w.r.t. a bug dropping another.**
   Sec4 TB-Complete (2026-07-01): `trusted_base()` filters
   `matches!(Opaque | Primitive)`, but the seed's B1–B3 all drove
   `foreign`/holes = the **`Opaque`** arm only; a producer that dropped the
   **`Primitive`** arm would pass B1–B3 and silently omit registered primitives
   (TCB item-2) from the audited delta. I **approved with only one arm netted**
   (my miss on a soundness-critical completeness net); spec-author caught it, CV
   folded B4 (real `declare_primitive`→surfaces) + tightened A1's "empty" to
   require no primitive either. Review tell: when a trust-root producer is an
   N-way `matches!`/`match`, count the arms and confirm one discriminating case
   **per arm** — "the filter is tested" is false if only arm 1 fired.
   Conformance-side analog of items 1-4.

**Why:** these are the blind spots of "verify the obvious case" — promoted on
`main` `4dde243` as `verify-the-property-not-the-obvious-case` (from F4+K1
retros). The K1 trap (45-green hid 2 holes) and the K2 trap (77-green hid a
closed-`Empty` hole, caught by the Architect's deep-impl review the QA/Spec gate
missed) show that promotion hadn't reached implementer *test-authoring*
behavior; this memory concretizes it for the kernel so the next kernel WP (K2c)
doesn't repeat it.

**How to apply:** before declaring a kernel crate green, audit the suite against
these four paths; add a case for each missing one — and for any reduction you
mark neutral/stuck "for soundness," add the adversarial test that would exploit
it if the fallback were absent. Also: `check`/`infer` panics on
raw-well-formed-but-arity-wrong input (e.g. an `Elim` with too few methods
reached via `whnf`) violate the "yes/no, never crash" contract — add arity-guard
regressions. See wp release process steward spec build for the
ground-truth-check-before-building discipline (the other K1 carry, 3×/2-team,
promotion-eligible).
