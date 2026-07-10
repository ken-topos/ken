# Maps, sets, and relations laws (CAT-4, Layer 2)

> Status: **DRAFT v0** (CAT-4). This chapter is the **contract** for Layer-2
> keyed-collection laws (`delete`, `union`/`intersection`/`difference`,
> `keys`/`values`), the **set algebra** that rides `Set = Map Unit`, and the
> **relations frontier** (composition, converse, the property predicates, and
> transitive closure). It **builds on the landed Map capstone** (`54`/`52`,
> `catalog/packages/collections/map.ken`) — laws 1/2/3/5 + `to_list`-ordered are proved
> there and are **reused, not re-derived** (frame §2 pin 2). It inherits `55`/
> `57`'s lawful-class template: laws are `Ω` propositions **proved over the
> landed carriers, zero `Axiom`, zero `trusted_base()` delta**, by the convoy
> induction + `trans`/`cong` route-around grammar (`54 §2`/`§3`) with the
> **sharpened per-branch `tt`-vs-`Refl`** endpoint rule (`57 §1 pt 3`).
> **CAT-2/CAT-3-independent** — value-level, no `Monad`/collection-view needed.
> **Kernel-untouched, outer-ring.** Four design forks are resolved here
> (Architect, `main@7169300f`): **A** `union` takes a **combining function**;
> **B** transitive closure is **bounded-reachability `IsTrue`** (`Ω`-native, the
> `Perm` move — never a raw multi-ctor `data … : Ω`); **C** a relation is
> `Map K (Set K)` (adjacency), with an explicit **land-now / defer-build** scope
> split; **D** `delete` is **rebuild-via-`from_list`**. Two enclave sub-rulings:
> set laws are stated **membership-extensionally** (never `Equal (Set K)`), and
> **`leq_nat` + its four order laws are a D0 carrier prerequisite** (`Nat`, not
> the `Axiom`-holed `Ord Int`/`Ord Char`). The `.ken` proofs are the **Runtime
> build** (Map was its substrate), held for the GPT window; this chapter is the
> elaboration.

## 1. What CAT-4 inherits (`54`/`52` and `55`/`57`)

Every law below is authored to five inherited points, so nothing is
re-litigated:

1. **`Ω` law fields, no proof-relevant inductive** (`55 §4`, `16 §6`). A lookup
   or membership equation is `Equal T u v : Ω` (or `IsTrue b := Equal Bool b
   True`), a direct value equation. The one construct that is *proof-relevant as
   a relation* — the **transitive closure** — is **not** a raw multi-ctor
   `data TC : … : Ω` (that is the inadmissible proof-relevant inductive, `16
   §1.4`+§1.1, the exact `Perm` hazard); it is pushed into a **decidable
   bounded-reachability `Bool`** and wrapped in `IsTrue` (`§7`, Fork B) — the
   same move CAT-3 made for `Perm`.
2. **Proved by the convoy idiom + the Gap-A route-around** (`54 §2`/`§3`): a law
   over the `Tree` carrier is a recursive `view` that reflects each stuck `leq k
   k'` through `bool_dichotomy` (a Gap-B dependent match on a `Bool`
   **variable**)
   and transports once with `J` (`53 §3`). The IH is an **ordinary
   self-recursive call on the subtree** (`54 §2.1` — the kernel's IH-slot binder
   is dead/surface-unreferenceable), never a synthesized `ih_l`/`ih_r`.
3. **Per-branch `tt`-vs-`Refl`**, never uniform (`55 §3.2`/`57 §1 pt 3`,
   sharpened by CAT-3): a branch closes with `tt` when both endpoints reduce to
   the **same fully-collapsing** head — a **nullary** ctor or one whose
   components all collapse — going to `Top` (K7); and with `Refl` when they
   reduce to a **neutral**, *including a non-nullary head with any neutral
   component* (it stays `Eq`-shaped, and `tt : Top` would be ill-typed there).
   The landed capstone bases are the template: `ordered_empty`/`lookup_empty_is_none
   → tt` (operation reduced into a collapsing `Equal Bool True True`); a
   `from_list_acc Nil acc` base is a **passthrough** (the accumulator's own
   `Ordered`, not `tt`).
