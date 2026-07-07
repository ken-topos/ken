# NC13 prerequisite - Checked-Core Body View

**Owner:** Language-led, with Runtime review. **Branch:**
`wp/NC13-checked-core-body-view`. **Size:** M. **Risk:** high.

## Objective

Expose a package-authoritative checked-core declaration body view that Runtime
can consume before opening NC13 expression lowering.

NC13 needs ordinary checked-core terms for selected reachable declarations:
variables, lets, lambdas, applications, and direct calls. Current `origin/main`
stores declaration bodies inside `CheckedCoreSemanticInputs.declarations` as
canonical bytes only, and the current erasure path still rejects reachable
declaration bodies with `checked declaration body lowering is not in NC5 seed`.
That is a real package-boundary gap, not a Runtime scope objection.

This WP provides the minimal Language-owned surface that lets Runtime inspect
those checked-core bodies from the emitted package path without falling back to
raw source or widening the compiler driver.

## Scope

In scope:

- an API or data model that returns structured checked-core body views for
  selected target declarations from `CheckedCorePackage v0` after package
  emission;
- preservation of exact package identity, semantic hash, target symbol, and
  closure membership while producing the view;
- stable failure for missing bodies, stale symbols, tampered canonical bytes,
  unsupported declaration kinds, and unresolved package metadata;
- focused fixtures that exercise ordinary body forms needed by NC13:
  lexical variables, non-recursive lets, lambdas, applications, and direct
  calls to reachable declarations;
- Runtime-facing documentation in the WP or code comments naming exactly what
  NC13 may consume.

Out of scope:

- runtime-IR lowering of the returned body view;
- Cranelift, native artifact, object, linker, or backend broadening;
- kernel rule changes or trusted primitive growth;
- raw-source inspection after checked-core emission;
- recursion, dictionaries/classes, modules/package-reference broadening,
  pattern matching, data constructors, effects, or foreign calls;
- broad checked-core serialization/version policy beyond what this minimal view
  requires.

## Deliverables

- A Language-owned checked-core body-view surface callable by later compiler
  stages.
- Positive tests proving the view can recover the selected target body and at
  least one reachable declaration body from an emitted package.
- Negative tests for tampered body bytes, missing declaration bodies,
  unsupported declaration kinds, and target symbols outside the checked closure.
- A short handoff note for Runtime naming the exact function/type to consume and
  the stable errors or lane names NC13 should propagate.

## Acceptance

- The body view is derived from the emitted `CheckedCorePackage v0` and selected
  target closure, not from raw source or a hand-maintained side table.
- The view exposes enough structured checked-core term data for NC13 to lower
  variables, lets, lambdas, applications, and direct calls in later Runtime
  work.
- Package identity, core semantic hash, target symbol, and closure membership
  remain checked before a body is returned.
- Tampering with canonical declaration bytes, dropping a reachable body, or
  requesting a body outside the selected closure rejects with a stable
  diagnostic before Runtime lowering.
- Unsupported shapes remain explicit unsupported results. This WP must not make
  Runtime silently treat them as lowered.
- Existing NC10-NC12 tests continue to pass, and the new surface does not change
  kernel admission or trusted-base accounting.

## Implemented Language Surface

Runtime should consume the Language-owned body view through
`ken_elaborator::checked_core`:

- `CheckedCoreBodyViewSelection` carries the selected closure facts:
  package identity, package semantic hash, package artifact hash, target symbol,
  and reachable declarations.
- `checked_core_body_view_for_selection(package, selection)` validates the
  package and selection facts, then returns all reachable transparent
  declaration bodies as `CheckedCoreBodyView`.
- `checked_core_declaration_body_view(package, selection, symbol)` returns one
  declaration body and rejects symbols outside the selected closure.

The returned `CheckedCoreBodyTerm` intentionally covers only the NC13 seed
expression forms: de Bruijn variables, direct declaration calls, lambdas,
applications, and non-recursive lets. Checked type annotations are preserved as
canonical checked-core term bytes, not source text.

Stable `CheckedCoreBodyViewError::lane()` values for downstream diagnostics:

- `invalid_checked_core_package`
- `body_view_package_identity_mismatch`
- `body_view_semantic_hash_mismatch`
- `body_view_artifact_hash_mismatch`
- `target_outside_selected_closure`
- `body_outside_selected_closure`
- `missing_checked_declaration_body`
- `mismatched_checked_declaration_symbol`
- `unsupported_checked_declaration_kind`
- `unsupported_checked_body_shape`
- `body_reference_outside_selected_closure`
- `body_reference_without_declaration`
- `malformed_checked_declaration_body`
- `trailing_checked_declaration_body_bytes`

## Guardrails

- Do not create a raw-source fallback or use source text as semantic evidence
  after checked-core emission.
- Do not make Runtime reconstruct unchecked terms from arbitrary bytes unless
  the reconstruction is tied back to the package's canonical semantic content
  and exact identity.
- Do not widen compiler-driver target selection, lowerability meaning, or NC13
  expression-lowering scope in this prerequisite.
- Do not claim any runtime-IR, evaluator, Cranelift, native, NC8, or NC9
  validation result from this WP.
- Keep any WP/campaign identifiers out of durable non-test implementation names
  and public report strings.

## D0 handoff

Runtime D0 for NC13 (`evt_16gtg3sbyymdk`) correctly stopped before D1 because
no landed package-side body-view surface exists on `origin/main @
bf8b2433991baaf6597f920742d28f721b8484fc`.

Language should confirm the boundary, then implement the narrowest package
body-view surface satisfying the acceptance criteria. Runtime should stay parked
on NC13 until this prerequisite lands or Language points to an already-landed
equivalent surface.
