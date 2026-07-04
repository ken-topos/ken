# CAT-3 collection-laws conformance — seed cases

Format: `../../README.md`. Third WP of the **catalog campaign**
(`docs/program/06-catalog-campaign.md`;
`docs/program/wp/CAT-3-collection-laws.md`). CAT-3 = **Layer 1**: the landed
`packages/collections` ops get their **laws as propositions, proved not
postulated** (the CAT-1 lawful-class discipline over a **value carrier**), the
capstone **verified `sort`** (`isSorted` and `Perm`), and the agent-facing
**projection abstraction** (the catalog's "view" unit, renamed to avoid the
SURF-1-retired `view` keyword). This seed pins the discriminating conformance
for all three deliverables.

The **Architect's three fork rulings** (`evt_4c3q1e611va69`) are folded in:

- **Fork A — `Perm` = count/multiset-equality, Ω-native, NOT `‖Perm_rel‖`.** A Π
  into `Ω` of `Nat`-value equations is a proposition by the predicative max
  (`16 §1.1`), needing **zero truncation machinery** — and it sidesteps the hard
  unsoundness pin the **same way the landed `Ord.total` law does** (push
  proof-relevant content into a decidable `Bool`/`Nat`, keep the law a
  value-equation). A raw multi-ctor `data Perm … : Ω` is **inadmissible**
  (proof-relevant, so `Ω`-PI collapse, so `Type → Ω` admits `true ≡ false`).
- **Fork B — plain `Σ`-record (lawful-class shape) per concrete flavor.** The
  concrete `lens`/`iso`/`refinement`/`indexed`/`setoid-quotient` flavors are
  ordinary Ken records shipping now; `obligation-producing` is a stated seam
  (Lane B / L12 / L14, deferred); the **polymorphic** family (`Lens s a` etc.)
  is design-now / build-later, gated on a bounded multi-param-`class` wall.
- **Fork C — family `optic`, flagship flavor `lens`.** The exact surface token
  is **routed to Steward** (operator-facing ergonomics, the same axis SURF-1
  routed) — so this seed pins the **concept** (a non-colliding name for the
  Layer-1 unit) and `(oracle)`-tags the literal spelling, per the
  don't-over-freeze-a-deferred-token rule.

**Red-until-built posture (CAT-1-Functor precedent).** `map`, `filter`, `sort`,
`isSorted`, `count`, `mem`, `length`, `min`, and the `optic`/`lens` records are
**absent from `packages/` on `origin/main`** (self-grepped — see Grounding).
Every case whose subject is one of these is **red-until-built** against the
CAT-3 Language build (held for the GPT window); it flips green when the build
lands the op + its law. Cases over **only** landed ops (`list_append`, `take`,
`drop`, `nth`) are **provable-now** (need only the proof authored + built), and
are tagged distinctly from the deeper **blocked-on-missing-op** reds.

## Grounding (content-verified against the landed targets)

- `packages/collections/collections.ken` — **landed**: `list_append`
  (`view list_append (a) (xs ys : List a) : List a`, recursing on its **first**
  argument, so **left unit is definitional**, right unit + assoc inductive — the
  CAT-1 monoid proofs this reuses are grounded on exactly this recursion),
  `nth`, `take`, `drop`, `natSub`, `list_eq (eqf : a -> a -> Bool)`,
  `list_compare (cmp : a -> a -> OrdResult)`, `data OrdResult = Lt | Eq | Gt`.
  **NOT landed anywhere in `packages/`** (via `git grep`): `count`, `map`,
  `filter`, `sort`, `isSorted`, `mem`, `length`, `min` — hence the
  red-until-built split above.
- `packages/lawful-classes/lawful_classes.ken` — the lawful-class pattern
  (`class` = `Σ`-record, op fields `Type`-valued, law fields `Ω`-valued;
  instance = `declare_def` record value re-checked by the kernel). **The Fork-A
  precedent is landed here:** `Ord.total` is stated
  `total : (x)(y) → IsTrue (bool_or (leq x y) (leq y x))` — a **decidable Bool
  equation**, chosen (verbatim comment) *because* a bare propositional
  disjunction "would be proof-relevant (which side holds is content) and need
  `||.||` to reach Omega." Count-equality is that identical move for `Perm`.
  `class DecEq a { eq : a -> a -> Bool ; sound ; complete }` (`:25`), whose
  `sound`/`complete` tie `eq` to kernel `Equal` — so a `count` deciding
  element-equality via `DecEq a` counts up to the **same** equality the `Perm`
  law is about (no equality-mismatch hole).
