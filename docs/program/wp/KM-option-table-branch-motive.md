# KM-option-table-branch-motive

**Owner:** Mechanism lane, initial owner Language for elaborator audit/build;
Kernel becomes co-owner only if D0 proves `crates/ken-kernel` is the rejecting
layer.
**Branch:** `wp/KM-option-table-branch-motive` when released.
**Status:** Steward frame. Queued; blocks CAT-4 option-table and Bool-dispatch
reflection routes.
**Size:** M. **Risk:** medium/high until D0 classifies whether the remaining
rejection is elaborator conversion or kernel checking.

## 0. Trigger

`KM-dependent-match-proof-motive-build` landed as PR #302 at
`origin/main @ 25e6c17`, fixing the original transparent-scrutinee proof-motive
sort failure. Its D3 CAT-4-shaped probe then classified the real downstream
state:

- the old whole-body `Type 0` vs `Omega0` proof-motive sorting failure is fixed;
- the CAT-4 option-table proof route still rejects;
- the remaining failure is a distinct mechanism:
  **non-nullary transparent-scrutinee option-table branch-motive conversion /
  constructor-argument specialization**.

The relevant Ken-owned events are:

- D3 candidate: `evt_74dhw371x73kr`;
- D3 QA approval/classification: `evt_4nz1veepkcrtx`;
- Architect approval of the classification: `evt_31wy19jttwd8h`;
- merge: PR #302 / `origin/main @ 25e6c17`;
- retros-in: `evt_14bm09xbe1z41`.

This WP exists because CAT-4 must not resume the option-table proof route merely
because the earlier proof-motive sort blocker is fixed.

After CAT-4 rebased onto `origin/main @ 9d653ac`, Architect classified the
first Bool-dispatch helper wall as the same mechanism family:

- runtime report: `evt_39qmms292ph14`;
- Architect classification: `evt_kf7m3fqhqa5`;
- clean parked CAT-4 head:
  `8e31534e60f593de8e1c05708e8048f38d03d917`.

The smaller reproducer is direct `Tree` recursion, not an option-table proof.
It still requires a proof-returning dependent match over a non-nullary
constructor to specialize an expected target mentioning transparent
`lookup`/`member` terms and constructor arguments introduced by the branch.
Include the failed `lookupNoneFromMemberFalse` helper and the explicit
bridge/transport variant from `evt_39qmms292ph14` in the D0/repro set.

## 1. Objective

Make the non-nullary transparent-scrutinee proof shape check when the branch
motive mentions constructor arguments introduced by the selected branch, or
prove a grounded no-fix ruling and name the smaller follow-on.

The target shapes are the D3 reduced option-table reconstruction and the later
direct lookup/member reflection helper, not the whole CAT-4
`catalog/packages/Data/Collections/Map.ken.md` proof:

- a transparent option-like scrutinee, e.g. `km_lookup b`;
- an option-table target whose expected result depends on the scrutinee and a
  reducible membership term;
- a non-nullary `Some x` branch;
- nested proof-returning matches whose branch targets require constructor
  argument specialization;
- a direct `Tree` recursion helper such as `lookupNoneFromMemberFalse`, whose
  expected proof target mentions transparent `lookup`/`member` terms under a
  non-nullary `Node` branch.

## 2. Scope

In scope:

- the D3 reduced probe from `KM-dependent-match-proof-motive-build`;
- branch-motive conversion after transparent scrutinee abstraction;
- constructor-argument substitution/specialization in branch expected types;
- direct lookup/member reflection helpers over non-nullary constructors,
  including `lookupNoneFromMemberFalse`;
- the explicit bridge/transport diagnostic variant from `evt_39qmms292ph14`;
- classification of whether the failure is elaborator-only or kernel-touching;
- focused executable positive/negative tests.

Out of scope:

- changing CAT-4 map/set semantics;
- proving `setIntersectionMemberLaw` directly in this WP, except as a final
  classification note about whether CAT-4 may resume;
- broad indexed-family, nested-matrix, or multi-scrutinee support unless the D0
  audit proves it is the same mechanism;
- using reference-derived guidance from the withdrawn CAT-4 route.

## 3. Required D0 Audit

Before implementation, post an audit that answers:

1. Does `check_match_dependent` build the correct motive after abstracting a
   transparent non-nullary scrutinee?
2. Are constructor arguments substituted into the branch-local expected type at
   the right de Bruijn depth?
3. Is the remaining mismatch caused by elaborator conversion/generalization,
   kernel `Term::Elim` checking, or a missing transport/J proof?
4. Does the current rejection remain fail-closed?

The audit must name exact files/functions touched or ruled out.

## 4. Acceptance Criteria

- **AC1 -- reduced option-table positive.** A focused reduced test preserving
  the D3 option-table shape checks after the fix.
- **AC2 -- direct reflection positive.** A focused reduced test preserving the
  direct `lookupNoneFromMemberFalse`/lookup-member reflection shape checks after
  the fix, or a no-fix ruling classifies why this smaller surface cannot be
  supported yet.
- **AC3 -- old blocker stays classified.** The test or review notes prove this
  is not the already-fixed `Type 0` vs `Omega0` proof-motive sort issue.
- **AC4 -- branch obligations remain exact.** A wrong-branch or
  wrong-constructor argument sibling still rejects through the intended type
  mismatch.
- **AC5 -- CAT-4 state explicit.** The WP states whether CAT-4 may resume the
  Bool-dispatch helper route after merge. If not, it names the next split.
- **AC6 -- TCB posture explicit.** If elaborator-only, `crates/ken-kernel`,
  `Cargo.lock`, and trusted-base output stay unchanged. If kernel-touching,
  route Kernel QA and Architect review before merge.
- **AC7 -- workspace green.** Focused tests and
  `scripts/ken-cargo test --workspace` pass.

## 5. Guardrails

- Build only from Ken-owned artifacts: landed spec/conformance, repo code,
  merged PR #302, and Convo event facts.
- Do not use or repeat the withdrawn reference-derived route in
  `evt_1ajnq8q62cv84`.
- Do not weaken proof checking with proof irrelevance before both branches check
  at exact specialized propositions.
- Do not change CAT-4 public laws or package semantics as a mechanism shortcut.

## 6. Downstream

Until this WP lands, CAT-4 remains parked at clean rebased head `8e31534` for
`setIntersectionMemberLaw`. Runtime should not keep probing alternative helper
spellings for that law. After the mechanism lands, Runtime resumes the
Bool-dispatch route with the helper family as the first validation slice, unless
Architect gives a new Ken-owned route.
