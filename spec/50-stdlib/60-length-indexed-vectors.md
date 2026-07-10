# Length-indexed vectors — `Vec`

> Status: **normative** for the `Vec` family, its constructors, and the total
> `head` showcase — the family-declaration + single-scrutinee dependent-`match`
> mechanism they rest on is **landed** on the surface elaborator (the
> acceptance suite `crates/ken-elaborator/tests/explicit_data_elaboration.rs`,
> 14 tests, exercises the identical indexed-family + total-`head` mechanism).
> The `Fin` bounded-index family's **declaration** is likewise landed (§3);
> `tail` / `zip` / `lookup` are **specified here but GATED** on one named
> elaborator enhancement (§6): dependent-`match` motive recovery today refines
> the scrutinee's *own* index only. That enhancement is **in flight this run as
> `DS-5b` (Kernel ring)**, parallel to this chapter and reconciling at the
> Architect's gate — a scoped, in-flight prerequisite, not an open-ended
> deferral. The chapter **names that prerequisite and does not itself resolve
> it**. **Zero `trusted_base()` delta and zero `Axiom`:** `Vec`
> is an ordinary inductive family with a real eliminator, its operations
> genuinely kernel-proved; the gated operations need **no** postulate either —
> the gap is elaboration *completeness*, not soundness (§6, §7). DS-5.

Length-indexed vectors are the canonical dependent-types showcase: the length
lives *in the type* as an **index**, so an operation that is partial on lists —
`head` on the empty list — becomes **total** on `Vec`, because the type rules
the empty case out before the term is ever written. This chapter specifies the
family and the operations that make that showcase concrete, and is explicit
about which are buildable on the landed elaborator today and which wait on the
single enhancement named in §6.

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
| `tail` | `(A : Type) → (n : Nat) → Vec A (Suc n) → Vec A n` | **gated** (§6) |
| `zip` | `(A B : Type) → (n : Nat) → Vec A n → Vec B n → Vec (Pair A B) n` | **gated** (§6) |
| `lookup` | `(A : Type) → (n : Nat) → Vec A n → Fin n → A` | **gated** (§6) |

`Pair` in `zip`'s codomain is the landed non-dependent pair
(`Pair : Type → Type → Type`, `prelude.rs`); the length index `n` guarantees the
two input vectors **align**, so `zip` is total and loses no elements — unlike the
`List` `zip`, which must truncate. That guarantee is the second showcase; it is
specified here and gated on §6 only for its *elaboration*, not its design.

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
    `lookup … (fsuc …) ≡ lookup … `) — deferred because the operations
    themselves are gated (§6); the laws land with the operations.
  - `zip` / `map` **naturality** and the `zip`-`unzip` round-trip.
  - The **length / `toList` bridge** (`Vec A n → List A` and `length ∘ toList ≡ n`)
    relating the indexed family to the unindexed `List`.

## 6. The prerequisite — one dependent-`match` elaborator enhancement

`tail`, `zip`, and `lookup` are **total by design** and their types are stated
above. They do not build on the landed elaborator **today** — and the reason is
one precisely-located capability, not a scattering of missing features. Stated
per the honesty principle: this is **not** "these operations are unimplemented,"
it is "**the surface offers exactly one motive-recovery path, and that path's
reach does not extend here yet.**"

**The one path.** A dependent `match` recovers its eliminator motive
**automatically** from the expected result type, generalized over the scrutinee
and *its own indices* (`34 §3.2`). There is **no manual-motive escape hatch**:
`match` has no explicit-motive / `return` / `in` syntax — the surface `match`
expression carries only a scrutinee and arms (`ast.rs::Expr::EMatch` has no
motive field; `parser.rs::parse_match_expr` builds `EMatch { scrut, arms, span }`),
and `34 §3.2` recovers the motive with no override. So the automatic path is the
*only* path.

**Where the automatic path stops.** Motive recovery refines the **scrutinee's
own index**. It does **not** yet:

1. **Re-type peeled recursive fields by constructor injectivity.** `tail`'s
   result must be `Vec A n`, and its `vcons` arm yields the recursive field
   `xs : Vec A m`; soundly `m ≡ n` follows from `vcons`'s injective index
   equation (`Suc m ≡ Suc n`), but that per-branch equation is not carried into
   the local context to re-type `xs`, so `xs : Vec A m` never converts to the
   required `Vec A n` (the kernel reports `expected Vec A n, found Vec A m`).
2. **Re-type sibling context binders (the convoy).** `zip` matches its first
   vector `v : Vec A n`; in the `vcons` arm the second vector `w : Vec B n` must
   be refined to `Vec B (Suc m)` so it too can be split — but the scrutinee's
   per-branch index equation is not propagated to the sibling binder `w`, so
   `w` stays `Vec B n` and the inner match on it cannot be justified.

**The fix, precisely located.** Both are the same missing step: carry each
branch's constructor index equation into the local context to refine (1) peeled
recursive-field types (**injectivity**) and (2) sibling binders (**convoy**).
This is an **elaborator enhancement in `crates/ken-elaborator/src/elab.rs`** on
the dependent-`match` path (near the omitted-method / index-premise machinery of
§2). It is **not** a kernel change (the kernel already admits the family and its
dependent `Elim` — the family-declaration path of §1 is complete) and **not** a
`data.rs` change. It is soundness-adjacent match-elaboration, so it routes
through the Kernel/Ergo ring and the Architect's gate.

**This chapter states the dependency; it does not resolve it — but the
dependency is scoped and in flight.** The Steward has ruled the fork: the
enhancement lands **this run** as `DS-5b`
(`docs/program/wp/ds-5b-dependent-match-refinement.md`), a separate Kernel-ring
WP running **parallel** to this chapter and reconciling at the Architect's gate.
So `tail` / `zip` / `lookup` remain **gated today** — they do not build on the
pre-`DS-5b` elaborator, and this chapter must not be read as claiming they do —
but the gate is a scoped, in-flight prerequisite, **not** an open-ended
deferral: once `DS-5b` lands, each builds with its stated total type and its §5
laws, with no further spec change.

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
