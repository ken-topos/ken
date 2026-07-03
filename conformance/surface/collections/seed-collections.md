# L3 (strings & collections) conformance — seed cases

Format: `../../README.md`. These pin the **L3 deliverable**
(`docs/program/wp/L3-strings-collections.md`,
`spec/30-surface/37-strings-collections.md`): the **`String` UTF-8 primitive**
(content-addressed, NFC-normalized, byte-length ≠ char-length, not `List Char`),
the **core collection types** (`List`/`Option`/`Result` transparent inductive;
`Array`/`Map`/`Set` abstract over the `41` heap), the **combinators with laws as
propositions**, **infinitude without coinduction** (the fuel-bounded inductive
unfold + the structural-absence net), and **structural equality +
`DecEq`/`Ord`** with the verified `sort` (which threads an **explicit
comparator** since the ES2-remainder §6 pin — the lawful-`Ord` class is
**deferred**). **L3b (AC7) adds the user-type instancing crossing** — a user
`instance DecEq` resolved by Lc's landed `instance_search` now that the §37 §6
gate is open (Lc, `4aa36c7`), with the user-`Ord`-drives-`sort` half deferred to
the lawful-`Ord` class. They extend
— and must not regress —
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
  L3; **user-type** `instance DecEq` was **L-classes-gated** at L3, `33 §5`/`39`
  — that gate is now **open** (Lc landed, `4aa36c7`) and **delivered in L3b**,
  AC7 `user-deceq-instance-keys-map-via-real-search`). The reject must be a
  **real** resolution failure, not
  a hand-fed "no instance" flag. (verdict-flip pair; hand-feed net; L-classes
  boundary pinned.)

---

## AC6 — the verified `sort` (the `Perm` conjunct is load-bearing)

`sort` threads an **explicit comparator** `leq : a → a → Bool` (the pinned §6
surface — the lawful-`Ord` class is **deferred**, `37 §6`) and produces the
refinement `{ ys : List a | isSorted leq ys ∧ Perm ys xs }` (`34 §5`); the
elaboration **emits the conjoined obligation**.

### surface/collections/sort-emits-issorted-and-perm (soundness)
- spec: `37 §6`, `34 §5` (refinement obligation), `22 §2.1`
- given: `view sort {a} (leq : a → a → Bool) (xs : List a) : R = …`, where the
  refinement `R = { ys : List a | isSorted leq ys ∧ Perm ys xs }` (`34 §5`) —
  the **explicit-comparator** surface (ES2-remainder pin; no `where Ord a`).
- expect: the result-introduction **emits the conjoined refinement obligation**
  `isSorted leq (sort leq xs) ∧ Perm (sort leq xs) xs` (`34 §5`, `22 §2.1`),
  dischargeable by a verified `sort` with a bundled proof. Assert the emitted
  obligation carries **both** conjuncts — **specifically that the
  `Perm (sort leq xs) xs` conjunct is present**, not `isSorted`-alone.
- why: AC6 — the canonical verification example, **structural on the emitted
  obligation**, and the **refinement-must-not-be-vacuous** discriminator.
  `isSorted`-alone is **degenerate**: `sort _ = Nil` satisfies
  `{ ys | isSorted leq ys }` (the empty list is vacuously sorted), so a
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

## AC7 — user-type `DecEq`/`Ord` instancing (L3b — the §6 gate crossing)

L3 pinned the `DecEq`/`Ord` boundary with **built-in** instances and tagged
user-type instancing `(oracle)` L-classes-gated (AC5/AC6). **Lc landed**
(`4aa36c7`) — the gate §37 §6 flagged is now **open for `DecEq`**: a user
`instance DecEq K` resolved by Lc's landed
`instance_search(class, head) -> Option<GlobalId>` (`classes.rs:91`; `Some` =
the canonical user instance, `None` = a no-instance error), which **extends**
AC5 (membership/identity) into user types. The **`Ord` half is deferred**:
ES2-remainder's §6 pin gives `sort` an **explicit comparator** and defers the
lawful-`Ord` class, so `sort` (and ordered `Map`/`Set` ops) resolve **no** user
`Ord` today — the two user-`Ord`-drives-`sort` cases below are gated on the
future lawful-`Ord`-class WP (they must **not** run green against today's
empty-stub `Ord`). None of these re-pin the base properties (one home per
property, `subsume-don't-proliferate`). No new kernel rule (§37 banner): pure
elaborator wiring of the collection ops to the landed resolver.

