# `lawful-functors` — `Semigroup`, `Monoid`, `Functor`, `Foldable`

**Spec catalog entry:** `spec/50-stdlib/55-lawful-functors.md`. The first WP
of the **catalog campaign** (`docs/program/06-catalog-campaign.md`, CAT-1) and
the pattern-setter for the constructor classes every later catalog layer
leans on. This package now carries the **value-level algebra** classes
`Semigroup`/`Monoid` and the **constructor classes** `Functor`/`Foldable`
over `f : Type -> Type`.

> **Naming.** Despite the package name, this tranche carries the value-level
> algebra companions (`Semigroup`/`Monoid`) alongside the functor classes,
> because `Foldable`'s `foldMap` consumes a `Monoid`. Open to a rename/split
> (e.g. `lawful-algebra` + `lawful-functors`) if `06`'s `ken.*` layer shape
> prefers it — flagged to the enclave leader, not load-bearing.

## Public API

- `class Semigroup a { op; assoc }` — an associative binary operation on `a`
  (`55 §2`). `op` is a plain identifier field, not an infix `<>` token (the
  operator spelling is deferred sugar, `OQ-syntax`).
- `class Monoid a { op; mempty; assoc; left_unit; right_unit }` — a Semigroup
  with a two-sided identity (`55 §3`). **Restates** `op`/`assoc` rather than
  wiring a `Semigroup` superclass field, recording the subsumption as a fact —
  the `DecEq`-subsumes-`Eq` precedent (`51 §2.2`); no new kind machinery.
- `instance Semigroup Bool`, `instance Monoid Bool` — the **Bool conjunction
  monoid** (`op = bool_and`, `mempty = True`); laws proved by **finite
  case-split**, zero `Axiom`.
- `instance Semigroup (List Nat)` and `instance Monoid (List a)` — the **List
  append monoid** (`op = list_append`, `mempty = Nil`); laws proved by
  **induction + `cong`**, zero `Axiom`. The `Monoid` dictionary is genuinely
  generic in the element type and cites the generic `list_assoc`,
  `list_left_unit`, and `list_right_unit` proofs.
- `class Functor (f : Type -> Type) { map; id_law; fusion_law }` — the
  constructor-class map interface (`55 §5.2`). The laws use the settled
  **single pointwise field** form only: identity quantifies over `(x : f a)`,
  and fusion quantifies over `(x : f a)`; there is no point-free duplicate.
- `instance Functor List`, `instance Functor Option` — map implementations
  over the inductive carriers; laws are proved by list induction and option
  case-split/definitional equality, zero `Axiom`.
- `class Foldable (f : Type -> Type) { foldr; foldMap; toList;
  foldr_toList; foldMap_coherence }` — `foldr` is primary, `toList` has a
  reconstruction law, and `foldMap` is pinned to the selected `Monoid`
  dictionary by the coherence law
  `foldMap g x = foldr (foldMap_step mon g) mempty x`.
- `instance Foldable List`, `instance Foldable Option` — folds over the same
  inductive carriers. List laws use induction + `cong`; Option laws close by
  case split or definitional equality, zero `Axiom`.

## Derivation path + `trusted_base()` delta

- **All four classes** are `class` declarations = record types (`33 §5.2`,
  right-nested Σ over `13 §3`), built from the kernel's `Equal`/`Omega`
  vocabulary (`15`/`16`, prelude) + the Σ/record machinery. **No new kernel
  former, zero delta.**
- **Law sort.** A Semigroup/Monoid `op` is `a`-valued (not `Bool`-valued), so
  its laws are the kernel's own propositional `Equal a u v : Omega` directly —
  *not* the `IsTrue`/`Bool` bridge `Eq`/`Ord` use. A value equation is
  proof-irrelevant (`Omega`), and the `51 §3` truncation catch does **not**
  fire (no bare `\/`/`exists`) — every law is `Omega`-clean, no `‖·‖`.
- **All instances are ZERO-DELTA — the inductive-carrier exemplar** (`51 §6`;
  the path the `Int` instances of `lawful-classes` could *not* take). `List`,
  `Option`, and `Bool` are real inductives with eliminators, so every ∀-law
  is a real kernel proof by induction / finite case-split / definitional
  equality (Ω-motive `Elim`, K4
  `3be0e30`; `Top`-intro `tt` / `Bottom`-elim, K5 `1c84a30`; operand-`whnf`,
  K7 `4ae2baf`). **No `Axiom` anywhere; nothing enters `trusted_base()`.**
- **`tt`-vs-`Refl` discrimination** (the K7 subtlety, documented inline): a
  law branch whose two endpoints reduce to the **same constructor**
  (`Nil a`; a `Bool` literal) observationally collapses to `Top` — closed by
  `tt`, not `Refl` (whose goal must stay `Eq`-shaped); a branch whose
  endpoints reduce to a **neutral** application (`list_append a ys zs`) stays
  `Eq`-shaped — closed by `Refl`.
- **Dependencies (reused, never re-defined — subsume-don't-proliferate):**
  - `cong`/`sym`/`trans` — `catalog/packages/Core/Transport.ken` (over the `J`
    former), for the inductive congruence steps. Zero delta (transport is
    itself zero-delta).
  - `list_append` — `catalog/packages/Data/Collections/Collections.ken`, the List monoid
    operation. Reused, not re-defined (a second append would collide with
    the landed one and violate subsume-don't-proliferate).
  - `bool_and` — a **transparent** (match-based) `view` defined here, **not**
    the `and_bool` primitive: a primitive never reduces on a symbolic
    argument (K1 `whnf` unfolds only `Decl::Transparent`), which would make
    the laws unprovable — the same reason `lawful-classes` defines its own
    transparent `bool_or`. Zero delta (ordinary Ken).

## Pinned sub-deliverables (outer-ring elaborator extensions, kernel-untouched)

Two surface gaps were needed for the fully general forms in this package. Both
landed as `ken-elaborator`-only work, **zero `ken-kernel` diff, no new
`Term`/`Decl`**.

1. **Higher-kinded class parameter** — `class Functor (f : Type -> Type)`.
   CAT-1 D1 landed the bounded elaborator extension (AST param-kind field,
   parser binder form, the `elab_class_decl` kind substitution, and
   bare-indformer instance-head verification). CAT-1 D3 uses it for
   `Functor`/`Foldable`.
2. **Parametric instance head** — `instance Monoid (List a)` (a value class
   over a *parametric* carrier). CAT-1 D1 landed the free-head-variable
   generalizer, and CAT-1 D2 uses it here: the dictionary is elaborated as
   `(a : Type) -> Monoid (List a)`.