4. **Reuse, don't re-derive, the Map capstone** (frame §2 pin 2). `delete`/
   `union`/`intersection`/`difference` build **on** the landed
   `insert`/`lookup`/
   `to_list`/`fold`/`from_list`/`member` + `preserves_ordered` (law 1) +
   `lookup_assoc_agree` (law 5) + `assoc`. **`Set a = Map a Unit`** (landed
   `set_insert`/`set_member`/`set_to_list`, spelled `Tree a Unit` directly — no
   `Set` type alias is landed); the set-algebra laws ride that identity.
5. **Surface spelling.** SURF-1's `view → const`/`fn`/`proc` migration is a
   **deferred build** — on `main` the lexer recognizes only `view`, and
   `catalog/packages/collections/` is 100% `view`-spelled. **New CAT-4 ops are written
   `view`**, matching every landed sibling; the migration re-spells them
   uniformly when SURF-1's build lands. (`Nat`'s successor is `Suc`, not `Succ`
   — `prelude.rs`, `data Nat = Zero | Suc Nat`.)

## 2. D0 — the `leq_nat` carrier prerequisite (the vacuity guard)

Before any law can be *discriminated* (a conformance flip that fails on the
wrong proof), CAT-4 needs a carrier with **≥ 3 distinct keys under an
`Axiom`-free order**. This is the CAT-3 `List Bool` lesson, sharpened: the
proved accept-arm is only load-bearing if the carrier's order laws are
**genuinely inhabited**.

- `Ord Int`/`Ord Char` on `main` are **`Axiom`-holed** — their order laws are
  postulated, so a proof that *uses* them is vacuous and the intended flip
  degenerates to reject-vs-reject (green-vs-green — no discrimination).
- `Ord Bool` is `Axiom`-free but has **only 2 keys** — it **cannot** exhibit a
  three-node transitivity witness (`a → b → c`: is `a → c` in the closure?),
  which the relation/closure discriminators require.
- **`Nat` is the carrier.** `leq_nat` + its four order laws are **net-new,
  ordinary total Ken, zero `Axiom`, kernel-untouched** — provable by structural
  induction on `Nat` (`data Nat = Zero | Suc Nat`, prelude). No `leq_nat`/`Ord
  Nat`/`nat_sub`-comparator exists on `main` today (grepped); this is the D0
  build item, within AC1/AC2, **not** a Steward re-fork.

```
view leq_nat (m : Nat) (n : Nat) : Bool =
  match m {
    Zero  => True ;
    Suc m2 => match n { Zero => False ; Suc n2 => leq_nat m2 n2 }
  }
```

The four laws are the **unbundled bare-parameter dictionary** the capstone
already threads (`52 §2` Architect-ruled encoding — `reflLeq`/`transLeq`/`total`
supplied as separate parameters, projected from a landed instance at the call
site). Proof shapes, all `Nat`-structural, comparison-driven only through
`leq_nat`'s own recursion:

- **`reflLeq : (x:Nat) → IsTrue (leq_nat x x)`** — induction on `x`: `Zero` base
  `leq_nat Zero Zero ⇝ True`, goal `Equal Bool True True → Top`, closed **`tt`**;
  `Suc m` step `leq_nat (Suc m)(Suc m) ⇝ leq_nat m m`, closed by the self-call IH
  `reflLeq m` (**neutral** result → the equality is carried, not collapsed).
- **`transLeq : (x y z:Nat) → IsTrue (leq_nat x y) → IsTrue (leq_nat y z) →
  IsTrue (leq_nat x z)`** — induction on all three; the `x = Zero` base is
  **live** and closes by `tt` (`leq_nat Zero z ⇝ True → IsTrue True → Top`, no
  false
  hypothesis); the `Suc/Suc/Suc → leq_nat m m' m''` arm lifts the IH; every
  remaining `Zero`/`Suc` mismatch arm discharges from a `False` hypothesis by
  `absurd` (`prelude.rs`, `Bottom`).
