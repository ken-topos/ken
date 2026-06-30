# L3 (strings & collections) conformance — seed cases

Format: `../../README.md`. These pin the **L3 deliverable**
(`docs/program/wp/L3-strings-collections.md`,
`spec/30-surface/37-strings-collections.md`): the **`String` UTF-8 primitive**
(content-addressed, NFC-normalized, byte-length ≠ char-length, not `List Char`),
the **core collection types** (`List`/`Option`/`Result` transparent inductive;
`Array`/`Map`/`Set` abstract over the `41` heap), the **combinators with laws as
propositions**, **infinitude without coinduction** (the fuel-bounded inductive
unfold + the structural-absence net), and **structural equality +
`DecEq`/`Ord`** with the verified `sort`. They extend — and must not regress —
the on-`main` surface invariants (`../seed-surface.md`,
`../data-match/seed-data-match.md`) and the `String` primitive already
registered for L6 (`../bytes-io/seed-bytes-io.md`).

**Grounded (content-verified against the landed targets, not heading numbers —
the `conformance-oracle-grounding-fallback` discipline):** `14 §5` (`String` is
a primitive type — opaque constant, registered reductions compute over literals,
trusted/audited `18 §5`); `41 §2`/`§4` (content-addressed append-mostly heap;
O(1) structural equality = slot-id comparison); `41 §3a` +
`docs/design/content-addressing.md §1.1` (kind tags **`String 0x04`,
`Array 0x06`, `Map 0x07`, `Set 0x08`** — verified against the enumerated table;
`String` = NFC-normalized UTF-8 **at construction**, `Map`/`Set` sorted by
canonical byte encoding ⇒ insertion-order-independent identity); `34 §1`/`§3`
(`List`/`Option`/`Result` inductive `data` + `elim_List`, **landed L2**);
`34 §5` (refinement types — the `sort` carrier); `33 §4` (a type exported
**abstractly** — name only, constructors hidden — the `Array`/`Map`/`Set`
opaque-carrier surface); `33 §5` (typeclasses as subobjects: **structure classes
`DecEq`/`Ord`**, the canonical-instance resolver convention, an unsatisfiable
constraint fails resolution = compile error); `42 §2` (`Lazy` is a thunk type
whose force/memo primitive **may be deferred for G1** — so the buildable-now
infinitude demo is the L2 unfold, **not** `Lazy`); `c3a3f1d`
(`L-resolver-globals`: an `EVar` scope-miss falls through to a global `RCon`
lookup — cross-declaration combinator references resolve). Cross-ref fidelity
verified at each target; no dangling forward-ref.

**Two staging facts that gate how a case is tagged (verified against the code,
not the frame):**
- **NFC normalization is currently STUBBED** (`content-addressing.md §1.4` K3
  note: "the F4 benchmark stubs NFC — strings are encoded as-is"). The spec pins
  NFC-aware O(1) equality as **normative** (`37 §2.1`), but the
  canonically-equivalent-strings-share-a-slot behavior **depends on real NFC
  landing**. So the NFC-equality case is **`(oracle)`-staged** (it asserts the
  spec's normative behavior, must **not** be run red against the stub — the
  `tag-deferred-seam-cases-at-elaboration-time` discipline). The byte-length ≠
  char-length fact is **NFC-independent** (a CJK/non-combining witness) and
  real.
- **`Array`/`Map`/`Set`, class/constraint resolution, the combinators, and
  `sort` are net-new elaborator surface** — none is registered in
  `ken-elaborator/src` today (only the `String` primitive, `List`/`Option`/
  `Result` L2 `data`, and the content-addressed heap are landed). Every positive
  case here therefore drives a **real new producer the build wires**, and the QA
  gate **producer-greps** that registration before counting green (the
  `conformance-hand-feeds-the-deliverable` net): **no synthetic collection
  literal, no hand-fed obligation, no hand-fed `DecEq` flag** where a real
  elaboration is asserted.

