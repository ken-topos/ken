# CAT-1 constructor-class conformance — seed cases (Semigroup/Monoid)

Format: `../../README.md`. First WP of the **catalog campaign**
(`docs/program/06-catalog-campaign.md`;
`docs/program/wp/CAT-1-constructor-classes.md`).
CAT-1 extends the landed `packages/lawful-classes` pattern (`Eq`/`DecEq`/`Ord`,
over a **value type** `a : Type`) to the workhorse **constructor classes** and
their value-level algebra companions. This seed pins the **discriminating
conformance for the value-level algebra classes — `Semigroup`/`Monoid`** (the
warm-up: same shape as `Eq`/`Ord`, over `a : Type`, no new kind machinery) —
**and the higher-kinded classes `Functor`/`Foldable`** (over `f : Type ->
Type`). The Functor/Foldable cases were **held** until the Architect's two CAT-1
design rulings landed (`evt_2h347mddbxwfb`); both are now in, and the
Functor/Foldable discriminators (below) are authored against them:
- **Higher-kinded admission (Axis A): NO today** — the landed `elab_class_decl`
  hard-codes the class param to `Type0` (three `Term::ty(Level::Zero)` sites,
  `crates/ken-elaborator/src/elab.rs` ~L1862-1902; `parse_class_decl` takes a
  bare ident, no kind binder), so a `class Functor f` over `f : Type -> Type`
  needs a **pinned outer-ring elaborator extension** (kernel-untouched, no new
  `Term`/`Decl`) — a CAT-1 sub-deliverable, not pure package Ken. The
  Functor/Foldable conformance is **red-until-built** against that extension +
  the instances (the static-vs-runtime split below).
- **Functor law form: POINTWISE is canonical** — funext is **definitional** in
  Ken's OTT (independently verified: `obs.rs:77` routes `Eq (Π..) f g` to
  `eq_at_pi`, `obs.rs:90` reduces `Eq ((x:A1)→B1) f g ⇝ (x:A1) → Eq (B1 x)
  (f x) (g x)`, cited `16 §2.2`), so the function-level law whnf-reduces to the
  pointwise law — **the same proposition, one reduction apart**, both Ω-clean,
  no truncation. Pointwise is the normal form (the prover's goal *is* the stated
  law), so one canonical pointwise field per law; a point-free restatement is
  definitionally equal and must **not** proliferate a second field.

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

## AC4/AC3 — Functor: the pointwise laws discriminate a bogus `map`

`Functor f` (over `f : Type -> Type`, needing the Axis-A extension) has
`map : (a b : Type) -> (a -> b) -> f a -> f b` and two laws in the
**Architect-ruled pointwise form** (`evt_2h347mddbxwfb`, grounded on the
funext-definitional fact verified above):

- **id:** `(a : Type) -> (x : f a) -> Equal (f a) (map a a (idf a) x) x`
- **fusion (applied-pointwise):** `(a b c : Type) -> (g : b -> c) -> (h : a ->
  b) -> (x : f a) -> Equal (f c) (map a c (comp a b c g h) x) (map b c g
  (map a b h x))`

Canonical instances `List`/`Option` (both inductive ⇒ proved by induction,
zero-delta). `idf`/`comp` are ordinary Ken views. The false-`map` flip below is
the AC4 analog over `f a` — the same discipline as
`monoid-unit-law-false-witness-rejected`, lifted to the higher-kinded carrier.

**Reconcile-at-transcription.** The pointwise form is Architect-ruled but the
durable **`spec/51 §4` (constructor-class template)** transcription is
spec-author's (Architect fidelity-gates) — not yet landed. These cases cite the
**ruling + `obs.rs` funext fact** (independently verified), not a `§4` anchor
that does not yet carry the claim
([[grounding-a-fabricated-citation-two-failure-modes]]); the `§4` cite is added
when the transcription lands, which the fidelity gate confirms.

### stdlib/classes/functor-id-law-false-map-rejected (soundness)
- spec: the Architect law-form ruling (`evt_2h347mddbxwfb`, pointwise id-law),
  `obs.rs:90` (funext definitional, `16 §2.2`), `51 §5` (laws PROVED = real
  proofs), `14 §3` (K4 `elim_List` into an `Equal`-motive); reconciled to
  `51 §4` at transcription
