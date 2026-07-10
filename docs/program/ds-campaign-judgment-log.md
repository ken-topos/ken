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

_(none yet — appended as they arise)_

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

### RUN STATUS / resume point (2026-07-10, ~04:xx UTC)

**Live checkpoint for lossless resume across compaction.**

- **DS-2** (`Ord Nat` export) — **KICKED to Foundation** (`evt_2axex7z2s0m77`),
  frame `wp/ds-2-ord-nat-export.md`. Building; comes back through
  foundation-qa → Architect (zero-new-`Axiom` gate) → git_request to Steward.
- **DS-7** (`Applicative`/`Monad`) — **GROUNDED, next to frame+kick** (do this
  after the pending compaction). Design is NOT just convo-recorded — it is a
  full spec chapter **`spec/50-stdlib/56-effectful-classes.md` (DRAFT v0, CAT-2)**:
  WIRE the superclass chain (`Applicative f` carries field `functor : Functor f`;
  `Monad f` carries `applicative : Applicative f`); class sigs at §3.1 (Applicative:
  `functor`,`pure`,`ap` + 4 laws `ap_id`/`ap_hom`/`ap_ich`/`ap_cmp` + `map_coh`)
  and §4.1 (`Monad` bind-primary, Fork B); instances `List` (cartesian `ap`,
  §3.3/§4.4) + `Option`, both **proved zero-delta** by induction; wired superclass
  dict supplied whole (six Functor+Applicative proofs NOT re-proved). Explicit
  wiring only — implicit-superclass-coercion sugar deferred (`OQ-syntax`), no new
  elaborator capability. **Unblocked** (only DS-8 Traversable is gated). Landed
  dep: `class Functor` (`LawfulFunctors.ken:188`). No Applicative/Monad package
  yet. **Open before kick:** decide whether 56-ch DRAFT v0 needs an enclave
  finalize-to-build-contract pass, or frame DS-7 straight to Foundation citing
  the chapter (lean: it's detailed enough — frame to Foundation, Architect
  fidelity-gates the build vs the chapter).
- **DS-8** (`Traversable`) — **GATED** on SURF-1 row-var surface (landed
  `main@ef791a3`) **+ SURF-2 class-field purity marker** (`33 §5.2`, `39 §6.0`)
  — verify SURF-2 landed before sequencing DS-8.
- **DS-5** (`Vector` spec chapter) — independent parallel Spec-enclave track,
  queued (kick when convenient to keep Spec productive).
- Kernel + Verify teams **freshly restarted** by operator (re-orienting) — in
  reserve for any DS-6 kernel move.

### P3 · Foundation is the catalog-authoring home; parallelize only independent tracks
- **Call:** Keep catalog authoring on the Foundation team (coherence — one
  author's hand across the tier); run genuinely-independent tracks in parallel
  where they don't contend (e.g. the DS-5 `Vector` spec chapter on the Spec
  enclave alongside a Foundation build).
- **Why:** `06`/program-doc home the catalog at Foundation; fragmenting
  authoring across idle teams would cost coherence for throughput.
- **Reversibility:** easy.
