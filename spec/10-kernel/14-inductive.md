# Inductive families

> Status: **K1 elaborated**. Normative. Declaration of inductive types, the
> strict-positivity requirement, the dependent eliminator and its ι-computation,
> and how primitive types attach. Identity is **not** a plain inductive in Ken —
> it is observational `Eq` (`15`, `16`); this chapter is the machinery `Eq`'s
> `J` and everything else reuse. §§7–9 add algorithmic ι-reduction, the
> strict-positivity check algorithm, K1 subject reduction, and the termination
> argument for K1-scoped conversion.

## 1. Declarations

An **inductive family** is declared in the global environment (`11 §4`):

```
data D (Δ_p) : (Δ_i) → Type ℓ where
  c₁ : (Δ₁) → D Δ_p t̄₁
  …
  cₙ : (Δₙ) → D Δ_p t̄ₙ
```

- `Δ_p` — **parameters**: fixed across the whole family and across all
  constructors (e.g. the `A` in `List A`).
- `Δ_i` — **indices**: may vary per constructor (e.g. the length in `Vec A n`).
  The family lands in `(Δ_i) → Type ℓ`.
- Each **constructor** `cₖ : (Δₖ) → D Δ_p t̄ₖ` takes arguments `Δₖ` (which may
  include recursive occurrences of `D`, subject to §2) and targets `D` at the
  *same parameters* `Δ_p` and *some* index instance `t̄ₖ`.
- `ℓ` is the family's universe level; constructor argument types must live at
  `ℓ` or below (predicativity, `12 §2`).

The kernel admits the declaration only if it passes (a) ordinary type-checking
of all constructor signatures in context `Δ_p`, (b) the **strict-positivity**
check (§2), and (c) universe-level checks. On admission the kernel adds: the
type former `D`, the constructors `cₖ`, and the **eliminator** `elim_D` (§3).

### Canonical examples (elaborated forms)

```
data Empty :            Type 0 where           -- ⊥, no constructors
data Unit  :            Type 0 where  tt : Unit -- ⊤
data Bool  :            Type 0 where  true : Bool ;  false : Bool
data Nat   :            Type 0 where  zero : Nat ;   suc : Nat → Nat
data List (A : Type ℓ): Type ℓ where  nil : List A ; cons : A → List A → List A
data Vec  (A : Type ℓ): Nat → Type ℓ where
  vnil  :                          Vec A zero
  vcons : (n : Nat) → A → Vec A n → Vec A (suc n)
```

`Vec` shows a genuine **index** (`Nat`) varying per constructor — an *indexed*
family, the feature that makes length-indexed vectors, well-typed syntax trees,
and the like expressible.

## 2. Strict positivity

To keep the logic consistent (no encoding of a fixpoint that inhabits `Empty`),
every recursive occurrence of `D` in a constructor argument MUST be **strictly
positive**: `D` may appear only as the *target* of a (possibly dependent)
function type, never to the *left* of an arrow, and never under another type
former applied to `D` in a non-positive position.

- **Allowed:** `A → List A → List A` (recursive arg `List A` is itself the
  type); `(n : Nat) → Vec A n → …` (recursive arg under a Π whose codomain is
  the recursive type); `(Nat → D) → D` (recursive occurrence strictly positive
  in the hypothesis — a branching/`W`-style argument).
- **Rejected:** `(D → Bool) → D` — `D` occurs to the left of an arrow
  (negative); admitting it would let one build a non-terminating, inconsistent
  fixpoint.

The kernel MUST run the strict-positivity check on every declaration and reject
negative occurrences. This check, plus the structural eliminator (§3), is what
guarantees the inductive fragment is total *without* needing the SCT machinery —
SCT (`17 §SCT`) is for *general* recursive definitions made via δ, not for
eliminator-based ones.

## 3. The dependent eliminator

For an inductive `D` the kernel generates one **dependent eliminator**
(induction principle) `elim_D`. It is the *only* primitive way to consume a
value of `D`; `match` and structural recursion at the surface
(`../30-surface/34-data-match.md`) elaborate to it
(`../30-surface/39-elaboration.md`).

**Shape.** Given a **motive**

```
M : (Δ_i) → D Δ_p Δ_i → Type ℓ'
```

(the result type, allowed to depend on the indices and the scrutinee — this
dependency is what makes it an *induction* principle, not just a recursor), and
one **method** `mₖ` per constructor giving the result for that constructor —
including, for each recursive argument, the **induction hypothesis** (the motive
already applied to that sub-value) — the eliminator has type:

```
elim_D : (M : (Δ_i) → D Δ_p Δ_i → Type ℓ')
       → (m₁ : ⟦method type for c₁⟧)
       → …
       → (mₙ : ⟦method type for cₙ⟧)
       → (i̅ : Δ_i) → (x : D Δ_p i̅) → M i̅ x
```

The method type for a constructor `cₖ : (Δₖ) → D Δ_p t̄ₖ` is: abstract over `Δₖ`,
add an induction hypothesis `M (…) r` for each recursive argument `r` in `Δₖ`,
and conclude `M t̄ₖ (cₖ …)`.

**Computation (ι).** On a constructor the eliminator reduces to the
corresponding method, applied to the constructor's arguments and the recursive
results:

```
elim_D M m̄ i̅ (cₖ ā)  ≡  mₖ ā [elim_D M m̄ … r  for each recursive r in ā]   (D-ι)
```

i.e. each recursive argument `r` is replaced in the method by `elim_D M m̄ … r` —
the structural recursive call. Because recursion is only ever on *structurally
smaller* sub-values, ι-reduction terminates; this is the totality of the
eliminator.

**Example (Nat).**

```
elim_Nat : (M : Nat → Type ℓ')
         → M zero
         → ((n : Nat) → M n → M (suc n))
         → (n : Nat) → M n
elim_Nat M z s zero      ≡  z
elim_Nat M z s (suc n)   ≡  s n (elim_Nat M z s n)
```

**Large elimination.** The motive may land in any `Type ℓ'`, including a
universe — so one may compute *types* by recursion on data (e.g. `if b then A
else B`, `elim_Bool`-style). Predicativity (`12`) keeps this sound; there is no
special restriction beyond the universe-level checks.

## 4. Σ as a record; relationship to Π

`Σ` (`13`) is presented natively (negatively, with η) rather than as an
inductive, because surjective-pairing η is wanted definitionally and a positive
inductive `Σ`'s η is only propositional. A *positive* inductive presentation
`data Σ' (A)(B) where pair : (a:A) → B a → Σ' A B` is **derivable** and
inter-derivable up to `Eq`, but the kernel's primitive Σ is the negative one
(`13 §2`).

**Definitional η is the `record` knob, not the `data` knob (`OQ-η-records`,
DECIDED).** η belongs to the **record / Σ class** — the negative,
projection-based presentation: a `record` (one field or many) elaborates to
right-nested Σ (`13 §3`) and inherits η, so `mk r.a r.b ≡ r` definitionally.
**`data` declarations — including single-constructor ones — do *not* get
definitional η**; if you want η on a wrapper, declare it a `record`, not a
`data`. This is deliberate, not an omission:

- It keeps the kernel's η rule to **one class** (negative records/Σ) — the
  type-directed machinery already needed for Σ (`17 §2`), not a new feature.
- It is **safe by construction**: records are finite nested Σ and therefore
  **never recursive**, so record-η always terminates; recursive
  single-constructor types must be `data` (η-free), sidestepping the well-known
  undecidability of η on recursive/coinductive records. (This is why a blanket
  "η for all single-constructor inductives" is *not* adopted.)
- It is **low-cost under observational equality**: even without η, `Eq` at a
  record type computes componentwise (`16 §2`), so `mk r.a r.b` and `r` are
  propositionally equal *and that equality reduces to `refl`* — η just makes it
  definitional. So `data` types lose little by lacking η.

The split matches Agda (`record` has η, `data` does not) and Lean's structure-η.

## 5. Primitive types

Machine types — `Int`, `Decimal`, `Float`, `Bytes`, …
(`../30-surface/35-numbers.md`, `../40-runtime/41-values.md`) — are **not**
inductive declarations (you cannot enumerate every `Int` with constructors).
They attach as **primitives** (`11 §4`):

```
Σ, (Int : Type 0 := prim …)               -- an opaque primitive type
Σ, (add : Int → Int → Int := prim …)      -- a primitive op + reduction rule
```

- A primitive type is an **opaque type constant**; it has no kernel-level
  constructors or eliminator. Its inhabitants are **literals** (introduced by
  the elaborator as opaque primitive values) and the results of primitive
  operations.
- A primitive operation carries a **registered reduction** `prim` (`41`):
  applied to literal arguments it computes a literal result *in the kernel's
  evaluator*, so `add 2 3 ≡ 5` holds definitionally and proofs can compute over
  literals. On non-literal (stuck) arguments it is a neutral term.
