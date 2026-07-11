# `Map`/`Set` — a proved, pure ordered binary search tree

A proved, pure ordered binary search tree realizing `Map` and `Set`, plus
keyed-collection operations: deletion, combining insertion, union,
intersection, difference, set algebra, key/value projections, and a small
binary-relations library.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws & proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [References](#6-references)
7. [Trust & derivation](#7-trust--derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws & proofs](#4-laws--proofs) §4.7 (Layer-2 operations)
- *Researcher* → [Laws & proofs](#4-laws--proofs) §4.1–§4.6 (the capstone) →
  [Design notes](#5-design-notes)

## 1. Motivation

`Map` is a transparent inductive tree. Its operations are ordinary checked
definitions, so their behavior and proofs remain visible to the reader.

The carrier is a bare, unbalanced ordered binary search tree. Entries use
`Pair k v`, and every operation is parametric in a comparator `leq`. Laws that
need ordering evidence take reflexivity, antisymmetry, transitivity, and
totality as explicit parameters, making the assumptions visible at each use.

## 2. Definition

`Tree k v` is the raw two-parameter carrier: `Leaf`, or a `Node` of a left
subtree, a key, a value, and a right subtree. `empty` is `Leaf` (`52
§4.1`). `to_list` is the in-order traversal (`52 §4.2`): `Leaf` yields
`Nil`, and a `Node` appends its left subtree's traversal, its own entry,
then its right subtree's traversal, reusing `list_append` — over an
`Ordered` tree (below) the output keys are ascending (`52 §5.3`), though
that direction of the claim needs `insert`'s own preservation, proved in
[§4.3](#4-laws--proofs). `fold f z m` folds `f` over the entries in
ascending key order (`52 §4.3`) — the same order `to_list` fixes, so
`fold` agrees with a left fold of `f` over `to_list m` by construction,
not by coincidence of a particular recursion.

`insert key val m` (`52 §4.1`) descends by `leq`: at `Node l k2 v2 r`, if
`leq key k2` and `leq k2 key` both hold the keys coincide and the node's
value is overwritten; otherwise the search recurses left (`leq key k2`,
key below `k2`) or right (key above `k2`); at `Leaf` a new singleton
`Node` is placed. `lookup key m` (`52 §4.1`) retraces `insert`'s exact
same `leq` decisions, returning `Some v` at the coinciding node and `None`
at `Leaf`. `member key m` is `option_is_some (lookup key m)`.

`from_list` folds `insert` over a list (`52 §4.2`): `from_list = foldr (\
(k,v) m. insert k v m) empty`, threaded via an accumulator (`from_list_acc`)
so earlier list entries are inserted first and later entries insert last —
and so overwrite on a duplicate key, matching `52 §4.2`'s last-writer-wins
semantics (`from_list [(2,b),(1,a),(2,c)]` → `to_list = [(1,a),(2,c)]`, `c`
being the last list entry, wins). A naive `insert head (from_list tail)`
would insert the head last instead (the wrong direction); `from_list_acc`
avoids that by inserting `p` into `acc` before recursing on the tail, so the
tail's own inserts land strictly after `p`'s in evaluation order.

`Set a := Map a Unit` (`52 §4.4`): values carry no information, so
`set_insert`/`set_member` are just the map's own `insert`/`member` at `v :=
Unit`, and `set_to_list` projects the keys out of `to_list`'s `Pair a Unit`
entries via `pair_keys`.

`Ordered` is the BST ordering invariant, naturally `Ω`-valued: built from
the `IsTrue b := Equal Bool b True` bridge and the derived conjunction
`And`. `all_keys p
m` says "every key in `m` satisfies `p`"; `Ordered leq m` says every key in
a `Node`'s left subtree is below its own key, every key in its right
subtree is above it, and both subtrees are themselves `Ordered`,
recursively.

Two base laws close the Definition: `ordered_empty` (`Ordered empty`)
unfolds to `Equal Bool True True`, which collapses to `Top`
and closes with `tt` — the same non-inductive shape as
`lookup_empty_is_none` (`lookup key empty = None`, `52 §5.2` law 1), also
immediate since `empty = Leaf`. Neither needs induction or a comparison.

```ken
data Tree k v = Leaf | Node (Tree k v) k v (Tree k v)

const empty (k : Type) (v : Type) : Tree k v = Leaf k v

fn to_list (k : Type) (v : Type) (m : Tree k v) : List (Pair k v) =
  match m {
    Leaf ⇒ Nil (Pair k v) ;
    Node l key val r ⇒
      list_append (Pair k v) (to_list k v l) (Cons (Pair k v) (mk_pair k v key val) (to_list k v r))
  }

fn fold (k : Type) (v : Type) (b : Type) (f : k → v → b → b) (z : b) (m : Tree k v) : b =
  match m {
    Leaf ⇒ z ;
    Node l key val r ⇒ fold k v b f (f key val (fold k v b f z l)) r
  }

fn insert (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v) (m : Tree k v) : Tree k v =
  match m {
    Leaf ⇒ Node k v (Leaf k v) key val (Leaf k v) ;
    Node l k2 v2 r ⇒
      match leq key k2 {
        True ⇒
          match leq k2 key {
            True  ⇒ Node k v l key val r ;
            False ⇒ Node k v (insert k v leq key val l) k2 v2 r
          } ;
        False ⇒ Node k v l k2 v2 (insert k v leq key val r)
      }
  }

fn lookup (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (m : Tree k v) : Option v =
  match m {
    Leaf ⇒ None v ;
    Node l k2 v2 r ⇒
      match leq key k2 {
        True ⇒
          match leq k2 key {
            True  ⇒ Some v v2 ;
            False ⇒ lookup k v leq key l
          } ;
        False ⇒ lookup k v leq key r
      }
  }

fn member (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (m : Tree k v) : Bool =
  match lookup k v leq key m {
    None    ⇒ False ;
    Some x  ⇒ True
  }

fn option_is_some (v : Type) (o : Option v) : Bool =
  match o { None ⇒ False ; Some x ⇒ True }

fn from_list_acc (k : Type) (v : Type) (leq : k → k → Bool) (xs : List (Pair k v)) (acc : Tree k v) : Tree k v =
  match xs {
    Nil ⇒ acc ;
    Cons p xs2 ⇒ from_list_acc k v leq xs2 (insert k v leq (pair_fst k v p) (pair_snd k v p) acc)
  }

fn from_list (k : Type) (v : Type) (leq : k → k → Bool) (xs : List (Pair k v)) : Tree k v =
  from_list_acc k v leq xs (Leaf k v)

fn pair_keys (k : Type) (v : Type) (xs : List (Pair k v)) : List k =
  match xs {
    Nil ⇒ Nil k ;
    Cons p xs2 ⇒ Cons k (pair_fst k v p) (pair_keys k v xs2)
  }

fn set_insert (a : Type) (leq : a → a → Bool) (x : a) (s : Tree a Unit) : Tree a Unit =
  insert a Unit leq x (MkUnit) s

fn set_member (a : Type) (leq : a → a → Bool) (x : a) (s : Tree a Unit) : Bool =
  member a Unit leq x s

fn set_to_list (a : Type) (s : Tree a Unit) : List a =
  pair_keys a Unit (to_list a Unit s)

fn all_keys (k : Type) (v : Type) (p : k → Prop) (m : Tree k v) : Prop =
  match m {
    Leaf ⇒ Equal Bool True True ;
    Node l key val r ⇒ And (p key) (And (all_keys k v p l) (all_keys k v p r))
  }

fn Ordered (k : Type) (v : Type) (leq : k → k → Bool) (m : Tree k v) : Prop =
  match m {
    Leaf ⇒ Equal Bool True True ;
    Node l key val r ⇒
      And (all_keys k v (λk2. Equal Bool (leq k2 key) True) l)
      (And (all_keys k v (λk2. Equal Bool (leq key k2) True) r)
      (And (Ordered k v leq l) (Ordered k v leq r)))
  }

fn ordered_empty (k : Type) (v : Type) (leq : k → k → Bool) :
  Ordered k v leq (empty k v) = tt

fn lookup_empty_is_none (k : Type) (v : Type) (leq : k → k → Bool) (key : k) :
  Equal (Option v) (lookup k v leq key (empty k v)) (None v) = tt
```

## 3. Using it

The package builds up in three layers, each riding the one before. The
[Definition](#2-definition) layer above gives the raw carrier and its four
basic operations (`insert`/`lookup`/`member`/`from_list`) plus the `Ordered`
invariant a caller can carry alongside a tree to certify it is well-formed.
The [Laws & proofs](#4-laws--proofs) capstone (§4.1–§4.6) proves the five
laws `52-map-verified-laws.md` (`54`) states for that invariant and for
`insert`/`lookup`: `Ordered` is preserved by `insert`; a just-inserted key is
found by `lookup`; `insert` at a distinct key does not disturb `lookup` at
any other key; an `Ordered` tree's `to_list` is sorted by the same
comparator; and, given no two entries share an order-equivalent key,
`lookup` agrees with the ordered-list lookup `assoc` over `to_list`. A
caller who threads `leq` together with its transitivity/totality/reflexivity
witnesses gets all five for free at any concrete instantiation — no
per-application re-proof.

Layer 2 (§4.7) is where most callers actually meet this package day to day:
`delete`, a combining `insert_with`, the three lookup-table combinators
`union`/`intersection`/`difference`, their `Set`-level wrappers with the
expected membership and algebraic laws (commutativity, associativity,
idempotence, identity for union/intersection), `keys`/`values` projections,
and a small binary-relations library (`succ`/`compose`/`converse`/
`is_equivalence`, …) built on `Tree k (Tree k Unit)` as an adjacency-map
representation. Every Layer-2 operation is proved against the same
`Ordered`/`lookup` contract the capstone establishes, so a caller reasoning
about a `union` or a `delete` gets to reuse the capstone's vocabulary
directly rather than re-deriving it.

## 4. Laws & proofs

The five capstone laws establish ordering preservation and lookup behavior.
They use equality transport and structural proof techniques while retaining a
zero `trusted_base()` delta: every proof is checked, and no `Axiom` appears.

### 4.1 Capstone preliminaries

`Or`, `Inl`, and `Inr` provide the disjoint alternatives used by the
comparison proofs. They are ordinary kernel-checked inductive data with no
additional trust category.

`bool_dichotomy b` reflects a `Bool` expression into its two
equation-carrying cases. The comparison proofs use it to expose the exact
branch selected by a stuck comparator.

`assoc leq key xs` (law 5) is the ordered-list lookup: a plain structural
`List` recursion (Gap-B-free, no dependent motive), scanning entries by
`leq key (pair_fst entry) ∧ leq (pair_fst entry) key` — the same
coincidence test `lookup`/`insert` use.

`all_in_list p xs` (laws 4/5) is the list analogue of `all_keys`: "every
entry's key satisfies `p`", mirrored structurally over `List (Pair k v)`.

```ken
fn bool_dichotomy (b : Bool) : Or (Equal Bool b True) (Equal Bool b False) =
  match b {
    True  ⇒ Inl (Equal Bool True True) (Equal Bool True False) tt ;
    False ⇒ Inr (Equal Bool False True) (Equal Bool False False) tt
  }

fn assoc (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (xs : List (Pair k v)) : Option v =
  match xs {
    Nil ⇒ None v ;
    Cons p xs2 ⇒
      match leq key (pair_fst k v p) {
        True ⇒
          match leq (pair_fst k v p) key {
            True  ⇒ Some v (pair_snd k v p) ;
            False ⇒ assoc k v leq key xs2
          } ;
        False ⇒ assoc k v leq key xs2
      }
  }

fn all_in_list (k : Type) (v : Type) (p : k → Prop) (xs : List (Pair k v)) : Prop =
  match xs {
    Nil ⇒ Equal Bool True True ;
    Cons e xs2 ⇒ And (p (pair_fst k v e)) (all_in_list k v p xs2)
  }
```

### 4.2 Law 4 — `to_list_ordered`: an ordered tree's traversal is sorted

`pair_leq` lifts a key comparator to a `Pair k v` comparator by comparing
first components — `is_sorted` is instantiated at `Pair k v`.
Law 4 (`54 §3`, "to_list ordered") states `Ordered leq m → is_sorted
(to_list m)`, built via the convoy idiom (`54 §2.1`, restated fully in
[Design notes](#5-design-notes)): every match's own return type stays
scrutinee-independent; any hypothesis whose type mentions the match's
scrutinee is curried in via `->` after the return type and bound per-arm
with `\h.`, letting the kernel narrow it automatically instead of requiring
a dependent motive.

`le_above`/`le_below` are the two one-sided order predicates the rest of
the chain is built from. `all_keys_to_all_in_list` bridges a `Tree`-shaped
`all_keys` witness to the `List`-shaped `all_in_list` goal over
`to_list`, via the helper `all_in_list_append_intro` (append preserves a
uniform `all_in_list` bound across both halves).

`head_bound_goal_p`/`all_in_list_to_head_bound` convert a full-list
`all_in_list` bound into a head-only bound (feeding the eventual
`cons_sorted_head` step). `is_sorted_append` is the route-around at the
center of the chain: rather than a second-level nested dependent match, it
factors every "peek at the tail's own shape" need into its own separate,
single-match, top-level helper (`cons_sorted_head`, `sorted_tail`,
`sorted_tail_head_bound`, `append_head_bound`) and assembles them by
ordinary composition.

The four named extractors `ord_below_l`/`ord_above_r`/`ord_l`/`ord_r` pull
each of `Ordered`'s four conjuncts out of a `Node` witness once, each as
its own top-level `fn`, so that source-text chain appears exactly once
instead of being duplicated at every use site — keeping the assembled
`to_list_ordered` term small enough to elaborate. `to_list_ordered` itself
is the final assembly: at `Leaf` the goal is `tt` directly; at a `Node`,
`is_sorted_append` combines the right subtree's own induction (prefixed
with its own head bound, established via `cons_sorted_head`) against the
left subtree's induction and its all-in-list bound.

```ken
fn pair_leq (k : Type) (v : Type) (leq : k → k → Bool) (p1 : Pair k v) (p2 : Pair k v) : Bool =
  leq (pair_fst k v p1) (pair_fst k v p2)


fn le_above (k : Type) (v : Type) (leq : k → k → Bool) (bound : k) (k2 : k) : Prop =
  Equal Bool (leq bound k2) True

fn le_below (k : Type) (v : Type) (leq : k → k → Bool) (bound : k) (k2 : k) : Prop =
  Equal Bool (leq k2 bound) True

fn all_in_list_append_intro
  (k : Type) (v : Type) (p : k → Prop) (xs : List (Pair k v)) (ys : List (Pair k v))
  (hys : all_in_list k v p ys)
  : all_in_list k v p xs → all_in_list k v p (list_append (Pair k v) xs ys) =
  match xs {
    Nil ⇒ λhxs. hys ;
    Cons e xs2 ⇒
      λhxs.
        and_intro (p (pair_fst k v e)) (all_in_list k v p (list_append (Pair k v) xs2 ys))
          (and_fst (p (pair_fst k v e)) (all_in_list k v p xs2) hxs)
          (all_in_list_append_intro k v p xs2 ys hys
             (and_snd (p (pair_fst k v e)) (all_in_list k v p xs2) hxs))
  }

fn all_keys_to_all_in_list
  (k : Type) (v : Type) (p : k → Prop) (m : Tree k v)
  : all_keys k v p m → all_in_list k v p (to_list k v m) =
  match m {
    Leaf ⇒ λh. h ;
    Node l key val r ⇒
      λh.
        all_in_list_append_intro k v p (to_list k v l)
          (Cons (Pair k v) (mk_pair k v key val) (to_list k v r))
          (and_intro (p key) (all_in_list k v p (to_list k v r))
            (and_fst (p key) (And (all_keys k v p l) (all_keys k v p r)) h)
            (all_keys_to_all_in_list k v p r (and_snd (all_keys k v p l) (all_keys k v p r)
               (and_snd (p key) (And (all_keys k v p l) (all_keys k v p r)) h))))
          (all_keys_to_all_in_list k v p l (and_fst (all_keys k v p l) (all_keys k v p r)
             (and_snd (p key) (And (all_keys k v p l) (all_keys k v p r)) h)))
  }

fn head_bound_goal_p (k : Type) (v : Type) (leq : k → k → Bool) (bound : Pair k v) (xs : List (Pair k v)) : Prop =
  match xs {
    Nil ⇒ Equal Bool True True ;
    Cons y r ⇒ Equal Bool (pair_leq k v leq bound y) True
  }

fn all_in_list_to_head_bound
  (k : Type) (v : Type) (leq : k → k → Bool) (bound : k) (bval : v) (xs : List (Pair k v))
  : all_in_list k v (le_above k v leq bound) xs →
    head_bound_goal_p k v leq (mk_pair k v bound bval) xs =
  match xs {
    Nil ⇒ λh. tt ;
    Cons e xs2 ⇒
      λh.
        and_fst (Equal Bool (leq bound (pair_fst k v e)) True) (all_in_list k v (le_above k v leq bound) xs2) h
  }

fn cons_sorted_head
  (k : Type) (v : Type) (leq : k → k → Bool) (m : Pair k v) (ys : List (Pair k v))
  : is_sorted (Pair k v) (pair_leq k v leq) ys →
    head_bound_goal_p k v leq m ys → is_sorted (Pair k v) (pair_leq k v leq) (Cons (Pair k v) m ys) =
  match ys {
    Nil ⇒ λhys. λhb. tt ;
    Cons y r ⇒
      λhys. λhb.
        and_intro (Equal Bool (pair_leq k v leq m y) True) (is_sorted (Pair k v) (pair_leq k v leq) (Cons (Pair k v) y r))
          hb hys
  }

fn sorted_tail
  (k : Type) (v : Type) (leq : k → k → Bool) (e : Pair k v) (xs2 : List (Pair k v))
  : is_sorted (Pair k v) (pair_leq k v leq) (Cons (Pair k v) e xs2) →
    is_sorted (Pair k v) (pair_leq k v leq) xs2 =
  match xs2 {
    Nil ⇒ λhCons. tt ;
    Cons y r ⇒
      λhCons.
        and_snd (Equal Bool (pair_leq k v leq e y) True) (is_sorted (Pair k v) (pair_leq k v leq) (Cons (Pair k v) y r)) hCons
  }

fn sorted_tail_head_bound
  (k : Type) (v : Type) (leq : k → k → Bool) (e : Pair k v) (xs2 : List (Pair k v))
  : is_sorted (Pair k v) (pair_leq k v leq) (Cons (Pair k v) e xs2) →
    head_bound_goal_p k v leq e xs2 =
  match xs2 {
    Nil ⇒ λhCons. tt ;
    Cons y r ⇒
      λhCons.
        and_fst (Equal Bool (pair_leq k v leq e y) True) (is_sorted (Pair k v) (pair_leq k v leq) (Cons (Pair k v) y r)) hCons
  }

fn append_head_bound
  (k : Type) (v : Type) (leq : k → k → Bool) (e : Pair k v) (xs2 : List (Pair k v)) (m : Pair k v) (ys : List (Pair k v))
  (eLeqM : Equal Bool (pair_leq k v leq e m) True)
  : head_bound_goal_p k v leq e xs2 →
    head_bound_goal_p k v leq e (list_append (Pair k v) xs2 (Cons (Pair k v) m ys)) =
  match xs2 {
    Nil ⇒ λxs2HeadBound. eLeqM ;
    Cons e2 xs3 ⇒ λxs2HeadBound. xs2HeadBound
  }

fn is_sorted_append
  (k : Type) (v : Type) (leq : k → k → Bool)
  (xs : List (Pair k v)) (m : Pair k v) (ys : List (Pair k v))
  (hcons : is_sorted (Pair k v) (pair_leq k v leq) (Cons (Pair k v) m ys))
  : is_sorted (Pair k v) (pair_leq k v leq) xs →
    all_in_list k v (le_below k v leq (pair_fst k v m)) xs →
    is_sorted (Pair k v) (pair_leq k v leq) (list_append (Pair k v) xs (Cons (Pair k v) m ys)) =
  match xs {
    Nil ⇒ λhxs. λhbound. hcons ;
    Cons e xs2 ⇒
      λhxs. λhbound.
        cons_sorted_head k v leq e (list_append (Pair k v) xs2 (Cons (Pair k v) m ys))
          (is_sorted_append k v leq xs2 m ys hcons
             (sorted_tail k v leq e xs2 hxs)
             (and_snd (Equal Bool (leq (pair_fst k v e) (pair_fst k v m)) True)
                     (all_in_list k v (le_below k v leq (pair_fst k v m)) xs2)
                     hbound))
          (append_head_bound k v leq e xs2 m ys
             (and_fst (Equal Bool (leq (pair_fst k v e) (pair_fst k v m)) True)
                     (all_in_list k v (le_below k v leq (pair_fst k v m)) xs2)
                     hbound)
             (sorted_tail_head_bound k v leq e xs2 hxs))
  }

fn ord_below_l
  (k : Type) (v : Type) (leq : k → k → Bool) (l : Tree k v) (key : k) (val : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l key val r))
  : all_keys k v (le_below k v leq key) l =
  and_fst (all_keys k v (le_below k v leq key) l)
         (And (all_keys k v (le_above k v leq key) r) (And (Ordered k v leq l) (Ordered k v leq r)))
    h

fn ord_above_r
  (k : Type) (v : Type) (leq : k → k → Bool) (l : Tree k v) (key : k) (val : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l key val r))
  : all_keys k v (le_above k v leq key) r =
  and_fst (all_keys k v (le_above k v leq key) r) (And (Ordered k v leq l) (Ordered k v leq r))
    (and_snd (all_keys k v (le_below k v leq key) l)
            (And (all_keys k v (le_above k v leq key) r) (And (Ordered k v leq l) (Ordered k v leq r)))
      h)

fn ord_l
  (k : Type) (v : Type) (leq : k → k → Bool) (l : Tree k v) (key : k) (val : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l key val r))
  : Ordered k v leq l =
  and_fst (Ordered k v leq l) (Ordered k v leq r)
    (and_snd (all_keys k v (le_above k v leq key) r) (And (Ordered k v leq l) (Ordered k v leq r))
      (and_snd (all_keys k v (le_below k v leq key) l)
              (And (all_keys k v (le_above k v leq key) r) (And (Ordered k v leq l) (Ordered k v leq r)))
        h))

fn ord_r
  (k : Type) (v : Type) (leq : k → k → Bool) (l : Tree k v) (key : k) (val : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l key val r))
  : Ordered k v leq r =
  and_snd (Ordered k v leq l) (Ordered k v leq r)
    (and_snd (all_keys k v (le_above k v leq key) r) (And (Ordered k v leq l) (Ordered k v leq r))
      (and_snd (all_keys k v (le_below k v leq key) l)
              (And (all_keys k v (le_above k v leq key) r) (And (Ordered k v leq l) (Ordered k v leq r)))
        h))

fn to_list_ordered
  (k : Type) (v : Type) (leq : k → k → Bool) (m : Tree k v)
  : Ordered k v leq m → is_sorted (Pair k v) (pair_leq k v leq) (to_list k v m) =
  match m {
    Leaf ⇒ λh. tt ;
    Node l key val r ⇒
      λh.
        is_sorted_append k v leq (to_list k v l) (mk_pair k v key val) (to_list k v r)
          (cons_sorted_head k v leq (mk_pair k v key val) (to_list k v r)
             (to_list_ordered k v leq r (ord_r k v leq l key val r h))
             (all_in_list_to_head_bound k v leq key val (to_list k v r)
                (all_keys_to_all_in_list k v (le_above k v leq key) r (ord_above_r k v leq l key val r h))))
          (to_list_ordered k v leq l (ord_l k v leq l key val r h))
          (all_keys_to_all_in_list k v (le_below k v leq key) l (ord_below_l k v leq l key val r h))
  }
```

### 4.3 Law 1 — `preserves_ordered`: insert preserves the `Ordered` invariant

Law 1 (`54 §5.1`, Map capstone unit 2) states `Ordered m → Ordered (insert
key val m)`. The dictionary laws are threaded as separate bare parameters
(the same unbundled convention as the rest of this file): `transLeq`
(transitivity) and `total` (totality, as a bare `Or` of the two `IsTrue`
facts — no `bool_or` primitive needed). No `antisym` — this matches `54
§5.2`'s law-1 dictionary-law list.

The mechanism, in the order it was worked out:

- `insert_step`/`insert_step_inner` mirror `insert`'s own two-level stuck
  match structurally (`(Bool) -> Tree k v`, matched on a bound parameter,
  not the real `leq` expression). `insert_case_transport_overwrite`/
  `_into_l`/`_into_r` are goal-generic "stop-one-step-short" `trans`/`cong`
  bridges from a real `insert key val (Node l k2 v2 r)` application to each
  of `insert`'s three real branches, letting the final delta+iota step land
  via ordinary conversion inside `J`'s own base-argument check rather than
  needing an explicit reflected-`Eq` witness for both stuck comparisons
  composed (the confirmed-broken nested-`J` shape — see [Design
  notes](#5-design-notes)).
- `insert_preserves_all_keys` (needed for law 1's own bound updates): a
  comparison-independent predicate `p` doesn't care which value `leq`
  returns, so its `Node`-case helper (`insert_step_inner_preserves_all_keys`/
  `insert_step_preserves_all_keys`) is proved generically over an abstract
  `Bool` parameter (ordinary Gap-B convoy on a bound variable) and simply
  applied at the real `leq key k2` expression — no stuck-boolean
  reflection, no transport, at all.
- `all_keys_trans_below`/`all_keys_trans_above` move an `all_keys` bound
  from one key to another via `transLeq`, by ordinary Gap-B structural
  induction, comparison-free.
- `derive_from_false` (totality-derived reflection, the `insert`-into-R
  branch's bound update): `insert`'s `False` branch doesn't consult `leq k2
  key` at all, but `Ordered`'s bound update for that branch needs it as a
  witness — derived from `leq key k2 = False` plus `total` via one more
  (non-dependent) `Or`-elimination.
- The top-level dispatch (`insert_case_transport_dispatch`/`dispatch_on_q1`/
  `dispatch_on_q2`) is two non-dependent `Or`-eliminations
  (`bool_dichotomy (leq key k2)`, then inside the `True` sub-case,
  `bool_dichotomy (leq k2 key)`) — the overall goal `Ordered (insert key val
  m)` is the same type regardless of which branch fired, so this does not
  hit the dependent-match-nesting restriction (only a goal that *depends*
  on which arm fired needs `check_match_dependent`'s single-bound-variable
  scrutinee gate).

The resulting proof families remain structural and total.

```ken
fn insert_step_inner
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v) (y : Bool) : Tree k v =
  match y {
    True  ⇒ Node k v l key val r ;
    False ⇒ Node k v (insert k v leq key val l) k2 v2 r
  }

fn insert_step
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v) (x : Bool) : Tree k v =
  match x {
    True  ⇒ insert_step_inner k v leq key val l k2 v2 r (leq k2 key) ;
    False ⇒ Node k v l k2 v2 (insert k v leq key val r)
  }

fn mk_inner_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q2 : Equal Bool (leq k2 key) False)
  : Equal (Tree k v) (insert_step_inner k v leq key val l k2 v2 r (leq k2 key))
                      (insert_step_inner k v leq key val l k2 v2 r False) =
  cong Bool (Tree k v) (leq k2 key) False (insert_step_inner k v leq key val l k2 v2 r) q2

fn mk_inner_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q2 : Equal Bool (leq k2 key) True)
  : Equal (Tree k v) (insert_step_inner k v leq key val l k2 v2 r (leq k2 key))
                      (insert_step_inner k v leq key val l k2 v2 r True) =
  cong Bool (Tree k v) (leq k2 key) True (insert_step_inner k v leq key val l k2 v2 r) q2

fn mk_step_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True)
  : Equal (Tree k v) (insert_step k v leq key val l k2 v2 r (leq key k2))
                      (insert_step k v leq key val l k2 v2 r True) =
  cong Bool (Tree k v) (leq key k2) True (insert_step k v leq key val l k2 v2 r) q1

fn mk_step_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) False)
  : Equal (Tree k v) (insert_step k v leq key val l k2 v2 r (leq key k2))
                      (insert_step k v leq key val l k2 v2 r False) =
  cong Bool (Tree k v) (leq key k2) False (insert_step k v leq key val l k2 v2 r) q1

fn mk_final_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  : Equal (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                      (insert_step k v leq key val l k2 v2 r (leq key k2)) = Refl

fn mk_step_true_reduces
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  : Equal (Tree k v) (insert_step k v leq key val l k2 v2 r True)
                      (insert_step_inner k v leq key val l k2 v2 r (leq k2 key)) = Refl

fn step_a_combined1
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True)
  : Equal (Tree k v) (insert_step k v leq key val l k2 v2 r (leq key k2))
                      (insert_step_inner k v leq key val l k2 v2 r (leq k2 key)) =
  trans (Tree k v) (insert_step k v leq key val l k2 v2 r (leq key k2))
                    (insert_step k v leq key val l k2 v2 r True)
                    (insert_step_inner k v leq key val l k2 v2 r (leq k2 key))
    (mk_step_true_eq k v leq key val l k2 v2 r q1) (mk_step_true_reduces k v leq key val l k2 v2 r)

fn step_b_combined2
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True)
  : Equal (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                      (insert_step_inner k v leq key val l k2 v2 r (leq k2 key)) =
  trans (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                    (insert_step k v leq key val l k2 v2 r (leq key k2))
                    (insert_step_inner k v leq key val l k2 v2 r (leq k2 key))
    (mk_final_bridge k v leq key val l k2 v2 r) (step_a_combined1 k v leq key val l k2 v2 r q1)

fn step_c_combined3
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) False)
  : Equal (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                      (insert_step_inner k v leq key val l k2 v2 r False) =
  trans (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                    (insert_step_inner k v leq key val l k2 v2 r (leq k2 key))
                    (insert_step_inner k v leq key val l k2 v2 r False)
    (step_b_combined2 k v leq key val l k2 v2 r q1) (mk_inner_false_eq k v leq key val l k2 v2 r q2)

fn step_e_combined_overwrite
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  : Equal (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                      (insert_step_inner k v leq key val l k2 v2 r True) =
  trans (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                    (insert_step_inner k v leq key val l k2 v2 r (leq k2 key))
                    (insert_step_inner k v leq key val l k2 v2 r True)
    (step_b_combined2 k v leq key val l k2 v2 r q1) (mk_inner_true_eq k v leq key val l k2 v2 r q2)

fn insert_case_transport_overwrite
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (goal : Tree k v → Prop)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  (overwrite : goal (Node k v l key val r))
  : goal (insert k v leq key val (Node k v l k2 v2 r)) =
  J (λx _. goal x) overwrite
    (sym (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                     (insert_step_inner k v leq key val l k2 v2 r True)
       (step_e_combined_overwrite k v leq key val l k2 v2 r q1 q2))

fn insert_case_transport_into_l
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (goal : Tree k v → Prop)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) False)
  (intoL : goal (Node k v (insert k v leq key val l) k2 v2 r))
  : goal (insert k v leq key val (Node k v l k2 v2 r)) =
  J (λx _. goal x) intoL
    (sym (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                     (insert_step_inner k v leq key val l k2 v2 r False)
       (step_c_combined3 k v leq key val l k2 v2 r q1 q2))

fn step_d_combined2
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) False)
  : Equal (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                      (insert_step k v leq key val l k2 v2 r False) =
  trans (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                    (insert_step k v leq key val l k2 v2 r (leq key k2))
                    (insert_step k v leq key val l k2 v2 r False)
    (mk_final_bridge k v leq key val l k2 v2 r) (mk_step_false_eq k v leq key val l k2 v2 r q1)

fn insert_case_transport_into_r
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (goal : Tree k v → Prop)
  (q1 : Equal Bool (leq key k2) False)
  (intoR : goal (Node k v l k2 v2 (insert k v leq key val r)))
  : goal (insert k v leq key val (Node k v l k2 v2 r)) =
  J (λx _. goal x) intoR
    (sym (Tree k v) (insert k v leq key val (Node k v l k2 v2 r))
                     (insert_step k v leq key val l k2 v2 r False)
       (step_d_combined2 k v leq key val l k2 v2 r q1))

fn all_keys_trans_below
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (a : k) (b : k) (m : Tree k v)
  (hab : Equal Bool (leq a b) True)
  : all_keys k v (le_below k v leq a) m → all_keys k v (le_below k v leq b) m =
  match m {
    Leaf ⇒ λh. tt ;
    Node l key val r ⇒
      λh.
        and_intro (Equal Bool (leq key b) True) (And (all_keys k v (le_below k v leq b) l) (all_keys k v (le_below k v leq b) r))
          (transLeq key a b (and_fst (Equal Bool (leq key a) True) (And (all_keys k v (le_below k v leq a) l) (all_keys k v (le_below k v leq a) r)) h) hab)
          (and_intro (all_keys k v (le_below k v leq b) l) (all_keys k v (le_below k v leq b) r)
             (all_keys_trans_below k v leq transLeq a b l hab
                (and_fst (all_keys k v (le_below k v leq a) l) (all_keys k v (le_below k v leq a) r)
                        (and_snd (Equal Bool (leq key a) True) (And (all_keys k v (le_below k v leq a) l) (all_keys k v (le_below k v leq a) r)) h)))
             (all_keys_trans_below k v leq transLeq a b r hab
                (and_snd (all_keys k v (le_below k v leq a) l) (all_keys k v (le_below k v leq a) r)
                        (and_snd (Equal Bool (leq key a) True) (And (all_keys k v (le_below k v leq a) l) (all_keys k v (le_below k v leq a) r)) h))))
  }

fn all_keys_trans_above
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (a : k) (b : k) (m : Tree k v)
  (hba : Equal Bool (leq b a) True)
  : all_keys k v (le_above k v leq a) m → all_keys k v (le_above k v leq b) m =
  match m {
    Leaf ⇒ λh. tt ;
    Node l key val r ⇒
      λh.
        and_intro (Equal Bool (leq b key) True) (And (all_keys k v (le_above k v leq b) l) (all_keys k v (le_above k v leq b) r))
          (transLeq b a key hba (and_fst (Equal Bool (leq a key) True) (And (all_keys k v (le_above k v leq a) l) (all_keys k v (le_above k v leq a) r)) h))
          (and_intro (all_keys k v (le_above k v leq b) l) (all_keys k v (le_above k v leq b) r)
             (all_keys_trans_above k v leq transLeq a b l hba
                (and_fst (all_keys k v (le_above k v leq a) l) (all_keys k v (le_above k v leq a) r)
                        (and_snd (Equal Bool (leq a key) True) (And (all_keys k v (le_above k v leq a) l) (all_keys k v (le_above k v leq a) r)) h)))
             (all_keys_trans_above k v leq transLeq a b r hba
                (and_snd (all_keys k v (le_above k v leq a) l) (all_keys k v (le_above k v leq a) r)
                        (and_snd (Equal Bool (leq a key) True) (And (all_keys k v (le_above k v leq a) l) (all_keys k v (le_above k v leq a) r)) h))))
  }

fn insert_step_inner_preserves_all_keys
  (k : Type) (v : Type) (leq : k → k → Bool) (p : k → Prop)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (insL : all_keys k v p (insert k v leq key val l))
  (hl : all_keys k v p l) (hr : all_keys k v p r) (hkey : p key) (hk2 : p k2)
  (y : Bool)
  : all_keys k v p (insert_step_inner k v leq key val l k2 v2 r y) =
  match y {
    True  ⇒ and_intro (p key) (And (all_keys k v p l) (all_keys k v p r)) hkey (and_intro (all_keys k v p l) (all_keys k v p r) hl hr) ;
    False ⇒ and_intro (p k2) (And (all_keys k v p (insert k v leq key val l)) (all_keys k v p r)) hk2 (and_intro (all_keys k v p (insert k v leq key val l)) (all_keys k v p r) insL hr)
  }

fn insert_step_preserves_all_keys
  (k : Type) (v : Type) (leq : k → k → Bool) (p : k → Prop)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (insL : all_keys k v p (insert k v leq key val l))
  (insR : all_keys k v p (insert k v leq key val r))
  (hl : all_keys k v p l) (hr : all_keys k v p r) (hkey : p key) (hk2 : p k2)
  (x : Bool)
  : all_keys k v p (insert_step k v leq key val l k2 v2 r x) =
  match x {
    True  ⇒ insert_step_inner_preserves_all_keys k v leq p key val l k2 v2 r insL hl hr hkey hk2 (leq k2 key) ;
    False ⇒ and_intro (p k2) (And (all_keys k v p l) (all_keys k v p (insert k v leq key val r))) hk2 (and_intro (all_keys k v p l) (all_keys k v p (insert k v leq key val r)) hl insR)
  }

fn insert_preserves_all_keys_node
  (k : Type) (v : Type) (leq : k → k → Bool) (p : k → Prop)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (insL : all_keys k v p (insert k v leq key val l))
  (insR : all_keys k v p (insert k v leq key val r))
  (hl : all_keys k v p l) (hr : all_keys k v p r) (hkey : p key) (hk2 : p k2)
  : all_keys k v p (insert k v leq key val (Node k v l k2 v2 r)) =
  insert_step_preserves_all_keys k v leq p key val l k2 v2 r insL insR hl hr hkey hk2 (leq key k2)

fn insert_preserves_all_keys
  (k : Type) (v : Type) (leq : k → k → Bool) (p : k → Prop)
  (key : k) (val : v) (m : Tree k v)
  : all_keys k v p m → p key → all_keys k v p (insert k v leq key val m) =
  match m {
    Leaf ⇒ λh. λhkey.
      and_intro (p key) (And (all_keys k v p (Leaf k v)) (all_keys k v p (Leaf k v))) hkey
        (and_intro (all_keys k v p (Leaf k v)) (all_keys k v p (Leaf k v)) tt tt) ;
    Node l k2 v2 r ⇒ λh. λhkey.
      insert_preserves_all_keys_node k v leq p key val l k2 v2 r
        (insert_preserves_all_keys k v leq p key val l
           (and_fst (all_keys k v p l) (all_keys k v p r) (and_snd (p k2) (And (all_keys k v p l) (all_keys k v p r)) h)) hkey)
        (insert_preserves_all_keys k v leq p key val r
           (and_snd (all_keys k v p l) (all_keys k v p r) (and_snd (p k2) (And (all_keys k v p l) (all_keys k v p r)) h)) hkey)
        (and_fst (all_keys k v p l) (all_keys k v p r) (and_snd (p k2) (And (all_keys k v p l) (all_keys k v p r)) h))
        (and_snd (all_keys k v p l) (all_keys k v p r) (and_snd (p k2) (And (all_keys k v p l) (all_keys k v p r)) h))
        hkey
        (and_fst (p k2) (And (all_keys k v p l) (all_keys k v p r)) h)
  }

fn derive_from_false_core
  (k : Type) (leq : k → k → Bool) (a : k) (b : k)
  (qFalse : Equal Bool (leq a b) False)
  (o : Or (Equal Bool (leq a b) True) (Equal Bool (leq b a) True))
  : Equal Bool (leq b a) True =
  match o {
    Inl hA ⇒ absurd (trans Bool False (leq a b) True (sym Bool (leq a b) False qFalse) hA) ;
    Inr hB ⇒ hB
  }

fn derive_from_false
  (k : Type) (leq : k → k → Bool) (a : k) (b : k)
  (qFalse : Equal Bool (leq a b) False)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  : Equal Bool (leq b a) True =
  derive_from_false_core k leq a b qFalse (total a b)

fn ow_witness_below
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (q2 : Equal Bool (leq k2 key) True)
  : all_keys k v (le_below k v leq key) l =
  all_keys_trans_below k v leq transLeq k2 key l q2
    (and_fst (all_keys k v (le_below k v leq k2) l) (And (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))) h)

fn ow_witness_above
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (q1 : Equal Bool (leq key k2) True)
  : all_keys k v (le_above k v leq key) r =
  all_keys_trans_above k v leq transLeq k2 key r q1
    (and_fst (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))
       (and_snd (all_keys k v (le_below k v leq k2) l) (And (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))) h))

fn ow_witness_ordered_l
  (k : Type) (v : Type) (leq : k → k → Bool)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  : Ordered k v leq l =
  and_fst (Ordered k v leq l) (Ordered k v leq r)
     (and_snd (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))
        (and_snd (all_keys k v (le_below k v leq k2) l) (And (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))) h))

