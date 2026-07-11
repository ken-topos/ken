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
  the prelude" — **load-bearing false**). **Exclude** `:540` ("Either way",
  English);
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
> **STATUS (2026-07-10 ~14:05): ✅ COMPLETE end-to-end.** WP (1)
> `Either` catalog package **✅ LANDED** (PR #458, `main @ a78f3b7`;
> `Data/Sums/Sums.ken`, Either verified absent from prelude — ruling
> B honored). WP (2) `Sum`→`Coproduct` rename **✅ LANDED** (PR
> #455, zero kernel delta, `declare_sum` gone). WP (3) `Sum` name now
> **freed** for `Data.Functor.Sum`.
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

### L6 · ⚠ PROMINENT — casing standard: PascalCase class-like, snake_case instance-like [OPERATOR-RULED]
> **STATUS (2026-07-10 ~15:35): ADOPTED + WIRED SURFACE COMPLETE.** Standard
> broadcast, **Foundation acknowledged**, **Librarian folded into authoritative
> guidance** (`07-catalog-style-guide.md §9` + `write-ken.md`, PR #460). The bulk
> rename of landed code has now LANDED in two arms:
> - **Phase 1 — catalog rename** (Librarian, PR #463, `main @ f71abba`): all 594
>   camelCase `fn`/`const`/`lemma`/`proof`/class-field declarations across the 14
>   catalog files → snake_case; symmetric 4049 ins/4049 del across 36 files, zero
>   `crates/src`+kernel delta, 123/123 workspace parity, Architect fidelity gate.
> - **Prelude arm — prelude Ken-names** (Steward-dispatched, PR #464,
>   `main @ 02374cbd`): the ~10 prelude-DECLARED camelCase Ken names
>   (`isSorted`→`is_sorted`, `andIntro`/`andFst`/`andSnd`→`and_*`,
>   `injectL`/`injectR`→`inject_l`/`inject_r`, `mkPair`→`mk_pair`,
>   `pairFst`/`pairSnd`→`pair_*`, `runState`→`run_state`) → snake_case in
>   `prelude.rs` + every reference across catalog/spec/tests/`examples/rosetta`;
>   symmetric 1006 ins/1006 del across 42 files, zero kernel delta, Architect gate.
>   (`mkPair` confirmed a `declare_def` helper fn, not the `Pair` former.)
>
> **L6 is now COMPLETE across the wired surface** (catalog Phase 1 + prelude).
> **Honest remainder = a doc/conformance sweep** (NOT build-wired, deferred): stale
> old names survive in `README.md`, `agent/memory|playbooks/**` (historical corpus,
> deliberately not rewritten), `docs/program/**`, and **two `conformance/**`
> fixtures** (`sound-verified-sort.ken`, `unsound-const-nil.ken` cite `isSorted`).
> Per Architect: the doc/memory prose is fine to defer indefinitely, but the two
> conformance fixtures are corpus meant to be elaborated — flag them to the
> **CV/conformance lane to sweep before any conformance run wires them**, not
> open-ended. **Precision note (Architect):** the accurate acceptance claim is
> "zero orphaned camelCase in tangled + `example` code; `reject`/`ignore`
> illustrative fences intentionally left as-authored" — ~10 camelCase survivors all
> sit inside failure-demo fences by design, not a miss. Reject/ignore-fence
> failure-demo identifiers are OUT of L6 scope; so are Rust-internal var/field names
> (e.g. `issorted_id`, `mkpair_ty` in `prelude.rs`) — L6 governs the Ken surface.
- **Call (operator, Pat):** adopt the **Python convention** for Ken-surface
  identifiers — **class-like → PascalCase, instance-like → snake_case**:
  - **PascalCase** (already conforms): types/type-constructors (`Either`,
    `Option`, `Result`, `Nat`, `List`, `Vec`), type classes (`Functor`,
    `Applicative`, `Monad`, `Traversable`, `Semigroup`, `Monoid`, `Foldable`),
    **data constructors** (`Left`/`Right`/`Some`/`None`/`Ok`/`Err`/`Cons`/`Suc`
    — "class-ilk", they construct values of a type).
  - **snake_case** (the change — currently **camelCase** in the catalog):
    functions/combinators (`getOrElse→get_or_else`, `isSome→is_some`,
    `orElse→or_else`, `mapErr→map_err`, `andThen→and_then`,
    `unwrapOr→unwrap_or`, `mapLeft→map_left`, `mapRight→map_right`,
    `concatMap→concat_map`, `rangeFrom→range_from`; single-word `either`/`swap`/
    `reverse`/`zip`/`foldl` already fine; `option_map` already conforms) AND
    **class methods / record fields** (instance-like → snake_case).
  - **Boundary calls (Steward-resolved, operator to correct if wrong):** data
    constructors follow types (Pascal); class *methods* follow functions (snake).
- **Why (operator):** more readable than the FP-common all-camelCase — it
  **distinguishes class-like from instance-like at a glance**, and reads better
  for the **far more common instance identifiers**. Deliberately diverges from
  Haskell/FP convention on the operator's explicit readability judgment.
- **Scope:** Ken-surface identifiers only — catalog `.ken`/`.ken.md` + Ken code in
  spec examples. **The Rust crates already conform** (Rust *is* this standard:
  snake_case fns, Pascal types), so the implementation is untouched.
- **Sequencing:** the **complete renaming pass rides the `.ken → .ken.md` literate
  transformation** (operator directive) — each file touched once, casing + literate
  encoding together. NOT done now.
- **Reversibility:** **moderate-class** (a catalog-wide identifier rename; pure
  mechanical, revert-clean, no semantic change) — PROMINENT because it's a
  language-surface convention binding all future catalog authoring.

### D1 · OPEN DISCUSSION (operator) — Ken auto-formatter for `.ken`/`.ken.md`
- **Raised by:** operator (Pat), 2026-07-10. **Not decided — on the discuss-list.**
- **Proposal:** a **strict automatic formatter** for Ken source (`.ken` and
  `.ken.md`), in the mold of `gofmt` / `rustfmt` / Python `black` — one canonical
  style, mechanically enforced, no per-author bikeshedding. Operator's rationale:
  strict auto-formatting has **proven to increase readability** across those
  ecosystems.
- **Trigger:** the operator noticed **exceptionally long single lines** in a
  `.ken.md` file. Confirmed on survey: `EffectfulClasses.ken.md` has many 200+
  column code lines; `catalog/guide/decomposition-abstraction.ken.md:129` is
  ~295 columns. There is **no line-length discipline on Ken code today** (the
  80-col rule is prose-only; Markdown code fences are exempt).
- **Sequencing (operator ruling):** the formatter is **mechanical, so it does NOT
  gate the literate transformation** — that WP proceeds now
  (`wp/literate-transformation.md`) and the formatter reformats the whole catalog
  (transformed files included) whenever it lands. The transformation pass is told
  explicitly **not** to hand-reflow long lines (wasted + error-prone; the
  formatter's job).
- **Open questions for the discussion (not pre-decided):** build vs adopt (is
  there prior art to reflect rather than write a bespoke tool? #6); scope (just
  line-wrapping, or full canonical layout — indentation, spacing, alignment,
  fence normalization); where it runs (CI check / pre-merge gate / editor); who
  owns it (Librarian for catalog encoding, or a tooling track). Route to the
  operator + Architect when we take it up.
- **Reversibility:** N/A (a proposal). When built, a formatter is additive tooling
  (revert-clean).

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

### RUN STATUS / resume point (2026-07-10, ~15:35 UTC)

**Live checkpoint for lossless resume across compaction.** **[operator BACK and
actively directing; autonomous window over.]** **CORE COMPLETE** (DS-8 PR #440).
§60 erratum (PR #438), K1 DS-5b, K2 DS-8b all landed. **DS-4 (List) LANDED**
(PR #443). **`main @ 02374cbd`** (Phase 1 catalog rename #463 + prelude L6 #464).

**✅ L6 casing — WIRED SURFACE COMPLETE.** Phase 1 catalog rename (PR #463,
`f71abba`) + prelude Ken-name rename (PR #464, `02374cbd`) both landed + Architect
fidelity-gated + honesty-gated. L6 complete across all build-wired Ken code; only
a NON-wired doc/conformance sweep remains (see L6 for the flagged remainder + the
CV-lane hand-off for the two conformance fixtures). Note the residual Rust-internal
`issorted_id`/`mkpair_ty` names are out of L6 scope.

**🔨 IN FLIGHT — Part 2 literate transformation (`.ken → .ken.md`),
Librarian-led.**
Progress **3/7** at last checkpoint: Transport (`a0bf8e2`), LawfulClasses,
LawfulFunctors (`bfaa80a`) done; remaining **Parsing → Collections → Map →
Sums**.
**Scope RULED 7-not-8** (my call, blessing Architect's read):
**`ProofErasure­Boundary­Checker.ken` EXCLUDED** — it's a
production-consumed input (`include_str!`'d by
`crates/ken-interp/src/proof_erasure_checker.rs`, driven via
`elaborate_file` as the runtime NC9 checker), NOT a literate-doc
entry; converting it needs a `crates/src` edit (outside the WP
boundary) + misclassifies a non-doc input. Stays plain `.ken` in
place. **Consumer loading-swaps BLESSED** (my call): each converted
file's raw consumers swap `elaborate_file(X.ken)` →
`elaborate_ken_md_file(X.ken.md)` + `include_str!` path — a
necessary zero-semantic consequence (checked fences carry
byte-identical code → identical `GlobalEnv`), tests-only,
kernel/suite-caught. **Did NOT build** the extension-dispatching
prerequisite-loader (new abstraction mid-mechanical-pass = wrong
move) — named as a reflect-don't-extend future for the test
harness. Librarian pinged to rebase Part 2 onto `02374cbd`
before touching `Collections`/`Map` (their prelude refs are now
snake_case upstream). Per-file-atomic, targeted cadence.

**Named futures (2 new, from Part 2):** (c) **relocate
`ProofErasureBoundary­Checker.ken`** out of `catalog/packages/` —
it's a production input, not a package (Architect + Steward
flagged; later); (d) **test-harness prerequisite-loader**
(name→path→`.ken`/`.ken.md` dispatch) to localize future conversions
— reflect-don't-extend, not now.

**🖥 Infra (Steward, operator-directed):** freed ~100G disk (per-worktree `target/`
cleanup after a disk-full condition); set up fleet-wide **sccache** + shared
`SCCACHE_DIR` + **`CARGO_INCREMENTAL=0`** in `/usr/local/cargo/config.toml`
(deps cache across the ~30 worktrees; incremental-off shrinks `target/`).
**Deliberately did NOT set a shared `target/`-dir** (contention across worktrees +
single-point-of-failure) — flagged the tradeoff to the operator, awaiting confirm.

**⛔ STANDING RULE (operator-reaffirmed, fleet-wide, effective now):** **NO full
local `cargo build`/`cargo test --workspace` or whole-repo baseline builds** —
TARGETED only (`CARGO_BUILD_JOBS=2 cargo build -p <crate>` +
`cargo test -p <crate> <filter>`). **GitHub CI does the full build+test at merge**
(publisher path is CI-gated) — that's where whole-workspace green is proven, not
locally. Full local builds waste CPU + bloat every worktree's `target/` (caused the
disk-full). "Full suite green" in any WP frame now means **CI-verified at merge**.
Broadcast to the Librarian (active on Part 2); memory saved. (I violated this
myself telling a rename agent to run `--workspace` — killed the builds mid-run,
corrected.)

**✅ L5 `Either`/coproduct thread — COMPLETE end-to-end** (operator-driven,
PROMINENT). All three arms landed: (1) **`Either a b = Left a | Right b` as a
CATALOG PACKAGE, NOT prelude** (ruling B) → **✅ LANDED PR #458,
`main @ a78f3b7`** (`Data/Sums/Sums.ken` — `either`/`mapLeft`/`mapRight`/`swap`
+ laws; spec reconcile
honestly frames it package-not-prelude; foundation-qa caught+fixed a real
Coproduct-reachability defect; Either verified absent from prelude); (2) **rename
effect `Sum`→`Coproduct`** (type-only, keep `InL`/`InR`) → **✅ LANDED PR #455**
(after I HELD their over-scoped `ee168a3` which had Either in prelude; zero kernel
delta; `declare_sum` gone); (3) `Sum` now **freed** for `Data.Functor.Sum`.
`Result` stays distinct. See L5 for the full trail.

**Hygiene fix (operator-directed, PR #457, `elab.rs`):** stripped leaked DS-5b WP
identifiers from production source (`refine_ds5b_goal`→`refine_branch_goal`, etc.)
— pure rename, 770 tests green. The leak had passed the entire Kernel ring
undetected; added a WP-token screen to my honesty gate (memory saved).

**🔨 IN FLIGHT — Part 2 literate transformation** (`.ken → .ken.md`,
**7 files** after the 7-not-8 ruling), Librarian-led. **Phase 1 (bulk L6 catalog
rename) split out + LANDED standalone** (PR #463) — Part 2 is now a pure-encode
diff. See the
RUN STATUS header above for live progress (3/7), the 7-not-8 scope ruling, and the
blessed consumer loading-swaps. Frame `wp/literate-transformation.md`. Mechanical,
zero-semantic; per-file bar = tangled-code byte-identical + targeted consumer tests
(NOT full-workspace — CI proves whole-suite at merge). Architect fidelity gate per
file-set → git_request → honesty-gate + merge.

**Named futures (operator sets direction):** (a) **core-data→packages migration**
(Option/Result/Nat/List/Prod/Unit are prelude-declared but the spec models them as
packages; `Either`-as-package is the first correct precedent — the larger
architectural prize); (b) **`Data.Functor.Sum`** (owns the freed `Sum` name).
**Open discussion (`D1`):** a **Ken auto-formatter** (`gofmt`/`black`-style) for
`.ken`/`.ken.md` — operator flagged exceptionally long code lines (no line
discipline on Ken code today); mechanical, so it does NOT gate the literate
transformation; on the discuss-list.

**DS-3 (Option/Result combinators) — ✅ LANDED PR #454, `main @ dd5dc51`.** New
package `catalog/packages/Data/Sums/Sums.ken` (Option getOrElse/isSome/orElse;
Result mapErr/andThen/unwrapOr + laws), outer-ring, zero kernel/Axiom/trusted_base
delta, foundation-qa (Err-first hand-trace + tt-vs-Refl cross-check) + Architect
gate. Retros closed. Real-kicked ~13:07 after my appended kick sat missed (idle
10:45→13:07; lesson saved). **DS-6** (`DecEq Char`, candidate kernel-move) — not
framed, held for operator input.

**HOLDING (event-driven) for Part 2 git_request(s):** the **literate
transformation** (Librarian-led, per-file-atomic, 7 files). Prior cycles all landed
+ honesty-gated: DS-3 (#454), Runtime rename-only (#455), elab.rs WP-name-strip
(#457), Either-catalog (#458), L6 guidance fold (#460), Phase 1 catalog rename
(#463), prelude L6 (#464), trackers (#456/#459/#461). L5 thread closed; L6 wired
surface complete. When Part 2 git_requests → honesty-gate (mechanical re-encode,
zero `crates/src`/kernel/spec-prose delta beyond blessed test-consumer loading-swaps,
every file's tangled code byte-identical) + merge. **Discuss-list awaiting operator
steer:** (a) core-data→packages migration, (b) `Data.Functor.Sum`, (c)
DS-6 (`DecEq Char`), (d) `D1` formatter. Named futures (relocate
ProofErasureBoundary­Checker, harness prerequisite-loader,
doc/conformance L6 sweep) parked. Kick every future WP STANDALONE +
mention-led (lesson from the DS-3 miss).

**DS-8 — VALVE TAKEN (composition law deferred to DS-8c for SIZE):** the
`traverse` composition coherence law (§5.3) turned out ~40-60 lemmas (not ~12-15)
— converging, **nothing walling, a SIZE trigger not a capability wall**. Per the
Steward valve (Architect defers timing to me; blessed the shape), **DS-8 Core
ships now**: `class Traversable` + `List`/`Option` instances + identity +
naturality laws (proved) + `Compose` applicative (3/4 laws: `ap_id`/`ap_hom`/
`ap_ich` + `map_coh` + Functor laws) + `ap_naturality` aux + `ap_cmp` LHS
reductions (partial, honestly marked). **`ap_cmp` (Compose's 4th law) + the
traverse composition law both deferred to DS-8c.** Foundation
transcribing into `Core/EffectfulClasses.ken.md` now → foundation-qa →
Architect gate → git_request.
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
- **DS-3** (`Option`/`Result` combinators) — ✅ **LANDED** `main @ dd5dc51`
  (PR #454, CI-green). New package `catalog/packages/Data/Sums/Sums.ken` (Option
  getOrElse/isSome/orElse; Result mapErr/andThen/unwrapOr + laws; reuses
  `option_map`/`Functor Option`, neither type re-declared). Outer-ring, zero
  kernel/`Axiom`/`trusted_base()` delta, foundation-qa APPROVE (Err-first field-
  order hand-trace of all six Result laws + tt-vs-Refl cross-check against DS-4's
  eq_at_inductive mechanism) + Architect fidelity+soundness. `orElse x None = x`
  fell out cleanly; no laws dropped. AC8 3 discriminators, specific variant. Steward
  honesty gate passed independently. Retros closed. ⚠ **My first kick
  (`evt_zpdcdwv8zkvr`, 10:45) was APPENDED after the DS-4 merge/retro
  and foundation-leader missed it on a truncated preview — DS-3 sat IDLE
  10:45→13:07** (~2.3h; "in flight" in prior status was wrong). Caught +
  owned both sides;
  re-kicked ~13:07 as a **pure lane-(a) combinator build** (the `Either` ruling lane
  MOOT per L5 — `Either` is a separate catalog package). Lesson saved: kick each WP
  standalone + mention-led, never appended.
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

---

## Autonomous window 2 — 2026-07-11 (~04:20 UTC → operator back ~11:00 UTC)

**Operator direction (locked):** two tracks; Track 1 (case_eq cleanup) sequenced
in front of Track 3 (Nat laws + DS-8c) as **Track B**; **Track A** (compare/Ord)
as-is; **formatter FULLY HELD** until the operator returns; maintain this decision
log; watch for a **2nd occurrence of any hand-rolled idiom** → track it + route
the shape to the Architect ([[second-occurrence-of-idiom-is-a-language-feature-signal]]).

### D2 · Two-track allocation across three teams (Steward)
- **Track A → Foundation:** `compare-ord-lexicographic` (PR #491) — lawful 3-way
  derived `compare`, rework Collections onto it, lexicographic `Ord (Pair)/(List)`
  + 4 laws.
- **Track B / Ergo (serial, operator order):** `case-eq-adoption` (PR #489,
  **KICKED**) → Nat arithmetic-laws package (to frame) → **DS-8c** (framed).
- **Track B / Language (parallel):** `def-path-constraint-binder-unification`
  (PR #490, **KICKED**).
- **Reversibility:** easy (re-sequenceable).

### D3 · DS-8c reassigned Foundation → Ergo (Steward)
- **Call:** DS-8c's frame says Owner=Foundation (its DS-8 context), but Foundation
  carries Track A. To honor the Track-B serial sequencing (case_eq → Nat → DS-8c
  on ONE lane) AND keep Foundation on the flagship, DS-8c moves to **Ergo** as its
  3rd serial item (within landed `fn`-synonym scaffolding; frame unchanged).
- **Reversibility:** easy (only the owning ring differs).

### D4 · Track A needs NO Architect pre-shape — `class Ord` is spec-settled (Steward)
- **Call:** `compare`-vs-`leq` primacy is **not open** — spec `51 §2.3` rules
  `leq` primitive, 3-way `cmp` "a derivable convenience." So Track A is
  **additive** (lexicographic `leq`-based `Ord` instances + derived `compare`),
  framed with that pinned; the Architect gates the lex-law **proofs** at merge,
  not the class shape. (Corrected an earlier instinct to route a shaping pass —
  that would have relitigated a decided fork.)
- **Reversibility:** N/A (following landed spec).

### D5 · Post-compact re-orient hook — discipline → machinery (operator-requested)
- **Call:** built + landed (`main @ 7b8ec13b`) a fleet-wide `SessionStart` hook
  (Claude Code `.claude/settings.json` + Codex `.codex/hooks.json` + shared
  `scripts/hooks/reorient-post-compact.sh`) re-injecting the CLAUDE.md re-orient
  directive after every compaction. Per-seat prereqs (NOT repo-shippable): Codex
  ≥0.129 + one-time `/hooks` trust; activates per-seat on next session (re)start.
- **Reversibility:** easy (additive config, revert-clean).

### D6 · Brick-1 proof shape — UNBUNDLED raw-`leq` interface (Architect, folded)
- **Call:** compare-ord brick 1's soundness lemmas land on the **unbundled
  raw-`leq` + explicit-law-argument** interface (Architect `evt_7v4argg2kp0b`),
  after two mid-flight rulings died on landed gaps: (a) the `.field`-in-declared-
  type **parser** gap killed inline `d.leq` in types → named-accessor `ord_leq_of`
  wrapper; (b) the K6 `conv_struct` **Eq-operand-congruence** gap (`LawfulClasses
  :689–723`) then killed the accessor path too (`Eq Bool (ord_leq_of…) True` won't
  δ-bridge to `Eq Bool (d.leq…) True`; both stuck neutrals, syntactically
  distinct). The unbundled shape (soundness over a raw `leq` param + explicit law
  args; the `Ord` forms thin δ-wrappers) sidesteps BOTH — every hypothesis + supplied
  law shares the literal `leq` term. Explicit-`J` for the soundness lemmas;
  `case_eq` only inside `compare_raw`'s own def. Theorems **unweakened** (raw form
  is strictly more general; the dict `Ord` version is a strict instance).
  Transcribed into the frame (brick 1 + AC1) — a stale "use `case_eq` for the
  dispatch" line would misdirect a post-compaction pickup to rebuild the rejected
  shape.
- **Reversibility:** N/A (following the landed capability envelope).

### Forward candidates (tracked; NOT compare-ord blockers — Architect-flagged)
- **[Language] modifier whnf-unfold before generalize** — `check_match_dependent`
  generalizes only *syntactic* scrutinee occurrences in the expected type; a
  scrutinee behind a transparent wrapper (compare's `compare_with`) is missed, so
  the `case_eq` sugar can't reach wrapper-soundness goals. **Occurrence #1** of the
  modifier's transparent-wrapper limitation (Architect `evt_7v4argg2kp0b`). A future
  Language item could whnf-unfold the expected type before generalizing (open design
  Q: which/how-deep to unfold, over-generalization risk). Reflect-don't-extend: NOT
  a mid-WP extension. Route to Architect when framed; **watch for occurrence #2.**
  **CORRECTION (Architect `evt_6bk169gj8d0kz`, 2026-07-11): the case-eq-adoption
  Map reject is NOT occurrence #2 — stays at #1.** I initially framed the Map
  dispatcher failure as a recurrence, but the Architect ground-truthed it as the
  **inverse**: the modifier *over*-transports (`set_intersection_member` goal has
  the scrutinee syntactically → motive substitutes `False` → arm goal becomes
  `bool_and … False` while the retained fixed-goal helper supplies the original →
  reject), whereas the whnf-unfold candidate is about *under*-reach (constant
  motive on a non-syntactic scrutinee). The reject direction proves it: an
  under-reach would have **accepted**. The whnf-unfold enhancement would fix only
  the insert/lookup under-reach half and would NOT rescue the over-transport half,
  so **Map is not demand for it** — see the case-eq applicability-boundary note
  below.
- **[Kernel] K6 `conv_struct` positional Eq-operand congruence** —
  `Eq Bool <neutral₁> True` won't δ-bridge to `Eq Bool <neutral₂> True` when the
  neutrals are syntactically distinct (`LawfulClasses :689–723`). **Now forced
  unbundling at ≥2 sites** (Ord Char documented it first; compare-ord brick 1 is the
  2nd). Per the standing recurring-idiom directive this crosses the **#2 threshold**
  → **tracked as a candidate Kernel WP** (trust-root: needs its own pseudocode/
  soundness gate; the Architect gates the algorithm). NOT a prerequisite — the
  unbundled idiom routes around it. Frame to Kernel only if it keeps forcing
  unbundling across enough proofs to justify a trust-root change.
- **[Language] type/class-vs-term/constructor namespace separation** — Ken has a
  single flat `globals: HashMap<String, GlobalId>`, so `class Eq` (LawfulClasses:60)
  **shadows** the `OrdResult` constructor `Eq` (Collections:74) for every
  declaration loaded after it; no type-qualified constructor spelling
  (`OrdResult.Eq`) exists (parser dotted-refs are module-qualification only).
  **Occurrence #1**
  (Architect `evt_3vygqece6p4ax`, concurs). Proper fix: separate namespaces
  (Haskell-style — the `Eq` class and an `Eq` ctor coexist) **or** a type-qualified
  constructor spelling — a Language/resolver WP, name-resolution-soundness-adjacent,
  the Architect gates the algorithm. Reflect-don't-extend keeps compare-ord on the
  zero-language fix (additive `const ord_eq/ord_lt/ord_gt : OrdResult` value aliases
  in Collections before `class Eq`, consumed downward). Frame to Language if it
  recurs.

**Arc note — compare-ord as a language-surface stress test.** Brick 1 surfaced
**three distinct gaps, every one routed around at the catalog level with zero
kernel/language change** (parser `.field`-in-declared-type → unbundled interface; K6
`conv_struct` Eq-operand congruence → raw-`leq` params; flat-namespace collision →
value aliases). This is the reflect-don't-extend discipline working as intended — the
outer-ring catalog exercises the language surface and each gap is met with a local,
zero-TCB idiom, not a mid-WP kernel/parser patch. All three are tracked candidates,
none blocking; the pattern to watch is whether any recurs enough to justify its
lane's WP (Kernel for K6, Language for the modifier-unfold and the namespace split).

### D7 · case-eq-adoption re-scoped — Map DROPPED, ship the two small sites (Architect + Steward)
- **Call:** the case-eq-adoption Map bulk (originally "79 uses — the bulk," 64
  dispatchers) is **dropped**; the WP re-scopes to the two small sites (EmptyDec 4
  + LawfulClasses `list_deceq`). Grounding (Architect `evt_6bk169gj8d0kz`): Map's
  dispatchers are **not** the inline-explicit-`J` idiom the frame assumed — they're
  a distinct, legitimate **precomputed-`Or` → plain-`match` → named-helper** idiom
  where the per-branch transport lives in named helpers, not a modifier-synth motive.
  case-eq can't subsume it (two sub-families fail **oppositely**: insert/lookup
  under-reaches to a constant motive; `set_intersection_member` over-transports and
  fail-closed rejects). **Not a defect, not an alt-form, not a modifier prerequisite
  to wait on — coexistence.** ergo restored the branch clean (no commit); nothing to
  unwind. WP stays **standalone** (its LawfulClasses edit is the Track-A merge
  anchor — lands first, Track A rebases its non-overlapping `Ord`-instances hunk
  onto it). AC1/byte-reduction re-scoped honestly to the two sites (no papering).
  Frame updated (RE-SCOPED banner + site list).
- **Reversibility:** N/A (following landed capability + Architect fidelity read).

**Insight — the case-eq modifier's genuine applicability boundary.** case-eq
subsumes **only** inline-transport of a **syntactically-present** scrutinee where
the branch proves the transported goal **in-place** (EmptyDec + LawfulClasses
`list_deceq` — the sites that succeeded). It does **not** fit the
**dispatch-to-explicit-motive/fixed-goal named-helper** idiom used for large
factored proofs (Map): under-reach where the scrutinee is non-syntactic, or
over-transport where it is but the goal is proven by a retained fixed-goal helper.
That named-helper idiom is a **legitimate coexisting pattern, NOT a hand-rolled
idiom to eliminate** — so it does **not** trigger the standing recurring-idiom /
subsume directive, and it is **not** demand for the modifier-whnf-unfold
enhancement (which addresses only the under-reach half). Reflect-don't-extend:
no modifier/kernel work for Map.

### Coordination faults (this window)
- **Foundation escalation #3 (`compare-ord` brick 1) was a branch-local
  misattribution** — the implementer/leader escalated a full-catalog-load
  `KernelRejected` as "raw law application is not admitted / capability
  prerequisite"; the Architect **reproduced the exact declaration shape green** in
  an isolated base env and localized it to a branch-only `class Eq`/`OrdResult.Eq`
  namespace shadow. Escalations #1 (parser gap) and #2 (K6) were legitimate, so this
  is not a crying-wolf pattern, and Foundation's refusal to paper atop a red probe is
  correct discipline — **but the lesson stands: isolate/diff against a known-green
  reference before attributing a red to a capability gap and pulling in the
  Architect** (the most expensive unit). Watch for a 4th misattribution → then coach
  isolate-before-escalate directly.
- **foundation-implementer §10 retro DROPPED** on `message_type:"handoff"` then
  `"retrospective"` (both 400 — closed enum); seat idle believing it posted. I
  **relayed it** (attributed) to close the ring. Every kickoff this window now
  carries an explicit `message_type`-enum warning.
- **Gt "falsification" was an offset-drift span mis-attribution (2nd attribution
  artifact in compare-ord brick 1).** The Architect set a falsifiable condition
  ("if my exact bound-var helper still red-lines in the full load, THAT is K6
  evidence"); Foundation applied the helper verbatim, the full
  `structural_deceq_acceptance` still `KernelRejected TypeMismatch`, byte-span
  `25526–26012` → the pre-existing **Pair** section. Foundation correctly
  re-escalated per the Architect's own directive (NOT a crying-wolf fault). The
  Architect then dissolved it (`evt_5qht368tv6g0`): the baseline suite is **green
  at `9b09124d` WITHOUT** the new compare/Ord/Gt section, so the red is the
  **uncommitted new section**, and the Pair-section span is **offset drift** —
  inserting a new block *before* a section shifts every downstream byte offset, so
  the kernel's reported span lands in innocent (green-on-main) bytes. **Reusable
  diagnostic for a full-`.md`-load red: (1) is the baseline green without the new
  section? then the new section is the cause regardless of the span; (2) bisect the
  new section; (3) never trust a byte-span that points into code that was green on
  the base ref.** Concrete lead the Architect flagged: flat-namespace collision
  (candidate #1 class — redefined `bool_or`/`bool_or_false_total`) or the OrdResult
  `Eq`/`Lt`/`Gt` alias discipline (`evt_3vygqece6p4ax`) not applied. Candidate #2
  (K6) **unchanged** — routes around cleanly, no Kernel trigger.
  **RESOLVED (`evt_40ydcv9a45yjd`, reproduced red→green):** neither lead — the
  Architect's own bisection (helper+aliases green; first flip = `compare_gt_sound_raw`)
  localized it to a **local incomplete-discrimination proof bug** (the `Inl` arm
  left `compare_second_result (leq y x)` stuck; fix = nested `bool_dichotomy`, the
  Eq lemma's own idiom). **The pattern is now 3-for-3: every assembled-load red on
  compare-ord brick 1 was LOCAL — namespace shadow (esc #3), offset-drift span
  misread, incomplete-discrimination proof bug — and ZERO were the kernel.** The
  standing coaching for this class of catalog-proof WP: a full-`.md`-load
  `KernelRejected` is almost never a capability/K6 gap; **reproduce + bisect
  locally against the green base ref FIRST** (the Architect did each time and the
  helper/kernel was always innocent), and treat a byte-span into base-green code as
  offset drift, not the cause. Foundation's refusal-to-paper discipline stayed
  correct throughout; the lever is cheaper self-triage before pulling the Architect
  (the most expensive unit) — the isolate-before-escalate lesson, now well-earned.
- **DS-8c kicked on already-delivered work — my stale-tracker error.** I ran the
  full release process (frame reconcile PR #505 + Handoff Gate ergo-ring compaction
  + kick `evt_7r0wkgsav5b62`) on a WP that had **already landed** (`a3a3a39d`,
  2026-07-10) in a prior window. Root: I verified the **frame** was on `main` and
  shovel-ready, but **never grepped `main` for the WORK itself** — a frame's
  presence ≠ the deliverable being undone. My D3 reassignment and run-status both
  carried DS-8c as "queued/framed," a line stale by a full window. **RULE (new): a
  WP framed in a prior window is presumed-suspect — before ANY kickoff/Handoff
  Gate, grep `origin/main` for the deliverable's artifacts (the committed
  file/fn/law, not the frame doc) and confirm it is genuinely absent.** Sibling of
  [[my-own-tracker-capability-landed-line-can-be-stale]] applied to WP-kicking. The
  ergo ring did exactly this (ground-truthed the frame against main before writing)
  and caught it — the cheap check I skipped. Cost was one compaction + an aborted
  rebase; no duplicate authored. Also: the frame's "buildable-now" language, itself
  authored the same day the work landed, is what made the staleness invisible —
  perishable "current state" in a frame ([[wp-frame-stale-vs-landed-kernel]] class).

### Run status / next triggers (event-driven) — updated ~10:39 UTC
- **MERGED:** Language `def-path-constraint-binder-unification` (`main @ 41df4e62`);
  ring §10 retros in. Enclave still holds the spec parity clause (`32`/`33`) → CV
  verdict → `git_request`. Playbook watchdog hardening (`main @ bc1c0643`).
- **CLOSED — Ergo `case-eq-adoption`:** MERGED `main @ 9b09124d` (PR #501, CI-green);
  **all three §10 retros IN** (leader/qa/implementer). Track-A merge **anchor**
  landed. Rebased approved `18bd3ff6` via cherry-pick (it had drifted behind my
  re-scope doc merges); tracker synced `main @ 1082ef89`. WP done.
- **CLOSED — Ergo `nat-arithmetic-laws`:** `main @ d762c99b` (PR #503, CI-green);
  **all three §10 retros IN** (leader `evt_kqb95k8m9t7j` / qa `evt_20trnmaps63mf` /
  implementer `evt_2cx4ynftx916j`) — WP done. Kicked after the full Handoff Gate
  (ergo ring compacted + drops verified); honesty gate clean (2-file outer-ring:
  `Core/NatArith.ken.md` + acceptance test; ancestry no-drift on `1082ef89`;
  trust-surface hits all benign test guards).
  QA (`evt_74gcd6am9pnk3`) + Architect fidelity (`evt_2j0f33gh09f64`) green —
  Architect walked every law (definitional Refl bases vs real-induction hard laws
  correctly discriminated; `mul_add_distrib_l`/`mul_one_r` honestly derived; zero
  papering, nothing size-deferred). Placement = **new `Core/NatArith.ken.md`**
  (separated from OrdNat export). **Parsing `nat_add` unification DEFERRED** to a
  follow-up de-dup seam —
  ergo-implementer's grounded finding: **catalog entries have no import/load
  mechanism**, so replacing Parsing's local `nat_add` with the canonical `add`
  would make the Parsing entry non-self-contained rather than a clean green
  unification. Correct per frame seam-1 ("unify only if clean/green, else flag").
  **Tracked candidate follow-on: a catalog de-dup pass** once/if a load mechanism
  exists (NOT a blocker, NOT a 2nd-occurrence-idiom escalation — a known
  duplicate-definition seam). Frame on `main`, case_eq-INDEPENDENT. DS-8c after.
- **ACTIVE — Foundation `compare-ord-lexicographic` (bricks 1+3):** Eq/Lt raw
  soundness green (unbundled D6). **Gt totality-transport RESOLVED — a local
  incomplete-discrimination PROOF BUG, definitively NOT K6, NOT the helper, NOT a
  name collision, NOT size-defer** (Architect `evt_40ydcv9a45yjd`, reproduced
  red→green). Root: `compare_gt_sound_raw`'s `Inl` arm (`leq x y = True`)
  transported only `leq x y → True`, leaving `compare_second_result (leq y x)`
  **stuck** on the neutral `leq y x`, so the Ω-`absurd` (elab.rs:526) got a stuck
  `Equal` instead of `Bottom` → `TypeMismatch { expected: g0, … }`. Lt was green
  only because its arm reduced to a canonical ctor (`ord_gt`); Gt's didn't — that
  asymmetry was the whole bug. **Fix (catalog-only, Architect-verified green):**
  nest a second `bool_dichotomy (leq y x)` in the `Inl` arm — the **Eq lemma's own
  two-level dispatch idiom** (`LawfulClasses:515-528`), transporting
  `compare_second_result b` (the inner reducer). **Gt now GREEN** (`7ed10f1`,
  `structural_deceq_acceptance` passed after named dispatcher + outer-J bridge —
  each step a local proof-shape iteration, `KernelRejected`→`NotAFunction`→bridge,
  no kernel escalation). **REBASE onto Track-B anchor in recovery:** replaying the
  scratch history hit the LawfulClasses conflict, and a `reset --soft` produced an
  over-scoped 9-file diff (`d351072a`, rejected). **Steward rebase ruling
  (`evt_1sy6r96120vab`):** DROP the scratch red artifacts (`6f84e07`/`346a732b`)
  and `d351072a` — land only the final green tree onto `9b09124d`; the LawfulClasses
  conflict is **Foundation's to resolve, NOT cross-WP arbitration** (Track-B
  `list_deceq` at `9b09124d` is merged/authoritative → take it verbatim, layer the
  compare/Ord/Gt section in its own region, seam-2 different regions). foundation-
  implementer executing the clean-transplant. **STEWARD RE-ANCHOR:** the leader's
  recovery cited base `1082ef89`, but I've since advanced `origin/main` to
  `2f09d625` (Nat-laws + docs — none touch LawfulClasses/Collections); told the
  implementer to base on **current** `origin/main` so the honesty-gate ancestry
  stays clean. **Implementer stalled TWICE mid-transplant** (no-poll terra seat
  ends its turn per step; re-roused both times) — watch for repeat; next: rebased
  green SHA → lex Pair/List → Architect fidelity gate → `git_request`.
  (Push-credential watch cleared.)
- **ALREADY DELIVERED — `ds-8c-traverse-composition-law` (`a3a3a39d`, 2026-07-10;
  my kick was a STALE-TRACKER ERROR).** ergo-leader ground-truthed the frame
  against `origin/main` before writing a line and found DS-8c **already landed**
  (`evt_5f9ee9rgahcv9`), which I independently confirmed: `compose_ap_cmp` proved
  (§9.6, `EffectfulClasses.ken.md:2283`); the traverse composition law proved (§9.7
  = spec `56 §5.3`); `ds8_traversable_acceptance.rs` asserts both
  (`traverse_composition_law_is_present_and_kernel_checked` +
  `compose_instance_head_still_genuinely_absent`); zero-`Axiom`/empty-`trusted_base`
  guards pass; `a3a3a39d` ("closes DS-8 deferred split") is an ancestor of main and
  EffectfulClasses is byte-unchanged since. **I ran a Handoff Gate + kick
  (`evt_7r0wkgsav5b62`, PR #505 frame reconcile) on already-done work** — cost: one
  ergo-ring compaction + a brief aborted stale-branch rebase. No duplicate authored
  (ring caught it). Assignment CLOSED as delivered (`evt_2r36ystyfz326`).
- **TRACK B (Ergo lane) COMPLETE:** `case-eq-adoption` (#501) → `nat-arithmetic-laws`
  (#503) → DS-8c (`a3a3a39d`) all delivered. Ergo idle, awaiting operator direction
  (~11:00 UTC) — no locked-plan work remains on its lane; do NOT invent work.
- **Next gate events:** Foundation compare-ord `git_request` (post rebase + lex +
  Architect gate) — the sole remaining locked-plan build; Language spec-clause CV
  verdict.
- **Forward candidates: 3, count UNCHANGED** (Kernel K6 conv_struct #2 — the Gt
  "falsification" did **not** increment it, Architect ruled it route-aroundable;
  Language modifier-whnf #1 — Map did NOT bump it; Language namespace-split #1). Map
  named-helper idiom logged as legitimate coexistence, not an elaborator-feature
  candidate.
- **Watchdog: CronCreate job `1236a1cd`** (15-min, pane stall-sweep + git/gates +
  mentions) — replaced the codex-era bash loop. **Formatter HELD.** Operator back
  ~11:00 UTC — this log is the review artifact.
