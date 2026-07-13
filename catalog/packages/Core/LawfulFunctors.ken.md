# `lawful-functors` — `Semigroup`, `Monoid`, `Functor`, `Foldable`

This package carries the **value-level algebra** classes `Semigroup`/`Monoid`
alongside the **constructor classes** `Functor`/`Foldable` over
`f : Type → Type`.

**Naming.** The value-level algebra companions belong here because
`Foldable`'s `fold_map` consumes a `Monoid`.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws  proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [References](#6-references)
7. [Trust  derivation](#7-trust--derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws  proofs](#4-laws--proofs)
- *Researcher* →
  [Laws  proofs](#4-laws--proofs) → [Design notes](#5-design-notes)

## 1. Motivation

Ken provides an associative operation (`Semigroup`), a monoid over it
(`Monoid`), a structure-preserving map over a
type constructor (`Functor`), and a fold that is coherent with a chosen
`Monoid` (`Foldable`) — the vocabulary every later constructor-class layer
(`Applicative`, `Monad`, `Traversable`, …) builds on. Every class here is an
ordinary package-level definition; no new kernel feature is required.

## 2. Definition

`Semigroup a` is an associative binary operation on `a`. `op` is the
ergonomic mappend operation, written as a plain identifier field. `assoc` is
a propositional equation in `Omega`.

```ken
class Semigroup a {
  op : a → a → a;
  assoc : (x : a) → (y : a) → (z : a) → Equal a (op (op x y) z) (op x (op y z))
}
```

`Monoid a` is a `Semigroup` with a two-sided identity `mempty`. It restates
the operation and associativity law: a `Monoid` yields a `Semigroup` by
forgetting `mempty`, `left_unit`, and `right_unit`, while keeping `op` and
`assoc`. This keeps the class records direct and independent.

```ken
class Monoid a {
  op : a → a → a;
  mempty : a;
  assoc : (x : a) → (y : a) → (z : a) → Equal a (op (op x y) z) (op x (op y z));
  left_unit : (x : a) → Equal a (op mempty x) x;
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
synthesized `C_instance_T` dictionary. `instance Monoid Bool` and
`instance Monoid (List a)` are the two worked
examples here: `Monoid_instance_Bool` restates the *same* `op`/`assoc`
definitions (`bool_and`/`bool_and::assoc`) that `Semigroup_instance_Bool` uses, and
`Monoid_instance_List`'s dictionary is genuinely generic in the element type
— it elaborates as `(a : Type) → Monoid (List a)`, so a caller applies it to
a concrete `a` before projecting a field, exactly like any other
parametric-head instance.

## 4. Laws  proofs

### 4.1 The `List` append monoid — the canonical inductive carrier

`op = list_append`, `mempty = Nil`. The three laws and the `Monoid` instance
are generic in `a`: `instance Monoid (List a)` elaborates as
`(a : Type) → Monoid (List a)`, and its law fields use generic list proofs
rather than re-proving them at a closed carrier.

Left unit is DEFINITIONAL: `list_append Nil x` iota-reduces to `x` by
append's first match arm, so the goal `Equal (List a) x x` stays `Eq`-shaped
over the neutral `x` — closed by `Refl`. Assoc is proved by induction on the
first list: base (`Nil`) both sides reduce to the NEUTRAL
`list_append a ys zs`, still `Eq`-shaped, `Refl`; step (`Cons h t`) lifts the
tail IH under `Cons a h` with `cong`. Right unit is proved by induction on
the list: base (`Nil`) both sides reduce to the CONSTRUCTOR `Nil a`, which
observationally collapses to `Top` (the same nullary constructor) — so
the goal is no longer `Eq`-shaped and `Refl` does not apply; it is
`Top`-introduced by `Proved` (the exact `Proved`-vs-`Refl` discrimination
constructor-headed endpoints give `Top` and `Proved`, while neutral endpoints
remain stuck `Eq` goals and use `Refl`; step
(`Cons h t`) is `cong` under `Cons a h` on the tail IH.

```ken
proof left_unit
for
list_append
(a : Type)
(xs : List
a) : Equal
(List
a)
(list_append
a
(Nil
a)
xs)
xs =
  Refl

proof assoc
for
list_append
(a : Type)
(xs : List
a)
(ys : List
a)
(zs : List
a) : Equal
(List
a)
(list_append
a
(list_append
a
xs
ys)
zs)
(list_append
a
xs
(list_append
a
ys
zs)) =
  match xs {
    Nil ↦ Refl;
    Cons h t ↦
      cong
        (List a)
        (List a)
        (list_append a (list_append a t ys) zs)
        (list_append a t (list_append a ys zs))
        (Cons a h)
        ((proof assoc for list_append) a t ys zs)
  }

proof right_unit
for
list_append
(a : Type)
(xs : List
a) : Equal
(List
a)
(list_append
a
xs
(Nil
a))
xs =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦
      cong
        (List a)
        (List a)
        (list_append a t (Nil a))
        t
        (Cons a h)
        ((proof right_unit for list_append) a t)
  }

instance Semigroup (List Nat) {
  op = list_append Nat;
  assoc = proof assoc for list_append Nat
}

instance Monoid (List a) {
  op = list_append a;
  mempty = Nil a;
  assoc = proof assoc for list_append a;
  left_unit = proof left_unit for list_append a;
  right_unit = proof right_unit for list_append a
}
```

### 4.2 The `Bool` conjunction monoid — the finite carrier

Complements the `List` monoid's inductive style with the FINITE case-split
style: no induction / no IH, every branch closes by `Proved` or `Refl` directly.
`bool_and` is a transparent match-based definition rather than a primitive.
It reduces on each concrete constructor, which lets the laws compute directly
even when an argument is initially symbolic.

Associativity is a full 2×2×2 case-split; each concrete branch reduces both
sides to the same literal, collapsing to `Top` → `Proved`. Left unit is
DEFINITIONAL: `bool_and True x` reduces to `x` (first match arm), goal
`Equal Bool x x` neutral → `Refl`. Right unit needs the case-split
(`bool_and x True` is stuck on symbolic `x`): each branch reduces to a
literal equal to itself → `Top` → `Proved`.

```ken
fn bool_and (p : Bool) (q : Bool) : Bool =
  match p {
    True ↦ q;
    False ↦ False
  }

proof assoc
for
bool_and
(x : Bool)
(y : Bool)
(z : Bool) : Equal
Bool
(bool_and
(bool_and
x
y)
z)
(bool_and
x
(bool_and
y
z)) =
  match x {
    True ↦
      match y {
        True ↦
          match z {
            True ↦ Proved;
            False ↦ Proved
          };
        False ↦
          match z {
            True ↦ Proved;
            False ↦ Proved
          }
      };
    False ↦
      match y {
        True ↦
          match z {
            True ↦ Proved;
            False ↦ Proved
          };
        False ↦
          match z {
            True ↦ Proved;
            False ↦ Proved
          }
      }
  }

proof left_unit for bool_and (x : Bool) : Equal Bool (bool_and True x) x = Refl

proof right_unit for bool_and (x : Bool) : Equal Bool (bool_and x True) x =
  match x {
    True ↦ Proved;
    False ↦ Proved
  }

instance Semigroup Bool {
  op = bool_and;
  assoc = proof assoc for bool_and
}

instance Monoid Bool {
  op = bool_and;
  mempty = True;
  assoc = proof assoc for bool_and;
  left_unit = proof left_unit for bool_and;
  right_unit = proof right_unit for bool_and
}
```

### 4.3 `Functor` — structure-preserving map over a type constructor

`class Functor (f : Type → Type)` takes a higher-kinded constructor parameter.
Its laws use a pointwise form: identity and fusion both quantify over
`(x : f a)`. `List`'s instance is proved by induction and `cong`; `Option`'s
instance closes by finite case split and definitional equality. `None`
collapses to `Top` and uses `Proved`, while `Some v` remains an `Eq`-shaped goal
and uses `Refl`.

```ken
fn idf (a : Type) (x : a) : a = x

fn comp (a : Type) (b : Type) (c : Type) (g : b → c) (h : a → b) (x : a) : c = g (h x)

class Functor (f : Type → Type) {
  map : (a : Type) → (b : Type) → (a → b) → f a → f b;
  id_law : (a : Type) → (x : f a) → Equal (f a) (map a a (idf a) x) x;
  fusion_law : (a : Type)
  → (b : Type)
  → (c : Type)
  → (g : b
  → c)
  → (h : a
  → b)
  → (x : f
  a)
  → Equal
  (f
  c)
  (map
  a
  c
  (comp
  a
  b
  c
  g
  h)
  x)
  (map
  b
  c
  g
  (map
  a
  b
  h
  x))
}

fn list_map (a : Type) (b : Type) (g : a → b) (xs : List a) : List b =
  match xs {
    Nil ↦ Nil b;
    Cons h t ↦
      Cons
        b
        (g h)
        (list_map a b g t)
  }

proof id
for
list_map
(a : Type)
(xs : List
a) : Equal
(List
a)
(list_map
a
a
(idf
a)
xs)
xs =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦
      cong
        (List a)
        (List a)
        (list_map a a (idf a) t)
        t
        (Cons a h)
        ((proof id for list_map) a t)
  }

