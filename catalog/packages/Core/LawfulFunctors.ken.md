# `lawful-functors` — `Semigroup`, `Monoid`, `Functor`, `Foldable`

The first WP of the catalog campaign (`docs/program/06-catalog-campaign.md`,
CAT-1) and the pattern-setter for every later constructor-class layer. This
package carries the **value-level algebra** classes `Semigroup`/`Monoid`
alongside the **constructor classes** `Functor`/`Foldable` over
`f : Type → Type`.

**Naming.** Despite the package name, this tranche carries the value-level
algebra companions (`Semigroup`/`Monoid`) alongside the functor classes,
because `Foldable`'s `fold_map` consumes a `Monoid`. Open to a rename/split
(e.g. `lawful-algebra` + `lawful-functors`) if `06`'s `ken.*` layer shape
prefers it — flagged to the enclave leader, not load-bearing.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws  proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [Findings](#6-findings)
7. [References](#7-references)
8. [Trust  derivation](#8-trust--derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws  proofs](#4-laws--proofs)
- *Researcher* →
  [Laws  proofs](#4-laws--proofs) → [Design notes](#5-design-notes)

## 1. Motivation

`spec/50-stdlib/55-lawful-functors.md` gives Ken an associative operation
(`Semigroup`), a monoid over it (`Monoid`), a structure-preserving map over a
type constructor (`Functor`), and a fold that is coherent with a chosen
`Monoid` (`Foldable`) — the vocabulary every later constructor-class layer
(`Applicative`, `Monad`, `Traversable`, …) builds on. Every class here stays
in the elaborator/package layer: no kernel feature, no new `Term`/`Decl`.

## 2. Definition

`Semigroup a` is an associative binary operation on `a`. `op` is the
ergonomic `<>`/mappend, a plain identifier field (not an infix token: a
surface `<>` operator is deferred sugar, OQ-syntax — exactly as `36 §4.5`'s
`get`/`put` name the effect ops rather than minting operators). `assoc` is a
propositional equation in `Omega`.

```ken
class Semigroup a {
  op    : a → a → a ;
  assoc : (x : a) → (y : a) → (z : a) → Equal a (op (op x y) z) (op x (op y z))
}
```

`Monoid a` is a `Semigroup` with a two-sided identity `mempty`. Following the
`DecEq`-subsumes-`Eq` precedent (`51 §2.2`): the STRONGER class RESTATES the
weaker one's operation + law rather than wiring a superclass field, and the
subsumption — "a `Monoid` yields a `Semigroup` by forgetting `mempty`/
`left_unit`/`right_unit`, keeping `op`/`assoc`" — is recorded as a FACT here,
not a `where`-constraint. Same shape as `Eq`/`Ord`, no new kind machinery.
(Whether the CONSTRUCTOR-class chain of CAT-2 — `Functor → Applicative →
Monad` — should instead WIRE superclass fields is a template question for the
Architect's higher-kinded lane, flagged there, not decided here.)

```ken
class Monoid a {
  op         : a → a → a ;
  mempty     : a ;
  assoc      : (x : a) → (y : a) → (z : a) → Equal a (op (op x y) z) (op x (op y z)) ;
  left_unit  : (x : a) → Equal a (op mempty x) x ;
  right_unit : (x : a) → Equal a (op x mempty) x
}
```

The constructor classes `Functor` and `Foldable` are introduced in §4
alongside their instances, each right after the small helper function its own
field types need (`idf`/`comp` for `Functor`'s laws, `monoid_mempty`/
`fold_map_step` for `Foldable`'s coherence law) — the same build order the
original package source uses.

## 3. Using it

Once a concrete instance is registered, its fields project directly off the
synthesized `C_instance_T` dictionary, the same projection form
`catalog/packages/Core/LawfulClasses.ken.md §3` documents for `Eq`/`Ord`. §4's
`instance Monoid Bool` and `instance Monoid (List a)` are the two worked
examples here: `Monoid_instance_Bool` restates the *same* `op`/`assoc`
definitions (`bool_and`/`band_assoc`) that `Semigroup_instance_Bool` uses, and
`Monoid_instance_List`'s dictionary is genuinely generic in the element type
— it elaborates as `(a : Type) → Monoid (List a)`, so a caller applies it to
a concrete `a` before projecting a field, exactly like any other
parametric-head instance.

## 4. Laws  proofs

### 4.1 The `List` append monoid — the canonical inductive carrier

`op = list_append`, `mempty = Nil`. The three laws and the `Monoid` instance
are GENERIC in `a`: the CAT-1 D1 parametric-instance-head path elaborates
`instance Monoid (List a)` as `(a : Type) → Monoid (List a)`, and the law
fields cite the existing generic `list_*` proofs rather than re-proving them
at a closed carrier.

Left unit is DEFINITIONAL: `list_append Nil x` iota-reduces to `x` by
append's first match arm, so the goal `Equal (List a) x x` stays `Eq`-shaped
over the neutral `x` — closed by `Refl`. Assoc is proved by induction on the
first list: base (`Nil`) both sides reduce to the NEUTRAL
`list_append a ys zs`, still `Eq`-shaped, `Refl`; step (`Cons h t`) lifts the
tail IH under `Cons a h` with `cong`. Right unit is proved by induction on
the list: base (`Nil`) both sides reduce to the CONSTRUCTOR `Nil a`, which
observationally collapses to `Top` (same nullary ctor, `16 §8.1` / K7) — so
the goal is no longer `Eq`-shaped and `Refl` does not apply; it is
`Top`-introduced by `tt` (the exact `tt`-vs-`Refl` discrimination
`catalog/packages/Core/LawfulClasses.ken.md §4` documents: constructor-headed
endpoints → `Top` → `tt`; neutral endpoints → stuck `Eq` → `Refl`); step
(`Cons h t`) is `cong` under `Cons a h` on the tail IH.

```ken
fn list_left_unit (a : Type) (xs : List a)
  : Equal (List a) (list_append a (Nil a) xs) xs =
  Refl

fn list_assoc (a : Type) (xs : List a) (ys : List a) (zs : List a)
  : Equal (List a) (list_append a (list_append a xs ys) zs)
                   (list_append a xs (list_append a ys zs)) =
  match xs {
    Nil ⇒ Refl ;
    Cons h t ⇒
      cong (List a) (List a)
        (list_append a (list_append a t ys) zs)
        (list_append a t (list_append a ys zs))
        (Cons a h)
        (list_assoc a t ys zs)
  }

fn list_right_unit (a : Type) (xs : List a)
  : Equal (List a) (list_append a xs (Nil a)) xs =
  match xs {
    Nil ⇒ tt ;
    Cons h t ⇒
      cong (List a) (List a)
        (list_append a t (Nil a))
        t
        (Cons a h)
        (list_right_unit a t)
  }

instance Semigroup (List Nat) {
  op    = list_append Nat ;
  assoc = list_assoc Nat
}

instance Monoid (List a) {
  op         = list_append a ;
  mempty     = Nil a ;
  assoc      = list_assoc a ;
  left_unit  = list_left_unit a ;
  right_unit = list_right_unit a
}
```

### 4.2 The `Bool` conjunction monoid — the finite carrier

Complements the `List` monoid's inductive style with the FINITE case-split
style: no induction / no IH, every branch closes by `tt` or `Refl` directly.
`bool_and` is a transparent (match-based) view, NOT the `and_bool`
primitive — deliberately, for the same reason
`catalog/packages/Core/LawfulClasses.ken.md` defines its own transparent
`bool_or`: a primitive never reduces on a symbolic argument (K1 `whnf` only
unfolds `Decl::Transparent`), which would make the laws unprovable; a
transparent `bool_and` reduces on each concrete constructor at zero kernel
cost.

Associativity is a full 2×2×2 case-split; each concrete branch reduces both
sides to the same literal, collapsing to `Top` → `tt`. Left unit is
DEFINITIONAL: `bool_and True x` reduces to `x` (first match arm), goal
`Equal Bool x x` neutral → `Refl`. Right unit needs the case-split
(`bool_and x True` is stuck on symbolic `x`): each branch reduces to a
literal equal to itself → `Top` → `tt`.

```ken
fn bool_and (p : Bool) (q : Bool) : Bool =
  match p { True ⇒ q ; False ⇒ False }

fn band_assoc (x : Bool) (y : Bool) (z : Bool)
  : Equal Bool (bool_and (bool_and x y) z) (bool_and x (bool_and y z)) =
  match x {
    True ⇒ match y {
      True  ⇒ match z { True ⇒ tt ; False ⇒ tt } ;
      False ⇒ match z { True ⇒ tt ; False ⇒ tt }
    } ;
    False ⇒ match y {
      True  ⇒ match z { True ⇒ tt ; False ⇒ tt } ;
      False ⇒ match z { True ⇒ tt ; False ⇒ tt }
    }
  }

fn band_left_unit (x : Bool) : Equal Bool (bool_and True x) x =
  Refl

fn band_right_unit (x : Bool) : Equal Bool (bool_and x True) x =
  match x { True ⇒ tt ; False ⇒ tt }

instance Semigroup Bool {
  op    = bool_and ;
  assoc = band_assoc
}

instance Monoid Bool {
  op         = bool_and ;
  mempty     = True ;
  assoc      = band_assoc ;
  left_unit  = band_left_unit ;
  right_unit = band_right_unit
}
```

### 4.3 `Functor` — structure-preserving map over a type constructor

`class Functor (f : Type → Type)` uses the CAT-1 higher-kinded
class-parameter extension (§5). Its laws use the settled single pointwise
field form only: identity quantifies over `(x : f a)`, and fusion quantifies
over `(x : f a)`; there is no point-free duplicate law surface. `List`'s
instance is proved by induction + `cong` (the same `tt`-vs-`Refl` shape as
§4.1); `Option`'s instance closes by finite case-split / definitional
equality — `None` collapses to `Top` (`tt`), `Some v` stays `Eq`-shaped
(`Refl`, since `map` reduces on the concrete `Some` head either way).

```ken
fn idf (a : Type) (x : a) : a = x

fn comp (a : Type) (b : Type) (c : Type) (g : b → c) (h : a → b) (x : a) : c =
  g (h x)

class Functor (f : Type → Type) {
  map        : (a : Type) → (b : Type) → (a → b) → f a → f b ;
  id_law     : (a : Type) → (x : f a) →
                 Equal (f a) (map a a (idf a) x) x ;
  fusion_law : (a : Type) → (b : Type) → (c : Type) →
                 (g : b → c) → (h : a → b) → (x : f a) →
                 Equal (f c)
                   (map a c (comp a b c g h) x)
                   (map b c g (map a b h x))
}

fn list_map (a : Type) (b : Type) (g : a → b) (xs : List a) : List b =
  match xs {
    Nil ⇒ Nil b ;
    Cons h t ⇒ Cons b (g h) (list_map a b g t)
  }

fn list_functor_id (a : Type) (xs : List a)
  : Equal (List a) (list_map a a (idf a) xs) xs =
  match xs {
    Nil ⇒ tt ;
    Cons h t ⇒
      cong (List a) (List a)
        (list_map a a (idf a) t)
        t
        (Cons a h)
        (list_functor_id a t)
  }

fn list_functor_fusion (a : Type) (b : Type) (c : Type)
  (g : b → c) (h : a → b) (xs : List a)
  : Equal (List c)
      (list_map a c (comp a b c g h) xs)
      (list_map b c g (list_map a b h xs)) =
  match xs {
    Nil ⇒ tt ;
    Cons x rest ⇒
      cong (List c) (List c)
        (list_map a c (comp a b c g h) rest)
        (list_map b c g (list_map a b h rest))
        (Cons c (g (h x)))
        (list_functor_fusion a b c g h rest)
  }

fn option_map (a : Type) (b : Type) (g : a → b) (x : Option a) : Option b =
  match x {
    None ⇒ None b ;
    Some v ⇒ Some b (g v)
  }

fn option_functor_id (a : Type) (x : Option a)
  : Equal (Option a) (option_map a a (idf a) x) x =
  match x {
    None ⇒ tt ;
    Some v ⇒ Refl
  }

fn option_functor_fusion (a : Type) (b : Type) (c : Type)
  (g : b → c) (h : a → b) (x : Option a)
  : Equal (Option c)
      (option_map a c (comp a b c g h) x)
      (option_map b c g (option_map a b h x)) =
  match x { None ⇒ tt ; Some v ⇒ Refl }

instance Functor List {
  map        = list_map ;
  id_law     = list_functor_id ;
  fusion_law = list_functor_fusion
}

instance Functor Option {
  map        = option_map ;
  id_law     = option_functor_id ;
  fusion_law = option_functor_fusion
}
```

### 4.4 `Foldable` — `foldr`-primary folds, coherent through `Monoid`

`foldr` is primary, `to_list` has a reconstruction law, and `fold_map` is
pinned to the selected `Monoid` dictionary by the coherence law
`fold_map g x = foldr (fold_map_step mon g) mempty x`. `List` laws use
induction + `cong`; `Option` laws close by case-split or definitional
equality.

```ken
fn monoid_mempty (m : Type) (mon : Monoid m) : m = mon.mempty

fn fold_map_step (a : Type) (m : Type) (mon : Monoid m) (g : a → m)
  (y : a) (acc : m) : m =
  (mon.op) (g y) acc

class Foldable (f : Type → Type) {
  foldr             : (a : Type) → (b : Type) → (a → b → b) → b → f a → b ;
  fold_map           : (a : Type) → (m : Type) → Monoid m → (a → m) → f a → m ;
  to_list            : (a : Type) → f a → List a ;
  foldr_to_list      : (a : Type) → (x : f a) →
                        Equal (List a)
                          (foldr a (List a) (Cons a) (Nil a) x)
                          (to_list a x) ;
  fold_map_coherence : (a : Type) → (m : Type) → (mon : Monoid m) →
                        (g : a → m) → (x : f a) →
                        Equal m
                          (fold_map a m mon g x)
                          (foldr a m (fold_map_step a m mon g) (monoid_mempty m mon) x)
}

fn list_foldr (a : Type) (b : Type) (k : a → b → b) (z : b) (xs : List a) : b =
  match xs {
    Nil ⇒ z ;
    Cons h t ⇒ k h (list_foldr a b k z t)
  }

fn list_fold_map (a : Type) (m : Type) (mon : Monoid m) (g : a → m)
  (xs : List a) : m =
  match xs {
    Nil ⇒ mon.mempty ;
    Cons h t ⇒ (mon.op) (g h) (list_fold_map a m mon g t)
  }

fn list_to_list (a : Type) (xs : List a) : List a = xs

fn list_foldr_to_list (a : Type) (xs : List a)
  : Equal (List a) (list_foldr a (List a) (Cons a) (Nil a) xs) (list_to_list a xs) =
  match xs {
    Nil ⇒ tt ;
    Cons h t ⇒
      cong (List a) (List a)
        (list_foldr a (List a) (Cons a) (Nil a) t)
        (list_to_list a t)
        (Cons a h)
        (list_foldr_to_list a t)
  }

fn list_fold_map_coherence (a : Type) (m : Type) (mon : Monoid m)
  (g : a → m) (xs : List a)
  : Equal m
      (list_fold_map a m mon g xs)
      (list_foldr a m (fold_map_step a m mon g) (monoid_mempty m mon) xs) =
  match xs {
    Nil ⇒ Refl ;
    Cons h t ⇒
      cong m m
        (list_fold_map a m mon g t)
        (list_foldr a m (fold_map_step a m mon g) (monoid_mempty m mon) t)
        ((mon.op) (g h))
        (list_fold_map_coherence a m mon g t)
  }

fn option_foldr (a : Type) (b : Type) (k : a → b → b) (z : b) (x : Option a) : b =
  match x {
    None ⇒ z ;
    Some v ⇒ k v z
  }

fn option_fold_map (a : Type) (m : Type) (mon : Monoid m) (g : a → m)
  (x : Option a) : m =
  match x {
    None ⇒ mon.mempty ;
    Some v ⇒ (mon.op) (g v) (mon.mempty)
  }

fn option_to_list (a : Type) (x : Option a) : List a =
  option_foldr a (List a) (Cons a) (Nil a) x

fn option_foldr_to_list (a : Type) (x : Option a)
  : Equal (List a) (option_foldr a (List a) (Cons a) (Nil a) x) (option_to_list a x) =
  Refl

fn option_fold_map_coherence (a : Type) (m : Type) (mon : Monoid m)
  (g : a → m) (x : Option a)
  : Equal m
      (option_fold_map a m mon g x)
      (option_foldr a m (fold_map_step a m mon g) (monoid_mempty m mon) x) =
  match x { None ⇒ Refl ; Some v ⇒ Refl }

instance Foldable List {
  foldr             = list_foldr ;
  fold_map           = list_fold_map ;
  to_list            = list_to_list ;
  foldr_to_list      = list_foldr_to_list ;
  fold_map_coherence = list_fold_map_coherence
}

instance Foldable Option {
  foldr             = option_foldr ;
  fold_map           = option_fold_map ;
  to_list            = option_to_list ;
  foldr_to_list      = option_foldr_to_list ;
  fold_map_coherence = option_fold_map_coherence
}
```

## 5. Design notes

**No new kernel feature (AC1).** A class is a record (`33 §5.2`, right-nested
Σ over `13 §3`); a law is an `Omega` proposition (`16 §1`). Unlike `Eq`/`Ord`
(whose ops are `Bool`-valued, so their laws ride the
`IsTrue b := Equal Bool b True` bridge, `51 §2`), a `Semigroup`/`Monoid`
operation RETURNS an `a`, so its laws are the kernel's OWN propositional
equality `Equal a u v : Omega` directly — an equation between `a`-values,
proof-irrelevant, no truncation (the `51 §3` truncation catch never fires:
these are value equations, not a bare `∨`/`∃`).

**Laws PROVED over inductive carriers, never postulated (AC3).** The
`List a` append monoid and `List` `Functor`/`Foldable` laws go by induction +
`cong` (`catalog/packages/Core/Transport.ken.md`); `Bool`'s conjunction
monoid and `Option` `Functor`/`Foldable` laws go by finite case-split /
definitional equality. Zero `Axiom`, zero `trusted_base()` delta — every
instance is a real `declare_def` record value, kernel-re-checked. This is the
same inductive-carrier zero-delta exemplar
`catalog/packages/Core/LawfulClasses.ken.md §6` names for `Bool` — here it
covers every instance in the package, not just one.

**Dependencies (reused, never re-defined — subsume-don't-proliferate).**
`cong`/`sym`/`trans` (`catalog/packages/Core/Transport.ken.md`, over the `J`
former) for the inductive congruence steps; `list_append`
(`catalog/packages/Data/Collections/Collections.ken`) as the `List` monoid
operation, reused rather than re-defined (a second append would collide with
the landed one).

**Two pinned sub-deliverables (outer-ring elaborator extensions,
kernel-untouched).** Both landed as `ken-elaborator`-only work, zero
`ken-kernel` diff, no new `Term`/`Decl`:

1. **Higher-kinded class parameter** — `class Functor (f : Type → Type)`.
   CAT-1 D1 landed the bounded elaborator extension (AST param-kind field,
   parser binder form, the `elab_class_decl` kind substitution, and
   bare-indformer instance-head verification). CAT-1 D3 uses it for
   `Functor`/`Foldable`.
2. **Parametric instance head** — `instance Monoid (List a)` (a value class
   over a *parametric* carrier). CAT-1 D1 landed the free-head-variable
   generalizer, and CAT-1 D2 uses it here: the dictionary is elaborated as
   `(a : Type) → Monoid (List a)`.

## 6. Findings

- **Kernel-reduction defect:** none.
- **Abstraction candidate:** none beyond what §2/§4 already provide — the two
  elaborator extensions above are pinned sub-deliverables, not open gaps.

## 7. References

None — this entry's design (the `Semigroup`/`Monoid` restatement pattern,
the higher-kinded constructor classes) is Ken-native, not consulted from an
external reference implementation.

## 8. Trust  derivation

1. **Spec / WP.** `spec/50-stdlib/55-lawful-functors.md`;
   `docs/program/06-catalog-campaign.md` (CAT-1, the catalog campaign's first
   WP, D1/D2/D3 sub-deliverables).
2. **Public API.** `class Semigroup`, `class Monoid`,
   `instance Semigroup (List Nat)`, `instance Monoid (List a)`,
   `instance Semigroup Bool`, `instance Monoid Bool`, `class Functor`,
   `instance Functor List`, `instance Functor Option`, `class Foldable`,
   `instance Foldable List`, `instance Foldable Option`.
3. **Source map.**

   | Task | Section |
   |---|---|
   | See the four classes | [Definition](#2-definition), [Laws  proofs](#4-laws--proofs) |
   | Project a field off a dictionary | [Using it](#3-using-it) |
   | The `List`/`Bool`/`Option` proofs | [Laws  proofs](#4-laws--proofs) |
   | Why value-level laws skip the `IsTrue` bridge, the two pinned sub-deliverables | [Design notes](#5-design-notes) |

4. **Derivation path.** All four classes are `class` declarations = record
   types (`33 §5.2`, right-nested Σ over `13 §3`), built from the kernel's
   `Equal`/`Omega` vocabulary (`15`/`16`, prelude) + the Σ/record machinery.
   No new kernel former. Every instance reduces through ordinary induction
   (`elim_List`) or finite case-split (`elim_Bool`/`elim_Option`) into an
   `Omega`-motive, closing with `tt`/`Refl`/`cong` — no `Axiom`.
5. **`trusted_base()` delta.** **Zero.** Every instance in this package —
   `List`, `Bool`, and `Option` alike — is a real, kernel-checked proof; no
   law field is postulated anywhere.
6. **Proof families.** `List` instances: structural induction + `cong`
   lifting the tail IH under the head constructor (`§4.1`, `§4.3`, `§4.4`).
   `Bool` instances: full finite case-split, every branch closing by `tt` or
   `Refl` directly, no IH (`§4.2`). `Option` instances: single-level
   case-split / definitional equality (`§4.3`, `§4.4`).
7. **Consumers.** `crates/ken-elaborator/tests/cat1_lawful_functors_package.rs`
   loads this package directly; `ds3_sum_combinators_acceptance.rs`,
   `ds7_applicative_monad_acceptance.rs`, `ds8_traversable_acceptance.rs`, and
   `either_catalog_package_acceptance.rs` load it as a cross-file
   prerequisite for their own `Functor`/`Foldable`/`Monoid`-consuming
   packages.
8. **Validation evidence.**
   `crates/ken-elaborator/tests/cat1_lawful_functors_package.rs` — confirms
   the parametric `Monoid (List a)` dictionary elaborates as a `Pi`-typed
   generic instance keyed by the bare `List` head (not a closed element
   type), confirms all four `Functor`/`Foldable` instances register under
   their class/head coherence keys, and confirms the package source cites
   the landed laws with no law field postulated.
