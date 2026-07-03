# Proof skeletons for the inductive `Map` laws (`map-verified-laws`)

> Status: **DRAFT v2 — Unit 1 landed (law 4 built); Unit 2 pending.** The
> proof-strategy elaboration for the inductive correctness laws that `52-map.md
> §7d` deferred. The capstone was **re-scoped into two landing units** along the
> elaborator fault-line (Steward `evt_40y1s0wpysa53`):
>
> - **Unit 1 — LANDED (real, kernel-rechecked proofs).** The **convoy idiom**
>   (Gap-B dependent induction, §2) + the **`allKeys`→`allInList` bridge** (L2,
>   §2.2) + the two **non-inductive** laws (`Ordered empty`, `lookup-empty`) +
>   **law 4** (`toList`-ordered) all build as **real, non-stubbed** proof terms
>   in `map.ken`. Law 4's conclusion `toListOrdered` (via the
>   `consSorted`/`isSortedAppend` append lemmas) is **no longer pending** — the
>   enabling kernel conversion fix **landed** as `obs-eq-termination`
>   (`9cf468a`,
>   PR #262 — the `(Eq,Eq)` `conv_struct` congruence arm plus a
>   congruence-first/lazy-δ fast path, §7). No `trusted_base` change was needed
>   in Unit 1's proof terms.
> - **Unit 2 — not yet buildable.** Laws **1/2/3/5**'s **Gap-A nested-`J`
>   transport composition**. "Wall 1" is a **real** `infer_j` nested-motive
>   scoping bug (Architect `evt_3vd9w6c779sm6`); its lane (a combinator
>   restructuring that dodges nested-`J`, vs. a fresh Language elaborator fix)
>   is pending. This doc gives laws 1/2/3/5's **Gap-B convoy skeleton only** and
>   marks their transport composition **Unit-2-pending** — it does **not**
>   specify the nested-`J` recipe (it does not build; §3).
>
> Adds **nothing** to `trusted_base()`: every proof reduces through the existing
> `Term::J`/`Term::Cast` (transport) and `Term::Elim` (dependent match) — no
> `declare_primitive`, no `declare_postulate`, **no `Axiom`**. A law (or lemma)
> that turns out not to be honestly buildable **re-defers to Steward** (§7),
> never postulated. Foundation fills these skeletons in as real proof terms in
> `packages/collections/map.ken`.

## 1. What this module is

`52-map.md §5` proves the `Map` invariant and operation-correctness laws as
**real proof terms**. Two of the seven — `Ordered empty` and `lookup k empty =
None` — are **non-inductive** (Branch A) and ship in `map.ken` already
(`orderedEmpty = tt`, `lookupEmptyIsNone = tt`). The remaining **five inductive
laws** (Branch B) each **induct over the non-nullary `Tree` carrier** via the
**convoy idiom** (§2); four of them additionally must **align a stuck `leq k
k'`** by transport (§3). This doc elaborates the proof strategy so a build model
does not reinvent it — and, per the two-unit re-scope, states precisely which
parts build today and which are deferred.

| # | Law (`52` ref) | Statement | Gap | Unit / status |
|---|---|---|---|---|
| 1 | preservation (`§5.1`) | `Ordered m → Ordered (insert leq key val m)` | A + B | Unit 2 (transport-blocked) |
| 2 | found-after-insert (`§5.2`) | `lookup leq key (insert leq key val m) = Some val` | A + B | Unit 2 (transport-blocked) |
| 3 | locality (`§5.2`) | `distinct key key' → lookup leq key' (insert leq key val m) = lookup leq key' m` | A + B | Unit 2 (transport-blocked) |
| 4 | `toList`-ordered (`§5.3`) | `Ordered m → isSorted keyLeq (toList m)` — **comparison-free** | **B only** | Unit 1 — **LANDED** (`toListOrdered` built, kernel-rechecked) |
| 5 | agreement (`§5.3`) | `lookup leq key m = assoc leq key (toList m)` | A + B | Unit 2 (transport-blocked) |

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

## 2. The convoy idiom — induction over `Tree` (Gap-B)

### 2.1 The idiom, stated

