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

- Erasure boundary spec: `spec/40-runtime/47-erasure-runtime-ir.md`.
- Runtime IR syntax and value model sketch:
  `crates/ken-runtime/src/ir.rs`.
- Observation relation against interpreter behavior:
  `spec/40-runtime/47-erasure-runtime-ir.md §5` and the NC5 seed examples in
  `crates/ken-runtime/src/ir.rs`.
- Unsupported-erasure diagnostics:
  `crates/ken-elaborator/src/erasure.rs`.
- Small examples lowered from checked core to runtime IR:
  `ken_runtime::nc5_seed_examples`.

## Acceptance

- Erased artifacts preserve obligations, assumptions, and trust metadata.
- Unsupported proof-erasure cases fail loudly.
- Runtime IR avoids hidden undefined behavior and backend-specific poison.
- Interpreter behavior can be compared against runtime IR behavior for a small
  subset.

## Guardrails

Do not target Cranelift directly from checked core. Do not make backend layout
the semantic authority.

## D1 Evidence

- Package-only consumption enters through
  `erase_checked_core_package_for_target`, which validates
  `CheckedCorePackage v0` and copies no source-identity lane into runtime IR
  meaning.
- Lowerability blockers, reachable unsupported entries, foreign boundaries, and
  runtime metadata gaps reject before backend work.
- Runtime artifacts preserve auditable checked-core metadata, including
  obligation status/origin/runtime-impact, assumption/trust kind/target/runtime
  impact, lowerability/unsupported status, and the checked-core semantic
  metadata lanes present in the package.
- Runtime IR contains explicit primitives, traps, closures, ADTs, records,
  effects, and calls, but no Cranelift, ABI, object-format, native layout, or
  pointer-identity surface.
- Observation checks are limited to returned ground values and explicit traps.
