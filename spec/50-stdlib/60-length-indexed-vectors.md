# Length-indexed vectors — `Vec`

> Status: **normative** for the `Vec` family, its constructors, and the total
> `head` showcase — the family-declaration + single-scrutinee dependent-`match`
> mechanism they rest on is **landed** on the surface elaborator (the
> acceptance suite `crates/ken-elaborator/tests/explicit_data_elaboration.rs`,
> 14 tests, exercises the identical indexed-family + total-`head` mechanism).
> `tail` and the `Fin` bounded-index family's **declaration** are also **landed**
> (§3, §6): **`DS-5b`** delivered the dependent-`match` refinement (single-
> scrutinee constructor-injectivity + single-level sibling convoy + goal
> refinement, `34 §3.2.1`), so `tail` now builds. `zip` (the full two-vector
> recursive step) and `lookup` (`Fin`-indexed) remain **specified but GATED** on
> a named follow-on, **`DS-5c`** (§6) — a scoped prerequisite, not an open-ended
> deferral; the chapter **names it and does not itself resolve it**. **Zero
> `trusted_base()` delta and zero `Axiom`:** `Vec`
> is an ordinary inductive family with a real eliminator, its operations
> genuinely kernel-proved; the gated operations need **no** postulate either —
> the gap is elaboration *completeness*, not soundness (§6, §7). DS-5.

Length-indexed vectors are the canonical dependent-types showcase: the length
lives *in the type* as an **index**, so an operation that is partial on lists —
`head` on the empty list — becomes **total** on `Vec`, because the type rules
the empty case out before the term is ever written. This chapter specifies the
family and the operations that make that showcase concrete, and is explicit
about which are buildable on the landed elaborator today and which wait on the
`DS-5c` follow-on named in §6.

## 1. The family — `Vec (A : Type) : Nat → Type`

`Vec` is an **indexed inductive family**: `A` is a **parameter** (one fixed type
of elements, uniform across constructors) and the length is a **`Nat` index**
that the constructors **refine** (each constructor targets a *specific* length,
not a uniform one). It is `14-inductive.md §2`'s canonical indexed example, and
`Nat` is the landed Peano inductive `data Nat = Zero | Suc Nat` (`§2` here uses
`Zero` / `Suc` throughout).

```
data Vec (A : Type) : Nat → Type where
  vnil  :                       Vec A Zero
  vcons : (n : Nat) → A → Vec A n → Vec A (Suc n)
```

- **`A` is a parameter, `Nat` is the index.** The distinction is load-bearing
  (`14 §2.1`): a parameter is fixed in every constructor's result (`Vec A _`),
  while the index *varies* per constructor — `vnil` targets `Vec A Zero`,
  `vcons` targets `Vec A (Suc n)`. A constructor that changed the parameter, or
  whose result head was not `Vec A …`, or that carried the wrong index arity, is
  **rejected** at declaration (`data.rs::validate_ctor_result_target`; the
  acceptance suite pins each rejection — wrong family head, changed parameter,
  too-few / too-many indices).
- **Strict positivity holds** (`14 §2`/`§8.2`): `Vec` occurs only in strictly
  positive position in `vcons`'s recursive argument `Vec A n`, so the family is
  admitted with a real dependent eliminator `elim_Vec` (`14 §3`).
- **Level.** For `A : Type ℓ`, `Vec A n : Type ℓ` — the family sits at the
  element type's level; the `Nat` index contributes no level (it is a value
  index, not a type parameter). Predicative, non-cumulative (`12`).

**Landed.** The result-index-refining declaration form elaborates today: the
acceptance suite declares the identical mechanism under the illustrative names
`data Vector (A : Type) : Nat → Type where { EmptyVector : Vector A Zero ;
ConsVector : (n : Nat) → A → Vector A n → Vector A (Suc n) }` and checks that the
kernel admits it as an inductive family with the refined constructor targets.
The choice of surface name `Vec` / `vnil` / `vcons` here is the canonical one
(matching `14 §2`); the mechanism is name-agnostic.

## 2. The showcase — total `head` (landed)

The empty vector has length `Zero`; a vector of length `Suc n` is provably
non-empty. So `head` restricted to `Vec A (Suc n)` is **total** — no `Option`,
no partiality, no run-time emptiness check:

```
head : (A : Type) → (n : Nat) → Vec A (Suc n) → A
head A n v = match v { vcons m x xs ⇒ x }
```

The single-arm `match` writes **only** the `vcons` arm and **omits** `vnil` —
and this is exhaustive, not partial, by `34-data-match §4.3`:

- At the scrutinee's index `Suc n`, `vcons` is **type-possible** (its target
  `Vec A (Suc _)` unifies) ⇒ its arm is **required**.
