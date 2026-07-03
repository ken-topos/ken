# Proof skeletons for the five inductive `Map` laws (`map-verified-laws`)

> Status: **DRAFT v0 (`map-verified-laws` WP, Map-arc capstone).** The
> proof-strategy elaboration for the five inductive correctness laws that
> `52-map.md §7d` deferred. Both enabling capabilities are now **landed** — Gap
> A (`surface-transport`, the `J` former + `packages/transport/transport.ken`,
> `19955d8`) and Gap B (`dependent-match-nonnullary`, the widened non-indexed
> dependent-match gate, `282856c`) — so the five laws are now **buildable**.
> This doc is the **skeleton** (induction target, the transport at each stuck
> `leq`, the base witness per branch, the helper lemmas), for **Foundation** to
> fill in as real proof terms in `packages/collections/map.ken`. It adds
> **nothing** to `trusted_base()`: every proof reduces through the existing
> `Term::J`/`Term::Cast` (transport) and `Term::Elim` (dependent match) — no
> `declare_primitive`, no `declare_postulate`, **no `Axiom`**. A law that turns
> out not to be honestly buildable **re-defers to Steward** (§6), never
> postulated.

## 1. What this module is

`52-map.md §5` proves the `Map` invariant and operation-correctness laws as
**real proof terms**. Two of the seven — `Ordered empty` and `lookup k empty =
None` — are **non-inductive** (Branch A) and ship in `map.ken` already. The
remaining **five inductive laws** (Branch B) each **induct over the non-nullary
`Tree` carrier**, and four of them additionally must **align a stuck `leq k
k'`** — the two capability walls `52 §5` named. Both walls are down; this doc
elaborates the proof strategy so a build model does not reinvent it from scratch
(the WP's own rationale for routing through the enclave first).

| # | Law (`52` ref) | Statement | Walls |
|---|---|---|---|
| 1 | preservation (`§5.1`) | `Ordered m → Ordered (insert leq key val m)` | A + B |
| 2 | found-after-insert (`§5.2`) | `lookup leq key (insert leq key val m) = Some val` | A + B |
| 3 | locality (`§5.2`) | `distinct key key' → lookup leq key' (insert leq key val m) = lookup leq key' m` | A + B |
| 4 | `toList`-ordered (`§5.3`) | `Ordered m → isSorted keyLeq (toList m)` — **comparison-free** | **B only** |
| 5 | agreement (`§5.3`) | `lookup leq key m = assoc leq key (toList m)` | A + B |

**None of the five uses `antisym → Equal`.** The core laws lean on
`refl`/`trans`/`total` only (`52 §5.2`/`§2.1` blast-radius localization); the
one `antisym`-dependent face — overwrite/uniqueness — is the separate,
**canonicity-gated** law that stays deferred under ADR 0010 (`52 §5.3`), out of
this WP.

All spellings below are the **landed** `map.ken` idiom (`52 §2`, the
C5-verified-sort unbundled encoding): every op/law takes `leq` and its laws as
**separate bare parameters**, and `IsTrue b := Equal Bool b True` (the landed
`packages/lawful-classes` bridge). The order-law parameters, verbatim from the
`Ord` class (`packages/lawful-classes/lawful_classes.ken`):

```
leq   : k -> k -> Bool
refl  : (x : k) -> IsTrue (leq x x)
trans : (x : k) -> (y : k) -> (z : k) -> IsTrue (leq x y) -> IsTrue (leq y z) -> IsTrue (leq x z)
total : (x : k) -> (y : k) -> IsTrue (bool_or (leq x y) (leq y x))
```

## 2. The two composed mechanisms (grounded against the landed idioms)

### 2.1 Induction over `Tree` — the Gap-B dependent match

A proof by induction on `m : Tree k v` is a dependent `match` whose motive is
the **whole goal as a function of the scrutinee** (`../30-surface/34 §3.2`,
`dependent-match-nonnullary`). The landed idiom
(`dependent_match_nonnullary_acceptance.rs` AC1): a `view` whose result type is
a **dependent function of the scrutinee**, matched arm-by-arm, each arm binding
the per-branch-narrowed hypothesis with `\h.`:

```
view preservation (k v : Type) (leq : ...) (refl : ...) (trans : ...) (total : ...)
                  (key : k) (val : v) (m : Tree k v)
  : Ordered k v leq m -> Ordered k v leq (insert k v leq key val m) =
  match m {
    Leaf                => \h. <base : goal at Leaf> ;
    Node l k2 v2 r      => \h. <step : goal at Node, using ih_l, ih_r, h>
  }
```

Three facts about this idiom are load-bearing (all grounded in the landed gate,
`elab.rs` `dependent_eligible` at **`:535-553`**, `ind.indices.is_empty()`):

- **The scrutinee must be a bare variable** (`elab.rs:539`, `Term::Var(_)`) of a
  **flat, non-indexed** family (`Tree`/`List` qualify). `match m` where `m` is
  the `view`'s own parameter is exactly this shape.
- **Per-branch narrowing.** In the `Node` arm the goal's every occurrence of `m`
  is replaced by the **reconstructed** `Node l k2 v2 r`, so both the hypothesis
  `h`'s type (`Ordered … (Node l k2 v2 r)`) **and** the codomain (`Ordered …
  (insert … (Node l k2 v2 r))`) narrow together.
- **`Node` yields two IH slots** (`l` and `r` are both recursive fields), each
  the **motive applied to that field**: `ih_l : Ordered … l → Ordered … (insert
  … l)` and `ih_r : Ordered … r → Ordered … (insert … r)`. They bind **before**
  the arm's own `\h.` (order: `l k2 r ih_l ih_r h`, `method_type`'s layout).

The base case (`Leaf`) is where the goal bottoms out with **no** induction —
its witness is `tt`/`refl` (§2.3).

### 2.2 Reflecting a stuck `leq` into an equation, then transporting — Gap A

At an interior `Node`, `insert`/`lookup` branch on `leq key k2`, which is
**irreducibly stuck** (both keys are variables). Two steps discharge it.

**Step 1 — reflect the stuck Boolean into a usable equation.** The Gap-B gate
requires a **variable** scrutinee, so `match (leq key k2) { … }` does **not**
narrow (the scrutinee is an application). Instead, reflect the Boolean through a
small reusable combinator that **inducts on a Bool variable** and returns the
two equation-carrying cases (§3, `boolDichotomy`):

```
boolDichotomy (leq key k2)
  : Or (Equal Bool (leq key k2) True) (Equal Bool (leq key k2) False)
```

Then a **non-dependent** `match` on that `Or` (constant motive → the landed
`infer_match` path, no Gap B needed) delivers, in each arm, the equation
`q : Equal Bool (leq key k2) True` (resp. `False`).

**Step 2 — transport the stuck goal with `J`.** With `q` in hand, fire the
stuck comparison by the landed transport idiom
(`surface_transport_acceptance.rs` AC3, `53-transport.md §3`) — a **hand-written
motive** `\x _. G[x]` that abstracts the goal over the value of `leq key k2`:

```
J (\x _. G[x]) (baseTrue : G[True]) (sym Bool (leq key k2) True q)
  : G[leq key k2]
```

where `G[x]` is the goal with the stuck `leq key k2` replaced by `x`, and
`G[True]` is the **reduced** branch goal (with the comparison forced to `True`,
so `insert`/`lookup` fire). `sym` flips `q : Equal Bool (leq key k2) True` to
`Equal Bool True (leq key k2)` so the transport runs from the concrete endpoint
(`True`, where the branch reduces and `baseTrue` is provable) to the stuck one.
Because the motive is **user-written**, it abstracts exactly the intended
occurrences — no auto-generalization hazard (`53 §2`, the Agda-`subst` posture).

The whole per-comparison move is therefore:

```
match (boolDichotomy (leq key k2)) {
  Inl q => J (\x _. G[x]) baseTrue  (sym Bool (leq key k2) True  q) ;
  Inr q => J (\x _. G[x]) baseFalse (sym Bool (leq key k2) False q)
}
```

**When the equation is free — no reflect needed.** Where the two compared keys
**coincide** (found-after-insert's placed node compares `leq key key`), the
equation is the dictionary law directly — `refl key : IsTrue (leq key key) =
Equal Bool (leq key key) True` — so Step 1 is skipped and `refl key` is fed
straight into the `J` transport (this is the `stuck_transport` acceptance test's
shape, with `q := refl key`).

### 2.3 Base-witness discipline — `tt` vs `Refl` (K7)

Every terminal equality obligation is discharged by one of two witnesses,
following the landed evidence (`lookupEmptyIsNone = tt`; the `surface-transport`
build retro):

- **`tt`** when the goal is an `IsTrue`/`Equal`-shaped proposition wrapping an
  **operation** that has now reduced, so the goal **observationally collapses to
  `Top`** (K7): `Equal Bool (op …) v` with `op … ⇝ v`, e.g. `Equal Bool (leq
  key key) True` **after** `refl` forced it, or `Ordered … Leaf ⇝ Equal Bool
  True True ⇝ Top`. This is the `Refl`/`tt`/`absurd` idiom
  `packages/lawful-classes/lawful_classes.ken` already documents.
- **`Refl a`** when the goal is `Equal A a a` with both sides the **same
  already-reduced** term and no operation to collapse (used inside `cong`/`sym`
  in `transport.ken`).

Choosing `Refl` where the goal K7-collapses gives a confusing `TypeMismatch`
(the goal is already `Top`, not `Eq`) — prefer `tt` whenever an operation
reduced into the equality; reach for `Refl` only for a bare reflexive `Eq`.

## 3. Reusable helpers to define (Foundation)

These are **structural** (Gap-B-only or no-induction) and belong in
`map.ken`/`collections.ken` alongside the ops. Already landed: `list_append`,
`isSorted` (`prelude.rs` — `isSorted a leq (Cons x (Cons y r)) = And (Equal Bool
(leq x y) True) (isSorted a leq (Cons y r))`), `Pair`/`mkPair`/`pairFst`/
`pairSnd`, `absurd`/`Bottom`, `And`, `Equal`/`Refl`/`tt`.

- **`Or` / `Inl` / `Inr`** — the two-constructor sum, `data Or a b = Inl a | Inr
  b` (confirm none exists under another name first). Used only as
  `boolDichotomy`'s result envelope. **It must be `Type`-valued (proof-
  relevant), not `Ω`** — the whole point is to *case-split on which disjunct
  holds*, so `Inl`/`Inr` must be distinguishable; an `Ω`-valued `Or` would make
  them proof-irrelevantly equal and the split would carry no information
  ([[proof-relevant-inductive-cannot-be-declared-at-omega]]). Eliminating this
  `Type`-valued `Or` (whose payloads are `Ω`-equalities) into the `Ω`-valued
  map goal is an ordinary case analysis — the permitted `Type → Ω` motive
  direction, not the restricted large elimination.
- **`boolDichotomy`** — the reflect combinator (§2.2), a one-line Gap-B
  dependent match on a Bool variable:

  ```
  view boolDichotomy (b : Bool) : Or (Equal Bool b True) (Equal Bool b False) =
    match b { True => Inl (Equal Bool b True) (Equal Bool b False) tt ;
              False => Inr (Equal Bool b True) (Equal Bool b False) tt }
  ```

  Each arm's narrowed goal is `Or (Equal Bool True True) …` (resp. `False`), so
  the `Inl`/`Inr` payload is the K7-collapsed `tt`. (Type args to `Or`/`Inl`/
  `Inr` elided above for readability; spell them per the landed explicit-arg
  convention.)
- **`assoc`** — the ordered-list lookup for law 5: `assoc leq key : List (Pair k
  v) → Option v`, scanning by `leq key (pairFst entry)`. A plain structural
  `List` recursion (Gap-B-free — no dependent motive).
- **`allInList`** — the list analogue of `allKeys` for law 4's bridge lemma:
  `allInList p : List (Pair k v) → Prop`, the `Ω`-conjunction of `p` over the
  list (mirrors `allKeys`, lifted to the flattened list).

## 4. Per-law proof skeletons

Notation: `G` is the current goal; `G[x]` is `G` with the named stuck `leq …`
abstracted to `x`; `keyLeq := \a b. leq (pairFst k v a) (pairFst k v b)` is
law 4/5's element comparator over `Pair k v`.

### 4.1 Law 1 — preservation (`Ordered m → Ordered (insert key val m)`)

Induct on `m` (§2.1). Motive `\m'. (Ordered … m' → Ordered … (insert … m'))`.

- **`Leaf`** — `insert … Leaf = Node Leaf key val Leaf`. Goal `Ordered … (Node
  Leaf key val Leaf)` unfolds to `And (allKeys (≤key) Leaf) (And (allKeys (≥key)
  Leaf) (And (Ordered Leaf) (Ordered Leaf)))`; every conjunct is `allKeys …
  Leaf ⇝ Top` / `Ordered Leaf ⇝ Top`. **Base witness:** the `And`-intro tree of
  `tt`s. **Comparison-free** — the `Leaf` case of preservation touches no
  `leq`.
- **`Node l k2 v2 r`** — `insert` branches on `leq key k2`, then (in the `True`
  case) on `leq k2 key`. Reflect `leq key k2` (§2.2). Three reduced branches:
  - **`leq key k2 = True`, `leq k2 key = True`** (overwrite) — result `Node l
    key val r`. `ih_l`/`ih_r` unused (subtrees unchanged). Rebuild the two
    `allKeys` bounds against the **new** label `key` from `h`'s bounds against
    `k2`, threading `trans` with the two equations (`leq key k2`, `leq k2 key`)
    to move a bound from `k2` to `key` — an **`allKeys`-under-a-transitive-
    step** lemma (§4.6, lemma L3).
  - **`leq key k2 = True`, `leq k2 key = False`** (descend left) — result `Node
    (insert … l) k2 v2 r`. Left subtree ordered by **`ih_l`**; right unchanged.
    New bound: every key of `insert … l` must stay `≤ k2`. From `h`'s `allKeys
    (≤k2) l` and the equation `leq key k2 = True`, an **`allKeys`-preserved-by-
    insert** lemma (§4.6, lemma L1) gives `allKeys (≤k2) (insert … l)`.
  - **`leq key k2 = False`** (descend right) — symmetric; right subtree by
    **`ih_r`**, and `total` + the equation give `leq k2 key = True` for the
    `≥k2` bound via lemma L1 mirrored.

  Each branch's goal is reached by `J`-transporting the stuck `insert` (§2.2)
  and rebuilding the `Ordered` conjunction. **Dictionary laws:** `trans`,
  `total`. **No `antisym`.**

### 4.2 Law 2 — found-after-insert (`lookup key (insert key val m) = Some val`)

No `Ordered` hypothesis (holds for any tree — `lookup` retraces `insert`'s
exact path). Induct on `m`. Motive `\m'. Equal (Option v) (lookup … key (insert
… key val m')) (Some val)`.

- **`Leaf`** — `insert … Leaf = Node Leaf key val Leaf`; `lookup key` on it
  compares `leq key key` twice. **Equation is free:** `q := refl key : Equal
  Bool (leq key key) True` — feed it into two `J` transports (outer and inner
  comparison), reducing to `Some val`. **Base witness:** `tt` (the goal
  K7-collapses to `Equal (Option v) (Some val) (Some val) ⇝ Top`).
- **`Node l k2 v2 r`** — reflect `leq key k2`. `insert` and `lookup` branch on
  the **identical** scrutinee, so each reflected equation reduces **both** their
  stuck matches consistently:
  - overwrite (`True`/`True`) — result node relabelled `key`; `lookup key` there
    hits `leq key key` (free `refl key`) → `Some val`. Base `tt`.
  - descend left (`True`/`False`) — both descend into `insert … l` / `l`; close
    by **`ih_l`**.
  - descend right (`False`) — both descend right; close by **`ih_r`**.

  **Dictionary laws:** `refl` only. **No `antisym`** (the value is `val`
  whichever label the node carries — `52 §5.2` law 2).

### 4.3 Law 3 — locality (insert leaves a distinct key's lookup unchanged)

`distinct key key' := (And (IsTrue (leq key key')) (IsTrue (leq key' key))) ->
Bottom` (order-distinctness; `absurd` eliminates the `Bottom`). Needs `Ordered
m`. Induct on `m`. Motive folds in both hypotheses (`Ordered … m' → distinct →
Equal (Option v) (lookup … key' (insert … m')) (lookup … key' m')`).

- **`Leaf`** — `insert … Leaf = Node Leaf key val Leaf`; `lookup key'` compares
  `leq key' key`. Reflect it; in each case `distinct` + the equation rule out a
  spurious hit (a `True`/`True` would contradict `distinct`, discharged by
  `absurd`), so both sides reduce to `None`. Base `tt`.
- **`Node l k2 v2 r`** — reflect **both** `leq key k2` (insert's branch) and
  `leq key' k2` (lookup's branch). The insert descends one way; `key'`'s lookup
  is unperturbed exactly when `key'` does **not** follow `key`'s inserted path.
  Use `trans`/`total` with `Ordered`'s subtree bounds and `distinct` to show:
  when both descend the **same** subtree, close by the **IH** for that subtree;
  when they **diverge**, the inserted `key` lies in a subtree `key'` never
  visits, so both sides are structurally equal (transport both stuck matches by
  their equations, then reflexivity). **Dictionary laws:** `trans`, `total` (and
  `distinct` as hypothesis). **No `antisym`.**

### 4.4 Law 4 — `toList`-ordered (`isSorted keyLeq (toList m)`) — Gap B only

**Comparison-free** — `toList` never calls `leq`; every `leq` fact is a stored
`IsTrue` witness threaded from `Ordered`'s `allKeys` conjuncts (which are
themselves `Equal Bool (leq …) True`, the exact shape `isSorted`'s conjuncts
want — they thread **directly**, no transport). So this law clears Gap A and
needs **only** the Gap-B induction. Induct on `m`. Motive `\m'. (Ordered … m' →
isSorted keyLeq (toList m'))`.

- **`Leaf`** — `toList Leaf = Nil`; `isSorted … Nil ⇝ Top`. Base `tt`.
- **`Node l k2 v2 r`** — `toList (Node …) = list_append (toList l) (Cons (k2,v2)
  (toList r))`. From **`ih_l`**/**`ih_r`**: `isSorted (toList l)`, `isSorted
  (toList r)`. From `h`'s `allKeys` conjuncts, via the bridge lemma L2 (§4.6):
  every key in `toList l` is `≤ k2`, and `k2 ≤` every key in `toList r`.
  Assemble with the append lemma L4 (§4.6):

  ```
  isSorted (toList l) ->
  isSorted (Cons (k2,v2) (toList r)) ->      -- from isSorted (toList r) + k2 ≤ head r
  (allInList (≤k2) (toList l)) ->            -- bridge L2 from allKeys
  isSorted (list_append (toList l) (Cons (k2,v2) (toList r)))
  ```

  **No dictionary laws** beyond the stored witnesses; **no transport.** This is
  the load-bearing ordered-iteration law — proved here as the naturally-`Ω`
  `isSorted` form, **never** permutation (§5).

### 4.5 Law 5 — agreement (`lookup key m = assoc key (toList m)`)

Needs `Ordered m` (so `assoc`'s linear scan of the sorted list agrees with
`lookup`'s tree descent). Induct on `m`. Motive `\m'. (Ordered … m' → Equal
(Option v) (lookup … key m') (assoc leq key (toList m')))`.

- **`Leaf`** — both sides `None` (`lookup … Leaf = None`; `assoc key Nil =
  None`). Base `tt`.
- **`Node l k2 v2 r`** — reflect `leq key k2`. `lookup` branches on it; `assoc`
  scans `list_append (toList l) (Cons (k2,v2) (toList r))`. Use `Ordered`'s
  bounds + `trans` to show the scan visits the same entry `lookup`'s descent
  selects: if `key < k2` (`leq key k2 = True`, `leq k2 key = False`), every key
  in `toList r` exceeds `key` (via L2 + `trans`), so `assoc` finds nothing right
  of `k2` and matches `lookup … l` by **`ih_l`**; symmetric for the right;
  overwrite matches at `k2`. Transport `lookup`'s stuck match by the reflected
  equations (§2.2) and align with an **`assoc`-over-`append`** lemma (§4.6, L5).
  **Dictionary laws:** `trans`, `total`. **No `antisym`.**

### 4.6 The supporting lemmas

Each is itself a small structural induction (Gap-B, comparison-free except where
noted); Foundation proves them alongside the laws:

- **L1 `allKeys`-preserved-by-insert** (law 1) — `allKeys p m → p key' → allKeys
  p (insert … key' … m)` for a bound `p` the inserted key satisfies. Induction
  on `m`, reflect the insert's stuck `leq` (Gap A).
- **L2 `allKeys ↔ allInList (toList)`** (laws 4, 5) — a tree's `allKeys p`
  transfers to `allInList p (toList m)`. Induction on `m`; uses an
  `allInList`-over-`append` split. Comparison-free (Gap-B only).
- **L3 `allKeys`-under-a-transitive-step** (law 1 overwrite) — move an `allKeys
  (≤a)` bound to `allKeys (≤b)` given `IsTrue (leq a b)`, mapping `trans` over
  the subtree. Comparison-free.
- **L4 `isSorted`-over-`++`** (law 4) — `isSorted xs → isSorted (Cons m ys) →
  allInList (keyLeq · m) xs → isSorted (list_append xs (Cons m ys))`. Induction
  on `xs`. Comparison-free (the `keyLeq` facts are supplied as `IsTrue`
  witnesses).
- **L5 `assoc`-over-`append`** (law 5) — relate `assoc key (append xs ys)` to
  `assoc key xs` / `assoc key ys` under the sortedness bounds. Induction on
  `xs`; reflect `assoc`'s stuck `leq` (Gap A).

## 5. Ω-discipline (the load-bearing guardrail)

- **`toList`-ordered is the `Ω` `isSorted` form — never permutation.** The
  permutation law (`toList` lists exactly the inserted entries once each) is
  **proof-relevant** (distinct interleavings are distinct derivations), so it
  **cannot** be `data Perm : Ω` directly
  ([[proof-relevant-inductive-cannot-be-declared-at-omega]]) — it needs
  `‖Perm_rel‖` / count-equality and inherits the C5 prover gap. It **stays
  deferred** (`52 §7c`, seed `tolist-permutation-law-deferred`). This doc proves
  only the naturally-`Ω` ordered form (law 4); **no `Perm` inductive, no `‖·‖`
  truncation** appears.
- **No stuck Boolean is reduced by fiat.** Every stuck `leq k k'` is discharged
  by a **transported order hypothesis** (§2.2) — a reflected equation or a
  dictionary law fed through `J` — never by asserting the result (the rejected
  K7-workaround anti-pattern). Law 4 clears Gap A precisely **because** it is
  comparison-free: its `leq` facts are `Ordered`'s stored witnesses, not stuck
  reductions.
- **All five goals live in `Ω`.** `Ordered`/`allKeys`/`isSorted`/`Equal`/`And`
  are `Ω`-valued (`52 §5.1`, `16 §1`); the `J` motives that transport them are
  `Ω`-valued, which the landed `infer_j` admits (its codomain sort is
  unconstrained — `../30-surface/34 §3.4`). No sort-polymorphic `subst` is
  needed; the `Ω` motive of `J` suffices (`53 §3`).

## 6. Buildability honesty + trust surface

These skeletons are **strategies verified against the two landed idioms** — the
Gap-B `dependent_match_nonnullary_acceptance.rs` shape and the Gap-A
`surface_transport_acceptance.rs` shape — **not** compiled proof terms. The
proof-engineering (exact motive lambdas, de Bruijn threading, lemma order) is
Foundation's, and the honest ceiling holds: **if a specific transport goal fails
to compute without a conversion change** (e.g. a motive that will not reduce, or
a stuck comparison no reflected equation fires) **that is a finding, re-deferred
to Steward** — never patched by an elaborator workaround that asserts the
result, and never postulated as `Axiom` (`52 §7d`, the guardrail that kept
Map-build correct).

**Zero `trusted_base()` delta.** Every proof reduces through the **existing**
`Term::J`/`Term::Cast` and `Term::Elim`; the helper defs (`Or`,
`boolDichotomy`, `assoc`, `allInList`, L1–L5) are ordinary `declare_def`/
`declare_inductive` admissions, kernel-rechecked. **Grep discipline for the
build:** no `crates/ken-kernel/` file is touched, no new `Decl`/`Term` variant,
no `declare_primitive`/`declare_postulate`, no `Axiom`. The true soundness net
is the kernel's whole-declaration `check` on each proof term (`check.rs`, the
per-`declare_def` check that recurses into every embedded `J`/`Elim`) — a
mis-built proof is **over-rejected** (fail-closed), never wrongly accepted.
Mirror the transport package's own `*_adds_zero_trusted_base_delta` set-equality
test on the map package.
