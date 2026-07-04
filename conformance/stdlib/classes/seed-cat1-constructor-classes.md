# CAT-1 constructor-class conformance — seed cases (Semigroup/Monoid)

Format: `../../README.md`. First WP of the **catalog campaign**
(`docs/program/06-catalog-campaign.md`;
`docs/program/wp/CAT-1-constructor-classes.md`).
CAT-1 extends the landed `packages/lawful-classes` pattern (`Eq`/`DecEq`/`Ord`,
over a **value type** `a : Type`) to the workhorse **constructor classes** and
their value-level algebra companions. This seed pins the **discriminating
conformance for the value-level algebra classes — `Semigroup`/`Monoid`** (the
warm-up: same shape as `Eq`/`Ord`, over `a : Type`, no new kind machinery). The
higher-kinded classes **`Functor`/`Foldable`** (over `f : Type -> Type`) are
**held** (see the HELD section) pending the Architect's two design rulings
(higher-kinded admission + the Functor-law statement form) — their conformance
is not authored until the law form is pinned, so this seed does not over-freeze
it ([[layer-dependent-pin-at-unconditional-layer]]).

The load-bearing property is **AC4**: a **broken-unit-law "Monoid"** — an
instance whose `mempty` witness is **wrong**, so a unit law is a **false
equation** — must be **rejected for the right reason**, not merely "some error."
The genuinely-new-for-Monoid content vs the `Eq`/`Ord` seed is the **structural
induction** in the proofs: over `List` the unit/assoc laws are the catalog's
first laws proved by **induction on a carrier** (not the finite `Bool`
case-split of `Eq`/`Ord`), and the **left/right unit asymmetry** that
single-argument recursion forces (left unit definitional, right unit + assoc
genuinely inductive) is a Monoid-first phenomenon.

## Grounding (content-verified against the landed targets)

- `51-lawful-classes.md` — the class-as-record / law-as-Ω-prop pattern:
  op fields `Type`-valued, **law fields `Ω`-valued** (`§3`); an instance is a
  `declare_def` **record value carrying real law proofs** re-checked by the
  kernel (`§5`); **zero `trusted_base()` delta on an inductive carrier**, an
  **audited delta** on a primitive carrier (`§6`, the two-axes rule: law *sort*
  Ω-clean **and** carrier *provability* via an eliminator). CAT-1's
  `Semigroup`/`Monoid` are the **same shape** over `a : Type` — the spec section
  is authored by spec-author on this WP; this seed pins the discriminating
  behavior, grounded on the `51` pattern + first principles.
- `50-stdlib/README.md §2` + `packages/README.md` — the taxonomy already names
  it: a `Monoid` is `(<>, empty)` **plus real proofs** of associativity and the
  unit laws — **proved, not postulated**; an inductive-carrier instance whose
  law fields are postulated/holed has a **non-empty `trusted_base_delta`** and
  is **not lawful**. `21 §5` — the `law Monoid { assoc ; unit_l ; unit_r }`
  bundle shape.
- `33 §5.2`/`§5.3` — a class is a **record** (`13 §3` Σ+η); an instance is a
  record value of ops **+ law proofs**. **No new kernel former** (AC1).
- `packages/collections/collections.ken` — the landed `list_append`
  (`view list_append (a) (xs ys : List a) : List a`), recursing on its **first**
  argument: `list_append a Nil ys => ys`, `list_append a (Cons x xs2) ys =>
  Cons a x (list_append a xs2 ys)`. This recursion structure is the ground for
  the left/right-unit proof asymmetry below (re-derived from first principles,
  not transcribed, per
  [[kernel-rejects-is-completeness-fix-is-where-soundness-converts]]).
