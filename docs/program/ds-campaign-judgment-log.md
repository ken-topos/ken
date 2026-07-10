# DS-campaign judgment log (autonomous run, 2026-07-10)

**Operator away ~02:30→11:30 UTC 2026-07-10.** Steward driving the catalog
data-structures program (`wp/catalog-data-structures-program.md`) autonomously,
with **`docs/PRINCIPLES.md` as the operator's stand-in** for any design fork the
spec does not settle. This log records **judgment calls that affect the language
surface, elaboration, or functionality** (the operator's explicit ask), plus a
separate section for process/sequencing calls. Each entry: the call, the
options, the deciding principle(s), and reversibility.

Legend — **Reversibility:** `easy` (doc/outer-ring, revert-clean) ·
`moderate` (a landed catalog API/name, mig­ratable) · `hard` (kernel/TCB/surface
grammar — flagged for operator review, NOT landed unattended per the boundary
rule below).

---

## Boundary rules for this run (operator-ruled 2026-07-10)

**Operator ruling:** *"You may fix kernel issues. I'll review those at the end.
This language has no users (other than us), so a change to the kernel that we
revert a few hours later has zero impact."* So the irreversibility ceiling I
proposed (stage-and-hold TCB changes) is **lifted** — the no-users reality makes
reverts cheap, which lowers the *irreversibility* bar, **not** the *correctness*
bar.

- **Kernel / TCB fixes are PERMITTED and may LAND**, routed through the full
  Kernel-team ring + Architect soundness gate + conformance, then **logged
  prominently here** for the operator's end-of-run review (candidate: DS-6
  `DecEq Char` / `Int`-lawfulness spike). Not staged/held.
- **Soundness is still non-negotiable.** The gate does not relax: a kernel change
  must pass conv/reduction/termination rigor and conformance. Cheap-revert
  latitude ≠ license to land an unsound change (an unsound `proved` is *wrong*,
  not merely revertable).
- **PRINCIPLES rigor still applies.** Prefer reflect-don't-extend (#6) and the
  outer ring; grow the TCB only when it is the *right* fix at the right layer
  (#5, #13), never an expedient (#4). A kernel change I'd land is one I'd defend
  on intrinsic merits, not one that's merely easy because reverts are cheap.
- **Surface-grammar changes** (keywords, fence roles, parser forms) — same
  regime: land when they're the right fix (subsume-don't-proliferate first, #7),
  logged prominently.
- **Outer-ring functionality/elaboration calls** (instance shapes, lemma
  phrasings, catalog API names, combinator sets) — enclave-ruled with PRINCIPLES
  as tiebreaker, landed through the normal ring+gate, logged here.

---

## Language-surface / elaboration / functionality calls

### L1 · DS-7 package home + basename — new `Core/EffectfulClasses.ken.md`
- **Call:** Home the CAT-2 effectful-class family (`Applicative`+`Monad` now,
  `Traversable` appended at DS-8) in **one new entry
  `catalog/packages/Core/EffectfulClasses.ken.md`**, alongside
  `LawfulFunctors.ken`/`LawfulClasses.ken` in `Core`, reusing the landed
  `class Functor`.
- **Options:** (a) new `Core/EffectfulClasses.ken.md` [chosen, suggested]; (b)
  append to the existing `LawfulFunctors.ken`; (c) the chapter-56 build-note's
  `catalog/packages/lawful-functors/` path.
- **Why:** subsume-don't-proliferate keeps the effectful classes together in one
  entry (they share the wired-superclass story); `Core` is where the
  class/instance vocabulary lives; a fresh entry mirrors DS-2's one-entry-per-WP
  shape and keeps `LawfulFunctors` stable. The chapter's `lawful-functors/` path
  is a perishable build-note; `P3` homes authoring at Foundation and I let the
  final basename be the build's call (frame states it as a suggestion, not a
  pin — flagged for the handback).
- **Reversibility:** easy (an unreferenced new catalog entry; renamable before
  any cross-package consumer exists — there is none).

### L2 · DS-7 framed straight to Foundation citing chapter 56 (no enclave finalize pass)
- **Call:** Frame DS-7 directly to Foundation with
  `spec/50-stdlib/56-effectful-classes.md` (CAT-2) as the build contract, rather
  than routing the DRAFT-v0 chapter through an enclave finalize-to-build-contract
  pass first. Architect fidelity-gates the build against the chapter at the gate.
