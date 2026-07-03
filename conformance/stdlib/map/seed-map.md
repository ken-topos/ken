# A proved, pure `Map k v` over `Ord k` — conformance seed

Format: `../../README.md`. These pin the **Map-container deliverable**
(`docs/program/wp/Map-container.md`, `spec/50-stdlib/52-map.md`; VAL2 #8 /
OQ-A): a **proved, pure, `Ord k`-keyed** associative map shipped as ordinary
**package Ken — out of `trusted_base()`, not a kernel builtin**. Its carrier is
an ordinary inductive (`data Tree k v = Leaf | Node …`), its operations are
kernel-re-checked `view` defs, and **every correctness law is a real kernel
proof, not a postulate** (`52 §1`, `21 §3`). This closes
`letter-frequency`'s gap — `Map` was a bare opaque primitive with **zero
operations** — and **retires** that opaque primitive (net-negative TCB).

**Grounded (content-verified against the landed `52-map.md` off
`wp/Map-container@cf0a1df`, not heading numbers — the
`conformance-oracle-grounding-fallback` discipline):** `52 §3` (bare unbalanced
BST carrier, `data Tree k v = Leaf | Node (Tree k v) k v (Tree k v)`); `52 §4`
(the seven-op API — `empty`/`insert`/`lookup`/`member`/`toList`/`fromList`/
`fold`, **no `delete`**); `52 §5` (the invariant + correctness laws as real
proof terms — `Ordered` preservation, the three core `lookup` laws, the `toList`
ordered law + agreement); `52 §2.1` (key identity via `Ord.antisym → Equal`,
localized to the overwrite law, ADR 0010 canonical-carrier); `52 §9` (AC1–AC5).
Cross-checked against the landed `../classes/seed-lawful-classes.md`
(laws-PROVED ≡ zero-`trusted_base()`-delta; `Ord Bool` the only non-`Axiom`
order-law carrier today, `51 §6`), `../../challenge/C1-deceq-noncanonical` (the
non-canonical-carrier `Bottom` exploit),
`../../challenge/C6-lawful-ord-vs-stub`, and `../../challenge/C2`/`C5` (the
proof-relevant-`Ω` / verified-sort coupling that the deferred permutation law
inherits).

**Red-until-built — this seed is the build's acceptance TARGET, not hand-fed
green ([[conformance-hand-feeds-the-deliverable]]).** No proved-`Map` package
exists on `main` yet: `insert`/`lookup`/`toList`/`fromList`/`fold` are not
defined, and the opaque `Map`/`Set` primitive is still registered in
`ken-elaborator/.../prelude.rs` (its retirement is a **hard Foundation build
AC**, `52 §9` AC5). Every AC2/AC3 case below therefore stays **red on `main`**
until the Foundation build lands the proved package and drops the primitive;
each drives the **real** producer (real `insert`/`lookup`/`toList` calls; the
real `trusted_base_delta` cone walk) once built, never a hand-fed harness. The
`(oracle)` tag marks (a) the still-open `where Ord k` / implicit-argument and
constructor surface spellings (`52 §4` defers them to `33 §5.4`; pin the
**concept + value + order**, `(oracle)`-tag the spelling — the T1 over-freezing
guard) and (b) confirmation against the reference interpreter once the package
is built.

**Supersession — companion reconcile in
`../../surface/collections/seed-collections.md`.** Before OQ-A, `Map`/`Set` were
conformance-pinned as **abstract `DecEq`-keyed content-addressed heap
primitives** (kinds `0x07`/`0x08`, insertion-order-independent canonical
identity, O(1) slot-id equality — `37 §3.3` DRAFT). This WP's `/spec` **flips**
`37 §3.3` to the proved-BST model (`Ord k`-keyed, **extensional** identity via
ordered `toList`, out of `trusted_base()`). The superseded heap-`Map`/`Set`
cases in `seed-collections.md` are reconciled to this model in the **same
branch** (else the corpus self-contradicts the spec on merge — the
internal-consistency gate); see that file's supersession note. There are **not
two `Map`s**.

