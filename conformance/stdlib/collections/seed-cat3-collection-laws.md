# CAT-3 collection-laws conformance — seed cases

Format: `../../README.md`. Third WP of the **catalog campaign**
(`docs/program/06-catalog-campaign.md`;
`docs/program/wp/CAT-3-collection-laws.md`). CAT-3 = **Layer 1**: the landed
`packages/collections` ops get their **laws as propositions, proved not
postulated** (the CAT-1 lawful-class discipline over a **value carrier**), the
capstone **verified `sort`** (`isSorted` and `Perm` as two laws), and the
agent-facing **`view` abstraction** (the catalog's projection unit — the six
optic flavors). This seed pins the discriminating conformance for all three
deliverables, **content-reconciled against the landed spec chapter**
`spec/50-stdlib/57-collections-and-views.md` (spec-author chapter tip
`829c999`, the `optic → view` rename + Fork-C/FOLD-IN-1 resolution).

The **Architect's three fork rulings** (`evt_4c3q1e611va69`), as transcribed in
`57`:

- **Fork A — `Perm` = count/multiset-equality, Ω-native, NOT `‖Perm_rel‖`**
  (`57 §3.1`). A Π into `Ω` of `Nat`-value equations is a proposition by the
  predicative max (`16 §1.1`), needing **zero truncation machinery** — the
  **same move as the landed `Ord.total` law** (`57 §3.2`; push proof-relevant
  content into a decidable `Bool`/`Nat`, keep the law a value-equation). A raw
  multi-ctor `data Perm … : Ω` is **inadmissible** (proof-relevant, `Ω`-PI
  collapse, `Type → Ω` admits `true ≡ false`, `16 §1.4`).
- **Fork B — plain `Σ`-record (lawful-class *shape*, first-class *value*) per
  concrete flavor** (`57 §4.2`). Five flavors ship concrete now; the
  polymorphic family + the quotient-carrier form are the two design-now /
  build-later walls (`57 §4.3`, `90`).
- **Fork C — family `view`, flagship flavor `lens`** (operator override, Pat
  2026-07-04, Steward `evt_4wsa1txzrx9nc`). Architect recommended `optic`; the
  operator ruled **`view`** — SURF-1's retirement of the `view` *keyword* frees
  the word, and `view` is the standard term for a read projection, so the
  keyword-collision concern does not bind. **RESOLVED at chapter tip `829c999`**
  (Architect fidelity gate green there): renamed `57-collections-and-views.md`,
  `§4.1`/`§90` rewritten open→resolved to `view`, Architect FOLD-IN 1 landed
  (§4.4 lens set-set `tt`→`Refl` — this seed already carried the fix), and the
  setoid-record field renamed lowercase `view` → `project` (KwView collision).
  This seed is byte-aligned to `829c999`.

**Red-until-built posture (CAT-1-Functor precedent).** `map`, `filter`, `sort`,
`isSorted`, `count`, `mem`, `length`, `min`, and the `view`/`lens` records are
**absent from `packages/` on `origin/main`** (self-grepped; `57 §2.1`/`§2.3`).
Every case whose subject is one of these is **red-until-built** against the
CAT-3 Language build (held for the GPT window); it flips green when the build
lands the op + its law. Cases over **only** landed ops (`list_append`, `take`,
`drop`, `nth`) are **provable-now** (need only the proof authored + built), and
are tagged distinctly from the deeper **blocked-on-missing-op** reds.

## Grounding (content-verified against the landed targets)

- `spec/50-stdlib/57-collections-and-views.md` (`829c999`) — the CAT-3
  contract: §2 D1 structural laws, §3 verified `sort`, §4 the `view`
  abstraction, §5 derivation paths / `trusted_base()` delta, §6 AC mapping. Law
  anchors: `57 §2.2` (D1), `57 §3.4`/`§3.7` (sort), `57 §4.4` (lens).
- `packages/collections/collections.ken` — **landed**: `list_append`
  (recursing on its **first** argument, so **left unit is definitional**, right
  unit + assoc inductive), `nth`, `take`, `drop`, `natSub`,
  `list_eq (eqf : a -> a -> Bool)`, `list_compare`,
  `data OrdResult = Lt | Eq | Gt`. **NOT landed** (`57 §2.1`): `count`, `map`,
  `filter`, `sort`, `isSorted`, `mem`, `length`, `min` — hence the
  red-until-built split.
- `packages/lawful-classes/lawful_classes.ken` — the lawful-class pattern and
  **the Fork-A precedent**: `Ord.total` is stated
  `total : (x)(y) → IsTrue (bool_or (leq x y) (leq y x))` — a decidable Bool
  equation, chosen *because* a bare propositional disjunction would be
  proof-relevant (`57 §3.2` cites this verbatim). `DecEq a`'s `sound`/`complete`
  fields (`:25`) tie its `eq` to kernel `Equal`.
