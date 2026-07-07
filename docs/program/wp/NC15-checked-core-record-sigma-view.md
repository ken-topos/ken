# NC15 Prerequisite - Checked-Core Record and Sigma Body View

**Owner:** Language. **Branch:**
`wp/NC15-checked-core-record-sigma-view`. **Size:** M. **Risk:** medium.

## Objective

Extend the checked-core executable body-view seam so the Runtime-owned NC15
lowering can consume package-authoritative record/Sigma construction and
projection facts without reading raw source or privately decoding canonical
bytes.

This is a prerequisite for
`NC15-record-sigma-proof-erasure-lowering`, not the Runtime lowering itself.

## Trigger

Runtime D0 for NC15 stopped on landed
`origin/main @ 51489675301eb52b3d44e70a867f3775077c97bf` because the package
metadata and NC9 proof-erasure witness lanes are present, but the executable
checked-core body-view seam does not yet expose record/Sigma construction or
projection terms.

Language D0 agreed with the stop. Runtime must not open NC15 D1 by decoding
raw canonical tags such as `pair`, `proj1`, or `proj2` itself.

## Scope

In scope:

- checked-core body-view terms for executable record/Sigma construction and
  projection, limited to the forms Runtime needs for NC15;
- stable field identity/order and field runtime-status visibility at the
  body-view point;
- validation against package-owned `RecordSigmaMetadata` and `FieldMetadata`;
- fail-closed lanes for unsupported dependent field shapes, erased-field
  projection that is not executable, stale field identity/order mismatches,
  and unsupported record/projection shapes;
- preservation of existing NC13/NC14 body-view behavior for variables, direct
  calls, constructor refs, erased constructor args, lambda/app/let, and match.

Out of scope:

- new kernel Sigma rules;
- proof reconstruction from erased fields;
- Runtime IR schema, evaluator, backend, Cranelift, native, or layout work;
- Verify implementation work;
- broad record ABI or object-layout commitments;
- any raw-source fallback or Runtime-side private canonical-byte authority.

## Deliverables

- Extended checked-core executable body-view representation in
  `crates/ken-elaborator/src/checked_core.rs`.
- Explicit fail-closed erasure behavior for the new body-view terms if they
  reach erasure before Runtime NC15 consumes them.
- Focused tests proving supported view extraction and each required rejection
  lane.
- A narrow package-contract note in
  `spec/40-runtime/46-checked-core-package.md` if the executable
  record/Sigma body-view extension becomes part of the durable checked-core
  package contract.

## Acceptance

- Runtime has a package-authoritative body-view surface for executable
  record/Sigma construction and projection facts.
- The view is derived only from validated `CheckedCorePackage v0` contents and
  package metadata.
- Field identity/order and runtime status are available where Runtime needs
  them for NC15.
- Unsupported dependent field shapes and non-executable erased-field
  projections fail before Runtime lowering with stable named lanes.
- Existing NC13/NC14 body-view and erasure tests continue to pass.
- No `crates/ken-kernel`, `crates/ken-runtime`, `crates/ken-interp`,
  backend, Cargo, package, or native-output files move in this prerequisite.

## Delivered seam

The prerequisite extends `ken_elaborator::checked_core` with structured
record/Sigma body-view terms:

- `CheckedCoreRecordSigmaConstructionView` binds a right-nested `pair` body to
  package-owned `RecordSigmaMetadata` / `FieldMetadata`. Runtime fields decode
  as executable checked-core body terms; erased law/proof fields remain opaque
  canonical term bytes and are not executable values.
- `CheckedCoreRecordSigmaProjectionView` recognizes the supported
  `proj1(proj2^k(base))` field-projection shape when the base's checked type
  identifies a package record/Sigma metadata entry. The projected field carries
  stable position, name, checked type symbol, and runtime status.
- The new stable lanes are
  `unsupported_dependent_field_shape`,
  `non_executable_erased_field_projection`, `stale_field_identity_order`, and
  `unsupported_record_projection_shape`.

The current erasure path rejects the new terms with
`record_construction_lowering_unsupported` and
`record_projection_lowering_unsupported` until the Runtime-owned NC15 lowering
consumes this seam.

## Guardrails

- Do not make erased proof/law fields observable at runtime.
- Do not let Runtime become a canonical-byte decoder for checked-core bodies.
- Do not claim NC15 Runtime lowering is implemented by this prerequisite.
- Do not claim whole proof-erasure correctness from field-status visibility.
- Do not broaden Sigma expressivity or positivity authority in this WP.
