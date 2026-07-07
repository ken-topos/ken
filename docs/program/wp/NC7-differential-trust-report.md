# NC7 - Differential Harness and Native Trust Report

**Owner:** Runtime/Verify. **Branch:** `wp/NC7-differential-trust-report`.
**Size:** M. **Risk:** medium.

## Objective

Make native compilation accountable by comparing against the interpreter and
emitting a trust report for every native artifact.

## Scope

Create a harness that runs supported examples through the interpreter and the
Cranelift path, compares Ken-observable behavior, and emits a native build trust
report.

## Deliverables

- Interpreter/native comparison command or test harness.
- Fixture format with source/core/runtime/backend hashes.
- Native build trust-report schema.
- F0/F1/F2 fidelity tier reporting.
- Failure diagnostics for mismatch versus unsupported construct.

## Acceptance

- CI can run the harness for supported examples.
- Behavioral mismatch names the artifact hashes and failing stage.
- Every native build can emit a report naming Cranelift, linker, runtime, and
  backend assumptions.
- Reports never promote tested native behavior to proved status.

## Guardrails

The harness must run the real landed interpreter as oracle, not compare against
hand-fed expected values.
