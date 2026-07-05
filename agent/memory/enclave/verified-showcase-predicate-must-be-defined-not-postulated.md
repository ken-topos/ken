---
scope: enclave
audience: (see scope README)
source: private memory
  `verified-showcase-predicate-must-be-defined-not-postulated`
---

# A verified-X showcase whose predicate is postulated is vacuous

A flagship "verified `X`" example (Ken's verified `sort`, `37 §6`:
`{ys | isSorted ys ∧ Perm ys xs}`) is only a real proof if the **predicates in
its refinement are definitions the prover can UNFOLD** — not `postulate`s. This
is a distinct check from the obligation's *shape*, and I missed it once.

**The miss (L3b, 2026-07-01).** My Fidelity gate confirmed the `sort` VC **emits
both conjuncts** (`isSorted ∧ Perm`, `Perm` load-bearing against the `const Nil`
degeneracy) — the emission-completeness face. But `isSorted`/`Perm` are
`declare_postulate` (opaque) in `prelude.rs`. As postulates the emitted
obligation `isSorted (sort xs)` is over an **undefined predicate**: the prover
can't unfold it, so the obligation is **undischargeable**, or the sort's proof
**assumes** it (circular — proving the conclusion by postulating it). Either way
the flagship "verified sort" **does not prove sortedness in any meaningful
sense**. CV's surface-minimality generation check surfaced it; the Architect
confirmed it is the **dual** of his L3b emission gate — **both** faces are
required for the guarantee to mean anything.

**The two independent faces of a "verified X" claim (check BOTH):**
1. **Emission-completeness** — the VC carries every needed conjunct, per-branch
   / with the IH (the obligation must descend into structure gate).
2. **Predicate-definedness** — every predicate the refinement/obligation NAMES
   is a real **definition** (recursive/inductive, unfoldable), never a
   `declare_postulate`. A postulated predicate is an **unchecked assumption in
   `trusted_base()`** the obligation silently rests on (untrusted layer backstop
   hole for omissions — the kernel re-checks the cert but the *predicate* is
   assumed).

**How to apply.** Reviewing (or authoring) any refinement / `ensures` / spec
predicate: **grep the predicate's declaration** — `declare_postulate` (opaque) ⇒
the claim is vacuous until it's demoted to `declare_def`; `declare_def`
(re-checked, unfoldable) ⇒ real. A verified-showcase spec must state that its
predicates are **defined**, and the def must land in the right sort — an
Ω-valued prop (proof-irrelevant), never a `Type`-sorted "prop" (a relevance leak
puts content in the refinement carrier — the `sort_sigma`/Σ-sort Ω check, sigma
sort pi vs sigma over equating). Prefer an inductive-relation `Perm`
(`refl|swap|trans|cons`, no `DecEq` dependency) over count-equality. Surface
analog of TB-Sound: a derivable/definable predicate left as a postulate is a
phantom `trusted_base()` entry.
