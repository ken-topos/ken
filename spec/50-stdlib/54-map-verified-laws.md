# Proof skeletons for the inductive `Map` laws (`map-verified-laws`)

> Status: **DRAFT v3 — CAPSTONE COMPLETE: all five inductive laws landed.** The
> proof-strategy elaboration for the inductive correctness laws that `52-map.md
> §7d` deferred. All five now build as **real, non-stubbed, kernel-checked**
> `view` proofs in `map.ken` (alongside the two non-inductive laws, already
> shipped):
>
> - **The convoy idiom** (Gap-B dependent induction, §2) carries every law's
>   carrier induction. **Law 4** (`to_list`-ordered) + the `all_keys`→`all_in_list`
>   bridge (L2) landed first — via the `obs-eq-termination` conversion fix
>   (`9cf468a`, PR #262, §7). **Laws 1/2/3/5** (preservation,
>   found-after-insert, locality, agreement) then landed via the
>   **`trans`/`cong` route-around** for their stuck-comparison transport (§3).
> - **No elaborator or kernel change was needed for laws 1/2/3/5.** The
>   originally-feared nested-`J` `infer_j` wall ("Wall 1") and the
>   `eq_at_inductive` `Tree`-congruence obstruction were **dissolved by the
>   route-around, not fixed** — goal-generic `trans`/`cong` transport bridges
>   avoid nesting `J` entirely (§3). The one new prelude admission is `Not : Ω →
>   Ω := λA. A → Bottom` (§4, for `NoDup`'s per-entry negation) — an ordinary
>   `declare_def`.
> - **Permutation is the only remaining `Map` correctness law**, and it stays
>   **permanently out of scope** — proof-relevant, needing `‖Perm_rel‖`/
>   count-equality (the C5 prover gap; §6, `52 §7c`). "All five *inductive* laws
>   landed" is the exact claim — **not** "all Map laws done."
>
> Adds **nothing** to `trusted_base()`: every proof reduces through the existing
> `Term::J`/`Term::Cast` (transport) and `Term::Elim` (dependent match); the
> helper defs (`Or`/`bool_dichotomy`/`Not`/`assoc`/`all_in_list`/L1–L5) are
> ordinary `declare_def`/`declare_inductive` admissions — no
> `declare_primitive`, no `declare_postulate`, **no `Axiom`** anywhere
> (`catalog/packages/collections/map.ken`).

## 1. What this module is

`52-map.md §5` proves the `Map` invariant and operation-correctness laws as
**real proof terms**. Two of the seven — `Ordered empty` and `lookup k empty =
None` — are **non-inductive** (Branch A) and ship in `map.ken` already
(`ordered_empty = tt`, `lookup_empty_is_none = tt`). The remaining **five inductive
laws** (Branch B) each **induct over the non-nullary `Tree` carrier** via the
**convoy idiom** (§2); four of them additionally **align a stuck `leq k k'`** by
transport (§3). This doc elaborates the proof strategy behind those landed
proofs — **all five inductive laws are now real terms** in `map.ken`;
permutation is the sole `Map` correctness law still deferred (§6).

| # | Law (`52` ref) | Statement | Gap | Unit / status |
|---|---|---|---|---|
| 1 | preservation (`§5.1`) | `Ordered m → Ordered (insert leq key val m)` | A + B | **LANDED** (`preserves_ordered`) |
| 2 | found-after-insert (`§5.2`) | `lookup leq key (insert leq key val m) = Some val` | A + B | **LANDED** (`lookup_found_after_insert`) |
| 3 | locality (`§5.2`) | `distinct key key' → lookup leq key' (insert leq key val m) = lookup leq key' m` | A + B | **LANDED** (`lookup_locality`) |
| 4 | `to_list`-ordered (`§5.3`) | `Ordered m → isSorted keyLeq (to_list m)` — **comparison-free** | **B only** | Unit 1 — **LANDED** (`to_list_ordered` built, kernel-rechecked) |
| 5 | agreement (`§5.3`) | `Ordered m → Distinct leq m → lookup leq key m = assoc leq key (to_list m)` | A + B | **LANDED** (`lookup_assoc_agree`) |

**Every law's own proof uses `refl`/`trans`/`total` only** — no `antisym →
Equal` in any of the five statements (`52 §5.2`/`§2.1`). Law 5 (agreement) takes
the extra hypothesis `Distinct leq m`, but its *proof* is **antisym-free and
carrier-general**: given `Distinct`, the matched-node value agreement reduces to
`refl` (both traversals return the value at the unique order-equivalent node —
no `Equal key k2` step). `antisym` is load-bearing **only** in the separate
**`Distinct`-discharge lemma** (`insert`/`from_list`-reachable ⟹ `Distinct`,
Foundation's, *not* part of law 5's statement); *that* lemma, alongside the
overwrite/uniqueness identity law, is what **inherits ADR 0010's
canonical-carrier obligation** (`52 §2.1`/`§5.3`). Law 5 itself does not — it
holds given `Distinct` even where `antisym` is false. (Architect's analysis
`evt_9q7hkxnrt3fm`, **confirmed at proof time**: the built `lookup_assoc_agree`
takes only `transLeq`, antisym-free.) The overwrite/uniqueness identity law
stays the separate, canonicity-gated law under ADR 0010 (`52 §5.3`), out of
this WP.

All spellings below are the **landed** `map.ken` idiom (`52 §2`, the
C5-verified-sort unbundled encoding): every op/law takes `leq` and its laws as
**separate bare parameters**, and `IsTrue b := Equal Bool b True` (the landed
`catalog/packages/lawful-classes` bridge). The order-law parameters, verbatim from the
`Ord` class (`catalog/packages/lawful-classes/lawful_classes.ken`):

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
   actual induction hypothesis is obtained the **same way `to_list`/`insert`
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

### 2.2 Worked example — the `all_keys → all_in_list` bridge (L2)

This is the convoy idiom carried end-to-end on the **real held-branch helpers**;
it **builds and kernel-checks clean on `4d4aaad`** in convoy form (Foundation's
real-`Tree` confirmation — Steward `evt_jb3nbm008xq0`). It is
**comparison-free** (no `leq`, no `J`, no `bool_dichotomy`): pure Gap-B, the
cleanest instance of the idiom.

**Goal (L2).** A tree's `all_keys p` transfers to `all_in_list p` of its
in-order flattening: `all_keys p m -> all_in_list p (to_list m)`. The landed defs
(`map.ken`, `collections.ken`):

```
all_keys p Leaf              ⇝ Equal Bool True True                 -- Top (K7)
all_keys p (Node l key val r) ⇝ And (p key) (And (all_keys p l) (all_keys p r))
to_list Leaf                 ⇝ Nil
to_list (Node l key val r)   ⇝ list_append (to_list l) (Cons (mkPair key val) (to_list r))
all_in_list p Nil             ⇝ Equal Bool True True                 -- Top (K7)
all_in_list p (Cons e xs)     ⇝ And (p (pairFst e)) (all_in_list p xs)
```

**Proof term (convoy).** Generalize `all_keys p m` into the motive; match `m`;
per-arm checked-mode `\h.`; self-recurse on `l` and `r` for the IH; assemble
with the `And`-projections (`andFst`/`andSnd`), `andIntro`, and the append lemma
`all_in_list_append_intro` (§4, a Gap-B-only structural induction on the left list):

```
view all_keys_to_all_in_list (k v : Type) (p : k -> Prop) (m : Tree k v)
  : all_keys k v p m -> all_in_list k v p (to_list k v m) =
  match m {
    Leaf => \h. tt ;                                   -- goal ⇝ all_in_list p Nil ⇝ Top
    Node l key val r => \h.
      all_in_list_append_intro k v p
        (to_list k v l)
        (Cons (Pair k v) (mkPair k v key val) (to_list k v r))
        (all_keys_to_all_in_list k v p l (andFst _ _ (andSnd _ _ h)))     -- IH on l : all_in_list p (to_list l)
        (andIntro (p key) (all_in_list k v p (to_list k v r))
          (andFst _ _ h)                                             -- p key
          (all_keys_to_all_in_list k v p r (andSnd _ _ (andSnd _ _ h))))  -- IH on r : all_in_list p (to_list r)
  }
```

The hypothesis threads structurally: `h : And (p key) (And (all_keys p l)
(all_keys p r))`, so `andFst h : p key`, `andFst (andSnd h) : all_keys p l`, and
`andSnd (andSnd h) : all_keys p r` feed the two self-recursive calls. The second
`all_in_list_append_intro` argument's type `all_in_list p (Cons (mkPair key val)
(to_list r))` reduces to `And (p key) (all_in_list p (to_list r))` (since
`pairFst (mkPair key val) ⇝ key`), built by `andIntro`. (`And`'s two `Prop`
arguments are elided as `_ _` for readability; spell them explicitly per the
landed convention.) **No dictionary laws, no transport** — the load-bearing
convoy shape (two recursive fields, nested-`And` destructuring, cross-function
composition) that Foundation confirmed builds.

### 2.3 Base-witness discipline — `tt` vs `Refl` (K7)

Every terminal equality obligation is discharged by one of two witnesses,
following the landed evidence (`lookup_empty_is_none = tt`, `ordered_empty = tt`;
the
`surface-transport` build retro):

- **`tt`** when the goal is an `IsTrue`/`Equal`-shaped proposition wrapping an
  **operation** that has now reduced, so the goal **observationally collapses to
  `Top`** (K7): `Equal Bool (op …) v` with `op … ⇝ v`, e.g. `all_in_list p Nil ⇝
  Equal Bool True True ⇝ Top`, or `Ordered … Leaf ⇝ Equal Bool True True ⇝ Top`.
  This is the `Refl`/`tt`/`absurd` idiom `lawful_classes.ken` already documents.
- **`Refl a`** when the goal is `Equal A a a` with both sides the **same
  already-reduced** term and no operation to collapse (used inside `cong`/`sym`
  in `transport.ken`).

Choosing `Refl` where the goal K7-collapses gives a confusing `TypeMismatch`
(the goal is already `Top`, not `Eq`) — prefer `tt` whenever an operation
reduced into the equality; reach for `Refl` only for a bare reflexive `Eq`.

## 3. The Gap-A transport — the `trans`/`cong` route-around (laws 1/2/3/5)

Laws 1/2/3/5 must additionally **align a stuck `leq k k'`** (both keys are
variables, so `insert`/`lookup`'s inner `match leq k k'` is irreducibly stuck).
The **single-comparison** transport primitive is landed (`surface-transport`,
`19955d8`; `53-transport.md §3`):

**Reflect the stuck Boolean, then transport once.** The Gap-B gate needs a
*variable* scrutinee, so `match (leq k k') { … }` does not narrow; reflect
through `bool_dichotomy` (§4) — a dependent match on a Bool **variable** — to get
`q : Equal Bool (leq k k') True` (resp. `False`) in each arm of a
**non-dependent** `Or`-match, then fire one `J`:

```
J (\x _. G[x]) (baseTrue : G[True]) (sym Bool (leq k k') True q) : G[leq k k']
```

where `G[x]` is the goal with the stuck `leq k k'` abstracted to `x`, and
`G[True]` is the **reduced** branch goal. Because the motive is user-written it
abstracts exactly the intended occurrences (`53 §2`, the Agda-`subst` posture).

**The two-comparison composition — via `trans`/`cong`, not nested `J`.** At an
interior `Node`, `insert` branches on **two sequential** comparisons (`leq key
k2`, then — in its `True` branch — `leq k2 key`). The *naive* composition nests
one `J` inside another `J`'s `base`; that hit an `infer_j` nested-motive scoping
wall ("Wall 1") and did **not** build. **That wall was dissolved, not fixed**
(no
elaborator change) by the **`trans`/`cong` route-around** — the same "avoid
nested/multi-level constructs, route around via helper lemmas" discipline
`lawful_classes.ken` documents. The as-built mechanism:

- **Goal-generic transport bridges** — `insert_case_transport_overwrite` /
  `…IntoL` / `…IntoR` compose the two `Eq` witnesses into **one** combined
  equation (a "stop-one-step-short" `trans`/`cong`), then fire a **single** `J`
  — never a `J` inside a `J`'s `base`. They are **goal-generic** (quantified
  over
  the goal predicate), so laws 1/2/3 reuse them unchanged, each instantiating at
  its own goal (preservation's `Ordered`; found-after-insert's `Equal … (Some
  val)`; locality's `Equal … (lookup …)`).
- **Comparison-independent predicates need no Gap-A transport at all.** A
  predicate like `all_keys p` over an *abstract* `p` (and `Ordered`'s subtree
  conjuncts) is proven **generically** and applied by β — there is no stuck
  `leq`
  to transport. Only the per-node comparisons hit the bridges; the bulk of each
  proof is comparison-free convoy induction (§2).
- **`derive_from_false`** — totality-derived reflection: `total`'s `IsTrue
  (bool_or (leq a b) (leq b a))` feeds a **non-dependent `Or`-elimination** that
  supplies the branch equation without a stuck match.
- Law 3 additionally uses **`bool_value_eq_from_biimpl`** (a two-way `Bool`
  implication forces *value*-equality — `Bool` has exactly two constructors);
  law
  5 threads `NoDup`-over-`append` decompositions + `assoc`-side step mirrors
  (§4, §5.2).

Both the nested-`J` `infer_j` wall **and** the `eq_at_inductive`
`Tree`-constructor congruence obstruction were **dissolved by this
route-around**
— neither needed an elaborator or kernel fix. The **only** new prelude admission
the capstone required is `Not : Ω → Ω := λA. A → Bottom` (§4), for `NoDup`'s
per-entry negation — an ordinary `declare_def`, zero `trusted_base` delta.
(Naming the surface gap it works around: Ken's surface has **no
expression-position `->`** — only a `view`'s type-annotation position parses the
`Π`-sugar — so `NoDup`'s per-entry negation predicate cannot be spelled inline
and needs the `Not` combinator; a first-class expression-level `->` / `Not` is a
**named Language-lane follow-on**, not a blocker.)

## 4. Reusable helpers to define (Foundation)

Already **landed** on the held base: `list_append`, `isSorted` (`prelude.rs` —
`isSorted a leq (Cons x (Cons y r)) = And (Equal Bool (leq x y) True) (isSorted
a leq (Cons y r))`), `Pair`/`mkPair`/`pairFst`/`pairSnd`, `And`/`andIntro`/
`andFst`/`andSnd` (`prelude.rs`, `And A B := Sigma(_:A).B`), `absurd`/`Bottom`,
`Equal`/`Refl`/`tt`, `all_keys`, `all_in_list`, `to_list`, `pair_leq`, and — added by
the capstone build — **`Not : Ω → Ω := λA. A → Bottom`** (`prelude.rs`, an
ordinary `declare_def` over `Π` + the existing `Bottom`, built like `And`/`Or`;
needed because the surface has no expression-position `->`, §3). The helpers
below are the remaining landed defs.

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
- **`bool_dichotomy`** — the reflect combinator (§3), a one-line Gap-B dependent
  match on a Bool **variable** (landed spelling — each arm's `b` is narrowed, so
  the payload types are `Equal Bool True True` etc., closed by `tt`):

  ```
  view bool_dichotomy (b : Bool) : Or (Equal Bool b True) (Equal Bool b False) =
    match b {
      True  => Inl (Equal Bool True True) (Equal Bool True False) tt ;
      False => Inr (Equal Bool False True) (Equal Bool False False) tt
    }
  ```

- **`all_in_list_append_intro`** (Unit 1, used by L2/§2.2 and law 4) — the
  `all_in_list`-over-`append` intro: `all_in_list p xs -> all_in_list p ys ->
  all_in_list p (list_append xs ys)`. A plain structural induction on `xs`
  (Gap-B-free — no dependent motive; `list_append`'s own recursion drives it).
  Comparison-free.
- **`assoc`** (law 5, landed) — the ordered-list lookup: `assoc leq key : List
  (Pair k v) → Option v`, scanning by the same `leq key (pairFst e) / leq
  (pairFst e) key` coincidence test `lookup` uses. A plain structural `List`
  recursion (Gap-B-free).
- **`order_equiv` / `NoDup` / `Distinct`** (law 5's uniqueness precondition,
  landed) — all `Ω`-valued, all comparison-free structural recursions:
  - `order_equiv leq a b := And (IsTrue (leq a b)) (IsTrue (leq b a))` — two keys
    are order-equivalent (each `≤` the other).
  - `NoDup leq (xs : List (Pair k v))` — no two entries carry order-equivalent
    keys: `NoDup leq Nil = ⊤`; `NoDup leq (Cons e xs) = And (all_in_list (\k'.
    order_equiv leq k' (pairFst e) -> Bottom) xs) (NoDup leq xs)` (each head's
    key is order-distinct from every tail key).
  - `Distinct leq m := NoDup leq (to_list m)` — **law 5's key-uniqueness
    precondition**, encoded over `to_list m`'s keys so it aligns with `assoc`'s
    first-match scan (Architect's recommended encoding). It is the no-duplicate
    invariant `insert`/`from_list` maintain by construction; a real
    `from_list`/insert-built map **provably discharges** `Distinct` via `antisym`
    (order-equivalent ⟹ `Equal` ⟹ the overwrite branch never duplicates) — that
    discharge lemma is Foundation's, **not** part of law 5's statement.

## 5. Per-law skeletons

Notation: `G` is the current goal; `G[x]` is `G` with the named stuck `leq …`
abstracted to `x`; `keyLeq := pair_leq leq` is law 4/5's element comparator over
`Pair k v` (compares first components).

### 5.1 Law 4 — `to_list`-ordered (`Ordered m → isSorted keyLeq (to_list m)`)

**Unit 1 — LANDED (Gap-B, comparison-free): law 4 builds end-to-end** as a
real, non-stubbed, kernel-rechecked `to_list_ordered` (`map.ken`, §7). `to_list`
never calls `leq`; every `leq` fact is a stored
`IsTrue` witness threaded from `Ordered`'s `all_keys` conjuncts (themselves
`Equal Bool (leq …) True`, the exact shape `isSorted`'s conjuncts want — they
thread **directly**, no transport). So this law clears Gap A and needs **only**
the convoy induction (§2). Induct on `m`; motive `\m'. (Ordered … m' -> isSorted
keyLeq (to_list m'))`.

- **`Leaf`** — `to_list Leaf = Nil`; `isSorted … Nil ⇝ Top`. Base `\h. tt`.
- **`Node l k2 v2 r`** — `to_list (Node …) = list_append (to_list l) (Cons (k2,v2)
  (to_list r))`. Self-recurse (§2.1 step 4) for the IH on each subtree:
  `to_list_ordered … l (Ordered l from h)` : `isSorted (to_list l)`, likewise for
  `r`. From `h`'s `all_keys` conjuncts, via the bridge lemma **L2** (§2.2,
  `all_keys_to_all_in_list`): every key in `to_list l` is `≤ k2`, and `k2 ≤` every key
  in `to_list r`. Assemble with the two append lemmas:

  ```
  isSorted (to_list l) ->
  isSorted (Cons (k2,v2) (to_list r)) ->   -- consSorted: from isSorted (to_list r) + k2 ≤ head r
  all_in_list (≤k2) (to_list l) ->           -- bridge L2 from all_keys
  isSorted (list_append (to_list l) (Cons (k2,v2) (to_list r)))   -- is_sorted_append
  ```

  **No dictionary laws** beyond the stored witnesses; **no transport.** This is
  the load-bearing ordered-iteration law — proved as the naturally-`Ω`
  `isSorted` form, **never** permutation (§6).

  > **Conclusion `to_list_ordered` is LANDED (§7).** The bridge **L2** and the
  > `consSorted`/`is_sorted_append` append lemmas assemble the `Node` step into a
  > real, kernel-rechecked `to_list_ordered`. The residual `TypeMismatch` that
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

### 5.2 Laws 1/2/3/5 — landed (convoy + `trans`/`cong` route-around)

All four are **landed** as real `view` proofs in `map.ken`. Each is proved by
the **same convoy induction** (§2) over `m`, generalizing its hypothesis into
the motive and self-recursing for the IH; the `Leaf` bases and subtree descent
are comparison-free (Gap-B, §2). The interior-`Node` steps align the stuck `leq
key k2` / `leq k2 key` comparisons via the **`trans`/`cong` route-around** (§3)
— the goal-generic transport bridges, **not** a nested `J`. Below is each law's
structure and the dictionary laws it uses.

- **Law 1 — preservation** (`Ordered m → Ordered (insert key val m)`). Motive
  `\m'. (Ordered … m' -> Ordered … (insert … m'))`. `Leaf`: `insert Leaf = Node
  Leaf key val Leaf`; goal is the `And`-tree of `all_keys … Leaf ⇝ Top` /
  `Ordered Leaf ⇝ Top`, base `\h.` the `andIntro` tree of `tt`s (comparison-
  free). `Node`: subtrees by the IH (self-recursive calls); rebuilding the two
  `all_keys` bounds against the new label needs **L1** + **L3** and the reflected
  `leq` equations — aligned by the goal-generic transport bridges (§3).
  Dictionary laws: `trans`,
  `total`; **no `antisym`.**
- **Law 2 — found-after-insert** (`lookup key (insert key val m) = Some val`).
  No `Ordered` hypothesis. Motive `\m'. Equal (Option v) (lookup … key (insert …
  key val m')) (Some val)`. `Leaf`: `insert` places `Node Leaf key val Leaf`;
  `lookup key` there compares `leq key key` twice — the equations are **free**
  (`refl key`), but reducing both stuck comparisons is still the two-transport
  composition (`insert`/`lookup` each branch on the pair `leq key k2`, `leq k2
  key`), reduced by the route-around's combined-equation transport (§3).
  `Node`: `insert`/`lookup` branch on the
  identical scrutinee; subtrees by the IH. Dictionary laws: `refl` only.
- **Law 3 — locality** (`distinct key key' → lookup key' (insert key val m) =
  lookup key' m`). `distinct key key' := (And (IsTrue (leq key key')) (IsTrue
  (leq key' key))) -> Bottom`; `absurd` eliminates the `Bottom`. Motive folds in
  both hypotheses. `Leaf`/`Node`: reflect the relevant comparisons; a spurious
  `True`/`True` contradicts `distinct` (discharged by `absurd`); the aligning
  transport uses the route-around bridges + `bool_value_eq_from_biimpl` (a two-way
  `Bool` implication forces value-equality, §3). Dictionary laws: `trans`,
  `total`; **no `antisym`.**
- **Law 5 — agreement** (`Ordered m → Distinct leq m → lookup key m = assoc key
  (to_list m)`). **Requires the key-uniqueness precondition `Distinct leq m`**
  (§4): `lookup`'s BST descent and `assoc`'s in-order scan of `to_list` are two
  *different* traversal orders that agree **iff** keys are unique. Without
  `Distinct` the law is **false** — `Ordered`'s weak `≤`/`≥` bounds admit
  duplicates, and `Node (Node Leaf key v1 Leaf) key v2 Leaf` (a legitimate
  `Ordered` witness) has `lookup key = Some v2` (root, first BST match) but
  `assoc key (to_list) = Some v1` (list-first). This holds even at a **fully
  lawful `≤`** (e.g. `Int`: `0 ≤ 0` makes the tree `Ordered`) — which is exactly
  why adding `antisym` as a *hypothesis* cannot rescue it (the duplicate is the
  same key value, so `antisym` yields the vacuous `Equal key key`). The fix is a
  uniqueness *precondition*, not a stronger dictionary. `Ordered` is
  **unchanged** and `Distinct` is added, not folded into it.
  Motive `\m'. (Ordered … m' -> Distinct … m' -> Equal (Option v) (lookup … key
  m') (assoc leq key (to_list m')))`. `Leaf`: both `None`, base `tt`. `Node`:
  align `lookup`'s descent with `assoc`'s scan of `list_append (to_list l) (Cons
  (k2,v2) (to_list r))` via `Ordered`'s bounds + `trans` and an
  **`assoc`-over-`append`** lemma (**L5**); `Distinct` supplies the *unique*
  order-equivalent entry both traversals select, so the matched-node value
  agreement is **`refl`** — both return the value at that entry, no `Equal key
  k2` step needed. `lookup`'s stuck match is aligned by the `trans`/`cong`
  route-around bridges (§3) — plus `NoDup`-over-`append` decompositions and
  `assoc`-side step mirrors for the list scan. Dictionary laws: **`trans`,
  `total`** (antisym-free, matching laws 1–4). `antisym` enters only the
  separate
  **`Distinct`-discharge lemma** (`insert`/`from_list`-reachable ⟹ `Distinct`,
  Foundation's — not part of this statement), so **law 5's statement is
  carrier-general**: it holds given `Distinct` even where `antisym` is false
  (a non-canonical carrier — `Decimal`, many reps per value — has a false
  `antisym`, but `Distinct` forbids two order-equivalent entries, so lookup and
  assoc still agree). Only the discharge lemma inherits ADR 0010's
  canonical-carrier obligation (`52 §2.1`). (Architect's analysis
  `evt_9q7hkxnrt3fm`, **confirmed at proof time**: the built `lookup_assoc_agree`
  takes only `transLeq` — antisym-free, agreement is `refl` — Architect
  `evt_5rgg2g2wtg75b`, foundation-qa.)

### 5.3 The supporting lemmas

Each is a small structural induction; all landed alongside the laws.

- **L1 `all_keys`-preserved-by-insert** (law 1) — `all_keys p m → p key' → all_keys
  p (insert … key' … m)`. Induction on `m`; reflects `insert`'s stuck `leq`
  (Gap A, via the route-around bridges §3). **Landed.**
- **L2 `all_keys → all_in_list (to_list)`** (laws 4, 5) — worked in §2.2
  (`all_keys_to_all_in_list`). Comparison-free (Gap-B only). **Landed.**
- **L3 `all_keys`-under-a-transitive-step** (law 1 overwrite) — move an `all_keys
  (≤a)` bound to `all_keys (≤b)` given `IsTrue (leq a b)`, mapping `trans` over
  the subtree. Comparison-free (Gap-B), consumed by law 1. **Landed.**
- **L4 `isSorted`-over-`++`** (law 4) — the `is_sorted_append`/`consSorted` pair
  assembling law 4's `Node` step: `isSorted xs → isSorted (Cons m ys) →
  all_in_list (keyLeq · m) xs → isSorted (list_append xs (Cons m ys))`. Induction
  on `xs`, comparison-free (the `keyLeq` facts are supplied as `IsTrue`
  witnesses). **Landed** (the earlier residual `TypeMismatch` here was fixed by
  `obs-eq-termination` `9cf468a`; §5.1, §7); the landed proof routes the
  two-element lookahead through single-match helper `view`s (nested-match
  avoidance).
- **L5 `assoc`-over-`append`** (law 5) — relate `assoc key (append xs ys)` to
  `assoc key xs` / `assoc key ys` under the sortedness bounds, threading law 5's
  `Distinct`/`NoDup` so the first-match scan hits the *unique* order-equivalent
  entry. Induction on `xs`; reflects `assoc`'s stuck `leq` (Gap A, via the
  route-around §3). **Landed.**

## 6. Ω-discipline (the load-bearing guardrail)

- **`to_list`-ordered is the `Ω` `isSorted` form — never permutation.** The
  permutation law (`to_list` lists exactly the inserted entries once each) is
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
- **All goals live in `Ω`.** `Ordered`/`all_keys`/`all_in_list`/`isSorted`/`Equal`/
  `And` — and law 5's `order_equiv`/`NoDup`/`Distinct` (built from
  `IsTrue`/`And`/`all_in_list`/`->Bottom`) — are `Ω`-valued (`52 §5.1`, `16 §1`);
  the `J` motives that transport them (laws 1/2/3/5, §3) are `Ω`-valued, which
  the landed `infer_j` admits (its codomain sort
  is unconstrained — `../30-surface/34 §3.4`). No sort-polymorphic `subst` is
  needed; the `Ω` motive of `J` suffices (`53 §3`).

## 7. Buildability honesty + trust surface

**All five inductive laws are landed** as real, kernel-rechecked `view` proofs
in `map.ken`, alongside the non-inductive laws
(`ordered_empty`/`lookup_empty_is_none`) — via the convoy idiom (§2) + the
`trans`/`cong` route-around (§3), zero elaborator/kernel change beyond the `Not`
prelude def. The honest boundary:

- **Law 4's conclusion `to_list_ordered` (via `consSorted`/`is_sorted_append`, L4)
  is LANDED** — real, non-stubbed, kernel-rechecked. The kernel conversion
  shortfall that earlier blocked it (an `Eq` type whose argument needed a
  delta/iota step — the pair-indexed comparator vs. the key-indexed bound
  predicate — was over-rejected, and the naive `(Eq, Eq)` congruence arm then
  triggered eager-δ non-termination on the *assembled* proof) is **fixed** by
  `obs-eq-termination` (`9cf468a`, PR #262 — the congruence arm plus a
  congruence-first/lazy-δ fast path). Zero `trusted_base` delta (kernel
  completeness + termination only).
- **Laws 1/2/3/5 are LANDED** — their Gap-A stuck-comparison alignment uses the
  `trans`/`cong` route-around (§3, the goal-generic bridges), which
  **dissolved**
  the nested-`J` `infer_j` wall and the `eq_at_inductive` `Tree`-congruence
  obstruction **without** any elaborator or kernel change. Real `view` proofs in
  `map.ken`.
- **A transport goal that will not compute without a conversion change** (a
  motive that will not reduce, a stuck comparison no reflected equation
  fires) is
  a **finding, re-deferred to Steward** (`52 §7d`) — never patched by an
  elaborator workaround that asserts the result, and never postulated as
  `Axiom`.

**Zero `trusted_base()` delta.** Every proof reduces through the **existing**
`Term::J`/`Term::Cast` and `Term::Elim`; the helper defs (`Or`, `bool_dichotomy`,
`all_in_list_append_intro`, `assoc`, L1–L5) are ordinary `declare_def`/
`declare_inductive` admissions, kernel-rechecked. **Grep discipline for the
build:** no `crates/ken-kernel/` file is touched, no new `Decl`/`Term` variant,
no `declare_primitive`/`declare_postulate`, no `Axiom`. The true soundness net
is the kernel's whole-declaration `check` on each proof term (`check.rs`, the
per-`declare_def` check that recurses into every embedded `J`/`Elim`) — a
mis-built proof is **over-rejected** (fail-closed), never wrongly accepted.
Mirror the transport package's own `*_adds_zero_trusted_base_delta` set-equality
test on the map package.
