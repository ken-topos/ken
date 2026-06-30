# Inductive families

> Status: **K1 elaborated; K1.5 extends it** (W-style recursive inductives).
> Normative. Declaration of inductive types, the strict-positivity requirement,
> the dependent eliminator and its ι-computation, and how primitive types
> attach. Identity is **not** a plain inductive in Ken — it is observational
> `Eq` (`15`, `16`); this chapter is the machinery `Eq`'s `J` and everything
> else reuse. §§7–9 add algorithmic ι-reduction, the strict-positivity check
> algorithm, K1 subject reduction, and the termination argument for K1-scoped
> conversion.
>
> **K1.5** admits **W-style (Π-bound) recursive occurrences** — a constructor
> argument that is a *function into* the recursive type, `(b:B) → D` — and
> generates the eliminator whose induction hypothesis is itself a function
> (§2.1, §3.1, §7.7, §9.4). It removes the K1-era blanket rejection of Π-bound
> recursion; strict positivity (§8) is unchanged and remains the sole structural
> admission gate. The motivating client is L5's interaction tree `ITree`
> (`../30-surface/36-effects.md`).

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
  in the hypothesis — a branching/`W`-style argument, **admitted in K1.5**,
  §2.1).
- **Rejected:** `(D → Bool) → D` — `D` occurs to the left of an arrow
  (negative); admitting it would let one build a non-terminating, inconsistent
  fixpoint.

The kernel MUST run the strict-positivity check on every declaration and reject
negative occurrences. This check, plus the structural eliminator (§3), is what
guarantees the inductive fragment is total *without* needing the SCT machinery —
SCT (`17 §SCT`) is for *general* recursive definitions made via δ, not for
eliminator-based ones.

### 2.1 W-style (Π-bound branching) recursive occurrences — admitted (K1.5)

A recursive occurrence may sit at the **target of a function type**: a
constructor argument of the shape

```
k : (b : B) → D Δ_p t̄[b]              -- B contains no occurrence of D
```

is **strictly positive** — `D` appears only as the arrow's target, never to its
left — and is therefore **sound**. This is the **W-style** (branching) argument:
`k` is a `B`-indexed family of sub-values, the branching that makes a value of
`D` a tree with `B`-many children at that node. The canonical shapes are the
`W`-type and L5's interaction tree:

```
data W (A : Type ℓ) (B : A → Type ℓ) : Type ℓ where
  sup : (a : A) → (B a → W A B) → W A B

ITree.Vis : (e : E.Op) → (E.Resp e → ITree E R) → ITree E R
```

**Admittance vs. positivity — the K1/K1.5 boundary.** Strict positivity (§2, §8)
is *necessary* for soundness but is **not, by itself, admittance**: to admit a
constructor the kernel must also **generate its eliminator**, and a W-style
argument needs an *induction hypothesis that is itself a function* (§3.1) —
machinery K1 did not build. So K1 took the conservative route: the positivity
check (§8.2) already *accepts* the W-style shape, but a **separate** admission
gate rejected every Π-bound recursive argument outright (deferring the
eliminator to this WP). **K1.5 removes that blanket gate.** Admission of a
Π-bound recursive occurrence is now exactly:

1. it is **strictly positive** — `D` is the arrow's target and `D` does **not**
   occur in any domain `B` (the polarity discipline of §8.1 already enforces
   this: a `D` in a domain is checked at `-` and fails); and
2. the domain telescope is therefore **D-free**, so the eliminator can build the
   Π-abstracted IH (§3.1) — single-level branching `(b:B) → D` and its curried/
   dependent generalisation `(b:B)(c:C[b]) → D`, where no domain mentions `D`.

**Still rejected** (unchanged): **negative** occurrences — `D` to the left of
any arrow (`(D → Bool) → D`, §8.3) — and **nested** occurrences under another
type former (`List (Rose A)`, §8.5); those stay out (the `occurs`-guard of §8.2
rejects them; nested/mutual remain a later extension). K1.5 widens admittance to
**exactly** the strictly-positive Π-bound class, no more.

