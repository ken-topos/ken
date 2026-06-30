# WP X1-effects-elab — Effect evaluation in the reference interpreter

> **Status:** Steward frame — **next enclave WP** (deps met: L5 + X1 both on
> `main`). spec-leader elaborates `spec/40-runtime/42-evaluation.md §3` (Effects)
> from DRAFT → implementation-ready, then **Team Runtime** builds X1's effect
> evaluation (replacing the currently-stuck effect forms).
>
> **Team:** Runtime (first activation — idle the whole program, dep-blocked on
> the enclave) · **Deps:** **L5** (`36`/effects: rows + **ITree denotation** +
> handlers + `space`/message-passing — the semantic basis) + **X1** (the base
> interpreter, G1) · **Size:** M · **Risk:** ★★ (X1 is the **reference oracle** —
> not in the TCB, but it *defines* the meaning of evaluation; everything
> downstream is judged by agreement with it, so reference-correctness is
> load-bearing) · ► Activates **WS-X / Runtime**; the oracle for effectful programs.

## Objective

Elaborate `42 §3` (Effects) to builder rigor and have Runtime implement it: give
**effectful** core terms their operational meaning in X1. Today X1 evaluates the
pure core (G1) but leaves **effect operations stuck** (out-of-scope deferrals);
this WP specifies and builds how X1 **performs** them — grounded in **L5's
interaction-tree denotation** (the effect semantics already landed), so X1's
operational story and L5's denotational one **agree**.

## The framing that sets the risk level

X1 is the **reference interpreter** — the oracle a native backend (X3) and all of
WS-X are judged against (`00 §3`). It is **not** trusted by the kernel (the kernel
re-checks *types*, not evaluation), so an X1 effect bug is not unsoundness — but
it *is* a wrong **definition of meaning**, so reference-correctness is the
load-bearing property. The specific obligations: **effect ordering/sequencing is
exactly the discipline `36` imposes** (effects occur in the order the effect
discipline mandates — no reordering of observable interactions); **the effect row
bounds which effects can occur** (an effect outside the declared row is a
type-level impossibility, not a runtime surprise); and **X1's operational result
agrees with L5's ITree denotation** (the same program denotes the same
interaction tree — the two semantics are reconciled, not parallel).

## Scope

**IN:** the operational rule for **performing a primitive effect** (`FS`, `Net`,
`Clock`, `Console`, `Rand`, …) — when evaluated, it performs its world-interaction
in the effect-discipline order; **effect sequencing** (the ordering of effectful
sub-evaluations + `space`/cell operations + message-passing per `36`); the
**handler** semantics (tail-resumptive handlers from L5 — how a handled effect
resumes); the **row-bounding** at evaluation (the declared effect row gates what
can perform); the **reconciliation with L5's ITree denotation** (X1's steps
realize the same interaction tree). The conformance oracle for effectful programs.

**OUT — other WPs:** the **type-level** effect machinery (`36`/L5, done — rows,
inference, denotation); native codegen (X3); the kernel (effects are outside the
TCB); capability *granting* policy (L5/Sec — X1 *performs* under granted caps,
doesn't decide policy).

## The elaboration this needs (spec-leader → spec-author + Architect)

Elaborate `42 §3` to builder rigor: each primitive-effect class's operational
rule (perform → observe → resume); the **sequencing/ordering** discipline stated
normatively (what is determined vs. implementation latitude — `42`'s header notes
internal strategy is latitude *so long as results agree*); the handler-resume
semantics; the **X1↔L5 agreement** stated as the reconciliation obligation.
**Ground against the *landed* L5 (the ITree denotation + `36`) and the landed X1
interpreter — the files, not status.** Conformance (`conformance/runtime/
effects/`): an effectful program's observable interaction sequence is
**discriminating** (correct order accepts / a reordered or dropped interaction is
a distinct, detectable result); a row-violating effect is rejected; handled vs.
unhandled effects resolve correctly; **X1 result == L5 ITree** on a shared corpus.
Determinism of the pure fragment is preserved (`42 §2`).

## Acceptance (testable)

1. **Effects perform + sequence:** each primitive effect performs its interaction;
   a multi-effect program's observable order is exactly `36`'s discipline (a
   reordering is a different, detectable trace).
2. **Row-bounding at eval:** an effect outside the declared row cannot perform
   (caught — type-level, not a runtime surprise).
3. **Handlers resume:** a tail-resumptive handler intercepts + resumes per L5.
4. **X1 ↔ L5 agreement:** on a shared corpus, X1's operational result realizes the
   same interaction tree L5 denotes (the two semantics reconciled, asserted).
5. **No regression:** pure (effect-free) evaluation is unchanged + deterministic
   (`42 §2`); G1's reference behavior holds.

## Sequencing

Next enclave WP after V2 (deps L5 + X1 landed). **Activates Team Runtime** (idle
the whole program). Parallel to the verification spine (Verify on V2-build →
V3-build); the enclave does this, then **V3** (prover). Unblocks WS-X downstream
(X3 native backend judged against this oracle). Build queries: effect semantics →
Spec; interpreter design → Architect. Clean-room: L5 + landed X1 + `36`/`42`, no
copyleft.
