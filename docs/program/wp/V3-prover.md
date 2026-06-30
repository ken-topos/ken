# WP V3 — The automated prover (obligation → kernel-checkable certificate)

> **Status:** Steward frame — **enclave WP after X1-effects-elab** (deps met: V2
> + K-api on `main`). spec-leader elaborates `spec/20-verification/23-prover.md`
> (DRAFT → implementation-ready), then **Team Verify** builds it (after V2-build).
>
> **Team:** Verify · **Deps:** **V2** (`ecd8ee8`, obligation generation — the
> input) + **K-api** (`18 §4`, the certificate re-check — the backstop) · **Size:**
> L · **Risk:** ★★ (**untrusted** — every certificate is kernel-re-checked, so a
> prover bug is a failed/weaker verdict, never unsoundness) · ► The **heart of
> the verification spine** (V1→V2→**V3**→V4) — the differentiator; gates the G2/G3
> thesis.

## Objective

Elaborate `23-prover` — the **automated prover**: take an obligation `Γ ⊢ φ`
(`φ : Ω`, from V2) and produce one of the **four-way** outcomes — a **certificate**
(a term `p` with `Γ ⊢ p : φ` that the **kernel re-checks**, `18 §4`), a
**disproved** with a countermodel, or **unknown** with a typed hole. This is what
turns Ken's spec annotations into an *actionable verdict without trusting the
prover*: the prover may be arbitrarily clever, but its `proved` is believed
**only** because the kernel re-checks the certificate (the de Bruijn criterion).

## The framing that sets the risk level

The prover is **entirely untrusted** — ★★. Soundness rides on **the kernel
re-checking the emitted certificate**, not on the prover's own correctness: a
bogus or mis-translated proof is **kernel-rejected**, so it can never yield a
false `proved`. Build it **capable + sound-by-construction-of-the-certificate**,
not trusted. The load-bearing properties:
- **Reflective-certificate soundness:** whatever a backend (SMT/Z3, the Kripke
  embedding) concludes must be **reified into a kernel-checkable core term** — the
  translation is the soundness bridge, and its *output* (not the backend's
  say-so) is what's trusted. A backend "unsat" with no checkable certificate is
  **not** a `proved`.
- **Honesty / the kernel-structural status** (V1-build carry — preserve it):
  `unknown ≢ proved`, derived from `trusted_base()` membership (an undischarged
  obligation *is* a postulate the kernel sees), **no side-channel/parallel store**.
  The prover cannot *mark* something proved; only a kernel-accepted cert makes it
  so.
- **The two-soundnesses split** (V2 carry): a *wrong* certificate is caught
  (kernel rejects it); a *never-attempted* obligation supplies no cert and would
  read `unknown` (honest) — but the **classifier must be exhaustive** (every
  obligation is routed to *some* outcome; no silent drop that masquerades as
  discharged). Completeness/exhaustiveness of routing is the load-bearing
  safeguard, asserted structurally.

## Scope

**IN:**
- **The prover contract** (`23 §1`): obligation → {certificate | disproved+model |
  unknown+hole}, wired to V1's four-way status projection (kernel-structural).
- **The classifier** — route each obligation to a backend/tactic by its shape
  (decidable fragment → SMT; structural → the Kripke/reflective path; …),
  **exhaustively** (a default route, never a silent skip).
- **The decision backend** (`23` route (a) target — the *settled* route per `23`
  + the OQ register; do **not** re-open the route choice): the SMT/Z3 path for the
  decidable fragment.
- **The Kripke embedding + reflective certificate** (`23`) — the mechanism that
  turns a backend result into a **kernel-checkable** core term (`Γ ⊢ p : φ`); the
  soundness bridge. Tag what must be *proved* about the embedding vs. assumed.
- **Certificate emission to `18 §4`** — the prover emits to the kernel's existing
  re-check API; it does **not** reimplement checking.

**OUT — other WPs:** **V4** (`23`'s downstream — counterexample feedback / the
remaining thesis gate); the **kernel cert checker** (already `18 §4`); V2's
obligation generation (done); diagnostics polish (`24`). No **new kernel
feature** — the cert rides the existing `Ω`/`Eq`/`18 §4` surface.

## The elaboration this needs (spec-leader → spec-author + Architect)

Elaborate `23` to builder rigor: the classifier's routing table (obligation shape
→ route, exhaustive); the backend contract + the **reflective-certificate
construction** as the soundness-load-bearing piece (state precisely what the
kernel re-checks and why a backend "proof" alone is insufficient); the four-way
projection wired to `trusted_base()`. **Ground against the *landed* V2
(obligation form) + K-api (`18 §4` cert API) + V1's status model — the files, not
status.** **The route choice is settled (`23` route (a)); pin it, don't reopen**
(the recon hedge is noted, not a fork to relitigate). Conformance
(`conformance/verify/prover/`): a dischargeable `φ` → a **kernel-accepted** cert
(and a **deliberately-corrupted** cert → kernel-**rejects** → verdict **not**
`proved`: the verdict-flip that proves the backstop is load-bearing); a false `φ`
→ disproved+model; an undecidable `φ` → unknown+hole, **distinct** from proved
(absence-assertion names its guard — `trusted_base()` membership); the classifier
is **exhaustive** (no obligation silently unrouted — structural).

## Acceptance (testable)

1. **Sound by re-check:** a discharged `φ` emits a cert the **kernel accepts**; a
   **corrupted/mis-translated** cert is **kernel-rejected** and the verdict is
   **not** `proved` (verdict-flip — the de Bruijn backstop is exercised, not
   assumed).
2. **Honest four-way:** `disproved` carries a countermodel; `unknown` carries a
   typed hole and is `trusted_base()`-distinct from `proved` (guard-gated absence,
   no side-channel).
3. **Exhaustive classifier:** every obligation is routed to an outcome; no silent
   drop (structural/exhaustive-traversal assertion — the two-soundnesses
   completeness backstop).
4. **Reflective bridge:** a backend result is reified into a checkable term; a
   backend "unsat" without a constructible certificate yields `unknown`, not
   `proved`.
5. **No regression:** V2's obligations + V1's status are consumed unchanged; the
   pure pipeline (no obligations) is unaffected.

## Sequencing

Enclave WP after X1-effects-elab; **build** follows V2-build on Team Verify.
Closes the verification spine to an end-to-end **actionable verdict** (V1 syntax →
V2 obligations → **V3 discharge + kernel re-check**) — the G2/G3 thesis gate.
Unblocks **V4**. Build queries: prover/embedding semantics → Spec; classifier +
backend architecture → Architect. Clean-room: landed V2/K-api + `23` + first
principles; permissive SMT-integration patterns are Spec-enclave-only, never
vendored.