- `vnil` is **index-impossible**: its target index `Zero` cannot unify with the
  scrutinee's `Suc n` (`Zero ≢ Suc n` by constructor disjointness / index
  discrimination, `15` / `16`). So the elaborator **synthesizes** `vnil`'s
  eliminator method by **absurdity** — the refuted index equation yields a proof
  of the empty index constraint, from which the method body is `Empty`-eliminated
  (`elab.rs`: the synthesized method is a `Term::Absurd` over the index premises
  computed by `method_index_premises`). The kernel still receives a **total**
  `elim_Vec` (`14 §3`), so omitting `vnil` is sound **by construction**.

The refinement is genuinely load-bearing, not decoration: `head` on an
**un-refined** scrutinee `v : Vec A n` (arbitrary length) correctly **rejects**
as non-exhaustive — there, `vnil` *is* type-possible, so its arm is required and
its omission is a real error. Totality is bought by the index, and only by the
index.

**Landed.** The acceptance suite elaborates exactly this showcase
(`fn vectorHead (A : Type) (n : Nat) (v : Vector A (Suc n)) : A = match v
{ ConsVector m x xs ⇒ x }`) and confirms both directions — the refined `head`
accepts, the un-refined one rejects non-exhaustive. This is the landed anchor
for the whole chapter: single-scrutinee dependent `match` with automatic motive
recovery (`34 §3.2`) + index-impossible auto-discharge (`34 §4.3`) is the
mechanism, and it is real.

## 3. Bounds — the `Fin` index family

A safe positional accessor needs an index that is **provably in range**. Ken
states this by construction with a bounded-index family `Fin` — `Fin n` is the
type of naturals strictly below `n`, so a `Fin n` **is** a witnessed in-bounds
index and no side-proof accompanies it:

```
data Fin : Nat → Type where
  fzero : (n : Nat) →         Fin (Suc n)
  fsuc  : (n : Nat) → Fin n → Fin (Suc n)
```

- `Fin Zero` is **uninhabited** (both constructors target `Fin (Suc _)`, never
  `Fin Zero`) — exactly the statement "there is no index into an empty vector,"
  captured at the type level.
- `Fin` is the *same indexed-family mechanism* as `Vec` (§1), so its
  **declaration elaborates today** on the landed surface (the result-index
  refinement is the landed capability). Prefer this one clean bounded-index
  story over proliferating an ad-hoc `Nat` index carrying a separate
  `IsTrue (lt i n)` proof (`reflect-don't-extend`, `subsume-don't-proliferate`):
  `Fin` states totality *by construction* — `lookup : Vec A n → Fin n → A` is
  total with **no** side-proof — which is the cleaner spec.

**Honest caveat (not hidden).** Choosing `Fin` over `Nat + proof` does **not**
dodge the §6 gap: `Fin` states the *totality* cleanly, but `lookup`'s
*definition* still recurses into the vector's tail, so it hits the same
dependent-`match` enhancement regardless (§6). The discipline choice is about the
cleanest totality *statement*, not about avoiding the enhancement.

## 4. The total API

Collecting the operations this chapter specifies, with each one's
buildability state (per the Architect's ground-truthed DS-5 ruling):

| Operation | Type | State |
|---|---|---|
| `vnil` / `vcons` | family constructors (§1) | **landed** |
| `head` | `(A : Type) → (n : Nat) → Vec A (Suc n) → A` | **landed** (§2) |
| `Fin` decl | `Nat → Type` (§3) | **landed** (decl) |
| `tail` | `(A : Type) → (n : Nat) → Vec A (Suc n) → Vec A n` | **landed** (§6; DS-5b) |
| `zip` | `(A B : Type) → (n : Nat) → Vec A n → Vec B n → Vec (Pair A B) n` | **gated** (§6; DS-5c) |
| `lookup` | `(A : Type) → (n : Nat) → Vec A n → Fin n → A` | **gated** (§6; DS-5c) |

`Pair` in `zip`'s codomain is the landed non-dependent pair
(`Pair : Type → Type → Type`, `prelude.rs`); the length index `n` guarantees the
two input vectors **align**, so `zip` is total and loses no elements — unlike the
`List` `zip`, which must truncate. That guarantee is the second showcase; it is
specified here and gated on §6 (`DS-5c`) only for its *elaboration*, not its
design.

## 5. Laws (scoped to the totality showcase)

Per the frame, this first chapter is scoped to the **totality showcase**; the
equational theory is named and deferred explicitly rather than under-specified
silently.

- **In-chapter (definitional).** `head` computes by the eliminator's ι-rule:
  `head A n (vcons n x xs) ≡ x` (`14 §3`). This is not a postulated law but the
  eliminator's computation rule, holding definitionally; it is the observable
  content of §2's totality.
- **Deferred (named, to a follow-on / the package WP).** The following are real
  and worth pinning but are **not** in this chapter's scope; each is deferred
  explicitly, not omitted silently:
  - `tail` / `lookup` computation (`tail A n (vcons n x xs) ≡ xs`;
    `lookup … (fsuc …) ≡ lookup … `) — deferred by this chapter's scope; `tail`'s
    is provable now (`tail` landed, §6), `lookup`'s lands with `lookup` (gated on
    `DS-5c`).
  - `zip` / `map` **naturality** and the `zip`-`unzip` round-trip.
  - The **length / `to_list` bridge** (`Vec A n → List A` and `length ∘ to_list ≡ n`)
    relating the indexed family to the unindexed `List`.

