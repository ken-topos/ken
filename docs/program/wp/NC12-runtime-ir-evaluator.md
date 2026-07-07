# NC12 - Runtime-IR Evaluator and Comparison Harness

**Owner:** Runtime-led, with Verify support. **Branch:**
`wp/NC12-runtime-ir-evaluator`. **Size:** L. **Risk:** high.

## Objective

Add a RuntimeProgram evaluator that can execute supported runtime IR directly,
without Cranelift. This gives the compiler a middle-artifact oracle before native
artifact work begins.

## Scope

In scope:

- evaluation of the currently supported `RuntimeExpr` subset;
- explicit values, constructors, records, closures, calls, traps, and primitive
  outcomes as supported by the IR;
- comparison harness between runtime-IR evaluation and the landed interpreter
  for closed executable targets;
- report fields that bind the exact package, target, runtime artifact, and
  observation.

Out of scope:

- native execution;
- linker or ABI behavior;
- effects or foreign calls beyond explicit unsupported/trap representation.

## Deliverables

- Runtime-IR evaluator for the current supported subset.
- Observation data model shared by runtime-IR evaluator and interpreter
  comparison harness.
- Positive fixtures over NC5/NC6 seed shapes and NC10/NC11 source-derived
  packages.
- Negative fixtures for unsupported expressions, traps, stale identities, and
  interpreter/runtime-IR disagreement.
- Trust-report or test-report wording that keeps runtime-IR evaluation separate
  from native execution.

## Acceptance

- Supported closed targets evaluate through runtime IR without invoking
  Cranelift.
- Interpreter and runtime-IR evaluator observations agree for the supported
  positive fixtures.
- Disagreement reports the exact target and artifact identities.
- Unsupported runtime constructs fail or trap explicitly; they do not produce
  arbitrary host behavior.
- The report does not claim backend, native, object, or linker validation.

## Guardrails

- The runtime-IR evaluator is not the kernel.
- Do not make evaluator success a proof of source-level semantics.
- Do not hide unsupported effects or foreign calls behind successful evaluation.
