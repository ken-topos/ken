# NC5 - Erasure Boundary and Runtime IR Seed

**Owner:** Runtime plus Architect review. **Branch:** `wp/NC5-erasure-runtime-ir`.
**Size:** L. **Risk:** high.

## Objective

Define the first executable compiler IR below checked core and the proof-erasure
boundary that feeds it.

## Scope

Specify erased executable core and Ken runtime IR with explicit effects,
closures, ADTs, records, calls, traps, primitives, and runtime observations.

## Deliverables

- Erasure boundary spec.
- Runtime IR syntax and value model sketch.
- Observation relation against interpreter behavior.
- Unsupported-erasure diagnostics.
- Small examples lowered from checked core to runtime IR.

## Acceptance

- Erased artifacts preserve obligations, assumptions, and trust metadata.
- Unsupported proof-erasure cases fail loudly.
- Runtime IR avoids hidden undefined behavior and backend-specific poison.
- Interpreter behavior can be compared against runtime IR behavior for a small
  subset.

## Guardrails

Do not target Cranelift directly from checked core. Do not make backend layout
the semantic authority.
