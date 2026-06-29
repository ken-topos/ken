# Kernel-2c (judgments) conformance — series-1 seed cases

Format: `../../README.md`. These pin the typing-judgment and kernel-API behavior
that K2c **series-1** touches: the **(Conv) mode switch** that invokes `17`
conversion (`18 §2`, `§3`), the **SCT admission gate** at `declare_def`
(`18 §4`, `17 §4`), and the trusted-boundary API and accounting (`18 §4`, `§5`).
The complete K-api conformance is a later WP (`18`); this seeds the
conversion-integration and admission-gate behavior series-1 delivers, per the
elaboration plan (`docs/program/wp/K2c-elaboration-plan.md §2`, `§3`).

Cases tagged **(soundness)** encode a soundness commitment
(`../../../spec/10-kernel/README.md §6`) and must never regress. Citations are
reconciled against spec-author's **landed** elaboration (`spec-author/work`
`cd3b19d`): `18-judgments.md` §2 (Conv), §3 bidirectional, §4 API, §5 trusted
base, §6 metatheory; the one `17` cite is §3.5 (δ-unfold trigger). Expected
results are determined by the on-`main` spec (`16`/`17`/`18`); none required
reading the prototype.

---

## Conversion integration — the (Conv) mode switch (plan §2; 18 §3)

(Conv) is the single place conversion (`17`) is called during checking — the
mode-switch fallback `t ⇐ A`: infer `t ⇒ A'`, then `convert(A, A')`.

### judgments/conv-switch-delta (soundness)
- spec: `18 §3` (mode switch), `§2` (Conv); `17 §3.5`
- given: transparent `N : Type 0 := Nat`; open `n : Nat`; check `n ⇐ N`
- expect: **accepts** — infer `n ⇒ Nat`; the switch calls `convert(N, Nat)`,
  which δ-unfolds `N → Nat` and converts
- why: the (Conv) switch invokes `17` conversion including controlled δ; a
  transparent type synonym is accepted because conversion unfolds it. The
  single conversion call at the switch.

### judgments/conv-switch-eta (soundness)
- spec: `18 §3`, `§2`; `17 §2`
- given: `A : Type 0`; open `f : (x:A) -> A`; check
  `(λ (x:A). f x) ⇐ (x:A) -> A`
- expect: **accepts** — checking the λ descends, and at the body the switch
  converts `f x` against itself; equivalently `f ≡ λ x. f x` by Π-η
- why: conversion invoked at checking consumes η, so an η-expanded term checks
  against the un-expanded type. Ties `18 §3` checking to `17 §2`.

### judgments/conv-switch-rejects (soundness)
- spec: `18 §3`, `§2`, `§4`
- given: open `n : Nat`; check `n ⇐ Bool`
- expect: **rejects** with a `KernelError` naming the failing subterm `n` and
  the two non-converting types (`Nat` vs `Bool`)
- why: when inferred and expected types do not convert, (Conv) fails and the
  kernel reports a minimal, precise reason (`18 §4`). Pairs with the accept
  cases.

### judgments/conv-switch-non-cumulative (soundness)
- spec: `18 §2` (non-cumulative; no subtyping); `12 §3`
- given: check `Type 0 ⇐ Type 2`
- expect: **rejects** — infer `Type 0 ⇒ Type 1`; `convert(Type 2, Type 1)` is
  false (`Type 1 ≢ Type 2`)
- why: Ken is non-cumulative; (Conv) uses **definitional equality, not
  subsumption**, and there is no subtype relation in the kernel. A universe lift
  must be explicit. A checker that accepted `Type 0 ⇐ Type 2` would be
  cumulative (OQ-2 DECIDED non-cumulative). Guards the no-subtyping invariant.

---

## SCT admission gate at declare_def (plan §2; 18 §4, 17 §4)

### judgments/declare-def-sct-admits (soundness)
- spec: `18 §4` (`declare_def` runs SCT); `17 §4`
- given: `declare_def(env, "ack", Nat -> Nat -> Nat, <Ackermann body>)` — the
  SCT-accepted lexicographic definition from
  `../conversion/seed-conversion.md` (`conversion/sct-accept-lexicographic`)
- expect: **Ok** — admitted transparent (δ-reducible); a subsequent
  `convert(Nat, ack (suc^3 zero) (suc^3 zero), suc^61 zero)` returns true
- why: `declare_def` type-checks the body **then** runs `sct_check` (`18 §4`
  step 3); on pass it admits transparent. Ties the API gate to the SCT
  criterion and to the δ-reduction it licenses.

### judgments/declare-def-sct-rejects (soundness)
- spec: `18 §4`; `17 §4`
- given: `declare_def(env, "loop", Nat -> Nat, <loop x = loop x>)`
- expect: **Err(KernelError)** — admission **refused**; `env` is unchanged;
  `loop` is not δ-reducible (it was never admitted)
- why: the kernel **never** admits uncertified transparent recursion (`17 §4`,
  `18 §4` step 5). The SCT gate runs at admission time, after type-checking the
  body. The "never loops" guarantee enforced at the trusted boundary.

### judgments/declare-def-eliminator-no-sct (soundness)
- spec: `17 §4` (scope: SCT gates δ-recursion; eliminator recursion is already
  total); `18 §4`
- given: `declare_def` of
  `double := λ n. elim_Nat (λ _. Nat) zero (λ _ ih. suc (suc ih)) n`
  (recursion via the inductive eliminator, not via a self-call)