- **Why:** the chapter is already a build contract — exact class signatures
  (§3.1/§4.1), the laws stated character-for-character (§3.2/§4.2), instance
  definitions (§3.3/§4.4), the ITree-bridge disposition (§4.3), and explicit
  acceptance criteria (§7). An enclave finalize pass would add latency without
  adding contract; the Architect gate already re-certs build-vs-chapter fidelity
  + soundness. Mirrors how DS-2 was framed off its own committed frame.
- **Reversibility:** easy (routing choice; escalate to the enclave mid-build if a
  chapter ambiguity surfaces).

---

## Process / sequencing calls

### P1 · Sequence: DS-2 → DS-7 → DS-8 → (Data) DS-3 → DS-4 → DS-6; DS-5 spec-track in parallel
- **Call:** Drive DS-2 (`Ord Nat` export) first, then the remaining Core toolkit
  (DS-7 `Applicative`/`Monad`, DS-8 `Traversable`), then the Data Section
  (DS-3 `Either`/`Result`/`Option`, DS-4 `List`, DS-6 `DecEq Char` capstone).
  DS-5 (`Vector`) is spec-gated → kick its `spec/50-stdlib/` chapter to the Spec
  enclave in parallel so the package can follow once the chapter lands.
- **Why:** matches the operator's "start on DS-2, move through Core, then Data";
  respects the dependency graph (DS-8←DS-7; DS-9 driver last); DS-2 is the
  smallest/most-mechanical, a good warm-up. `catalog-data-structures-program.md`.
- **Reversibility:** easy (re-sequenceable any time).

### P2 · Functional-build quality first, favor breadth
- **Call:** Land functional builds (proofs real, trusted-base honest) across
  Core then Data; refinement-to-guide-quality is a follow-on track, not a
  blocker — to "get as far as you can."
- **Why:** the `06` two-phase cadence explicitly permits functional-first;
  breadth over the tier is the operator's stated goal for the window.
- **Reversibility:** easy.

### RUN STATUS / resume point (2026-07-10, ~05:xx UTC)

**Live checkpoint for lossless resume across compaction.**

- **DS-2** (`Ord Nat` export) — ✅ **LANDED** `origin/main @ 971aaad` (PR #421,
  CI-green). Two added files, outer-ring, zero-`Axiom`/zero-`trusted_base()`
  delta, Architect terminal APPROVE. Foundation stood down, retros in.
- **DS-7** (`Applicative`/`Monad`) — ✅ **FRAMED + KICKED to Foundation**
  (`evt_2f39kxxfnbtjr`). Frame `docs/program/wp/ds-7-applicative-monad.md`
  (`origin/main @ b12123a`); design contract `spec/50-stdlib/56-effectful-classes.md`
  (CAT-2). Scope = D1 `Applicative` + D2 `Monad` + List/Option instances + the
  ITree bridge as an **attested correspondence** (no minted second `bind`, no
  surface `instance Monad (ITree e resp)`). Building; comes back through
  foundation-qa → Architect (fidelity vs chapter + zero-`Axiom` gate) →
  git_request to Steward. See `L1`/`L2` for the home + routing judgment calls.
- **DS-8** (`Traversable`) — **GATED** on SURF-1 row-var surface (landed
  `main@ef791a3`) **+ SURF-2 class-field purity marker** (`33 §5.2`, `39 §6.0`,
  chapter §5.1) — **verify SURF-2 landed before sequencing DS-8** (still to check).
- **DS-5** (`Vector` spec chapter) — independent parallel Spec-enclave track,
  queued (kick to spec-leader to keep Spec productive alongside the Foundation
  build).
- **Data section (DS-3/DS-4/DS-6)** — queued after Core; DS-6 (`DecEq Char`
  capstone) is the candidate kernel-move (boundary rules permit landing).
- Kernel + Verify teams restarted by operator, re-oriented, idle in reserve for
  any DS-6 kernel move.

### P3 · Foundation is the catalog-authoring home; parallelize only independent tracks
- **Call:** Keep catalog authoring on the Foundation team (coherence — one
  author's hand across the tier); run genuinely-independent tracks in parallel
  where they don't contend (e.g. the DS-5 `Vector` spec chapter on the Spec
  enclave alongside a Foundation build).
- **Why:** `06`/program-doc home the catalog at Foundation; fragmenting
  authoring across idle teams would cost coherence for throughput.
- **Reversibility:** easy.
