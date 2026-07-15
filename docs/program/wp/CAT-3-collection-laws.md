# CAT-3 — Collection laws + the projection abstraction (Layer 1)

**Owner:** Spec enclave (elaboration) → Language build.
**Branch:** `wp/CAT-3-collection-laws` (off `origin/main @ 24a414b`).
**Status:** Steward frame (this doc). **Not yet elaborated.** Elaboration WP;
build held for the GPT window.

**Sequence:** **after CAT-1** (merged — the lawful-class pattern + the landed
append-monoid proofs this reuses) and **after SURF-1** (its `view`-keyword
retirement fixes the naming for the projection abstraction — see D3). **CAT-2-
independent** (collection laws are value-level; no `Monad` needed) — so it *may*
slot before or after CAT-2 at Steward's discretion; default consumption order is
after CAT-2. See `../06-catalog-campaign.md` §"Lane A" (CAT-3 = Layer 1).

> **Frame-by-objective, not by current state (§2c).** Every "the landed code has
> X" line is **perishable** — re-verify against `catalog/packages/Data/Collections/` +
> `catalog/packages/Core/Classes/` as they stand at pickup. SURF-1 migrates the `.ken`
> surface (`view` → `fn`/`proc`) under this frame; do not assume the `view`
> spelling still exists at elaboration.

---

## 1. Objective

