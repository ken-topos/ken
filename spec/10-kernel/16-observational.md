# Observational equality and computation

> Status: **DRAFT v0**. Normative for the *interface and computation
> commitments*; exact reduction forms are tagged **(blueprint)** against
> `CICobs` / `TTobs` / `CCobs` (ADR 0005). This is the machinery
> `15-identity.md` reuses: the strict proposition universe **Ω**, observational
> equality **`Eq`** computed by recursion on type structure, the **`cast`**
> coercion, **quotient types**, and **propositional truncation**. It replaces
> the cubical layer (ADR 0005): there is **no** interval, cofibration,
> `transp`/`hcomp`/`comp`, `Glue`, univalence, or higher inductive type. Ken is
> a **set-level** theory (UIP holds), which is what software is.

## 1. The strict proposition universe Ω (`SProp`)

Ken has a universe **Ω** of **strict (definitionally proof-irrelevant)
propositions** — the subobject classifier and the home of equality and the
logic.

```
  Γ ⊢ P : Ω      Γ ⊢ p : P      Γ ⊢ q : P
  ───────────────────────────────────────────  (Ω-proof-irrelevance)
  Γ ⊢ p ≡ q : P
```

- Any two proofs of a proposition are **definitionally equal**. So **UIP** and
  proof irrelevance hold by construction, and the conversion checker may **skip
  the contents of propositional arguments** entirely (`17 §2`) — which also
  means agents need not synthesise coherence/transport terms for them.
- Ω is **predicative** (consistent with `12 §2`): it is not an impredicative
  sort (OQ-Prop DECIDED, ADR 0005: impredicativity ruled out). `Ω : Type 1`;
  level-polymorphic `Ω ℓ` for larger props (`12 §5.3`).
- Ω carries the **Heyting** structure (`12 §5.2`) — `⊤`, `⊥`, `∧`, `⇒`, `¬`,
  plus `∨` and `∃` via truncation (§6); intuitionistic, not Boolean. Excluded
  middle is not assumed (it holds for *decidable* props as data).
- **`Eq` (below) lands in Ω**, so equality is a proof-irrelevant proposition.

## 2. Observational equality `Eq`

Propositional equality is **`Eq A a b : Ω`**, *computed by recursion on the
structure of the type `A`* — equality "observes" the type. `refl a : Eq A a a`.
The defining computations (the heart of OTT; exact forms **(blueprint)** per
`TTobs`/`CICobs`):

| `A` is… | `Eq A a b` reduces to |
|---|---|
| a **Π-type** `(x:A')→B` | `(x : A') → Eq (B x) (a x) (b x)` — **funext is definitional** |
| a **Σ-type** `(x:A')×B` | `Eq A' a.1 b.1 ∧ Eq (B b.1) (cast … a.2) b.2` — componentwise |
| an **inductive** `D` | structural: same constructor ⇒ conjunction of arg-equalities; different constructors ⇒ `⊥` |
| a **quotient** `A'/R` | `R a b` (the user relation, §5) |
| the **universe** `Type ℓ` | structural type equality (§3) — **not** univalence |
| Ω | mutual implication (**propext**): `(a ⇒ b) ∧ (b ⇒ a)` |
| a **primitive** (`Int`,…) | decided equality of literals (`14 §5`) |

Because `Eq` is in Ω it is proof-irrelevant (§1): there is at most one proof of
`Eq A a b` up to conversion, so equality reasoning never carries coherence
baggage. On a **neutral** `A` or neutral arguments, `Eq` is a neutral
proposition.

## 3. Type equality and `cast` (transport)

Equality of **types** is **structural, not univalent**: `Eq Type A B` holds when
`A` and `B` have the **same head former with equal parts** — `Eq Type
((x:A₁)→B₁) ((x:A₂)→B₂)` reduces to `Eq Type A₁ A₂ ∧ …`, two inductives are
equal iff the same family at equal parameters/indices, etc. There is **no** `(A
≃ B) → Eq Type A B` (that is univalence, deliberately absent, ADR 0005).

**`cast`** transports a value along a type equality:

```
  Γ ⊢ A : Type ℓ   Γ ⊢ B : Type ℓ   Γ ⊢ e : Eq Type A B   Γ ⊢ a : A
  ─────────────────────────────────────────────────────────────────────  (cast)
  Γ ⊢ cast A B e a : B
```

with two commitments:

- **Reflexivity computes (regularity):** `cast A A refl a ≡ a`. Unlike De Morgan
  cubical (where constant compositions are *not* constructively the identity),
  OTT's `cast` on a reflexive proof **reduces to the input** — a cleaner
  equational theory and a real simplicity win.
- **`cast`-by-type:** on a canonical type-equality `cast` reduces **by recursion
  on the type structure** (push into Π domain/codomain, Σ components, inductive
  constructor arguments, quotient classes) — the mechanism that makes
  transport/`subst`/`J` **compute on non-`refl`** (§4). Exact rules
  **(blueprint)** per `CICobs §cast`. On a neutral type-equality, `cast` is
  neutral.

