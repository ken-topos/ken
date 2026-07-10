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

### K2 · ⚠ PROMINENT (elaborator completeness fix) — DS-8b pure-witness ⊆ `proc`-field widening
- **Call:** **Land, this run,** a ~1-arm elaborator completeness fix — relax
  `check_instance_field_purity`'s `Proc if !impure` arm (`elab.rs:3182`) to
  **accept a pure/`∅` witness for a `proc` class field** (covariant subsumption
  `∅ ⊆ open row`). Scoped WP **DS-8b**, kicked to the **Ergo team**
  (`evt_7gfgqx5bbp48t`), Architect-gated. Unblocks DS-8's `instance Traversable
  List/Option`.
- **Trigger:** DS-8's `class Traversable` `proc traverse` field is inherently
  row-polymorphic (must be `proc`), but every lawful witness is genuinely pure
  (only calls DS-7's pure `ap`/`pure`), and the exact-match purity gate
  over-rejects it → the class had **no possible inhabitants** (a completeness bug,
  fail-closed). Architect ruling `evt_6vbgk65sj4jva`.
- **Options:** (a) **land the widening** [chosen] — honest (accepts the witness's
  true pure classification), minimal (~1 arm), safe-direction; (b) SURF-1 D1
  `visits [e]` surface binder — **rejected**: would force a *false* effect
  annotation on a pure fn, and is a deferred whole-surface-feature; (c) gate the
  `Traversable` instances — **rejected**: leaves the class uninhabitable, gutting
  the Core capstone.
- **Why (a):** the textbook **completeness-bug-fixed-in-the-safe-direction** shape
  — a valid program is fail-closed-rejected; the fix opens only `∅ ⊆ proc` and
  leaves the dangerous `impure ⊄ pure` direction (`Const|Fn if impure`, `:3189`)
  rejecting; the field stays `proc` (AC6 intact); **pre-kernel-erased, zero
  TCB/kernel/sort delta** — genuinely lighter than DS-5b. Matches SURF-1 §1.6
  do-not-optimize (a `proc` may instantiate to `∅`; a pure witness *is* that).
  I corrected my own initial lean toward D1 on the Architect's grounding.
- **Reversibility:** **moderate-class** (a landed elaborator completeness fix,
  pre-kernel-erased, zero TCB delta) — lighter than K1's hard-class; revert-clean;
  flagged for operator review. Gate net: AC8 dangerous-direction discriminator
  (effectful witness on a pure field still rejects, specific variant).

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

### RUN STATUS / resume point (2026-07-10, ~07:4x UTC)

**Live checkpoint for lossless resume across compaction.** **One build track
live** — **DS-8** (Foundation, assembling final `Traversable` instances → lands
whole, completes Core) — plus two enclave doc/conformance reconciles trailing
(the DS-8b `∅⊆proc` fast-follow **LANDED** PR #435; the **DS-5 §60 erratum**
in flight, spec-author + CV). **DS-5b LANDED** (PR #436) — both elaborator
capability builds (K1 hard-class + K2 moderate) are now on main, CI-green.
Kernel + Ergo rings freed. Next after DS-8: Data-section breadth.

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
  instances; SURF-1/SURF-2). **Building the unblocked core** (class decl w/
  `proc traverse`, `Compose`+`compose_*`+4 laws, List/Option `traverse` bodies,
  identity+naturality). DS-8b (the `proc`-field purity blocker) **now landed**, so
  Foundation is **assembling the final `instance Traversable List/Option`** on a
  rebase → DS-8 lands **whole**. Compose scope (`L3`): Architect ruled `Compose`
  buildable in-scope (`evt_37wddd1qb1pq9`), composition law rides this WP.
  Watch: the recurring dot-projection/`λ`-in-type Finding (DS-7 Finding 1).
- **DS-8b** (pure-witness ⊆ `proc`-field widening) — ✅ **LANDED**
  `main @ 5c698dd` (PR #433, CI-green). The `Proc if !impure` arm purely deleted
  (dangerous `Const|Fn` arm byte-identical), zero kernel/prelude/spec/conformance
  delta, Architect terminal gate (full keyword×impure matrix; AC6 via a *separate
  untouched* effect-escape mechanism `elab.rs:2581`; 110/110 purity suite). **K2**
  (moderate-class). Retros in. **Spec fast-follow** (the `∅⊆proc` rule in
  `36-effects.md`) released to spec-author, doc-only, trails this. Note:
  ergo-implementer's lane-discipline (flag-don't-self-author the spec gap) let CV
  catch a stale-conformance-fixture contradiction (CFP3) pre-ship — handled.
- **DS-5** (`Vector` spec chapter) — ✅ **LANDED** `main @ efdc09d` (PR #427,
  doc-only). Honest landed/gated split (head/`Fin` landed; tail/zip/lookup gated
  on DS-5b). Chapter `60-length-indexed-vectors.md`. Enclave stood down; CV has
  forward conformance work staged on the DS-5b gate. See `L1`/`L2` + `K1`.
- **DS-5b** (dependent-match index refinement) — ✅ **LANDED** `main @ 5058d72`
  (PR #436, CI-green). **First hard-class land of the run (`K1`).** Pure
  elaborator (zero kernel/data.rs/surface delta), 3 capabilities discharged via
  kernel `J`/`Cast` (zero `Axiom`, executable `trusted_base()` set-diff), kernel
  backstop intact (`var_refinements` elaborator-only + `kernel_check` re-run as
  arbiter), AC8 over-refinement → `KernelRejected`, 750-test suite green,
  non-indexed inert. **K1 addendum:** (a) a **3rd capability (goal refinement)**
  was an honest beyond-frame implementer finding, kept in the elab.rs layer +
  gated sound; (b) **`zip` two-vector step + `lookup` (Fin) did NOT land** —
  precisely diagnosed wall (convoy can't yet distinguish an outer param from a
  match-bound field), honestly deferred to a **named follow-on `DS-5c`** (§3.2.1).
- **DS-5 §60 erratum** — 🔨 IN FLIGHT (spec-author + CV, `evt_2s4q19y178hh5`).
  The merged `60-length-indexed-vectors.md §6` over-claims (says DS-5b unblocks
  all of tail/zip/lookup; it lands `tail`+single-convoy only) — Architect + CV
  both caught it. Fix: `tail` gated→landed; `zip`/`lookup` re-pointed to `DS-5c`.
  Bundled spec+conformance on one branch, lands right after (main-honesty), CI-
  gated. **Do not let it sit.**
- **`Vector` package** (Foundation follow-on) — **framable now** for the
  buildable API (`head` + `tail` + single-convoy ops), with `zip`/`lookup` gated
  on `DS-5c` — an honest partial package (DS-5-chapter split pattern). Queue
  behind DS-8; frame when Foundation frees.
- **`DS-5c`** (zip two-vector convoy + Fin-indexed `lookup`) — **named deferred
  WP**, NOT kicked this window (breadth over depth; would be a 3rd concurrent
  capability build). The §60 erratum + Vector package both point to it.
- **Data section (DS-3/DS-4/DS-6)** — next after Core, the breadth priority.
  DS-4 (List ext) near-mechanical → kick first; DS-3 (Either/Result/Option) +
  DS-6 (`DecEq Char` capstone, candidate 2nd kernel-move) are T1-design-needed.
- Verify team idle in reserve. Kernel + Ergo freed. Foundation assembling DS-8.
  spec-author/CV on the §60 erratum.

**Next-move triggers (event-driven):** DS-8 git_request → merge (CI-gated) →
**Core COMPLETE** → open Data section (kick DS-4 first, near-mechanical, to
Foundation). §60 erratum git_request → merge (CI-gated, spec+conformance).
Then Data breadth; Vector package + DS-5c are queued/named.

### P3 · Foundation is the catalog-authoring home; parallelize only independent tracks
- **Call:** Keep catalog authoring on the Foundation team (coherence — one
  author's hand across the tier); run genuinely-independent tracks in parallel
  where they don't contend (e.g. the DS-5 `Vector` spec chapter on the Spec
  enclave alongside a Foundation build).
- **Why:** `06`/program-doc home the catalog at Foundation; fragmenting
  authoring across idle teams would cost coherence for throughput.
- **Reversibility:** easy.