- **Carrier soundness (CV caveat, Architect-bound):** `instance DecEq Int`,
  `instance Ord Int` are **all-`Axiom`** (`Int` is a K1 primitive: its
  universally-quantified laws are not kernel-provable, honest visible
  postulates); `Ord Char` is **zero-NEW-delta by transport** — it references
  `Ord Int`'s `Axiom`s, not Axiom-free. **Only `Bool` is the zero-delta
  exemplar** — `DecEq Bool` and `Ord Bool` are real, kernel-checked, Axiom-free
  proofs (finite case-split). ⇒ the **proved** arms carry on `List Bool`.
- `16 §1.1` — predicative universe max (a Π into `Ω` is in `Ω`); `16 §1.2` —
  `Ω` proof-irrelevance (SProp); `16 §1.4` — the `Type → Ω` unsoundness pin
  (`Bool` gives `true ≡ false`), the **actual** home of the raw-`Perm` argument
  (spec-author's correction — the frame's `§1.3` is the derived-connectives /
  truncation route, adjacent-but-not-the-pin); `16 §5` — quotients (kernel has
  `Term::Quot`/`QuotClass`/`QuotElim`, **no surface intro path**); `16 §6` —
  `‖·‖` truncation (the route Fork A did **not** take).
- `55 §3.1` — the two-line induction+`cong` proof grammar; `55 §3.2` — the
  `tt`-vs-`Refl` endpoint discrimination, **per-branch not uniform**
  (constructor-headed gives `Top` gives `tt`; neutral gives stuck `Eq` gives
  `Refl`).
- `22 §2` / `ast.rs` `TRefine` / `elab.rs` refinement lowering — refinement
  types `{ x : A | P x }` lower to the carrier + a kernel-re-checked obligation
  (the `refinement` optic flavor + the verified-`sort` result type ride this).
- `conformance/challenge/C5-verified-sort/` — **corroborates D2**: the landed
  ES2 `sort` result refinement type-checks in `es2_acceptance.rs`; the
  `const Nil` unsound arm is the canonical Perm-load-bearing discriminator.
  **Known-gap** carried from C5: full **Perm proof discharge** (the prover
  closing `Perm (insert …) …`) is not landed — *emission of both conjoined
  obligations* is the checkable property today; the ACCEPT arm's full discharge
  is red-until-built.

## Scope — canonical shapes, and the provable-now / red split

The verified-`sort` cases reference these shapes (spelled once here; the exact
field/signature spellings reconcile against the landed CAT-3 chapter body):

```
sort    (a) (leq) (xs) : { ys : List a | And (isSorted a leq ys) (Perm a ys xs) }
sortBad (a) (leq) (xs) : <same refinement> = Nil a     -- non-permuting stub
sortId  (a) (leq) (xs) : <same refinement> = xs        -- non-ordering (identity)
Perm      a xs ys := (x : a) -> Equal Nat (count x xs) (count x ys)   -- Fork A
eqFromOrd x y    := bool_and (leq x y) (leq y x)   -- count-eq from Ord alone
```

- **D1 structural laws** (AC5): `take`/`drop` decomposition #1 **provable-now**
  (landed ops); the `length`/`min` decomposition #2, `map` length-preservation,
  `filter` membership are **red-until-built** (op unlanded).
- **D2 verified `sort`** (AC3/AC4): the count-equality `Perm`, `isSorted`, and
  the two dual flips (`sortBad` fails `Perm`; `sortId` fails `isSorted`), on
  `List Bool`. Full ACCEPT-arm discharge red-until-built (C5 known-gap).
- **D3 `optic`** (AC6): the concrete `lens` coherence flip, the per-flavor
  mechanism enumeration, the name reconcile.
- **Carrier:** `List Bool` for every proved obligation; `List a` /
  comparator-parametric only where no concrete lawful instance is needed.
