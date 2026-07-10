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
  dependent-match path (near `method_index_premises`/
  `synthesize_omitted_index_method`). Kicked to the **Kernel team**
  through the full ring + Architect soundness gate (frame
  `wp/ds-5b-dependent-match-refinement.md`, `evt_1q7m4wjk4cy8a`).
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

### L4 · DS-3 `Either` ruling — SUBSUME [⚠ SUPERSEDED on the neutral-coproduct point by L5 — operator ruling]
> **SUPERSEDED (2026-07-10, operator):** L4 subsumed `Either` into `Result` on a
> structural-isomorphism argument. On the **humans-read** axis that was imprecise
> — `Either`/`Left`/`Right` (neutral disjunction) and `Result`/`Ok`/`Err`
> (fallible computation) carry **different reader semantics**; `Either` never
> duplicated `Result`, it duplicates the *neutral coproduct slot* (`Sum`). The
> operator reopened it and ruled the neutral coproduct should be spelled `Either`,
> not `Sum` — see **L5**. What L4 got right and still stands: `Result` stays a
> distinct named error type, and there is only ONE neutral coproduct (now renamed,
> not a third spelling). What flips: a first-party `Either` DOES exist (as the
> renamed `Sum`); the erratum note re-annotates `Sum a b` → `Either a b`.
- **Call:** Catalog carries **no distinct `Either`** — `Result e a = Err e | Ok a`
  (prelude-declared, load-bearing) **subsumes** it. Steward recommendation;
  **@architect CONFIRMED** on the design axis (he owns shape). Framed in
  `wp/ds-3-sum-type-combinators.md`.
- **Why:** `Either e a = Left e | Right a` is **structurally isomorphic** to
  `Result` (constructor bijection, identical eliminator shape) — an isomorphic
  twin adding **zero capability** = the proliferation #7 forbids. `Result` is
  wired into the effect layer (`fs_resp`, `prelude.rs:1077`) + codec error path;
  `Either` has zero declaration/user. Trust levels identical → coexist-when-trust-
  differs does **not** apply. **Architect grounding that seals it:** the trust
  root **already** carries a general parametric coproduct `Sum a b = InL a | InR b`
  (`prelude.rs:157`, load-bearing for ITree `⊕`), so *both* of Either's roles —
  error sum (`Result`) and neutral sum (`Sum`) — already have first-class homes;
  `Either` would be a **third** spelling of a twice-existing shape.
