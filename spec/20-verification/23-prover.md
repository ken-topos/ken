# The automated prover

> Status: **DRAFT v0**. Normative for the architecture, the soundness
> discipline, and the classifier boundaries; the embedding's exact encoding is
> specified to the level the Verify team needs and tagged where it must be
> proved. Contract for WS-V **V3**. The analysis's strongest seam (digest §8),
> adopted wholesale; the prototype already has narrow Z3/Coq backends to
> **generalize**, not rebuild.

## 1. Contract

**Input:** an obligation `Γ ⊢ φ` (`22`), `φ : Ω`. **Output:** either

- a **certificate** — a term `p` with `Γ ⊢ p : φ` that the **kernel re-checks**
  (`../10-kernel/18 §4`); the prover is believed only after the kernel agrees;
  or
- a **failure** carrying a structured diagnostic (`24`).

**The cardinal rule (de Bruijn criterion).** The prover is **untrusted**. No
backend's "yes" is accepted on its own authority — a kernel-checkable
certificate is always produced and re-checked. A bug in the classifier, the
embedding, Z3, cvc5, or a tactic can only cause a *failure to prove* or a
*rejected certificate*, **never a false `proved`.** This is what licenses using
a classical solver under an intuitionistic logic.

## 2. The fragment classifier

Each obligation is routed by a syntactic analysis of `φ` (and the atom theories
it mentions) to the cheapest **sound** method. Three fragments:

| Fragment | What's in it | Method |
|---|---|---|
| **D — decidable** | atoms where `φ ∨ ¬φ` holds: equality/disequality of scalars & handles, `Int`/`Decimal` arithmetic comparisons, Boolean combinations, finite/bounded membership and quantifiers | **direct** decision (reflective) + Z3 to *search* |
| **FO — first-order intuitionistic** | first-order formulas over decidable atoms with intuitionistic connectives/quantifiers that are *not* themselves decidable | **Kripke embedding** → Z3 (§4) |
| **HO — higher-order / inductive** | quantification over types or predicates, goals needing induction, anything outside FO | **native intuitionistic** (prop skeleton) + **tactics / manual**; typed hole if open |