---

## AC1 — net-negative TCB (`Map` is derived Ken, not a primitive)

`52 §9` AC1: (a) the kernel crate is untouched; (b) `trusted_base()` **shrinks**
by exactly the retired `Map`/`Set`, **zero additions**. A proved map *requires*
a transparent inductive carrier (only constructors + an eliminator give the
case-split + redex reduction a proof needs — `52 §1.1`), so "proved"
structurally forces the retirement of the constructor-less opaque primitive.

### stdlib/map/carrier-inductive-ops-defs-not-primitive (soundness)
- spec: `52 §1.1`/`§3`/`§9` (AC1), `16 §1` (`Ω` props), `34 §1`
  (transparent inductive `data`)
- given: the built `Map` package (`packages/collections/…`) as admitted into the
  kernel environment.
- expect: `Tree k v` is admitted by **`declare_inductive`** (two constructors
  `Leaf`/`Node`, kernel-rechecked, an induction principle exists); `insert`/
  `lookup`/`member`/`toList`/`fromList`/`fold` are **`declare_def`** `view`s;
  the `Ordered` invariant + the `§5` laws are **`declare_def`** `Ω`-props /
  proof terms. **`declare_primitive` and `declare_postulate` are ABSENT** for
  `Map`/`Tree`/`Set` and every op and law. Assert the admission **kind** of each
  global structurally (grep the real admission call, not the source text).
- why: AC1's derived-not-primitive core and the `52 §1.1` structural premise.
  **`(soundness)`** — a `Map` law reaching the kernel as a `declare_postulate`
  (or the carrier as a `declare_primitive OpaqueType`) would re-grow
  `trusted_base()` and mean the "proof" is postulated. Verify the trust
  **level** by the admission primitive, not the name
  ([[kernel-backed-claim-grep-the-emission-not-the-name]]). (structural
  admission-kind; producer-grepped.)

### stdlib/map/kernel-crate-untouched (soundness)
- spec: `52 §9` AC1(a)
- given: the merged Map-container build diff.
- expect: `git diff origin/main -- crates/ken-kernel/` is **empty** — no new
  kernel rule, former, or `Decl` variant for `Map`/`Tree`/`Set`; the map is
  admitted through the **existing** `declare_inductive`/`declare_def` API. The
  primitive lived in `ken-elaborator/prelude.rs` (not the kernel crate), so the
  kernel is untouched by both its retirement and the map's arrival.
- why: AC1's zero-kernel-delta face — a new kernel feature for `Map` would be an
  over-build (`52` banner). **`(soundness)`** — the TCB shape is a kernel-crate
  fact. Structural (empty diff), not a value. (structural; empty kernel diff.)

### stdlib/map/opaque-primitive-retired-trusted-base-shrinks (soundness)
- spec: `52 §1.1`/`§9` (AC1(b), AC5), `30 §6` (`trusted_base()`
  item-2 taxonomy)
- given: `trusted_base()` **before** (opaque `Map`/`Set`
  `declare_primitive OpaqueType` present, item-2, like `String`/`Bytes`) and
  **after** the build.
- expect: **the membership flips one way only.** After: the opaque `Map`/`Set`
  primitive entries are **GONE** from `trusted_base()`, and **nothing new is
  added** for the proved map (its carrier/ops/laws are kernel-rechecked, §AC1
  above). Net delta is a **shrink** by exactly the two retired entries — the
  strong direction of zero-TCB. Assert both halves: old entries absent **and**
  zero new entries.
- why: AC1(b)/AC5 — "proved, not trusted" pays out as a **net-negative** trust
  delta ([[abstraction-visibility-feature-soundness-gate]]: zero-delta is
  grounded by the kernel being untouched **and** the retirement being real).
  **`(soundness)`** — a build that adds the proved map **without** removing the
  primitive grows the surface (two `Map`s) and misses AC1's strong direction;
  the removal is only real when the code drops it (AC5), verified structurally
  on the merged `trusted_base()`, not from the spec.
  Red-until-Foundation-retires. (structural trusted_base membership;
  net-negative.)

