# Inductive families

> Status: **K1 elaborated; K1.5 extends it; nested-positive specified and
> implementation-gated**.
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
> recursion. Strict positivity (§8) remains the sole structural admission gate;
> §8.5 extends that gate through checked positive-parameter paths and requires
> the lifted eliminator machinery of §3.2/§7.8. That extension remains gated on
> `KERNEL-NESTED-IND`. The motivating K1.5 client is L5's interaction tree
> `ITree` (`../30-surface/36-effects.md`).

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
positive**: `D` may appear directly as the *target* of a (possibly dependent)
function type or through a declared strictly-positive parameter path (§8.5),
never to the *left* of an arrow and never through an unknown or non-positive
parameter position.

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
any arrow (`(D → Bool) → D`, §8.3) — and nested occurrences through an unknown
or non-positive parameter position (§8.5). Mutual families remain a separate
later extension. Nested occurrences through declared strictly-positive
parameter positions are specified in §8.5; the on-`main` kernel remains
fail-closed on that new class until `KERNEL-NESTED-IND` lands.

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
dependency is what makes it an *induction* principle, not just a recursor; the
codomain is a **sort** — `Type ℓ'`, shown here, or a proposition `Ω_l` (the
"Elimination into Ω" case, below)), and one **method** `mₖ` per constructor
giving the result for that constructor —
including, for each recursive argument, the **induction hypothesis** (the motive
already applied to that sub-value; §3.1 generalises this to W-style arguments,
whose IH is itself a *function*, and §3.2 lifts it through nested
strictly-positive content) — the eliminator has type:

```
elim_D : (M : (Δ_i) → D Δ_p Δ_i → Type ℓ')
       → (m₁ : ⟦method type for c₁⟧)
       → …
       → (mₙ : ⟦method type for cₙ⟧)
       → (i̅ : Δ_i) → (x : D Δ_p i̅) → M i̅ x
```

For a constructor `cₖ : (Δₖ) → D Δ_p t̄ₖ`, the method type abstracts over
`Δₖ`, adds the direct, Π-abstracted, or structurally lifted induction
hypotheses specified by §3.1–§3.2, and concludes `M t̄ₖ (cₖ …)`.

**Computation (ι).** On a constructor the eliminator reduces to the
corresponding method, applied to the constructor's arguments and the recursive
results:

```
elim_D M m̄ i̅ (cₖ ā)  ≡  mₖ ā [elim_D M m̄ … r  for each recursive r in ā]   (D-ι)
```

i.e. each direct recursive argument `r` is replaced in the method by `elim_D M
m̄ … r` — the structural recursive call. §3.1 and §3.2 give the corresponding
Π-abstracted and nested lifted forms. Because recursion is only ever on
*structurally smaller* sub-values, ι-reduction terminates; this is the totality
of the eliminator.

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

**Elimination into `Ω` (the `Ω`-motive — K4).** The motive's codomain is a
**sort**, and besides a `Type ℓ'` (type-*selecting*, above) it may be a
proposition `Ω_l`. An **`Ω`-codomain motive** `M : (Δ_i) → D Δ_p Δ_i → Ω_l` lets
the eliminator **prove a per-branch-varying proposition** by case-split on a
relevant inductive scrutinee — the induction principle landing *at* `Ω`, rather
than computing which type. This is what makes a **universally-quantified law
over an inductive carrier provable**: e.g.
`refl : (x : Bool) → IsTrue (bool_leq x x)` is `elim_Bool` at the motive
`λx. IsTrue (bool_leq x x) : Bool → Ω_0` with a per-constructor proof method
(the lawful-structure-classes law fields, `../50-stdlib/51 §5`/`§6`). The
admissible
codomain is exactly `Type ℓ' ∪ Ω_l` — a **sort**, not a wildcard: a non-sort
codomain is still rejected.

- **ι-reduction is sort-agnostic.** The eliminator's computation (`D-ι` above)
  is the **same** constructor-selects-method rule regardless of the motive's
  codomain sort — an `Ω`-motive `elim_D` reduces to the selected branch's method
  exactly as a `Type`-motive one does. **No new reduction path** is added; the
  ι-rule never inspects the motive's sort.