**Reading disciplines (what makes a case here load-bearing):**
- **Structural, not "compiles."** AC1 asserts **both** view-lengths
  (`byteLength` **and** `charLength`) and the elaborated type, not that it
  type-checks; AC2 asserts the **sharing** (unchanged sub-structure = same
  slot-id), not a correct result value; AC3/AC6 assert the **emitted
  obligation's shape**, not "it type-checks".
- **Verdict-flip / structural-flip on the target bug**
  (`discriminating-conformance-verdict-must-flip`): AC5 pairs a `DecEq`-key
  **accept** vs a non-`DecEq`-key **reject** (the classification boundary is a
  non-degenerate **pair**, never one case — COORDINATION §7); AC2's sharing
  flips slot-id under a deep-copy bug; AC6's `Perm` conjunct flips against a
  `const Nil` sort.
- **Absence is pinned by CONSTRUCT signature, not lexeme** (`B2`/`Sec1-N1`
  absence-net carry, sharpened here): AC4's no-coinduction net targets the
  **AST/judgment node** (a `codata` former / a greatest-fixpoint type / `cofix`
  / a guardedness pass), and **names the benign homonyms** (`Lazy` thunk type,
  lazy-WHNF conversion strategy, stdlib `Stream` library type) so a grep cannot
  false-alarm on the word `lazy`/`stream` — and is **paired** with a working
  inductive infinitude producer (presence), since an absence case alone is the
  highest-risk kind.
- **Refinement must not be vacuous** (the discriminating-example discipline,
  `34 §5`): AC6 asserts the emitted obligation carries the **`Perm` conjunct**,
  because `isSorted`-alone is satisfied by `const Nil` (the empty list is
  vacuously sorted) — a sortedness-only obligation guards nothing.
- **One home per property** (`subsume-don't-proliferate`): the `String ↔ Bytes`
  partial decode and the round-trip law are **L6's** (`../bytes-io/`),
  referenced not re-pinned; the `data`/`match`/`elim_List` and
  refinement-carrier machinery are **L2's** (`../data-match/`), referenced; L3
  pins only what is L3-specific (collection shape, persistence-sharing, the
  laws, the no-coinduction decision, `DecEq`/`Ord` split, `sort`).

**Tags.** `(soundness)` — a kernel **trusted-base** commitment whose wrongness
is a soundness bug: the no-coinduction structural absence (`37 §7`, a kernel
admission-gate commitment) and the `sort` obligation **completeness** (a dropped
`Perm` conjunct is a verification-soundness omission, the untrusted-layer
lesson). `(property)` — an invariant over many inputs / a law, not a single
trace. `(oracle)` — confirmed by the Spec enclave / at build time, safe as it is
**not** kernel-normative: the prelude/op **spellings** (`byteLength`/
`lengthBytes`, `get`/`index`, `map`/…), the `Array`/`Map`/`Set` **internal
representation** (RRB/HAMT/branching factor — `41 §5` X2 tuning), whether
`Set a` is literally `Map a Unit`, and the **NFC-equality case** (pending real
NFC, per the staging fact above). The **kind tags `0x04`/`0x06`/`0x07`/`0x08`,
the byte/char length distinction, the convertible-view totalities, the
persistence-sharing, the `DecEq`-membership verdict flip, the no-coinduction
absence, the `isSorted ∧ Perm` refinement, and every verdict** are
**normative**.

---

## AC1 — `String` is a content-addressed UTF-8 primitive (not `List Char`)

`String` is a `14 §5` primitive (opaque constant), content-addressed (`41 §2`,
kind `0x04`), NFC-normalized at construction (`41 §3a`); it is **not**
`List Char` (a separate, convertible view, `37 §2.3`).

### surface/collections/string-byte-length-differs-from-char-length
- spec: `37 §2.2`, `14 §5` (registered reductions over literals)
- given: an ASCII literal `"abc"` and a **single-code-point multi-byte** literal
  whose code point is **not combining and NFC-invariant** — a CJK scalar
  `U+4E16` (3 UTF-8 bytes, 1 code point), chosen so the byte/char gap is
  **independent of NFC normalization**.
