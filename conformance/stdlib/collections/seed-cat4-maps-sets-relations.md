# CAT-4 — Maps / Sets / Relations laws (Layer 2) — conformance seed

Format: `../../README.md`. These pin the **CAT-4 deliverable**
(`docs/program/wp/CAT-4-maps-sets-relations.md`; the spec chapter is
spec-author's landed chapter `spec/50-stdlib/58-maps-sets-relations.md` (tip
`516ba78`, Architect's 3 fidelity fold-ins applied) — cites below are
**re-pointed to `58 §X`** and re-anchored to that post-fold-in tip). CAT-4
**extends the landed Map capstone** (`catalog/packages/collections/map.ken`,
`spec/50-stdlib/54-map-verified-laws.md` — laws 1/2/3/5 + `toList`-ordered
proved) with `delete`, the keyed-merge ops
(`union`/`intersection`/`difference`), `keys`/`values` coherence, the
Set-algebra laws (Set = Map-Unit), and the `Relation` frontier (properties now;
transitive closure design-now/build-later). **Outer-ring, kernel-untouched, zero
`Axiom`, zero `trusted_base()` delta** — every law is a real proof over the
landed carriers, `subsume`/`reuse`-don't-re-derive.

**Grounded (content-verified against `origin/main @ 7169300` — `map.ken`,
`54`/`52`, `16 §1.1`–`§1.4`/`§6`, `lawful_classes.ken`, `prelude.rs` — plus the
Architect fork rulings `evt_55htg0ss8y1v6`/`evt_3z7c592g37rtr`, not heading
numbers, per the `conformance-oracle-grounding-fallback` discipline):**

- **Reuse corpus (`map.ken`):** `Tree k v = Leaf | Node`; `insert`/`lookup`/
  `member`/`toList`/`fold`/`fromList`/`fromListAcc`; `Ordered`/`allKeys` (`Ω`
  via `IsTrue b := Equal Bool b True` + derived `And`,
  `Leaf → Equal Bool True True` K7-collapses to `Top` → `tt`); the convoy idiom
  (`boolDichotomy b : Or (Equal Bool b True) (Equal Bool b False)`, `Or` a
  `Type`-sum built directly against `declare_inductive`); laws 1/2/3/5 +
  `toList`-ordered (`preservesOrdered`, `lookupFoundAfterInsert`,
  `lookupLocality`, `lookupAssocAgree` (`map.ken:2212`,
  `lookup key m ≡ assoc key (toList m)` under `Ordered`+`Distinct`),
  `toListOrdered`) + `allKeysTransBelow`/`Above`, `isSortedAppend`, and the
  `trans`/`cong` "stop-one-step-short" transport bridges. **Zero `Axiom`, zero
  `trusted_base` delta throughout.** Permutation is the one Map law still
  deferred (proof-relevant, C5).
- **All six D1–D3 ops absent** on `main`
  (`git grep '^view (delete|union| intersection|difference|keys|values) '`
  empty) + `glue`/`deleteMin`/`pairVals` absent — every CAT-4 op is net-new
  (red-until-built). `setInsert`/`setMember`/ `setToList` + `allKeys`/`pairKeys`
  landed (Set = Map-Unit, `Tree a Unit`).
- **Two build prerequisites, both ordinary total Ken (kernel-untouched):**
  `size` (`Tree k v → Nat`, the closure bound `N`, **absent on main**) plus the
  D0 carrier `leqNat` (`Nat → Nat → Bool`) + its 4 order laws (**absent on
  main** — `git grep 'Nat -> Nat -> Bool'` empty; `Nat` is inductive so refl/
  antisym/trans/total are genuinely provable Axiom-free by structural
  induction).

**Architect fork rulings (source of truth, `evt_55htg0ss8y1v6` +
`evt_3z7c592g37rtr`):**

- **Fork A — `union` = combining fn**
  `(V → V → V) → Map K V → Map K V → Map K V` (subsumes left/right bias; `f`
  takes `(from-a, from-b)`, Haskell `unionWith` orientation). `f` touches **only
  values** — `Ordered`-preservation is `f`-independent. **Map union is NOT
  commutative** in general — maps get only the **lookup characterization** +
  `Ordered`-preservation, **never** a map-commutativity law. Impl
  `union f a b := fold (\k v acc. insertWith f k v acc) b a`.
- **Fork B — transitive closure = BOUNDED/DECIDABLE reachability**, `Ω`-native
  `IsTrue`, NOT `‖·‖`-truncation. `R⁺ x y := IsTrue (reachableWithin N x y)`,
  `N := size (dom R)`. The exact CAT-3 `Perm` move — push proof-relevant
  "there-exists-a-path" into a **decidable Bool** bounded computation, then
  bridge `IsTrue (·) := Equal Bool · True` (`16 §1.1` predicative Π-into-`Ω`).
  **No raw multi-ctor `data … : Ω`** (proof-relevant, inadmissible
  `16 §1.4`+§1.1). Faithful: any walk shortens to a simple path ≤ `N−1`, so
  bounded reachability at `N−1` **equals** full closure. **Decidable beats
  truncation decisively for conformance: a truncated `‖Reach_rel‖` is
  non-computational ⇒ NO verdict-flip case can discriminate it**
  (green-vs-green); `IsTrue (reachableWithin N)` reduces to a concrete `Bool` a
  broken closure flips.
- **Fork C-rep — `Map K (Set K)` (adjacency), NOT `Set (Pair K K)`.** Rides
  `Ord K` only (`Set (Pair K K)` would force a total lexicographic
  pair-comparator + 4 pair-order-laws; the landed `pairLeq` compares first
  components only, non-total on pairs). Just `Tree K (Tree K Unit)` — a
  landed-`Tree` instantiation at `v := Set K`, zero new machinery. **C-scope:
  LAND NOW** compose/converse/`succ`/membership +
  `isReflexive`/`isSymmetric`/`isTransitive`/`isEquivalence` (Π-into-`Ω`);
  **DESIGN-NOW/DEFER-BUILD** transitive closure (rep pinned; faithfulness law +
  `size` are the fast-follow — stated, not silently dropped).
