# KM-dependent-match-proof-motive-build

**Owner:** Language team initially. Kernel becomes co-owner only if the audit
proves a `crates/ken-kernel` change is required.
**Branch:** `wp/KM-dependent-match-proof-motive-build`.
**Base:** `origin/main @ 0527467` or later.
**Status:** Ready after the Language retro/compaction handoff gate.
**Size:** M. **Risk:** medium/high until D0 classifies the rejecting layer.

## 0. Inputs

This is the build lane for the merged spec/conformance WP
`KM-dependent-match-proof-motive`:

- merged commit: `origin/main @ 0527467`;
- WP brief: `docs/program/wp/KM-dependent-match-proof-motive.md`;
- spec rule: `spec/30-surface/34-data-match.md` and
  `spec/30-surface/39-elaboration.md`;
- conformance seed: AC8 in
  `conformance/surface/data-match/seed-data-match.md`;
- triggering CAT-4 reproducer: `evt_q1c395tkqd6k` on clean
  `runtime-implementer/CAT-4-G4G6-proofs @ db5ae3e`.

Build from these Ken-owned artifacts only. Do not use the withdrawn
reference-derived CAT-4 route in `evt_1ajnq8q62cv84`.

## 1. Objective

Make proof-returning dependent `match` check when the recovered motive has an
Omega-codomain proof target, especially an `Equal ...` target that mentions the
scrutinee or a transparent/reducible term of the scrutinee.

The fix must preserve exact branch obligations. A branch that supplies a proof
of the wrong specialized target must still reject.

## 2. Deliverables

### D0 -- audit and classification

Before changing behavior, post a short audit in the WP thread naming the
rejecting layer and the files/functions touched or ruled out:

- surface motive construction in `infer_match` / dependent-match checking;
- proof-sort handling for Type-vs-Omega motives;
- transparent `Equal` / kernel `Eq` lowering under branch refinement;
- kernel `Term::Elim` motive checking;
- fail-closed posture if the elaborator builds the wrong motive.

If the audit shows an elaborator-only fix, keep the implementation in
`crates/ken-elaborator` and tests. If it shows a kernel bug, stop and route the
Kernel co-owner/review lane before implementing.

### D1 -- minimized red-to-green

Turn the landed AC8 positive seed into a focused executable regression. It must
preserve the load-bearing shape:

- two-constructor or option-like scrutinee;
- dependent `match`;
- target in Omega, spelled through `Equal ...`;
- target mentions the scrutinee through the transparent `km_scrutinee` head or
  an equivalent Ken-owned transparent term;
- branch methods close locally with `tt` or already checked primitives.

The pre-fix failure class must be asserted as the whole-body
Type-vs-Omega rejection, not a generic error.

### D2 -- negative sibling remains rejecting

Add the wrong-specialized-branch sibling from AC8 as an executable negative
test. The false branch must still reject because its specialized codomain is the
wrong proof target, not because of an unrelated parser or name-resolution error.

### D3 -- CAT-4 reproducer classification

Re-run or mechanically reconstruct the CAT-4
`intersectionLookupMemberCharacterization` shape from `evt_q1c395tkqd6k` against
the fixed branch.

Record one of two outcomes:

- it now checks, so CAT-4 may resume the option-table proof route; or
- it still rejects for a distinct mechanism, naming the follow-on split.

Do not edit the CAT-4 branch from this WP except for an optional isolated test
fixture. Runtime resumes CAT-4 after this WP merges or after Architect gives a
separate Ken-owned local proof route.

### D4 -- workspace and review gate

Run focused `ken-elaborator` tests plus:

```bash
source scripts/ken-env.sh && scripts/ken-cargo test --workspace
```

Merge review requirements:

- Architect always reviews.
- Language QA reviews an elaborator-only candidate.
- Kernel QA also reviews if `crates/ken-kernel` or trusted checker behavior
  changes.

## 3. Acceptance Criteria

- **AC1 -- positive flips.** The AC8 positive proof-returning dependent match
  fails before the fix with the intended Type-vs-Omega class and checks after
  the fix.
- **AC2 -- negative holds.** The wrong-specialized-branch sibling still rejects
  for the intended branch-obligation reason.
- **AC3 -- exact mechanism classified.** The D0 audit names whether the fix is
  elaborator-only or kernel-touching, and the merge Decision repeats that
  classification.
- **AC4 -- CAT-4 state is explicit.** The CAT-4 reproducer is classified as
  unblocked by this fix or split to a named follow-on mechanism.
- **AC5 -- TCB posture is explicit.** If elaborator-only, `crates/ken-kernel`,
  `Cargo.lock`, and trusted-base output are unchanged. If kernel-touching, the
  trusted-surface delta is stated and Kernel QA is in the gate.
- **AC6 -- workspace green.** Focused tests and `scripts/ken-cargo
  test --workspace` pass.

## 4. Guardrails

- Do not broaden this into general indexed-family, nested-matrix, or
  multi-scrutinee recovery unless the minimized seed proves it is the same
  mechanism. Split follow-ons instead.
- Do not weaken branch checking with proof irrelevance before both branches have
  checked at their exact specialized propositions.
- Do not change CAT-4 map/set semantics or public law shapes in this WP.
- Do not introduce `Axiom`, postulates, proof holes, or proposition-only
  surfaces to bypass a proof obligation.
- Do not route reference-material guidance to build roles. Build from merged
  Ken spec, conformance, repo code, and Convo event facts only.

## 5. Downstream

Once this WP merges and retros are in, Steward should relay the result to
Runtime. If D3 says the CAT-4 reproducer is unblocked, Runtime resumes from
clean `runtime-implementer/CAT-4-G4G6-proofs @ db5ae3e` or a later
Runtime-owned proof checkpoint. If D3 names a follow-on mechanism, Steward
frames that split before CAT-4 continues down the blocked route.