- expect: `byteLength "abc" ≡ 3` and `charLength "abc" ≡ 3` **coincide**
  (ASCII); for the CJK literal `byteLength ≡ 3` while `charLength ≡ 1` — they
  **differ**. Assert **both** view-lengths as definitional reductions (`14 §5`),
  not "it compiles" and not a single "length".
- why: AC1's headline — treating `String` as packed UTF-8, not `List Char`,
  makes `byteLength` (stored bytes) and `charLength` (scalar count)
  **distinct**. A bug that conflates them (one `length` meaning bytes-or-chars
  ambiguously, or modeling `String` as `List Char` so `length ≡ charLength`
  only) is caught by asserting **both** differ on the multi-byte witness. The
  witness is NFC-invariant so this case is **real now**, independent of the NFC
  stub. (structural; both-views.)

### surface/collections/string-is-not-list-char-but-convertible
- spec: `37 §2.3`, `35 §2.4` (`Char` = scalar value, surrogate-excluded)
- given: a `String` `s` and the four conversions of `37 §2.3`.
- expect: `String` and `List Char` are **distinct types** (a `String` is **not**
  accepted where a `List Char` is required without an explicit conversion).
  `String → List Char` is **total** (decode the `charLength`-long view);
  `List Char → String` is **total** (encode UTF-8 then NFC-normalize + intern —
  and cannot encode an invalid scalar, since `Char` excludes the surrogate block
  `U+D800–U+DFFF`, `35 §2.4`). The `String ↔ Bytes` pair (`String → Bytes`
  total; **`Bytes → String` partial → `Result`**) is **L6's home**
  (`../bytes-io/seed-bytes-io.md`, `text-from-bytes-requires-named-decode`) —
  **referenced, not re-pinned**.
- why: AC1's "not `List Char`" face + the convertible-view **totalities** at the
  spec's locked granularity. The one partial direction (`Bytes → String`) is the
  L6 decode boundary; pinning it here would duplicate L6, so this case pins the
  **L3-new** `String ↔ List Char` totalities and references L6 for `Bytes`. A
  bug that makes `String = List Char` (no distinct type) or makes
  `List Char → String` partial (admitting invalid scalars) is caught.
  (type-distinction + totality.)

### surface/collections/string-nfc-canonically-equal-shares-slot (oracle)
- spec: `37 §2.1`, `41 §3a` / `content-addressing.md §1.4` (NFC at construction)
- given: two `String` literals that are **canonically equivalent under NFC** but
  spelled differently in source — the precomposed `U+00E9` ("é") and the
  decomposed `U+0065 U+0301` ("e" + combining acute).
- expect: both intern to the **same slot** (NFC-normalized before interning,
  `41 §3a`), so `s == t` is **O(1)-true** (a slot-id comparison, `41 §4`) and
  `byteLength s ≡ byteLength t` over the **stored normalized** bytes. Equality
  is decided once at intern time, never by re-traversal.
- why: AC1's content-addressed-NFC face — the normative `37 §2.1` behavior.
  **`(oracle)`-staged:** NFC normalization is currently **stubbed**
  (`content-addressing.md §1.4` K3 note: strings encoded as-is), so this asserts
  the spec's normative target and must **not** be run red against the stub — it
  is confirmed when real NFC lands (the `tag-deferred-seam` discipline). The
  byte/char case above is the NFC-independent, real-now sibling. (oracle; NFC
  normative target.)

---

## AC2 — collections are immutable + persistent (sharing observable as slot-id)

`List`/`Option`/`Result` are transparent inductive `data` (L2, landed);
`Array`/`Map`/`Set` are abstract over the `41 §2` append-mostly heap. An
"update" allocates the changed spine and **shares** the rest — observable as
slot-id equality (`41 §2`).

### surface/collections/list-pattern-matches-via-real-elim
- spec: `37 §3.1`, `34 §1`/`§3` (`List` inductive `data`, `elim_List`)
- given: `match xs { Nil => …; Cons h t => … }` for `xs : List a`.
- expect: the `match` lowers through the **real `elim_List`** (`34 §3`) — the
  same L2 eliminator, not a special collection protocol; `Nil`/`Cons` are real
  constructors a program may `match` on (`List` is **transparent**, `37 §3.1`).
  Assert the lowering to `elim_List` (structural), reducing by ι on a
  constructor scrutinee.