- **Carrier soundness (`57 §3.4`, CV caveat folded in):** `DecEq Int`/`Ord Int`
  are **all-`Axiom`** (`Int` primitive), `Ord Char` **transports** those same
  `Axiom`s; **only `Bool` has a real, Axiom-free `DecEq`+`Ord`**.
  So the proved arms carry on `List Bool`; on `List Int` the honest-sort
  discharge cannot be Axiom-free and the flip degenerates to reject-vs-reject
  (green-vs-green vacuity).
- `16 §1.1` (Π into `Ω`), `16 §1.2` (`Ω`-PI), `16 §1.4` (the `Type → Ω`
  unsoundness pin — NOT `§1.3`, the derived-connectives/truncation home),
  `16 §5` (quotients: kernel `Term::Quot`/`QuotClass`/`QuotElim`, **no surface
  intro path**), `16 §6` (`‖·‖` truncation — the route Fork A did **not** take).
- `55 §3.1`/`§3.2` — the two-line induction+`cong` grammar and the
  `tt`-vs-`Refl` endpoint discrimination, **per-branch not uniform**
  (constructor-headed gives `Top` gives `tt`; neutral gives stuck `Eq` gives
  `Refl`) — inherited by `57 §1` and used at `57 §2.2`/`§3.7`/`§4.4`.
- `21 §6.1`/`§6.3` + `22` — refinement types `{ x : A | φ }` lower to the
  carrier + a kernel-re-checked obligation (`57 §4.2` refinement flavor).
  `13 §6` — the negative `Σ` and **definitional Σ-η** (`57 §4.4` lens set-get).
- `conformance/challenge/C5-verified-sort/` — corroborates the sort
  discriminator (the `const Nil` unsound arm is a *stronger* non-permuting sort
  than `57 §3.7`'s dedup — both fail `Perm`; §3.7's dedup is the canonical one).

## Scope — canonical shapes (from `57`), and the provable-now / red split

The verified-`sort` cases reference these landed shapes (`57 §3.1`–`§3.5`):

```
sort      (a) (le : a -> a -> Bool) (xs : List a) : List a   -- plain, SCT-total
Perm      (a) (eqf : a -> a -> Bool) (xs ys : List a) : Prop
            = (x : a) -> Equal Nat (count eqf x xs) (count eqf x ys)   -- Fork A
eqFromOrd (a) (le)  (x y : a) : Bool = bool_and (le x y) (le y x)
-- the TWO correctness laws (separate fields, NOT a conjoined refinement):
isSorted a le (sort a le xs)                        -- ordering law
Perm a (eqFromOrd a le) xs (sort a le xs)           -- permutation law
sortBad (a) (le) (xs) : List a = Nil a   -- dedup/drop stub (fails Perm)
sortId  (a) (le) (xs) : List a = xs      -- identity (fails isSorted)
```

- **D1 structural laws** (AC5, `57 §2.2`/`§2.3`): `take`/`drop` decomposition #1
  **provable-now** (landed ops); `length`/`min` decomposition #2, `map`
  length-preservation, `filter` membership are **red-until-built**.
- **D2 verified `sort`** (AC3/AC4, `57 §3`): the count-equality `Perm` and
  `isSorted` as **two separate law fields**, and the two dual flips (`sortBad`
  fails `Perm`; `sortId` fails `isSorted`), on `List Bool`.
- **D3 `view`** (AC6, `57 §4`): the concrete `lens` coherence flip over
  `Pair Bool Bool`, the per-flavor mechanism enumeration, the name reconcile.
- **Carrier:** `List Bool` for every proved obligation; `List a` /
  comparator-parametric only where no concrete lawful instance is needed.

---

## AC4 / AC3 — verified `sort`: count-equality `Perm` and the two dual flips

### stdlib/collections/verified-sort-both-laws-hold (soundness)
- spec: `57 §3.4` (the two laws; `sort : List a → List a` plain), `57 §3.6`
  (insertion sort, SCT-structural), `55 §3.1` (induction+`cong`), Fork-A ruling
- given: the landed explicit-comparator insertion `sort` over `List Bool`, with
  the two **separate** correctness laws (Scope) — the ordering law
  `isSorted a le (sort a le xs)` and the permutation law
  `Perm a (eqFromOrd a le) xs (sort a le xs)` — each an `Ω` field proved by
  induction; input a concrete `List Bool`
