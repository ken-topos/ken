# Definitional equality, conversion, and termination

> Status: **DRAFT v0**. Normative. Defines what the kernel treats as *the same*
> — the reductions (β/ι/δ/cubical), the type-directed η, the conversion
> algorithm (NbE), and the **size-change termination (SCT)** gate that keeps
> δ-unfolding — and therefore type-checking — decidable. The contract for K2.

Two terms that are **definitionally equal** are interchangeable everywhere with
no proof obligation; equalities that are *not* definitional are propositions to
prove via `Path` (`15`). Getting this boundary right is most of what a dependent
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
| **Path-β** | `(⟨i⟩ t) @ r → t[r/i]`, `p@0→a`, `p@1→b` | `15 §2` |
| **prim** | `op lit̄ → lit` (registered primitive reduction) | `14 §5` |
| **cubical** | `transp`/`hcomp`/`comp`/`unglue`/`ua`-β/HIT rules | `16` |
| **interval** | de Morgan laws; under `φ`, the face equations | `16 §1–2` |

- δ (constant unfolding) is **controlled**: the conversion algorithm unfolds a
  definition only when needed to make progress (§3), never eagerly. Opaque
  constants (`11 §4`) never δ-reduce.
- **prim** reductions are the trusted boundary (`14 §5`): `add 2 3 → 5` lets
  proofs compute over literals.
- A term with no applicable reduction at its head is **neutral** (a variable, an
  opaque constant, a primitive on non-literals, an `elim`/`@`/`transp`/`hcomp`
  on a neutral target). Conversion compares neutrals structurally (§3).

**Confluence.** The reduction system is confluent (Church–Rosser); normal forms
are unique up to α (de Bruijn identity) and the interval/cofibration laws. This
is a metatheoretic commitment (`18 §Metatheory`), tested behaviorally by the
conformance corpus.

## 2. η (type-directed)

η-equalities are **not** plain reductions (they depend on the *type*) and are
applied by the conversion algorithm at the relevant type:

- **Π-η:** at a Π-type, `f ≡ g` iff `f x ≡ g x` for a fresh `x` (`13 §1`). The
  algorithm η-expands both sides under a fresh binder.
- **Σ-η:** at a Σ-type, `p ≡ q` iff `p.1 ≡ q.1` and `p.2 ≡ q.2` (`13 §2`).
- **Path-η:** at a `Path`/`PathP` type, `p ≡ q` iff `p @ i ≡ q @ i` under a
  fresh `i` (`15 §2`).
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
that is **extended to compute the cubical operations**
(`transp`/`hcomp`/`comp`/`Glue`/ `ua`/HITs), the part Lean's (non-cubical)
kernel does not have. The reference read-back is:

1. **Evaluate** each side into a semantic domain of **values** — weak-head
   normal forms with closures for binders and **neutrals** for stuck
   computations. Evaluation performs β/Σ-β/ι/δ/prim/cubical reductions lazily to
   weak-head normal form; it does *not* go under binders.
2. **Compare** values type-directed, head-first:
   - At a **Π/Σ/Path type**, apply the η rule (§2): descend under a fresh
     variable / project / apply at a fresh dimension, and recurse at the
     component type.
   - At a **neutral vs neutral**, compare heads; if equal, compare spines
     argument-by-argument at the head's type; unfold δ only if heads differ and
     at least one is a transparent constant (then retry).
   - At **canonical vs canonical** (same type former), compare components.
   - **Universes/levels** compare by decidable level equality (`12 §1`).
   - **Interval/cofibration** subterms compare by the de Morgan / face laws (`16
     §1–2`); cubical neutrals (`transp`/`hcomp` on a neutral) compare their
     type-lines, systems, and bases.
3. **Read-back (quote)** to a normal form is used where a syntactic normal form
   is needed (e.g. to store an elaborated term, or for the conformance corpus's
   normal-form checks); η-long, δ-short normal forms are the reference output.

The algorithm is **sound and complete** for definitional equality and
**terminates** (§4). The **recommended implementation** is the **Lean-style
lazy-WHNF + on-the-fly conversion** above (avoid full normalization; compare
incrementally; unfold δ lazily); NbE read-back is the declarative reference and
is used where a syntactic normal form is genuinely needed. The observable
equality MUST be identical whichever way it is computed.

**Where Ken deliberately does *not* follow Lean** (its *theory*, not its engine,
fixed by other Ken decisions): `J` reduces on **non-`refl`** paths via the
cubical rules (`15`, opposite of Lean's `Eq.rec`); **canonicity is kept** — Ken
bakes in **no** canonicity-breaking classical axioms
(`propext`/`Quot.sound`/choice), since computational univalence comes from
cubical and the reflective prover (`../20-verification/23 §3`) relies on closed
terms computing. Lean's **definitional proof irrelevance** depends on a
primitive impredicative `Prop`, which Ken has **not** adopted (`OQ-Prop` open,
derived Ω); if that proof irrelevance is later wanted, it is an argument to
revisit `OQ-Prop`.

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
(SCT)** check at definition-admission time (`11 §4`). This is the mechanism the
digest confirms the prototype already uses to close its "δ-debt."

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

**(oracle)** The exact size order (what counts as `↓` for cubical/coinductive or
primitive values, and the treatment of `transp`/`hcomp` under recursion) is to
be confirmed against the prototype's working SCT; the *principle* and the accept
condition above are the commitment.

## 5. Decidability (the payoff)

Together: the core reductions are **strongly normalizing** (β/ι/η/Path/cubical
on well-typed terms), and δ-unfolding is **SCT-bounded**, so conversion
terminates on well-typed inputs and **type-checking is decidable** (soundness
commitment `README.md §5.3`; metatheory status in `18`). Decidability is what
lets the kernel be a *checker* (always halts with yes/no) rather than a
semi-decision procedure — the precondition for the whole verification loop.

## 6. What the kernel checks here

A conforming kernel MUST: implement all §1 reductions and §2 η rules as
**type-directed conversion**; decide level and interval/cofibration equality;
compare neutrals structurally with controlled δ; run the **SCT check** at
admission and refuse transparent admission of uncertified recursion; and
terminate on every well-typed input. Conformance:
`../../conformance/kernel/conversion/` — β/ι/δ/η equalities, Π/Σ/Path η,
primitive-literal computation, a δ-heavy convertibility that must terminate, an
SCT-accept (lexicographic/mutual) and an SCT-reject (a non-terminating
definition) case, and cubical conversions (`transp`/`hcomp` boundary
equalities).
