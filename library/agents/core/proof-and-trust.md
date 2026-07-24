# Proof and trust

## 1. Use when

Use this module to read, write, or repair a proof and to report a claim's trust
posture. Do not use it to convert testing, delegation, or an assumption into a
proof claim.

## 2. Prerequisites

Load `read-ken.md`. Load `write-ken.md` before editing proof source and
`../tasks/prove-or-repair.md` for a repair procedure.

## 3. Current capability

Ken checks proof terms in the kernel. Claims can be proved, tested, delegated,
or unknown; these labels are not interchangeable. `Axiom` and registered
primitives extend the trusted base and must remain visible in the trusted-base
ledger.

## 4. Canonical forms

For an equality goal, first normalize both endpoints:

| Reduced goal | Terminal |
|---|---|
| both endpoints collapse to the same constructor and the goal becomes `Top` | `Proved` |
| the goal remains a reflexive `Equal` because an endpoint is stuck | `Refl` |
| endpoints differ or proof requires a missing premise | neither; continue or stop |

For larger proofs, expose one case split or induction step at a time, then
re-run the checker on the resulting local goals.

## 5. Invariants and prohibitions

- `tested`, `delegated`, and `unknown` never mean `proved`.
- A proof ending in `Proved` is not weaker than one ending in `Refl`; the
  reduced goal shapes differ.
- Do not add `Axiom`, `foreign`, a primitive, or an open hole merely to make a
  proof compile.
- Trusted-base accounting includes inherited assumptions as well as new local
  declarations.
- A runtime success cannot repair an unproved kernel obligation.

## 6. Decision procedure

1. Read the exact goal and all premises.
2. Normalize both sides before selecting a terminal.
3. If the goal is not terminal, split only on a variable that exposes a
   constructor or induction hypothesis relevant to it.
4. Check after each structural step.
5. Inspect the trusted-base delta before and after the repair.
6. Stop if closure requires a new assumption or an authority not granted by
   the task.

## 7. Failure signatures

| Signature | Meaning | Next source |
|---|---|---|
| `Refl expects an Eq-shaped goal` | reduction changed the goal shape | reduced endpoints |
| `Proved` rejected at equality | goal did not collapse | stuck endpoint |
| unsolved metavariable or mismatch | missing inference or proof step | local expected type |
| new trusted-base entry | assumption or primitive introduced | declaration and trust ledger |
| `unknown` | open/partial boundary reached | claim status and source assumption |

## 8. Validation

Run `ken check <file>` and inspect the trusted-base artifact named by the
package or test. A repair is complete only when the original obligation checks
without an unintended trusted-base increase. Keep a negative control for the
distinction being repaired when practical.

## 9. Authority and sources

Normative sources are `spec/20-verification/21-spec-syntax.md`,
`spec/10-kernel/`, and `spec/60-security/64-trust-model.md`. Checked techniques
come from `catalog/guide/proof-techniques.ken.md`. The verified revision is in
`library/agents/manifest.toml`.

## 10. Known unavailable or partial behavior

Ken cannot manufacture a proof for a false, underspecified, or externally
delegated claim. Some runtime properties remain delegated even when structural
properties of the program are proved. If the only apparent closure adds trust,
assumes an unproved lemma, or relies on execution evidence, stop and state that
boundary instead of returning a fabricated proof.