proof fusion
for
list_map
(a : Type)
(b : Type)
(c : Type)
(g : b
→ c)
(h : a
→ b)
(xs : List
a) : Equal
(List
c)
(list_map
a
c
(comp
a
b
c
g
h)
xs)
(list_map
b
c
g
(list_map
a
b
h
xs)) =
  match xs {
    Nil ↦ Proved;
    Cons x rest ↦
      cong
        (List c)
        (List c)
        (list_map a c (comp a b c g h) rest)
        (list_map b c g (list_map a b h rest))
        (Cons c (g (h x)))
        ((proof fusion for list_map) a b c g h rest)
  }

fn option_map (a : Type) (b : Type) (g : a → b) (x : Option a) : Option b =
  match x {
    None ↦ None b;
    Some v ↦
      Some
        b
        (g v)
  }

proof id
for
option_map
(a : Type)
(x : Option
a) : Equal
(Option
a)
(option_map
a
a
(idf
a)
x)
x =
  match x {
    None ↦ Proved;
    Some v ↦ Refl
  }

proof fusion
for
option_map
(a : Type)
(b : Type)
(c : Type)
(g : b
→ c)
(h : a
→ b)
(x : Option
a) : Equal
(Option
c)
(option_map
a
c
(comp
a
b
c
g
h)
x)
(option_map
b
c
g
(option_map
a
b
h
x)) =
  match x {
    None ↦ Proved;
    Some v ↦ Refl
  }