- Primitives are **trusted**: a wrong primitive reduction is a soundness bug, so
  the set of primitives is small, audited, and part of the kernel's trusted base
  (listed in `18 §5`). This is the one place computation enters the kernel from
  outside the term language; everything else is β/ι/δ/obs.
- Equational properties of primitives that are *not* definitional (e.g.
  commutativity of `add`) are **propositions to prove**, provided as a small
  axiomatized interface or proved against a reference model, not assumed.

## 6. What the kernel checks here

A conforming kernel MUST:

1. Type-check inductive declarations and **enforce strict positivity** (§2, §8),
   rejecting negative occurrences.
2. Generate the constructors and the **dependent** eliminator with induction
   hypotheses for recursive arguments (§3).
3. Implement **ι-reduction** of the eliminator on constructor forms (§3, §7),
   driving structural recursion; ensure it terminates (structural decrease).
4. Permit **large elimination** under the predicative universe rules (§3).
5. Treat **primitive** types/operations as opaque constants with registered,
   audited reductions (§5), never as inductives. K1 defines only the interface;
   the value model (`../40-runtime/41-values.md`, K3) and the kernel API
   (`18-judgments.md`, K-api) elaborate the registration mechanism.

Conformance: `../../conformance/kernel/inductive/` — positivity acceptance and
rejection, `elim_Nat`/`elim_Vec` ι-computation, large elimination (`elim_Bool`
into `Type`), and primitive-literal reduction (`add 2 3 ⇓ 5`).

## 7. Algorithmic ι-reduction for conversion

The ι-reduction scheme described in §3 is declarative; this section gives the
algorithmic form the conversion checker (`13-pi-sigma.md §6`) calls.

### 7.1 Eliminator application form

The eliminator is applied to `n+3` arguments: the motive `M`, one method per
constructor `m₁ … mₙ`, the index tuple `i̅`, and the scrutinee `s`:

```
elim_D : (M : (Δ_i) → D Δ_p Δ_i → Type ℓ')
       → (m₁ : MethodType(c₁, D, M))
       → …
       → (mₙ : MethodType(cₙ, D, M))
       → (i̅ : Δ_i) → (s : D Δ_p i̅) → M i̅ s
```

### 7.2 ι-redex condition

`elim_D M m̄ i̅ s` is an **ι-redex** when `s` is a constructor-headed term
`cₖ ā` for some constructor `cₖ` of `D`. The indices in the scrutinee must
match `i̅`; if they are not syntactically identical (e.g. one is `zero`, the
other is a neutral `n`), the eliminator is **stuck** (neutral) — ι does not
fire.

### 7.3 Reduction rule (algorithmic)

```
elim_D M m₁…mₙ i̅ (cₖ ā)  ⇝  mₖ ā [ih₁ … ih_p]
```

where:

- `cₖ` has constructor arguments `Δₖ = (x₁ : A₁) … (x_q : A_q)`.
- `ā = a₁ … a_q` are the actual constructor arguments.
- For each constructor argument position `j` where `A_j` is a **recursive
  occurrence** of `D` (applied to its parameters and some index), the induction
  hypothesis `ih` is:

  ```
  ih = elim_D M m₁…mₙ idx(a_j) a_j
  ```

  where `idx(a_j)` computes the index instance for that recursive argument (the
  indices at which `D` appears in `A_j`). For a simple recursive occurrence `D
  Δ_p t̄`, the indices are `t̄`; for a Π-bound recursive occurrence `(y:Y) → D Δ_p
  t̄ y`, the indices are `t̄ y` applied to the bound variable (which is abstracted
  in the method type convention of §3).

- `p` is the number of recursive positions in `cₖ`'s constructor arguments.

The reduction is **capture-avoiding**: the method `mₖ` is applied to the
constructor arguments and the induction hypotheses, with substitutions handled
by the kernel's capture-avoiding substitution engine (`11 §5`).

### 7.4 Example: Nat

```
elim_Nat M z s zero      ⇝  z
elim_Nat M z s (suc n)   ⇝  s n (elim_Nat M z s n)
```

Constructor `zero`: no recursive arguments, `p = 0`. Constructor `suc`: one
recursive argument at position 0, giving one induction hypothesis `elim_Nat M z
s n`.

### 7.5 Example: Vec

Given `Vec (A : Type ℓ) : Nat → Type ℓ`:

```
elim_Vec M vn vc zero     (vnil A)        ⇝  vn
elim_Vec M vn vc (suc n)  (vcons A n a xs) ⇝  vc n a xs (elim_Vec M vn vc n xs)
```