---

## AC2 — operations correct end-to-end through the real interpreter

`52 §9` AC2 + `§8`: `insert`/`lookup` round-trips, ordered `toList` iteration,
and the `letter-frequency` shape — **driving real ops**, reading back via real
`lookup`/`toList`, never hand-feeding the result. These are **value-flips**: the
correct op and a broken op land on **different observable values**. `Ord Char`'s
`leq = int_leq` **computes**, so Char-keyed maps evaluate end-to-end (the
`letter-frequency` shape) even though `Char`'s order-*laws* are `Axiom`
(`52 §5.4`); the fully-real proof witness (AC3) keys on `Bool`.

### stdlib/map/insert-lookup-roundtrip-some
- spec: `52 §4.1`/`§5.2` (law 2, found-after-insert), `§8`
- given: `lookup k (insert k v empty)` — a real `insert` into the real `empty`,
  read back by a real `lookup`. Run at `k : Char` (computes) and, for the
  all-the-way-down witness, `k : Bool`.
- expect: **reduces-to `Some v`.** The value threaded in comes back out under
  the *same* key. A broken `insert` (drops the entry) or `lookup` (misreads the
  branch) yields `None` or a wrong `Some v'` — the flip.
- why: AC2's found-after-insert round-trip, the `§5.2` law 2 made executable.
  **Drive the real producer:** construct via a real `insert`, read via a real
  `lookup` — not a hand-built `Node`. (reduces-to value-flip; oracle.)

### stdlib/map/lookup-order-distinct-key-is-none
- spec: `52 §4.1`/`§5.2` (law 1 + locality), `§8`
- given: `lookup k' (insert k v empty)` where `k'` is **order-distinct** from
  `k` (`¬ (leq k k' ∧ leq k' k)`); and `lookup k empty`.
- expect: **reduces-to `None`** in both — the absent key is not found; inserting
  `k` does not fabricate a hit for a distinct `k'` (the `§5.2` locality law's
  runtime face). A bug that returns the last-inserted value for **any** query
  (ignoring the key) yields `Some v` — the flip.
