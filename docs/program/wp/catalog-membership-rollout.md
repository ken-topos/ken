# WP: membership-convention rollout — remaining catalog packages

**Owner:** Foundation (catalog authoring). **Size:** L (tiered into ≥3 PRs).
**Risk:** low — outer-ring catalog; rename+keyword only; zero
spec/conformance/kernel/TCB. **Base:** `origin/main @ 62f76ab1` (fetched;
re-verify at pickup). **Review:** Architect-terminal (surface + fidelity +
naming), as the pilot (`proof-attachment-membership-pilot`, merged `900c135f`) —
no CV/Spec lane (no `spec/`/`conformance/` touch). **Kind:** catalog authoring.

## Objective

Apply the **proof-attachment membership convention** — ruled by the Architect
(`evt_5ae24zwdjratp`, precision `evt_1h6ez7mx5024t`), operator-concurred, piloted
on NatArith + OrdNat, and now the catalog **standard** (`07-catalog-style-guide.md`
§6/§9) — to the **remaining** catalog packages, so each subject's laws are
grouped discoverably under `S::…` and the `proof … for S` keyword does its
honest job corpus-wide.

## Fixed inputs (settled — do NOT reopen or relitigate)

The convention is **ruled + operator-concurred + piloted.** Do not redesign it;
apply it. (Full statement: style guide §6/§9 + the pilot frame
`proof-attachment-membership-pilot.md`.) In brief:

- **`proof name for S`** = a named theorem in S's public theory — S occurs
  applied in the proposition (membership, checked by the elaborator). Bound and
  referenced **`S::name`**.
- **`lemma`** = a theorem with **no single owning subject**: a genuinely
  interior step, or a cross-cutting **interaction** law about several
  definitions where no one subject owns it.
- **Decision procedure (tie-breaker):** *can you name exactly **one** owning
  subject?* → `proof name for S`. *Cannot?* → `lemma`.
- **Membership is NOT usage/exclusivity.** A `proof name for S` law may be
  freely cited from anywhere; reuse does **not** demote it to `lemma`. (Overrides
  the old "reused ⇒ lemma" instinct — reuse is orthogonal.)
- **Interaction laws stay `lemma`.** Distributivity-style laws naming two owners
  (`mul_add_distrib_*`), naturality/coherence squares, and cross-definition
  bridges with no single owner **stay `lemma`** unless one definition genuinely
  owns the statement. This is the dominant case in Map/EffectfulClasses — expect
  **most** of their many `lemma`s to remain `lemma`.
- **Pedagogical showcases → Librarian's discretion (operator ruling
  2026-07-12).** Deliberately-wrong / counter-example decls (`…_wrong`) are
  commentary, not API; whether they attach or stay `lemma` is the **Librarian's
  call**, not a forced attach. Don't force them either way.
- **`fn`/`const` and the Ω-partition are untouched.** Type-valued
  proof-relevant defs (`total_leq_nat`-shaped `Or`-producers) **stay `fn`** — the
  `fn`⊥Ω line (`proof-vocab-completion`) is preserved, this refines only *which
  proof keyword*. The `↦` match glyph is already migrated (do not touch arms).
- **No elaborator dependency** — surface/authoring change only.

## Scope / deliverables — tiered into ≥3 PRs (one branch each)

Apply the single-owning-subject procedure **per law** in each package; rename
subject-owned laws to `proof name for S` / `S::name`; leave genuinely-ownerless
laws as `lemma`. **This is a judgment pass, not a mass-conversion** — the law
count is a ceiling, not a target.

- **PR-A (Tier A — S/M packages):** `EmptyDec`, `Transport`, `Sums`,
  `LawfulFunctors`, `Parsing`, `Collections`, and **finish `LawfulClasses`**
  (already 19 `S::` attached; complete the remaining subject-owned laws). These
  are small law-counts; batch as one branch.
- **PR-B (`EffectfulClasses`, L):** own branch — ~171 `lemma`s, mostly
  effect/class *interaction* laws expected to **stay `lemma`**; attach only the
  genuinely subject-owned ones.
- **PR-C (`Map`, L):** own branch — ~339 `lemma`s, overwhelmingly collection
  *interaction* laws expected to **stay `lemma`**; attach the map-owned laws.

Sequence PR-A → PR-B → PR-C; the Steward merges each through the honesty gate.

**Preserve (fidelity — rename+keyword ONLY):** every law's exact proposition
**and** proof term is unchanged; no law re-proved by `Proved`/`Refl`/postulate;
no `fn`/`const` changed; the `fn`⊥Ω partition intact. Self/cross references use
`S::name`.

## Acceptance criteria (testable, per PR)

1. Every subject-owned law in the PR's packages is `proof name for S` bound
   `S::name` (single-owning-subject); every genuinely-ownerless/interaction law
   stays `lemma`; no `fn`/`const` changed; Ω-partition preserved.
2. **Fidelity:** every original proposition + proof term preserved (name/keyword
   change only); no law weakened. (Architect diffs bodies, as for the pilot.)
3. Recursive/cross references use `S::name`; **all touched packages elaborate
   green** (per-package catalog acceptance nets + the package build tests).
4. **Doc/catalog-only:** no `spec/`, `conformance/`, `crates/`, `Cargo`, or
   lockfile change; `git diff --check` clean.
5. **Whole-harness check (pilot carry):** if any renamed law name is pinned by a
   test assertion (as NatArith's flat globals were), migrate the coupled
   assertion in the same branch and grep the whole test tree for every retired
   name — a green focused net is not enough; the workspace must be green.

## Do-not-reopen guardrails

- Convention is ruled + operator-concurred + piloted. Don't relitigate the axis;
  don't demote a subject-owned law for being reused.
- Interaction laws staying `lemma` is **correct**, not incomplete — do not force
  them onto a subject.
- Don't touch `fn`/`const`, the Ω-partition, or the `↦` glyph.
- Showcases are the Librarian's call — don't force-attach `…_wrong` decls.
