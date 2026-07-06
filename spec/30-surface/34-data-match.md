# Sum types, pattern matching, and refinements

> Status: **impl-ready (L2)**. Normative and high-priority for the feature.
> Sum types, real constructors and eliminators, `match` with exhaustiveness +
> reachability, `Result`/`Option`/`Either`, indexed (GADT-like) families, and
> refinement types are first-class and fully checked from day one — each `data`
> declaration lowers to a genuine inductive type with real constructors and a
> real eliminator, never an opaque base.
>
> **No new kernel rule.** Everything here lowers to the **landed** kernel: a
> `data` decl elaborates to a kernel inductive family + its generated `elim_D`
> (`../10-kernel/14`, K1 and **K1.5**); `match` elaborates to `elim_D`
> (`39 §2.6`); a refinement `{x:A|φ}` elaborates to its **carrier `A` plus an
> emitted obligation** (`../20-verification/21 §2`, `22 §2.1`), never a kernel
> type former. The elaborator and the exhaustiveness/reachability checker are
> **untrusted** (`39 §1`): a bug yields a rejected valid program or a poor
> diagnostic, **never** an unsound acceptance — the kernel re-checks the emitted
> `elim_D` (`§4.4`).
>
> **Perishable — pin against the landed kernel, not this prose.** K1.5 has
> landed: the K1-era blanket rejection of Π-bound recursion
> (`check_no_pi_bound_recursive`) is **retired**, and `check_positivity` is the
> sole structural admission gate (`../10-kernel/14 §8.4`;
> `ken-kernel/src/inductive.rs`). W-style `(b:B) → D` recursive constructor
> arguments are **admitted**. Verify against the on-`main` `14`/kernel before
> building.

## 1. Sum types (real, not stubbed)

```
data Option a = None | Some a
data Result e a = Err e | Ok a
data Color = Red | Green | Blue
data Tree a = Leaf | Node (Tree a) a (Tree a)
data Expr = Lit Int | Add Expr Expr | Neg Expr
```

A `data` declaration elaborates to an **inductive family** (`../10-kernel/14
§1`): the kernel admits the **type former** `D`, its **constructors** `cₖ` (real
introduction forms), and the **generated dependent eliminator** `elim_D` (`14
§3`) — the *only* primitive way to consume a value of `D`. Constructors are real
intro forms and `elim_D` is a real eliminator **with computation**: `elim_D …
(Some x) ≡` the `Some` method applied to `x` (`14 §3` ι-reduction). Values can
be built **and** taken apart, and the eliminator reduces.

- **Constructor arguments** are positional or named-record style (`32 §1`); the
  surface `C A B` form and the record form `C { f : A, g : B }` both elaborate
  to a constructor telescope `(Δₖ) → D Δ_p t̄ₖ` (`14 §1`).
- **Recursive constructors** are admitted **subject to strict positivity** (`14
  §2`, `§8`): a recursive occurrence of `D` may appear only as the *target* of a
  (possibly dependent) function type, never to the left of an arrow. The
  elaborator emits the declaration; the **kernel** runs `check_positivity` and
  rejects negative or nested occurrences (`14 §8.3`/`§8.5`). The elaborator does
  **not** re-implement positivity — it is a kernel admission gate (`§4.4`, the
  trust boundary).
- **`Result`, `Option`, `Either`** are ordinary prelude `data` decls
  (`../50-stdlib/`): fallibility and absence are **honest sum types**, not
  sentinel values. There is no `null` and no error code — `None`/`Err` are
  constructors the exhaustiveness checker (`§4`) forces every consumer to
  handle.

**What the elaborator builds vs. what the kernel admits (the K1/K1.5 line).**
The elaborator lowers a `data` decl to a kernel `InductiveDecl` and relies on
the kernel's admission gates. Non-recursive and **direct-recursive**
(`A → List A → List A`) constructors are K1. **W-style** (Π-bound) recursive
arguments — `(b:B) → D`, the branching shape of `W` and L5's `ITree` — are
**K1.5** and now admitted (`14 §2.1`, landed). **Nested** (`List (Rose A)`) and
**mutual** families remain rejected by the on-`main` kernel (`14 §8.5`); a
`data` decl that needs them is a compile error citing the unadmitted shape, not
a silent lowering — declare the stage dependency, do not present it as
satisfied.

