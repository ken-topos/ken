# NC8 - First Certificate Validator

**Owner:** Verify/Kernel. **Branch:** `wp/NC8-first-certificate-validator`.
**Size:** M. **Risk:** high.

## Objective

Move one compiler step from merely tested toward independently validated.

## Scope

Choose one bounded compiler transformation and define a certificate format plus
checker for that transformation.

## Candidate Passes

- Proof erasure.
- Pattern-match lowering.
- Closure conversion.
- Dictionary lowering.
- Runtime IR validation.

## Deliverables

- Certificate schema for one pass.
- Checker independent of the producer.
- Positive and negative examples.
- Trust-report integration.

## Acceptance

- Valid certificates accept.
- Malformed or false certificates reject.
- Native trust report can mark the pass as validated rather than merely tested.
- The checker does not trust the Cranelift backend.

## Guardrails

Do not claim global compiler verification from one validator. Do not put the
backend in the kernel TCB.
