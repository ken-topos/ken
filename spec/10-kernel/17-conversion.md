# Definitional equality, conversion, and termination

> Status: **DRAFT v0**. Normative. Defines what the kernel treats as *the same*
> — the reductions (β/ι/δ/obs), the type-directed η + proof irrelevance, the
> conversion algorithm (NbE), and the **size-change termination (SCT)** gate
> that keeps δ-unfolding — and therefore type-checking — decidable. The contract
> for K2.

Two terms that are **definitionally equal** are interchangeable everywhere with
no proof obligation; equalities that are *not* definitional are propositions to
prove via `Eq` (`15`). Getting this boundary right is most of what a dependent
kernel *is*.

## 1. The reductions

Definitional equality `Γ ⊢ a ≡ b : A` is the least typed congruence closed under
the following reductions and the η rules (§2):

| Rule | Redex → reduct | From |
|---|---|---|
| **β** | `(λ (x:A). t) u → t[u/x]` | `13 §1` |
| **Σ-β** | `(a,b).1 → a`, `(a,b).2 → b` | `13 §2` |
| **ι** | `elim_D M m̄ … (cₖ ā) → mₖ …` (structural) | `14 §3` |
| **δ** | `c → t` for `(c : A := t) ∈ Σ` (transparent) | `11 §4` |
| **prim** | `op lit̄ → lit` (registered primitive reduction) | `14 §5` |
| **obs** | `Eq`-by-type; `cast A A refl a → a` + `cast`-by-type; quotient elim | `16` |

- δ (constant unfolding) is **controlled**: the conversion algorithm unfolds a
  definition only when needed to make progress (§3), never eagerly. Opaque
  constants (`11 §4`) never δ-reduce.
- **prim** reductions are the trusted boundary (`14 §5`): `add 2 3 → 5` lets
  proofs compute over literals.
- A term with no applicable reduction at its head is **neutral** (a variable, an
  opaque constant, a primitive on non-literals, an `elim`/`cast`/quotient-elim
  on a neutral target). Conversion compares neutrals structurally (§3).

**Confluence.** The reduction system is confluent (Church–Rosser); normal forms
are unique up to α (de Bruijn identity) and Ω proof-irrelevance. This is a
metatheoretic commitment (`18 §6`), tested behaviorally by the conformance
corpus.

## 2. η (type-directed)

η-equalities are **not** plain reductions (they depend on the *type*) and are
applied by the conversion algorithm at the relevant type:

- **Π-η:** at a Π-type, `f ≡ g` iff `f x ≡ g x` for a fresh `x` (`13 §1`). The
  algorithm η-expands both sides under a fresh binder.
- **Σ-η:** at a Σ-type, `p ≡ q` iff `p.1 ≡ q.1` and `p.2 ≡ q.2` (`13 §2`).
- **Proof irrelevance (Ω):** at a type `P : Ω`, **any** two terms are equal — `p
  ≡ q` with no comparison of contents (`16 §1`). This is definitional and is
  what makes equality (`Eq : Ω`) and the whole logic proof-irrelevant; the
  checker **skips propositional arguments** entirely.
- **Unit-η** (and record-η for single-constructor types with η, `14 §4`): any
  two elements of `Unit` are equal; the η for records follows from Σ-η.

Type-directed η is why conversion needs the type, not just the two terms — the
algorithm is `conv Γ A a b`, not `conv a b`.

## 3. The conversion algorithm

**`OQ-eval-strategy` — DECIDED (operator, 2026-06-27): follow Lean's kernel.**
The *operational* algorithm is **lazy weak-head normalization with on-the-fly
structural conversion and lazy δ-unfolding** — Lean 4's battle-tested,
heavily-scrutinised approach (consistent with Ken already adopting Lean's
small-trusted-kernel model, ADR 0001): reduce only enough to expose a head,
compare heads incrementally, and unfold a transparent definition (δ) **only when
forced** (heads differ and at least one is transparent), preferring *not* to
unfold. **Normalization by evaluation (NbE)** is the **declarative reference** —
the meaning of "equal" — realised over a value domain of closures + neutrals
that is **extended to compute the observational operations** (`Eq`-by-type and
`cast`, `16`) and **definitional proof irrelevance** for Ω; OTT is *closer* to
Lean's (non-cubical) setting than cubical was. The reference read-back is:

1. **Evaluate** each side into a semantic domain of **values** — weak-head
   normal forms with closures for binders and **neutrals** for stuck
   computations. Evaluation performs β/Σ-β/ι/δ/prim/obs reductions lazily to
   weak-head normal form; it does *not* go under binders.
2. **Compare** values type-directed, head-first:
   - At a **Π/Σ type**, apply the η rule (§2): descend under a fresh variable /
     project, and recurse at the component type.
   - At a type **`P : Ω`**, apply **proof irrelevance** (§2): equal immediately,
     without comparing contents.
   - At a **neutral vs neutral**, compare heads; if equal, compare spines
     argument-by-argument at the head's type; unfold δ only if heads differ and
     at least one is a transparent constant (then retry).
   - At **canonical vs canonical** (same type former), compare components.
   - **Universes/levels** compare by decidable level equality (`12 §1`).
   - **`cast`/`Eq`/quotient neutrals** compare structurally (their type
     arguments, endpoints, and bases, `16`).
