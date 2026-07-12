# WP: proof-attachment membership convention — style guide + NatArith/OrdNat pilot

**Owner:** Foundation (catalog authoring). **Size:** M. **Risk:** low
(outer-ring catalog + program-doc; zero spec/conformance/kernel/TCB).
**Base:** `origin/main @ 6ec4577e` (fetched ref). **Review:** Architect-terminal
(surface + fidelity + naming), as the pedagogic prototype (`288b3979`) — no
CV/Spec lane (no `spec/`/`conformance/` touch). **Kind:** catalog + style-guide.

## Objective

Adopt the **proof-attachment membership convention** the Architect ruled
(`evt_5ae24zwdjratp`, precision `evt_1h6ez7mx5024t`; operator-concurred) in the
catalog authoring **standard**, and realize it in the two reference packages
`NatArith` and `OrdNat` (the pedagogic-prototype pair). This gives the
post-migration-near-dead `proof … for` keyword its honest job and groups each
subject's laws discoverably under `S::…`.

## Fixed inputs (settled — do NOT reopen or relitigate)

The convention encodes **MEMBERSHIP, not ROLE** (a keyword may carry only what
the elaborator can check; `occurs-applied-in-φ` checks membership, not "headline
vs scaffolding"):

- **`proof name for S`** = a **named theorem in S's public theory** — checked to
  be *about S* (S occurs applied in the proposition). Bound and referenced as
  **`S::name`** (e.g. `leq_nat::refl`, `resolve.rs:939`).
- **`lemma`** = a theorem with **no single owning subject**: a genuinely
  interior step, or a cross-cutting law about the **interaction** of several
  definitions where no one subject owns it (e.g. `listApCmpIsNotJustRefl`).
- **Decision procedure (the tie-breaker):** *can you name exactly **one** owning
  subject?* → `proof name for S`. *Cannot?* → `lemma`.
- **Membership is NOT a usage/exclusivity test.** A `proof name for S` law may
  be **freely cited from anywhere** — being reused by other subjects' proofs
  does **not** demote it (cross-namespace `S::name` citation is ordinary). Do
  **not** phrase the rule as "used only by S," and do **not** move a
  subject-owned law to `lemma` merely because it is reused. (This overrides the
  earlier "reused ⇒ lemma" instinct — reuse is orthogonal.)
- **Reference form:** self- and cross-reference use the lightweight **`S::name`**
  form — the recursive arm reads `Suc x2 ⇒ leq_nat::refl x2`, no heavier than
  the old `refl_leq_nat x2`.
- **No elaborator dependency.** Recursive `proof … for S` + attachment already
  compose (Architect probed green on `main @ 6ec4577e`); this is a
  **surface/authoring** change only — no kernel/elaborator work.

## Scope / deliverables

**1. `docs/program/07-catalog-style-guide.md` — the standard.** Update §6 (Proof
presentation) and §9 (Naming) to state the convention: the `proof name for S`
vs `lemma` distinction, the single-owning-subject decision procedure,
membership-not-role (with the "don't demote a reused law" caveat), and the
`S::name` bind/reference form. Retire the flat `X_subject` lemma naming (e.g.
`refl_leq_nat`) for **subject-owned** laws in favour of `proof X for S` /
`S::X`. Keep the existing Ω-partition rule (`proof-vocab-completion`) intact —
this convention refines *which proof keyword*, not the fn⊥Ω line.

**2. `catalog/packages/Core/NatArith.ken.md`** — rework every **subject-owned**
law to `proof name for S` (`S::name`):
- `leq_nat` laws: `refl_leq_nat`→`proof refl for leq_nat` (`leq_nat::refl`),
  `trans_leq_nat`→`leq_nat::trans`, `antisym_leq_nat`→`leq_nat::antisym`.
- `add`/`mul` laws by their single owner: `add_comm`→`add::comm`,
  `add_assoc`→`add::assoc`, `mul_comm`→`mul::comm`, `mul_assoc`→`mul::assoc`,
  `add_zero_l`→`add::zero_l`, etc. — the owner is the definition that occurs
  applied in φ.
- **Judgment cases (apply the procedure, don't force):** an *interaction* law
  naming two owners with no single owner — e.g. distributivity
  `mul_add_distrib_*` (about `mul`×`add` together) — is a candidate to **stay
  `lemma`** unless one definition genuinely owns it. Foundation decides per law
  via the single-owning-subject test; flag the calls for Architect review.

**3. `catalog/packages/Core/OrdNat.ken.md`** — same: `leq`/order laws → their
owning subject (`leq_nat::…` / the Ord subject), `lemma` only where no single
owner.

**Preserve (fidelity, keyword/name change ONLY):** every law's exact
proposition **and** proof term is unchanged — this is a rename+keyword swap, not
a re-proof. Do **not** replace any proof body with `Proved`/`Refl`/postulate.
`fn total_leq_nat : Or …` **stays `fn`** (Type-valued, proof-relevant — not a
proof at all). `fn`/`const` computational defs are untouched.

## Acceptance criteria (testable)

1. §6/§9 of the style guide state the membership convention, the
   single-owning-subject procedure, membership-not-role (incl. the no-demote
   caveat), and the `S::name` form.
2. In both packages, every subject-owned law is `proof name for S` bound
   `S::name`; every genuinely-ownerless law is `lemma`; no `fn`/`const` changed;
   the fn⊥Ω partition is preserved.
3. Recursive/cross references use `S::name`; **both packages elaborate green**
   (full catalog acceptance net — `nat_arithmetic_laws_acceptance`,
   `ds2_ord_nat_acceptance`, `surface_named_proof_claims`, and the package build
   tests).
4. **Fidelity:** every original law preserved by proposition + proof term
   (name/keyword change only); no law weakened, none re-proved by `Proved`/`Refl`
   /postulate. (Architect diffs bodies, as for the prototype.)
5. **Doc/catalog-only:** no `spec/`, `conformance/`, `crates/`, `Cargo`, or
   lockfile change; `git diff --check` clean.

## Do-not-reopen guardrails

- The convention is **ruled + operator-concurred**: membership not role. Don't
  relitigate the axis; don't move a subject-owned law to `lemma` for being
  reused.
- Don't touch `fn total_leq_nat` (Type-valued) or the Ω-partition.
- Don't invent a new reference syntax — use the landed `S::name`.

## Companion (separate track, not this WP)

The **normative spec convention clause** (spec-author) + the `tt`→`Proved`
errata (`spec/` + `conformance/`, CV) run on the Spec-enclave track
(frame `tt-proved-errata.md` + the ruled clause). This WP is the catalog +
authoring-standard realization only; the two are grounded on the same ruling.
