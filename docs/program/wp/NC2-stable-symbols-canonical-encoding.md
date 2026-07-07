# NC2 - Stable Symbols and Canonical Encoding

**Owner:** Language/Runtime. **Branch:** `wp/NC2-stable-symbols-canonical-encoding`.
**Size:** M. **Risk:** medium.

## Objective

Make checked-core package identity deterministic across elaboration sessions.

## Scope

Define stable symbol identity and canonical encoding for declarations, kernel
terms, primitive registry entries, modules, metadata tables, obligations, and
assumptions.

## Deliverables

- Stable symbol-id rule.
- Mapping from local `GlobalId` to stable symbols.
- Canonical encoding sketch for terms and declarations.
- Package hash rule.
- Negative examples for order-dependent or metadata-dropping encodings.

## Acceptance

- Two elaboration sessions can produce comparable symbols for the same package
  content.
- Local `GlobalId`s remain internal indices only.
- Equal checked artifacts produce identical hashes.
- Changed assumptions, primitives, or declarations change the hash or a
  referenced export hash.

## Guardrails

Do not make module paths kernel primitives. Do not depend on allocation order as
external identity.
