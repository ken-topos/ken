# Kernel-1 conformance — K1-scoped seed cases

Format: `../README.md`. These are the K1-scoped seed cases for the core
dependent type theory (syntax, universes, Π/Σ, inductive families),
covering all 8 acceptance criteria from the Steward frame
(`../../docs/program/wp/K1-core-type-theory.md §2`).

Cases tagged [K2], [K2c], or [K-api] in `seed-kernel.md` are out of scope
here — they are acceptance criteria for later WPs. This file is the K1
subset that must pass before K1 ships.

---

## Acceptance criterion 1 — No Type : Type (AC-1)

Spec: `spec/10-kernel/12-universes.md §1, §6`; frame §2 item 1.

### kernel/universes/type-in-type-rejected (soundness)
- spec: `spec/10-kernel/12-universes.md §1`
- given: a derivation asserting `Type ℓ : Type ℓ` (same level)
- expect: **rejects** (universe inconsistency / level mismatch)
- why: no `Type:Type` (Girard's paradox). The defining rule is
  `Type ℓ : Type (suc ℓ)`; a self-loop breaks the hierarchy and
  makes the system inconsistent. Frame AC-1.

### kernel/universes/hierarchy-well-founded
- spec: `spec/10-kernel/12-universes.md §1`
- given: term `Type 0` checked against `Type 1`; term `Type 1` checked
  against `Type 2`
- expect: **both accept** (each universe lives in the next)
- why: the predicative hierarchy is well-founded: `Type 0 : Type 1 :
  Type 2 : …`. The step is always `suc`, never a loop. Frame AC-1.

---

## Acceptance criterion 2 — Genuinely dependent Σ (AC-2)

Spec: `spec/10-kernel/13-pi-sigma.md §2`; frame §2 item 2.

### kernel/pi-sigma/dependent-sigma-formation
- spec: `spec/10-kernel/13-pi-sigma.md §2`
- given: `A : Type 0`, `Vec A : Nat → Type 0` in context; form
  `(n : Nat) × Vec A n`
- expect: **accepts** at `Type (max 0 0) = Type 0`
- why: Σ is genuinely dependent — the second component's type `Vec A n`
  mentions the first component `n`. A non-dependent Σ would either
  reject this or type `p.2` at `Vec A 0` regardless of `p.1`. The
  predicative `max` rule applies. Frame AC-2.

### kernel/pi-sigma/dependent-second-projection
- spec: `spec/10-kernel/13-pi-sigma.md §2`
- given: `p : (n : Nat) × Vec A n` well-typed in context; the term `p.2`
- expect: **accepts** with type `Vec A p.1` (dependent second projection)
- why: `p.2`'s type substitutes `p.1` for `n` in `B = Vec A n`, giving
  `Vec A p.1`. This is the defining property of genuinely dependent Σ.
  Frame AC-2.

---

## Acceptance criterion 3 — Π β/η and Σ projection-η (AC-3)

Spec: `spec/10-kernel/13-pi-sigma.md §1–2, §6`; frame §2 item 3.

### kernel/pi-sigma/pi-beta
- spec: `spec/10-kernel/13-pi-sigma.md §1`
- given: `(λ (x : A). t) a` well-typed (Γ ⊢ λx.t : (x:A)→B, Γ ⊢ a : A)
- expect: **definitionally equal to** `t[a/x]` at type `B[a/x]`
- why: Π-β is the basic computation rule for functions. Conversion must
  treat the redex and its reduct as definitionally equal. Frame AC-3.

### kernel/pi-sigma/pi-eta
- spec: `spec/10-kernel/13-pi-sigma.md §1`
- given: `f : (x : A) → B` well-typed in context
- expect: `f ≡ λ (x : A). f x` **holds definitionally**
- why: Π-η is type-directed: two functions are convertible iff they
  agree on a fresh variable. η makes `λx. f x ≡ f` hold without a
  proof. Frame AC-3, README §6.3.

### kernel/pi-sigma/sigma-projection-beta
- spec: `spec/10-kernel/13-pi-sigma.md §2`
- given: `(a , b).1` and `(a , b).2` where the pair is well-typed
- expect: `(a , b).1 ≡ a : A` and `(a , b).2 ≡ b : B[a/x]`
  **definitionally**
- why: projection-β makes pair elimination compute. Frame AC-3.

### kernel/pi-sigma/sigma-eta
- spec: `spec/10-kernel/13-pi-sigma.md §2`
- given: `p : (x : A) × B` well-typed in context
- expect: `p ≡ (p.1 , p.2)` **holds definitionally** (surjective pairing)
- why: Σ-η is the record/negative-type η rule. With it, one-field
  records and their projections round-trip definitionally.
  `OQ-η-records` (DECIDED): η is for records/Σ only, not for `data`.
  Frame AC-3.

---

## Acceptance criterion 4 — Inductive eliminator ι + dependent eliminator (AC-4)

Spec: `spec/10-kernel/14-inductive.md §3, §7`; frame §2 item 4.

### kernel/inductive/elim-nat-iota-zero
- spec: `spec/10-kernel/14-inductive.md §3, §7.4`
- given: `elim_Nat M z s zero` well-typed (motive `M : Nat → Type`,
  base `z : M zero`, step `s : (n:Nat) → M n → M (suc n)`)
- expect: **reduces-to** `z` (ι)
- why: ι-reduction for the zero constructor — no recursive arguments,
  so the method `z` is the result directly. Frame AC-4.

### kernel/inductive/elim-nat-iota-suc
- spec: `spec/10-kernel/14-inductive.md §3, §7.4`
- given: `elim_Nat M z s (suc n)` well-typed
- expect: **reduces-to** `s n (elim_Nat M z s n)` (ι)
- why: ι-reduction for the suc constructor inserts one induction
  hypothesis `elim_Nat M z s n` for the recursive argument `n`.
  The eliminator computes structurally. Frame AC-4.

### kernel/inductive/elim-vec-iota-vcons
- spec: `spec/10-kernel/14-inductive.md §3, §7.5`
- given: `elim_Vec M vn vc (suc n) (vcons A n a xs)` well-typed
  (dependent eliminator for `Vec A : Nat → Type`)
- expect: **reduces-to** `vc n a xs (elim_Vec M vn vc n xs)` (ι)
- why: `Vec` is a dependent (indexed) family — its eliminator's motive
  `M : (n:Nat) → Vec A n → Type` depends on both the index and the
  scrutinee. `vcons` carries one recursive argument `xs : Vec A n` at
  index `n`, producing one induction hypothesis. Frame AC-4.

### kernel/inductive/elim-vec-type-checks
- spec: `spec/10-kernel/14-inductive.md §3`
- given: a use of `elim_Vec` to prove a property of a vector by
  induction (e.g. length-indexed map preserves length)
- expect: **accepts** — the dependent eliminator type-checks
- why: the eliminator's motive depends on the family's indices *and*
  the scrutinee, which is the signature of dependent induction. A
  non-dependent recursor would not accept this. Frame AC-4.

---

## Acceptance criterion 5 — Strict positivity (AC-5)

Spec: `spec/10-kernel/14-inductive.md §2, §8`; frame §2 item 5.

### kernel/inductive/positive-nat-admitted
- spec: `spec/10-kernel/14-inductive.md §2, §8.3`
- given: declaration `data Nat : Type 0 where { zero : Nat ; suc :
  Nat → Nat }`
- expect: **accepted** (strictly positive)
- why: `Nat`'s only recursive occurrence (`suc : Nat → Nat`) has `Nat`
  as target (positive position). Passes the check algorithm of §8.
  Frame AC-5.

### kernel/inductive/positive-list-admitted
- spec: `spec/10-kernel/14-inductive.md §1, §2`
- given: declaration `data List (A : Type ℓ) : Type ℓ where { nil :
  List A ; cons : A → List A → List A }`
- expect: **accepted** (strictly positive)
- why: `List A` appears only as the target of arrows in `cons`, which
  is the strictly-positive pattern. Parameters (`A`) are orthogonal.
  Frame AC-5.

### kernel/inductive/negative-bad-rejected
- spec: `spec/10-kernel/14-inductive.md §2, §8.3`
- given: declaration `data Bad : Type 0 where { mk : (Bad → Bool) →
  Bad }`
- expect: **rejected** at admission (non-strictly-positive occurrence)
- why: `Bad` appears to the left of an arrow in `Bad → Bool` (domain of
  the constructor argument), which is a negative position. The
  strict-positivity check (§8) flips polarity under the arrow and
  rejects the negative occurrence. Admitting it would allow a
  non-terminating, inconsistent fixpoint. Frame AC-5.

### kernel/inductive/negative-under-pi-rejected
- spec: `spec/10-kernel/14-inductive.md §2, §8.3`
- given: declaration `data Bad2 : Type 0 where { mk : (Bad2 → Bool) →
  Bad2 }` — `Bad2` appears in the domain of the constructor argument
  (to the left of an arrow), a negative position.
- expect: **rejected** at admission (non-strictly-positive occurrence)
- why: under `+` polarisation, `(x : Bad2) → Bool` checks the domain at
  `-` (flipped), and `D` at `-` polarity is always rejected. Frame AC-5.

### kernel/inductive/nested-negative-in-application-rejected
- spec: `spec/10-kernel/14-inductive.md §8.1–8.3`
- given: declaration `data Bad3 : Type 0 where { mk : Pair (Bad3 → Empty)
  Unit → Bad3 }` — `Bad3` occurs in the argument of an application (`Pair
  (Bad3 → Empty) Unit`), hidden from the structural polarisation check.
- expect: **rejected** at admission (non-strictly-positive occurrence in
  application argument)
- why: the positivity algorithm's `occurs` guard on `C u` inspects the
  application argument `u` and finds `Bad3` there (negatively, under the
  arrow in `Bad3 → Empty`). Without this guard the algorithm would
  recurse only into the head `Pair`, return true, and admit the paradox.
  Frame AC-5; Architect review blocker.

### kernel/inductive/d-in-own-indices-rejected
- spec: `spec/10-kernel/14-inductive.md §8.1–8.3`
- given: declaration `data Bad4 : (Bad4 → Empty) → Type 0 where { mk :
  Bad4 Empty }` — `Bad4` occurs negatively in its own index telescope.
- expect: **rejected** at admission (non-strictly-positive occurrence
  in indices)
- why: the `occurs` guard on `D Δ_p t̄` inspects the index tuple `t̄` and
  finds `Bad4` there. Frame AC-5; Architect review blocker.

---

## Acceptance criterion 6 — Subject reduction on K1 fragment (AC-6)

Spec: `spec/10-kernel/13-pi-sigma.md §6.4`, `14-inductive.md §9.1`;
frame §2 item 6.

### kernel/conversion/subject-reduction-pi-beta
- spec: `spec/10-kernel/13-pi-sigma.md §6.4`
- given: `Γ ⊢ (λ (x:A). t) a : B[a/x]`; reduction `(λx.t) a ⇝ t[a/x]`
- expect: `Γ ⊢ t[a/x] : B[a/x]` (reduct has same type)
- why: Π-β preserves type by the substitution lemma — typing is closed
  under well-typed substitution. Frame AC-6.

### kernel/conversion/subject-reduction-sigma-beta
- spec: `spec/10-kernel/13-pi-sigma.md §6.4`
- given: `Γ ⊢ (a , b).1 : A` and `Γ ⊢ (a , b).2 : B[a/x]`
- expect: `Γ ⊢ a : A` and `Γ ⊢ b : B[a/x]` (reducts have same types)
- why: Σ-β reductions return the constructor arguments, whose types
  match the projection types by inversion. Frame AC-6.

### kernel/conversion/subject-reduction-iota-nat
- spec: `spec/10-kernel/14-inductive.md §9.1`
- given: `Γ ⊢ elim_Nat M z s (suc n) : M (suc n)`
- expect: `Γ ⊢ s n (elim_Nat M z s n) : M (suc n)` (ι-reduct has same
  type)
- why: the method type for `suc` guarantees that `s n h : M (suc n)`
  when `h : M n`, and `elim_Nat M z s n : M n` by the induction
  hypothesis. Frame AC-6.

### kernel/conversion/subject-reduction-k1-property (property test)
- spec: `spec/10-kernel/13-pi-sigma.md §6.4`, `14-inductive.md §9.1`
- given: randomly generated well-typed K1 terms (Π, Σ, inductive
  families, their eliminators) reduced by one β/η/ι/δ step
- expect: the reduct type-checks in the same context at the same type
  (up to conversion)
- why: subject reduction must hold across ALL K1 reduction rules, not
  just individual cases. A property test fuzzes this by generating
  random well-typed K1 terms and verifying every reduction step
  preserves typing. Any regression in typing rules or reduction
  breaks this. Frame AC-6.

---

## Acceptance criterion 7 — Decidable checking on K1 fragment (AC-7)

Spec: `spec/10-kernel/13-pi-sigma.md §6.2–6.3`, `14-inductive.md
§9.2–9.3`; frame §2 item 7.

### kernel/conversion/beta-reduction-terminates
- spec: `spec/10-kernel/13-pi-sigma.md §6.2`, `14-inductive.md §9.2`
- given: a K1 term `t` with β-redexes (Π-β, Σ-β₁, Σ-β₂); repeatedly
  apply leftmost-outermost reduction
- expect: reduction **terminates** (reaches a form with no β-redexes)
- why: each β-step removes a λ or pair node and a projection, strictly
  decreasing term size. K1 has no recursive letrec, so β-reduction
  always bottoms out. Frame AC-7.

### kernel/conversion/eta-expansion-terminates
- spec: `spec/10-kernel/13-pi-sigma.md §6.2`
- given: compare `f` and `g` at a Π-type `(x:A)→B` with nested η-
  opportunities
- expect: η-expansion **terminates** in finite steps
- why: η-expansion descends structurally on the type (domain/codomain
  for Π; component types for Σ), and K1 types are finite trees.
  Frame AC-7.

### kernel/conversion/iota-reduction-terminates
- spec: `spec/10-kernel/14-inductive.md §9.2`
- given: an eliminator `elim_D` applied to a constructor-headed
  scrutinee; recursively reduce the resulting method applications
- expect: ι-reduction **terminates**
- why: each ι-step replaces the eliminator on a constructor with method
  applications whose recursive calls `elim_D … a_j` are on
  **structurally smaller** sub-values `a_j`. Structural decrease
  guarantees finite descent. Frame AC-7.

### kernel/conversion/delta-unfolding-terminates
- spec: `spec/10-kernel/13-pi-sigma.md §6.2`, `14-inductive.md §9.2`
- given: a chain of transparent definitions `c1 := … c2 …`, `c2 := …`
  in an acyclic environment; unfold repeatedly
- expect: δ-unfolding **terminates**
- why: the global environment is acyclic and append-only (`11 §4`), and
  the conversion checker memoises unfolded constants. No infinite
  unfolding is possible. Frame AC-7.

### kernel/conversion/checking-terminates-k1 (property test)
- spec: `spec/10-kernel/14-inductive.md §9.3`
- given: `check`/`infer` invoked on a suite of K1 terms exercising all
  formers (Π, Σ, universes, inductives, eliminators) in valid
  contexts
- expect: all calls **terminate** with accept/reject (no infinite loops,
  no stack overflow from recursive conversion)
- why: the K1 type-checker is syntax-directed and conversion terminates
  by structural decrease across all reduction forms (β/η/ι/δ). The
  full SCT termination argument is K2c; K1's conversion is
  structurally recursive and decidable on its own rules. Frame AC-7.

---

## Acceptance criterion 8 — K1 conformance subset passes (AC-8)

Spec: frame §2 item 8.

### kernel/conformance/k1-subset-green
- spec: `spec/10-kernel/README.md §6` (commitments table),
  `../../docs/program/wp/K1-core-type-theory.md §2`
- given: all K1-scoped seed cases in this file plus the K1 cases from
  `seed-kernel.md` (untagged cases: `type-in-type-rejected`,
  `predicative-pi`, `dependent-second-projection`, `eta`,
  `elim-computes`)
- expect: **all pass**; linting and CI are green; build/test via
  `scripts/ken-cargo -p kernel`
- why: the K1 subset is the gating condition for the K1 release. Every
  case exercises a specific commitment from `README.md §6`. A single
  failure blocks the merge. Frame AC-8.

---

## Cross-reference: seed-kernel.md untagged (K1-scope) cases

These cases in `seed-kernel.md` are K1-scope (no phase tag) and are
equivalent / complementary to the cases above. The build team should
implement the union of both files' K1 cases:

- `kernel/universes/type-in-type-rejected` — same as §AC-1 case above
- `kernel/universes/predicative-pi` — predicative max at Π formation
- `kernel/pi-sigma/dependent-second-projection` — same as §AC-2 case
  above
- `kernel/pi-sigma/eta` — Π and Σ η (complementary to §AC-3)
- `kernel/inductive/elim-computes` — same as §AC-4 `elim-nat-iota-suc`
  above

The K1 subset is: 5 seed-kernel.md cases (untagged) + 28 seed-k1.md
cases above = 33 seed cases total.