- `25 §3` — `trusted_base_delta`; `16 §1` — `Ω`/proof-irrelevance; `14 §3` — K4
  Ω-motive elimination (the capability an inductive carrier's law proofs use).

## Scope — the algebra-side of the ES1 zero-delta invariant

Like the `Eq`/`Ord` seed, AC3 here is **not** a new soundness mechanism: it is
the ES1 **zero-`trusted_base()` delta** invariant (`../surface/taxonomy/
minimality.md`, `../../security/trust-model/`) read from the **law side**
(`51 §5`) — a `Semigroup`/`Monoid` law field inhabited by a `postulate` is an
`Opaque` entry → non-empty delta; a holed field fails the kernel re-check. This
seed does **not** re-pin the `trusted_base()` mechanism; it pins that a
`Monoid` instance's **algebra law fields are subject to it**, and adds the
Monoid-specific content the `Eq`/`Ord` seed cannot cover: the **false-witness**
flip (AC4) and the **induction-vs-`Refl` proof asymmetry** (AC3).

**Tags.** `(soundness)` — the laws-PROVED / false-witness-rejected gates (a
broken or law-less instance admitted as lawful is a verification-soundness hole:
downstream `fold`/generic algorithms *assume the laws*, and a false unit law
**inhabits `Bottom`**, [[deceq-on-noncanonical-carrier-inhabits-bottom]]).
`(oracle)` — the literal field spellings (`<>`/`mappend`/`mempty`/`empty`/
`assoc`/`unit_l`/`left_unit`/`right_unit`), whether `Monoid` **extends**
`Semigroup` or restates `<>`+`assoc`, and the concrete second/broken carrier
(`List Bool` vs `List Unit` vs `Nat`) — spec-author finalizes these on the
package; this seed pins the **concept + verdict + reason** and tags the
spellings, per the granularity discipline (pin the locked granularity, not
tighter — the T1 dual). The **law-field `Ω`-sorts, the false-witness rejection,
the induction requirement, the zero-delta lawfulness, and every verdict** are
**normative**.

**Static vs runtime face.** These pin the **static face** — the spec discipline
(lawful ≡ zero-delta on an inductive carrier; a false witness fails the law
re-check) and the discriminating shape. The **runtime/build face** — the actual
canonical instances in `packages/` carrying real `elim_List` proof terms,
producer-grepped for `declare_postulate`/holes on the law fields — is the
**Team-Language build** (CAT-1 build gate), reconciled against the **landed**
package body there, not asserted from this pre-package draft
([[soundness-ac-static-vs-runtime-face]],
[[lock-structural-output-against-landed-body]]).

**Reconcile-at-build note.** This seed is authored **in parallel with**
spec-author's `packages/lawful-functors/` (name per spec-leader's coordination).
The exact field spellings, the `Semigroup`/`Monoid` layering choice, and the
broken-witness carrier are **reconciled against the landed body** at the build
gate — a heading/draft guess is not ground until the package lands
([[disclaimed-framing-still-binds-your-own-companion-artifact]]). The
**proof-structure claims** (left-unit definitional, right-unit/assoc inductive,
false-witness fails conversion) are re-derived from the landed `list_append`
recursion and hold **regardless** of package naming.

---

## AC4 — a broken-unit-law "Monoid" is rejected, right-reason (soundness gate)

The centerpiece. A `Monoid` instance over an inductive carrier whose `mempty`
witness is **not** the true identity makes a unit law a **false equation**. The
discriminator holds the **proof term fixed** and varies **only the witness**, so
the verdict flips **solely** on witness correctness — never green-vs-green.

### stdlib/classes/monoid-unit-law-false-witness-rejected (soundness)
- spec: `51 §5` (laws PROVED = real kernel proofs), `50-stdlib/README §2` +
  `packages/README` (a `Monoid` is `(<>, empty)` **plus** real unit proofs),
  `13 §2` (Σ-Intro re-check), `16 §1` (Ω-equality); grounds on
  `packages/collections/collections.ken` `list_append`
