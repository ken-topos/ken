# NC24 - Native Execution Differential Suite

**Owner:** Runtime/Verify-led. **Branch:**
`wp/NC24-native-execution-differential-suite`. **Size:** L. **Risk:** high.

## Objective

Build the first native execution harness and differential suite over the
NC10-NC18 starter executable corpus.

## Scope

In scope:

- native executable run harness;
- comparison of native observations against runtime-IR evaluator observations
  and interpreter observations when available;
- stable mismatch diagnostics naming package, target, artifact, and lane;
- positive and negative corpus selection for the starter executable phase.

Out of scope:

- translation-validation proof;
- performance benchmarking as a gate;
- effect/foreign execution beyond NC25 policy;
- library execution tests.

## Deliverables

- Differential test harness for native executable artifacts.
- Corpus covering pure starter programs, traps, records, data matches,
  primitives, recursion, dictionaries, and imports in the supported subset.
- Mismatch reports with exact artifact identities.

## Acceptance

- Native observations agree with runtime-IR evaluator observations on the
  starter corpus.
- Interpreter observations are used where the NC18 report says they are
  available.
- Unavailable comparisons remain unavailable rather than passing silently.
- Failures identify the concrete mismatching lane.

## Guardrails

- Do not promote differential tests to proof.
- Do not test hand-built native artifacts as if they were compiler output.
- Do not include library ABI or foreign-interoperability cases.
