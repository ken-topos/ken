# Observational equality and computation

> Status: **K2 elaborated; K2c series-2 completes the three obs-reduction
> seams** (§3.2 inductive index rewrite, §4.1 non-constant-motive `J`, §5.1
> full quotient `respect`). Normative for the interface, computation rules,
> and algorithmic reduction behavior. This is the machinery `15-identity.md`
> reuses: the strict proposition universe Omega, observational equality `Eq`
> computed by recursion on type structure, the `cast` coercion, quotient
> types, and propositional truncation. It replaces the cubical layer
> (ADR 0005): there is **no** interval, cofibration,
> `transp`/`hcomp`/`comp`, `Glue`, univalence, or higher inductive
> type. Ken is a **set-level** theory (UIP holds), which is what software
> is. Rules tagged **(oracle)** are to be validated against the prototype at
> build time; all other rules are normative.
>
> **Series-2 note.** The three seams K2 left **sound-stuck** (cast at an
> inductive index change, `J` at a dependent motive, quotient elim into a
> Type target) now **compute**, each gated on a structural condition
> (canonical index decomposition / endpoint type structure / the verified
> respect schema) — derived from first principles, **not** (oracle). The
> kernel's prior fallbacks were *stuck/reject* (sound but incomplete); these
> rules make them reduce without weakening any guard.
>
> **§5.1 erratum (transport direction).** The quotient-respect schema's
> transport `cast` is `cast (M [y]) (M [x]) (sym (cong M h')) (f y)` —
> source `M [y]`, target `M [x]` (it carries `f y : M [y]` into `M [x]`,
> the type the obligation `Eq (M [x]) (f x) …` requires). An earlier
> revision had the source/target reversed; the prose intent ("transport
> `f y` from `M [y]` to `M [x]`") was always correct. The direction is
> only observable at a **dependent** motive (`M [x] ≢ M [y]`) — see §5.1.

## 1. The strict proposition universe Omega (SProp)

Ken has a universe **Omega** of **strict (definitionally proof-irrelevant)
propositions** -- the subobject classifier and the home of equality and the
logic.

### 1.1 Formation and level discipline

Omega is **level-indexed** and **predicative**: propositions about types at
level `l` live in `Omega_l : Type (suc l)`. The formation rule:

```
Omega_l : Type (suc l)                                    (Omega-Form)
```

`Omega_l` is a strict proposition universe at each level. Propositions over
small types (`Type 0`) are in `Omega_0`; propositions over larger types are
in the corresponding `Omega_l`. For brevity, unqualified `Omega` in prose
refers to `Omega_0`, but the kernel implements the level-indexed form.
This matches the level-polymorphic regime of `12-universes.md` §4.

- Omega is **predicative** (OQ-Prop DECIDED, ADR 0005): `Omega_l : Type (suc l)`,
  and a quantifier over a large type `(x : A) → P` where `A : Type l` and
  `P : Omega_l` is itself in `Omega_l` — no impredicative lowering. The level
  is determined by the **predicative `max`** rule (same as Π/Σ formation in
  `13-pi-sigma.md`).
- Ω is **strict**: any two proofs `p, q : P : Omega_l` are definitionally
  equal (§1.2). This makes propositional arguments computationally irrelevant.
- There is **no cumulativity** for Omega (Ken is non-cumulative, `12 §3`):
  `Omega_l` does not automatically inhabit `Omega_(suc l)`. Lifting is
  explicit when needed.

### 1.2 Proof irrelevance (definitional)

```
  Gamma |- P : Omega     Gamma |- p : P     Gamma |- q : P
  ─────────────────────────────────────────────────────────  (Omega-PI)
  Gamma |- p ≡ q : P
```

- Any two proofs of a proposition are **definitionally equal**. So UIP and
  proof irrelevance hold by construction, and the conversion checker may
  **skip the contents of propositional arguments** entirely (`17` par. 2)
  -- which also means agents need not synthesise coherence/transport terms
  for them.
- This rule is algorithmic: when the conversion checker compares two terms
  at a type `P : Omega`, it returns `true` immediately without inspecting
  the terms. The only precondition is that both terms have been judged to
  inhabit `P` (i.e. a prior type-check ensures `Gamma |- p : P` and
  `Gamma |- q : P`); the conversion check itself becomes a constant-time
  "yes."
- The **propositional-argument-skip shortcut**: when conversion is
  comparing two applications `f a1 ... an` and `f' b1 ... bn` where some
  `ai` or `bi` is at an Omega type, the comparison may skip the contents
  of those arguments (treating them as definitionally equal). The precise
  rule is in the type-directed structural comparison (`17`): if the binder
  type is in Omega, the argument position is exempt from conversion
  checking.

### 1.3 Derived propositional connectives

Omega carries the **Heyting** structure (`12-universes.md` par. 5.2). The
following are **derived operations** from K1 formers -- they do not need
primitive kernel support beyond the Omega formation and PI rules above:

| Connective | Definition | K1 former used |
|------------|------------|----------------|
| Top        | `Unit`     | Unit : Type 0 (unit type, `14-inductive.md`) |
| Bottom     | `Empty`    | Empty : Type 0 (empty inductive, `14-inductive.md`) |
| P and Q    | `(x : P) x Q` | Sigma-type (`13-pi-sigma.md` par. 2) |
| P => Q     | `(x : P) -> Q`  | Pi-type (`13-pi-sigma.md` par. 1) |
| not P      | `P -> Empty`    | Pi-type + Bottom |

`Unit` and `Empty` as defined in K1 live in `Type 0`. When used as
propositions in `Omega_0`, the kernel provides them as **Omega-typed
prelude constants** — `Top : Omega_0` (wrapping `Unit`) and `Bottom :
Omega_0` (wrapping `Empty`). These are direct declarations, not a
general `Type → Omega` coercion (only genuine sub-singleton types may
enter the strict-prop universe; an unrestricted `Type 0 → Omega_0` would
admit `Bool`, making `true ≡ false` by Ω-PI and breaking consistency).

`P or Q` and `exists x. P` are provided via truncation (par. 6):
`P or Q := || P + Q ||` and `exists x:A. P x := || (x:A) x P x ||`. These
land in Omega because truncation lands in Omega.

All connectives are intuitionistic, not Boolean. Excluded middle is not
assumed (it holds for *decidable* props as data, via `14-inductive.md`).

- **`Eq` (below) lands in `Omega_l`** (for the appropriate level `l`),
  so equality is a proof-irrelevant proposition.

## 2. Observational equality `Eq`

Propositional equality is **`Eq A a b : Omega`**, *computed by recursion
on the structure of the type `A`* -- equality "observes" the type.
`refl a : Eq A a a`.

### 2.1 Formation

```
  Gamma |- A : Type l     Gamma |- a : A     Gamma |- b : A
  ───────────────────────────────────────────────────────────  (Eq-Form)
  Gamma |- Eq A a b : Omega_l
```

Equality is a proposition at the level of `A`. For `A : Type l`, `Eq A a b`
lands in `Omega_l : Type (suc l)` — predicative, matching the formation rule
for Omega (§1.1).

### 2.2 Reduction rules (by type structure)

The defining computations -- the heart of OTT. For each type former, `Eq A
a b` reduces by case analysis on the **weak-head normal form** of `A`.
When `A` is **neutral** (a variable, an eliminator applied to a neutral
scrutinee, or a stuck cast), `Eq A a b` is itself neutral -- no reduction
applies.

In the rules below, `⇝` is the kernel's reduction relation (weak-head
reduction; `whnf`). Each rule is a **WHNF reduction rule**: when `Eq A a
b` is in WHNF position, the left-hand side is the weak-head form and the
right-hand side is its reduct.