- **`antisymLeq : (x y:Nat) → IsTrue (leq_nat x y) → IsTrue (leq_nat y x) →
  Equal Nat x y`** — the load-bearing one; induction on both, `Zero/Zero → tt`
  (goal `Equal Nat Zero Zero` — two occurrences of the **nullary** ctor `Zero`,
  K7-collapses to `Top`, exactly like `ordered_empty`/`lookup_empty_is_none`; `Refl`
  fails on the collapsed goal), `Suc/Suc → cong Suc` of the IH (non-nullary
  head, neutral components → stays `Eq`-shaped), the two mixed arms `absurd`
  from a
  `False` premise.
- **`totalLeq : (x y:Nat) → Or (IsTrue (leq_nat x y)) (IsTrue (leq_nat y x))`** —
  induction on both; `Zero` on either side gives the corresponding `Inl`/`Inr`
  with `tt`; `Suc/Suc` lifts the IH's `Or` unchanged.

`antisymLeq` is needed only for the `Distinct`-discharge boundary (`54 §4`, ADR
0010-gated, **out of scope**); `delete`/`union` invariant-preservation uses only
`transLeq`/`total`, matching the landed law-1 dictionary.

Two trivial net-new `Bool` combinators ride alongside `leq_nat` as **D0
prerequisites** (needed from D1 onward — `drop_key`'s decision (`§3`) and the set
membership algebra (`§5`) — so they must be in scope before D1 in the build
order), transparent and match-based like the landed `bool_or`
(`lawful_classes.ken:39`): `bool_and a b := match a { True => b ; False => False
}` and `bool_not b := match b { True => False ; False => True }`.

## 3. D1 — `delete` (rebuild-via-`from_list`, Fork D)

`delete`'s equal-key case has **no analog in `insert`**: `insert` overwrites in
place (one path, structure-preserving), but `delete` must **remove** the node
and merge its two subtrees. Fork D rules the **rebuild** route (reuse the
capstone wholesale; the structural-`glue` alternative re-derives a whole new
invariant apparatus — `glue`/`deleteMin` + a cross-subtree-bound transport with
no analog in the landed corpus):

```
view drop_key (k : Type) (v : Type) (leq : k -> k -> Bool) (key : k) (xs : List (Pair k v)) : List (Pair k v) =
  match xs {
    Nil => Nil (Pair k v) ;
    Cons e xs2 =>
      match order_equiv_key k v leq key (pair_fst k v e) {
        True  => drop_key k v leq key xs2 ;                       -- drop ALL matches (filter)
        False => Cons (Pair k v) e (drop_key k v leq key xs2)
      }
  }

view delete (k : Type) (v : Type) (leq : k -> k -> Bool) (key : k) (m : Tree k v) : Tree k v =
  from_list k v leq (drop_key k v leq key (to_list k v m))
```

- **`drop_key` is FILTER (remove **all** order-equivalent entries), not
  drop-first** (Fork D build-pin). The order-equivalence **decision** is
  **Bool-valued** —
  `order_equiv_key leq a b : Bool = bool_and (leq a b) (leq b a)`
  (§2's `bool_and`) — so `drop_key`'s `match … { True => … ; False => … }` has a
  `Bool` scrutinee, exactly as `insert`/`lookup` branch on `leq key k2 : Bool`.
  (This is the *decision*; the landed **Prop**-valued `order_equiv`,
  `map.ken:1600`, is its `Ω` counterpart — used in the *laws*, never as an
  executable-`match` scrutinee.) `drop_key` is a plain `List` recursion
  (Gap-B-free, like `pair_keys`/`assoc`).
- **`delete` is NON-recursive** — a pipeline of landed structural ops
  (`to_list → drop_key → from_list`), so it carries **zero SCT obligation of its
  own** (one less thing to check than a self-recursive `glue`-`delete`).

### 3.1 `Ordered`-preservation

`delete` produces `from_list … (…)`, and `from_list` of **any** list is `Ordered`
by construction, so preservation needs one new lemma and no `delete`-specific
induction:

```
view from_list_preserves_ordered
  (k : Type) (v : Type) (leq : k -> k -> Bool)
  (transLeq : …) (total : …)
  (xs : List (Pair k v))
  : Ordered k v leq (from_list k v leq xs) = …          -- List-induction; each step = landed preserves_ordered
```

- **List-induction over `xs`** (the `from_list_acc` accumulator): base `Nil` is a
  **passthrough** to the accumulator's `Ordered` (the initial `Leaf`, i.e.
  `ordered_empty → tt`); step inserts one entry, closed by the **landed**
  `preserves_ordered` (law 1) applied to the accumulator's `Ordered`. Nothing
  new about `insert` is proved — it is reused wholesale.
