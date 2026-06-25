# Verification conformance — seed cases

Format: `../README.md`. These pin the verification verdicts (G2–G4) and the
soundness backbone (the prover is untrusted; the kernel re-checks).

## verify/proved-postcondition (G2)
- spec: `spec/20-verification/21-spec-syntax.md §1`, `23-prover.md`
- given: `view abs (n : Int) : Int ensures result ≥ 0 = if n < 0 then -n else n`
  with a correct proof
- expect: **proved**; `trusted_base_delta` empty
- why: a correct postcondition proof is accepted.

## verify/wrong-proof-rejected (G2, soundness)
- spec: `spec/20-verification/23-prover.md §1`, `18 §4`
- given: the same function with a **wrong** proof certificate for `result ≥ 0`
- expect: **rejected** by the kernel re-check (not accepted on the prover's
  word)
- why: the de Bruijn criterion — a bad certificate cannot pass.

## verify/disproved-with-countermodel (G4)
- spec: `spec/20-verification/24-diagnostics.md §1`
- given: `view f (n : Int) : Int ensures result > 0 = n` (false for `n ≤ 0`)
- expect: **disproved**; diagnostic `kind=countermodel`, `verdict=false`, naming
  the failing input class (`n ≤ 0`)
- why: a genuine counterexample is reported with the false-vs-unknown verdict.

## verify/incomplete-with-hole (G4)
- spec: `spec/20-verification/24-diagnostics.md §2`, `21 §5`
- given: a function with an `ensures` the prover cannot discharge (no
  counterexample either)
- expect: **incomplete**; diagnostic `kind=hole` with goal+context; the program
  **still runs**, producing `unknown` where the property is observed; the hole
  appears in `trusted_base_delta`
- why: partial verification — runs, marks the gap, lists the assumption.

## verify/decidable-via-reflection (G3)
- spec: `spec/20-verification/23-prover.md §3`
- given: a decidable arithmetic goal over `Int` (e.g. `2 + 2 == 4`)
- expect: **proved** by reflective decision (the kernel computes the decision
  procedure), no external solver in the trusted path
- why: the computing kernel discharges the decidable fragment directly.

## verify/fo-via-kripke (G3)
- spec: `spec/20-verification/23-prover.md §4`
- given: a first-order intuitionistic obligation routed to the Kripke embedding
- expect: **proved** with a **kernel-re-checked certificate** (embedding route a
  or b)
- why: the classical solver is used soundly via the embedding.

## verify/soundness-regression (G3, soundness — the critical one)
- spec: `spec/20-verification/23-prover.md §7`, `README.md §4`
- given: a `φ` that is **classically valid but topos-invalid** (e.g. an instance
  of `¬¬p ⇒ p` for an undecidable `p`), with Z3 "proving" it
- expect: the produced certificate **fails the kernel re-check** → **not**
  reported `proved`
- why: demonstrates a classical solver **cannot** make a Ken proof unsound.

## verify/protocol-schema-valid (G4, T1)
- spec: `spec/20-verification/25-protocol.md`
- given: any verification run emitting `--format=json`
- expect: every verdict document validates against `ken.verify/v1`; each
  non-discharged obligation carries a diagnostic; obligation ids are **stable**
  across an unrelated edit
- why: the agent contract is total and stable.