- **Spec reconcile — FOUR normative sites (Architect's whole-surface sweep, not
  my cited one):** subsume makes false every "`Either` is a declared type" claim.
  (1) `50-stdlib/README.md:42`; (2) `30-surface/34-data-match.md:5`; (3)
  `34-data-match.md:56` ("Result, Option, Either are ordinary prelude data decls"
  — **load-bearing false**); (4) `34-data-match.md:633` ("Result/Option/Either in
  the prelude" — **load-bearing false**). **Exclude** `:540` ("Either way", English);
  `_notes/` non-normative. **Erratum LANDED** `main @ dcc34ed` (PR #446, doc-only;
  routed `evt_1qkfgg6p8dkam`, spec-author + CV, DS-5 §60 pattern) — annotated
  `Either` "subsumed by `Result`;
  no distinct type — neutral sum is `Result` or the `Sum a b` coproduct" at all
  four sites. Correcting only one leaves the two `34-data-match` claims false
  (correcting-scope-must-sweep-whole-doc — validated: the over-claim WAS restated).
- **Coupled package-home call:** recommend one entry
  `catalog/packages/Data/Sums/Sums.ken` for both L2-sum combinator families
  (Option + Result), not two — subsume-don't-proliferate on package count.
- **Named deferred (NOT this window):** whether to bless `Sum a b` as the
  *user-facing* neutral coproduct (it's presently effect-framed only) is a
  separate non-DS-3 question the Architect flagged — logged, not acted on.
- **Reversibility:** easy (if ever reversed, a distinct `Either` is a small
  additive `data` + combinators, not a rework). The DS-3 combinator build (lane a)
  is independent and proceeds regardless.

### L5 · ⚠ PROMINENT — coproduct family: `Either` (catalog package), rename effect `Sum`→`Coproduct` [OPERATOR-RULED]
- **Call (COEXIST by role — three distinct coproducts; SPLIT into two WPs after
  the placement ruling):** (1) **`Either a b = Left a | Right b`** — the
  user-facing value disjunction, defined as a **user-level CATALOG PACKAGE, NOT
  the prelude** (operator arm 3, below) → **Foundation** WP
  `wp/either-catalog-package.md`; (2) **RENAME the internal effect coproduct
  `Sum`→`Coproduct`** (type name only, **keep `InL`/`InR`** — `eval.rs` peel
  untouched) → **Runtime** WP `wp/either-neutral-coproduct.md`, which reworks to
  **rename-only** (drop the prelude `Either` decl + the `34-data-match` Either
  reconcile — those move to the Foundation WP); (3) **RESERVE the freed `Sum`** for
  a future `Data.Functor.Sum`. `Result` stays a distinct named error type. Both
  WPs Architect-gated + Spec-vote, CI-gated.
- **Placement (operator, arm 3):** Pat asked whether `Either` needs to be built
  in; I answered no (ordinary non-dependent sum; nothing depends on it) AND that
  the spec's OWN model (`50-stdlib/README.md:42`) says core data are **packages,
  not prelude** — the impl puts Option/Result in the prelude only as a bootstrap
  shortcut (a spec-vs-impl gap). Runtime's first build had added `Either` to the
  prelude (following the shortcut); I **held that merge** (`ee168a3`) pending this
  ruling. Pat: **(B) user-level `Either` as a catalog package.** So `Either` is the
  first core sum done per the stated model; the prelude→packages migration of its
  siblings is a **named future** (see below).
- **Decider:** **the operator (Pat), directly** — two-step. First: Pat asked if
  `Either` differs semantically from `Result`; I answered yes (Rust/F#/Elm keep a
  named `Result` distinct from a neutral `Either`/`Choice`) → *"Reopen, prefer
  Either to Sum."* Then Pat probed whether the *internal* `Sum` is the same as
  `Either` ("Either means one or the other, not both") → I clarified they're
  structurally identical in Ken (both Type0) but different-role; Pat ruled
  **COEXIST** (don't unify), and chose the effect coproduct be renamed to the
  precise term **`Coproduct`** (freeing `Sum` for the real `Data.Functor.Sum`).
- **Why:** **humans-read** — `Either`/`Left`/`Right` is the value-disjunction
  idiom; `Sum` reads as *addition* outside a narrow CT audience AND squats on the
  name the real functor coproduct wants (Ken's effect `Sum` is a Type0 *value*
  coproduct, NOT the higher-kinded `Data.Functor.Sum`). Distinct reader-roles earn
  distinct names — same principle as `Result` vs `Either` (coexist, not subsume,
  because roles differ; #7 is not violated — three *different-role* types, no
  redundant spelling). L4's structural-isomorphism argument under-weighted this.
- **Scope/boundary:** add `data Either` to prelude (surface `data`, mirrors
  `Result`); rename `Sum`→`Coproduct` in `effects::state::declare_sum` + prelude
  globals + `injectL/R` + `resp_sum`(→`resp_coproduct`?) + tests + `36-effects.md`;
  `eval.rs` peel = **comment-only** (InL/InR kept). **The landed L4 subsume erratum
  (PR #446) is SUPERSEDED** — spec touch must REWRITE the `34-data-match` note
  (Either = distinct declared coproduct; Result = distinct error sum) + RESTORE
  `Either` at the three list-sites, NOT find-replace (Architect catch
  `evt_60ahxgw3vpnqn`). **Zero kernel-crate delta**; no alias (no users); no name
  collision. Open sub-question (Architect): `Coproduct` hand-built vs surface
  `data`; `resp_sum` rename.
- **Reversibility:** **moderate-class** (a trust-root prelude declared-type
  add+rename, pure/semantics-preserving, zero kernel-crate delta, revert-clean) —
  PROMINENT for operator review. Not soundness-adjacent.
- **Downstream (named follow-ons, NOT the two L5 WPs):** DS-3 (Option/Result
  combinators, in flight) unaffected; `Either` type + combinators are the
  Foundation catalog WP itself (`wp/either-catalog-package.md`). **Core-data →
  packages migration:** `Option`/`Result`/`Nat`/`List`/`Prod`/`Unit` are
  prelude-declared but the spec models them as packages — a standing spec-vs-impl
  gap; aligning them is a **separate architectural WP** (operator sets direction);
  `Either`-as-package is the first correct precedent. **`Data.Functor.Sum f g`** →
  a functor-combinator WP alongside `Compose`, owning the freed `Sum` name.

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

### RUN STATUS / resume point (2026-07-10, ~10:50 UTC)

**Live checkpoint for lossless resume across compaction.** **CORE COMPLETE** —
the constructor-class chain Functor → Applicative → Monad → Traversable is fully
landed (**DS-8 Core merged, PR #440**). §60 erratum LANDED (PR #438); both
capability builds (K1 DS-5b + K2 DS-8b) landed CI-green. **Data section OPEN and
progressing: DS-4 (List combinators) LANDED** (PR #443, `main @ ab64104`, first
Data item); **DS-3 (Option/Result combinators + the `Either` ruling, `L4`) KICKED
to Foundation** (`evt_zpdcdwv8zkvr`, second Data item — build in flight). All
rings idle except Foundation (DS-3 build). Next after DS-3: frame DS-6 (`DecEq
Char` capstone, T1-design + candidate 2nd kernel-move, not yet framed) if
window-time remains.

**DS-8 — VALVE TAKEN (composition law deferred to DS-8c for SIZE):** the
`traverse` composition coherence law (§5.3) turned out ~40-60 lemmas (not ~12-15)
— converging, **nothing walling, a SIZE trigger not a capability wall**. Per the
Steward valve (Architect defers timing to me; blessed the shape), **DS-8 Core
ships now**: `class Traversable` + `List`/`Option` instances + identity +
naturality laws (proved) + `Compose` applicative (3/4 laws: `ap_id`/`ap_hom`/
`ap_ich` + `map_coh` + Functor laws) + `ap_naturality` aux + `ap_cmp` LHS
reductions (partial, honestly marked). **`ap_cmp` (Compose's 4th law) + the
traverse composition law both deferred to DS-8c.** Foundation transcribing into
`Core/EffectfulClasses.ken.md` now → foundation-qa → Architect gate → git_request.
  - **Architect's 5 honesty pins bind DS-8 Core's entry + his gate**
    (`evt_7an7q5pbztdr0`): (1) deferral is **SIZE not capability** — say
    buildable-now/deferred-for-size, NOT "gated/capability-blocked" (unlike DS-5c
    which IS capability-blocked); (2) TWO things deferred (`ap_cmp` = 1 of
    Compose's 4 Applicative laws + the traverse composition law that consumes it)
    — scope both; (3) scope the "lawful" claims to laws actually proved (identity
    + naturality, NOT "fully lawful"); (4) **no `Axiom`/`Refl`-papering** on the
    partial `ap_cmp` — he greps the tangled code at the gate; (5) DS-8c spec =
    the implementer's concrete 4-stage closing plan.
  - **DS-8 Core git_request — DONE.** Arrived `wp/ds-8-traversable @ ee497ba`
    (foundation-qa APPROVE ×2 — caught + fixed a transcription gap and an
    ill-typed `instance Functor Compose`; Architect 3-role gate APPROVE
    `evt_74t0z7jprmww0`, incl. an empirical-probe correction that the dropped
    `instance Functor (Compose g h)` fails on **parametric-instance-head
    KINDING** — free `g`/`h` default-kinded `Type` vs needed `Type→Type` — NOT
    the §6.1/ITree `UnresolvedCon` wall). **Steward honesty gate passed
    independently:** 2 files only (entry +1112, acceptance +221), zero
    kernel/Cargo/elaborator-src delta, no Axiom/postulate emission in any code
    fence, conflict-free, all 5 pins present. Merged PR #440, `main @ 709c55d`.
    **CORE COMPLETE.**
- **DS-8c** (traverse composition coherence law) — **NAMED deferred WP, NOT
  kicked this window** (breadth over depth, like DS-5c). Distinct from DS-5c:
  DS-8c is **SIZE-deferred, buildable-now** (~40-60 lemmas, zero missing
  capability); spec = the implementer's 4-stage plan (rewrite ψ5 pointwise via
  `aph.map_coh` → triple-pointwise `aph.ap_cmp` via `eq_at_pi` → lift through the
  3 nested `apg` apps → reconcile vs the free RHS).
- **Pipeline-stall check (operator asked ~09:30):** NOT stalled. DS-8 implementer
  was silent 07:24→09:29 (~2h) on the sole live track — a **coordination lapse
  (no progress ping), not a hang**; responded promptly to foundation-leader's
  direct ping with real progress. Lesson saved (anchor silence-duration on the
  real clock, not last-event-seen — I under-reported 2h as 40min first).

- **DS-2** (`Ord Nat`) — ✅ **LANDED** `main @ 971aaad` (PR #421). Retros in.
- **DS-7** (`Applicative`/`Monad`) — ✅ **LANDED** `main @ 88dce79` (PR #428,
  CI-green). 2 added files, outer-ring, zero-`Axiom`/zero-`trusted_base()`-delta,
  Architect dual gate (fidelity vs chapter 56 char-for-char + soundness). WIRE
  chain consistent; ITree bridge prose-only (no 2nd `bind`). 3 Ergo Findings
  (dot-projection/`λ` in type position; `concatMap` inlined; arg-order). Retros in.
  Entry `Core/EffectfulClasses.ken.md`.
- **DS-8 Core** (`Traversable`) — ✅ **LANDED** `main @ 709c55d` (PR #440,
  CI-green). **Completes Core** (Functor→Applicative→Monad→Traversable). Entry
  `Core/EffectfulClasses.ken.md §9`, design contract chapter 56 §5. **VALVE-SPLIT**
  (not whole): `class Traversable` + `List`/`Option` instances (identity +
  naturality proved) + `Compose g h` (`fn`-synonym) Functor instance + 3/4
  Applicative laws (`ap_id`/`ap_hom`/`ap_ich` + `map_coh`) + `ap_naturality` aux
  + `ap_cmp` LHS reductions (partial, honestly marked). Outer-ring only, zero
  `Axiom`, zero-`trusted_base` delta, Architect 3-role gate + foundation-qa ×2.
  **Composition law + Compose's own `ap_cmp` deferred to DS-8c** (SIZE, not
  capability). See the VALVE section above for the 5 honesty pins + gate detail.
  `L3` (Compose in-scope) held. Retros in.
- **DS-4** (`List` combinator completion) — ✅ **FRAMED + KICKED to Foundation**
  frame `wp/ds-4-list-combinators.md`. **LANDED** `main @ ab64104` (PR #443,
  CI-green). Near-mechanical: `reverse` (+ involutive law via `reverse_snoc` —
  the one real induction), `zip` (non-dependent — verified NOT the DS-5c-gated
  Vector zip), `concatMap`/`foldl` (structural-only, dropped laws documented per
  subsume-don't-proliferate), `range` appended to `Collections.ken`. Outer-ring,
  zero Axiom, zero-`trusted_base` delta, foundation-qa + Architect gate. One
  non-blocking nit (AC8 #1 reject also accepts `|| ParseError`) recorded for
  next-touch, not folded (Architect ruling). Retros in. Proof-technique finding:
  `Cons`-vs-`Cons` abstract-element base needs `cong` not bare `tt`/`Refl`
  (memory saved).
- **DS-3** (`Option`/`Result` combinators) — 🔨 **KICKED (for real) ~13:07**.
  ⚠ **My first kick (`evt_zpdcdwv8zkvr`, 10:45) was APPENDED after the DS-4
  merge/retro and foundation-leader missed it on a truncated preview — DS-3 sat
  IDLE 10:45→13:07** (~2.3h; "in flight" in prior status was wrong). Caught +
  owned by foundation-leader (`evt_728s356ja0x0`), re-kicked as a **pure lane-(a)
  combinator build** (Option getOrElse/isSome/orElse; Result mapErr/andThen/
  unwrapOr + laws; reuse `option_map`/`Functor Option`; `Err`-first caution; home
  `Data/Sums/`). The **`Either` ruling lane is MOOT** — superseded by L5 (`Either`
  is a separate catalog package, not subsumed into `Result`); nothing about
  `Either` in DS-3. Lesson saved: kick each WP standalone + mention-led, never
  appended. Normal ring → foundation-qa → Architect → git_request.
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
- **DS-5 §60 erratum** — ✅ **LANDED** `main @ 5c0ae61` (PR #438, CI-green).
  Reconciled the over-claiming `60-length-indexed-vectors.md §6` (`tail`
  gated→landed with the real acceptance-test cite; `zip`/`lookup` re-pointed to
  `DS-5c`) + the coupled conformance fixtures (`dr-injectivity-and-over-refinement`
  retained as the live AC8 enforcer). Architect + CV both caught the over-claim;
  bundled spec+conformance on one branch. Retros in.
- **`Vector` package** (Foundation follow-on) — **framable now** for the
  buildable API (`head` + `tail` + single-convoy ops), with `zip`/`lookup` gated
  on `DS-5c` — an honest partial package (DS-5-chapter split pattern). Queue
  behind DS-8; frame when Foundation frees.
- **`DS-5c`** (zip two-vector convoy + Fin-indexed `lookup`) — **named deferred
  WP**, NOT kicked this window (breadth over depth; would be a 3rd concurrent
  capability build). The §60 erratum + Vector package both point to it.
- **Data section (DS-3/DS-4/DS-6)** — the breadth priority, progressing. DS-4
  (List ext) **LANDED** (PR #443); DS-3 (Option/Result + Either ruling) **KICKED**
  (in flight); DS-6 (`DecEq Char` capstone, candidate 2nd kernel-move)
  T1-design-needed, not yet framed.
- Verify team idle in reserve. Kernel + Ergo freed. Foundation on DS-3.
  All enclave WPs (DS-5/DS-5b/DS-8b/§60) + DS-4 closed; retros collected.

**Next-move triggers (event-driven):** DS-3 git_request → honesty-gate + merge
(CI-gated, lane-a build) + confirm the Architect ruled the `Either` lane-b (if
SUBSUME, a spec-author/CV README:42 reconcile erratum follows). If window-time
remains: frame DS-6 (`DecEq Char`, careful — candidate kernel-move, T1-design).
Vector package + DS-5c + DS-8c are named/queued, not kicked this window (breadth
over depth). **Operator returns ~11:30 UTC** — judgment log is the review
artifact; keep it current.

### P3 · Foundation is the catalog-authoring home; parallelize only independent tracks
- **Call:** Keep catalog authoring on the Foundation team (coherence — one
  author's hand across the tier); run genuinely-independent tracks in parallel
  where they don't contend (e.g. the DS-5 `Vector` spec chapter on the Spec
  enclave alongside a Foundation build).
- **Why:** `06`/program-doc home the catalog at Foundation; fragmenting
  authoring across idle teams would cost coherence for throughput.
- **Reversibility:** easy.
