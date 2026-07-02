# K5 — observational fragment: `Top`-intro + `Bottom`-elim — seed cases

Format: `../../README.md`. These pin the **K5** kernel capability
(`spec/10-kernel/16-observational.md §1.4` "Top and Bottom: the observational
fragment"): the two prelude rules that **close** the two outcomes of `Eq`
reduction on inductive data (`§2.2`) — `(Top-Intro)` `tt : Top` (the
trivially-true, same-nullary-constructor equality) and `(Bottom-Elim)`
`absurd C p : C` with **`C : Omega_l` only** (ex-falso, the impossible,
distinct-constructor equality). `Top`/`Bottom` are **bare `Omega_0`
sub-singleton prelude constants** — a genuine singleton and the empty prop — not
the K1 `Unit`/`Empty` `Type 0` inductives coerced in (`§1.3` correction; there
is **no** general `Type → Omega` coercion). This is what a **complete**
proof-carrying instance over a decidable carrier needs (`50-stdlib/51 §6` —
`Ord.antisym`, `DecEq.sound`/`complete`), the fragment **beyond** K4's
`Eq`-motive elimination.

## Grounding (content-verified against the landed targets)

- `16 §1.4` (landed, `9646d37`) — the two rules: `(Top-Intro)` `tt : Top`, a
  prelude constant, **unique inhabitant up to Ω-PI** (no elim — a `Top` proof
  carries no info); `(Bottom-Elim)` `absurd C p : C` with the **scope check
  `C : Omega_l`** (a *sort* check, not a wildcard — a `Type`-motive is rejected;
  into-`Type` `False_rect` is a **noted-not-admitted** cheap reopen), **never
  reduces** (`Bottom` has no constructor → neutral forever). Soundness =
  **typing-admission only**: no new reduction rule, no new conversion rule,
  `eq_reduce` **unchanged**; distinct from the forbidden elim-*out*-of-Ω.
- `16 §1.1` — **formation** (only genuine **sub-singletons** enter `Omega`;
  `Top` = 1 inhabitant, `Bottom` = 0, so both are admitted directly); a Π into
  `Omega` is in `Omega` (so `not P := P -> Bottom : Omega`, `§1.3`).
- `16 §1.2` — conversion at an `Omega`-type is **definitionally irrelevant**
  (the shortcut that settles two `absurd` terms without a new congruence rule).
- `16 §2.2` — `Eq`-at-inductive reduces: **same nullary constructor ⇝ `Top`**
  (`Eq Bool True True ⇝ Top`), **distinct constructor ⇝ `Bottom`**
  (`Eq Bool True False ⇝ Bottom`) — homed in `seed-observational.md`
  (`eq-inductive-same-ctor` / `eq-inductive-diff-ctor`); K5 provides the
  intro/elim that **close** those reducts, it does not re-pin the reduction.
- `14 §3` (K4, landed) — `Eq`-motive elimination case-splits a decidable law
  into per-branch goals; the equal branches produce `Top`-goals, the
  contradictory branches produce `Bottom`-hypotheses — the two-branch shape K5
  closes.
- `50-stdlib/51 §6` — the **concrete-equality-conclusion** laws
  (`antisym`/`sound`/`complete` → `Top`/`Bottom`) K5 makes provable; the
  `../../stdlib/classes/seed-lawful-classes.md` **`(gated: K5)`** complete
  accept arm this capability un-gates.
- SCT/guardedness termination check (`../judgments/seed-judgments.md`, the K2c
  SCT seam) + WP K5 **AC6** — the call graph must traverse **every** syntactic
  position that can hold a group-member reference; a new `Term` position it
  skips is where recursion hides.

## Scope — the two rules, their exact admission, and the trust root

These pin the **new admissibility** (`tt` inhabits `Top`; `absurd` discharges an
`Omega` goal from `Bottom`), its **two exact boundaries** (the motive is a
*proposition* — sort, not wildcard; the proof is *genuinely `Bottom`* — the
consistency-critical premise), the **trust root** (a new syntactic position must
not let recursion escape the termination gate — the ★★ AC6 hard gate), the
**soundness posture** (typing-admission only — no new reduction/conversion), and
the **integration** (the two-branch decidable-order law that ties K5 to
`51 §6`). The reduction that *produces* the `Top`/`Bottom` goals (`16 §2.2`) and
the forbidden **elim-out-of-Ω** direction are **distinct** and homed elsewhere
(referenced, not re-pinned).

**Tags.** `(soundness)` — a kernel admissibility/consistency commitment (a wrong
K5 rule is a soundness hole: an `absurd` from a non-empty proof, or a motive
scope leak, or a recursion the SCT gate cannot see, each inhabits `Bottom`).
`(oracle)` — the concrete term spellings (`tt`, `absurd`, `Bool`, `bool_leq`).
The **admissibility, the two boundaries, the SCT traversal, the no-new-rule
posture, and every verdict** are **normative**.

---

## The capability — the two rules close the two `Eq`-reduction outcomes

### kernel/observational/top-intro-proves-reduced-top-goal (soundness)
- spec: `16 §1.4` (Top-Intro), `16 §2.2` (`Eq` same-nullary-ctor ⇝ `Top`),
  `16 §1.2` (Ω-PI)
- given: the goal `Eq Bool True True`, which whnf's to **`Top`** (`§2.2`,
  same-nullary-constructor), discharged by the prelude constant `tt : Top`