A proof by induction on `m : Tree k v` whose per-arm obligation depends on a
**scrutinee-dependent hypothesis** `h : P m` (typically `Ordered … m`) is
written by the **convoy idiom**. The naive form — leave `h` a *free parameter*
of the `match` and hope the elaborator narrows its type per-arm — does **not**
build (that was the reported "wall"; the fix is a proof idiom, not an elaborator
change — Architect probeC/probeD, `evt_3f1nzav3z4ab3`). Instead:

1. **Generalize the hypothesis into the motive.** The `view`'s result type is
   the **dependent function** `P m -> G m` — move `h` to the *right* of the
   colon so it becomes part of the goal the match's motive abstracts over the
   scrutinee:

   ```
   view f (k v : Type) (…dictionary…) (m : Tree k v) : P m -> G m =
     match m {
       Leaf             => \h. <base : G Leaf> ;
       Node l k2 v2 r   => \h. <step : G (Node l k2 v2 r)>
     }
   ```

2. **Match a bare variable of a flat, non-indexed family.** `match m` where `m`
   is the `view`'s own parameter is exactly the Gap-B gate's shape (`elab.rs`
   `dependent_eligible`, **`:535-553`** — scrutinee is a `Term::Var(_)`, family
   has `ind.indices.is_empty()`; `Tree`/`List` qualify).

3. **Each arm is a checked-mode λ binding the hypothesis at its per-arm narrowed
   type.** Because the arm body is checked against the narrowed return type
   `P (Node l k2 v2 r) -> G (Node l k2 v2 r)`, the leading `\h.` binds
   `h : P (Node l k2 v2 r)` — every occurrence of `m` in `P` **and** `G` is
   replaced by the reconstructed `Node l k2 v2 r`. This per-arm narrowing is the
   whole point of moving `h` into the motive; it is the narrowing the naive
   free-parameter form fails to obtain. (Landed shape:
   `dependent_match_nonnullary_acceptance.rs` AC1 — `view tailGoal (xs) :
   allTrue xs -> Prop = match xs { Nil => \h. … ; Cons b bs => \h. … }`.)