- **Soundness — into-`Ω` is the safe direction.** (i) The motive *type* is
  already well-formed with no restriction: `Ω`'s **predicative** Π-formation
  (`16 §1.1`) admits `(Δ_i) → D Δ_p Δ_i → Ω_l` (a `Type`-domain, `Ω`-codomain
  Π); the restriction was only ever an incompleteness in the eliminator's
  codomain check, **not** a Π-formation boundary. (ii) The classic
  large-elimination danger requires an **impredicative** proposition universe;
  Ken's `Ω` is predicative (`16 §1.1`, `12`, ADR 0005), so the precondition is
  **absent**. (iii) The direction **narrows *into* `Ω`** (a `Type` scrutinee →
  an `Ω` result) — **distinct from the two shapes that *are* restricted**:
  *declaring* a proof-relevant inductive **at** `Ω` (`16 §1.1`, only
  sub-singletons enter `Ω` — the *formation* axis) and *singleton-eliminating*
  an `Ω`-inhabitant **out into** a relevant `Type` (which would leak
  which-proof information); this is neither. (iv) **Proof irrelevance is
  preserved for free**:
  an `Ω`-motive result is typed `M i̅ s : Ω_l`, and conversion at an `Ω`-type is
  definitionally irrelevant (`16 §1.2`) — the irrelevance short-circuit fires on
  the *type*, upstream of the term, so no which-branch distinction leaks back
  through conversion. The soundness obligation is **entirely about typing
  admission**, not conversion (no new conversion rule).

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

### 3.2 Nested arguments and lifted induction hypotheses

A nested recursive argument is not itself headed by `D`: it contains values of
`D` through one or more declared strictly-positive parameter positions (§8.5).
The direct recipe "one IH per recursive argument" is therefore strengthened to
**one IH per contained recursive occurrence**. A nested argument contributes
one structured lifted hypothesis whose leaves are in bijection with those
contained occurrences and whose shape follows the containing value.

Write `Lift_D(M, A, a)` for the kernel's dependent lifting of motive `M` through
the type `A` of value `a`. This is a metatheoretic operation used to state the
generated method type; it is not a new surface type former. It is defined
structurally:

- at a direct occurrence `r : D Δ_p t̄`,
  `Lift_D(M, D Δ_p t̄, r) = M t̄ r`;
- through a declared strictly-positive parameter position, it preserves the
  enclosing constructor shape and recursively lifts every contained occurrence;
- through a Π-bound recursive position, it is the Π-abstracted IH of §3.1;
- at a type with no occurrence of `D`, it contributes no IH.

Thus a constructor argument `a_j : A_j` with nested recursive content adds

```
ih_j : Lift_D(M, A_j, a_j)
```

to its method type. The lift is available only along the same checked
strictly-positive paths that admitted `A_j` (§8.5). An unknown or non-positive
parameter position cannot acquire a lift and is rejected at admission; the
eliminator generator never guesses how to traverse it.

**Computation (ι), nested case.** The outer ι-rule still selects exactly one
constructor method. For each nested argument, it supplies the structurally
lifted value obtained by following the enclosing value's constructors and
placing `elim_D M m̄ … r` at every contained recursive occurrence `r`:

```
elim_D M m̄ ī (cₖ … a_j …)
  ≡ mₖ … a_j … lift-elim_D(M, m̄, A_j, a_j) …                 (Nested-D-ι)
```

The auxiliary `lift-elim_D` is determined by the admitted former's own
constructor/eliminator structure. It may not be supplied by a name allow-list,
an unchecked user function, or an axiom. §7.8 gives the reduction rule and
§9.5 states its subject-reduction and termination obligations.