- expect: **accepts** — `tt : Top` is the canonical proof of the
  reduced-to-`Top` goal. **Verdict flip (anti-green-vs-green):** on the
  **pre-K5** kernel the same goal is **rejected/unprovable** — no `tt` symbol
  exists, so the `Top` reduct has **no canonical inhabitant** and the proof term
  fails to elaborate. Also assert the **structural** AC3 fact: `tt` is a
  **prelude constant**, so it does **not** enter `GlobalEnv::trusted_base()` — a
  `tt`-using proof adds **zero delta** (grep: `tt` is prelude-filtered, not a
  fresh `Opaque`)
- why: (soundness) `Top`-intro closes the trivially-true equality — the empty
  conjunction of argument equalities for a nullary constructor (`§2.2`). With
  Ω-PI (`§1.2`) `tt` is the **unique** inhabitant of `Top`, so `Top` is a
  genuine singleton (no elim needed — a `Top` proof carries no info). Assert the
  **observable**: the `Top`-goal **admits `tt`** (a real proof term), not "it
  resolves"; and **`tt ∉ trusted_base()`** (a `Top` proof is information-free,
  so it must be zero-delta — a build declaring `tt` as a non-prelude `Opaque`
  would grow `trusted_base()`, breaking AC3,
  [[kernel-backed-claim-grep-the-emission-not-the-name]]). The pre/post-K5 flip
  makes it discriminating.

### kernel/observational/bottom-elim-discharges-from-contradiction (soundness)
- spec: `16 §1.4` (Bottom-Elim), `16 §2.2` (`Eq` distinct-ctor ⇝ `Bottom`)
- given: a branch carrying a hypothesis `h : Eq Bool True False`, which whnf's
  to **`Bottom`** (`§2.2`, distinct-constructor); an arbitrary goal
  `C : Omega_l` (e.g. some `Eq A x y`); discharged by `absurd C h`
- expect: **accepts** — `absurd C h : C` discharges the `Omega` goal from the
  `Bottom`-typed hypothesis. **Verdict flip:** on the **pre-K5** kernel the same
  contradictory branch is **rejected/unprovable** — no `absurd` machinery exists
  to discharge it
- why: (soundness) ex-falso — from `Bottom` any proposition follows.
  **Vacuous:** `Bottom` is empty, so `absurd` **never fires** — the branch is
  unreachable, so the discharge cannot manufacture a false proof, sound
  **regardless of `C`** (`§1.4` (ii)). Assert the **observable**: the
  contradictory branch **discharges** to the goal via `absurd`, not "it
  resolves". Ties to `51 §6`'s `antisym` contradictory branch; composed
  end-to-end in `antisym-two-branch-tt-and-absurd` below.

---

## The exact boundary #1 — the motive is a proposition (a sort, not a wildcard)

### kernel/observational/absurd-motive-must-be-omega-not-type (soundness)
- spec: `16 §1.4` (scope: `C : Omega_l` only — a sort check, not a wildcard)
- given: `absurd C p` with the **proof held fixed** at `p : Bottom`, the motive
  in two forms: (a) `C := Eq Bool x y : Omega_l` (an **`Omega`-sort** motive);
  (b) `C := Bool : Type 0` (a **`Type`-sort** motive)
- expect: **the verdict splits on the motive's sort.** (a) **accepts**; (b)
  **rejected** — the kernel enforces `classify(C) = Sort::Omega(_)`; a
  `Type`-classified motive is refused (`BadEliminator`). **Non-degenerate pair**
  keyed on the structural discriminator *motive-classifies-as-`Omega`*
  (COORDINATION §7), the proof held fixed