4. **The IH is a self-recursive call — not a kernel IH-slot.** The kernel's
   `method_type` (`ken_kernel::inductive`, `recursive_args`) requires one IH-`Π`
   per recursive field, and the elaborator emits them — but they are **dead,
   surface-unreferenceable binders** (`dependent-match-nonnullary.md`, "IHs are
   DEAD binders"; confirmed empirically, not merely read off the doc). The
   actual induction hypothesis is obtained the **same way `toList`/`insert`
   already recurse**: an **ordinary self-recursive `view` call on each recursive
   field** — `f … l (project h to P l)` yielding `G l`, `f … r (project h to P
   r)` yielding `G r` — whose **result is** the IH, consumed directly by name.
   The self-recursive descent is SCT-checked (`sct_check`, structural descent on
   the subtrees; the motive does not change across the recursion).

The base case (`Leaf`) is where the goal bottoms out with **no** induction — its
witness is `tt`/`Refl` (§2.3).

> **Correction folded here (grounded against the landed idiom).** Earlier drafts
> of this doc described the `Node` IH as two synthesized binders `ih_l`/`ih_r`
> bound *before* the arm's own `\h.`. That is **not** how Ken's dependent match
> works: those kernel IH-slots are dead, and the IH comes from a self-recursive
> `view` call as above (step 4). The [[buildability-ruling-must-ground-every-
> axis]] rule — reconstruct each mechanism from the landed code, never a prior
> artifact — applies to this doc's own prior version.

### 2.2 Worked example — the `allKeys → allInList` bridge (L2)

This is the convoy idiom carried end-to-end on the **real held-branch helpers**;
it **builds and kernel-checks clean on `4d4aaad`** in convoy form (Foundation's
real-`Tree` confirmation — Steward `evt_jb3nbm008xq0`). It is
**comparison-free** (no `leq`, no `J`, no `boolDichotomy`): pure Gap-B, the
cleanest instance of the idiom.

**Goal (L2).** A tree's `allKeys p` transfers to `allInList p` of its
in-order flattening: `allKeys p m -> allInList p (toList m)`. The landed defs
(`map.ken`, `collections.ken`):

```
allKeys p Leaf              ⇝ Equal Bool True True                 -- Top (K7)
allKeys p (Node l key val r) ⇝ And (p key) (And (allKeys p l) (allKeys p r))
toList Leaf                 ⇝ Nil
toList (Node l key val r)   ⇝ list_append (toList l) (Cons (mkPair key val) (toList r))
allInList p Nil             ⇝ Equal Bool True True                 -- Top (K7)
allInList p (Cons e xs)     ⇝ And (p (pairFst e)) (allInList p xs)
```

**Proof term (convoy).** Generalize `allKeys p m` into the motive; match `m`;
per-arm checked-mode `\h.`; self-recurse on `l` and `r` for the IH; assemble
with the `And`-projections (`andFst`/`andSnd`), `andIntro`, and the append lemma
`allInListAppendIntro` (§4, a Gap-B-only structural induction on the left list):

```
view allKeysToAllInList (k v : Type) (p : k -> Prop) (m : Tree k v)
  : allKeys k v p m -> allInList k v p (toList k v m) =
  match m {
    Leaf => \h. tt ;                                   -- goal ⇝ allInList p Nil ⇝ Top
    Node l key val r => \h.
      allInListAppendIntro k v p
        (toList k v l)
        (Cons (Pair k v) (mkPair k v key val) (toList k v r))
        (allKeysToAllInList k v p l (andFst _ _ (andSnd _ _ h)))     -- IH on l : allInList p (toList l)
        (andIntro (p key) (allInList k v p (toList k v r))
          (andFst _ _ h)                                             -- p key
          (allKeysToAllInList k v p r (andSnd _ _ (andSnd _ _ h))))  -- IH on r : allInList p (toList r)
  }
```

The hypothesis threads structurally: `h : And (p key) (And (allKeys p l)
(allKeys p r))`, so `andFst h : p key`, `andFst (andSnd h) : allKeys p l`, and
`andSnd (andSnd h) : allKeys p r` feed the two self-recursive calls. The second
`allInListAppendIntro` argument's type `allInList p (Cons (mkPair key val)
(toList r))` reduces to `And (p key) (allInList p (toList r))` (since
`pairFst (mkPair key val) ⇝ key`), built by `andIntro`. (`And`'s two `Prop`
arguments are elided as `_ _` for readability; spell them explicitly per the
landed convention.) **No dictionary laws, no transport** — the load-bearing
convoy shape (two recursive fields, nested-`And` destructuring, cross-function
composition) that Foundation confirmed builds.

### 2.3 Base-witness discipline — `tt` vs `Refl` (K7)

Every terminal equality obligation is discharged by one of two witnesses,
following the landed evidence (`lookupEmptyIsNone = tt`, `orderedEmpty = tt`;
the
`surface-transport` build retro):

- **`tt`** when the goal is an `IsTrue`/`Equal`-shaped proposition wrapping an
  **operation** that has now reduced, so the goal **observationally collapses to
  `Top`** (K7): `Equal Bool (op …) v` with `op … ⇝ v`, e.g. `allInList p Nil ⇝
  Equal Bool True True ⇝ Top`, or `Ordered … Leaf ⇝ Equal Bool True True ⇝ Top`.
  This is the `Refl`/`tt`/`absurd` idiom `lawful_classes.ken` already documents.
- **`Refl a`** when the goal is `Equal A a a` with both sides the **same
  already-reduced** term and no operation to collapse (used inside `cong`/`sym`
  in `transport.ken`).

Choosing `Refl` where the goal K7-collapses gives a confusing `TypeMismatch`
(the goal is already `Top`, not `Eq`) — prefer `tt` whenever an operation
reduced into the equality; reach for `Refl` only for a bare reflexive `Eq`.

## 3. The Gap-A transport primitive — and why laws 1/2/3/5 are Unit-2-blocked

Laws 1/2/3/5 must additionally **align a stuck `leq k k'`** (both keys are
variables, so `insert`/`lookup`'s inner `match leq k k'` is irreducibly stuck).
The **single-comparison** transport primitive for this is **landed**
(`surface-transport`, `19955d8`; `53-transport.md §3`), and this doc records it
so the Unit-2 boundary is precise:

**Reflect the stuck Boolean, then transport once.** The Gap-B gate needs a
*variable* scrutinee, so `match (leq k k') { … }` does not narrow; reflect
through `boolDichotomy` (§4) — a dependent match on a Bool **variable** — to get
`q : Equal Bool (leq k k') True` (resp. `False`) in each arm of a
**non-dependent**
`Or`-match, then fire one `J`:

```
J (\x _. G[x]) (baseTrue : G[True]) (sym Bool (leq k k') True q) : G[leq k k']
```

where `G[x]` is the goal with the stuck `leq k k'` abstracted to `x`, and
`G[True]` is the **reduced** branch goal. Because the motive is user-written it
abstracts exactly the intended occurrences (`53 §2`, the Agda-`subst` posture).
A **single** such transport builds today.

**Why laws 1/2/3/5 are blocked (Unit 2).** At an interior `Node`, `insert`
branches on **two sequential** comparisons (`leq key k2`, then — in its `True`
branch — `leq k2 key`), so discharging the goal requires **composing two**
transports: the `base` of the outer `J` is itself an inner `J`-application. This
**nested-`J`** shape hits the **Wall-1 `infer_j` nested-motive scoping bug**
(Architect `evt_3vd9w6c779sm6`): `infer_j` checks the inner `J` against an
**unreduced** `base_expected_ty` whose motive closes over the
outer context variable, producing a `TypeMismatch` (de Bruijn `@5`/`@7`).
Ascription does **not** dodge it — `RExpr::RAsc` is infer-only and never reaches
`infer_j`'s internal `base_expected_ty` recomputation (Foundation ground-truth
`evt_2srmersyfaeer`). Whether a `trans`/`cong` combinator restructuring can
avoid the nesting entirely, or a fresh Language elaborator fix is required, is
the pending Unit-2 lane question.

