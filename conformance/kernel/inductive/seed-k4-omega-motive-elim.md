# K4 — elimination into `Ω` (Ω-motive elimination) conformance — seed cases

Format: `../../README.md`. These pin the **K4** kernel capability
(`spec/10-kernel/14-inductive.md §3` "Elimination into `Ω`", with the
formation-vs-elimination note in `16 §1.1`): the general eliminator's motive
codomain is a **sort** — `Type ℓ'` (type-selecting, unchanged) **or** `Ω_l`
(new). An `Ω`-codomain motive `M : (Δ_i) → D Δ_p Δ_i → Ω_l` proves a
**per-branch-varying proposition** by case-split on a relevant inductive
scrutinee — what makes a **∀-law over an inductive carrier provable** (the
lawful-structure-classes law fields, `50-stdlib/51 §5`/`§6`).

## Grounding (content-verified against the landed targets)

- `14 §3` (landed) — the eliminator's motive codomain is a **sort**
  (`Type ℓ' ∪ Ω_l`, **not** a wildcard); the **`Ω`-codomain** rule;
  **ι-reduction sort-agnostic** (no new reduction path); the 4-point soundness
  (motive type well-formed via predicative Π-formation; predicative `Ω` ⇒ no
  impredicative large-elim danger; **narrows *into* `Ω`**; proof irrelevance
  preserved for free, no new conversion rule).
- `16 §1.1` (landed) — **formation** (what *enters* `Ω`, `§1.3` sub-singletons
  only) vs **elimination** (a `Type`-inductive *into* an `Ω`-motive) — distinct;
  the latter is the safe narrowing.
- `16 §1.2` — conversion at an `Ω`-type is **definitionally irrelevant** (the
  shortcut the conv-embedding case rides).
- `50-stdlib/51 §5`/`§6` — the lawful-classes law fields K4 makes provable; the
  `../../stdlib/classes/seed-lawful-classes.md` **`(gated: K4)`** accept arm
  this capability un-gates.

## Scope — the eliminator's Ω-codomain rule and its soundness

These pin the **new admissibility** (an `Ω`-codomain motive is accepted), its
**exact boundary** (a **sort**, not any type), its **computation** (ι is
sort-agnostic), and the **load-bearing soundness** (into-`Ω` narrows, preserving
proof irrelevance through conversion — no leak *out* of `Ω`). The two shapes
that **are** restricted — *declaring* a proof-relevant inductive **at** `Ω`
(`16 §1.3`) and *singleton-eliminating* an `Ω`-inhabitant **out into** a
relevant `Type` — are **distinct** from this and homed elsewhere (referenced
below, not re-pinned).

**Tags.** `(soundness)` — a kernel admissibility/irrelevance commitment (a wrong
`Ω`-elim rule is a soundness hole: over-broad admits a non-sort codomain; a
leak-out breaks proof irrelevance). `(oracle)` — the concrete term spellings
(`elim_Bool`, `bool_leq`, `IsTrue`). The **admissibility boundary, the
sort-agnostic ι, the irrelevance-through-conversion, and every verdict** are
**normative**.

---

## The capability — an Ω-motive elimination proves a per-branch law

### kernel/inductive/omega-motive-elim-proves-per-branch-law (soundness)
- spec: `14 §3` (elimination into `Ω`), `16 §1.1` (formation vs elimination),
  `50-stdlib/51 §5`
- given: `elim_Bool` at the **`Ω`-motive**
  `M := λx. IsTrue (bool_leq x x) : Bool → Ω_0` (`Bool` real
  `data Bool = True | False`; `bool_leq` a `declare_def` structural match;
  `IsTrue b := Eq Bool b True : Ω`), with per-constructor proof methods
  `m_True : IsTrue (bool_leq True True)` and
  `m_False : IsTrue (bool_leq False False)` (each `bool_leq c c ⇝ True`, so the
  method is the reflexivity proof) — yielding
  `refl : (x : Bool) → IsTrue (bool_leq x x)`
