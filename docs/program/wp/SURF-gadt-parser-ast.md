# SURF-gadt-parser-ast

## 1. Objective

Implement the first build slice for the dependent-constructor surface described
in `spec/30-surface/34-data-match.md §8`: parse and represent the explicit
`data ... : ... where { ... }` family form without changing current simple sums
or elaborating indexed families yet.

This WP is a Language parser/AST slice. It makes the syntax admissible to the
front end and structurally inspectable by tests. It does **not** lower explicit
families to kernel inductives; that belongs to `SURF-gadt-elaboration`.

## 2. Spec Inputs

Fixed inputs:

- `spec/30-surface/32-grammar.md §1`
  - `data D tyvar* = simple_ctor ...` remains the legacy simple form.
  - `data D data_param* : data_family where { data_ctor ; ... }` is the
    explicit inductive-family form.
  - Legacy `=` form accepts only `simple_ctor`, not `C : ctor_type`.
- `spec/30-surface/34-data-match.md §2`
  - parameters are binders before the colon;
  - the type after the colon is an index telescope ending in `Type`;
  - explicit constructor signatures have the shape
    `C : telescope -> D params indices`;
  - earlier constructor binders are in scope for later argument types and final
    result-index expressions;
  - named-record constructor shorthand remains only simple default-result sugar.
- `spec/30-surface/39-elaboration.md §2.2`
  - this slice prepares the syntax/AST shape that the later elaboration slice
    will lower.
- `conformance/surface/data-match/seed-data-match.md` AC9
  - positive Vec declaration;
  - legacy-form explicit signature rejected;
  - proof-carrying constructor signature accepted.

Current implementation state is perishable and must be verified against
`origin/main` at pickup. As of this frame, `parser.rs` accepts only
`data D p... = C type_atom* | ...`, and `ast.rs::CtorDecl` stores only
positional argument types.

## 3. Deliverables

Implement on branch `wp/SURF-gadt-parser-ast`:

1. Extend the surface AST enough to distinguish:
   - legacy simple data declarations;
   - explicit family declarations with data-head parameter binders;
   - family result type / index telescope syntax;
   - constructors in simple default-result form;
   - constructors with explicit `C : ctor_type` signatures.
2. Extend the parser to accept:
   - `data Vec (A : Type) : Nat -> Type where { ... }`;
   - constructor signatures with named binders, implicit binders, and anonymous
     arrows;
   - semicolon-separated constructor declarations in the `where` block;
   - simple constructors inside the explicit `where` block as default-result
     sugar where the spec admits `data_ctor ::= simple_ctor`.
3. Preserve the legacy simple form:
   - `data Box A = Mk A` still parses as before;
   - existing L2 data/match tests continue to pass;
   - constructor atom parsing for the legacy form is not widened to accept
     explicit signatures.
4. Add focused parser/AST tests in `crates/ken-elaborator/tests/`.
   The tests should inspect parser/AST output or a parser-level verdict. They
   should not require successful elaboration of indexed families.

## 4. Acceptance Criteria

AC1. Positive explicit family parse.

- A source equivalent to:

  ```ken
  data Vec (A : Type) : Nat -> Type where {
    VNil  : Vec A 0;
    VCons : (n : Nat) -> A -> Vec A n -> Vec A (n+1)
  }
  ```

  parses successfully into an explicit-family AST shape.
- The AST preserves:
  - family name `Vec`;
  - parameter binder `A : Type`;
  - family result/index type `Nat -> Type`;
  - constructor names `VNil` and `VCons`;
  - `VCons` telescope entries in order, distinguishing named binder
    `(n : Nat)` from anonymous arrows.

AC2. Proof-carrying constructor syntax parse.

- A checked-source-style constructor signature from `34 §2.2` parses.
- Later argument types can syntactically mention earlier binder names
  (`bs`, `len`) without being rejected by the parser.
- The test asserts the parsed constructor signature shape, not elaboration.

AC3. Legacy grammar boundary.

- `data Box A = Mk A` still parses.
- `data Box (A : Type) : Type where { Mk : A -> Box A }` parses.
- `data Box A = Mk : A -> Box A` rejects at the syntax boundary.
- The rejection should be attributable to the legacy-form grammar, not to
  elaboration/kernel behavior.

AC4. No elaboration scope creep.

- The WP does not implement kernel lowering, bad-result-target checking,
  positivity routing, or indexed coverage diagnostics.
- Tests for this WP must not claim that explicit families elaborate or
  kernel-check. Those belong to later WPs.

AC5. Regression gate.

- Existing data/match acceptance tests continue to pass.
- New tests are scoped to parser/AST behavior and durable grammar boundaries.
- `git diff --check` is clean.
- No `crates/ken-kernel`, `Cargo.lock`, `packages`, `spec`, or `conformance`
  movement is expected.

AC6. Durable naming hygiene.

- Do not introduce WP/slice names into implementation identifiers. `SURF`,
  `gadt`, or AC labels may appear in Rust test names/comments and this WP doc,
  but AST variants, parser helpers, diagnostics, and source fixtures should use
  domain names such as `ExplicitData`, `ConstructorSignature`, or
  `IndexedFamily`.

## 5. Review Routing

Language owns implementation and QA.

Architect review is required before integration because this WP changes the
accepted surface grammar and AST shape for a future dependent-family feature,
even though it should not touch the kernel. Conformance-validator is not
required unless the build changes `conformance/` unexpectedly.

## 6. Do Not Reopen

- Do not implement CAT-5 or route CAT-5 through this feature.
- Do not implement `SURF-gadt-elaboration`, result-target validation,
  positivity, or coverage diagnostics here.
- Do not add record-field labels inside explicit constructor signatures.
- Do not add named-argument application.
- Do not change kernel inductive admission.
- Do not weaken the legacy-form boundary by accepting `data D = C : ...`.
- Do not carry operational WP names into implementation/source identifiers.

## 7. Follow-On Queue

If this WP lands, the next queued data-match build WP is
`SURF-gadt-elaboration`, followed by `SURF-gadt-coverage-diagnostics` and the
later ergonomics slice `SURF-gadt-field-sugar`.
