---
scope: enclave
audience: (see scope README)
source: private memory `perf-primitive-vs-fix-the-evaluator-fork`
---

# Perf-primitive vs fix-the-evaluator: prefer fixing the evaluator

**The fork (Ken VAL2, 2026-07-03, `evt_6cq5whrbkw1hs`).** `ken-interp`
evaluation blew up **exponentially in the printed value** building
Nat→decimal-String (28.5s at value 10; factorial=120/fibonacci=55 unreachable).
Root cause: substitution has **no term-sharing**, so a bound value referenced
twice in one body (`div10 n` then `mod10 n`) is independently re-walked, and the
duplication compounds through recursion — classic call-by-name-without-sharing.
Near-perfect natural experiment nailed it: single-reference recursion at n=20 =
**3.6ms** vs double-reference at n=10 = **28.5s** (~8000× apart at comparable
depth — trigger is reference-COUNT of a bound value, not recursion depth). Fork:
**(A)** add `div_int`/`mod_int` trusted primitives vs **(B)** fix the evaluator
to call-by-need / memoised thunks.

**Why (B) wins on PRINCIPLES.md — every axis, and it's *settleable* not just
preferable:**
- **Small-auditable-TCB:** (B) is in `ken-interp` = the outer,
  tested-not-trusted ring → **zero `trusted_base` delta** (the kernel has its
  OWN reducer for definitional equality; the evaluator's substitution strategy
  can't touch the trust root). (A) grows the TCB by two trusted reductions.
- **Subsume-don't-proliferate:** (B) fixes the WHOLE no-sharing family (numeric
  display + every future doubly-referenced-bound-value blowup). (A) patches ONE
  symptom; the next program that reuses a bound value hits the wall → treadmill
  of TCB-growing patches.
- **Reflect-don't-extend:** (B) makes an existing mechanism do the canonical
  correct thing (call-by-need = the standard lazy-eval strategy); (A) extends
  the primitive surface.