- given: two `Functor List`-shaped instances, identical in every field
  **except** `map`, with the **same** proof term attempted for the id-law:
  (a) the **canonical** `map` (`map a b fn (Cons x xs) = Cons (fn x)
  (map a b fn xs)`, `map a b fn Nil = Nil`) with `map_id` a real `elim_List`
  induction on `x`;
  (b) a **bogus** structure-dropping `map` (`bad a b fn xs = Nil` — well-typed
  at `(a -> b) -> f a -> f b`, so nothing rejects it *as an operation*), whose
  id-law is offered at the concrete carrier value `x = Cons True Nil`
- expect: **the verdict flips on the `map`, witnessed at a concrete carrier.**
  (a) **accepts** — `map_id`'s inductive proof closes (base `Nil`: `map (idf)
  Nil ≡ Nil`, `Refl`; step: IH + `Cons` congruence), law fields re-check.
  (b) **rejected — conversion failure at the id-law field**: at `x =
  Cons True Nil` the goal is `Equal (List Bool) (bad … (Cons True Nil))
  (Cons True Nil)` = `Equal (List Bool) Nil (Cons True Nil)`, a **false**
  equation (`Nil` vs `Cons`-headed), which **no** honest proof term can close
  (`Refl` fails: `Nil ≢ Cons True Nil`). Assert the **specific observable**:
  (a) elaborates + law re-checks; (b) elaboration fails **at the id-law field**
  with a conversion/constructor-clash on `Nil` vs `Cons True Nil` — **not**
  `is_err()`, not a missing-field/kind error
  ([[assert-specific-error-variant-not-is-err]])
- why: (soundness) AC4 over the higher-kinded carrier — the Functor id law is
  the identity-preservation guarantee downstream generic code assumes; a `map`
  that is not identity-on-values makes it a **false** `Equal (f a)`, unprovable.
  **Two-arm net** ([[two-arm-producer-needs-a-case-per-arm]]): the honest proof
  fails **conversion** (this case); a **masked** proof (`map_id = Axiom`) fails
  the **delta gate** (`functor-law-fields-real-proofs-not-postulates` below) and
  **inhabits `Bottom`** (`Axiom : Equal (List Bool) Nil (Cons True Nil)` is a
  false postulate — [[deceq-on-noncanonical-carrier-inhabits-bottom]]). The
  concrete carrier value `x = Cons True Nil` is the discriminating input
  Architect named ("breaks the pointwise id-law at a concrete carrier →
  rejected right-reason"). **Verdict-flip**, keyed solely on the `map`.

### stdlib/classes/functor-fusion-law-pointwise-direct-induction (soundness)
- spec: the ruling (`evt_2h347mddbxwfb`, applied-pointwise fusion),
  `obs.rs:90` (funext definitional), `51 §5`, `14 §3`; reconciled to `51 §4`
- given: the fusion law on the canonical `Functor List` in **applied-pointwise**
  form (`map (comp g h) x = map g (map h x)`), and a **bogus** `map` (e.g. one
  that reverses or duplicates) whose fusion equation is false at a concrete `x`
- expect: (a) the canonical fusion closes by **direct `elim_List` induction on
  `x`** — **no funext layer to strip first**, because pointwise **is** the
  normal form (the function-level `Equal (f a -> f c) (map (comp g h)) (comp
  (map g) (map h))` whnf-reduces to exactly this pointwise goal by `obs.rs:90`);
  (b) the bogus `map`'s fusion is a false `Equal (f c)` at a concrete carrier →
  **conversion failure**, right-reason. Assert: the stated goal **is** the
  prover's goal (no funext strip), and the bogus arm rejects at the fusion field
- why: (soundness/AC3) pins that fusion is discharged by direct induction in the
  pointwise form — the property CAT-2's Monad laws inherit. Grounds the
  "pointwise is the normal form" ruling on the verified `obs.rs:90` reduction:
  were the law stated function-level, it would reduce to this same goal, so the
  induction is identical — which is *why* one canonical field suffices
  (`functor-one-canonical-pointwise-field` below).

### stdlib/classes/functor-one-canonical-pointwise-field (soundness)
- spec: the ruling (`evt_2h347mddbxwfb`, "do not proliferate a second law
  field"), `obs.rs:77`/`:90` (funext definitional), `51 §3` (law-field `Ω`
  sorts), `16 §2.2`
- given: the Functor id/fusion laws stated **pointwise** (one field each) vs a
  build that **additionally** carries the **function-level/point-free**
  restatement (`Equal (f a -> f a) (map (idf a)) (idf (f a))`) as a **second**
  law field
- expect: (a) the pointwise fields are `Ω`-clean (`Equal (f a) _ _ : Ω`,
  proof-irrelevant, **no truncation** — direct value equations, the `§3`
  truncation catch does **not** fire); (b) the point-free restatement is
  **definitionally equal** to the pointwise one (funext, `obs.rs:90`), so a
  **second** law field carrying it is **redundant proliferation** — the record
  must carry **one** canonical pointwise field per law, not two. Assert: each
  law is a single pointwise `Ω`-field; a duplicated point-free field is flagged
  as reflect-don't-extend proliferation, not a distinct obligation
- why: (soundness) the no-proliferation / `Ω`-cleanliness pin for the
  higher-kinded laws, the `functor` analog of
  `semigroup-assoc-shared-with-monoid` and
  `monoid-laws-omega-clean-carrier-equality`. Because funext is
  **definitional** (verified: `obs.rs:90` reduces the function-level `Eq` to the
  pointwise one), the two forms are the same proposition — so the point-free
  equation is available **for free** and a second field is content-free
  duplication. Guards a build that "adds the categorical law too."

### stdlib/classes/functor-law-fields-real-proofs-not-postulates (soundness)
- spec: `51 §5` (laws PROVED = zero-delta), `33 §5.3` (instance = record value +
  law proofs), `25 §3` (`trusted_base_delta`)
- given: two `Functor List`-shaped instances identical in `map` (the canonical
  `map`), differing only in the law fields: (a) real `elim_List` id/fusion
  proofs; (b) postulated / holed / stubbed-absent law fields
- expect: **verdict flips** — (a) accepts, empty `trusted_base_delta`; (b)
  rejected (postulate → non-empty delta; hole → re-check failure; missing →
  uninhabited record). Assert the delta/re-check observable, not a message
- why: (soundness) AC3 for the higher-kinded carrier — the exact provenance flip
  `monoid-law-fields-real-proofs-not-postulates` pins for `Monoid`, lifted to
  `Functor`. **References** the shared `trusted_base()` mechanism (ES1/Sec4),
  does not re-pin it. Carrier precondition: `List`/`Option` are inductive, so a
  postulated Functor law is an avoidable **defect** (the laws *were* provable by
  `elim_List`), the second arm of the false-`map` two-arm net.

---

## Foldable — element-preservation + `Monoid` coherence

`Foldable f` has `foldr` (and/or `foldMap` via `Monoid`) + the fold laws + the
`Monoid` coherence; instances `List`/`Option`. **The fold interface — `foldr`-
primary vs `foldMap`-primary — is spec-author's `(oracle)` choice** (frame
deliverable §3), so these pin the discriminating **shape** and the coherence
**tie**, tagging the interface + exact-law spellings oracle, not over-freezing
them.

### stdlib/classes/foldable-element-dropping-fold-rejected (soundness)
- spec: `51 §5` (laws PROVED), `14 §3` (`elim_List`), the frame's Foldable
  deliverable §3; reconciled to the landed `51`/package
- given: two `Foldable List`-shaped instances differing in the fold operation,
  with a **fold law** that pins **element preservation** (e.g. the
  reconstruction law `foldr Cons Nil xs = xs`, or a `toList`/length-agreement
  law — the exact law is `(oracle)`, spec-author's interface choice):
  (a) the canonical fold; (b) a **bogus** fold that **drops** an element
  (e.g. skips the head), so the preservation law is false at a concrete `xs`
- expect: **verdict flips at a concrete carrier** — (a) the preservation law
  closes by `elim_List` induction; (b) the bogus fold's law is a false `Equal`
  at (e.g.) `xs = Cons True (Cons False Nil)` → **conversion failure**,
  right-reason (not a missing-field/kind error). Assert the concrete-carrier
  conversion-failure observable
- why: (soundness) the AC4 analog for `Foldable` — a fold that silently drops or
  reorders elements is the fold-family's "wrong witness," caught by a
  preservation law at a concrete carrier, the same discipline as the Functor
  id-law and Monoid unit-law flips. The **exact preservation law** and the
  **`foldr`-vs-`foldMap` interface** are `(oracle)` (spec-author finalizes); the
  **element-preservation discriminator** is normative regardless
  ([[two-arm-producer-needs-a-case-per-arm]]: masked-postulate arm → delta gate)

### stdlib/classes/foldable-monoid-coherence (oracle)
- spec: the frame's Foldable §3 (`foldMap` via `Monoid`), `51 §5`, this seed's
  `Monoid` cases; reconciled to the landed interface choice
- given: `foldMap g` and its factorization through the pinned `Monoid`
  (`foldMap g xs ≡ foldr (\x acc. g x <> acc) mempty xs`) on `Foldable List`
- expect: the coherence equation **holds** (proved by `elim_List` induction over
  `xs`, leaning on the `Monoid` `assoc`/`unit` laws this seed pins), and a fold
  that does **not** factor through the `Monoid` (uses a non-`mempty` seed or a
  non-`<>` combine) **fails** the coherence law. Assert the coherence obligation
  is emitted and discharged via the `Monoid`, not re-derived
- why: (oracle) the `Foldable`↔`Monoid` seam — `foldMap` is *defined* through
  the `Monoid`, so `Foldable`'s correctness **leans on** this seed's `Monoid`
  laws (the CAT-3 collection-laws hook). Tagged `(oracle)` because the interface
  primary (`foldr` vs `foldMap`) is spec-author's; the
  **coherence-leans-on-Monoid** invariant is normative. This is where CAT-1's
  value-level algebra (`Monoid`) and its first constructor class (`Foldable`)
  compose — the pattern CAT-3 reuses.

---

## Higher-kinded admission — the mechanism flip (surface's home)

The **capability flip** for the Axis-A extension — a `class C (f : Type ->
Type) { … }` binder is **admitted** post-extension (the `(f : K)` binder parses,
`f a` type-checks with `f : Type0 -> Type0`) and **rejected** pre-extension
(the landed `Type0`-pinned param makes `f a` a non-Π application) — is the
observable of the pinned sub-deliverable. It is a **class-mechanism** property,
so its home is **`../surface/classes/seed-classes.md`** (`33 §5`), not this
lawful-content seed (one home per property). Noted here as the **build-verify
dependency** the Functor/Foldable law cases above sit on: they are
**red-until-built** against that extension + the higher-kinded instances (the
static-vs-runtime split). The extension is Architect-sized, kernel-untouched
(zero `ken-kernel` diff, no new `Term`/`Decl`); the CV producer-grep at the
build gate confirms AC1 (`ken-kernel` diff empty) on the built diff.

---

## Coverage map

- **AC4** (false-witness rejected, soundness):
  `monoid-unit-law-false-witness-rejected` (wrong `mempty` → false unit law;
  honest proof fails **conversion**, masked proof fails the **delta gate** and
  inhabits `Bottom`), `functor-id-law-false-map-rejected` (bogus `map` → false
  pointwise id-law at a concrete carrier),
  `foldable-element-dropping-fold-rejected` (element-dropping fold → false
  preservation law). Two-arm nets.
- **AC3** (laws PROVED, soundness):
  `monoid-unit-asymmetry-left-defn-right-inductive` (left unit definitional /
  right unit + assoc genuine `elim_List` induction),
  `monoid-law-fields-real-proofs-not-postulates` +
  `functor-law-fields-real-proofs-not-postulates` (provenance flips; reference
  the shared `trusted_base()` mechanism, do not re-pin it),
  `functor-fusion-law-pointwise-direct-induction` (fusion by direct induction in
  the normal-form pointwise shape, no funext strip).
- **AC2** (Ω-clean laws): `monoid-laws-omega-clean-carrier-equality`,
  `functor-one-canonical-pointwise-field` — laws are `Equal (carrier/f a) _ _ :
  Ω`, proof-irrelevant, no truncation; one canonical pointwise field per law
  (funext-definitional ⇒ no second point-free field).
- **Layering / coherence** (oracle): `semigroup-assoc-shared-with-monoid` (one
  `assoc` view across `Semigroup`/`Monoid`), `foldable-monoid-coherence`
  (`foldMap` factors through the pinned `Monoid`).
- **Mechanism (surface's home)**: the higher-kinded-admission capability flip is
  `../surface/classes/`'s; the Functor/Foldable law cases are
  **red-until-built** against the Axis-A elaborator extension (pinned CAT-1
  sub-deliverable).

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
- **The false-witness discipline is one rule across all three classes.**
  `monoid-unit-law-false-witness-rejected`, `functor-id-law-false-map-rejected`,
  and `foldable-element-dropping-fold-rejected` agree: a wrong operation witness
  (a non-identity `mempty`, a structure-dropping `map`, an element-dropping
  fold) makes a law a **false `Equal`**, caught at a **concrete carrier** by a
  **conversion failure** on the honest proof and by the **delta gate** (+
  `Bottom`-inhabitation) on a masked postulate. A case rejecting a false-witness
  instance for an *unrelated* reason (missing field / kind error) contradicts
  the right-reason discipline this class enforces.
- **Pointwise is the normal form; one canonical field per higher-kinded law.**
  `functor-fusion-law-pointwise-direct-induction` and
  `functor-one-canonical-pointwise-field` agree with the verified `obs.rs:90`
  funext reduction: the function-level law whnf-reduces to the pointwise one, so
  the pointwise field **is** the prover's goal (direct induction, no strip) and
  a second point-free field is content-free proliferation. A case stating a
  Functor law function-level-**only** (needing a funext strip before induction),
  or carrying both forms as distinct obligations, contradicts this pair.

## Subsumed / not-duplicated (one home per property)

- **The `trusted_base()` / zero-delta mechanism** is **ES1/Sec4's**
  (`../surface/taxonomy/minimality.md`, `../../security/trust-model/`) and its
  law-side reading is pinned once in **`seed-lawful-classes.md`** (the
  `Eq`/`Ord` seed). This seed pins the **CAT-1-specific** content (false witness
  across all three classes, the induction asymmetry, the pointwise-normal-form
  fact) and **references** the shared provenance flip; it does not re-pin the
  delta computation.
- **The K4 Ω-motive elimination** capability (`14 §3`) is
  **`../../kernel/inductive/seed-k4-omega-motive-elim.md`'s**. The Monoid and
  Functor/Foldable inductions consume it (`elim_List` into an `Equal`-motive);
  this seed does not re-pin the elimination rule.
- **The funext-definitional reduction** (`Eq (Pi..) f g ⇝` pointwise,
  `obs.rs:90`, `16 §2.2`) is the **observational kernel's**
  (`../../kernel/observational/`).
  The Functor pointwise-normal-form cases **consume** it as a ground fact; they
  do not re-pin the funext rule.
- **The class mechanism** (record elaboration, `sort_sigma`, `instance_search`,
  `where`-desugaring, **and the higher-kinded param binder** `class C (f : K)`)
  is **`33 §5`'s / `../surface/classes/`'s**. This seed pins the **lawful
  content** (the law proofs are real and, when the witness is wrong, fail for
  the right reason), not the resolution machinery nor the Axis-A admission flip.
- **`Functor`/`Foldable` law conformance is now authored** (the Architect
  law-form ruling landed, `evt_2h347mddbxwfb`); the pointwise form's durable
  `spec/51 §4` anchor is **reconciled at spec-author's transcription** (which
  the CV fidelity gate confirms), and the cases are **red-until-built** against
  the Axis-A extension + instances.
