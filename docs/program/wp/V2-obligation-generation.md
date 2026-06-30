# WP V2 — Obligation generation (the verification-condition extractor)

> **Status:** Steward frame — **next enclave WP** (deps met: K-api + V1 both on
> `main`). spec-leader elaborates `spec/20-verification/22-obligations.md` (DRAFT
> → implementation-ready), then the **Verify team** builds (after V1-build).
>
> **Team:** Verify · **Deps:** **K-api** (`2f1cdf8`, the cert/`Ω` contract) +
> **V1** (`a79c790`, the spec syntax it consumes) · **Size:** M · **Risk:** ★★
> (**untrusted** — every obligation/cert is kernel-re-checked; a bug is a missed
> or spurious obligation, never unsoundness) · ► On the verification spine
> **V1→V2→V3** (V2 feeds the prover V3 + the four-way status).

## Objective

Elaborate `22-obligations.md` — the **verification-condition extractor**: turn a
V1-spec'd program into a set of **proof obligations**, each a triple *(a
proposition in `Ω`, its local hypothesis context `Γ`, provenance back to the
source clause)*, that the prover (`23`/V3) discharges and the **kernel
re-checks** (`18 §4`). This is the bridge from V1's *syntax* (`requires`/`ensures`
/refinements) to V3's *proof search* — the obligations are V2's deliverable, the
discharge is V3's.

## The framing that sets the risk level

V2 is **untrusted** — ★★. The kernel re-checks every discharged certificate, so a
V2 bug is a **missed** obligation (an unproven property silently treated as
holding → surfaces as a wrong `proved`/`unknown` *verdict*, caught downstream) or
a **spurious** one (over-conservative — a false `unknown`), **never** unsoundness
(a bogus cert is kernel-rejected regardless). The load-bearing properties are
**completeness of extraction** (every spec clause that bears a proof burden
yields its obligation — the absent-clause scan applies: which `ensures`/refinement
sub-case yields *no* obligation?) and **honest provenance** (each obligation
traces to its source clause for diagnostics + the four-way status).

## Scope

**IN:** the **obligation triple** (proposition `: Ω` / context `Γ` / provenance);
the **extraction algorithm** — from V1's elaborated `requires`/`ensures`/
refinements/goals to obligations, with the local hypothesis context built at each
site (preconditions + path conditions + `old`-bindings in scope); the **V1→V2
interface** consumed (V1 emits carrier-plus-obligation, *not* `Σ(B,ψ)` — V2
reads that bare-obligation form, **decoupled from the Σ-sort erratum**); the
**V2→V3 interface** (the obligation set handed to the prover, keyed for the
verdict projection).

**OUT — other WPs:** the **prover** / proof search (`23`, V3); the **kernel cert
check** (`18 §4`, already the kernel's); refinement-subtyping *proof* (the
obligation is generated here; discharged in V3); diagnostics polish (`24`).

## The elaboration this needs (spec-leader → spec-author + Architect)

Elaborate `22` to builder rigor: the obligation triple's precise shape; the
extraction as **defensive pseudocode** (per V1 clause-form → obligation(s),
context construction, `old`/path-condition handling); the V1→V2 and V2→V3
interfaces stated explicitly. **Ground against the *landed* V1 (`21`) + K-api
(`18 §4`) + the corpus — the files, not status** (the V1 squash-drop lesson:
a base can silently lag its notification). Conformance (`conformance/verify/
obligations/`): each obligation-bearing clause yields its obligation
(**discriminating** — a clause with a real burden vs. a trivially-true one give
**different** obligation sets; per-dimension); the **absent-clause scan**
(a spec sub-case that yields no obligation is a hole); obligations are in `Ω`
with correct `Γ`; provenance round-trips. Level-discipline reconcile any
`Ω`-level computation.

## Acceptance (testable)

1. **Extraction completeness:** every `requires`/`ensures`/refinement/goal that
   bears a proof burden yields its obligation(s) with the right `Γ`; a clause that
   is trivially true yields the (provable) obligation, not *no* obligation
   (absent-clause discriminating).
2. **`Ω` + context correctness:** each obligation's proposition is in `Ω`; the
   hypothesis context carries the in-scope preconditions/path-conditions/`old`.
3. **Decoupled from Σ-sort:** obligations are read from V1's **bare-carrier +
   separate-obligation** form, never `Σ(B,ψ)`; V2 does not depend on `sort_sigma`.
4. **V2→V3 interface:** the obligation set is in exactly the form V3's proof
   search + the verdict projection consume (stated as the interface).
5. **No regression:** V1's elaboration + V0 unchanged for spec-free programs.

## Sequencing

Next enclave WP (K-api + V1 landed). **Build** follows V1-build. Unblocks **V3**
(the prover) → the four-way verdict end-to-end → **T1/V4**. Runs the verification
spine; interleaves with X1-effects-elab + Sec1 in the enclave. Build queries:
obligation semantics → Spec; design → Architect.