## 6. Dependent-`match` refinement — `tail` landed, `zip`/`lookup` on `DS-5c`

`tail` is **landed**; `zip` and `lookup` are **specified and gated on the named
follow-on `DS-5c`**. The single dependent-`match` capability these three needed
is no longer one indivisible gap: **`DS-5b`** (landed, `origin/main`) delivered
its first tranche, which is exactly enough for `tail`.

**What `DS-5b` landed (`34 §3.2.1`).** A dependent `match` recovers its motive
automatically over the scrutinee's own index (`34 §3.2`); `DS-5b` extended that
automatic path to carry each branch's **constructor index equation** into the
local context, in three forms (acceptance suite
`crates/ken-elaborator/tests/ds5b_dependent_match_refinement_acceptance.rs`, all
zero-`trusted_base()`-delta):

1. **Constructor injectivity for a peeled recursive field** — `tail`'s `vcons`
   arm yields `xs : Vec A m`, and `vcons`'s injective equation `Suc m ≡ Suc n`
   now re-types it to the required `Vec A n`. **`tail` builds.**
2. **Single-level sibling convoy** — matching one scrutinee refines an *outer*
   binder sharing the same index, so a nested match on that sibling is licensed.
3. **Base-case goal refinement** — a base-case arm's checking goal is re-typed
   by the branch equation.

The refinement is load-bearing, not permissive: the **AC8** over-refinement case
(a goal demanding an equation the branch does *not* license) **stays a genuine
kernel rejection**, never a silent accept.

**What remains — the `DS-5c` follow-on (`zip`, `lookup`).** Two operations need
more than `DS-5b`'s single-level convoy:

- **`zip`** — the full **two-vector recursive step**. The recursive `zip xs ys`
  reuses *both* vectors' peeled tails; `DS-5b`'s convoy handles **one** sibling
  through **one** nested match with **no further reuse** (`34 §3.2.1`), and the
  recursive reuse hits the root cause `34 §3.2.1` names: the sibling-convoy
  **cannot yet distinguish a genuine outer parameter from a field the enclosing
  `match` already bound**.
- **`lookup`** — the `Fin`-indexed recursion into the vector's tail, the same
  follow-on capability.

Both are captured as **`DS-5c`** (two-vector convoy + `Fin`-indexed `lookup`) —
still an **`elab.rs`** dependent-`match` enhancement (**not** a kernel or
`data.rs` change; the family-declaration path of §1 is complete), and still
zero-`Axiom`: the gap is elaboration *completeness* (a sound over-rejection),
never a soundness hole. There remains **no manual-motive escape hatch** — `match`
carries only a scrutinee and arms (`ast.rs::Expr::EMatch` has no motive field;
`parser.rs::parse_match_expr` builds `EMatch { scrut, arms, span }`), so the
automatic path that `DS-5c` extends is the only path. The chapter **names
`DS-5c` and does not itself resolve it.**

## 7. Trust, provenance, and clean-room

- **Zero `trusted_base()` delta, zero `Axiom`.** `Vec` and `Fin` are ordinary
  inductive families with real eliminators; `head`'s totality is
  **kernel-proved** by the total `elim_Vec` the index-impossible auto-discharge
  synthesizes (§2), not postulated. The gated operations (§6) likewise need
  **no** `Axiom`: once the enhancement lands, each is discharged by its
  dependent `match`. There is **no spot in this design that forces an `Axiom`** —
  the §6 gap is elaboration *completeness* (an over-rejection of a valid
  program, fail-closed and sound), never a soundness hole papered over by a
  postulate. This is the `List` / `Nat` posture (`reflect-don't-extend`).
- **Landed evidence.** The family-declaration + total-`head` mechanism is the
  acceptance suite `crates/ken-elaborator/tests/explicit_data_elaboration.rs`
  (14 tests); the spec rules it rests on are `14 §2`/`§3` (indexed families +
  dependent eliminator), `34 §3.2` (automatic motive recovery), and `34 §4.3`
  (index-impossible auto-discharge — the totality mechanism).
- **Clean-room.** Length-indexed vectors and the total-`head` construction are
  standard dependent-type-theory material; this chapter is authored from the
  landed Ken surface + `14`/`34` + first principles, in Ken's own words, with no
  reference source copied.
