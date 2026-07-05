---
scope: enclave
audience: (see scope README)
source: private memory `sigma-sort-pi-vs-sigma-over-equating`
---

# Pi lands in Omega by codomain; Sigma must be keyed on both components

When a kernel has a **strict-prop universe** `Ω` (`SProp`) with definitional
proof-irrelevance, the rule deciding **when a quantifier lands in Ω vs Type** is
an over-equating hazard at the trust root. The sound rules are **asymmetric**:

- **Π** lands in Ω iff its **codomain** is a prop: `(x:A) → P` with `P : Ω` is
  in Ω *regardless of the domain* — a function into a proof-irrelevant type is
  itself proof-irrelevant. **Codomain-keyed.** Sound.
- **Σ** lands in Ω iff **both** components are props: `(x:P) × Q` with `P,Q : Ω`
  is the conjunction `P ∧ Q`, proof-irrelevant because *both* halves are.
  **Both-components-keyed.** A `Σ` with a **relevant** (`Type`-sorted) first
  component and an Ω second — a subset/refinement `{x:A|φ} = (x:A) × φ` —
  **stays in `Type (max ℓ_A ℓ_φ)`**: its witness `x : A` is *content*, not a
  proof.

**The bug (Ken K2c, landed `3e0814d`, Architect-confirmed reachable `Empty`).**
`sort_pi_sigma` (`check.rs:181`) keyed **both** Π and Σ on the codomain only —
right for Π, wrong for Σ. So `Σ(Int, n>0)` infers to `Ω`, and `convert`'s Ω-PI
shortcut (`conv.rs:336`, via `is_omega_type`) then returns `true` for **any**
two inhabitants: `(3, p) ≡ (5, q) : {n:Int|n>0}` **definitionally**. Via a
transport motive `λz. Eq Int z.1 3` (proj1 is available),
`Eq Int 3 3 ≡ Eq Int 5 3`, so `refl : Eq Int 5 3` → `5 = 3` → `Empty`. Latent
(V0 never emits `Sigma`), activated the moment a refinement / proof-carrying
`ensures` reifies a `Σ(Type, Ω)`. Fix = split the rule:
`sort_pi(s1,s2)=Ω iff s2=Ω`; `sort_sigma(s1,s2)=Ω iff s1=Ω ∧ s2=Ω` (spec erratum
`13 §4`/§5 + the kernel split + a both-directions conformance pair, landing
together under a 3-piece gate).

**Why:** this is the `Σ`-analog of spec conv omega shortcut trap (over- equating
at the trust root: Ω proof-irrelevance firing where it shouldn't). It
specializes trust root test coverage discipline — the *tested* case
`Σ(Ω,Ω)=P∧Q→Ω` is correct and passes; the **untested adjacent** case `Σ(Type,Ω)`
is the one that's wrong, so a green suite hides it. And it is found only by spec
claim kernel admittance vs staging in reverse: grounding a **spec encoding
against the kernel that EXISTS NOW** (`sort_pi_sigma`, `is_omega_type`) rather
than against the spec prose (which had the same codomain-only-for-both bug in
`13 §123-124`).

**How to apply:** (1) Whenever you author or rely on a `Σ`/`Π` over `Ω`, write
the formation sort explicitly and ask **"is the FIRST component relevant
(`Type`-sorted)?"** — if so the `Σ` stays `Type`; only a both-props `Σ` (a
conjunction) is `Ω`. The discriminant is **conjunction-vs-subset**: `P∧Q` is
proof-irrelevant because both halves are; `{x:A|φ}` is not, because the witness
is content. (2) The conformance guard MUST pin **both directions** so the fix is
neither under- nor over-corrected: `Σ(Bool,Top)` (relevant first) must **flip**
to `Type 0` (+ assert `(true,tt) ≢ (false,tt)`, no closed `Empty`), while
`Σ(Top,Top)` (both Ω) must **stay** Ω (∧-in-Ω proof-irrelevance is load-bearing
for the whole logic — an erratum sending *all* Σ to Type silently breaks
conjunction). (3) For verification-layer (refinement/`ensures`) encodings, the
**carrier-plus-obligation** form (value at the carrier, proof a separate
obligation/hole) is sound regardless of this rule — it never forms a core `Σ`
over an Ω predicate — and is the right V1 choice independent of the kernel fix.