- given: two `Monoid (List Bool)`-shaped instances, **identical** in their
  operation field (`<> = list_append Bool`) and identical in the **proof term**
  offered for the left-unit law (`left_unit = Refl`), differing **only** in the
  `mempty` witness — (a) the **true identity** `mempty = Nil` (empty list);
  (b) a **wrong witness** `mempty = Cons True Nil` (a nonempty list — still
  **well-typed** at `mempty : List Bool`, so nothing rejects it *as a value*)
- expect: **the verdict flips on the witness, with the same proof term.**
  (a) **accepts** — `list_append Bool Nil x` δι-reduces to `x`, so
  `left_unit`'s goal is `Equal (List Bool) x x`, closed by `Refl`; the instance
  elaborates and its law fields kernel-re-check.
  (b) **rejected — conversion failure at the `left_unit` field**:
  `list_append Bool (Cons True Nil) x` reduces to `Cons True x`, so the goal is
  `Equal (List Bool) (Cons True x) x`, a **false** equation (`Cons`-headed vs
  the neutral tail `x`); `Refl` requires `Cons True x ≡ x`, which is **not
  convertible** → the kernel rejects the `left_unit` proof (a `TypeMismatch` /
  conversion failure), **not** a missing-field / kind / resolution error.
  Assert the **specific observable**: (a) elaborates + law fields re-check;
  (b) elaboration fails **at the `left_unit` law field** with a
  conversion/type-mismatch on `Cons True x` vs `x` — **not** `is_err()`
  ([[assert-specific-error-variant-not-is-err]]), and **not** a message string.
- why: (soundness) AC4 — the whole point of a *lawful* `Monoid`. The verdict is
  keyed **solely** on the witness: the two instances share `<>` **and** the
  `Refl` proof term, so the flip is not about provenance but about whether the
  unit **equation is true**. This is the
  [[verify-a-proposed-fix-excludes-the-counterexample]] discipline made a
  conformance case: the true identity makes the reduction
  close, the wrong one makes it a false `Equal` the kernel cannot check. **The
  second arm is the [[two-arm-producer-needs-a-case-per-arm]] guard the frame's
  AC4 names:** the *other* way a false-witness Monoid could try to pass is to
  **mask** the false law with a postulate — `left_unit = Axiom` type-checks the
  `Opaque` field — which is caught **not** by conversion but by the **AC3 delta
  gate** (`monoid-law-fields-real-proofs-not-postulates`): a postulated law is a
  non-empty `trusted_base_delta`, and here it is a postulate of a **false**
  statement, so it **inhabits `Bottom`** (`Axiom : Equal (List Bool)
  (Cons True x) x` is a false postulate — the
  [[deceq-on-noncanonical-carrier-inhabits-bottom]] shape: a wrong witness makes
  the "law" unprovable-because-false, and postulating it is unsound, not an
  honest audited delta). So the two arms — **honest proof fails conversion** and
  **masked proof fails the delta gate (and would inhabit Bottom)** — exhaust the
  ways a broken-witness Monoid could sneak through. **Verdict-flip, not
  green-vs-green:** true-witness accepts / false-witness rejects on the same
  proof term — opposite observables.

---

## AC3 — laws PROVED by induction (the induction-vs-`Refl` asymmetry)

Over `List` the Monoid laws are the catalog's first laws proved by **structural
induction**. Because `list_append` recurses on its **first** argument, the
left/right unit laws are **not symmetric** in provability — a Monoid-first
structural fact that a build can plausibly get wrong (assuming both units are
definitional, or postulating the inductive one when `Refl` "doesn't work").

### stdlib/classes/monoid-unit-asymmetry-left-defn-right-inductive (soundness)
- spec: `51 §5` (real proofs), `14 §3` (K4 Ω-motive `elim_List`), `13 §2`
  (Σ-Intro re-check); grounds on `list_append` (recurses on the **first** arg)