fn ow_witness_ordered_r
  (k : Type) (v : Type) (leq : k → k → Bool)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  : Ordered k v leq r =
  and_snd (Ordered k v leq l) (Ordered k v leq r)
     (and_snd (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))
        (and_snd (all_keys k v (le_below k v leq k2) l) (And (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))) h))

fn preserves_ordered_overwrite_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  : Ordered k v leq (Node k v l key val r) =
  and_intro (all_keys k v (le_below k v leq key) l) (And (all_keys k v (le_above k v leq key) r) (And (Ordered k v leq l) (Ordered k v leq r)))
    (ow_witness_below k v leq transLeq key val l k2 v2 r h q2)
    (and_intro (all_keys k v (le_above k v leq key) r) (And (Ordered k v leq l) (Ordered k v leq r))
       (ow_witness_above k v leq transLeq key val l k2 v2 r h q1)
       (and_intro (Ordered k v leq l) (Ordered k v leq r)
          (ow_witness_ordered_l k v leq l k2 v2 r h)
          (ow_witness_ordered_r k v leq l k2 v2 r h)))

fn get_ordered_l
  (k : Type) (v : Type) (leq : k → k → Bool)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  : Ordered k v leq l =
  and_fst (Ordered k v leq l) (Ordered k v leq r)
    (and_snd (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))
       (and_snd (all_keys k v (le_below k v leq k2) l) (And (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))) h))

fn get_ordered_r
  (k : Type) (v : Type) (leq : k → k → Bool)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  : Ordered k v leq r =
  and_snd (Ordered k v leq l) (Ordered k v leq r)
    (and_snd (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))
       (and_snd (all_keys k v (le_below k v leq k2) l) (And (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))) h))

fn get_below_l
  (k : Type) (v : Type) (leq : k → k → Bool)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  : all_keys k v (le_below k v leq k2) l =
  and_fst (all_keys k v (le_below k v leq k2) l) (And (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))) h

fn get_above_r
  (k : Type) (v : Type) (leq : k → k → Bool)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  : all_keys k v (le_above k v leq k2) r =
  and_fst (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))
    (and_snd (all_keys k v (le_below k v leq k2) l) (And (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq l) (Ordered k v leq r))) h)

fn preserves_ordered_into_l_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (insL : Ordered k v leq (insert k v leq key val l))
  (insLBelow : all_keys k v (le_below k v leq k2) (insert k v leq key val l))
  : Ordered k v leq (Node k v (insert k v leq key val l) k2 v2 r) =
  and_intro (all_keys k v (le_below k v leq k2) (insert k v leq key val l))
           (And (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq (insert k v leq key val l)) (Ordered k v leq r)))
    insLBelow
    (and_intro (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq (insert k v leq key val l)) (Ordered k v leq r))
       (get_above_r k v leq l k2 v2 r h)
       (and_intro (Ordered k v leq (insert k v leq key val l)) (Ordered k v leq r)
          insL
          (get_ordered_r k v leq l k2 v2 r h)))

fn preserves_ordered_into_r_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (insR : Ordered k v leq (insert k v leq key val r))
  (insRAbove : all_keys k v (le_above k v leq k2) (insert k v leq key val r))
  : Ordered k v leq (Node k v l k2 v2 (insert k v leq key val r)) =
  and_intro (all_keys k v (le_below k v leq k2) l)
           (And (all_keys k v (le_above k v leq k2) (insert k v leq key val r)) (And (Ordered k v leq l) (Ordered k v leq (insert k v leq key val r))))
    (get_below_l k v leq l k2 v2 r h)
    (and_intro (all_keys k v (le_above k v leq k2) (insert k v leq key val r)) (And (Ordered k v leq l) (Ordered k v leq (insert k v leq key val r)))
       insRAbove
       (and_intro (Ordered k v leq l) (Ordered k v leq (insert k v leq key val r))
          (get_ordered_l k v leq l k2 v2 r h)
          insR))

fn dispatch_on_q2
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (insL : Ordered k v leq (insert k v leq key val l))
  (insR : Ordered k v leq (insert k v leq key val r))
  (q1 : Equal Bool (leq key k2) True)
  (o2 : Or (Equal Bool (leq k2 key) True) (Equal Bool (leq k2 key) False))
  : Ordered k v leq (insert k v leq key val (Node k v l k2 v2 r)) =
  match o2 {
    Inl q2 ⇒
      insert_case_transport_overwrite k v leq key val l k2 v2 r (λx. Ordered k v leq x) q1 q2
        (preserves_ordered_overwrite_witness k v leq transLeq key val l k2 v2 r h q1 q2) ;
    Inr q2' ⇒
      insert_case_transport_into_l k v leq key val l k2 v2 r (λx. Ordered k v leq x) q1 q2'
        (preserves_ordered_into_l_witness k v leq key val l k2 v2 r h insL
           (insert_preserves_all_keys k v leq (le_below k v leq k2) key val l (get_below_l k v leq l k2 v2 r h) q1))
  }

fn dispatch_on_q1
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (insL : Ordered k v leq (insert k v leq key val l))
  (insR : Ordered k v leq (insert k v leq key val r))
  (o1 : Or (Equal Bool (leq key k2) True) (Equal Bool (leq key k2) False))
  : Ordered k v leq (insert k v leq key val (Node k v l k2 v2 r)) =
  match o1 {
    Inl q1 ⇒ dispatch_on_q2 k v leq transLeq total key val l k2 v2 r h insL insR q1 (bool_dichotomy (leq k2 key)) ;
    Inr q1' ⇒
      insert_case_transport_into_r k v leq key val l k2 v2 r (λx. Ordered k v leq x) q1'
        (preserves_ordered_into_r_witness k v leq key val l k2 v2 r h insR
           (insert_preserves_all_keys k v leq (le_above k v leq k2) key val r (get_above_r k v leq l k2 v2 r h)
              (derive_from_false k leq key k2 q1' total)))
  }

fn insert_case_transport_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (insL : Ordered k v leq (insert k v leq key val l))
  (insR : Ordered k v leq (insert k v leq key val r))
  : Ordered k v leq (insert k v leq key val (Node k v l k2 v2 r)) =
  dispatch_on_q1 k v leq transLeq total key val l k2 v2 r h insL insR (bool_dichotomy (leq key k2))

fn preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (key : k) (val : v) (m : Tree k v)
  : Ordered k v leq m → Ordered k v leq (insert k v leq key val m) =
  match m {
    Leaf ⇒ λh.
      and_intro (all_keys k v (le_below k v leq key) (Leaf k v))
               (And (all_keys k v (le_above k v leq key) (Leaf k v)) (And (Ordered k v leq (Leaf k v)) (Ordered k v leq (Leaf k v))))
        tt (and_intro (all_keys k v (le_above k v leq key) (Leaf k v)) (And (Ordered k v leq (Leaf k v)) (Ordered k v leq (Leaf k v)))
             tt (and_intro (Ordered k v leq (Leaf k v)) (Ordered k v leq (Leaf k v)) tt tt)) ;
    Node l k2 v2 r ⇒
      λh.
        insert_case_transport_dispatch k v leq transLeq total key val l k2 v2 r h
          (preserves_ordered k v leq transLeq total key val l (get_ordered_l k v leq l k2 v2 r h))
          (preserves_ordered k v leq transLeq total key val r (get_ordered_r k v leq l k2 v2 r h))
  }
```

### 4.4 Law 2 — `lookup_found_after_insert`: a just-inserted key is found

Law 2 (`54 §5.2`, Map capstone unit 2) states `lookup key (insert key val
m) = Some val`. The dictionary law needed is `reflLeq : (x:k) -> leq x x =
True` only (matching `54 §5.2`'s law-2 list) — no `Ordered` hypothesis is
needed.

This law reuses Law 1's goal-generic transport bridges
(`insert_case_transport_overwrite`/`_into_l`/`_into_r`) directly, with
`goal := \x. Equal (Option v) (lookup key x) (Some val)` — the per-branch
case-split (`bool_dichotomy (leq key k2)` then `bool_dichotomy (leq k2
key)`) needed no re-derivation at all.

The machinery specific to this law: `lookup`'s own two-level stuck match
mirrors `insert`'s (`lookup_step`/`lookup_step_inner`), with three bridges.
`lookup_overwrite_result`: both of `lookup`'s stuck comparisons at an
overwritten node are the identical expression `leq key key`, so a single
`reflLeq key` witness feeds both steps of the composition (no case-split
needed, unlike Law 1's overwrite witness, which needed two different
reflected facts). `lookup_into_l_bridge`/`lookup_into_r_bridge`: `lookup`'s
traversal of the original (pre-insert) node, given the same `q1`/`q2` that
routed `insert`'s own branch, reduces to `lookup key l` / `lookup key r`
directly (`lookup` and `insert` branch on the identical comparisons) —
composed with the outer induction's IH via `trans`.

Confirmed on the fixed kernel: builds clean, well under a second.

```ken
fn lookup_step_inner
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v) (y : Bool) : Option v =
  match y {
    True  ⇒ Some v v2 ;
    False ⇒ lookup k v leq key l
  }

fn lookup_step
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v) (x : Bool) : Option v =
  match x {
    True  ⇒ lookup_step_inner k v leq key l k2 v2 r (leq k2 key) ;
    False ⇒ lookup k v leq key r
  }

fn lookup_final_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                      (lookup_step k v leq key l k2 v2 r (leq key k2)) = Refl

fn lookup_step_true_reduces
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  : Equal (Option v) (lookup_step k v leq key l k2 v2 r True)
                      (lookup_step_inner k v leq key l k2 v2 r (leq k2 key)) = Refl

fn lookup_mk_step_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True)
  : Equal (Option v) (lookup_step k v leq key l k2 v2 r (leq key k2))
                      (lookup_step k v leq key l k2 v2 r True) =
  cong Bool (Option v) (leq key k2) True (lookup_step k v leq key l k2 v2 r) q1

fn lookup_mk_step_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) False)
  : Equal (Option v) (lookup_step k v leq key l k2 v2 r (leq key k2))
                      (lookup_step k v leq key l k2 v2 r False) =
  cong Bool (Option v) (leq key k2) False (lookup_step k v leq key l k2 v2 r) q1

fn lookup_mk_inner_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q2 : Equal Bool (leq k2 key) False)
  : Equal (Option v) (lookup_step_inner k v leq key l k2 v2 r (leq k2 key))
                      (lookup_step_inner k v leq key l k2 v2 r False) =
  cong Bool (Option v) (leq k2 key) False (lookup_step_inner k v leq key l k2 v2 r) q2

fn lookup_mk_inner_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q2 : Equal Bool (leq k2 key) True)
  : Equal (Option v) (lookup_step_inner k v leq key l k2 v2 r (leq k2 key))
                      (lookup_step_inner k v leq key l k2 v2 r True) =
  cong Bool (Option v) (leq k2 key) True (lookup_step_inner k v leq key l k2 v2 r) q2

fn lookup_into_l_bridge_step1
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True)
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                      (lookup_step_inner k v leq key l k2 v2 r (leq k2 key)) =
  trans (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                    (lookup_step k v leq key l k2 v2 r (leq key k2))
                    (lookup_step_inner k v leq key l k2 v2 r (leq k2 key))
    (lookup_final_bridge k v leq key l k2 v2 r)
    (trans (Option v) (lookup_step k v leq key l k2 v2 r (leq key k2))
                       (lookup_step k v leq key l k2 v2 r True)
                       (lookup_step_inner k v leq key l k2 v2 r (leq k2 key))
       (lookup_mk_step_true_eq k v leq key l k2 v2 r q1) (lookup_step_true_reduces k v leq key l k2 v2 r))

fn lookup_into_l_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) False)
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                      (lookup k v leq key l) =
  trans (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                    (lookup_step_inner k v leq key l k2 v2 r (leq k2 key))
                    (lookup_step_inner k v leq key l k2 v2 r False)
    (lookup_into_l_bridge_step1 k v leq key l k2 v2 r q1)
    (lookup_mk_inner_false_eq k v leq key l k2 v2 r q2)

fn lookup_into_r_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) False)
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                      (lookup k v leq key r) =
  trans (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                    (lookup_step k v leq key l k2 v2 r (leq key k2))
                    (lookup_step k v leq key l k2 v2 r False)
    (lookup_final_bridge k v leq key l k2 v2 r)
    (lookup_mk_step_false_eq k v leq key l k2 v2 r q1)

fn lookup_overwrite_result
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v)
  (l : Tree k v) (r : Tree k v)
  (q : Equal Bool (leq key key) True)
  : Equal (Option v) (lookup k v leq key (Node k v l key val r)) (Some v val) =
  trans (Option v) (lookup k v leq key (Node k v l key val r))
                    (lookup_step_inner k v leq key l key val r (leq key key))
                    (Some v val)
    (trans (Option v) (lookup k v leq key (Node k v l key val r))
                       (lookup_step k v leq key l key val r (leq key key))
                       (lookup_step_inner k v leq key l key val r (leq key key))
       (lookup_final_bridge k v leq key l key val r)
       (trans (Option v) (lookup_step k v leq key l key val r (leq key key))
                          (lookup_step k v leq key l key val r True)
                          (lookup_step_inner k v leq key l key val r (leq key key))
          (lookup_mk_step_true_eq k v leq key l key val r q) (lookup_step_true_reduces k v leq key l key val r)))
    (lookup_mk_inner_true_eq k v leq key l key val r q)