**The nested-`J` transport recipe is therefore *deliberately not specified
here*** — it does not build, and a `54` that documented it would be the
stale-frame hazard this WP forbids (per the Unit-1 scope: laws 1/2/3/5 get their
Gap-B convoy skeleton in §5.2, and their Gap-A transport composition is tracked
as Unit 2).

## 4. Reusable helpers to define (Foundation)

Already **landed** on the held base: `list_append`, `isSorted` (`prelude.rs` —
`isSorted a leq (Cons x (Cons y r)) = And (Equal Bool (leq x y) True) (isSorted
a leq (Cons y r))`), `Pair`/`mkPair`/`pairFst`/`pairSnd`, `And`/`andIntro`/
`andFst`/`andSnd` (`prelude.rs`, `And A B := Sigma(_:A).B`), `absurd`/`Bottom`,
`Equal`/`Refl`/`tt`, `allKeys`, `allInList`, `toList`, `pairLeq`, and the two
reflect helpers below.

- **`Or` / `Inl` / `Inr`** — the two-constructor sum. Registered directly as a
  kernel `declare_inductive` in `prelude.rs` (the surface `data` sugar hardcodes
  every parameter to `Type 0`, but `Or`'s two parameters are `Ω`-sorted
  propositions — the same "kernel API one level below the surface wrapper"
  technique `Pair`/`And` already use). **It must be `Type`-valued (proof-
  relevant), not `Ω`** — the whole point is to *case-split on which disjunct
  holds*, so `Inl`/`Inr` must be distinguishable; an `Ω`-valued `Or` would make
  them proof-irrelevantly equal and the split would carry no information
  ([[proof-relevant-inductive-cannot-be-declared-at-omega]]). Eliminating this
  `Type`-valued `Or` (whose payloads are `Ω`-equalities) into the `Ω`-valued map
  goal is an ordinary case analysis — the permitted `Type → Ω` motive direction,
  not the restricted large elimination.
- **`boolDichotomy`** — the reflect combinator (§3), a one-line Gap-B dependent
  match on a Bool **variable** (landed spelling — each arm's `b` is narrowed, so
  the payload types are `Equal Bool True True` etc., closed by `tt`):

  ```
  view boolDichotomy (b : Bool) : Or (Equal Bool b True) (Equal Bool b False) =
    match b {
      True  => Inl (Equal Bool True True) (Equal Bool True False) tt ;
      False => Inr (Equal Bool False True) (Equal Bool False False) tt
    }
  ```

