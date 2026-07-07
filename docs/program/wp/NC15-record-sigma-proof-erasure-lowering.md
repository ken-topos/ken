# NC15 - Records, Sigma, and Proof-Erasure Lowering

**Owner:** Runtime-led, with Verify support. **Branch:**
`wp/NC15-record-sigma-proof-erasure-lowering`. **Size:** L. **Risk:** high.

## Objective

Lower runtime records, dependent Sigma-shaped data that has an executable
runtime representation, projections, and proof/law-erased fields while preserving
the NC9 proof-erasure metadata lanes.

## Scope

In scope:

- record construction and projection for runtime fields;
- dependent Sigma shapes that erase to runtime tuples/records under existing
  checked-core metadata;
- erased proof and law fields as explicit non-runtime lanes;
- projection diagnostics for erased or unavailable fields;
- metadata survival through target closure and runtime IR.

Out of scope:

- new kernel Sigma rules;
- proof reconstruction from erased fields;
- object layout or native record ABI.

## Deliverables

- Lowering for supported records, projections, and executable Sigma-shaped
  values.
- Preservation of `Runtime`, `ErasedProof`, and `ErasedLaw` field statuses in
  runtime metadata.
- Runtime-IR evaluator support for generated record/projection forms.
- Positive fixtures with runtime fields plus erased proof/law fields.
- Negative fixtures for projecting erased fields, dropped field metadata, stale
  record identity, and unsupported dependent field shapes.

## Acceptance

- Runtime record/projection programs lower and evaluate through runtime IR.
- Erased proof/law fields are not available as runtime values.
- NC9-style proof-erasure lanes remain derivable from the concrete
  package/program pair.
- Dropping or changing field status causes a named-lane rejection.

## Guardrails

- Do not make erased evidence observable at runtime.
- Do not let native layout concerns define record semantics.
- Do not claim whole proof-erasure correctness from field-status preservation.
