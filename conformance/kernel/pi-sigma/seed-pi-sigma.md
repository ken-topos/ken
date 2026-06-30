# Π / Σ formation, projection, and η — conformance (Σ-sort erratum)

Format: `../../README.md`. These pin `spec/10-kernel/13-pi-sigma.md` — the
predicative formation rules, dependent second projection, and **both η rules** —
with the **Σ-sort discriminating pair** that is the executable third of the
**Σ-sort soundness erratum** (`13 §4`/§5, Architect-confirmed). Expected results
are grounded in the **landed** `13` (`wp/V1-sigma-sort`), `16 §1` (the
strict-prop
Ω layer), and first principles; the prototype is not mounted and none of these
required it.

**The erratum in one line (`13 §4`).** The strict-prop landing is **not** a
uniform codomain rule: `sort_pi(s1, s2) = Ω` iff `s2 = Ω` (codomain-keyed — a
function *into* a proposition is a proposition, regardless of the domain), but
`sort_sigma(s1, s2) = Ω` iff **`s1 = Ω ∧ s2 = Ω`** (both-components-keyed — a
subset `Σ` with a **relevant** first component carries *content* and must stay
in
`Type`). The pre-erratum kernel keyed Σ on the codomain too, which over-admits a
subset `Σ(Type, Ω)` into Ω — an Architect-confirmed trust-root over-equating
hole
(the `Σ`-analogue of the Ω-PI shortcuts). This file pins **both directions** so
the fix is neither under- nor over-corrected.

This corpus **leads** the kernel: the `sort_sigma` split is kernel-leader's
lane,
and this guard + the `13` spec erratum + that kernel change land **together** on
one Decision with all three verified on `main` (the Architect's 3-piece gate —
the same discipline that caught the §5.1 erratum shipping only 1 of 3 pieces).

Cases tagged **(soundness)** encode the strict-prop landing the kernel's
consistency rests on (`13 §4`, `16 §1.2`) and must never regress.

---

## A. The strict-prop landing — the Σ-sort discriminating pair (`13 §4`/§5)

### pi-sigma/sigma-subset-relevant-stays-type (soundness)
- spec: `13 §4`/§5 (`sort_sigma` Ω iff both Ω); `16 §1.2` (Ω-PI), `§2.2`
- given: the subset `Σ(Bool, λ _. Top)` = `(b : Bool) × Top` — a **relevant**
  first component (`Bool : Type 0`) and an Ω second (`Top : Ω`); the two pairs
  `(true, tt)` and `(false, tt)`
- expect: the type **forms at `Type 0`**, **not** Ω; consequently
  `(true, tt) ≢ (false, tt)` (they are **not** convertible), and **no** closed
  inhabitant of `Empty` is derivable.
- why: a subset with a relevant carrier carries content, so `sort_sigma` keys on
  **both** components (`13 §4`) and lands in `Type`. **Verdict-flips on the sort
  rule** — re-derived, not transcribed: under the **pre-erratum** codomain-keyed
  `sort_sigma` the type lands in **Ω**, so by Ω-PI (`16 §1.2`)
  `(true, tt) ≡ (false, tt)`; then with motive `M := λ z. Eq Bool (z.1) true` —
  `M (true, tt) = Eq Bool true true` (inhabited by `refl`) — transport along
  that convertibility gives `M (false, tt) = Eq Bool false true ⇝ Bottom`
  (`§2.2`, distinct closed Bools), a **closed inhabitant of `Empty`**. The
  corrected kernel (`Type 0`) keeps `(true, tt) ≢ (false, tt)`, so the transport
  never forms. **Structural discriminators (three, all flip):**
  `infer(Σ(Bool, λ_.Top)) = Type 0` (not Ω);
  `convert((true, tt), (false, tt)) = false` (not true); no closed `Empty`.
  **Absence-assertion (no closed `Empty`):** guard = `sort_sigma` keys on
  **both** components; disconfirming — would `(true, tt) ≢ (false, tt)` still
  hold under the codomain-keying bug? **No** — that bug makes the Σ an Ω and
  collapses them, which is exactly what manufactures the `Empty`. Green
  (`Type 0`, distinct, no `Empty`) vs red (`Ω`, collapsed, closed `Empty`)
  precisely on the sort rule.

### pi-sigma/sigma-conjunction-both-omega-stays-omega (soundness)
- spec: `13 §4`/§5; `16 §1.3` (conjunction in Ω), `§1.2` (Ω-PI)
- given: the conjunction `Σ(Top, λ _. Top)` = `(_ : Top) × Top` — **both**
  components `Top : Ω`
- expect: the type **forms at Ω** (it **is** a proposition, the conjunction
  `Top ∧ Top`); any two inhabitants are convertible by Ω-PI.
- why: both-components-Ω is the **sound** Ω-landing (`13 §4`: `sort_sigma = Ω`
  iff both Ω; `16 §1.3` conjunction). **The other direction of the pair:** with
  `sigma-subset-relevant-stays-type`, this pins `sort_sigma` is **neither under-
  nor over-corrected** — a "fix" that pushed **all** Σ to `Type`
  (over-correction) would wrongly make this conjunction a `Type`, losing the
  propositional proof-irrelevance that `16` relies on. **Verdict-flips on the
  sort:** correct → Ω here; a both-to-`Type` over-correction → `Type`.
  (Internal-consistency tie: green-vs-red here guards the *opposite* mistake
  from the case above, so the two bracket the rule.)

### pi-sigma/pi-into-prop-is-prop-codomain-keyed (soundness)
- spec: `13 §4` (Π codomain-keyed); `16 §1.1`
- given: `(x : Bool) → Top` — a **relevant** domain (`Bool : Type 0`) and an Ω
  codomain (`Top : Ω`)
