# The trusted kernel

> Status: **DRAFT v0**. Normative. This is the contract for WS-K (K1/K2/K3) and
> the re-checking target for WS-V. Conformance: `../../conformance/kernel/`.

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

## 3. The core calculus at a glance

The kernel's type theory is:

- A **predicative, non-cumulative** hierarchy of universes `Type 0 : Type 1 :
  …`, **checked** — there is no `Type : Type` (`12-universes.md`). (OQ-2 decided
  — non-cumulative; ergonomics via the elaborator, see `12-universes.md §3`.)
- **Dependent functions** `(x : A) → B` (Π) with β and η (`13-pi-sigma.md`).
- **Dependent pairs** `(x : A) × B` (Σ), genuinely dependent — `B` may mention
  `x` — with projections and η (`13-pi-sigma.md`). The prototype's non-dependent
  Σ is *not* reproduced.
- **Inductive families** with dependent eliminators and a strict-positivity
  requirement (`14-inductive.md`): `Nat`, `Bool`, `List`, `Vec`, `Σ`/`W` as
  needed, and user inductives.
- **Observational equality** `Eq A a b` as the identity type, a proposition
  computed by recursion on `A` (`15-identity.md`, `16-observational.md`). `J` is
  *derived* from the `cast` coercion and **reduces on non-`refl` equalities** —
  closing the prototype's `J`-only-on-`refl` gap, via OTT not cubical (ADR
  0005). **funext, propext, UIP** are *definitional*; Ken is **set-level**.
- **`cast`** (transport along a type-equality, computing by type structure, with
  `cast A A refl a ≡ a`), **native set-quotients** `A / R`, and **propositional
  truncation** `‖A‖` (`16-observational.md`). No interval, `Glue`, univalence,
  or higher inductive types (ADR 0005).
- The **strict proposition universe Ω** (`SProp`) — the subobject classifier,
  with **definitional proof irrelevance** and a Heyting structure — where `Eq`
  and the logic live (`12-universes.md §5`, `16-observational.md §1`), developed
  in `../20-verification/`.

The metatheoretic commitments this calculus must satisfy — and that the kernel's
tests encode — are in §5.

## 4. Chapter map

| File | Subject |
|---|---|
| `11-syntax.md` | Core grammar: terms, de Bruijn indices, telescopes, contexts, the global environment |
| `12-universes.md` | Universe hierarchy, predicativity, checking, the strict-prop Ω, cumulativity |
| `13-pi-sigma.md` | Π and Σ: formation, intro, elim, computation, η |
| `14-inductive.md` | Inductive families, constructors, the dependent eliminator, strict positivity, reduction |
| `15-identity.md` | Identity as observational `Eq`; `refl`; `cast`; `J` and its computation; funext/UIP |
| `16-observational.md` | The strict-prop Ω, `Eq`-by-type, `cast`, quotient types, propositional truncation |
| `17-conversion.md` | Definitional equality, NbE, decidable conversion, β/η/δ/ι, regularity, SCT termination |
| `18-judgments.md` | The complete typing judgment, the checking/inference algorithm, and the kernel's Rust API |

## 5. Soundness commitments (what "the kernel is correct" means)

A conforming kernel MUST satisfy, and its test suite MUST exercise:

1. **No `Type : Type`.** Universe levels are checked; the classic paradox is
   ill-typed (`12-universes.md`). *(G5, G1.)*
2. **Subject reduction.** If `Γ ⊢ t : A` and `t` reduces to `t'`, then `Γ ⊢ t' :
   A`. Reduction preserves typing.
3. **Decidable type-checking.** `Γ ⊢ t : A` is decidable; conversion terminates
   (guaranteed by SCT-gated δ and a terminating NbE, `17-conversion.md`).
4. **Canonicity / normalization.** Every closed term of an inductive type
   reduces to a constructor form; `Eq`/`cast` on closed terms compute (the
   *computational* content that makes `J`-on-non-`refl` reduce). Proven for OTT
   (`TTobs`/`CICobs`, ADR 0005).
5. **Consistency.** There is no closed proof of the empty type `⊥`; the logic is
   not degenerate. *(A documented argument, not a kernel runtime check.)*

These are the criteria by which the kernel's design (and any later formal
verification of it) is judged. Where a commitment is currently an argument
rather than a mechanized proof, `18-judgments.md §Metatheory` says so
explicitly.

## 6. Relationship to the prototype (clean-room)

The kernel is designed from type-theoretic first principles and the strategy's
locked decisions, **not** transcribed from the prototype. Three of the
prototype's documented soundness gaps are corrected *by construction* here and
MUST NOT be reintroduced: unchecked universes, non-dependent Σ, and `J` that
only reduces on `refl`. Where this spec needs a behavioral detail it cannot
derive (e.g. an exact reduction-order choice the prototype made), it tags the
point **(oracle)** for the Spec enclave to confirm against the prototype — never
by copying source.
