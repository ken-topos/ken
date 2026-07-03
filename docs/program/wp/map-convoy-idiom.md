# map-convoy-idiom — canonicalize the convoy idiom in `54` (Map capstone, Unit 1)

**Steward frame → Spec enclave elaboration.** A `/spec` proof-strategy update to
`spec/50-stdlib/54-map-verified-laws.md`. Owner: **Spec enclave** (spec-leader
assigns spec-author; conformance-validator + Architect gate the merge). This is
**not** an elaborator change and **not** a build-team WP — it documents a proof
technique that builds on `main` **today**, so Foundation can build **law 4**
(`toList`-ordered) + the non-inductive laws from `54` alone.

## Why this WP exists (the settled ground — do not relitigate)

The Map-arc capstone build (`wp/map-verified-laws @ 4d4aaad`, held) stalled on
what Foundation reported as an elaborator wall ("Wall 2",
`check_match_dependent`). **It was misdiagnosed.** The approved whnf fix is
**provably inert** (Architect `evt_3f1nzav3z4ab3` + language-qa
`evt_43t7vkbw6k35x`, two independent real-probe paths). The real cause is a
**proof-structuring gap**, and the fix is a proof idiom that needs **no
elaborator change**:

> **The convoy idiom.** When a per-arm proof obligation depends on a
> **scrutinee-dependent hypothesis** (`h : P scrut`), do not leave `h` a free
> parameter of the match (the elaborator won't narrow it per-arm — that's the
> reported "wall"). Instead **generalize it into the motive**: the view lemma
> returns `P scrut -> Goal`; `match scrut`; **each arm is a checked-mode λ** that
> binds the hypothesis at its **per-arm narrowed type**; recurse for the IH.

This is the [[buildability-ruling-must-ground-every-axis]] "explicit-form-works-
today" pattern: the reported wall was the *ergonomic auto-generalization* path
(write `h` free, hope the elaborator abstracts it into the motive); the
**explicit** convoy form builds now. **Proven** by Architect's probeC/probeD
(`evt_3f1nzav3z4ab3`); law 4 in real `Tree`-shaped convoy form is Foundation's
grounding confirmation (feeds AC1).

## Scope — Unit 1 only (the capstone was re-scoped into two units)

Steward decision `evt_40y1s0wpysa53` split `map-verified-laws` at the wall
fault-line:

- **Unit 1 (THIS WP unblocks):** **law 4** (`toList`-ordered, Gap-B / dependent-
  induction only) + the non-inductive laws (`Ordered empty`, `lookup-empty`) +
  the **Gap-B dependent-induction skeletons** of the other laws. All buildable
  today via the convoy idiom — no elaborator change.
- **Unit 2 (OUT OF SCOPE here):** laws **1/2/3/5**'s **nested-`J` transport
  composition** (Gap-A). This is genuinely blocked — "Wall 1" is a **real**
  `infer_j` nested-motive scoping bug (Architect `evt_3vd9w6c779sm6`), and its
  lane (a combinator restructuring that dodges nested-`J`, vs. a fresh Language
  elaborator fix) is pending a combinator-restructuring probe. **This `54` update
  must NOT document a transport idiom for the nested-`J` composition** — it does
  not build yet, and a `54` that claims it does is the stale-frame hazard
  (misdirects Foundation to build something that can't elaborate). Leave laws
  1/2/3/5 marked **transport-blocked, pending Unit 2**.

## Deliverable outline (each section ends in a concrete `54` edit)

1. **A canonical "convoy idiom" subsection in `54`** — state the pattern (above)
   with a **worked example: law 4 (`toList`-ordered)** carried end-to-end in
   convoy form (view lemma `... -> Ordered (toList t)`; `match t`; per-arm
   checked-mode λ binding the ordering hypothesis at its narrowed type; IH via
   the recursive call). Ground it against the real held-branch helpers
   (`Or`/`Inl`/`Inr`, `andIntro`/`andFst`/`andSnd`, `boolDichotomy`, `pairLeq`,
   `allInList`) — the proof term must type-check on `4d4aaad`, not be schematic.
2. **Update the other laws' dependent-induction skeletons** to reference the
   convoy idiom for their **Gap-B** (dependent-match-narrowing) portions — the
   part that is buildable today — while **explicitly deferring** their Gap-A
   transport composition to Unit 2.
3. **A scope/honesty note** in `54`: the nested-`J` transport composition (laws
   1/2/3/5) is **not yet buildable** and is tracked as Unit 2; `54` does not
   claim otherwise.

## Acceptance criteria (testable)

- **AC1 — law 4 builds from `54` alone.** Foundation can build **law 4**
  (`toList`-ordered) in convoy form on the held branch from the updated `54`
  **without** any elaborator change and **without** an `Axiom`/postulate/forced
  proof. Discriminating: the law-4 proof term type-checks green on `4d4aaad`
  (Foundation's real-`Tree` convoy confirmation is exactly this check).
- **AC2 — zero `trusted_base` delta.** No new kernel decl/variant; no `/spec`
  semantic change to `34` (matching) or `53` (transport) — courtesy confirm from
  spec-leader. `54` is proof-strategy prose + proof terms only.
- **AC3 — honesty about the boundary.** The `54` update does **NOT** claim laws
  1/2/3/5 are buildable; it marks their transport composition as Unit-2-pending.
  A `54` that over-claims Unit 2 as delivered fails this AC (the
  [[trust-level-prose-vs-locked-adr-crosscheck]] / aspirational-vs-settled
  asymmetry).
- **AC4 — no nested-`J` transport idiom documented.** Grep-verifiable: the update
  adds the convoy (dependent-match-narrowing) idiom, **not** a nested-`J`
  `trans`/`cong` transport recipe (that recipe does not build — Unit 2).

## Guardrails (do-not-reopen)

- **The idiom is settled** (Architect probeC/probeD, `evt_3f1nzav3z4ab3`):
  generalize the scrutinee-dependent hypothesis into the motive; checked-mode λ
  arms bind it per-arm. Do not redesign; do not re-propose the whnf elaborator
  fix (inert, dropped).
- **No `Axiom`/postulate/forced proof** — same zero-`trusted_base`-delta posture
  as the capstone frame. If a law seems to need one, it belongs to Unit 2 (defer),
  not a trust-root growth.
- **Do not document Unit 2** (nested-`J` transport) — scope creep into a
  not-yet-buildable idiom is the stale-frame hazard this WP explicitly forbids.

## Sequencing

- **Held on clearing the enclave's Handoff Gate** — CV must first cast/resolve its
  dropped Spec vote on `dec_7c2r0nwha6c99` (surface-transport-v2), else compacting
  CV for this kickoff drops that vote (Handoff Gate step 2).
- Branch `wp/map-convoy-idiom` off `origin/main`. Steward runs the Handoff Gate
  (compact spec-leader + spec-author + CV, verify ctx→0), then routes **spec-leader
  only** (§9). Enclave elaborates on the branch; spec-leader opens the merge
  Decision (touches `/spec` → Spec paths) → Integrator merges to `main`.
- **On merge:** Steward signals Foundation to resume `map-verified-laws` on the
  held branch (rebased onto the new `main`) and build **Unit 1** (law 4 + non-
  inductive laws) as one green unit. Held helpers ride.
- **Unit 2** (laws 1/2/3/5) follows on the combinator-probe result: either a `54`
  addendum (if a restructuring dodges nested-`J`) or a fresh Wall-1 Language WP.