**Level (predicativity) — no new rule, one instance of `14 §1`.** A W-style
argument type `(b:B) → D Δ_p t̄` lives at `max(level B, ℓ_D)`; `14 §1`'s rule
(constructor-argument types sit at the family level `ℓ_D` or below, `12 §2`)
therefore forces `level B ≤ ℓ_D`. No universe rule is added — the W-style
argument is admissible iff its domains already fit under the family's level
(e.g. `ITree`'s `Op`/`Resp` levels are absorbed into `ℓ_D = max ℓ_R ℓ_op
ℓ_resp`, `../30-surface/36-effects.md §2.1`).

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
already applied to that sub-value; §3.1 generalises this to W-style arguments,
whose IH is itself a *function*) — the eliminator has type:

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

### 3.1 W-style arguments and the Π-abstracted induction hypothesis (K1.5)

The method-type recipe above ("an induction hypothesis `M (…) r` for each
recursive argument `r`") is the **direct** case, where `r : D Δ_p t̄` is itself a
value of the family. A **W-style** argument (§2.1) is not a value of `D` but a
*function into* `D`:

```
k : (b : B) → D Δ_p t̄[b]
```

so "the motive applied to the sub-value" is not yet a type — there is no single
sub-value but a `B`-indexed family of them. The induction hypothesis is
therefore **Π-abstracted over the branching domain**:

```
ih_k : (b : B) → M t̄[b] (k b)
```

— for every branch `b`, the motive holds at that child `k b`. This is the only
new ingredient K1.5 adds to the eliminator. The method type for a constructor
`cₖ` is built by abstracting over `Δₖ` and inserting, **for each recursive
argument position**:

- **direct** `r : D Δ_p t̄`  → an IH  `M t̄ r`  (as in §3, unchanged);
- **W-style** `k : (b:B) → D Δ_p t̄[b]`  → an IH  `(b : B) → M t̄[b] (k b)`;
- **curried W-style** `k : (b:B)(c:C[b]) → D Δ_p t̄[b,c]`  → an IH
  `(b:B)(c:C[b]) → M t̄[b,c] (k b c)`  (one Π-abstraction per branching binder);

then concluding `M t̄ₖ (cₖ …)`. A direct argument is the degenerate W-style case
with an empty branching telescope.

**Computation (ι), W-style.** On a W-style constructor the eliminator threads
the recursive result **through the branching function**: the IH passed to the
method is the eliminator applied *under the branch binder*,

```
elim_D M m̄ ī (cₖ … k …)
  ≡  mₖ … (λ b. elim_D M m̄ (idx k b) (k b)) …                         (W-ι)
```

i.e. each W-style argument `k` contributes the IH term `λ b. elim_D M m̄ … (k b)`
(and `λ b c. …` for the curried case). The recursive call lands on `k b`, a
**child** of the scrutinee node `cₖ … k …`; in a finite (inductive, not
coinductive) tree it is structurally smaller, so the recursion is well-founded
(§9.4).

**Example (W-type).**

```
elim_W : (M : W A B → Type ℓ')
       → ((a : A) (k : B a → W A B) (ih : (b : B a) → M (k b)) → M (sup a k))
       → (w : W A B) → M w
elim_W M s (sup a k)  ≡  s a k (λ b. elim_W M s (k b))
```

The method `s` receives the node label `a`, the branching `k`, and the IH `ih`
as a **function** `(b : B a) → M (k b)`; the ι-rule supplies `λ b. elim_W M s (k
b)`. A method that *uses* `ih b` (rather than β-discarding it) is what makes
this an induction principle over the whole subtree — the conformance corpus pins
exactly this (an IH-discarding method must reach a *different* result).

**`elim_ITree` (the L5 client).** Specialising to `ITree E R` (§2.1) gives L5
its fold:

```
elim_ITree : (M : ITree E R → Type ℓ')
  → ((r : R) → M (Ret r))
  → ((e : E.Op) (k : E.Resp e → ITree E R)
        (ih : (x : E.Resp e) → M (k x)) → M (Vis e k))
  → (t : ITree E R) → M t
```

on which L5's `bind`/handlers/denotation are structural folds (`36 §2`, total by
§9.4, no SCT). Generating `elim_ITree` is the concrete deliverable that unblocks
L5's denotation half.

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
   rejecting negative occurrences; **admit strictly-positive W-style (Π-bound)
   recursive occurrences** (§2.1) — positivity (§8.2) is the sole structural
   gate, with no separate Π-bound rejection.
2. Generate the constructors and the **dependent** eliminator with induction
   hypotheses for recursive arguments (§3) — including the **Π-abstracted**
   induction hypothesis `(b:B) → M t̄[b] (k b)` for W-style arguments (§3.1).
3. Implement **ι-reduction** of the eliminator on constructor forms (§3, §7),
   driving structural recursion; for W-style arguments thread the recursive
   result through the branching function (`λ b. elim_D … (k b)`, §3.1, §7.7);
   ensure it terminates (structural decrease, §9).
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
`cₖ ā` for some constructor `cₖ` of `D`. ι fires on the scrutinee's head
constructor alone — it does not require the index arguments `i̅` to be
syntactically identical to the constructor's index instance. (In a well-typed
term the indices are definitionally equal to the constructor's target indices;
gating ι on syntactic identity would make conversion incomplete — valid
programs with computed indices stuck.)

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

When the scrutinee `s` is not a constructor-headed term — it is a variable,
a neutral application, or any term whose head is not a constructor of `D` —
the eliminator is **stuck** (neutral, no ι-reduction fires). Conversion treats
it as a neutral term: two stuck eliminators are convertible iff their
scrutinees and arguments are pointwise convertible. The full NbE in K2c (`17`)
gives this a systematic treatment in a semantic domain; K1's structural
conversion handles it via the congruence rules in `13 §6.2`. (The index
arguments `i̅` are part of the pointwise comparison but do not gate ι firing
— a constructor-headed scrutinee always fires ι, per §7.2.)

### 7.7 W-style ι (K1.5)

For a W-style argument position `j` — where `A_j = (b : B) → D Δ_p t̄[b]` with
`B` D-free (§2.1) — the induction hypothesis built by §7.3's index rule is
**not** a recursive call on a value but a **function**:

```
ih_j  =  λ (b : B). elim_D M m₁…mₙ idx(a_j, b) (a_j b)
```

where `a_j` is the actual W-style argument (a function term), `a_j b` applies it
to the fresh branch variable `b`, and `idx(a_j, b) = t̄[b]` is the index instance
at that branch (the §7.3 rule "`(y:Y) → D Δ_p t̄ y` ⇒ indices `t̄ y`", now read
under the binder `b`). The curried case `(b:B)(c:C[b]) → D` contributes `λ b c.
elim_D M m̄ idx(a_j,b,c) (a_j b c)`. The reduct is therefore

```
elim_D M m̄ ī (cₖ ā)  ⇝  mₖ ā [ih₁ … ih_p]
```

with each `ih` either a direct recursive call (§7.3) or a W-style λ-abstracted
call as above — selected by whether `A_j` has leading Π binders whose body head
is `D` (the same syntactic test the admission gate uses, §2.1). `recursive_args`
collection (§7.3) extends to return, for each Π-bound recursive position, the
branching telescope `(b:B…)` alongside the index expressions.

**Why conversion still decides.** Decidability rests on **finiteness of the
value**, the same structural decrease as the direct eliminator (§9.2(3)) — *not*
on the inner eliminator being stuck. ι fires on the outermost constructor and
yields the IH `λ b. elim_D M m̄ … (a_j b)`. The inner `elim_D (a_j b)` **does
fire** whenever `a_j` is a constructor-producing branching function — the
typical case: `a_j = λx. cₖ' … ⇒ a_j b ⇝ cₖ' …` is **constructor-headed even
for an abstract `b`** (the head does not depend on `b`), so ι re-fires and
recurses on `a_j b`, a **structurally smaller child** of the scrutinee (reached
*through* the branching function — one β-step on `a_j` — not directly). This
firing happens during conversion too: comparing two IHs at their Π type applies
a fresh
branch variable `b*` (η, §7.6) and drives exactly this recursion. Because the
scrutinee is a **finite** inductive W-tree and the branching functions are
**finite** λ-terms, the descent peels finitely many constructors and bottoms out
(§9.4) — finiteness, not stuckness, is what decides. A function-typed IH
therefore introduces no non-termination into K2c conversion: it is the same
finite structural descent, staged through `a_j`.

The inner elim is genuinely **neutral** only in the special case where `a_j`
*inspects* its branch argument — e.g. `a_j = λx. elim_Bool x … `, for which
`a_j b*` is stuck on the abstract `b*`. That is a legitimate sub-case, not the
general mechanism; decidability does not depend on it (§9.4).

## 8. Strict-positivity check algorithm

§2 defines *what* strict positivity means. This section gives the *how* — the
recursive descent the kernel runs at admission time.

### 8.1 Positivity judgment

For a family `D` being declared, the judgment `Pos_D^n(A)` — "`A` is positive
in `D` at polarisation `n`" — where `n ∈ {+, -}` (positive/negative
polarisation). The check starts with each constructor argument type at `n = +`
and recurses structurally. Every case that would discard subterms without
inspection **must** confirm `D` does not occur in those subterms; if it does,
the declaration is rejected (the kernel conservatively forbids nested
occurrences, per §8.5).

```
Pos_D^+(D Δ_p t̄)        holds  if D does not occur in t̄
Pos_D^+(X)              holds  if D does not occur in X
Pos_D^+(A)              holds  if A is a universe Type ℓ (and D not in ℓ)
Pos_D^+(x : A) → B      holds  if Pos_D^-(A) and Pos_D^+(B)
Pos_D^+(x : A) × B      holds  if Pos_D^+(A) and Pos_D^+(B)

Pos_D^-(D Δ_p t̄)        FAILS  (negative occurrence — reject)
Pos_D^-(X)              holds  if D does not occur in X
Pos_D^-(A)              holds  if A is a universe Type ℓ
Pos_D^-(x : A) → B      holds  if Pos_D^+(A) and Pos_D^-(B)
Pos_D^-(x : A) × B      holds  if Pos_D^-(A) and Pos_D^-(B)
```

Here `D` occurring in a term `t` means `D` appears as a sub-expression
anywhere in `t` (syntactic occurrence, resolved by de Bruijn indices — trivial
since the environment determines what names refer to).

Key: `D` may appear strictly positively (as the target of a function type under
`+` polarisation), but never under `-` polarisation. Any position the algorithm
cannot structurally classify (application arguments, indices, type parameters
containing `D`) is **conservatively rejected** — K1 accepts only the clean
non-nested strictly-positive patterns. This blocks `(D → ⊥) → D`, nested
negatives like `T (D → ⊥)`, and index-embedded occurrences.

### 8.2 Algorithm

```
check-positivity(D):
  for each constructor cₖ of D:
    for each argument type A_j in cₖ's telescope Δₖ:
      if not check-pos-arg(D, +, A_j):
        reject "non-strictly-positive occurrence of D in cₖ"

check-pos-arg(D, pol, A):
  match A:
    D Δ_p t̄  →  return (pol == +) and not occurs(D, t̄)
    Type ℓ   →  return true                    -- ℓ is a level, D is a type
    X        →  return not occurs(D, X)        -- parameter or other type:
                                                 reject if D appears within
    (x : C) → B  →  return check-pos-arg(D, flip(pol), C)
                    and check-pos-arg(D, pol, B)
    (x : C) × B  →  return check-pos-arg(D, pol, C)
                    and check-pos-arg(D, pol, B)
    C u      →  return check-pos-arg(D, pol, C)   -- recurse into head
                    and not occurs(D, u)            -- reject D in argument
```

where `flip(+) = -`, `flip(-) = +`, and `occurs(D, t)` is true iff `D` appears
as a sub-expression anywhere in `t` (a simple term traversal — de Bruijn
indices make this unambiguous).

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

**Rejected (nested negative in application argument):**
`data Bad3 = mk : Pair (Bad3 → Empty) Unit → Bad3`. Argument telescope
`(f : Pair (Bad3 → Empty) Unit)`, argument type `Pair (Bad3 → Empty) Unit`.
Under `+`:

```
check-pos-arg(Bad3, +, Pair (Bad3 → Empty) Unit)
  = check-pos-arg(Bad3, +, Pair)      -- recurse into head (X → not occurs → true)
    and not occurs(Bad3, (Bad3 → Empty, Unit))
  = true and false                    -- occurs finds Bad3 in arguments
  = false → FAILS
```

The application argument `(Bad3 → Empty)` is inspected by `occurs`; `Bad3`
appears there (and negatively, since it's under a Π whose domain flips
polarity), so the check correctly rejects. Without the `occurs` guard on
application arguments, the algorithm would have recursed into the head `Pair`,
returned `true` for the unknown type, and admitted the paradox.

**Rejected (D in its own indices):**
`data Vec (A : Type) : Nat → Type where …` is fine (the index `Nat` is not
`Vec`), but `data Bad4 : (Bad4 → Empty) → Type where …` — where `D` occurs
negatively in its own index — is caught by `occurs(D, t̄)` on the recursive
`D Δ_p t̄` case at `+` polarity: `occurs(Bad4, (Bad4 → Empty))` is true, reject.

### 8.4 W-style (Π-bound) recursive occurrences — admitted in K1.5

The strict-positivity algorithm of §8.2 **already accepts** a W-style argument
`(b:B) → D Δ_p t̄` (the `(x:C) → B` case recurses into the D-free domain at
flipped polarity, then accepts `D` as the target at `+`) — positivity was never
the obstacle. K1 nonetheless **rejected** every Π-bound recursive argument
through a **separate** admission gate, because its eliminator generation did not
build the Π-abstracted induction hypothesis (§3.1). **K1.5 retires that separate
gate**: §8.2 positivity becomes the sole structural admission test for recursive
occurrences, and the eliminator generator handles the W-style IH and its ι
(§3.1, §7.7, §9.4).

No change to §8.1/§8.2 is needed — they are already correct; only the extra
blanket rejection is removed. The algorithm continues to reject, with no gap,
every **negative** occurrence (`D` left of an arrow → `Pos_D^-(D)` fails, §8.3)
and every **nested** occurrence (`D` inside an application argument → the
`occurs` guard fails, §8.5). The admission test for a recursive position is
exactly: peel the argument's leading Π binders; if the body's head is `D`, the
argument is a recursive occurrence and §8.2's positivity verdict on the whole
argument type decides it (positive ⇒ admit, with the Π-abstracted IH; negative
⇒ reject).

### 8.5 Nested and mutually-defined inductives — still deferred

Two classes remain outside K1.5 and are rejected by the on-`main` kernel:

- **Nested** occurrences under another type former — `data Rose A = node : A →
  List (Rose A) → Rose A` — are caught by the `C u` application case of §8.2
  (the `occurs(D, u)` guard rejects `Rose` inside `List (Rose A)`). Admitting
  these needs positivity to *unfold* the host former (`List`) to check the guest
  (`Rose`) within it — a strict generalisation, deferred.
- **Mutually-defined** families are rejected at declaration.

Both are a later extension; neither is required by L5 (`ITree` is single,
non-mutual, non-nested W-style). Keeping them out preserves the minimal-TCB
discipline: K1.5 widens admittance by **exactly** the strictly-positive Π-bound
class and nothing adjacent.

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

### 9.4 W-style ι: subject reduction and termination (K1.5)

W-style admission (§2.1) adds new TCB machinery — a Π-abstracted IH and its ι
(§3.1, §7.7) — so it carries its own soundness obligations, met here at the
K1/K2 bar.

**Subject reduction.** For a W-style constructor `cₖ` with argument `k : (b:B) →
D Δ_p t̄[b]`, the method type (§3.1) ascribes `mₖ` an IH parameter of type `(b:B)
→ M t̄[b] (k b)`. The ι-rule (§7.7) supplies the term `λ (b:B). elim_D M m̄
(t̄[b]) (k b)`. It has that type: for `b : B`, `k b : D Δ_p t̄[b]`, so `elim_D M m̄
t̄[b] (k b) : M t̄[b] (k b)`; abstracting `b` gives `(b:B) → M t̄[b] (k b)` —
exactly the IH parameter's type. Hence `mₖ` applied to the constructor arguments
and these IHs has type `M t̄ₖ (cₖ ā)`, matching the redex (the §9.1 argument,
with the function-typed IH in place of the value IH). The reduct preserves type.
The curried case adds one λ per branching binder and types the same way.

**Termination of conversion.** A function-typed IH raises the question of
whether normalisation can loop. It cannot — and the reason is **finiteness of
the value**, the same structural decrease as the direct eliminator (§9.2(3)),
*not* any stuckness of the inner elim:

1. **The inner elim fires; it recurses on a child.** `elim_D M m̄ ī (cₖ ā) ⇝ mₖ
   ā[ih]` removes the head constructor and introduces the IH `λ b. elim_D M m̄ …
   (k b)`. When `k` is a constructor-producing branching function — the typical
   case, `k = λx. Vis e' (k' x)` — `k b` whnf's to a constructor **even for an
   abstract `b`** (the head `Vis` does not depend on `b`), so the inner `elim_D
   (k b)` **fires** and recurses on `k b`, a **structurally smaller child** of
   the scrutinee (reached through a β-step on `k`, not directly). This drives
   during conversion too: comparing two IHs at their Π type applies a fresh `b*`
   (η, §7.6) and fires exactly this recursion.
2. **Finiteness bounds the descent.** The scrutinee is a **finite** inductive
   W-tree (no coinduction; Scope OUT) and each branching function is a
   **finite** λ-term, so the recursion peels finitely many constructors: each
   step lands on a proper subtree, and the descent bottoms out at a leaf — a
   **base** constructor with no recursive argument (`Ret`, `zero`, `nil`) or a
   W-branching with **empty** domain (`sup a k` with `B a` empty). Finiteness,
   not stuckness, is what decides.

So W-style ι decides for the **same structural-decrease reason** as §9.2(3) —
the recursion is on **children** of the scrutinee — with the children reached
*through* the branching function (a β-step on `k`) rather than directly. The
inner elim genuinely stalls only in the special case where `k` *inspects* its
branch (`k = λx. elim_Bool x …`, so `k b*` is neutral on an abstract `b*`); that
sub-case is incidental, not the basis of decidability. The K2c SCT/decidability
guarantee is untouched: eliminator recursion remains total **without** SCT (§2),
and W-style ι introduces no general recursive δ-definition. Large W-trees
terminate by **finiteness**, not by a size budget.

**Boundary check (the adversarial guard).** Soundness rests on rejecting the
*negative* sibling. `(D → Bool) → D` is **not** admitted (§8.3: `D` in the
arrow's domain is checked at `−` and fails) — exactly the occurrence whose
eliminator would let one build a non-terminating fixpoint. K1.5 admits the
**target** position and only that; the polarity discipline of §8.1 is the line,
and the conformance corpus exercises both sides (a W-style elim that *uses* its
Π-abstracted IH, and a negative occurrence that must still be rejected).