- **`allInListAppendIntro`** (Unit 1, used by L2/§2.2 and law 4) — the
  `allInList`-over-`append` intro: `allInList p xs -> allInList p ys ->
  allInList p (list_append xs ys)`. A plain structural induction on `xs`
  (Gap-B-free — no dependent motive; `list_append`'s own recursion drives it).
  Comparison-free.
- **`assoc`** (Unit 2, law 5) — the ordered-list lookup: `assoc leq key : List
  (Pair k v) → Option v`, scanning by the same `leq key (pairFst e) / leq
  (pairFst e) key` coincidence test `lookup` uses. A plain structural `List`
  recursion (Gap-B-free).

## 5. Per-law skeletons

Notation: `G` is the current goal; `G[x]` is `G` with the named stuck `leq …`
abstracted to `x`; `keyLeq := pairLeq leq` is law 4/5's element comparator over
`Pair k v` (compares first components).

### 5.1 Law 4 — `toList`-ordered (`Ordered m → isSorted keyLeq (toList m)`)

**Unit 1 — LANDED (Gap-B, comparison-free): law 4 builds end-to-end** as a
real, non-stubbed, kernel-rechecked `toListOrdered` (`map.ken`, §7). `toList`
never calls `leq`; every `leq` fact is a stored
`IsTrue` witness threaded from `Ordered`'s `allKeys` conjuncts (themselves
`Equal Bool (leq …) True`, the exact shape `isSorted`'s conjuncts want — they
thread **directly**, no transport). So this law clears Gap A and needs **only**
the convoy induction (§2). Induct on `m`; motive `\m'. (Ordered … m' -> isSorted
keyLeq (toList m'))`.

- **`Leaf`** — `toList Leaf = Nil`; `isSorted … Nil ⇝ Top`. Base `\h. tt`.
- **`Node l k2 v2 r`** — `toList (Node …) = list_append (toList l) (Cons (k2,v2)
  (toList r))`. Self-recurse (§2.1 step 4) for the IH on each subtree:
  `toListOrdered … l (Ordered l from h)` : `isSorted (toList l)`, likewise for
  `r`. From `h`'s `allKeys` conjuncts, via the bridge lemma **L2** (§2.2,
  `allKeysToAllInList`): every key in `toList l` is `≤ k2`, and `k2 ≤` every key
  in `toList r`. Assemble with the two append lemmas:

  ```
  isSorted (toList l) ->
  isSorted (Cons (k2,v2) (toList r)) ->   -- consSorted: from isSorted (toList r) + k2 ≤ head r
  allInList (≤k2) (toList l) ->           -- bridge L2 from allKeys
  isSorted (list_append (toList l) (Cons (k2,v2) (toList r)))   -- isSortedAppend
  ```

  **No dictionary laws** beyond the stored witnesses; **no transport.** This is
  the load-bearing ordered-iteration law — proved as the naturally-`Ω`
  `isSorted` form, **never** permutation (§6).

  > **Conclusion `toListOrdered` is LANDED (§7).** The bridge **L2** and the
  > `consSorted`/`isSortedAppend` append lemmas assemble the `Node` step into a
  > real, kernel-rechecked `toListOrdered`. The residual `TypeMismatch` that
  > earlier blocked it — `isSorted`'s pair-indexed comparator vs. the bound
  > chain's key-indexed predicate not converging by a delta/iota step **inside
  > an `Eq` argument** — was a kernel conversion shortfall, now **fixed** by the
  > `(Eq, Eq)` `conv_struct` congruence arm shipped with a
  > congruence-first/lazy-δ fast path (`obs-eq-termination`, `9cf468a`, PR
  > #262). (The arm's first, naive form triggered an eager-δ non-termination on
  > the *assembled* proof — the deeper root — which the fast-path re-land
  > resolved; the completeness arm alone was not enough.) The landed proof
  > factors `isSorted`'s two-element lookahead into single-match helper `view`s
  > (the **nested-match-avoidance idiom**), to be canonicalized with the full
  > idiom set in a follow-on pass.

### 5.2 Laws 1/2/3/5 — Gap-B convoy skeleton (Gap-A transport → Unit 2)

