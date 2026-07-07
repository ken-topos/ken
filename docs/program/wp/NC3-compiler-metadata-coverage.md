# NC3 - Compiler Metadata Coverage

**Owner:** Language/Runtime. **Branch:** `wp/NC3-compiler-metadata-coverage`.
**Size:** L. **Risk:** high.

## Objective

Ensure `CheckedCorePackage v0` exposes enough checked metadata for future
native lowering without inventing semantics downstream.

## Scope

Cover primitive registry metadata, data/constructor metadata, record and Sigma
metadata, class/instance/dictionary metadata, recursion metadata,
effect/capability/foreign metadata, obligations, assumptions, and trust delta.

## Deliverables

- Metadata schemas for each area.
- Lowerability status fields: supported, unsupported, deferred, or explicit.
- Hash participation rules.
- Examples for `Bool`, `Nat`, `Option`, lists, class dictionaries, primitive
  operations, and accepted recursive groups.

## Acceptance

- Unsupported primitives or constructs fail loudly during lowering.
- Compiler can distinguish runtime fields from erasable law/proof fields.
- Accepted recursive groups have enough metadata for diagnostics and lowering
  decisions.
- Open holes, postulates, FFI, declassification, primitive assumptions, and
  delegated obligations appear in the package trust view.

## Guardrails

Do not silently omit metadata that affects runtime meaning. Do not finalize
native layout in this WP.
