# Prove or repair

## 1. Use when

Use this module when a stated Ken obligation does not check or a proof terminal
is wrong. Do not use it to weaken the proposition, add hidden trust, or replace
a proof with a passing runtime example.

## 2. Prerequisites

Load `../core/write-ken.md` and `../core/proof-and-trust.md`. Preserve the
original failing source and diagnostic.

## 3. Current capability

The checker exposes the expected proof type and validates reduction,
constructor reasoning, case splits, induction hypotheses, equality, and trusted
assumptions through the ordinary term language.

## 4. Canonical forms

At a terminal equality:

```text
normalize endpoints
same constructor throughout -> Proved
stuck but definitionally identical -> Refl
otherwise -> more reasoning or honest stop
```

For a structural proof, split the constructor-bearing input first and introduce
remaining binders in the order required by each branch.

## 5. Invariants and prohibitions

- Preserve the proposition unless the task explicitly changes the contract.
- Do not add `Axiom`, `foreign`, primitives, or holes as a repair.
- A proof-by-test is not a kernel proof.
- Each case chooses its terminal from its own reduced goal.
- Keep the trusted-base delta unchanged unless new trust is explicitly in
  scope and disclosed.

## 6. Decision procedure

1. Reproduce the exact diagnostic.
2. Read the expected goal after elaboration.
3. Normalize endpoints and try only the matching terminal.
4. If non-terminal, expose one relevant constructor or induction hypothesis.
5. Re-check after each step.
6. Compare trusted-base accounting.
7. Stop when closure requires a missing premise or new trust.

## 7. Failure signatures

`Refl expects an Eq-shaped goal` means the goal reduced away from equality;
`Proved` rejected means it did not reduce to `Top`; binder/type mismatches
usually mean a case introduced arguments in the wrong order; a new trust entry
means the repair crossed the boundary.

## 8. Validation

Run the original targeted check and a negative control that restores the old
terminal or removes the repaired step. Confirm the intended artifact goes red,
restore byte-identically, re-run green, and record the trusted-base comparison.

## 9. Authority and sources

Proof rules come from `spec/10-kernel/` and
`spec/20-verification/21-spec-syntax.md`; checked techniques come from
`library/guide/proof-techniques.ken.md`. Revision:
`library/agents/manifest.toml`.

## 10. Known unavailable or partial behavior

Some propositions are false, underspecified, depend on unavailable induction
principles, or describe delegated runtime behavior. If the existing premises
cannot close the goal without new trust, refuse to fabricate a proof and
report the smallest missing premise or unsupported mechanism.