---

**Pi-type.** `A` reduces to `(x : A1) -> B1`.

```
Eq ((x : A1) -> B1) f g
  ⇝ (x : A1) -> Eq (B1 x) (f x) (g x)
```

This is **funext definitional**: two functions are equal exactly when they
are pointwise equal. No axiom needed. The reduct is a Π whose body is an
`Eq` in `Omega_l` (where `l` is determined by the level of `B1 x`); since
the whole Π quantifies over propositions, the result lives in `Omega_l` by
the **predicative** formation rule — `Omega_l` is closed under Π-types
whose codomain is in `Omega_l`. No cumulativity is required (Ken is
non-cumulative, `12 §3`).

If `A1 : Type l1` and `B1 x : Type l2` (so `(x:A1)→B1 : Type (max l1 l2)`),
then `Eq ((x:A1)→B1) f g : Omega_(max l1 l2)` and reduces to
`(x:A1) → Eq (B1 x) (f x) (g x) : Omega_(max l1 l2)` — the level is the
same, determined by the predicative `max` of `13 §1` applied to the
Omega-level domains. This is **not** impredicative: the quantifier ranges
over the large domain `A1` at level `l1`, but the body `Eq (B1 x) (f x)
(g x) : Omega_l2` is already in `Omega_l2`, and the Π lands in
`Omega_(max l1 l2)`.

**Neutral case.** When `f` or `g` is neutral and `A1` is neutral, `Eq`
stays neutral (no reduction). If `A1` is canonical but `f` or `g` is
neutral, the `Eq` reduces to a Pi whose body contains a neutral `Eq` --
this is fine; the outer Pi is canonical.

---

**Sigma-type.** `A` reduces to `(x : A1) x B1`.

```
Eq ((x : A1) x B1) p q
  ⇝ Eq A1 p.1 q.1
      and Eq (B1 q.1)
            (cast (B1 p.1) (B1 q.1)
                  (cong (x. B1 x) (eq-fst))
                  p.2)
            q.2
  where eq-fst : Eq A1 p.1 q.1
```

The `cast` on `p.2` is required because `p.2 : B1[p.1/x]` and we need to
compare it with `q.2 : B1[q.1/x]` at the **same** type `B1[q.1/x]`, so we
transport `p.2` from `B1[p.1/x]` to `B1[q.1/x]` along the equality of the
first components.

The `cong (x. B1 x) eq-fst` is the proof that `B1[p.1/x] = B1[q.1/x]` in
`Type` -- it is obtained by applying `cong` (par. 4) to the family `B1`
and the first-component equality.

**Neutral case.** When `p` or `q` is neutral, `Eq` is neutral. When `p.1`
and `q.1` are canonical but `p.2` or `q.2` is neutral, the `Eq` reduces to
a conjunction with neutral components.

---

**Inductive type.** `A` reduces to `D Delta_p i-bar`, where `D` is an
inductive family (`14-inductive.md`) with parameters `Delta_p` and indices
`i-bar`. `a` and `b` are constructor applications.

*Same constructor* (both `c_k`):
```
Eq (D Delta_p i-bar) (c_k a-bar) (c_k b-bar)
  ⇝ Eq A_1 a_1 b_1
      and Eq (A_2[b_1/x_1]) (cast (A_2[a_1/x_1]) (A_2[b_1/x_1]) eq_1' a_2) b_2
      and ...
      and Eq (A_n[b_1/x_1 ... b_{n-1}/x_{n-1}])
            (cast (A_n[a_1/x_1 ... a_{n-1}/x_{n-1}])
                  (A_n[b_1/x_1 ... b_{n-1}/x_{n-1}])
                  eq_{n-1}' a_n)
            b_n
```
where the constructor telescope is `(x_1 : A_1) ... (x_n : A_n)`. For each
argument position `j > 1`, the type `A_j` may depend on earlier arguments
`x_1 … x_{j-1}`. When comparing `a_j` at type `A_j[a_1/x_1 … a_{j-1}/x_{j-1}]`
and `b_j` at type `A_j[b_1/x_1 … b_{j-1}/x_{j-1}]`, the latter argument must
be transported to the former's type along the equalities of all earlier
arguments — the same `cast`-on-dependent-component pattern as the Σ rule.
Each `eq_j'` is the `cong` of the family `A_{j+1}` along the accumulated
equalities of arguments `1…j`.

This is the **dependent telescope** treatment the `cast`-at-inductive rule
(`§3.2`) already applies, mirrored here for `Eq`-at-inductive. For a
concrete example see the Vec instance below.

**Example (Vec).** For `Vec A : Nat → Type`:
```
Eq (Vec A (suc n)) (vcons A n a xs) (vcons A n' a' xs')
  ⇝ Eq Nat n n'                              -- index equality (suc n vs suc n')
      and Eq A (cast A A refl a) a'           -- element type A, a : A, a' : A
      and Eq (Vec A n) (cast (Vec A n) (Vec A n') (cong (Vec A) eq_n) xs) xs'
```
where `eq_n : Eq Nat n n'` is the first argument equality. The third
component transports `xs : Vec A n` to `Vec A n'` using `eq_n` before
comparing with `xs' : Vec A n'` — the dependent-telescope `cast`.

*Different constructors* (`k /= l`):
```
Eq (D Delta_p i-bar) (c_k a-bar) (c_l b-bar)
  ⇝ Bottom
```

*Index mismatch.* When the indices `i-bar` differ between the scrutinees
(because the inductive family is indexed), the term is ill-typed before we
reach `Eq` -- `c_k a-bar : D Delta_p i-bar` and `c_l b-bar : D Delta_p
j-bar` with `i-bar /= j-bar` fail at `check`, so `Eq` is never formed on
such a pair.

**Neutral case.** When either scrutinee is neutral (not headed by a
constructor), `Eq` is neutral.

