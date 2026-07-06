# SURF-gadt-elaboration

## 1. Objective

Implement the second build slice for the dependent-constructor surface
described in `spec/30-surface/34-data-match.md §8`: lower explicit
`data ... : ... where { ... }` family declarations and constructor signatures
to real kernel inductive-family declarations.

This WP starts from the landed parser/AST slice:

- PR #317, `origin/main @
  8488af0f44158824daeae60f8408ab8615245c44`;
- final reviewed parser/AST head
  `82fba10733ab306b5b1df7cbfa19f0381f2987ff`.

The deliverable is elaboration/admission for explicit family declarations. It
does **not** implement indexed-pattern coverage, index-impossible arm omission,
or dependent `match` compilation over those families; that belongs to
`SURF-gadt-coverage-diagnostics`.

## 2. Spec Inputs

Fixed inputs:

- `spec/30-surface/34-data-match.md §2`
  - data-head binders before the colon are parameters;
  - the type after the colon is an index telescope ending in `Type`;
  - constructor signatures peel to a constructor telescope plus final
    `D params indices` target;
  - earlier constructor binders are in scope for later argument types and final
    result-index expressions;
  - bad constructor result targets reject before or while forming the kernel
    inductive declaration.
- `spec/30-surface/39-elaboration.md §2.2`
  - explicit family declarations elaborate to kernel inductive-family
    declarations;
  - constructor codomains must expose the declared family head with declared
    parameters in order and the declared index arity;
  - the kernel remains responsible for constructor type checking, universe
    checks, strict positivity, W-style admission, nested/mutual rejection, and
    eliminator generation.
- `spec/10-kernel/14-inductive.md §1`, `§2`, and `§8`
  - kernel inductive families already support parameters, indices, constructor
    telescopes, eliminators, strict positivity, and W-style recursive
    occurrences;
  - negative, nested, and mutual-family shapes remain kernel rejections.
- `conformance/surface/data-match/seed-data-match.md` AC9
  - positive `Vec`/proof-carrying declarations;
  - bad result-target rejection;
  - positivity rejection through the kernel gate.

Current implementation state is perishable and must be verified against
`origin/main` at pickup. As of this frame, explicit family declarations parse
but are still staged fail-closed before elaboration.

## 3. Deliverables

Implement on branch `wp/SURF-gadt-elaboration`:

1. Lower explicit family AST declarations to the kernel inductive-family
   declaration path rather than rejecting them in resolution/elaboration.
2. Elaborate data-head parameter binders and the family result/index telescope:
   - `data Box (A : Type) : Type where { ... }` has zero indices;
   - `data Vec (A : Type) : Nat -> Type where { ... }` has one index;
   - named index binders such as `(n : Nat) -> Type` preserve binder scope
     where the current elaborator representation supports it.
3. Elaborate explicit constructor signatures:
   - peel explicit binders, implicit binders, and anonymous arrows into the
     constructor telescope;
   - make data-head parameters available throughout constructor signatures;
   - make earlier constructor binders available to later argument types and
     result-index expressions.
4. Validate constructor result targets before kernel admission:
   - exposed result head must be the declared family;
   - data-head parameters must match the declared parameters in order;
   - index arity must match the family index telescope;
   - non-family, wrong-family, wrong-parameter, undersaturated, and
     oversaturated targets reject with a surface diagnostic naming the
     constructor and expected family head.
5. Route recursive-shape admission through the kernel:
   - strictly positive direct and W-style shapes may admit if the kernel admits
     them;
   - negative, nested, or mutual-family shapes reject from the kernel admission
     verdict and surface at the declaration/constructor span.
6. Preserve existing simple `data D a = C A | ...` behavior.

## 4. Acceptance Criteria

AC1. Non-indexed explicit family elaborates.

- A declaration equivalent to:

  ```ken
  data Box (A : Type) : Type where {
    Mk : A -> Box A
  }
  ```

  parses, elaborates, and kernel-checks as a real inductive family.
- The constructor is usable through the same constructor/introduction path as
  legacy simple data, not as an opaque postulate or smart constructor.

AC2. Indexed `Vec` declaration elaborates.

- A declaration equivalent to:

  ```ken
  data Vec (A : Type) : Nat -> Type where {
    VNil  : Vec A zero;
    VCons : (n : Nat) -> A -> Vec A n -> Vec A (suc n)
  }
  ```

  elaborates to a kernel inductive family with parameter `A`, index `Nat`, and
  constructor targets at distinct index instances.
- Existing kernel positivity and universe checks re-check the emitted
  declaration.

AC3. Proof-carrying constructor telescope elaborates.

- A checked-source-style non-indexed declaration with a constructor telescope
  containing computational fields followed by proof fields elaborates and
  kernel-checks without adding CAT-5-specific behavior.
- Earlier constructor binders are in scope for later proof-field types.

AC4. Bad result-target diagnostics.

Focused negatives reject with surface diagnostics before any successful
declaration is installed:

- constructor target has the wrong family head;
- constructor target changes a data-head parameter;
- constructor target has too few or too many indices;
- constructor signature ends in a non-family result.

The diagnostic should name the constructor and expected family head. It may
also include the offending span if that is available in the current diagnostic
surface.

AC5. Kernel admission remains the positivity authority.

- A strictly positive explicit family accepted by the kernel is accepted by the
  elaborator.
- A negative recursive occurrence rejects through the kernel admission verdict;
  the elaborator must not implement a parallel positivity checker.
- Nested or mutual-family shapes remain rejected if the landed kernel rejects
  them.

AC6. Existing behavior and scope remain stable.

- Legacy simple `data` declarations and existing L2 data/match tests continue
  to pass.
- Explicit families still do not imply indexed coverage support or
  index-impossible arm omission in `match`.
- `git diff --check` is clean.
- Expected touched area is `crates/ken-elaborator/` tests and implementation.
  No `crates/ken-kernel`, `Cargo.lock`, `packages`, `spec`, or `conformance`
  movement is expected unless the team first routes a scope fork to Steward and
  Architect.

AC7. Durable naming hygiene.

- Do not introduce WP/slice labels into implementation identifiers,
  diagnostics, or user-facing fixture names. `SURF`, `gadt`, and AC labels may
  appear in Rust test names/comments and this WP doc, but implementation
  symbols and Ken source fixtures should use domain names such as
  `ExplicitData`, `IndexedFamily`, `Vector`, or `CheckedSource`.

## 5. Review Routing

Language owns implementation and QA.

Architect review is required before integration because this WP changes
surface-to-kernel elaboration for inductive-family declarations and depends on
the kernel admission boundary. Kernel QA is not required for the expected scope
because the kernel is not supposed to move; if a kernel diff becomes necessary,
stop and route the scope fork to Steward and Architect before continuing.

Conformance-validator is not required unless the build changes `conformance/`
unexpectedly.

## 6. Do Not Reopen

- Do not implement `SURF-gadt-coverage-diagnostics` here.
- Do not implement indexed-pattern coverage, omitted index-impossible arms, or
  absurd method synthesis.
- Do not implement dependent `match` motive recovery beyond what is already
  landed.
- Do not add record-field labels inside explicit constructor signatures.
- Do not add named-argument application.
- Do not change kernel inductive admission or positivity rules.
- Do not implement CAT-5 or route CAT-5 behavior through this feature.
- Do not weaken the legacy-form boundary by accepting `data D = C : ...`.
- Do not carry operational WP names into implementation/source identifiers.

## 7. Follow-On Queue

If this WP lands, the next queued data-match build WP is
`SURF-gadt-coverage-diagnostics`, followed by the later ergonomics slice
`SURF-gadt-field-sugar`.
