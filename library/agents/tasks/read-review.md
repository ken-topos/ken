# Read and review

## 1. Use when

Use this module to explain an existing Ken file to a human and assess its
contract, proof, trust, authority, and execution posture. Do not use it to
silently repair the source or certify behavior that was not checked.

## 2. Prerequisites

Load `../core/read-ken.md`; add `../core/proof-and-trust.md` for claims and
`../core/toolchain.md` for execution evidence.

## 3. Current capability

Source signatures expose types, result contracts, effects, and capabilities.
Catalog entries add purpose, laws, trust and derivation, validation, and
references. The current tools can check source and, for a valid entrypoint,
observe reference or native execution.

## 4. Canonical forms

Return a review with these fields:

```text
Purpose
Public contract
Proof and assurance
Effects and required authority
Trust and derivation
Validation observed
Unavailable evidence
Sources
```

Every statement should identify the declaration or artifact that supports it.

## 5. Invariants and prohibitions

- Separate what the type states, what a proof establishes, and what a test
  observed.
- Report inherited and local trust separately.
- Do not infer runtime success from `ken check`.
- Do not call a delegated or unknown property proved.
- Do not edit while performing a read-only review.

## 6. Decision procedure

1. Classify the file and inventory public declarations.
2. Summarize each contract in plain language.
3. Record assurance and trusted-base evidence.
4. Record effect rows and capability requirements.
5. Bind every execution claim to a command or test.
6. Stop and list missing evidence before giving the verdict.

## 7. Failure signatures

Conflicting prose and signatures indicate a documentation defect; trust prose
without a ledger indicates missing evidence; a check-only artifact attached to
a runtime claim indicates an execution-evidence gap. Inspect the cited spec,
package validation section, or current test rather than choosing the friendlier
interpretation.

## 8. Validation

Run the artifact's declared targeted check when authorized. Verify every cited
path exists. A review is complete only when it contains both supported claims
and an explicit unavailable-evidence section.

## 9. Authority and sources

Use `spec/` for language rules, checked source for the program's own contract,
and generated/test artifacts for current implementation behavior. The review
shape is grounded in `docs/program/07-catalog-style-guide.md`. Revision:
`library/agents/manifest.toml`.

## 10. Known unavailable or partial behavior

A source-only review cannot establish external service behavior, host effects,
performance, liveness, or native parity. If the requested verdict depends on
one of those and no matching artifact is supplied, refuse that part of the
verdict and name the evidence required.