- `delete_preserves_ordered` is then `from_list_preserves_ordered … (drop_key …
  (to_list
  … m))` — immediate, `drop_key`/`to_list` are irrelevant to the conclusion (any
  list input suffices).

### 3.2 The two lookup laws

- **None-law (UNCONDITIONAL):** `lookup key (delete key m) ≡ None`. Because
  `drop_key` is **filter**, no entry order-equivalent to `key` survives into the
  rebuilt tree, so `lookup` finds nothing — **no `Ordered`/`Distinct`
  hypothesis** (a drop-first `drop_key` would let a duplicate survive if the
  input weren't distinct; filter closes that off structurally). Routes through
  the `from_list`/`assoc` dual: `lookup key (from_list xs) ≡ assoc key xs`, and
  `assoc key (drop_key key ys) ≡ None` (a `List`-level lemma: filtering out
  `key` leaves nothing for `assoc` to match).
- **Other-key law:** `Not (order_equiv_key leq k key) → lookup k (delete key m) ≡
  lookup k m`. This one **threads `Ordered`+`Distinct`** through the landed
  **law 5** (`lookup_assoc_agree`, `map.ken:2212` — `lookup k m ≡ assoc k (to_list
  m)` under `Ordered`+`Distinct`) + its `from_list` dual + a `drop_key`/`assoc`
  lemma (`k ≠ key → assoc k (drop_key key ys) ≡ assoc k ys` — dropping a
  *different* key doesn't disturb `k`'s first-match). All `List`-level, reusing
  the landed `assoc`.

## 4. D2 — `union` / `intersection` / `difference` (Fork A)

**Fork A: `union` takes a combining function** (subsume-don't-proliferate — the
combining fn subsumes both biases: left `= union (\x _. x)`, right `= union (\_
y. y)`; a biased-only op forces a second op the first time anyone merges
values). Orientation `f (from-a) (from-b)`, matching `unionWith`:

```
view insert_with (k : Type) (v : Type) (leq : k -> k -> Bool) (f : v -> v -> v) (key : k) (val : v) (m : Tree k v) : Tree k v =
  …                                                    -- like `insert`, but on key-collision store `f val old`

view union (k : Type) (v : Type) (leq : k -> k -> Bool) (f : v -> v -> v) (a : Tree k v) (b : Tree k v) : Tree k v =
  fold k v (Tree k v) (\key val acc. insert_with k v leq f key val acc) b a
```

`intersection`/`difference` are the same fold-into-a-fresh-accumulator shape
with a **membership test** against the other map (no combining fn needed — they
select keys, they don't merge values):

```
view intersection … (a : Tree k v) (b : Tree k v) : Tree k v =
  fold k v (Tree k v) (\key val acc. match member k v leq key b { True => insert k v leq key val acc ; False => acc }) (empty k v) a

view difference … (a : Tree k v) (b : Tree k v) : Tree k v =
  fold k v (Tree k v) (\key val acc. match member k v leq key b { True => acc ; False => insert k v leq key val acc }) (empty k v) a
```

- **Lookup characterization (the D2 map law, Fork A):** `lookup k (union f a b)`
  is the 2×2 table — both-`None → None`; `(Some x, None) → Some x`; `(None, Some
  y) → Some y`; `(Some x, Some y) → Some (f x y)`. `intersection`: `Some` iff
  `k ∈ a ∧ k ∈ b` (value from `a`); `difference`: `Some` iff `k ∈ a ∧ k ∉ b`.
- **`Ordered`-preservation is `f`-independent** and rides **one** shared
  `foldPreservesOrdered`-shaped lemma: each of the three ops is a fold whose
  step
  either `insert`s (landed `preserves_ordered`) into an `Ordered` accumulator or
  returns it unchanged; the fold therefore preserves `Ordered` (base = the
  initial `Ordered` accumulator, `b` or `empty`). `f` touches **only values**;
  `Ordered` is about **keys**, so it never enters this proof.
- **Map `union` is NOT commutative in general** (unless `f` is) — the trees
  differ in shape *and* in a collided value's argument order. So **maps get only
  the lookup characterization + `Ordered`-preservation, never a commutativity
  law.** Commutativity/associativity/idempotence are **Set-only** (`§5`). Do not
  over-claim.

## 5. Set algebra (`Set = Map Unit`)

Set ops are the map ops at `v := Unit` (`set_union s t := union … (\_ _. MkUnit)
s t`, etc. — at `Unit` every combining policy coincides, so Fork A's choice is a
**no-op** here). The set membership algebra uses the landed `bool_or`
(`lawful_classes.ken:39`) together with the `bool_and`/`bool_not` introduced as
D0 prerequisites (`§2`).

**The set laws are stated MEMBERSHIP-EXTENSIONALLY** (enclave sub-ruling —
load-bearing soundness):

```
setUnionComm :  (k : Type) (leq : …) (s : Tree k Unit) (t : Tree k Unit) (x : k)
             -> Equal Bool (set_member k leq x (set_union k leq s t))
                           (set_member k leq x (set_union k leq t s))
```

— **never** `Equal (Set K) (set_union s t) (set_union t s)`. **Tree-`Equal` set
laws are FALSE, not merely unprovable:** `union a b` and `union b a` are
built by
`fold`+`insert` and produce **shape-different trees with the same key-set**.
Extensional (`∀x. set_member x lhs ≡ set_member x rhs`) is the only sound
formulation — and it is exactly what makes the laws **corollaries** rather than
fresh `Tree` inductions:

- A **membership-homomorphism** lemma reduces each op to Bool algebra:
  `set_member x (set_union s t) ≡ bool_or (set_member x s) (set_member x t)`
  (from the D2 lookup characterization at `v = Unit`), and likewise
  `bool_and` for `∩`, `bool_and _ (bool_not _)` for `∖`.
- Then **commutativity / associativity / idempotence / identity** of `∪`/`∩`
  follow from the same properties of `bool_or`/`bool_and` — a **finite 2×2**
  discharged via the landed `bool_dichotomy` (no induction over the tree at all).
- **`difference` membership:** `set_member x (setDiff s t) ≡ bool_and (set_member
  x s) (bool_not (set_member x t))`, the same shape.

## 6. D3 — `keys` / `values` coherence

- **`keys`** reuses the landed `pair_keys` + `to_list` (it is literally
  `set_to_list` generalized off `Unit`): `keys k v m := pair_keys k v (to_list k v
  m)`. Net-new = just the named binding.
- **`values`** needs one **net-new** projection, the exact mirror of `pair_keys`
  with `pair_snd` for `pair_fst`: `pair_vals k v xs := match xs { Nil => Nil v ;
  Cons p xs2 => Cons v (pair_snd k v p) (pair_vals k v xs2) }`, then `values k v m
  := pair_vals k v (to_list k v m)`.
- **Coherence with ordering:** `keys` are **ascending** under `Ordered m` —
  `is_sorted leq (keys m)` — off the landed **`to_list_ordered`** (which gives
  `is_sorted (pair_leq leq) (to_list m)`) + a small `pair_keys`-preserves-sortedness
  lemma (`pair_leq` compares first components, i.e. keys, so projecting keeps the
  order). `values` carry **no** ordering claim (values are unordered).
- **Coherence with `to_list`:** `keys`/`values` are the two **componentwise
  projections** of `to_list` — same length, positionally aligned (`keys` reads
  `pair_fst` where `values` reads `pair_snd` of the same entry). Stated
  structurally over the shared `to_list m` traversal (no `length`/`zip` primitive
  needed — those are CAT-3/List-level and out of this chapter).

## 7. D4 — Relations (the frontier; Fork C)

**Fork C-rep: a relation is `Map K (Set K)` (adjacency) — `Tree K (Tree K
Unit)`**, a plain instantiation of the landed `Tree` carrier at `v := Set K`, so
every landed parametric op works at that `v` with **zero new machinery**. Rides
**`Ord K` only** (outer map + inner set both keyed by `K`). *Not* `Set (Pair K
K)`: the landed `pair_leq` compares **first components only** (a partial,
non-total order) so it cannot even key a `Set (Pair K K)`, and a proper pair-set
would force a lexicographic pair-comparator + its four order laws as a whole
extra bundle; and composition over `Set (Pair K K)` is a quadratic nested scan.

```
view succ (k : Type) (leq : k -> k -> Bool) (x : k) (r : Tree k (Tree k Unit)) : Tree k Unit =
  match lookup k (Tree k Unit) leq x r { None => empty k Unit ; Some s => s }

view rel_member (k : Type) (leq : …) (x : k) (y : k) (r : Tree k (Tree k Unit)) : Prop =
  IsTrue (set_member k leq y (succ k leq x r))          -- Ω-native membership
```

**C-scope — the land-now / defer-build split, stated explicitly (no silent
truncation):**

**LAND NOW** (the cheap `Ω`-provable half — full laws + discriminators):

- **`compose`** `R∘S`: `succ x (compose R S) = ⋃ { succ y S : y ∈ succ x R }` —
  a `fold` over `succ x R` unioning the `S`-images (rides landed `fold`/`union`/
  `member`). **`converse`** `R˘`: `y ∈ succ x R ⇔ x ∈ succ y R˘`.
- The **property predicates as `Π`-into-`Ω`** (`16 §1.1`, fine — properties are
  **not** proof-relevant): `is_reflexive`/`is_symmetric`/`is_transitive`/
  `is_equivalence`, e.g. `is_transitive r := (x y z:k) → rel_member x y r →
  rel_member y z r → rel_member x z r`. **Discriminator:** a non-transitive
  relation (`a → b`, `b → c`, no `a → c`) **fails** `is_transitive` — the ≥3-key
  witness that mandates the `Nat` carrier (`§2`; `Bool` cannot exhibit it).

**DESIGN-NOW / DEFER-BUILD** (representation pinned, proof is the fast-follow):

- **Transitive closure (Fork B): `R⁺ x y := IsTrue (reachableWithin N x y)`**,
  `N := size (dom R)` (node count). This is the **`Perm` move** — push the
  proof-relevant "there exists a path" into a **decidable bounded `Bool`
  computation**, then wrap in `IsTrue`, an `Ω`-native value-equation. **Never a
  raw multi-ctor `data TC : … : Ω`** carrying paths (inadmissible
  proof-relevant, `16 §1.4`+§1.1).
- **Faithful, not an approximation:** any walk `x ⇝ y` shortens to a simple path
  of length ≤ `N−1` (cycle removal), so bounded reachability at bound ≥ `N−1`
  **equals** full transitive closure (monotone, saturates). Why bounded beats
  truncation: (i) `‖ Σpath. isPath path x y ‖` is `Ω`-sound but
  **non-computational** — no verdict-flipping conformance case can run against
  it; the bounded form **evaluates**; (ii) the downstream consumer is L14
  model-check interop, which literally computes bounded reachability — the
  decidable encoding is the right semantic fit; (iii) it rides the **landed**
  `Map`/`Set` `fold`/`union`/`member`; truncation needs a `Σ`-of-paths inductive
  we don't have.
- **Why deferred:** the **faithfulness/saturation laws** (simple-path shortening
  + `N`-round fixpoint saturation) are a genuine theorem needing `size : Tree k
  v → Nat` (**absent on `main` — a net-new fast-follow build item**) plus a
  bounded iteration. The frame grants this latitude ("transitive closure may be
  designed-and-deferred if the sound encoding is heavy" — it is). **The design
  is pinned here; the proof is the fast-follow**, not truncated away.
- **Seam:** relations feed L14 + Lane B; the adjacency rep is one design with
  those consumers. Do not over-build here.

## 8. Derivation paths and build sequencing

Everything bottoms out in landed built-ins + the Map capstone: `Nat` (prelude
`Zero`/`Suc`); `Tree`/`empty`/`insert`/`lookup`/`member`/`to_list`/`fold`/
`from_list`/`pair_keys`/`set_insert`/`set_member`/`set_to_list`/`all_keys`/`Ordered`/
`order_equiv`/`assoc` + laws `preserves_ordered`(1)/`lookup_assoc_agree`(5)/
`to_list_ordered`(4) (`map.ken`, all landed); `IsTrue`/`And`/`and_intro`/`Or`/
`Inl`/`Inr`/`Not`/`absurd`/`bool_or`/`bool_dichotomy` (`prelude.rs` +
`lawful_classes.ken`). **Net-new (this WP):** `leq_nat`+4 laws (D0);
`bool_and`/`bool_not`; `drop_key`/`delete`+`from_list_preserves_ordered`+None-/
other-key laws (D1); `insert_with`/`union`/`intersection`/`difference`+
`foldPreservesOrdered`+lookup characterizations (D2);
`set_union`/`set_intersection`/
`set_difference`+membership-homomorphism + extensional algebra laws (§5);
`pair_vals`/`keys`/`values`+coherence (D3); `succ`/`rel_member`/`compose`/
`converse`+property predicates (D4 land-half). **Deferred fast-follow:** `size`,
`reachableWithin`, the closure faithfulness/saturation laws.

**Build order** (Runtime-owned, GPT window): **D0 (`leq_nat`) → D1 (`delete`) →
D2 (`union`/…) → D3 (`keys`/`values`) → D4 (land-half)**. Architect re-certifies
**AC1** (kernel-untouched) and **AC5** (relation `Ω`-soundness) on the built
diff. Conformance: `../../conformance/stdlib/collections/` (D5 seed, authored
with CV — discriminators on the `Nat` carrier with the real `leq_nat` dictionary,
never `Int`).

## 9. Acceptance

- **AC1 — Kernel-untouched.** No `crates/ken-kernel/` change; no new `Term`/
  `Decl`; no `declare_primitive`/`declare_postulate`; **no `Axiom`** anywhere.
  The two surfaced build items (`size` for the closure bound; `leq_nat`+laws for
  the carrier) are ordinary total Ken, kernel-untouched.
- **AC2 — Reuse, not re-derive.** `delete`/`union`/`intersection`/`difference`
  build on the landed `insert`/`lookup`/`to_list`/`fold`/`from_list`+
  `preserves_ordered`+law 5; `leq_nat`+4 laws are `Axiom`-free.
- **AC3 — `Ordered`-preservation** for `delete` (via `from_list_preserves_ordered`)
  and `union`/`intersection`/`difference` (via one shared
  `foldPreservesOrdered`,
  `f`-independent).
- **AC4 — The characterizations:** D2 lookup 2×2 table; D1 None-law
  (unconditional) + other-key law (via law-5 roundtrip); D3 keys-ascending +
  keys/values projection coherence.
- **AC5 — Relation `Ω`-soundness.** Property predicates are `Π`-into-`Ω`;
  transitive closure is **bounded-reachability `IsTrue`**, **never** a raw
  `data … : Ω`; the deferred faithfulness split is stated (no silent
  truncation).
- **AC6 — Set laws are membership-EXTENSIONAL** (`∀x. set_member x lhs ≡
  set_member x rhs`), **never** `Equal (Set K)`; discharged as `bool_or`/
  `bool_and` corollaries via `bool_dichotomy`, not fresh `Tree` inductions.
- **AC7 — Carrier vacuity guard.** Discriminators run on the **`Nat`** carrier
  with the real `leq_nat` dictionary (≥ 3 distinct keys for transitivity/
  closure), **never** the `Axiom`-holed `Ord Int`/`Ord Char`.