- given: the canonical `Monoid (List a)` (`<> = list_append a`, `mempty = Nil`),
  and the **same** proof term `Refl` offered for **each** unit law in turn:
  (a) `left_unit = Refl` for `left_unit : (x) -> Equal (List a)
  (list_append a Nil x) x`; (b) `right_unit = Refl` for `right_unit : (x) ->
  Equal (List a) (list_append a x Nil) x`
- expect: **the same `Refl` term flips verdict across the two fields.**
  (a) **accepts** — `list_append a Nil x` ι-reduces to `x` (the `Nil` branch is
  definitional), goal `Equal (List a) x x` closes by `Refl`.
  (b) **rejected — conversion failure**: `list_append a x Nil` with `x` a
  **variable** is **stuck** (`list_append` matches on its first arg; a neutral
  `x` blocks ι), so the goal `Equal (List a) (list_append a x Nil) x` does
  **not** reduce to reflexivity and `Refl` fails to check. The **correct**
  `right_unit` is a **real `elim_List` induction on `x`** (base `x = Nil`:
  `list_append a Nil Nil ≡ Nil`, `Refl`; step: IH + `Cons` congruence) — a
  proof term, **never** `Axiom` (it is *provable*, so a postulate here is an
  **avoidable delta = defect**, `51 §6`). Assert the **observable**: `Refl`
  accepts at `left_unit`, `Refl` rejects (conversion failure) at `right_unit`;
  and the accepted canonical `right_unit`/`assoc` fields grep as **real proof
  terms** (`elim_List`), **not** `declare_postulate`
  ([[kernel-backed-claim-grep-the-emission-not-the-name]]).
- why: (soundness) AC3 — pins the **structural** proof shape single-argument
  recursion forces. `assoc : (x y z) -> Equal (List a) (list_append a x
  (list_append a y z)) (list_append a (list_append a x y) z)` is likewise a
  genuine `elim_List` induction on `x` (base closes by `Refl`; step: IH + `Cons`
  congruence), **not** definitional. A build that (i) assumes `right_unit`
  closes by `Refl` **fails to elaborate** (this case), or (ii) "gives up" and
  postulates `right_unit`/`assoc` because `Refl` stuck — an **avoidable hidden
  delta**, a defect on an inductive carrier (the induction *is* the deliverable,
  not a bolt-on — [[sizing-a-subsume-fix-enumerate-every-piece]]). This is
  genuinely new vs
  the `Eq`/`Ord` seed: those laws over `Bool` are finite **case-splits**; the
  Monoid-over-`List` laws are the first requiring **induction** (an IH), so the
  "postulate = defect because provable" precondition is realized here by a real
  inductive proof, and the left/right asymmetry is a real conformance
  discriminator, not prose.

### stdlib/classes/monoid-law-fields-real-proofs-not-postulates (soundness)
- spec: `51 §5` (laws PROVED = zero-delta), `33 §5.3` (instance = record value +
  law proofs), `25 §3` (`trusted_base_delta`), `13 §2` (Σ-Intro re-check)
- given: two `Monoid (List a)`-shaped instances **identical** in their operation
  field (`<> = list_append a`, `mempty = Nil`), differing **only** in the law
  fields: (a) a **canonical** instance whose `assoc`/`left_unit`/`right_unit`
  are **real kernel proofs** (`Refl` at `left_unit`, `elim_List` inductions at
  `right_unit`/`assoc`); (b) a **law-less** instance whose law fields are
  `declare_postulate`d (and, as further arms, holed / stubbed-absent)
- expect: **the verdict flips.** (a) **accepts as lawful** — every law prop is
  kernel-proved, so `trusted_base_delta` is **empty** (the law props ∉
  `trusted_base()`); (b) **rejected as unlawful** — a **postulated** law field
  is an `Opaque` entry → **non-empty `trusted_base_delta`**; a **holed** field
  **fails the kernel re-check**; a **missing** field leaves the record value
  **uninhabited** (`Monoid (List a)` cannot be constructed). Assert the
  **observable**: (a) empty delta / law props ∉ `trusted_base()`; (b) non-empty
  delta or re-check/inhabitation failure — **not** a message string.
