# NC2 - Stable Symbols and Canonical Encoding

**Owner:** Language. **Branch:** `wp/NC2-stable-symbols-canonical-encoding`.
**Size:** M. **Risk:** medium.

## Objective

Make checked-core package identity deterministic across elaboration sessions.

## Scope

Define stable symbol identity and canonical encoding for declarations, kernel
terms, primitive registry entries, modules, metadata tables, obligations, and
assumptions.

NC2 stays inside the checked-core artifact boundary: after elaboration and
kernel admission, before erasure/runtime IR. Runtime ownership begins when a
later package consumes the checked-core package for erasure, lowering, ABI, or
runtime IR. This work does not add a runtime-owned seam.

## Deliverables

- Stable symbol-id rule:
  - top-level declarations are package/module-qualified symbols;
  - constructors are keyed by parent family stable symbol plus constructor
    name;
  - primitives are keyed by primitive-registry symbol string;
  - obligations use stable clause ids;
  - assumptions/trust entries attach to stable symbols or stable obligation ids.
- Mapping from local `GlobalId` to stable symbols. Local ids are producer-side
  evidence only and may never be serialized as durable package identity.
- Canonical encoding rules for levels, terms, declarations, semantic metadata,
  obligations, assumptions, unsupported entries, dependencies, and annotations.
- Package hash rule:
  - `core_semantic_hash` covers only checked-core semantic/trust inputs;
  - `artifact_hash` may additionally cover provenance and non-semantic
    annotations.
- Negative examples for allocation-order-dependent, order-dependent,
  primitive-id-dependent, metadata-dropping, annotation-leaking, and
  trust-dropping encodings.

## Acceptance

- Two elaboration sessions can produce comparable symbols for the same package
  content.
- Local `GlobalId`s remain internal indices only.
- Equal checked artifacts produce identical hashes.
- Changed assumptions, primitives, or declarations change the hash or a
  referenced export hash.
- Reordering declarations or metadata entries without changing meaning does not
  change `core_semantic_hash`.
- Primitive `GlobalId` drift does not change primitive identity when the
  primitive-registry symbol is unchanged.
- Non-semantic annotation changes do not change `core_semantic_hash`, but may
  change `artifact_hash`.
- Missing stable bindings for local ids reject before emitting canonical bytes.

## Guardrails

Do not make module paths kernel primitives. Do not depend on allocation order as
external identity. Do not add kernel rules, raw source consumption, erasure,
runtime IR, ABI/layout, Cranelift lowering, backend claims, or compiler
verification claims in NC2.