- why: AC2's transparent-inductive face. `List` is the **canonical** collection
  for proofs precisely because it rides L2 directly — no new kernel rule, no
  collection-specific elimination. This case drives the **landed** `elim_List`
  (real producer, testable now). A bug introducing a bespoke list eliminator
  (bypassing `34 §3`) is caught by asserting the `elim_List` lowering.
  (structural; landed mechanism.)

### surface/collections/array-update-shares-unchanged-structure
- spec: `37 §3.2`/`§3.4`, `41 §2` (append-mostly heap, structural sharing)
- given: an `Array a` `v` of several elements; `w = set i x v` updating one
  index `i`.
- expect: `w` is a **new** value (distinct root slot-id from `v`), but the
  sub-structures **not on the root→`i` path are the same slots** as in `v` —
  `set` allocates only the changed path and **shares** the rest (`41 §2`).
  Assert the **shared sub-structure = same slot-id** (structural sharing),
  **and** `v` is unchanged (same root slot, same contents) after. The persistent
  index tree reconciles O(1)-ish index with sharing (`37 §3.2`): a flat `O(1)`
  buffer **cannot** also share on update, so the honest claim is bounded-depth
  descent, not literal `O(1)`.
- why: AC2's headline as a **structural sharing flip**, per `41`-style — not
  "the result is correct". A deep-copy / non-sharing `set` (correct value, no
  sharing) gives the **same result** but **different** sub-structure slot-ids —
  caught **only** by the slot-id sharing assertion (a value-only check is
  green-vs-green here). **Net-new producer:** the QA gate producer-greps the
  real `Array` registration + `set`; the sharing must be observed on the
  **real** persistent `set`, not a hand-constructed two-slot pair. (structural
  sharing; hand-feed net.)

---

## AC3 — combinators are stdlib `view`s with laws as propositions

`map`/`filter`/`fold`/`zip` are prelude `view`s (`37 §4`), **not** a kernel
iteration protocol; their laws are `≡`-propositions discharged by the prover,
adding **no kernel rule**. A law in one declaration may reference a combinator
in **another** (`map_id` references `map`) — the cross-declaration lowercase
reference resolves via the landed `L-resolver-globals` fallback (`c3a3f1d`).

### surface/collections/functor-law-emits-obligation-cross-decl-resolves
- spec: `37 §4`, `c3a3f1d` (`L-resolver-globals`), `22` (obligation emission)
- given: `map_id : map id xs ≡ xs` stated in a declaration **separate** from the
  one defining `map`.
- expect: two faces. **(a) Resolution (real, landed):** the lowercase
  cross-declaration reference `map` inside `map_id` **resolves** — an `EVar`
  scope-miss falls through to the global `RCon` lookup (`c3a3f1d`), locals still
  shadowing; it does **not** error `UnboundName`. **(b) Obligation (net-new):**
  elaborating `map_id` **emits a real `≡`-obligation** `map id xs ≡ xs` to the
  `22` pipeline (a proposition, `14 §5`/`21 §3`), dischargeable by the prover —
  observe the **emitted obligation**, not "it type-checks".
- why: AC3 — combinator laws as propositions, **structural on the emitted
  obligation**, plus the cleared resolver blocker. Face (a) drives the
  **landed** resolver fallback (real, testable now); face (b) drives the
  **net-new** law emission (producer-grep the real `22` emission, not a
  synthetic obligation). A bug emitting **no** obligation (treating the law as a
  comment) or failing the cross-decl reference is caught. (structural obligation
  + resolver.)

### surface/collections/map-lookup-insert-law-emits-obligation
- spec: `37 §4`, `37 §3.3` (`Map` `DecEq`-keyed), `22`
- given: the canonical algebraic `Map` spec —
  `lookup_insert_eq : lookup k (insert k v m) ≡ Some v` and
  `lookup_insert_neq : k ≠ k' → lookup k (insert k' v m) ≡ lookup k m`.