- expect: **accepts** — the `Ω_0`-codomain motive is admissible (`14 §3`), the
  eliminator type-checks, and `refl` is a **real kernel proof** of the ∀-law.
  **Verified flip (anti-green-vs-green):** on the **pre-K4** kernel the same
  term is **rejected** (the eliminator's codomain check admitted only `Type ℓ'`,
  so an `Ω_0`-codomain motive was refused) — the verdict genuinely **flips** on
  the K4 rule, not for an incidental reason
- why: (soundness) K4's raison d'être — an `Ω`-motive elimination proves a
  **per-branch-varying** proposition by case-split on a relevant inductive, so a
  ∀-law over an inductive carrier becomes provable. This is **exactly** the
  capability the lawful-structure-classes need (`51 §5`/`§6`): it **un-gates**
  the `(gated: K4)` accept arm of
  `../../stdlib/classes/seed-lawful-classes.md`'s
  `law-fields-real-proofs-not-postulates` (real law proofs → zero-delta). Assert
  the **observable**: the eliminator **admits** (a real proof term of an `Ω`
  proposition), not "it resolves". The motivating positive; its pre/post-K4 flip
  makes it discriminating.

---

## The exact boundary — a sort, not a wildcard

### kernel/inductive/non-sort-motive-codomain-rejected (soundness)
- spec: `14 §3` (admissible codomain **exactly** `Type ℓ' ∪ Ω_l`)
- given: an eliminator on `Bool` under three motive codomains: (a)
  `M : Bool → Ω_0` (an **`Ω`-sort** codomain); (b) `M : Bool → Type 0` (a
  **`Type`-sort** codomain); (c) `M : Bool → Bool` — a **non-sort** codomain
  (`Bool` is a `Type`-*inductive*, not a sort)
- expect: **the verdict splits on sort-ness.** (a) and (b) **accept** — both
  codomains whnf to a **sort**; (c) **rejected** — a non-sort codomain is
  refused, `14 §3`'s admissible set is **exactly** `Type ℓ' ∪ Ω_l`
- why: (soundness) K4 relaxes the codomain **to sorts**, **not** to an arbitrary
  type — the guard is "**the motive's codomain whnf's to a sort**". **Non-
  degenerate pair** keyed on the structural discriminator *codomain-is-a-sort*
  (COORDINATION §7): the accept arm alone is green-vs-green under a relaxation
  that dropped the sort check entirely (a **wildcard** codomain); the **`Bool`
  reject arm is the guard**. **Disconfirming check:** would (c) also reject if
  its codomain were a *sort*? **No** — (a)/(b) accept — so the reject is gated
  on non-sort-ness, not coincidental. Matches the kernel's ground-truth reject
  test (a non-sort codomain like `Bool` stays rejected).

---

## The computation — ι is sort-agnostic

### kernel/inductive/omega-motive-iota-is-sort-agnostic
- spec: `14 §3` (ι-reduction sort-agnostic, no new reduction path)
- given: `elim_Bool M_Ω m_True m_False True` where `M_Ω : Bool → Ω_0`, and (for
  contrast) `elim_Bool M_T m_True' m_False' True` where `M_T : Bool → Type 0`
- expect: **both reduce-to their `True`-branch method** — the `Ω`-motive elim
  `⇝ m_True` **exactly as** the `Type`-motive one `⇝ m_True'` (the same
  constructor-selects-method ι-rule). The ι-reduction is **sort-agnostic** — it
  **never inspects the motive's codomain sort**, and **no new reduction path**
  is added
- why: (structural, not a value-flip) K4 adds admissibility, **not**
  computation: the ι-rule is identical across motive sorts. Assert the
  **structural reduct** (the selected method), **sort-independent** — a bug
  adding a sort-specific ι-path, or one that **fails to reduce** an `Ω`-motive
  elim (stuck), fails here. Matches the kernel's **identical-pre/post `whnf`**
  test (the `Ω`-motive addition leaves the reduction relation unchanged). The
  reduct is the selected method regardless of `M`'s sort — the observable the
  bug perturbs.

---

## The soundness — into-`Ω` preserves proof irrelevance (no leak out)

### kernel/inductive/omega-elim-conv-embedding-commutes (soundness)
- spec: `14 §3` (soundness point (iv): proof irrelevance preserved, no new
  conversion rule), `16 §1.2` (definitional irrelevance at an `Ω`-type)
- given: two `Ω`-motive eliminations at the **same** motive `M : Bool → Ω_0` but
  **different** proof methods — `e₁ := elim_Bool M m_True m_False x` and
  `e₂ := elim_Bool M m_True' m_False' x`, where `m_True, m_True' : M True` and
  `m_False, m_False' : M False` are **distinct proof terms** of the same `Ω`
  propositions; and — the **foil** — (c) the **same shape at a `Type`-codomain**
  motive `M_T : Bool → Type 0` with **genuinely distinct reducts**
  `e_T1 := elim_Bool M_T a b x` and `e_T2 := elim_Bool M_T a' b' x`, where the
  branch methods reduce to **distinct values** (`a ≠ a'` as `Type 0` inhabitants
  — proof-**relevant**, not proof-irrelevant)
- expect: **`e₁` and `e₂` convert** (`e₁ ≡ e₂` definitionally) — both are typed
  `M x : Ω_0`, and conversion at an `Ω`-type is **definitionally irrelevant**
  (`16 §1.2`), so the **which-method / which-proof distinction does not leak
  back through conversion**. Assert the **observable**: conversion **succeeds**
  (the irrelevance short-circuit fires on the **`Ω`-type**, upstream of the
  term), **not** a term-structural comparison of the methods. **Foil (c):**
  `e_T1`, `e_T2` do **not** convert (`convert == false`) — at a `Type`-codomain
  the reducts are proof-**relevant**, so distinct values are distinguished. The
  foil proves conversion is **discriminating**, so the Ω-side equality
  (`e₁ ≡ e₂`) is a **real proof-irrelevance property**, not a degenerate
  always-true `convert`