3. **Read-back (quote)** to a normal form is used where a syntactic normal form
   is needed (e.g. to store an elaborated term, or for the conformance corpus's
   normal-form checks); η-long, δ-short normal forms are the reference output.

The algorithm is **sound and complete** for definitional equality and
**terminates** (§4). The **recommended implementation** is the **Lean-style
lazy-WHNF + on-the-fly conversion** above (avoid full normalization; compare
incrementally; unfold δ lazily); NbE read-back is the declarative reference and
is used where a syntactic normal form is genuinely needed. The observable
equality MUST be identical whichever way it is computed.

**Where Ken's *theory* differs from Lean's** (its engine is shared; ADR 0005):
`J`/`subst` reduce on **non-`refl`** equalities via the observational `cast`
rules (`15`, `16 §3`), where Lean's `Eq.rec` is stuck off `refl`. **Canonicity
is kept** — and Ken assumes **none** of Lean's axioms: in the observational
foundation **funext and propext are *definitional*** and **quotient soundness is
definitional** (quotient equality *is* the relation, `16 §5`), so Ken needs no
axiom where Lean postulates `propext`/`Quot.sound`, and assumes no `choice`; the
reflective prover (`../20-verification/23 §3`) relies on closed terms computing.
**Definitional proof irrelevance** — which Lean gets from its impredicative
`Prop` — Ken **also has**, from the *predicative* strict-prop universe Ω (`16
§1`, `OQ-Prop`), without impredicativity.

**Fast paths (non-normative, for performance).** Because the runtime is
content-addressed (`../40-runtime/41-values.md`), two closed terms with the same
content hash are equal — an O(1) shortcut conversion MAY take before structural
comparison. Memoizing whnf and sharing via the heap make repeated conversions
cheap. These are optimizations; they must never report unequal terms equal or
vice versa.

## 4. Termination of conversion — the SCT gate

Type-checking calls conversion; conversion unfolds δ; an unrestricted recursive
definition would make δ-unfolding (and hence conversion, and hence
type-checking) **loop**. Ken keeps decidability with a **size-change termination
(SCT)** check at definition-admission time (`11 §4`).

**What SCT checks.** When admitting a (possibly recursive, possibly mutually
recursive) transparent definition, the kernel:

1. Extracts the **call graph**: nodes are the definitions, edges are calls, each
   edge annotated with a **size-change matrix** recording, for each pair (caller
   parameter, callee argument), whether the argument is **strictly smaller**
   (`↓`), **not larger** (`↓=`), or **unrelated** (`?`) than the parameter, in
   the structural (subterm) order on canonical/inductive values.
2. Forms the **idempotent closure** of the call graph under composition of
   size-change matrices.
3. **Accepts** iff every idempotent loop (every way a call can return to itself)
   has at least one parameter that **strictly decreases** (`↓` on the diagonal)
   — i.e. every infinite call sequence would have an infinitely-decreasing
   thread, which is impossible over a well-founded order. This is the
   size-change principle.

**Consequences.**

- A definition that **passes** SCT is admitted **transparent** (δ-unfoldable);
  unfolding it during conversion is guaranteed to terminate.
- A definition that **fails** SCT is **rejected** as a transparent definition.
  (The elaborator MAY offer to admit it **opaque** — usable as a postulate-style
  constant that never δ-reduces — or report a totality error; policy is
  surface-level, `../30-surface/`. The *kernel* never admits a transparent
  definition it cannot certify terminating.)
- SCT is strictly more permissive than "structural recursion on one fixed
  argument": it handles permuted/lexicographic descent and mutual recursion,
  which is why it is the chosen criterion.
- **Scope.** SCT gates **general recursive definitions** made via δ. Recursion
  via an inductive **eliminator** (`14 §3`) is *already* structural and total
  and needs no SCT. Most surface functions elaborate to eliminators; SCT covers
  the rest.

**(oracle)** The exact size order (what counts as `↓` for primitive values, and
the treatment of `cast` under recursion) is to be validated against Ken's
reference interpreter during implementation; the *principle* and the accept
condition above are the commitment. The resolution path: primitives are treated
as neutral (no strict-decrease ordering) and `cast` under recursion is
conservatively flagged `?` pending that validation. (Coinductive values do not
arise: coinduction is excluded per `OQ-coinduction`.)

## 5. Decidability (the payoff)

Together: the core reductions are **strongly normalizing** (β/ι/η/obs on
well-typed terms), and δ-unfolding is **SCT-bounded**, so conversion terminates
on well-typed inputs and **type-checking is decidable** (soundness commitment
`README.md §5 item 3`; metatheory status in `18`). Decidability is what lets the
kernel be a *checker* (always halts with yes/no) rather than a semi-decision
procedure — the precondition for the whole verification loop.

## 6. What the kernel checks here

A conforming kernel MUST: implement all §1 reductions and the §2 η + proof-
irrelevance rules as **type-directed conversion**; decide level equality and Ω
proof-irrelevance; compare neutrals structurally with controlled δ; run the
**SCT check** at admission and refuse transparent admission of uncertified
recursion; and terminate on every well-typed input. Conformance:
`../../conformance/kernel/conversion/` — β/ι/δ/η equalities, Π/Σ η + Ω proof
irrelevance, primitive-literal computation, a δ-heavy convertibility that must
terminate, an SCT-accept (lexicographic/mutual) and an SCT-reject (a
non-terminating definition) case, and observational conversions (`cast`-refl,
`Eq`-by-type, quotient equality).
