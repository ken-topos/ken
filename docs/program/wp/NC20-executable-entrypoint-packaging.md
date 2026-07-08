# NC20 - Executable Entry Point and Packaging Metadata

**Owner:** Language/Runtime-led. **Branch:**
`wp/NC20-executable-entrypoint-packaging`. **Size:** M. **Risk:** medium.

## Objective

Select closed executable entry points and package the metadata required for
native Ken-only executable emission.

## Scope

In scope:

- entry-point selection from checked-core target closure reports;
- executable package metadata for arguments, result observation, traps, and
  required runtime support;
- diagnostics for missing, ambiguous, non-closed, or non-executable entry
  points;
- tests over compiler-produced checked-core and runtime-IR artifacts.

Out of scope:

- native object emission;
- host effect execution;
- library exports or imported native dependencies;
- new source-level entry syntax unless Architect/Language routes it.

## Deliverables

- Entry-point packaging surface used by later native emission.
- Positive fixture for at least one closed Ken-only executable target.
- Negative fixtures for stale package identity, non-closed target, unsupported
  target lowerability, and unresolved dependency.

## Acceptance

- The selected executable entry point is bound by stable symbol and exact
  checked-core package identity.
- Packaging metadata is deterministic and content-addressed.
- Unsupported entry points fail before backend/native work.
- The package does not infer semantics from raw source, module paths, or private
  runtime metadata.

## Guardrails

- Do not make successful entry selection imply native support.
- Do not invent a cross-package native linking contract.
- Do not consume library artifacts as semantic input.