- **Fork D — `delete` = REBUILD-via-`fromList`.**
  `delete key m := fromList leq (dropKey key (toList m))`, `dropKey` =
  **filter** (removes **ALL** matching entries, not drop-first). Non-recursive
  (zero own SCT). `Ordered`-preservation reuses landed `preservesOrdered`
  wholesale via one `fromListPreservesOrdered`. **None-law
  `lookup key (delete key m) ≡ None` is UNCONDITIONAL** (nothing with `key`
  survives to `fromList` — filter semantics); the **other-key** law threads
  `Ordered`+`Distinct` through the law-5 roundtrip.
- **Sub-ruling (1) — set laws MEMBERSHIP-EXTENSIONAL:**
  `(∀k. setMember k lhs ≡ setMember k rhs)`, **NEVER** `Equal (Set K) lhs rhs`.
  Tree-`Equal` set laws are **false** (fold+insert produces shape-different
  trees with the same key-set) — extensional is the **only sound** formulation,
  and is what makes the set laws corollaries of the map lookup-characterization
  + `bool_or`/`bool_and` algebra.
- **Sub-ruling (2) — carrier:** build a real Axiom-free `leqNat` + 4 laws as the
  D0 prerequisite; proved discriminators key on **`Map Nat`/`Set Nat`/
  `Relation Nat`**, **never `Map Int`** (`Ord Int`/`Ord Char` are Axiom-holed ⇒
  the proved accept-arm goes vacuous, the flip degenerates to reject-vs-reject).
  `Ord Bool` is Axiom-free but 2-key — too degenerate for a ≥3-key relation
  discriminator (a→b→c ⊬ a→c). The carrier-vacuity guard binds (the CAT-3
  `List Bool` lesson, one carrier up).

**Status — all cases RED-UNTIL-BUILT** (every CAT-4 op is net-new; the build is
Runtime-owned, held for the GPT window). This is the intended CAT-1-Functor
posture: the seed is the contract the build codes against; the exact per-branch
endpoint tokens (delete lookup-side `Refl` vs `tt`; the
`reachableWithin`/`size`/ `isTransitive` spellings) reconcile with CV at the
build alongside Architect's AC1/AC5 re-cert. Two soundness/boundary pins ship
now as **design** assertions (the closure `Ω`-native rep; the set-extensionality
soundness), assertable ahead of the proofs.

---

## Scope — canonical shapes (Architect-pinned; reconcile spellings at merge)

```
-- D0 carrier prerequisite (Axiom-free, Nat inductive):
leqNat : Nat -> Nat -> Bool
leqNatRefl   : (x : Nat) -> Equal Bool (leqNat x x) True
leqNatTrans  : (x y z : Nat) -> Equal Bool (leqNat x y) True
             -> Equal Bool (leqNat y z) True -> Equal Bool (leqNat x z) True
leqNatTotal  : (x y : Nat) -> Or (Equal Bool (leqNat x y) True)
                                  (Equal Bool (leqNat y x) True)
leqNatAntisym: (x y : Nat) -> Equal Bool (leqNat x y) True
             -> Equal Bool (leqNat y x) True -> Equal Nat x y

-- D1 delete (Fork D — rebuild, dropKey = filter):
dropKey (k v : Type) (leq : k -> k -> Bool) (key : k)
        (xs : List (Pair k v)) : List (Pair k v)      -- filter out ALL key
delete  (k v : Type) (leq : k -> k -> Bool) (key : k)
        (m : Tree k v) : Tree k v
      = fromList k v leq (dropKey k v leq key (toList k v m))
-- laws:
--   None-law  (UNCONDITIONAL): lookup key (delete key m) = None
--   other-key (Ordered+Distinct): k /= key ->
--                lookup k (delete key m) = lookup k m
--   Ordered-preservation: Ordered m -> Ordered (delete key m)

-- D2 union (Fork A — combining fn f (from-a, from-b)); lookup characterization:
union (k v : Type) (leq : k -> k -> Bool) (f : v -> v -> v)
      (a b : Tree k v) : Tree k v
--   lookup k (union f a b) =
--     None,None    -> None
--     Some x,None  -> Some x
--     None,Some y  -> Some y
--     Some x,Some y-> Some (f x y)          -- orientation f x y, NOT f y x
--   Ordered-preservation: f-independent
-- MAPS GET NO COMMUTATIVITY LAW (union f a b /= union f b a unless f commutes).

-- Set algebra (Set = Map-Unit) — MEMBERSHIP-EXTENSIONAL, never Equal (Set K):
--   setUnionComm: (k : Nat) ->
--     Equal Bool (setMember k (setUnion a b)) (setMember k (setUnion b a))
--   (idempotence / identity / difference-membership: same extensional shape)

-- D3 keys/values:
keys   (k v : Type) (m : Tree k v) : List k       -- = pairKeys (toList m)
values (k v : Type) (m : Tree k v) : List v       -- = pairVals (toList m)
--   keys-coherence: mem k (keys m) = isSome (lookup k m)   (Ω / Bool bridge)

-- D4 relations (Fork C — adjacency Map K (Set K)):
Relation (K : Type) : Type = Tree K (Tree K Unit)
succ (K : Type) (x : K) (r : Relation K) : Tree K Unit
relMember (K : Type) (leq : K -> K -> Bool) (x y : K) (r : Relation K) : Prop
        = IsTrue (setMember K leq y (succ K x r))       -- already Prop (Ω-native)
compose  (K)(leq)(R S : Relation K) : Relation K    -- succ x (R.S) = U{succ y S}
converse (K)(leq)(R : Relation K)   : Relation K    -- y in succ x R <=> x in succ y R'
isTransitive (K)(leq)(r : Relation K) : Prop =                    -- Pi-into-Omega
  (x y z : K) -> relMember x y r -> relMember y z r -> relMember x z r
-- NOTE: relMember is ALREADY IsTrue(...), so premises are NOT re-wrapped.
-- transitive closure (DESIGN-NOW / DEFER-BUILD):
--   TransClosure x y := IsTrue (reachableWithin N x y), N := size (dom r)
--   NOT data TransClosure : ... : Omega   (proof-relevant, inadmissible)
```

