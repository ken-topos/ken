# `lawful-functors` — `Semigroup`, `Monoid` (+ held `Functor`, `Foldable`)

**Spec catalog entry:** `spec/50-stdlib/55-lawful-functors.md`. The first WP
of the **catalog campaign** (`docs/program/06-catalog-campaign.md`, CAT-1) and
the pattern-setter for the constructor classes every later catalog layer
leans on. This slice delivers the **value-level algebra** half — `Semigroup`
and `Monoid`. The **constructor classes** `Functor`/`Foldable` (over
`f : Type -> Type`) land in this same package once the higher-kinded
class-parameter extension does (see *Pinned sub-deliverables* below); they are
**held**, not yet present.

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
- `instance Semigroup (List Nat)`, `instance Monoid (List Nat)` — the **List
  append monoid** (`op = list_append`, `mempty = Nil`); laws proved by
  **induction + `cong`**, zero `Axiom`. The law *proofs* (`list_assoc`,
  `list_left_unit`, `list_right_unit`) are **generic in the element type**
  (`(a : Type) -> …` views); only the `instance` bundling is monomorphic —
  see *Pinned sub-deliverables*.

## Derivation path + `trusted_base()` delta

- **Both classes** are `class` declarations = record types (`33 §5.2`,
  right-nested Σ over `13 §3`), built from the kernel's `Equal`/`Omega`
  vocabulary (`15`/`16`, prelude) + the Σ/record machinery. **No new kernel
  former, zero delta.**
- **Law sort.** A Semigroup/Monoid `op` is `a`-valued (not `Bool`-valued), so
  its laws are the kernel's own propositional `Equal a u v : Omega` directly —
  *not* the `IsTrue`/`Bool` bridge `Eq`/`Ord` use. A value equation is
  proof-irrelevant (`Omega`), and the `51 §3` truncation catch does **not**
  fire (no bare `\/`/`exists`) — every law is `Omega`-clean, no `‖·‖`.
- **Both instances are ZERO-DELTA — the inductive-carrier exemplar** (`51 §6`;
  the path the `Int` instances of `lawful-classes` could *not* take). `List`
  and `Bool` are real inductives with eliminators, so every ∀-law is a real
  kernel proof by induction / finite case-split (Ω-motive `Elim`, K4
  `3be0e30`; `Top`-intro `tt` / `Bottom`-elim, K5 `1c84a30`; operand-`whnf`,
  K7 `4ae2baf`). **No `Axiom` anywhere; nothing enters `trusted_base()`.**
- **`tt`-vs-`Refl` discrimination** (the K7 subtlety, documented inline): a
  law branch whose two endpoints reduce to the **same constructor**
  (`Nil a`; a `Bool` literal) observationally collapses to `Top` — closed by
  `tt`, not `Refl` (whose goal must stay `Eq`-shaped); a branch whose
  endpoints reduce to a **neutral** application (`list_append a ys zs`) stays
  `Eq`-shaped — closed by `Refl`.
- **Dependencies (reused, never re-defined — subsume-don't-proliferate):**
  - `cong`/`sym`/`trans` — `packages/transport/transport.ken` (over the `J`
    former), for the inductive congruence steps. Zero delta (transport is
    itself zero-delta).
  - `list_append` — `packages/collections/collections.ken`, the List monoid
    operation. Reused, not re-defined (a second append would collide with
    the landed one and violate subsume-don't-proliferate).
  - `bool_and` — a **transparent** (match-based) `view` defined here, **not**
    the `and_bool` primitive: a primitive never reduces on a symbolic
    argument (K1 `whnf` unfolds only `Decl::Transparent`), which would make
    the laws unprovable — the same reason `lawful-classes` defines its own
    transparent `bool_or`. Zero delta (ordinary Ken).

## Pinned sub-deliverables (outer-ring elaborator extensions, kernel-untouched)

Two surface gaps block the *fully general* forms of this tranche. Both are
`ken-elaborator`-only, **zero `ken-kernel` diff, no new `Term`/`Decl`** —
flagged to Steward (`findings/forks → Steward`, per the CAT-1 frame).

1. **Higher-kinded class parameter** — `class Functor (f : Type -> Type)`.
   The class param is hard-coded to `Type0` (`elab_class_decl`, 3 sites) and
   the parser takes a bare ident only. **Architect's CAT-1 core ruling** sized
   this (AST param-kind field, parser binder form, ~10-line elab fix, an
   instance-side build-verify point). **Blocks `Functor`/`Foldable`** — held.
2. **Parametric instance head** — `instance Monoid (List a)` (a value class
   over a *parametric* carrier). The parser *accepts* the head, but
   elaboration has no binder path for the head's free `a` and does not
   generalize the instance over it (`UnresolvedCon "a"`, grounded by probe).
   This is **distinct from (1)** (which is about the class *param*'s kind);
   here the *instance head* needs its free vars bound/generalized. **Does not
   block the tranche** — the value monoids bundle at closed carriers (`Bool`,
   `List Nat`) today and their proofs are already generic; the parametric
   `instance` is a generality upgrade. The `map-verified-laws` precedent is
   the same shape — a parametric structure's `instance`/`where` bundling is
   `(oracle)`-deferred while the parametric substance is real.