- why: AC2's absent/locality face; the non-degenerate partner of the
  found-after-insert case (a hit-only test is green-vs-green under a "return
  anything inserted" bug). (reduces-to value-flip; oracle.)

### stdlib/map/overwrite-last-writer-wins
- spec: `52 §4.1`/`§5.3` (overwrite/uniqueness — the one `antisym → Equal`
  site), `§2.1`
- given: `lookup k (insert k v2 (insert k v1 empty))` with `v1 ≠ v2`, and
  `toList (insert k v2 (insert k v1 empty))`.
- expect: `lookup` **reduces-to `Some v2`** (last writer wins), and `toList`
  yields a **single** entry `[(k, v2)]` — re-inserting an existing key
  **overwrites in place**, it does not duplicate the node (the map stays a
  genuine partial *function*, `Ordered` preserved). A bug that inserts a second
  `Node` for the same key yields `Some v1` (finds the first) or a two-entry
  `toList` — the flip.
- why: AC2 exercising the **overwrite** path — the single place `§5.3`'s
  `antisym`-identified-keys reasoning bites at runtime. Guards against a
  duplicate-on-reinsert bug that a fresh-key-only round-trip cannot see.
  (reduces-to value-flip; oracle.)

### stdlib/map/tolist-ascending-by-key (the ordered law, load-bearing)
- spec: `52 §4.2`/`§5.3` (the `toList` ordered law), `37 §6` (`isSorted`)
- given: a map built by inserting keys in a **deliberately non-ascending** order
  (e.g. `insert 3 · insert 1 · insert 2` over `Int`/`Char` keys), then `toList`.
- expect: **reduces-to a `List (k × v)` whose keys are strictly ascending** —
  `isSorted (λ a b. leq (fst a) (fst b)) (toList m)` holds regardless of
  insertion order (the in-order BST traversal over an `Ordered` tree). A bug
  that emits in insertion order, or a non-`Ordered` tree, yields a non-ascending
  list — the flip is on the **list order**, not just its element set.
- why: AC2's **load-bearing** ordered-iteration law (`§5.3`) — the property that
  makes `letter-frequency`'s output **deterministic**. Insertion out of order is
  the discriminator: an insertion-order-preserving bug passes a pre-sorted-input
  test (green-vs-green) but fails here. The `k × v` result element is the
  **Σ-pair** (`13 §3`, right-nested Σ with η — the `runState`-result construct,
  **distinct** from inductive `Prod`;
  [[composition-wp-real-producer-may-be-deferred-engine]] carry from
  State-effect); pin the pair as Σ, not `Prod`. (reduces-to value-flip on list
  order; oracle.)

### stdlib/map/letter-frequency-shape (the mandated demo)
- spec: `52 §8`/`§9` (AC2), `§4.1`/`§4.2`/`§4.3`
- given: the `letter-frequency` critical path over a `String`/`List Char` input
  `"banana"` (Char keys): fold the characters, `insert`/`lookup`-updating a
  count per key, then emit via ordered `toList`/`fold`.
- expect: **reduces-to** the ascending-by-key frequency list
  `[('a',3), ('b',1), ('n',2)]` — real `insert`/`lookup` build the counts, real
  ordered `toList` emits them deterministically. A miscount (wrong `insert`/
  `lookup` update) changes a value; a non-ordered emit changes the order — both
  flip the observed list.
- why: AC2's headline end-to-end demonstration — the exact `KNOWN-GAP` this WP
  closes, driven through the **real** interpreter over **computing** `Char` keys
  (`leq = int_leq`, `52 §5.4`). Not hand-fed: the counts come from real op
  sequences, the order from the real ordered traversal. (reduces-to value-flip;
  oracle.)

### stdlib/map/fold-agrees-with-tolist-ascending
- spec: `52 §4.3` (the `fold` order contract)
- given: `fold f z m` and the left fold of `f` over `toList m`, for a concrete
  `f`/`z` sensitive to order (e.g. `f k v acc = append acc [k]`).
- expect: **the two reduce to the same value** — `fold` visits entries in the
  **ascending key order** `toList` fixes (`§4.3` pins the order *contract*, not
  a particular recursion). A `fold` that visits in tree/insertion order
  disagrees with the `toList` fold on an order-sensitive `f` — the flip.
- why: AC2's `fold` order contract; the discriminator is an **order-sensitive**
  `f` (a commutative/associative `f` like `+` would be green-vs-green — order
  invisible). (reduces-to value-flip; oracle.)

### stdlib/map/fromList-last-writer-and-ordered
- spec: `52 §4.2` (`fromList` folds `insert`)
- given: `fromList [(2,'b'), (1,'a'), (2,'c')]` then `toList` of the result.
- expect: **reduces-to `[(1,'a'), (2,'c')]`** — `fromList` folds `insert` over
  the list, so it is `Ordered` (ascending) **and** last-writer-wins on the
  duplicate key `2` (`'c'` over `'b'`). A bug preserving list order, or keeping
  the first writer, flips the output.
- why: AC2's `fromList`/`toList` round-trip — ties construction-from-a-list to
  the ordered invariant + overwrite semantics in one value. (reduces-to
  value-flip; oracle.)

### stdlib/map/set-is-map-unit
- spec: `52 §4.4` (`Set a := Map a Unit`)
- given: `member a (insert a (insert b empty))` and `toList` of a `Set`
  (key-projection), over `Ord a` keys, with `a ≠ b`.
- expect: `member` **reduces-to `True`** for an inserted element and `False` for
  an absent one; `toList` yields the elements in **ascending** order. `Set`'s
  behavior is exactly the map's at `v := Unit` (the value carries no
  information). This is what retires the opaque `Set` primitive alongside `Map`.
- why: AC2 for `Set` as the `v := Unit` specialization — no separate mechanism,
  the map's laws project (`§4.4`). The `member` accept/reject is the
  non-degenerate pair. (reduces-to value-flip; oracle.)