---

**Quotient type.** `A` reduces to `A' / R` (par. 5).

```
Eq (A' / R) [a] [b]
  ⇝ R a b
```

Equality on a quotient *is* the user-supplied relation. Two equivalence
classes are equal exactly when their representatives are related by `R`.

**Neutral case.** When either class representative is neutral, `Eq` is
neutral.

---

**Type universe.** `A` reduces to `Type l`.

```
Eq Type l A B
  ⇝ structural type equality
```

Type equality is **structural** (par. 3), not univalent. The reduction is:
- `Eq Type l ((x:A1)->B1) ((x:A2)->B2)` ⇝
  `Eq Type l A1 A2 and (x:A1) -> Eq Type l (B1 x) (B2 (cast A2 A1 ... x))`
  (and similarly for Sigma, inductive, quotient)
- `Eq Type l A B` where `A` and `B` have **different heads** (e.g. Pi vs
  Sigma) ⇝ `Bottom`
- `Eq Type l A A` (same neutral head) is neutral

This is defined structurally; the algorithmic rules for structural
type-equality are given in par. 3 (they are the same as the `cast`-by-type
decomposition, mutual with `Eq`-by-type).

---

**Omega.** `A` reduces to `Omega`.

```
Eq Omega P Q
  ⇝ (P -> Q) and (Q -> P)
```

This is **propext definitional**: two propositions are equal exactly when
they imply each other. No axiom needed. Since `Eq Omega P Q : Omega`
itself (Omega is a universe, `Omega : Type 1`, `Eq Omega P Q : Omega`),
this is definitionally equivalent to `(P <-> Q)`, expressed via the
connectives of par. 1.3.

---

**Primitive type.** `A` reduces to a primitive type (e.g. `Int`, `Float`,
`String`).

```
Eq Int m n  ⇝  primEqInt m n
```

Where `primEqInt` is the kernel's built-in integer equality, returning a
WHNF boolean in Omega (or `Top`/`Bottom`). Analogous for each primitive
type (`14-inductive.md` par. 5).

**Neutral case.** When `m` or `n` is neutral, `Eq` reduces to a neutral
`primEqInt m n` -- which the convertibility checker treats as stuck.

---

### 2.3 General properties