- why: (soundness) K5's **minimal-safe scope** is `Omega`-only, matching the
  observational-fragment need exactly (every discharge routes to an
  `Eq … : Omega`). Into-`Type` `False_rect` is **also sound** (vacuous
  regardless of codomain) but is **noted-not-admitted** — a cheap
  independently-justified reopen with its own rule (`§1.4`; Architect ruling
  `evt_2ke4y023edywm`). This case pins the **admitted boundary**: a build
  widening the motive to `Type` (though sound) is **non-conformant** with the
  Ω-only spec. The accept arm alone is green-vs-green under a **dropped
  sort-check** (a wildcard motive); the **`Type`-motive reject arm is the
  guard**. **Disconfirming check:** would (b) also reject if its motive were
  `Omega`? **No** — (a) accepts — so the reject is gated on non-`Omega`-ness,
  not coincidental. (The classify match is exhaustive-by-construction: the
  two-variant `Sort` enum has no `_` arm, so a third sort would be a compile
  error, not a silent fallthrough — COORDINATION §7.)

---

## The exact boundary #2 — ex-falso requires a genuinely empty hypothesis

### kernel/observational/absurd-proof-must-be-bottom-typed (soundness)
- spec: `16 §1.4` (Bottom-Elim premise: `Gamma |- p : Bottom`)
- given: `absurd C p` with the **motive held fixed** at `C : Omega_l`, the proof
  slot in two forms: (a) `p : Bottom` (a genuine `Bottom`-typed proof — e.g. a
  hypothesis variable of type `Bottom`); (b) `p := tt : Top` (a **well-typed
  term of the wrong type** — `Top`, not `Bottom`)
- expect: **the verdict splits on the proof's type.** (a) **accepts**; (b)
  **rejected** — the proof slot is `check`ed at `Bottom` (`check(p, Bottom)`),
  and `tt : Top` fails it (`Top ≢ Bottom`). Verdict flips on the proof's type,
  motive held fixed
- why: (soundness ★) the **consistency-critical premise**. `absurd` is sound
  **only because** its proof is genuinely `Bottom`-typed (empty), so the
  eliminator never fires. A kernel admitting a **non-`Bottom`** proof —
  `tt : Top` is a valid, *inhabited*-type term — would let `absurd C tt`
  manufacture a proof of an **arbitrary** `C : Omega` from an **inhabited**
  hypothesis (prove `Eq Bool True False`, …) → **inconsistency**. The
  `check(p, Bottom)` is exactly what makes ex-falso vacuous. **Disconfirming
  check:** a kernel that skipped the proof-type check would **accept** (b) —
  green-vs-green with the accept arm — so this flip is the **sole net** for that
  hole. Assert the **observable**: (b) is **rejected at the proof-type check**,
  not accepted.

---

## The trust root — a new syntactic position must not hide recursion (★★ AC6)

### kernel/observational/sct-rejects-recursion-through-absurd (soundness)
- spec: WP K5 **AC6** (Architect/Steward hard gate); the SCT/guardedness
  termination check (`../judgments/seed-judgments.md`, K2c SCT seam); `16 §1.4`
  (`absurd` is a new `Term` former with two subterm positions)
- given: a **transparent** (`declare_def`) recursive definition whose **only**
  self-reference sits **inside an `absurd` subterm** —
  `loop : Bottom := absurd Bottom loop` (the group-member reference `loop` in
  `absurd`'s **proof** position; a sibling arm places the self-reference in the
  **motive** position)
- expect: **rejected** — `NotTerminating`: the SCT call-graph builder
  (`collect_calls`) **traverses `absurd`'s motive and proof** subterms, finds
  the self-call, and — no size-decrease / no descent — rejects. **Discriminating
  flip:** against a `collect_calls` that does **not** recurse into `absurd`, the
  self-reference is **invisible** to the call graph → the def is **wrongly
  admitted** → a transparent `loop : Bottom` **δ-loops into a closed inhabitant
  of `Bottom`** → inconsistency. The verdict flips **admit↔reject** on whether
  SCT traverses `absurd`
- why: (soundness ★★) the **one spot where "additive, obviously fine" hides a
  soundness regression** — Architect's + Steward's named hard AC, **explicitly
  not** covered by AC3 zero-regression. A new syntactic position (`absurd`'s
  subterms) the call-graph builder doesn't traverse is a place a recursive call
  can **hide**, escape the termination gate, and δ-loop into a `Bottom`
  inhabitant — the K2c unapplied-self-reference hole
  ([[sct-unapplied-self-reference-over-accepts]]), **one syntactic position
  over**. **Every existing SCT case stays green with a non-traversing
  `collect_calls`** (no existing test contains an `absurd` term) — so a green
  regression suite is *exactly* what this hole looks like; this
  **discriminating** case is the **sole** net. **Absence-gate satisfied:** (a)
  names the exact guard — `collect_calls` traverses `absurd`'s motive+proof; (b)
  **disconfirming** — would the def **also** be rejected under the precise bug?
  **No**: a non-traversing `collect_calls` **admits** it — so the reject is
  guard-gated, not coincidental. Assert the **observable**: the def is
  **rejected** (`NotTerminating`), the flip is
  admit-under-the-dropped-traversal.