instance Functor List {
  map = list_map;
  id_law = proof id for list_map;
  fusion_law = proof fusion for list_map
}

instance Functor Option {
  map = option_map;
  id_law = proof id for option_map;
  fusion_law = proof fusion for option_map
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

fn fold_map_step
(a : Type)
(m : Type)
(mon : Monoid
m)
(g : a
→ m)
(y : a)
(acc : m) : m =
  mon.op (g y) acc

class Foldable (f : Type → Type) {
  foldr : (a : Type) → (b : Type) → (a → b → b) → b → f a → b;
  fold_map : (a : Type) → (m : Type) → Monoid m → (a → m) → f a → m;
  to_list : (a : Type) → f a → List a;
  foldr_to_list : (a : Type)
  → (x : f
  a)
  → Equal
  (List
  a)
  (foldr
  a
  (List
  a)
  (Cons
  a)
  (Nil
  a)
  x)
  (to_list
  a
  x);
  fold_map_coherence : (a : Type)
  → (m : Type)
  → (mon : Monoid
  m)
  → (g : a
  → m)
  → (x : f
  a)
  → Equal
  m
  (fold_map
  a
  m
  mon
  g
  x)
  (foldr
  a
  m
  (fold_map_step
  a
  m
  mon
  g)
  (monoid_mempty
  m
  mon)
  x)
}

fn list_foldr (a : Type) (b : Type) (k : a → b → b) (z : b) (xs : List a) : b =
  match xs {
    Nil ↦ z;
    Cons h t ↦
      k
        h
        (list_foldr a b k z t)
  }

fn list_fold_map (a : Type) (m : Type) (mon : Monoid m) (g : a → m) (xs : List a) : m =
  match xs {
    Nil ↦ mon.mempty;
    Cons h t ↦
      mon.op
        (g h)
        (list_fold_map a m mon g t)
  }

fn list_to_list (a : Type) (xs : List a) : List a = xs

lemma list_foldr_to_list
(a : Type)
(xs : List
a) : Equal
(List
a)
(list_foldr
a
(List
a)
(Cons
a)
(Nil
a)
xs)
(list_to_list
a
xs) =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦
      cong
        (List a)
        (List a)
        (list_foldr a (List a) (Cons a) (Nil a) t)
        (list_to_list a t)
        (Cons a h)
        (list_foldr_to_list a t)
  }

lemma list_fold_map_coherence
(a : Type)
(m : Type)
(mon : Monoid
m)
(g : a
→ m)
(xs : List
a) : Equal
m
(list_fold_map
a
m
mon
g
xs)
(list_foldr
a
m
(fold_map_step
a
m
mon
g)
(monoid_mempty
m
mon)
xs) =
  match xs {
    Nil ↦ Refl;
    Cons h t ↦
      cong
        m
        m
        (list_fold_map a m mon g t)
        (list_foldr a m (fold_map_step a m mon g) (monoid_mempty m mon) t)
        (mon.op (g h))
        (list_fold_map_coherence a m mon g t)
  }

fn option_foldr (a : Type) (b : Type) (k : a → b → b) (z : b) (x : Option a) : b =
  match x {
    None ↦ z;
    Some v ↦
      k
        v
        z
  }