fn lookup_found_dispatch_q2
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (insL : Equal (Option v) (lookup k v leq key (insert k v leq key val l)) (Some v val))
  (q1 : Equal Bool (leq key k2) True)
  (o2 : Or (Equal Bool (leq k2 key) True) (Equal Bool (leq k2 key) False))
  : Equal (Option v) (lookup k v leq key (insert k v leq key val (Node k v l k2 v2 r))) (Some v val) =
  match o2 {
    Inl q2 ⇒
      insert_case_transport_overwrite k v leq key val l k2 v2 r (λx. Equal (Option v) (lookup k v leq key x) (Some v val)) q1 q2
        (lookup_overwrite_result k v leq key val l r (reflLeq key)) ;
    Inr q2' ⇒
      insert_case_transport_into_l k v leq key val l k2 v2 r (λx. Equal (Option v) (lookup k v leq key x) (Some v val)) q1 q2'
        (trans (Option v) (lookup k v leq key (Node k v (insert k v leq key val l) k2 v2 r))
                           (lookup k v leq key (insert k v leq key val l))
                           (Some v val)
          (lookup_into_l_bridge k v leq key (insert k v leq key val l) k2 v2 r q1 q2')
          insL)
  }

fn lookup_found_dispatch_q1
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (insL : Equal (Option v) (lookup k v leq key (insert k v leq key val l)) (Some v val))
  (insR : Equal (Option v) (lookup k v leq key (insert k v leq key val r)) (Some v val))
  (o1 : Or (Equal Bool (leq key k2) True) (Equal Bool (leq key k2) False))
  : Equal (Option v) (lookup k v leq key (insert k v leq key val (Node k v l k2 v2 r))) (Some v val) =
  match o1 {
    Inl q1 ⇒ lookup_found_dispatch_q2 k v leq reflLeq key val l k2 v2 r insL q1 (bool_dichotomy (leq k2 key)) ;
    Inr q1' ⇒
      insert_case_transport_into_r k v leq key val l k2 v2 r (λx. Equal (Option v) (lookup k v leq key x) (Some v val)) q1'
        (trans (Option v) (lookup k v leq key (Node k v l k2 v2 (insert k v leq key val r)))
                           (lookup k v leq key (insert k v leq key val r))
                           (Some v val)
          (lookup_into_r_bridge k v leq key l k2 v2 (insert k v leq key val r) q1')
          insR)
  }

fn lookup_found_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (insL : Equal (Option v) (lookup k v leq key (insert k v leq key val l)) (Some v val))
  (insR : Equal (Option v) (lookup k v leq key (insert k v leq key val r)) (Some v val))
  : Equal (Option v) (lookup k v leq key (insert k v leq key val (Node k v l k2 v2 r))) (Some v val) =
  lookup_found_dispatch_q1 k v leq reflLeq key val l k2 v2 r insL insR (bool_dichotomy (leq key k2))

fn lookup_found_after_insert
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (key : k) (val : v) (m : Tree k v)
  : Equal (Option v) (lookup k v leq key (insert k v leq key val m)) (Some v val) =
  match m {
    Leaf ⇒ lookup_overwrite_result k v leq key val (Leaf k v) (Leaf k v) (reflLeq key) ;
    Node l k2 v2 r ⇒
      lookup_found_dispatch k v leq reflLeq key val l k2 v2 r
        (lookup_found_after_insert k v leq reflLeq key val l)
        (lookup_found_after_insert k v leq reflLeq key val r)
  }
```

### 4.5 Law 3 — `lookup_locality`: insert at a distinct key doesn't disturb lookup elsewhere

Law 3 (`54 §5.2`, Map capstone unit 2) states `distinct key key' → lookup
key' (insert key val m) = lookup key' m`, where `distinct key key' := And
(leq key key' = True) (leq key' key = True) -> Bottom` (order-distinctness,
`52 §5.2`). The dictionary law needed is `transLeq` only (matching `54
§5.2`'s law-3 list; no `antisym`, no `total`). This law reuses Law 1's
goal-generic transport bridges directly (the same pattern as Law 2),
instantiated at `goal := \x. Equal (Option v) (lookup key' x) (lookup key'
(Node l k2 v2 r))`.

The machinery, in order of discovery: `bool_value_eq_from_biimpl` — from a
two-directional Bool-value implication (`b1=True -> b2=True` and back),
concludes `b1 = b2` as values (not just an iff) — `Bool` has exactly two
constructors, so a value mismatch with agreeing truth at `True` forces
agreement at `False` too (the disagreeing case collapses to `Equal Bool
True False`, absurd via K7). `lookup_overwrite_agrees_outer`/`_inner`: at
the overwrite branch (`key`~`k2` via `q1`,`q2`), `leq key' key` and `leq
key' k2` (resp. `leq key key'` and `leq k2 key'`) are provably the same
value via `transLeq` composed both directions with
`bool_value_eq_from_biimpl` — so `lookup key'` takes the identical branch
decision whether the node is labeled `key` or `k2`. The only case that can
actually differ (both directions agreeing `True`, i.e. `key'`~`key`) is
exactly what `distinct` forbids — closed via `absurd`.
`lookup_into_l_locality_witness`/`_into_r`: the node label is unchanged for
these two branches (only one subtree is replaced by its own recursive
insert), so `lookup key'` takes the identical branch decision in both pre-
and post-insert forms; the only non-`Refl`-trivial sub-case (the replaced
subtree's own recursion) is exactly the outer induction's IH.
`lookup_leaf_locality_witness`: `insert` at `Leaf` produces a single node
labeled `key`; the same True/True-contradiction / False-either-way dispatch
as the overwrite case, but simpler (no `k2`/`v2` to relate — the target is
the empty-lookup `None v` directly via the same `lookup_into_l_bridge`/
`_into_r` bridges Law 2 already built).

This section also proves order-equivalence agreement along the way:
`lookup_order_equiv_agree` — `lookup` agrees for order-equivalent query
keys, any tree — since two keys that are mutually `leq` in both directions
route `insert`/`lookup` through the identical branch decisions at every
node, by the same `bool_value_eq_from_biimpl` technique.

Confirmed on the fixed kernel: builds clean, well under a second.

```ken
fn bool_value_eq_from_biimpl
  (b1 : Bool) (b2 : Bool)
  (hAB : Equal Bool b1 True → Equal Bool b2 True)
  (hBA : Equal Bool b2 True → Equal Bool b1 True)
  (o1 : Or (Equal Bool b1 True) (Equal Bool b1 False))
  (o2 : Or (Equal Bool b2 True) (Equal Bool b2 False))
  : Equal Bool b1 b2 =
  match o1 {
    Inl q1 ⇒ trans Bool b1 True b2 q1 (sym Bool b2 True (hAB q1)) ;
    Inr q1' ⇒
      match o2 {
        Inl q2 ⇒ absurd (trans Bool True b1 False (sym Bool b1 True (hBA q2)) q1') ;
        Inr q2' ⇒ trans Bool b1 False b2 q1' (sym Bool b2 False q2')
      }
  }

fn lookup_overwrite_agrees_outer
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (k2 : k)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  : Equal Bool (leq key' key) (leq key' k2) =
  bool_value_eq_from_biimpl (leq key' key) (leq key' k2)
    (λh. transLeq key' key k2 h q1)
    (λh. transLeq key' k2 key h q2)
    (bool_dichotomy (leq key' key))
    (bool_dichotomy (leq key' k2))

fn lookup_overwrite_agrees_inner
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (k2 : k)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  : Equal Bool (leq key key') (leq k2 key') =
  bool_value_eq_from_biimpl (leq key key') (leq k2 key')
    (λh. transLeq k2 key key' q2 h)
    (λh. transLeq key k2 key' q1 h)
    (bool_dichotomy (leq key key'))
    (bool_dichotomy (leq k2 key'))

fn lookup_overwrite_both_false
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (hKey : Equal Bool (leq key' key) False) (hK2 : Equal Bool (leq key' k2) False)
  : Equal (Option v) (lookup k v leq key' (Node k v l key val r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  trans (Option v) (lookup k v leq key' (Node k v l key val r))
                    (lookup k v leq key' r)
                    (lookup k v leq key' (Node k v l k2 v2 r))
    (lookup_into_r_bridge k v leq key' l key val r hKey)
    (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' r)
       (lookup_into_r_bridge k v leq key' l k2 v2 r hK2))

fn lookup_overwrite_both_inner_false
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (hOuterKey : Equal Bool (leq key' key) True) (hInnerKey : Equal Bool (leq key key') False)
  (hOuterK2 : Equal Bool (leq key' k2) True) (hInnerK2 : Equal Bool (leq k2 key') False)
  : Equal (Option v) (lookup k v leq key' (Node k v l key val r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  trans (Option v) (lookup k v leq key' (Node k v l key val r))
                    (lookup k v leq key' l)
                    (lookup k v leq key' (Node k v l k2 v2 r))
    (lookup_into_l_bridge k v leq key' l key val r hOuterKey hInnerKey)
    (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' l)
       (lookup_into_l_bridge k v leq key' l k2 v2 r hOuterK2 hInnerK2))

fn lookup_overwrite_contradiction
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  (hOuterKey : Equal Bool (leq key' key) True) (hInnerKey : Equal Bool (leq key key') True)
  : Equal (Option v) (lookup k v leq key' (Node k v l key val r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  absurd (hdist (and_intro (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) hInnerKey hOuterKey))

fn lookup_overwrite_inner_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  (hOuterKey : Equal Bool (leq key' key) True) (hOuterK2 : Equal Bool (leq key' k2) True)
  (innerAgree : Equal Bool (leq key key') (leq k2 key'))
  (oInner : Or (Equal Bool (leq key key') True) (Equal Bool (leq key key') False))
  : Equal (Option v) (lookup k v leq key' (Node k v l key val r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oInner {
    Inl hInnerKey ⇒ lookup_overwrite_contradiction k v leq key key' val k2 v2 l r hdist hOuterKey hInnerKey ;
    Inr hInnerFalse ⇒
      lookup_overwrite_both_inner_false k v leq key key' val k2 v2 l r hOuterKey hInnerFalse hOuterK2
        (trans Bool (leq k2 key') (leq key key') False (sym Bool (leq key key') (leq k2 key') innerAgree) hInnerFalse)
  }

fn lookup_overwrite_outer_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (val : v) (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  (outerAgree : Equal Bool (leq key' key) (leq key' k2))
  (innerAgree : Equal Bool (leq key key') (leq k2 key'))
  (oOuter : Or (Equal Bool (leq key' key) True) (Equal Bool (leq key' key) False))
  : Equal (Option v) (lookup k v leq key' (Node k v l key val r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oOuter {
    Inl hOuterKey ⇒
      lookup_overwrite_inner_dispatch k v leq key key' val k2 v2 l r hdist
        hOuterKey (trans Bool (leq key' k2) (leq key' key) True (sym Bool (leq key' key) (leq key' k2) outerAgree) hOuterKey)
        innerAgree
        (bool_dichotomy (leq key key')) ;
    Inr hOuterFalse ⇒
      lookup_overwrite_both_false k v leq key key' val k2 v2 l r hOuterFalse
        (trans Bool (leq key' k2) (leq key' key) False (sym Bool (leq key' key) (leq key' k2) outerAgree) hOuterFalse)
  }

fn lookup_overwrite_locality_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (val : v) (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  : Equal (Option v) (lookup k v leq key' (Node k v l key val r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  lookup_overwrite_outer_dispatch k v leq transLeq key key' val k2 v2 l r q1 q2 hdist
    (lookup_overwrite_agrees_outer k leq transLeq key key' k2 q1 q2)
    (lookup_overwrite_agrees_inner k leq transLeq key key' k2 q1 q2)
    (bool_dichotomy (leq key' key))

fn lookup_stop_result
  (k : Type) (v : Type) (leq : k → k → Bool) (key' : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (hOuter : Equal Bool (leq key' k2) True) (hInner : Equal Bool (leq k2 key') True)
  : Equal (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (Some v v2) =
  trans (Option v) (lookup k v leq key' (Node k v l k2 v2 r))
                    (lookup_step_inner k v leq key' l k2 v2 r (leq k2 key'))
                    (Some v v2)
    (trans (Option v) (lookup k v leq key' (Node k v l k2 v2 r))
                       (lookup_step k v leq key' l k2 v2 r (leq key' k2))
                       (lookup_step_inner k v leq key' l k2 v2 r (leq k2 key'))
      (lookup_final_bridge k v leq key' l k2 v2 r)
      (trans (Option v) (lookup_step k v leq key' l k2 v2 r (leq key' k2))
                         (lookup_step k v leq key' l k2 v2 r True)
                         (lookup_step_inner k v leq key' l k2 v2 r (leq k2 key'))
        (lookup_mk_step_true_eq k v leq key' l k2 v2 r hOuter) (lookup_step_true_reduces k v leq key' l k2 v2 r)))
    (lookup_mk_inner_true_eq k v leq key' l k2 v2 r hInner)

fn order_equiv (k : Type) (leq : k → k → Bool) (a : k) (b : k) : Prop =
  And (Equal Bool (leq a b) True) (Equal Bool (leq b a) True)

fn lookup_order_equiv_outer_agree
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (k2 : k)
  (heq : order_equiv k leq key key')
  : Equal Bool (leq key k2) (leq key' k2) =
  bool_value_eq_from_biimpl (leq key k2) (leq key' k2)
    (λh. transLeq key' key k2 (and_snd (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) heq) h)
    (λh. transLeq key key' k2 (and_fst (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) heq) h)
    (bool_dichotomy (leq key k2))
    (bool_dichotomy (leq key' k2))

fn lookup_order_equiv_inner_agree
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (k2 : k)
  (heq : order_equiv k leq key key')
  : Equal Bool (leq k2 key) (leq k2 key') =
  bool_value_eq_from_biimpl (leq k2 key) (leq k2 key')
    (λh. transLeq k2 key key' h (and_fst (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) heq))
    (λh. transLeq k2 key' key h (and_snd (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) heq))
    (bool_dichotomy (leq k2 key))
    (bool_dichotomy (leq k2 key'))

fn lookup_order_equiv_both_false
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ihR : Equal (Option v) (lookup k v leq key r) (lookup k v leq key' r))
  (hOuter : Equal Bool (leq key k2) False) (hOuter' : Equal Bool (leq key' k2) False)
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  trans (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                    (lookup k v leq key r)
                    (lookup k v leq key' (Node k v l k2 v2 r))
    (lookup_into_r_bridge k v leq key l k2 v2 r hOuter)
    (trans (Option v) (lookup k v leq key r)
                       (lookup k v leq key' r)
                       (lookup k v leq key' (Node k v l k2 v2 r))
      ihR
      (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' r)
         (lookup_into_r_bridge k v leq key' l k2 v2 r hOuter')))

fn lookup_order_equiv_both_inner_false
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ihL : Equal (Option v) (lookup k v leq key l) (lookup k v leq key' l))
  (hOuter : Equal Bool (leq key k2) True) (hInner : Equal Bool (leq k2 key) False)
  (hOuter' : Equal Bool (leq key' k2) True) (hInner' : Equal Bool (leq k2 key') False)
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  trans (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                    (lookup k v leq key l)
                    (lookup k v leq key' (Node k v l k2 v2 r))
    (lookup_into_l_bridge k v leq key l k2 v2 r hOuter hInner)
    (trans (Option v) (lookup k v leq key l)
                       (lookup k v leq key' l)
                       (lookup k v leq key' (Node k v l k2 v2 r))
      ihL
      (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' l)
         (lookup_into_l_bridge k v leq key' l k2 v2 r hOuter' hInner')))

fn lookup_order_equiv_both_stop
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (hOuter : Equal Bool (leq key k2) True) (hInner : Equal Bool (leq k2 key) True)
  (hOuter' : Equal Bool (leq key' k2) True) (hInner' : Equal Bool (leq k2 key') True)
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  trans (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                    (Some v v2)
                    (lookup k v leq key' (Node k v l k2 v2 r))
    (lookup_stop_result k v leq key l k2 v2 r hOuter hInner)
    (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (Some v v2)
       (lookup_stop_result k v leq key' l k2 v2 r hOuter' hInner'))

fn lookup_order_equiv_inner_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (heq : order_equiv k leq key key')
  (ihL : Equal (Option v) (lookup k v leq key l) (lookup k v leq key' l))
  (outerAgree : Equal Bool (leq key k2) (leq key' k2))
  (innerAgree : Equal Bool (leq k2 key) (leq k2 key'))
  (hOuter : Equal Bool (leq key k2) True)
  (oInner : Or (Equal Bool (leq k2 key) True) (Equal Bool (leq k2 key) False))
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oInner {
    Inl hInner ⇒
      lookup_order_equiv_both_stop k v leq key key' l k2 v2 r hOuter hInner
        (trans Bool (leq key' k2) (leq key k2) True (sym Bool (leq key k2) (leq key' k2) outerAgree) hOuter)
        (trans Bool (leq k2 key') (leq k2 key) True (sym Bool (leq k2 key) (leq k2 key') innerAgree) hInner) ;
    Inr hInnerFalse ⇒
      lookup_order_equiv_both_inner_false k v leq key key' l k2 v2 r ihL hOuter hInnerFalse
        (trans Bool (leq key' k2) (leq key k2) True (sym Bool (leq key k2) (leq key' k2) outerAgree) hOuter)
        (trans Bool (leq k2 key') (leq k2 key) False (sym Bool (leq k2 key) (leq k2 key') innerAgree) hInnerFalse)
  }

fn lookup_order_equiv_node_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (heq : order_equiv k leq key key')
  (ihL : Equal (Option v) (lookup k v leq key l) (lookup k v leq key' l))
  (ihR : Equal (Option v) (lookup k v leq key r) (lookup k v leq key' r))
  (outerAgree : Equal Bool (leq key k2) (leq key' k2))
  (innerAgree : Equal Bool (leq k2 key) (leq k2 key'))
  (oOuter : Or (Equal Bool (leq key k2) True) (Equal Bool (leq key k2) False))
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oOuter {
    Inl hOuter ⇒ lookup_order_equiv_inner_dispatch k v leq transLeq key key' l k2 v2 r heq ihL outerAgree innerAgree hOuter (bool_dichotomy (leq k2 key)) ;
    Inr hOuterFalse ⇒
      lookup_order_equiv_both_false k v leq key key' l k2 v2 r ihR hOuterFalse
        (trans Bool (leq key' k2) (leq key k2) False (sym Bool (leq key k2) (leq key' k2) outerAgree) hOuterFalse)
  }

fn lookup_order_equiv_agree
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (m : Tree k v)
  : order_equiv k leq key key' →
    Equal (Option v) (lookup k v leq key m) (lookup k v leq key' m) =
  match m {
    Leaf ⇒ λheq. tt ;
    Node l k2 v2 r ⇒
      λheq.
        lookup_order_equiv_node_dispatch k v leq transLeq key key' l k2 v2 r heq
          (lookup_order_equiv_agree k v leq transLeq key key' l heq)
          (lookup_order_equiv_agree k v leq transLeq key key' r heq)
          (lookup_order_equiv_outer_agree k leq transLeq key key' k2 heq)
          (lookup_order_equiv_inner_agree k leq transLeq key key' k2 heq)
          (bool_dichotomy (leq key k2))
  }

fn member_order_equiv_agree
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (m : Tree k v)
  : order_equiv k leq key key' →
    Equal Bool (member k v leq key m) (member k v leq key' m) =
  λheq.
    cong (Option v) Bool (lookup k v leq key m) (lookup k v leq key' m)
      (λo. match o { None ⇒ False ; Some x ⇒ True })
      (lookup_order_equiv_agree k v leq transLeq key key' m heq)

fn set_member_order_equiv_agree
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (s : Tree k Unit)
  : order_equiv k leq key key' →
    Equal Bool (set_member k leq key s) (set_member k leq key' s) =
  member_order_equiv_agree k Unit leq transLeq key key' s

fn lookup_into_l_inner_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' (insert k v leq key val l)) (lookup k v leq key' l))
  (hOuter : Equal Bool (leq key' k2) True)
  (oInner : Or (Equal Bool (leq k2 key') True) (Equal Bool (leq k2 key') False))
  : Equal (Option v) (lookup k v leq key' (Node k v (insert k v leq key val l) k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oInner {
    Inl hInner ⇒
      trans (Option v) (lookup k v leq key' (Node k v (insert k v leq key val l) k2 v2 r)) (Some v v2) (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_stop_result k v leq key' (insert k v leq key val l) k2 v2 r hOuter hInner)
        (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (Some v v2) (lookup_stop_result k v leq key' l k2 v2 r hOuter hInner)) ;
    Inr hInnerFalse ⇒
      trans (Option v) (lookup k v leq key' (Node k v (insert k v leq key val l) k2 v2 r))
                        (lookup k v leq key' (insert k v leq key val l))
                        (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_into_l_bridge k v leq key' (insert k v leq key val l) k2 v2 r hOuter hInnerFalse)
        (trans (Option v) (lookup k v leq key' (insert k v leq key val l))
                           (lookup k v leq key' l)
                           (lookup k v leq key' (Node k v l k2 v2 r))
          ih
          (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' l)
             (lookup_into_l_bridge k v leq key' l k2 v2 r hOuter hInnerFalse)))
  }

fn lookup_into_l_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' (insert k v leq key val l)) (lookup k v leq key' l))
  (oOuter : Or (Equal Bool (leq key' k2) True) (Equal Bool (leq key' k2) False))
  : Equal (Option v) (lookup k v leq key' (Node k v (insert k v leq key val l) k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oOuter {
    Inl hOuter ⇒ lookup_into_l_inner_dispatch k v leq key key' val k2 v2 l r ih hOuter (bool_dichotomy (leq k2 key')) ;
    Inr hOuterFalse ⇒
      trans (Option v) (lookup k v leq key' (Node k v (insert k v leq key val l) k2 v2 r))
                        (lookup k v leq key' r)
                        (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_into_r_bridge k v leq key' (insert k v leq key val l) k2 v2 r hOuterFalse)
        (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' r)
           (lookup_into_r_bridge k v leq key' l k2 v2 r hOuterFalse))
  }

fn lookup_into_l_locality_witness
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' (insert k v leq key val l)) (lookup k v leq key' l))
  : Equal (Option v) (lookup k v leq key' (Node k v (insert k v leq key val l) k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  lookup_into_l_dispatch k v leq key key' val k2 v2 l r ih (bool_dichotomy (leq key' k2))

fn lookup_into_r_inner_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' (insert k v leq key val r)) (lookup k v leq key' r))
  (hOuter : Equal Bool (leq key' k2) True)
  (oInner : Or (Equal Bool (leq k2 key') True) (Equal Bool (leq k2 key') False))
  : Equal (Option v) (lookup k v leq key' (Node k v l k2 v2 (insert k v leq key val r)))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oInner {
    Inl hInner ⇒
      trans (Option v) (lookup k v leq key' (Node k v l k2 v2 (insert k v leq key val r))) (Some v v2) (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_stop_result k v leq key' l k2 v2 (insert k v leq key val r) hOuter hInner)
        (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (Some v v2) (lookup_stop_result k v leq key' l k2 v2 r hOuter hInner)) ;
    Inr hInnerFalse ⇒
      trans (Option v) (lookup k v leq key' (Node k v l k2 v2 (insert k v leq key val r)))
                        (lookup k v leq key' l)
                        (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_into_l_bridge k v leq key' l k2 v2 (insert k v leq key val r) hOuter hInnerFalse)
        (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' l)
           (lookup_into_l_bridge k v leq key' l k2 v2 r hOuter hInnerFalse))
  }

fn lookup_into_r_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' (insert k v leq key val r)) (lookup k v leq key' r))
  (oOuter : Or (Equal Bool (leq key' k2) True) (Equal Bool (leq key' k2) False))
  : Equal (Option v) (lookup k v leq key' (Node k v l k2 v2 (insert k v leq key val r)))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oOuter {
    Inl hOuter ⇒ lookup_into_r_inner_dispatch k v leq key key' val k2 v2 l r ih hOuter (bool_dichotomy (leq k2 key')) ;
    Inr hOuterFalse ⇒
      trans (Option v) (lookup k v leq key' (Node k v l k2 v2 (insert k v leq key val r)))
                        (lookup k v leq key' (insert k v leq key val r))
                        (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_into_r_bridge k v leq key' l k2 v2 (insert k v leq key val r) hOuterFalse)
        (trans (Option v) (lookup k v leq key' (insert k v leq key val r))
                           (lookup k v leq key' r)
                           (lookup k v leq key' (Node k v l k2 v2 r))
          ih
          (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' r)
             (lookup_into_r_bridge k v leq key' l k2 v2 r hOuterFalse)))
  }

fn lookup_into_r_locality_witness
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (k2 : k) (v2 : v) (l : Tree k v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' (insert k v leq key val r)) (lookup k v leq key' r))
  : Equal (Option v) (lookup k v leq key' (Node k v l k2 v2 (insert k v leq key val r)))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  lookup_into_r_dispatch k v leq key key' val k2 v2 l r ih (bool_dichotomy (leq key' k2))

fn lookup_leaf_locality_dispatch_inner
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  (hOuter : Equal Bool (leq key' key) True)
  (oInner : Or (Equal Bool (leq key key') True) (Equal Bool (leq key key') False))
  : Equal (Option v) (lookup k v leq key' (Node k v (Leaf k v) key val (Leaf k v))) (None v) =
  match oInner {
    Inl hInner ⇒
      absurd (hdist (and_intro (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) hInner hOuter)) ;
    Inr hInnerFalse ⇒
      lookup_into_l_bridge k v leq key' (Leaf k v) key val (Leaf k v) hOuter hInnerFalse
  }

fn lookup_leaf_locality_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  (oOuter : Or (Equal Bool (leq key' key) True) (Equal Bool (leq key' key) False))
  : Equal (Option v) (lookup k v leq key' (Node k v (Leaf k v) key val (Leaf k v))) (None v) =
  match oOuter {
    Inl hOuter ⇒ lookup_leaf_locality_dispatch_inner k v leq key key' val hdist hOuter (bool_dichotomy (leq key key')) ;
    Inr hOuterFalse ⇒ lookup_into_r_bridge k v leq key' (Leaf k v) key val (Leaf k v) hOuterFalse
  }

fn lookup_leaf_locality_witness
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (key' : k) (val : v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  : Equal (Option v) (lookup k v leq key' (Node k v (Leaf k v) key val (Leaf k v))) (None v) =
  lookup_leaf_locality_dispatch k v leq key key' val hdist (bool_dichotomy (leq key' key))

fn lookup_locality_q2_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  (insL : Equal (Option v) (lookup k v leq key' (insert k v leq key val l)) (lookup k v leq key' l))
  (insR : Equal (Option v) (lookup k v leq key' (insert k v leq key val r)) (lookup k v leq key' r))
  (q1 : Equal Bool (leq key k2) True)
  (o2 : Or (Equal Bool (leq k2 key) True) (Equal Bool (leq k2 key) False))
  : Equal (Option v) (lookup k v leq key' (insert k v leq key val (Node k v l k2 v2 r)))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match o2 {
    Inl q2 ⇒
      insert_case_transport_overwrite k v leq key val l k2 v2 r (λx. Equal (Option v) (lookup k v leq key' x) (lookup k v leq key' (Node k v l k2 v2 r))) q1 q2
        (lookup_overwrite_locality_witness k v leq transLeq key key' val k2 v2 l r q1 q2 hdist) ;
    Inr q2' ⇒
      insert_case_transport_into_l k v leq key val l k2 v2 r (λx. Equal (Option v) (lookup k v leq key' x) (lookup k v leq key' (Node k v l k2 v2 r))) q1 q2'
        (lookup_into_l_locality_witness k v leq key key' val k2 v2 l r insL)
  }

fn lookup_locality_node_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  (insL : Equal (Option v) (lookup k v leq key' (insert k v leq key val l)) (lookup k v leq key' l))
  (insR : Equal (Option v) (lookup k v leq key' (insert k v leq key val r)) (lookup k v leq key' r))
  (o1 : Or (Equal Bool (leq key k2) True) (Equal Bool (leq key k2) False))
  : Equal (Option v) (lookup k v leq key' (insert k v leq key val (Node k v l k2 v2 r)))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match o1 {
    Inl q1 ⇒ lookup_locality_q2_dispatch k v leq transLeq key key' val l k2 v2 r hdist insL insR q1 (bool_dichotomy (leq k2 key)) ;
    Inr q1' ⇒
      insert_case_transport_into_r k v leq key val l k2 v2 r (λx. Equal (Option v) (lookup k v leq key' x) (lookup k v leq key' (Node k v l k2 v2 r))) q1'
        (lookup_into_r_locality_witness k v leq key key' val k2 v2 l r insR)
  }

fn lookup_locality
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (val : v) (m : Tree k v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  : Equal (Option v) (lookup k v leq key' (insert k v leq key val m)) (lookup k v leq key' m) =
  match m {
    Leaf ⇒ lookup_leaf_locality_witness k v leq key key' val hdist ;
    Node l k2 v2 r ⇒
      lookup_locality_node_dispatch k v leq transLeq key key' val l k2 v2 r hdist
        (lookup_locality k v leq transLeq key key' val l hdist)
        (lookup_locality k v leq transLeq key key' val r hdist)
        (bool_dichotomy (leq key k2))
  }

fn insert_lookup_hit
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (inserted : k) (query : k) (val : v) (acc : Tree k v)
  (heq : order_equiv k leq query inserted)
  : Equal (Option v)
      (lookup k v leq query (insert k v leq inserted val acc))
      (Some v val) =
  trans (Option v)
    (lookup k v leq query (insert k v leq inserted val acc))
    (lookup k v leq inserted (insert k v leq inserted val acc))
    (Some v val)
    (lookup_order_equiv_agree k v leq transLeq query inserted
      (insert k v leq inserted val acc) heq)
    (lookup_found_after_insert k v leq reflLeq inserted val acc)

```

### 4.6 Law 5 — `lookup_assoc_agree`: dictionary agreement with the ordered-list lookup

Law 5 (`54 §5.3`, the final capstone law) states `Ordered m → Distinct leq m
→ lookup key m = assoc key (to_list m)`. The dictionary law needed is
`transLeq` only (matching `54 §5.2`'s restated law-5 list; `antisym`
belongs only to a separate `Distinct`-discharge lemma, not this statement.

`Distinct leq m := NoDup leq (to_list m)` — no two entries in the in-order
traversal carry order-equivalent keys. Without this hypothesis the law is
false: `Node (Node Leaf key v1 Leaf) key v2 Leaf` is a legitimate
weak-bounds `Ordered` witness (confirmed empirically via `ken-interp`
evaluation, escalated and resolved as a spec restatement) with `lookup key
= Some v2` (the root, first BST match) but `assoc key (to_list) = Some v1`
(the list-first match).

The mechanism: `order_equiv`/`NoDup`/`Distinct` are `Ω`-valued,
comparison-free structural predicates (`54 §4`). `Not` (a new prelude
registration, `¬A := A -> Bottom`) is needed here because the surface has
no expression-position `->` — only a `view`'s type-annotation position
parses the Pi-sugar (confirmed empirically); `NoDup`'s per-entry negation
predicate is a `Prop`-returning value, not a type annotation.
`assoc_step`/`assoc_step_inner` mirror `assoc`'s own two-level stuck match
(the same technique as `lookup_step`/`insert_step`), with the usual
stop-one-step-short bridges. `assoc_skip_prefix` shows that if no entry in
a list prefix order-matches `key`, `assoc` skips it entirely and continues
into the suffix; `assoc_prefix_wins`/`assoc_no_match_is_none` are the
mirror-image lemmas for when the suffix (not the prefix) is guaranteed
match-free. `no_dup_append_head_excl`/`no_dup_append_left`/
`no_dup_append_right` are structural decompositions of `NoDup` over
`list_append`, extracting "this entry excludes every earlier entry" and the
two tail-`Distinct` facts the outer induction's IH needs.
`not_match_from_bound_below`/`_above` plus `all_in_list_map_not_match_below`/
`_above` convert an `Ordered`-derived `le_below`/`le_above` bound plus a
single `leq key k2 = False` fact into "no bounded entry order-matches key",
via `transLeq` plus `absurd` on the resulting `Equal Bool True False`.
`not_match_transfer_via_equiv`: at the order-equivalent (`key`~`k2`)
branch, transfers a "not order-equiv to `k2`" fact (directly available from
`Distinct`) to "not order-equiv to `key`" — the matched-node value
agreement itself is then `refl`: both traversals return the value at the
unique order-equivalent entry, so no `Equal key k2` step is needed.

```ken
fn not_order_equiv_to_key (k : Type) (leq : k → k → Bool) (key : k) (k2 : k) : Prop =
  Not (order_equiv k leq key k2)

fn not_order_equiv_from_left_false
  (k : Type) (leq : k → k → Bool) (a : k) (b : k)
  (hfalse : Equal Bool (leq a b) False)
  : not_order_equiv_to_key k leq a b =
  λheq.
    absurd (trans Bool True (leq a b) False
      (sym Bool (leq a b) True
        (and_fst (Equal Bool (leq a b) True) (Equal Bool (leq b a) True) heq))
      hfalse)

fn not_order_equiv_from_right_false
  (k : Type) (leq : k → k → Bool) (a : k) (b : k)
  (hfalse : Equal Bool (leq b a) False)
  : not_order_equiv_to_key k leq a b =
  λheq.
    absurd (trans Bool True (leq b a) False
      (sym Bool (leq b a) True
        (and_snd (Equal Bool (leq a b) True) (Equal Bool (leq b a) True) heq))
      hfalse)

fn NoDup (k : Type) (v : Type) (leq : k → k → Bool) (xs : List (Pair k v)) : Prop =
  match xs {
    Nil ⇒ Equal Bool True True ;
    Cons e xs2 ⇒ And (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2) (NoDup k v leq xs2)
  }

fn Distinct (k : Type) (v : Type) (leq : k → k → Bool) (m : Tree k v) : Prop =
  NoDup k v leq (to_list k v m)

fn distinct_empty (k : Type) (v : Type) (leq : k → k → Bool) :
  Distinct k v leq (empty k v) = tt

fn assoc_step_inner
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (y : Bool) : Option v =
  match y {
    True  ⇒ Some v (pair_snd k v e) ;
    False ⇒ assoc k v leq key xs2
  }

fn assoc_step
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (x : Bool) : Option v =
  match x {
    True  ⇒ assoc_step_inner k v leq key e xs2 (leq (pair_fst k v e) key) ;
    False ⇒ assoc k v leq key xs2
  }

fn assoc_final_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2))
                      (assoc_step k v leq key e xs2 (leq key (pair_fst k v e))) = Refl

fn assoc_mk_step_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (q1 : Equal Bool (leq key (pair_fst k v e)) True)
  : Equal (Option v) (assoc_step k v leq key e xs2 (leq key (pair_fst k v e)))
                      (assoc_step k v leq key e xs2 True) =
  cong Bool (Option v) (leq key (pair_fst k v e)) True (assoc_step k v leq key e xs2) q1

fn assoc_mk_step_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (q1 : Equal Bool (leq key (pair_fst k v e)) False)
  : Equal (Option v) (assoc_step k v leq key e xs2 (leq key (pair_fst k v e)))
                      (assoc_step k v leq key e xs2 False) =
  cong Bool (Option v) (leq key (pair_fst k v e)) False (assoc_step k v leq key e xs2) q1

fn assoc_mk_inner_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (q2 : Equal Bool (leq (pair_fst k v e) key) False)
  : Equal (Option v) (assoc_step_inner k v leq key e xs2 (leq (pair_fst k v e) key))
                      (assoc_step_inner k v leq key e xs2 False) =
  cong Bool (Option v) (leq (pair_fst k v e) key) False (assoc_step_inner k v leq key e xs2) q2

fn assoc_step_true_reduces
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  : Equal (Option v) (assoc_step k v leq key e xs2 True)
                      (assoc_step_inner k v leq key e xs2 (leq (pair_fst k v e) key)) = Refl

fn assoc_skip_head_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (q1 : Equal Bool (leq key (pair_fst k v e)) True) (q2 : Equal Bool (leq (pair_fst k v e) key) False)
  : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (assoc k v leq key xs2) =
  trans (Option v) (assoc k v leq key (Cons (Pair k v) e xs2))
                    (assoc_step_inner k v leq key e xs2 (leq (pair_fst k v e) key))
                    (assoc k v leq key xs2)
    (trans (Option v) (assoc k v leq key (Cons (Pair k v) e xs2))
                       (assoc_step k v leq key e xs2 (leq key (pair_fst k v e)))
                       (assoc_step_inner k v leq key e xs2 (leq (pair_fst k v e) key))
      (assoc_final_bridge k v leq key e xs2)
      (trans (Option v) (assoc_step k v leq key e xs2 (leq key (pair_fst k v e)))
                         (assoc_step k v leq key e xs2 True)
                         (assoc_step_inner k v leq key e xs2 (leq (pair_fst k v e) key))
        (assoc_mk_step_true_eq k v leq key e xs2 q1) (assoc_step_true_reduces k v leq key e xs2)))
    (assoc_mk_inner_false_eq k v leq key e xs2 q2)

fn assoc_skip_head_bridge_false
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (q1 : Equal Bool (leq key (pair_fst k v e)) False)
  : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (assoc k v leq key xs2) =
  trans (Option v) (assoc k v leq key (Cons (Pair k v) e xs2))
                    (assoc_step k v leq key e xs2 (leq key (pair_fst k v e)))
                    (assoc k v leq key xs2)
    (assoc_final_bridge k v leq key e xs2)
    (assoc_mk_step_false_eq k v leq key e xs2 q1)

fn assoc_skip_prefix_inner
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (ys : List (Pair k v))
  (hnot : Not (order_equiv k leq key (pair_fst k v e)))
  (ihTail : Equal (Option v) (assoc k v leq key (list_append (Pair k v) xs2 ys)) (assoc k v leq key ys))
  (hOuter : Equal Bool (leq key (pair_fst k v e)) True)
  (oInner : Or (Equal Bool (leq (pair_fst k v e) key) True) (Equal Bool (leq (pair_fst k v e) key) False))
  : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e (list_append (Pair k v) xs2 ys))) (assoc k v leq key ys) =
  match oInner {
    Inl hInner ⇒
      absurd (hnot (and_intro (Equal Bool (leq key (pair_fst k v e)) True) (Equal Bool (leq (pair_fst k v e) key) True) hOuter hInner)) ;
    Inr hInnerFalse ⇒
      trans (Option v) (assoc k v leq key (Cons (Pair k v) e (list_append (Pair k v) xs2 ys)))
                        (assoc k v leq key (list_append (Pair k v) xs2 ys))
                        (assoc k v leq key ys)
        (assoc_skip_head_bridge k v leq key e (list_append (Pair k v) xs2 ys) hOuter hInnerFalse)
        ihTail
  }

fn assoc_skip_prefix_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (ys : List (Pair k v))
  (hnot : Not (order_equiv k leq key (pair_fst k v e)))
  (ihTail : Equal (Option v) (assoc k v leq key (list_append (Pair k v) xs2 ys)) (assoc k v leq key ys))
  (oOuter : Or (Equal Bool (leq key (pair_fst k v e)) True) (Equal Bool (leq key (pair_fst k v e)) False))
  : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e (list_append (Pair k v) xs2 ys))) (assoc k v leq key ys) =
  match oOuter {
    Inl hOuter ⇒
      assoc_skip_prefix_inner k v leq key e xs2 ys hnot ihTail hOuter (bool_dichotomy (leq (pair_fst k v e) key)) ;
    Inr hOuterFalse ⇒
      trans (Option v) (assoc k v leq key (Cons (Pair k v) e (list_append (Pair k v) xs2 ys)))
                        (assoc k v leq key (list_append (Pair k v) xs2 ys))
                        (assoc k v leq key ys)
        (assoc_skip_head_bridge_false k v leq key e (list_append (Pair k v) xs2 ys) hOuterFalse)
        ihTail
  }

fn assoc_skip_prefix
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (xs : List (Pair k v)) (ys : List (Pair k v))
  : all_in_list k v (not_order_equiv_to_key k leq key) xs →
    Equal (Option v) (assoc k v leq key (list_append (Pair k v) xs ys)) (assoc k v leq key ys) =
  match xs {
    Nil ⇒ λhskip. Refl ;
    Cons e xs2 ⇒
      λhskip.
        assoc_skip_prefix_dispatch k v leq key e xs2 ys
          (and_fst (not_order_equiv_to_key k leq key (pair_fst k v e)) (all_in_list k v (not_order_equiv_to_key k leq key) xs2) hskip)
          (assoc_skip_prefix k v leq key xs2 ys
             (and_snd (not_order_equiv_to_key k leq key (pair_fst k v e)) (all_in_list k v (not_order_equiv_to_key k leq key) xs2) hskip))
          (bool_dichotomy (leq key (pair_fst k v e)))
  }

fn order_equiv_sym
  (k : Type) (leq : k → k → Bool) (a : k) (b : k) (h : order_equiv k leq a b)
  : order_equiv k leq b a =
  and_intro (Equal Bool (leq b a) True) (Equal Bool (leq a b) True)
    (and_snd (Equal Bool (leq a b) True) (Equal Bool (leq b a) True) h)
    (and_fst (Equal Bool (leq a b) True) (Equal Bool (leq b a) True) h)

fn order_equiv_trans
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (a : k) (b : k) (c : k)
  (hab : order_equiv k leq a b) (hbc : order_equiv k leq b c)
  : order_equiv k leq a c =
  and_intro (Equal Bool (leq a c) True) (Equal Bool (leq c a) True)
    (transLeq a b c
      (and_fst (Equal Bool (leq a b) True) (Equal Bool (leq b a) True) hab)
      (and_fst (Equal Bool (leq b c) True) (Equal Bool (leq c b) True) hbc))
    (transLeq c b a
      (and_snd (Equal Bool (leq b c) True) (Equal Bool (leq c b) True) hbc)
      (and_snd (Equal Bool (leq a b) True) (Equal Bool (leq b a) True) hab))

fn not_order_equiv_swap
  (k : Type) (leq : k → k → Bool) (a : k) (b : k)
  (hnot : Not (order_equiv k leq a b))
  : Not (order_equiv k leq b a) =
  λh. hnot (order_equiv_sym k leq b a h)

fn all_in_list_append_elim_right
  (k : Type) (v : Type) (p : k → Prop) (xs : List (Pair k v)) (ys : List (Pair k v))
  : all_in_list k v p (list_append (Pair k v) xs ys) → all_in_list k v p ys =
  match xs {
    Nil ⇒ λh. h ;
    Cons e xs2 ⇒
      λh.
        all_in_list_append_elim_right k v p xs2 ys
          (and_snd (p (pair_fst k v e)) (all_in_list k v p (list_append (Pair k v) xs2 ys)) h)
  }

fn no_dup_append_head_excl_head_fact
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e2 : Pair k v) (e : Pair k v) (xs2 : List (Pair k v)) (ys : List (Pair k v))
  (h : And (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e2)) (list_append (Pair k v) xs2 (Cons (Pair k v) e ys)))
           (NoDup k v leq (list_append (Pair k v) xs2 (Cons (Pair k v) e ys))))
  : not_order_equiv_to_key k leq (pair_fst k v e) (pair_fst k v e2) =
  not_order_equiv_swap k leq (pair_fst k v e2) (pair_fst k v e)
    (and_fst (not_order_equiv_to_key k leq (pair_fst k v e2) (pair_fst k v e))
            (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e2)) ys)
       (all_in_list_append_elim_right k v (not_order_equiv_to_key k leq (pair_fst k v e2)) xs2 (Cons (Pair k v) e ys)
          (and_fst (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e2)) (list_append (Pair k v) xs2 (Cons (Pair k v) e ys)))
                  (NoDup k v leq (list_append (Pair k v) xs2 (Cons (Pair k v) e ys)))
             h)))

fn no_dup_append_head_excl_cons_arm
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e2 : Pair k v) (e : Pair k v) (xs2 : List (Pair k v)) (ys : List (Pair k v))
  (h : And (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e2)) (list_append (Pair k v) xs2 (Cons (Pair k v) e ys)))
           (NoDup k v leq (list_append (Pair k v) xs2 (Cons (Pair k v) e ys))))
  (rec : NoDup k v leq (list_append (Pair k v) xs2 (Cons (Pair k v) e ys)) →
         all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
  : And (not_order_equiv_to_key k leq (pair_fst k v e) (pair_fst k v e2))
        (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2) =
  and_intro (not_order_equiv_to_key k leq (pair_fst k v e) (pair_fst k v e2))
            (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
    (no_dup_append_head_excl_head_fact k v leq e2 e xs2 ys h)
    (rec (and_snd (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e2)) (list_append (Pair k v) xs2 (Cons (Pair k v) e ys)))
                 (NoDup k v leq (list_append (Pair k v) xs2 (Cons (Pair k v) e ys)))
            h))

fn no_dup_append_head_excl
  (k : Type) (v : Type) (leq : k → k → Bool)
  (xs : List (Pair k v)) (e : Pair k v) (ys : List (Pair k v))
  : NoDup k v leq (list_append (Pair k v) xs (Cons (Pair k v) e ys)) →
    all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs =
  match xs {
    Nil ⇒ λh. tt ;
    Cons e2 xs2 ⇒
      λh. no_dup_append_head_excl_cons_arm k v leq e2 e xs2 ys h (no_dup_append_head_excl k v leq xs2 e ys)
  }

fn assoc_no_match_inner_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (hnot : Not (order_equiv k leq key (pair_fst k v e)))
  (ihTail : Equal (Option v) (assoc k v leq key xs2) (None v))
  (hOuter : Equal Bool (leq key (pair_fst k v e)) True)
  (oInner : Or (Equal Bool (leq (pair_fst k v e) key) True) (Equal Bool (leq (pair_fst k v e) key) False))
  : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (None v) =
  match oInner {
    Inl hInner ⇒
      absurd (hnot (and_intro (Equal Bool (leq key (pair_fst k v e)) True) (Equal Bool (leq (pair_fst k v e) key) True) hOuter hInner)) ;
    Inr hInnerFalse ⇒
      trans (Option v) (assoc k v leq key (Cons (Pair k v) e xs2))
                        (assoc k v leq key xs2)
                        (None v)
        (assoc_skip_head_bridge k v leq key e xs2 hOuter hInnerFalse)
        ihTail
  }

fn assoc_no_match_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (hnot : Not (order_equiv k leq key (pair_fst k v e)))
  (ihTail : Equal (Option v) (assoc k v leq key xs2) (None v))
  (oOuter : Or (Equal Bool (leq key (pair_fst k v e)) True) (Equal Bool (leq key (pair_fst k v e)) False))
  : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (None v) =
  match oOuter {
    Inl hOuter ⇒ assoc_no_match_inner_dispatch k v leq key e xs2 hnot ihTail hOuter (bool_dichotomy (leq (pair_fst k v e) key)) ;
    Inr hOuterFalse ⇒
      trans (Option v) (assoc k v leq key (Cons (Pair k v) e xs2))
                        (assoc k v leq key xs2)
                        (None v)
        (assoc_skip_head_bridge_false k v leq key e xs2 hOuterFalse)
        ihTail
  }

fn assoc_no_match_is_none
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (xs : List (Pair k v))
  : all_in_list k v (not_order_equiv_to_key k leq key) xs → Equal (Option v) (assoc k v leq key xs) (None v) =
  match xs {
    Nil ⇒ λhskip. tt ;
    Cons e xs2 ⇒
      λhskip.
        assoc_no_match_dispatch k v leq key e xs2
          (and_fst (not_order_equiv_to_key k leq key (pair_fst k v e)) (all_in_list k v (not_order_equiv_to_key k leq key) xs2) hskip)
          (assoc_no_match_is_none k v leq key xs2
             (and_snd (not_order_equiv_to_key k leq key (pair_fst k v e)) (all_in_list k v (not_order_equiv_to_key k leq key) xs2) hskip))
          (bool_dichotomy (leq key (pair_fst k v e)))
  }

fn assoc_mk_inner_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (q2 : Equal Bool (leq (pair_fst k v e) key) True)
  : Equal (Option v) (assoc_step_inner k v leq key e xs2 (leq (pair_fst k v e) key))
                      (assoc_step_inner k v leq key e xs2 True) =
  cong Bool (Option v) (leq (pair_fst k v e) key) True (assoc_step_inner k v leq key e xs2) q2

fn assoc_skip_head_stop_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (q1 : Equal Bool (leq key (pair_fst k v e)) True) (q2 : Equal Bool (leq (pair_fst k v e) key) True)
  : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (Some v (pair_snd k v e)) =
  trans (Option v) (assoc k v leq key (Cons (Pair k v) e xs2))
                    (assoc_step_inner k v leq key e xs2 (leq (pair_fst k v e) key))
                    (Some v (pair_snd k v e))
    (trans (Option v) (assoc k v leq key (Cons (Pair k v) e xs2))
                       (assoc_step k v leq key e xs2 (leq key (pair_fst k v e)))
                       (assoc_step_inner k v leq key e xs2 (leq (pair_fst k v e) key))
      (assoc_final_bridge k v leq key e xs2)
      (trans (Option v) (assoc_step k v leq key e xs2 (leq key (pair_fst k v e)))
                         (assoc_step k v leq key e xs2 True)
                         (assoc_step_inner k v leq key e xs2 (leq (pair_fst k v e) key))
        (assoc_mk_step_true_eq k v leq key e xs2 q1) (assoc_step_true_reduces k v leq key e xs2)))
    (assoc_mk_inner_true_eq k v leq key e xs2 q2)

fn assoc_prefix_wins_inner_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (ys : List (Pair k v))
  (ihTail : Equal (Option v) (assoc k v leq key (list_append (Pair k v) xs2 ys)) (assoc k v leq key xs2))
  (hOuter : Equal Bool (leq key (pair_fst k v e)) True)
  (oInner : Or (Equal Bool (leq (pair_fst k v e) key) True) (Equal Bool (leq (pair_fst k v e) key) False))
  : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e (list_append (Pair k v) xs2 ys)))
                      (assoc k v leq key (Cons (Pair k v) e xs2)) =
  match oInner {
    Inl hInner ⇒
      trans (Option v) (assoc k v leq key (Cons (Pair k v) e (list_append (Pair k v) xs2 ys)))
                        (Some v (pair_snd k v e))
                        (assoc k v leq key (Cons (Pair k v) e xs2))
        (assoc_skip_head_stop_bridge k v leq key e (list_append (Pair k v) xs2 ys) hOuter hInner)
        (sym (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (Some v (pair_snd k v e))
           (assoc_skip_head_stop_bridge k v leq key e xs2 hOuter hInner)) ;
    Inr hInnerFalse ⇒
      trans (Option v) (assoc k v leq key (Cons (Pair k v) e (list_append (Pair k v) xs2 ys)))
                        (assoc k v leq key (list_append (Pair k v) xs2 ys))
                        (assoc k v leq key (Cons (Pair k v) e xs2))
        (assoc_skip_head_bridge k v leq key e (list_append (Pair k v) xs2 ys) hOuter hInnerFalse)
        (trans (Option v) (assoc k v leq key (list_append (Pair k v) xs2 ys))
                           (assoc k v leq key xs2)
                           (assoc k v leq key (Cons (Pair k v) e xs2))
          ihTail
          (sym (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (assoc k v leq key xs2)
             (assoc_skip_head_bridge k v leq key e xs2 hOuter hInnerFalse)))
  }

fn assoc_prefix_wins_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (ys : List (Pair k v))
  (ihTail : Equal (Option v) (assoc k v leq key (list_append (Pair k v) xs2 ys)) (assoc k v leq key xs2))
  (oOuter : Or (Equal Bool (leq key (pair_fst k v e)) True) (Equal Bool (leq key (pair_fst k v e)) False))
  : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e (list_append (Pair k v) xs2 ys)))
                      (assoc k v leq key (Cons (Pair k v) e xs2)) =
  match oOuter {
    Inl hOuter ⇒
      assoc_prefix_wins_inner_dispatch k v leq key e xs2 ys ihTail hOuter (bool_dichotomy (leq (pair_fst k v e) key)) ;
    Inr hOuterFalse ⇒
      trans (Option v) (assoc k v leq key (Cons (Pair k v) e (list_append (Pair k v) xs2 ys)))
                        (assoc k v leq key (list_append (Pair k v) xs2 ys))
                        (assoc k v leq key (Cons (Pair k v) e xs2))
        (assoc_skip_head_bridge_false k v leq key e (list_append (Pair k v) xs2 ys) hOuterFalse)
        (trans (Option v) (assoc k v leq key (list_append (Pair k v) xs2 ys))
                           (assoc k v leq key xs2)
                           (assoc k v leq key (Cons (Pair k v) e xs2))
          ihTail
          (sym (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (assoc k v leq key xs2)
             (assoc_skip_head_bridge_false k v leq key e xs2 hOuterFalse)))
  }

fn assoc_prefix_wins
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (xs : List (Pair k v)) (ys : List (Pair k v))
  : all_in_list k v (not_order_equiv_to_key k leq key) ys →
    Equal (Option v) (assoc k v leq key (list_append (Pair k v) xs ys)) (assoc k v leq key xs) =
  match xs {
    Nil ⇒ λhskip. assoc_no_match_is_none k v leq key ys hskip ;
    Cons e xs2 ⇒
      λhskip. assoc_prefix_wins_dispatch k v leq key e xs2 ys (assoc_prefix_wins k v leq key xs2 ys hskip) (bool_dichotomy (leq key (pair_fst k v e)))
  }

fn assoc_none_implies_no_match_inner
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (hAssoc : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (None v))
  (q1 : Equal Bool (leq key (pair_fst k v e)) True)
  (o2 : Or (Equal Bool (leq (pair_fst k v e) key) True) (Equal Bool (leq (pair_fst k v e) key) False))
  : all_in_list k v (not_order_equiv_to_key k leq key) (Cons (Pair k v) e xs2) =
  match o2 {
    Inl q2 ⇒
      absurd
        (trans (Option v)
          (Some v (pair_snd k v e))
          (assoc k v leq key (Cons (Pair k v) e xs2))
          (None v)
          (sym (Option v)
            (assoc k v leq key (Cons (Pair k v) e xs2))
            (Some v (pair_snd k v e))
            (assoc_skip_head_stop_bridge k v leq key e xs2 q1 q2))
          hAssoc) ;
    Inr q2False ⇒
      and_intro
        (not_order_equiv_to_key k leq key (pair_fst k v e))
        (all_in_list k v (not_order_equiv_to_key k leq key) xs2)
        (not_order_equiv_from_right_false k leq key (pair_fst k v e) q2False)
        (assoc_none_implies_no_match k v leq key xs2
          (trans (Option v)
            (assoc k v leq key xs2)
            (assoc k v leq key (Cons (Pair k v) e xs2))
            (None v)
            (sym (Option v)
              (assoc k v leq key (Cons (Pair k v) e xs2))
              (assoc k v leq key xs2)
              (assoc_skip_head_bridge k v leq key e xs2 q1 q2False))
            hAssoc))
  }

fn assoc_none_implies_no_match_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v))
  (hAssoc : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (None v))
  (o1 : Or (Equal Bool (leq key (pair_fst k v e)) True) (Equal Bool (leq key (pair_fst k v e)) False))
  : all_in_list k v (not_order_equiv_to_key k leq key) (Cons (Pair k v) e xs2) =
  match o1 {
    Inl q1 ⇒
      assoc_none_implies_no_match_inner k v leq key e xs2 hAssoc q1
        (bool_dichotomy (leq (pair_fst k v e) key)) ;
    Inr q1False ⇒
      and_intro
        (not_order_equiv_to_key k leq key (pair_fst k v e))
        (all_in_list k v (not_order_equiv_to_key k leq key) xs2)
        (not_order_equiv_from_left_false k leq key (pair_fst k v e) q1False)
        (assoc_none_implies_no_match k v leq key xs2
          (trans (Option v)
            (assoc k v leq key xs2)
            (assoc k v leq key (Cons (Pair k v) e xs2))
            (None v)
            (sym (Option v)
              (assoc k v leq key (Cons (Pair k v) e xs2))
              (assoc k v leq key xs2)
              (assoc_skip_head_bridge_false k v leq key e xs2 q1False))
            hAssoc))
  }

fn assoc_none_implies_no_match
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (xs : List (Pair k v))
  : Equal (Option v) (assoc k v leq key xs) (None v) →
    all_in_list k v (not_order_equiv_to_key k leq key) xs =
  match xs {
    Nil ⇒ λhAssoc. tt ;
    Cons e xs2 ⇒
      λhAssoc.
        assoc_none_implies_no_match_dispatch k v leq key e xs2 hAssoc
          (bool_dichotomy (leq key (pair_fst k v e)))
  }

fn not_match_from_bound_below
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (k2 : k) (hFalse : Equal Bool (leq key k2) False)
  (ekey : k) (hBound : Equal Bool (leq ekey k2) True)
  : not_order_equiv_to_key k leq key ekey =
  λhorderEq.
    absurd (trans Bool True (leq key k2) False
              (sym Bool (leq key k2) True (transLeq key ekey k2 (and_fst (Equal Bool (leq key ekey) True) (Equal Bool (leq ekey key) True) horderEq) hBound))
              hFalse)

fn all_in_list_map_not_match_below
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (k2 : k) (hFalse : Equal Bool (leq key k2) False) (xs : List (Pair k v))
  : all_in_list k v (le_below k v leq k2) xs → all_in_list k v (not_order_equiv_to_key k leq key) xs =
  match xs {
    Nil ⇒ λh. tt ;
    Cons e xs2 ⇒
      λh.
        and_intro (not_order_equiv_to_key k leq key (pair_fst k v e))
                  (all_in_list k v (not_order_equiv_to_key k leq key) xs2)
          (not_match_from_bound_below k leq transLeq key k2 hFalse (pair_fst k v e)
             (and_fst (Equal Bool (leq (pair_fst k v e) k2) True) (all_in_list k v (le_below k v leq k2) xs2) h))
          (all_in_list_map_not_match_below k v leq transLeq key k2 hFalse xs2
             (and_snd (Equal Bool (leq (pair_fst k v e) k2) True) (all_in_list k v (le_below k v leq k2) xs2) h))
  }

fn not_match_from_bound_above
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (k2 : k) (hFalse : Equal Bool (leq k2 key) False)
  (ekey : k) (hBound : Equal Bool (leq k2 ekey) True)
  : not_order_equiv_to_key k leq key ekey =
  λhorderEq.
    absurd (trans Bool True (leq k2 key) False
              (sym Bool (leq k2 key) True (transLeq k2 ekey key hBound (and_snd (Equal Bool (leq key ekey) True) (Equal Bool (leq ekey key) True) horderEq)))
              hFalse)

fn all_in_list_map_not_match_above
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (k2 : k) (hFalse : Equal Bool (leq k2 key) False) (xs : List (Pair k v))
  : all_in_list k v (le_above k v leq k2) xs → all_in_list k v (not_order_equiv_to_key k leq key) xs =
  match xs {
    Nil ⇒ λh. tt ;
    Cons e xs2 ⇒
      λh.
        and_intro (not_order_equiv_to_key k leq key (pair_fst k v e))
                  (all_in_list k v (not_order_equiv_to_key k leq key) xs2)
          (not_match_from_bound_above k leq transLeq key k2 hFalse (pair_fst k v e)
             (and_fst (Equal Bool (leq k2 (pair_fst k v e)) True) (all_in_list k v (le_above k v leq k2) xs2) h))
          (all_in_list_map_not_match_above k v leq transLeq key k2 hFalse xs2
             (and_snd (Equal Bool (leq k2 (pair_fst k v e)) True) (all_in_list k v (le_above k v leq k2) xs2) h))
  }

fn all_in_list_append_elim_left
  (k : Type) (v : Type) (p : k → Prop) (xs : List (Pair k v)) (ys : List (Pair k v))
  : all_in_list k v p (list_append (Pair k v) xs ys) → all_in_list k v p xs =
  match xs {
    Nil ⇒ λh. tt ;
    Cons e xs2 ⇒
      λh.
        and_intro (p (pair_fst k v e)) (all_in_list k v p xs2)
          (and_fst (p (pair_fst k v e)) (all_in_list k v p (list_append (Pair k v) xs2 ys)) h)
          (all_in_list_append_elim_left k v p xs2 ys
             (and_snd (p (pair_fst k v e)) (all_in_list k v p (list_append (Pair k v) xs2 ys)) h))
  }

fn no_dup_append_left
  (k : Type) (v : Type) (leq : k → k → Bool) (xs : List (Pair k v)) (ys : List (Pair k v))
  : NoDup k v leq (list_append (Pair k v) xs ys) → NoDup k v leq xs =
  match xs {
    Nil ⇒ λh. tt ;
    Cons e xs2 ⇒
      λh.
        and_intro (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
                  (NoDup k v leq xs2)
          (all_in_list_append_elim_left k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2 ys
             (and_fst (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) (list_append (Pair k v) xs2 ys))
                     (NoDup k v leq (list_append (Pair k v) xs2 ys))
                h))
          (no_dup_append_left k v leq xs2 ys
             (and_snd (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) (list_append (Pair k v) xs2 ys))
                     (NoDup k v leq (list_append (Pair k v) xs2 ys))
                h))
  }

fn no_dup_append_right
  (k : Type) (v : Type) (leq : k → k → Bool) (xs : List (Pair k v)) (ys : List (Pair k v))
  : NoDup k v leq (list_append (Pair k v) xs ys) → NoDup k v leq ys =
  match xs {
    Nil ⇒ λh. h ;
    Cons e xs2 ⇒
      λh.
        no_dup_append_right k v leq xs2 ys
          (and_snd (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) (list_append (Pair k v) xs2 ys))
                  (NoDup k v leq (list_append (Pair k v) xs2 ys))
             h)
  }

fn lookup_stop_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r)) (Some v v2) =
  trans (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                    (lookup_step_inner k v leq key l k2 v2 r (leq k2 key))
                    (Some v v2)
    (trans (Option v) (lookup k v leq key (Node k v l k2 v2 r))
                       (lookup_step k v leq key l k2 v2 r (leq key k2))
                       (lookup_step_inner k v leq key l k2 v2 r (leq k2 key))
      (lookup_final_bridge k v leq key l k2 v2 r)
      (trans (Option v) (lookup_step k v leq key l k2 v2 r (leq key k2))
                         (lookup_step k v leq key l k2 v2 r True)
                         (lookup_step_inner k v leq key l k2 v2 r (leq k2 key))
        (lookup_mk_step_true_eq k v leq key l k2 v2 r q1) (lookup_step_true_reduces k v leq key l k2 v2 r)))
    (lookup_mk_inner_true_eq k v leq key l k2 v2 r q2)

fn not_match_transfer_via_equiv
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (k2 : k) (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  (ekey : k) (hnot : not_order_equiv_to_key k leq k2 ekey)
  : not_order_equiv_to_key k leq key ekey =
  λhorderEq.
    hnot (and_intro (Equal Bool (leq k2 ekey) True) (Equal Bool (leq ekey k2) True)
            (transLeq k2 key ekey q2 (and_fst (Equal Bool (leq key ekey) True) (Equal Bool (leq ekey key) True) horderEq))
            (transLeq ekey key k2 (and_snd (Equal Bool (leq key ekey) True) (Equal Bool (leq ekey key) True) horderEq) q1))

fn all_in_list_map_not_match_transfer
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (k2 : k) (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  (xs : List (Pair k v))
  : all_in_list k v (not_order_equiv_to_key k leq k2) xs → all_in_list k v (not_order_equiv_to_key k leq key) xs =
  match xs {
    Nil ⇒ λh. tt ;
    Cons e xs2 ⇒
      λh.
        and_intro (not_order_equiv_to_key k leq key (pair_fst k v e))
                  (all_in_list k v (not_order_equiv_to_key k leq key) xs2)
          (not_match_transfer_via_equiv k leq transLeq key k2 q1 q2 (pair_fst k v e)
             (and_fst (not_order_equiv_to_key k leq k2 (pair_fst k v e)) (all_in_list k v (not_order_equiv_to_key k leq k2) xs2) h))
          (all_in_list_map_not_match_transfer k v leq transLeq key k2 q1 q2 xs2
             (and_snd (not_order_equiv_to_key k leq k2 (pair_fst k v e)) (all_in_list k v (not_order_equiv_to_key k leq k2) xs2) h))
  }

fn law5_node_q2_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r)) (hd : Distinct k v leq (Node k v l k2 v2 r))
  (ihL : Equal (Option v) (lookup k v leq key l) (assoc k v leq key (to_list k v l)))
  (q1 : Equal Bool (leq key k2) True)
  (o2 : Or (Equal Bool (leq k2 key) True) (Equal Bool (leq k2 key) False))
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r)) (assoc k v leq key (to_list k v (Node k v l k2 v2 r))) =
  match o2 {
    Inl q2 ⇒
      sym (Option v) (assoc k v leq key (to_list k v (Node k v l k2 v2 r))) (lookup k v leq key (Node k v l k2 v2 r))
        (trans (Option v) (assoc k v leq key (to_list k v (Node k v l k2 v2 r)))
                           (assoc k v leq key (Cons (Pair k v) (mk_pair k v k2 v2) (to_list k v r)))
                           (lookup k v leq key (Node k v l k2 v2 r))
          (assoc_skip_prefix k v leq key (to_list k v l) (Cons (Pair k v) (mk_pair k v k2 v2) (to_list k v r))
             (all_in_list_map_not_match_transfer k v leq transLeq key k2 q1 q2 (to_list k v l)
                (no_dup_append_head_excl k v leq (to_list k v l) (mk_pair k v k2 v2) (to_list k v r) hd)))
          (trans (Option v) (assoc k v leq key (Cons (Pair k v) (mk_pair k v k2 v2) (to_list k v r)))
                             (Some v v2)
                             (lookup k v leq key (Node k v l k2 v2 r))
            (assoc_skip_head_stop_bridge k v leq key (mk_pair k v k2 v2) (to_list k v r) q1 q2)
            (sym (Option v) (lookup k v leq key (Node k v l k2 v2 r)) (Some v v2)
               (lookup_stop_bridge k v leq key l k2 v2 r q1 q2)))) ;
    Inr q2' ⇒
      sym (Option v) (assoc k v leq key (to_list k v (Node k v l k2 v2 r))) (lookup k v leq key (Node k v l k2 v2 r))
        (trans (Option v) (assoc k v leq key (to_list k v (Node k v l k2 v2 r)))
                           (assoc k v leq key (to_list k v l))
                           (lookup k v leq key (Node k v l k2 v2 r))
          (assoc_prefix_wins k v leq key (to_list k v l) (Cons (Pair k v) (mk_pair k v k2 v2) (to_list k v r))
             (and_intro (not_order_equiv_to_key k leq key k2) (all_in_list k v (not_order_equiv_to_key k leq key) (to_list k v r))
                (λhorderEq. absurd (trans Bool True (leq k2 key) False
                                       (sym Bool (leq k2 key) True (and_snd (Equal Bool (leq key k2) True) (Equal Bool (leq k2 key) True) horderEq))
                                       q2'))
                (all_in_list_map_not_match_above k v leq transLeq key k2 q2' (to_list k v r)
                   (all_keys_to_all_in_list k v (le_above k v leq k2) r (get_above_r k v leq l k2 v2 r h)))))
          (trans (Option v) (assoc k v leq key (to_list k v l))
                             (lookup k v leq key l)
                             (lookup k v leq key (Node k v l k2 v2 r))
            (sym (Option v) (lookup k v leq key l) (assoc k v leq key (to_list k v l)) ihL)
            (sym (Option v) (lookup k v leq key (Node k v l k2 v2 r)) (lookup k v leq key l)
               (lookup_into_l_bridge k v leq key l k2 v2 r q1 q2'))))
  }

fn law5_node_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r)) (hd : Distinct k v leq (Node k v l k2 v2 r))
  (ihL : Equal (Option v) (lookup k v leq key l) (assoc k v leq key (to_list k v l)))
  (ihR : Equal (Option v) (lookup k v leq key r) (assoc k v leq key (to_list k v r)))
  (o1 : Or (Equal Bool (leq key k2) True) (Equal Bool (leq key k2) False))
  : Equal (Option v) (lookup k v leq key (Node k v l k2 v2 r)) (assoc k v leq key (to_list k v (Node k v l k2 v2 r))) =
  match o1 {
    Inl q1 ⇒ law5_node_q2_dispatch k v leq transLeq key l k2 v2 r h hd ihL q1 (bool_dichotomy (leq k2 key)) ;
    Inr q1' ⇒
      sym (Option v) (assoc k v leq key (to_list k v (Node k v l k2 v2 r))) (lookup k v leq key (Node k v l k2 v2 r))
        (trans (Option v) (assoc k v leq key (to_list k v (Node k v l k2 v2 r)))
                           (assoc k v leq key (to_list k v r))
                           (lookup k v leq key (Node k v l k2 v2 r))
          (trans (Option v) (assoc k v leq key (to_list k v (Node k v l k2 v2 r)))
                             (assoc k v leq key (Cons (Pair k v) (mk_pair k v k2 v2) (to_list k v r)))
                             (assoc k v leq key (to_list k v r))
            (assoc_skip_prefix k v leq key (to_list k v l) (Cons (Pair k v) (mk_pair k v k2 v2) (to_list k v r))
               (all_in_list_map_not_match_below k v leq transLeq key k2 q1' (to_list k v l)
                  (all_keys_to_all_in_list k v (le_below k v leq k2) l (get_below_l k v leq l k2 v2 r h))))
            (assoc_skip_head_bridge_false k v leq key (mk_pair k v k2 v2) (to_list k v r) q1'))
          (trans (Option v) (assoc k v leq key (to_list k v r))
                             (lookup k v leq key r)
                             (lookup k v leq key (Node k v l k2 v2 r))
            (sym (Option v) (lookup k v leq key r) (assoc k v leq key (to_list k v r)) ihR)
            (sym (Option v) (lookup k v leq key (Node k v l k2 v2 r)) (lookup k v leq key r)
               (lookup_into_r_bridge k v leq key l k2 v2 r q1'))))
  }

fn law5_distinct_l
  (k : Type) (v : Type) (leq : k → k → Bool)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (hd : Distinct k v leq (Node k v l k2 v2 r))
  : Distinct k v leq l =
  no_dup_append_left k v leq (to_list k v l) (Cons (Pair k v) (mk_pair k v k2 v2) (to_list k v r)) hd

fn law5_distinct_r
  (k : Type) (v : Type) (leq : k → k → Bool)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (hd : Distinct k v leq (Node k v l k2 v2 r))
  : Distinct k v leq r =
  and_snd (all_in_list k v (not_order_equiv_to_key k leq k2) (to_list k v r)) (Distinct k v leq r)
    (no_dup_append_right k v leq (to_list k v l) (Cons (Pair k v) (mk_pair k v k2 v2) (to_list k v r)) hd)

fn lookup_assoc_agree
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (m : Tree k v)
  : Ordered k v leq m → Distinct k v leq m → Equal (Option v) (lookup k v leq key m) (assoc k v leq key (to_list k v m)) =
  match m {
    Leaf ⇒ λh. λhd. tt ;
    Node l k2 v2 r ⇒
      λh. λhd.
        law5_node_dispatch k v leq transLeq key l k2 v2 r h hd
          (lookup_assoc_agree k v leq transLeq key l (get_ordered_l k v leq l k2 v2 r h) (law5_distinct_l k v leq l k2 v2 r hd))
          (lookup_assoc_agree k v leq transLeq key r (get_ordered_r k v leq l k2 v2 r h) (law5_distinct_r k v leq l k2 v2 r hd))
          (bool_dichotomy (leq key k2))
  }
```

### 4.7 Layer-2 keyed-collection operations

Everything from here to the end of the package sits on top of the map
capstone: no new kernel primitive, postulate, or proof-relevant `Ω`
inductive. Every operation is pure and declared as a `fn`. This is
additional functionality,
not further capstone laws: `delete`, a combining `insert_with`, three
lookup-table set/map combinators (`union`/`intersection`/`difference`) each
with a full correctness suite, `Set`-level wrappers with membership and
algebraic laws, `keys`/`values` projections, and a small binary-relations
library.

#### 4.7.1 Bool algebra and the Boolean order-equivalence test

A small self-contained `Bool` algebra (`bool_and`/`bool_not`/
`cat4_bool_or` and their commutativity/associativity/idempotence/identity
laws) and a `Nat` total order (`leq_nat` with reflexivity/transitivity/
antisymmetry/totality) supporting later sections, followed by
`order_equiv_key` — a `Bool`-valued order-equivalence test — and its
correspondence lemmas to the `Prop`-valued `order_equiv` from
[§4.6](#46-law-5--lookup_assoc_agree-dictionary-agreement-with-the-ordered-list-lookup).

```ken
fn bool_and (a : Bool) (b : Bool) : Bool =
  match a { True ⇒ b ; False ⇒ False }

fn bool_not (b : Bool) : Bool =
  match b { True ⇒ False ; False ⇒ True }

fn cat4_bool_or (a : Bool) (b : Bool) : Bool =
  match a { True ⇒ True ; False ⇒ b }

fn cat4_bool_or_comm (a : Bool) (b : Bool) : Equal Bool (cat4_bool_or a b) (cat4_bool_or b a) =
  match a {
    True ⇒ match b { True ⇒ tt ; False ⇒ tt } ;
    False ⇒ match b { True ⇒ tt ; False ⇒ tt }
  }

fn cat4_bool_or_assoc (a : Bool) (b : Bool) (c : Bool)
  : Equal Bool (cat4_bool_or (cat4_bool_or a b) c) (cat4_bool_or a (cat4_bool_or b c)) =
  match a {
    True ⇒ match b { True ⇒ match c { True ⇒ tt ; False ⇒ tt } ; False ⇒ match c { True ⇒ tt ; False ⇒ tt } } ;
    False ⇒ match b { True ⇒ match c { True ⇒ tt ; False ⇒ tt } ; False ⇒ match c { True ⇒ tt ; False ⇒ tt } }
  }

fn cat4_bool_or_idempotent (a : Bool) : Equal Bool (cat4_bool_or a a) a =
  match a { True ⇒ tt ; False ⇒ tt }

fn cat4_bool_or_left_identity (a : Bool) : Equal Bool (cat4_bool_or False a) a =
  Refl

fn cat4_bool_or_right_identity (a : Bool) : Equal Bool (cat4_bool_or a False) a =
  match a { True ⇒ tt ; False ⇒ tt }

fn bool_and_comm (a : Bool) (b : Bool) : Equal Bool (bool_and a b) (bool_and b a) =
  match a {
    True ⇒ match b { True ⇒ tt ; False ⇒ tt } ;
    False ⇒ match b { True ⇒ tt ; False ⇒ tt }
  }

fn bool_and_assoc (a : Bool) (b : Bool) (c : Bool)
  : Equal Bool (bool_and (bool_and a b) c) (bool_and a (bool_and b c)) =
  match a {
    True ⇒ match b { True ⇒ match c { True ⇒ tt ; False ⇒ tt } ; False ⇒ match c { True ⇒ tt ; False ⇒ tt } } ;
    False ⇒ match b { True ⇒ match c { True ⇒ tt ; False ⇒ tt } ; False ⇒ match c { True ⇒ tt ; False ⇒ tt } }
  }

fn bool_and_idempotent (a : Bool) : Equal Bool (bool_and a a) a =
  match a { True ⇒ tt ; False ⇒ tt }

fn bool_and_left_identity (a : Bool) : Equal Bool (bool_and True a) a =
  Refl

fn bool_and_right_identity (a : Bool) : Equal Bool (bool_and a True) a =
  match a { True ⇒ tt ; False ⇒ tt }

fn leq_nat (m : Nat) (n : Nat) : Bool =
  match m {
    Zero ⇒ True ;
    Suc m2 ⇒ match n { Zero ⇒ False ; Suc n2 ⇒ leq_nat m2 n2 }
  }

fn refl_leq_nat (x : Nat) : Equal Bool (leq_nat x x) True =
  match x { Zero ⇒ tt ; Suc x2 ⇒ refl_leq_nat x2 }

fn trans_leq_nat
  (x : Nat)
  : (y : Nat) → (z : Nat) → Equal Bool (leq_nat x y) True → Equal Bool (leq_nat y z) True → Equal Bool (leq_nat x z) True =
  match x {
    Zero ⇒ λy.λz.λp.λq. tt ;
    Suc x2 ⇒
      λy. match y {
        Zero ⇒ λz.λp.λq. absurd p ;
        Suc y2 ⇒
          λz. match z {
            Zero ⇒ λp.λq. absurd q ;
            Suc z2 ⇒ λp.λq. trans_leq_nat x2 y2 z2 p q
          }
      }
  }

fn antisym_leq_nat
  (x : Nat)
  : (y : Nat) → Equal Bool (leq_nat x y) True → Equal Bool (leq_nat y x) True → Equal Nat x y =
  match x {
    Zero ⇒
      λy. match y {
        Zero ⇒ λp.λq. tt ;
        Suc y2 ⇒ λp.λq. absurd q
      } ;
    Suc x2 ⇒
      λy. match y {
        Zero ⇒ λp.λq. absurd p ;
        Suc y2 ⇒ λp.λq. cong Nat Nat x2 y2 Suc (antisym_leq_nat x2 y2 p q)
      }
  }

fn total_leq_nat (x : Nat) (y : Nat) : Or (Equal Bool (leq_nat x y) True) (Equal Bool (leq_nat y x) True) =
  match x {
    Zero ⇒ Inl (Equal Bool (leq_nat Zero y) True) (Equal Bool (leq_nat y Zero) True) tt ;
    Suc x2 ⇒
      match y {
        Zero ⇒ Inr (Equal Bool (leq_nat (Suc x2) Zero) True) (Equal Bool (leq_nat Zero (Suc x2)) True) tt ;
        Suc y2 ⇒
          match total_leq_nat x2 y2 {
            Inl h ⇒ Inl (Equal Bool (leq_nat (Suc x2) (Suc y2)) True) (Equal Bool (leq_nat (Suc y2) (Suc x2)) True) h ;
            Inr h ⇒ Inr (Equal Bool (leq_nat (Suc x2) (Suc y2)) True) (Equal Bool (leq_nat (Suc y2) (Suc x2)) True) h
          }
      }
  }

fn order_equiv_key (k : Type) (leq : k → k → Bool) (a : k) (b : k) : Bool =
  bool_and (leq a b) (leq b a)

fn bool_and_true_intro
  (a : Bool) (b : Bool)
  (ha : Equal Bool a True) (hb : Equal Bool b True)
  : Equal Bool (bool_and a b) True =
  trans Bool (bool_and a b) (bool_and True b) True
    (cong Bool Bool a True (λx. bool_and x b) ha)
    hb

fn order_equiv_key_true_from_order_equiv
  (k : Type) (leq : k → k → Bool) (a : k) (b : k)
  : order_equiv k leq a b → Equal Bool (order_equiv_key k leq a b) True =
  λh.
    bool_and_true_intro (leq a b) (leq b a)
      (and_fst (Equal Bool (leq a b) True) (Equal Bool (leq b a) True) h)
      (and_snd (Equal Bool (leq a b) True) (Equal Bool (leq b a) True) h)

fn order_equiv_from_order_equiv_key_true_inner
  (k : Type) (leq : k → k → Bool) (a : k) (b : k)
  (q1 : Equal Bool (leq a b) True)
  (o2 : Or (Equal Bool (leq b a) True) (Equal Bool (leq b a) False))
  : Equal Bool (order_equiv_key k leq a b) True → order_equiv k leq a b =
  match o2 {
    Inl q2 ⇒ λh.
      and_intro (Equal Bool (leq a b) True) (Equal Bool (leq b a) True) q1 q2 ;
    Inr q2False ⇒ λh.
      absurd
        (trans Bool
          True
          (order_equiv_key k leq a b)
          False
          (sym Bool (order_equiv_key k leq a b) True h)
          (trans Bool
            (order_equiv_key k leq a b)
            (bool_and True (leq b a))
            False
            (cong Bool Bool
              (leq a b)
              True
              (λleft. bool_and left (leq b a))
              q1)
            (cong Bool Bool
              (leq b a)
              False
              (λright. bool_and True right)
              q2False)))
  }

fn order_equiv_from_order_equiv_key_true_dispatch
  (k : Type) (leq : k → k → Bool) (a : k) (b : k)
  (o1 : Or (Equal Bool (leq a b) True) (Equal Bool (leq a b) False))
  : Equal Bool (order_equiv_key k leq a b) True → order_equiv k leq a b =
  match o1 {
    Inl q1 ⇒ order_equiv_from_order_equiv_key_true_inner k leq a b q1 (bool_dichotomy (leq b a)) ;
    Inr q1False ⇒ λh.
      absurd
        (trans Bool
          True
          (order_equiv_key k leq a b)
          False
          (sym Bool (order_equiv_key k leq a b) True h)
          (trans Bool
            (order_equiv_key k leq a b)
            (bool_and False (leq b a))
            False
            (cong Bool Bool
              (leq a b)
              False
              (λleft. bool_and left (leq b a))
              q1False)
            tt))
  }

fn order_equiv_from_order_equiv_key_true
  (k : Type) (leq : k → k → Bool) (a : k) (b : k)
  : Equal Bool (order_equiv_key k leq a b) True → order_equiv k leq a b =
  order_equiv_from_order_equiv_key_true_dispatch k leq a b (bool_dichotomy (leq a b))

fn order_equiv_key_false_to_not
  (k : Type) (leq : k → k → Bool) (a : k) (b : k)
  (hfalse : Equal Bool (order_equiv_key k leq a b) False)
  : not_order_equiv_to_key k leq a b =
  λh.
    absurd (trans Bool True (order_equiv_key k leq a b) False
              (sym Bool (order_equiv_key k leq a b) True (order_equiv_key_true_from_order_equiv k leq a b h))
              hfalse)

```

#### 4.7.2 `delete` and its correctness proofs

`delete` rebuilds the tree from a filtered `to_list`: `drop_key` marks
entries to remove, `delete_from_list_acc`/`delete_from_list` filter and
rebuild via `from_list_acc`, and `delete` composes filtering with
reconstruction. Its correctness proofs: `delete_lookup_none_law` (a deleted
key now misses), and `delete_lookup_other_key_law` (keys elsewhere are
unaffected), each built from a chain of `_dispatch`/`_hit`/`_miss`/`_survivor`
helpers mirroring the case-split structure the capstone's Law 3 locality
proof already established.

```ken
fn drop_key (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (xs : List (Pair k v)) : List (Pair k v) =
  match xs {
    Nil ⇒ Nil (Pair k v) ;
    Cons e xs2 ⇒
      match order_equiv_key k leq key (pair_fst k v e) {
        True ⇒ drop_key k v leq key xs2 ;
        False ⇒ Cons (Pair k v) e (drop_key k v leq key xs2)
      }
  }

fn delete_from_list_acc (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (xs : List (Pair k v)) (acc : Tree k v) : Tree k v =
  match xs {
    Nil ⇒ acc ;
    Cons e xs2 ⇒
      match order_equiv_key k leq key (pair_fst k v e) {
        True ⇒ delete_from_list_acc k v leq key xs2 acc ;
        False ⇒ delete_from_list_acc k v leq key xs2 (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
      }
  }

fn delete_from_list_acc_step
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v) (b : Bool) : Tree k v =
  match b {
    True ⇒ delete_from_list_acc k v leq key xs2 acc ;
    False ⇒ delete_from_list_acc k v leq key xs2 (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
  }

fn delete_from_list_acc_final_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  : Equal (Tree k v)
      (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc)
      (delete_from_list_acc_step k v leq key e xs2 acc (order_equiv_key k leq key (pair_fst k v e))) = Refl

fn delete_from_list_acc_step_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (q : Equal Bool (order_equiv_key k leq key (pair_fst k v e)) True)
  : Equal (Tree k v)
      (delete_from_list_acc_step k v leq key e xs2 acc (order_equiv_key k leq key (pair_fst k v e)))
      (delete_from_list_acc_step k v leq key e xs2 acc True) =
  cong Bool (Tree k v) (order_equiv_key k leq key (pair_fst k v e)) True (delete_from_list_acc_step k v leq key e xs2 acc) q

fn delete_from_list_acc_step_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (q : Equal Bool (order_equiv_key k leq key (pair_fst k v e)) False)
  : Equal (Tree k v)
      (delete_from_list_acc_step k v leq key e xs2 acc (order_equiv_key k leq key (pair_fst k v e)))
      (delete_from_list_acc_step k v leq key e xs2 acc False) =
  cong Bool (Tree k v) (order_equiv_key k leq key (pair_fst k v e)) False (delete_from_list_acc_step k v leq key e xs2 acc) q

fn delete_from_list_acc_step_true_reduces
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  : Equal (Tree k v)
      (delete_from_list_acc_step k v leq key e xs2 acc True)
      (delete_from_list_acc k v leq key xs2 acc) = Refl

fn delete_from_list_acc_step_false_reduces
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  : Equal (Tree k v)
      (delete_from_list_acc_step k v leq key e xs2 acc False)
      (delete_from_list_acc k v leq key xs2 (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)) = Refl

fn delete_from_list_acc_true_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (q : Equal Bool (order_equiv_key k leq key (pair_fst k v e)) True)
  : Equal (Tree k v)
      (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc)
      (delete_from_list_acc k v leq key xs2 acc) =
  trans (Tree k v)
    (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc)
    (delete_from_list_acc_step k v leq key e xs2 acc (order_equiv_key k leq key (pair_fst k v e)))
    (delete_from_list_acc k v leq key xs2 acc)
    (delete_from_list_acc_final_bridge k v leq key e xs2 acc)
    (trans (Tree k v)
      (delete_from_list_acc_step k v leq key e xs2 acc (order_equiv_key k leq key (pair_fst k v e)))
      (delete_from_list_acc_step k v leq key e xs2 acc True)
      (delete_from_list_acc k v leq key xs2 acc)
      (delete_from_list_acc_step_true_eq k v leq key e xs2 acc q)
      (delete_from_list_acc_step_true_reduces k v leq key e xs2 acc))

fn delete_from_list_acc_false_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k)
  (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (q : Equal Bool (order_equiv_key k leq key (pair_fst k v e)) False)
  : Equal (Tree k v)
      (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc)
      (delete_from_list_acc k v leq key xs2 (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)) =
  trans (Tree k v)
    (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc)
    (delete_from_list_acc_step k v leq key e xs2 acc (order_equiv_key k leq key (pair_fst k v e)))
    (delete_from_list_acc k v leq key xs2 (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
    (delete_from_list_acc_final_bridge k v leq key e xs2 acc)
    (trans (Tree k v)
      (delete_from_list_acc_step k v leq key e xs2 acc (order_equiv_key k leq key (pair_fst k v e)))
      (delete_from_list_acc_step k v leq key e xs2 acc False)
      (delete_from_list_acc k v leq key xs2 (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
      (delete_from_list_acc_step_false_eq k v leq key e xs2 acc q)
      (delete_from_list_acc_step_false_reduces k v leq key e xs2 acc))

fn delete_from_list (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (xs : List (Pair k v)) : Tree k v =
  delete_from_list_acc k v leq key xs (Leaf k v)

fn delete (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (m : Tree k v) : Tree k v =
  delete_from_list k v leq key (to_list k v m)

fn delete_from_list_acc_lookup_none_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (hacc : Equal (Option v) (lookup k v leq key acc) (None v))
  (o : Or (Equal Bool (order_equiv_key k leq key (pair_fst k v e)) True)
          (Equal Bool (order_equiv_key k leq key (pair_fst k v e)) False))
  : Equal (Option v)
      (lookup k v leq key (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc))
      (None v) =
  match o {
    Inl q ⇒
      J (λt _. Equal (Option v) (lookup k v leq key t) (None v))
        (delete_from_list_acc_lookup_none k v leq transLeq key xs2 acc hacc)
        (sym (Tree k v)
          (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc)
          (delete_from_list_acc k v leq key xs2 acc)
          (delete_from_list_acc_true_bridge k v leq key e xs2 acc q)) ;
    Inr q ⇒
      J (λt _. Equal (Option v) (lookup k v leq key t) (None v))
        (delete_from_list_acc_lookup_none k v leq transLeq key xs2
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
          (trans (Option v)
            (lookup k v leq key (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
            (lookup k v leq key acc)
            (None v)
            (lookup_locality k v leq transLeq (pair_fst k v e) key (pair_snd k v e) acc
              (not_order_equiv_swap k leq key (pair_fst k v e)
                (order_equiv_key_false_to_not k leq key (pair_fst k v e) q)))
            hacc))
        (sym (Tree k v)
          (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc)
          (delete_from_list_acc k v leq key xs2 (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (delete_from_list_acc_false_bridge k v leq key e xs2 acc q))
  }

fn delete_from_list_acc_lookup_none
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (xs : List (Pair k v)) (acc : Tree k v)
  : Equal (Option v) (lookup k v leq key acc) (None v) →
    Equal (Option v) (lookup k v leq key (delete_from_list_acc k v leq key xs acc)) (None v) =
  match xs {
    Nil ⇒ λhacc. hacc ;
    Cons e xs2 ⇒
      λhacc.
        delete_from_list_acc_lookup_none_dispatch k v leq transLeq key e xs2 acc hacc
          (bool_dichotomy (order_equiv_key k leq key (pair_fst k v e)))
  }

fn delete_from_list_acc_lookup_locality_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (hskip : all_in_list k v (not_order_equiv_to_key k leq query) (Cons (Pair k v) e xs2))
  (o : Or (Equal Bool (order_equiv_key k leq deleted (pair_fst k v e)) True)
          (Equal Bool (order_equiv_key k leq deleted (pair_fst k v e)) False))
  : Equal (Option v)
      (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
      (lookup k v leq query acc) =
  match o {
    Inl q ⇒
      trans (Option v)
        (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
        (lookup k v leq query (delete_from_list_acc k v leq deleted xs2 acc))
        (lookup k v leq query acc)
        (cong (Tree k v) (Option v)
          (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc)
          (delete_from_list_acc k v leq deleted xs2 acc)
          (lookup k v leq query)
          (delete_from_list_acc_true_bridge k v leq deleted e xs2 acc q))
        (delete_from_list_acc_lookup_locality k v leq transLeq deleted query xs2 acc
          (and_snd
            (not_order_equiv_to_key k leq query (pair_fst k v e))
            (all_in_list k v (not_order_equiv_to_key k leq query) xs2) hskip)) ;
    Inr q ⇒
      trans (Option v)
        (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
        (lookup k v leq query
          (delete_from_list_acc k v leq deleted xs2
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
        (lookup k v leq query acc)
        (cong (Tree k v) (Option v)
          (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc)
          (delete_from_list_acc k v leq deleted xs2
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq query)
          (delete_from_list_acc_false_bridge k v leq deleted e xs2 acc q))
        (trans (Option v)
          (lookup k v leq query
            (delete_from_list_acc k v leq deleted xs2
              (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
          (lookup k v leq query (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq query acc)
          (delete_from_list_acc_lookup_locality k v leq transLeq deleted query xs2
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
            (and_snd
              (not_order_equiv_to_key k leq query (pair_fst k v e))
              (all_in_list k v (not_order_equiv_to_key k leq query) xs2) hskip))
          (lookup_locality k v leq transLeq (pair_fst k v e) query (pair_snd k v e) acc
            (not_order_equiv_swap k leq query (pair_fst k v e)
              (and_fst
                (not_order_equiv_to_key k leq query (pair_fst k v e))
                (all_in_list k v (not_order_equiv_to_key k leq query) xs2) hskip))))
  }

fn delete_from_list_acc_lookup_locality
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (query : k) (xs : List (Pair k v)) (acc : Tree k v)
  : all_in_list k v (not_order_equiv_to_key k leq query) xs →
    Equal (Option v)
      (lookup k v leq query (delete_from_list_acc k v leq deleted xs acc))
      (lookup k v leq query acc) =
  match xs {
    Nil ⇒ λhskip. Refl ;
    Cons e xs2 ⇒
      λhskip.
        delete_from_list_acc_lookup_locality_dispatch k v leq transLeq deleted query e xs2 acc hskip
          (bool_dichotomy (order_equiv_key k leq deleted (pair_fst k v e)))
  }

fn delete_lookup_none_law
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (m : Tree k v)
  : Equal (Option v) (lookup k v leq key (delete k v leq key m)) (None v) =
  delete_from_list_acc_lookup_none k v leq transLeq key (to_list k v m) (Leaf k v)
    (lookup_empty_is_none k v leq key)

fn not_order_equiv_from_deleted_match
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (query : k) (entryKey : k)
  (hDeletedEntry : Equal Bool (order_equiv_key k leq deleted entryKey) True)
  (hnotDeletedQuery : not_order_equiv_to_key k leq deleted query)
  : not_order_equiv_to_key k leq query entryKey =
  λhQueryEntry.
    hnotDeletedQuery
      (order_equiv_trans k leq transLeq deleted entryKey query
        (order_equiv_from_order_equiv_key_true k leq deleted entryKey hDeletedEntry)
        (order_equiv_sym k leq query entryKey hQueryEntry))

fn delete_from_list_acc_lookup_other_assoc_deleted_hit_absurd
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (hDeletedEntry : Equal Bool (order_equiv_key k leq deleted (pair_fst k v e)) True)
  (hnotDeletedQuery : not_order_equiv_to_key k leq deleted query)
  (q1 : Equal Bool (leq query (pair_fst k v e)) True)
  (q2 : Equal Bool (leq (pair_fst k v e) query) True)
  : Equal (Option v)
      (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
      (assoc k v leq query (Cons (Pair k v) e xs2)) =
  absurd
    (hnotDeletedQuery
      (order_equiv_trans k leq transLeq deleted (pair_fst k v e) query
        (order_equiv_from_order_equiv_key_true k leq deleted (pair_fst k v e) hDeletedEntry)
        (and_intro (Equal Bool (leq (pair_fst k v e) query) True)
          (Equal Bool (leq query (pair_fst k v e)) True) q2 q1)))

fn delete_from_list_acc_lookup_other_assoc_hit_survivor
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (qDeletedFalse : Equal Bool (order_equiv_key k leq deleted (pair_fst k v e)) False)
  (q1 : Equal Bool (leq query (pair_fst k v e)) True)
  (q2 : Equal Bool (leq (pair_fst k v e) query) True)
  : Equal (Option v)
      (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
      (assoc k v leq query (Cons (Pair k v) e xs2)) =
  trans (Option v)
    (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
    (lookup k v leq query
      (delete_from_list_acc k v leq deleted xs2
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
    (assoc k v leq query (Cons (Pair k v) e xs2))
    (cong (Tree k v) (Option v)
      (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc)
      (delete_from_list_acc k v leq deleted xs2
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
      (lookup k v leq query)
      (delete_from_list_acc_false_bridge k v leq deleted e xs2 acc qDeletedFalse))
    (trans (Option v)
      (lookup k v leq query
        (delete_from_list_acc k v leq deleted xs2
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
      (lookup k v leq query (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
      (assoc k v leq query (Cons (Pair k v) e xs2))
      (delete_from_list_acc_lookup_locality k v leq transLeq deleted query xs2
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
        (all_in_list_map_not_match_transfer k v leq transLeq query (pair_fst k v e) q1 q2 xs2
          (and_fst
            (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
            (NoDup k v leq xs2) h)))
      (trans (Option v)
        (lookup k v leq query (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
        (Some v (pair_snd k v e))
        (assoc k v leq query (Cons (Pair k v) e xs2))
        (insert_lookup_hit k v leq reflLeq transLeq (pair_fst k v e) query (pair_snd k v e) acc
          (and_intro (Equal Bool (leq query (pair_fst k v e)) True)
            (Equal Bool (leq (pair_fst k v e) query) True) q1 q2))
        (sym (Option v)
          (assoc k v leq query (Cons (Pair k v) e xs2))
          (Some v (pair_snd k v e))
          (assoc_skip_head_stop_bridge k v leq query e xs2 q1 q2))))

fn delete_from_list_acc_lookup_other_assoc_hit
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (hnotDeletedQuery : not_order_equiv_to_key k leq deleted query)
  (q1 : Equal Bool (leq query (pair_fst k v e)) True)
  (q2 : Equal Bool (leq (pair_fst k v e) query) True)
  (oDeleted : Or (Equal Bool (order_equiv_key k leq deleted (pair_fst k v e)) True)
                 (Equal Bool (order_equiv_key k leq deleted (pair_fst k v e)) False))
  : Equal (Option v)
      (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
      (assoc k v leq query (Cons (Pair k v) e xs2)) =
  match oDeleted {
    Inl qDeleted ⇒
      delete_from_list_acc_lookup_other_assoc_deleted_hit_absurd k v leq transLeq deleted query e xs2 acc
        qDeleted hnotDeletedQuery q1 q2 ;
    Inr qDeletedFalse ⇒
      delete_from_list_acc_lookup_other_assoc_hit_survivor k v leq reflLeq transLeq deleted query e xs2 acc
        h qDeletedFalse q1 q2
  }

fn delete_from_list_acc_lookup_other_assoc_miss
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (hTail : NoDup k v leq xs2)
  (hnotDeletedQuery : not_order_equiv_to_key k leq deleted query)
  (hacc : Equal (Option v) (lookup k v leq query acc) (None v))
  (hnotQueryEntry : not_order_equiv_to_key k leq query (pair_fst k v e))
  (assocSkip : Equal (Option v) (assoc k v leq query (Cons (Pair k v) e xs2)) (assoc k v leq query xs2))
  (oDeleted : Or (Equal Bool (order_equiv_key k leq deleted (pair_fst k v e)) True)
                 (Equal Bool (order_equiv_key k leq deleted (pair_fst k v e)) False))
  : Equal (Option v)
      (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
      (assoc k v leq query (Cons (Pair k v) e xs2)) =
  match oDeleted {
    Inl qDeleted ⇒
      trans (Option v)
        (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
        (lookup k v leq query (delete_from_list_acc k v leq deleted xs2 acc))
        (assoc k v leq query (Cons (Pair k v) e xs2))
        (cong (Tree k v) (Option v)
          (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc)
          (delete_from_list_acc k v leq deleted xs2 acc)
          (lookup k v leq query)
          (delete_from_list_acc_true_bridge k v leq deleted e xs2 acc qDeleted))
        (trans (Option v)
          (lookup k v leq query (delete_from_list_acc k v leq deleted xs2 acc))
          (assoc k v leq query xs2)
          (assoc k v leq query (Cons (Pair k v) e xs2))
          (delete_from_list_acc_lookup_other_assoc k v leq reflLeq transLeq deleted query xs2 acc
            hTail hnotDeletedQuery hacc)
          (sym (Option v)
            (assoc k v leq query (Cons (Pair k v) e xs2))
            (assoc k v leq query xs2)
            assocSkip)) ;
    Inr qDeletedFalse ⇒
      trans (Option v)
        (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
        (lookup k v leq query
          (delete_from_list_acc k v leq deleted xs2
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
        (assoc k v leq query (Cons (Pair k v) e xs2))
        (cong (Tree k v) (Option v)
          (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc)
          (delete_from_list_acc k v leq deleted xs2
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq query)
          (delete_from_list_acc_false_bridge k v leq deleted e xs2 acc qDeletedFalse))
        (trans (Option v)
          (lookup k v leq query
            (delete_from_list_acc k v leq deleted xs2
              (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
          (assoc k v leq query xs2)
          (assoc k v leq query (Cons (Pair k v) e xs2))
          (delete_from_list_acc_lookup_other_assoc k v leq reflLeq transLeq deleted query xs2
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
            hTail hnotDeletedQuery
            (trans (Option v)
              (lookup k v leq query (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
              (lookup k v leq query acc)
              (None v)
              (lookup_locality k v leq transLeq (pair_fst k v e) query (pair_snd k v e) acc
                (not_order_equiv_swap k leq query (pair_fst k v e) hnotQueryEntry))
              hacc))
          (sym (Option v)
            (assoc k v leq query (Cons (Pair k v) e xs2))
            (assoc k v leq query xs2)
            assocSkip))
  }

fn delete_from_list_acc_lookup_other_assoc_inner
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (hnotDeletedQuery : not_order_equiv_to_key k leq deleted query)
  (hacc : Equal (Option v) (lookup k v leq query acc) (None v))
  (q1 : Equal Bool (leq query (pair_fst k v e)) True)
  (o2 : Or (Equal Bool (leq (pair_fst k v e) query) True) (Equal Bool (leq (pair_fst k v e) query) False))
  : Equal (Option v)
      (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
      (assoc k v leq query (Cons (Pair k v) e xs2)) =
  match o2 {
    Inl q2 ⇒
      delete_from_list_acc_lookup_other_assoc_hit k v leq reflLeq transLeq deleted query e xs2 acc
        h hnotDeletedQuery q1 q2
        (bool_dichotomy (order_equiv_key k leq deleted (pair_fst k v e))) ;
    Inr q2False ⇒
      delete_from_list_acc_lookup_other_assoc_miss k v leq reflLeq transLeq deleted query e xs2 acc
        (and_snd
          (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
          (NoDup k v leq xs2) h)
        hnotDeletedQuery hacc
        (not_order_equiv_from_right_false k leq query (pair_fst k v e) q2False)
        (assoc_skip_head_bridge k v leq query e xs2 q1 q2False)
        (bool_dichotomy (order_equiv_key k leq deleted (pair_fst k v e)))
  }

fn delete_from_list_acc_lookup_other_assoc_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (hnotDeletedQuery : not_order_equiv_to_key k leq deleted query)
  (hacc : Equal (Option v) (lookup k v leq query acc) (None v))
  (o1 : Or (Equal Bool (leq query (pair_fst k v e)) True) (Equal Bool (leq query (pair_fst k v e)) False))
  : Equal (Option v)
      (lookup k v leq query (delete_from_list_acc k v leq deleted (Cons (Pair k v) e xs2) acc))
      (assoc k v leq query (Cons (Pair k v) e xs2)) =
  match o1 {
    Inl q1 ⇒
      delete_from_list_acc_lookup_other_assoc_inner k v leq reflLeq transLeq deleted query e xs2 acc
        h hnotDeletedQuery hacc q1
        (bool_dichotomy (leq (pair_fst k v e) query)) ;
    Inr q1False ⇒
      delete_from_list_acc_lookup_other_assoc_miss k v leq reflLeq transLeq deleted query e xs2 acc
        (and_snd
          (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
          (NoDup k v leq xs2) h)
        hnotDeletedQuery hacc
        (not_order_equiv_from_left_false k leq query (pair_fst k v e) q1False)
        (assoc_skip_head_bridge_false k v leq query e xs2 q1False)
        (bool_dichotomy (order_equiv_key k leq deleted (pair_fst k v e)))
  }

fn delete_from_list_acc_lookup_other_assoc
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (query : k) (xs : List (Pair k v)) (acc : Tree k v)
  : NoDup k v leq xs →
    not_order_equiv_to_key k leq deleted query →
    Equal (Option v) (lookup k v leq query acc) (None v) →
    Equal (Option v)
      (lookup k v leq query (delete_from_list_acc k v leq deleted xs acc))
      (assoc k v leq query xs) =
  match xs {
    Nil ⇒ λh. λhnotDeletedQuery. λhacc. hacc ;
    Cons e xs2 ⇒
      λh. λhnotDeletedQuery. λhacc.
        delete_from_list_acc_lookup_other_assoc_dispatch k v leq reflLeq transLeq deleted query e xs2 acc
          h hnotDeletedQuery hacc
          (bool_dichotomy (leq query (pair_fst k v e)))
  }

fn delete_lookup_other_key_law
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (deleted : k) (key : k) (m : Tree k v)
  : Ordered k v leq m → Distinct k v leq m →
    not_order_equiv_to_key k leq deleted key →
    Equal (Option v) (lookup k v leq key (delete k v leq deleted m)) (lookup k v leq key m) =
  λhord. λhdist. λhnot.
    trans (Option v)
      (lookup k v leq key (delete k v leq deleted m))
      (assoc k v leq key (to_list k v m))
      (lookup k v leq key m)
      (delete_from_list_acc_lookup_other_assoc k v leq reflLeq transLeq deleted key (to_list k v m) (Leaf k v)
        hdist hnot (lookup_empty_is_none k v leq key))
      (sym (Option v)
        (lookup k v leq key m)
        (assoc k v leq key (to_list k v m))
        (lookup_assoc_agree k v leq transLeq key m hord hdist))
```

#### 4.7.3 Ordered-preservation for `from_list` and `delete`

`from_list_acc_preserves_ordered`/`from_list_preserves_ordered` show
folding `insert` over a list preserves `Ordered`; `delete_preserves_ordered`
composes this with the filter-then-rebuild shape of `delete` itself.

```ken
fn from_list_acc_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (xs : List (Pair k v)) (acc : Tree k v)
  : Ordered k v leq acc → Ordered k v leq (from_list_acc k v leq xs acc) =
  match xs {
    Nil ⇒ λh. h ;
    Cons p xs2 ⇒
      λh.
        from_list_acc_preserves_ordered k v leq transLeq total xs2
          (insert k v leq (pair_fst k v p) (pair_snd k v p) acc)
          (preserves_ordered k v leq transLeq total (pair_fst k v p) (pair_snd k v p) acc h)
  }

fn from_list_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (xs : List (Pair k v))
  : Ordered k v leq (from_list k v leq xs) =
  from_list_acc_preserves_ordered k v leq transLeq total xs (Leaf k v) (ordered_empty k v leq)

fn delete_from_list_acc_preserves_ordered_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (key : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (h : Ordered k v leq acc)
  (o : Or (Equal Bool (order_equiv_key k leq key (pair_fst k v e)) True)
          (Equal Bool (order_equiv_key k leq key (pair_fst k v e)) False))
  : Ordered k v leq (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc) =
  match o {
    Inl q ⇒
      J (λt _. Ordered k v leq t)
        (delete_from_list_acc_preserves_ordered k v leq transLeq total key xs2 acc h)
        (sym (Tree k v)
          (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc)
          (delete_from_list_acc k v leq key xs2 acc)
          (delete_from_list_acc_true_bridge k v leq key e xs2 acc q)) ;
    Inr q ⇒
      J (λt _. Ordered k v leq t)
        (delete_from_list_acc_preserves_ordered k v leq transLeq total key xs2
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
          (preserves_ordered k v leq transLeq total (pair_fst k v e) (pair_snd k v e) acc h))
        (sym (Tree k v)
          (delete_from_list_acc k v leq key (Cons (Pair k v) e xs2) acc)
          (delete_from_list_acc k v leq key xs2 (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (delete_from_list_acc_false_bridge k v leq key e xs2 acc q))
  }

fn delete_from_list_acc_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (key : k) (xs : List (Pair k v)) (acc : Tree k v)
  : Ordered k v leq acc → Ordered k v leq (delete_from_list_acc k v leq key xs acc) =
  match xs {
    Nil ⇒ λh. h ;
    Cons e xs2 ⇒
      λh.
        delete_from_list_acc_preserves_ordered_dispatch k v leq transLeq total key e xs2 acc h
          (bool_dichotomy (order_equiv_key k leq key (pair_fst k v e)))
  }

fn delete_from_list_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (key : k) (xs : List (Pair k v))
  : Ordered k v leq (delete_from_list k v leq key xs) =
  delete_from_list_acc_preserves_ordered k v leq transLeq total key xs (Leaf k v) (ordered_empty k v leq)

fn delete_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (key : k) (m : Tree k v)
  : Ordered k v leq (delete k v leq key m) =
  delete_from_list_preserves_ordered k v leq transLeq total key (to_list k v m)
```

#### 4.7.4 `insert_with`, `union`, `intersection`, `difference` — the operations

`insert_with` generalizes `insert` with a combining function for the
overwrite case, via a fold step (`insert_with_fold_step`) mirroring
`insert`'s own stuck-match shape. `union`/`union_from_list_acc` build a
combined tree from two `to_list`s, folding the second list's entries into the
first via `insert_with`; `union_lookup_table`/`unit_combine` give the
map-of-maps lookup-table view `union`'s correctness proofs are stated
against. `intersection_lookup_table`/`difference_lookup_table` are the
analogous expected-result tables for `intersection`/`difference`, together
with `all_keys_map_not_match_below`/`_above` — bound-based non-match
lemmas over a whole subtree, needed by both combinators' correctness
proofs. `intersection`/`intersection_from_list_acc` and
`difference`/`difference_from_list_acc` each filter one list against
lookups into the other tree, following the same fold-and-rebuild shape as
`union`.

```ken
fn insert_with (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v) (m : Tree k v) : Tree k v =
  match m {
    Leaf ⇒ Node k v (Leaf k v) key val (Leaf k v) ;
    Node l k2 v2 r ⇒
      match leq key k2 {
        True ⇒
          match leq k2 key {
            True ⇒ Node k v l key (f val v2) r ;
            False ⇒ Node k v (insert_with k v leq f key val l) k2 v2 r
          } ;
        False ⇒ Node k v l k2 v2 (insert_with k v leq f key val r)
      }
  }

fn insert_with_fold_step (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v) (acc : Tree k v) : Tree k v =
  insert_with k v leq f key val acc

fn insert_with_fold_step_reduces
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v) (acc : Tree k v)
  : Equal (Tree k v)
      (insert_with_fold_step k v leq f key val acc)
      (insert_with k v leq f key val acc) = Refl

fn union_from_list_acc
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v)
  (xs : List (Pair k v)) (acc : Tree k v) : Tree k v =
  match xs {
    Nil ⇒ acc ;
    Cons e xs2 ⇒
      union_from_list_acc k v leq f xs2
        (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)
  }

fn union_from_list_acc_cons_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v)
  (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  : Equal (Tree k v)
      (union_from_list_acc k v leq f (Cons (Pair k v) e xs2) acc)
      (union_from_list_acc k v leq f xs2
        (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)) = Refl

fn union (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (a : Tree k v) (b : Tree k v) : Tree k v =
  union_from_list_acc k v leq f (to_list k v a) b

fn union_lookup_table (v : Type) (f : v → v → v) (left : Option v) (right : Option v) : Option v =
  match left {
    None ⇒ right ;
    Some x ⇒ match right { None ⇒ Some v x ; Some y ⇒ Some v (f x y) }
  }

fn unit_combine (x : Unit) (y : Unit) : Unit = MkUnit

fn union_lookup_table_member
  (left : Option Unit) (right : Option Unit)
  : Equal Bool
      (option_is_some Unit (union_lookup_table Unit unit_combine left right))
      (cat4_bool_or (option_is_some Unit left) (option_is_some Unit right)) =
  match left {
    None ⇒ match right { None ⇒ tt ; Some y ⇒ tt } ;
    Some x ⇒ match right { None ⇒ tt ; Some y ⇒ tt }
  }

fn intersection_lookup_table (v : Type) (left : Option v) (keep : Bool) (prior : Option v) : Option v =
  match left {
    None ⇒ prior ;
    Some x ⇒ match keep { True ⇒ Some v x ; False ⇒ prior }
  }

fn difference_lookup_table (v : Type) (left : Option v) (reject : Bool) (prior : Option v) : Option v =
  match left {
    None ⇒ prior ;
    Some x ⇒ match reject { True ⇒ prior ; False ⇒ Some v x }
  }

const difference_lookup_table_false_none_none
  (v : Type)
  : Equal (Option v) (difference_lookup_table v (None v) False (None v)) (None v) = tt

fn difference_lookup_table_false_none_some
  (v : Type) (x : v)
  : Equal (Option v) (difference_lookup_table v (Some v x) False (None v)) (Some v x) = Refl

fn difference_lookup_table_false_none
  (v : Type) (left : Option v)
  : Equal (Option v) (difference_lookup_table v left False (None v)) left =
  match left {
    None ⇒ difference_lookup_table_false_none_none v ;
    Some x ⇒ difference_lookup_table_false_none_some v x
  }

fn difference_lookup_expected
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (a : Tree k v) (b : Tree k v) : Option v =
  match member k v leq key b {
    True ⇒ None v ;
    False ⇒ lookup k v leq key a
  }

fn difference_lookup_expected_true
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (a : Tree k v) (b : Tree k v)
  (q : Equal Bool (member k v leq key b) True)
  : Equal (Option v) (difference_lookup_expected k v leq key a b) (None v) =
  cong Bool (Option v)
    (member k v leq key b)
    True
    (λm. match m { True ⇒ None v ; False ⇒ lookup k v leq key a })
    q

fn difference_lookup_expected_false
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (a : Tree k v) (b : Tree k v)
  (q : Equal Bool (member k v leq key b) False)
  : Equal (Option v) (difference_lookup_expected k v leq key a b) (lookup k v leq key a) =
  cong Bool (Option v)
    (member k v leq key b)
    False
    (λm. match m { True ⇒ None v ; False ⇒ lookup k v leq key a })
    q

fn difference_lookup_expected_member_option
  (left : Option Unit) (reject : Bool) : Option Unit =
  match reject {
    True ⇒ None Unit ;
    False ⇒ left
  }

fn difference_lookup_expected_member_table
  (left : Option Unit) (reject : Bool)
  : Equal Bool
      (option_is_some Unit (difference_lookup_expected_member_option left reject))
      (bool_and (option_is_some Unit left) (bool_not reject)) =
  match reject {
    True ⇒ match left { None ⇒ tt ; Some x ⇒ tt } ;
    False ⇒ match left { None ⇒ tt ; Some x ⇒ tt }
  }

fn difference_lookup_expected_member
  (k : Type) (leq : k → k → Bool) (key : k) (s : Tree k Unit) (t : Tree k Unit)
  : Equal Bool
      (option_is_some Unit (difference_lookup_expected k Unit leq key s t))
      (bool_and (set_member k leq key s) (bool_not (set_member k leq key t))) =
  difference_lookup_expected_member_table
    (lookup k Unit leq key s)
    (member k Unit leq key t)

fn insert_with_lookup_result (v : Type) (f : v → v → v) (val : v) (prior : Option v) : Option v =
  match prior {
    None ⇒ Some v val ;
    Some oldVal ⇒ Some v (f val oldVal)
  }

fn insert_with_lookup_result_for
  (k : Type) (v : Type) (leq : k → k → Bool)
  (f : v → v → v) (key : k) (val : v) (m : Tree k v) : Option v =
  insert_with_lookup_result v f val (lookup k v leq key m)

fn all_keys_map_not_match_below
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (k2 : k) (hFalse : Equal Bool (leq key k2) False) (m : Tree k v)
  : all_keys k v (le_below k v leq k2) m → all_keys k v (not_order_equiv_to_key k leq key) m =
  match m {
    Leaf ⇒ λh. tt ;
    Node l ekey eval r ⇒
      λh.
        and_intro (not_order_equiv_to_key k leq key ekey)
                  (And (all_keys k v (not_order_equiv_to_key k leq key) l)
                       (all_keys k v (not_order_equiv_to_key k leq key) r))
          (not_match_from_bound_below k leq transLeq key k2 hFalse ekey
            (and_fst (Equal Bool (leq ekey k2) True)
                    (And (all_keys k v (le_below k v leq k2) l)
                         (all_keys k v (le_below k v leq k2) r)) h))
          (and_intro (all_keys k v (not_order_equiv_to_key k leq key) l)
                    (all_keys k v (not_order_equiv_to_key k leq key) r)
            (all_keys_map_not_match_below k v leq transLeq key k2 hFalse l
              (and_fst (all_keys k v (le_below k v leq k2) l)
                      (all_keys k v (le_below k v leq k2) r)
                (and_snd (Equal Bool (leq ekey k2) True)
                        (And (all_keys k v (le_below k v leq k2) l)
                             (all_keys k v (le_below k v leq k2) r)) h)))
            (all_keys_map_not_match_below k v leq transLeq key k2 hFalse r
              (and_snd (all_keys k v (le_below k v leq k2) l)
                      (all_keys k v (le_below k v leq k2) r)
                (and_snd (Equal Bool (leq ekey k2) True)
                        (And (all_keys k v (le_below k v leq k2) l)
                             (all_keys k v (le_below k v leq k2) r)) h))))
  }

fn all_keys_map_not_match_above
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (k2 : k) (hFalse : Equal Bool (leq k2 key) False) (m : Tree k v)
  : all_keys k v (le_above k v leq k2) m → all_keys k v (not_order_equiv_to_key k leq key) m =
  match m {
    Leaf ⇒ λh. tt ;
    Node l ekey eval r ⇒
      λh.
        and_intro (not_order_equiv_to_key k leq key ekey)
                  (And (all_keys k v (not_order_equiv_to_key k leq key) l)
                       (all_keys k v (not_order_equiv_to_key k leq key) r))
          (not_match_from_bound_above k leq transLeq key k2 hFalse ekey
            (and_fst (Equal Bool (leq k2 ekey) True)
                    (And (all_keys k v (le_above k v leq k2) l)
                         (all_keys k v (le_above k v leq k2) r)) h))
          (and_intro (all_keys k v (not_order_equiv_to_key k leq key) l)
                    (all_keys k v (not_order_equiv_to_key k leq key) r)
            (all_keys_map_not_match_above k v leq transLeq key k2 hFalse l
              (and_fst (all_keys k v (le_above k v leq k2) l)
                      (all_keys k v (le_above k v leq k2) r)
                (and_snd (Equal Bool (leq k2 ekey) True)
                        (And (all_keys k v (le_above k v leq k2) l)
                             (all_keys k v (le_above k v leq k2) r)) h)))
            (all_keys_map_not_match_above k v leq transLeq key k2 hFalse r
              (and_snd (all_keys k v (le_above k v leq k2) l)
                      (all_keys k v (le_above k v leq k2) r)
                (and_snd (Equal Bool (leq k2 ekey) True)
                        (And (all_keys k v (le_above k v leq k2) l)
                             (all_keys k v (le_above k v leq k2) r)) h))))
  }

fn intersection_from_list_acc
  (k : Type) (v : Type) (leq : k → k → Bool)
  (xs : List (Pair k v)) (keep : Tree k v) (acc : Tree k v) : Tree k v =
  match xs {
    Nil ⇒ acc ;
    Cons e xs2 ⇒
      match member k v leq (pair_fst k v e) keep {
        True ⇒ intersection_from_list_acc k v leq xs2 keep
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc) ;
        False ⇒ intersection_from_list_acc k v leq xs2 keep acc
      }
  }

fn intersection_from_list_acc_step
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v) (m : Bool) : Tree k v =
  match m {
    True ⇒ intersection_from_list_acc k v leq xs2 keep
      (insert k v leq (pair_fst k v e) (pair_snd k v e) acc) ;
    False ⇒ intersection_from_list_acc k v leq xs2 keep acc
  }

fn intersection_from_list_acc_final_bridge
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  : Equal (Tree k v)
      (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
      (intersection_from_list_acc_step k v leq e xs2 keep acc (member k v leq (pair_fst k v e) keep)) = Refl

fn intersection_from_list_acc_step_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (q : Equal Bool (member k v leq (pair_fst k v e) keep) True)
  : Equal (Tree k v)
      (intersection_from_list_acc_step k v leq e xs2 keep acc (member k v leq (pair_fst k v e) keep))
      (intersection_from_list_acc_step k v leq e xs2 keep acc True) =
  cong Bool (Tree k v) (member k v leq (pair_fst k v e) keep) True
    (intersection_from_list_acc_step k v leq e xs2 keep acc) q

fn intersection_from_list_acc_step_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (q : Equal Bool (member k v leq (pair_fst k v e) keep) False)
  : Equal (Tree k v)
      (intersection_from_list_acc_step k v leq e xs2 keep acc (member k v leq (pair_fst k v e) keep))
      (intersection_from_list_acc_step k v leq e xs2 keep acc False) =
  cong Bool (Tree k v) (member k v leq (pair_fst k v e) keep) False
    (intersection_from_list_acc_step k v leq e xs2 keep acc) q

fn intersection_from_list_acc_step_true_reduces
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  : Equal (Tree k v)
      (intersection_from_list_acc_step k v leq e xs2 keep acc True)
      (intersection_from_list_acc k v leq xs2 keep
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)) = Refl

fn intersection_from_list_acc_step_false_reduces
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  : Equal (Tree k v)
      (intersection_from_list_acc_step k v leq e xs2 keep acc False)
      (intersection_from_list_acc k v leq xs2 keep acc) = Refl

fn intersection_from_list_acc_true_bridge
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (q : Equal Bool (member k v leq (pair_fst k v e) keep) True)
  : Equal (Tree k v)
      (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
      (intersection_from_list_acc k v leq xs2 keep
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)) =
  trans (Tree k v)
    (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
    (intersection_from_list_acc_step k v leq e xs2 keep acc (member k v leq (pair_fst k v e) keep))
    (intersection_from_list_acc k v leq xs2 keep
      (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
    (intersection_from_list_acc_final_bridge k v leq e xs2 keep acc)
    (trans (Tree k v)
      (intersection_from_list_acc_step k v leq e xs2 keep acc (member k v leq (pair_fst k v e) keep))
      (intersection_from_list_acc_step k v leq e xs2 keep acc True)
      (intersection_from_list_acc k v leq xs2 keep
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
      (intersection_from_list_acc_step_true_eq k v leq e xs2 keep acc q)
      (intersection_from_list_acc_step_true_reduces k v leq e xs2 keep acc))

fn intersection_from_list_acc_false_bridge
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (q : Equal Bool (member k v leq (pair_fst k v e) keep) False)
  : Equal (Tree k v)
      (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
      (intersection_from_list_acc k v leq xs2 keep acc) =
  trans (Tree k v)
    (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
    (intersection_from_list_acc_step k v leq e xs2 keep acc (member k v leq (pair_fst k v e) keep))
    (intersection_from_list_acc k v leq xs2 keep acc)
    (intersection_from_list_acc_final_bridge k v leq e xs2 keep acc)
    (trans (Tree k v)
      (intersection_from_list_acc_step k v leq e xs2 keep acc (member k v leq (pair_fst k v e) keep))
      (intersection_from_list_acc_step k v leq e xs2 keep acc False)
      (intersection_from_list_acc k v leq xs2 keep acc)
      (intersection_from_list_acc_step_false_eq k v leq e xs2 keep acc q)
      (intersection_from_list_acc_step_false_reduces k v leq e xs2 keep acc))

fn intersection (k : Type) (v : Type) (leq : k → k → Bool) (a : Tree k v) (b : Tree k v) : Tree k v =
  intersection_from_list_acc k v leq (to_list k v a) b (empty k v)

fn difference_from_list_acc
  (k : Type) (v : Type) (leq : k → k → Bool)
  (xs : List (Pair k v)) (reject : Tree k v) (acc : Tree k v) : Tree k v =
  match xs {
    Nil ⇒ acc ;
    Cons e xs2 ⇒
      match member k v leq (pair_fst k v e) reject {
        True ⇒ difference_from_list_acc k v leq xs2 reject acc ;
        False ⇒ difference_from_list_acc k v leq xs2 reject
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
      }
  }

fn difference_from_list_acc_step
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v) (m : Bool) : Tree k v =
  match m {
    True ⇒ difference_from_list_acc k v leq xs2 reject acc ;
    False ⇒ difference_from_list_acc k v leq xs2 reject
      (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
  }

fn difference_from_list_acc_final_bridge
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  : Equal (Tree k v)
      (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
      (difference_from_list_acc_step k v leq e xs2 reject acc (member k v leq (pair_fst k v e) reject)) = Refl

fn difference_from_list_acc_step_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (q : Equal Bool (member k v leq (pair_fst k v e) reject) True)
  : Equal (Tree k v)
      (difference_from_list_acc_step k v leq e xs2 reject acc (member k v leq (pair_fst k v e) reject))
      (difference_from_list_acc_step k v leq e xs2 reject acc True) =
  cong Bool (Tree k v) (member k v leq (pair_fst k v e) reject) True
    (difference_from_list_acc_step k v leq e xs2 reject acc) q

fn difference_from_list_acc_step_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (q : Equal Bool (member k v leq (pair_fst k v e) reject) False)
  : Equal (Tree k v)
      (difference_from_list_acc_step k v leq e xs2 reject acc (member k v leq (pair_fst k v e) reject))
      (difference_from_list_acc_step k v leq e xs2 reject acc False) =
  cong Bool (Tree k v) (member k v leq (pair_fst k v e) reject) False
    (difference_from_list_acc_step k v leq e xs2 reject acc) q

fn difference_from_list_acc_step_true_reduces
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  : Equal (Tree k v)
      (difference_from_list_acc_step k v leq e xs2 reject acc True)
      (difference_from_list_acc k v leq xs2 reject acc) = Refl

fn difference_from_list_acc_step_false_reduces
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  : Equal (Tree k v)
      (difference_from_list_acc_step k v leq e xs2 reject acc False)
      (difference_from_list_acc k v leq xs2 reject
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)) = Refl

fn difference_from_list_acc_true_bridge
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (q : Equal Bool (member k v leq (pair_fst k v e) reject) True)
  : Equal (Tree k v)
      (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
      (difference_from_list_acc k v leq xs2 reject acc) =
  trans (Tree k v)
    (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
    (difference_from_list_acc_step k v leq e xs2 reject acc (member k v leq (pair_fst k v e) reject))
    (difference_from_list_acc k v leq xs2 reject acc)
    (difference_from_list_acc_final_bridge k v leq e xs2 reject acc)
    (trans (Tree k v)
      (difference_from_list_acc_step k v leq e xs2 reject acc (member k v leq (pair_fst k v e) reject))
      (difference_from_list_acc_step k v leq e xs2 reject acc True)
      (difference_from_list_acc k v leq xs2 reject acc)
      (difference_from_list_acc_step_true_eq k v leq e xs2 reject acc q)
      (difference_from_list_acc_step_true_reduces k v leq e xs2 reject acc))

fn difference_from_list_acc_false_bridge
  (k : Type) (v : Type) (leq : k → k → Bool)
  (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (q : Equal Bool (member k v leq (pair_fst k v e) reject) False)
  : Equal (Tree k v)
      (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
      (difference_from_list_acc k v leq xs2 reject
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)) =
  trans (Tree k v)
    (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
    (difference_from_list_acc_step k v leq e xs2 reject acc (member k v leq (pair_fst k v e) reject))
    (difference_from_list_acc k v leq xs2 reject
      (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
    (difference_from_list_acc_final_bridge k v leq e xs2 reject acc)
    (trans (Tree k v)
      (difference_from_list_acc_step k v leq e xs2 reject acc (member k v leq (pair_fst k v e) reject))
      (difference_from_list_acc_step k v leq e xs2 reject acc False)
      (difference_from_list_acc k v leq xs2 reject
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
      (difference_from_list_acc_step_false_eq k v leq e xs2 reject acc q)
      (difference_from_list_acc_step_false_reduces k v leq e xs2 reject acc))

fn difference (k : Type) (v : Type) (leq : k → k → Bool) (a : Tree k v) (b : Tree k v) : Tree k v =
  difference_from_list_acc k v leq (to_list k v a) b (empty k v)
```

#### 4.7.5 Fold-based `insert`/`insert_with` — Ordered-preservation

`fold_insert_preserves_ordered` establishes Ordered-preservation for a
generic `insert`-fold (used by `from_list`/`union`'s reconstruction step).
`insert_with_preserves_ordered` is the `insert_with` analogue of Law 1
(§4.3): the same `insert_with_step`/`_step_inner` stuck-match staging, the
same `insert_with_transport_overwrite`/`_into_l`/`_into_r` stop-one-step-short
bridges, and the same two-level `Or`-elimination dispatch, adapted for
`insert_with`'s combining-function overwrite case
(`replace_left_ordered_witness`/`replace_right_ordered_witness`).

```ken
fn insert_fold_step (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (val : v) (acc : Tree k v) : Tree k v =
  insert k v leq key val acc

fn fold_insert_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (src : Tree k v) (acc : Tree k v)
  : Ordered k v leq acc → Ordered k v leq (fold k v (Tree k v) (insert_fold_step k v leq) acc src) =
  match src {
    Leaf ⇒ λh. h ;
    Node l key val r ⇒
      λh.
        fold_insert_preserves_ordered k v leq transLeq total r
          (insert k v leq key val (fold k v (Tree k v) (insert_fold_step k v leq) acc l))
          (preserves_ordered k v leq transLeq total key val
            (fold k v (Tree k v) (insert_fold_step k v leq) acc l)
            (fold_insert_preserves_ordered k v leq transLeq total l acc h))
  }

fn insert_with_step_inner
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v) (y : Bool) : Tree k v =
  match y {
    True ⇒ Node k v l key (f val v2) r ;
    False ⇒ Node k v (insert_with k v leq f key val l) k2 v2 r
  }

fn insert_with_step
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v) (x : Bool) : Tree k v =
  match x {
    True ⇒ insert_with_step_inner k v leq f key val l k2 v2 r (leq k2 key) ;
    False ⇒ Node k v l k2 v2 (insert_with k v leq f key val r)
  }

fn insert_with_inner_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q2 : Equal Bool (leq k2 key) False)
  : Equal (Tree k v) (insert_with_step_inner k v leq f key val l k2 v2 r (leq k2 key))
                      (insert_with_step_inner k v leq f key val l k2 v2 r False) =
  cong Bool (Tree k v) (leq k2 key) False (insert_with_step_inner k v leq f key val l k2 v2 r) q2

fn insert_with_inner_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q2 : Equal Bool (leq k2 key) True)
  : Equal (Tree k v) (insert_with_step_inner k v leq f key val l k2 v2 r (leq k2 key))
                      (insert_with_step_inner k v leq f key val l k2 v2 r True) =
  cong Bool (Tree k v) (leq k2 key) True (insert_with_step_inner k v leq f key val l k2 v2 r) q2

fn insert_with_step_true_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True)
  : Equal (Tree k v) (insert_with_step k v leq f key val l k2 v2 r (leq key k2))
                      (insert_with_step k v leq f key val l k2 v2 r True) =
  cong Bool (Tree k v) (leq key k2) True (insert_with_step k v leq f key val l k2 v2 r) q1

fn insert_with_step_false_eq
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) False)
  : Equal (Tree k v) (insert_with_step k v leq f key val l k2 v2 r (leq key k2))
                      (insert_with_step k v leq f key val l k2 v2 r False) =
  cong Bool (Tree k v) (leq key k2) False (insert_with_step k v leq f key val l k2 v2 r) q1

fn insert_with_final_bridge
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  : Equal (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                      (insert_with_step k v leq f key val l k2 v2 r (leq key k2)) = Refl

fn insert_with_step_true_reduces
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  : Equal (Tree k v) (insert_with_step k v leq f key val l k2 v2 r True)
                      (insert_with_step_inner k v leq f key val l k2 v2 r (leq k2 key)) = Refl

fn insert_with_step_a
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True)
  : Equal (Tree k v) (insert_with_step k v leq f key val l k2 v2 r (leq key k2))
                      (insert_with_step_inner k v leq f key val l k2 v2 r (leq k2 key)) =
  trans (Tree k v) (insert_with_step k v leq f key val l k2 v2 r (leq key k2))
                    (insert_with_step k v leq f key val l k2 v2 r True)
                    (insert_with_step_inner k v leq f key val l k2 v2 r (leq k2 key))
    (insert_with_step_true_eq k v leq f key val l k2 v2 r q1)
    (insert_with_step_true_reduces k v leq f key val l k2 v2 r)

fn insert_with_step_b
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True)
  : Equal (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                      (insert_with_step_inner k v leq f key val l k2 v2 r (leq k2 key)) =
  trans (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                    (insert_with_step k v leq f key val l k2 v2 r (leq key k2))
                    (insert_with_step_inner k v leq f key val l k2 v2 r (leq k2 key))
    (insert_with_final_bridge k v leq f key val l k2 v2 r)
    (insert_with_step_a k v leq f key val l k2 v2 r q1)

fn insert_with_step_c
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) False)
  : Equal (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                      (insert_with_step_inner k v leq f key val l k2 v2 r False) =
  trans (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                    (insert_with_step_inner k v leq f key val l k2 v2 r (leq k2 key))
                    (insert_with_step_inner k v leq f key val l k2 v2 r False)
    (insert_with_step_b k v leq f key val l k2 v2 r q1)
    (insert_with_inner_false_eq k v leq f key val l k2 v2 r q2)

fn insert_with_step_overwrite
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  : Equal (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                      (insert_with_step_inner k v leq f key val l k2 v2 r True) =
  trans (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                    (insert_with_step_inner k v leq f key val l k2 v2 r (leq k2 key))
                    (insert_with_step_inner k v leq f key val l k2 v2 r True)
    (insert_with_step_b k v leq f key val l k2 v2 r q1)
    (insert_with_inner_true_eq k v leq f key val l k2 v2 r q2)

fn insert_with_transport_overwrite
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (goal : Tree k v → Prop)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  (overwrite : goal (Node k v l key (f val v2) r))
  : goal (insert_with k v leq f key val (Node k v l k2 v2 r)) =
  J (λx _. goal x) overwrite
    (sym (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                     (insert_with_step_inner k v leq f key val l k2 v2 r True)
       (insert_with_step_overwrite k v leq f key val l k2 v2 r q1 q2))

fn insert_with_transport_into_l
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (goal : Tree k v → Prop)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) False)
  (intoL : goal (Node k v (insert_with k v leq f key val l) k2 v2 r))
  : goal (insert_with k v leq f key val (Node k v l k2 v2 r)) =
  J (λx _. goal x) intoL
    (sym (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                     (insert_with_step_inner k v leq f key val l k2 v2 r False)
       (insert_with_step_c k v leq f key val l k2 v2 r q1 q2))

fn insert_with_step_d
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) False)
  : Equal (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                      (insert_with_step k v leq f key val l k2 v2 r False) =
  trans (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                    (insert_with_step k v leq f key val l k2 v2 r (leq key k2))
                    (insert_with_step k v leq f key val l k2 v2 r False)
    (insert_with_final_bridge k v leq f key val l k2 v2 r)
    (insert_with_step_false_eq k v leq f key val l k2 v2 r q1)

fn insert_with_transport_into_r
  (k : Type) (v : Type) (leq : k → k → Bool) (f : v → v → v) (key : k) (val : v)
  (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (goal : Tree k v → Prop)
  (q1 : Equal Bool (leq key k2) False)
  (intoR : goal (Node k v l k2 v2 (insert_with k v leq f key val r)))
  : goal (insert_with k v leq f key val (Node k v l k2 v2 r)) =
  J (λx _. goal x) intoR
    (sym (Tree k v) (insert_with k v leq f key val (Node k v l k2 v2 r))
                     (insert_with_step k v leq f key val l k2 v2 r False)
       (insert_with_step_d k v leq f key val l k2 v2 r q1))

fn replace_left_ordered_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (newL : Tree k v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (ordNewL : Ordered k v leq newL)
  (belowNewL : all_keys k v (le_below k v leq k2) newL)
  : Ordered k v leq (Node k v newL k2 v2 r) =
  and_intro (all_keys k v (le_below k v leq k2) newL)
           (And (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq newL) (Ordered k v leq r)))
    belowNewL
    (and_intro (all_keys k v (le_above k v leq k2) r) (And (Ordered k v leq newL) (Ordered k v leq r))
       (get_above_r k v leq l k2 v2 r h)
       (and_intro (Ordered k v leq newL) (Ordered k v leq r)
          ordNewL
          (get_ordered_r k v leq l k2 v2 r h)))

fn replace_right_ordered_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (newR : Tree k v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (ordNewR : Ordered k v leq newR)
  (aboveNewR : all_keys k v (le_above k v leq k2) newR)
  : Ordered k v leq (Node k v l k2 v2 newR) =
  and_intro (all_keys k v (le_below k v leq k2) l)
           (And (all_keys k v (le_above k v leq k2) newR) (And (Ordered k v leq l) (Ordered k v leq newR)))
    (get_below_l k v leq l k2 v2 r h)
    (and_intro (all_keys k v (le_above k v leq k2) newR) (And (Ordered k v leq l) (Ordered k v leq newR))
       aboveNewR
       (and_intro (Ordered k v leq l) (Ordered k v leq newR)
          (get_ordered_l k v leq l k2 v2 r h)
          ordNewR))

fn insert_with_step_inner_preserves_all_keys
  (k : Type) (v : Type) (leq : k → k → Bool) (p : k → Prop)
  (f : v → v → v) (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (insL : all_keys k v p (insert_with k v leq f key val l))
  (hl : all_keys k v p l) (hr : all_keys k v p r) (hkey : p key) (hk2 : p k2)
  (y : Bool)
  : all_keys k v p (insert_with_step_inner k v leq f key val l k2 v2 r y) =
  match y {
    True ⇒ and_intro (p key) (And (all_keys k v p l) (all_keys k v p r)) hkey (and_intro (all_keys k v p l) (all_keys k v p r) hl hr) ;
    False ⇒ and_intro (p k2) (And (all_keys k v p (insert_with k v leq f key val l)) (all_keys k v p r)) hk2 (and_intro (all_keys k v p (insert_with k v leq f key val l)) (all_keys k v p r) insL hr)
  }

fn insert_with_step_preserves_all_keys
  (k : Type) (v : Type) (leq : k → k → Bool) (p : k → Prop)
  (f : v → v → v) (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (insL : all_keys k v p (insert_with k v leq f key val l))
  (insR : all_keys k v p (insert_with k v leq f key val r))
  (hl : all_keys k v p l) (hr : all_keys k v p r) (hkey : p key) (hk2 : p k2)
  (x : Bool)
  : all_keys k v p (insert_with_step k v leq f key val l k2 v2 r x) =
  match x {
	    True ⇒ insert_with_step_inner_preserves_all_keys k v leq p f key val l k2 v2 r insL hl hr hkey hk2 (leq k2 key) ;
	    False ⇒ and_intro (p k2) (And (all_keys k v p l) (all_keys k v p (insert_with k v leq f key val r))) hk2 (and_intro (all_keys k v p l) (all_keys k v p (insert_with k v leq f key val r)) hl insR)
	  }

fn insert_with_preserves_all_keys_node
  (k : Type) (v : Type) (leq : k → k → Bool) (p : k → Prop)
  (f : v → v → v) (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (insL : all_keys k v p (insert_with k v leq f key val l))
  (insR : all_keys k v p (insert_with k v leq f key val r))
  (hl : all_keys k v p l) (hr : all_keys k v p r) (hkey : p key) (hk2 : p k2)
  : all_keys k v p (insert_with k v leq f key val (Node k v l k2 v2 r)) =
  insert_with_step_preserves_all_keys k v leq p f key val l k2 v2 r insL insR hl hr hkey hk2 (leq key k2)

fn insert_with_preserves_all_keys
  (k : Type) (v : Type) (leq : k → k → Bool) (p : k → Prop)
  (f : v → v → v) (key : k) (val : v) (m : Tree k v)
  : all_keys k v p m → p key → all_keys k v p (insert_with k v leq f key val m) =
  match m {
	    Leaf ⇒ λh. λhkey.
	      and_intro (p key) (And (all_keys k v p (Leaf k v)) (all_keys k v p (Leaf k v))) hkey
	        (and_intro (all_keys k v p (Leaf k v)) (all_keys k v p (Leaf k v)) tt tt) ;
	    Node l k2 v2 r ⇒ λh. λhkey.
	      insert_with_preserves_all_keys_node k v leq p f key val l k2 v2 r
	        (insert_with_preserves_all_keys k v leq p f key val l
	           (and_fst (all_keys k v p l) (all_keys k v p r) (and_snd (p k2) (And (all_keys k v p l) (all_keys k v p r)) h)) hkey)
	        (insert_with_preserves_all_keys k v leq p f key val r
           (and_snd (all_keys k v p l) (all_keys k v p r) (and_snd (p k2) (And (all_keys k v p l) (all_keys k v p r)) h)) hkey)
        (and_fst (all_keys k v p l) (all_keys k v p r) (and_snd (p k2) (And (all_keys k v p l) (all_keys k v p r)) h))
        (and_snd (all_keys k v p l) (all_keys k v p r) (and_snd (p k2) (And (all_keys k v p l) (all_keys k v p r)) h))
        hkey
        (and_fst (p k2) (And (all_keys k v p l) (all_keys k v p r)) h)
  }

fn insert_with_dispatch_on_q2
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (f : v → v → v) (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (insL : Ordered k v leq (insert_with k v leq f key val l))
  (insR : Ordered k v leq (insert_with k v leq f key val r))
  (q1 : Equal Bool (leq key k2) True)
  (o2 : Or (Equal Bool (leq k2 key) True) (Equal Bool (leq k2 key) False))
  : Ordered k v leq (insert_with k v leq f key val (Node k v l k2 v2 r)) =
  match o2 {
    Inl q2 ⇒
      insert_with_transport_overwrite k v leq f key val l k2 v2 r (λx. Ordered k v leq x) q1 q2
        (preserves_ordered_overwrite_witness k v leq transLeq key (f val v2) l k2 v2 r h q1 q2) ;
    Inr q2' ⇒
      insert_with_transport_into_l k v leq f key val l k2 v2 r (λx. Ordered k v leq x) q1 q2'
        (replace_left_ordered_witness k v leq (insert_with k v leq f key val l) l k2 v2 r h insL
           (insert_with_preserves_all_keys k v leq (le_below k v leq k2) f key val l (get_below_l k v leq l k2 v2 r h) q1))
  }

fn insert_with_dispatch_on_q1
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (f : v → v → v) (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (h : Ordered k v leq (Node k v l k2 v2 r))
  (insL : Ordered k v leq (insert_with k v leq f key val l))
  (insR : Ordered k v leq (insert_with k v leq f key val r))
  (o1 : Or (Equal Bool (leq key k2) True) (Equal Bool (leq key k2) False))
  : Ordered k v leq (insert_with k v leq f key val (Node k v l k2 v2 r)) =
  match o1 {
    Inl q1 ⇒ insert_with_dispatch_on_q2 k v leq transLeq total f key val l k2 v2 r h insL insR q1 (bool_dichotomy (leq k2 key)) ;
    Inr q1' ⇒
      insert_with_transport_into_r k v leq f key val l k2 v2 r (λx. Ordered k v leq x) q1'
        (replace_right_ordered_witness k v leq (insert_with k v leq f key val r) l k2 v2 r h insR
           (insert_with_preserves_all_keys k v leq (le_above k v leq k2) f key val r (get_above_r k v leq l k2 v2 r h)
              (derive_from_false k leq key k2 q1' total)))
  }

fn insert_with_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (f : v → v → v) (key : k) (val : v) (m : Tree k v)
  : Ordered k v leq m → Ordered k v leq (insert_with k v leq f key val m) =
  match m {
    Leaf ⇒ λh.
      and_intro (all_keys k v (le_below k v leq key) (Leaf k v))
               (And (all_keys k v (le_above k v leq key) (Leaf k v)) (And (Ordered k v leq (Leaf k v)) (Ordered k v leq (Leaf k v))))
        tt (and_intro (all_keys k v (le_above k v leq key) (Leaf k v)) (And (Ordered k v leq (Leaf k v)) (Ordered k v leq (Leaf k v)))
             tt (and_intro (Ordered k v leq (Leaf k v)) (Ordered k v leq (Leaf k v)) tt tt)) ;
    Node l k2 v2 r ⇒
      λh.
        insert_with_dispatch_on_q1 k v leq transLeq total f key val l k2 v2 r h
          (insert_with_preserves_ordered k v leq transLeq total f key val l (get_ordered_l k v leq l k2 v2 r h))
          (insert_with_preserves_ordered k v leq transLeq total f key val r (get_ordered_r k v leq l k2 v2 r h))
	          (bool_dichotomy (leq key k2))
	  }
```

#### 4.7.6 `insert_with` — lookup characterization and locality

`insert_with_lookup_characterization` states what `lookup key (insert_with
combine key val m)` evaluates to (the combined value if `key` was already
present, `val` otherwise), proved via the same overwrite/into-L/into-R
transport-witness triad as the Ordered-preservation proof above.
`lookup_replace_l_inner_dispatch`/`_witness` and their right-hand mirrors
are generic "replace a subtree, lookup elsewhere is unaffected" lemmas,
reused across several of this section's locality proofs.
`insert_with_lookup_locality` shows `insert_with` at a distinct key doesn't
disturb `lookup` at any other key (the `insert_with` analogue of Law 3,
§4.5), and `insert_with_fold_step_lookup_locality`/`_lookup_hit` lift that
to the fold step `union`/`from_list` reuse.

```ken
fn insert_with_lookup_overwrite_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (f : v → v → v) (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) True)
  : Equal (Option v)
      (lookup k v leq key (Node k v l key (f val v2) r))
      (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r)) =
  trans (Option v)
    (lookup k v leq key (Node k v l key (f val v2) r))
    (Some v (f val v2))
    (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r))
    (lookup_overwrite_result k v leq key (f val v2) l r (reflLeq key))
    (sym (Option v)
      (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r))
      (Some v (f val v2))
      (cong (Option v) (Option v)
        (lookup k v leq key (Node k v l k2 v2 r))
        (Some v v2)
        (insert_with_lookup_result v f val)
        (lookup_stop_bridge k v leq key l k2 v2 r q1 q2)))

fn insert_with_lookup_into_l_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (f : v → v → v) (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ihL : Equal (Option v)
           (lookup k v leq key (insert_with k v leq f key val l))
           (insert_with_lookup_result_for k v leq f key val l))
  (q1 : Equal Bool (leq key k2) True) (q2 : Equal Bool (leq k2 key) False)
  : Equal (Option v)
      (lookup k v leq key (Node k v (insert_with k v leq f key val l) k2 v2 r))
      (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r)) =
  trans (Option v)
    (lookup k v leq key (Node k v (insert_with k v leq f key val l) k2 v2 r))
    (lookup k v leq key (insert_with k v leq f key val l))
    (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r))
    (lookup_into_l_bridge k v leq key (insert_with k v leq f key val l) k2 v2 r q1 q2)
    (trans (Option v)
      (lookup k v leq key (insert_with k v leq f key val l))
      (insert_with_lookup_result_for k v leq f key val l)
      (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r))
      ihL
      (cong (Option v) (Option v)
        (lookup k v leq key l)
        (lookup k v leq key (Node k v l k2 v2 r))
        (insert_with_lookup_result v f val)
        (sym (Option v)
          (lookup k v leq key (Node k v l k2 v2 r))
          (lookup k v leq key l)
          (lookup_into_l_bridge k v leq key l k2 v2 r q1 q2))))

fn insert_with_lookup_into_r_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (f : v → v → v) (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ihR : Equal (Option v)
           (lookup k v leq key (insert_with k v leq f key val r))
           (insert_with_lookup_result_for k v leq f key val r))
  (q1 : Equal Bool (leq key k2) False)
  : Equal (Option v)
      (lookup k v leq key (Node k v l k2 v2 (insert_with k v leq f key val r)))
      (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r)) =
  trans (Option v)
    (lookup k v leq key (Node k v l k2 v2 (insert_with k v leq f key val r)))
    (lookup k v leq key (insert_with k v leq f key val r))
    (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r))
    (lookup_into_r_bridge k v leq key l k2 v2 (insert_with k v leq f key val r) q1)
    (trans (Option v)
      (lookup k v leq key (insert_with k v leq f key val r))
      (insert_with_lookup_result_for k v leq f key val r)
      (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r))
      ihR
      (cong (Option v) (Option v)
        (lookup k v leq key r)
        (lookup k v leq key (Node k v l k2 v2 r))
        (insert_with_lookup_result v f val)
        (sym (Option v)
          (lookup k v leq key (Node k v l k2 v2 r))
          (lookup k v leq key r)
          (lookup_into_r_bridge k v leq key l k2 v2 r q1))))

fn insert_with_lookup_dispatch_q2
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (f : v → v → v) (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ihL : Equal (Option v)
           (lookup k v leq key (insert_with k v leq f key val l))
           (insert_with_lookup_result_for k v leq f key val l))
  (q1 : Equal Bool (leq key k2) True)
  (o2 : Or (Equal Bool (leq k2 key) True) (Equal Bool (leq k2 key) False))
  : Equal (Option v)
      (lookup k v leq key (insert_with k v leq f key val (Node k v l k2 v2 r)))
      (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r)) =
  match o2 {
    Inl q2 ⇒
      insert_with_transport_overwrite k v leq f key val l k2 v2 r
        (λx. Equal (Option v) (lookup k v leq key x)
              (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r)))
        q1 q2
        (insert_with_lookup_overwrite_witness k v leq reflLeq f key val l k2 v2 r q1 q2) ;
    Inr q2' ⇒
      insert_with_transport_into_l k v leq f key val l k2 v2 r
        (λx. Equal (Option v) (lookup k v leq key x)
              (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r)))
        q1 q2'
        (insert_with_lookup_into_l_witness k v leq f key val l k2 v2 r ihL q1 q2')
  }

fn insert_with_lookup_dispatch_q1
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (f : v → v → v) (key : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ihL : Equal (Option v)
           (lookup k v leq key (insert_with k v leq f key val l))
           (insert_with_lookup_result_for k v leq f key val l))
  (ihR : Equal (Option v)
           (lookup k v leq key (insert_with k v leq f key val r))
           (insert_with_lookup_result_for k v leq f key val r))
  (o1 : Or (Equal Bool (leq key k2) True) (Equal Bool (leq key k2) False))
  : Equal (Option v)
      (lookup k v leq key (insert_with k v leq f key val (Node k v l k2 v2 r)))
      (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r)) =
  match o1 {
    Inl q1 ⇒ insert_with_lookup_dispatch_q2 k v leq reflLeq f key val l k2 v2 r ihL q1 (bool_dichotomy (leq k2 key)) ;
    Inr q1' ⇒
      insert_with_transport_into_r k v leq f key val l k2 v2 r
        (λx. Equal (Option v) (lookup k v leq key x)
              (insert_with_lookup_result_for k v leq f key val (Node k v l k2 v2 r)))
        q1'
        (insert_with_lookup_into_r_witness k v leq f key val l k2 v2 r ihR q1')
  }

fn insert_with_lookup_characterization
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (f : v → v → v) (key : k) (val : v) (m : Tree k v)
  : Equal (Option v)
      (lookup k v leq key (insert_with k v leq f key val m))
      (insert_with_lookup_result_for k v leq f key val m) =
  match m {
    Leaf ⇒ lookup_overwrite_result k v leq key val (Leaf k v) (Leaf k v) (reflLeq key) ;
    Node l k2 v2 r ⇒
      insert_with_lookup_dispatch_q1 k v leq reflLeq f key val l k2 v2 r
        (insert_with_lookup_characterization k v leq reflLeq f key val l)
        (insert_with_lookup_characterization k v leq reflLeq f key val r)
        (bool_dichotomy (leq key k2))
  }

fn lookup_replace_l_inner_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (key' : k) (newL : Tree k v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' newL) (lookup k v leq key' l))
  (hOuter : Equal Bool (leq key' k2) True)
  (oInner : Or (Equal Bool (leq k2 key') True) (Equal Bool (leq k2 key') False))
  : Equal (Option v) (lookup k v leq key' (Node k v newL k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oInner {
    Inl hInner ⇒
      trans (Option v) (lookup k v leq key' (Node k v newL k2 v2 r)) (Some v v2) (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_stop_result k v leq key' newL k2 v2 r hOuter hInner)
        (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (Some v v2) (lookup_stop_result k v leq key' l k2 v2 r hOuter hInner)) ;
    Inr hInnerFalse ⇒
      trans (Option v) (lookup k v leq key' (Node k v newL k2 v2 r))
                        (lookup k v leq key' newL)
                        (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_into_l_bridge k v leq key' newL k2 v2 r hOuter hInnerFalse)
        (trans (Option v) (lookup k v leq key' newL)
                           (lookup k v leq key' l)
                           (lookup k v leq key' (Node k v l k2 v2 r))
          ih
          (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' l)
             (lookup_into_l_bridge k v leq key' l k2 v2 r hOuter hInnerFalse)))
  }

fn lookup_replace_l_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (key' : k) (newL : Tree k v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' newL) (lookup k v leq key' l))
  (oOuter : Or (Equal Bool (leq key' k2) True) (Equal Bool (leq key' k2) False))
  : Equal (Option v) (lookup k v leq key' (Node k v newL k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oOuter {
    Inl hOuter ⇒ lookup_replace_l_inner_dispatch k v leq key' newL l k2 v2 r ih hOuter (bool_dichotomy (leq k2 key')) ;
    Inr hOuterFalse ⇒
      trans (Option v) (lookup k v leq key' (Node k v newL k2 v2 r))
                        (lookup k v leq key' r)
                        (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_into_r_bridge k v leq key' newL k2 v2 r hOuterFalse)
        (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' r)
           (lookup_into_r_bridge k v leq key' l k2 v2 r hOuterFalse))
  }

fn lookup_replace_l_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (key' : k) (newL : Tree k v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' newL) (lookup k v leq key' l))
  : Equal (Option v) (lookup k v leq key' (Node k v newL k2 v2 r))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  lookup_replace_l_dispatch k v leq key' newL l k2 v2 r ih (bool_dichotomy (leq key' k2))

fn lookup_replace_r_inner_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (key' : k) (newR : Tree k v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' newR) (lookup k v leq key' r))
  (hOuter : Equal Bool (leq key' k2) True)
  (oInner : Or (Equal Bool (leq k2 key') True) (Equal Bool (leq k2 key') False))
  : Equal (Option v) (lookup k v leq key' (Node k v l k2 v2 newR))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oInner {
    Inl hInner ⇒
      trans (Option v) (lookup k v leq key' (Node k v l k2 v2 newR)) (Some v v2) (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_stop_result k v leq key' l k2 v2 newR hOuter hInner)
        (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (Some v v2) (lookup_stop_result k v leq key' l k2 v2 r hOuter hInner)) ;
    Inr hInnerFalse ⇒
      trans (Option v) (lookup k v leq key' (Node k v l k2 v2 newR))
                        (lookup k v leq key' l)
                        (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_into_l_bridge k v leq key' l k2 v2 newR hOuter hInnerFalse)
        (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' l)
           (lookup_into_l_bridge k v leq key' l k2 v2 r hOuter hInnerFalse))
  }

fn lookup_replace_r_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (key' : k) (newR : Tree k v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' newR) (lookup k v leq key' r))
  (oOuter : Or (Equal Bool (leq key' k2) True) (Equal Bool (leq key' k2) False))
  : Equal (Option v) (lookup k v leq key' (Node k v l k2 v2 newR))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match oOuter {
    Inl hOuter ⇒ lookup_replace_r_inner_dispatch k v leq key' newR l k2 v2 r ih hOuter (bool_dichotomy (leq k2 key')) ;
    Inr hOuterFalse ⇒
      trans (Option v) (lookup k v leq key' (Node k v l k2 v2 newR))
                        (lookup k v leq key' newR)
                        (lookup k v leq key' (Node k v l k2 v2 r))
        (lookup_into_r_bridge k v leq key' l k2 v2 newR hOuterFalse)
        (trans (Option v) (lookup k v leq key' newR)
                           (lookup k v leq key' r)
                           (lookup k v leq key' (Node k v l k2 v2 r))
          ih
          (sym (Option v) (lookup k v leq key' (Node k v l k2 v2 r)) (lookup k v leq key' r)
             (lookup_into_r_bridge k v leq key' l k2 v2 r hOuterFalse)))
  }

fn lookup_replace_r_witness
  (k : Type) (v : Type) (leq : k → k → Bool)
  (key' : k) (newR : Tree k v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v)
  (ih : Equal (Option v) (lookup k v leq key' newR) (lookup k v leq key' r))
  : Equal (Option v) (lookup k v leq key' (Node k v l k2 v2 newR))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  lookup_replace_r_dispatch k v leq key' newR l k2 v2 r ih (bool_dichotomy (leq key' k2))

fn insert_with_lookup_locality_q2_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v) (f : v → v → v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  (ihL : Equal (Option v) (lookup k v leq key' (insert_with k v leq f key val l)) (lookup k v leq key' l))
  (ihR : Equal (Option v) (lookup k v leq key' (insert_with k v leq f key val r)) (lookup k v leq key' r))
  (q1 : Equal Bool (leq key k2) True)
  (o2 : Or (Equal Bool (leq k2 key) True) (Equal Bool (leq k2 key) False))
  : Equal (Option v) (lookup k v leq key' (insert_with k v leq f key val (Node k v l k2 v2 r)))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match o2 {
    Inl q2 ⇒
      insert_with_transport_overwrite k v leq f key val l k2 v2 r
        (λx. Equal (Option v) (lookup k v leq key' x) (lookup k v leq key' (Node k v l k2 v2 r)))
        q1 q2
        (lookup_overwrite_locality_witness k v leq transLeq key key' (f val v2) k2 v2 l r q1 q2 hdist) ;
    Inr q2' ⇒
      insert_with_transport_into_l k v leq f key val l k2 v2 r
        (λx. Equal (Option v) (lookup k v leq key' x) (lookup k v leq key' (Node k v l k2 v2 r)))
        q1 q2'
        (lookup_replace_l_witness k v leq key' (insert_with k v leq f key val l) l k2 v2 r ihL)
  }

fn insert_with_lookup_locality_node_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (key' : k) (val : v) (l : Tree k v) (k2 : k) (v2 : v) (r : Tree k v) (f : v → v → v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  (ihL : Equal (Option v) (lookup k v leq key' (insert_with k v leq f key val l)) (lookup k v leq key' l))
  (ihR : Equal (Option v) (lookup k v leq key' (insert_with k v leq f key val r)) (lookup k v leq key' r))
  (o1 : Or (Equal Bool (leq key k2) True) (Equal Bool (leq key k2) False))
  : Equal (Option v) (lookup k v leq key' (insert_with k v leq f key val (Node k v l k2 v2 r)))
                      (lookup k v leq key' (Node k v l k2 v2 r)) =
  match o1 {
    Inl q1 ⇒ insert_with_lookup_locality_q2_dispatch k v leq transLeq key key' val l k2 v2 r f hdist ihL ihR q1 (bool_dichotomy (leq k2 key)) ;
    Inr q1' ⇒
      insert_with_transport_into_r k v leq f key val l k2 v2 r
        (λx. Equal (Option v) (lookup k v leq key' x) (lookup k v leq key' (Node k v l k2 v2 r)))
        q1'
        (lookup_replace_r_witness k v leq key' (insert_with k v leq f key val r) l k2 v2 r ihR)
  }

fn insert_with_lookup_locality
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (key : k) (key' : k) (val : v) (m : Tree k v)
  (hdist : And (Equal Bool (leq key key') True) (Equal Bool (leq key' key) True) → Bottom)
  : Equal (Option v) (lookup k v leq key' (insert_with k v leq f key val m)) (lookup k v leq key' m) =
  match m {
    Leaf ⇒ lookup_leaf_locality_witness k v leq key key' val hdist ;
    Node l k2 v2 r ⇒
      insert_with_lookup_locality_node_dispatch k v leq transLeq key key' val l k2 v2 r f hdist
        (insert_with_lookup_locality k v leq transLeq f key key' val l hdist)
        (insert_with_lookup_locality k v leq transLeq f key key' val r hdist)
        (bool_dichotomy (leq key k2))
  }

fn insert_with_fold_step_lookup_locality
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (inserted : k) (key : k) (val : v) (acc : Tree k v)
  (hdist : not_order_equiv_to_key k leq inserted key)
  : Equal (Option v)
      (lookup k v leq key (insert_with_fold_step k v leq f inserted val acc))
      (lookup k v leq key acc) =
  J (λt _. Equal (Option v) (lookup k v leq key t) (lookup k v leq key acc))
    (insert_with_lookup_locality k v leq transLeq f inserted key val acc hdist)
    (sym (Tree k v)
      (insert_with_fold_step k v leq f inserted val acc)
      (insert_with k v leq f inserted val acc)
      (insert_with_fold_step_reduces k v leq f inserted val acc))

fn insert_with_fold_step_lookup_hit
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (inserted : k) (query : k) (val : v) (acc : Tree k v)
  (heq : order_equiv k leq query inserted)
  : Equal (Option v)
      (lookup k v leq query (insert_with_fold_step k v leq f inserted val acc))
      (insert_with_lookup_result v f val (lookup k v leq query acc)) =
  J (λt _. Equal (Option v)
      (lookup k v leq query t)
      (insert_with_lookup_result v f val (lookup k v leq query acc)))
    (trans (Option v)
      (lookup k v leq query (insert_with k v leq f inserted val acc))
      (lookup k v leq inserted (insert_with k v leq f inserted val acc))
      (insert_with_lookup_result v f val (lookup k v leq query acc))
      (lookup_order_equiv_agree k v leq transLeq query inserted
        (insert_with k v leq f inserted val acc) heq)
      (trans (Option v)
        (lookup k v leq inserted (insert_with k v leq f inserted val acc))
        (insert_with_lookup_result v f val (lookup k v leq inserted acc))
        (insert_with_lookup_result v f val (lookup k v leq query acc))
        (insert_with_lookup_characterization k v leq reflLeq f inserted val acc)
        (sym (Option v)
          (insert_with_lookup_result v f val (lookup k v leq query acc))
          (insert_with_lookup_result v f val (lookup k v leq inserted acc))
          (cong (Option v) (Option v)
            (lookup k v leq query acc)
            (lookup k v leq inserted acc)
            (insert_with_lookup_result v f val)
            (lookup_order_equiv_agree k v leq transLeq query inserted acc heq)))))
    (sym (Tree k v)
      (insert_with_fold_step k v leq f inserted val acc)
      (insert_with k v leq f inserted val acc)
      (insert_with_fold_step_reduces k v leq f inserted val acc))
```

#### 4.7.7 `union` — lookup characterization and member/lookup bridges

`union_lookup_characterization` states `union`'s expected lookup result in
terms of the two input trees' own lookups, with four pointwise corollaries
(`union_lookup_both_none_law`/`_left_only_law`/`_right_only_law`/
`_both_some_law`) covering each combination of hit/miss on the two sides.
`member_from_lookup_none`/`_some` and `lookup_none_from_member_false_*`/
`lookup_unit_some_from_member_true_*` are small bridging helpers converting
between `Bool`-valued `member` facts and `Option`-valued `lookup` facts,
needed to state the `Set`-level laws in §4.7.11 in terms of `member` while
proving them via `lookup`.

```ken
fn union_from_list_acc_lookup_assoc_hit
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (q1 : Equal Bool (leq query (pair_fst k v e)) True)
  (q2 : Equal Bool (leq (pair_fst k v e) query) True)
  (tailSkip : all_in_list k v (not_order_equiv_to_key k leq query) xs2)
  (ih : Equal (Option v)
          (lookup k v leq query
            (union_from_list_acc k v leq f xs2
              (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
          (union_lookup_table v f
            (assoc k v leq query xs2)
            (lookup k v leq query
              (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc))))
  : Equal (Option v)
      (lookup k v leq query (union_from_list_acc k v leq f (Cons (Pair k v) e xs2) acc))
      (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc)) =
  trans (Option v)
    (lookup k v leq query (union_from_list_acc k v leq f (Cons (Pair k v) e xs2) acc))
    (lookup k v leq query
      (union_from_list_acc k v leq f xs2
        (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
    (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc))
    (cong (Tree k v) (Option v)
      (union_from_list_acc k v leq f (Cons (Pair k v) e xs2) acc)
      (union_from_list_acc k v leq f xs2
        (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc))
      (lookup k v leq query)
      (union_from_list_acc_cons_bridge k v leq f e xs2 acc))
    (trans (Option v)
      (lookup k v leq query
        (union_from_list_acc k v leq f xs2
          (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
      (union_lookup_table v f
        (assoc k v leq query xs2)
        (lookup k v leq query
          (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
      (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc))
      ih
      (trans (Option v)
        (union_lookup_table v f
          (assoc k v leq query xs2)
          (lookup k v leq query
            (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
        (lookup k v leq query
          (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc))
        (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc))
        (trans (Option v)
          (union_lookup_table v f
            (assoc k v leq query xs2)
            (lookup k v leq query
              (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
          (union_lookup_table v f
            (None v)
            (lookup k v leq query
              (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
          (lookup k v leq query
            (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc))
          (cong (Option v) (Option v)
            (assoc k v leq query xs2)
            (None v)
            (λleft. union_lookup_table v f left
              (lookup k v leq query
                (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
            (assoc_no_match_is_none k v leq query xs2 tailSkip))
          Refl)
        (trans (Option v)
          (lookup k v leq query
            (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc))
          (insert_with_lookup_result v f (pair_snd k v e) (lookup k v leq query acc))
          (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc))
          (insert_with_fold_step_lookup_hit k v leq reflLeq transLeq f
            (pair_fst k v e) query (pair_snd k v e) acc
            (and_intro (Equal Bool (leq query (pair_fst k v e)) True)
              (Equal Bool (leq (pair_fst k v e) query) True) q1 q2))
          (sym (Option v)
            (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc))
            (insert_with_lookup_result v f (pair_snd k v e) (lookup k v leq query acc))
            (cong (Option v) (Option v)
              (assoc k v leq query (Cons (Pair k v) e xs2))
              (Some v (pair_snd k v e))
              (λleft. union_lookup_table v f left (lookup k v leq query acc))
              (assoc_skip_head_stop_bridge k v leq query e xs2 q1 q2))))))

fn union_from_list_acc_lookup_assoc_miss
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (hnot : not_order_equiv_to_key k leq (pair_fst k v e) query)
  (assocSkip : Equal (Option v) (assoc k v leq query (Cons (Pair k v) e xs2)) (assoc k v leq query xs2))
  (ih : Equal (Option v)
          (lookup k v leq query
            (union_from_list_acc k v leq f xs2
              (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
          (union_lookup_table v f
            (assoc k v leq query xs2)
            (lookup k v leq query
              (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc))))
  : Equal (Option v)
      (lookup k v leq query (union_from_list_acc k v leq f (Cons (Pair k v) e xs2) acc))
      (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc)) =
  trans (Option v)
    (lookup k v leq query (union_from_list_acc k v leq f (Cons (Pair k v) e xs2) acc))
    (lookup k v leq query
      (union_from_list_acc k v leq f xs2
        (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
    (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc))
    (cong (Tree k v) (Option v)
      (union_from_list_acc k v leq f (Cons (Pair k v) e xs2) acc)
      (union_from_list_acc k v leq f xs2
        (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc))
      (lookup k v leq query)
      (union_from_list_acc_cons_bridge k v leq f e xs2 acc))
    (trans (Option v)
      (lookup k v leq query
        (union_from_list_acc k v leq f xs2
          (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
      (union_lookup_table v f
        (assoc k v leq query xs2)
        (lookup k v leq query
          (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
      (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc))
      ih
      (trans (Option v)
        (union_lookup_table v f
          (assoc k v leq query xs2)
          (lookup k v leq query
            (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
        (union_lookup_table v f (assoc k v leq query xs2) (lookup k v leq query acc))
        (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc))
        (cong (Option v) (Option v)
          (lookup k v leq query
            (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq query acc)
          (λright. union_lookup_table v f (assoc k v leq query xs2) right)
          (insert_with_fold_step_lookup_locality k v leq transLeq f
            (pair_fst k v e) query (pair_snd k v e) acc hnot))
        (sym (Option v)
          (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc))
          (union_lookup_table v f (assoc k v leq query xs2) (lookup k v leq query acc))
          (cong (Option v) (Option v)
            (assoc k v leq query (Cons (Pair k v) e xs2))
            (assoc k v leq query xs2)
            (λleft. union_lookup_table v f left (lookup k v leq query acc))
            assocSkip))))

fn union_from_list_acc_lookup_assoc_inner
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (ih : Equal (Option v)
          (lookup k v leq query
            (union_from_list_acc k v leq f xs2
              (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
          (union_lookup_table v f
            (assoc k v leq query xs2)
            (lookup k v leq query
              (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc))))
  (q1 : Equal Bool (leq query (pair_fst k v e)) True)
  (o2 : Or (Equal Bool (leq (pair_fst k v e) query) True) (Equal Bool (leq (pair_fst k v e) query) False))
  : Equal (Option v)
      (lookup k v leq query (union_from_list_acc k v leq f (Cons (Pair k v) e xs2) acc))
      (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc)) =
  match o2 {
    Inl q2 ⇒
      union_from_list_acc_lookup_assoc_hit k v leq reflLeq transLeq f query e xs2 acc q1 q2
        (all_in_list_map_not_match_transfer k v leq transLeq query (pair_fst k v e) q1 q2 xs2
          (and_fst
            (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
            (NoDup k v leq xs2) h))
        ih ;
    Inr q2False ⇒
      union_from_list_acc_lookup_assoc_miss k v leq transLeq f query e xs2 acc
        (not_order_equiv_from_left_false k leq (pair_fst k v e) query q2False)
        (assoc_skip_head_bridge k v leq query e xs2 q1 q2False)
        ih
  }

fn union_from_list_acc_lookup_assoc_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (query : k) (e : Pair k v) (xs2 : List (Pair k v)) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (ih : Equal (Option v)
          (lookup k v leq query
            (union_from_list_acc k v leq f xs2
              (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)))
          (union_lookup_table v f
            (assoc k v leq query xs2)
            (lookup k v leq query
              (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc))))
  (o1 : Or (Equal Bool (leq query (pair_fst k v e)) True) (Equal Bool (leq query (pair_fst k v e)) False))
  : Equal (Option v)
      (lookup k v leq query (union_from_list_acc k v leq f (Cons (Pair k v) e xs2) acc))
      (union_lookup_table v f (assoc k v leq query (Cons (Pair k v) e xs2)) (lookup k v leq query acc)) =
  match o1 {
    Inl q1 ⇒
      union_from_list_acc_lookup_assoc_inner k v leq reflLeq transLeq f query e xs2 acc h ih q1
        (bool_dichotomy (leq (pair_fst k v e) query)) ;
    Inr q1False ⇒
      union_from_list_acc_lookup_assoc_miss k v leq transLeq f query e xs2 acc
        (not_order_equiv_from_right_false k leq (pair_fst k v e) query q1False)
        (assoc_skip_head_bridge_false k v leq query e xs2 q1False)
        ih
  }

fn union_from_list_acc_lookup_assoc
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (query : k) (xs : List (Pair k v)) (acc : Tree k v)
  : NoDup k v leq xs →
    Equal (Option v)
      (lookup k v leq query (union_from_list_acc k v leq f xs acc))
      (union_lookup_table v f (assoc k v leq query xs) (lookup k v leq query acc)) =
  match xs {
    Nil ⇒ λh. Refl ;
    Cons e xs2 ⇒
      λh.
        union_from_list_acc_lookup_assoc_dispatch k v leq reflLeq transLeq f query e xs2 acc h
          (union_from_list_acc_lookup_assoc k v leq reflLeq transLeq f query xs2
            (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)
            (and_snd
              (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
            (NoDup k v leq xs2) h))
          (bool_dichotomy (leq query (pair_fst k v e)))
  }

fn union_lookup_characterization
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (key : k) (a : Tree k v) (b : Tree k v)
  : Ordered k v leq a → Distinct k v leq a →
    Equal (Option v)
      (lookup k v leq key (union k v leq f a b))
      (union_lookup_table v f (lookup k v leq key a) (lookup k v leq key b)) =
  λhord. λhdist.
    trans (Option v)
      (lookup k v leq key (union k v leq f a b))
      (union_lookup_table v f (assoc k v leq key (to_list k v a)) (lookup k v leq key b))
      (union_lookup_table v f (lookup k v leq key a) (lookup k v leq key b))
      (union_from_list_acc_lookup_assoc k v leq reflLeq transLeq f key (to_list k v a) b hdist)
      (cong (Option v) (Option v)
        (assoc k v leq key (to_list k v a))
        (lookup k v leq key a)
        (λleft. union_lookup_table v f left (lookup k v leq key b))
        (sym (Option v)
          (lookup k v leq key a)
          (assoc k v leq key (to_list k v a))
          (lookup_assoc_agree k v leq transLeq key a hord hdist)))

fn union_lookup_both_none_law
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (key : k) (a : Tree k v) (b : Tree k v)
  : Ordered k v leq a → Distinct k v leq a →
    Equal (Option v) (lookup k v leq key a) (None v) →
    Equal (Option v) (lookup k v leq key b) (None v) →
    Equal (Option v) (lookup k v leq key (union k v leq f a b)) (None v) =
  λhord. λhdist. λha. λhb.
    trans (Option v)
      (lookup k v leq key (union k v leq f a b))
      (union_lookup_table v f (lookup k v leq key a) (lookup k v leq key b))
      (None v)
      (union_lookup_characterization k v leq reflLeq transLeq f key a b hord hdist)
      (trans (Option v)
        (union_lookup_table v f (lookup k v leq key a) (lookup k v leq key b))
        (union_lookup_table v f (None v) (lookup k v leq key b))
        (None v)
        (cong (Option v) (Option v)
          (lookup k v leq key a)
          (None v)
          (λleft. union_lookup_table v f left (lookup k v leq key b))
          ha)
        hb)

fn union_lookup_left_only_law
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (key : k) (x : v) (a : Tree k v) (b : Tree k v)
  : Ordered k v leq a → Distinct k v leq a →
    Equal (Option v) (lookup k v leq key a) (Some v x) →
    Equal (Option v) (lookup k v leq key b) (None v) →
    Equal (Option v) (lookup k v leq key (union k v leq f a b)) (Some v x) =
  λhord. λhdist. λha. λhb.
    trans (Option v)
      (lookup k v leq key (union k v leq f a b))
      (union_lookup_table v f (lookup k v leq key a) (lookup k v leq key b))
      (Some v x)
      (union_lookup_characterization k v leq reflLeq transLeq f key a b hord hdist)
      (trans (Option v)
        (union_lookup_table v f (lookup k v leq key a) (lookup k v leq key b))
        (union_lookup_table v f (Some v x) (lookup k v leq key b))
        (Some v x)
        (cong (Option v) (Option v)
          (lookup k v leq key a)
          (Some v x)
          (λleft. union_lookup_table v f left (lookup k v leq key b))
          ha)
        (cong (Option v) (Option v)
          (lookup k v leq key b)
          (None v)
          (λright. union_lookup_table v f (Some v x) right)
          hb))

fn union_lookup_right_only_law
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (key : k) (y : v) (a : Tree k v) (b : Tree k v)
  : Ordered k v leq a → Distinct k v leq a →
    Equal (Option v) (lookup k v leq key a) (None v) →
    Equal (Option v) (lookup k v leq key b) (Some v y) →
    Equal (Option v) (lookup k v leq key (union k v leq f a b)) (Some v y) =
  λhord. λhdist. λha. λhb.
    trans (Option v)
      (lookup k v leq key (union k v leq f a b))
      (union_lookup_table v f (lookup k v leq key a) (lookup k v leq key b))
      (Some v y)
      (union_lookup_characterization k v leq reflLeq transLeq f key a b hord hdist)
      (trans (Option v)
        (union_lookup_table v f (lookup k v leq key a) (lookup k v leq key b))
        (union_lookup_table v f (None v) (lookup k v leq key b))
        (Some v y)
        (cong (Option v) (Option v)
          (lookup k v leq key a)
          (None v)
          (λleft. union_lookup_table v f left (lookup k v leq key b))
          ha)
        hb)

fn union_lookup_both_some_law
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (f : v → v → v) (key : k) (x : v) (y : v) (a : Tree k v) (b : Tree k v)
  : Ordered k v leq a → Distinct k v leq a →
    Equal (Option v) (lookup k v leq key a) (Some v x) →
    Equal (Option v) (lookup k v leq key b) (Some v y) →
    Equal (Option v) (lookup k v leq key (union k v leq f a b)) (Some v (f x y)) =
  λhord. λhdist. λha. λhb.
    trans (Option v)
      (lookup k v leq key (union k v leq f a b))
      (union_lookup_table v f (lookup k v leq key a) (lookup k v leq key b))
      (Some v (f x y))
      (union_lookup_characterization k v leq reflLeq transLeq f key a b hord hdist)
      (trans (Option v)
        (union_lookup_table v f (lookup k v leq key a) (lookup k v leq key b))
        (union_lookup_table v f (Some v x) (lookup k v leq key b))
        (Some v (f x y))
        (cong (Option v) (Option v)
          (lookup k v leq key a)
          (Some v x)
          (λleft. union_lookup_table v f left (lookup k v leq key b))
          ha)
        (cong (Option v) (Option v)
          (lookup k v leq key b)
          (Some v y)
          (λright. union_lookup_table v f (Some v x) right)
          hb))

fn member_from_lookup_none
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (m : Tree k v)
  (h : Equal (Option v) (lookup k v leq key m) (None v))
  : Equal Bool (member k v leq key m) False =
  cong (Option v) Bool
    (lookup k v leq key m)
    (None v)
    (λo. match o { None ⇒ False ; Some x ⇒ True })
    h

fn member_from_lookup_some
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (m : Tree k v) (val : v)
  (h : Equal (Option v) (lookup k v leq key m) (Some v val))
  : Equal Bool (member k v leq key m) True =
  cong (Option v) Bool
    (lookup k v leq key m)
    (Some v val)
    (option_is_some v)
    h

fn lookup_none_from_member_false_hit
  (v : Type) (val : v)
  (h : Equal Bool True False)
  : Equal (Option v) (Some v val) (None v) =
  absurd h

fn lookup_none_from_member_false
  (k : Type) (v : Type) (leq : k → k → Bool) (key : k) (m : Tree k v)
  : Equal Bool (member k v leq key m) False →
    Equal (Option v) (lookup k v leq key m) (None v) =
  match m {
    Leaf ⇒ λh. tt ;
    Node l k2 v2 r ⇒
      match leq key k2 {
        True ⇒
          match leq k2 key {
            True ⇒ lookup_none_from_member_false_hit v v2 ;
            False ⇒ lookup_none_from_member_false k v leq key l
          } ;
        False ⇒ lookup_none_from_member_false k v leq key r
      }
  }

fn lookup_unit_some_from_member_true_leaf
  (h : Equal Bool False True)
  : Equal (Option Unit) (None Unit) (Some Unit MkUnit) =
  absurd h

fn lookup_unit_some_from_member_true_hit
  (val : Unit) (h : Equal Bool True True)
  : Equal (Option Unit) (Some Unit val) (Some Unit MkUnit) =
  match val { MkUnit ⇒ tt }

fn lookup_unit_some_from_member_true
  (k : Type) (leq : k → k → Bool) (key : k) (m : Tree k Unit)
  : Equal Bool (member k Unit leq key m) True →
    Equal (Option Unit) (lookup k Unit leq key m) (Some Unit MkUnit) =
  match m {
    Leaf ⇒ lookup_unit_some_from_member_true_leaf ;
    Node l k2 v2 r ⇒
      match leq key k2 {
        True ⇒
          match leq k2 key {
            True ⇒ lookup_unit_some_from_member_true_hit v2 ;
            False ⇒ lookup_unit_some_from_member_true k leq key l
          } ;
        False ⇒ lookup_unit_some_from_member_true k leq key r
      }
  }

fn not_order_equiv_from_member_true_false
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (inserted : k) (query : k) (keep : Tree k v)
  (hInserted : Equal Bool (member k v leq inserted keep) True)
  (hQuery : Equal Bool (member k v leq query keep) False)
  : not_order_equiv_to_key k leq inserted query =
  λheq.
    absurd (trans Bool True (member k v leq inserted keep) False
      (sym Bool (member k v leq inserted keep) True hInserted)
      (trans Bool
        (member k v leq inserted keep)
        (member k v leq query keep)
        False
        (member_order_equiv_agree k v leq transLeq inserted query keep heq)
        hQuery))

fn not_order_equiv_from_member_false_true
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (inserted : k) (query : k) (reject : Tree k v)
  (hInserted : Equal Bool (member k v leq inserted reject) False)
  (hQuery : Equal Bool (member k v leq query reject) True)
  : not_order_equiv_to_key k leq inserted query =
  λheq.
    absurd (trans Bool True (member k v leq inserted reject) False
      (sym Bool (member k v leq inserted reject) True
        (trans Bool
          (member k v leq inserted reject)
          (member k v leq query reject)
          True
          (member_order_equiv_agree k v leq transLeq inserted query reject heq)
          hQuery))
      hInserted)
```

#### 4.7.8 `intersection` — lookup characterization

`intersection_lookup_characterization` states `intersection`'s expected
lookup result (present with the *left* tree's value, exactly when both
sides have the key), with `intersection_lookup_some_law`/
`_lookup_left_none_law` as the two pointwise corollaries. Built via the
same locality/hit/miss dispatch pattern as `union`'s characterization
(§4.7.7), instantiated against `intersection_from_list_acc`'s own
filter-by-lookup shape.

```ken
fn intersection_from_list_acc_lookup_none_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (hKeep : Equal Bool (member k v leq key keep) False)
  (hAcc : Equal (Option v) (lookup k v leq key acc) (None v))
  (o : Or (Equal Bool (member k v leq (pair_fst k v e) keep) True)
          (Equal Bool (member k v leq (pair_fst k v e) keep) False))
  : Equal (Option v)
      (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
      (None v) =
  match o {
    Inl q ⇒
      trans (Option v)
        (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
        (lookup k v leq key
          (intersection_from_list_acc k v leq xs2 keep
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
        (None v)
        (cong (Tree k v) (Option v)
          (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
          (intersection_from_list_acc k v leq xs2 keep
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq key)
          (intersection_from_list_acc_true_bridge k v leq e xs2 keep acc q))
        (intersection_from_list_acc_lookup_none k v leq transLeq key xs2 keep
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
          hKeep
          (trans (Option v)
            (lookup k v leq key (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
            (lookup k v leq key acc)
            (None v)
            (lookup_locality k v leq transLeq (pair_fst k v e) key (pair_snd k v e) acc
              (not_order_equiv_from_member_true_false k v leq transLeq (pair_fst k v e) key keep q hKeep))
            hAcc)) ;
    Inr q ⇒
      trans (Option v)
        (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
        (lookup k v leq key (intersection_from_list_acc k v leq xs2 keep acc))
        (None v)
        (cong (Tree k v) (Option v)
          (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
          (intersection_from_list_acc k v leq xs2 keep acc)
          (lookup k v leq key)
          (intersection_from_list_acc_false_bridge k v leq e xs2 keep acc q))
        (intersection_from_list_acc_lookup_none k v leq transLeq key xs2 keep acc hKeep hAcc)
  }

fn intersection_from_list_acc_lookup_none
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (xs : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  : Equal Bool (member k v leq key keep) False →
    Equal (Option v) (lookup k v leq key acc) (None v) →
    Equal (Option v) (lookup k v leq key (intersection_from_list_acc k v leq xs keep acc)) (None v) =
  match xs {
    Nil ⇒ λhKeep. λhAcc. hAcc ;
    Cons e xs2 ⇒
      λhKeep. λhAcc.
        intersection_from_list_acc_lookup_none_dispatch k v leq transLeq key e xs2 keep acc hKeep hAcc
          (bool_dichotomy (member k v leq (pair_fst k v e) keep))
  }

fn intersection_lookup_characterization
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (a : Tree k v) (b : Tree k v)
  : Equal (Option v) (lookup k v leq key b) (None v) →
    Equal (Option v) (lookup k v leq key (intersection k v leq a b)) (None v) =
  λhb.
    intersection_from_list_acc_lookup_none k v leq transLeq key (to_list k v a) b (empty k v)
      (member_from_lookup_none k v leq key b hb)
      (lookup_empty_is_none k v leq key)

fn intersection_from_list_acc_lookup_locality_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (hskip : all_in_list k v (not_order_equiv_to_key k leq key) (Cons (Pair k v) e xs2))
  (o : Or (Equal Bool (member k v leq (pair_fst k v e) keep) True)
          (Equal Bool (member k v leq (pair_fst k v e) keep) False))
  : Equal (Option v)
      (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
      (lookup k v leq key acc) =
  match o {
    Inl q ⇒
      trans (Option v)
        (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
        (lookup k v leq key
          (intersection_from_list_acc k v leq xs2 keep
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
        (lookup k v leq key acc)
        (cong (Tree k v) (Option v)
          (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
          (intersection_from_list_acc k v leq xs2 keep
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq key)
          (intersection_from_list_acc_true_bridge k v leq e xs2 keep acc q))
        (trans (Option v)
          (lookup k v leq key
            (intersection_from_list_acc k v leq xs2 keep
              (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
          (lookup k v leq key
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq key acc)
          (intersection_from_list_acc_lookup_locality k v leq transLeq key xs2 keep
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
            (and_snd
              (not_order_equiv_to_key k leq key (pair_fst k v e))
              (all_in_list k v (not_order_equiv_to_key k leq key) xs2) hskip))
          (lookup_locality k v leq transLeq (pair_fst k v e) key (pair_snd k v e) acc
            (not_order_equiv_swap k leq key (pair_fst k v e)
              (and_fst
                (not_order_equiv_to_key k leq key (pair_fst k v e))
                (all_in_list k v (not_order_equiv_to_key k leq key) xs2) hskip)))) ;
    Inr q ⇒
      trans (Option v)
        (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
        (lookup k v leq key (intersection_from_list_acc k v leq xs2 keep acc))
        (lookup k v leq key acc)
        (cong (Tree k v) (Option v)
          (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
          (intersection_from_list_acc k v leq xs2 keep acc)
          (lookup k v leq key)
          (intersection_from_list_acc_false_bridge k v leq e xs2 keep acc q))
        (intersection_from_list_acc_lookup_locality k v leq transLeq key xs2 keep acc
          (and_snd
            (not_order_equiv_to_key k leq key (pair_fst k v e))
            (all_in_list k v (not_order_equiv_to_key k leq key) xs2) hskip))
  }

fn intersection_from_list_acc_lookup_locality
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (xs : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  : all_in_list k v (not_order_equiv_to_key k leq key) xs →
    Equal (Option v)
      (lookup k v leq key (intersection_from_list_acc k v leq xs keep acc))
      (lookup k v leq key acc) =
  match xs {
    Nil ⇒ λhskip. Refl ;
    Cons e xs2 ⇒
      λhskip.
        intersection_from_list_acc_lookup_locality_dispatch k v leq transLeq key e xs2 keep acc hskip
          (bool_dichotomy (member k v leq (pair_fst k v e) keep))
  }

fn intersection_from_list_acc_lookup_some_hit
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (x : v) (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (hAssoc : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (Some v x))
  (q1 : Equal Bool (leq key (pair_fst k v e)) True)
  (q2 : Equal Bool (leq (pair_fst k v e) key) True)
  (hMember : Equal Bool (member k v leq key keep) True)
  : Equal (Option v)
      (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
      (Some v x) =
  trans (Option v)
    (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
    (lookup k v leq key
      (intersection_from_list_acc k v leq xs2 keep
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
    (Some v x)
    (cong (Tree k v) (Option v)
      (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
      (intersection_from_list_acc k v leq xs2 keep
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
      (lookup k v leq key)
      (intersection_from_list_acc_true_bridge k v leq e xs2 keep acc
        (trans Bool
          (member k v leq (pair_fst k v e) keep)
          (member k v leq key keep)
          True
          (member_order_equiv_agree k v leq transLeq (pair_fst k v e) key keep
            (and_intro (Equal Bool (leq (pair_fst k v e) key) True)
              (Equal Bool (leq key (pair_fst k v e)) True) q2 q1))
          hMember)))
    (trans (Option v)
      (lookup k v leq key
        (intersection_from_list_acc k v leq xs2 keep
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
      (lookup k v leq key (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
      (Some v x)
      (intersection_from_list_acc_lookup_locality k v leq transLeq key xs2 keep
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
        (all_in_list_map_not_match_transfer k v leq transLeq key (pair_fst k v e) q1 q2 xs2
          (and_fst
            (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
            (NoDup k v leq xs2) h)))
      (trans (Option v)
        (lookup k v leq key (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
        (Some v (pair_snd k v e))
        (Some v x)
        (insert_lookup_hit k v leq reflLeq transLeq (pair_fst k v e) key (pair_snd k v e) acc
          (and_intro (Equal Bool (leq key (pair_fst k v e)) True)
            (Equal Bool (leq (pair_fst k v e) key) True) q1 q2))
        (trans (Option v)
          (Some v (pair_snd k v e))
          (assoc k v leq key (Cons (Pair k v) e xs2))
          (Some v x)
          (sym (Option v)
            (assoc k v leq key (Cons (Pair k v) e xs2))
            (Some v (pair_snd k v e))
            (assoc_skip_head_stop_bridge k v leq key e xs2 q1 q2))
          hAssoc)))

fn intersection_from_list_acc_lookup_some_miss_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (x : v) (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (hTail : NoDup k v leq xs2)
  (hAssocTail : Equal (Option v) (assoc k v leq key xs2) (Some v x))
  (hMember : Equal Bool (member k v leq key keep) True)
  (o : Or (Equal Bool (member k v leq (pair_fst k v e) keep) True)
          (Equal Bool (member k v leq (pair_fst k v e) keep) False))
  : Equal (Option v)
      (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
      (Some v x) =
  match o {
    Inl q ⇒
      trans (Option v)
        (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
        (lookup k v leq key
          (intersection_from_list_acc k v leq xs2 keep
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
        (Some v x)
        (cong (Tree k v) (Option v)
          (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
          (intersection_from_list_acc k v leq xs2 keep
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq key)
          (intersection_from_list_acc_true_bridge k v leq e xs2 keep acc q))
        (intersection_from_list_acc_lookup_some k v leq reflLeq transLeq key x xs2 keep
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
          hTail hAssocTail hMember) ;
    Inr q ⇒
      trans (Option v)
        (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
        (lookup k v leq key (intersection_from_list_acc k v leq xs2 keep acc))
        (Some v x)
        (cong (Tree k v) (Option v)
          (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
          (intersection_from_list_acc k v leq xs2 keep acc)
          (lookup k v leq key)
          (intersection_from_list_acc_false_bridge k v leq e xs2 keep acc q))
        (intersection_from_list_acc_lookup_some k v leq reflLeq transLeq key x xs2 keep acc
          hTail hAssocTail hMember)
  }

fn intersection_from_list_acc_lookup_some_inner
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (x : v) (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (hAssoc : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (Some v x))
  (hMember : Equal Bool (member k v leq key keep) True)
  (q1 : Equal Bool (leq key (pair_fst k v e)) True)
  (o2 : Or (Equal Bool (leq (pair_fst k v e) key) True) (Equal Bool (leq (pair_fst k v e) key) False))
  : Equal (Option v)
      (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
      (Some v x) =
  match o2 {
    Inl q2 ⇒
      intersection_from_list_acc_lookup_some_hit k v leq reflLeq transLeq key x e xs2 keep acc h hAssoc q1 q2 hMember ;
    Inr q2False ⇒
      intersection_from_list_acc_lookup_some_miss_dispatch k v leq reflLeq transLeq key x e xs2 keep acc
        (and_snd
          (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
          (NoDup k v leq xs2) h)
        (trans (Option v)
          (assoc k v leq key xs2)
          (assoc k v leq key (Cons (Pair k v) e xs2))
          (Some v x)
          (sym (Option v)
            (assoc k v leq key (Cons (Pair k v) e xs2))
            (assoc k v leq key xs2)
            (assoc_skip_head_bridge k v leq key e xs2 q1 q2False))
          hAssoc)
        hMember
        (bool_dichotomy (member k v leq (pair_fst k v e) keep))
  }

fn intersection_from_list_acc_lookup_some_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (x : v) (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (hAssoc : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (Some v x))
  (hMember : Equal Bool (member k v leq key keep) True)
  (o1 : Or (Equal Bool (leq key (pair_fst k v e)) True) (Equal Bool (leq key (pair_fst k v e)) False))
  : Equal (Option v)
      (lookup k v leq key (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc))
      (Some v x) =
  match o1 {
    Inl q1 ⇒
      intersection_from_list_acc_lookup_some_inner k v leq reflLeq transLeq key x e xs2 keep acc h hAssoc hMember q1
        (bool_dichotomy (leq (pair_fst k v e) key)) ;
    Inr q1False ⇒
      intersection_from_list_acc_lookup_some_miss_dispatch k v leq reflLeq transLeq key x e xs2 keep acc
        (and_snd
          (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
          (NoDup k v leq xs2) h)
        (trans (Option v)
          (assoc k v leq key xs2)
          (assoc k v leq key (Cons (Pair k v) e xs2))
          (Some v x)
          (sym (Option v)
            (assoc k v leq key (Cons (Pair k v) e xs2))
            (assoc k v leq key xs2)
            (assoc_skip_head_bridge_false k v leq key e xs2 q1False))
          hAssoc)
        hMember
        (bool_dichotomy (member k v leq (pair_fst k v e) keep))
  }

fn intersection_from_list_acc_lookup_some
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (x : v) (xs : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  : NoDup k v leq xs →
    Equal (Option v) (assoc k v leq key xs) (Some v x) →
    Equal Bool (member k v leq key keep) True →
    Equal (Option v)
      (lookup k v leq key (intersection_from_list_acc k v leq xs keep acc))
      (Some v x) =
  match xs {
    Nil ⇒ λh. λhAssoc. λhMember. absurd hAssoc ;
    Cons e xs2 ⇒
      λh. λhAssoc. λhMember.
        intersection_from_list_acc_lookup_some_dispatch k v leq reflLeq transLeq key x e xs2 keep acc h hAssoc hMember
          (bool_dichotomy (leq key (pair_fst k v e)))
  }

fn intersection_lookup_some_law
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (x : v) (a : Tree k v) (b : Tree k v)
  : Ordered k v leq a → Distinct k v leq a →
    Equal (Option v) (lookup k v leq key a) (Some v x) →
    Equal Bool (member k v leq key b) True →
    Equal (Option v) (lookup k v leq key (intersection k v leq a b)) (Some v x) =
  λhord. λhdist. λha. λhmem.
    intersection_from_list_acc_lookup_some k v leq reflLeq transLeq key x (to_list k v a) b (empty k v)
      hdist
      (trans (Option v)
        (assoc k v leq key (to_list k v a))
        (lookup k v leq key a)
        (Some v x)
        (sym (Option v)
          (lookup k v leq key a)
          (assoc k v leq key (to_list k v a))
          (lookup_assoc_agree k v leq transLeq key a hord hdist))
        ha)
      hmem

fn intersection_lookup_left_none_law
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (a : Tree k Unit) (b : Tree k Unit)
  : Ordered k Unit leq a → Distinct k Unit leq a →
    Equal Bool (member k Unit leq key a) False →
    Equal (Option Unit) (lookup k Unit leq key (intersection k Unit leq a b)) (None Unit) =
  λhord. λhdist. λhmem.
    trans (Option Unit)
      (lookup k Unit leq key (intersection k Unit leq a b))
      (lookup k Unit leq key (empty k Unit))
      (None Unit)
      (intersection_from_list_acc_lookup_locality k Unit leq transLeq key
        (to_list k Unit a) b (empty k Unit)
        (assoc_none_implies_no_match k Unit leq key (to_list k Unit a)
          (trans (Option Unit)
            (assoc k Unit leq key (to_list k Unit a))
            (lookup k Unit leq key a)
            (None Unit)
            (sym (Option Unit)
              (lookup k Unit leq key a)
              (assoc k Unit leq key (to_list k Unit a))
              (lookup_assoc_agree k Unit leq transLeq key a hord hdist))
            (lookup_none_from_member_false k Unit leq key a hmem))))
      (lookup_empty_is_none k Unit leq key)
```

#### 4.7.9 `difference` — lookup characterization

`difference_lookup_characterization` states `difference`'s expected lookup
result (present with the *left* tree's value exactly when the *right* tree
lacks the key), assembled from `_lookup_locality`, `_lookup_none`, and
`_lookup_keep` sub-lemmas covering the three ways a key can relate to the
two input trees, the same three-way structure `intersection`'s
characterization uses.

```ken
fn difference_from_list_acc_lookup_locality_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (hskip : all_in_list k v (not_order_equiv_to_key k leq key) (Cons (Pair k v) e xs2))
  (o : Or (Equal Bool (member k v leq (pair_fst k v e) reject) True)
          (Equal Bool (member k v leq (pair_fst k v e) reject) False))
  : Equal (Option v)
      (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
      (lookup k v leq key acc) =
  match o {
    Inl q ⇒
      trans (Option v)
        (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
        (lookup k v leq key (difference_from_list_acc k v leq xs2 reject acc))
        (lookup k v leq key acc)
        (cong (Tree k v) (Option v)
          (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
          (difference_from_list_acc k v leq xs2 reject acc)
          (lookup k v leq key)
          (difference_from_list_acc_true_bridge k v leq e xs2 reject acc q))
        (difference_from_list_acc_lookup_locality k v leq transLeq key xs2 reject acc
          (and_snd
            (not_order_equiv_to_key k leq key (pair_fst k v e))
            (all_in_list k v (not_order_equiv_to_key k leq key) xs2) hskip)) ;
    Inr q ⇒
      trans (Option v)
        (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
        (lookup k v leq key
          (difference_from_list_acc k v leq xs2 reject
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
        (lookup k v leq key acc)
        (cong (Tree k v) (Option v)
          (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
          (difference_from_list_acc k v leq xs2 reject
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq key)
          (difference_from_list_acc_false_bridge k v leq e xs2 reject acc q))
        (trans (Option v)
          (lookup k v leq key
            (difference_from_list_acc k v leq xs2 reject
              (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
          (lookup k v leq key
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq key acc)
          (difference_from_list_acc_lookup_locality k v leq transLeq key xs2 reject
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
            (and_snd
              (not_order_equiv_to_key k leq key (pair_fst k v e))
              (all_in_list k v (not_order_equiv_to_key k leq key) xs2) hskip))
          (lookup_locality k v leq transLeq (pair_fst k v e) key (pair_snd k v e) acc
            (not_order_equiv_swap k leq key (pair_fst k v e)
              (and_fst
                (not_order_equiv_to_key k leq key (pair_fst k v e))
                (all_in_list k v (not_order_equiv_to_key k leq key) xs2) hskip))))
  }

fn difference_from_list_acc_lookup_locality
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (xs : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  : all_in_list k v (not_order_equiv_to_key k leq key) xs →
    Equal (Option v)
      (lookup k v leq key (difference_from_list_acc k v leq xs reject acc))
      (lookup k v leq key acc) =
  match xs {
    Nil ⇒ λhskip. Refl ;
    Cons e xs2 ⇒
      λhskip.
        difference_from_list_acc_lookup_locality_dispatch k v leq transLeq key e xs2 reject acc hskip
          (bool_dichotomy (member k v leq (pair_fst k v e) reject))
  }

fn difference_from_list_acc_lookup_none_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (hReject : Equal Bool (member k v leq key reject) True)
  (hAcc : Equal (Option v) (lookup k v leq key acc) (None v))
  (o : Or (Equal Bool (member k v leq (pair_fst k v e) reject) True)
          (Equal Bool (member k v leq (pair_fst k v e) reject) False))
  : Equal (Option v)
      (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
      (None v) =
  match o {
    Inl q ⇒
      trans (Option v)
        (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
        (lookup k v leq key (difference_from_list_acc k v leq xs2 reject acc))
        (None v)
        (cong (Tree k v) (Option v)
          (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
          (difference_from_list_acc k v leq xs2 reject acc)
          (lookup k v leq key)
          (difference_from_list_acc_true_bridge k v leq e xs2 reject acc q))
        (difference_from_list_acc_lookup_none k v leq transLeq key xs2 reject acc hReject hAcc) ;
    Inr q ⇒
      trans (Option v)
        (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
        (lookup k v leq key
          (difference_from_list_acc k v leq xs2 reject
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
        (None v)
        (cong (Tree k v) (Option v)
          (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
          (difference_from_list_acc k v leq xs2 reject
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq key)
          (difference_from_list_acc_false_bridge k v leq e xs2 reject acc q))
        (difference_from_list_acc_lookup_none k v leq transLeq key xs2 reject
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
          hReject
          (trans (Option v)
            (lookup k v leq key (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
            (lookup k v leq key acc)
            (None v)
            (lookup_locality k v leq transLeq (pair_fst k v e) key (pair_snd k v e) acc
              (not_order_equiv_from_member_false_true k v leq transLeq (pair_fst k v e) key reject q hReject))
            hAcc))
  }

fn difference_from_list_acc_lookup_none
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (xs : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  : Equal Bool (member k v leq key reject) True →
    Equal (Option v) (lookup k v leq key acc) (None v) →
    Equal (Option v) (lookup k v leq key (difference_from_list_acc k v leq xs reject acc)) (None v) =
  match xs {
    Nil ⇒ λhReject. λhAcc. hAcc ;
    Cons e xs2 ⇒
      λhReject. λhAcc.
        difference_from_list_acc_lookup_none_dispatch k v leq transLeq key e xs2 reject acc hReject hAcc
          (bool_dichotomy (member k v leq (pair_fst k v e) reject))
  }

fn difference_from_list_acc_lookup_keep_hit
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (hReject : Equal Bool (member k v leq key reject) False)
  (q1 : Equal Bool (leq key (pair_fst k v e)) True)
  (q2 : Equal Bool (leq (pair_fst k v e) key) True)
  : Equal (Option v)
      (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
      (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc)) =
  trans (Option v)
    (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
    (lookup k v leq key
      (difference_from_list_acc k v leq xs2 reject
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
    (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
    (cong (Tree k v) (Option v)
      (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
      (difference_from_list_acc k v leq xs2 reject
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
      (lookup k v leq key)
      (difference_from_list_acc_false_bridge k v leq e xs2 reject acc
        (trans Bool
          (member k v leq (pair_fst k v e) reject)
          (member k v leq key reject)
          False
          (member_order_equiv_agree k v leq transLeq (pair_fst k v e) key reject
            (and_intro (Equal Bool (leq (pair_fst k v e) key) True)
              (Equal Bool (leq key (pair_fst k v e)) True) q2 q1))
          hReject)))
    (trans (Option v)
      (lookup k v leq key
        (difference_from_list_acc k v leq xs2 reject
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
      (lookup k v leq key (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
      (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
      (difference_from_list_acc_lookup_locality k v leq transLeq key xs2 reject
        (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
        (all_in_list_map_not_match_transfer k v leq transLeq key (pair_fst k v e) q1 q2 xs2
          (and_fst
            (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
            (NoDup k v leq xs2) h)))
      (trans (Option v)
        (lookup k v leq key (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
        (Some v (pair_snd k v e))
        (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
        (insert_lookup_hit k v leq reflLeq transLeq (pair_fst k v e) key (pair_snd k v e) acc
          (and_intro (Equal Bool (leq key (pair_fst k v e)) True)
            (Equal Bool (leq (pair_fst k v e) key) True) q1 q2))
        (sym (Option v)
          (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
          (Some v (pair_snd k v e))
          (cong (Option v) (Option v)
            (assoc k v leq key (Cons (Pair k v) e xs2))
            (Some v (pair_snd k v e))
            (λleft. difference_lookup_table v left False (lookup k v leq key acc))
            (assoc_skip_head_stop_bridge k v leq key e xs2 q1 q2)))))

fn difference_from_list_acc_lookup_keep_miss_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (hTail : NoDup k v leq xs2)
  (hReject : Equal Bool (member k v leq key reject) False)
  (hnot : not_order_equiv_to_key k leq (pair_fst k v e) key)
  (assocSkip : Equal (Option v) (assoc k v leq key (Cons (Pair k v) e xs2)) (assoc k v leq key xs2))
  (o : Or (Equal Bool (member k v leq (pair_fst k v e) reject) True)
          (Equal Bool (member k v leq (pair_fst k v e) reject) False))
  : Equal (Option v)
      (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
      (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc)) =
  match o {
    Inl q ⇒
      trans (Option v)
        (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
        (lookup k v leq key (difference_from_list_acc k v leq xs2 reject acc))
        (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
        (cong (Tree k v) (Option v)
          (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
          (difference_from_list_acc k v leq xs2 reject acc)
          (lookup k v leq key)
          (difference_from_list_acc_true_bridge k v leq e xs2 reject acc q))
        (trans (Option v)
          (lookup k v leq key (difference_from_list_acc k v leq xs2 reject acc))
          (difference_lookup_table v (assoc k v leq key xs2) False (lookup k v leq key acc))
          (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
          (difference_from_list_acc_lookup_keep k v leq reflLeq transLeq key xs2 reject acc hTail hReject)
          (sym (Option v)
            (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
            (difference_lookup_table v (assoc k v leq key xs2) False (lookup k v leq key acc))
            (cong (Option v) (Option v)
              (assoc k v leq key (Cons (Pair k v) e xs2))
              (assoc k v leq key xs2)
              (λleft. difference_lookup_table v left False (lookup k v leq key acc))
              assocSkip))) ;
    Inr q ⇒
      trans (Option v)
        (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
        (lookup k v leq key
          (difference_from_list_acc k v leq xs2 reject
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
        (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
        (cong (Tree k v) (Option v)
          (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
          (difference_from_list_acc k v leq xs2 reject
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (lookup k v leq key)
          (difference_from_list_acc_false_bridge k v leq e xs2 reject acc q))
        (trans (Option v)
          (lookup k v leq key
            (difference_from_list_acc k v leq xs2 reject
              (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
          (difference_lookup_table v (assoc k v leq key xs2) False
            (lookup k v leq key (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
          (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
          (difference_from_list_acc_lookup_keep k v leq reflLeq transLeq key xs2 reject
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
            hTail hReject)
          (trans (Option v)
            (difference_lookup_table v (assoc k v leq key xs2) False
              (lookup k v leq key (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)))
            (difference_lookup_table v (assoc k v leq key xs2) False (lookup k v leq key acc))
            (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
            (cong (Option v) (Option v)
              (lookup k v leq key (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
              (lookup k v leq key acc)
              (λprior. difference_lookup_table v (assoc k v leq key xs2) False prior)
              (lookup_locality k v leq transLeq (pair_fst k v e) key (pair_snd k v e) acc hnot))
            (sym (Option v)
              (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc))
              (difference_lookup_table v (assoc k v leq key xs2) False (lookup k v leq key acc))
              (cong (Option v) (Option v)
                (assoc k v leq key (Cons (Pair k v) e xs2))
                (assoc k v leq key xs2)
                (λleft. difference_lookup_table v left False (lookup k v leq key acc))
                assocSkip))))
  }

fn difference_from_list_acc_lookup_keep_inner
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (hReject : Equal Bool (member k v leq key reject) False)
  (q1 : Equal Bool (leq key (pair_fst k v e)) True)
  (o2 : Or (Equal Bool (leq (pair_fst k v e) key) True) (Equal Bool (leq (pair_fst k v e) key) False))
  : Equal (Option v)
      (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
      (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc)) =
  match o2 {
    Inl q2 ⇒
      difference_from_list_acc_lookup_keep_hit k v leq reflLeq transLeq key e xs2 reject acc h hReject q1 q2 ;
    Inr q2False ⇒
      difference_from_list_acc_lookup_keep_miss_dispatch k v leq reflLeq transLeq key e xs2 reject acc
        (and_snd
          (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
          (NoDup k v leq xs2) h)
        hReject
        (not_order_equiv_from_left_false k leq (pair_fst k v e) key q2False)
        (assoc_skip_head_bridge k v leq key e xs2 q1 q2False)
        (bool_dichotomy (member k v leq (pair_fst k v e) reject))
  }

fn difference_from_list_acc_lookup_keep_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (h : NoDup k v leq (Cons (Pair k v) e xs2))
  (hReject : Equal Bool (member k v leq key reject) False)
  (o1 : Or (Equal Bool (leq key (pair_fst k v e)) True) (Equal Bool (leq key (pair_fst k v e)) False))
  : Equal (Option v)
      (lookup k v leq key (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc))
      (difference_lookup_table v (assoc k v leq key (Cons (Pair k v) e xs2)) False (lookup k v leq key acc)) =
  match o1 {
    Inl q1 ⇒
      difference_from_list_acc_lookup_keep_inner k v leq reflLeq transLeq key e xs2 reject acc h hReject q1
        (bool_dichotomy (leq (pair_fst k v e) key)) ;
    Inr q1False ⇒
      difference_from_list_acc_lookup_keep_miss_dispatch k v leq reflLeq transLeq key e xs2 reject acc
        (and_snd
          (all_in_list k v (not_order_equiv_to_key k leq (pair_fst k v e)) xs2)
          (NoDup k v leq xs2) h)
        hReject
        (not_order_equiv_from_right_false k leq (pair_fst k v e) key q1False)
        (assoc_skip_head_bridge_false k v leq key e xs2 q1False)
        (bool_dichotomy (member k v leq (pair_fst k v e) reject))
  }

fn difference_from_list_acc_lookup_keep
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (xs : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  : NoDup k v leq xs →
    Equal Bool (member k v leq key reject) False →
    Equal (Option v)
      (lookup k v leq key (difference_from_list_acc k v leq xs reject acc))
      (difference_lookup_table v (assoc k v leq key xs) False (lookup k v leq key acc)) =
  match xs {
    Nil ⇒ λh. λhReject. Refl ;
    Cons e xs2 ⇒
      λh. λhReject.
        difference_from_list_acc_lookup_keep_dispatch k v leq reflLeq transLeq key e xs2 reject acc h hReject
          (bool_dichotomy (leq key (pair_fst k v e)))
  }

fn difference_lookup_characterization_reject
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (a : Tree k v) (b : Tree k v)
  (qReject : Equal Bool (member k v leq key b) True)
  : Equal (Option v) (lookup k v leq key (difference k v leq a b))
      (difference_lookup_expected k v leq key a b) =
  trans (Option v)
    (lookup k v leq key (difference k v leq a b))
    (None v)
    (difference_lookup_expected k v leq key a b)
    (difference_from_list_acc_lookup_none k v leq transLeq key (to_list k v a) b (empty k v)
      qReject
      (lookup_empty_is_none k v leq key))
    (sym (Option v)
      (difference_lookup_expected k v leq key a b)
      (None v)
      (difference_lookup_expected_true k v leq key a b qReject))

fn difference_lookup_characterization_keep
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (a : Tree k v) (b : Tree k v)
  (hord : Ordered k v leq a) (hdist : Distinct k v leq a)
  (qKeep : Equal Bool (member k v leq key b) False)
  : Equal (Option v) (lookup k v leq key (difference k v leq a b))
      (difference_lookup_expected k v leq key a b) =
  trans (Option v)
    (lookup k v leq key (difference k v leq a b))
    (difference_lookup_table v (assoc k v leq key (to_list k v a)) False (lookup k v leq key (empty k v)))
    (difference_lookup_expected k v leq key a b)
    (difference_from_list_acc_lookup_keep k v leq reflLeq transLeq key (to_list k v a) b (empty k v)
      hdist qKeep)
    (trans (Option v)
      (difference_lookup_table v (assoc k v leq key (to_list k v a)) False (lookup k v leq key (empty k v)))
      (assoc k v leq key (to_list k v a))
      (difference_lookup_expected k v leq key a b)
      (trans (Option v)
        (difference_lookup_table v (assoc k v leq key (to_list k v a)) False (lookup k v leq key (empty k v)))
        (difference_lookup_table v (assoc k v leq key (to_list k v a)) False (None v))
        (assoc k v leq key (to_list k v a))
        (cong (Option v) (Option v)
          (lookup k v leq key (empty k v))
          (None v)
          (λprior. difference_lookup_table v (assoc k v leq key (to_list k v a)) False prior)
          (lookup_empty_is_none k v leq key))
        (difference_lookup_table_false_none v (assoc k v leq key (to_list k v a))))
      (trans (Option v)
        (assoc k v leq key (to_list k v a))
        (lookup k v leq key a)
        (difference_lookup_expected k v leq key a b)
        (sym (Option v)
          (lookup k v leq key a)
          (assoc k v leq key (to_list k v a))
          (lookup_assoc_agree k v leq transLeq key a hord hdist))
        (sym (Option v)
          (difference_lookup_expected k v leq key a b)
          (lookup k v leq key a)
          (difference_lookup_expected_false k v leq key a b qKeep))))

fn difference_lookup_characterization_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (a : Tree k v) (b : Tree k v)
  (hord : Ordered k v leq a) (hdist : Distinct k v leq a)
  (o : Or (Equal Bool (member k v leq key b) True) (Equal Bool (member k v leq key b) False))
  : Equal (Option v) (lookup k v leq key (difference k v leq a b))
      (difference_lookup_expected k v leq key a b) =
  match o {
    Inl qReject ⇒ difference_lookup_characterization_reject k v leq reflLeq transLeq key a b qReject ;
    Inr qKeep ⇒ difference_lookup_characterization_keep k v leq reflLeq transLeq key a b hord hdist qKeep
  }

fn difference_lookup_characterization
  (k : Type) (v : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (key : k) (a : Tree k v) (b : Tree k v)
  : Ordered k v leq a → Distinct k v leq a →
    Equal (Option v) (lookup k v leq key (difference k v leq a b))
      (difference_lookup_expected k v leq key a b) =
  λhord. λhdist.
    difference_lookup_characterization_dispatch k v leq reflLeq transLeq key a b hord hdist
      (bool_dichotomy (member k v leq key b))
```

#### 4.7.10 Ordered-preservation for `insert_with`'s fold, `union`, `intersection`, `difference`

`insert_with_fold_step_preserves_ordered`/`fold_insert_with_preserves_ordered`
extend §4.7.5's fold-preservation lemma to the combining case;
`union_preserves_ordered`/`intersection_preserves_ordered`/
`difference_preserves_ordered` each compose that fold-preservation fact
with their own filter-then-rebuild construction, closing the
Ordered-preservation obligation for every Layer-2 combinator.

```ken
fn insert_with_fold_step_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (f : v → v → v) (key : k) (val : v) (acc : Tree k v)
  : Ordered k v leq acc → Ordered k v leq (insert_with_fold_step k v leq f key val acc) =
  λh.
    J (λt _. Ordered k v leq t)
      (insert_with_preserves_ordered k v leq transLeq total f key val acc h)
      (sym (Tree k v)
        (insert_with_fold_step k v leq f key val acc)
        (insert_with k v leq f key val acc)
        (insert_with_fold_step_reduces k v leq f key val acc))

fn fold_insert_with_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (f : v → v → v) (src : Tree k v) (acc : Tree k v)
  : Ordered k v leq acc → Ordered k v leq (fold k v (Tree k v) (insert_with_fold_step k v leq f) acc src) =
  match src {
    Leaf ⇒ λh. h ;
    Node l key val r ⇒
      λh.
        fold_insert_with_preserves_ordered k v leq transLeq total f r
          (insert_with_fold_step k v leq f key val (fold k v (Tree k v) (insert_with_fold_step k v leq f) acc l))
          (insert_with_preserves_ordered k v leq transLeq total f key val
            (fold k v (Tree k v) (insert_with_fold_step k v leq f) acc l)
            (fold_insert_with_preserves_ordered k v leq transLeq total f l acc h))
  }

fn union_from_list_acc_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (f : v → v → v) (xs : List (Pair k v)) (acc : Tree k v)
  : Ordered k v leq acc → Ordered k v leq (union_from_list_acc k v leq f xs acc) =
  match xs {
    Nil ⇒ λh. h ;
    Cons e xs2 ⇒
      λh.
        union_from_list_acc_preserves_ordered k v leq transLeq total f xs2
          (insert_with_fold_step k v leq f (pair_fst k v e) (pair_snd k v e) acc)
          (insert_with_fold_step_preserves_ordered k v leq transLeq total f
            (pair_fst k v e) (pair_snd k v e) acc h)
  }

fn union_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (f : v → v → v) (a : Tree k v) (b : Tree k v)
  : Ordered k v leq b → Ordered k v leq (union k v leq f a b) =
  union_from_list_acc_preserves_ordered k v leq transLeq total f (to_list k v a) b

fn intersection_from_list_acc_preserves_ordered_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (e : Pair k v) (xs2 : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  (h : Ordered k v leq acc)
  (o : Or (Equal Bool (member k v leq (pair_fst k v e) keep) True)
          (Equal Bool (member k v leq (pair_fst k v e) keep) False))
  : Ordered k v leq (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc) =
  match o {
    Inl q ⇒
      J (λt _. Ordered k v leq t)
        (intersection_from_list_acc_preserves_ordered k v leq transLeq total xs2 keep
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
          (preserves_ordered k v leq transLeq total (pair_fst k v e) (pair_snd k v e) acc h))
        (sym (Tree k v)
          (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
          (intersection_from_list_acc k v leq xs2 keep
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (intersection_from_list_acc_true_bridge k v leq e xs2 keep acc q)) ;
    Inr q ⇒
      J (λt _. Ordered k v leq t)
        (intersection_from_list_acc_preserves_ordered k v leq transLeq total xs2 keep acc h)
        (sym (Tree k v)
          (intersection_from_list_acc k v leq (Cons (Pair k v) e xs2) keep acc)
          (intersection_from_list_acc k v leq xs2 keep acc)
          (intersection_from_list_acc_false_bridge k v leq e xs2 keep acc q))
  }

fn intersection_from_list_acc_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (xs : List (Pair k v)) (keep : Tree k v) (acc : Tree k v)
  : Ordered k v leq acc → Ordered k v leq (intersection_from_list_acc k v leq xs keep acc) =
  match xs {
    Nil ⇒ λh. h ;
    Cons e xs2 ⇒
      λh.
        intersection_from_list_acc_preserves_ordered_dispatch k v leq transLeq total e xs2 keep acc h
          (bool_dichotomy (member k v leq (pair_fst k v e) keep))
  }

fn intersection_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (a : Tree k v) (b : Tree k v)
  : Ordered k v leq (intersection k v leq a b) =
  intersection_from_list_acc_preserves_ordered k v leq transLeq total (to_list k v a) b (empty k v) (ordered_empty k v leq)

fn difference_from_list_acc_preserves_ordered_dispatch
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (e : Pair k v) (xs2 : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  (h : Ordered k v leq acc)
  (o : Or (Equal Bool (member k v leq (pair_fst k v e) reject) True)
          (Equal Bool (member k v leq (pair_fst k v e) reject) False))
  : Ordered k v leq (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc) =
  match o {
    Inl q ⇒
      J (λt _. Ordered k v leq t)
        (difference_from_list_acc_preserves_ordered k v leq transLeq total xs2 reject acc h)
        (sym (Tree k v)
          (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
          (difference_from_list_acc k v leq xs2 reject acc)
          (difference_from_list_acc_true_bridge k v leq e xs2 reject acc q)) ;
    Inr q ⇒
      J (λt _. Ordered k v leq t)
        (difference_from_list_acc_preserves_ordered k v leq transLeq total xs2 reject
          (insert k v leq (pair_fst k v e) (pair_snd k v e) acc)
          (preserves_ordered k v leq transLeq total (pair_fst k v e) (pair_snd k v e) acc h))
        (sym (Tree k v)
          (difference_from_list_acc k v leq (Cons (Pair k v) e xs2) reject acc)
          (difference_from_list_acc k v leq xs2 reject
            (insert k v leq (pair_fst k v e) (pair_snd k v e) acc))
          (difference_from_list_acc_false_bridge k v leq e xs2 reject acc q))
  }

fn difference_from_list_acc_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (xs : List (Pair k v)) (reject : Tree k v) (acc : Tree k v)
  : Ordered k v leq acc → Ordered k v leq (difference_from_list_acc k v leq xs reject acc) =
  match xs {
    Nil ⇒ λh. h ;
    Cons e xs2 ⇒
      λh.
        difference_from_list_acc_preserves_ordered_dispatch k v leq transLeq total e xs2 reject acc h
          (bool_dichotomy (member k v leq (pair_fst k v e) reject))
  }

fn difference_preserves_ordered
  (k : Type) (v : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (total : (x : k) → (y : k) → Or (Equal Bool (leq x y) True) (Equal Bool (leq y x) True))
  (a : Tree k v) (b : Tree k v)
  : Ordered k v leq (difference k v leq a b) =
  difference_from_list_acc_preserves_ordered k v leq transLeq total (to_list k v a) b (empty k v) (ordered_empty k v leq)
```

#### 4.7.11 Set operations — membership and algebraic laws

`set_union`/`set_intersection`/`set_difference` are the `Tree k Unit`
specializations of the three combinators above. `set_union_member_law`
characterizes membership as an `Or`; `set_intersection_member_law` and
`set_difference_member_law` characterize membership as an `And` (the
latter with a negation on the right side) — each built via the same
`_rhs`/`_case`/`_dispatch` chain shape. `set_union_comm_law`/`_assoc_law`/
`_idempotent_law`/`_identity_law` and the intersection mirrors
(`set_intersection_comm_law`/`_assoc_law`/`_idempotent_law`/
`_identity_law`) give the expected algebraic laws for both operations —
proved via `member`-extensionality against the lookup-table
characterizations above. `set_intersection_identity_law` states the same
idempotence property as its companion law; its name should not be read as an
identity-element claim.

```ken
fn set_union (k : Type) (leq : k → k → Bool) (s : Tree k Unit) (t : Tree k Unit) : Tree k Unit =
  union k Unit leq unit_combine s t

fn set_intersection (k : Type) (leq : k → k → Bool) (s : Tree k Unit) (t : Tree k Unit) : Tree k Unit =
  intersection k Unit leq s t

fn set_difference (k : Type) (leq : k → k → Bool) (s : Tree k Unit) (t : Tree k Unit) : Tree k Unit =
  difference k Unit leq s t

fn set_union_member_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit) (t : Tree k Unit)
  : Ordered k Unit leq s → Distinct k Unit leq s →
    Equal Bool
      (set_member k leq x (set_union k leq s t))
      (cat4_bool_or (set_member k leq x s) (set_member k leq x t)) =
  λhord. λhdist.
    trans Bool
      (set_member k leq x (set_union k leq s t))
      (option_is_some Unit
        (union_lookup_table Unit unit_combine
          (lookup k Unit leq x s)
          (lookup k Unit leq x t)))
      (cat4_bool_or (set_member k leq x s) (set_member k leq x t))
      (cong (Option Unit) Bool
        (lookup k Unit leq x (set_union k leq s t))
        (union_lookup_table Unit unit_combine
          (lookup k Unit leq x s)
          (lookup k Unit leq x t))
        (option_is_some Unit)
        (union_lookup_characterization k Unit leq reflLeq transLeq unit_combine x s t hord hdist))
      (union_lookup_table_member (lookup k Unit leq x s) (lookup k Unit leq x t))

fn set_intersection_member_left_false_rhs
  (k : Type) (leq : k → k → Bool) (x : k) (s : Tree k Unit) (t : Tree k Unit)
  (hLeft : Equal Bool (set_member k leq x s) False)
  : Equal Bool (bool_and (set_member k leq x s) (set_member k leq x t)) False =
  trans Bool
    (bool_and (set_member k leq x s) (set_member k leq x t))
    (bool_and False (set_member k leq x t))
    False
    (cong Bool Bool
      (set_member k leq x s)
      False
      (λleft. bool_and left (set_member k leq x t))
      hLeft)
    tt

fn set_intersection_member_right_false_rhs
  (k : Type) (leq : k → k → Bool) (x : k) (s : Tree k Unit) (t : Tree k Unit)
  (hLeft : Equal Bool (set_member k leq x s) True)
  (hRight : Equal Bool (set_member k leq x t) False)
  : Equal Bool (bool_and (set_member k leq x s) (set_member k leq x t)) False =
  trans Bool
    (bool_and (set_member k leq x s) (set_member k leq x t))
    (bool_and True (set_member k leq x t))
    False
    (cong Bool Bool
      (set_member k leq x s)
      True
      (λleft. bool_and left (set_member k leq x t))
      hLeft)
    (trans Bool
      (bool_and True (set_member k leq x t))
      (bool_and True False)
      False
      (cong Bool Bool
        (set_member k leq x t)
        False
        (λright. bool_and True right)
        hRight)
      tt)

fn set_intersection_member_both_true_rhs
  (k : Type) (leq : k → k → Bool) (x : k) (s : Tree k Unit) (t : Tree k Unit)
  (hLeft : Equal Bool (set_member k leq x s) True)
  (hRight : Equal Bool (set_member k leq x t) True)
  : Equal Bool (bool_and (set_member k leq x s) (set_member k leq x t)) True =
  trans Bool
    (bool_and (set_member k leq x s) (set_member k leq x t))
    (bool_and True (set_member k leq x t))
    True
    (cong Bool Bool
      (set_member k leq x s)
      True
      (λleft. bool_and left (set_member k leq x t))
      hLeft)
    (trans Bool
      (bool_and True (set_member k leq x t))
      (bool_and True True)
      True
      (cong Bool Bool
        (set_member k leq x t)
        True
        (λright. bool_and True right)
        hRight)
      tt)

fn set_intersection_member_left_false_case
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit) (t : Tree k Unit)
  (hord : Ordered k Unit leq s) (hdist : Distinct k Unit leq s)
  (hLeft : Equal Bool (set_member k leq x s) False)
  : Equal Bool
      (set_member k leq x (set_intersection k leq s t))
      (bool_and (set_member k leq x s) (set_member k leq x t)) =
  trans Bool
    (set_member k leq x (set_intersection k leq s t))
    False
    (bool_and (set_member k leq x s) (set_member k leq x t))
    (member_from_lookup_none k Unit leq x (set_intersection k leq s t)
      (intersection_lookup_left_none_law k leq transLeq x s t hord hdist hLeft))
    (sym Bool
      (bool_and (set_member k leq x s) (set_member k leq x t))
      False
      (set_intersection_member_left_false_rhs k leq x s t hLeft))

fn set_intersection_member_right_false_case
  (k : Type) (leq : k → k → Bool)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit) (t : Tree k Unit)
  (hLeft : Equal Bool (set_member k leq x s) True)
  (hRight : Equal Bool (set_member k leq x t) False)
  : Equal Bool
      (set_member k leq x (set_intersection k leq s t))
      (bool_and (set_member k leq x s) (set_member k leq x t)) =
  trans Bool
    (set_member k leq x (set_intersection k leq s t))
    False
    (bool_and (set_member k leq x s) (set_member k leq x t))
    (member_from_lookup_none k Unit leq x (set_intersection k leq s t)
      (intersection_lookup_characterization k Unit leq transLeq x s t
        (lookup_none_from_member_false k Unit leq x t hRight)))
    (sym Bool
      (bool_and (set_member k leq x s) (set_member k leq x t))
      False
      (set_intersection_member_right_false_rhs k leq x s t hLeft hRight))

fn set_intersection_member_both_true_case
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit) (t : Tree k Unit)
  (hord : Ordered k Unit leq s) (hdist : Distinct k Unit leq s)
  (hLeft : Equal Bool (set_member k leq x s) True)
  (hRight : Equal Bool (set_member k leq x t) True)
  : Equal Bool
      (set_member k leq x (set_intersection k leq s t))
      (bool_and (set_member k leq x s) (set_member k leq x t)) =
  trans Bool
    (set_member k leq x (set_intersection k leq s t))
    True
    (bool_and (set_member k leq x s) (set_member k leq x t))
    (member_from_lookup_some k Unit leq x (set_intersection k leq s t) MkUnit
      (intersection_lookup_some_law k Unit leq reflLeq transLeq x MkUnit s t hord hdist
        (lookup_unit_some_from_member_true k leq x s hLeft)
        hRight))
    (sym Bool
      (bool_and (set_member k leq x s) (set_member k leq x t))
      True
      (set_intersection_member_both_true_rhs k leq x s t hLeft hRight))

fn set_intersection_member_right_dispatch
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit) (t : Tree k Unit)
  (hord : Ordered k Unit leq s) (hdist : Distinct k Unit leq s)
  (hLeft : Equal Bool (set_member k leq x s) True)
  (oRight : Or (Equal Bool (set_member k leq x t) True) (Equal Bool (set_member k leq x t) False))
  : Equal Bool
      (set_member k leq x (set_intersection k leq s t))
      (bool_and (set_member k leq x s) (set_member k leq x t)) =
  match oRight {
    Inl hRight ⇒ set_intersection_member_both_true_case k leq reflLeq transLeq x s t hord hdist hLeft hRight ;
    Inr hRight ⇒ set_intersection_member_right_false_case k leq transLeq x s t hLeft hRight
  }

fn set_intersection_member_dispatch
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit) (t : Tree k Unit)
  (hord : Ordered k Unit leq s) (hdist : Distinct k Unit leq s)
  (oLeft : Or (Equal Bool (set_member k leq x s) True) (Equal Bool (set_member k leq x s) False))
  : Equal Bool
      (set_member k leq x (set_intersection k leq s t))
      (bool_and (set_member k leq x s) (set_member k leq x t)) =
  match oLeft {
    Inl hLeft ⇒ set_intersection_member_right_dispatch k leq reflLeq transLeq x s t hord hdist hLeft
      (bool_dichotomy (set_member k leq x t)) ;
    Inr hLeft ⇒ set_intersection_member_left_false_case k leq transLeq x s t hord hdist hLeft
  }

fn set_intersection_member_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit) (t : Tree k Unit)
  : Ordered k Unit leq s → Distinct k Unit leq s →
    Equal Bool
      (set_member k leq x (set_intersection k leq s t))
      (bool_and (set_member k leq x s) (set_member k leq x t)) =
  λhord. λhdist.
    set_intersection_member_dispatch k leq reflLeq transLeq x s t hord hdist
      (bool_dichotomy (set_member k leq x s))

fn set_difference_member_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit) (t : Tree k Unit)
  : Ordered k Unit leq s → Distinct k Unit leq s →
    Equal Bool
      (set_member k leq x (set_difference k leq s t))
      (bool_and (set_member k leq x s) (bool_not (set_member k leq x t))) =
  λhord. λhdist.
    trans Bool
      (set_member k leq x (set_difference k leq s t))
      (option_is_some Unit (difference_lookup_expected k Unit leq x s t))
      (bool_and (set_member k leq x s) (bool_not (set_member k leq x t)))
      (cong (Option Unit) Bool
        (lookup k Unit leq x (set_difference k leq s t))
        (difference_lookup_expected k Unit leq x s t)
        (option_is_some Unit)
        (difference_lookup_characterization k Unit leq reflLeq transLeq x s t hord hdist))
      (difference_lookup_expected_member k leq x s t)

fn set_member_empty_false
  (k : Type) (leq : k → k → Bool) (x : k)
  : Equal Bool (set_member k leq x (empty k Unit)) False =
  member_from_lookup_none k Unit leq x (empty k Unit)
    (lookup_empty_is_none k Unit leq x)

fn set_union_comm_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit) (t : Tree k Unit)
  : Ordered k Unit leq s → Distinct k Unit leq s →
    Ordered k Unit leq t → Distinct k Unit leq t →
    Equal Bool
      (set_member k leq x (set_union k leq s t))
      (set_member k leq x (set_union k leq t s)) =
  λhordS. λhdistS. λhordT. λhdistT.
    trans Bool
      (set_member k leq x (set_union k leq s t))
      (cat4_bool_or (set_member k leq x s) (set_member k leq x t))
      (set_member k leq x (set_union k leq t s))
      (set_union_member_law k leq reflLeq transLeq x s t hordS hdistS)
      (trans Bool
        (cat4_bool_or (set_member k leq x s) (set_member k leq x t))
        (cat4_bool_or (set_member k leq x t) (set_member k leq x s))
        (set_member k leq x (set_union k leq t s))
        (cat4_bool_or_comm (set_member k leq x s) (set_member k leq x t))
        (sym Bool
          (set_member k leq x (set_union k leq t s))
          (cat4_bool_or (set_member k leq x t) (set_member k leq x s))
          (set_union_member_law k leq reflLeq transLeq x t s hordT hdistT)))

fn set_union_assoc_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (a : Tree k Unit) (b : Tree k Unit) (c : Tree k Unit)
  : Ordered k Unit leq a → Distinct k Unit leq a →
    Ordered k Unit leq b → Distinct k Unit leq b →
    Ordered k Unit leq (set_union k leq a b) → Distinct k Unit leq (set_union k leq a b) →
    Equal Bool
      (set_member k leq x (set_union k leq (set_union k leq a b) c))
      (set_member k leq x (set_union k leq a (set_union k leq b c))) =
  λhordA. λhdistA. λhordB. λhdistB. λhordAB. λhdistAB.
    trans Bool
      (set_member k leq x (set_union k leq (set_union k leq a b) c))
      (cat4_bool_or (set_member k leq x (set_union k leq a b)) (set_member k leq x c))
      (set_member k leq x (set_union k leq a (set_union k leq b c)))
      (set_union_member_law k leq reflLeq transLeq x (set_union k leq a b) c hordAB hdistAB)
      (trans Bool
        (cat4_bool_or (set_member k leq x (set_union k leq a b)) (set_member k leq x c))
        (cat4_bool_or (cat4_bool_or (set_member k leq x a) (set_member k leq x b)) (set_member k leq x c))
        (set_member k leq x (set_union k leq a (set_union k leq b c)))
        (cong Bool Bool
          (set_member k leq x (set_union k leq a b))
          (cat4_bool_or (set_member k leq x a) (set_member k leq x b))
          (λleft. cat4_bool_or left (set_member k leq x c))
          (set_union_member_law k leq reflLeq transLeq x a b hordA hdistA))
        (trans Bool
          (cat4_bool_or (cat4_bool_or (set_member k leq x a) (set_member k leq x b)) (set_member k leq x c))
          (cat4_bool_or (set_member k leq x a) (cat4_bool_or (set_member k leq x b) (set_member k leq x c)))
          (set_member k leq x (set_union k leq a (set_union k leq b c)))
          (cat4_bool_or_assoc (set_member k leq x a) (set_member k leq x b) (set_member k leq x c))
          (trans Bool
            (cat4_bool_or (set_member k leq x a) (cat4_bool_or (set_member k leq x b) (set_member k leq x c)))
            (cat4_bool_or (set_member k leq x a) (set_member k leq x (set_union k leq b c)))
            (set_member k leq x (set_union k leq a (set_union k leq b c)))
            (cong Bool Bool
              (cat4_bool_or (set_member k leq x b) (set_member k leq x c))
              (set_member k leq x (set_union k leq b c))
              (λright. cat4_bool_or (set_member k leq x a) right)
              (sym Bool
                (set_member k leq x (set_union k leq b c))
                (cat4_bool_or (set_member k leq x b) (set_member k leq x c))
                (set_union_member_law k leq reflLeq transLeq x b c hordB hdistB)))
            (sym Bool
              (set_member k leq x (set_union k leq a (set_union k leq b c)))
              (cat4_bool_or (set_member k leq x a) (set_member k leq x (set_union k leq b c)))
              (set_union_member_law k leq reflLeq transLeq x a (set_union k leq b c) hordA hdistA)))))

fn set_union_idempotent_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit)
  : Ordered k Unit leq s → Distinct k Unit leq s →
    Equal Bool
      (set_member k leq x (set_union k leq s s))
      (set_member k leq x s) =
  λhord. λhdist.
    trans Bool
      (set_member k leq x (set_union k leq s s))
      (cat4_bool_or (set_member k leq x s) (set_member k leq x s))
      (set_member k leq x s)
      (set_union_member_law k leq reflLeq transLeq x s s hord hdist)
      (cat4_bool_or_idempotent (set_member k leq x s))

fn set_union_identity_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit)
  : Ordered k Unit leq s → Distinct k Unit leq s →
    And
      (Equal Bool (set_member k leq x (set_union k leq (empty k Unit) s)) (set_member k leq x s))
      (Equal Bool (set_member k leq x (set_union k leq s (empty k Unit))) (set_member k leq x s)) =
  λhord. λhdist.
    and_intro
      (Equal Bool (set_member k leq x (set_union k leq (empty k Unit) s)) (set_member k leq x s))
      (Equal Bool (set_member k leq x (set_union k leq s (empty k Unit))) (set_member k leq x s))
      (trans Bool
        (set_member k leq x (set_union k leq (empty k Unit) s))
        (cat4_bool_or (set_member k leq x (empty k Unit)) (set_member k leq x s))
        (set_member k leq x s)
        (set_union_member_law k leq reflLeq transLeq x (empty k Unit) s
          (ordered_empty k Unit leq)
          (distinct_empty k Unit leq))
        (trans Bool
          (cat4_bool_or (set_member k leq x (empty k Unit)) (set_member k leq x s))
          (cat4_bool_or False (set_member k leq x s))
          (set_member k leq x s)
          (cong Bool Bool
            (set_member k leq x (empty k Unit))
            False
            (λleft. cat4_bool_or left (set_member k leq x s))
            (set_member_empty_false k leq x))
          (cat4_bool_or_left_identity (set_member k leq x s))))
      (trans Bool
        (set_member k leq x (set_union k leq s (empty k Unit)))
        (cat4_bool_or (set_member k leq x s) (set_member k leq x (empty k Unit)))
        (set_member k leq x s)
        (set_union_member_law k leq reflLeq transLeq x s (empty k Unit) hord hdist)
        (trans Bool
          (cat4_bool_or (set_member k leq x s) (set_member k leq x (empty k Unit)))
          (cat4_bool_or (set_member k leq x s) False)
          (set_member k leq x s)
          (cong Bool Bool
            (set_member k leq x (empty k Unit))
            False
            (λright. cat4_bool_or (set_member k leq x s) right)
            (set_member_empty_false k leq x))
          (cat4_bool_or_right_identity (set_member k leq x s))))

fn set_intersection_comm_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit) (t : Tree k Unit)
  : Ordered k Unit leq s → Distinct k Unit leq s →
    Ordered k Unit leq t → Distinct k Unit leq t →
    Equal Bool
      (set_member k leq x (set_intersection k leq s t))
      (set_member k leq x (set_intersection k leq t s)) =
  λhordS. λhdistS. λhordT. λhdistT.
    trans Bool
      (set_member k leq x (set_intersection k leq s t))
      (bool_and (set_member k leq x s) (set_member k leq x t))
      (set_member k leq x (set_intersection k leq t s))
      (set_intersection_member_law k leq reflLeq transLeq x s t hordS hdistS)
      (trans Bool
        (bool_and (set_member k leq x s) (set_member k leq x t))
        (bool_and (set_member k leq x t) (set_member k leq x s))
        (set_member k leq x (set_intersection k leq t s))
        (bool_and_comm (set_member k leq x s) (set_member k leq x t))
        (sym Bool
          (set_member k leq x (set_intersection k leq t s))
          (bool_and (set_member k leq x t) (set_member k leq x s))
          (set_intersection_member_law k leq reflLeq transLeq x t s hordT hdistT)))

fn set_intersection_assoc_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (a : Tree k Unit) (b : Tree k Unit) (c : Tree k Unit)
  : Ordered k Unit leq a → Distinct k Unit leq a →
    Ordered k Unit leq b → Distinct k Unit leq b →
    Ordered k Unit leq (set_intersection k leq a b) → Distinct k Unit leq (set_intersection k leq a b) →
    Equal Bool
      (set_member k leq x (set_intersection k leq (set_intersection k leq a b) c))
      (set_member k leq x (set_intersection k leq a (set_intersection k leq b c))) =
  λhordA. λhdistA. λhordB. λhdistB. λhordAB. λhdistAB.
    trans Bool
      (set_member k leq x (set_intersection k leq (set_intersection k leq a b) c))
      (bool_and (set_member k leq x (set_intersection k leq a b)) (set_member k leq x c))
      (set_member k leq x (set_intersection k leq a (set_intersection k leq b c)))
      (set_intersection_member_law k leq reflLeq transLeq x (set_intersection k leq a b) c hordAB hdistAB)
      (trans Bool
        (bool_and (set_member k leq x (set_intersection k leq a b)) (set_member k leq x c))
        (bool_and (bool_and (set_member k leq x a) (set_member k leq x b)) (set_member k leq x c))
        (set_member k leq x (set_intersection k leq a (set_intersection k leq b c)))
        (cong Bool Bool
          (set_member k leq x (set_intersection k leq a b))
          (bool_and (set_member k leq x a) (set_member k leq x b))
          (λleft. bool_and left (set_member k leq x c))
          (set_intersection_member_law k leq reflLeq transLeq x a b hordA hdistA))
        (trans Bool
          (bool_and (bool_and (set_member k leq x a) (set_member k leq x b)) (set_member k leq x c))
          (bool_and (set_member k leq x a) (bool_and (set_member k leq x b) (set_member k leq x c)))
          (set_member k leq x (set_intersection k leq a (set_intersection k leq b c)))
          (bool_and_assoc (set_member k leq x a) (set_member k leq x b) (set_member k leq x c))
          (trans Bool
            (bool_and (set_member k leq x a) (bool_and (set_member k leq x b) (set_member k leq x c)))
            (bool_and (set_member k leq x a) (set_member k leq x (set_intersection k leq b c)))
            (set_member k leq x (set_intersection k leq a (set_intersection k leq b c)))
            (cong Bool Bool
              (bool_and (set_member k leq x b) (set_member k leq x c))
              (set_member k leq x (set_intersection k leq b c))
              (λright. bool_and (set_member k leq x a) right)
              (sym Bool
                (set_member k leq x (set_intersection k leq b c))
                (bool_and (set_member k leq x b) (set_member k leq x c))
                (set_intersection_member_law k leq reflLeq transLeq x b c hordB hdistB)))
            (sym Bool
              (set_member k leq x (set_intersection k leq a (set_intersection k leq b c)))
              (bool_and (set_member k leq x a) (set_member k leq x (set_intersection k leq b c)))
              (set_intersection_member_law k leq reflLeq transLeq x a (set_intersection k leq b c) hordA hdistA)))))

fn set_intersection_idempotent_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit)
  : Ordered k Unit leq s → Distinct k Unit leq s →
    Equal Bool
      (set_member k leq x (set_intersection k leq s s))
      (set_member k leq x s) =
  λhord. λhdist.
    trans Bool
      (set_member k leq x (set_intersection k leq s s))
      (bool_and (set_member k leq x s) (set_member k leq x s))
      (set_member k leq x s)
      (set_intersection_member_law k leq reflLeq transLeq x s s hord hdist)
      (bool_and_idempotent (set_member k leq x s))

fn set_intersection_identity_law
  (k : Type) (leq : k → k → Bool)
  (reflLeq : (x : k) → Equal Bool (leq x x) True)
  (transLeq : (x : k) → (y : k) → (z : k) → Equal Bool (leq x y) True → Equal Bool (leq y z) True → Equal Bool (leq x z) True)
  (x : k) (s : Tree k Unit)
  : Ordered k Unit leq s → Distinct k Unit leq s →
    Equal Bool
      (set_member k leq x (set_intersection k leq s s))
      (set_member k leq x s) =
  set_intersection_idempotent_law k leq reflLeq transLeq x s
```

#### 4.7.12 `keys`/`values` projections and a binary-relations library

`keys`/`values` project the two halves of `to_list`'s `Pair k v` entries;
`keys_project_to_list`/`values_project_to_list`/
`keys_values_projection_coherence` connect the two projections back to
`to_list`, and `keys_ascending` reuses Law 4's `to_list_ordered` (§4.2) to
show `keys` is sorted over an `Ordered` tree.

The package closes with a small binary-relations library represented as
`Tree k (Tree k Unit)` — an adjacency map from a key to the set of its
successors: `succ`/`rel_member`/`add_edge` for the raw relation,
`compose`/`converse` for relational composition and reversal, and
`is_reflexive`/`is_symmetric`/`is_transitive`/`is_equivalence` as the
standard relation-property predicates stated directly against `rel_member`.

Transitive closure is intentionally design-now/defer-build: it is to be
represented as bounded reachability (`IsTrue (reachableWithin N x y)`) once
`size` and bounded iteration land. This package deliberately does not
define a raw proof-relevant `data ... : Ω` closure.

```ken
fn pair_vals (k : Type) (v : Type) (xs : List (Pair k v)) : List v =
  match xs {
    Nil ⇒ Nil v ;
    Cons p xs2 ⇒ Cons v (pair_snd k v p) (pair_vals k v xs2)
  }

fn pair_keys_preserves_sorted_cons
  (k : Type) (v : Type) (leq : k → k → Bool) (e : Pair k v) (xs2 : List (Pair k v))
  : is_sorted (Pair k v) (pair_leq k v leq) (Cons (Pair k v) e xs2) →
    is_sorted k leq (pair_keys k v (Cons (Pair k v) e xs2)) =
  match xs2 {
    Nil ⇒ λh. tt ;
    Cons e2 xs3 ⇒
      λh.
        and_intro (Equal Bool (leq (pair_fst k v e) (pair_fst k v e2)) True)
                 (is_sorted k leq (pair_keys k v (Cons (Pair k v) e2 xs3)))
          (and_fst (Equal Bool (pair_leq k v leq e e2) True)
                  (is_sorted (Pair k v) (pair_leq k v leq) (Cons (Pair k v) e2 xs3))
                  h)
          (pair_keys_preserves_sorted_cons k v leq e2 xs3
            (and_snd (Equal Bool (pair_leq k v leq e e2) True)
                    (is_sorted (Pair k v) (pair_leq k v leq) (Cons (Pair k v) e2 xs3))
                    h))
  }

fn pair_keys_preserves_sorted
  (k : Type) (v : Type) (leq : k → k → Bool) (xs : List (Pair k v))
  : is_sorted (Pair k v) (pair_leq k v leq) xs → is_sorted k leq (pair_keys k v xs) =
  match xs {
    Nil ⇒ λh. tt ;
    Cons e xs2 ⇒
      pair_keys_preserves_sorted_cons k v leq e xs2
  }

fn keys (k : Type) (v : Type) (m : Tree k v) : List k =
  pair_keys k v (to_list k v m)

fn values (k : Type) (v : Type) (m : Tree k v) : List v =
  pair_vals k v (to_list k v m)

fn keys_project_to_list (k : Type) (v : Type) (m : Tree k v)
  : Equal (List k) (keys k v m) (pair_keys k v (to_list k v m)) =
  Refl

fn values_project_to_list (k : Type) (v : Type) (m : Tree k v)
  : Equal (List v) (values k v m) (pair_vals k v (to_list k v m)) =
  Refl

fn keys_values_projection_coherence (k : Type) (v : Type) (m : Tree k v)
  : And
      (Equal (List k) (keys k v m) (pair_keys k v (to_list k v m)))
      (Equal (List v) (values k v m) (pair_vals k v (to_list k v m))) =
  and_intro
    (Equal (List k) (keys k v m) (pair_keys k v (to_list k v m)))
    (Equal (List v) (values k v m) (pair_vals k v (to_list k v m)))
    (keys_project_to_list k v m)
    (values_project_to_list k v m)

fn keys_ascending
  (k : Type) (v : Type) (leq : k → k → Bool) (m : Tree k v)
  : Ordered k v leq m → is_sorted k leq (keys k v m) =
  λh. pair_keys_preserves_sorted k v leq (to_list k v m) (to_list_ordered k v leq m h)

fn succ (k : Type) (leq : k → k → Bool) (x : k) (r : Tree k (Tree k Unit)) : Tree k Unit =
  match lookup k (Tree k Unit) leq x r {
    None ⇒ empty k Unit ;
    Some s ⇒ s
  }

fn rel_member (k : Type) (leq : k → k → Bool) (x : k) (y : k) (r : Tree k (Tree k Unit)) : Prop =
  Equal Bool (set_member k leq y (succ k leq x r)) True

fn add_edge (k : Type) (leq : k → k → Bool) (x : k) (y : k) (r : Tree k (Tree k Unit)) : Tree k (Tree k Unit) =
  insert k (Tree k Unit) leq x (set_insert k leq y (succ k leq x r)) r

fn compose_succ_step
  (k : Type) (leq : k → k → Bool) (s : Tree k (Tree k Unit)) (y : k) (u : Unit) (acc : Tree k Unit) : Tree k Unit =
  set_union k leq acc (succ k leq y s)

fn compose_succ (k : Type) (leq : k → k → Bool) (rSucc : Tree k Unit) (s : Tree k (Tree k Unit)) : Tree k Unit =
  fold k Unit (Tree k Unit) (compose_succ_step k leq s) (empty k Unit) rSucc

fn compose (k : Type) (leq : k → k → Bool) (r : Tree k (Tree k Unit)) (s : Tree k (Tree k Unit)) : Tree k (Tree k Unit) =
  fold k (Tree k Unit) (Tree k (Tree k Unit))
    (λx.λtargets.λacc. insert k (Tree k Unit) leq x (compose_succ k leq targets s) acc)
    (empty k (Tree k Unit)) r

fn converse_targets
  (k : Type) (leq : k → k → Bool) (x : k) (targets : Tree k Unit) (acc : Tree k (Tree k Unit)) : Tree k (Tree k Unit) =
  fold k Unit (Tree k (Tree k Unit)) (λy.λu.λacc2. add_edge k leq y x acc2) acc targets

fn converse (k : Type) (leq : k → k → Bool) (r : Tree k (Tree k Unit)) : Tree k (Tree k Unit) =
  fold k (Tree k Unit) (Tree k (Tree k Unit)) (λx.λtargets.λacc. converse_targets k leq x targets acc) (empty k (Tree k Unit)) r

fn is_reflexive (k : Type) (leq : k → k → Bool) (r : Tree k (Tree k Unit)) : Prop =
  (x : k) → rel_member k leq x x r

fn is_symmetric (k : Type) (leq : k → k → Bool) (r : Tree k (Tree k Unit)) : Prop =
  (x : k) → (y : k) → rel_member k leq x y r → rel_member k leq y x r

fn is_transitive (k : Type) (leq : k → k → Bool) (r : Tree k (Tree k Unit)) : Prop =
  (x : k) → (y : k) → (z : k) → rel_member k leq x y r → rel_member k leq y z r → rel_member k leq x z r

fn is_equivalence (k : Type) (leq : k → k → Bool) (r : Tree k (Tree k Unit)) : Prop =
  And (is_reflexive k leq r) (And (is_symmetric k leq r) (is_transitive k leq r))

```

## 5. Design notes

**Structural induction.** Recursive proofs call themselves on the relevant
subtree; that returned proof is the induction hypothesis used by the parent
case. This follows the tree's own structural recursion and keeps each proof
local to the operation it justifies.

**Convoy-style hypotheses.** A match's return type remains independent of
its scrutinee. Hypotheses whose types mention the scrutinee are curried after
the result type and bound inside each arm, allowing the selected constructor
to refine them without a separate dependent motive.

**Transport through comparison dispatch.** Insertion and lookup make nested
decisions from comparator results. Their proofs transport the goal to the
branch selected by a comparison, stopping at the last still-stuck match
redex. This lets ordinary conversion finish the final reduction and avoids
constructing unnecessary equalities between fully expanded tree nodes.

**Explicit ordering evidence.** Comparator operations and their laws are
passed as parameters. This preserves genericity while keeping the exact
ordering assumptions visible in every theorem that needs them. Bounded
reachability is intentionally left to a future bounded-iteration interface.

## 6. References

- **Binary search trees** — Wikipedia,
  <https://en.wikipedia.org/wiki/Binary_search_tree> — general orientation
  on the unbalanced-BST carrier and its ordering invariant.
- **Purely functional data structures** — Chris Okasaki, Cambridge
  University Press, 1998 — the standard reference for persistent,
  purely-functional map/set implementations over trees; this package's
  carrier is the simplest member of that family (no balancing).
- **Introduction to Objectual Type Theory / dependent pattern matching
  literature** — for readers porting from Coq/Agda/Lean: the "convoy
  pattern" name and shape used throughout [§4](#4-laws--proofs) and
  [§5](#5-design-notes) is a direct analogue of the well-known Coq
  `refine`/convoy pattern for avoiding dependent-motive inference failures
  by currying scrutinee-dependent hypotheses after the match.

## 7. Trust & derivation

**Public API (stable names):** `Tree`, `empty`, `to_list`, `fold`,
`insert`, `lookup`, `member`, `from_list`, `Set` projections
(`set_insert`/`set_member`/`set_to_list`), `Ordered`, the five capstone
laws (`preserves_ordered`, `lookup_found_after_insert`, `lookup_locality`,
`to_list_ordered`, `lookup_assoc_agree`), `delete`, `insert_with`,
`union`/`intersection`/`difference` and their `Set`-level wrappers,
`keys`/`values`, and the binary-relations combinators
(`succ`/`compose`/`converse`/`is_equivalence`).

**Source map:**

| Reader task | Section |
|---|---|
| Understand the carrier and basic operations | [§2](#2-definition) |
| See how the package composes / who reaches for what | [§3](#3-using-it) |
| Find a specific law's statement and proof | [§4.1](#41-capstone-preliminaries)–[§4.6](#46-law-5--lookup_assoc_agree-dictionary-agreement-with-the-ordered-list-lookup) |
| Find a keyed-collection operation (`delete`/`union`/…) | [§4.7](#47-layer-2-keyed-collection-operations) |
| Understand the recurring proof idioms | [§5](#5-design-notes) |

**Derivation path from built-ins.** `Tree` is a checked inductive data type.
Every operation is an ordinary checked definition; `Or`/`Inl`/`Inr` are
kernel-checked inductive data, not native primitives.

**`trusted_base()` delta: zero**, throughout the capstone and the keyed
operations. No `Axiom` appears in this file; every recursive operation is
pure and termination checked.

**Proof families.** Base-case transport (`insert_case_transport_*`/
`insert_with_transport_*`, the stop-one-step-short bridges, §4.3/§5) →
comparison-independent structural induction
(`insert_preserves_all_keys`/`all_keys_trans_*`, §4.3) → the convoy-idiom
recursive assembly (every law's own top-level `fn`, §4.1–§4.6, §4.7.5–§4.7.10)
→ `member`-extensionality against a lookup-table characterization (the
`Set`-level algebraic laws, §4.7.11).

**Consumers.** Programs that need ordered maps, sets, or finite relations
can use the capstone laws and keyed operations directly.

**Validation evidence.** `ken check` elaborates this entry's tangled source
fences; the catalog checks its capstone laws and keyed operations.