- **Neutral head.** If `A` is neutral (a variable, an eliminator on a
  neutral scrutinee, or a stuck cast), `Eq A a b` is a **neutral
  proposition** -- no reduction applies. The conversion checker treats it
  as stuck (K2c's NbE will handle incomplete `Eq` forms).
- **Proof irrelevance.** `Eq A a b : Omega_l`, so any two proofs of equality
  are definitionally equal (par. 1.2). There is no "equality of
  equalities."
- **UIP.** `Eq (Eq A a b) p q` is definitionally trivial: the type is in
  `Omega_l`, so `p ≡ q` by Omega-PI. Ken is set-level.
- **refl.** `refl a : Eq A a a` is the canonical proof; it is neutral when
  `a` is neutral, and reduces (by the rules above) when `A` is canonical.

## 3. Type equality and `cast` (transport)

Equality of **types** is **structural, not univalent**: `Eq Type A B` holds
when `A` and `B` have the **same head former with equal parts** -- `Eq
Type ((x:A1)->B1) ((x:A2)->B2)` reduces to `Eq Type A1 A2 and ...`, two
inductives are equal iff the same family at equal parameters/indices, etc.
There is **no** `(A ∼ B) -> Eq Type A B` (that is univalence, deliberately
absent, ADR 0005).

### 3.1 `cast` formation

```
  Gamma |- A : Type l   Gamma |- B : Type l
  Gamma |- e : Eq Type A B   Gamma |- a : A
  ───────────────────────────────────────────  (cast)
  Gamma |- cast A B e a : B
```

### 3.2 `cast` reduction rules (by type structure)

`cast` is the primitive transport operation. It reduces **by recursion on
the structure of `A` and `B`**. The equality proof `e` is **never
inspected** -- `cast` computes from the endpoints, not from the proof.
This is the canonicity-friendly property: `J`/`subst` compute on non-`refl`
because `cast` computes on the type structure, not on the proof.

The rules below are **WHNF reduction rules**. When `A` and `B` have
different heads (e.g. Pi vs Sigma), or when the arguments are neutral,
`cast` is neutral (stuck). When `A` and `B` share the same head former,
`cast` reduces by structural recursion into the sub-components. The
sub-equality proofs needed for recursive calls are obtained from the
mutual definition with `Eq`-by-type (par. 2) on type structure -- the
kernel's `Eq` and `cast` reductions are a **single mutual reduction
system**.

---

**Regularity (any A).**

```
cast A A refl a  ⇝  a
```

This is the key regularity rule: transporting along a reflexive
type-equality is the identity. Unlike De Morgan cubical (where constant
compositions are not constructively the identity), OTT's `cast` on a
reflexive proof reduces to the input -- a cleaner equational theory and a
real simplicity win.

---

**Pi-type.** `A` is `(x : A1) -> B1`, `B` is `(x : A2) -> B2`.

```
cast ((x : A1) -> B1) ((x : A2) -> B2) e f
  ⇝ l (x : A2). cast (B1 (back x)) (B2 x) (cod-eq x) (f (back x))
  where back x = cast A2 A1 (sym dom-eq) x
  and dom-eq : Eq Type A1 A2,
      cod-eq x : Eq Type (B1 (back x)) (B2 x)
  are the structural decompositions of Eq Type ((x:A1)->B1) ((x:A2)->B2)
```

In words: to transport a function from `A1 -> B1` to `A2 -> B2`, bind `x :
A2`, coerce it backwards to `A1`, apply `f`, and coerce the result
forwards to `B2 x`. The sub-equality proofs `dom-eq` and `cod-eq` are
recovered from the Eq-by-type reduction at Type/Pi (par. 2) -- the kernel
reduces `Eq Type ((x:A1)->B1) ((x:A2)->B2)` structurally to obtain the
domain and codomain equalities, and uses them (or their symmetries) in the
recursive `cast` calls.

**Neutral case.** When `f` is neutral, or when the sub-equalities `dom-eq`
or `cod-eq` are neutral (because `A1`/`A2`/`B1`/`B2` are neutral), the
outer `cast` is neutral.

---

**Sigma-type.** `A` is `(x : A1) x B1`, `B` is `(x : A2) x B2`.

```
cast ((x : A1) x B1) ((x : A2) x B2) e p
  ⇝ (cast A1 A2 dom-eq p.1 ,
     cast (B1 p.1) (B2 (cast A1 A2 dom-eq p.1)) cod-eq' p.2)
  where dom-eq : Eq Type A1 A2,
        cod-eq' : Eq Type (B1 p.1) (B2 (cast A1 A2 dom-eq p.1))
  are the structural decompositions of Eq Type ((x:A1)xB1) ((x:A2)xB2)
```

The second component's cast uses `cod-eq'` at the specific `p.1`, not a
family-level equality -- the codomain equality is instantiated at the
transported first component.

**Neutral case.** When `p` is neutral, or when the sub-equalities are
neutral, the outer `cast` is neutral.

---

**Inductive type.** `A` is `D Delta_p i-bar`, `B` is `D Delta_p j-bar`.
Both are the same inductive family `D`. `a = c_k a-bar` (constructed by
`c_k`). The constructor is `c_k : (x_1:A_1) … (x_n:A_n) → D Delta_p
t-bar_k(x-bar)`, so its *target indices* `t-bar_k` are expressions in the
constructor arguments, and the scrutinee's source indices satisfy
`i-bar = t-bar_k(a-bar)`.

There are two regimes, split by whether the family **indices** change.

*Parameters only (`i-bar ≡ j-bar`).* Each constructor argument keeps its
value and is re-typed under the target parameters; an argument whose type
does not depend on the changed parameter transports by regularity:

```
cast (D p-bar i-bar) (D p-bar' i-bar) e (c_k a-bar)
  ⇝ c_k a-bar'        where a_l' = cast A_l[a-bar'_<l, p-bar]
                                        A_l[a-bar'_<l, p-bar'] eq_l a_l
```

*Index rewrite (`i-bar ≢ j-bar`) — the suc-injectivity seam.* The naive
move ("keep the index argument, re-head the constructor at `j-bar`") is
**unsound**: a constructor argument's type is written in terms of *earlier
arguments* (`vcons`'s `xs : Vec A x_1`), not the family index, so
substituting `j-bar` for `i-bar` in the argument types is a **no-op that
leaves the reduct ill-typed** (`xs : Vec A n` under a `Vec A (suc m)`
head). The reduction must instead **rewrite the index-determining
arguments** and **sub-cast the dependent arguments**, driven by the
decomposition of the index equality:

1. **Decompose the type equality (the *same* path as `Eq`-at-inductive).**
   `Eq Type (D p-bar i-bar) (D p-bar j-bar)` reduces to parameter
   equalities and one **index equality** `eq_idx,m : Eq (IdxT_m) i_m j_m`
   per index position `m`. This decomposition is the **same** `Eq`-at-
   inductive machinery (§2.2) being completed in parallel (the mutual
   sibling) — `cast` consumes it rather than re-implementing an index
   comparison, so `cast` and `Eq` cannot diverge on what "canonically
   decomposes" means.
2. **Invert each index constructor (suc-injectivity).** Where the
   constructor's index expression applies an index constructor to a
   forced argument — `t_k,m = suc x_l`, and `i_m = suc(v)`, `j_m = suc(w)`
   are headed by that same constructor — the `Eq`-at-inductive rule
   (§2.2) decomposes `eq_idx,m` to the **argument-level** equality
   `eq' : Eq (…) v w`, exposing the forced argument's new value `w` on the
   `j-bar` side.
3. **Rebuild `c_k`.** Form `a-bar'` left-to-right:
   - a **forced** argument (one the index expression `t-bar_k` is built
     from) takes its **`j-bar`-side value**, read off by peeling the index
     constructors of `j-bar` that `t-bar_k` applies (`vcons`'s `n ↦ m`);
   - a **non-forced** argument transports along its type equality
     (regularity when index-independent: `cast A A refl a ⇝ a`);
   - a **dependent/recursive** argument whose type mentions a forced
     argument is **sub-cast** along the induced equality
     (`cong` of the family at `eq'`).
   Re-apply: `c_k a-bar' : D p-bar j-bar`.

**Worked example (Vec).** `vcons : (x_1:Nat)(x_2:A)(x_3:Vec A x_1) → Vec A
(suc x_1)`. Casting `vcons n a xs : Vec A (suc n)` to `Vec A (suc m)`:

```
cast (Vec A (suc n)) (Vec A (suc m)) e (vcons n a xs)
  ⇝ vcons m a (cast (Vec A n) (Vec A m) (cong (Vec A) eq') xs)
  where eq' : Eq Nat n m  is the suc-injective decomposition of
        eq_idx : Eq Nat (suc n) (suc m)
```

The forced index argument `n` becomes `m` (read off the target index
`suc m`), the element `a : A` is unchanged (regularity), and the recursive
`xs : Vec A n` is sub-cast to `Vec A m`. The reduct is constructor-headed
at the **target** index — the cast computed through, not stuck — and is
well-typed (`vcons m a (…:Vec A m) : Vec A (suc m)`), which the removed
naive rewrite was not.

**Where the guard sits.** The index rewrite fires **only** when every
index equality `eq_idx,m` decomposes **canonically** — `i_m` and `j_m` are
headed by the **same index constructor**, so the forced arguments are
exposed. If any index equality is **neutral** (an index is a variable, not
constructor-headed, or the two heads differ without the structural
decomposition reducing), the cast stays **neutral** — it does **not**
fabricate a re-indexed constructor. (When the indices are headed by
*different* constructors, `Eq Type (D … i-bar)(D … j-bar)` reduces to
`Bottom` by §2.2, so the proof `e : Bottom` is unavailable except in an
inconsistent context, and the cast is never formed on a closed term —
consistent with `cast` never inspecting `e`.) This gate is the K2
closed-`Empty` discipline applied to index transport: the rewrite is
gated on the index condition being **structurally derivable**, never
deferred past a fired redex. Index maps that are not constructor patterns
(non-invertible index expressions) remain **sound-stuck** — out of scope,
not wrong.

**Neutral case.** When the scrutinee is not constructor-headed, `cast` is
neutral.

---

**Quotient type.** `A` is `A1 / R`, `B` is `A2 / S`.

```
cast (A1 / R) (A2 / S) e [a]
  ⇝ [cast A1 A2 e0 a]
  where e0 : Eq Type A1 A2 is the underlying type equality from the
  structural decomposition of Eq Type (A1/R) (A2/S)
```

The proof `e : Eq Type (A1/R) (A2/S)` reduces structurally (par. 2) to
`Eq Type A1 A2 and` compatibility of `R` with `S`; `cast` uses the
underlying type equality `e0` to transport the representative, and the
quotient class wrapping preserves the quotient structure.

**Neutral case.** When `a` is not headed by `[_]`, or when `e0` is
neutral, `cast` is neutral.

---

**Omega.**

```
cast Omega_l Omega_l e P  ⇝  P
```

Transport at the Omega universe is the identity: `Omega_l` is a universe
constant on both sides, so `cast` returns `P` unchanged — the same
behaviour as `cast` at any universe (`Type`, `Omega`). This is justified
by the `e : Eq Type (Omega_l) (Omega_l)` reducing structurally to `refl`
(§2.2, `Type` rule), triggering regularity, except in the (oracle) non-refl
Type case where `cast` at a universe is stuck. Note: this is about
transport of propositions *as elements* of Omega, not about proof
irrelevance (which says proofs of a *single* proposition are equal, §1.2).

---

**Type universe.** (oracle)

```
cast Type l Type l (refl _) A  ⇝  A
```

Cast at the universe level is restricted: `cast Type l Type l e A` only
reduces when `e` is `refl`. On a non-refl type-equality at `Type l`,
`cast` is neutral. (This is safe because the structural type equality is
definitional -- see par. 2.2 -- and the kernel never needs to transport
*types* across type-equalities; values only.)

---

**Mismatched heads.** When `A` and `B` have **different head formers**
(e.g. Pi vs Sigma), `cast A B e a` is **ill-typed** (the proof `e : Eq
Type A B` would have to witness an impossible equality, and `Eq Type A B`
would reduce to `Bottom` by par. 2's structural type-equality rule -- so
`e : Bottom` is impossible in a consistent context). In the well-typed
case, heads always match.

### 3.3 Termination of Eq/cast mutual recursion

`Eq`-by-type (par. 2) and `cast`-by-type (par. 3) form a **single mutual
reduction system**: `Eq` at a compound type may call `cast` on component
types (Σ, inductive), and `cast` at a compound type calls `Eq` to
decompose the type-equality proof into sub-equalities. This mutual
recursion **terminates structurally** on the type `A` being traversed:

- Each `Eq` reduction at type `A` reduces the problem to sub-equalities at
  **strictly smaller** types (the domain/codomain for Π, the component
  types for Σ, the argument types for an inductive constructor).
- Each `cast` reduction at type `A` reduces to transporting at
  **strictly smaller** types (the domain/codomain for Π, the component
  types for Σ, the constructor arguments for an inductive).
- The type structure is a finite tree; structural descent bottoms out at
  primitive types, neutral types, Omega, or the Type universe.
- **The index-rewrite edge (§3.2 "Index rewrite") adds no divergence.**
  Completing `cast`-at-inductive on an index change calls `Eq`-at-inductive
  to **decompose the index equality** — but that decomposition is at the
  **index types** (`Nat` for `Vec`), which are arguments of, hence
  **strictly smaller than**, the inductive type `D Delta_p i-bar` being
  traversed; and the sub-casts it emits are at the family at smaller
  indices (`Vec A n` under `Vec A (suc n)`). So the new mutual edge
  (`cast`-at-inductive → `Eq`-at-inductive on indices → sub-`cast`s)
  descends on the same finite type-tree measure and bottoms out. The guard
  (canonical decomposition only, §3.2) means a non-decomposing index leaves
  the cast neutral — no edge is taken at all.

The K1 conversion algorithm's structural termination argument
(`14-inductive.md §9.2`) extends directly to this mutual system: each
step reduces the type being traversed. The full decidable conversion with
NbE (K2c, `17`) subsumes this structural argument with a semantic
termination proof.

### 3.4 Key properties

- **`cast` never inspects the proof `e`.** It computes by recursion on the
  type structure of `A` and `B`. The proof is only passed along to
  recursive calls or used at regularity. This means OTT tolerates adding
  consistent axioms without breaking canonicity.
- **Regularity.** `cast A A refl a ≡ a` holds definitionally.
- **Canonicity.** On closed, canonical type-equalities, `cast` reduces to
  a constructor form or lambda (never stuck at a non-base type). This
  guarantees `J` reduces on non-`refl` (par. 4, `15-identity.md` par. 4).

## 4. Derived equalities (theorems, mostly definitional)

Everything `15-identity.md` exposes is derived here and computes:

- **`subst` / `J` / transport** --
  `subst P (e : Eq A a b) : P a -> P b := cast (P a) (P b) (cong P e)`.
  Because `cast` computes (par. 3), `J` **reduces on non-`refl`**
  (`15-identity.md` par. 4), via observational equality.
- **funext** -- *definitional* (`Eq` at a Pi-type **is** pointwise `Eq`,
  par. 2). A major ergonomic and verification win.
- **propext** -- *definitional* (`Eq` at Omega is mutual implication,
  par. 2).
- **UIP / proof irrelevance** -- *definitional* (`Eq : Omega`, par. 1).
- **`sym`, `trans`, `cong`** -- derivable; all compute.

  - `sym e : Eq A b a` where `e : Eq A a b`: obtained by using `e` to
    derive `Eq Type A A` (via `refl` or `trans`), then `cast` the
    `refl a : Eq A a a` along the equality of the second index. Computes
    because `cast` computes.
  - `trans e1 e2 : Eq A a c` where `e1 : Eq A a b` and `e2 : Eq A b c`:
    `cast` `e2` along a type equality (the dependent family `Eq A a`).
    Computes because `cast` computes.
  - `cong f e : Eq B (f a) (f b)` where `f : (x:A) -> B x` and
    `e : Eq A a b`: defined as `subst (x. Eq (B x) (f a) (f x)) e (refl
    (f a))`. Computes because `subst`/`cast` computes.

So Ken is a **set-level** theory: every type's equality is a proposition
with UIP. There is no higher path structure (no `Eq (Eq A a b) p q`
content) -- which is exactly right for software data (ADR 0005).

### 4.1 `J` at a dependent motive (the non-constant-motive rule)

`J` (`15 §4`) is `cast` at the singleton type. On a non-`refl` equality it
reduces to a transport of the base case:

```
J A a P d b e   ⇝   cast (P a (refl a)) (P b e) pair-eq d        (J-cast)
```

(the `refl` case is `J-β`, reducing to `d` — `15 §4.2`). This rule fires
for **every** non-`refl` `e`; the motive being **constant or dependent is
not a side condition** — both are subsumed by the single `cast`:

- **Constant motive.** `P a (refl a) ≡ P b e` (the two instantiations are
  convertible), so `pair-eq` is `refl` and the cast reduces by
  **regularity** (§3.2) to `d` — the headline non-`refl`-`J` computation,
  unchanged.
- **Dependent motive.** `P a (refl a)` and `P b e` are **different** type
  expressions; the cast computes by **`cast`-by-type** (§3.2) on their
  structure — descending into Π/Σ, and **through the inductive index
  rewrite (§3.2 "Index rewrite")** when `P` lands in an indexed family
  (`P b e = Vec A (f b)`). This is the seam K2 left stuck.

**`pair-eq` is a typing witness, not a computation driver.** The proof
`pair-eq : Eq Type (P a (refl a)) (P b e)` is built by the fixed singleton
schema of `15 §4.1`: from `Eq S (a, refl a) (b, e)` where `S := (b':A) ×
Eq A a b'` (its first conjunct is `e`, its second is trivial by Ω-PI),
apply `cong (λ s:S. P s.1 s.2)`. Because **`cast` never inspects its proof**
(§3.4), `pair-eq` is needed **only** to make `(J-cast)` well-typed — the
*computation* is entirely the `cast`-by-type on the two endpoint types.
The kernel therefore **synthesizes** `pair-eq` by this schema (any witness
of that `Omega` type serves; one exists because `a = b` via `e` and `refl
a = e` via Ω-PI) and never branches on the motive's shape.

**Firing and the residual (stress-tested at an open proof).** Do not state
that a dependent-motive `J` is "stuck": `(J-cast)` **fires**, and the cast
makes structural progress as far as the endpoint types are determined. The
residual neutrality lives precisely at **`cast`'s** guards — a Π/Σ head
descends; an indexed-inductive endpoint descends only when its index
equality decomposes canonically (§3.2 guard). When `e` is **open** and `P`
depends on the index through `e` (so the index equality needs `e` and stays
neutral), the cast halts there as a neutral `Cast` — `J` has still reduced
(to that cast), and the stall is the **seam-1 index guard**, not a special
`J` rule. (Asking "does this redex fire when the proof is abstract?": yes —
`(J-cast)` fires on any non-`refl` `e`; only the inner `cast` may stall.)

**Termination.** `(J-cast)` fires once and hands off to `cast`-by-type,
which terminates by structural descent on the finite type `P a (refl a)`
(§3.3). No new recursion enters `whnf`; the K2c decidability/SCT gate (`17
§4`) is unaffected (it already scores `cast` under recursion as `?`).

## 5. Quotient types

Set-quotients are **native** (not HITs):

```
  Gamma |- A : Type l      Gamma |- R : A -> A -> Omega
  ────────────────────────────────────────────────────────  (Quot-Form)
  Gamma |- A / R : Type l
```

- **Introduction:** `[a] : A / R` for `a : A`. This is the equivalence
  class of `a`.
- **Equality:** `Eq (A/R) [a] [b]` reduces to `R a b` (par. 2) -- quotient
  equality *is* the user relation. This means the quotient's equality is
  definitionally the relation -- no extra axioms, no setoid boilerplate.
- **Elimination:**
  ```
    Gamma |- M : (z : A/R) -> Type k
    Gamma |- f : (x : A) -> M [x]
    Gamma |- r : (x y : A) -> R x y -> Eq (M [x]) (f x)
                    (cast (M [y]) (M [x]) (sym (cong M (R x y))) (f y))
    Gamma |- q : A / R
    ─────────────────────────────────────────────────────────  (Quot-Elim)
    Gamma |- elim_/ M f r q : M q
  ```
  The respect condition `r` records that `f` sends `R`-related elements
  to equal results in `M`. The equality type uses `cast` to handle any
  dependence of `M` on the class representative.

  **Computation:**
  ```
  elim_/ M f r [a]  ≡  f a  :  M [a]
  ```
  This is the i-reduction for quotients: eliminating on a class just
  applies the underlying function.

  **Respect-free elimination (Omega target).** When `M z : Omega` for all
  `z`, the respect condition `r` is **free** by Omega-PI (par. 1.2): any
  two proofs of `Eq (M [x]) ...` in Omega are definitionally equal, so the
  kernel can fill a trivial proof. This makes quotients into Omega
  convenient and coherence-free.

- Quotients give `Int`-as-quotient, finite maps/sets up to equivalence,
  and the set-level constructions HITs would have provided. *General*
  quotient-inductive types (QITs) are a possible later extension
  (blueprint: QITs-in-OTT); K2 delivers set-quotients.

### 5.1 Respect verification (the `cong`/`cast` schema for non-Ω targets)

The eliminator's admission **gates on the respect proof `r`**. The kernel
dispatches on the **sort of the motive's codomain** — `M : (z : A/R) →
S`, where `S` is `whnf`'d:

- **`S = Omega_l` (proposition target) — respect-free.** When every `M z`
  is a proposition, any two inhabitants of `M [x]` are definitionally equal
  by Ω-PI (§1.2), so the respect equality holds for free; the kernel fills
  a trivial proof and **requires only that `r` be well-scoped**. (The test
  is on the codomain **sort** — `typeOf(M z) = Omega_l`, i.e. `M z : Ω` —
  not `M z ≡ Omega_l`; PI is about *elements of a proposition*, §1.2.) This
  is the K2 deliverable and is **unchanged** — do not regress it.

- **`S = Type ℓ` (genuine type target) — the full schema.** Here respect
  is the **entire** soundness content: without it a non-respecting `f`
  (one that *observes* the class representative) would let `elim_/`
  distinguish `R`-related elements, and `cong` of the observation across
  `Eq (A/R) [x] [y] ⇝ R x y` (§2.2) would derive `Eq Bool true false ⇝
  Bottom` — a **closed inhabitant of `Empty`**. So the kernel MUST verify
  `r` against the exact `cong`/`cast` schema:

  ```
  r : (x y : A) → (h : R x y)
        → Eq (M [x]) (f x) (cast (M [y]) (M [x]) (sym (cong M h')) (f y))
    where h' : Eq (A/R) [x] [y]   is h transported through the quotient-Eq
          reduction Eq (A/R) [x] [y] ⇝ R x y (§2.2), and
          cong M h' : Eq Type (M [x]) (M [y])   (a proof in Omega, §4)
          sym (cong M h') : Eq Type (M [y]) (M [x])   — the transport
              direction: with the kernel convention cast A B e (a:A) : B
              (§3.1), the value f y : M [y] is the source and the result
              lands in M [x] (the type the enclosing Eq (M [x]) … requires)
  ```

  i.e. `r` must prove that `f x` equals — at the type `M [x]`, after
  transporting `f y` from `M [y]` to `M [x]` along the motive's action on
  the class equality — the result `f y`. The kernel **forms this expected
  type and `check`s `r` against it** (`check Gamma r expected`); admission
  **fires only if that check succeeds**. An `r` that does not inhabit the
  schema — because `f` genuinely fails to respect `R` — is **rejected**.

**Where the guard sits.** Admission is gated on the schema `check`, never
deferred: the eliminator is added to the term language only after `r` is
verified against the exact respect type (Type target) or confirmed
well-scoped (Ω target). This is the K2 closed-`Empty` discipline — an
un-invoked respect check while `elim_/` reduces unconditionally
(`elim_/ M f r [a] ⇝ f a`, §5) would be an unsound **accept**, not a sound
stuck fallback. The **i-reduction itself is unchanged**; respect is purely
an admission-time obligation, so completing it adds **no** new reduction to
`whnf` and cannot affect conversion termination.

**Adversarial (the `Empty` probe).** `A/R = Bool/(λ_ _. Top)` (the total
relation, collapsing `Bool` to one class), `M := λ_. Bool` (a **Type**
target), `f := λx. x` (observes the representative). No valid `r` exists:
`r` would have to prove `Eq Bool (f true) (cast … (f false))`, i.e. `Eq
Bool true false ⇝ Bottom` — uninhabited. So the elim is **rejected** (the
verdict flips: a *respecting* `f`, e.g. `λ_. true`, supplies `r : Eq Bool
true true ⇝ Top` and is **accepted**). A kernel that raw-well-formed `r`
instead of checking the schema would accept the observing `f` and reduce
`elim_/ … [true]` to a closed proof of `Empty`.

**The transport direction is load-bearing — verify it at a *dependent*
motive.** The probe above uses a **constant** motive (`M := λ_. Bool`), so
`M [x] ≡ M [y]` and the schema's `cast (M [y]) (M [x]) …` collapses by
**regularity** (`cast B B refl a ⇝ a`, §3.2) *regardless of source/target
order* — it confirms the respect *check fires* but cannot witness the
**direction**. A conforming kernel MUST therefore also be exercised at a
**dependent** motive where `M [x] ≢ M [y]` (only `[x] = [y]`
*propositionally*, so the two motive instances are not definitionally
equal): the **correct-direction** respect proof — written by the user as
`cast (M [y]) (M [x]) (sym (cong M h')) (f y)` — must be **accepted**, and a
**reversed** one (`cast (M [x]) (M [y]) (cong M h') (f y)`) must be
**rejected** (it is ill-typed: it feeds `f y : M [y]` to a cast whose source
is `M [x]`, and lands in `M [y]` where the enclosing `Eq (M [x]) …` requires
`M [x]`). The verdict thus **flips on the transport direction itself** — the
dimension a constant motive holds fixed. (Conformance:
`conversion/quotient-respect-schema-dependent-motive`; this is the
discriminating case the constant-motive probe structurally cannot be.)

## 6. Propositional truncation

`||A|| : Omega` is the **propositional truncation** of `A` -- `A` squashed
to a mere proposition (a quotient of `A` by the total relation, landing in
Omega).

- **Formation:** `||A|| : Omega` for `A : Type l`. (Since Omega : Type 1,
  this is well-formed; level polymorphism covers larger `l`.)
- **Introduction:** `|a| : ||A||` for `a : A`. Any element of `A` can be
  injected into the truncation.
- **Elimination:** to eliminate `||A||` into any `P : Omega`, provide a
  map `f : A -> P`:
  ```
    Gamma |- P : Omega      Gamma |- f : A -> P
    Gamma |- t : ||A||
    ──────────────────────────────────────────────  (Trunc-Elim)
    Gamma |- elim_trunc P f t : P
  ```
  The computation rule:
  ```
  elim_trunc P f |a|  ≡  f a
  ```
  The "respect" side-condition is free because the target `P` is in Omega
  (par. 1.2), so `f` is automatically constant on the total relation.

- **Derived logical operations.** Omega's `or` and `exists`:
  - `P or Q := || P + Q ||` where `+` is the binary sum inductive from
    K1 (`14-inductive.md`).
  - `exists x:A. P x := || (x:A) x P x ||`.
  These land in Omega because truncation lands in Omega, matching the
  Heyting structure (par. 1.3).

Because the target is in Omega, truncation is itself proof-irrelevant.

## 7. What is deliberately absent (vs the cubical alternative)

Not in Ken (ADR 0005): the **interval** and dimension variables;
**cofibrations** and partial elements; `transp`/`hcomp`/`comp`; **`Glue`**
and **computational univalence**; **`PathP`** / heterogeneous paths; and
**higher inductive types**. These buy univalence + higher-dimensional
structure, which software does not use, at the cost of the largest and
most canonicity-fragile part of a cubical kernel. Ken trades them for a
smaller, set-level, UIP-validating core.

## 8. Definitional equations -- conversion extension

The observational layer extends K1's `convert` (`17-conversion.md`) with
the following algorithmic rules. K1's structural comparison (alpha
equivalence, reduce-to-normal-form, type-directed eta) remains unchanged;
K2 adds these WHNF reduction rules and one shortcut.

### 8.1 WHNF reduction rules

The kernel's `whnf` (weak-head normalisation) is extended with the
reduction rules from par. 2 (Eq-by-type) and par. 3 (cast-by-type). The
rule selection is:

1. **Eq reduction.** When `whnf` encounters `Eq A a b`:
   - Compute `whnf(A)`.
   - If `A` is canonical (reduces to a type former), apply the
     corresponding reduction from par. 2.2.
   - If `A` is neutral, `Eq` is neutral -- no further reduction.
   - The recursive `Eq` reductions inside the result (e.g. the `Eq A_j a_j
     b_j` conjuncts from inductive case) are reduced lazily by subsequent
     `whnf` calls.

2. **cast reduction.** When `whnf` encounters `cast A B e a`:
   - Compute `whnf(A)` and `whnf(B)`.
   - If both are canonical with the **same head**, apply the corresponding
     structural reduction from par. 3.2.
   - If `A ≡ B` and `e` is `refl`, apply regularity: reduce to `a`.
   - If heads differ, or either is neutral, `cast` is neutral.

3. **Omega-PI shortcut.** When `whnf` encounters a stuck comparison at
   Omega type, no reduction is needed -- the conversion checker handles it
   directly (par. 8.2).

### 8.2 Omega proof-irrelevance in conversion

During structural comparison (type-directed, `17` par. 2), when the
conversion checker compares two terms at a type `T`:

- If `T` is known to be in Omega (by checking `T : Omega` against the
  context), the checker returns `true` **immediately** without inspecting
  the terms. This is the Omega-PI shortcut -- a constant-time path.
- To determine "`T` is in Omega": the checker can test `check Gamma T
  Omega` (a fast universe-membership check) or maintain a flag from the
  binder type. The former is adequate -- Omega membership is decidable and
  cheap.
- The propositional-argument skip: during structural comparison of
  applications `h a1 ... an` vs `h b1 ... bn`, if a binder position `i` has
  type in Omega, the argument pair `(ai, bi)` is skipped (treated as
  definitionally equal). This applies recursively: any sub-term at Omega
  type is definitionally trivial.

### 8.3 Interaction with K1 conversion

K1's `convert` function (`17`) operates as a type-directed structural
comparison with WHNF reduction. The K2 extension:

1. **Plug into `whnf`**: the rules of par. 8.1 are added to the WHNF
   reduction engine. No change to the call-site in `convert`.
2. **Omega shortcut before structural descent**: before comparing two
   terms structurally, check if the type (from the context/binder) is in
   Omega. If so, return `true`.
3. **All other K1 behaviour unchanged**: alpha equivalence,
   reduce-to-normal-form, eta for Pi/Sigma, i/delta reduction -- these
   stay exactly as in K1.

### 8.4 Subject reduction across K2 rules

Subject reduction must hold for the extended reduction system:
if `Gamma |- t : A` and `whnf(t) = t'`, then `Gamma |- t' : A`. The
critical cases:

- Eq reduction (par. 2.2): `Eq A a b : Omega`, the reduct is also in
  Omega by the corresponding Omega formation rules.
- cast reduction (par. 3.2): `cast A B e a : B`, and each recursive
  `cast` in the reduct has its target type preserved by the structural
  decomposition.
- cast at an inductive **index change** (par. 3.2 "Index rewrite"):
  `cast (D p-bar i-bar) (D p-bar j-bar) e (c_k a-bar) ⇝ c_k a-bar'`, and
  the reduct is constructor-headed at the **target** indices —
  `c_k a-bar' : D p-bar j-bar = B` — because each forced argument takes its
  `j-bar`-side value and each dependent argument is sub-cast to its
  `j-bar`-instance type (so it is the unsound naive rewrite, which left
  `a-bar` at the source index, that *failed* subject reduction; the
  decomposition-driven reduct restores it).
- `J` at a dependent motive (par. 4.1): `J A a P d b e ⇝ cast (P a (refl
  a)) (P b e) pair-eq d : P b e`, the declared result type — `pair-eq`
  witnesses `Eq Type (P a (refl a)) (P b e)`, so the `cast` lands in
  `P b e`.
- Quotient elim: `elim_/ M f r [a] ⇝ f a : M [a]`, which is the type of
  `f` applied to `a` — sound for a **Type** target precisely because
  admission verified `r` against the respect schema (par. 5.1), so `f`
  is well-defined on classes; the i-reduction is unchanged.
- Truncation elim: `elim_trunc P f |a| ⇝ f a : P`, preserved.

The subject-reduction argument for the full OTT system is proved in the
references (`TTobs`/`CICobs`, ADR 0005); the kernel encodes the reduction
rules such that this holds operationally.

## 9. What the kernel checks here

A conforming kernel MUST:

1. Provide the strict proposition universe **Omega** with **definitional
   proof irrelevance** (par. 1.2).
2. Compute **`Eq`-by-type** (par. 2.2) including definitional **funext**
   (Eq at Pi) and **propext** (Eq at Omega).
3. Provide **`cast`** with **`cast`-refl regularity** and
   **`cast`-by-type** computation (par. 3.2), from which `subst`/`J`
   **reduce on non-`refl`** (par. 4, `15-identity.md` par. 4).
4. Provide **quotient types** `A / R` with the relation-as-equality and
   the respect-checked eliminator (par. 5).
5. Provide **propositional truncation** `||A||` (par. 6).
6. Extend **conversion** with the reductions and Omega-PI shortcut
   (par. 8), leaving K1's structural comparison unchanged.
7. Preserve **subject reduction** across all K2 reductions (par. 8.4).

The soundness-critical, separately-tested behaviours:

| # | Behaviour | Spec | Conformance |
|---|-----------|------|-------------|
| C1 | Omega-PI: any two proofs of `P : Omega` are definitionally equal | par. 1.2 | `observational/omega-pi-convertible` |
| C2 | `Eq` at Pi reduces to pointwise `Eq` (definitional funext) | par. 2.2 | `observational/funext-definitional` |
| C3 | `Eq` at Omega reduces to mutual implication (definitional propext) | par. 2.2 | `observational/propext-definitional` |
| C4 | `Eq` at inductive: same ctor → conj, diff ctors → Bottom | par. 2.2 | `observational/eq-inductive` |
| C5 | `cast A A refl a` reduces to `a` (regularity) | par. 3.2 | `observational/cast-refl` |
| C6 | `cast` computes on closed canonical type-equalities (canonicity) | par. 3.2 | `observational/cast-computes` |
| C7 | `J` reduces on non-`refl` equality (via cast) | par. 4 | `observational/j-nonrefl` |
| C8 | `Eq (A/R) [a] [b]` reduces to `R a b` | par. 5 | `observational/quotient-eq` |
| C9 | `elim_/ M f r [a]` reduces to `f a` | par. 5 | `observational/quotient-elim` |
| C10 | `elim_trunc P f |a|` reduces to `f a` | par. 6 | `observational/trunc-elim` |
| C11 | All K1 rules preserved (no regression) | par. 8.3 | CI on K1 conformance subset |
| C12 | `cast` at an inductive **index change** computes through (suc-injectivity decomposition + sub-cast) to the target-indexed constructor; a non-canonical/neutral index stays stuck | par. 3.2 "Index rewrite" | `observational/cast-inductive-index` |
| C13 | `Eq` at an inductive with a **dependent telescope** decomposes with the inter-argument `cast`s (the mutual sibling of C12) | par. 2.2 | `observational/eq-inductive-dependent` |
| C14 | `J` reduces at a **dependent (non-constant) motive** via `cast` on the endpoint types (bottoming through C12); stays stuck only where the inner `cast` stalls on an open index | par. 4.1 | `observational/j-dependent-motive` |
| C15 | quotient elim into a **Type** target checks `r` against the `cong`/`cast` respect schema — a respecting `f` is **accepted**, a non-respecting `f` is **rejected** (closed-`Empty` guard); the **transport direction** `cast (M [y]) (M [x]) (sym (cong M h'))` is exercised at a **dependent** motive (`M [x] ≢ M [y]`): correct-direction `r` accepted, reversed rejected; Ω targets stay respect-free | par. 5.1 | `conversion/quotient-respect-schema{,-dependent-motive}` |

Conformance corpus: `../../conformance/kernel/observational/` and
`../../conformance/kernel/conversion/`. C12–C15 are the **series-2**
obs-reduction completions (cast-at-inductive index rewrite, the mutual
`Eq`-at-inductive dependent telescope, non-constant-motive `J`, full
quotient `respect`); each is **discriminating** (verdict-flips or asserts a
structural reduct), and each carries the adversarial "would this seam
inhabit `Empty`?" case (ill-justified index → stuck; non-respecting `f` →
rejected).

### 9.1 Oracle-tagged behaviours

Items below are tagged **(oracle)** in par. 3.2: the reduction at
`cast Type Type` on non-refl equalities, and certain edge cases in
quotient transport. These are to be validated against the prototype's
observed behaviour by the Spec enclave at build time. The rules as given
represent the expected OTT behaviour; if the prototype diverges, the Spec
enclave updates the spec to match the observed behaviour (or documents the
divergence).