- why: (soundness) AC3, the algebra-carrier reading of the **exact** discipline
  the `Eq`/`Ord` seed pins for `Ord`/`DecEq`
  (`stdlib/classes/law-fields-real-proofs-not-postulates`, `51 §5`). **Does not
  re-pin** the `trusted_base()` / delta mechanism (ES1/Sec4,
  `../surface/taxonomy/minimality.md`) — pins that a `Monoid` instance's law
  fields are **subject** to it. The postulated arm is the discriminating
  green-vs-green the frame warns of: a postulated-law instance **type-checks**
  (the `Opaque` inhabits the field), so a test asserting only "a `Monoid`
  resolves" passes it vacuously ([[conformance-hand-feeds-the-deliverable]]) —
  the net is **structural** (grep the law fields for `declare_postulate`/holes;
  their **absence** is the guarantee). **Carrier precondition:** the postulate
  is a **defect because `List a` is inductive** (the laws *were* provable by
  `elim_List`); the same postulated field on a **primitive** carrier would be
  the honest **audited-delta** (`51 §6`), so the reject arm is conditioned on
  carrier provability, not the postulate alone.

---

## AC2 — the algebra laws are `Ω`-clean carrier-equalities (no truncation)

`51 §3`: every law field is `Ω`-valued. The Monoid laws conclude the **kernel
propositional equality** `Equal (carrier) u v`, which is `Ω` (proof-irrelevant)
in OTT — the **same shape** as `Ord`'s `antisym`/`DecEq`'s `sound`/`complete`
(all conclude `Equal a x y : Ω`), so no truncation is needed and no
proof-relevance leaks into the record.

### stdlib/classes/monoid-laws-omega-clean-carrier-equality (soundness)
- spec: `51 §3` (law-field sorts — every law is `Ω`), `16 §1`/`§6` (`Ω` /
  truncation), `13 §4` (`sort_sigma`)
- given: the `Monoid a` `assoc`/`left_unit`/`right_unit` law fields as authored:
  each a `Π` into `Equal a u v` (e.g. `right_unit : (x : a) -> Equal a
  (x <> mempty) x`); contrasted with a **hypothetical** proof-**relevant**
  formulation of a "law" (a `Type`-valued sum / bare `∨`, the `total`-as-bare-∨
  shape `ord-total-law-is-omega-bool-equation` rules out)
- expect: (a) the authored laws are **admissible `Ω` law fields** — `Equal a u v
  : Ω` is proof-irrelevant, **no truncation**, and the record's sort is `Type`
  **because of its op fields** (`<>`, `mempty`), never because a law leaked to
  `Type`; (b) a `Type`-valued "law" is **not** an admissible law field (it makes
  the record carry proof-relevant content the law must not, or needs `‖·‖`).
  Assert that each law field's type is a `Π` into `Equal a _ _ : Ω`, a
  proof-irrelevant proposition
- why: (soundness) the law-field-sort no-regression net for the algebra classes:
  the Monoid laws are equalities of **carrier values** (`Equal (List a) …`), and
  in OTT such equalities are `Ω` — the identical account that makes `Ord`'s
  `antisym`/`DecEq`'s `sound`/`complete` valid `Ω` law fields in the landed
  `lawful-classes` package. Unlike `Ord.total` (a disjunction in spirit needing
  the `Bool`-equation trick to dodge truncation,
  [[proof-relevant-inductive-cannot-be-declared-at-omega]]), the Monoid
  equations are **already** proof-irrelevant equalities — no truncation, no
  `Bool`-equation reformulation. This pins the sort forward so a later
  `Monoid`-variant cannot quietly regress a law to a proof-relevant `Type`.

---

## Semigroup ⊆ Monoid layering