`vnil` has no recursive arguments. `vcons` has one recursive argument `xs : Vec
A n` at the index `n`, producing the induction hypothesis shown.

### 7.6 Stuck eliminators

When the scrutinee `s` is not a constructor — it is a variable, a neutral
application, or any term whose head is not a constructor of `D` — the
eliminator is **stuck** (neutral, no ι-reduction fires). Conversion treats it
as a neutral term: two stuck eliminators are convertible iff their scrutinees
and arguments are pointwise convertible. The full NbE in K2c (`17`) gives this
a systematic treatment in a semantic domain; K1's structural conversion handles
it via the congruence rules in `13 §6.2`.

## 8. Strict-positivity check algorithm

§2 defines *what* strict positivity means. This section gives the *how* — the
recursive descent the kernel runs at admission time.

### 8.1 Positivity judgment

For a family `D` being declared, the judgment `Pos_D^n(A)` — "`A` is positive
in `D` at polarisation `n`" — where `n ∈ {+, -}` (positive/negative
polarisation). The check starts with each constructor argument type at `n = +`
and recurses structurally:

```
Pos_D^+(D Δ_p t̄)        holds  (strictly-positive recursive occurrence)
Pos_D^+(X)              holds  if X is a parameter or a different type
Pos_D^+(A)              holds  if A is a universe Type ℓ
Pos_D^+(x : A) → B      holds  if Pos_D^-(A) and Pos_D^+(B)
Pos_D^+(x : A) × B      holds  if Pos_D^+(A) and Pos_D^+(B)

Pos_D^-(D Δ_p t̄)        FAILS  (negative occurrence — reject)
Pos_D^-(X)              holds  if X is a parameter or a different type
Pos_D^-(A)              holds  if A is a universe Type ℓ
Pos_D^-(x : A) → B      holds  if Pos_D^+(A) and Pos_D^-(B)
Pos_D^-(x : A) × B      holds  if Pos_D^-(A) and Pos_D^-(B)
```

Key: `D` may appear strictly positively (as the target of a function type under
`+` polarisation), but never under `-` polarisation. This blocks `(D → ⊥) → D`
and similar negative encodings.

### 8.2 Algorithm

```
check-positivity(D):
  for each constructor cₖ of D:
    for each argument type A_j in cₖ's telescope Δₖ:
      if not check-pos-arg(D, +, A_j):
        reject "non-strictly-positive occurrence of D in cₖ"

check-pos-arg(D, pol, A):
  match A:
    D Δ_p t̄  →  return (pol == +)     -- only positive allowed
    Type ℓ   →  return true
    X        →  return true           -- parameter or other type
    (x : C) → B  →  return check-pos-arg(D, flip(pol), C)
                    and check-pos-arg(D, pol, B)
    (x : C) × B  →  return check-pos-arg(D, pol, C)
                    and check-pos-arg(D, pol, B)
    C u      →  return check-pos-arg(D, pol, C)   -- application, recurse into head
```

where `flip(+) = -`, `flip(-) = +`.

### 8.3 Worked examples

**Accepted:**
```
data Nat : Type 0 where
  zero : Nat
  suc  : Nat → Nat
```

Constructor `zero`: no arguments, trivially positive. Constructor `suc`:
argument telescope `(n : Nat)`, argument type `Nat`. Under `+`:

```
check-pos-arg(Nat, +, Nat) → D = Nat at pol = + → true
```

**Rejected:** `data Bad = mk : (Bad → Bool) → Bad`. Argument telescope `(f : Bad
→ Bool)`, argument type `Bad → Bool = (x : Bad) → Bool`. Under `+`:

```
check-pos-arg(Bad, +, (x : Bad) → Bool)
  = check-pos-arg(Bad, -, Bad) and check-pos-arg(Bad, +, Bool)
  = false (D under -) → FAILS
```

**Rejected (negative under a Π):** `data Lam = mk : (Nat → Nat) → Nat`.
Argument telescope `(f : (Nat → Nat))`, argument type `(x : Nat) → Nat`. Under
`+`:

```
check-pos-arg(Nat, +, (x : Nat) → Nat)
  = check-pos-arg(Nat, -, Nat) and check-pos-arg(Nat, +, Nat)
  = false (D under -) → FAILS
```

Note: even though the outermost polarisation is `+`, the domain of the arrow
flips to `-`, so `Nat` appears negatively and is caught.