### surface/collections/user-deceq-instance-keys-map-via-real-search
- spec: `37 §3.3` (`DecEq`-keyed `Map`), `37 §6` (staging boundary now open),
  `33 §5`/`39 §6` (Lc instance search)
- given: a user `data K = …` with (a) a user `instance DecEq K`, and (b) the
  **same** `data K` with **no** `DecEq K` instance — each used to key a
  `Map K v` (construction + `lookup`)
- expect: **the verdict flips on the user instance.** (a) **accepts** —
  `instance_search("DecEq", "K")` returns `Some(id)`, the user dictionary keys
  the map and `lookup`/`insert` work; (b) **rejects** at compile time —
  `instance_search` returns `None`, a **no-instance error naming the missing
  `DecEq K`**, **not** a silent built-in fallback and **not** a runtime
  failure
- why: (L3b-AC1 ★) the user-instancing crossing — extends AC5's built-in
  `map-key-without-deceq-rejected` into **user** types. **Producer-grep the
  real resolver:** the `Map` key op must call `instance_search`
  (`classes.rs:91`) for the user type — **not** a built-in `DecEq`-only table
  (which would pass a primitive-keyed test while a user-keyed map silently
  falls back or fails: the built-in-fallback trap). The **reject arm is the
  guard** — a single accept is green-vs-green under a resolver that ignores
  the instance requirement.

### surface/collections/user-ord-instance-drives-verified-sort (deferred)
- spec: `51-lawful-classes.md` (the lawful `Ord` class + `where Ord a` supplying
  `d.leq`, `33 §5.4`), `37 §6`, `34 §5`, `33 §5`/`39 §6`
- **deferred — the coupling this case tests does not exist on `main` yet.** The
  ES2-remainder §6 pin makes `sort` take an **explicit comparator**
  `leq : a → a → Bool` (no `where Ord a`) and ordered `Map`/`Set` ops use
  built-in comparators, so **nothing** resolves a user `Ord` instance today;
  and the landed `Ord` class is an **empty stub** carrying **no** law fields.
  Asserting "the `Ord` dictionary carries the total-order law proofs the prover
  uses" against that stub is **green-vs-green** — it passes with zero
  law-carrying content. This case is **gated on the ES4-classes WP**
  (`51-lawful-classes.md` — spec **pinned**; the Team-Language build follow-on
  lands the `Ord`) that (a) builds `Ord` instances **actually carrying** the
  total-order law proofs (`refl`/`antisym`/`trans`/`total`, `51 §5` — not stubs;
  the `stdlib/classes/law-fields-real-proofs-not-postulates` net) and
  (b) supplies the instance's `leq` (`d.leq`) to `sort` via `where Ord a`
  (`51 §4`, `33 §5.4`). **Un-defers on that build**, not on this spec pin.
- given (on that WP): a user type `K` with (a) a user `instance Ord K`
  (law-carrying), and (b) the **same** `K` with **no** `Ord K` — each used in a
  `where Ord a`-constrained `sort (xs : List K)` (and an ordered op, e.g.
  `minKey`)
- expect (on that WP): **the verdict flips.** (a) **accepts** —
  `instance_search("Ord", "K")` returns `Some(id)`, the desugaring threads its
  `leq`, `sort` type-checks and its refinement obligation is discharged with the
  instance's law proofs; (b) **rejects** — `instance_search` returns `None`, a
  **no-instance error naming the missing `Ord K`**
