# KM-index-impossible-branch-synthesis

**Owner:** Mechanism lane. Kernel owns D0 and any kernel/conversion build.
Language becomes co-owner only if D0 selects an Architect-approved
elaborator-local route.
**Reviewer:** Architect mandatory. Kernel QA mandatory for any kernel or
conversion change. Language QA mandatory for any elaborator behavior change.
**Branch:** `wp/KM-index-impossible-branch-synthesis`.
**Status:** Steward frame. Blocks `SURF-gadt-coverage-diagnostics` D1.
**Size:** M. **Risk:** high until D0 chooses the authority boundary.

## 0. Trigger

`SURF-gadt-coverage-diagnostics` stopped at D0 with no behavior-change
candidate on:

```text
wp/SURF-gadt-coverage-diagnostics @
4b558457e80ffd2637ca65e10a1739a72b124097
```

Language established that indexed coverage classification inputs are present in
the elaborator: the scrutinee type exposes `D p̄ ī`, and each constructor has
`ConstructorDecl.target_indices`.

The blocker is not classification. The blocker is method synthesis for omitted
index-impossible constructors in value-returning indexed matches. For a
`head`-style function:

```ken
head : Vec A (Suc n) -> A
```

the omitted `VNil` branch must still produce a method term for the kernel
eliminator, at a branch target like `M Zero VNil`. Current kernel method types
require that method term directly; they do not provide a branch-local
index-equality or impossibility premise. Existing `Term::Absurd` is scoped to
`Ω` motives and rejects `Type` motives.

Architect ruled at `evt_1ez4b96dqdnsk`:

- hold `SURF-gadt-coverage-diagnostics` D1;
- do not use an elaborator-local dummy-motive construction under the current
  frame;
- route a prerequisite mechanism WP that decides the exact proof-producing
  route.

## 1. Objective

Provide a sanctioned mechanism for omitted index-impossible branch synthesis in
value-returning indexed matches, or produce an Architect-approved no-build
decision that re-specifies the route.

The downstream contract is `34 §4.3` / AC5:

- type-possible constructors at the scrutinee index remain required;
- index-impossible constructors may be omitted only when the elaborator can
  synthesize the corresponding `elim_D` method by absurdity;
- the kernel still receives a total eliminator with one method per constructor;
- no method is fabricated for an unknown or type-possible constructor.

This WP does not implement indexed coverage itself. It unblocks the mechanism
that makes the omitted impossible method honest.

## 2. Grounding

Use these Ken-owned artifacts:

- Architect ruling: `evt_1ez4b96dqdnsk`;
- Language D0 stop: `evt_6w4475nx6cssc`;
- Language leader hold: `evt_hprzvp0y3aa`;
- blocked frame:
  `docs/program/wp/SURF-gadt-coverage-diagnostics.md`;
- `spec/30-surface/34-data-match.md §4.3`, `§4.4`, and `§8`;
- `conformance/surface/data-match/seed-data-match.md` AC5 and AC6;
- kernel eliminator method typing:
  `crates/ken-kernel/src/inductive.rs::method_type`;
- kernel eliminator checking:
  `crates/ken-kernel/src/check.rs` eliminator method checks;
- current `Absurd` rule:
  `crates/ken-kernel/src/check.rs::infer_absurd`;
- constructor-disjoint equality:
  `crates/ken-kernel/src/obs.rs::eq_at_inductive`.

Do not use `local/refs/`.

## 3. Required D0 Decision

D0 must post an exact route before implementation. It must choose one of these
families, or stop with a smaller named prerequisite:

1. **Type-level absurd route.**
   Extend the kernel's empty/absurd eliminator so an impossible proof can
   eliminate into `Type` motives where sound, and define the elaborator evidence
   construction that turns index-disjointness into the needed empty proof.
2. **Branch-local evidence route.**
   Change dependent eliminator / method typing so each constructor method gets
   usable branch-local index equality or impossibility evidence, then use that
   evidence to synthesize omitted impossible methods.
3. **Re-specification route.**
   Explicitly re-specify an elaborator-local dummy-motive construction, with
   Architect approval, if the project chooses that design instead of
   "by absurdity."

D0 must answer:

- What term proves the omitted branch impossible?
- What type does that proof inhabit?
- How is a method term of `M t̄_k (c_k ...)` obtained from it?
- Does the proof route work for value-returning `Type` motives, not only `Ω`?
- Does the route preserve proof irrelevance and kernel soundness?
- Which crates move?
- What exact downstream `SURF-gadt-coverage-diagnostics` AC5/AC6 shape becomes
  implementable after this lands?

If D0 selects route 3, do not implement it as a mechanism build without first
routing the spec/frame change through Steward and Architect.

## 4. Deliverables

The final deliverable depends on D0's selected route.

For route 1 or route 2, implement the minimal mechanism and focused tests:

1. Add the kernel/conversion/elaborator support needed to construct a sound
   omitted-method term for a proven index-impossible branch in a Type-valued
   motive.
2. Add a minimized Ken-owned regression for a `Vec`-like family where the
   `VNil` method target is Type-valued and the impossibility proof is available.
3. Add negatives proving that unknown index comparisons and type-possible
   constructors do not get fabricated methods.
4. Preserve existing `Absurd` / `J` / `cast` / eliminator behavior outside the
   selected route.
5. Leave `SURF-gadt-coverage-diagnostics` package behavior out of scope except
   for a minimized mechanism test if Language is explicitly co-owned by D0.

For route 3, produce the spec/frame change and stop before build unless
Architect explicitly approves implementation under this WP.

## 5. Acceptance Criteria

AC1. Authority boundary explicit.

- The final handoff states whether the mechanism is kernel, conversion,
  elaborator-local, or a routed spec reframe.
- Any kernel/conversion change receives Kernel QA and Architect review.

AC2. Type-valued impossible branch is constructible only from evidence.

- A minimized value-returning indexed impossible branch can produce the needed
  method term from a proof of impossibility.
- The mechanism is not limited to `Ω` motives if the downstream AC5 case needs
  `Type`.

AC3. Unknown and type-possible cases remain safe.

- Unknown index comparisons are not treated as impossible.
- Type-possible omitted constructors still require user branches in the
  downstream coverage WP.
- No arbitrary dummy inhabitant is accepted without the D0-approved evidence
  route.

AC4. Existing kernel behavior remains stable.

- Existing eliminator, `Absurd`, observational equality, and data/match tests
  continue to pass.
- Constructor disjointness and proof irrelevance are not weakened.

AC5. Downstream unblock is precise.

- The final review note says exactly how `SURF-gadt-coverage-diagnostics` may
  resume, and what AC5/AC6 tests should be run against the landed mechanism.
- `KM-target-index-positivity` remains separate unless D0 proves the same
  mechanism must also change target-index admission.

AC6. Workspace and scope are clean.

- `git diff --check` is clean.
- `scripts/ken-cargo test --workspace` passes, or any narrower gate is
  justified by Architect for a no-build/spec-only route.
- No package/catalog behavior changes ship in this WP.

## 6. Review Routing

Kernel owns D0. If D0 selects route 1 or route 2 with a kernel/conversion diff,
route:

```text
kernel-implementer -> kernel-qa -> Architect -> Integrator
```

If D0 selects an elaborator-only implementation after Architect approval, route
Language implementer and Language QA as co-owners before Architect review.

If D0 selects re-specification, stop and route the spec/frame change through
Steward before implementation.

## 7. Do Not Reopen

- Do not implement indexed coverage classification in this WP.
- Do not re-release `SURF-gadt-coverage-diagnostics` until this WP lands or an
  Architect-approved alternate route is recorded.
- Do not change `KM-target-index-positivity` here unless D0 proves a shared
  mechanism.
- Do not silently accept unknown index-impossibility cases.
- Do not introduce an arbitrary dummy branch inhabitant without an explicit
  approved re-specification.
- Do not use `local/refs/`.

## 8. Downstream

Until this WP lands or receives an Architect-approved alternate ruling,
`SURF-gadt-coverage-diagnostics` remains held at:

```text
wp/SURF-gadt-coverage-diagnostics @
4b558457e80ffd2637ca65e10a1739a72b124097
```

After this mechanism lands, Steward re-releases
`SURF-gadt-coverage-diagnostics` with the landed mechanism named explicitly in
the kickoff and tracker.