Each of laws 1/2/3/5 is proved by the **same convoy induction** (§2) over `m`,
generalizing its hypothesis into the motive and self-recursing for the IH. Their
`Leaf` bases and subtree-descent structure are Gap-B and follow §2 directly.
Their **interior-`Node` steps additionally require Gap-A transport** to align
the stuck `leq key k2` / `leq k2 key` comparisons — the **nested-`J`
composition** that is **Unit-2-pending and not specified here** (§3). Below is
each law's Gap-B skeleton and the dictionary laws its Unit-2 step will need.

- **Law 1 — preservation** (`Ordered m → Ordered (insert key val m)`). Motive
  `\m'. (Ordered … m' -> Ordered … (insert … m'))`. `Leaf`: `insert Leaf = Node
  Leaf key val Leaf`; goal is the `And`-tree of `allKeys … Leaf ⇝ Top` /
  `Ordered Leaf ⇝ Top`, base `\h.` the `andIntro` tree of `tt`s (comparison-
  free). `Node`: subtrees by the IH (self-recursive calls); rebuilding the two
  `allKeys` bounds against the new label needs **L1** + **L3** and the reflected
  `leq` equations — **Gap-A transport, Unit 2.** Dictionary laws: `trans`,
  `total`; **no `antisym`.**
- **Law 2 — found-after-insert** (`lookup key (insert key val m) = Some val`).
  No `Ordered` hypothesis. Motive `\m'. Equal (Option v) (lookup … key (insert …
  key val m')) (Some val)`. `Leaf`: `insert` places `Node Leaf key val Leaf`;
  `lookup key` there compares `leq key key` twice — the equations are **free**
  (`refl key`), but reducing both stuck comparisons is still the two-transport
  composition (`insert`/`lookup` each branch on the pair `leq key k2`, `leq k2
  key`), which is the **Gap-A transport step — Unit 2** (whether a same-term
  single-`J` form dodges the nesting for the free-equation sub-cases is part of
  the pending combinator probe, §3). `Node`: `insert`/`lookup` branch on the
  identical scrutinee; subtrees by the IH. Dictionary laws: `refl` only.
- **Law 3 — locality** (`distinct key key' → lookup key' (insert key val m) =
  lookup key' m`). `distinct key key' := (And (IsTrue (leq key key')) (IsTrue
  (leq key' key))) -> Bottom`; `absurd` eliminates the `Bottom`. Motive folds in
  both hypotheses. `Leaf`/`Node`: reflect the relevant comparisons; a spurious
  `True`/`True` contradicts `distinct` (discharged by `absurd`); the aligning
  transport is the **nested-`J` composition — Unit 2.** Dictionary laws:
  `trans`, `total`; **no `antisym`.**