- expect: each elaborates to a **real emitted `≡`-obligation** over the `Map`
  operations (dischargeable as a proposition); the second carries the `k ≠ k'`
  **premise** (a hypothesis discharged into the obligation, not dropped).
  Observe the emitted obligations structurally.
- why: AC3 on a **distinct law shape** (the associative-array algebra, not the
  functor law) so the two AC3 cases are not the same witness. The
  premise-carrying `neq` law guards against an elaborator that drops the
  hypothesis (emitting an unconditional, **false**
  `lookup k (insert k' v m) ≡ lookup k m`). **Net-new producer.** (structural
  obligation; distinct mechanism.)

---

## AC4 — no coinduction (structural absence) + inductive infinitude (the pair)

The §1 decision (state inductively, do not coinduct) is enforced by a
**structural-absence net** pinned by **construct**, paired with a working
**inductive** infinitude producer so the absence is not the only evidence.

### surface/collections/no-coinductive-construct-in-kernel (soundness)
- spec: `37 §7`, `14 §8` (strict positivity — the only inductive gate), `17 §4`
  (SCT — the only recursion gate)
- given: the kernel + surface admission machinery (`crates/ken-kernel`,
  `crates/ken-elaborator`).
- expect: **no coinductive type former** (no `codata` declaration form, no
  greatest-fixpoint type constructor, no `Stream`/`Colist` **kernel** type);
  **no `cofix` / copattern term former**; **no productivity or guardedness
  checker pass**. The kernel's **sole** structural admission gates are **strict
  positivity** (`14 §8`, for inductives) and the **SCT termination measure**
  (`17 §4`, for recursion) — there is **no dual guardedness analysis**. The net
  asserts the absence of the **construct** (the `codata`/`cofix`/guardedness AST
  node or kernel judgment), **naming the benign homonyms** so it targets the
  construct, not a word: **`Lazy`** (`42 §2`) is a **thunk type**, not
  coinduction; **lazy WHNF** (`42 §1`) is the conversion **strategy**, not a
  productivity rule; a stdlib **`Stream`** (`37 §5`) is a **library type** over
  inductive idioms.
- why: AC4's headline guardrail and the §1 durable decision. **`(soundness)`** —
  a coinductive former / guardedness gate slipping in is a kernel
  admission-soundness change. **Construct-not-lexeme (the B2/Sec1-N1 carry):** a
  lexeme grep for `lazy`/`stream`/`guard` false-alarms (these words are
  pervasive ordinary vocabulary — `guard` alone is dozens of
  positivity/underflow/arity guards) **or** is tuned permissive enough to miss a
  real `▷`-style former; the net must target the **formation rule / admission
  gate** (its AST/judgment), and **disconfirm**: the case is guard-gated (not
  coincidental) because it pins that the **only** admission gates are positivity
  + SCT, so a new guardedness pass would be a **new gate** the net detects, not
  an absent string. Grounded: the kernel today has **zero**
  `codata`/`cofix`/`corecursion`/guardedness construct. (soundness;
  construct-signature absence, named homonyms.)

### surface/collections/fuel-bounded-unfold-produces-finite-prefix
- spec: `37 §5` (item 1, the mandated demo), `34 §1` (`List`/`Option`/`Nat`
  inductive `data`), `17 §4` (SCT)
- given: `unfoldUpTo : (s → Option (a × s)) → Nat → s → List a`, the
  structurally-recursive unfold of `37 §5` (recurses on the `Nat` fuel), applied
  with a concrete step and fuel `n`.
- expect: `unfoldUpTo step n s` **reduces to a finite `List` prefix** of length
  ≤ `n` (terminating by **structural descent on the `Nat` fuel**, SCT-accepted,
  `17 §4`) — an **ordinary total `List`-producing function** over the landed
  `34 §1` `data`, with **no** coinductive value, **no** `Lazy`, **no** effect.
  Assert it produces a concrete prefix **and** that SCT **accepts** the
  recursion.
