# Kernel conformance — seed cases

Format: `../README.md`. These pin the kernel's load-bearing soundness
commitments (`../../spec/10-kernel/README.md §5`) and the three prototype-gap
non-reproductions (`../../spec/10-kernel/README.md §6`).

**Phase tags.** Cases tagged [K2], [K2c], or [K-api] are out of K1 scope -- they
are acceptance criteria for later WPs. The K1 subset is extracted separately in
`seed-k1.md`. K2-tagged cases are elaborated in the K2 seed file
`observational/seed-observational.md`.

## kernel/universes/type-in-type-rejected (soundness)
- spec: `spec/10-kernel/12-universes.md §1`
- given: a derivation asserting `Type ℓ : Type ℓ` (same level)
- expect: **rejects** (universe inconsistency / level mismatch)
- why: no `Type:Type` (Girard's paradox); the prototype's unchecked universes
  are not reproduced. *(G1, G5.)*

## kernel/universes/predicative-pi (soundness)
- spec: `spec/10-kernel/12-universes.md §2`, `13 §1`
- given: `A : Type 0`, `B : Type 1`, form `(x : A) → B`
- expect: **accepts** at `Type (max 0 1) = Type 1` (predicative `max`)
- why: predicative formation; no universe drop.

## kernel/pi-sigma/dependent-second-projection
- spec: `spec/10-kernel/13-pi-sigma.md §2`
- given: `p : (n : Nat) × Vec A n`; the term `p.2`
- expect: **accepts** with type `Vec A p.1` (dependent second projection)
- why: Σ is genuinely dependent — the prototype's non-dependent Σ is not
  reproduced.

## kernel/pi-sigma/eta (Π and Σ)
- spec: `spec/10-kernel/13-pi-sigma.md §1–2`, `17 §2`
- given: `f : (x:A) → B`; `p : (x:A) × B`
- expect: `f ≡ λ x. f x` and `p ≡ (p.1, p.2)` hold **definitionally**
- why: type-directed η in conversion.

## kernel/identity/j-on-refl [K2]
- spec: `spec/10-kernel/15-identity.md §4`
- given: `J A a P d a (refl a)`
- expect: **reduces-to** `d` (J-β)
- why: the base computation rule.

## kernel/observational/j-on-nonrefl (soundness, the headline non-reproduction) [K2]
- spec: `spec/10-kernel/15-identity.md §4`
- given: `J` applied to a **non-`refl`** canonical equality (e.g. produced by
  `subst`/`cast`/a constructor congruence)
- expect: **reduces** (to a constructor form), does **not** get stuck
- why: closes the prototype's `J`-only-on-`refl` gap, via `cast` (ADR 0005).
  **Fails on any kernel that only reduces `J` on `refl`.**

## kernel/observational/cast-refl (soundness, canonicity/regularity) [K2]
- spec: `spec/10-kernel/16-observational.md §3`
- given: `cast A A refl a`
- expect: **reduces-to** `a` (regularity)
- why: `cast` on a reflexive type-equality is the identity — the clean
  equational theory OTT achieves (and De Morgan cubical does not).

## kernel/observational/funext-definitional (soundness) [K2]
- spec: `spec/10-kernel/16-observational.md §2`
- given: `Eq ((x:A)→B) f g`
- expect: **reduces-to** `(x:A) → Eq B (f x) (g x)` (funext is definitional)
- why: observational `Eq` at a Π-type *is* pointwise equality.

## kernel/observational/quotient-eq [K2]
- spec: `spec/10-kernel/16-observational.md §5`
- given: `Eq (A / R) [a] [b]`
- expect: **reduces-to** `R a b`
- why: quotient equality is the user relation; quotient soundness is
  definitional.

## kernel/inductive/elim-computes
- spec: `spec/10-kernel/14-inductive.md §3`
- given: `elim_Nat M z s (suc n)`
- expect: **reduces-to** `s n (elim_Nat M z s n)` (ι)
- why: eliminators compute structurally (the prototype's stubbed sum types do
  not).

## kernel/conversion/sct-accepts-lexicographic [K2c]
- spec: `spec/10-kernel/17-conversion.md §4`
- given: a definition recursing with lexicographic / permuted descent (e.g.
  Ackermann-shaped but well-founded)
- expect: **accepts** as transparent (SCT certifies termination)
- why: SCT is more permissive than single-argument structural recursion.

## kernel/conversion/sct-rejects-nonterminating (soundness) [K2c]
- spec: `spec/10-kernel/17-conversion.md §4`
- given: a non-terminating definition `loop x = loop x`
- expect: **rejects** transparent admission (totality error); δ-unfolding stays
  terminating
- why: keeps conversion (hence type-checking) decidable.

## kernel/judgments/certificate-recheck (soundness) [K-api]
- spec: `spec/10-kernel/18-judgments.md §4`
- given: a prover-produced certificate term `p` and a goal `φ`
- expect: `check_proof p φ` **accepts** iff `p` genuinely has type `φ`; a
  tampered certificate **rejects**
- why: the de Bruijn criterion — the prover is never trusted.