---

## AC1 — kernel-untouched, outer-ring

### stdlib/collections/cat4-kernel-untouched-outer-ring (soundness)
- spec: `58 §9` AC1, `52 §1.1`/§9 (the Map-container AC1 precedent), `16 §1`.
- given: the CAT-4 build diff (`delete`/`union`/`intersection`/`difference`/
  `keys`/`values`/`Relation` ops + laws + the `size`/`leqNat` prerequisites).
- expect: `git diff origin/main -- crates/ken-kernel/` is **empty**; **zero
  `trusted_base()` delta**; **no new `Term`/`Decl`**. Every op is a
  `declare_def` `view` over the **existing** landed `Tree` carrier + the
  existing `declare_inductive`/`Term::Trunc`/`Term::Elim` machinery;
  `size`/`leqNat` are ordinary total Ken. Assert structurally (grep the
  admission calls + the empty kernel diff), not from prose.
- why: AC1's derived-not-primitive core — CAT-4 rides the outer ring entirely
  ([[enriching-opaque-former-kind-is-kernel-clean]] sibling: even the `Relation`
  former is a plain `Tree K (Set K)` instantiation, no kind extension).
  **`(soundness)`** — a kernel touch or a new `Decl` for any CAT-4 op would grow
  the TCB. Structural (empty kernel diff + admission-kind), not a value.
  (structural; kernel-untouched; red-until-built.)

---

## AC2 — reuse the landed capstone, zero re-derivation

### stdlib/collections/delete-union-reuse-landed-no-rederivation (soundness)
- spec: `58 §9` AC2 / `58 §1 pt 4`, `54 §5` (laws 1–5), `map.ken`
  (`preservesOrdered`/`lookupAssocAgree`/`fold`/`insert`).
- given: the `delete`/`union`/`intersection`/`difference` proof terms as
  admitted.
- expect: `Ordered`-preservation for **all four** routes through **one**
  `fromListPreservesOrdered`-shaped lemma reusing the landed
  **`preservesOrdered`** per-insert step (base `orderedEmpty`), and the
  lookup-side delete/merge laws route through the landed **law 5
  (`lookupAssocAgree`)** + its `fromList` dual — **no fresh two-tree
  simultaneous-descent induction, no re-proof of laws 1–5**. Grep-clean of
  `Axiom`/`declare_postulate` for every CAT-4 law. **The flip:** a build that
  re-derives a preservation lemma from scratch **and** `Axiom`-stubs a step is
  caught by the zero-delta cone walk (below); a faithful reuse adds zero delta.
- why: `§2 pin 2` (`subsume`/`reuse`-don't-re-derive), the WP's central
  directive. **`(soundness)`** — a re-derivation that smuggles an `Axiom` reads
  `proved`-by-default ([[untrusted-layer-backstop-hole-for-omissions]]); the
  delta net is the backstop. Structural (proof-term shape + delta membership).
  (soundness; structural reuse + zero-delta; red-until-built.)

### stdlib/collections/leqnat-d0-real-axiom-free-carrier (soundness)
- spec: `58 §2` sub-ruling 2, `51 §6` (Axiom-free order carriers),
  `../classes/seed-lawful-classes.md` (`Ord Bool` the only landed Axiom-free
  order carrier; `Ord Int`/`Ord Char` Axiom-holed).
- given: the D0 prerequisite `leqNat` + its 4 order laws (refl/antisym/trans/
  total) as admitted.
- expect: each of the 4 laws is a **real, kernel-checked** proof term over the
  inductive `Nat` (structural induction on the two `Nat` args), **zero `Axiom`,
  zero `trusted_base` delta** — `Nat` being inductive, its order laws are
  genuinely provable (unlike the `Int` primitive, whose `Ord` laws are honest
  `Axiom`s). **The flip:** stub any of the 4 with `Axiom` → the cone walk counts
  it → non-empty delta → **rejected**; the real proof → empty delta → accepted.
- why: the carrier prerequisite that makes the proved CAT-4 discriminators
  **non-vacuous** — see the standing carrier discriminator below.
  **`(soundness)`** — an `Axiom`-holed `leqNat` would make every proved
  `Map Nat` law's accept-arm vacuous. Structural delta-flip. (soundness;
  structural zero-delta; red-until-built.)

---

## AC3 — invariant preservation (the convoy idiom over `Tree`)

### stdlib/collections/delete-preserves-ordered (soundness)
- spec: `58 §3.1` D1, `54 §5.1` (`preservesOrdered`), Fork D
  (`evt_3z7c592g37rtr`).
- given: `Ordered m → Ordered (delete key m)` as a proof term, over `Map Nat`
  (real `leqNat`).