- why: AC4's **presence** half — infinitude served the **inductive** way, the
  non-degenerate **pair** with the absence net (an absence case alone is
  highest-risk). This is the **mandated** buildable-now demonstration and it
  rests **only on landed L2** (not the deferred `Lazy` force/memo, `42 §2` — the
  defer-spelling-not-concept / B2 carry: a buildable-now deliverable must not
  depend on a deferred spelling). A bug making this the *only* way (no `Lazy`
  ever) is fine; a bug requiring a coinductive value to stream is what §1
  forbids. (reduces-to + SCT-accepts; landed L2.)

---

## AC5 — structural equality + `DecEq` (the membership verdict flip)

Equality is structural + content-addressed (`41 §4`); `DecEq` (`33 §5`) is the
**membership** constraint for `Map`/`Set`, `Ord` the **order** constraint — the
pinned split. A key type without `DecEq` is a compile error.

### surface/collections/structurally-equal-collections-o1-comparable
- spec: `37 §6`/`§3.3`, `41 §4` (O(1) slot-id equality), `41 §3a`
  (insertion-order-independent canonical form)
- given: two `List` values built by different expressions but **structurally
  equal**; and (richer) two `Map` values built in **different insertion orders**
  with the same key→value content.
- expect: the two `List`s **share one slot** (content-addressed) and compare
  **O(1)-equal** (`41 §4`) — real now (landed heap). The two `Map`s **intern to
  the same slot** regardless of insertion order (canonical form sorted by the
  byte encoding of each key, `41 §3a`), so identity needs **no** user `Ord` —
  structural O(1) equality for free. Assert **same slot-id**, not just `==`.
- why: AC5's equality face — content-addressed identity as **slot-id**,
  including the insertion-order-independence that makes `Map`/`Set` identity
  canonical. The `List` half is real-now; the `Map` half is **net-new**
  (producer-grep the real `Map` registration / canonical form). A bug that makes
  equality structure-walk (not slot-id) or that lets insertion order leak into
  `Map` identity is caught. (structural slot-id.)

### surface/collections/map-key-without-deceq-rejected
- spec: `37 §3.3`, `33 §5` (`DecEq` membership constraint; unsatisfiable ⇒
  compile error)
- given: `Map k v` (and `Set a`) instantiated with (a) a key type that **has**
  `DecEq` (a core type, built-in instance — `Int`); (b) a key type that
  **lacks** `DecEq` (e.g. a function type `A → B`, for which decidable equality
  cannot exist).
- expect: **the verdict flips.** (a) **accepts** — `DecEq Int` resolves
  (built-in instance, `37 §6`); (b) **rejects** at compile time, the constraint
  `DecEq (A → B)` **unsatisfiable** (proof search for subobject membership
  fails, `33 §5`), the error **naming the missing `DecEq` instance**. `Ord` is
  **not** required for the core `Map`/`Set` (canonical byte order already orders
  stored keys) — it is the constraint for **ordered** ops (`minKey`/range), the
  pinned split.
- why: AC5's membership verdict flip — a **non-degenerate pair** keyed on a
  **structural** discriminator (constraint resolution succeeds vs fails), per
  COORDINATION §7, not a self-reported string. A single accept case is
  green-vs-green under a bug that drops the `DecEq` requirement entirely (it
  would accept **both**); the reject arm is the guard. **Net-new producer:**
  class/constraint resolution does not exist in `ken-elaborator` today —
  producer-grep the real constraint check (built-in `DecEq` instances ship in
  L3; **user-type** `instance DecEq` is **L-classes-gated**, `33 §5`/`39`,
  `(oracle)` for that half — the L1-numerics precedent: built-in now,
  user-instancing gated). The reject must be a **real** resolution failure, not
  a hand-fed "no instance" flag. (verdict-flip pair; hand-feed net; L-classes
  boundary pinned.)

---

## AC6 — the verified `sort` (the `Perm` conjunct is load-bearing)

`sort` requires `Ord a` and produces the refinement
`{ ys : List a | isSorted ys ∧ Perm ys xs }` (`34 §5`); the elaboration **emits
the conjoined obligation**.