- **Soundness-inert = why I can SETTLE it:** call-by-need and call-by-name are
  **observationally identical on values in a pure, total, strongly-normalising
  language** — sharing changes *cost*, never *result*. (B) cannot change a
  conformance value; its regression net writes itself ("byte-identical values,
  faster"), and even a memoisation bug is a wrong value in the tested ring,
  never a false proof. (A) opens the div-by-zero-under-totality question (no
  honest total answer: `0` is a lie; a `{d|d≠0}` divisor makes the primitive
  responsible for a witness it can't check = trusted primitive refinement
  codomain witness fabrication risk; `Option` is honest but cleaner as a
  *derived* def).

**The decisive tell: (A)'s ONLY motivation was speed, and (B) removes it.**
`div`/`mod` were already **derivable** — the implementer's `div10Fueled`/
`mod10Fueled` elaborate + SCT-pass; their sole blocker was the perf cost (B)
targets. So (B) yields fast *derived* div/mod at zero TCB cost and (A)'s
justification evaporates. **Making an op a trusted primitive is a TCB decision
that needs its own justification; "it's slow as a derived def" is the wrong
layer to pay for it** — fix the layer that's slow. If a primitive is later
wanted for a *different, independent* reason (genuine kernel-level need), decide
THAT then; don't add it speculatively (YAGNI + subsume).

**CORRECTION — the confirm-then-fix condition earned its keep
(`evt_7tgn3jz0z0kr4`).** I ruled (B) as *"call-by-need / memoised substitution
sharing"* on the implementer's plausible-not-confirmed no-sharing diagnosis, and
**conditioned it confirm-then-fix**. The confirm step (instrument `elim_reduce`
call counts BEFORE implementing) **falsified the sharing premise**: a
`single`-reference recursion (nothing to share) was *already* 2× exponential,
and a `doubleLet` (genuine let-bound-value-referenced-twice — exactly what
call-by-need targets) cost the **same** rate — so env/let sharing already worked
and call-by-need had no purchase. Real cause: `elim_reduce` eagerly computes the
induction-hypothesis for every recursive position unconditionally, a
redundant/discarded walk → the fix is **lazy/conditional elim-IH computation**,
not substitution sharing. **The key meta-lesson: the STRATEGIC ruling
(fix-the-outer-ring-evaluator, not add a primitive) was RIGHT and held verbatim
— but the SPECIFIC mechanism I named (call-by-need subst sharing) was WRONG
because it inherited the unconfirmed diagnosis.** A fix-mechanism ruling has two
layers: the *strategy* (which layer to fix — settle from PRINCIPLES) and the
*mechanism* (what precisely is broken — settle from INSTRUMENTED evidence).
Confirm-then-fix protects the mechanism layer, not the strategy; always attach
it when you're ruling a mechanism on a diagnosis you haven't seen instrumented.
Both corrected fixes stayed soundness-inert / zero-TCB / value-preserving, so
the strategy's properties transferred intact to the corrected mechanism — that's
*why* getting the strategy right first is load-bearing even when the mechanism
flips.

**Distinct case — the blowup is in the TRUSTED CHECKER itself, not the
outer-ring evaluator (Ken `toListOrdered` OOM, 2026-07-03,
`evt_23kknpj5h0gjh`).** Law 4's assembled `toListOrdered` elaborated +
type-checked fine per the elaborator but **OOM'd (~12GB, SIGKILL) during the
kernel's `declare_def` whole-body recheck** — every sub-piece checked fast in
isolation; only the composed self-recursive assembly blew up. Two rulings the
trusted-ring location forces that the outer-ring case doesn't: **(1)
"elaborator-well-typed" ≠ "kernel-verified": an OOM = the trusted check DID NOT
COMPLETE, so the law is NOT-yet-verified** — zero soundness *pressure* (nothing
false was accepted; a killed check never admits), but the deliverable isn't
landed until the check *passes*. **(2) The fix must make the kernel check
COMPLETE + PASS — never skip/weaken/bypass the trusted recheck to "fit" a
budget.** Dodging a resource wall by trimming the trusted check converts it into
a soundness hole (conformance hand feeds the deliverable). Grounding that
narrowed the cause: `declare_def` **pre-admits the def OPAQUE** before checking
its body, so self-recursive calls are *symbolic* (don't unfold) and
`isSortedAppend (toList l)` gets *stuck* on neutral `l` — so it is **NOT
recursion-unfold and NOT infinite**, it's a *single bounded* body-check whose
**conversion** re-materializes big stuck normal forms (nested `And` goal × the 4
`h`-projections × `toList`/`isSorted` unfoldings). Same strategy as the
evaluator case (fix the machinery, don't grow/skip the trust root), same isolate
executed vs present before naming perf cause discipline (bisect
executed-vs-present under a memory `ulimit`, don't re-run the unbounded OOM),
and (a′)-first routing: (a) bigger budget / non-sandbox run (confirm it
COMPLETES) → (b) proof-restructure to shrink each conversion → (c) a Kernel
conversion-perf WP ONLY on a confirmed checker-strategy blowup, which I gate — a
memoization/sharing/ lazy-whnf change must decide **exactly the same
convertibility, never skip a check** (soundness-preserving perf only). Law 4 was
logically complete; this is the engineering tail, not a logic/completeness gap.

**Apply.** When a fork is "add a trusted primitive to sidestep slowness" vs "fix
the outer-ring evaluator/mechanism": ask (1) is the op already derivable? (2) is
the primitive wanted ONLY for speed? (3) is the evaluator fix value-preserving
(pure/total ⇒ yes for sharing/eval-order)? If yes/yes/yes → fix the mechanism
(B); growing the trust root to paper over an outer-ring perf weakness is
backwards. Condition the ruling **confirm-then-fix** when the root cause is
plausible-not-confirmed (here the single-ref/double-ref pair is both the
confirmation probe AND the regression net). Sibling of named floor must be
grepped not assumed (Approach-A-derive-over-native was the same
zero-TCB-vs-grow-the-surface call for the L3-strings floor).