- expect: **accepts** — `delete = fromList ∘ dropKey ∘ toList`, so preservation
  is `fromListPreservesOrdered` (plain `List` induction: each step = the landed
  per-insert `preservesOrdered`, base = `orderedEmpty` `Leaf → tt`; the
  `fromListAcc Nil acc → acc` base is a **passthrough of `acc`'s `Ordered`, NOT
  `tt`** — per the sharpened `55 §3.2`/`57 §1 pt 3` endpoint rule,
  build-pinned). **Zero `Axiom`, zero delta.** **The flip:** a `delete` that
  emitted a non-`Ordered` tree (e.g. a glue that mis-orders the promoted node)
  could not discharge preservation → `Axiom`-stub → non-empty delta →
  **rejected**.
- why: AC3's invariant face, reusing the landed machinery wholesale (the reason
  Fork D chose rebuild). **`(soundness)`** — structural zero-delta + the
  per-branch base-witness pin (`tt` on the `Leaf`/`Equal Bool True True`
  collapse, passthrough on `fromListAcc Nil`, `Refl`/`cong` on neutral steps —
  never uniform [[tt-vs-refl-endpoint-rule-for-inductive-equal-law-bases]]).
  (soundness; structural; red-until-built; endpoint tokens build-pinned.)

### stdlib/collections/union-preserves-ordered-f-independent (soundness)
- spec: `58 §4` D2, Fork A (`f` touches only values).
- given: `Ordered a → Ordered b → Ordered (union f a b)`, over `Map Nat`.
- expect: **accepts, and the proof does NOT mention `f`** —
  `union f a b = fold (\k v acc. insertWith f k v acc) b a`, and `insertWith`'s
  **key** placement is `insert`'s, so `Ordered`-preservation is `f`-independent
  (`f` only combines **values** at a collision, never moves a key). One shared
  `foldPreservesOrdered`-shaped lemma covers
  `union`/`intersection`/`difference`. **Mechanism-consistency check (my V2
  carry):** the three merge ops must agree on the shape of this shared lemma.
  **The flip:** a `union` whose merge mis-placed a key by `Ordered` fails
  preservation → `Axiom` → delta → reject.
- why: AC3 for the merge ops; pins `f`-independence structurally (guards a
  conformance over-claim that threads `f` into the invariant proof).
  **`(soundness)`** — structural (proof does not cite `f`) + zero-delta.
  (soundness; structural; red-until-built.)

---

## D1 — `delete` laws

### stdlib/collections/delete-none-law-unconditional
- spec: `58 §3.2` D1, Fork D (filter `dropKey`, unconditional None-law).
- given: `lookup key (delete key m) ≡ None`, over a **real inserted** `Map Nat`
  (computes) — driven through real `delete`/`lookup`, not hand-fed.
- expect: **reduces-to `None`** with **no `Ordered`/`Distinct` hypothesis** —
  `dropKey` filters out every entry matching `key`, so nothing with `key`
  survives to `fromList`, and `lookup` finds nothing. A `delete` that fails to
  remove the key (or removes a different key) yields `Some v` — the value-flip.
- why: D1's headline delete law, made executable (seed-map
  `insert-lookup-roundtrip` twin, delete side). **Drive the real producer:**
  build via real `insert`, delete via real `delete`, read via real `lookup`.
  (reduces-to value-flip; red-until-built.)

### stdlib/collections/dropkey-drop-first-fails-none-law-dup (soundness)
- spec: `58 §3` D1, Fork D **build-pin** (`dropKey` = filter, NOT drop-first;
  `evt_3z7c592g37rtr` + spec-leader `evt_51q1f8jcshgcd`).
- given: a **hand-built duplicate-key** tree
  `Node (Node Leaf k v1 Leaf) k v2 Leaf` (a non-`Distinct` input, the law-5
  counterexample shape) over `Nat`, and `lookup k (delete k ·)`.
- expect: **filter-`dropKey` reduces-to `None`** (removes **both** `k` entries →
  nothing survives) — the None-law holds **unconditionally**. A **drop-first**
  `dropKey` (removes only the first match) leaves the second `k` entry →
  `fromList` keeps it → `lookup k` reduces-to `Some v` → **the None-law FAILS**.
  The two `dropKey` semantics land on **opposite** verdicts on this input; over
  a `Distinct` map they coincide (both `None`) — the dup input is what
  discriminates.
- why: pins the exact soundness-adjacent build-pin — **filter is what makes the
  None-law unconditional**; drop-first silently reintroduces a `Distinct`
  dependency. **`(soundness)`** — a drop-first build passes every
  `Distinct`-input case (green-vs-green) and only this dup-input case catches
  it; keyed on the **structural** discriminator (duplicate-key survivor). Honest
  note: the input is a non-`Distinct` hand-built tree (not reachable by real
  `insert`), asserting the **robustness** the filter semantics guarantee.
  (soundness; verdict-flip on `dropKey` semantics; red-until-built.)

### stdlib/collections/delete-other-key-law-threads-ordered-distinct
- spec: `58 §3.2` D1, `54 §5` law 5 (`lookupAssocAgree`, `map.ken:2212`), Fork D
  (other-key threads `Ordered`+`Distinct`).
- given: when `k` is **order-distinct** from `key` (i.e.
  `IsTrue (orderEquivKey leq k key)` is **false**, with
  `orderEquivKey leq a b := bool_and (leq a b) (leq b a) : Bool` — Bool-valued
  per fold-in 1, `58 §3`), the other-key law
  `lookup k (delete key m) ≡ lookup k m`, over a **real inserted** `Map Nat` (so
  `Ordered`+`Distinct` hold), with `k` an **untouched** key present in `m`.
- expect: **`lookup k (delete key m)` reduces-to the SAME `Some v` as
  `lookup k m`** — deleting `key` leaves `k`'s entry untouched. Non-degenerate:
  probe an **untouched present** key (probing only the **deleted** key is
  green-vs-green — both `None`, `delete-none-law` territory). A `delete` that
  removes the wrong subtree / too much yields `None` or a wrong value at `k` —
  the flip.