- expect: **both law fields hold for the honest sort.** Assert the observable:
  the ordering law and the permutation law are **two independently-stated `Ω`
  fields** (not conjuncts of one refinement obligation), both discharged
  Axiom-free on `List Bool`. **RED-UNTIL-BUILT** on two axes: `sort`/`isSorted`
  unlanded (op), and full **Perm discharge** is the C5 known-gap
- why: (soundness) the ACCEPT arm of AC4. It anchors the two REJECT flips below
  — each holds `sort`'s type fixed and breaks exactly one law. The **permutation
  law is independently load-bearing**: an `isSorted`-only contract is
  green-vs-green (`sortBad` passes it — see the next case). The carrier is
  `List Bool` per `57 §3.4`: on `List Int` the honest discharge cannot be
  Axiom-free and the flip degenerates
  ([[green-vs-green-does-not-confirm-a-fix]]: the ACCEPT arm must be genuinely
  dischargeable, not vacuously red)

### stdlib/collections/nonpermuting-sort-dedup-fails-perm (soundness)
- spec: `57 §3.7` (the `Perm` flip, dedup example), Fork-A ruling
  (count-equality), `16 §1.1`; C5 `unsound-const-nil.ken` (a stronger variant)
- given: a **dedup** "sort" `sortBad` (Scope) that drops duplicate elements
  (`57 §3.7`'s example), claiming the same `sort : List a → List a` type but not
  permuting; input `xs = [True, True, False]` on `List Bool`
- expect: **verdict flips vs the honest `sort`, keyed on the permutation law.**
  **Rejected — at the `perm` law**: `Perm a (eqFromOrd a le) xs (sort a le xs)`
  unfolds to `(x) → Equal Nat (count eqf x xs) (count eqf x (sort xs))`; the
  honest sort preserves the count (the abstract permutation law closes
  per-branch — base `Nil` gives `Eq Nat Zero Zero` → `Top` → **`tt`** (nullary),
  inductive steps → `Refl`/`cong`; `55 §3.2` + Architect §3.7 note, exact tokens
  build-pinned); a dedup drops `count True [T,T,F]` from `2` to `1`, so the
  goal is `Equal Nat 2 1`, uninhabited. Assert the **specific observable**:
  elaboration fails at the
  **`perm` law field** with a conversion / unprovable obligation on
  `Equal Nat 2 1` — **not** `is_err()`, not at `isSorted`, not a missing-field
  error ([[assert-specific-error-variant-not-is-err]])
- why: (soundness) AC4 — the discriminator that forces `sort` to *be* a sort;
  the `perm` law (count-equality) is what a dedup cannot satisfy. **Two-arm
  net**: a masked `perm = Axiom` postulates `Equal Nat 2 1` — a false equation
  on a concrete `Nat` — failing the delta gate into `Bottom` (`57 §3.7`).
  Verdict-flip vs `verified-sort-both-laws-hold`, keyed on the `perm` law. (C5's
  `const Nil` is an even stronger non-permuting sort — `count True Nil = 0` — a
  corroborating variant, not the primary)

### stdlib/collections/nonordering-sort-identity-fails-issorted (soundness)
- spec: `57 §3.7` (the `isSorted` flip, identity-on-descending), `55 §3.2`
  (`tt`/`Refl` endpoints), Fork-A ruling
- given: `sortId` (Scope) — the identity "sort" (a genuine permutation, but does
  not order); input a **descending** `List Bool` pair `[hi, lo]` under the
  `Ord Bool` order (direction reconciled against the landed instance)
- expect: **verdict flips, keyed on the ordering law — the dual of the previous
  case.** The permutation law **holds** — `Perm a eqf xs xs` is
  `(x) → Equal Nat (count eqf x xs) (count eqf x xs)`, a **neutral** endpoint
  closing **`Refl`** (`55 §3.2`, not `tt`). The ordering law **fails**:
  `isSorted le [hi, lo]` requires `IsTrue (le hi lo)` = `IsTrue False`,
  uninhabited. Assert the **specific observable**: elaboration fails at the
  **`isSorted` law** on `IsTrue False` — **not** at `perm`, **not** `is_err()`
- why: (soundness) AC4 — the **dual** flip: it isolates the *other* law, so the
  pair (this + `sortBad`) proves **each** law is independently load-bearing (the
  multi-dimensional-guard rule: `perm` and `isSorted` are two dimensions; a sort
  broken on only one must fail at that one). `perm`-holds-here vs
  `perm`-fails-there, and `isSorted`-fails-here vs `isSorted`-holds-there, is
  the crossed pair. Endpoint `Refl`-vs-`tt` re-derived per `55 §3.2` (honest
  reorder
  → `IsTrue True` = `Top` → `tt`), not transcribed

### stdlib/collections/perm-is-count-equality-not-raw-omega-inductive (property)
- spec: `57 §3.1` (Fork A), `16 §1.1` (Π into `Ω`), `16 §1.2` (`Ω`-PI),
  `16 §1.4` (`Type → Ω` unsoundness pin — NOT `§1.3`), `57 §3.2`
  (`Ord.total` precedent)
- given: two candidate `Perm` representations: (a) the ruled count/multiset
  `Perm a eqf xs ys` (Scope); (b) a raw multi-constructor `data Perm : … : Ω` (a
  `perm_nil`/`perm_skip`/`perm_swap`/`perm_trans`-style inductive relation at
  `Ω`)
- expect: **(a) is admissible and Ω-native; (b) is inadmissible.** Assert the
  **structural** observable: (a) `Perm` is a `Π`-type into `Ω` (`Equal Nat _ _`
  is `Ω`; the `Π` stays `Ω` by the predicative max `16 §1.1`), needs **no**
  `‖·‖` intro/elim, provable by the landed two-line grammar; (b) a
  proof-relevant multi-ctor inductive **cannot** be declared at `Ω` — distinct
  re-orderings are distinct derivations, `Ω`-PI (`16 §1.2`) collapses them, and
  unrestricted `Type → Ω` admits `Bool ⇒ true ≡ false` (`16 §1.4`). **Not** a
  verdict-flip: a `(property)` case pinning the ruling's representation —
  the AC3 soundness pin
- why: (property) AC3 — the load-bearing soundness call. Count-equality is the
  **same move as the landed `Ord.total`** (`57 §3.2`): push proof-relevant
  content into a decidable `Nat` count, keep the law a value-equation
  (subsume-don't-proliferate on a soundness pattern the stdlib already chose,
  [[proof-relevant-inductive-cannot-be-declared-at-omega]]). Stated as a
  **property** because the discriminator is the *representation choice*, not a
  witness — a build that declared `data Perm : Ω` would be blocked here

### stdlib/collections/perm-eq-derives-from-ord-no-separate-deceq (property)
- spec: `57 §3.5` (`eqFromOrd`), Fork-A ruling pt 3, `lawful_classes.ken`
  (`Ord`/`DecEq`), `51`
- given: two ways `count` obtains its element-equality for the **sort capstone**
  (which already carries `le`): (a) `eqFromOrd` (Scope, `57 §3.5`) — derived
  from the sort's own `le` (antisym gives `Equal` from `IsTrue`; refl gives the
  converse); (b) a **separate** `DecEq a` dict threaded alongside
