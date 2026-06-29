# The trusted kernel

> Status: **K2 elaborated**. Normative. This is the contract for WS-K
> (K1/K2/K2c/K-api) and the re-checking target for WS-V. Conformance:
> `../../conformance/kernel/`.

The kernel is Ken's **trust root**: the one component whose correctness the
soundness of every Ken program depends on. It is small, written in Rust, and
**permanent** — the elaborator, prover, and (later) native backend may
eventually be written in Ken, but the kernel stays a small Rust core (the Lean
model). This chapter fixes what the kernel *is*, what it *checks*, and what it
deliberately keeps *out*.

## 1. What the kernel does

The kernel implements a **dependent type theory with an observational equality
layer** (OTT; ADR 0005). Concretely it provides exactly these capabilities, and
no more:

1. **Type-checking** of fully-explicit **core terms** (`11-syntax.md`): given a
   context Γ, a term `t`, and a type `A`, decide whether `Γ ⊢ t : A`.
2. **Type inference** for the syntax-directed fragment: given Γ and `t`, produce
   the `A` for which `Γ ⊢ t : A`, or fail.
3. **Conversion** (`17-conversion.md`): decide definitional equality `Γ ⊢ a ≡ b
   : A`, via lazy-WHNF + NbE, with η for Π/Σ, **proof irrelevance** for Ω, and
   the observational `Eq`/`cast` equations.
4. **Normalization / evaluation** to (weak head) normal form, used by conversion
   and exposed for the interpreter and the prover's certificate checker.
5. **Admission of definitions** into a global environment, each gated by a
   **termination check** (size-change termination over δ-unfolding,
   `17-conversion.md §SCT`) and a positivity check for inductive declarations
   (`14-inductive.md`).
6. **Proof checking**: a proof is just a core term whose type is the
   proposition; checking a proof *is* type-checking (3). The prover's
   certificates (`../20-verification/23-prover.md`) are re-checked here —
   nothing the prover says is trusted until the kernel re-derives it.

The kernel's public surface is enumerated in `18-judgments.md §Kernel API`.

## 2. What the kernel does NOT do

Everything below is **untrusted infrastructure** that lives *outside* the kernel
and produces core terms or certificates for it to re-check (the **de Bruijn
criterion**, `../00-overview.md §3`):

- **Elaboration**: surface syntax → core terms, implicit-argument insertion,
  unification, metavariable solving (`../30-surface/39-elaboration.md`). The
  kernel receives only fully-explicit core terms.
- **Proof search / automation**: the fragment classifier, Z3, the Kripke
  embedding (`../20-verification/23-prover.md`). These *find* proofs; the kernel
  *checks* them. A buggy prover cannot make an ill-typed program check.
- **Error recovery, diagnostics, holes**: the kernel returns a precise yes/no
  with a minimal reason; turning that into a countermodel or a typed hole is
  V4's job.
- **Evaluation strategy for performance**: the kernel defines *the* reference
  reduction; a fast native backend is differential-tested against it but is not
  the kernel.

This separation is the whole security model: keep the trusted core small enough
to audit and (eventually) formally verify, and push all the cleverness outside
it.

## 3. K1 delivery scope

The kernel is delivered in phases. **K1** (`11`–`14`) is the set-level MLTT
core — the well-understood foundation the observational layer sits on. **K2**
(`15`, `16`) adds the OTT equality layer. **K2c** (`17`) adds the full
decidable conversion (NbE + SCT). **K-api** (`18`) publishes the stable,
audited TCB boundary. This chapter describes the complete kernel; the table
below maps what each phase delivers.

| Phase | Files | What it delivers | Blocked by |
|-------|-------|------------------|------------|
| **K1** — core calculus | `11`, `12`, `13`, `14` | Syntax + de Bruijn, predicative non-cumulative checked universes, Π/Σ with βη, inductive families with dependent eliminator + strict positivity, basic structural conversion (β/η/ι/δ) | F2, F3 (both merged) |
| **K2** — observational layer | `15`, `16` | `Eq`-by-type (definitional funext/propext), `cast` with regularity + by-type computation, derived `J` (reduces on non-`refl`), strict-prop Ω with proof irrelevance + Heyting logic, set-quotients `A/R` with relation-as-equality, propositional truncation `‖A‖` | K1 |
| **K2c** — full conversion | `17` | Lazy-WHNF NbE, `Eq`/`cast` conversion equations, SCT termination gating δ, full decidable conversion | K1, K2 |
| **K-api** — stable API | `18` | Audited `check`/`infer`/`convert`/`whnf` TCB boundary, complete typing judgment, kernel Rust API | K1, K2, K2c |