- why: (soundness ★) the load-bearing property of into-`Ω` elimination — it
  **narrows into `Ω`** (a relevant scrutinee → a proof-irrelevant result), so no
  which-proof content leaks **out** through conversion. **The commutation:** the
  `Ω`-motive elimination **commutes with the conv-embedding** — embedding either
  result into the conversion checker yields the **same** (irrelevant) verdict,
  because the shortcut fires on the `Ω`-**type**, not the term; this is why the
  soundness obligation is **entirely typing-admission** and needs **no new
  conversion rule** (the `conv.rs` irrelevance shortcut is untouched).
  **Verdict-flip (anti-green-vs-green):** a conversion that compared the
  `Ω`-elim results **structurally** (proof-relevantly — distinguishing `e₁` from
  `e₂` by their methods) would make them **conv-distinct**, **leaking
  which-proof info out of `Ω`** — the classic large-elimination danger. Correct
  impl → **conv-equal**; the proof-relevant-`Ω`-elim bug → **conv-distinct**:
  opposite observables. Pins that the elim result is **typed at `Ω`** (so the
  shortcut applies) — the "no leak out" half that distinguishes safe into-`Ω`
  elimination from the restricted singleton-elimination-**out**-of-`Ω`.
  **Non-degenerate pair (the foil makes the positive non-vacuous):** Ω-side
  distinct-methods → **convert** paired with the `Type`-side foil
  distinct-reducts → **do not convert** — without the foil a broken always-true
  `convert` would pass the Ω-positive for the wrong reason; the pair keys the
  irrelevance on the codomain sort (`Ω` irrelevant / `Type` relevant), mirroring
  the kernel's fold-now foil test
  (`omega_pi_shortcut_fires_on_distinct_proofs_not_alpha`).

---

## Coverage map

- **Capability (soundness):** `omega-motive-elim-proves-per-branch-law` (the
  pre/post-K4 flip; un-gates the lawful-classes accept arm).
- **Boundary (soundness):** `non-sort-motive-codomain-rejected` (sort, not
  wildcard).
- **Computation:** `omega-motive-iota-is-sort-agnostic` (no new reduction path).
- **Soundness ★:** `omega-elim-conv-embedding-commutes` (irrelevance preserved
  through conversion; no leak out of `Ω`).

## Cross-case consistency sweep

- **`Ω`-motive elimination is admissibility-only — never a new reduction or
  conversion rule.** `omega-motive-iota-is-sort-agnostic` (ι unchanged) and
  `omega-elim-conv-embedding-commutes` (conv unchanged, the existing `16 §1.2`
  shortcut) agree: K4 touches **only** the eliminator's typing-admission (the
  codomain-sort check). A case adding a sort-specific ι-path, or a new
  `Ω`-elim-specific conversion rule, would contradict this — and the soundness
  argument (`14 §3` (iv)) that rests on it.
- **The boundary is a *sort*, both directions.**
  `non-sort-motive-codomain-rejected` (a non-sort codomain rejects) and
  `omega-motive-elim-proves-per-branch-law` / the `Type`-motive baseline (a
  sort codomain accepts) agree: the admissible codomain is **exactly**
  `Type ℓ' ∪ Ω_l`. A case admitting a non-sort codomain (a wildcard) or
  **re-rejecting** a valid `Ω`-sort codomain (the pre-K4 incompleteness)
  contradicts the pinned boundary.

## Subsumed / not-duplicated (one home per property)

- **Declaring a proof-relevant inductive *at* `Ω`** (`16 §1.3` — only
  sub-singletons enter `Ω`; a proof-relevant `Type → Ω` is inadmissible) is the
  **formation** axis, homed in **`../observational/`** (the `Perm`-at-`Ω` /
  Ω-admissibility net). K4 is the **converse** (eliminating a `Type`-inductive
  *into* an `Ω`-motive) — `16 §1.1` explicitly disambiguates them; this seed
  references the distinction, does **not** re-pin formation.
- **Singleton-elimination *out of* `Ω`** (projecting proof content from an
  `Ω`-inhabitant into a relevant `Type` — the leak K4 does **not** perform) is
  the restricted **out**-direction; `omega-elim-conv-embedding-commutes` pins
  the **no-leak-out** property from the into-`Ω` side, and does not re-pin the
  out-direction restriction (kernel `16`).
- **The general eliminator / ι-reduction machinery, `elim_D`, `match`→`elim_D`**
  are **`../../surface/data-match/`'s** and the base kernel inductive seed's; K4
  pins only the **`Ω`-codomain** extension, referencing the base ι-rule.
- **The lawful-classes law fields + the zero-delta accept arm** are
  **`../../stdlib/classes/seed-lawful-classes.md`'s**; K4 provides the
  **capability** that un-gates them, it does not re-pin the lawful-instance net.