**Level.** Let `ℓ_path` be the maximum level of the field domains traversed by
the positive path. If `M` lands in `Type ℓ'`, the lifted hypothesis lands in
`Type (max ℓ_path ℓ')`; if `M` lands in `Ω_l`, it lands in
`Ω_(max ℓ_path l)`. These are ordinary predicative Π/Σ maxima (`12 §2`,
`16 §1.1`), not a new universe rule or an impredicative collapse.

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
Σ, (add : Int → Int → Int := prim …)      -- an opaque primitive operation
```

- A primitive type is an **opaque type constant**; it has no kernel-level
  constructors or eliminator. Its inhabitants are **literals** (introduced by
  the elaborator as opaque primitive values) and the results of primitive
  operations.
- A primitive operation carries a registered `PrimReduction::Op` symbol (`41`).
  In the landed system that registration is an **interpreter dispatch
  descriptor**, not a kernel-conversion rule: `ken-interp` computes `add 2 3`
  to `5` at runtime, while the kernel leaves the application neutral even when
  every argument is a literal. Thus `add 2 3 ≡ 5` does not hold definitionally
  and `Refl` cannot prove that equation.
- `PrimReduction::Literal` is a different case. A checked surface literal is
  already a value, not an operation application; registered literal equality
  may compare two such values as specified in `16 §2.2` and ADR 0013. This does
  not make an enclosing `Op` application reduce.
- Primitive declarations and operation symbols are small, audited, and listed
  by `trusted_base()` (`18 §5`). Correct `Op` results remain a semantic
  correctness obligation on the interpreter's `prim_reduce`; a wrong result is
  a wrong runtime value, not a false kernel proof, because conversion never
  consumes that result. Kernel execution of registered operations is the
  **K3-deferred** trusted-reduction design fork.
- Equational properties of primitive operations, including equations on
  concrete literals, are **propositions to prove**. Until K3 provides a
  proof-relevant conversion or certificate mechanism, a direct law needs a
  visible postulate/`Axiom` or a proof over an independent model, not `Refl`.

## 6. What the kernel checks here

A conforming kernel MUST:

1. Type-check inductive declarations and **enforce strict positivity** (§2, §8),
   rejecting negative occurrences; **admit strictly-positive W-style (Π-bound)
   recursive occurrences** (§2.1) — positivity (§8.2) is the sole structural
   gate, with no separate Π-bound rejection; and admit nested occurrences only
   through declared strictly-positive parameter paths (§8.5), rejecting unknown
   and non-positive paths while mutual families remain deferred (§8.6).
2. Generate the constructors and the **dependent** eliminator with induction
   hypotheses for recursive arguments (§3) — including the **Π-abstracted**
   induction hypothesis `(b:B) → M t̄[b] (k b)` for W-style arguments (§3.1)
   and one structurally lifted hypothesis per contained recursive occurrence
   for nested arguments (§3.2).
3. Implement **ι-reduction** of the eliminator on constructor forms (§3, §7),
   driving structural recursion; for W-style arguments thread the recursive
   result through the branching function (`λ b. elim_D … (k b)`, §3.1, §7.7);
   for nested arguments structurally lift the result through the enclosing
   value (§3.2, §7.8); ensure both terminate (structural decrease, §9).
4. Permit **large elimination** under the predicative universe rules (§3).
5. Treat **primitive** types/operations as opaque constants with registered,
   audited operation descriptors (§5), never as inductives. K1 defines only the
   interface; the runtime value model (`../40-runtime/41-values.md`) and
   interpreter elaborate the current operation semantics. Kernel conversion for
   registered operations remains K3-deferred.

Conformance: `../../conformance/kernel/inductive/` — positivity acceptance and
rejection, `elim_Nat`/`elim_Vec` ι-computation, large elimination (`elim_Bool`
into `Type`), checked literal values, and primitive-`Op` opacity under
conversion (`add_int 2 3` does not convert to `5`). Runtime primitive values are
covered by the surface/runtime corpora, not this kernel-conversion corpus.

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

### 7.8 Nested ι

For a nested argument position `j`, §3.2's lifted IH is computed by structural
recursion over the actual enclosing value `a_j`. At each constructor of that
value, `lift-elim_D` preserves the constructor, recursively processes fields
whose types lead to a declared strictly-positive parameter position, and
replaces every contained `r : D Δ_p t̄` with:

```
elim_D M m₁…mₙ t̄ r
```

The outer reduct is:

```
elim_D M m̄ ī (cₖ ā)  ⇝  mₖ ā [ih₁ … ih_p]
```

where an `ih` is direct (§7.3), Π-abstracted (§7.7), or structurally lifted
through an enclosing value (this section). A lifted `ih` contains exactly one
recursive result for each contained recursive occurrence and preserves the
enclosing value's constructor topology. Its own ι-reductions are the
constructor-selects-method reductions of the enclosing former followed by the
`D-ι` call at each contained child.

The generated method type (§3.2), the generated lifted term, and every nested
ι-reduct are kernel-checked. Admission alone is insufficient: if the lift
cannot be generated and checked, the declaration is rejected.

## 8. Strict-positivity check algorithm

§2 defines *what* strict positivity means. This section gives the *how* — the
recursive descent the kernel runs at admission time.

### 8.1 Positivity judgment

For a family `D` being declared, the judgment `Pos_D^n(A)` — "`A` is positive
in `D` at polarisation `n`" — where `n ∈ {+, -}` (positive/negative
polarisation). The check starts with each constructor argument type at `n = +`
and recurses structurally. Every case that would discard subterms without
inspection **must** confirm `D` does not occur in those subterms, except where
§8.5 supplies a checked structural descent through a declared
strictly-positive parameter position. Every other occurrence is rejected.

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
cannot structurally classify (indices, unknown parameter positions, or
non-positive parameter positions containing `D`) is **conservatively
rejected**. §8.5 is the only refinement of the application-argument case. This
blocks `(D → ⊥) → D`, nested negatives like `T (D → ⊥)`, and index-embedded
occurrences.

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
    C u      →  return check-pos-application(D, pol, C, u)
                                                    -- §8.5; unknown fails closed
```

