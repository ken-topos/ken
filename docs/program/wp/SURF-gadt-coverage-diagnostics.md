# SURF-gadt-coverage-diagnostics

## 1. Objective

Implement the next dependent-constructor build slice from
`spec/30-surface/34-data-match.md §4.3` and `§8`: make `match` over indexed
families use index-aware coverage and diagnostics.

This WP starts from the landed explicit-family slices:

- parser/AST: PR #317, `origin/main @
  8488af0f44158824daeae60f8408ab8615245c44`;
- elaboration/admission: PR #318, `origin/main @
  3e3e8f419da365b9b8f5f21f7bcff9e71b796855`.

The deliverable is the AC5/AC6 continuation for indexed families:

- type-possible constructors at the scrutinee index are required;
- index-impossible constructors may be omitted only when the elaborator can
  synthesize the corresponding `elim_D` method by absurdity;
- non-exhaustiveness diagnostics name the missing type-possible constructor or
  pattern witness;
- dependent-match motive recovery handles the indexed-family shape needed by
  the accepted cases.

This WP does **not** implement the later field-sugar ergonomics slice, record
labels in constructor signatures, named arguments, or the queued kernel
target-index positivity follow-up.

## 2. Spec Inputs

Fixed inputs:

- `spec/30-surface/34-data-match.md §4.1`
  - exhaustiveness is over constructors that are type-possible at the
    scrutinee type;
  - a missing type-possible constructor is a compile error that names the
    unmatched pattern witness.
- `spec/30-surface/34-data-match.md §4.3`
  - indexed families split constructors into type-possible and
    index-impossible at the scrutinee index;
  - type-possible means required;
  - index-impossible means omittable only if the elaborator synthesizes the
    constructor method by absurdity, leaving a total `elim_D` for the kernel.
- `spec/30-surface/34-data-match.md §4.4`
  - the coverage checker is untrusted surface logic, but safety is
    kernel-backed: a partial `elim_D` must not type-check.
- `spec/30-surface/34-data-match.md §8`
  - AC5 pins the non-degenerate pair: omitted impossible `VNil` arm accepts for
    `Vec A (suc n)` while applying that function to `VNil` rejects;
  - AC6 pins dependent motive / branch refinement shape for indexed matches.
- `conformance/surface/data-match/seed-data-match.md` AC5 and AC6
  - AC5 `indexed-impossible-pair`;
  - AC6 `branch-refinement-is-hypothesis`.

Current implementation state is perishable and must be verified against
`origin/main` at pickup. As of PR #318, explicit indexed families elaborate, but
indexed coverage, omitted impossible arms, and indexed dependent-match expansion
remain out of scope.

## 3. Required D0 Audit

Before implementation, post a D0 audit in the WP thread that answers:

1. Where does `infer_match` / `compile_match_matrix` currently reject or avoid
   indexed-family matching?
2. What exact scrutinee-index and constructor-target-index terms are available
   at the coverage decision point?
3. What index-discrimination cases can be decided by existing conversion /
   constructor disjointness today, without adding a kernel rule?
4. How will the implementation synthesize an omitted index-impossible
   constructor method so that the kernel receives a complete `elim_D`?
5. Does AC6 dependent-motive recovery require a separate smaller mechanism
   before AC5 can land?
6. Is the planned implementation confined to `crates/ken-elaborator/`, or does
   it require a kernel/conversion/mechanism fork?

If D0 shows that absurd-method synthesis or motive recovery requires a kernel
or conversion change, stop and route the scope fork to Steward and Architect
before implementing beyond minimized reproducer tests.

## 4. Deliverables

Implement on branch `wp/SURF-gadt-coverage-diagnostics`:

1. Add an index-aware coverage classifier for indexed-family matches.
   - Constructors whose target indices are type-possible at the scrutinee
     indices remain required.
   - Constructors whose target indices are provably impossible may be omitted.
   - Unknown / undecidable cases are treated as type-possible for safety.
2. Extend diagnostics so a missing type-possible indexed constructor reports a
   surface non-exhaustiveness error with the missing constructor or most-general
   pattern witness.
3. Synthesize complete `elim_D` methods for omitted index-impossible
   constructors by absurdity. Do not fabricate methods for unknown or
   type-possible constructors.
4. Recover the dependent motive shape needed for indexed-family matches in the
   accepted cases. The motive must mention the index/scrutinee where the result
   type requires it; do not silently fall back to a constant motive when a
   dependent motive is required.
5. Preserve existing non-indexed exhaustiveness, reachability, nested-pattern,
   and simple-data behavior.
6. Add focused tests for the AC5 pair and AC6 structural motive shape.

## 5. Acceptance Criteria

AC1. Indexed impossible omission accepts.

- A `Vec`-style family over `Nat` can define a `head`-style function whose
  scrutinee has type `Vec A (suc n)` and whose match omits `VNil`.
- The emitted core is a total `elim_Vec` with a synthesized method for `VNil`;
  it is not an under-applied eliminator and not an opaque shortcut.

AC2. Impossible application rejects.

- Applying the `head`-style function to `VNil : Vec A zero` rejects because the
  domain index `suc n` cannot unify with `zero`.
- This is the companion to AC1; both must be tested so the rule cannot drift
  into "everything required" or "everything accepted."

AC3. Type-possible omissions still reject with a named witness.

- Omitting a constructor whose target index can match the scrutinee index
  rejects as a surface non-exhaustiveness diagnostic.
- The diagnostic names the constructor or most-general missing pattern witness.
- Unknown index cases must be classified as required, not silently omitted.

AC4. Dependent motive shape is structural.

- The indexed accepted case emits an `elim_D` whose motive depends on the
  family index and/or scrutinee where required.
- A constant motive must not be accepted as the evidence for AC6 if it would
  erase the branch refinement that `34 §3.3` / `22 §3` require.

AC5. Non-indexed behavior is unchanged.

- Existing L2 AC1--AC4 tests remain green.
- Existing reachability behavior, including redundant-arm and guarded-arm
  subtleties, remains unchanged.
- Legacy simple-data declarations and explicit-family declarations from
  `SURF-gadt-elaboration` remain green.

AC6. Scope is bounded.

- Expected touched area is `crates/ken-elaborator/` implementation and tests.
- No `crates/ken-kernel`, `Cargo.lock`, `packages`, `spec`, or `conformance`
  movement is expected unless D0 routes and receives approval for a scope fork.
- `git diff --check` is clean.

AC7. Durable naming hygiene.

- Do not introduce WP/slice labels into implementation identifiers,
  diagnostics, or user-facing Ken fixture names. `SURF`, `gadt`, and AC labels
  may appear in Rust test names/comments and this WP doc, but implementation
  symbols and Ken source fixtures should use domain names such as `Vector`,
  `NonEmptyVector`, `IndexedFamily`, or `ImpossibleBranch`.

## 6. Review Routing

Language owns implementation and QA.

Architect review is mandatory before integration because this WP changes
surface totality checking and dependent eliminator emission for indexed
families. Kernel QA is not expected for the framed scope, but becomes mandatory
if any `crates/ken-kernel/` path changes or if D0 routes a kernel/conversion
mechanism fork.

Conformance-validator is not required unless the build changes `conformance/`
unexpectedly.

## 7. Do Not Reopen

- Do not implement `SURF-gadt-field-sugar` here.
- Do not add record-field labels inside explicit constructor signatures.
- Do not add named-argument application.
- Do not change kernel inductive admission or target-index positivity rules.
- Do not change `SURF-gadt-elaboration`'s same-family result-index guard.
- Do not weaken non-indexed exhaustiveness or reachability diagnostics.
- Do not accept unknown index-possibility cases by omission.
- Do not route CAT-5, catalog parsing, or package behavior through this WP.
- Do not use `local/refs/`.

## 8. Follow-On Queue

If this WP lands, the remaining data-match build WP is the ergonomics slice
`SURF-gadt-field-sugar`.

The separately queued `KM-target-index-positivity` mechanism remains
non-blocking for this WP unless D0 proves that indexed coverage depends on
kernel target-index admission rather than the landed surface guard.