- For **D**, classical and intuitionistic logic **coincide** (excluded middle is
  available *because the atom is decidable*), so the classical solver is sound
  with no embedding (the analysis's key boundary).
- The classifier is **conservative**: when unsure whether an atom is decidable
  or a formula is FO, it routes *upward* (to the more general, more expensive
  method). Misclassification downward would risk unsoundness *if the certificate
  weren't re-checked* — and it is, so the worst case of even a buggy classifier
  is wasted work or a failed proof, never unsoundness.

## 3. Fragment D — decidable atoms

Two cooperating mechanisms, both yielding kernel certificates:

1. **Reflective decision (preferred).** For atoms with a kernel-verified
   **decision procedure** `dec : (x : A) → Decidable (φ x)` (`12 §5.2`), the
   certificate is *by computation*: the kernel evaluates `dec a` (canonicity/`16
   §11` guarantees it reduces) to `inl proof` or `inr refutation`. Because the
   kernel **computes**, "decide it" produces a real proof term with no external
   solver in the trusted path — Ken's computing core is a verification asset
   here. Used for concrete/closed decidable goals.
2. **SMT-assisted search + reconstruction.** For decidable goals with free
   variables (e.g. linear arithmetic over `Int` with universally-quantified
   parameters), Z3 *searches*; on success the result is turned into a kernel
   certificate by reflection (instantiating a verified arithmetic decision
   procedure) or by reconstructing the proof (SMTCoq-style) and re-checking. The
   solver finds the witness/cut; the kernel re-derives validity.

## 4. Fragment FO — the Kripke embedding (the headline)

A genuinely intuitionistic first-order obligation **cannot** be sent to Z3
directly (Z3 would use `¬¬`-elimination / excluded middle and accept topos-false
goals). Ken instead sends Z3 the obligation's **Kripke truth condition**.

**The translation `φ ↦ φ#`.** Introduce a sort `World` with a preorder `≤`
(accessibility = information growth) and a monotone forcing predicate `⊩`. Each
n-ary Ken predicate `P` becomes an `(n+1)`-ary classical predicate `P#(w, …)`
monotone in `w`. The connectives translate by the Kripke clauses:

```
  (P t̄)#      :=  w ⊩ P# t̄
  (φ ∧ ψ)#    :=  φ# ∧ ψ#
  (φ ∨ ψ)#    :=  φ# ∨ ψ#
  (φ ⇒ ψ)#    :=  ∀ w' ≥ w.  φ#[w'] ⇒ ψ#[w']        -- the intuitionistic clause
  (¬ φ)#      :=  ∀ w' ≥ w.  ¬ φ#[w']
  (∀x. φ)#    :=  ∀ w' ≥ w. ∀x ∈ dom(w'). φ#[w']
  (∃x. φ)#    :=  ∃x ∈ dom(w).  φ#
```

(Exact domain/monotonicity axioms **(oracle / standard)** — the Kripke-sheaf
semantics of the topos.) The **soundness theorem** the embedding rests on:

> **`φ` is intuitionistically valid in the topos iff `φ#` (a classical FO
> theory) is classically valid.**

Z3 then decides `φ#` with full classical power, *soundly*, because the
intuitionistic content lives in the translation, not in an assumption. The
reason this is principled and not a trick: **Ken's topos semantics *are* Kripke
semantics** (`README.md §4`), so `φ#` is `φ`'s *native meaning*.

**Producing a kernel certificate (the trust step).** Even here, "Z3 says `φ#` is
valid" is **not** accepted by itself. Two routes, both ending at a
kernel-checked term:

- **(a) Reflective: proved adequacy + verified certificate checker (the target,
  `OQ-12` DECIDED).** Mechanize, *once and in the kernel*, two theorems: the
  **embedding-adequacy** lemma `classically_valid(φ#) → φ` (the §4 soundness
  theorem, internalized — Ken's topos = Kripke semantics makes it a genuine
  theorem), and the **soundness of a deep-embedded checker** `check : Form →
  Cert → Bool` for the solver's proof certificate. Then a positive Z3 result
  discharges `φ` by **computation**: `check (embed φ) π` reduces to `true`
  (canonicity, `16 §9`), and the discharge term is `sound φ π (refl true)`. Per
  obligation is a tiny term + a kernel evaluation; no per-proof reconstruction.
- **(b) Reconstruction (feasibility hedge).** Translate the solver's proof of
  `φ#` rule-by-rule into a native kernel proof (SMTCoq-style), or constructivize
  a classical proof (Herbrand/expansion-proof) — re-checked by the kernel.
  Retained **only** for any theory whose adequacy/checker proof turns out
  genuinely intractable, and as a differential cross-check during bring-up.

**Decision (`OQ-12`, ADR-class):** **(a) is the target architecture**, chosen on
*intrinsic* merits — it is a permanent semantic artifact (the kernel is
permanent), robust to solver proof-format drift, scales (tiny proof terms), and
**yields the mechanized embedding-adequacy theorem the kernel-soundness story
(G5) wants anyway**. It is *not* deferred on effort grounds; effort is the wrong
axis for a foundation (`SPEC-PROGRESS.md §stance`). (a) leans on the kernel
already decided: **canonicity** (proven for OTT, ADR 0005) makes the reflective
discharge compute, and **definitional proof irrelevance** (Ω is strict-prop)
means *any* kernel-accepted inhabitant discharges a propositional goal. The one
residual risk — whether the adequacy + checker-soundness metatheory *mechanizes
cleanly* — is a **feasibility** risk, not a labor one; it is retired by
front-loading a thin vertical slice of (a) (a minimal rule set, proved
end-to-end) before the full build, with (b) as the hedge if a fragment resists.

Cost note: the embedding adds a `World` sort and +1 arity to every predicate,
slowing Z3 — so it is reserved for FO; D uses direct/decision (§3).

## 5. Fragment HO — native intuitionistic + tactics

- **Propositional skeleton.** The intuitionistic propositional structure of a
  goal is decided by a kernel-verified **IPC decision procedure** (an
  `Itauto`/`intuit`- style reflective tactic): it returns a proof term or a
  Kripke countermodel (`24`). This handles the connective scaffolding even when
  atoms are abstract.
- **Induction / higher-order.** Goals needing induction over an inductive
  family, or quantifying over types/predicates, are out of SMT scope. The prover
  applies a small library of **tactics**
  (intro/apply/induction/rewrite-by-`Eq`/`decide`) and, where automation stops,
  leaves a **typed hole** with the remaining goal and context for an agent or
  human to fill (`24 §holes`, the REPL loop `21 §3`).
- Full higher-order *automated* proving is an explicit non-goal
  (`01-strategy.md`); interactive tactics + the agent loop serve instead.

## 6. Generalize the existing backends (the V3 work)

The prototype's Z3/Coq backends handle only **naturality of natural
transformations with single-variable Real-arithmetic bodies** (digest §8b). V3
is to **generalize**: arbitrary decidable atoms over
`Int`/`Decimal`/`Bool`/handles and finite domains (D); the full Kripke embedding
for FO; the IPC tactic and the induction tactics for HO — not to build a prover
from scratch. The **Coq backend is retired** (`OQ-12`): Ken's own kernel is the
proof checker, so an external Coq dependency would enlarge the trusted base
against the small-permanent-Rust-kernel principle (ADR 0001/0004). Z3 is the
primary solver; **cvc5** is an optional second oracle (proof-friendly
Alethe/LFSC output, useful for the (a) checker and for cross-checking).

## 7. Soundness obligations (what must actually be proved/ensured)

1. **Kernel re-checks every certificate** (§1) — the backbone; nothing else here
   can break soundness if this holds.
2. **The Kripke embedding's adequacy theorem** (§4) — mechanized once as a
   kernel meta-lemma (route a, the target), paired with the verified certificate
   checker; reconstruction (b) is the feasibility hedge. Needed for the FO tier
   either way.
3. **Reflective decision procedures are kernel-verified** (§3, §5) — `dec`
   returns a genuine `Decidable φ`, checked by the kernel like any term.
4. **The classifier is conservative** (§2) — and even if it weren't, (1) holds.

Only (1) and (3) are *enforced* by the kernel automatically; (2) is a proof
obligation on the prover's construction and (4) is a quality property. The
trusted base (`../10-kernel/18 §5`) gains **nothing** from the prover — Z3/cvc5
are never trusted.

## 8. What WS-V must deliver here (V3)

The classifier (D/FO/HO); reflective decision for D + SMT search/reconstruction;
the Kripke embedding + the **reflective certificate route (a)** — mechanized
adequacy + a verified certificate checker — with (b) reconstruction as a
feasibility hedge; the IPC reflective tactic and the core induction/rewrite
tactics; generalization beyond the naturality domain; and the documented
guarantee (G3) that the classical solver cannot yield a false `proved`.
Acceptance ties to **G3**. Conformance: `../../conformance/verify/prover/` — a
decidable arithmetic goal (reflective), an FO-intuitionistic goal via the
embedding (with a re-checked certificate), an IPC propositional goal, and a
**soundness regression** in which Z3 "proves" a
classically-valid-but-topos-invalid `φ` whose certificate the kernel **rejects**
(demonstrating the criterion).
