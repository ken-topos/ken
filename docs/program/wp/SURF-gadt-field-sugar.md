# SURF-gadt-field-sugar

## 1. Objective

Implement the remaining dependent-constructor ergonomics slice named in the
data-match queue: bounded named-field sugar for constructor declarations and
uses, lowering to the already-landed positional constructor telescope.

This WP starts from the landed GADT surface build chain:

- `SURF-gadt-parser-ast`: PR #317, `origin/main @
  8488af0f44158824daeae60f8408ab8615245c44`;
- `SURF-gadt-elaboration`: PR #318, `origin/main @
  3e3e8f419da365b9b8f5f21f7bcff9e71b796855`;
- `SURF-gadt-coverage-diagnostics`: PR #320, `origin/main @
  127066d5b4e0926478a229490ac505f2469e590d`.

The deliverable is ergonomic surface spelling only. It must not add new kernel
expressivity, new inductive admission rules, new match coverage semantics, or a
general record-system redesign.

## 2. Spec Inputs

Fixed inputs:

- `spec/30-surface/32-grammar.md §1`
  - `ctor ::= ConId arg_types?`;
  - `arg_types ::= type+ | "{" field ("," field)* "}"`;
  - constructor arguments may be positional or named-record style.
- `spec/30-surface/32-grammar.md §3--§4`
  - record literals and record patterns already exist as surface constructs;
  - constructor patterns are `ConId pattern*`.
- `spec/30-surface/34-data-match.md §1`
  - `C A B` and `C { f : A, g : B }` both elaborate to the same constructor
    telescope `(Delta_k) -> D Delta_p tbar_k`;
  - the sugar is not a new kernel construct.
- `spec/30-surface/34-data-match.md §3`
  - constructor patterns continue to drive `elim_D`;
  - record patterns project negative records and are not themselves data
    eliminators.

Current implementation state is perishable and must be verified against
`origin/main` at pickup.

## 3. Required D0 Audit

Before implementation, post a D0 audit in the WP thread that answers:

1. Which of the spec spellings are already parsed for simple-data constructor
   declarations, explicit-family constructor signatures, constructor
   expressions, and constructor patterns?
2. Which AST representation can carry field labels without changing kernel
   constructor declarations or generated eliminator signatures?
3. Can constructor declaration field labels be preserved far enough to validate
   constructor expressions and patterns, or is declaration-only sugar the
   bounded slice for this WP?
4. If named constructor expressions or patterns are implemented, what exact
   syntax is accepted, and how are duplicate, missing, unknown, and reordered
   fields handled?
5. Does any part of the planned work require `crates/ken-kernel`, `spec`,
   `conformance`, `packages`, or `Cargo.lock` movement?

If D0 shows that expression/pattern labels require a broad record-field
mechanism, parser redesign, or kernel admission change, stop and route the
smaller declaration-only option or a scope fork to Steward and Architect before
implementing beyond minimized tests.

## 4. Deliverables

Implement on branch `wp/SURF-gadt-field-sugar`:

1. Support named-record constructor argument spelling where D0 proves it is
   bounded. At minimum, this should cover constructor declarations if the
   current surface can carry labels there without changing kernel semantics.
2. Lower named fields to the existing positional constructor telescope in
   declaration order.
3. Preserve field-label metadata only in the untrusted surface layer where it
   is needed for diagnostics and optional constructor use checking. Kernel
   terms, constructor arity, and eliminator method types remain positional.
4. If named constructor expressions or named constructor patterns are included,
   validate them against the constructor's declared labels:
   - unknown labels reject;
   - duplicate labels reject;
   - missing required fields reject;
   - extra fields reject;
   - reordering is either supported by canonicalizing to declaration order, or
     explicitly rejected with a focused diagnostic.
5. Preserve existing positional constructor declarations, expressions, and
   patterns.
6. Add focused parser and elaboration tests for the accepted sugar and the
   rejection cases D0 includes.

## 5. Acceptance Criteria

AC1. Named constructor declaration sugar lowers positionally.

- A declaration using named-record constructor arguments elaborates to the same
  kernel constructor telescope shape as the equivalent positional declaration.
- The generated constructor, eliminator, and method arities remain positional.

AC2. Existing positional forms are unchanged.

- Existing explicit-family, simple-data, and coverage tests remain green.
- Existing positional constructor expressions and patterns continue to parse,
  elaborate, and match as before.

AC3. Label checking is fail-closed.

- Unknown, duplicate, missing, and extra labels reject in every syntax form this
  WP chooses to support.
- Diagnostics name the constructor and relevant field label when the current
  diagnostic surface can carry that information.

AC4. Indexed-family semantics are unchanged.

- Field sugar does not alter constructor target indices, type-possible
  classification, omitted impossible branch synthesis, dependent motive
  recovery, or reachability behavior.
- `SURF-gadt-coverage-diagnostics` regressions remain green.

AC5. Scope is bounded.

- Expected touched area is `crates/ken-elaborator/` implementation and tests.
- No `crates/ken-kernel`, `Cargo.lock`, `packages`, `spec`, or `conformance`
  movement is expected unless D0 routes and receives approval for a scope fork.
- `git diff --check` is clean.

AC6. Durable naming hygiene.

- Do not introduce WP/slice labels into implementation identifiers,
  diagnostics, or user-facing Ken fixture names. `SURF`, `gadt`, and AC labels
  may appear in Rust test names/comments and this WP doc, but implementation
  symbols and Ken source fixtures should use domain names such as `Vector`,
  `RecordStyleConstructor`, `Point`, or `Shape`.

## 6. Review Routing

Language owns implementation and QA.

Architect review is mandatory before integration because this WP changes
surface syntax and its elaboration boundary for inductive constructors. Kernel
QA is not expected for the framed scope, but becomes mandatory if any
`crates/ken-kernel/` path changes or if D0 routes a kernel mechanism fork.

Conformance-validator is not required unless the build changes `conformance/`
unexpectedly.

## 7. Do Not Reopen

- Do not change kernel inductive admission or positivity rules.
- Do not implement `KM-target-index-positivity` here.
- Do not relax the landed same-family target-index guard from
  `SURF-gadt-elaboration`.
- Do not alter indexed coverage, omitted impossible branch synthesis, or
  dependent-match motive recovery.
- Do not add smart constructors, views, or invariant-enforcing wrapper
  constructors.
- Do not redesign records or class records.
- Do not route CAT-5, catalog parsing, or package behavior through this WP.
- Do not use `local/refs/`.

## 8. Follow-On Queue

This is the final queued surface GADT ergonomics WP from the data-match build
sequence. After it lands and closes, the separate kernel expressivity follow-up
is `KM-target-index-positivity`, already framed on
`wp/KM-target-index-positivity @ 2f827bc`.