`Semigroup` is `(<>)` + `assoc`; `Monoid` adds `mempty` + the two unit laws.
Whether `Monoid` **extends** `Semigroup` (superclass/subobject, `33 §5.4`) or
restates `<>`+`assoc` is spec-author's `(oracle)` choice; the invariant pinned
here is that the **`assoc` law is the same proposition** in both, so a
`Monoid`'s associativity is not a second, distinct law.

### stdlib/classes/semigroup-assoc-shared-with-monoid (oracle)
- spec: `51 §5` (laws PROVED), `33 §5.4` (superclass constraint / `where C a`
  desugaring), `50-stdlib/README §2` (the taxonomy row `Semigroup`/`Monoid`)
- given: the `assoc` law as it appears on `Semigroup a` and on `Monoid a` —
  whether `Monoid` carries `where Semigroup a` (inheriting `<>`/`assoc`) or
  restates them
- expect: the `assoc` proposition is **structurally identical** in both — `(x y
  z) -> Equal a (x <> (y <> z)) ((x <> y) <> z)` — and a canonical `Monoid (List
  a)` reuses the **same** `list_append` associativity proof a `Semigroup (List
  a)` would carry (no second, distinct associativity obligation). Assert the
  **observable**: the two `assoc` field types are the **same structural shape**
  (modulo the shared `<>`), not "both type-check"
- why: (oracle) reflect-don't-extend for the layering — `Monoid` is `Semigroup`
  **plus** identity, not a re-derivation of associativity. This is the
  `where-ord-same-sort-obligation` analog for the algebra layering: one `assoc`
  view, inherited or restated, never duplicated. Tagged `(oracle)` because the
  **layering spelling** (extends vs restates) is spec-author's to finalize; the
  **shared-`assoc`** invariant is normative regardless.

---

## HELD — `Functor`/`Foldable` conformance (pending the law-form ruling)

Per the CAT-1 kickoff (spec-leader, `evt_2y7c7w7c3p3wb`) and the frame's design
question §2, the higher-kinded classes' conformance is **not authored yet**. Two
inputs must land first, both routed to the Architect:

1. **Higher-kinded admission** — whether the landed `class`/instance machinery
   admits an `f : Type -> Type` record parameter today, or needs a pinned
   outer-ring elaborator extension (`buildability, every axis` —
   [[buildability-ruling-must-ground-every-axis]]). The discriminating
   conformance shape depends on how a higher-kinded instance elaborates.
2. **Functor-law statement form** — pointwise `(x : f a) -> Equal (f a)
   (map idf x) x` vs a function-level `Equal (f a -> f a) (map idf) idf` needing
   funext, and how OTT's observational equality discharges the fusion law
   `map (g ∘ f) = map g ∘ map f`. This is the form **CAT-2's Monad laws
   inherit**, so the conformance must pin the **ruled** form, not a guess —
   authoring it now would over-freeze a deferred degree of freedom (the T1 dual;
   [[layer-dependent-pin-at-unconditional-layer]]).

**When the rulings land**, the Functor/Foldable discriminators to author (noted
here as a placeholder, not pinned): the **identity-law false-`map` flip** (a
bogus `Functor` whose `map` is not identity-preserving is rejected — the AC4
analog over `f a`); the **fusion/composition law** in the ruled form; the
`Foldable`/`Monoid` coherence (`foldMap` via the `Monoid` this seed pins). These
inherit the false-witness + laws-proved-not-postulated discipline above, lifted
to the higher-kinded carrier.

---

## Coverage map

- **AC4** (false-witness rejected, soundness):
  `monoid-unit-law-false-witness-rejected` — wrong `mempty` makes a unit law a
  false equation; honest proof fails **conversion at the law field**, masked
  proof fails the **delta gate** (and inhabits `Bottom`). Two-arm net.
