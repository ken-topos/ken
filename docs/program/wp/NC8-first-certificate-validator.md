# NC8 - First Certificate Validator

**Owner:** Verify/Kernel. **Branch:** `wp/NC8-first-certificate-validator`.
**Size:** M. **Risk:** high.

## Objective

Move one compiler boundary from "tested native behavior" toward an independently
checked validation claim by adding the first explicit certificate schema and
checker for a bounded runtime-IR property.

NC8 is not whole-compiler verification. It is the first small, auditable
certificate/checker loop that later compiler passes can reuse.

## Settled Inputs

- `docs/program/07-compiler-program.md §4-§5` defines the fidelity ladder.
  NC8 feeds the first F2-style claim: a Ken-owned IR invariant or pass contract
  is checked by tests and report-visible validation evidence.
- `spec/40-runtime/46-checked-core-package.md` defines the checked-core package
  artifact consumed before erasure/runtime IR. Raw source bytes are not semantic
  input to this WP.
- `spec/40-runtime/47-erasure-runtime-ir.md` defines the NC5 erasure/runtime-IR
  boundary, including metadata survival, loud-fail cases, runtime IR non-goals,
  and the supported seed examples.
- NC7 landed an interpreter-backed differential report path. NC8 may extend the
  native trust-report vocabulary, but it must not weaken NC7's F1 identity
  binding or compare anything broader than the checked runtime artifact it
  validates.

## D0 Decision Frame

Verify and Kernel must begin with a D0 boundary read before implementation.

Preferred NC8 target:

- a runtime-IR artifact validator for the NC5/NC6 supported subset, checking a
  certificate that names the exact `RuntimeProgram` identity and proves that the
  artifact is within the bounded lowerable subset already accepted by
  `spec/40-runtime/47-erasure-runtime-ir.md`.

D0 may choose a narrower target if the code shape demands it, but it must keep
the following properties:

- the checker is independent of the producer path that constructs the
  `RuntimeProgram`;
- the checked claim is about runtime IR / metadata shape, not Cranelift
  correctness;
- the certificate names the exact artifact identity it validates:
  `package_identity`, `core_semantic_hash`, and runtime artifact hash;
- malformed, stale, or false certificates reject loudly;
- the result can be surfaced in the native trust report as a bounded
  "validated" entry without claiming F3+ proof or backend certification.

If D0 finds that the target cannot be expressed without moving authority into
the kernel, stop and route an Architect/Steward fork. Do not silently replace
the target with a backend smoke test.

## Scope

In scope:

- define a small Rust-side certificate data model for one runtime-IR validation
  claim;
- implement a checker that recomputes the checked facts from the exact
  `RuntimeProgram` and rejects mismatches;
- add positive and negative fixtures covering acceptance, malformed
  certificates, stale artifact identity, and false claims;
- connect successful validation to the native trust-report surface as an F2
  style validated-pass entry, or an equivalent report-visible validation fact
  if D0 determines the existing report model needs a smaller first step.

Out of scope:

- proving compiler correctness in Ken;
- validating Cranelift lowering, machine code, ABI/layout, object files, native
  pointer identity, or linker behavior;
- adding a new kernel trusted primitive or making the backend part of the
  kernel TCB;
- accepting raw surface source as the semantic input to the checker;
- global validation of every runtime-IR construct.

## Suggested Validation Claim

The recommended first certificate is a "supported runtime artifact" certificate.
It should be intentionally boring:

- the certificate binds to the exact runtime artifact identity;
- it enumerates the subset assumptions it claims, such as no effects, no
  capabilities, no runtime checks, no unsupported lowerability entries, and only
  declarations/examples in the NC5/NC6 supported seed shape;
- the checker recomputes those facts from `RuntimeProgram` and rejects if any
  claimed fact is missing, stale, or contradicted by the artifact;
- the native trust report may then say the runtime artifact passed this bounded
  validator, while all backend/toolchain facts remain at their existing evidence
  level.

This target deliberately complements NC7: NC7 compares interpreter/native
observations for supported examples; NC8 checks that the runtime artifact is
actually within a named supported validation boundary before that comparison is
reported as upgraded evidence.

## Deliverables

- A certificate schema and checker for the D0-selected runtime-IR validation
  claim.
- Positive fixture(s) for a valid NC5/NC6 seed artifact certificate.
- Negative fixture(s) for:
  - wrong artifact identity;
  - unsupported metadata or lowerability state hidden by the certificate;
  - malformed / missing certificate fields;
  - a certificate whose claimed validation facts do not match the runtime
    artifact.
- Trust-report integration that records the validated claim separately from
  F1 interpreter/native agreement.
- A short as-built note in this WP file if D0 selects a narrower target than
  the suggested validation claim.

## Acceptance Criteria

AC1. Exact-artifact binding:
The checker rejects any certificate whose identity does not match the
`RuntimeProgram` being validated.

AC2. Independent recomputation:
The checker recomputes the validation facts from the runtime artifact and does
not trust the certificate producer's assertion.

AC3. Loud false-claim rejection:
Malformed, stale, unsupported, or contradictory certificates reject with
diagnostics naming the failing validation stage or fact.

AC4. Positive bounded validation:
At least one supported seed artifact accepts and records a report-visible
validation fact distinct from NC7's F1 interpreter/native agreement.

AC5. Trust-report honesty:
The native trust report can name the bounded validated claim, but it must not
promote the backend, Cranelift lowering, machine code, or whole compiler to a
proved status.

AC6. Kernel boundary:
Any kernel changes, if D0 concludes they are needed, require an explicit
Architect/Steward fork before D1. The default expectation is no kernel TCB
growth.

## Guardrails

- Do not claim global compiler verification from one validator.
- Do not put Cranelift, the linker, object emission, or native execution in the
  kernel TCB.
- Do not replace validation with another interpreter/native differential test;
  NC7 already owns that axis.
- Do not let a certificate validate a different artifact than the one executed
  or reported.
- Do not turn a missing exact-run source into prose evidence. If a fact is not
  captured, report it as unavailable.
