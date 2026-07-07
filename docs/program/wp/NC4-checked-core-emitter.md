# NC4 - Checked Core Package Emitter

**Owner:** Language. **Branch:** `wp/NC4-checked-core-emitter`.
**Size:** L. **Risk:** medium.

## Objective

Emit `CheckedCorePackage v0` from the current elaborator/kernel state.

## Scope

Add an emitter entry point that turns current checked examples into validated
package artifacts. Missing compiler-relevant metadata must be represented
loudly, not omitted.

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
- No kernel TCB expansion.

## Guardrails

No backend lowering. No Cranelift dependency. No surface-parser shortcut around
the checked artifact.