K1 reserves the K2 grammar formers (`Ω`, `Eq`, `cast`, `J`, `A/R`, `‖A‖`) in
the syntax (`11-syntax.md §1` — they parse; raw-well-formedness checks their
scoping) but implements **none** of their typing or computation. That is K2. K1
builds only the conversion its own rules require — β/η/ι/δ — structured so K2c
can extend it with NbE later. The full SCT termination argument is K2c; K1's
conversion must at least terminate on its own rules (structural decrease for ι,
acyclic δ, β/η size-bounded on K1 terms).

## 4. The core calculus at a glance

The kernel's type theory is:

- A **predicative, non-cumulative** hierarchy of universes `Type 0 : Type 1 :
  …`, **checked** — there is no `Type : Type` (`12-universes.md`). (OQ-2 decided
  — non-cumulative; ergonomics via the elaborator, see `12-universes.md §3`.)
- **Dependent functions** `(x : A) → B` (Π) with β and η (`13-pi-sigma.md`).
- **Dependent pairs** `(x : A) × B` (Σ), genuinely dependent — `B` may mention
  `x` — with projections and η (`13-pi-sigma.md`).
- **Inductive families** with dependent eliminators and a strict-positivity
  requirement (`14-inductive.md`): `Nat`, `Bool`, `List`, `Vec`, `Σ`/`W` as
  needed, and user inductives.
- **[K2]** **Observational equality** `Eq A a b` as the identity type, a
  proposition computed by recursion on `A` (`15-identity.md`,
  `16-observational.md`). `J` is *derived* from the `cast` coercion and
  **reduces on non-`refl` equalities**, via OTT not cubical (ADR 0005).
  **funext, propext, UIP** are *definitional*; Ken is **set-level**.
- **[K2]** **`cast`** (transport along a type-equality, computing by type
  structure, with `cast A A refl a ≡ a`), **native set-quotients** `A / R`, and
  **propositional truncation** `‖A‖` (`16-observational.md`). No interval,
  `Glue`, univalence, or higher inductive types (ADR 0005).
- **[K2]** The **strict proposition universe Ω** (`SProp`) — the subobject
  classifier, with **definitional proof irrelevance** and a Heyting structure —
  where `Eq` and the logic live (`12-universes.md §5`, `16-observational.md
  §1`), developed in `../20-verification/`.

The metatheoretic commitments this calculus must satisfy — and that the kernel's
tests encode — are in §6.

## 5. Chapter map

| File | Subject | Phase |
|------|---------|-------|
| `11-syntax.md` | Core grammar: terms, de Bruijn indices, telescopes, contexts, the global environment | **K1** |
| `12-universes.md` | Universe hierarchy, predicativity, checking, level polymorphism; Ω stub (K1) | **K1** (Ω fully in K2) |
| `13-pi-sigma.md` | Π and Σ: formation, intro, elim, computation, η; K1 conversion + subject reduction | **K1** |
| `14-inductive.md` | Inductive families, constructors, dependent eliminator, strict positivity, ι-reduction | **K1** |
| `15-identity.md` | Identity as observational `Eq`; `refl`; `cast`; `J` and its computation; funext/UIP | **K2** |
| `16-observational.md` | The strict-prop Ω, `Eq`-by-type, `cast`, quotient types, propositional truncation | **K2** |
| `17-conversion.md` | Full definitional equality, NbE, decidable conversion, β/η/δ/ι, regularity, SCT termination | K2c |
| `18-judgments.md` | The complete typing judgment, the checking/inference algorithm, and the kernel's Rust API | K-api |

## 6. Soundness commitments (what "the kernel is correct" means)

A conforming kernel MUST satisfy, and its test suite MUST exercise, the
following. Each is mapped to the Steward frame's 8 K1 acceptance criteria
(`docs/program/wp/K1-core-type-theory.md §2`). Criteria marked K2/K2c-gated are
designed now, verified later; the K1 fragment satisfies them within its scope.

