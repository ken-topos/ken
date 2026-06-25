# The cubical layer

> Status: **DRAFT v0**. Normative for the *interface and computation
> commitments*; the full equational system is large and some boundary rules are
> tagged **(oracle)** for cross-check. Defines the interval, cofibrations,
> partial elements, `PathP`, transport `transp`, composition `hcomp`/`comp`,
> `Glue` + univalence, equivalences `isEquiv`, and higher inductive types. This
> is the machinery that makes `15-identity.md`'s `J` and univalence **compute**.
> The digest confirms a computing cubical core is feasible (the prototype runs
> transport, comp/hcomp, Glue, univalence-via-Glue, HITs, isEquiv).
>
> **OQ-4** records the scope choice: full cubical (this chapter) vs. a lighter
> HoTT-with-`Id`/`J` core. The DRAFT commits to full cubical because it is what
> makes `J`-on-non-`refl` and univalence reduce; a lighter core would reopen the
> prototype's `J` gap.

## 1. The interval `𝕀`

`𝕀` is an abstract interval with two endpoints `0` and `1` and a **de Morgan
algebra** structure:

```
r, s ::= 0 | 1 | i | r ∧ s | r ∨ s | ~ r
```

with: `∧`, `∨` associative, commutative, idempotent, mutually distributive; `0`
the unit of `∨` and zero of `∧`; `1` the unit of `∧` and zero of `∨`; `~`
involutive with `~0 = 1`, `~1 = 0`, and the de Morgan laws `~(r∧s) = ~r ∨ ~s`,
`~(r∨s) = ~r ∧ ~s`. Interval equality (these laws) is decidable and is used by
conversion (`17`).

**`𝕀` is not a type** (`11 §3`). There is no `Type ℓ` containing `𝕀`; you may
bind an interval variable (`⟨i⟩ …`, the `transp`/`comp` lines, systems) but you
may not form `(i : 𝕀) → …` with `Π`, store an interval in a data structure, or
eliminate into `𝕀`. This containment is what keeps the interval purely
"dimensional" and the ordinary type theory unchanged by its presence.

## 2. Cofibrations (the face lattice)

A **cofibration** `φ` describes a subset of the cube — the faces where it holds:

```
φ, ψ ::= (r = 0) | (r = 1) | φ ∧ ψ | φ ∨ ψ | ⊤ | ⊥
```

ordered by entailment; `Γ, φ` (a context restriction, `11 §3`) assumes `φ` holds
and makes the corresponding interval equation definitional (e.g. under `(i =
0)`, `i ≡ 0`). The kernel decides cofibration entailment `φ ⊨ ψ` (a decidable
problem over the face lattice), used to check that systems agree on overlaps
(§3) and that boundaries match.

## 3. Partial elements and systems

A **partial element** of `A` defined on cofibration `φ` is a term `u` typed
under the restriction:

```
  Γ, φ ⊢ u : A          -- u is "A-valued where φ holds"
```

A **system** `{φ₁ ↦ u₁ ; … ; φₙ ↦ uₙ}` glues partial elements, with the
**agreement condition**: on any overlap `φₖ ∧ φⱼ`, `uₖ ≡ uⱼ` (checked by
conversion under the combined restriction). A system on `φ = φ₁ ∨ … ∨ φₙ` is a
single partial element on `φ`.

**Extension types.** `A[φ ↦ u]` is the type of elements `a : A` that *agree with
`u` on `φ`* (`a ≡ u` under `Γ, φ`). The cubical operations are typed to return
such extensions — this is how "boundary conditions" are tracked. `a : A[⊥ ↦ u]`
is just `a : A`.

## 4. Dependent paths — `PathP`

The primitive path is **heterogeneous**: over a *line of types* `A : 𝕀 → Type ℓ`
(written `⟨i⟩ A`), a dependent path connects an element of `A[0/i]` to one of
`A[1/i]`.

```
  Γ, i : 𝕀 ⊢ A : Type ℓ    Γ ⊢ a₀ : A[0/i]    Γ ⊢ a₁ : A[1/i]
  ─────────────────────────────────────────────────────────────  (PathP-Form)
  Γ ⊢ PathP (⟨i⟩ A) a₀ a₁ : Type ℓ
```

Introduction, application, boundary, and η are as for `Path` (`15 §2`) with the
endpoint types varying along the line. The non-dependent path is the constant
line: `Path A a b :≡ PathP (⟨_⟩ A) a b`. `PathP` is what lets a path connect
elements whose *types* are only equal up to a path — essential for `Σ`-paths
(`15 §5`) and transport.

## 5. Transport — `transp`

`transp` moves an element along a line of types.

```
  Γ, i : 𝕀 ⊢ A : Type ℓ    Γ ⊢ φ cof    A constant on φ    Γ ⊢ a : A[0/i]
  ──────────────────────────────────────────────────────────────  (transp)
  Γ ⊢ transp (⟨i⟩ A) φ a : A[1/i]      with  transp (⟨i⟩ A) φ a ≡ a  under φ
```

