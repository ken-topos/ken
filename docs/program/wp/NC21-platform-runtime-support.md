# NC21 - Platform Runtime Support for Ken Executables

**Owner:** Runtime-led. **Branch:** `wp/NC21-platform-runtime-support`.
**Size:** L. **Risk:** high.

## Objective

Define and implement the first runtime support layer needed by Ken-only native
executables.

The runtime support is an internal executable implementation boundary, not a
public library ABI.

## Scope

In scope:

- internal representation policy for values, closures, constructors, records,
  primitive literals, traps, and runtime observations;
- executable startup/shutdown support for the starter platform target;
- deterministic trap and result observation surfaces for differential tests;
- platform/runtime facts recorded for trust reports.

Out of scope:

- stable C ABI or Rust ABI;
- shared-library or static-library generation;
- garbage-collection policy beyond the minimum starter runtime support;
- host-effect execution unless already explicit and unavailable.

## Deliverables

- Runtime support module or crate surface for Ken-only executables.
- Tests for value/closure/data/record/trap support used by NC10-NC18 fixtures.
- Metadata hooks for NC26 trust-report provenance.

## Acceptance

- Native executable support can represent the starter runtime-IR value shapes.
- Trap and result observations are comparable with runtime-IR evaluator output.
- Runtime support facts are recorded without claiming library or FFI stability.
- Unsupported shapes fail before native execution.

## Guardrails

- Keep this phase Ken-only and closed-target-only.
- Do not expose the internal runtime support as a stable ABI.
- Do not add kernel or checked-core authority to runtime support code.