## 2. Indexed families (GADT-like)

Constructors may target different **indices** (`../10-kernel/14 §1`), giving
length-indexed and well-typed-by-construction data:

```
data Vec a : Nat → Type {                 -- explicit-index form
  VNil  : Vec a 0
  VCons : {n : Nat} → a → Vec a n → Vec a (n+1)
}
```

This elaborates to the kernel family `Vec (a : Type ℓ) : Nat → Type ℓ` with
constructors at distinct **index instances** — `VNil` at `0`, `VCons` at `n+1`
(`14 §1`, the `vnil`/`vcons` canonical form). The index varies per constructor;
the **parameters** (`a`) are fixed across the family. The same power refinement
types give (`§5`) is expressed *in the data declaration*: a `view head {n} (v :
Vec a (n+1)) : a` **cannot** be applied to an empty vector — the non-emptiness
is in the type, and the impossible application is a kernel type error (the index
`n+1` cannot unify with `VNil`'s `0`).

**Impossible constructors at an index** (the load-bearing reconcile, resolved in
`§4.3`). Matching a scrutinee of type `Vec a (n+1)` **need not** write the
`VNil` arm: `VNil : Vec a 0`, and `0 ≢ n+1`, so `VNil` is **type-impossible** at
this index. This is sound and does **not** weaken the kernel's requirement that
`elim_Vec` receive a method for *every* constructor (`14 §3`): the elaborator
**synthesizes** the `VNil` method by **absurdity** from the unsatisfiable index
equation (`§4.3`), so the kernel still receives a *total* `elim_Vec`. The
surface omission is a convenience the elaborator discharges, never a hole in the
eliminator.

## 3. Pattern matching → `elim_D`

```
view area (s : Shape) : Decimal = match s {
  Circle r       => 3.14159d * r * r
  Rect   w h     => w * h
  Tri    b h     => 0.5d * b * h
}
```

`match` scrutinizes one or more expressions and selects the **first** arm whose
pattern matches. Patterns are as in `32 §4`: constructors `C p̄`, variable
binders, the wildcard `_`, literals, tuples/pairs, record patterns, as-patterns
(`p as x`), or-patterns (`p | q`, same binders), and optional **guards**. It is
**not** a new kernel primitive — it **elaborates to `elim_D`** (`../10-kernel/14
§3`, `39 §2.6`).

### 3.1 Compilation (pattern matrix → nested eliminators)

The elaborator compiles a `match` by the standard pattern-matrix algorithm
(column-by-column): pick a scrutinee column, split on its head constructors into
one `elim_D` per matched type, and recurse on the residual matrix under each
constructor's freshly-bound fields. The result is a tree of nested `elim_D`
applications — one eliminator per scrutinized inductive, nested for nested
patterns. Specifically:

- **Constructor patterns** drive the `elim_D` split: arm `Cₖ p̄ => e` becomes the
  `cₖ` method, with `p̄` matched against `cₖ`'s fields in the residual matrix.
- **Variable / wildcard** patterns bind (or discard) the scrutinee in a method
  that does not split further; a column of all-variables needs no eliminator.
- **Literal** patterns (`35`) compile to a decidable-equality test on the
  primitive (a guard-like `if` chain, `39 §2.7`); a literal column is **not**
  closed (the primitive type is not an enumerable `data`), so a literal `match`
  is exhaustive **only** with a final variable/wildcard arm (`§4.2`).
- **Tuple / record** patterns project the (negative) `Σ`/record components (`13
  §3`, `33 §2`) and match componentwise — no `elim_D` (records are negative,
  matched by projection, `14 §4`).
- **As-patterns** `p as x` bind `x` to the whole scrutinee in `p`'s scope;
  **or-patterns** `p | q` duplicate the residual arm under both (requiring
  identical binder sets, `32 §4`).

### 3.2 Dependent-motive recovery

`elim_D` takes a **motive** `M : (Δ_i) → D Δ_p Δ_i → Type ℓ'` (`14 §3`). The
elaborator **recovers** `M` from the `match`'s expected result type by
abstracting it over the scrutinee and its indices:

- **Non-dependent `match`** (result type independent of the scrutinee) → the
  motive is **constant**, `M = λ ī x. T` for the expected `T`; this is the
  ordinary recursor.
- **Dependent `match`** (result type mentions the scrutinee or its indices —
  indexed families, and the body-as-motive obligations of
  `../20-verification/22 §4`) → `M` is the expected type **generalized** over
  the scrutinee `x : D Δ_p ī` and the indices `ī`. Recovering this dependency is
  what lets a branch refine the result type (essential for `§2` indexed families
  and for `22 §4`'s inductive postconditions). The elaborator solves the motive
  by higher-order pattern unification against the expected type (`39 §2.3`);
  genuine ambiguity is a surface error, never a guess (`39 §3`).

### 3.3 Per-branch definitional refinement (the hypothesis)

In the `cₖ` arm, after the `elim_D` split, the scrutinee is **definitionally**
the matched constructor: `s ≡ cₖ field̄` holds by the ι-rule the branch sits
under (`14 §3`). The verification layer turns this into a **hypothesis** — the
scrutinee equation `(_ : Eq A s (cₖ field̄))` added to the local context `Γ`
(`../20-verification/22 §3`) — so inside the `Circle r` arm one may *assume*
`s ≡ Circle r`, and a dependent motive (`§3.2`) refines the **result type** of
that arm accordingly. This per-branch refinement (a fact about the value **and**
a refinement of the type) is what AC6 pins — the surface origin of `22 §3`'s
path-sensitive `Γ`.

**Guards do not refine and do not cover.** A guarded arm `Cₖ p̄ if g => e`
elaborates to a conditional *inside* the `cₖ` method (`39 §2.7`); because the
guard `g` may fail, the arm does **not** by itself discharge the `cₖ`
constructor for exhaustiveness (`§4.2`). Guards are an arm-selection refinement,
not a coverage contribution.

### 3.4 Transport by a propositional equality — the `J` former

Per-branch refinement (`§3.3`) rewrites the goal by an equality that holds
**definitionally** — the scrutinee equation `s ≡ cₖ field̄` that the `elim_D`
split makes true by the ι-rule. It does **nothing** for an equality that holds
only **propositionally**: a proved `p : Eq A a b` that is *not* a definitional
convertibility (e.g. an order hypothesis `IsTrue (leq k k') = Eq Bool (leq k k')
True` over an **abstract** key `k`, where `leq k k'` is a *stuck* redex no match
can fire). To rewrite a goal mentioning `a` into one mentioning `b` along such a
`p`, the surface provides one term-former, the identity eliminator **`J`** — the
explicit, proof-carrying transport every dependent theory supplies (Agda
`subst`, Lean `▸`, Coq `eq_rect`).

`J` is **not** a new kernel construct. It elaborates directly to the kernel's
existing `Term::J` (`../10-kernel/15 §4`), which the kernel derives to `cast`
and **reduces on any equality** (`../10-kernel/16 §3`); both `J` and `cast` are
already in `trusted_base()`, so this former adds **nothing** to the trust
surface — it only makes an already-trusted eliminator reachable from `.ken`.

**Surface syntax and typing rule (the pin).** `J` is written applied to three
arguments, `J motive base eq`, and is elaborated in **infer mode** (like the
kernel eliminators, unlike the checked-mode `Refl`/`absurd`/`tt` sugar whose
motive comes from the goal): the equality's type `A` and endpoints `a`, `b` are
recovered from `eq`'s inferred type, and the result type is synthesized. The
rule is verbatim the kernel's `J`-formation (`../10-kernel/15 §4`):

```
  Γ ⊢ eq : Eq A a b
  Γ ⊢ motive : (b' : A) → Eq A a b' → s     (first domain ≡ A;  s any sort)
  Γ ⊢ base : motive a (refl a)
  ─────────────────────────────────────────────────────────────  (J)
  Γ ⊢ J motive base eq  :  motive b eq
```

The motive's codomain **sort `s` is unconstrained** — it may be `Type ℓ`
**or `Ω`**. This is load-bearing, not incidental: an `Ω`-valued motive is what
lets `cong` conclude a proof-irrelevant type-equality and what lets a Branch-B
proof obligation living in `Ω` (`../50-stdlib/52 §5`) be discharged by transport
at all. `J` derives its computation from `cast`, whose rule deliberately does
**not** require the endpoints to be convertible (that non-requirement *is*
transport):

```
  Γ ⊢ e : Eq Type A B      Γ ⊢ t : A
  ─────────────────────────────────────  (cast)   -- ../10-kernel/15 §3, 16 §3.1
  Γ ⊢ cast A B e t  :  B
```

Because the motive's binder types are fixed by `eq`, the user writes the motive
as an unannotated lambda `\b' _. G[b']` — its domains need no ascription.

**The motive is user-written (the Agda-`subst` posture).** The user abstracts
the rewritten occurrence explicitly, naming `G[·]` in the motive. Inferring
*which* occurrences of `a` to generalize — `rewrite` / with-abstraction /
auto-motive spelling — is a **separate, non-soundness ergonomic sugar, out of
scope here**; it is deferred to a later surface-syntax WP.

**Why this is sound (and what it is not).** `J` **asserts nothing**: the motive,
the base, and the user-supplied witness `eq` are all kernel obligations on the
emitted `Term::J`, kernel-re-checked in full — an ill-typed transport (wrong
motive, wrong endpoints, or a proof of the *wrong* equation) is **rejected by
the kernel**, never silently accepted. Every rewrite is witnessed by a proof the
user supplied; the elaborator never manufactures an equality. This is
deliberately **not** an implicit congruence: the unsound cross-wise
`Eq`-congruence that would identify `bool_eq x y` with `bool_eq y x` (smuggling
propositional symmetry into definitional equality) stays a hard **NO**
(`../10-kernel/16`; `../50-stdlib/51` K6), and if a realistic transport goal
fails to *compute*, the remedy is a sound kernel completeness fix, never an
elaborator that routes around the kernel. There is **no conversion change**
here — transport discharges through the `J`/`cast`
*typing* rule (the equation obligation lands on the user's `eq`), so this former
implies **no `../10-kernel` conversion-completeness note**.

The five everyday combinators built on `J` — `subst`, `cong`, `cast`, `sym`,
`trans` — are ordinary non-recursive library `view`s, **not** formers; they are
listed in `../50-stdlib/53-transport.md`.

### 3.5 Proof-returning dependent motives

A `match` may be checked against an expected **proof target** whose type is an
`Ω` proposition and whose statement depends on the scrutinee. This is not a new
surface construct and not a CAT-4 special case: it is the ordinary dependent
eliminator of `§3.2` with an `Ω`-codomain motive (`../10-kernel/14 §3`).

The canonical shape is:

```
match s {
  C0        => p0
  C1 x ...  => p1
}
```

checked at a target `P[s] : Ω`, where `P` may be a direct equality
(`Equal A lhs[s] rhs[s]` / `Eq A lhs[s] rhs[s]`) or any proof expression that
mentions `s` after the same transparent unfolding and WHNF exposure used by
ordinary elaboration. The elaborator recovers the motive
`M = λx. P[x]` (and, for indexed families, `λī x. P[ī, x]`) and emits
`elim_D M ... s`. Each branch is then checked against the **specialized**
target:

- the `C0` method must inhabit `P[C0]`;
- the `C1 x ...` method must inhabit `P[C1 x ...]`;
- a nested proof-returning `match` repeats the same rule at its own scrutinee.

`Equal` has no separate rule here. The prelude spelling is a transparent alias
for the kernel's computing `Eq` (`../50-stdlib/53 §1`), so branch refinement is
ordinary substitution into the equality operands followed by the existing
`Eq`/`Top`/`Bottom` reductions (`../10-kernel/16 §1.4`, §2). A branch may close
with `tt`, `Refl`, `J`, or a library combinator only if that term checks against
the branch-specialized proof target. In particular, proof irrelevance does **not
erase the branch obligation**: it equates proofs only after both proofs have
already been checked at the same proposition (`../10-kernel/16 §1.2`).

**Acceptance boundary.** A proof-returning dependent `match` accepts when:

- the scrutinee elaborates to an inductive value and the target classifies as a
  sort (`Ω_l` for proof targets, `Type l` for ordinary large elimination);
- the motive can be recovered by generalizing the expected target over the
  scrutinee and any indices it mentions;
- every type-possible constructor has a method whose body checks against the
  target after the constructor fields and branch refinement have been applied.

**Rejection boundary.** The same surface rejects when any branch supplies a proof
for the wrong specialized target, when the target does not classify as a sort,
when motive recovery would require guessing which occurrences to generalize, or
when the ordinary exhaustiveness/reachability checks of `§4` fail. The
elaborator may report an explicit "unsupported dependent motive" for a
not-yet-implemented recovery shape, but it must not silently fall back to a
constant motive that ignores the dependency, and it must not accept all proof
branches by proof irrelevance.

## 4. Exhaustiveness and reachability (required — the headline safety)

Ken requires this from day one. The checker is a **surface algorithm** (not a
kernel rule, `§4.4`) run during `match` compilation (`39 §2.6`).

### 4.1 Exhaustiveness

The arms MUST cover every **type-possible** constructor of the scrutinee type:
every value of the scrutinee type matches some (unguarded, `§3.3`) arm. The
algorithm walks the pattern matrix and, per scrutinee column, computes the set
of constructors the arms cover; a constructor of the scrutinee's `data` left
uncovered makes the `match` a **compile error** that **names the unmatched
pattern** (`§4.4` explains why naming it is the *surface* checker's job and not
the kernel's). A wildcard/variable column covers all remaining constructors. For
a **closed** `data` an exhaustive `match` needs **no** `default` arm, and the
compiler **proves totality** of the case analysis (`§4.4`).

> The unmatched-pattern **witness** is a most-general pattern not covered by any
> arm (e.g. `Blue`, or `VCons _ _ _` for a partial `Vec` match) — the
> constructive evidence of the gap, reported in the diagnostic (`24`).

### 4.2 Reachability

Every arm MUST be **reachable**: under the first-match semantics (`§3`), an arm
whose patterns are entirely subsumed by the union of the earlier arms matches no
value and is a **redundant-arm** warning/error. The same matrix walk detects it
(an arm reaching an empty residual matrix is unreachable). Two subtleties:

- A **guarded** arm is *not* counted as covering its constructor (`§3.3`), so a
  later unguarded arm for the same constructor is **reachable** (it catches the
  guard-failure cases) — guards loosen reachability exactly as they loosen
  coverage.
- A **literal** column never closes (`§3.1`), so a final variable/wildcard arm
  after literal arms is reachable (and *required* for exhaustiveness).

### 4.3 Indexed families — type-possible vs. index-impossible

For an indexed scrutinee (`§2`) the coverage obligation is over the constructors
**type-possible at the scrutinee's index**, not all constructors of the family.
A constructor whose target index cannot unify with the scrutinee's index is
**index-impossible** and need not be written. This splits cleanly, and the split
is the **soundness rule** the whole feature rests on:

- **Type-possible at the index** ⇒ the arm is **required**; omitting it is a
  non-exhaustiveness compile error (`§4.1`). The elaborator MUST NOT fabricate a
  method for a type-possible constructor (that would silently admit a partial
  function — the unsound failure mode).
- **Index-impossible** ⇒ the elaborator **synthesizes** the constructor's
  `elim_D` method by **absurdity**: the constructor's index equation (its target
  index ≐ the scrutinee's index) is refuted by constructor disjointness /
  index discrimination (e.g. `0 ≢ n+1` via `15`/`16`), yielding a proof of the
  empty index constraint from which the method body is `Empty`-eliminated. The
  kernel still receives a **total** `elim_D` (`14 §3`), so the omission is sound
  by construction.

The discriminator between "required" and "omittable" is therefore exactly **"is
this constructor type-possible at the scrutinee's index?"** — `§4.1` (closed,
non-indexed: every constructor type-possible ⇒ all required) and `§2`/this
section (indexed: only the index-satisfiable ones required) are the **same
rule** at two index regimes. The conformance corpus pins both faces and their
agreement (`§8`).

### 4.4 The trust boundary (why this is safe though untrusted)

The exhaustiveness/reachability checker is **untrusted surface** (`39 §1`), yet
the safety it enforces is **kernel-backed**, double-netting the headline:

- **The kernel proves the *eliminator* sound.** `elim_D` is total by strict
  positivity + structural ι (`14 §2`, `§9`); it **requires a method for every
  constructor** of the family.
- **The surface proves the *match covers* it.** A non-exhaustive `match` over a
  type-possible constructor cannot elaborate to a complete `elim_D` — the
  elaborator has no method body for the missing arm and (per `§4.3`) MUST NOT
  fabricate one, so the only honest outputs are **(a)** the surface
  exhaustiveness error naming the unmatched pattern (the good diagnostic), or
  **(b)** an under-applied `elim_D` the **kernel rejects** as ill-typed (the
  backstop). Either way a non-exhaustive `match` is **never** silently accepted
  as a partial function.

So the *safety* (totality of case analysis) is guaranteed even against a buggy
checker — the kernel cannot type an `elim_D` missing a method. What the surface
checker uniquely provides is the **quality** of the failure: a precise
"non-exhaustive, unmatched pattern `Blue`" rather than the kernel's bare
"eliminator under-applied." This is the crisp boundary: **kernel = the
eliminator is sound and total; surface = the match covers it, with a precise
witness.** (It is the `match` analogue of `39 §1`'s split — cleverness outside,
certainty inside.)

## 5. Refinement types

`{ x : A | φ x }` — the type of `x : A` for which the proposition `φ x : Ω`
holds (the comprehension subobject, `../20-verification/21 §2`; the predicate's
universe `Ω` is `../10-kernel/12 §5`, `16 §1`). At the surface:

```
type Nat        = { n : Int | n ≥ 0 }
type NonEmpty a = { xs : List a | xs ≠ Nil }
view head {a} (xs : NonEmpty a) : a = match xs { Cons x _ => x }
```

**The encoding — carrier plus obligation (normative, `21 §2`).** A refinement
`{x:A|φ}` is **not** a kernel type former. It elaborates to its **carrier `A`**;
the predicate `φ` is tracked by the (untrusted) elaborator and **every
introduction** of a value at the refinement — using an `A` where `{x:A|φ}` is
expected — **emits the obligation** `φ a` (`22 §2.1`), discharged by the prover
or surfaced as a typed hole (`24 §2`). Consequences, each grounded:

- **No implicit subset coercion past `φ`.** `A ≤ {x:A|φ}` (the introduction
  direction) costs the obligation `φ`; it is never a silent coercion. The
  **forgetful** direction `{x:A|φ} ≤ A` is **free** — in the carrier encoding it
  is the identity on `A` — and emits **no** obligation (`22 §2.1`/§2.5).
- **No runtime payload.** The proof component is a **mere proposition** (`16
  §1.2`) — proof-irrelevant and computationally irrelevant — so a refined value
  behaves as a bare `A` at runtime; refinements are **zero-cost** and pure
  compile-time enforcement.
- **Not a core `Σ` over Ω.** The naive reification `{x:A|φ}=Σ(A,φ)` is **not**
  used — it is collapsed by Ω-proof-irrelevance when the carrier is relevant
  (the landed `sort_pi_sigma` Σ-sort caveat, `21 §2`, `13 §4`/§5). The
  carrier-plus-obligation form is **independent** of that kernel erratum (it
  never forms a core `Σ` over an Ω predicate), so L2 builds on it as-is.

Refinements compose with `data`, records, and function arguments/results, and
are how `requires`/`ensures` desugar (`21 §1`/§2). Pushing a property into a
type makes the checker enforce it at **every** use — the surface route to L2
verification.

## 6. Smart constructors & views (optional sugar)

Pattern synonyms / view patterns (matching through an abstraction) and smart
constructors (a `view` that enforces an invariant and returns a refined type,
`§5`) are ergonomic sugar over `§1`–`§5`; whether to include them is
**OQ-syntax**. The semantic core — constructors + `elim_D` + exhaustiveness +
refinements — does not depend on them.

## 7. Level-discipline reconcile

Per the standing directive, every formation rule here is given its explicit
level computation and reconciled against `../10-kernel/12`; no rule adds a
universe computation — each is an instance of a landed kernel rule.

- **`data D (Δ_p) : (Δ_i) → Type ℓ`.** Constructor-argument types live at `ℓ` or
  below (predicativity, `14 §1` → `12 §2`); the family lands at the declared
  `ℓ`. **No new rule** — the kernel's inductive formation (`14 §1`) computes it.
- **W-style argument `(b:B) → D Δ_p t̄`.** Lives at `max(level B, ℓ)`; `14 §1`'s
  rule forces `level B ≤ ℓ` (the domain is absorbed into the family level, `14
  §2.1`). Existing rule, **no new formation**.
- **`match` → `elim_D`.** The motive `M` may land in any `Type ℓ'` — **large
  elimination** is permitted under the predicative universe checks (`14 §3`), so
  a `match` may compute a *type* (e.g. an indexed result). No level restriction
  beyond the kernel's `Univ` checks (`12 §1`).
- **Proof-returning `match` → `elim_D`.** A proof target such as
  `Equal A lhs rhs` lands in `Ω_l` (`15 §2`, `16 §2`), so the recovered motive
  may have codomain `Ω_l` rather than `Type ℓ'` (`§3.5`, `14 §3`). This is the
  same predicative Π-into-Ω rule as other propositions (`16 §1.1`); it adds no
  universe coercion and does not turn the proof target into a `Type`.
- **Refinement `{x:A|φ}`.** `φ : Ω` (proof-irrelevant, `12 §5`/`16 §1`); the
  refinement's **core image is its carrier `A`** (`§5`), so it sits at `A`'s
  level `l` (`A : Type l`) — a *subtype at the same level*, predicative, **no
  universe bump** (`12 §2`/§3, non-cumulative). The obligation `φ a` is an Ω
  proposition discharged in V3 (`22`); it introduces no new universe (`22 §7`).

## 8. What WS-L must deliver here

Real sum types with constructors + a computing eliminator (no opaque lowering);
indexed/GADT-like families with index-aware coverage; `match` → `elim_D` with
dependent-motive recovery and per-branch definitional refinement;
proof-returning dependent motives whose `Equal`/`Ω` target mentions the
scrutinee, with wrong-specialized-branch negatives;
**exhaustiveness + reachability** checking with a named unmatched-pattern
witness; `Result`/`Option`/`Either` in the prelude; and refinement types with
free forgetful coercion + obligation emission on introduction. Acceptance is
part of **G6** (real sum types end-to-end). The whole layer is **untrusted**;
the kernel re-checks every emitted `elim_D` (`§4.4`).

Conformance: `../../conformance/surface/data-match/` — AC1 (`data` is real:
construct-then-eliminate **reduces**), AC2 (`match`→`elim_D` computes; nested→
nested), AC3 (exhaustiveness — the headline: a missing case **rejects naming the
unmatched pattern**, the exhaustive version accepts — verdict **and** the named
witness flip), AC4 (reachability — a redundant arm flagged, verdict flips), AC5
(indexed family — the impossible application **rejects** *while* the impossible
arm may be **omitted**: a non-degenerate pair on the same `§4.3` rule), AC6
(branch refinement = `22 §3` hypothesis — a *dependent* motive, asserted
structurally), AC7 (refinement type — the obligation `φ` is **emitted** on
introduction (observe the VC structurally), the forgetful direction free, no
silent coercion). Per-case verdict/structural-flip + the **cross-case sweep**:
the exhaustiveness/coverage class (`§4.1`/§4.3) agrees: "type-possible at the
index ⇒ required; index-impossible ⇒ omittable-and-absurd-filled."
