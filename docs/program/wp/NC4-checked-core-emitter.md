# NC4 - Checked Core Package Emitter

**Owner:** Language. **Branch:** `wp/NC4-checked-core-emitter`.
**Size:** L. **Risk:** medium.

## Objective

Emit `CheckedCorePackage v0` from the current elaborator/kernel state.

## Scope

Add an emitter entry point that turns current checked examples into validated
package artifacts. Missing compiler-relevant metadata must be represented
loudly, not omitted.

NC4 stays Language-owned at the checked-core package boundary. The D1
implementation must stop and route a separate owner-boundary fork if validation,
fixture emission, or consume-without-source tests require runtime IR, ABI or
layout, backend or Cranelift behavior, compiler verification, raw-source
fallback, or new kernel authority.

## Deliverables

- Emitter entry point and schema/version field.
- Fixture generation for representative checked examples.
- Artifact validator or round-trip checker.
- Diagnostics for unsupported metadata.
- Tests proving the compiler artifact can be consumed without reading surface
  source.

## Acceptance

- Representative examples emit artifacts.
- Artifacts validate against the NC1-NC3 contract.
- Missing fields are explicit unsupported entries.
- Unsupported artifact versions reject.
- Valid emitted packages can be consumed from the checked-core artifact without
  reading surface source bytes.
- No kernel TCB expansion.

## Guardrails

No backend lowering. No Cranelift dependency. No surface-parser shortcut around
the checked artifact.

## D1 Shape

The elaborator-side `checked_core` scaffold provides the NC4 package envelope:

- `CheckedCorePackageHeader` carries `package_kind`, explicit schema
  `version = 0`, producer and contract refs, package identity, and dependency
  semantic hashes.
- `emit_checked_core_package` materializes the package, computes the semantic
  and artifact hashes, and validates the result before returning it.
- `validate_checked_core_package` rejects bad kinds or versions, orphan
  metadata, missing lowerability, hash mismatches, and unsupported entries that
  are not paired with blocking lowerability.
- `consume_checked_core_package_for_target` validates the artifact and then
  applies the package-side lowerability gate; it has no raw-source input.
- `representative_checked_core_fixtures` emits a current checked-core fixture
  covering data/constructors, primitive metadata, class dictionary metadata,
  recursion metadata, and effect metadata.