- **AC3** (laws PROVED, soundness):
  `monoid-unit-asymmetry-left-defn-right-inductive` (left unit definitional /
  right unit + assoc genuine `elim_List` induction — the Monoid-first
  structural-induction content),
  `monoid-law-fields-real-proofs-not-postulates` (provenance flip; references
  the shared `trusted_base()` mechanism, does not re-pin it).
- **AC2** (Ω-clean laws): `monoid-laws-omega-clean-carrier-equality` — the
  algebra laws are `Equal (carrier) _ _ : Ω`, proof-irrelevant, no truncation.
- **Layering** (oracle): `semigroup-assoc-shared-with-monoid` — one `assoc`
  view across `Semigroup`/`Monoid`.
- **HELD**: `Functor`/`Foldable` conformance — pending the Architect's
  higher-kinded-admission + Functor-law-form rulings.

## Cross-case consistency sweep

- **A false witness fails the law re-check; a masked (postulated) false law
  fails the delta gate.** `monoid-unit-law-false-witness-rejected` (honest
  `Refl` on a wrong `mempty` → conversion failure) and
  `monoid-law-fields-real-proofs-not-postulates` (a postulated law → non-empty
  delta) agree and **compose**: they are the two arms of the AC4 net — a false
  unit law cannot be honestly proved (conversion) **nor** honestly postulated
  (it is provable-when-true on this inductive carrier, so a postulate is a
  defect; and here it is *false*, so it inhabits `Bottom`). A case treating a
  postulated false law as an honest audited delta contradicts this pair
  ([[deceq-on-noncanonical-carrier-inhabits-bottom]]).
- **Left unit is definitional; right unit and assoc are inductive.**
  `monoid-unit-asymmetry-left-defn-right-inductive` is consistent with the
  landed `list_append` recursing on its first argument: `list_append a Nil x`
  reduces (left unit by `Refl`), `list_append a x Nil` is stuck on a variable
  (right unit + assoc by `elim_List`). A case expecting `right_unit`/`assoc` to
  close by `Refl`, or expecting `left_unit` to need induction, contradicts the
  recursion structure.
- **Every law field is `Ω`.** `monoid-laws-omega-clean-carrier-equality` agrees
  with the `51 §3` law-field-sort pin and the landed `Ord`/`DecEq` `Equal a x y`
  law fields: the record is `Type`-sorted because of its **op** fields, never
  because a law leaked to `Type`. A case with a proof-relevant `Type`-valued
  "law" contradicts the structure-class sort discipline.
- **One `assoc` view across the layering.** `semigroup-assoc-shared-with-monoid`
  agrees with reflect-don't-extend (`where-ord-same-sort-obligation` analog):
  `Monoid` is `Semigroup` plus identity, so associativity is one proposition,
  inherited or restated, never a second distinct obligation.

## Subsumed / not-duplicated (one home per property)

- **The `trusted_base()` / zero-delta mechanism** is **ES1/Sec4's**
  (`../surface/taxonomy/minimality.md`, `../../security/trust-model/`) and its
  law-side reading is pinned once in **`seed-lawful-classes.md`** (the
  `Eq`/`Ord` seed). This seed pins only the **Monoid-specific** content (false
  witness, induction asymmetry) and **references** the shared provenance flip;
  it does not re-pin the delta computation.
- **The K4 Ω-motive elimination** capability (`14 §3`) is
  **`../../kernel/inductive/seed-k4-omega-motive-elim.md`'s**. The Monoid
  inductions consume it (`elim_List` into an `Equal`-motive); this seed does not
  re-pin the elimination rule.
- **The class mechanism** (record elaboration, `sort_sigma`, `instance_search`,
  `where`-desugaring) is **`33 §5`'s** (surface). This seed pins the **lawful
  content** (the algebra law proofs are real and, when the witness is wrong,
  fail for the right reason), not the resolution machinery.
- **`Functor`/`Foldable`** conformance is **HELD** (not authored) pending the
  Architect's law-form ruling — it is not homed anywhere yet, by design.