---

## The soundness posture — typing-admission only (no new reduction/conversion)

### kernel/observational/absurd-neutral-no-new-rule (soundness)
- spec: `16 §1.4` (never reduces; soundness = typing-admission; `eq_reduce`
  unchanged; no new conversion rule), `16 §1.2` (Ω-PI conversion shortcut)
- given: an `absurd C p` term (motive `C : Omega_l`, `p : Bottom`); and two such
  terms `a₁ := absurd C p₁`, `a₂ := absurd C p₂` at the **same** motive
  `C : Omega_l` but **distinct** `Bottom`-proof subterms `p₁ ≠ p₂` (as terms)
- expect: (structural, **not** a value-flip) **`absurd C p` does not reduce** —
  `Bottom` has no constructor, so `whnf` leaves it **neutral forever** (no
  ι-rule, no new `whnf` case); and **`eq_reduce` is byte-unchanged** (no new
  reduction rule). **`a₁ ≡ a₂` convert** — both typed at `C : Omega_l`, so the
  `§1.2` proof-irrelevance shortcut fires on the **`Omega`-type** (upstream of
  the term), settling them **regardless** of the distinct proof subterms —
  **not** a bespoke `absurd`-congruence rule
- why: (soundness) K5 adds **no** new reduction or conversion rule — soundness
  is **entirely typing-admission** (`§1.4`), the analog of K4's
  `omega-elim-conv-embedding-commutes`. The observable is **structural**
  (neutral + Ω-PI-settled conversion), **not** a value-flip, because "no new
  rule" isn't value-observable — it is a trace/structural property (the X1
  discipline: assert the structure the bug perturbs). A build that added an
  `absurd`-**reduction** (fired it), or a bespoke `absurd`-**congruence** rule
  instead of the Ω-PI shortcut, perturbs it. **Honest note on the flip
  condition:** this becomes a value-flip only if the spurious rule produced a
  *wrong value*; here it is pinned structurally (neutral **and** conversion
  settled on the `Omega`-type, not the subterms).

---

## The integration — the two-branch decidable-order law (ties to `51 §6`)

### kernel/observational/antisym-two-branch-tt-and-absurd (soundness)
- spec: `16 §1.4` ("Why these two complete the observational fragment"), `14 §3`
  (K4 `Eq`-motive elim case-split), `50-stdlib/51 §6` (`Ord.antisym`)
- given: the `Ord.antisym`-shaped obligation over `Bool` — from `leq x y` and
  `leq y x`, conclude `Eq Bool x y` — case-split (K4 Ω-motive elim) on `x`, `y`:
  the **equal-constructor** branches conclude `Eq Bool c c ⇝ Top` (closed by
  `tt`); the **distinct-constructor** branches carry a **contradictory
  hypothesis** (`leq True False ∧ leq False True` is false, a `Bottom`),
  discharged by `absurd`
- expect: **accepts** — a **complete, real** proof of `antisym` over `Bool` is
  constructible: `tt` closes the equal branches, `absurd` discharges the
  contradictory ones. **Verdict flip:** with **K4 only** (pre-K5) the same
  obligation is **rejected/unprovable** — the equal branch's `Top`-goal has no
  canonical proof and the contradictory branch's `Bottom` cannot be discharged,
  so `antisym` is **unprovable** (K4's `Eq`-motive elim discharges only
  live-`Eq`-conclusion laws, `51 §6`)
- why: (soundness, integration) the concrete `51 §6` un-gate: the **complete**
  zero-delta `Ord Bool` accept arm (currently **`(gated: K5)`** in
  `../../stdlib/classes/seed-lawful-classes.md`) becomes realizable **because**
  `tt`/`absurd` close the exact two branches K4 alone could not. K5's two rules
  are **precisely** the two-branch shape a decidable-order law needs — this case
  is the end-to-end tie, and the driver of the post-K5 un-stage (the seed's
  `(gated: K5)` tags → live). Assert the **observable**: the composed obligation
  **admits a complete proof term** (both branches closed), flipping pre-K5.

---

## Coverage map

