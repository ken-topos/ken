# KM-dependent-match-proof-motive

**Owner:** Mechanism lane, initial owner Language for elaborator audit/build;
Kernel becomes co-owner if the fix touches `crates/ken-kernel`.
**Reviewer:** Architect mandatory. Kernel QA mandatory if any kernel path
changes. Language QA mandatory for an elaborator-only candidate.
**Branch:** `wp/KM-dependent-match-proof-motive`.
**Status:** Steward frame. Queued; not yet kicked off.
**Size:** M. **Risk:** medium/high until the audit classifies the TCB touch.

## 0. Trigger

CAT-4 G6 isolated a recurring dependent-match roadblock in Ken-owned code.

The triggering reproducer is runtime-implementer's
`intersectionLookupMemberCharacterization` probe at `evt_q1c395tkqd6k`, on
clean `runtime-implementer/CAT-4-G4G6-proofs @
db5ae3ead335766b670a960249fc576baca1eacf`.

The proof body is a nested dependent `match lookup ...` whose result type is an
`Equal (Option Unit) ...` target mentioning the scrutinee, or terms reducible
from it. Each branch can be closed using already checked Ken lemmas:

- `intersectionLookupLeftNoneLaw`;
- `intersectionLookupCharacterization`;
- `intersectionLookupSomeLaw`;
- `tt` into the option-table target.

But the whole proof body kernel-rejects with:

```text
KernelRejected { error: TypeMismatch { expected: Type 0, found: Omega0 } }
```

The span covers the entire proof body. That makes the failure a mechanism
boundary problem, not a CAT-4 proof-search problem.

## 1. Objective

Make proof-returning dependent `match` bodies check when the result motive is a
proof type, especially an `Equal ...` / `Prop` expression that mentions the
scrutinee or terms reducible from it, provided every branch supplies a term of
the correctly specialized proof target.

The deliverable is a mechanism fix, or a grounded no-fix ruling plus a smaller
mechanism WP split. It is not a CAT-4 semantic rewrite.

## 2. Scope

In scope:

- minimized Ken regression derived from `evt_q1c395tkqd6k`;
- surface elaborator motive construction for dependent `match`;
- Omega-vs-Type sort handling for proof motives;
- lowering/erasure path for `Equal` and proof-returning match bodies;
- kernel checking of `Term::Elim` / match motives if the audit shows the
  trusted checker is the rejecting layer;
- full classification of TCB impact.

Out of scope:

- changing CAT-4 map/set semantics;
- landing a CAT-4-specific workaround as the fix;
- using reference-derived guidance from the CAT-4 clean-room stop;
- broad indexed-family/GADT match support unless the minimized reproducer proves
  it is the same required mechanism.

## 3. Required Audit

Before implementation, the owner must identify the rejecting layer:

1. **Surface motive construction.** Does `infer_match` or the dependent-match
   checker build a motive whose sort is wrong for a proof-returning target?
2. **Proof-sort handling.** Is a proof motive being forced through `Type` when
   the intended target is `Omega` / `Prop`, or conversely being lowered as proof
   when the kernel expects a type former?
3. **`Equal` lowering.** Does the elaborator preserve the dependency of the
   `Equal ...` target on the branch scrutinee after branch refinement?
4. **Kernel elim checking.** Does `Term::Elim` reject a well-formed proof motive
   even when branch methods match the specialized motive?
5. **Backstop posture.** If the elaborator builds the wrong motive, does the
   kernel reject fail-closed? If not, this is a TCB soundness issue and must
   route through Kernel plus Architect before any build continues.

The audit result must name the exact files/functions touched or ruled out.

## 4. Regression Shape

Add a focused Ken test smaller than `catalog/packages/collections/map.ken` but preserving
the load-bearing shape:

- an option-like or two-constructor scrutinee;
- a dependent `match` whose return target is an `Equal ...` / proof expression;
- the target mentions the scrutinee or a term reducible from it;
- every branch closes locally using already checked lemmas or `tt`;
- the pre-fix behavior rejects at the whole-match boundary with the same
  Type-vs-Omega shape;
- the post-fix behavior checks without weakening the branch obligations.

The regression must be Ken-owned. It may be inspired by the CAT-4 reproducer,
but it must not depend on `local/refs/` or reference-derived proof strategy.

## 5. Acceptance Criteria

- **AC1 -- minimized red-to-green.** A reduced Ken reproducer with the shape in
  section 4 fails before the fix and checks after it. The test must assert the
  specific failure class before the fix, not a bare `is_err()`.
- **AC2 -- CAT-4 reproducer classified.** The exact
  `intersectionLookupMemberCharacterization` shape from `evt_q1c395tkqd6k` is
  re-run or mechanically reconstructed on the fixed branch. It either checks, or
  the remaining rejection is proven to be a distinct follow-on mechanism.
- **AC3 -- branch obligations remain real.** A negative sibling test where one
  branch supplies a proof of the wrong specialized target still rejects. The fix
  must not erase the dependency or accept all proof branches by proof
  irrelevance.
- **AC4 -- TCB impact explicit.** If the diff is elaborator-only, `crates/ken-
  kernel`, `Cargo.lock`, and `trusted_base()` are unchanged. If the diff touches
  kernel code, the PR states the trusted-surface delta and receives Architect +
  Kernel QA review.
- **AC5 -- workspace gate.** `scripts/ken-cargo test --workspace` passes. Run
  focused `ken-elaborator` tests for the minimized reproducer and the CAT-4
  reconstructed probe.
- **AC6 -- no CAT-4 semantic drift.** CAT-4 stays at clean `db5ae3e` or a later
  Runtime-owned proof commit. This WP must not rewrite maps/sets/relations except
  for an optional test fixture that imports the existing committed proof shape.

## 6. Guardrails

- Do not build from the reference-derived CAT-4 route withdrawn during the
  clean-room stop. The WP brief and implementation must use Ken-owned artifacts:
  repo code, `/spec`, `/conformance`, and Convo events.
- Do not weaken proof requirements by replacing an inhabited proof with a
  proposition surface, postulate, `Axiom`, primitive, or raw proof hole.
- Do not change CAT-4's public law shape as a workaround.
- Do not treat all Omega/Prop motives as proof-irrelevant unless the target type
  actually permits that by the existing kernel rules.
- Do not fold unrelated dependent-match work into this WP. If the minimized
  reproducer points at indexed-family support, W-style IH slots, or transport/J
  rather than proof-motive sorting, split that as a follow-on.

## 7. Sequencing

This WP is queued behind the active CAT-4 containment/guidance loop. It should
not interrupt the Runtime team or authorize further CAT-4 option-table work.

Release process:

1. Steward finalizes and commits this frame.
2. When a mechanism lane is available, run the normal handoff gate before
   kickoff.
3. Owner performs the audit and posts the classification before implementation.
4. Build proceeds under Architect review; add Kernel QA if the kernel is touched.

Downstream:

- CAT-4 G6 may resume from Architect's Ken-owned proof-shape ruling without this
  WP if a local proof route avoids the mechanism wall.
- If CAT-4 needs this mechanism to finish `setIntersectionMemberLaw`, keep the
  Runtime branch parked cleanly and sequence this WP before the final CAT-4 G6
  proof.
