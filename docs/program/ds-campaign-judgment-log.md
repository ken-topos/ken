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

### K1 · ⚠ PROMINENT (elaborator-capability land) — DS-5b dependent-match index refinement
- **Call:** **Land, this run,** a new **elaborator capability** — dependent-`match`
  index refinement (constructor **injectivity** for peeled recursive fields +
  sibling-binder **convoy**) — in `crates/ken-elaborator/src/elab.rs`'s
  dependent-match path (near `method_index_premises`/`synthesize_omitted_index_method`).
  Kicked to the **Kernel team** through the full ring + Architect soundness gate
  (frame `wp/ds-5b-dependent-match-refinement.md`, `evt_1q7m4wjk4cy8a`).
- **Trigger:** the DS-5 Architect ruling (`evt_1mnh5sngvhaty`) ground-truthed
  that `Vec`/`vnil`/`vcons`/`head`/`Fin` build+test **today**, but
  `tail`/`zip`/`lookup` are gated on exactly this one missing capability
  (dependent-match recovers the motive over the scrutinee's own index only; it
  does not re-type peeled fields by injectivity, nor refine sibling binders).
- **Options:** (a) **land the enhancement this run** [chosen] — kick to Kernel,
  full ring + soundness gate, in parallel with the DS-5 chapter; (b) ship DS-5
  head-only-buildable, spec `tail`/`zip`/`lookup` as gated, defer the enhancement
  to a named later WP.
- **Why (a):** it is the **right fix at the right layer** (#5/#13 — `elab.rs`, not
  kernel, not `data.rs`; the Architect located it precisely and the kernel already
  admits the family + dependent `Elim`); it is a **general** dependent-match
  capability (subsume-don't-proliferate #7 — unblocks *every* future indexed-family
  match, not a Vector-only bolt-on); the **kernel re-check stays the fail-closed
  backstop** so a wrong enhancement rejects rather than admitting unsoundness, and
  the full Kernel ring + Architect gate supply the soundness supervision the run's
  boundary rules require. The operator lifted the TCB ceiling for exactly this and
  asked for breadth; the Kernel team was idle (zero contention with DS-7/DS-5-ch).
  Soundness stays non-negotiable — the frame's bars: injectivity **discharged via
  kernel no-confusion, never postulated** (zero-`Axiom`/zero-`trusted_base()`-delta),
  an **over-refinement discriminator** (over-refinement is the unsoundness vector),
  **full-suite-green + non-indexed-match inertness** (protects in-flight DS-7).
- **Reversibility:** **hard-class** (a soundness-adjacent elaborator capability) —
  revert-clean per the no-users reality, but **flagged for the operator's review**
  regardless. This is the first hard-class land of the run. If the operator would
  rather it had waited for review, it reverts cleanly and DS-5 still ships
  head-only with the rest spec'd-and-gated.

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

### L3 · DS-8 composition-law scope — build `Compose` in-scope vs gate the law
- **Call:** Frame DS-8 to land the Traversable **core unconditionally** (both
  classes, both instances, `sequence`, identity + naturality laws), and decide
  the **composition** law by a build-probe: if the `Compose g h` applicative it
  is stated over builds cleanly in-scope (a small derived `instance Applicative
  (Compose g h)`), build it and prove composition; else ship composition **gated
  on a named `Compose`-applicative follow-up** with an honest landed/gated split,
  Architect ruling the boundary at the gate. **Not pre-deciding build-vs-gate** —
  the frame routes it to a probe + the Architect, not my unilateral call.