- The cofibration `φ` marks where the line `A` is **constant**; there `transp`
  is the identity (this is the typed generalization of `15 §3`). `φ = ⊥` is
  ordinary transport over a genuinely varying line; `φ = ⊤` requires `A`
  constant and gives the identity.
- **Regularity:** if `A` does not depend on `i` at all, `transp (⟨_⟩ A) φ a ≡
  a`.
- **Computation by type (`transp`-by-type).** On a canonical type line, `transp`
  reduces structurally (sketch; full rules **(oracle)** for exact form):
  - **Π:** `transp (⟨i⟩ (x:A)→B) φ f` transports the argument backwards and the
    result forwards — `λ x₁. transp (⟨i⟩ B[…]) φ (f (transp (⟨i⟩ A) φ … x₁))`.
  - **Σ:** componentwise, the second component over the `PathP` induced by the
    first.
  - **Path/PathP:** transports endpoints and composes.
  - **inductive `D`:** pushes into constructor arguments (a `transp` of `cons a
    as` is `cons` of transported parts); on indices it uses `comp`.
  - **`Glue` / `U`:** the rules that make univalence compute (§7–8). This
    structural recursion on the type is exactly why `J`, `subst`, and univalence
    transport *run* rather than getting stuck.

## 6. Composition — `hcomp` and `comp`

Composition fills an open box: given a base and the sides, it returns the
missing lid.

**Homogeneous (`hcomp`)** — the type does not vary:

```
  Γ ⊢ A : Type ℓ   Γ ⊢ φ cof   Γ,φ,i:𝕀 ⊢ u : A   Γ ⊢ u₀ : A[φ ↦ u[0/i]]
  ──────────────────────────────────────────────────────────────  (hcomp)
  Γ ⊢ hcomp {φ ↦ ⟨i⟩ u} u₀ : A[φ ↦ u[1/i]]
```

`u₀` is the bottom of the box, the system `{φ ↦ u}` is the sides; the result is
the top, agreeing with the sides on `φ` and reducing to `u₀` when `φ = ⊥` is the
only constraint at the base. The **boundary law**: `hcomp {φ ↦ u} u₀ ≡ u[1/i]`
under `φ`.

**Heterogeneous (`comp`)** — the type varies along the same dimension; definable
from `transp` + `hcomp`:

```
  comp (⟨i⟩ A) {φ ↦ ⟨i⟩ u} u₀
    :≡  hcomp {φ ↦ ⟨i⟩ transp (⟨j⟩ A[i∨j / i]) i (u)} (transp (⟨i⟩ A) ⊥ u₀)
    :  A[1/i] [ φ ↦ u[1/i] ]                                          (comp)
```

(The exact fill is **(oracle)**; the commitment is that `comp` exists, satisfies
its boundary `comp … ≡ u[1/i]` under `φ`, and computes by `transp`-by-type +
`hcomp`-by-type.) `comp` is the workhorse: transitivity of paths, `J`, and most
derived equalities are `comp`s, which is why they compute.

**`hcomp`-by-type.** Like `transp`, `hcomp` reduces structurally on canonical
types (a composite of pairs is a pair of composites, etc.), and on an inductive
type it commutes with constructors. The base cases (composition *at* a `Glue`,
at `U`, and at each inductive) are the bulk of the kernel's cubical code.

## 7. `Glue` types

`Glue` attaches, over a cofibration `φ`, a type `T` *equivalent* to the ambient
`A`, producing a type that is `T` on `φ` and `A` off it. It is the mechanism
behind univalence.

```
  Γ ⊢ A : Type ℓ   Γ ⊢ φ cof   Γ,φ ⊢ T : Type ℓ   Γ,φ ⊢ e : T ≃ A
  ─────────────────────────────────────────────────────────────────  (Glue-Form)
  Γ ⊢ Glue A {φ ↦ (T , e)} : Type ℓ        with  Glue A {⊤ ↦ (T,e)} ≡ T
```

- **`glue`** introduces: from `t : T` (on `φ`) and `a : A` agreeing (`e.fun t ≡
  a` on `φ`), `glue {φ ↦ t} a : Glue A {φ ↦ (T,e)}`, with `glue {⊤ ↦ t} a ≡ t`.
- **`unglue`** eliminates: `unglue g : A`, projecting back to the ambient type,
  with `unglue (glue {φ ↦ t} a) ≡ a` and `unglue g ≡ e.fun g` under `φ`.
- `transp`/`hcomp` **at a `Glue`** are the rules (oracle for exact form) that
  make transport along a `Glue`-line apply the stored equivalence — the
  computational heart of univalence.

## 8. Equivalences and univalence

**`isEquiv` / `≃`.** An equivalence is a function with **contractible fibers**
(the definition that is itself a mere proposition, so "being an equivalence" is
proof-irrelevant):

