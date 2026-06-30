# The automated prover

> Status: **V3 elaborated** (implementation-ready). Normative for the prover
> contract, the **verdict trichotomy** and its projection to V1's four-way
> epistemic status (§1), the **exhaustive** classifier (§2), the soundness
> discipline (the de Bruijn re-check, §1.5), and the Kripke embedding +
> **reflective certificate route (a)** (`OQ-12` DECIDED, §4). The embedding's
> exact frame axioms are tagged `(oracle/standard)`; what must be *proved* vs
> *assumed* is ledgered (§4/§7). **★★ (untrusted):** every certificate is
> kernel-re-checked (`../10-kernel/18 §4`), so a prover bug is a failed/weaker
> verdict, never unsoundness. Contract for WS-V **V3** (third WP of the spine
> V1→V2→V3).

## 1. Contract

### 1.1 Input — one obligation

The prover consumes **one obligation at a time**: a triple
`⟨id, Γ ⊢ φ, provenance⟩` (`22 §1`) with goal `φ : Ω_ℓ` in a local hypothesis
context `Γ`. Obligations are **independent** — provable in any order / in
parallel (`22 §5`/§6) — so the prover is a per-obligation function with no
cross-obligation state. `id` and `provenance` are threaded to the verdict for
diagnostics (`24`) and the protocol (`25`); the proof search reads only `Γ ⊢ φ`.

### 1.2 Output — the verdict trichotomy (`21 §5.1`)

Attempting `Γ ⊢ φ` yields exactly one **verdict** — the kernel/Heyting
trichotomy V1 fixes (`21 §5.1`, `12 §5`), each carrying the evidence that makes
it actionable and (for `proved`) re-checkable:

| Verdict | Evidence produced | Kernel re-check |
|---|---|---|
| `proved` | a **certificate** — a core term `p` with `Γ ⊢ p : φ` | `check(env, Γ, p, φ)` accepts (`18 §4.5`) — the de Bruijn criterion; the *sole* reason `proved` is believed |
| `disproved` | a **countermodel** — a finite Kripke model forcing `¬φ` at some world (`24 §1`); where the backend yields a proof of `¬φ`, the cert `q : ¬φ` is `check`ed too | proof of `¬φ`: `check(env, Γ, q, ¬φ)`; bare countermodel: prover-asserted refutation — untrusted, a concrete falsifying witness (`21 §5.1`) |
| `unknown` | a **typed hole** `?id : φ` in `Γ`, admitted as a **postulate** of `φ` (`22 §1`, `24 §2`) | none — the hole is *assumed*; its goal appears in `trusted_base()` (§1.3) |

There is **no fourth verdict and no `failure` catch-all**: a search that neither
closes nor refutes `φ` is `unknown`-with-hole (honest — the program still runs,
`21 §5.1`), never a silent drop. (This sharpens the earlier
`certificate | failure` split: "failure" resolves into `disproved` vs `unknown`,
which carry *different evidence* and *different downstream meaning* — a refuted
claim is fixed, an open one is left running.)

### 1.3 The honesty guard — `proved` is kernel-structural, not a prover flag

The prover **cannot mark** an obligation `proved`. Per the V1 honesty guard
(`21 §5.4`, `18 §5`): `Γ ⊢ φ` is `proved` **iff** a certificate `p` `check`s
**and** no postulate carrying `φ` sits in `GlobalEnv::trusted_base()`
(`18 §4.1`/§5). An undischarged obligation *is* a `declare_postulate` of `φ`, so
its goal is enumerated by `trusted_base()`; discharging retires the postulate
(the certificate replaces the assumption). The verdict is therefore decidable
from the **kernel's own state**, with **no side-channel / parallel "proved"
store** the prover could write — a prover bug can leave a hole (`unknown`) or
emit a cert the kernel rejects, but can **never** forge `proved`. This is the
V1-build kernel-structural-status carry, preserved.

### 1.4 Projection to V1's four-way epistemic status (the reconcile)

The frame's "four-way" is V1's **epistemic status** (`21 §5.2`, `OQ-spec`
DECIDED) — `proved` / `tested` / `delegated` / `unknown` — which is **not** the
prover's output. The prover produces the **verdict trichotomy** (§1.2); V1's
projection (`21 §5.3`) resolves *verdict × disposition* into the epistemic
status. The prover therefore realizes **three** of the four labels and **never**
the other two:

- `proved` verdict → **`proved`** status (the default disposition, discharged);
- `disproved` verdict → *no* exported status — a refuted claim is a hard
  verification error (`24 §3`), fixed not shipped (`21 §5.3`);
- `unknown` verdict → **`unknown`** status (an open typed hole);
- **`tested`** and **`delegated`** are **dispositions** (`test`/`assume` and
  temporal clauses, `21 §5.2`) that **bypass the prover entirely** — they are
  not static obligations V3 attempts, so V3 never produces them.

So "wire the prover to V1's four-way status" means: emit the trichotomy verdict
with its evidence, keyed by `id`, for `21 §5.3` to project — **not** that the
prover itself emits four outcomes. Conflating the two is this chapter's central
reconcile hazard (the analog of `21 §5`'s verdict-vs-status separation).

### 1.5 The cardinal rule (de Bruijn criterion)

The prover is **untrusted**. No backend's "yes" is accepted on its own authority
— a kernel-checkable certificate is always produced and re-checked (§1.2). A bug
in the classifier, the embedding, Z3, cvc5, or a tactic can only cause a
*failure to prove* (→ `unknown`) or a *rejected certificate* (→ not `proved`),
**never a false `proved`.** This is what licenses using a classical solver under
an intuitionistic logic (§4), and it is the single property the rest of this
chapter exists to preserve.

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
  with no embedding (the key fragment boundary).
- The classifier is **conservative**: when unsure whether an atom is decidable
  or a formula is FO, it routes *upward* (to the more general, more expensive
  method). Misclassification downward would risk unsoundness *if the certificate
  weren't re-checked* — and it is, so the worst case of even a buggy classifier
  is wasted work or a failed proof, never unsoundness.

### 2.1 Routing is total — the completeness backstop

The classifier is a **total function** `classify : Obligation → Route` over the
*fixed* obligation form (`22 §1`: a goal `φ : Ω` built from the kernel's
proposition formers, `16 §1`). It is **exhaustive by construction**, mirroring
V2's extraction discipline (`22 §2.5`/§5): every `φ` routes to D, FO, or **HO as
the default** — HO is the catch-all that always applies (a typed hole is always
a legal HO outcome), so an unrecognized or future formula shape **falls to HO**,
never to a silent skip:

```
classify(⟨id, Γ ⊢ φ, _⟩) → Route:
  case shape(φ) of
    decidableAtoms(φ)       → D       -- §3: φ ∨ ¬φ holds on every atom
    firstOrderIntuit(φ)     → FO      -- §4: the Kripke embedding
    _                       → HO      -- §5: tactics + typed hole — the DEFAULT arm
  -- NO `_ ⇒ skip`. Every obligation receives a route; one the classifier cannot
  -- place lands in HO and is attempted (or left an honest typed hole), never
  -- dropped as if discharged.
```

**Why totality is load-bearing (the two-soundnesses split, V2 carry).** A
*wrong* route is harmless: a misclassified-downward `φ` either yields a
certificate the kernel **re-checks** (still sound) or fails and becomes an
honest `unknown` — wasted work, never a false `proved`. But a **never-routed**
obligation is **not** backstopped: it supplies *no* certificate and leaves *no*
hole, so its goal never enters `trusted_base()` and the claim reads as
discharged though never attempted — a silent verification-soundness gap (the
exact V2 *omission* hazard, `22`). The kernel re-checks what the prover
*supplies*; it cannot see what the classifier *omits*. So **exhaustiveness of
routing is the sole safeguard against a dropped obligation**, asserted
**structurally** (a total `case` with a default arm, no `_ ⇒ skip`) — the
discriminating conformance case drives an obligation of *every* shape through
`classify` and asserts each receives a route (no silent unrouting), the
omission-guard analog of V2's `exhaustive-traversal-no-silent-skip`.

## 3. Fragment D — decidable atoms

Two cooperating mechanisms, both yielding kernel certificates:

1. **Reflective decision (preferred).** For atoms with a kernel-verified
   **decision procedure** `dec : (x : A) → Decidable (φ x)`, the certificate is
   *by computation*: the kernel evaluates `dec a` (**canonicity**, `16 §9` C6:
   closed canonical terms compute) to `inl proof` or `inr refutation`. Here
   `Decidable P` is the **derived** sum `P + (P → Empty)` (`16 §1.3` connectives
   + `Empty`) — *not* a kernel primitive — and `dec` is an ordinary
   kernel-checked function; "kernel-verified" means `dec`'s type `check`s like
   any term (`18 §4`). Because the kernel **computes**, "decide it" produces a
   real proof term with no external solver in the trusted path — Ken's computing
   core is a verification asset here. Used for concrete/closed decidable goals.
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
  theorem), and the **soundness of a deep-embedded checker** `check_cert : Form
  → Cert → Bool` for the solver's proof certificate. (`check_cert` is a
  *Ken-level reflective function* over quoted formulas — **distinct from the
  kernel API `check`** of `18 §4`, which is the trusted re-check of its output.)
  Then a positive Z3 result discharges `φ` by **computation**: `check_cert
  (embed φ) π` reduces to `true` (canonicity, `16 §9`), and the discharge is
  `sound φ π (refl true)`. Per obligation is a tiny term + a kernel evaluation;
  no per-proof reconstruction.
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
axis for a foundation (`OQ-12`). (a) leans on the kernel already decided:
**canonicity** (proven for OTT, ADR 0005) makes the reflective discharge
compute, and **definitional proof irrelevance** (Ω is strict-prop) means *any*
kernel-accepted inhabitant discharges a propositional goal. The one residual
risk — whether the adequacy + checker-soundness metatheory *mechanizes cleanly*
— is a **feasibility** risk, not a labor one; it is retired by front-loading a
thin vertical slice of (a) (a minimal rule set, proved end-to-end) before the
full build, with (b) as the hedge if a fragment resists.