- **Law 5 — agreement** (`lookup key m = assoc key (toList m)`). Needs `Ordered
  m`. Motive `\m'. (Ordered … m' -> Equal (Option v) (lookup … key m')
  (assoc leq
  key (toList m')))`. `Leaf`: both `None`, base `\h. tt`. `Node`: `lookup`'s
  tree descent must be aligned with `assoc`'s scan of `list_append (toList l)
  (Cons (k2,v2) (toList r))` via `Ordered`'s bounds + `trans` and an
  **`assoc`-over-`append`** lemma (**L5**) — reflecting/transporting `lookup`'s
  stuck match is the **nested-`J` composition — Unit 2.** Dictionary laws:
  `trans`, `total`; **no `antisym`.**

### 5.3 The supporting lemmas

Each is a small structural induction; unit tags follow the same Gap-A/Gap-B
split. Foundation proves them alongside the laws.

- **L1 `allKeys`-preserved-by-insert** (law 1) — `allKeys p m → p key' → allKeys
  p (insert … key' … m)`. Induction on `m`; reflects `insert`'s stuck `leq`
  (Gap A) — **Unit 2.**
- **L2 `allKeys → allInList (toList)`** (laws 4, 5) — worked in §2.2
  (`allKeysToAllInList`). Comparison-free (Gap-B only). **Unit 1 — builds
  today.**
- **L3 `allKeys`-under-a-transitive-step** (law 1 overwrite) — move an `allKeys
  (≤a)` bound to `allKeys (≤b)` given `IsTrue (leq a b)`, mapping `trans` over
  the subtree. Comparison-free (Gap-B); buildable in isolation, but only
  *consumed* by law 1 (Unit 2).
- **L4 `isSorted`-over-`++`** (law 4) — the `isSortedAppend`/`consSorted` pair
  assembling law 4's `Node` step: `isSorted xs → isSorted (Cons m ys) →
  allInList (keyLeq · m) xs → isSorted (list_append xs (Cons m ys))`. Induction
  on `xs`, comparison-free (the `keyLeq` facts are supplied as `IsTrue`
  witnesses). **Landed** (the earlier residual `TypeMismatch` here was fixed by
  `obs-eq-termination` `9cf468a`; §5.1, §7); the landed proof routes the
  two-element lookahead through single-match helper `view`s (nested-match
  avoidance).
- **L5 `assoc`-over-`append`** (law 5) — relate `assoc key (append xs ys)` to
  `assoc key xs` / `assoc key ys` under the sortedness bounds. Induction on
  `xs`;
  reflects `assoc`'s stuck `leq` (Gap A) — **Unit 2.**

## 6. Ω-discipline (the load-bearing guardrail)

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
  by a **transported order hypothesis** (§3) — a reflected equation or a
  dictionary law fed through `J` — never by asserting the result (the rejected
  K7-workaround anti-pattern). Law 4 clears Gap A precisely **because** it is
  comparison-free: its `leq` facts are `Ordered`'s stored witnesses, not stuck
  reductions.
- **All goals live in `Ω`.** `Ordered`/`allKeys`/`allInList`/`isSorted`/`Equal`/
  `And` are `Ω`-valued (`52 §5.1`, `16 §1`); the `J` motives that transport them
  (Unit 2) are `Ω`-valued, which the landed `infer_j` admits (its codomain sort
  is unconstrained — `../30-surface/34 §3.4`). No sort-polymorphic `subst` is
  needed; the `Ω` motive of `J` suffices (`53 §3`).

## 7. Buildability honesty + trust surface

**Unit 1 is landed** as real, kernel-rechecked proofs in `map.ken` — the convoy
idiom (§2), the L2 bridge (§2.2), the non-inductive laws
(`orderedEmpty`/`lookupEmptyIsNone`), and law 4's `toListOrdered`. Beyond that,
the honest ceiling holds:

- **Law 4's conclusion `toListOrdered` (via `consSorted`/`isSortedAppend`, L4)
  is LANDED** — real, non-stubbed, kernel-rechecked. The kernel conversion
  shortfall that earlier blocked it (an `Eq` type whose argument needed a
  delta/iota step — the pair-indexed comparator vs. the key-indexed bound
  predicate — was over-rejected, and the naive `(Eq, Eq)` congruence arm then
  triggered eager-δ non-termination on the *assembled* proof) is **fixed** by
  `obs-eq-termination` (`9cf468a`, PR #262 — the congruence arm plus a
  congruence-first/lazy-δ fast path). Zero `trusted_base` delta (kernel
  completeness + termination only).
- **Laws 1/2/3/5 are Unit-2, transport-blocked** — their Gap-A nested-`J`
  composition hits the real `infer_j` scoping bug (§3). Their Gap-B convoy
  skeleton (§5.2) is buildable structure; the transport step is not, and the
  nested-`J` recipe is deliberately **not** specified here.
- **A transport goal that will not compute without a conversion change** (a
  motive that will not reduce, a stuck comparison no reflected equation
  fires) is
  a **finding, re-deferred to Steward** (`52 §7d`) — never patched by an
  elaborator workaround that asserts the result, and never postulated as
  `Axiom`.

**Zero `trusted_base()` delta.** Every proof reduces through the **existing**
`Term::J`/`Term::Cast` and `Term::Elim`; the helper defs (`Or`, `boolDichotomy`,
`allInListAppendIntro`, `assoc`, L1–L5) are ordinary `declare_def`/
`declare_inductive` admissions, kernel-rechecked. **Grep discipline for the
build:** no `crates/ken-kernel/` file is touched, no new `Decl`/`Term` variant,
no `declare_primitive`/`declare_postulate`, no `Axiom`. The true soundness net
is the kernel's whole-declaration `check` on each proof term (`check.rs`, the
per-`declare_def` check that recurses into every embedded `J`/`Elim`) — a
mis-built proof is **over-rejected** (fail-closed), never wrongly accepted.
Mirror the transport package's own `*_adds_zero_trusted_base_delta` set-equality
test on the map package.
