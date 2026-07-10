# A proved, pure `Map k v` over `Ord k`

> Status: **DRAFT v0 (VAL2 #8 / OQ-A).** A `catalog/packages/` catalog entry that
> closes `letter-frequency`'s gap — `Map` was a bare opaque type with **zero
> operations**. Operator-locked shape (OQ-A, 2026-07-03): a **proved, pure,
> `Ord k`-keyed** associative map, **shipped as ordinary package Ken — out of
> `trusted_base()`, not a kernel builtin**. It is the container companion to
> the lawful classes (`51`): its structure is an ordinary inductive and its
> correctness laws are **real kernel proofs, not postulates — no `Axiom`
> stubs** (`README §2`, `../20-verification/21 §3`), with the **five inductive**
> law proofs deferred to a **named** follow-on (`map-verified-laws`, §7d)
> pending two build-completeness capabilities (dependent-motive recovery +
> transport) the elaborator did not expose at Map-build time (§5) — a
> **buildability** boundary, not soundness. **Both have since landed** (Gap A
> `19955d8`, Gap B `282856c`); the five proofs' strategy is in
> `54-map-verified-laws.md`, and Foundation builds the terms in **two staged
> units** (the capstone was re-scoped along the elaborator fault-line): **Unit
> 1** — the convoy dependent-induction idiom + **law 4** (`to_list`-ordered) +
> the non-inductive laws — is **landed** as real proofs in `map.ken` (law 4's
> conclusion built once the enabling kernel conversion fix landed,
> `obs-eq-termination` `9cf468a`); **laws 1/2/3/5** then landed via the
> **`trans`/`cong` route-around** for their stuck-comparison transport (`54 §3`,
> §7). **No new
> kernel feature:** the
> carrier is a `data` type (`../30-surface/34 §1`), the operations are `view`
> defs, the laws are `Ω` propositions (`../10-kernel/16 §1`); if the build adds
> a kernel
> rule/former or a `declare_primitive`, it has over-built.

## 1. What this module is — and why it is *derived*, not a primitive

`Map k v` is a finite **associative map**: a partial function from keys `k` to
values `v`, keyed on a lawful **`Ord k`** (`51 §2.3`). It is an ordinary Ken
`data` structure whose operations are kernel-re-checked `view` defs — it adds
**nothing** to `trusted_base()`, and (see §1.1) it **removes** the placeholder
that was there.

`Set a` is **`Map a Unit`** (a map whose values carry no information); its
operations derive from the map's (§4.4). Everything below is stated for `Map`;
`Set` follows by `v := Unit`.

### 1.1 Supersession — the proved map replaces the opaque primitive

Before this module, `Map`/`Set` existed only as **opaque audited primitives**
(`declare_primitive OpaqueType`, `ken-elaborator/.../prelude.rs`; in
`trusted_base()`, item-2, like `String`/`Bytes` — `30 §6`) with **no operations
at all** — a bare `Map : Type → Type → Type` a program could *name* but never
*build* or *query* (the `letter-frequency` `KNOWN-GAP`). This module **retires**
that placeholder: the proved map **supersedes** it (OQ-A;
subsume-don't-proliferate, `PRINCIPLES.md`). There are not two `Map`s.

The retirement is **structurally forced by "proved", not a stylistic choice.**
"Proved, not tested" means the map's invariant and its operation-correctness
laws are carried as **real proof terms**. A proof of `lookup k (insert k v m) =
Some v` proceeds by **case-analysis on the map's constructors** and reduction of
the `lookup`/`insert` redexes (`Ω`-motive elimination, `../10-kernel/14 §3`,
K4). An **opaque** primitive has **no constructors and no eliminator** — nothing
to case-split, no redex to reduce — so its laws could only be **postulated**
(`Axiom`). This is exactly why the landed `Ord Int` laws are `Axiom` (a K1
primitive, no induction principle — `51 §6`) while `Ord Bool`, a real inductive,
carries **real** case-split proofs. A proved `Map` therefore **requires a
transparent inductive carrier** (§3); the opaque primitive is incompatible with
the WP's own premise.

**The trade OQ-A makes (honesty-about-the-boundary).** The retired primitive was
*genuinely runtime*: an O(1) content-addressed, insertion-order-independent
canonical form (`30 §6`, `41 §3a`). A program-level tree cannot match that — it
is O(log n) and not insertion-order canonical (§3, §5.4). OQ-A **chooses proved
+ pure + zero-TCB over the runtime-O(1) heap form**: the operator prefers a
small auditable trust root and real proofs to a constant-factor speed win over
an opaque primitive. The content-addressed heap map is not deleted from the
design space — it is **parked** as a possible later fast-map (the "HAMT-later"
analog, `Array`'s abstract persistent tree, §6), and would itself be proved if
it ever lands.

**TCB effect (net-negative, AC1).** Retiring the primitive **shrinks**
`trusted_base()` by exactly the `Map`/`Set` entries, with **zero additions** —
the direct payoff of "proved, not trusted". It is a net-negative trust delta,
not a growth; the zero-TCB acceptance is met in the strong direction (§9, AC1).

> **Whole-doc reconcile (this `/spec`).** Three sites describe `Map`/`Set` as
> `declare_primitive` audited types and are updated by this WP to point here:
> `../30-surface/37 §3`/`§3.3` (the collection table + the `DecEq`-keyed heap
> description), `README §3` (built-in audited runtime types), and
> `../30-surface/30 §6` (the `trusted_base()` item-2/item-3 taxonomy ruling).
> The coupled **build-lane** removal (`prelude.rs` primitive + the
> `es2_acceptance.rs` "Map/Set are primitives" assertion, which now **flips**)
> is Foundation's, named a **hard build AC** at merge_ready (§9, AC5).

## 2. The `Ord k` constraint — dictionary-passed, same-repo

Every operation that compares keys takes the key order as an **implicit lawful
dictionary**: the surface `where Ord k` desugars to an implicit instance
argument `{d : Ord k}` (`../30-surface/33 §5.4`), and inside the body the
comparator is the projection `d.leq : k → k → Bool` (`51 §4`). Instance
resolution is the landed canonical resolver — one canonical `Ord k` per key type
(`instance_search`, ADR 0008 coherence). No new mechanism.

This uses the **landed** same-repo dictionary idiom — the one `Ord Char`'s
transport already uses (`(Ord_instance_Int).leq`) — **not** a cross-package
`import`, which `33 §3.2` leaves out of scope. So the map needs **no
module-system extension**: it is package Ken resolving a `where Ord k`
constraint, self-contained.

`Ord k` supplies **both** faculties the map needs:

- the **decision** `leq` — the search-tree branch test (`lookup`/`insert`
  descend by `leq`), and
- the **laws** `refl`/`antisym`/`trans`/`total` — the hypotheses the invariant
  and correctness proofs discharge with (§5).

There is **no separate `DecEq k` constraint**: key equality is *derived from the
order*, `k = k' :⟺ leq k k' ∧ leq k' k`. This **supersedes** `37 §3.3`'s earlier
"`DecEq` for membership, `Ord` only for ordered ops" split — a search tree needs
the order for its *core* operations, so `Ord k` is the single keying constraint
throughout.

### 2.1 Key identity via `Ord.antisym` — keys must be canonical (hard)

`insert k v` must decide whether `k` **already occurs** (overwrite the value) or
is **new** (extend). It decides this by order: a stored `k'` is *the same key*
as `k` exactly when `leq k k' ∧ leq k' k`. To conclude these are the **same key
propositionally** — `Equal k k'`, needed for the overwrite/uniqueness reasoning
(§5.3) — the proof invokes `Ord.antisym : IsTrue (leq x y) → IsTrue (leq y x) →
Equal k x y`.

Because that conclusion is the kernel's `Equal`, **ADR 0010 applies as a hard
soundness constraint, not a footnote.** `antisym` is a theorem only when `leq`'s
induced equality agrees with the carrier's *definitional* equality — i.e. only
over a **canonical** carrier (exactly one representation per semantic key):

- **Sound** for `Int`/`Char`/`Bool` (canonical carriers).
- **Unsound** over a **non-canonical** carrier — many representations per value,
  e.g. `Decimal = MkDecimalPair coeff exp` where `10×10⁻¹` and `1×10⁰` are
  distinct pairs denoting `1`. There a postulated `antisym` proves `Equal`
  between distinct representatives and injectivity refutes it → **inhabits
  `Bottom`** (the `DecEq Decimal` trap, ADR 0010). Such a type is **not** a
  lawful `Ord` key and therefore **not** a lawful `Map` key.

The map **inherits** ADR 0010's canonicity obligation on its key type — it does
not create a new one.

**Blast-radius localization (Architect ruling).** Only reasoning that must
identify two *order-equivalent* keys as propositionally equal needs the
`antisym → Equal` step — the **overwrite/uniqueness** face (§5.3, where `insert`
overwrites) **and the `Distinct`-discharge lemma** (§5.3 — `insert`/`from_list`-
reachable ⟹ no order-equivalent duplicates). Every law's *own* proof —
including agreement (law 5), whose matched-node agreement reduces to **`refl`**
given its `Distinct` precondition — leans on **`refl`**/**`trans`**/**`total`**
and needs **no `Equal` promotion**. Confining `antisym → Equal` to the overwrite
law and the discharge lemma keeps the canonical-carrier dependency **localized
and auditable** — every law holds over the induced order alone (law 5 given
`Distinct`; the antisym-using *discharge* of `Distinct` is where the
canonical-carrier obligation attaches, `54 §5.2`). (This is the **canonicity**
axis, orthogonal to *buildability*: laws 1/2/3/5 are Branch-B proofs hitting
Gap A + Gap B, §5/§7d; law 4 and the non-inductive laws have landed.)

## 3. Representation — a bare ordered binary search tree

The carrier is a plain, **unbalanced** ordered binary search tree (Architect
ruling, decide-on-intrinsic-merits + YAGNI):

```
data Tree k v = Leaf | Node (Tree k v) k v (Tree k v)
```

`Map k v` is `Tree k v` under the **ordering invariant** `Ordered` (§5.1); a
"valid map" is an ordered tree. **No balance metadata** (height/color/size) is
carried: the correctness laws (§5.2) hold over an *unbalanced* BST, so balance
is a **perf property orthogonal to correctness** (§6). Carrying speculative
metadata now would burden every proof to thread a field that buys nothing this
WP, and would guess the follow-on's scheme (AVL vs red-black vs weight-balanced)
wrong.

**The balancing follow-on is a superseding representation, not an extension.**
It introduces whatever metadata its scheme needs and **re-proves the same small
law set** under the new rep. That bounded re-proof is the honest, explicit price
of the correctness-now / perf-later split (flagged in §7, not hidden). The
**surface API (§4) is representation-independent at the signature level**, so it
is stable across the rep change — only the bodies and the invariant's balance
conjunct change.

Package home: the `collections` package (`catalog/packages/collections/`, alongside the
landed `List`/`Nat` floor); exact file path is Foundation's to fix.

## 4. Core API

Signatures are representation-independent (they do not vary by tree kind; only
the bodies do). `Option`/`List` are the landed L2 inductives (`34 §1`); `k × v`
is the **Σ-pair** (`../10-kernel/13 §3`, right-nested Σ with η — the same
construct as `run_state`'s result in `36 §4.5`, **distinct** from the inductive
`Prod`). The exact spelling of the `where`/implicit-argument surface follows
`33 §5.4`; any still-open surface-syntax token is tagged `(oracle)`.

```
empty    : {k v} → Map k v
insert   : {k v} → where Ord k ⇒ k → v → Map k v → Map k v
lookup   : {k v} → where Ord k ⇒ k → Map k v → Option v
member   : {k v} → where Ord k ⇒ k → Map k v → Bool
to_list   : {k v} → Map k v → List (k × v)          -- ascending key order
from_list : {k v} → where Ord k ⇒ List (k × v) → Map k v
fold     : {k v b} → (k → v → b → b) → b → Map k v → b   -- ascending key order
```

**`delete` is deferred** — operation *and* proof together (§7), not shipped as
an unproved op under the "proved map" banner. `letter-frequency`'s critical path
(insert + lookup + ordered iteration) does not need it.

### 4.1 `empty`, `insert`, `lookup`, `member`

- `empty = Leaf`.
- `insert k v` descends by `leq`: at `Node l k' v' r`, if `leq k k' ∧ leq k' k`
  the keys coincide → **overwrite the value** at that node; else recurse into
  the left (`k < k'`) or right (`k' < k`) subtree; at `Leaf`, create `Node Leaf
  k v Leaf`.
- `lookup k` descends by the *same* `leq` decisions: returns `Some v'` at the
  coinciding node, `None` at `Leaf`.
- `member k m = is_some (lookup k m)`.

### 4.2 `to_list` / `from_list` (ordered iteration)

- `to_list` is the **in-order** traversal: `to_list Leaf = Nil`,
  `to_list (Node l k v r) = append (to_list l) (Cons (k, v) (to_list r))` (reusing
  the landed `list_append`, `catalog/packages/collections`). Over an `Ordered` tree its
  output keys are **ascending** (§5.3, the load-bearing law).
- `from_list` folds `insert` over the list (`from_list = foldr (λ (k,v) m. insert
  k v m) empty`); the result is `Ordered` (invariant preserved, §5.1) and
  last-writer-wins on duplicate keys.

### 4.3 `fold`

`fold f z m` folds `f` over the entries in **ascending key order** — the same
order as `to_list`. The spec pins the *order contract* (`fold f z m` agrees with
the left fold of `f` over `to_list m`), not a particular recursion; the build
supplies the in-order recursion.

### 4.4 `Set a = Map a Unit`

`Set a := Map a Unit`, with `insert a s := insert a tt s`, `member`/`to_list`
projecting the keys. Its laws are the map's at `v := Unit`. This is what retires
the opaque `Set` primitive alongside `Map` (§1.1).

## 5. Invariant + correctness — real proof terms

The `§5` laws split by **whether the proof inducts over the (non-nullary)
`Tree`/`List` carrier at all**, and then by which construction capability that
induction needs. There were **two distinct capability walls** — both
**build-completeness shortfalls against already-specified behavior**, not spec
gaps (the elaborator lagged the spec). **Both have since landed** (Gap A
`19955d8`, Gap B `282856c`), and their per-law proof strategy is elaborated in
`54-map-verified-laws.md` (`map-verified-laws` WP). Foundation builds the proof
terms in **two staged units**: the **convoy idiom** (Gap-B dependent induction)
+ **law 4** (`to_list`-ordered) + the non-inductive laws are **landed** as real
proofs in `map.ken` (Unit 1 — law 4's conclusion built once the enabling kernel
conversion fix landed, `obs-eq-termination` `9cf468a`); laws 1/2/3/5's
**stuck-comparison transport** (Gap A) then landed via the **`trans`/`cong`
route-around** (`54 §3`, §7). Honesty about the boundary: **permutation** is the
one `Map` correctness law that stays deferred (§7c).

- **Gap B — dependent-motive recovery over non-nullary families.** A proof that
  **inducts** over `Tree`/`List` and must **narrow a hypothesis about the
  scrutinee** through the `match` was blocked: the per-branch type-narrowing
  path was gated to **nullary constructors only**, so a `Cons`/`Node` scrutinee
  fell to `infer_match`'s **constant** motive (no narrowing). `dependent-match-
  nonnullary` (`282856c`) **widened the gate** to any **flat, non-indexed**
  family (`List`/`Tree`): `elab.rs` `dependent_eligible` at **`:535-553`**
  (`ind.indices.is_empty()` over a bare-`Var` scrutinee). This realizes the
  general mechanism the spec already committed to (`../30-surface/34-data-
  match.md §3.2`).
- **Gap A — transport over a stuck comparison.** A proof that *additionally*
  must **reduce/align an abstract, irreducibly *stuck* `leq k k'`** (keys are
  variables, not the concrete constructors `Ord Bool` case-splits on) needs a
  **propositional-rewrite / transport (`J`/`cast`) step** to fire the internal
  `if`. `surface-transport` (`19955d8`) **surfaced** the `J` former + the
  derived `catalog/packages/transport/transport.ken` combinators (`53-transport.md`),
  reducing to the kernel's **existing** `Term::J`/`Cast` (`Refl` still checks
  only pre-existing convertibility; `J` is the reachable transport).

**In this WP (Branch A) — the two *non-inductive* laws.** Only proofs that never
induct clear both walls, discharging by the landed idiom (`51 §6`, `Ord Bool` /
`DecEq Bool.sound`): `Ordered empty` (`Ordered Leaf = ⊤`, closed by `tt`) and
`lookup k empty = None` (`lookup k Leaf = None`). Neither case-splits a
`Node`/`Cons`, so neither needs hypothesis-narrowing or transport.

**Branch B — the five inductive laws** (§7d `map-verified-laws`). **All five
have since landed** as real proofs (`54`); each is tagged with the wall(s) it
cleared:

- **Gap B only — LANDED:** the **`to_list` ordered law** + its two list lemmas
  (§5.3) — inducts over `Tree`/`List` (dependent-motive recovery) but is
  **comparison-free** (`to_list` never calls `leq`), so it cleared Gap A; built
  as `to_list_ordered` (`map.ken`).
- **Gap A + Gap B — LANDED:** `Ordered`-preservation (§5.1), found-after-insert
  and locality (§5.2 laws 2–3), and the `to_list`↔`lookup` agreement (§5.3) —
  each both inducts over the carrier **and** aligns a stuck `leq k k'`, via the
  convoy idiom + the `trans`/`cong` route-around (`54 §3`).

**Buildability, not soundness.** All five laws are **true** and spec-consistent
(they use the dependent-match `34` already specifies), and **all five are now
built** as real proof terms (`54`) — permutation is the one `Map` law that stays
deferred (§7c). **No `Axiom` stubs** — no proof was ever postulated; "proved,
not tested" holds. And **no shipped code leans on a deferred law** — the
operations are plain functions, not `Ordered`-indexed / proof-carrying types
(the AC3 guardrail, §9).

### 5.1 The ordering invariant `Ordered` (naturally `Ω`)

```
Ordered : {k v} → where Ord k ⇒ Tree k v → Ω
Ordered Leaf              = ⊤
Ordered (Node l k v r)    = all_keys (λ k'. IsTrue (leq k' k)) l
                          ∧ all_keys (λ k'. IsTrue (leq k  k')) r
                          ∧ Ordered l ∧ Ordered r
```

where `all_keys : (k → Ω) → Tree k v → Ω` is the `Ω`-valued structural recursion
"every key in the subtree satisfies `P`". `Ordered` is **naturally `Ω`** — built
from the `IsTrue b := Equal Bool b True` bridge (`51 §2`) and the derived
`Ω`-conjunction `∧` (`16 §1.3`) — and is a **definition the prover unfolds, out
of `trusted_base()`**, never a postulate (the `37 §6` surface-minimality
discipline: an opaque invariant makes the obligation undischargeable or
circular). It models `37 §6`'s `is_sorted` exactly, lifted from lists to trees.

**Preservation.** `Ordered empty` is immediate (`Ordered Leaf = ⊤`, closed by
`tt` — **Branch A, built**; no induction). `Ordered m ⇒ Ordered (insert k v m)`
is **Branch B, LANDED** (`map-verified-laws`, §7d — Gap A via the `trans`/`cong`
route-around + Gap B): it inducts over `m : Tree` narrowing the `Ordered`
hypothesis (Gap B) **and** aligns the stuck `leq k k'` at each `Node` to thread
key bounds (Gap A). It makes `lookup` provably correct — both capabilities
landed (`preserves_ordered`).

### 5.2 The core `lookup` laws (`refl`/`total`, no `Equal` promotion)

1. **`lookup k empty = None`** — immediate, no induction. **Branch A, built.**
2. **`lookup k (insert k v m) = Some v`** (found-after-insert) — induction on
   `m`; `lookup` retraces `insert`'s path under the *same* key, and at the node
   where `insert` placed/overwrote `k`, `leq k k` holds by **`refl`** so the
   found branch returns `v` (no `antisym → Equal` — the value is `v` whichever
   key label the node carries). **Branch B, LANDED (§7d):** inducts over `m`
   (Gap
   B) and aligns the stuck `leq k k'` at each interior `Node` via the
   route-around to prove the `insert` and `lookup` paths coincide
   (`lookup_found_after_insert`).
3. **locality** — `distinct k k' ⇒ lookup k' (insert k v m) = lookup k' m`,
   where `distinct k k' := ¬ (IsTrue (leq k k') ∧ IsTrue (leq k' k))`
   (order-distinct); inserting `k` does not perturb an order-distinct `k'`'s
   lookup, by induction with `Ordered m` + `trans`/`total` (order laws, **not**
   `Equal`). **Branch B, LANDED (§7d):** carrier induction + stuck-`leq`
   alignment
   via the route-around (`lookup_locality`).

### 5.3 `to_list` ordered law + agreement (both landed)

- **Ordered law (load-bearing) — Branch B, LANDED (`map-verified-laws` Unit 1,
  `54`; was §7d-deferred — Gap B only).**
  `Ordered m ⇒ is_sorted (λ a b. leq (fst a) (fst b)) (to_list m)` — the in-order
  traversal is **ascending by key**, reusing `37 §6`'s `is_sorted` predicate. It
  is **comparison-free** (`to_list` never calls `leq`; the proof feeds `IsTrue`
  witnesses from `Ordered`'s `all_keys` into `is_sorted`'s `Ω`-conjuncts via two
  list lemmas — `is_sorted`-over-`++` and `all_keys`↔`all_in_list (to_list)` — never
  reducing a stuck boolean), so it **clears Gap A**. But it **inducts over
  `Tree`/`List`** narrowing an `Ordered`/`all_keys` hypothesis, so it hits **Gap
  B** (the nullary dependent-match gate). Consequence: `letter-frequency`'s
  deterministic ordered output was honest **by the conformance test**
  (red-until-built) **in Map-build**, and is now honest **by proof** —
  `map-verified-laws` Unit 1 built it (`to_list_ordered`; §8). Still **without
  touching permutation** (§7c).
- **Agreement — Branch B, LANDED (§7d).** `Ordered m →
  Distinct leq m → lookup k m = assoc k (to_list m)` — a key's map lookup agrees
  with a linear scan of its ordered entry list (`assoc` the landed list-lookup
  shape). **The key-uniqueness precondition `Distinct leq m` is required** (no
  two nodes carry order-equivalent keys): `lookup`'s BST descent and `assoc`'s
  in-order scan are different traversal orders that agree **iff** keys are
  unique — `Ordered`'s weak `≤`/`≥` bounds alone admit duplicates (e.g. `Node
  (Node Leaf k v1 Leaf) k v2 Leaf`, where `lookup k = Some v2` but `assoc k
  (to_list) = Some v1`). `Distinct` is the no-duplicate invariant `insert`/
  `from_list` maintain by construction (discharged via `antisym` in a separate
  lemma, Foundation's). Law 5's *own* proof is **antisym-free** — given
  `Distinct`, the matched-node agreement is `refl` (`54 §5.2`), so the law is
  carrier-general. `Ordered` is **unchanged** — the precondition is added, not
  folded in. Its
  proof **inducts** (Gap B) **and** must **align `lookup`'s comparison descent
  with the ordered list**, matching a **stuck** `leq k k'` (Gap A).

**The overwrite/uniqueness face** — where two order-equivalent keys with
`leq k k' ∧ leq k' k` are identified so `insert` overwrites rather than
duplicates (keeping `Ordered` and making the map a genuine partial *function*) —
invokes `Ord.antisym : … → Equal k k'`. So does the **`Distinct`-discharge
lemma** (`insert`/`from_list`-reachable ⟹ no order-equivalent duplicates). These
two are where the §2.1 canonical-carrier constraint bites. **Law 5's own
statement does not** — its matched-node agreement is `refl` given `Distinct`
(carrier-general, holding even where `antisym` is false); the antisym dependency
sits in the discharge lemma, not the law. Localized to the overwrite face +
discharge lemma by design.

### 5.4 The proofs are parametric in the dictionary

The map's proofs — the two Branch-A laws and all five Branch-B inductive laws,
all built (§5, §7d) — take `d : Ord k` and use
`d.antisym`/`d.trans`/`d.total`/`d.refl` as **hypotheses**. They are therefore
**real proof terms independent of whether a given `Ord k` instance's own laws
are `Axiom` (Int/Char) or real (Bool)** — the proof does not care *how* the
order laws were established, only that the dictionary provides them (so a
Branch-B proof, when built, is real regardless of the key's instance). Two
consequences pin the conformance (§8):

- **AC3's "fails against a stub" net is at the *map-proof* level**, not the
  `Ord k`-instance level: replace a **shipped (Branch-A)** proof term with
  `Axiom` and the discriminating case must **fail** — and **for the right
  reason** (it must *exercise* the swapped-to-`Axiom` proof via a downstream
  consumer or kernel obligation that needs the real term, not merely detect its
  textual absence). The **Branch-B** laws (§7d) are **absent, not `Axiom`** —
  named-deferred, so nothing stubs them and no client leans on them (§9 AC3
  guardrail). ([[lawful-class-instances-must-carry-law-proofs]].)
- **Operations run over `Char` keys** end-to-end through the real interpreter:
  `Ord Char`'s `leq = int_leq` **computes**, so `insert`/`lookup`/`to_list`
  evaluate on real Char-keyed maps (the `letter-frequency` shape) even though
  `Char`'s *order-laws* are `Axiom`. The **fully-real-all-the-way-down** proof
  witness uses `Ord Bool`/`Ord Two` keys (the only carriers with non-`Axiom`
  order-laws today, `51 §6`).

**Non-canonical carrier ≠ non-`Axiom` laws** — these are the two orthogonal ADR
0010 axes: a `Char` key is *canonical* (so the map's `antisym` use is **sound**)
even though `Ord Char`'s laws are *audited* (`Axiom`); a `Decimal` key would be
*non-canonical* (so the `antisym` use inhabits `Bottom` — §2.1) regardless of
how its laws are filed.

## 6. Perf note

Operations are **O(log n) on a balanced tree, O(n) worst-case on this
unbalanced one** — stated honestly. Balance is a **perf property, separable from
correctness** (§3), and is the §7(a) follow-on. **HAMT is explicitly out of
scope** — a named, parked *later* fast-map, only if profiling demands, and also
proved (the operator's "tree-first, HAMT-later"; mirrors `Array`'s abstract
persistent-tree parking, `37 §3.2`).

## 7. Proof-scope split — this WP vs. tracked follow-on

Confirmed by Architect (structure/scope) + spec-leader + Steward.

**In this WP** (Branch A — the two *non-inductive* laws; neither case-splits a
`Node`/`Cons`, so both clear Gap A **and** Gap B; §5):

- `Ordered`'s / `all_keys`' **definitions** (§5.1) — the `Ω`-predicates
  themselves (`declare_def` recursions), not any proof about them.
- `Ordered empty` (`Leaf` → `tt`) and `lookup k empty = None` (`Leaf` → `None`)
  — the two shipped **law proofs**.

**Deferred to a tracked follow-on** (each a *named* follow-on, never a silent
gap — AC3):

- **(a) balance / rotations / O(log n) worst-case** — perf, separable; a
  **superseding representation** with a bounded re-proof of this same law set
  (§3). Explicitly the honest price of correctness-now/perf-later.
- **(b) `delete`** — the **operation and its correctness proof, together** (not
  op-in/proof-out, which would muddy the proved-not-tested identity). Nothing on
  `letter-frequency`'s critical path needs it.
- **(c) the `to_list` multiset/permutation law** ("`to_list` lists exactly the
  inserted entries, once each") — **proof-relevant** (distinct interleavings are
  distinct derivations), so it cannot be `data Perm : Ω` directly; it needs
  `∥Perm_rel∥` or a count-equality form (`37 §6`,
  [[proof-relevant-inductive-cannot-be-declared-at-omega]]), **and** Perm
  discharge is the known `C5` prover gap. The **ordered** `to_list` law (§5.3, in
  (d) below) is its naturally-`Ω` substitute — it sidesteps permutation
  entirely, and delivers ordered iteration by the conformance test in Map-build
  (its *proof* landed with `map-verified-laws` Unit 1, §5.3/§8).
- **(d) `map-verified-laws` — the five inductive laws** (§5.1 preservation, §5.2
  found-after-insert + locality, §5.3 `to_list` ordered law + agreement). Each
  **inducts over the non-nullary `Tree`/`List` carrier** narrowing a hypothesis
  → **Gap B** (dependent-motive recovery, the widened non-indexed gate `elab.rs`
  `dependent_eligible` at `:535-553`, realizing `../30-surface/34-data-match.md
  §3.2`). Four of the five *additionally* must **align a stuck `leq k k'`** →
  **Gap A** (the `J` former + `catalog/packages/transport/transport.ken`, surfacing the
  kernel's *existing* `Term::J`/`Cast`, `53-transport.md` — the same frontier
  `lawful_classes.ken`'s relational laws need). `to_list`-ordered is
  **Gap-B-only**. **Both capability WPs are now landed** (Gap A `19955d8`, Gap B
  `282856c`) — each a build-completeness fix against already-specified behavior,
  **zero `trusted_base` delta**; the five proofs' **per-law strategy is
  elaborated in `54-map-verified-laws.md`**, and Foundation builds the terms on
  top. **Not `Axiom`-stubbed** — named, never postulated (§5); **all five
  inductive
  laws have since landed** (laws 1/2/3/5 via the `trans`/`cong` route-around,
  law 4 via `obs-eq-termination`). Permutation is the only `Map` law still
  deferred.

## 8. Conformance pointer

`/conformance` (CV-authored) drives the **real interpreter** — `insert`/`lookup`
round-trips, ordered `to_list` iteration, and the `letter-frequency` shape (Char
keys: count occurrences, emit in ascending key order) — **not** a hand-fed
harness ([[conformance-hand-feeds-the-deliverable]]). Cases pin the value
**and** the ascending-key order (the `to_list` law is what makes output
deterministic), tagging any deferred surface spelling `(oracle)`. The
proved-not-stubbed discriminator (AC3) flips at the **map-proof** level and must
fail **for the right reason** (§5.4) — applied to the **two shipped Branch-A
proofs** (`Ordered empty`, `lookup k empty = None`). **Ordered iteration is
honest by the conformance test from Map-build** (the `to_list`-ordered law was
red-until-built, Gap B — §5.3), and is now honest **by proof** —
`map-verified-laws` built it (`54-map-verified-laws.md`); **all five Branch-B**
proofs (§7d, laws 1/2/3/4/5) are now landed — AC3's discriminating target is
met.

## 9. Acceptance

- **AC1 — net-negative TCB (reworded from the frame's perishable "unchanged").**
  (a) `git diff origin/main -- crates/ken-kernel/` **empty** (the primitive
  lives in `ken-elaborator/prelude.rs`, not the kernel crate — untouched). (b)
  `trusted_base()` **shrinks** by exactly the retired `Map`/`Set`, **zero
  additions**. Discriminating grep: the new `Map`/`Tree` and its ops are
  `declare_inductive`/`declare_def` (kernel-rechecked); **`declare_primitive` /
  `declare_postulate` absent** for them; **and** the old `Map`/`Set` primitive
  is **gone** from `trusted_base()`.
- **AC2 — operations correct end-to-end** through the real interpreter (the §8
  round-trips + ordered iteration + `letter-frequency` shape).
- **AC3 — proved, not stubbed.** The **two Branch-A** §5 proofs (`Ordered
  empty`, `lookup k empty = None`) + the `Ordered`/`all_keys` **definitions** are
  **real proof terms** (no induction, so both capability walls are cleared); a
  discriminating test **fails against a stub/`Axiom` map-proof, for the right
  reason** (§5.4). The **five Branch-B** inductive laws (§7d
  `map-verified-laws`) are now **all landed** as real proof terms, **never
  `Axiom`-stubbed**; permutation stays the one deferred `Map` law. The guardrail
  held throughout — no shipped code ever leaned on a deferred law (operations
  are plain functions, not `Ordered`-indexed / proof-carrying types). Named
  follow-on, not a silent gap.
- **AC4 — no regression.** `cargo test --workspace` green; lawful `Ord` and the
  rest of `catalog/packages/` behave identically pre/post.
- **AC5 — build-lane retirement is real (hard Foundation build AC).** The
  `prelude.rs` `Map`/`Set` primitive is **removed** and the `es2_acceptance.rs`
  "Map/Set are primitives" assertion **flips** (they are now proved inductives),
  with the `trusted_base()`-shrink **verified as landed** by grep on the merged
  code — the retirement is only real when the code drops it, not when the spec
  says so.
