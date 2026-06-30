# Temporal obligations as data

> Status: **DRAFT v0**. Normative for the **discipline** (temporal logic is
> deeply-embedded *data*, not kernel structure) and the about-vs-with boundary;
> the concrete `Temporal` constructors and surface notation are tuned with the
> sibling (`Ward`) and `72`'s encoding pass. **`OQ-temporal` DECIDED**
> (operator, 2026-06-27): **data-only — no temporal modalities in the kernel.**
> ADR 0006 (the seam); a durable application of Ken's reflect-don't-extend
> principle, not a v1 hedge.

## 1. The decision: state temporal properties, do not modalize the kernel

Ken **states** a temporal/behavioral property — "eventually settled", "never two
leaders", "every request is eventually answered" — as an ordinary
**deeply-embedded inductive value**: a `Temporal` datatype (LTL / μ-calculus,
§3) with surface notation (§4) elaborating to it. The value is then **exported**
(`71`) and discharged by `Ward` (model-checking + monitoring), **not** in Ken.

Ken does **not** gain a guarded/`▷`-modal layer in its trusted core — no "later"
modality, tick variables, Löb rule, or clock structure. That would add new
judgmental structure and new metatheory (guarded recursion's normalization,
productivity) to the kernel — precisely the kind of TCB growth and adversarial
surface that ADR 0005 rejected cubical to avoid. Temporal modalities are the
*other fragment* (`README §1`).

This is **durable, not a v1 expedient.** It is:

- **The consistent application of Ken's deepest principle** — *deep-embed and
  reason reflectively rather than grow the trusted core*: OTT's `Eq`-as-data
  over cubical primitives (ADR 0005); reflective `decide`/certificate-checking
  over trusting Z3 (`OQ-12`); `Temporal`-as-data here. Same move, three times.
- **The semantic decomposition that justifies the seam** — behaviors-over-time
  form a topos (Temporal Type Theory); Ken occupies the **static,
  propositional** fragment and `Ward` the **temporal/modal** fragment of *the
  same* logic (`README §1`). A temporal modality in Ken's kernel is Ken reaching
  into `Ward`'s fragment, collapsing the decomposition that keeps Ken small.
- **The right division of labor** — liveness/fairness/eventual-consistency over
  infinite behaviors and interleavings is automata-theoretic / model-checking
  territory (Apalache/TLA decide what is brutal-to-impossible in interactive
  proof). Internal temporal proof would re-implement, worse, what `Ward` does
  well.

## 2. The boundary: reason *about* formulas, not *with* modalities

Data-only does **not** make Ken helpless about temporal logic. Because
`Temporal` is ordinary inductive data, Ken's existing static logic can prove
properties **about** temporal formulas. The line is precise:

> Ken reasons **about** temporal formulas (they are data); Ken does **not**
> reason **with** temporal modalities (no `▷` in the judgment).

| Ken **may** (static propositions over the `Temporal` datatype) | Ken **may not** (needs the temporal model — `Ward`) |
|---|---|
| Prove a formula transformation preserves semantics (e.g. an LTL→NNF rewrite) | Discharge `□(req → ◇resp)` against a system's behaviors |
| Prove a normalization/`simplify` is sound and terminating | Decide liveness/fairness, or find a fair-interleaving counterexample |
| Prove an exported obligation is well-formed / closed | Establish refinement (does the implementation refine the model) |
| Derive one obligation from others by sound `Temporal`-level lemmas | Anything quantifying over infinite traces |

So a Ken library can carry verified *operations* on temporal formulas (the
metatheory of the embedding), while the obligations those formulas denote are
delegated. The proofs Ken does here are unremarkable static proofs about an
inductive type — no new kernel power.

## 3. The `Temporal` datatype (shape; encoding deferred)

`Temporal` is a deeply-embedded logic over the **effect/event alphabet** `Σ`
(the interaction-tree perform-node signatures, `71 §2`) — so its atoms are
exactly the events `Ward` monitors. The DRAFT shape (an LTL core with a
μ-calculus extension):

```
Temporal Σ :=
  | atom   (p : Pred Σ)            -- a state/event predicate over the alphabet
  | not    (φ : Temporal Σ)
  | and    (φ ψ : Temporal Σ) | or (φ ψ : Temporal Σ)
  | next   (φ : Temporal Σ)        -- ◯ / X
  | until  (φ ψ : Temporal Σ)      -- φ U ψ   (◇, □ derived)
  -- μ-calculus extension (for properties beyond LTL):
  | mu  (X : Var) (φ : Temporal Σ) -- least fixpoint   (guarded in X)
  | nu  (X : Var) (φ : Temporal Σ) -- greatest fixpoint
  | var (X : Var)
```

`◇φ := until true φ`, `□φ := not (until true (not φ))`. The **exact constructor
set, the `Pred Σ` atom language, the fixpoint variable discipline, and the
ITF-facing serialization are an encoding pass** (with `Ward`), not fixed here.
Whatever the final set: it is **inert data** to the kernel — `Temporal` is a
normal inductive type (`../10-kernel/14`), elimination is ordinary `elim`, and
nothing about it touches conversion or the judgmental structure.

**Two sibling `compile` projections (disambiguation).** `Temporal Σ` is the
shared source of **two distinct functions**, both conventionally written
`compile` — *not* one function with two directions:

- `compile : Temporal Σ → WardFormula` — the **property translation** for static
  `Ward` model-checking, proved semantics-preserving once (`71 §5`, B1/B2).
- `compile : Temporal Σ → Monitor` — the **runtime monitor synthesis** (LTL →
  Büchi, `73 §2.4`, B3).

They share only the `Temporal` source and the alphabet `Σ`; their codomains and
purposes differ. A faithfulness claim names **which** projection it is over.

## 4. Surface notation (sketch)

A `temporal { … }` block (or a `delegated`/`assume` clause carrying a temporal
formula, `../20-verification/21 §5`) provides readable notation — `always`,
`eventually`, `until`, `next`, `leadsto` (`p ~> q := always (p → eventually q)`)
— elaborating to the `Temporal` constructors (§3). Such a claim is tagged
**`delegated`** in the four-way epistemic status, appears in source
(human-visible), and flows into the export (`71`). The concrete keywords/layout
settle with the surface-syntax pass (`OQ-syntax`).

## 5. The revisit-trigger (and the principled response)

The one thing data-only gives up: **unbounded** liveness — "no deadlock for
**all** `N`", where model-checking covers only `N ≤ k`. If that becomes
load-bearing, the response is **not** kernel modalities. It is a **contained
reflective model**: define the temporal semantics in the deep embedding and
prove the property *in that model* with Ken's existing logic (the same
reflection-over-extension move as `OQ-12`), weighed explicitly against TCB cost.
The exception is handled by the existing principle, not by abandoning it
(`../90-open-decisions.md`, `OQ-temporal`).

## 6. What this area must deliver

The `Temporal` datatype (§3) as an ordinary inductive; the surface notation (§4)
+ its elaboration; the `delegated` tagging and export wiring (`71`); and a
stdlib home for *verified* `Temporal` operations (the about-the-formulas
metatheory, §2). Acceptance: a stated temporal property elaborates to `Temporal`
data, carries no kernel power (the kernel treats it as inert), is tagged
`delegated`, and appears in the export. Conformance:
`../../conformance/behavioral/temporal/`.