- **Why:** `Compose g h` is grep-confirmed **not landed**, and §5.3 flags it
  "CAT-2/CAT-3 derived" — so whether it builds today is a real buildability axis
  (ground every axis, don't assume). Landing the core regardless honors "get as
  far as you can"; gating the one law honestly (DS-5 pattern) beats either
  forcing an unproven law or blocking the whole WP. Subsume-don't-proliferate
  prefers building `Compose` once as a reusable derived instance if clean.
- **Reversibility:** easy (a scope decision; the core lands either way, the
  composition boundary is a doc/gate note).

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

### RUN STATUS / resume point (2026-07-10, ~06:1x UTC)

**Live checkpoint for lossless resume across compaction.** Core toolkit is
landed-or-in-flight; two tracks live: **DS-8** (Foundation), **DS-5b** (Kernel).

- **DS-2** (`Ord Nat`) — ✅ **LANDED** `main @ 971aaad` (PR #421). Retros in.
- **DS-7** (`Applicative`/`Monad`) — ✅ **LANDED** `main @ 88dce79` (PR #428,
  CI-green). 2 added files, outer-ring, zero-`Axiom`/zero-`trusted_base()`-delta,
  Architect dual gate (fidelity vs chapter 56 char-for-char + soundness). WIRE
  chain consistent; ITree bridge prose-only (no 2nd `bind`). 3 Ergo Findings
  (dot-projection/`λ` in type position; `concatMap` inlined; arg-order). Retros in.
  Entry `Core/EffectfulClasses.ken.md`.
- **DS-8** (`Traversable`) — ✅ **FRAMED + KICKED to Foundation**
  (`evt_368s003ta85w3`); frame `wp/ds-8-traversable.md` (`main @ 229dcea`), design
  contract chapter 56 §5. **The last Core item.** Appends to
  `Core/EffectfulClasses.ken.md`. Prereqs all landed (DS-7; Functor/Foldable
  instances; SURF-1/SURF-2). **Open scope decision (see L3):** §5.3's composition
  law needs a `Compose g h` applicative that is NOT landed — frame scopes it as
  build-core-unconditionally (identity+naturality) then probe building `Compose`
  in-scope, else ship composition gated on a named follow-up; Architect rules the
  boundary at the gate. Building.
- **DS-5** (`Vector` spec chapter) — ✅ **LANDED** `main @ efdc09d` (PR #427,
  doc-only). Honest landed/gated split (head/`Fin` landed; tail/zip/lookup gated
  on DS-5b). Chapter `60-length-indexed-vectors.md`. Enclave stood down; CV has
  forward conformance work staged on the DS-5b gate. See `L1`/`L2` + `K1`.
- **DS-5b** (dependent-match index refinement) — 🔨 **BUILDING on Kernel**
  (`evt_1q7m4wjk4cy8a`); frame `wp/ds-5b-dependent-match-refinement.md`
  (`main @ c958b66`). Elaborator enhancement unblocking DS-5's tail/zip/lookup.
  **First hard-class land of the run — see `K1`.** Comes back through kernel-qa →
  Architect soundness gate → git_request. Soundness bars: injectivity
  discharged-not-postulated, over-refinement discriminator, full-suite-green +
  non-indexed-match inertness.
- **`Vector` package** (Foundation follow-on) — **queued behind DS-5b landing**
  (frame it for the full API once tail/zip/lookup elaborate; head-only is
  buildable today but ship the package whole).
- **Data section (DS-3/DS-4/DS-6)** — next after Core. DS-4 (List ext) near-
  mechanical; DS-3 (Either/Result/Option) + DS-6 (`DecEq Char` capstone,
  candidate 2nd kernel-move) are T1-design-needed.
- Verify team idle in reserve. Kernel on DS-5b; Foundation on DS-8; Spec/CV freed.

### P3 · Foundation is the catalog-authoring home; parallelize only independent tracks
- **Call:** Keep catalog authoring on the Foundation team (coherence — one
  author's hand across the tier); run genuinely-independent tracks in parallel
  where they don't contend (e.g. the DS-5 `Vector` spec chapter on the Spec
  enclave alongside a Foundation build).
- **Why:** `06`/program-doc home the catalog at Foundation; fragmenting
  authoring across idle teams would cost coherence for throughput.
- **Reversibility:** easy.