---

## AC3 — proved, not stubbed (the load-bearing discriminator)

`52 §9` AC3 + `§5.4`: the `§5` invariant + correctness laws are **real proof
terms**; a discriminating test **fails against a stub/`Axiom` map-proof, for the
right reason** — the swapped-to-`Axiom` proof must be **exercised** by a real
kernel/elaborator obligation (the `trusted_base_delta` cone walk, a downstream
consumer), **not** merely detected as textually absent (Architect's sharpening).

**Why this is NOT a value-flip (the X1 discipline, stated honestly).** The map's
*operations* compute the same value whether the *laws* are proved or postulated:
`lookup k (insert k v m)` reduces to `Some v` by the `insert`/`lookup` **defs**,
independent of whether the found-after-insert **law** carries a real proof or an
`Axiom`. So every AC2 round-trip above is **green-vs-green** for proved-ness —
it cannot witness it. "Proved" is observable only as a **structural fact**: the
map's law terms add **zero** to `trusted_base()`
([[soundness-AC-static-vs-runtime-face]] — this is the static/structural face;
there is no runtime face, the property is compile-time by construction). The
cases below assert that structural fact via the **real** delta-accounting
producer, and flag exactly what would make it a value-flip (a consumer whose
*type-checking* forces a reduction `Axiom` short-circuits — deferred, §7c
territory).

### stdlib/map/laws-real-proofs-zero-new-delta (soundness)
- spec: `52 §5`/`§5.4`/`§9` (AC3), `../classes/seed-lawful-classes.md`
  (`law-fields-real-proofs-not-postulates`), `30 §6`
- given: the `§5` proof terms as admitted — `Ordered`-preservation, the three
  `lookup` laws (`§5.2`), the `toList` ordered law + agreement (`§5.3`) — each
  parametric in a `d : Ord k` dictionary (`§5.4`), witnessed at `k := Bool` (a
  carrier whose order-laws are **real** proofs, not `Axiom` — `Bool`/`Two`,
  `51 §6` — so the witness is real all the way down).
- expect: the map package's **incremental** `trusted_base_delta` over the
  `Ord Bool` dictionary is **∅** (zero-NEW-delta): the proofs discharge by
  case-split on the tree constructor + redex reduction, closing with
  `refl`/`tt`/`absurd`, citing the dictionary's `refl`/`antisym`/`trans`/`total`
  only as **hypotheses** — postulating **nothing** of their own. **The flip:**
  replace any `§5` law's proof term with `Axiom` (a `declare_postulate`) and the
  elaborator's `collect_consts_in_tb` cone walk **counts** it → the incremental
  delta is **non-empty** → the instance is **rejected as unlawful** (the exact
  `law-fields-real-proofs-not-postulates` mechanism, one layer up from `Ord` to
  `Map`). Real proof: empty delta, accepts. Stub: non-empty delta, rejects.
- why: AC3's headline, at the **map-proof level**
  ([[lawful-class-instances-must-carry-law-proofs]]). **Exercised, not
  textual** — the cone walk traverses the **actual** proof term (including
  `absurd` subterms, per `absurd-subterm-postulate-counted-in-delta`), so it
  catches a postulate the build hid anywhere, not a source grep.
  **Zero-NEW-delta, not zero-delta**
  ([[deceq-on-noncanonical-carrier-inhabits-bottom]] sibling; the `Ord Int`/
  `Ord Char` order-laws being `Axiom` is a **separate, honest**
  primitive-carrier audited delta — the map's OWN proof must add zero
  *further*). **`(soundness)`** — a stubbed map-law reads `proved`-by-default
  and the kernel does not catch it; the delta net is the sole backstop
  ([[untrusted-layer-backstop-hole-for-omissions]]). Verdict-independent
  structural (delta membership). (soundness; structural delta-flip; oracle.)

### stdlib/map/law-consumer-obligation-carries-no-postulate (soundness)
- spec: `52 §5.4`/`§9` (AC3, the downstream-consumer sharpening), `22`
- given: a downstream client program that **uses** the found-after-insert law
  `lookup k (insert k v m) ≡ Some v` as a load-bearing hypothesis (transports
  along it to type a term whose well-typedness depends on the lookup result),
  discharged through the real obligation machinery (`22`).
- expect: with the **real** proved map, the client's obligation discharges and
  the **client's** `trusted_base_delta` is **∅**. With an `Axiom`-stubbed
  map-proof, the client still elaborates (`Axiom` inhabits the type) **but** the
  postulate **propagates into the client's `trusted_base()`** via the cone walk
  — the client's delta becomes `{map_law_postulate}`. **The flip is at the
  consumer:** `trusted_base_delta(client) = ∅` (proved) vs `{postulate}`
  (stubbed).
- why: AC3's "**exercised by a downstream consumer**" face — the postulate is
  caught because a real consumer's obligation drags it into the consumer's trust
  cone, not because a grep found the word `Axiom`. **Fails for the right
  reason:** the stub fails precisely *because the map never proved its law*, and
  the failure surfaces as a trust-cone growth a proved map would not cause.
  **Flag (X1):** this is deliberately **not** a value-flip — the client
  type-checks either way; it becomes a value-flip only if the consumer's
  reduction were made to depend on the law computing (which `Axiom`
  short-circuits) — the §7c permutation/proof-carrying territory, deferred.
  **`(soundness)`.** (soundness; structural consumer-delta flip; oracle.)

### stdlib/map/ordered-invariant-derived-not-opaque (soundness)
- spec: `52 §5.1`/`§9`, `37 §6` (`isSorted`-style `Ω` recursion), `16 §1.3`
- given: the `Ordered` invariant and its helper `allKeys` as admitted.
- expect: `Ordered`/`allKeys` are **`declare_def` `Ω`-valued structural
  recursions** the prover **unfolds** (built from the
  `IsTrue b := Equal Bool b True` bridge + the derived `Ω`-conjunction `∧`),
  **not** postulates. An **opaque** invariant (a `declare_postulate`/primitive
  `Ordered`) makes the preservation obligation **undischargeable or circular**
  (`52 §5.1`, the `37 §6` surface-minimality discipline). Assert
  `Ordered ∉ trusted_base()` and that it **reduces** on a concrete tree.
- why: the invariant-transparency half of "proved, not stubbed" — a proof by
  case-split needs an invariant that **unfolds** (`52 §5.1`). **`(soundness)`**
  — an opaque invariant hides the very obligation the proof must discharge.
  Structural (admission-kind + reduces), not a value. (soundness; structural;
  oracle.)

---

## AC — canonical-carrier constraint (§2.1, the `antisym → Equal` localization)

`52 §2.1` + ADR 0010: the overwrite law's `Ord.antisym : … → Equal k x y` is a
theorem **only over a canonical carrier** (exactly one representation per
semantic key). The map **inherits** this obligation on its key type; it does not
create a new one. This is the `../../challenge/C1-deceq-noncanonical` story
applied to the map's one `Equal`-promotion site.

### stdlib/map/antisym-equal-sound-over-canonical-key
- spec: `52 §2.1`/`§5.3`, `ADR 0010`
- given: the overwrite/uniqueness proof (`§5.3`) over a **canonical** key
  carrier — `Int`/`Char`/`Bool` (one representation per value).
- expect: **accepts** — `leq k k' ∧ leq k' k ⇒ Equal k k'` is sound (the induced
  order-equality agrees with definitional equality on a canonical carrier), so
  the overwrite proof goes through and the map is a lawful partial function.
- why: the sound arm of `§2.1`'s canonical-carrier pair. (accepts; oracle.)

### stdlib/map/noncanonical-key-not-a-lawful-map-key (soundness)
- spec: `52 §2.1`, `ADR 0010`, `../../challenge/C1-deceq-noncanonical`,
  `../classes/seed-lawful-classes.md` (`Num`/`DecEq Decimal` re-defer)
- given: a **non-canonical** key carrier — `Decimal = MkDecimalPair coeff exp`,
  where `10×10⁻¹` and `1×10⁰` are **distinct pairs** denoting `1` — used to key
  a `Map`, with a **postulated** `antisym` over it.
- expect: **rejects / not a lawful `Map` key.** A postulated `antisym` there
  proves `Equal Decimal (10,-1) (1,0)` between distinct representatives;
  `MkDecimalPair`-injectivity refutes it → **inhabits `Bottom`** (the
  `DecEq Decimal` trap, ADR 0010). Such a type is **not** a lawful `Ord` key,
  therefore **not** a lawful `Map` key — the overwrite law is unavailable over
  it. The correct disposition is to **re-defer** (canonicalize the carrier or a
  setoid/quotient `Eq`), exactly as `Num`/`DecEq Decimal` are re-deferred in the
  lawful-classes lane.
- why: the unsound arm — the **non-degenerate pair** with the canonical case,
  keyed on the **structural** discriminator **canonicity per carrier** (not
  per-type: `Char` is canonical though its order-*laws* are `Axiom`; `Decimal`
  is non-canonical regardless — the two orthogonal ADR 0010 axes, `52 §5.4`).
  **`(soundness)`** — accepting a `Decimal`-keyed map with postulated `antisym`
  inhabits `Bottom` ([[deceq-on-noncanonical-carrier-inhabits-bottom]]). A
  single canonical-accept case is green-vs-green under a build that never checks
  canonicity. (soundness; verdict-flip pair on canonicity; oracle.)

### stdlib/map/lookup-laws-need-no-equal-promotion
- spec: `52 §2.1` (blast-radius localization), `§5.2`
- given: the three core `lookup` laws (`§5.2`, law 1 / found-after-insert /
  locality) as proof terms.
- expect: they discharge using **`refl`** (`IsTrue (leq k k)`) and
  **`total`**/`leq`-determinism **only** — **no `antisym → Equal` step**. The
  found-after-insert value is `v` whichever key label the node carries; locality
  uses order-distinctness. So the three `lookup` laws hold over the **induced
  order alone**, independent of carrier canonicity — only the **overwrite** law
  (`§5.3`) needs the canonical carrier.
- why: pins the `§2.1` **localization** — confining the canonical-carrier
  dependency to the single overwrite site keeps it auditable. Guards against a
  build (or a conformance over-claim) that spreads the `Equal`-promotion into
  the `lookup` laws, over-coupling them to canonicity. **Mechanism-consistency
  check** (my V2 carry): the `lookup` laws and the overwrite law must agree on
  *which* order-faculty each uses (`refl`/`total` vs `antisym → Equal`).
  (structural proof-shape; oracle.)

---

## AC4 — no regression

### stdlib/map/workspace-green-siblings-unchanged
- spec: `52 §9` AC4
- given: the workspace after the Map-container build.
- expect: `cargo test --workspace` **green**; the lawful `Ord`/`DecEq` instances
  and the rest of `packages/` (`List`/`Nat`) behave **identically** pre/post —
  the map is an additive package plus a primitive **removal** that nothing else
  depended on for *operations* (there were none). Any consumer that only *named*
  `Map` (never built/queried it) still elaborates.
- why: AC4's no-regression face; the retirement removes an operation-less
  primitive, so no working program loses behavior. (property; workspace-green.)

---

## Deferred (§7) — named follow-ons, never silent gaps (AC3)

`52 §7` defers three items, each a **named** follow-on. Conformance pins the
**boundary** (do not test the deferred behavior; do not let its absence read as
coverage), per the absurd-nothing-silently-dropped discipline.

### stdlib/map/no-delete-this-wp
- spec: `52 §4`/`§7b`
- given: the `§4` API surface.
- expect: **`delete` is absent** — operation *and* proof deferred **together**
  (not an unproved op under the "proved map" banner). A conformance suite for
  this WP **must not** assert a `delete` behavior; the absence is the **named**
  `§7b` follow-on, not a gap. `letter-frequency`'s critical path (insert +
  lookup + ordered iteration) does not need it.