### surface/collections/sort-emits-issorted-and-perm (soundness)
- spec: `37 §6`, `34 §5` (refinement obligation), `22 §2.1`
- given: `view sort {a} (xs : List a) : R where Ord a = …`, where the refinement
  `R = { ys : List a | isSorted ys ∧ Perm ys xs }` (`34 §5`).
- expect: the result-introduction **emits the conjoined refinement obligation**
  `isSorted (sort xs) ∧ Perm (sort xs) xs` (`34 §5`, `22 §2.1`), dischargeable
  by a verified `sort` with a bundled proof. Assert the emitted obligation
  carries **both** conjuncts — **specifically that the `Perm (sort xs) xs`
  conjunct is present**, not `isSorted`-alone.
- why: AC6 — the canonical verification example, **structural on the emitted
  obligation**, and the **refinement-must-not-be-vacuous** discriminator.
  `isSorted`-alone is **degenerate**: `sort _ = Nil` satisfies
  `{ ys | isSorted ys }` (the empty list is vacuously sorted), so a
  sortedness-only obligation is met by a **`const Nil`** implementation that
  discards the input — it guards nothing. The `Perm` conjunct forces `sort` to
  **be** a sort. **`(soundness)`** via the untrusted-layer **omission** lesson:
  the bug is the elaborator **emitting only `isSorted`** (silently dropping
  `Perm`) — a never-generated conjunct supplies no proof obligation and reads
  `proved`-by-default, a verification-soundness gap the kernel does **not**
  catch. The case asserts the **completeness** of the emitted obligation (both
  conjuncts present), not just that **an** obligation fires. **Net-new
  producer.** (soundness; obligation completeness; Perm-present.)

---

## Coverage map (AC → cases)

- **AC1** (`String` UTF-8 primitive, not `List Char`):
  `string-byte-length-differs-from-char-length`,
  `string-is-not-list-char-but-convertible`,
  `string-nfc-canonically-equal-shares-slot` (oracle).
- **AC2** (persistent collections, sharing):
  `list-pattern-matches-via-real-elim`,
  `array-update-shares-unchanged-structure`.
- **AC3** (lawful combinators):
  `functor-law-emits-obligation-cross-decl-resolves`,
  `map-lookup-insert-law-emits-obligation`.
- **AC4** (no coinduction + inductive infinitude):
  `no-coinductive-construct-in-kernel` (soundness),
  `fuel-bounded-unfold-produces-finite-prefix`.
- **AC5** (structural equality + `DecEq` flip):
  `structurally-equal-collections-o1-comparable`,
  `map-key-without-deceq-rejected`.
- **AC6** (verified `sort`, `Perm` present): `sort-emits-issorted-and-perm`
  (soundness).

## Cross-case consistency sweep

- **Content-addressed equality is one story across every collection (`41 §4`).**
  `string-nfc-…-shares-slot`, `array-update-shares-unchanged-structure`, and
  `structurally-equal-collections-o1-comparable` must **agree**: equality is
  **always** a slot-id comparison (O(1)), and "sharing/identity" is **always**
  observed as slot-id — never a structural re-walk, never insertion-order- or
  construction-history-dependent. A case asserting an O(n) structural equality
  or an order-dependent `Map` identity would contradict this class.
- **`DecEq`-membership vs `Ord`-order split is consistent across `Map` and
  `Set`.** `map-key-without-deceq-rejected` pins that the **core** `Map`/`Set`
  require **`DecEq`** (membership) and **not** `Ord` (`Ord` gates only ordered
  ops). The same split must hold for `Set` (it is `Map a Unit` semantically,
  `37 §3.3`); a case requiring `Ord` for plain membership/identity would
  contradict it.
- **Infinitude is inductive on both faces.** `no-coinductive-construct-…`
  (absence of a coinductive former) and `fuel-bounded-unfold-…` (presence of an
  inductive producer) are duals of the §1 decision: every way to "stream" is an
  inductive idiom (fuel-unfold / `Lazy`-thunk / generator / seam), **none** a
  coinductive value. A case introducing a `Stream` **kernel** type would
  contradict both.