```
  isContr A :≡ (c : A) × ((x : A) → Path A c x)
  fiber f b :≡ (a : A) × Path B (f a) b
  isEquiv (f : A → B) :≡ (b : B) → isContr (fiber f b)
  A ≃ B :≡ (f : A → B) × isEquiv f         -- e.fun := the first projection
```

`isEquiv f` is a mere proposition (`12 §5`); `idEquiv : A ≃ A` is the identity.
(The half-adjoint or bi-invertible formulations are inter-derivable; the kernel
fixes one — **(oracle/OQ)** which — and the others are library lemmas.)

**Univalence (`ua`).** From an equivalence build a path between the types:

```
  ua : (A ≃ B) → Path (Type ℓ) A B
  ua e :≡ ⟨i⟩ Glue B { (i = 0) ↦ (A , e) ; (i = 1) ↦ (B , idEquiv) }
```

with the **computation rule** (the payoff):

```
  transport (λ X. X) (ua e)  ≡  e.fun        (ua-β, up to Glue transp rule)
```

So "equal types are interchangeable, and transporting along the equality *runs*
the equivalence." Full univalence (that `ua` is itself an equivalence between `A
≃ B` and `Path U A B`) is then derivable. The verification layer uses `ua` +
`funext` to make structurally-identical representations interchangeable in
proofs without manual coercion lemmas.

## 9. Higher inductive types (HITs)

A **HIT** is an inductive type with **path constructors** in addition to point
constructors:

```
data ∥_∥ (A : Type ℓ) : Type ℓ where        -- propositional truncation
  | inc  : A → ∥ A ∥
  | squash : (x y : ∥ A ∥) → Path ∥ A ∥ x y   -- a path constructor
```

- **Point constructors** behave as in `14`.
- **Path (and higher-path) constructors** add elements of `Path`/`PathP` (or
  iterated) between expressions in the type. `∥ A ∥` is `A` made a mere
  proposition; it is exactly the truncation used by `Ω`'s `∨` and `∃` (`12 §5`).
- **Eliminator.** `elim` for a HIT takes, in addition to point-methods, a method
  for each path constructor proving the motive **respects** that path (a `PathP`
  in the motive). ι-computation: on point constructors as in `14`; on path
  constructors, the eliminator's action *is* the supplied path-method
  (definitionally) — and `hcomp`/`transp` at the HIT compute via these. This
  "reducing eliminator on path constructors" is the feature the digest confirms
  the prototype achieves and Ken requires.
- **Canonical HITs Ken provides:** propositional truncation `∥A∥`, set quotients
  `A / R`, pushouts (the real coproduct-with-gluing, *not* the prototype's
  stubbed `pushout ↦ 0.0`), and the circle `S¹` (as a test of higher structure).
- **Scope (OQ-4a).** Arbitrary user HITs vs. a fixed menu of kernel-provided
  HITs. The DRAFT provides the menu above as primitives and leaves *general*
  user-defined HITs as an extension; general HITs need a schema for admissible
  path constructors and their `hcomp` rules, which enlarges the kernel.

## 10. Definitional equations summary (what conversion must know)

The cubical layer adds these to definitional equality (`17`): the de Morgan
interval laws (§1); cofibration entailment and the under-`φ` interval equations
(§2); system agreement on overlaps (§3); `PathP`/`Path` boundary + β + η (§4,
`15`); the `transp` regularity + by-type rules and the `φ`-constant identity
(§5); the `hcomp`/`comp` boundary laws and by-type rules (§6); the
`Glue`/`glue`/`unglue` boundary + computation (§7); `ua-β` (§8); and HIT
point/path ι-rules (§9). Several *by-type* rules are tagged **(oracle)** — their
existence and boundary behaviour are required; their exact normal forms are to
be confirmed against a reference (the prototype or a published cubical system),
never copied.

## 11. What the kernel checks here

A conforming kernel MUST: decide interval (de Morgan) and cofibration
entailment; check partial-element typing and system agreement; form/intro/elim
`PathP`; implement `transp` with regularity and by-type computation; implement
`hcomp`/`comp` with boundary laws and by-type computation; implement
`Glue`/`glue`/`unglue` with their boundary/computation and the `transp`/`hcomp`-
at-`Glue` rules; provide `isEquiv`/`≃`, `ua`, and `ua-β`; and provide the menu
of HITs with reducing eliminators on point **and path** constructors. The
soundness-critical, separately-tested behaviours: **`transp`/`comp` compute on
closed canonical terms** (canonicity, `README.md §5.4`), **`ua-β` reduces**, and
**HIT eliminators reduce on path constructors**. Conformance:
`../../conformance/kernel/cubical/` (`glue`, `isequiv`, `path-core`,
`hit-compute`, `ua-beta`, mirroring the prototype's green oracle suite as
behavioral targets).