- expect: the type **forms at Ω** — a function **into** a proposition is itself
  a proposition, **regardless of the domain's sort**.
- why: Π lands in Ω iff its **codomain** is Ω (`13 §4`, codomain-keyed) — the
  sound rule the erratum **preserves** for Π (only Σ changes).
  **Internal-consistency contrast (the load-bearing pin that the erratum splits
  Π vs Σ correctly):** same components, different former, different sort —
  `(x:Bool) → Top : Ω` (Π, codomain-keyed) but `(b:Bool) × Top : Type 0` (Σ,
  both-keyed, case A.1). A kernel that wrongly "fixed" Π to both-keyed too would
  make this `(x:Bool) → Top` a `Type` — **verdict-flips on the sort** under that
  over-correction.

---

## B. Dependent projection and the dependent-Σ regression (`13 §5`)

### pi-sigma/dependent-second-projection-typing (soundness)
- spec: `13 §5` (`p.2 : B[p.1/x]`; reject the non-dependent shortcut), `13 §2`
  (Σ-Elim)
- given: a dependent pair `p : (n : Nat) × Vec Bool n` with
  `p := (2, [true, false])`; the projections `p.1`, `p.2`
- expect: `p.1 ⇝ 2 : Nat`; `p.2` types at **`Vec Bool p.1` = `Vec Bool 2`** (the
  second-component type with `p.1` substituted) and `⇝ [true, false]`. A use of
  `p.2` where `Vec Bool 0` (or an un-substituted `Vec Bool n`) is expected is
  **rejected**.
- why: dependent Σ-Elim types `p.2` at `B[p.1/x]` (`13 §5`); the index travels
  from the first component. **Structural verdict-flip:** the projected type is
  the **substituted** `Vec Bool 2`. A **non-dependent shortcut** that ignored
  the dependency (typing `p.2` at a fixed/free index) is exactly what `13 §5`
  requires the kernel to reject — so a correct kernel accepts `p.2 : Vec Bool 2`
  and rejects `p.2 : Vec Bool 0`, the shortcut kernel does the opposite. Also
  the dependent-`B` **regression**: the formation `(n : Nat) × Vec Bool n`
  type-checks at the predicative `Type (max …)` and projects correctly.

---

## C. Both η rules in conversion (`13 §5`, `17 §2`)

### pi-sigma/pi-eta-conversion (soundness)
- spec: `13 §5` (Π-η); `17 §2` (typed η)
- given: `A : Type 0`, `f : (x : A) → A`; the η-expansion `λ x. f x`
- expect: `f ≡ λ x. f x` — they **convert** (`convert(f, λ x. f x) = true`).
- why: Π-η is part of definitional equality (`13 §5`, `17 §2`); η-expansion is
  type-driven. Structural: the conversion query returns true. A kernel missing
  Π-η leaves the neutral `f` un-equal to its expansion — the query flips to
  false. (Consistent with `../conversion/seed-conversion.md`-adjacent
  `judgments/conv-switch-eta`.)

### pi-sigma/sigma-eta-conversion (soundness)
- spec: `13 §5` (Σ-η); `17 §2`
- given: `p : (x : A) × B` (a neutral variable of a Σ-type); the η-expansion
  `(p.1, p.2)`
- expect: `p ≡ (p.1, p.2)` — they **convert** (`convert(p, (p.1, p.2)) = true`).
- why: Σ-η is part of definitional equality (`13 §5`) — a pair is its
  projections re-paired. Structural: the conversion query returns true; a kernel
  missing Σ-η leaves the neutral `p` un-equal to `(p.1, p.2)` — flips to false.
  Together with projection-β (`p` a literal pair) this makes Σ equality
  componentwise (`16 §2`).

---

## D. Regression — K1 Π/Σ unchanged

### pi-sigma/k1-pi-sigma-still-green (soundness)
- spec: `spec/10-kernel/README.md §6`; `13 §6` (K1 conversion + subject
  reduction)
- given: the existing K1 Π/Σ cases (`../seed-k1.md`, `../seed-kernel.md`) —
  formation, β, projection-β, application
- expect: **all pass** — the erratum refines only the Σ-formation **sort**
  (`sort_sigma` both-keyed), not β, projection-β, η, or the typing relation.
- why: the Σ-sort split is an admission-time sort computation; it changes which
  universe a subset `Σ` inhabits, not how Π/Σ reduce or project. K1's Π/Σ
  behavior must not regress. Mirrors `../judgments/seed-judgments.md`
  `judgments/k1-k2-judgments-still-green`.

---

## Coverage map (`13 §5`)

- **Σ-sort discriminating pair (both directions)** —
  `sigma-subset-relevant-stays-type` (relevant → `Type`, the soundness core) +
  `sigma-conjunction-both-omega-stays-omega` (both-Ω → Ω); bracketed by
  `pi-into-prop-is-prop-codomain-keyed` (Π stays codomain-keyed — the split is
  correct).
- **dependent `p.2` / dependent-`B` regression** —
  `dependent-second-projection-typing`.
- **both η** — `pi-eta-conversion`, `sigma-eta-conversion`.
- **K1 regression** — `k1-pi-sigma-still-green`.

Build-sequencing: this guard rides `wp/V1-sigma-sort` with the `13` spec erratum
(`8533a63`) and kernel-leader's `sort_sigma` split; all three land on one
Decision,
verified together on `main` per the Architect's 3-piece gate. It is **not** part
of
WS-V V1 (`conformance/verify/spec-syntax/`), which is independent of this fix
(V1's carrier-plus-obligation encoding never forms a core `Σ` over an Ω
predicate).
