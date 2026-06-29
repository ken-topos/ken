# Kernel-2 (observational) conformance -- K2-scoped seed cases

Format: `../../README.md`. These pin the K2 acceptance criteria
(`../../docs/program/wp/K2-observational.md` par. 2) and the soundness
commitments from `../../spec/10-kernel/README.md` par. 6 (#9-14).
All cases are K2-scoped; K1 conformance must continue to pass (no
regression).

Cases tagged **(oracle)** are to be validated against the prototype at
build time by the Spec enclave.

---

## Acceptance criterion: Omega proof-irrelevance (frame par. 2 item 5, README #9)

### observational/omega-pi-convertible (soundness)
- spec: `spec/10-kernel/16-observational.md` par. 1.2
- given: `P : Omega`, `p : P`, `q : P` in context; conversion check
  `p` vs `q` at type `P`
- expect: **convertible** (conversion returns true)
- why: Omega-PI is definitional -- any two proofs of a proposition are
  equal. The checker must not inspect the terms. Conversion at Omega
  type is a constant-time "yes."

### observational/omega-skip-prop-args
- spec: `spec/10-kernel/16-observational.md` par. 1.2, 8.2
- given: `f g : ((x:A) -> P x -> B)` where `P x : Omega`;
  `f a p` vs `g a q` at type `B[a/x]` where `p /= q` as terms but
  both `p, q : P a`
- expect: `f a p ≡ g a q` (if `f ≡ g` pointwise and `a` matches);
  the propositional argument `p`/`q` is skipped
- why: propositional-argument-skip shortcut. The arguments at Omega
  type are exempt from structural comparison. This saves agents from
  synthesising coherence proofs.

---

## Acceptance criterion: Eq-by-type computes (frame par. 2 item 1, README #10-11)

### observational/funext-definitional (soundness)
- spec: `spec/10-kernel/16-observational.md` par. 2.2
- given: `f g : (x : A) -> B x` where `A : Type 0`, `B : A -> Type 0`;
  `Eq ((x:A) -> B x) f g`
- expect: **reduces-to** `(x : A) -> Eq (B x) (f x) (g x)`
- why: Eq at Pi is pointwise equality -- funext is definitional.
  The result type is in Omega. Test with both neutral `f`/`g`
  (reduction still applies to the Eq node) and lambda-headed `f`/`g`.
  Frame AC: "exercise with open terms and >=2 distinct type variables."

### observational/funext-with-levels
- spec: `spec/10-kernel/16-observational.md` par. 2.2
- given: `A : Type 1`, `B : A -> Type 2`, `f g : (x:A) -> B x`;
  `Eq ((x:A) -> B x) f g`
- expect: **reduces-to** `(x : A) -> Eq (B x) (f x) (g x)` at
  `Omega` (level poly)
- why: Eq-by-type works at higher universe levels. The result is
  level-polymorphic Omega.

### observational/propext-definitional (soundness)
- spec: `spec/10-kernel/16-observational.md` par. 2.2
- given: `P Q : Omega`; `Eq Omega P Q`
- expect: **reduces-to** `(P -> Q) and (Q -> P)`
- where: `P -> Q := (x : P) -> Q` (par. 1.3), `and` is Sigma
- why: Eq at Omega is mutual implication -- propext is definitional.
  Test with neutral `P`/`Q` and with known Omega propositions.

### observational/eq-inductive-same-ctor
- spec: `spec/10-kernel/16-observational.md` par. 2.2
- given: `data Nat : Type 0 where { zero : Nat ; suc : Nat -> Nat }`;
  `Eq Nat (suc (suc zero)) (suc x)` where `x : Nat` in context
- expect: **reduces-to** `Eq Nat (suc zero) x` (conjunction of one
  argument equality for `suc`)
- why: same constructor => conjunction of argument-equalities. One
  recursive argument for `suc`.

### observational/eq-inductive-diff-ctor
- spec: `spec/10-kernel/16-observational.md` par. 2.2
- given: `data Nat : Type 0 where { zero : Nat ; suc : Nat -> Nat }`;
  `Eq Nat zero (suc n)` where `n : Nat`
- expect: **reduces-to** `Bottom` (i.e. `Empty`)
- why: different constructors => the empty proposition. No possible
  proof.

---

## Acceptance criterion: cast regularity and computation (frame par. 2 items 2-3, README #12)

### observational/cast-refl (soundness)
- spec: `spec/10-kernel/16-observational.md` par. 3.2
- given: `A : Type 0`, `a : A`; `cast A A refl a`
- expect: **reduces-to** `a`
- why: regularity -- cast on a reflexive type equality is the identity.
  Test with neutral `A` (cast stays neutral) and canonical `A` (cast
  reduces). Frame AC: "exercise with >=2 distinct type variables."

### observational/cast-computes-pi
- spec: `spec/10-kernel/16-observational.md` par. 3.2
- given: `f : (x : A1) -> B1 x` where `Eq Type A1 A2` and
  `Eq ((x:A1) -> B1 x) ((x:A2) -> B2 x)` are canonical
  (built from structural components); `cast ((x:A1)->B1 x) ((x:A2)->B2 x)
  proof f` applied to `x : A2`
- expect: **reduces** -- the cast produces a lambda that computes when
  applied
- why: cast-by-type at Pi decomposes and transports. This is the
  canonicity test for cast at Pi.

### observational/cast-computes-sigma
- spec: `spec/10-kernel/16-observational.md` par. 3.2
- given: `p : (x : A1) x B1 x`; canonical type equality
  `Eq Type ((x:A1)xB1 x) ((x:A2)xB2 x)`;
  `cast ((x:A1)xB1 x) ((x:A2)xB2 x) proof p`
- expect: **reduces-to** a pair (constructor form), not stuck
- why: cast-by-type at Sigma decomposes into componentwise casts.
  Canonicity: the result is a constructor form.

### observational/cast-computes-inductive
- spec: `spec/10-kernel/16-observational.md` par. 3.2
- given: `data Vec (A : Type 0) : Nat -> Type 0 where
  { vnil : Vec A zero ; vcons : (n:Nat) -> A -> Vec A n -> Vec A (suc n)
  }`; canonical type equality `Eq Type (Vec A n) (Vec A m)` with
  `e_nm : Eq Nat n m`; `cast (Vec A n) (Vec A m) proof (vcons n a xs)`
- expect: **reduces-to** `vcons m (cast A A refl a) (cast ... xs)`
  (constructor form, with recursive casts)
- why: cast-by-type at inductive preserves constructor structure. Each
  argument is transported individually.

### observational/cast-computes-quotient
- spec: `spec/10-kernel/16-observational.md` par. 3.2
- given: `A/R : Type 0`, `a : A`; canonical type equality
  `Eq Type (A/R) (A/R)`; `cast (A/R) (A/R) proof [a]`
- expect: **reduces** (class preserved)
- why: cast at quotient preserves the class structure, transporting
  the representative.

---

## Acceptance criterion: J on non-refl (frame par. 2 item 4, README #13)

### observational/j-on-refl (soundness, beta)
- spec: `spec/10-kernel/15-identity.md` par. 4.2
- given: `J A a P d a (refl a)` well-typed
- expect: **reduces-to** `d`
- why: J-beta -- the base computation rule for the eliminator.

### observational/j-nonrefl (soundness, the headline)
- spec: `spec/10-kernel/15-identity.md` par. 4.3
- given: `J A a P d b e` where `e : Eq A a b` is a canonical non-refl
  equality (e.g. `trans (refl a) (refl b)` or a proof built by
  `cong` on a constructor)
- expect: **reduces** -- to a constructor/lambda/pair form, **not** stuck
  at a neutral `J` node
- why: J on non-refl MUST reduce via cast. Fails on any kernel that
  only reduces J on refl. This is the headline non-reproduction of
  the prototype's J-only-on-refl gap. Test with at least two distinct
  non-refl shapes (symmetry, transitivity, congruence).

---

## Acceptance criterion: Quotients (frame par. 2 item 7, README #14)

### observational/quotient-eq (soundness)
- spec: `spec/10-kernel/16-observational.md` par. 5
- given: `R : A -> A -> Omega`, `a b : A`; `Eq (A / R) [a] [b]`
- expect: **reduces-to** `R a b`
- why: quotient equality IS the user relation. Definitional, no setoid
  boilerplate.

### observational/quotient-elim (soundness)
- spec: `spec/10-kernel/16-observational.md` par. 5
- given: `M : (z : A/R) -> Type 0`, `f : (x:A) -> M [x]`,
  `r` (respect proof), `a : A`; `elim_/ M f r [a]`
- expect: **reduces-to** `f a`
- why: quotient eliminator computes on a class -- the i-reduction.

### observational/quotient-elim-omega-free
- spec: `spec/10-kernel/16-observational.md` par. 5
- given: `M z : Omega` for all `z : A/R`; `f : A -> M [_]`;
  `elim_/ M f (free by Omega-PI) [a]` -- the respect proof is
  auto-filled
- expect: **accepts** and **reduces-to** `f a`
- why: respect-free elimination when the target is in Omega. No manual
  coherence proof required.

---

## Acceptance criterion: Truncation (frame par. 2 item 8)

### observational/trunc-elim
- spec: `spec/10-kernel/16-observational.md` par. 6
- given: `P : Omega`, `f : A -> P`, `a : A`; `elim_trunc P f |a|`
- expect: **reduces-to** `f a`
- why: truncation eliminator computes. Since `P : Omega`, no respect
  condition is required.

### observational/trunc-or-exists
- spec: `spec/10-kernel/16-observational.md` par. 6
- given: `P :|| Nat + Bool ||` (i.e. `P or` with `Nat` and `Bool`),
  eliminate into `Unit : Omega`
- expect: **accepts** and **computes** (elimination works)
- why: truncation backs Omega's or/exists. Derived operations
  must compute.

---

## Acceptance criterion: UIP (frame par. 2 item 6)

### observational/uip-definitional
- spec: `spec/10-kernel/15-identity.md` par. 5, `16` par. 1.2
- given: `p q : Eq A a b`; conversion check `p` vs `q` at type
  `Eq A a b : Omega`
- expect: **convertible** (definitionally equal)
- why: Eq lands in Omega, so there is no nontrivial equality of
  equalities. UIP holds definitionally.

---

## Regression: K1 conformance unchanged

### observational/k1-subset-still-green
- spec: `spec/10-kernel/README.md` par. 6, K1 commitments #1-8
- given: all K1-scoped seed cases from `../seed-k1.md` + untagged
  `../seed-kernel.md` cases
- expect: **all pass** (K2 does not regress K1)
- why: K2 extends, does not rewrite, K1's check/infer/whnf/conv.
  The entire K1 conformance suite must stay green.