- why: pins the API boundary honestly — guards against a test that expects
  `delete` (would false-fail a faithful build) **and** against the absence
  reading as silent incompleteness. (boundary; named deferral.)

### stdlib/map/balance-deferred-perf-not-correctness
- spec: `52 §3`/`§6`/`§7a`
- given: the unbalanced BST carrier.
- expect: operations are **O(log n) balanced / O(n) worst-case** on this
  **unbalanced** tree — stated honestly. Balance is a **perf property, separable
  from correctness**: the `§5` laws hold over the unbalanced tree, so **no
  conformance case asserts a complexity bound or balance metadata**. The
  balancing follow-on (`§7a`) is a **superseding representation** that re-proves
  the same law set — the signature-level API (`§4`) is stable across it. HAMT is
  explicitly out of scope (a parked later fast-map).
- why: pins the correctness/perf split — the correctness corpus is
  representation-independent, so it does **not** regress when balance lands.
  Guards against baking a complexity assertion into a black-box behavioral case.
  (boundary; named deferral.)

### stdlib/map/tolist-permutation-law-deferred
- spec: `52 §5.3`/`§7c`, `../../challenge/C2-proof-relevant-omega`,
  `../../challenge/C5-verified-sort`, `37 §6`
- given: the `toList` multiset/permutation law ("`toList` lists exactly the
  inserted entries, once each").
- expect: it is **deferred** — **proof-relevant** (distinct interleavings are
  distinct derivations), so it **cannot** be `data Perm : Ω` directly
  ([[proof-relevant-inductive-cannot-be-declared-at-omega]]); it needs
  `‖Perm_rel‖` or a count-equality form (`37 §6`, the C2 universe), **and**
  `Perm` discharge is the known **C5** prover gap. The **ordered** `toList` law
  (`§5.3`, `stdlib/map/tolist-ascending-by-key`) is its **naturally-`Ω`
  substitute that ships now** and delivers `letter-frequency`'s determinism
  **without** touching permutation. No conformance case asserts the permutation
  law for this WP.
- why: pins the `§7c` deferral to the **right universe** — a permutation law
  authored as a proof-relevant `Ω` inductive would be a consistency finding (the
  C2 boundary). The ordered law is the checkable substitute. Couples C2 (the
  universe) + C5 (the prover gap). (boundary; named deferral; universe-pinned.)

---

## Coverage map (AC → cases)

- **AC1 (net-negative TCB):** `carrier-inductive-ops-defs-not-primitive`,
  `kernel-crate-untouched`, `opaque-primitive-retired-trusted-base-shrinks`.
- **AC2 (operations correct end-to-end):** `insert-lookup-roundtrip-some`,
  `lookup-order-distinct-key-is-none`, `overwrite-last-writer-wins`,
  `tolist-ascending-by-key`, `letter-frequency-shape`,
  `fold-agrees-with-tolist-ascending`, `fromList-last-writer-and-ordered`,
  `set-is-map-unit`.
- **AC3 (proved, not stubbed):** `laws-real-proofs-zero-new-delta`,
  `law-consumer-obligation-carries-no-postulate`,
  `ordered-invariant-derived-not-opaque`.
- **§2.1 (canonical carrier):** `antisym-equal-sound-over-canonical-key`,
  `noncanonical-key-not-a-lawful-map-key`,
  `lookup-laws-need-no-equal-promotion`.
- **AC4 (no regression):** `workspace-green-siblings-unchanged`.
- **Deferred (§7), named:** `no-delete-this-wp`,
  `balance-deferred-perf-not-correctness`, `tolist-permutation-law-deferred`.
- **AC5 (build-lane retirement real):** the Foundation-owned `prelude.rs`
  primitive removal + `es2_acceptance.rs` "Map/Set are primitives" **flip** is
  pinned structurally by `opaque-primitive-retired-trusted-base-shrinks`
  (verified on the merged `trusted_base()`, not the spec) — the retirement is
  only real when the code drops it.