Give **Layer 1 collections** their **laws as propositions, proved not
postulated** — the verified-substrate discipline (`spec/50-stdlib/README §intro`:
"a `Monoid` is not just `(append, empty)` — it is that plus *proofs*") — and land
the **agent-facing projection abstraction** (the catalog's "view" unit) that the
higher layers (parsers, maps, relational data) build on. Two halves:

- **Collection laws** over the landed list/collection ops
  (`catalog/packages/Data/Collections/Derived.ken.md`: `list_append`, `take`, `drop`, `nth`,
  `slice`, `concat`, …) plus the two structural operations a law layer needs that
  are **not yet landed** (`map`, `filter`) — length/membership/decomposition
  laws, the append monoid (**reuse CAT-1's landed proofs**), and **verified
  `sort`** (sorted + permutation), the layer's capstone.
- **The projection abstraction** — the agent-facing unit for *looking at data a
  different way*: projection / refinement / representation / indexed / quotient-
  respecting / obligation-producing views. This is the "view" the core-catalog
  report names at Layer 1; its **name must be reconciled** with SURF-1's
  retirement of the `view` *keyword* (D3).

**This chapter is the contract**; the Language build lands the `.ken` proofs +
any elaborator support the projection abstraction needs. The build is **held for
the GPT window**; this elaboration runs on the T1 enclave.

---

## 2. Fixed inputs — pinned, do not reopen

1. **Lawful-class discipline (CAT-1).** Laws are `Ω` propositions, **proved over
   inductive carriers, zero `Axiom`, zero `trusted_base()` delta**; the two-line
   induction+`cong` proof grammar (`55 §3.1`) and the **`tt`-vs-`Refl` endpoint
   discrimination** (`55 §3.2`: constructor-headed → `Top` → `tt`; neutral →
   stuck `Eq` → `Refl`) apply verbatim.
2. **Reuse, don't re-derive, the append monoid.** `list_append`'s associativity +
   unit laws are **already proved** in `catalog/packages/Core/Classes/LawfulFunctors.ken.md`
   (CAT-1: `list_assoc`/`list_left_unit`/`list_right_unit`, generic in the element
   type). CAT-3's append-monoid law is a **reference/instance**, not a new proof
   (subsume-don't-proliferate). Adding `Monoid (List a)` as a *parametric* instance
   depends on CAT-1-build's parametric-instance-head piece (`wp/CAT-1-build` D2).
3. **`Ω` law fields, no truncation for value equations** — length-preservation,
   decomposition, membership-as-`Ω`-predicate are value/`Ω` equations (`55 §4`).
   **The one exception is `Perm` — see §2.4.**
4. **Verified sort's `Perm` cannot be a raw multi-ctor `Ω` inductive (HARD
   soundness pin).** A permutation relation is **proof-relevant** (distinct
   re-orderings are distinct derivations); a proof-relevant multi-constructor
   inductive **cannot** be `data Perm … : Ω` directly — `Ω` is definitionally
   proof-irrelevant SProp, and unrestricted `Type → Ω` admits `Bool → true ≡
   false`, breaking consistency (`spec/10-kernel/16 §1.3`). `Perm` reaches `Ω`
   **only** via truncation `‖Perm_rel‖` (`16 §6`, the `∃ := ‖Σ‖` analog) **or** a
   **natively-`Ω` form** (a **count/multiset-equality** `∀ x. count x xs ≡ count x
   ys`, which is a `Π`-into-`Ω` of value equations). `isSorted` is fine as a
   structural recursion over `Ω`-connectives (`∧`, `≤`-as-`Ω`). This is the
   [[proof-relevant-inductive-cannot-be-declared-at-omega]] constraint — a prior
   CV-Spec BLOCKING finding shape; **the frame states it so the enclave designs
   `Perm` correctly from the start**, not discovers it at the gate.
5. **Kernel-untouched, outer-ring.** No new kernel `Term`/`Decl`; no
   `trusted_base()` delta. `map`/`filter`/`sort` are ordinary Ken over the
   built-ins (the catalog's derivation-path discipline, `README §intro`).

---

## 3. Mandated deliverables

### D1 — Structural collection laws

Add the two missing structural ops if absent (**verify at pickup** — `map` may be
reachable via the `Functor List` instance from CAT-1's extension; `filter` is
new), then their laws, each an `Ω` proposition proved by induction:

- **`map` length-preservation:** `length (map f xs) ≡ length xs`. (If `map` comes
  from `Functor List`, this is a law *about* the instance.)
- **`filter` membership characterization:** `mem x (filter p xs) ⇔ (mem x xs ∧
  IsTrue (p x))` — with `mem`/`⇔` as `Ω`-predicates.
- **`take`/`drop` decomposition:** `list_append (take n xs) (drop n xs) ≡ xs`;
  `length (take n xs) ≡ min n (length xs)`. (`take`/`drop` are landed.)
- **Append monoid:** the `Monoid (List a)` instance — **reuse** CAT-1's proved
  `list_assoc`/`list_*_unit` (`§2` pin 2), gated on the parametric-instance-head
  piece.
- Pin the exact pointwise statements (character-for-character, as `55 §5.2` did)
  and the carrier(s): `List` at minimum; `Option`/`Vec` if cheap.

### D2 — Verified `sort` (the capstone)

- **`sort : (a : Type) → (a → a → Bool) → List a → List a`** (or `Ord a`-
  constrained), ordinary total Ken (SCT-terminating — pick insertion or merge
  sort; the enclave decides, grounding termination).
- **Two correctness laws:**
  - **`isSorted (sort le xs)`** — `isSorted` a structural recursion into `Ω`
    (`∧` of pairwise `le`), `Ω`-valued, no truncation.
  - **`Perm xs (sort le xs)`** — permutation. **Design `Perm` per `§2.4`:**
    resolve **fork A (§5)** — `‖Perm_rel‖` truncation vs **count-equality**
    `∀ x. count x xs ≡ count x (sort le xs)`. **Count-equality is the
    recommended default** (natively `Ω`, no truncation machinery, directly
    provable by induction) — but the Architect owns the soundness call and the
    `count` requires a `DecEq a` (landed, `51`).
- The verdict-flipping conformance cases: a **non-permuting "sort"** (e.g. one
  that drops duplicates) must **fail `Perm`**; a **non-ordering "sort"** must fail
  `isSorted` — each at the named law field, specific error variant.

### D3 — The projection abstraction (+ the naming reconcile)

The core-catalog report's Layer-1 "view" unit: a first-class way to *look at data
differently* — the six flavors are **projection** (a lens-like focus),
**refinement** (a `{x : A | P x}` view), **representation** (an isomorphic re-
encoding), **indexed** (a key/position view), **quotient-respecting** (a view
through a setoid/quotient that respects `≈`), and **obligation-producing** (a view
whose use emits a proof obligation — the Ward/L12/L14 seam).

- **Resolve fork B (§5) — the mechanism.** Is this abstraction *ordinary Ken*
  (a `Σ`-record of `get`/`set`/coherence-law fields, exactly the lawful-class
  shape — **strongly preferred**, kernel-untouched) or does any flavor need
  elaborator support? Ground each flavor on landed machinery (records, refinement
  types `{x:A|P}`, quotients `16 §5`, the obligation/effect system) **before**
  committing; enumerate per-flavor, do not hand-wave "views are records."
- **Resolve fork C (§5) — the NAME.** SURF-1 **retires the `view` keyword**
  (→ `const`/`fn`/`proc`). This abstraction is a *noun concept*, not a definition
  keyword — but shipping a Layer-1 unit literally called "view" the same cycle the
  `view` keyword dies is a collision that will confuse every reader. Pick a non-
  colliding name (`lens`/`projection`/`optic`/`focus` for the projection flavor;
  a family name for the six). **Route the final name to Steward** (operator-facing
  ergonomics) if the enclave is unsure — this is exactly the `view`-alien-to-users
  concern SURF-1 acted on.
- Pin **at least the projection (lens) flavor** with its coherence laws
  (get-set / set-get / set-set) proved over a concrete carrier; the other flavors
  may be **designed** (law form stated) and built as fast-follows, stating which
  land now vs. later. The obligation-producing flavor **coordinates with Lane B /
  L12 / L14** — state the seam, do not fully specify it here.

### D4 — Conformance seed

Discriminating, verdict-flipping cases per the CAT-1/effect-composition
discipline: the sort `Perm`/`isSorted` flips (D2), the decomposition/membership
laws (D1), and the lens coherence flips (D3). Model on CV's CAT-1 seed
(`55 §3.2` endpoint discriminator + specific-variant assertions).

---

## 4. Acceptance criteria (testable)

- **AC1 — kernel-untouched.** `git diff origin/main -- crates/ken-kernel/` empty;
  zero `trusted_base()` delta; no new `Term`/`Decl`. Any elaborator need for D3
  re-forks to Steward.
- **AC2 — proved, zero Axiom.** Every law field a real kernel proof, grep-clean of
  `Axiom`/postulate/opaque; append monoid **reuses** CAT-1's proofs (no
  re-derivation).
- **AC3 — `Perm` is `Ω`-sound.** `Perm` is **not** a raw multi-ctor `Ω` inductive;
  it is count-equality or `‖·‖`-truncated per `§2.4` (fork A resolved + grounded
  against `16 §1.3`/§6).
- **AC4 — sort correctness flips.** A non-permuting sort fails `Perm`; a non-
  ordering sort fails `isSorted`; each at the named field, specific variant.
- **AC5 — laws `Ω`, pointwise, one field.** Value/predicate laws are `Ω`, stated
  pointwise, one canonical field (`55 §4`/§5.2).
- **AC6 — projection mechanism grounded.** D3's mechanism is enumerated per-flavor
  against landed machinery; the shipped flavor(s) have coherence laws proved; the
  name does not collide with the retired `view` keyword (fork C resolved).
- **AC7 — green.** `cargo test --workspace` + the rosetta corpus (16/0) green;
  new package(s) under `catalog/packages/` with MANIFEST + derivation path.

---

## 5. Open sub-decisions — routed to the Architect / enclave

- **Fork A — `Perm` representation:** count-equality (recommended, natively `Ω`,
  needs `DecEq a`) vs `‖Perm_rel‖` truncation. **Soundness call — Architect owns**
  (`§2.4`). Get it wrong and verified sort either can't be stated in `Ω` or
  admits a false permutation.
- **Fork B — projection mechanism:** ordinary `Σ`-record (preferred) vs any
  per-flavor elaborator need. Enumerate per flavor.
- **Fork C — projection NAME:** a non-colliding name for the abstraction (the
  `view` keyword is retired by SURF-1). Route to Steward if unsure.
- **Sort algorithm** (insertion vs merge) + its termination grounding — enclave's
  call.
- **Carrier breadth** — `List` mandatory; `Option`/`Vec`/`String` as cheap
  fast-follows or deferred.
- **`map` provenance** — the `Functor List` instance (CAT-1) vs a standalone
  `collections` `map`; verify at pickup and pin one.

Anything **beyond** scope — a kernel touch, a new `Term`/`Decl`, reopening a
decided OQ, or changing the lawful-class discipline — **re-forks to Steward**.

---

## 6. Do-not-reopen guardrails

- Do **not** re-prove the append monoid — reuse CAT-1's landed proofs (`§2` pin 2).
- Do **not** declare `Perm` as a raw multi-ctor `Ω` inductive (`§2.4` / AC3) — it
  is unsound and will be blocked.
- Do **not** postulate a sort law or introduce an `Axiom` to close it (`§2` pin 1).
- Do **not** ship the projection abstraction under the name `view` (fork C).
- Do **not** touch the kernel or add a `Term`/`Decl` (`§2` pin 5).
- Do **not** fully specify the obligation-producing flavor here — it is the Lane
  B / L12 / L14 seam; state the boundary and coordinate.

---

## 7. Dependencies & sequencing

```mermaid
graph LR
  CAT1[CAT-1 merged<br/>lawful-class pattern + landed append-monoid proofs] --> CAT3
  SURF1[SURF-1<br/>view-keyword retirement] --> D3
  CAT1B[CAT-1-build D2<br/>parametric instance head] -.gates.-> D1mono[D1 Monoid List a]
  subgraph CAT3[CAT-3 this WP]
    D1[D1 structural laws] --- D2[D2 verified sort]
    D2 --- D3[D3 projection abstraction]
  end
  CAT3 --> CAT4[CAT-4 maps/sets laws]
  CAT3 --> Lane B / L12 / L14 (obligation-producing views)
```

- **Upstream:** CAT-1 (merged), SURF-1 (naming for D3), CAT-1-build's parametric-
  instance-head (gates the `Monoid (List a)` instance form only).
- **CAT-2-independent** — may elaborate before or after CAT-2; Steward sequences
  at the seam. Default: after CAT-2 in consumption order.
- **Downstream:** CAT-4 (maps/sets laws) leans on the collection-law patterns;
  the obligation-producing view flavor feeds Lane B / L12 / L14.
- **Build:** held for the GPT window (T2); Architect re-certs AC1/AC3 (the `Perm`-
  `Ω`-soundness) on the built diff in the Phase-3 Opus re-review.
- **Kickoff:** enclave picks up §2c compact-gated at the seam after its prior WP.

---

## 8. Enclave elaboration (spec-author, `main@9fe9617`)

The contract is `spec/50-stdlib/57-collections-and-views.md`; this section is
the durable elaboration record for the Language build — the three fork rulings
(Architect), transcribed with verified code anchors. Every anchor was
re-grounded against `main@9fe9617` at pickup (the frame's lines are perishable).

**E1 — Fork A: `Perm` = count/multiset-equality (`Ω`-native).**
`Perm a eqf xs ys := (x : a) → Equal Nat (count a eqf x xs) (count a eqf x ys)`.
`Π` into `Ω` of `Nat`-value-equations ⇒ `Ω` by predicative `max` (`16 §1.1`); no
truncation. The soundness pin the frame flagged — a raw multi-ctor `data Perm …
: Ω` is inadmissible — is grounded on `16 §1.4`+`§1.1` (an unrestricted `Type→Ω`
admits `Bool` ⇒ `true ≡ false` under Ω-PI `16 §1.2`), **not** `§1.3` (the
frame's cite; `§1.3` is the derived-connectives/truncation home,
adjacent-not-the-pin — spec-author's correction, Architect-verified). The move
is the **landed `Ord.total` precedent**: `lawful_classes.ken:49–54` states
totality as `IsTrue (bool_or (leq x y) (leq y x))` — its own comment (`:43–47`)
records that a bare propositional `∨` "would be proof-relevant … and need `‖·‖`
to reach `Ω`," so the decidable `Bool` sidesteps it; count-equality is the
identical move for permutation. `bool_or` = transparent match-based (`:39`).

**E2 — D1 structural laws + the red/green split.** Landed ops
(`Derived.ken`, all `view`): `list_append:52`, `nth:58`, `take:67`,
`drop:76`. **Absent from all `catalog/packages/`:** `map`, `filter`, `mem`, `length`,
`min` (only `natSub:87`, saturating monus). ⇒ decomposition #1 (`list_append
(take n xs) (drop n xs) ≡ xs`) is **provable now**; decomposition #2 (`length
(take n xs) ≡ min n (length xs)`) needs `length` **and** `min`, so it joins
`map`/`filter`/`mem` laws **red-until-built**.

**E3 — append monoid reuse + the gate.** `list_assoc`/`list_left_unit`/
`list_right_unit` (`lawful_functors.ken:81–119`) are generic `(a : Type) → …`,
cited not re-proved. The parametric instance head `instance Monoid (List a)`
still hits `UnresolvedCon` (the `55 §6.1` gap, code-confirmed at
`lawful_functors.ken: 64–76`); the `Monoid (List a)` **instance form** stays
gated on `wp/CAT-1-build` D2, the generic proofs reusable today.

**E4 — verified sort.** Insertion sort (`insert` places `x` before the first
`le`-element; `sort` folds `insert`), both **structural** on the `Cons`-tail ⇒
SCT admits (same posture as landed `take`/`drop`). `isSorted` = structural
recursion into `Ω` via `∧` of pairwise `le` (`16 §1.3`). Two laws: `isSorted
(sort le xs)` and `Perm (eqFromOrd le) xs (sort le xs)`, where `eqFromOrd le x y
:= bool_and (le x y) (le y x)` derives the count comparator from the sort's own
`le` — **no separate `DecEq`** (antisym/refl of a lawful `Ord` make it decide
`Equal`; `DecEq.sound`/ `complete` at `lawful_classes.ken:25–28` tie `eq` to
kernel `Equal`).

**E5 — Fork B: view mechanism, per-flavor (concrete now / polymorphic later).**
Ordinary Ken `Σ`-records. Grounded anchors: refinement rides landed `{x:A|φ}` —
`ast.rs:369 TRefine`, `parser.rs:813 parse_refinement_type` (`21 §6.1`),
`elab.rs` `RRefine` lowers to the carrier + obligation (`21 §6.3`/`22`).
Quotient-respecting setoid-morphism form is a plain `Σ`-record (ships now); the
**quotient-carrier** form needs a surface path the parser lacks — kernel has
`Term::Quot`/`QuotClass`/ `QuotElim` (`conv.rs:187–291`, `16 §5`) but
`parser.rs` parses only refinement, not quotient-intro. Obligation-producing
rides `capabilities.rs attenuate`'s kernel-re-checked refinement obligation
(Ward/L12/L14 seam, boundary only). **The one shared wall:** polymorphic `Lens s
a`/`Iso a b` need a two-param dependent record — `class` is single-param
(`parser.rs parse_class_decl`), `data` ctor args are non-dependent atoms
(`parse_ctor_decl`, no named telescope), no surface `Σ`. A bounded
param-telescope-on-`class` extension (cousin of CAT-1 `§6`) unlocks the whole
family, kernel-untouched, no new `Term`/`Decl`. Shipped concrete lens: `Pair
Bool Bool` fst-lens with get-set/set-get/set-set proved.

**E6 — Fork C: the name.** Family **`view`** (operator's veto-window call),
flagship `lens`, six-flavor structure unchanged. SURF-1 retires the `view`
*keyword* (→ `const`/`fn`/`proc`), which **frees the word** — and `view` is the
industry-standard term for a read projection, so it is the family umbrella, not
a collision. Structure + law forms pinned in `57 §4`. Build-order: a
**capitalized** `View` type is collision-free with the still-lexed lowercase
`view` keyword; a lowercase `view` identifier would sequence CAT-3-build after
SURF-1's keyword-retirement build (Steward tracks it).

**E7 — scope register (no new immediate Steward re-fork).** Two design-now,
build-later elaborator walls, each re-forks to the Steward **when its general
form is built** (AC1), recorded as `90` deferred follow-ons: (a) **multi-param
`class`** (polymorphic-view vehicle), (b) **surface quotient-intro**
(quotient-carrier flavor). Plus one **naming token** routed to the Steward (Fork
C). All existing open items — no new fork opened.

**E8 — build sequencing (red-until-built).** The proved carrier is **`List
Bool`** (the only `Axiom`-free `DecEq`+`Ord` on `main`; `Int`/`Char` are
`Axiom`-holed — CV's caveat, Architect-verified). D1's
`map`/`filter`/`length`/`min` laws + decomposition #2 are red until their ops
land; D2 sort + D1 decomposition #1 + D3 concrete views build now. Architect
re-certs AC1 (kernel-untouched) + AC3 (`Perm`-`Ω`-soundness) on the built diff
in the Phase-3 Opus re-review.
