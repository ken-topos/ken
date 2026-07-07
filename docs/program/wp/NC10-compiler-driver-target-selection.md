# NC10 - Compiler Driver and Target Selection

**Owner:** Language-led, with Runtime and Verify support. **Branch:**
`wp/NC10-compiler-driver-target-selection`. **Size:** M. **Risk:** medium-high.

## Objective

Create the first general compiler entry point for Ken source/packages. The driver
must accept normal Ken input, run the existing elaborator and kernel admission
path, emit `CheckedCorePackage v0`, and select executable or library targets for
later lowering.

NC10 starts the General Ken-to-Runtime-IR Compiler campaign. It does not broaden
runtime lowering by itself; it makes the input and target boundary explicit.

## Scope

In scope:

- a compiler-facing API or CLI path that accepts a `.ken` file or package root;
- deterministic package identity and checked-core package emission;
- target selection by stable symbol or manifest-declared target;
- a report that records package identity, selected targets, dependencies,
  obligations, assumptions, unsupported lanes, and `trusted_base_delta`;
- diagnostics for missing, ambiguous, stale, or non-executable targets;
- fixtures that compile source through checked-core without hand-fed package
  construction.

Out of scope:

- new runtime-IR lowering support;
- native artifact emission;
- new validation or proof claims;
- kernel TCB growth.

## Deliverables

- Compiler driver entry point for source/package to `CheckedCorePackage v0`.
- Stable target-selection model for executable and library targets.
- Target-selection report surfaced in tests and suitable for later trust-report
  integration.
- Positive fixtures using real source/package input.
- Negative fixtures for missing target, ambiguous target, non-runtime target,
  unsupported target metadata, and stale or mismatched package identity.
- Documentation of the boundary in this WP file or the compiler program chapter.

## Acceptance

AC1. Source-to-package path:
At least one real `.ken` source/package fixture compiles through elaboration and
kernel admission into `CheckedCorePackage v0` without hand-authored checked-core
fixtures.

AC2. Exact target binding:
The selected target is identified by stable symbol and exact package identity.
A target from another package or stale package identity rejects.

AC3. Deterministic selection:
The same input and target selector produce the same selected target set and
package/report identities.

AC4. Loud unsupported lanes:
If the target cannot enter later runtime lowering, the driver records the
specific unsupported or non-runtime lane instead of silently dropping it.

AC5. Report honesty:
The report distinguishes emitted checked-core, selected target, unavailable
runtime lowering, unavailable native artifact, and unavailable validation facts.

AC6. No authority movement:
NC10 does not add kernel rules, trusted primitives, backend authority, or raw
source semantic evidence after checked-core emission.

## Guardrails

- The checked-core package remains the compiler semantic boundary.
- Do not make raw source hashes stand in for checked-core identity.
- Do not let target-selection success imply lowerability, runtime validation, or
  native executability.
- Do not broaden NC8/NC9 evidence labels.
- Do not introduce a Rust-kernel dependency on compiler driver output.