- **Capability (soundness):** `top-intro-proves-reduced-top-goal` (the `Top`
  reduct closes; `tt ∉ trusted_base()`),
  `bottom-elim-discharges-from-contradiction` (ex-falso). Both pre/post-K5
  flips.
- **Boundary #1 (soundness):** `absurd-motive-must-be-omega-not-type` (the
  motive is a *sort*, `Omega`-only, not a wildcard).
- **Boundary #2 (soundness ★):** `absurd-proof-must-be-bottom-typed` (the
  consistency-critical premise — a non-`Bottom` proof is rejected).
- **Trust root (soundness ★★):** `sct-rejects-recursion-through-absurd` (the AC6
  hard gate — SCT traverses `absurd`; a dropped traversal δ-loops into
  `Bottom`).
- **Posture (soundness):** `absurd-neutral-no-new-rule` (typing-admission only —
  neutral forever, `eq_reduce` unchanged, Ω-PI-settled conversion).
- **Integration (soundness):** `antisym-two-branch-tt-and-absurd` (the `51 §6`
  un-gate — `tt`/`absurd` close the decidable-order law's two branches).

## Cross-case consistency sweep

- **K5 is admissibility/typing only — never a new reduction or conversion
  rule.** `absurd-neutral-no-new-rule` (`absurd` neutral, `eq_reduce` unchanged,
  Ω-PI conversion) and the two capability cases (which *admit* `tt`/`absurd` but
  assert no new reduct) agree: K5 touches **only** typing-admission (`§1.4`
  soundness). A case adding an `absurd`-reduction path, or an `absurd`-specific
  conversion rule, would contradict this — and the soundness argument that rests
  on it.
- **`Bottom`-elim's two premises are both structural checks.**
  `absurd-motive-must-be-omega-not-type` (motive is a *sort*, `Omega`) and
  `absurd-proof-must-be-bottom-typed` (proof is genuinely `Bottom`) agree: both
  premises of `(Bottom-Elim)` are enforced as type/sort checks, neither a
  wildcard. A case accepting a `Type`-motive **or** a non-`Bottom` proof
  contradicts the rule's two hypotheses.
- **The two rules close exactly the two `Eq`-reduction outcomes.**
  `top-intro-proves-reduced-top-goal` (same-ctor ⇝ `Top`, closed by `tt`),
  `bottom-elim-discharges-from-contradiction` (distinct-ctor ⇝ `Bottom`,
  discharged by `absurd`), and `antisym-two-branch-tt-and-absurd` (both
  together) agree with `16 §2.2`'s reduction (homed in `seed-observational.md`):
  `Top`/ `Bottom` are the two `Eq`-reducts, and `tt`/`absurd` are their
  intro/elim. A case where `tt` proved a distinct-ctor goal, or `absurd` were
  needed for a same-ctor goal, would contradict `§2.2`.

## Subsumed / not-duplicated (one home per property)

- **The `Eq`-at-inductive reduction** (same-ctor ⇝ `Top`, distinct-ctor ⇝
  `Bottom`) is **`seed-observational.md`'s** (`eq-inductive-same-ctor` /
  `eq-inductive-diff-ctor`). K5 pins the **intro/elim that close** those
  reducts, referencing the reduction, not re-pinning it.
- **`Eq`-motive elimination (K4, `14 §3`)** — the case-split that *produces* the
  per-branch `Top`-goals / `Bottom`-hypotheses — is
  **`../inductive/seed-k4-omega-motive-elim.md`'s**. K5 provides the fragment
  **beyond** K4 (closing the concrete `Top`/`Bottom` goals K4 cannot), it does
  not re-pin the elimination.
- **The forbidden elim-*out*-of-`Omega`** (projecting a proof-relevant Ω
  inhabitant into a relevant `Type` — the general large-elim danger) is the
  restricted **out**-direction (`16 §1.1`). K5's `absurd` narrows *into* `Omega`
  (a vacuous elim of the empty prop), **distinct** from it;
  `absurd-neutral-no-new-rule` pins the no-leak posture, not the out-direction
  restriction.
- **The lawful-classes complete accept arm + the `(gated: K5)` tags** are
  **`../../stdlib/classes/seed-lawful-classes.md`'s**; K5 provides the
  **capability** that un-gates them (`antisym-two-branch-tt-and-absurd`), it
  does not re-pin the lawful-instance net. The un-stage of those tags is task
  #36 (on K5 merging to main).
- **The SCT/guardedness termination mechanism** is
  **`../judgments/seed-judgments.md`'s** (the K2c SCT seam). This seed pins
  **one new obligation** on it — that `collect_calls` traverses the new `absurd`
  positions — it does not re-pin the SCT algorithm.