- expect: **Ok** — admitted transparent; **no** SCT obligation arises
- why: SCT gates only **general** recursive δ definitions; recursion through an
  inductive eliminator is already structural and total (`14 §3`, `17 §4`
  scope). A checker that demanded SCT of every definition — or, worse, admitted
  a non-eliminator self-recursion **without** SCT — misclassifies this.

---

## Certificate re-checking — check_proof is check (18 §4, §5)

### judgments/certificate-recheck-valid (soundness)
- spec: `18 §4` (`check_proof ≡ check`)
- given: proposition `G := Eq Nat (add 1 1) 2` and certificate term
  `proof := refl 2`; `check_proof(proof, G)`
- expect: **Ok** — the kernel re-derives the type: `refl 2 : Eq Nat 2 2`, and
  `Eq Nat (add 1 1) 2` converts to `Eq Nat 2 2` because `add 1 1` prim-reduces
  to `2`
- why: a proof IS a term; checking it IS typing (`18 §4`). The kernel re-checks
  the prover's certificate — nothing the prover says is trusted until the kernel
  re-derives it (the de Bruijn criterion).

### judgments/certificate-recheck-rejects (soundness)
- spec: `18 §4`; `§5` (trusted base); `16 §2.2`
- given: a **false** proposition `G' := Eq Nat 1 2` and a plausible-looking
  wrong certificate `c := refl 1`; `check_proof(c, G')`
- expect: **Err** — infer `refl 1 ⇒ Eq Nat 1 1`; `convert(Eq Nat 1 2,
  Eq Nat 1 1)` is false (distinct closed numerals reduce `Eq Nat 1 2` to
  `Bottom`, uninhabited)
- why: a wrong certificate cannot make a false proposition inhabited. The
  kernel's re-check is the soundness firewall around the untrusted prover/Z3 —
  no false `proved`.

---

## Trusted-base enumeration (18 §5)

### judgments/trusted-base-enumerate (soundness)
- spec: `18 §5` (`env.trusted_base()`)
- given: an `env` with a `declare_postulate` (axiom `P`), a `declare_primitive`
  (`add`), and several `declare_def`s; call `env.trusted_base()`
- expect: **enumerates exactly** the postulate `P` and the primitive `add`
  (names + types) — and **not** the `declare_def`s
- why: soundness rests on exactly the kernel code + registered primitives +
  admitted postulates (`18 §5`); the kernel MUST be able to enumerate (2) + (3)
  so a reviewer or agent can see every unchecked assumption. Definitions are
  re-checked, not trusted, so they do not appear.

### judgments/trusted-base-idiomatic-empty (soundness)
- spec: `18 §5`
- given: an `env` built only from `declare_def` and `declare_inductive` over
  the standard primitives — no postulates
- expect: `trusted_base()` lists only the registered primitives; **no
  postulates**
- why: a program that adds no axioms depends on nothing beyond the kernel +
  primitives. Idiomatic Ken adds no postulates; any classical axiom, if used,
  appears here and is visible. Pins that inductives and definitions are
  re-checked, not trusted.

---

## Bidirectional round-trips and minimal errors (18 §3, §4)

### judgments/infer-check-roundtrip (soundness)
- spec: `18 §3`
- given: open `A : Type 0`, `a : A`, `f : (x:A) -> A`; infer `f a ⇒ A0`, then
  check `f a ⇐ A0`
- expect: infer yields `A`; check at `A` **accepts** (round-trip)
- why: infer produces the unique type; checking the term against that type
  succeeds — the two syntax-directed modes agree (`18 §3`). Open terms, an
  application spine.

### judgments/ill-typed-minimal-error (soundness)
- spec: `18 §4` (`KernelError`: failing subterm + the two types)
- given: open `f : (x:Nat) -> Nat`; the application `f true` (`true : Bool`)
- expect: **rejects** — the error names the argument `true`, expected `Nat`,
  got `Bool` (the failing subterm and the two non-converting types)
- why: application checks the argument against the domain; `Bool ≢ Nat` at the
  argument position. The error is minimal and precise (`18 §4`) — the kernel
  does no recovery, no proof search, no unification.

### judgments/no-unification (soundness)
- spec: `18 §3` (kernel receives fully-explicit core terms)
- given: a core term with a genuinely missing annotation that would require the
  kernel to **guess** (solve a metavariable) to type it
- expect: **rejects** (or errors) — the kernel does not invent the annotation
- why: the kernel performs no unification or implicit insertion (`18 §3`, `§7`);
  fully-explicit core terms are the elaborator's responsibility. A kernel that
  "helpfully" solved the gap would have absorbed elaborator logic into the TCB.
  Pins the trusted-boundary minimality.

---

## Regression: K1 + K2 judgments unchanged

### judgments/k1-k2-judgments-still-green (soundness)
- spec: `spec/10-kernel/README.md §6`
- given: all K1/K2 check/infer cases already pinned (`../seed-k1.md`,
  `../seed-kernel.md`, `../observational/seed-observational.md`)
- expect: **all pass** — K2c integrates conversion at the (Conv) switch and
  adds the SCT gate; it does not change the typing relation K1/K2 fixed
- why: the bidirectional algorithm and the API surface K1/K2 established must
  not regress; K2c only hardens the conversion call and the admission gate.
