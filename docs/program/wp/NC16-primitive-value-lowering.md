# NC16 - Primitive Value Lowering

**Owner:** Runtime-led, with Language support. **Branch:**
`wp/NC16-primitive-value-lowering`. **Size:** L. **Risk:** medium-high.

## Objective

Lower the language's executable primitive values and operations into runtime IR:
integers, booleans, strings, bytes, comparisons, arithmetic, checked partial
operations, and explicit traps.

## Scope

In scope:

- primitive constants and value encoding;
- supported arithmetic and comparisons;
- string and bytes values where the checked-core package exposes runtime
  semantics;
- checked partial operations as explicit trap-capable runtime IR;
- unsupported primitive diagnostics by stable primitive symbol.

Out of scope:

- native representation optimization;
- unchecked overflow or host-dependent primitive behavior;
- foreign I/O execution.

## Deliverables

- Lowering support for selected primitive values and operations.
- Runtime-IR evaluator support for generated primitive operations.
- Positive fixtures for integer, boolean, string, and bytes operations that are
  already semantically available in Ken.
- Negative fixtures for unsupported primitive, partial operation trap, stale
  primitive metadata, and host-dependent primitive attempt.
- Report lanes for primitive lowerability and trap behavior.

## Acceptance

- Supported primitive programs lower and evaluate through runtime IR.
- Runtime-IR evaluator and interpreter observations agree for positive fixtures.
- Partial primitive failure is explicit and report-visible.
- Unsupported primitives fail before backend work with stable primitive names.

## Guardrails

- Do not rely on Rust host overflow, pointer identity, locale, filesystem, or
  platform behavior as Ken primitive semantics.
- Do not silently coerce unsupported primitives to traps if that would hide a
  required obligation or assumption.