where `flip(+) = -`, `flip(-) = +`, `check-pos-application` is the structural
parameter-position rule of §8.5, and `occurs(D, t)` is true iff `D` appears as a
sub-expression anywhere in `t` (a simple term traversal — de Bruijn indices
make this unambiguous).

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

The Π-bound rule itself is unchanged. The algorithm continues to reject, with
no gap, every **negative** occurrence (`D` left of an arrow →
`Pos_D^-(D)` fails, §8.3). The admission test for a Π-bound recursive position
is exactly: peel the argument's leading Π binders; if the body's head is `D`,
the argument is a recursive occurrence and §8.2's positivity verdict on the
whole argument type decides it (positive ⇒ admit, with the Π-abstracted IH;
negative ⇒ reject). §8.5 separately governs a recursive occurrence contained
in an application argument; it does not change this Π-bound class.

### 8.5 Nested inductives — structural parameter polarity

A nested occurrence is an occurrence of the family being declared inside an
argument to another type former. The former's name is irrelevant. Admission is
decided only by the checked polarity of the parameter path from the enclosing
former to the occurrence.

**Nested-parameter rule (normative).** First collect an application into its
maximal spine `F a₁ … aₙ`. For that spine at positive polarisation:

1. Resolve `F` to a previously admitted type-former declaration, transparently
   unfolding an admitted definition before classification. If it does not
   resolve, or the resulting former carries no checked parameter-polarity
   information, every argument position is **unknown** and an argument
   containing `D` is rejected.
2. A parameter position is **declared strictly positive** only when the
   declaration's checked definition or constructor telescopes establish that
   the parameter occurs solely at positive polarisation. An index or other
   position the checker cannot structurally classify makes the polarity
   unknown. Native formers obtain the same fact from their kernel formation
   rule. This fact is computed and kernel-checked when the declaration is
   admitted; it is not an unchecked user assertion.
3. If `D` occurs in `a_j`, position `j` must be declared strictly positive and
   `check-pos-arg(D, +, a_j)` must itself hold. This composes through any finite
   chain of declared strictly-positive parameter positions.
4. If position `j` is declared **non-positive**—negative, mixed, or otherwise
   not solely positive—an argument containing `D` is rejected. The checker does
   not flip or infer a variance for that position.
5. At negative polarisation, any contained occurrence of `D` is rejected,
   including one reached through a declared strictly-positive parameter.