**Proved vs assumed — the soundness ledger (route (a)).** State precisely what
the FO tier rests on; only the first two rows are theorems the build must
*prove*, the rest is computed-and-re-checked or assumed-as-standard:

| Piece | Status | How it is discharged |
|---|---|---|
| **Embedding-adequacy** `classically_valid(φ#) → φ` | **must be PROVED** — mechanized *once* as a kernel meta-lemma (Ken's topos = Kripke semantics makes it a genuine theorem, not an axiom) | a kernel-checked term: the §4 soundness theorem internalized |
| **Checker soundness** `check_cert (embed φ) π = true → φ` | **must be PROVED** — proved once for the deep-embedded `check_cert` | kernel-checked; thereafter each discharge is `sound φ π (refl true)` |
| **Per-obligation discharge** `check_cert (embed φ) π ⇝ true` | **computed + re-checked** every time | `whnf`/`check` (`18 §4`), canonicity (`16 §9`) |
| **Kripke frame axioms** (domain monotonicity, `≤` preorder, monotone forcing) | **ASSUMED** `(oracle/standard)` — the standard Kripke-sheaf side conditions of the topos | not Ken terms; they shape `φ#`, the *external* classical theory only |
| **Z3 / cvc5 "unsat"** | **ASSUMED NOTHING** — a search oracle only | discarded unless it yields a `check_cert`-passing `π` |

The two **PROVED** rows are the soundness bridge; everything the solver does is
*search* whose output (a certificate `π`) is re-validated. A backend "unsat"
with **no constructible `π`** is **`unknown`**, never `proved` (§1.2) — the same
de Bruijn discipline as D, one tier up. Front-loading a thin vertical slice of
the two PROVED rows (a minimal rule set proved end-to-end) is what retires the
one residual feasibility risk (`OQ-12`), with (b) as the per-fragment hedge.

Cost note: the embedding adds a `World` sort and +1 arity to every predicate,
slowing Z3 — so it is reserved for FO; D uses direct/decision (§3).

## 5. Fragment HO — native intuitionistic + tactics

- **Propositional skeleton.** The intuitionistic propositional structure of a
  goal is decided by a kernel-verified **IPC decision procedure** (an
  `Itauto`/`intuit`-style reflective tactic): it returns a **proof term** (→
  `proved`) or a **Kripke counter-model**. The counter-model's verdict follows
  §1.2 and `24 §1`/§3 — **not** "invalid ⇒ disproved": a model that **forces
  `¬φ`** (the `S_{¬φ}` region, `24 §3`) is `disproved`; a model that merely
  **fails to force `φ`** while `¬¬φ` still holds — the `¬¬φ ⇒ φ` gap, e.g. an
  abstract-atom LEM instance `p ∨ ¬ p` (intuitionistically invalid but **not
  refutable**, since `¬(p ∨ ¬ p)` is itself false) — is **`unknown`**, not
  `disproved`. (The de Bruijn discipline still holds either way: `proved`
  requires the returned proof term to `check`.) This handles the connective
  scaffolding even when atoms are abstract.
- **Induction / higher-order.** Goals needing induction over an inductive
  family, or quantifying over types/predicates, are out of SMT scope. The prover
  applies a small library of **tactics**
  (intro/apply/induction/rewrite-by-`Eq`/`decide`) and, where automation stops,
  leaves a **typed hole** with the remaining goal and context for an agent or
  human to fill (`24 §2`, the REPL loop `21 §3`).
- **Sub-obligation descent + certificate composition (the V2-descend carry).** A
  tactic that **decomposes** a structured goal generates **sub-obligations**,
  each itself routed (§2.1) and discharged, with the certificate **composed**
  from the parts — never a single opaque obligation over the whole structured
  term:
  - **∧-split / all-prop record goal** `φ ∧ ψ` → subgoals `φ`, `ψ`; certificate
    is the pair `(p_φ, p_ψ)` (`16 §1.3`).
  - **⇒/∀-intro** → move the antecedent / binder into `Γ`, discharge the body;
    certificate is the `λ`.
  - **induction over an inductive family** → **one subgoal per constructor**,
    and each recursive-field subgoal carries the **induction hypothesis** — the
    motive instance `M zᵢ` — in its `Γ`, exactly the body-as-motive structure V2
    builds at extraction (`22 §4`); here the *tactic* synthesizes it. A single
    goal over the whole inductive structure carries **no IH** and cannot be
    discharged — the descent is **required, not an optimization**. The cert is
    the eliminator application `elim_D M methods… z` whose methods are the
    per-constructor sub-certificates (`14 §3`).

  The composed certificate is **one core term**, `check`ed once at the top goal
  (`18 §4.5`); a sub-certificate the prover cannot build leaves a **typed hole
  at that subgoal** (precisely located, `24 §2`), turning *that* leaf `unknown`
  while its siblings stay `proved` — partiality is per-subgoal, not
  all-or-nothing. (This is the prover-side instance of the obligation-descend
  discipline V2 applies at extraction: an obligation over an eliminator must
  split per-branch with the IH, or it is unprovable.)
- Full higher-order *automated* proving is an explicit non-goal
  (`../../docs/program/01-strategy.md`); interactive tactics + the agent loop
  serve instead.

## 6. Backend scope (the V3 work)

V3 builds the SMT-backed tiers in full: arbitrary decidable atoms over
`Int`/`Decimal`/`Bool`/handles and finite domains (D); the full Kripke embedding
for FO; the IPC tactic and the induction tactics for HO. There is **no external
proof-checker dependency**: Ken's own kernel is the proof checker, so an
external Coq dependency would enlarge the trusted base against the
small-permanent-Rust-kernel principle (ADR 0001/0004). Z3 is the primary solver;
**cvc5** is an optional second solver (proof-friendly Alethe/LFSC output, useful
for the (a) checker and for cross-checking).

## 7. Soundness obligations (what must actually be proved/ensured)

1. **Kernel re-checks every certificate** (§1) — the backbone; nothing else here
   can break soundness if this holds.
2. **The Kripke embedding's adequacy theorem** (§4) — mechanized once as a
   kernel meta-lemma (route a, the target), paired with the verified certificate
   checker; reconstruction (b) is the feasibility hedge. Needed for the FO tier
   either way.
3. **Reflective decision procedures are kernel-verified** (§3, §5) — `dec`
   returns a genuine `Decidable φ`, checked by the kernel like any term.
4. **The classifier is exhaustive** (§2.1) — every obligation is routed to
   *some* outcome (no silent drop). This is **NOT kernel-enforced** and is the
   verification-soundness linchpin on the prover side: a never-routed obligation
   supplies no cert and no hole, so it escapes `trusted_base()` and reads as
   discharged though never attempted (the two-soundnesses *omission* gap, `22`).
   Backstopped **only** by the structural totality of `classify` (§2.1).
5. **The classifier is conservative** (§2) — routes *upward* when unsure. A
   **quality** property only; even a non-conservative classifier cannot break
   (1), since the certificate is re-checked regardless.

Only (1) and (3) are *enforced* by the kernel automatically; (2) is a proof
obligation on the prover's construction (the §4 ledger); (4) is a **structural
completeness** obligation on the classifier — *not* kernel-caught, discharged by
exhaustive-by-construction routing (§2.1); and (5) is a quality property. The
trusted base (`../10-kernel/18 §5`) gains **nothing** from the prover — Z3/cvc5
are never trusted.

**Two classical bridges, not one (contrast with the Ward seam).** This chapter's
bridge uses a classical solver to discharge an obligation **here, with a kernel
certificate** — the result becomes `proved`. The downstream **behavioral seam**
(`../70-behavioral/71`) also runs a classical engine under Ken's logic, but its
results are **not re-checkable as Ken proofs** (a depth-`k` model-check is not a
proof for all `N`; a green monitor is not a proof). So that bridge is
**one-way** (`OQ-classical-bridge`): Ken exports obligations + assumptions, Ward
discharges them, and the outcome returns as a signed **discharge attestation**
(`../60-security/63 §5a`) tagged `delegated`/`tested` — **never promoted to
`proved`**. Soundness is by *assume-guarantee construction* (Ken proves `Q ⊣ P`;
the discharge of `P` is a separate, lower-trust artifact), and the strong,
*Ken-checked* part is **translation faithfulness** (`../70-behavioral/71 §5`),
the exact analog of §4's adequacy lemma — proved once at the compiler level.

## 8. Level-discipline reconcile

Per the standing directive, the level computations are made explicit and
reconciled against `12`/`16 §1.1`. V3 introduces **no new kernel former or
universe**, so the reconcile is mostly accounting that nothing bumps a level:

- **Goals stay in Ω at V2's level.** Every obligation goal is `φ : Ω_ℓ`
  (`22 §1`/§7); the prover *consumes* it and *produces* a proof `p : φ` at the
  **same** `Ω_ℓ` — proof terms in Ω are proof-irrelevant and erasable
  (`16 §1.2`), so the certificate adds no level. `check(env, Γ, p, φ)` is an
  ordinary kernel check (`18 §4.5`); no level appears beyond the goal's.
- **The Kripke embedding is external.** `φ#`, the `World` sort, and the
  `(n+1)`-ary `P#` (§4) live in the **classical FO theory handed to Z3** — they
  are **not** Ken kernel terms and carry **no Ken universe level**. The only Ken
  terms the FO tier produces are the certificate `π : Cert` and the discharge
  `sound φ π (refl true) : φ`, at `Ω_ℓ` as above.
- **The reflective types are ordinary data.** `Form`, `Cert`, `Decidable P`
  (§3), and the IPC proof terms (§5) are **derived inductives** (`14`, `16 §1.3`
  connectives + `Empty`) at their natural `Type ℓ` — concrete data with no level
  parameters beyond those of the atoms they quote — and `check_cert : Form →
  Cert → Bool` is an ordinary kernel-checked function. None introduces a
  universe or a proposition former.
- **The adequacy + checker-soundness meta-lemmas** (§4 ledger) are themselves
  kernel-checked terms whose statements are propositions (`→`/`∀` over the
  reflective data, landing in Ω by codomain-keying, `16 §1.1`); they reuse Ω and
  `Eq`, introducing no new universe. Consistent with `12`'s predicative,
  non-cumulative regime — no implicit lifts anywhere in the prover.

## 9. What WS-V must deliver here (V3)

The per-obligation contract emitting the **verdict trichotomy** (§1.2) keyed by
`id` for V1's status projection (`21 §5.3`), with the honesty guard
kernel-structural via `trusted_base()` (§1.3); the **exhaustive** classifier
(D/FO/HO with HO the default, §2.1); reflective decision for D + SMT
search/reconstruction; the Kripke embedding + the **reflective certificate route
(a)** — mechanized adequacy + a verified `check_cert`, the proved-vs-assumed
ledger (§4) — with (b) reconstruction as a feasibility hedge; the IPC reflective
tactic and the core induction/rewrite tactics with **per-branch sub-obligation
descent + certificate composition** (§5); generalization beyond the naturality
domain; and the documented guarantee (G3) that the classical solver cannot yield
a false `proved`. Acceptance ties to **G3**. Conformance:
`../../conformance/verify/prover/` — a decidable arithmetic goal (reflective);
an FO-intuitionistic goal via the embedding (re-checked certificate); an IPC
propositional goal; an `unknown` goal whose typed hole is `trusted_base()`-
distinct from `proved` (the absence-assertion, §1.3, naming its guard —
postulate membership); an **exhaustive-classifier** case driving an obligation
of each shape through `classify` with none silently unrouted (§2.1, structural);
and a **soundness regression** in which Z3 "proves" a classically-valid-but-
topos-invalid `φ` whose certificate the kernel **rejects** — the verdict-flip
(`proved` → not `proved`) showing the de Bruijn criterion is load-bearing.