- expect: **(a) suffices for `sort`'s `Perm`; (b) is redundant for the
  capstone.** Assert the structural observable: `sort`'s `Perm` law needs
  **no** extra `DecEq` dict — `eqFromOrd` decides the same `Equal` the law is
  about (`DecEq`'s `sound`/`complete` tie, `57 §3.5`). A **standalone** generic
  `Perm` (no ambient order) does take a `DecEq a` param. **Not** a verdict-flip:
  a `(property)` case grounding the dict economy
- why: (property) AC3/AC4 — pins the dict economy so the build does **not**
  proliferate a `DecEq` requirement onto `sort` that the order already
  discharges. The `count`-eq threading via `eqFromOrd` is the landed `57 §3.5`
  form ([[class-dict-explicit-vs-implicit-abstract-tyvar]]: the ambient-order
  path avoids an extra dict)

### stdlib/collections/verified-sort-proved-carrier-is-lawful-bool (property)
- spec: `57 §3.4` (the carrier caveat, CV-sourced, Architect-bound),
  `lawful_classes.ken` (`DecEq`/`Ord` Bool vs Int)
- given: the proved ACCEPT arm of the verified `sort` instantiated at (a)
  `List Bool` vs (b) `List Int`
- expect: **the proved arm must carry on `List Bool`.** Assert the structural
  observable: on (a) `List Bool` the honest `Perm`/`isSorted` discharge is
  **Axiom-free** (`DecEq Bool`/`Ord Bool` real); on (b) `List Int` it **cannot**
  be (`Ord Int`/`DecEq Int` all-`Axiom`) so the `sortBad`/`sortId` flips
  degenerate to **reject-vs-reject** (green-vs-green vacuity — both arms fail,
  the flip guards nothing). **Not** a verdict-flip: a `(property)` case that
  **guards the ACCEPT arm from vacuity**
- why: (property) AC4 — a discriminating flip is only real if the ACCEPT arm is
  genuinely dischargeable ([[green-vs-green-does-not-confirm-a-fix]]). `57 §3.4`
  folds this in verbatim: `List Bool` for D2's proof obligations, `List Int`/
  generic only where a law is comparator-parametric and needs no concrete lawful
  instance. Promoted to a standing case so the build cannot silently pick
  `List Int` and ship a vacuous green

---

## AC5 — D1 structural laws: `Ω`, pointwise, one canonical field

### stdlib/collections/take-drop-decomposition-holds (soundness)
- spec: `57 §2.2` (decomposition #1, GREEN), `57 §2.3` (provable-now),
  `55 §3.1`/`§3.2`, `collections.ken` (`take`/`drop`/`list_append` landed)
- given: the law
  `Equal (List a) (list_append a (take a n xs) (drop a n xs)) xs` as a single
  `Ω` field, stated pointwise; proved by induction on `n`/`xs`; carrier
  `List Bool` (or generic `List a` — element-agnostic)
- expect: **accepts — provable-NOW** (all three ops landed; needs only the proof
  authored + the CAT-3 package built). Endpoints per `55 §3.2`: the `Zero`/`Nil`
  base reduces both sides to `xs`/`Nil` — **constructor-headed gives `Top` gives
  `tt`**; the `Suc`/`Cons` step is inductive. Assert: one canonical `Ω` field,
  the pointwise equation closes. **Tagged provable-now**, distinct from the reds
  below
- why: (soundness) AC5 — the one D1 law statable+provable on landed ops today
  (`57 §2.3`). Serves as the "provable-now" anchor against which the missing-op
  reds are contrasted

### stdlib/collections/take-length-law-red-until-built (property)
- spec: `57 §2.2` (decomposition #2, RED), `57 §2.3` (`length` AND `min` both
  unlanded)
- given: the length-of-take law
  `Equal Nat (length a (take a n xs)) (min n (length a xs))`
- expect: **RED-UNTIL-BUILT — blocked on missing ops** (`length` and `min` both
  absent; only `natSub` is landed, per `57 §2.3`). The law cannot be *stated*
  without them. Flips green when the build lands `length` + `min` + the proof.
  Assert: the case is red for a **deeper** reason than decomposition #1 — not
  "proof unwritten" but "operator unlanded"
- why: (property) AC5 — the provable-now / blocked-red distinction `57 §2.3`
  sharpens (decomposition #1 provable, #2 blocked). Tagging **which** reds are
  op-blocked vs proof-pending is the elaboration-time deferred-seam tag
  ([[layer-dependent-pin-at-unconditional-layer]])

### stdlib/collections/map-length-preservation-red-until-built (property)
- spec: `57 §2.2` (`Equal Nat (length b (map a b f xs)) (length a xs)`, RED),
  CAT-1-Functor red posture
- given: the `map` length-preservation law (if `map` arrives via `Functor List`,
  a law *about* the instance)
- expect: **RED-UNTIL-BUILT — blocked** (`map` and `length` both unlanded; no
  `Functor List` / standalone `map` on `List` in `packages/`). Flips green when
  the build lands `map` + `length` + the proof
- why: (property) AC5 — CAT-1-Functor posture verbatim; the `map` provenance
  (`Functor List` vs standalone) is an open build sub-decision

### stdlib/collections/filter-membership-red-until-built (property)
- spec: `57 §2.2` (the `filter` membership `Iff`, RED)
- given: the `filter` membership law with `mem`/`Iff` as `Ω`-predicates
- expect: **RED-UNTIL-BUILT — blocked** (`filter` and `mem` both unlanded). The
  `Iff` is a two-sided `Ω`-implication (`16 §1.3` mutual `→`, no truncation).
  Flips green when the build lands `filter` + `mem` + the proof
- why: (property) AC5 — the one D1 law whose statement is a bi-implication
  rather than an equation; pinning it as `Iff`-of-`Ω`-predicates (`16 §1.3`, not
  a `Bool`-equation) keeps it in the value/`Ω` fragment (`57 §2.2`)

---

## AC2 — proved, zero `Axiom`; the append monoid REUSES CAT-1

### stdlib/collections/append-monoid-reuses-cat1-proofs (property)
- spec: `57 §2.4` (reuse, not re-proof), `lawful_functors.ken`
  (`list_assoc`/`list_left_unit`/`list_right_unit`, landed CAT-1), CAT-1-build
  §6.1 (parametric-instance-head gate)
- given: the `Monoid (List a)` instance in two forms: (a) its law fields
  **cite** the landed generic CAT-1 proofs; (b) a variant that **re-proves**
  associativity/unit inline in the `collections` package
- expect: **(a) is the mandated form; (b) a proliferation defect.** Assert the
  structural observable: the `Monoid (List a)` law fields are **citations** to
  the CAT-1 proofs (grep: no duplicated `list_assoc`/`list_*_unit` proof terms),
  zero `Axiom`. **The parametric instance *form*** `instance Monoid (List a)` is
  gated on CAT-1-build's parametric-instance-head piece (`55 §6.1`, free `a` →
  `UnresolvedCon`; the landed instance bundles monomorphically) — so the
  instance is red-until-that-lands while the **generic append proofs** are
  reusable today (`57 §2.4`). **Not** a verdict-flip: a `(property)` case
  pinning reuse-don't-proliferate
- why: (property) AC2 — subsume-don't-proliferate: re-deriving the monoid laws
  would duplicate proof terms (a divergence surface) that CAT-1 already
  discharges once, generically. The gate on the parametric-instance-head keeps
  the honest split ([[class-dict-explicit-vs-implicit-abstract-tyvar]]: the
  monomorphic/landed path is available; the parametric-head path is the gated
  one)

---

## AC6 — D3: the `view` abstraction

### stdlib/collections/lens-coherence-laws-hold (soundness)
- spec: `57 §4.4` (the concrete lens over `Pair Bool Bool`), `57 §4.2` (Fork B:
  plain `Σ`-record), `13 §6` (negative `Σ` + definitional Σ-η), `55 §3.2`
  (endpoints)
- given: the shipped concrete `lens` (`57 §4.4`) — a `Σ`-record of `get`/`set`
  + the three coherence proofs — onto the first component of `Pair Bool Bool`,
  with `get := pairFst` and `set s b := mkPair b (pairSnd s)` over the landed
  prelude Σ-pair, and the three coherence laws proved
- expect: **accepts** — the three coherence laws close **definitionally**, with
  the per-branch endpoint discipline (`55 §3.2`): **get-set**
  `Equal Bool (get (set s b)) b` computes by Σ-β to neutral `b` both sides →
  **`Refl`**; **set-set**
  `Equal (Pair Bool Bool) (set (set s b) c) (set s c)` computes by Σ-β so both
  sides are the identical `mkPair c (pairSnd s)` → **`Refl`** (a **non-nullary**
  head with a **neutral** component `pairSnd s`, so the `Eq Bool` on the second
  components stays stuck and does NOT collapse to `Top` — `tt : Top` is
  ill-typed;
  Architect FOLD-IN 1); **set-get**
  `Equal (Pair Bool Bool) (set s (get s)) s` holds by **definitional Σ-η**
  (`mkPair (pairFst s) (pairSnd s) ≡ s`, `13 §6`) → `Refl`. Assert: the record
  elaborates, all three proofs close by `Refl`, zero `Axiom`, **no `match` on
  the Σ-pair**
  (Σ-η, not a case-split). **RED-UNTIL-BUILT** (the `view`/`lens` record is not
  in `packages/`)
- why: (soundness) AC6 — the ACCEPT arm for the shipped concrete flavor. It is
  ordinary Ken (Fork B: plain `Σ`-record, kernel-untouched). **All three laws
  close by `Refl`** — none by `tt`: the `55 §3.2` "same head → `Top` → `tt`"
  rule fires only for a **nullary** head that collapses fully; `Pair`'s `mkPair`
  is non-nullary with a neutral component, so every endpoint is a stuck `Eq` on
  equal terms ([[tt-vs-refl-endpoint-rule-for-inductive-equal-law-bases]],
  Architect FOLD-IN 1 — the same non-nullary-head subtlety §3.2 pins). No
  `DecEq`/`Ord` needed (structural over `Pair`), so the `57 §3.4` carrier caveat
  does not bind here. Anchors the broken-lens flip below

### stdlib/collections/broken-lens-getset-false-witness-rejected (soundness)
- spec: `57 §4.4` (the get-set law), Fork-B ruling, `55 §3.2`; sibling of
  CAT-2's `applicative-map-coheres-with-wired-functor` (coherence-flip shape)
- given: two `lens`-shaped records **identical** in type over `Pair Bool Bool`,
  differing only in `set` and the coherence proof: (a) the canonical lens (real
  proofs); (b) a **broken** lens whose `set s _ = s` (drops its value arg —
  well-typed at `Pair Bool Bool → Bool → Pair Bool Bool`), whose `get_set` proof
  is then false
- expect: **verdict flips on the coherence of `set`, at the get-set law.** (a)
  **accepts**; (b) **rejected — conversion failure at the `get_set` field**:
  `get (set s b) = pairFst s` (the dropped `b`) while the law demands `≡ b`, a
  false `Equal Bool (pairFst s) b` (two distinct neutrals — not `Refl`). Assert
  the **specific observable**: (b) fails **at the `get_set` field** with a
  neutral/value clash — **not** `is_err()`, not a missing-field error, and
  **not** at `set_get` (which the broken `set s _ = s` still satisfies:
  `set s (get s) = s`) ([[assert-specific-error-variant-not-is-err]])
- why: (soundness) AC6 — proves the coherence laws are **load-bearing** (a lens
  is `(get, set)` **plus proofs**). The get-set law is precisely what forces
  `set` to *store* its argument; a value-dropping `set` fails it while still
  passing set-get/set-set — so the discriminator must target **get-set**
  specifically (multi-dimensional-guard: naming the wrong law would be
  green-vs-green). **Two-arm net**: `get_set = Axiom` postulates a false
  `Equal` into `Bottom`. Verdict-flip, keyed solely on `set`'s get-set coherence

### stdlib/collections/view-flavors-mechanism-per-flavor-grounded (property)
- spec: `57 §4.2` (the per-flavor table), `57 §4.3` (the two walls), `21 §6.1`
  (refinement), `16 §5` (quotients, no surface intro), CAT-1 §6 (multi-param
  `class`), Lane B / L12 / L14 (obligation seam)
- given: the six `view` flavors — `projection (lens)`, `representation (iso)`,
  `refinement`, `indexed`, `quotient-respecting`, `obligation-producing` — each
  claimed against **landed** machinery
- expect: **each flavor's mechanism is grounded, not hand-waved "views are
  records".** Assert the structural observable, per `57 §4.2`: `lens`/`iso`/
  `refinement`/`indexed` = plain `Σ`-record (concrete) → **ship now**;
  `quotient-respecting` **setoid-morphism** form
  `{ project ; respects }` is a plain `Σ`-record → **now** (the field is
  `project`, not a lowercase `view`, so it never collides with the live
  `KwView` keyword — `57 §4.2`), but the
  **quotient-carrier** form (a view *out of* `A/R`) needs a surface
  **quotient-intro** path the parser lacks (`16 §5` kernel formers exist, no
  surface) → **deferred**; `obligation-producing` = the **Lane B / L12 / L14
  seam** → **boundary stated, deferred**; and the **polymorphic** forms
  (`Lens s a`, `Iso a b`) need a **multi-param `class`** (`57 §4.3`) → **design
  now, build fast-follow**. **Not** a verdict-flip: a `(property)` case pinning
  the per-flavor mechanism + what-ships-now
- why: (property) AC6 — Fork B mandates enumerating the mechanism per flavor
  against landed code ([[sizing-a-subsume-fix-enumerate-every-piece]]: list each
  flavor's distinct vehicle and grep it). Surfacing the two design-now/
  build-later walls (surface-quotient-intro, multi-param-`class`) as a standing
  case keeps AC1 honest — they re-fork to Steward (`57 §4.3`/`90`), not now

### stdlib/collections/view-family-name-reuses-freed-keyword (property)
- spec: `57 §4.1` (Fork C, resolved to family `view` at `829c999`), Steward
  `evt_4wsa1txzrx9nc` (operator override), SURF-1 (`view` keyword retired), `90`
  (naming token routed)
- given: the Layer-1 abstraction's family name, after the operator ruled it
  **`view`** (over Architect's recommended `optic`)
- expect: **the family name is `view` (the freed noun), flagship flavor
  `lens`.** Assert the observable: the shipped name is `view` — the operator
  ruled the `view`-*keyword* collision concern **does not bind** (no extant
  users of the keyword outside this repo; `view` is the standard term for a read
  projection). **Build-order nuance** (Steward, not a blocker): a
  **capitalized** `View` type/class is collision-free on `main` today; a
  **lowercase** `view` identifier collides with the still-live `KwView` lexer
  token until SURF-1's keyword-retirement build lands, so CAT-3-build then
  sequences after it. **Not** a verdict-flip
- why: (property) AC6 / Fork C — **this case's premise inverted at the operator
  gate**: I originally pinned "name must NOT be `view`" and `(oracle)`-tagged
  the token; the operator resolved it to **`view`**, so the `(oracle)`-tag
  is discharged to the ruled value
  ([[reconcile-binds-a-co-reviewers-plausible-reading-too]]: re-derive against
  the ruling, do not carry the stale hypothesis). Not
  hard-freezing `optic` is exactly what let this reconcile cleanly instead of
  shipping a case contradicting the ruling. **`§4.1`/`§90` are RESOLVED to
  `view` at `ec94c62`, chapter tip `829c999`** (Architect-verified faithful);
  this seed is byte-aligned to `829c999`

---

## AC1 — kernel-untouched, outer-ring

### stdlib/collections/cat3-kernel-untouched-outer-ring (property)
- spec: `57 §5`/`§6` (AC1), `57 §4.3` (the two build-later walls), Fork-A/B
  rulings (no new capability)
- given: the full CAT-3 deliverable set — `count`/`map`/`filter`/`sort`/
  `isSorted`/`mem`/`length`/`min`, the count-equality `Perm`, the `view`/`lens`
  records
- expect: **kernel-untouched.** Assert the structural observable:
  `git diff origin/main -- crates/ken-kernel/` empty on the CAT-3 build; zero
  `trusted_base()` delta; no new `Term`/`Decl`. `count`/`sort`/`lens` are
  ordinary Ken over the built-ins; `Perm`-as-count-equality **adds no
  capability** (`count` is ordinary recursion, the law is `Π`-into-`Ω`). The two
  surfaced elaborator walls (surface **quotient-intro**; **multi-param `class`**
  for polymorphic views) **re-fork to Steward WHEN their general forms are
  built** (`57 §4.3`) — not in CAT-3's concrete scope. **Not** a verdict-flip
- why: (property) AC1 — the outer-ring guarantee (`57 §5`). Fork A chose
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
- **AC4** (sort flips) — `verified-sort-both-laws-hold`,
  `nonpermuting-sort-dedup-fails-perm`,
  `nonordering-sort-identity-fails-issorted`,
  `verified-sort-proved-carrier-is-lawful-bool`
- **AC5** (laws Ω pointwise) — `take-drop-decomposition-holds`,
  `take-length-law-red-until-built`, `map-length-preservation-red-until-built`,
  `filter-membership-red-until-built`
- **AC6** (view mechanism + name) — `lens-coherence-laws-hold`,
  `broken-lens-getset-false-witness-rejected`,
  `view-flavors-mechanism-per-flavor-grounded`,
  `view-family-name-reuses-freed-keyword`
- **AC7** (green) — the red-until-built posture (cases flip green as the CAT-3
  build lands each op + law; CI gate)

## Cross-case consistency sweep

- **`Perm` = count-equality everywhere.** Every case that names `Perm` uses the
  Fork-A `Perm a eqf xs ys` count-equality form (Scope, `57 §3.1`) — the sort
  flips, the AC3 pin, the dict-economy case agree on the representation
  (mechanism-consistency, not just I/O-consistency).
- **Two dual sort flips, two separate law fields.** `sortBad` (dedup) fails
  **`perm`** (`isSorted` vacuous); `sortId` (identity) fails **`isSorted`**
  (`perm` holds via `Refl`). The laws are **independent `Ω` fields** (`57 §3.4`,
  not a conjoined refinement) and each breaks exactly one — no case is
  green-vs-green.
- **Endpoint discipline per `55 §3.2`, per-branch (NOT uniform).** Identity-sort
  `Perm a xs xs` self-equality (abstract `xs`) → **neutral** → `Refl`; the
  abstract sort-`Perm` law is a **mix** — base `count x Nil = Zero` gives
  `Eq Nat Zero Zero` → `Top` → **`tt`** (nullary), inductive steps → `Refl`/
  `cong` (Architect §3.7 note). `take`/`drop` base + honest `isSorted` leaf are
  **nullary constructor-headed** → `Top` → `tt`. **All three lens laws →
  `Refl`** — set-set's shared `mkPair c (pairSnd s)` is non-nullary with a
  neutral component, so `tt : Top` is ill-typed (Architect FOLD-IN 1). Exact
  sort/lens tokens reconciled at build.
- **Carrier `List Bool` for proved *sort* arms; `Pair Bool Bool` for the lens.**
  No proved sort-arm carries on `List Int` (Axiom-holed); the lens is structural
  over `Pair` (needs no `DecEq`/`Ord`, `57 §4.4`).
- **Red-until-built vs provable-now is tagged, not blurred.**
  `take-drop-decomposition-holds` is provable-now (`57 §2.3`);
  `take-length`/`map`/`filter` are op-blocked reds; the verified-sort ACCEPT arm
  is discharge-gap red. Distinct red reasons, each named.
- **Fork-C name reconcile pending.** `view-family-name-reuses-freed-keyword` is
  authored on the ruled `view`, RESOLVED at chapter tip `829c999` (`§4.1`/`§90`,
  FOLD-IN 1 in `§4.4`, setoid field → `project`); this seed is byte-aligned.

## Subsumed / not-duplicated (one home per property)

- **The `Ord.total` `bool_or` precedent** is cited as *grounding* for the
  count-equality choice (`57 §3.2`); its own conformance lives in
  `seed-lawful-classes.md`. CAT-3 tests only that `Perm` **follows** the move.
- **Refinement-obligation emission/enforcement** is exercised by C5 and the
  `refinement` view flavor; the general refinement machinery's conformance is
  not re-authored here.
- **`DecEq`/`Ord` law provability by carrier** (Bool real vs Int Axiom-holed) is
  owned by `seed-lawful-classes.md`; CAT-3's carrier case cites it only to
  justify `List Bool` for the proved sort arms.