6. Every application argument not traversed by clauses 1–5 remains guarded by
   `not occurs(D, a_j)`. Thus an unknown, unclassified, or discarded position
   fails closed rather than becoming positive by default.

This rule is structural and compositional: only a checked positive path admits
the nested occurrence, and the same path determines the lifted IH and ι of
§3.2/§7.8. Merely deleting the `occurs` guard would satisfy none of clauses
1–6 and would not generate the required eliminator machinery.

**Illustrative admitted shapes.**

```ken
data Rose A = node : A → List (Rose A) → Rose A
data Json = ... | JsonArray (List Json)
                    | JsonObject (List (Pair String Json)) | ...
```

These examples are admitted because every path to the recursive family follows
checked strictly-positive parameters. They are examples of the rule, not
special cases in it.

**Fail-closed boundaries.** A recursive occurrence under a parameter with no
checked polarity is rejected as **unknown**. A recursive occurrence under a
declared negative or mixed parameter is rejected as **non-positive**. A nested
negative such as `Pair (Bad → Empty) Unit` remains rejected by §8.3 even when
the outer parameter is positive: the recursive descent reaches the Π-domain at
negative polarisation and fails.

**Implementation stage.** `SPEC-NESTED-IND` states this rule;
`KERNEL-NESTED-IND` implements its admission metadata, lifted IHs, and ι. Until
that kernel node lands, the on-`main` kernel continues to reject the newly
specified nested-positive class. This is a safe completeness boundary, not an
unsound kernel result.

### 8.6 Mutually-defined inductives — still deferred

Mutually-defined families remain rejected at declaration. They require
simultaneous positivity over a declaration block plus jointly generated
eliminators and a joint termination argument; no current consumer requires that
distinct extension, so this nested-only change does not admit them.

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

### 9.5 Nested ι: subject reduction, termination, and conformance

Nested admission (§8.5) adds trusted machinery only together with the lifted IH
and ι of §3.2/§7.8. Admitting the declaration without those consumers is not a
conforming implementation of this chapter.

**Subject reduction.** For a nested constructor argument `a : A`, the method
type requires `Lift_D(M, A, a)`. The generated `lift-elim_D(M, m̄, A, a)` has
exactly that type by induction on the already-admitted declaration for `A`:
each enclosing constructor is rebuilt at the lift, each field follows its
checked positive-parameter path, and each contained `r : D Δ_p t̄` contributes
`elim_D M m̄ t̄ r : M t̄ r`. Supplying these lifted terms therefore gives the
selected method its declared result `M t̄ₖ (cₖ ā)`, the same type as the
redex.

**Termination.** The well-founded measure is the lexicographic pair consisting
of the outer `D` value's structural size and the already-admitted host former's
own eliminator measure. A recursive lift call follows a host-eliminator call and
strictly decreases the second component (including the Π-bound measure of
§9.4). Applying `elim_D` to a contained child strictly decreases the first,
regardless of the second component for that child. The host former was admitted
only with a terminating eliminator, so nested ι terminates. No SCT edge or
general recursive δ-definition is introduced.

**Surface consumability.** Surface `match` and structural recursion elaborate
to the generated method type, including `Lift_D`; the elaborator and termination
checker must preserve and consume those lifted hypotheses rather than
reconstructing recursive calls or discarding the lift. Matching an enclosing
value deconstructs its lift in lockstep, so an exposed recursive child carries
its motive instance and an exposed enclosing child carries the residual lift
(`../30-surface/34 §3.1`, `../30-surface/39 §2.2`,
`../40-runtime/43 §1`). A theorem or recursive computation over a nested branch
must therefore be writable from the generated IHs.

**Required conformance population.**

1. A positive nested declaration through a freshly declared positive container,
   with a real recursive proof or computation that fails if its lifted IH is
   removed.
2. A nested-negative declaration that is rejected at the specific positivity
   boundary.
3. Fail-closed parameter controls: one unknown-position rejection and one
   separately asserted non-positive-position rejection.
4. Direct recursive and existing Π-bound/W-style declarations and their ι
   behavior remain unchanged.
