# NC3 - Compiler Metadata Coverage

**Owner:** Language.
**Branch:** `wp/NC3-compiler-metadata-coverage`.
**Size:** L. **Risk:** high.

## Objective

Ensure `CheckedCorePackage v0` exposes enough checked metadata for future
native lowering without inventing semantics downstream.

## Scope

Cover primitive registry metadata, data/constructor metadata, record and Sigma
metadata, class/instance/dictionary metadata, recursion metadata,
effect/capability/foreign metadata, obligations, assumptions, and trust delta.

NC3 stays at the `CheckedCorePackage v0` boundary: post-elaboration,
kernel-admitted checked core before erasure/runtime IR. Runtime joins only if
implementation uncovers a concrete metadata decision that cannot be expressed
by the package contract without specifying ABI, layout, backend IR, or native
lowering behavior.

## Deliverables

- Metadata schemas for each area.
- Lowerability status fields: supported, unsupported, deferred, or explicit.
- Hash participation rules.
- Examples for `Bool`, `Nat`, `Option`, lists, class dictionaries, primitive
  operations, and accepted recursive groups.

## D1 Shape

- Extend the elaborator-side checked-core package scaffold with typed metadata
  sections for primitives, data/constructors, records/Sigma,
  classes/instances/dictionaries, recursion, effects/foreigns, obligations,
  assumptions, and trust delta.
- Add explicit lowerability status values: `supported`, `unsupported`,
  `deferred`, `requires-feature`, and named explicit states.
- Ensure runtime-meaning metadata participates in `core_semantic_hash`; keep
  source identity and annotations artifact-only/non-semantic.
- Add a package-side lowerability gate that rejects a target closure reaching a
  lowering-blocking entry before erasure/runtime IR.
- Keep the implementation out of runtime crates, kernel admission, ABI/layout,
  Cranelift/backend lowering, compiler verification, and raw-source consume
  paths.

## Acceptance

- Unsupported primitives or constructs fail loudly during lowering.
- Compiler can distinguish runtime fields from erasable law/proof fields.
- Accepted recursive groups have enough metadata for diagnostics and lowering
  decisions.
- Open holes, postulates, FFI, declassification, primitive assumptions, and
  delegated obligations appear in the package trust view.
- Metadata changes that affect runtime meaning perturb `core_semantic_hash`.
- `Bool`, `Nat`, `Option`, lists, class dictionaries, primitive operations, and
  accepted recursive groups are representable without committing to native
  layout.

## Guardrails

Do not silently omit metadata that affects runtime meaning. Do not finalize
native layout in this WP. Do not introduce runtime IR, ABI/layout, backend
lowering, Cranelift, compiler-verification, self-hosting, kernel TCB expansion,
or raw surface-source consumption. Do not consult `local/refs`.