- **The Fork-C literal token is `(oracle)`** pending Steward; this seed uses
  `optic`/`lens` (Architect's recommendation) as the working spelling.

---

## AC4 / AC3 — verified `sort`: count-equality `Perm` and the two dual flips

### stdlib/collections/verified-sort-emits-both-conjuncts (soundness)
- spec: CAT-3 §D2 (verified `sort`), `22 §2` (refinement obligation),
  `55 §3.1` (induction+`cong`), Fork-A ruling (`Perm` = count-equality); C5
  `sound-verified-sort.ken` (corroborating shape, `es2_acceptance.rs`)
- given: the landed explicit-comparator insertion `sort` over `List Bool` with
  the conjoined refinement result type (Scope), `leq` the real `Ord Bool`
  comparator; input a concrete `List Bool` (e.g.
  `Cons True (Cons False (Cons True Nil))`)
- expect: **elaborates and emits the conjoined obligation with BOTH conjuncts
  present** — the `And` head over `isSorted a leq (sort …)` and
  `Perm a (sort …) xs`. Assert the **specific observable**: the emitted
  obligation carries *both* conjuncts (grep the emitted core term for the `And`
  head over the two named predicates), and the carrier is `List Bool` (real
  `Ord`/`DecEq`, so the discharge is Axiom-free). **RED-UNTIL-BUILT** on two
  axes: `sort`/`isSorted` unlanded (op), and full **Perm discharge** is the C5
  known-gap — the checkable property today is *emission of both conjuncts*, not
  closed proof
- why: (soundness) the ACCEPT arm of AC4. It anchors the two REJECT flips below
  — each holds `sort`'s type fixed and breaks exactly one conjunct. Emission of
  **both** conjuncts is load-bearing: an `isSorted`-only obligation is
  green-vs-green (`sortBad` passes it). The carrier is `List Bool` per the
  caveat: on `List Int` the honest discharge cannot be Axiom-free and the flip
  degenerates ([[green-vs-green-does-not-confirm-a-fix]]: the ACCEPT arm must be
  genuinely dischargeable, not vacuously red)

### stdlib/collections/nonpermuting-sort-const-nil-fails-perm (soundness)
- spec: CAT-3 §D2 (a non-permuting "sort" must fail `Perm`), Fork-A ruling
  (count-equality), `16 §1.1`; C5 `unsound-const-nil.ken`
- given: `sortBad` (Scope) — discards its input, returns `Nil a`, claims the
  **same** refinement as the real `sort`; input a **non-empty** `List Bool`
  (e.g. `Cons True (Cons False Nil)`)
- expect: **verdict flips vs the honest `sort`, keyed on the `Perm` conjunct.**
  **Rejected — at the `Perm` conjunct**: `Perm a (Nil a) xs` unfolds to
  `(x : Bool) → Equal Nat (count x Nil) (count x xs)`; at `x = True`,
  `count True Nil = 0` but `count True xs = 1`, a false `Equal Nat 0 1` — no
  witness (multiset counts differ). The `isSorted` conjunct is **vacuously
  satisfiable** (`isSorted leq Nil` holds), so the rejection is **solely** the
  `Perm` conjunct. Assert the **specific observable**: elaboration fails at the
  **`Perm` conjunct** of the emitted `And` with a conversion / unprovable
  obligation on `Equal Nat 0 1` — **not** `is_err()`, not at `isSorted`, not a
  missing-field error ([[assert-specific-error-variant-not-is-err]])
- why: (soundness) AC4 — this is the discriminator that forces `sort` to *be* a
  sort. `isSorted`-alone is vacuous; the `Perm` conjunct (as count-equality) is
  what `const Nil` cannot satisfy. **Two-arm net**: a masked `Perm = Axiom` on
  `sortBad` postulates `Equal Nat 0 1` — a false equation on a concrete `Nat` —
  which fails the delta gate and inhabits `Bottom`. Verdict-flip vs
  `verified-sort-emits-both-conjuncts`, keyed on the `Perm` conjunct. Reuses the
  C5 `const Nil` shape

### stdlib/collections/nonordering-sort-identity-fails-issorted (soundness)
- spec: CAT-3 §D2 (a non-ordering "sort" must fail `isSorted`), `55 §3.2`
  (`tt`/`Refl` endpoints), Fork-A ruling
- given: `sortId` (Scope) — the identity "sort" (a genuine permutation, but does
  not order); input a **descending** `List Bool` under the `Ord Bool` order —
  the pair `[hi, lo]` (higher then lower per landed `Ord Bool.leq`; direction
  reconciled against the landed instance at authoring)
- expect: **verdict flips, keyed on the `isSorted` conjunct — the dual of the
  previous case.** The `Perm` conjunct **holds** — `Perm a xs xs` is
  `(x) → Equal Nat (count x xs) (count x xs)`, a **neutral** endpoint closing
  **`Refl`** (§3.2 neutral, not `tt`). The `isSorted` conjunct **fails**:
  `isSorted leq [hi, lo]` reduces to a conjunct requiring `IsTrue (leq hi lo)`
  = `IsTrue False`, uninhabited. Assert the **specific observable**: elaboration
  fails at the **`isSorted` conjunct** on `IsTrue False` — **not** at `Perm`,
  **not** `is_err()`
- why: (soundness) AC4 — the **dual** flip: it isolates the *other* conjunct, so
  the pair (this + `sortBad`) proves **each** conjunct is independently
  load-bearing (the multi-dimensional-guard rule: `Perm` and `isSorted` are two
  dimensions; each needs its own discriminating case — a sort broken on only one
  must fail at that one). `Perm`-holds-here vs `Perm`-fails-there, and
  `isSorted`-fails-here vs `isSorted`-holds-there, is the crossed pair. Endpoint
  `Refl`-vs-`tt` re-derived per §3.2, not transcribed

### stdlib/collections/perm-is-count-equality-not-raw-omega-inductive (property)
- spec: Fork-A ruling, `16 §1.1` (Π into `Ω`), `16 §1.2` (`Ω`-PI), `16 §1.4`
  (`Type → Ω` unsoundness pin — NOT `§1.3`), `lawful_classes.ken` `Ord.total`
  (the landed precedent)
- given: two candidate `Perm` representations: (a) the ruled count/multiset
  `Perm a xs ys` (Scope); (b) a raw multi-constructor `data Perm : … : Ω` (a
  `perm_nil`/`perm_skip`/`perm_swap`/`perm_trans`-style inductive relation
  declared directly at `Ω`)
- expect: **(a) is admissible and Ω-native; (b) is inadmissible.** Assert the
  **structural** observable: (a) `Perm` is a `Π`-type into `Ω` (`Equal Nat _ _`
  is `Ω`; the `Π` stays `Ω` by the predicative max `16 §1.1`), needs **no**
  `‖·‖` intro/elim, and is provable by the landed two-line grammar; (b) a
  proof-relevant multi-ctor inductive **cannot** be declared at `Ω` — distinct
  re-orderings are distinct derivations, `Ω`-PI (`16 §1.2`) collapses them, and
  unrestricted `Type → Ω` admits `Bool ⇒ true ≡ false` (`16 §1.4`). **Not** a
  verdict-flip: a `(property)` case pinning the ruling's representation —
  the AC3 soundness pin
- why: (property) AC3 — the load-bearing soundness call. Count-equality is the
  **same move as the landed `Ord.total`** (`bool_or` Bool-equation over a bare
  proof-relevant disjunction): push proof-relevant content into a decidable
  `Nat` count, keep the law a value-equation (subsume-don't-proliferate on a
  soundness pattern the stdlib already chose,
  [[proof-relevant-inductive-cannot-be-declared-at-omega]]). Stated as a
  **property** because the discriminator is the *representation choice*, not a
  witness — a build that declared `data Perm : Ω` would be blocked here

### stdlib/collections/perm-eq-derives-from-ord-no-separate-deceq (property)
- spec: Fork-A ruling pt 4, `lawful_classes.ken` (`Ord`/`DecEq`), `51`
- given: two ways `count` obtains its element-equality for the **sort capstone**
  (which already carries `Ord a` via `leq`): (a) `eqFromOrd` (Scope) — derived
  from `Ord a` alone (antisym gives `Equal` from `IsTrue`; refl + `Ord` gives
  the converse); (b) a **separate** `DecEq a` dict threaded alongside `Ord a`
- expect: **(a) suffices for `sort`'s `Perm`; (b) is redundant for the
  capstone.** Assert the structural observable: `sort`'s `Perm` law needs
  **no** extra `DecEq` dict beyond the `Ord a` it already has — `eqFromOrd`
  decides the same `Equal` (via `antisym`/`refl`). A **standalone** generic
  `Perm` (no ambient `Ord`) does take a `DecEq a` param. **Not** a verdict-flip:
  a `(property)` case grounding Architect pt 4 (no extra dict beyond the
  capstone's)
- why: (property) AC3/AC4 — pins the dict economy so the build does **not**
  proliferate a `DecEq` requirement onto `sort` that the order already
  discharges. Reconcile point: the exact `count`-eq threading (`eqFromOrd` vs a
  `DecEq` field) is a spec spelling — bind against the landed CAT-3 chapter

### stdlib/collections/verified-sort-proved-carrier-is-lawful-bool (property)
- spec: CV carrier caveat (Architect-bound, `evt_4c3q1e611va69`), CAT-3 §5
  (carrier breadth), `lawful_classes.ken` (`DecEq`/`Ord` Bool vs Int)
- given: the proved ACCEPT arm of the verified `sort` instantiated at (a)
  `List Bool` vs (b) `List Int`
- expect: **the proved arm must carry on `List Bool`.** Assert the structural
  observable: on (a) `List Bool` the honest `Perm`/`isSorted` discharge is
  **Axiom-free** (`DecEq Bool`/`Ord Bool` are real proofs); on (b) `List Int`
  the discharge **cannot** be Axiom-free (`Ord Int`/`DecEq Int` are all-`Axiom`)
  so the `sortBad` / `sortId` flips degenerate to **reject-vs-reject**
  (green-vs-green vacuity — both arms fail, the flip guards nothing). **Not** a
  verdict-flip: a `(property)` case that **guards the ACCEPT arm from vacuity**
- why: (property) AC4 — a discriminating flip is only real if the ACCEPT arm is
  genuinely dischargeable ([[green-vs-green-does-not-confirm-a-fix]]). Pinning
  `List Bool` as the proved carrier is what keeps the two sort flips above
  non-vacuous; `List Int`/generic is admissible only where the law is
  comparator-parametric and needs no concrete lawful instance. This is my
  grounding caveat, promoted to a standing case so the build cannot silently
  pick `List Int` and ship a vacuous green

---

## AC5 — D1 structural laws: `Ω`, pointwise, one canonical field

### stdlib/collections/take-drop-decomposition-holds (soundness)
- spec: CAT-3 §D1 (decomposition), `55 §3.1`/`§3.2`, `collections.ken`
  (`take`/`drop`/`list_append` landed)
- given: the law `list_append (take n xs) (drop n xs) ≡ xs` as a single `Ω`
  field, stated pointwise; proved by induction (on `n` then `xs`, matching the
  landed `take`/`drop` recursion); carrier `List Bool` (or generic `List a` —
  the law is element-agnostic)
- expect: **accepts — provable-NOW** (all three ops landed; needs only the proof
  authored + the CAT-3 package built). Endpoints per §3.2: the `Zero`/`Nil` base
  reduces both sides to `xs`/`Nil` — **constructor-headed gives `Top` gives
  `tt`**; the `Suc`/`Cons` step is inductive (IH + `cong` under `Cons`). Assert
  the observable: one canonical `Ω` field, the pointwise equation closes.
  **Tagged provable-now**, distinct from the blocked reds below
- why: (soundness) AC5 — the one D1 law statable+provable on landed ops today.
  Left-unit-definitional grounding (append recurses first arg) carries from
  CAT-1. Serves as the "provable-now" anchor against which the missing-op reds
  are contrasted

### stdlib/collections/take-length-law-red-until-built (property)
- spec: CAT-3 §D1 (`length (take n xs) ≡ min n (length xs)`), spec-leader
  sharpening (`length` AND `min` both unlanded)
- given: the length-of-take law `length (take n xs) ≡ min n (length xs)`
- expect: **RED-UNTIL-BUILT — blocked on missing ops** (`length` and `min` are
  both absent from `packages/`). The law cannot even be *stated* without them.
  Flips green when the CAT-3 build lands `length` + `min` + the proof. Assert
  the observable: the case is red for a **deeper** reason than the `take`/`drop`
  decomposition #1 — not "proof unwritten" but "operator unlanded"
- why: (property) AC5 — the provable-now / blocked-red distinction spec-leader
  surfaced (decomposition #1 provable, #2 blocked). Tagging **which** reds are
  op-blocked vs proof-pending is the elaboration-time deferred-seam tag
  ([[layer-dependent-pin-at-unconditional-layer]] discipline: name where the
  behavior is unconditional vs blocked)

### stdlib/collections/map-length-preservation-red-until-built (property)
- spec: CAT-3 §D1 (`length (map f xs) ≡ length xs`), CAT-1-Functor red posture
- given: the `map` length-preservation law `length (map f xs) ≡ length xs` (if
  `map` arrives via a `Functor List` instance, this is a law *about* the
  instance)
- expect: **RED-UNTIL-BUILT — blocked** (`map` and `length` both unlanded; no
  `Functor List` / standalone `map` on `List` exists in `packages/`). Flips
  green when the build lands `map` + `length` + the proof
- why: (property) AC5 — CAT-1-Functor posture verbatim; the `map` provenance
  (`Functor List` instance vs standalone) is an open CAT-3 §5 sub-decision the
  build pins

### stdlib/collections/filter-membership-red-until-built (property)
- spec: CAT-3 §D1 (`mem x (filter p xs) ⇔ (mem x xs ∧ IsTrue (p x))`)
- given: the `filter` membership law with `mem`/`⇔` as `Ω`-predicates
- expect: **RED-UNTIL-BUILT — blocked** (`filter` and `mem` both unlanded). The
  `⇔` is a two-sided `Ω`-implication (a pair of `Π`s into `Ω`, no truncation).
  Flips green when the build lands `filter` + `mem` + the proof
- why: (property) AC5 — the membership law is the D1 law whose statement is a
  bi-implication rather than an equation; pinning it as `⇔`-of-`Ω`-predicates
  (not a `Bool`-equation) keeps it in the value/`Ω` fragment (`55 §4`)

---

## AC2 — proved, zero `Axiom`; the append monoid REUSES CAT-1

### stdlib/collections/append-monoid-reuses-cat1-proofs (property)
- spec: CAT-3 §2 pin 2 / §6 (do-not-re-prove), `lawful_functors.ken`
  (`list_assoc`/`list_left_unit`/`list_right_unit`, landed CAT-1), CAT-1-build
  §6.1 (parametric-instance-head gate)
- given: the `Monoid (List a)` instance in two forms: (a) its associativity +
  unit law fields **reference** the landed generic `list_assoc`/
  `list_left_unit`/`list_right_unit` (CAT-1); (b) a variant that **re-proves**
  associativity/unit inline in the `collections` package
- expect: **(a) is the mandated form; (b) a proliferation defect.** Assert the
  structural observable: the `Monoid (List a)` law fields are **citations** to
  the CAT-1 proofs (grep: no duplicated `list_assoc`/`list_*_unit` proof terms
  in the CAT-3 package), zero `Axiom`. **The instance *form* is gated** on
  CAT-1-build's parametric-instance-head piece (`Monoid (List a)` is a
  parametric instance head, the `55 §6.1` gap), so the **instance** is
  red-until-that-lands while the **generic append proofs** are reusable today.
  **Not** a verdict-flip: a `(property)` case pinning reuse-don't-proliferate
- why: (property) AC2 — subsume-don't-proliferate: re-deriving the monoid laws
  would duplicate proof terms (a divergence surface) that CAT-1 already
  discharges once, generically. The gate on the parametric-instance-head keeps
  the honest split — proofs reusable now, instance-wiring pending the elaborator
  piece ([[class-dict-explicit-vs-implicit-abstract-tyvar]]: the explicit/landed
  path is available; the implicit/parametric-head path is the gated one)

---

## AC6 — D3: the `optic` abstraction

### stdlib/collections/lens-coherence-laws-hold (soundness)
- spec: CAT-3 §D3 (projection/lens flavor), Fork-B ruling (plain `Σ`-record over
  a concrete carrier), `33 §5.2` (record = `Σ`+η), `51 §5` (laws PROVED)
- given: a concrete `lens` as a lawful-class `Σ`-record
  `{ get : S -> A ; set : S -> A -> S ; get_set ; set_get ; set_set }` over a
  **concrete** carrier `S`/`A` (single/zero type param, the `Monoid`/`Ord`
  shape), with the three coherence laws (`get (set s a) ≡ a`;
  `set s (get s) ≡ s`; `set (set s a) b ≡ set s b`) as real proofs — e.g. the
  first-projection lens on a concrete pair record
- expect: **accepts** — the three coherence law fields re-check against the
  concrete `get`/`set`. Assert the observable: the record elaborates, all three
  coherence proofs close, zero `Axiom`. **RED-UNTIL-BUILT** (the `optic`/`lens`
  record is not in `packages/`; ships at the CAT-3 build)
- why: (soundness) AC6 — the ACCEPT arm for the shipped concrete flavor. It is
  ordinary Ken (Fork B: plain `Σ`-record, kernel-untouched), the exact
  lawful-class shape CAT-1 established. Anchors the broken-lens flip below

### stdlib/collections/broken-lens-setget-false-witness-rejected (soundness)
- spec: CAT-3 §D3, Fork-B ruling, `55 §3.2`; sibling of CAT-2's
  `applicative-map-coheres-with-wired-functor` (the coherence-flip shape)
- given: two `lens`-shaped records **identical** in type, differing only in
  `set` and the coherence proof it must satisfy: (a) the canonical lens (real
  proofs); (b) a **broken** lens whose `set s _ = s` (drops its value arg —
  well-typed at `S -> A -> S`), whose `get_set`/`set_get` proof is then false
- expect: **verdict flips on the coherence of `set`.** (a) **accepts**; (b)
  **rejected — conversion failure at the `get_set` (or `set_get`) field**: at a
  concrete carrier, `get (set s a) = get s` (the dropped `a`) while the law
  demands `≡ a`, a false `Equal A (get s) a` for `get s ≢ a`. Assert the
  **specific observable**: (b) fails **at the named coherence field** with a
  constructor/value clash — **not** `is_err()`, not a missing-field error
  ([[assert-specific-error-variant-not-is-err]])
- why: (soundness) AC6 — proves the coherence laws are **load-bearing** (a lens
  is `(get, set)` **plus proofs**, the verified-substrate discipline). **Two-arm
  net**: a masked `get_set = Axiom` on the broken `set` postulates a false
  `Equal` and inhabits `Bottom` via the delta gate. Verdict-flip, keyed solely
  on `set`'s coherence — the D3 analog of the D2 `Perm`/`isSorted` flips and the
  CAT-2 `map_coh` flip

### stdlib/collections/optic-flavors-mechanism-per-flavor-grounded (property)
- spec: Fork-B ruling (the per-flavor table), `22 §2` (refinement), `16 §5`
  (quotients, no surface intro), CAT-1 §6 (higher-kinded / multi-param `class`
  wall), Lane B / L12 / L14 (obligation seam)
- given: the six optic flavors — `projection (lens)`, `representation (iso)`,
  `refinement`, `indexed`, `quotient-respecting`, `obligation-producing` — each
  claimed against **landed** machinery
- expect: **each flavor's mechanism is grounded, not hand-waved "views are
  records".** Assert the structural observable, per flavor: `lens`/`iso`/
  `refinement`/`indexed` = plain `Σ`-record (concrete) → **ship now**;
  `quotient-respecting` has **two** forms — the **setoid-morphism** form
  `{ view : A -> B ; respects : (x y) -> R x y -> Equal B (view x) (view y) }`
  is a plain `Σ`-record → **now**, but the **quotient-carrier** form (a view
  *out of* `A/R`) needs a surface **quotient-intro** path the parser lacks
  (`16 §5` kernel formers exist, no surface) → **deferred**;
  `obligation-producing` is the **Lane B / L12 / L14 seam** → **boundary stated,
  deferred**; and the **polymorphic** forms (`Lens s a`, `Iso a b`) need a
  **multi-param `class`** (landed `class` takes one type param) → **design now,
  build fast-follow** (a bounded outer-ring extension, re-forks to Steward WHEN
  built). **Not** a verdict-flip: a `(property)` case pinning the per-flavor
  mechanism + what-ships-now
- why: (property) AC6 — Fork B mandates enumerating the mechanism per flavor
  against landed code, not asserting a single construct
  ([[sizing-a-subsume-fix-enumerate-every-piece]]: list each flavor's distinct
  vehicle and grep it). Surfacing the two design-now/build-later walls
  (surface-quotient-intro, multi-param-`class`) as a standing case keeps AC1
  honest — they re-fork to Steward at build time, not now

### stdlib/collections/optic-name-not-retired-view-keyword (property)
- spec: Fork-C ruling (family `optic`, flagship `lens`; token → Steward),
  SURF-1 (`view` keyword retired), CAT-3 §5 fork C
- given: the Layer-1 abstraction's name, against the SURF-1 decision that
  **retires the `view` keyword** (→ `fn`/`proc`/`const`)
- expect: **the abstraction's name does NOT collide with the retired `view`
  keyword.** Assert the observable: the shipped name is a non-colliding noun
  (`optic` family / `lens` flavor per Architect's recommendation), **not**
  `view`. The **exact surface token is `(oracle)`** — routed to Steward
  (operator-facing ergonomics); this case pins the **concept** (non-collision)
  and does **not** freeze the literal spelling. **Not** a verdict-flip
- why: (property) AC6 / fork C — pin the value-set + invariant (a non-`view`
  name), `(oracle)`-tag the deferred token, per the don't-over-freeze rule: a
  case that hard-froze `optic` would **falsely fail** a valid build if Steward
  rules `focus`/`projection` instead. Reconcile the final token against the
  landed CAT-3 chapter + Steward's ruling at the merge gate
  ([[reconcile-binds-a-co-reviewers-plausible-reading-too]])

---

## AC1 — kernel-untouched, outer-ring

### stdlib/collections/cat3-kernel-untouched-outer-ring (property)
- spec: CAT-3 §2 pin 5 / AC1, Fork-A/B rulings (no new capability)
- given: the full CAT-3 deliverable set — `count`/`map`/`filter`/`sort`/
  `isSorted`/`mem`/`length`/`min`, the count-equality `Perm`, the `optic`/`lens`
  records
- expect: **kernel-untouched.** Assert the structural observable:
  `git diff origin/main -- crates/ken-kernel/` is empty on the CAT-3 build; zero
  `trusted_base()` delta; no new `Term`/`Decl`. `count`/`sort`/`lens` are
  ordinary Ken over the built-ins; `Perm`-as-count-equality **adds no
  capability** (`count` is ordinary recursion, the law is `Π`-into-`Ω`). The two
  surfaced elaborator walls (surface **quotient-intro**; **multi-param `class`**
  for polymorphic optics) **re-fork to Steward WHEN their general forms are
  built** — they are *not* in CAT-3's concrete scope. **Not** a verdict-flip
- why: (property) AC1 — the outer-ring guarantee. Fork A deliberately chose
  count-equality partly *because* it adds no capability (vs `‖Perm_rel‖`'s
  truncation intro/elim); Fork B ships every concrete flavor as plain records.
  Any elaborator need re-forks to Steward, per the frame's do-not-reopen §6

---

## Coverage map

- **AC1** (kernel-untouched) — `cat3-kernel-untouched-outer-ring`
- **AC2** (proved, zero Axiom, append reuse) —
  `append-monoid-reuses-cat1-proofs`
- **AC3** (`Perm` Ω-sound) — `perm-is-count-equality-not-raw-omega-inductive`,
  `perm-eq-derives-from-ord-no-separate-deceq`
- **AC4** (sort flips) — `verified-sort-emits-both-conjuncts`,
  `nonpermuting-sort-const-nil-fails-perm`,
  `nonordering-sort-identity-fails-issorted`,
  `verified-sort-proved-carrier-is-lawful-bool`
- **AC5** (laws Ω pointwise) — `take-drop-decomposition-holds`,
  `take-length-law-red-until-built`, `map-length-preservation-red-until-built`,
  `filter-membership-red-until-built`
- **AC6** (projection mechanism + name) — `lens-coherence-laws-hold`,
  `broken-lens-setget-false-witness-rejected`,
  `optic-flavors-mechanism-per-flavor-grounded`,
  `optic-name-not-retired-view-keyword`
- **AC7** (green) — the red-until-built posture (cases flip green as the CAT-3
  build lands each op + law; CI gate)

## Cross-case consistency sweep

- **`Perm` = count-equality everywhere.** Every case that names `Perm` uses the
  Fork-A count-equality form (Scope) — the sort flips, the AC3 pin, the
  dict-economy case agree on the representation (mechanism-consistency, not just
  I/O-consistency).
- **Two dual sort flips, one shared obligation mechanism.** `sortBad` fails
  **`Perm`** (isSorted vacuous); `sortId` fails **`isSorted`** (Perm holds via
  `Refl`). They agree on the emitted conjoined `And` obligation and each breaks
  exactly one conjunct — the two dimensions are independently guarded, no case
  is green-vs-green.
- **Endpoint discipline per §3.2, per-branch.** Honest `Perm` self-equality is a
  **neutral** endpoint → `Refl`; the `take`/`drop` base + honest `isSorted` leaf
  are **constructor-headed** → `Top` → `tt`. Not uniform; re-derived, not
  transcribed.
- **Carrier `List Bool` for proved arms.** No proved-arm case carries on
  `List Int` (Axiom-holed) — the carrier case pins this so no flip degenerates
  to reject-vs-reject.
- **Red-until-built vs provable-now is tagged, not blurred.**
  `take-drop-decomposition-holds` is provable-now (landed ops);
  `take-length`/`map`/`filter` are op-blocked reds; the verified-sort ACCEPT
  arm is discharge-gap red (C5 known-gap). Three distinct red reasons, each
  named.

## Subsumed / not-duplicated (one home per property)

- **The `Ord.total` `bool_or` precedent** is cited as *grounding* for the
  count-equality choice; it is **not** re-tested here (its own conformance lives
  in `seed-lawful-classes.md`). CAT-3 tests only that `Perm` **follows** the
  same move.
- **Refinement-obligation emission/enforcement** is exercised by the verified
  `sort` cases (and canonically by C5); the general refinement machinery's
  conformance is not re-authored here.
- **`DecEq`/`Ord` law provability by carrier** (Bool real vs Int Axiom-holed)
  is owned by `seed-lawful-classes.md`; CAT-3's carrier case cites it only to
  justify `List Bool` for the proved arms.