| # | Commitment | AC | Testable |
|---|-----------|-----|----------|
| 1 | **No `Type : Type`.** Universe-level loop is **rejected**; the classic paradox is ill-typed (`12-universes.md`). | AC-1 | **K1** — `conformance/kernel/universes/` |
| 2 | **Genuinely dependent Σ.** `(n : Nat) × Vec A n` type-checks; projections and η hold (`13-pi-sigma.md`). | AC-2 | **K1** — `conformance/kernel/pi-sigma/` |
| 3 | **Π/Σ βη.** Π β and η hold; Σ projection-β and surjective-pairing η hold (`13-pi-sigma.md`). | AC-3 | **K1** — `conformance/kernel/pi-sigma/` |
| 4 | **Inductive eliminator ι + dependent eliminator.** Eliminator reduces (ι) over a constructor; dependent eliminator checks (e.g. `Vec` length-indexed elimination) (`14-inductive.md`). | AC-4 | **K1** — `conformance/kernel/inductive/` |
| 5 | **Strict positivity.** Positive inductive admitted; negative one (`data Bad = mk (Bad → Bad)`) rejected at admission (`14-inductive.md §2`). | AC-5 | **K1** — `conformance/kernel/inductive/` |
| 6 | **Subject reduction on K1 fragment.** If `Γ ⊢ t : A` and `t` reduces by β/η/ι/δ, then `Γ ⊢ t' : A` (`13-pi-sigma.md §K1 conversion`, `14-inductive.md §K1 conversion`). | AC-6 | **K1** — property test across all K1 rules |
| 7 | **Decidable checking on K1 fragment.** `check`/`infer` terminate on K1 rules (structural decrease for ι, acyclic δ, β/η size-bounded). The full SCT termination argument is K2c. | AC-7 | **K1** (within its scope) — termination test suite |
| 8 | **K1 conformance passes.** All `conformance/kernel/` K1-subset tests pass; lint/CI green. | AC-8 | **K1** — CI gate |

Beyond these K1-verifiable commitments, the full kernel (K2 + K2c + K-api)
must also satisfy:

| # | Commitment | Gated by | Notes |
|---|-----------|----------|-------|
| 9 | **Omega proof-irrelevance.** Any two `p, q : P : Omega` are definitionally equal; conversion skips propositional argument contents. | K2 | `16` par. 1.2; `observational/omega-pi-convertible` |
| 10 | **Definitional funext.** `Eq ((x:A)->B) f g` reduces to `(x:A) -> Eq B (f x) (g x)`. | K2 | `16` par. 2.2; `observational/funext-definitional` |
| 11 | **Definitional propext.** `Eq Omega P Q` reduces to `(P->Q) and (Q->P)`. | K2 | `16` par. 2.2; `observational/propext-definitional` |
| 12 | **cast regularity.** `cast A A refl a` reduces to `a` definitionally. | K2 | `16` par. 3.2; `observational/cast-refl` |
| 13 | **J reduces on non-refl.** `J` on a non-`refl` canonical equality reduces to a constructor form (not stuck). | K2 | `15` par. 4; `observational/j-nonrefl` |
| 14 | **Quotient equality.** `Eq (A/R) [a] [b]` reduces to `R a b`; `elim_/ M f r [a]` reduces to `f a`. | K2 | `16` par. 5; `observational/quotient-eq`, `observational/quotient-elim` |
| 15 | **Full subject reduction.** Across all formers including `Eq`, `cast`, `J`, `A/R`, `‖A‖`, and Ω. | K2 + K2c | Designed in `15`, `16`; the K1 subject-reduction architecture is structured for extension. |
| 16 | **Canonicity / normalization.** Every closed term of an inductive type reduces to a constructor form; `Eq`/`cast` on closed terms compute (the computational content that makes `J`-on-non-`refl` reduce). Proven for OTT (`TTobs`/`CICobs`, ADR 0005). | K2 + K2c | Requires the full NbE in `17`. |
| 17 | **Consistency.** There is no closed proof of the empty type `⊥`; the logic is not degenerate. | K2 | A documented argument; the positivity + predicativity + termination architecture is designed to support a future mechanized proof. |

Where a commitment is currently an argument rather than a mechanized proof,
`18-judgments.md §Metatheory` says so explicitly.

## 7. Design discipline

The kernel is designed from type-theoretic first principles and the strategy's
locked decisions. Three soundness properties are guaranteed *by construction*
here and MUST NOT be compromised: universes are checked, Σ is genuinely
dependent, and `J` reduces on non-`refl` equalities. Where this spec fixes a
behavioral detail that is a free implementation choice (e.g. an exact
reduction-order), it tags the point **(oracle)**: the detail is to be validated
against Ken's reference interpreter during implementation.