- why: (L3b-AC2, deferred) user `Ord` drives the verified `sort` **once the
  lawful class + desugaring land**; the reject arm + the **law-carrying**
  dictionary (not the empty stub) are the guard. Until then the **live**
  user-instancing coverage is the `DecEq` path
  (`user-deceq-instance-keys-map-via-real-search`); the explicit-comparator
  `sort` emission is pinned comparator-side by `sort-emits-issorted-and-perm`.
  (deferred; do **not** count green against today's empty-stub `Ord`.)

### surface/collections/user-ord-sort-emits-both-conjuncts (soundness, deferred)
- spec: `51-lawful-classes.md` (`where Ord a` supplies `d.leq`, `§4`), `37 §6`,
  `34 §5` (refinement obligation), `22 §2.1`
- **deferred — no user-instance `sort` path exists on `main` yet.** Post-pin
  `sort` takes an explicit `leq` and has **no** `Ord`-resolved path (built-in or
  user), so there is no user-`Ord` `sort` site whose emission could regress.
  This case is **gated on the same ES4-classes WP** (`51-lawful-classes.md`;
  `where Ord a` supplying `d.leq` re-introduces the user-instance `sort` path) —
  **un-defers on that build**, not on this spec pin.
- given (on that WP): a `where Ord a`-constrained `sort (xs : List K)` where
  `Ord K` is a **user** `instance Ord K` resolved via
  `instance_search("Ord", "K")` and desugared to threading its `leq`
- expect (on that WP): the result-introduction **emits the conjoined
  obligation** `isSorted leq (sort xs) ∧ Perm (sort xs) xs` — **both conjuncts,
  `Perm` present** — identically to the explicit-comparator base; the emission
  does **not** regress through the desugaring
- why: (L3b-AC3 ★, deferred) (soundness) the VC-emission must **not regress
  through the `where Ord a` desugaring** onto the user-instance path. Extends
  the base completeness (`sort-emits-issorted-and-perm`, which is the **live**
  home for both-conjuncts/`Perm`-load-bearing/`const Nil`-degeneracy) — it pins
  that **desugaring `where Ord a` to the threaded `leq` preserves the conjoined
  emission**. **Discriminating (on that WP):** a build whose desugaring emits
  the VC for the explicit-comparator form but **drops `Perm` (or the whole
  obligation) on the desugared user-`Ord` path** passes the base yet **fails
  here** — the untrusted-layer **omission** (a never-generated conjunct reads
  `proved`-by-default; the kernel does not catch it). Producer (on that WP):
  grep the **emitted** obligation at the `sort` result site **on the desugared
  path** — not "it type-checks." (deferred; the live emission net is the
  explicit-comparator base.)

### surface/collections/user-deceq-keyed-map-canonical-identity
- spec: `37 §3.3` (byte-encoding canonical, **no `Ord` for identity**),
  `41 §3a`, `33 §5` (user `DecEq`)
- given: two `Map K v` keyed by a **user** type `K` (with `instance DecEq K`),
  built by inserting the **same** (key, value) set in **different insertion
  orders**; and (contrast) a pair differing in one entry
- expect: the same-content pair **interns to the same slot** (O(1) slot-id) —
  the canonical form is sorted by the **canonical byte encoding** of each key
  (`41 §3a`), so identity is insertion-order-independent **for a user key type
  too**, needing **no** user `Ord`; the differing-entry pair is **unequal**
- why: (L3b-AC4) the user-key extension of AC5's
  `structurally-equal-collections-o1-comparable`. **Identity is byte-order,
  not `Ord`:** `Ord K` (AC2) gates only *ordered* ops (`minKey`/range),
  **never** identity (`37 §3.3`, the pinned split) — a case requiring user
  `Ord` for `Map` identity would contradict §3.3. The user `DecEq K` is the
  **membership** constraint (AC1); the canonical byte encoding of the heap
  value (`41 §3a`) gives identity for free. Producer: the real key-sorted
  canonicalization over a **user** key (byte-order), **not** a list-compare
  and **not** an `Ord`-keyed sort. Assert **same slot-id**, not just `==`.

## Derived string surface (slice 2) — the `List Char` floor + 5 string ops

**Slice 2/2 of the string surface** (WP `L3-strings-surface`, `191b023`;
`spec/30-surface/37-strings-collections.md` §2.4/§2.5/§2.5.1/§4.1, DS-AC1–7 in
§9). Delivers the derived string operations — `concat`/`slice`/`charAt`/
`eq`/`compare` over `String` — as (a) a minimal **7-combinator `List Char`
floor** (`list_append`/`nth`/`take`/`drop`/`natSub`/`list_eq`/`list_compare`)
built as termination-checked recursive derived defs over the **real** generic
`Term::Elim` (`34 §3`), and (b) the 5 string ops derived on top, routed through
the now-real `string_to_list_char` (`s2l`) / `list_char_to_string` (`l2s`)
(slice 1, `f50be22`). **Zero new native prims, zero `trusted_base()` delta**
(Approach A, Architect ruling `evt_4k1yqah3yvpds`).

**Grounded (content-verified against the landed code at `191b023`, not the frame
prose — the frame's "landed" premises are perishable, and two were stale):**
`s2l`/`l2s` real (`crates/ken-interp/src/eval.rs` — the `Neutral` fallback arms
are reached only when `store.list_char_ids` is unwired; the real reduction fires
when it is, as slice 1); the `natSub` saturating-monus shape + the local
`data OrdResult = Lt | Eq | Gt` + `natCmp` 3-way **precedents** elaborate + SCT-
pass today (`crates/ken-elaborator/tests/val1_string_literals.rs:327`/`:334`,
with the ES2 sanction-comment "a genuine 3-way comparison still gets one,
declared locally"); `Ord Char` is **`leq`-only**
(`instance Ord Char { leq refl antisym trans total }`, no `compare` method,
`packages/lawful-classes/lawful_classes.ken:359`); `eqChar`/`leqChar` landed
(`= eq_int`/`= leq_int` under `Char`'s `Int` erasure,
`crates/ken-elaborator/src/decimal_char.rs:242`); the
`map`/`zip`/`unfoldUpTo`/`insert` recursion precedents landed
(`l3a_acceptance.rs`).

**Two stale frame-premise corrections (do NOT inherit the frame's prose — spec
reconciled them, `§4.1`/`§2.5.1`, Architect `evt_1stp9sspm6ag8`):**
- **The floor is 7 combinators, not 6.** The frame's `sub` (for `slice`'s
  length) is **not landed** — no `Nat` subtraction exists (only signed
  non-saturating `sub_int`), so `natSub` (saturating monus) is a 7th derived
  combinator.
- **`compare` is 3-way over a local `OrdResult`, not `Ordering`.** The frame's
  `compare = list_compare (Ord Char).compare : Ordering` names a **type and a
  method that do not exist** on `main` (`Ord Char` is `leq`-only; no `Ordering`/
  `OrdResult` type). It is delivered 3-way over a **locally-declared, string-
  surface-exported checked inductive** `data OrdResult = Lt | Eq | Gt`, with
  `compareChar` **repackaging** the landed `leqChar`/`eqChar` (`Eq` on `eqChar`;
  else `Lt`/`Gt` by `leqChar`). So `compare` cases assert `OrdResult` values
  (`Lt`/`Eq`/`Gt`), **never** an `Ordering` or a `Bool`.

**Deliverability honesty — this WP ships the value-level FUNCTIONS, not lawful
instances (the trust level, `§2.5`).** `eq : String → String → Bool` and
`compare : String → String → OrdResult` are Boolean/decision operations in the
**tested-not-trusted** interpreter ring ("a wrong value, never a false proof").
`String` **is** canonical w.r.t. `List Char` (the `s2l`/`l2s` round-trip is a
bijection on scalar sequences, ADR 0010 §2), so `DecEq String`/`Ord String`
**instances** are *soundly deliverable* later — unlike `Decimal`, whose non-
canonical carrier makes `DecEq Decimal` inhabit `Bottom`
(`../numbers/seed-numbers.md`, `../10-kernel/18a §5.6.1(4)`,
`deceq-on-noncanonical-carrier-inhabits-bottom`). But that transport
additionally needs a **lawful `DecEq Char`**, which is **not yet landed** (only
the `eqChar` *view* + `Ord Char`-by-transport are on `main`); so the proof-
carrying `DecEq String`/`Ord String` instances are a **tracked follow-on** (the
lawful-`Ord`/`DecEq`-class WP, AC7 above; `48`), **not** delivered here. Filing
these *functions* as proof-carrying instances would over-claim the trust level
(`trusted-by-typing-guarantee-is-not-kernel-proved-Q`).

**Reading disciplines specific to this slice:** producer-grep the **real**
7-combinator registration (net-new on `main` — grep-verified absent at
`f50be22`: `list_append`/`list_eq`/`list_compare` have **zero** hits;
`nth`/`take`/`drop` only lexical false-positives like `std::mem::take` — so a
green must drive the real producer, the `conformance-hand-feeds-the-deliverable`
net); DS-AC4 is a **non-degenerate accept+reject pair** (COORDINATION §7) plus
the **NFC-blindness** pin at the `list_eq` layer (unconditional there); assert
result **values** (`Lt`/`Eq`/`Gt`, not `Ordering`); the SCT check stays in its
**sound zone** (applied call on a strict subterm), not the over-accept zone
(`sct-unapplied-self-reference-over-accepts`).

### surface/collections/list-combinator-floor-derived-over-real-elim
- spec: `37 §4.1` (the 7-combinator floor), `34 §3` (real `Term::Elim`),
  `18a §5` (small audited core)
- given: the 7 floor combinators (`list_append`/`nth`/`take`/`drop`/`natSub`/
  `list_eq`/`list_compare`) + `compareChar`, as elaborated on
  `wp/L3-strings-surface`.
- expect: each **producer-greps** in `crates/ken-elaborator/src` as a
  `declare_recursive_group` / `declare_def` member whose `match` on the
  recursive argument lowers to the **real generic `Term::Elim { fam }`** over
  the `List`/`Nat` family (`34 §3`) — **not** a bespoke reducer, and **not** a
  registered `elim_List`/`elim_Nat` **constant** (there is none; `§4.1`).
  `OrdResult` greps as a **checked inductive** (`data OrdResult = Lt | Eq | Gt`,
  kernel-admitted by positivity) — **not** a
  `declare_primitive`/`declare_postulate`/`declare_opaque`.
  `git diff origin/main -- crates/ken-kernel/` is **empty**; `trusted_base()`
  unchanged.
- why: DS-AC1/AC5 — the floor is real derived surface at **zero-TCB-delta**.
  **Producer-grep, not hand-fed:** the combinators are **net-new**
  (grep-verified absent at `f50be22`), so a green must observe the **real**
  registration, not a synthetic def (the
  `conformance-hand-feeds-the-deliverable` net). A bug adding a native prim for
  a combinator (growing `trusted_base()`) or declaring `OrdResult` as a
  postulate is caught by the empty-kernel-diff + no-new-trusted-decl grep.
  (structural; producer-grep; zero-delta.)

### surface/collections/list-floor-recursion-in-sct-sound-zone (soundness)
- spec: `37 §4.1` (mandated defining equations), `17 §4` (SCT),
  `ken-kernel/src/sct.rs`
- given: the 7 floor combinators' mandated defining equations (`37 §4.1`).
- expect: for **each**, the recursive call is an **applied** call whose
  decreasing argument is a **strict subterm** of a matched argument — the `Cons`
  tail (`list_append`/`nth`/`take`/`drop`/`list_eq`/`list_compare`) or the `Suc`
  predecessor (`nth`/`take`/`drop`/`natSub`) — so the SCT **accepts** (real-now:
  every shape SCT-passes via the landed `map`/`zip`/`unfoldUpTo`/`insert`/
  `natSub` precedents, `l3a_acceptance.rs` + `val1_string_literals.rs`).
  **Discriminator:** a sibling recursing on a **non-subterm** — the whole
  matched argument reconstructed, `bad (Cons x xs) = bad (Cons x xs)`, an
  **applied** call carrying **no** `Down` argument — is **rejected** (SCT
  requires an applied call with a `Down` argument; `§4.1`, `sct.rs`).
- why: DS-AC2 (Architect brief-condition 1). **(soundness)** — a disguised
  non-terminator admitted by SCT inhabits `Bottom` via a δ-loop. The floor stays
  in the SCT **sound zone** and does **not** lean on the SCT **over-accept**
  zone (`sct-unapplied-self-reference-over-accepts`: a bare *unapplied*
  self-`Const` / recursion-through-an-opaque-`Map` — `c := c`, `loop := id loop`
  — is transparently edge-free and mis-admitted; none of the 7 need that shape).
  The accept/reject **verdict-flip** on the applied-subterm vs
  reconstructed-non-subterm pair is the guard; the SCT accept/reject
  **mechanism** is kernel-homed (`../../kernel/seed-kernel.md`), referenced not
  re-pinned. (soundness; sound-zone; applied-subterm verdict-flip.)

### surface/strings/derived-string-ops-reduce-over-real-roundtrip
- spec: `37 §2.5` (mandated bodies), `37 §2.3` (real `s2l`/`l2s`)
- given: `concat`/`slice`/`charAt` applied to a multi-byte corpus through the
  real `s2l`/`l2s` (reuse slice-1's boundary corpus: ASCII + `é`/CJK/emoji, plus
  empty).
- expect: reduce to the **correct value** — `concat "ab" "cd" ≡ "abcd"`, and
  `concat` over a multi-byte pair preserves every scalar;
  `slice 1 3 "abcde" ≡ "bc"`; `slice` **clamps** — `slice 0 99 "abc" ≡ "abc"`
  (over-range `take` stops at the end) and `slice 2 1 "abc" ≡ ""`
  (`natSub 1 2 ≡ 0` → `take 0 ≡ Nil`, no underflow); `charAt` is `Option Char` —
  `charAt 1 "abc" ≡ Some 'b'`, `charAt 5 "abc" ≡ None`, `charAt 0 "" ≡ None`.
  Indices are **code-point** positions (over the `List Char` view), never byte
  offsets. Assert the reduced **values**.
- why: DS-AC3 — the 5 ops compute correctly through the real round-trip.
  **Reduces-to on real producers:** the ops run through the landed `s2l`/`l2s`
  (wired via `store.list_char_ids` in test setup, as slice 1), not a hand-fed
  `List Char`. `charAt`'s `Option` (honest absence, `34 §1`, not a sentinel) and
  `slice`'s saturating clamp (`natSub`, never underflow) are the totality faces;
  a byte-offset `slice` (splitting a multi-byte scalar) or a partial `charAt`
  (stuck/sentinel out of range) is caught by the multi-byte + out-of-range
  witnesses. (reduces-to; totality; real round-trip.)

### surface/strings/string-eq-codepoint-wise-accept-reject-pair
- spec: `37 §2.5` (`eq` codepoint-wise), ADR 0010 §2, COORDINATION §7
- given: `eq : String → String → Bool` on (a) two **equal** scalar sequences,
  (b) a **same-length, single-codepoint-differing** pair, (c) a **length-
  differing** pair.
- expect: **the verdict flips** — (a) `eq "abc" "abc" ≡ True`; (b)
  `eq "abc" "abd" ≡ False` (same length, one codepoint differs — `list_eq`
  short-circuits on the first mismatch); (c) `eq "ab" "abc" ≡ False` (`Nil` vs
  `Cons` at position 2). Assert the **result value** (`True`/`False`), not "it
  type-checks".
- why: DS-AC4 accept+reject face — a **non-degenerate pair** (COORDINATION §7),
  not a single accept. A single `eq _ _ ≡ True` case is green-vs-green under an
  `eq` that ignores its second argument (or always returns `True`); the same-
  length single-differ reject (b) is the tightest guard — a length-only equality
  would pass (a)+(c) yet **fail** (b). `eq` rides the landed `eqChar`
  (`= eq_int` under `Char`'s `Int` erasure). (verdict-flip pair; result-value.)

### surface/strings/string-compare-3way-lexicographic-triple
- spec: `37 §2.5.1` (`compare` 3-way over `OrdResult`), ADR 0010
- given: `compare : String → String → OrdResult` on the ordered triple `"a"`,
  `"ab"`, `"b"`, and a reflexive input.
- expect: `compare "a" "ab" ≡ Lt` (`'a' ≡ 'a'`, then `Nil` vs `Cons` → shorter-
  prefix `Lt`); `compare "ab" "b" ≡ Lt` (`compareChar 'a' 'b' ≡ Lt`, since
  `leqChar 97 98` and `'a' ≠ 'b'`); `compare "b" "a" ≡ Gt`;
  `compare "ab" "ab" ≡ Eq`. Assert the **`OrdResult`** value `Lt`/`Eq`/`Gt` —
  **not** an `Ordering` (no such type on `main`) and **not** a `Bool`.
- why: DS-AC4 order face — the 3-way lexicographic order `"a" < "ab" < "b"` at
  the spec's **locked granularity** (`OrdResult`, not the frame's non-existent
  `Ordering`; `Ord Char` is `leq`-only, `lawful_classes.ken:359`). `compareChar`
  **repackages** the landed `leqChar`/`eqChar` (a faithful 3-way of the landed
  total order; a `declare_def`, so a bug is a wrong value, never a false proof).
  The prefix rule (`Nil` vs `Cons` → `Lt`) is what orders `"a" < "ab"`; a
  `compare` returning `Bool` or dropping the prefix rule is caught.
  (verdict-flip triple; `OrdResult` granularity.)

### surface/strings/list-eq-is-codepoint-wise-not-nfc-folding (property)
- spec: `37 §2.5` (codepoint-wise; NFC-normalization equality OUT), ADR 0010 §3
- given: `list_eq eqChar` on two **canonically-equivalent but
  codepoint-distinct** scalar sequences, constructed **directly** as `List Char`
  literals — `cs_nfc = [Char U+00E9]` (precomposed "é", one scalar) and
  `cs_nfd = [Char U+0065, Char U+0301]` ("e" + combining acute, two scalars).
- expect: `list_eq eqChar cs_nfc cs_nfd ≡ False` — the two scalar **sequences**
  differ (length 1 vs 2), so codepoint-wise equality is **False**, regardless of
  their NFC canonical equivalence. `list_eq`/`eq` **never** folds NFC-
  normalization into the comparison.
- why: DS-AC4 NFC-absence face (ADR 0010 §3) — pins that NFC-normalization
  equality was **not smuggled** into `eq`. **Constructed at the `List Char`
  layer on purpose:** the pin is **unconditional** there (two distinct scalar
  sequences are always codepoint-unequal), whereas at the `String`-literal layer
  it would depend on whether NFC-at-construction is real vs stubbed — pinning
  `eq "é" "e◌́" ≡ False` on **literals** would falsely fail once real NFC lands
  and merges them at construction (the over-pinning-a-deferred-behavior trap). A
  `list_eq` that NFC-normalized before comparing would return `True` here — the
  discriminating flip. **This is why an NFC equality is not deliverable as a
  lawful `DecEq`** (it identifies distinct sequences → non-canonical → would
  inhabit `Bottom`, ADR 0010 §3); the codepoint-wise `eq` is canonical over the
  `List Char` carrier and sound. **Reconciliation with
  `string-nfc-canonically-equal-shares-slot` (oracle):** that is a **different
  operation at a different layer** — the `String` content-addressed `==`
  (slot-id, O(1)) on two NFC-equivalent **literals** that NFC-at-construction
  **merges** to one slot (→ `==` `True`, when real NFC lands); **this** case is
  the derived codepoint-wise `list_eq`/`eq` on two genuinely-distinct **scalar
  sequences** (which `String` construction never yields as distinct values). No
  overlapping-input contradiction: `==` decides content-addressed **identity**
  (post-normalization); `eq`/`list_eq` decides **scalar-sequence** equality
  (NFC-blind); under real NFC they **agree** on `String` values. (property;
  NFC-blind; layer reconciliation.)

### surface/collections/list-append-does-not-shadow-bytes-append
- spec: `37 §4.1` (name hygiene, Architect brief-condition 2)
- given: `list_append : {a} → List a → List a → List a` and the landed
  `Bytes`-domain `append : Bytes → Bytes → Bytes` (FS-effect, `bytes.rs`).
- expect: `list_append` resolves to the **List** op (pure,
  `List a → List a → List a`) and **not** the `Bytes` `append` (`visits [FS]`,
  `bytes.rs:144`); both names resolve to their **intended** op, and a
  `list_append` application on `List a` does **not** pick up the `[FS]` effect
  row.
- why: DS-AC6 (Architect brief-condition 2). The `Bytes` `append` is **landed**
  (grep-verified: `bytes.rs:144`, `declare_primitive`
  `PrimReduction::Op{symbol:"append"}`, `[FS]` row) — a `list_append` that
  **shadowed** it would mis-resolve the `List` op to the `Bytes` primitive (type
  error / wrong reduction) or leak an `[FS]` effect onto pure list
  concatenation. Producer-grep the **distinct** registration. The other floor
  names (`nth`/`take`/`drop`/`natSub`/`list_eq`/`list_compare`) are free
  (grep-verified — only lexical false-positives). (name hygiene;
  distinct-registration.)

### surface/strings/concat-slice-compose-and-floor-totality
- spec: `37 §4.1` (totality), `37 §2.5` (`concat`/`slice` compose), `18a §3`
  (defining-law oracle)
- given: `concat`/`slice`/`list_append` on a small scalar-clean corpus.
- expect: `slice 0 (charLength a) (concat a b) ≡ a` for scalar-clean `a` (the
  length-`charLength a` prefix of `concat a b` is `a`); `list_append` is
  **associative** on a small corpus
  (`list_append (list_append xs ys) zs ≡ list_append xs (list_append ys zs)`);
  every combinator is **total** — `natSub` saturates at `0`, `nth`/`take`/`drop`
  totalize out-of-range to `None`/`Nil`, and **no** well-typed application
  reduces to `Neutral`/stuck. Assert the reduced values + non-`Neutral`.
- why: DS-AC7 — compositional sanity + totality. The `concat`/`slice` round-trip
  is a **defining-law oracle** (`18a §3`-style — non-circular, cannot alias the
  reduction it audits), exercising the ops end-to-end; `list_append`
  associativity is the canonical structural law; totality closes the "no stuck
  on well-typed input" face. A `slice`/`natSub` underflow (partial) or a
  `Neutral`-producing combinator is caught. (reduces-to; law; totality.)

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
- **AC6** (verified `sort`, explicit comparator, `Perm` present):
  `sort-emits-issorted-and-perm` (soundness).
- **AC7** (user-type `DecEq`/`Ord` instancing, L3b — the §6 gate crossing):
  `user-deceq-instance-keys-map-via-real-search`,
  `user-deceq-keyed-map-canonical-identity`; **deferred (lawful-`Ord`-class
  WP):** `user-ord-instance-drives-verified-sort`,
  `user-ord-sort-emits-both-conjuncts` (soundness).
- **DS-AC1–7** (derived string surface, slice 2 — the `List Char` floor + 5
  string ops):
  `list-combinator-floor-derived-over-real-elim` (DS-AC1/AC5),
  `list-floor-recursion-in-sct-sound-zone` (DS-AC2, soundness),
  `derived-string-ops-reduce-over-real-roundtrip` (DS-AC3),
  `string-eq-codepoint-wise-accept-reject-pair` +
  `string-compare-3way-lexicographic-triple` +
  `list-eq-is-codepoint-wise-not-nfc-folding` (DS-AC4),
  `list-append-does-not-shadow-bytes-append` (DS-AC6),
  `concat-slice-compose-and-floor-totality` (DS-AC7).

## Cross-case consistency sweep

- **Content-addressed equality is one story across every collection (`41 §4`).**
  `string-nfc-…-shares-slot`, `array-update-shares-unchanged-structure`, and
  `structurally-equal-collections-o1-comparable` must **agree**: equality is
  **always** a slot-id comparison (O(1)), and "sharing/identity" is **always**
  observed as slot-id — never a structural re-walk, never insertion-order- or
  construction-history-dependent. A case asserting an O(n) structural equality
  or an order-dependent `Map` identity would contradict this class. This is the
  content-addressed **identity** `==`; the derived slice-2 `eq`/`compare`
  (`string-eq-…`/`string-compare-…`) are a **different, coexisting** story
  (next bullet).
- **Derived string `eq`/`compare` are codepoint-wise functions, distinct from
  content-addressed `==` but agreeing in result.** The slice-2
  `eq`/`compare`/`list_eq` (`string-eq-codepoint-wise-accept-reject-pair`,
  `string-compare-3way-lexicographic-triple`,
  `list-eq-is-codepoint-wise-not-nfc-folding`) decide **scalar-sequence**
  equality/order over the `List Char` view — an explicit codepoint walk, **not**
  the O(1) slot-id `==` (so they do **not** violate the "identity is always
  slot-id" invariant above; they are derived decision *functions*, not the
  content-addressed identity). On well-formed `String` values they **agree** in
  result with `==` (both decide the NFC-normalized scalar sequence). The
  NFC-vs-NFD pin (`list-eq-…-not-nfc-folding`) lives at the `List Char` layer,
  where distinct scalar sequences are **unconditionally** unequal — it does
  **not** contradict `string-nfc-canonically-equal-shares-slot` (oracle), which
  is `==` on NFC-equivalent **literals** that construction merges to one slot.
  A case asserting the derived `eq` folds NFC-equivalence (or that `==`
  structure-walks) would contradict this split.
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
- **The user-instance path (AC7) is one story with the built-in path
  (AC5/AC6).** AC7's user `DecEq` cases resolve via the **same** landed
  `instance_search` (`classes.rs:91`) — so the built-in and user `DecEq` paths
  must **agree**: `Map` identity is **always** byte-order canonical (never
  `Ord`; `user-deceq-keyed-map-canonical-identity` vs the frame's "via resolved
  Ord"), and a missing `DecEq` instance is **always** a no-instance compile
  error, **never** a silent built-in fallback or runtime failure. A case letting
  the user `DecEq` path diverge — `Ord`-keyed `Map` identity or a runtime
  fallback — would contradict this class. The **`Ord` sort-VC leg** of this
  equivalence (`user-ord-sort-emits-both-conjuncts` ≡
  `sort-emits-issorted-and-perm`, both conjuncts on the user path) is
  **deferred** with the lawful-`Ord`-class WP — there is no user-`Ord` `sort`
  path on `main` to agree with the base, so `sort-emits-issorted-and-perm` (the
  explicit-comparator form) is the **sole live** emission-completeness home.

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
is **delivered in L3b** (AC7, post-Lc `4aa36c7`) — the collection ops wire to
the landed `instance_search` (`classes.rs:91`) for user types (net-new build).
The NFC half stays `(oracle)`; the rest is normative.