- why: D1's locality face; the law-5-roundtrip law (reuses landed
  `lookupAssocAgree` + `fromList` dual + a `dropKey`/`assoc` lemma). The
  discriminator MUST probe an untouched key — the
  [[assert-specific-error-variant-not-is-err]] / non-degenerate-pair rule.
  (reduces-to value-flip; red-until-built.)

---

## D2 — `union`/`intersection`/`difference` + set algebra

### stdlib/collections/union-lookup-characterization-four-case
- spec: `58 §4` D2, Fork A (combining fn `f (from-a, from-b)`, 4-case
  orientation).
- given: `lookup k (union f a b)` at each of the 4 cases, over `Map Nat`, with a
  **non-commutative `f`** (e.g. `f x y = x`, left-projection) and a key present
  in **both** `a` and `b` with distinct values.
- expect: **reduces-to** — both-`None`→`None`; `(Some x, None)`→`Some x`;
  `(None, Some y)`→`Some y`; `(Some x, Some y)`→**`Some (f x y)`** (orientation
  `f x y`, the `a`-value first). The collision case with a non-commutative `f`
  **pins the orientation**: `Some (f x y)`, NOT `Some (f y x)`. A `union` that
  drops `b`'s exclusive keys, or applies `f` reversed, flips the observed value.
- why: D2's headline map law (`§5.2`-analog for merge). The **non-commutative
  `f` + both-present key** is the discriminator: a commutative `f` or a
  one-side-only key would be green-vs-green on the orientation. (reduces-to
  value-flip; orientation-pinned; red-until-built.)

### stdlib/collections/map-union-not-commutative-no-false-law (soundness)
- spec: `58 §4` D2, Fork A (**maps get NO commutativity law**).
- given: over `Map Nat` with a **non-commutative `f`** and a key `k` present in
  both with `x ≠ y`: `lookup k (union f a b)` vs `lookup k (union f b a)`.
- expect: **they reduce to DIFFERENT values** — `Some (f x y)` vs
  `Some (f y x)`, and `f x y ≠ f y x`. So **`union f a b ≡ union f b a` is
  FALSE** for maps — a conformance case (or a spec law) asserting map-union
  commutativity **unconditionally** is **unsound** (it asserts a false
  equation). Only the **lookup characterization** (above) and
  `Ordered`-preservation are map laws; commutativity is **Set-only** (below, and
  only because `V = Unit` makes `f` trivial). Assert the **dis-equality**
  structurally.
- why: the "don't over-claim" boundary — pins that map-union commutativity is
  **false, not merely unproved**. **`(soundness)`** — a build (or seed) that
  shipped a map-commutativity law would either `Axiom`-postulate a false
  equation or fail to elaborate; the discriminator is the concrete
  counterexample. Keyed on the non-commutative `f` (a commutative `f` hides it).
  (soundness; verdict-flip on a false-law claim; red-until-built.)

### stdlib/collections/set-union-comm-extensional-not-tree-equal (soundness)
- spec: `58 §5`/`§9 AC6` sub-ruling 1, `52 §4.4` (Set = Map-Unit), `16 §1.3`
  (`Ω` props).
- given: two formulations of set-union commutativity over `Set Nat`
  (`Tree Nat Unit`): **(A) membership-extensional**
  `(∀k. setMember k (setUnion a b) ≡ setMember k (setUnion b a))`; **(B)
  Tree-`Equal`** `Equal (Set Nat) (setUnion a b) (setUnion b a)`.
- expect: **(A) accepts, (B) rejects** — the two formulations **flip**. (A) is a
  real proof: both sides reduce (pointwise) to
  `bool_or (setMember k a) (setMember k b)` vs the swap, closed by `bool_or`
  commutativity (finite `2×2` via landed `boolDichotomy`), **no `Tree`
  induction**. (B) is **false, not merely unprovable**: `setUnion a b` and
  `setUnion b a` are built by `fold`+`insert` and produce **shape-different
  trees with the same key-set**, so `Equal (Set Nat) …` between them is
  uninhabited (would need `Node`-injectivity on distinct shapes). A build
  proving (B) would inhabit `Bottom` (or `Axiom`-postulate a false `Equal`).
- why: the load-bearing sub-ruling-(1) soundness pin — **extensional is the ONLY
  sound set-law formulation**. **`(soundness)`** — the non-degenerate PAIR
  ([[differential-verify-which-mechanism-is-the-net]] discipline: same inputs,
  two formulations, opposite verdicts), keyed on the **structural**
  discriminator (extensional-membership vs Tree-`Equal`). A single (A)-accept
  case is green-vs-green under a build that never tries (B). (soundness;
  verdict-flip pair on formulation; red-until-built.)

---

## D3 — `keys` / `values` coherence

### stdlib/collections/keys-coherence-mem-iff-issome-lookup
- spec: `58 §6` D3, `52 §5.3` (`toList`-ordered), `map.ken`
  (`pairKeys`/`allKeys`/`toListOrdered`).
- given: `mem k (keys m)` vs `isSome (lookup k m)`, over `Map Nat`, for **k
  present** and **k absent** (the non-degenerate pair).
- expect: **the two `Bool`s reduce to the SAME value** — `True`/`True` for a
  present key, `False`/`False` for an absent one. `keys m = pairKeys (toList m)`
  (landed pieces), so membership in `keys` tracks `lookup`-someness exactly. A
  `keys` that drops an entry (→ `mem` `False` while `lookup` `Some`) or
  duplicates/fabricates one flips the agreement.
