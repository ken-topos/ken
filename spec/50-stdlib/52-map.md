# A proved, pure `Map k v` over `Ord k`

> Status: **DRAFT v0 (VAL2 #8 / OQ-A).** A `packages/` catalog entry that
> closes `letter-frequency`'s gap — `Map` was a bare opaque type with **zero
> operations**. Operator-locked shape (OQ-A, 2026-07-03): a **proved, pure,
> `Ord k`-keyed** associative map, **shipped as ordinary package Ken — out of
> `trusted_base()`, not a kernel builtin**. It is the container companion to
> the lawful classes (`51`): its structure is an ordinary inductive and **every
> correctness law is a real kernel proof, not a postulate** (`README §2`,
> `../20-verification/21 §3`). **No new kernel feature:** the carrier is a
> `data` type (`../30-surface/34 §1`), the operations are `view` defs, the laws
> are `Ω` propositions (`../10-kernel/16 §1`); if the build adds a kernel
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

**Blast-radius localization (Architect ruling).** Only the reasoning that must
identify two *distinct-looking* keys as propositionally equal needs the
`antisym → Equal` step — namely the **overwrite/uniqueness** face (§5.3). The
three core `lookup` laws (§5.2) instead lean on **`refl`** (`IsTrue (leq k k)`,
the found key equals itself) and **`total`**/`leq`-determinism, which need **no
`Equal` promotion**. Confining `antisym → Equal` to the overwrite law keeps the
canonical-carrier dependency **localized and auditable** — the found-after-
insert and locality laws hold over the induced order alone.

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

Package home: the `collections` package (`packages/collections/`, alongside the
landed `List`/`Nat` floor); exact file path is Foundation's to fix.

## 4. Core API

Signatures are representation-independent (they do not vary by tree kind; only
the bodies do). `Option`/`List` are the landed L2 inductives (`34 §1`); `k × v`
is the **Σ-pair** (`../10-kernel/13 §3`, right-nested Σ with η — the same
construct as `runState`'s result in `36 §4.5`, **distinct** from the inductive
`Prod`). The exact spelling of the `where`/implicit-argument surface follows
`33 §5.4`; any still-open surface-syntax token is tagged `(oracle)`.

```
empty    : {k v} → Map k v
insert   : {k v} → where Ord k ⇒ k → v → Map k v → Map k v
lookup   : {k v} → where Ord k ⇒ k → Map k v → Option v
member   : {k v} → where Ord k ⇒ k → Map k v → Bool
toList   : {k v} → Map k v → List (k × v)          -- ascending key order
fromList : {k v} → where Ord k ⇒ List (k × v) → Map k v
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
- `member k m = isSome (lookup k m)`.

### 4.2 `toList` / `fromList` (ordered iteration)

- `toList` is the **in-order** traversal: `toList Leaf = Nil`,
  `toList (Node l k v r) = append (toList l) (Cons (k, v) (toList r))` (reusing
  the landed `list_append`, `packages/collections`). Over an `Ordered` tree its
  output keys are **ascending** (§5.3, the load-bearing law).
- `fromList` folds `insert` over the list (`fromList = foldr (λ (k,v) m. insert
  k v m) empty`); the result is `Ordered` (invariant preserved, §5.1) and
  last-writer-wins on duplicate keys.

### 4.3 `fold`

`fold f z m` folds `f` over the entries in **ascending key order** — the same
order as `toList`. The spec pins the *order contract* (`fold f z m` agrees with
the left fold of `f` over `toList m`), not a particular recursion; the build
supplies the in-order recursion.

### 4.4 `Set a = Map a Unit`

`Set a := Map a Unit`, with `insert a s := insert a tt s`, `member`/`toList`
projecting the keys. Its laws are the map's at `v := Unit`. This is what retires
the opaque `Set` primitive alongside `Map` (§1.1).

## 5. Invariant + correctness — real proof terms

All laws below are carried as **real kernel proofs**, parametric in the `Ord k`
dictionary (§5.4). The proof idiom is the landed one (`51 §6`, `Ord Bool` /
`DecEq Bool.sound`): case-split on the tree constructor with the `Ω`-motive
eliminator (`14 §3`, K4), reduce the operation redex so an equation goal
`Equal … (Some v)` / `Equal … True` whnf-collapses and closes with `refl`/`tt`;
an impossible ordering branch closes with `absurd`.

### 5.1 The ordering invariant `Ordered` (naturally `Ω`)

```
Ordered : {k v} → where Ord k ⇒ Tree k v → Ω
Ordered Leaf              = ⊤
Ordered (Node l k v r)    = allKeys (λ k'. IsTrue (leq k' k)) l
                          ∧ allKeys (λ k'. IsTrue (leq k  k')) r
                          ∧ Ordered l ∧ Ordered r
