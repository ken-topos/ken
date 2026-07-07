# V3 - Z3 Throughput Evaluation

**Owner:** Verify, with Spec/Architect review. **Branch:**
`wp/V3-z3-throughput-evaluation`. **Status:** Deferred until the software
catalog has a large enough proof-heavy corpus to measure solver impact.

## Objective

Decide, with data, whether an optional Z3 proof-search path is worth the added
dependency and operational complexity.

## Scope

This is a two-step evaluation program, not a trust-boundary change.

1. **Z3 proof-search adapter.** Add an optional, off-by-default solver-backed
   search path for the decidable and first-order prover fragments described in
   `spec/20-verification/23-prover.md`. Z3 may propose witnesses,
   certificates, or proof-reconstruction inputs. It must not mark an obligation
   proved by itself.
2. **Throughput characterization.** Run the same catalog verification corpus
   with the solver path disabled and enabled. Use catalog-scale packages, not
   toy fixtures, so the measurement can expose real throughput differences.

## Guardrails

- Z3 is an oracle, never an authority.
- `proved` still requires a kernel-checked certificate.
- Solver failure, timeout, nondeterminism, or missing certificate yields
  `unknown` or a rejected certificate, not a false proof.
- The disabled path remains the baseline and must keep passing.
- No kernel trusted-base expansion is in scope for this WP.

## Measurements

The comparison report must include:

- wall-clock verification time;
- obligation closure rate;
- kernel certificate-check time;
- solver time, timeout rate, and failure modes;
- deterministic replay behavior;
- dependency/build complexity and CI impact;
- examples where Z3 materially helps, if any;
- examples where Z3 adds cost without closing obligations, if any.

## Acceptance

- Z3-enabled and Z3-disabled runs are both reproducible on the same catalog
  corpus.
- The report recommends one of: expand solver use, keep solver use manual and
  opt-in, or remove/defer the path.
- Z3 is not made default unless the catalog-scale measurements show a clear
  verification-throughput benefit.