- **Obligation cases observe emission, not type-checking.** `functor-law-…`,
  `map-lookup-insert-law-…`, and `sort-emits-…` are one class: each asserts a
  **real emitted obligation** to the `22` pipeline (and `sort` its
  **completeness** — both conjuncts). None may degrade to "it type-checks",
  which passes vacuously when no obligation is emitted (the untrusted-layer
  omission hole).

## Subsumed / not-duplicated (one home per property)

- **`String ↔ Bytes` (the partial `Bytes → String` decode) + the round-trip
  law** are **L6's** (`../bytes-io/seed-bytes-io.md`:
  `text-from-bytes-requires-named-decode`, `decode-encode-roundtrip-provable`,
  `reverse-roundtrip-is-not-a-law`). L3 references them for the `String ↔ Bytes`
  totalities; it does **not** re-pin the decode boundary or the round-trip.
- **`data`/`match`/`elim_List`, indexed families, per-branch refinement, and the
  refinement-types carrier** are **L2's** (`../data-match/seed-data-match.md`).
  L3 drives `elim_List` (AC2) and the `34 §5` refinement (AC6) but does **not**
  re-pin the L2 machinery.
- **`Char` (scalar, surrogate exclusion) and numeric literals** are **L1's**
  (`../numbers/seed-numbers.md`). L3 references `Char` (`35 §2.4`) for the
  `List Char` view, not re-pinned.
- **The content-addressed heap, O(1) equality, dedup, and capacity** are the
  **runtime's** (`../../runtime/seed-runtime.md`, `../../runtime/capacity/`). L3
  observes slot-id sharing/equality as the **surface** consequence; the heap
  mechanism is X1/X2's home.
- **`Lazy` force/memo, generators, the behavioral seam** are **deferred /
  other-WP** (`42 §2` G1 / L5 / `70-behavioral/`). L3 pins only the **fuel-
  bounded unfold** (item 1) as buildable-now; the other three idioms are named
  in `37 §5` but not the mandated demo.

## Build-sequencing note

L3 builds on **landed** substrate: the `String` **primitive** (`14 §5`,
registered for L6 in `ken-elaborator/src/bytes.rs`), `List`/`Option`/`Result`
**L2 `data`** + `elim_List` (`34`), the **content-addressed heap** with O(1)
slot-id equality (`41 §2`/`§4`), the **`L-resolver-globals`** cross-declaration
fallback (`c3a3f1d`), and the **strict-positivity + SCT** admission gates
(`14 §8`/`17 §4`). The cases that ride **only** landed machinery are real now:
`list-pattern-matches-via-real-elim`, the resolution face of `functor-law-…`,
`structurally-equal-collections-o1-comparable` (the `List` half),
`fuel-bounded-unfold-…`, and `no-coinductive-construct-…` (the kernel is clean
today).

The build-half **Team Language** delivers is **net-new**: the `String` byte/char
ops + the four conversions; `Array`/`Map`/`Set` (kinds `0x06`/`0x07`/`0x08`)
with persistent `set`/`insert`/`lookup`; the `map`/`filter`/`fold`/`zip`
combinators + their laws; the `DecEq`/`Ord` built-in instances + constraint
resolution; and the verified `sort`. So the QA gate (new-surface WP)
**producer-greps** the `String`/`Array`/`Map`/`Set` **registration** in
`ken-elaborator/src/` (and the `String` primitive in the kernel set, `18 §5`)
**before** counting green; the laws + `sort` must route through **real `22`
obligation emission**, the `Map` `DecEq` reject through **real** constraint
resolution, the `Array` sharing through the **real** persistent `set` — **no**
synthetic literal or hand-fed obligation where a real elaboration is asserted
(the `conformance-hand-feeds-the-deliverable` net). The **NFC-equality** case is
`(oracle)`-staged until real NFC normalization lands
(`content-addressing.md §1.4` K3 note); **user-type** `DecEq`/`Ord` instancing
is **L-classes-gated** (`33 §5`/`39`) — both flagged, the rest is normative.