**The canonicity-friendly property:** the equality *eliminator* (`cast`, `J`)
computes from the **endpoints**, and **never inspects the equality proof**. This
is precisely why OTT tolerates additional *consistent* axioms without breaking
canonicity (`README.md §5`) — a robustness that matters when a fleet of agents
throws generated proofs at the checker (ADR 0005).

## 4. Derived equalities (theorems, mostly definitional)

Everything `15-identity.md` exposes is derived here and computes:

- **`subst` / `J` / transport** — `subst P (e : Eq A a b) : P a → P b :≡ cast (P
  a) (P b) (cong P e)`. Because `cast` computes (§3), `J` **reduces on
  non-`refl`** (`15 §4`), via observational equality.
- **funext** — *definitional* (`Eq` at a Π-type **is** pointwise `Eq`, §2). A
  major ergonomic and verification win.
- **propext** — *definitional* (`Eq` at Ω is mutual implication, §2).
- **UIP / proof irrelevance** — *definitional* (`Eq : Ω`, §1).
- **`sym`, `trans`, `cong`** — derivable; all compute.

So Ken is a **set-level** theory: every type's equality is a proposition with
UIP. There is no higher path structure (no `Eq (Eq A a b) p q` content) — which
is exactly right for software data (ADR 0005).

## 5. Quotient types

Set-quotients are **native** (not HITs):

```
  Γ ⊢ A : Type ℓ      Γ ⊢ R : A → A → Ω
  ────────────────────────────────────────  (Quot-Form)
  Γ ⊢ A / R : Type ℓ
```

- **Introduction:** `[a] : A / R` for `a : A`.
- **Equality:** `Eq (A/R) [a] [b]` reduces to `R a b` (§2) — quotient equality
  *is* the user relation.
- **Elimination:** a function `A/R → C` is given by a function on `A` together
  with a proof it **respects `R`** (maps `R`-related inputs to `Eq`-equal
  outputs); computation: `elim … [a] ≡ f a`. (For `C : Ω` the respect proof is
  free by §1.)
- Quotients give `Int`-as-quotient, finite maps/sets up to equivalence, and the
  set-level constructions HITs would have provided. *General* quotient-inductive
  types (QITs) are a possible later extension (**blueprint:** QITs-in-OTT); the
  DRAFT provides set-quotients.

## 6. Propositional truncation `∥A∥`

`∥A∥ : Ω` is the **propositional truncation** of `A` — `A` squashed to a mere
proposition (a quotient of `A` by the total relation, landing in Ω). `|a| :
∥A∥`; to use it you map into a proposition. It provides Ω's `∨` and `∃` (`12
§5.2`): `φ ∨ ψ :≡ ∥ φ + ψ ∥`, `∃ x. φ :≡ ∥ (x : A) × φ ∥`. Because the target is
in Ω, truncation is itself proof-irrelevant.

## 7. What is deliberately absent (vs the cubical alternative)

Not in Ken (ADR 0005): the **interval** and dimension variables;
**cofibrations** and partial elements; `transp`/`hcomp`/`comp`; **`Glue`** and
**computational univalence**; **`PathP`** / heterogeneous paths; and **higher
inductive types**. These buy univalence + higher-dimensional structure, which
software does not use, at the cost of the largest and most canonicity-fragile
part of a cubical kernel. Ken trades them for a smaller, set-level,
UIP-validating core.

## 8. Definitional equations summary (what conversion must know)

The observational layer adds to definitional equality (`17`): **Ω
proof-irrelevance** (§1, any two proofs of a `P : Ω` are equal); the
**`Eq`-by-type reductions** (§2); the **`cast`-refl** rule and
**`cast`-by-type** reductions (§3); the **quotient** equality (`Eq (A/R) [a] [b]
≡ R a b`) and eliminator computation (§5); and `∥A∥` (§6). Several by-type forms
are **(blueprint)** — their existence and boundary behaviour are required; exact
normal forms are confirmed against `CICobs`/`TTobs` (never copied,
`../../CLEAN-ROOM.md`).

## 9. What the kernel checks here

A conforming kernel MUST: provide the strict proposition universe **Ω** with
**definitional proof irrelevance** (§1); compute **`Eq`-by-type** (§2) including
definitional **funext** and **propext**; provide **`cast`** with **`cast`-refl
regularity** and **`cast`-by-type** computation (§3), from which `subst`/`J`
**reduce on non-`refl`** (§4); provide **quotient types** with the relation-as-
equality and respect-checked eliminator (§5); and **propositional truncation**
(§6). The soundness-critical, separately-tested behaviours: **`cast` computes on
closed canonical type-equalities** (canonicity), **`cast A A refl a ≡ a`**,
**`J` reduces on non-`refl`**, **funext/propext/UIP hold definitionally**, and
**quotient equality reduces to the relation**. Conformance:
`../../conformance/kernel/observational/`.