```

where `allKeys : (k → Ω) → Tree k v → Ω` is the `Ω`-valued structural recursion
"every key in the subtree satisfies `P`". `Ordered` is **naturally `Ω`** — built
from the `IsTrue b := Equal Bool b True` bridge (`51 §2`) and the derived
`Ω`-conjunction `∧` (`16 §1.3`) — and is a **definition the prover unfolds, out
of `trusted_base()`**, never a postulate (the `37 §6` surface-minimality
discipline: an opaque invariant makes the obligation undischargeable or
circular). It models `37 §6`'s `isSorted` exactly, lifted from lists to trees.

**Preservation (load-bearing, this WP):** `Ordered empty` and `Ordered m ⇒
Ordered (insert k v m)`. This is what makes `lookup` correct — proved by
induction on the tree, using `trans`/`total` to thread the key bounds through
the recursive `insert`.

### 5.2 The core `lookup` laws (this WP; `refl`/`total`, no `Equal` promotion)

1. **`lookup k empty = None`** — immediate (`empty = Leaf`).
2. **`lookup k (insert k v m) = Some v`** (found-after-insert) — induction on
   `m`. `lookup` retraces `insert`'s path under the *same* key and comparisons;
   at the node where `insert` placed/overwrote `k`, `leq k k` holds by
   **`refl`** so the found branch fires and returns `v`. Leans on `refl` +
   `leq`-determinism; the returned value is `v` whichever key label the node
   carries, so **no `antisym → Equal` step is needed.**
3. **locality** — `distinct k k' ⇒ lookup k' (insert k v m) = lookup k' m`,
   where `distinct k k' := ¬ (IsTrue (leq k k') ∧ IsTrue (leq k' k))`
   (order-distinct). Inserting `k` does not perturb the lookup of an
   order-distinct `k'`. Proved by induction using `Ordered m` + `trans`/`total`
   to show `k`'s insertion path and `k'`'s lookup path diverge. Uses the order
   laws, **not** `Equal`.

### 5.3 `toList` ordered law + agreement (this WP)

- **Ordered law (load-bearing):** `Ordered m ⇒ isSorted (λ a b. leq (fst a)
  (fst b)) (toList m)` — the in-order traversal is **ascending by key**, reusing
  `37 §6`'s `isSorted` predicate. This delivers the frame's "ordered iteration"
  conformance and `letter-frequency`'s deterministic output **without touching
  permutation** (§7c). Naturally `Ω`.
- **Agreement (naturally `Ω`, included):** `lookup k m = assoc k (toList m)` — a
  key's map lookup agrees with a linear scan of its ordered entry list (`assoc`
  the landed list-lookup shape). Naturally structural/`Ω`; ties the tree and its
  list view together.

**The overwrite/uniqueness face** — where two distinct-looking keys with
`leq k k' ∧ leq k' k` are identified so `insert` overwrites rather than
duplicates (keeping `Ordered` and making the map a genuine partial *function*) —
is the **one** place `Ord.antisym : … → Equal k k'` is invoked, and thus the one
place the §2.1 canonical-carrier constraint bites. It is localized here by
design.