- why: D3's coherence law; reuses `pairKeys`/`toList`. Non-degenerate present +
  absent pair (a present-only test is green-vs-green under a "keys returns
  everything" bug). (reduces-to value-flip; red-until-built.)

### stdlib/collections/keys-ascending-off-tolistordered
- spec: `58 §6` D3 (keys-ascending coherence), `54 §5` (`toListOrdered`),
  `map.ken` (`pairKeys`/`pairLeq`).
- given: `isSorted leq (keys m)` over an `Ordered` `Map Nat` built by inserting
  keys in **non-ascending** order (e.g. `insert 3 · insert 1 · insert 2`).
- expect: **reduces-to a proof / ascending list** —
  `keys m = pairKeys (toList m)`, and the landed `toListOrdered` gives
  `isSorted (pairLeq leq) (toList m)`; since `pairLeq` compares first components
  (the keys), projecting via `pairKeys` **preserves** the ascending order, so
  `isSorted leq (keys m)` holds regardless of insertion order. A `keys` that
  emitted in insertion/tree order (not the `toList` order) yields a
  non-ascending list — the flip is on the **list order**.
- why: D3's ordering-coherence face (`58 §6`), **distinct** from the membership
  coherence above — reuses the landed `toListOrdered` (the reconcile-surfaced
  gated addition; `values` carry NO such claim, the adjacent case). Insertion
  out of order is the discriminator (a pre-sorted input is green-vs-green).
  (value-flip on list order; red-until-built.)

### stdlib/collections/values-no-ordering-coherence-claim
- spec: `58 §6` D3 (`values` = `pairVals (toList m)`; no ordering claim).
- given: the `values` op + the D3 law surface.
- expect: `values m` yields the values **in `toList` (ascending-key) order**,
  but **no conformance case asserts `values` is itself sorted** — values carry
  no order (`pairVals` is `pairKeys`'s mirror with `pairSnd`). The **only**
  ordering coherence is `keys`'s (above), tracking `toListOrdered`. A case
  asserting `isSorted (values m)` would false-fail a faithful build (values
  needn't be ordered).
- why: pins the `keys`/`values` **asymmetry** honestly — keys inherit the key
  ordering, values do not. Guards a conformance over-claim. (boundary; named
  scope; red-until-built.)

---

## D4 — `Relation` (LAND-NOW half + the deferred-closure boundary)

### stdlib/collections/non-transitive-relation-fails-istransitive (soundness)
- spec: `58 §7` D4 / Fork C-scope (LAND-NOW `isTransitive` Π-into-`Ω`),
  `16 §1.1`.
- given: a concrete relation over **≥3 distinct `Nat` keys** `{a, b, c}` as
  adjacency `Map Nat (Set Nat)`: `R = {a↦{b}, b↦{c}}` (edges a→b, b→c, **no**
  a→c), and its transitive-completion `R' = R ∪ {a↦{b,c}}`.
- expect: **proof-flip on the Π-into-`Ω` predicate `isTransitive` over `Nat`** —
  it is `(x y z) → relMember x y r → relMember y z r → relMember x z r`
  (`relMember` is **already** `IsTrue (setMember …)` : Prop, `58 §7` — the
  premises are NOT re-wrapped in `IsTrue`). For `R'` it is **provable** (each
  live triple's conclusion holds; every other triple is vacuous — a false
  premise `relMember _ _ R'` discharges by `absurd`). For `R` the sole live
  triple `(a,b,c)` has conclusion `relMember a c R`, which reduces to
  `IsTrue False` = **uninhabited** → **not provable** → **reject**. Accept `R'`
  / reject `R`: opposite verdicts, keyed on the closing edge a→c.
- why: D4's headline relation discriminator (`58 §7`, the chapter's own
  example). **`(soundness)`** — needs **≥3 distinct keys** (a→b→c ⊬ a→c), which
  `Bool` (2 keys) **cannot** exhibit — hence the `Nat` carrier prerequisite
  binds here. Over `Nat` the Ω-predicate itself IS the discriminator (no
  decidable `Bool` `isTransitive` is defined in `58 §7`); its accept-arm proof
  needs the Axiom-free `leqNat`, so a `Map Int` accept-arm goes vacuous and the
  flip degenerates to reject-vs-reject. Non-degenerate pair on the closing edge.
  (soundness; Ω-predicate proof-flip; ≥3-key carrier-gated; red-until-built.)

### stdlib/collections/relation-properties-are-pi-into-omega
- spec: `58 §7` D4 / Fork C-scope, `16 §1.1` (Π-into-`Ω` is sound — properties
  are not proof-relevant).
- given: `isSymmetric` over concrete `Nat` relations — a symmetric `R_s` (`a↔b`)
  and a non-symmetric `R_a` (`a→b`, no `b→a`).
- expect: **proof-flip** —
  `isSymmetric r := (x y) → relMember x y r → relMember y x r` is **provable**
  for `R_s` (each live pair's converse holds, the rest vacuous) and **not
  provable** for `R_a` (the `(a,b)` obligation `relMember b a R_a` reduces to
  `IsTrue False`). `isEquivalence` = the `And` of refl/sym/trans. These are
  `Π`-into-`Ω` predicates (**fine** — proof-irrelevant, no path data), **not**
  proof-relevant inductives. Non-degenerate: the missing `b→a` edge.
- why: pins the cheap `Ω`-provable half of D4 that **lands now** (`58 §7`).
  **Reflexivity caveat (soundness — a self-catch):**
  `isReflexive r := (x:k) → relMember x x r` quantifies over **all** of
  `k = Nat`, so a **finite** adjacency relation can **never** inhabit it (most
  `x` have empty `succ`) — a sound `isReflexive`-accept needs a
  reflexive-by-construction (total-on-domain) relation, not a finite one;
  symmetry/transitivity avoid this (vacuous off the finite live set), which is
  why the chapter's own discriminator is transitivity, not reflexivity. Do NOT
  author a finite-relation `isReflexive`-accept arm. (proof-flip on the
  Ω-predicate; red-until-built.)

### stdlib/collections/converse-and-compose-membership
- spec: `58 §7` D4 (LAND-NOW `compose`/`converse` + their laws), `16 §1.1`.
- given: over `Nat` relations (adjacency `Map Nat (Set Nat)`): the `converse`
  law `relMember y x (converse R) ⇔ relMember x y R`, and the `compose` law
  `relMember x z (compose R S)` reflecting
  `succ x (compose R S) = ⋃ { succ y S : y ∈ succ x R }` (landed
  `fold`/`union`/`member`).
- expect: **value-flip on relation membership.** With `R = {a→b}`:
  `relMember b a (converse R)` holds while `relMember a b (converse R)` reduces
  to `IsTrue False`; a `converse` that fails to transpose flips it. With
  `R = {a→b}`, `S = {b→c}`: `relMember a c (compose R S)` holds (composed via
  `b`) while `relMember a b (compose R S)` does not; a `compose` that mis-unions
  the `S`-images flips it. Non-degenerate: a present composed edge AND an absent
  one.
- why: the LAND-NOW `compose`/`converse` half of D4 needs its **own**
  discriminators — `58 §7` lands them with laws, and a corpus testing only the
  property predicates would leave compose/converse untested (my coverage-gap
  catch at the reconcile). Reuses landed `fold`/`union`/`member`. (value-flip on
  relation membership; red-until-built.)

### stdlib/collections/transitive-closure-decidable-not-raw-omega (soundness)
- spec: `58 §7`/`§1 pt 1` Fork B (`evt_55htg0ss8y1v6`), `16 §1.4`+§1.1
  (proof-relevant inductive inadmissible at `Ω`),
  `../../challenge/C2-proof-relevant-omega`.
- given: the transitive-closure **representation** as pinned:
  `R⁺ x y := IsTrue (reachableWithin N x y)`, `N := size (dom R)`,
  `reachableWithin` a **decidable Bool** bounded fixpoint over the adjacency
  `Map K (Set K)`.
- expect: the closure is an **`Ω`-native value-equation**
  (`IsTrue (·) := Equal Bool · True`, `16 §1.1` predicative Π-into-`Ω`) —
  **NOT** a raw multi-ctor `data TransClosure … : Ω` carrying paths (that is the
  inadmissible proof-relevant inductive, `16 §1.4`+§1.1, the exact `Perm`
  hazard). **The soundness discriminator vs the wrong design:** a raw-`Ω` path
  inductive admits a `Type`-relevant leak into `Ω` (distinct paths = distinct
  proofs) that collapses `true ≡ false` — inadmissible; the `IsTrue`-bounded
  form has proof-irrelevant `Ω` content. **AND the conformance-testability
  discriminator (my lane, Fork B reason 1):** the decidable form **reduces to a
  concrete `Bool`** a broken closure flips (a→c present/absent over ≥3 `Nat`
  keys), so D5 gets a real verdict-flip; a `‖·‖`-truncated closure is
  **non-computational** ⇒ green-vs-green, no discriminator. Pin the rep as
  `IsTrue`-decidable, not truncation, not raw-`Ω`.
- why: AC5's relation-`Ω`-soundness pin, the CAT-3 `Perm` move applied to
  closure (same `16 §1.4`+§1.1 hazard). **`(soundness)`** — a raw-`Ω` closure is
  a consistency finding; the `IsTrue`-bounded rep is the checkable substitute.
  Reconcile note: the landed prelude ships a **different** `Ω`-safe permutation
  — `Perm := ‖Perm_rel‖` (truncation, `prelude.rs:778`, matching `37 §6`) — and
  CAT-3 `57 §3.1` ships count-equality `Perm (a)(eqf)…`; closure takes
  **neither**, it takes decidable-bounded (Fork B), for the computability +
  L14-model-check-fit reasons. (soundness; design-rep pin; verdict-independent
  structural; red-until-built.)

### stdlib/collections/closure-faithfulness-and-size-deferred
- spec: `58 §7` Fork C-scope (DESIGN-NOW/DEFER-BUILD).
- given: the transitive-closure **faithfulness law** (`reachableWithin N` = full
  closure, via simple-path shortening + `N`-round fixpoint saturation) and its
  `size` (`Tree k v → Nat`) prerequisite.
- expect: **deferred — representation pinned (above), the `.ken` faithfulness/
  saturation proof + `size` are the named fast-follow.** A CAT-4 conformance
  suite **must not** assert the faithfulness behavior (bounded = full closure)
  yet, and the absence is the **named** Fork-C-scope deferral, **not** a silent
  scope truncation. The LAND-NOW half (compose/converse/properties + the closure
  *rep*) is complete; the closure *proof* defers. Relations feed L14 + Lane B —
  the seam is stated, not over-built.
- why: pins the Fork-C-scope split honestly (the
  [[untrusted-layer-backstop-hole-for-omissions]] absent-clause discipline:
  state what defers, so its absence never reads as coverage). (boundary; named
  deferral; red-until-built.)

---

## Carrier — the standing vacuity guard

### stdlib/collections/proved-carrier-is-lawful-nat-not-int (soundness)
- spec: `58 §2`/`§9 AC7` sub-ruling 2, `52 §5.4` (Axiom-holed
  `Ord Int`/`Ord Char`), `../classes/seed-lawful-classes.md`, the CAT-3
  `List Bool` carrier lesson.
- given: the proved CAT-4 discriminators (delete/union/set/relation laws) and
  the choice of key carrier.
- expect: the proved accept-arms key on **`Map Nat`/`Set Nat`/`Relation Nat`**
  with the **real Axiom-free `leqNat`** dict — **never `Map Int`**. Over `Int`
  (or `Char`), `Ord` is **Axiom-holed**, so a "proved" law's accept-arm cites an
  `Axiom` and the arm goes **vacuous**; the discriminator flip degenerates to
  **reject-vs-reject** (green-vs-green). And `Bool` (Axiom-free) has only 2 keys
  — **too few** for a ≥3-key relation-transitivity discriminator. So the sound
  proved carrier is **`Nat` + real `leqNat`**. **The flip:** a build that
  shipped the proved arms over `Map Int` would have a **vacuous** accept-arm
  (its "proof" leans on the `Ord Int` `Axiom`) — the carrier-check catches it.
- why: the carrier-vacuity guard **promoted to a standing discriminator** so the
  build cannot silently pick an Axiom-holed carrier and ship a vacuous green
  (exactly CAT-3's `verified-sort-proved-carrier-is-lawful-bool`, one carrier up
  — Nat here because relations need ≥3 keys). **`(soundness)`** —
  verdict-independent structural (which carrier + whether its `Ord` cites an
  `Axiom`). (soundness; standing carrier discriminator; red-until-built.)

---

## Coverage map (AC / deliverable → cases)

- **AC1 (kernel-untouched):** `cat4-kernel-untouched-outer-ring`.
- **AC2 (reuse, zero re-derivation):**
  `delete-union-reuse-landed-no-rederivation`,
  `leqnat-d0-real-axiom-free-carrier`.
- **AC3 (invariant preservation):** `delete-preserves-ordered`,
  `union-preserves-ordered-f-independent`.
- **D1 (`delete` laws):** `delete-none-law-unconditional`,
  `dropkey-drop-first-fails-none-law-dup`,
  `delete-other-key-law-threads-ordered-distinct`.
- **D2 (`union` + set algebra):** `union-lookup-characterization-four-case`,
  `map-union-not-commutative-no-false-law`,
  `set-union-comm-extensional-not-tree-equal`.
- **D3 (`keys`/`values`):** `keys-coherence-mem-iff-issome-lookup`,
  `keys-ascending-off-tolistordered`, `values-no-ordering-coherence-claim`.
- **D4 (relations LAND-NOW + deferred closure):**
  `non-transitive-relation-fails-istransitive`,
  `relation-properties-are-pi-into-omega`, `converse-and-compose-membership`,
  `transitive-closure-decidable-not-raw-omega` (AC5 rep pin),
  `closure-faithfulness-and-size-deferred` (named deferral).
- **Carrier (standing vacuity guard):** `proved-carrier-is-lawful-nat-not-int`.

## Cross-case consistency sweep

- **Carrier discipline uniform:** every proved soundness accept-arm keys on
  `Nat` + real `leqNat` (never `Int`/`Char` Axiom-holed, never `Bool` for the
  ≥3-key relation cases) — the standing guard binds all of them.
- **Commutativity, one story across cases:** `map-union-not-commutative` (maps —
  false) and `set-union-comm-membership-extensional` (sets — true, extensional)
  do **not** contradict: map union is non-commutative in general; set union is
  commutative **only** because `V = Unit` trivializes `f`, and **only** stated
  membership-extensionally (Tree-`Equal` is false for both). No case asserts map
  commutativity; no case asserts a Tree-`Equal` set law.
- **`delete` laws, hypothesis-consistent:** None-law **unconditional** (filter);
  other-key **threads `Ordered`+`Distinct`** (law-5 roundtrip);
  `dropkey-drop-first` fires **only** on a non-`Distinct` dup input — the three
  agree on when `Distinct` is / isn't needed (Fork D's exact split).
- **Endpoint tokens per-branch, NOT uniform:** `Leaf`/`Equal Bool True True`
  collapse → `tt`; `fromListAcc Nil acc` → passthrough of `acc` (not `tt`);
  neutral steps → `Refl`/`cong`; the delete lookup-side laws'
  non-nullary-head-with-neutral- component closers → `Refl` (the sharpened
  `57 §1 pt 3` rule) — build-pinned, reconciled at the build.
- **Closure rep, one design:** `transitive-closure-decidable-not-raw-omega` (rep
  = `IsTrue` bounded, not raw-`Ω`, not truncation) and
  `closure-faithfulness-and-size-deferred` (proof + `size` deferred) are the two
  halves of Fork B/C-scope — rep now, proof later, nothing silently dropped.

## Reconcile items for the merge gate (rolling, per the CAT-3 carry)

Cites are re-pointed to the **landed chapter `58`**, re-anchored to the
post-fold-in tip **`516ba78`**. Reconcile DONE: (a) **field-name spellings**
aligned (`fromListPreservesOrdered`/`insertWith`/`dropKey`/`reachableWithin`/
`size`/`relMember`/`succ`/`isTransitive`/`orderEquivKey`); (b) **gated coherence
additions** folded with a discriminator each — `compose`/`converse` (LAND-NOW,
`58 §7`) and keys-ascending (`58 §6`), plus the
`isTransitive`-not-double-`IsTrue` shape fix and the
`isReflexive`-over-infinite-`Nat` soundness caveat
([[transcription-moves-contract-requires-three-part-reconcile]]); (c) per-branch
**endpoint tokens** deferred to the build. **Re-anchor VERIFIED:** the diff
`de88e5b → 516ba78` is **exactly** Architect's 3 fold-ins (1: `orderEquivKey`
Bool-valued `bool_and` + `bool_and`/`bool_not` moved to D0 — aligned in this
seed; 2: `antisymLeq Zero/Zero → tt` not `Refl`, nullary-endpoint; 3: `transLeq`
`x=Zero → tt` base) — all chapter `§2`/`§3` `leqNat`-proof details, **none moves
a D5 discriminator**. Reconcile + re-anchor COMPLETE; ready for the
Spec/fidelity vote on the assembled candidate. The fidelity vote binds the
seed's **citations AND coverage** to the landed chapter, not just its verdicts.