fn option_fold_map
(a : Type)
(m : Type)
(mon : Monoid
m)
(g : a
→ m)
(x : Option
a) : m =
  match x {
    None ↦ mon.mempty;
    Some v ↦
      mon.op
        (g v)
        mon.mempty
  }

fn option_to_list (a : Type) (x : Option a) : List a =
  option_foldr a (List a) (Cons a) (Nil a) x

lemma option_foldr_to_list
(a : Type)
(x : Option
a) : Equal
(List
a)
(option_foldr
a
(List
a)
(Cons
a)
(Nil
a)
x)
(option_to_list
a
x) =
  Refl

lemma option_fold_map_coherence
(a : Type)
(m : Type)
(mon : Monoid
m)
(g : a
→ m)
(x : Option
a) : Equal
m
(option_fold_map
a
m
mon
g
x)
(option_foldr
a
m
(fold_map_step
a
m
mon
g)
(monoid_mempty
m
mon)
x) =
  match x {
    None ↦ Refl;
    Some v ↦ Refl
  }

instance Foldable List {
  foldr = list_foldr;
  fold_map = list_fold_map;
  to_list = list_to_list;
  foldr_to_list = list_foldr_to_list;
  fold_map_coherence = list_fold_map_coherence
}

instance Foldable Option {
  foldr = option_foldr;
  fold_map = option_fold_map;
  to_list = option_to_list;
  foldr_to_list = option_foldr_to_list;
  fold_map_coherence = option_fold_map_coherence
}
```

## 5. Design notes

**No new kernel feature.** A class is a record and a law is an `Omega`
proposition. `Semigroup` and `Monoid` operations return values of their
carrier, so their laws use propositional equality `Equal a u v : Omega`
directly.

**Laws are proved over ordinary carriers.** The `List a` append monoid and
`List` `Functor`/`Foldable` laws use induction and `cong`; the corresponding
`Bool` and `Option` laws use finite case splits and definitional equality.
Every instance has a zero `trusted_base()` delta.

**Shared building blocks.** `cong`, `sym`, and `trans` support inductive
congruence steps, while `list_append` supplies the `List` monoid operation.
Reusing these names keeps the common operations and proof idioms consistent.

**Higher-kinded and parametric declarations.** `Functor` and `Foldable` take
a constructor parameter `f : Type → Type`, while `instance Monoid (List a)`
is generic in `a` and produces a dictionary of type
`(a : Type) → Monoid (List a)`. These forms let the classes describe a whole
family of carriers without duplicating declarations for each element type.

## 6. References

None — this entry's design (the `Semigroup`/`Monoid` restatement pattern,
the higher-kinded constructor classes) is Ken-native, not consulted from an
external reference implementation.

## 7. Trust  derivation

1. **Public API.** `class Semigroup`, `class Monoid`,
   `instance Semigroup (List Nat)`, `instance Monoid (List a)`,
   `instance Semigroup Bool`, `instance Monoid Bool`, `class Functor`,
   `instance Functor List`, `instance Functor Option`, `class Foldable`,
   `instance Foldable List`, `instance Foldable Option`.
2. **Source map.**

   | Task | Section |
   |---|---|
   | See the four classes | [Definition](#2-definition), [Laws  proofs](#4-laws--proofs) |
   | Project a field off a dictionary | [Using it](#3-using-it) |
   | The `List`/`Bool`/`Option` proofs | [Laws  proofs](#4-laws--proofs) |
   | Why value-level laws use `Equal`, and how generic declarations work | [Design notes](#5-design-notes) |

3. **Derivation path.** All four classes are record declarations built from
   `Equal`, `Omega`, and record machinery. Every instance reduces through
   ordinary induction or finite case splitting, closing with `Proved`, `Refl`,
   or `cong`; no `Axiom` is needed.
4. **`trusted_base()` delta.** **Zero.** Every instance in this package —
   `List`, `Bool`, and `Option` alike — is a real, kernel-checked proof; no
   law field is postulated anywhere.
5. **Proof families.** `List` instances: structural induction + `cong`
   lifting the tail IH under the head constructor (`§4.1`, `§4.3`, `§4.4`).
   `Bool` instances: full finite case-split, every branch closing by `Proved` or
   `Refl` directly, no IH (`§4.2`). `Option` instances: single-level
   case-split / definitional equality (`§4.3`, `§4.4`).
6. **Consumers.** `Functor`, `Foldable`, and `Monoid`-using packages build on
   these class declarations and instances.
7. **Validation evidence.** The catalog checks the parametric
   `Monoid (List a)` dictionary, all four `Functor`/`Foldable` instance
   registrations, and the absence of postulated law fields.