### 5.4 The proofs are parametric in the dictionary

The map's proofs take `d : Ord k` and use
`d.antisym`/`d.trans`/`d.total`/`d.refl` as **hypotheses**. They are therefore
**real proof terms independent of whether a given `Ord k` instance's own laws
are `Axiom` (Int/Char) or real (Bool)** — the proof does not care *how* the
order laws were established, only that the dictionary provides them. Two
consequences pin the conformance (§8):

- **AC3's "fails against a stub" net is at the *map-proof* level**, not the
  `Ord k`-instance level: replace the map's invariant/correctness term with
  `Axiom` and the discriminating case must **fail** — and **for the right
  reason** (it must *exercise* the swapped-to-`Axiom` proof via a downstream
  consumer or kernel obligation that needs the real term, not merely detect its
  textual absence). ([[lawful-class-instances-must-carry-law-proofs]].)
- **Operations run over `Char` keys** end-to-end through the real interpreter:
  `Ord Char`'s `leq = int_leq` **computes**, so `insert`/`lookup`/`toList`
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

**In this WP** (all naturally-`Ω`, real proof terms, no truncation, unblocked):

- `Ordered` invariant + **its preservation by `insert`** (§5.1) — load-bearing.
- The three core `lookup` laws (§5.2): `lookup k empty = None`; `lookup k
  (insert k v m) = Some v`; the order-distinct locality law.
- The **`toList` ordered law** + the `lookup`↔`toList` agreement (§5.3).

**Deferred to a tracked follow-on** (each a *named* follow-on, never a silent
gap — AC3):

- **(a) balance / rotations / O(log n) worst-case** — perf, separable; a
  **superseding representation** with a bounded re-proof of this same law set
  (§3). Explicitly the honest price of correctness-now/perf-later.
- **(b) `delete`** — the **operation and its correctness proof, together** (not
  op-in/proof-out, which would muddy the proved-not-tested identity). Nothing on
  `letter-frequency`'s critical path needs it.
- **(c) the `toList` multiset/permutation law** ("`toList` lists exactly the
  inserted entries, once each") — **proof-relevant** (distinct interleavings are
  distinct derivations), so it cannot be `data Perm : Ω` directly; it needs
  `∥Perm_rel∥` or a count-equality form (`37 §6`,
  [[proof-relevant-inductive-cannot-be-declared-at-omega]]), **and** Perm
  discharge is the known `C5` prover gap. The **ordered** `toList` law (§5.3) is
  its naturally-`Ω` substitute that ships now.

## 8. Conformance pointer

`/conformance` (CV-authored) drives the **real interpreter** — `insert`/`lookup`
round-trips, ordered `toList` iteration, and the `letter-frequency` shape (Char
keys: count occurrences, emit in ascending key order) — **not** a hand-fed
harness ([[conformance-hand-feeds-the-deliverable]]). Cases pin the value
**and** the ascending-key order (the `toList` law is what makes output
deterministic), tagging any deferred surface spelling `(oracle)`. The
proved-not-stubbed discriminator (AC3) flips at the **map-proof** level and
must fail **for the right reason** (§5.4).

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
- **AC3 — proved, not stubbed.** The §5 invariant + laws are real proof terms; a
  discriminating test **fails against a stub/`Axiom` map-proof, for the right
  reason** (§5.4). Deferred laws (§7) are named follow-ons, not silent gaps.
- **AC4 — no regression.** `cargo test --workspace` green; lawful `Ord` and the
  rest of `packages/` behave identically pre/post.
- **AC5 — build-lane retirement is real (hard Foundation build AC).** The
  `prelude.rs` `Map`/`Set` primitive is **removed** and the `es2_acceptance.rs`
  "Map/Set are primitives" assertion **flips** (they are now proved inductives),
  with the `trusted_base()`-shrink **verified as landed** by grep on the merged
  code — the retirement is only real when the code drops it, not when the spec
  says so.