### 8.4 Nested and mutually-defined inductives

K1 handles **non-mutual, non-nested** inductives. Mutually-defined inductive
families and nested inductive occurrences (e.g. `data Rose A = node : A → List
(Rose A) → Rose A`) are **K1.5** (a future extension within the inductive
chapter). K1 rejects mutual declarations and nested `D` occurrences under other
type formers. The strict-positivity algorithm above is the **base case**; the
extension to nested occurrences is a straightforward generalisation (treat
`List (Rose A)` by unfolding `List`'s definition to check positivity of `Rose`
within it) but is deferred to keep K1's kernel minimal.

## 9. K1 subject reduction and termination

### 9.1 Subject reduction for ι

**Theorem (ι subject reduction).** If `Γ ⊢ elim_D M m̄ i̅ (cₖ ā) : M i̅ (cₖ ā)`
(under ambient `Σ`) and the eliminator is applied to the constructor `cₖ` with
arguments `ā`, then `Γ ⊢ mₖ ā [ih₁ … ih_p] : M i̅ (cₖ ā)` — the ι-reduct has
the same type.

*Proof.* The typing of the eliminator application gives:

- `Γ ⊢ M : (Δ_i) → D Δ_p Δ_i → Type ℓ'` (motive well-typed).
- For each method `m_j`: `Γ ⊢ m_j : MethodType(c_j, D, M)`.
- `Γ ⊢ i̅ : Δ_i` (the index arguments inhabit the index telescope).
- `Γ ⊢ cₖ ā : D Δ_p i̅` (the scrutinee is well-typed at the given indices).

The method type for `cₖ` is defined in §3 to conclude `M t̄ₖ (cₖ …)` when
applied to the constructor arguments and the induction hypotheses. Since the
actual indices `i̅` match the constructor's index instance `t̄ₖ` (they must, for
the scrutinee to have type `D Δ_p i̅`), the method application has type `M i̅
(cₖ ā)`. The result follows.

### 9.2 Termination of K1-scoped conversion

The K1 conversion algorithm (`13 §6.2`) terminates on the K1 fragment for the
following reasons:

1. **β-reduction (Π, Σ).** Each β-redex `(λx.t) a`, `(a,b).1`, `(a,b).2` is
   eliminated in one step; the reduct is structurally smaller than the redex (a
   subterm is substituted). The conversion checker does not iterate β-reduction
   indefinitely — it reduces to a normal form using a leftmost-outermost
   strategy, and the total size of terms *strictly decreases* at each β-step
   (substitution replaces a variable with a term, but the λ binder and
   application node are removed, and K1 terms have no recursive letrec — only
   acyclic δ unfolding).

2. **η-expansion.** η-expansion (Π-η, Σ-η) is type-directed and compares
   subterms at strictly smaller types (the domain/codomain for Π; the component
   types for Σ). The type structure is finite, so η-expansion descends
   finitely.

3. **ι-reduction.** Each ι-redex `elim_D … (cₖ ā)` replaces the eliminator
   applied to a constructor with a method application. The recursive calls
   `elim_D … a_j` are on **structurally smaller** sub-values `a_j` (the
   constructor arguments that are recursive). Structural decrease guarantees
   termination: the scrutinee of each recursive call is a proper subterm of the
   original scrutinee. Because K1 terms are finite trees (no coinduction, no
   recursive letrec), structural descent bottoms out at non-recursive
   constructors.

4. **δ-unfolding.** The global environment is **acyclic** (`11 §4`), so
   unfolding a definition `c` to its body `t` replaces a constant with a term
   that may contain references to *earlier* definitions only. Chasing δ never
   loops; the conversion checker memoises unfolded constants to avoid
   re-unfolding.

5. **No Ω, Eq, cast, or quotient equations** — those are K2/K2c, and their
   termination depends on the NbE + SCT machinery of `17`.

The full SCT-gated termination argument for general recursive δ-definitions is
in K2c (`17-conversion.md §SCT`). K1's δ is only for non-recursive transparent
definitions; recursive definitions are admitted via the inductive eliminator
(whose termination is structural, not SCT-reliant) or deferred to K2c.

### 9.3 Decidable checking

**Corollary.** `check`/`infer` for the K1 fragment (Π, Σ, universes, inductive
families, and their eliminators, with K1-scoped conversion as in `13 §6`) is
decidable — it always terminates. The type-checker is syntax-directed (one rule
per term former); conversion is called at the leaves (checking inferred against
expected types) and terminates by §9.2.
