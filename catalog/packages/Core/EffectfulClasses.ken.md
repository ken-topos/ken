# `Applicative`, `Monad`, and `Traversable` — effectful constructor classes

`Applicative` and `Monad` are the two constructor classes every effectful
computation over a container (`List`, `Option`, and — separately, by
attested correspondence, not a surface instance — the interaction-tree
effect denotation `ITree`) ultimately builds on: `Applicative` sequences
independent effectful values, `Monad` sequences effectful values where
later steps depend on earlier results. This entry lands both classes,
proves them lawfully for `List` and `Option`, and attests the third,
already-landed instance without minting a duplicate. `§9` extends the
family with `Traversable`, the class that walks a container while
threading an arbitrary `Applicative` effect — the last Core item of the
`Functor → Applicative → Monad → Traversable` toolkit chain (kept in this
one entry per judgment call `L1`, rather than a new file).

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws  proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [Findings](#6-findings)
7. [References](#7-references)
8. [Trust  derivation](#8-trust--derivation)
9. [`Traversable`](#9-traversable)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws  proofs](#4-laws--proofs)
- *Researcher* → [Laws  proofs](#4-laws--proofs) →
  [Design notes](#5-design-notes)
- *Porting from Haskell/Lean/Agda* → [Design notes](#5-design-notes)

## 1. Motivation

`class Functor` (`Core/LawfulFunctors.ken`) lets a container be mapped
over, but `map` alone cannot combine two INDEPENDENT effectful values
(there is no way to apply a function living inside the container to an
argument also living inside the container) or sequence a computation whose
NEXT step depends on a PREVIOUS result. `Applicative` adds `pure` (lift a
value in, effect-free) and `ap` (apply a contained function to a contained
argument); `Monad` adds `bind` (sequence, with the next step allowed to
depend on the previous result). Both are proved lawfully here for `List`
(the cartesian/list-monad reading — every combination of elements) and
`Option` (short-circuiting on `None`), and reconciled — without minting a
second implementation — with the landed interaction-tree `bind` that
already denotes every effect row in the language.

## 2. Definition

### 2.1 The wired superclass chain

`Applicative f` carries an explicit `functor : Functor f` field, and
`Monad f` carries an explicit `applicative : Applicative f` field — the
already-built superclass dictionary, supplied WHOLE at each instance, not
restated. This is a real capability confirmed directly against the landed
elaborator: a class field typed as another class applied to the same
parameter (`functor : Functor f`) elaborates exactly like any other field,
nested projection (`d.applicative.functor.map`) composes cleanly through
`infer_proj`, and an instance supplying an already-built dictionary as a
field checks its VALUE against the field's expected TYPE — so `instance
Monad List` below reuses `Applicative List`'s six already-proved laws
verbatim; only `bind` and its three laws are new.

`ap_id`/`ap_hom`/`ap_ich`/`ap_cmp`/`map_coh`/`bind_lid`/`bind_rid`/
`bind_asc` are all `Ω`-classified value equations (`Equal (f _) u v`), one
canonical field per law, matched character-for-character against
`spec/50-stdlib/56-effectful-classes.md` `§3.2`/`§4.2`:

```ken
fn applyTo (a : Type) (b : Type) (y : a) (g : a -> b) : b = g y

fn compose (a : Type) (b : Type) (c : Type) (g : b -> c) (h : a -> b) (x : a) : c = g (h x)

fn functorMapOf (g_ty : Type -> Type) (d : Functor g_ty) (a : Type) (b : Type) (h : a -> b) (x : g_ty a) : g_ty b = d.map a b h x

class Applicative (f : Type -> Type) {
  functor : Functor f ;
  pure : (a : Type) -> a -> f a ;
  ap : (a : Type) -> (b : Type) -> f (a -> b) -> f a -> f b ;
  ap_id : (a : Type) -> (v : f a) -> Equal (f a) (ap a a (pure (a -> a) (idf a)) v) v ;
  ap_hom : (a : Type) -> (b : Type) -> (g : a -> b) -> (x : a) -> Equal (f b) (ap a b (pure (a -> b) g) (pure a x)) (pure b (g x)) ;
  ap_ich : (a : Type) -> (b : Type) -> (u : f (a -> b)) -> (y : a) -> Equal (f b) (ap a b u (pure a y)) (ap (a -> b) b (pure ((a -> b) -> b) (applyTo a b y)) u) ;
  ap_cmp : (a : Type) -> (b : Type) -> (c : Type) -> (u : f (b -> c)) -> (v : f (a -> b)) -> (w : f a) ->
    Equal (f c)
      (ap a c (ap (a -> b) (a -> c) (ap (b -> c) ((a -> b) -> (a -> c)) (pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
      (ap b c u (ap a b v w)) ;
  map_coh : (a : Type) -> (b : Type) -> (g : a -> b) -> (x : f a) -> Equal (f b) (functorMapOf f functor a b g x) (ap a b (pure (a -> b) g) x)
}

fn applicativePureOf (g_ty : Type -> Type) (d : Applicative g_ty) (a : Type) (x : a) : g_ty a = d.pure a x

fn composeKleisli (g_ty : Type -> Type) (bindfn : (a : Type) -> (b : Type) -> g_ty a -> (a -> g_ty b) -> g_ty b)
  (a : Type) (b : Type) (c : Type) (k : a -> g_ty b) (h : b -> g_ty c) (x : a) : g_ty c = bindfn b c (k x) h

class Monad (f : Type -> Type) {
  applicative : Applicative f ;
  bind : (a : Type) -> (b : Type) -> f a -> (a -> f b) -> f b ;
  bind_lid : (a : Type) -> (b : Type) -> (x : a) -> (k : a -> f b) -> Equal (f b) (bind a b (applicativePureOf f applicative a x) k) (k x) ;
  bind_rid : (a : Type) -> (m : f a) -> Equal (f a) (bind a a m (applicativePureOf f applicative a)) m ;
  bind_asc : (a : Type) -> (b : Type) -> (c : Type) -> (m : f a) -> (k : a -> f b) -> (h : b -> f c) ->
    Equal (f c) (bind b c (bind a b m k) h) (bind a c m (composeKleisli f bind a b c k h))
}
```

`functorMapOf`/`applicativePureOf`/`composeKleisli` exist because a
`.field` projection and a bare `λ` both fail to parse inside a `fn`'s own
declared TYPE (a genuine landed grammar gap, `§6` Finding) — `map_coh`
needs `functor`'s `map` field and `bind_lid`/`bind_rid` need
`applicative`'s `pure` field and `bind_asc` needs a Kleisli-composed
`bind` INSIDE a law's TYPE, not just its proof body, so each is routed
through a named accessor taking the dictionary/function explicitly
instead of projecting or abstracting inline.

### 2.2 `Option` — finite case-split, zero induction

`pure = Some`; `ap` short-circuits to `None` on either side; `bind (Some
x) k = k x`, `bind None k = None`. Every law closes by direct case
analysis — no recursion, since `Option` has no recursive structure:

```ken
fn option_pure (a : Type) (x : a) : Option a = Some a x

fn option_ap (a : Type) (b : Type) (mf : Option (a -> b)) (mx : Option a) : Option b =
  match mf { None ⇒ None b ; Some g ⇒ match mx { None ⇒ None b ; Some x ⇒ Some b (g x) } }

fn option_bind (a : Type) (b : Type) (m : Option a) (k : a -> Option b) : Option b =
  match m { None ⇒ None b ; Some x ⇒ k x }
```

For an ABSTRACT type parameter, a `Some a x` endpoint (`x` itself
abstract) is STUCK, not collapsed — `Refl`, not `tt` (`§5`). Inlining a
proof directly in a self-recursive match arm also repeatedly hit a kernel
`TypeMismatch` a dispatched version did not (`§5`) — every law below
proves each branch as its own top-level, directly-ascribed lemma, then
dispatches via a thin outer `match`:

```ken
const option_ap_id_none : (a : Type) -> Equal (Option a) (option_ap a a (option_pure (a -> a) (idf a)) (None a)) (None a) = λa. tt

fn option_ap_id_some (a : Type) (x : a) : Equal (Option a) (option_ap a a (option_pure (a -> a) (idf a)) (Some a x)) (Some a x) = Refl

fn option_ap_id (a : Type) (v : Option a) : Equal (Option a) (option_ap a a (option_pure (a -> a) (idf a)) v) v =
  match v { None ⇒ option_ap_id_none a ; Some x ⇒ option_ap_id_some a x }

fn option_ap_hom (a : Type) (b : Type) (g : a -> b) (x : a) : Equal (Option b) (option_ap a b (option_pure (a -> b) g) (option_pure a x)) (option_pure b (g x)) = Refl

fn option_ap_ich_none (a : Type) (b : Type) (y : a) :
  Equal (Option b) (option_ap a b (None (a -> b)) (option_pure a y)) (option_ap (a -> b) b (option_pure ((a -> b) -> b) (applyTo a b y)) (None (a -> b))) = tt

fn option_ap_ich_some (a : Type) (b : Type) (g : a -> b) (y : a) :
  Equal (Option b) (option_ap a b (Some (a -> b) g) (option_pure a y)) (option_ap (a -> b) b (option_pure ((a -> b) -> b) (applyTo a b y)) (Some (a -> b) g)) = Refl

fn option_ap_ich (a : Type) (b : Type) (u : Option (a -> b)) (y : a) :
  Equal (Option b) (option_ap a b u (option_pure a y)) (option_ap (a -> b) b (option_pure ((a -> b) -> b) (applyTo a b y)) u) =
  match u { None ⇒ option_ap_ich_none a b y ; Some g ⇒ option_ap_ich_some a b g y }

fn option_ap_cmp_none_u (a : Type) (b : Type) (c : Type) (v : Option (a -> b)) (w : Option a) :
  Equal (Option c)
    (option_ap a c (option_ap (a -> b) (a -> c) (option_ap (b -> c) ((a -> b) -> (a -> c)) (option_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) (None (b -> c))) v) w)
    (option_ap b c (None (b -> c)) (option_ap a b v w)) = tt

fn option_ap_cmp_some_u_none_v (a : Type) (b : Type) (c : Type) (g : b -> c) (w : Option a) :
  Equal (Option c)
    (option_ap a c (option_ap (a -> b) (a -> c) (option_ap (b -> c) ((a -> b) -> (a -> c)) (option_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) (Some (b -> c) g)) (None (a -> b))) w)
    (option_ap b c (Some (b -> c) g) (option_ap a b (None (a -> b)) w)) = tt

fn option_ap_cmp_some_u_some_v_none_w (a : Type) (b : Type) (c : Type) (g : b -> c) (h : a -> b) :
  Equal (Option c)
    (option_ap a c (option_ap (a -> b) (a -> c) (option_ap (b -> c) ((a -> b) -> (a -> c)) (option_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) (Some (b -> c) g)) (Some (a -> b) h)) (None a))
    (option_ap b c (Some (b -> c) g) (option_ap a b (Some (a -> b) h) (None a))) = tt

fn option_ap_cmp_all_some (a : Type) (b : Type) (c : Type) (g : b -> c) (h : a -> b) (x : a) :
  Equal (Option c)
    (option_ap a c (option_ap (a -> b) (a -> c) (option_ap (b -> c) ((a -> b) -> (a -> c)) (option_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) (Some (b -> c) g)) (Some (a -> b) h)) (Some a x))
    (option_ap b c (Some (b -> c) g) (option_ap a b (Some (a -> b) h) (Some a x))) = Refl

fn option_ap_cmp (a : Type) (b : Type) (c : Type) (u : Option (b -> c)) (v : Option (a -> b)) (w : Option a) :
  Equal (Option c)
    (option_ap a c (option_ap (a -> b) (a -> c) (option_ap (b -> c) ((a -> b) -> (a -> c)) (option_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
    (option_ap b c u (option_ap a b v w)) =
  match u {
    None ⇒ option_ap_cmp_none_u a b c v w ;
    Some g ⇒ match v {
      None ⇒ option_ap_cmp_some_u_none_v a b c g w ;
      Some h ⇒ match w {
        None ⇒ option_ap_cmp_some_u_some_v_none_w a b c g h ;
        Some x ⇒ option_ap_cmp_all_some a b c g h x
      }
    }
  }

fn option_map_coh_none (a : Type) (b : Type) (g : a -> b) :
  Equal (Option b) (functorMapOf Option Functor_instance_Option a b g (None a)) (option_ap a b (option_pure (a -> b) g) (None a)) = tt

fn option_map_coh_some (a : Type) (b : Type) (g : a -> b) (v : a) :
  Equal (Option b) (functorMapOf Option Functor_instance_Option a b g (Some a v)) (option_ap a b (option_pure (a -> b) g) (Some a v)) = Refl

fn option_map_coh (a : Type) (b : Type) (g : a -> b) (x : Option a) :
  Equal (Option b) (functorMapOf Option Functor_instance_Option a b g x) (option_ap a b (option_pure (a -> b) g) x) =
  match x { None ⇒ option_map_coh_none a b g ; Some v ⇒ option_map_coh_some a b g v }

instance Applicative Option {
  functor = Functor_instance_Option ;
  pure = option_pure ;
  ap = option_ap ;
  ap_id = option_ap_id ;
  ap_hom = option_ap_hom ;
  ap_ich = option_ap_ich ;
  ap_cmp = option_ap_cmp ;
  map_coh = option_map_coh
}
```

`instance Applicative Option` is declared here, immediately after its own
laws, because `Monad Option`'s `bind_lid`/`bind_rid` (next) reference the
now-real `Applicative_instance_Option` dictionary directly (the WIRE
mechanism, `§2.1`) — every `instance C T { ... }` registers a real global
`C_instance_T`, not just a `where`-resolved implicit dictionary.

```ken
fn option_bind_lid (a : Type) (b : Type) (x : a) (k : a -> Option b) :
  Equal (Option b) (option_bind a b (applicativePureOf Option Applicative_instance_Option a x) k) (k x) = Refl

const option_bind_rid_none : (a : Type) ->
  Equal (Option a) (option_bind a a (None a) (applicativePureOf Option Applicative_instance_Option a)) (None a) = λa. tt

fn option_bind_rid_some (a : Type) (x : a) :
  Equal (Option a) (option_bind a a (Some a x) (applicativePureOf Option Applicative_instance_Option a)) (Some a x) = Refl

fn option_bind_rid (a : Type) (m : Option a) : Equal (Option a) (option_bind a a m (applicativePureOf Option Applicative_instance_Option a)) m =
  match m { None ⇒ option_bind_rid_none a ; Some x ⇒ option_bind_rid_some a x }

fn option_bind_asc_none (a : Type) (b : Type) (c : Type) (k : a -> Option b) (h : b -> Option c) :
  Equal (Option c) (option_bind b c (option_bind a b (None a) k) h) (option_bind a c (None a) (composeKleisli Option option_bind a b c k h)) = tt

fn option_bind_asc_some (a : Type) (b : Type) (c : Type) (x : a) (k : a -> Option b) (h : b -> Option c) :
  Equal (Option c) (option_bind b c (option_bind a b (Some a x) k) h) (option_bind a c (Some a x) (composeKleisli Option option_bind a b c k h)) = Refl

fn option_bind_asc (a : Type) (b : Type) (c : Type) (m : Option a) (k : a -> Option b) (h : b -> Option c) :
  Equal (Option c) (option_bind b c (option_bind a b m k) h) (option_bind a c m (composeKleisli Option option_bind a b c k h)) =
  match m { None ⇒ option_bind_asc_none a b c k h ; Some x ⇒ option_bind_asc_some a b c x k h }

instance Monad Option {
  applicative = Applicative_instance_Option ;
  bind = option_bind ;
  bind_lid = option_bind_lid ;
  bind_rid = option_bind_rid ;
  bind_asc = option_bind_asc
}
```

### 2.3 `List` — the cartesian instance, induction throughout

`pure x = [x]`; `ap` is the cartesian product-of-effects; `bind = concatMap`
(chapter `§3.3`/`§4.4`, Fork D — the shape coherent with `Monad List`).
`concatMap` itself is not landed anywhere in the catalog today (`§6`
Finding) — inlined here, a straightforward structural recursion off the
landed `list_append`:

```ken
fn concatMap (a : Type) (b : Type) (f : a -> List b) (xs : List a) : List b =
  match xs { Nil ⇒ Nil b ; Cons h t ⇒ list_append b (f h) (concatMap a b f t) }

fn list_pure (a : Type) (x : a) : List a = Cons a x (Nil a)

fn list_ap (a : Type) (b : Type) (mf : List (a -> b)) (mx : List a) : List b =
  concatMap (a -> b) b (λg. list_map a b g mx) mf

fn list_bind (a : Type) (b : Type) (m : List a) (k : a -> List b) : List b = concatMap a b k m
```

`list_bind` exists because `concatMap`'s own natural argument order
(function first, matching its `foldr`-shaped recursion) is the OPPOSITE
of `bind`'s field order (container first, per the chapter's own `bind m k
= concatMap k m`) — a real argument-order mismatch, not a cosmetic one
(`§6` Finding).

`bind_lid`/`bind_rid`/`bind_asc` for `List`:

```ken
fn list_bind_lid (a : Type) (b : Type) (x : a) (k : a -> List b) :
  Equal (List b) (list_bind a b (list_pure a x) k) (k x) = list_right_unit b (k x)

fn list_bind_rid (a : Type) (m : List a) : Equal (List a) (list_bind a a m (list_pure a)) m =
  match m {
    Nil ⇒ tt ;
    Cons h t ⇒ cong (List a) (List a) (list_bind a a t (list_pure a)) t (Cons a h) (list_bind_rid a t)
  }

fn concatMap_append_distrib (a : Type) (b : Type) (f : a -> List b) (xs : List a) (ys : List a) :
  Equal (List b) (concatMap a b f (list_append a xs ys)) (list_append b (concatMap a b f xs) (concatMap a b f ys)) =
  match xs {
    Nil ⇒ Refl ;
    Cons h t ⇒
      trans (List b)
        (list_append b (f h) (concatMap a b f (list_append a t ys)))
        (list_append b (f h) (list_append b (concatMap a b f t) (concatMap a b f ys)))
        (list_append b (list_append b (f h) (concatMap a b f t)) (concatMap a b f ys))
        (cong (List b) (List b) (concatMap a b f (list_append a t ys)) (list_append b (concatMap a b f t) (concatMap a b f ys)) (list_append b (f h)) (concatMap_append_distrib a b f t ys))
        (sym (List b) (list_append b (list_append b (f h) (concatMap a b f t)) (concatMap a b f ys)) (list_append b (f h) (list_append b (concatMap a b f t) (concatMap a b f ys))) (list_assoc b (f h) (concatMap a b f t) (concatMap a b f ys)))
  }

fn list_bind_asc (a : Type) (b : Type) (c : Type) (m : List a) (k : a -> List b) (h : b -> List c) :
  Equal (List c) (list_bind b c (list_bind a b m k) h) (list_bind a c m (composeKleisli List list_bind a b c k h)) =
  match m {
    Nil ⇒ tt ;
    Cons h0 t ⇒
      trans (List c)
        (concatMap b c h (list_append b (k h0) (concatMap a b k t)))
        (list_append c (concatMap b c h (k h0)) (concatMap b c h (concatMap a b k t)))
        (list_append c (composeKleisli List list_bind a b c k h h0) (list_bind a c t (composeKleisli List list_bind a b c k h)))
        (concatMap_append_distrib b c h (k h0) (concatMap a b k t))
        (cong (List c) (List c) (concatMap b c h (concatMap a b k t)) (list_bind a c t (composeKleisli List list_bind a b c k h)) (list_append c (concatMap b c h (k h0))) (list_bind_asc a b c t k h))
  }
```

`ap_id`/`ap_hom`/`map_coh` for `List` compose with the ALREADY-LANDED
`list_right_unit`/`list_functor_id` (`Core/LawfulFunctors.ken`) — zero new
induction needed for any of the three:

```ken
fn list_ap_id (a : Type) (v : List a) : Equal (List a) (list_ap a a (list_pure (a -> a) (idf a)) v) v =
  trans (List a)
    (list_append a (list_map a a (idf a) v) (Nil a))
    (list_map a a (idf a) v)
    v
    (list_right_unit a (list_map a a (idf a) v))
    (list_functor_id a v)

fn list_ap_hom (a : Type) (b : Type) (g : a -> b) (x : a) :
  Equal (List b) (list_ap a b (list_pure (a -> b) g) (list_pure a x)) (list_pure b (g x)) =
  list_right_unit b (Cons b (g x) (Nil b))

fn list_map_coh (a : Type) (b : Type) (g : a -> b) (x : List a) :
  Equal (List b) (functorMapOf List Functor_instance_List a b g x) (list_ap a b (list_pure (a -> b) g) x) =
  sym (List b) (list_append b (list_map a b g x) (Nil b)) (list_map a b g x) (list_right_unit b (list_map a b g x))
```

`ap_ich` needs one real induction. A self-recursive proof stated directly
via `list_ap` does not typecheck at the recursive call (`list_ap`'s own
unfolding to a `list_map` form needs `list_right_unit`, a PROOF, not a raw
reduction, so the induction hypothesis's type does not definitionally
match what `cong` needs at each step) — split into the true inductive
content, phrased directly over `concatMap`/`list_map`, and the outer
`list_ap`-phrased lemma composing it with the one `list_right_unit` step:

```ken
fn list_ap_inner (a : Type) (b : Type) (y : a) (g : a -> b) : List b = list_map a b g (list_pure a y)

fn list_ap_ich_general (a : Type) (b : Type) (u : List (a -> b)) (y : a) :
  Equal (List b) (concatMap (a -> b) b (list_ap_inner a b y) u) (list_map (a -> b) b (applyTo a b y) u) =
  match u {
    Nil ⇒ tt ;
    Cons g0 t ⇒
      cong (List b) (List b)
        (concatMap (a -> b) b (list_ap_inner a b y) t)
        (list_map (a -> b) b (applyTo a b y) t)
        (Cons b (g0 y))
        (list_ap_ich_general a b t y)
  }

fn list_ap_ich (a : Type) (b : Type) (u : List (a -> b)) (y : a) :
  Equal (List b) (list_ap a b u (list_pure a y)) (list_ap (a -> b) b (list_pure ((a -> b) -> b) (applyTo a b y)) u) =
  trans (List b)
    (concatMap (a -> b) b (list_ap_inner a b y) u)
    (list_map (a -> b) b (applyTo a b y) u)
    (list_append b (list_map (a -> b) b (applyTo a b y) u) (Nil b))
    (list_ap_ich_general a b u y)
    (sym (List b) (list_append b (list_map (a -> b) b (applyTo a b y) u) (Nil b)) (list_map (a -> b) b (applyTo a b y) u) (list_right_unit b (list_map (a -> b) b (applyTo a b y) u)))
```

`ap_cmp` (composition) is the load-bearing law of the four — the standard
"every combination of three lists" associativity fact. `pure f`'s own
`ap` reduces to a plain `list_map` (a fact worth its own name,
`list_ap_pure_left`, since it generalizes both `ap_hom` and the front of
`ap_cmp`); the rest is three "fusion" facts relating `concatMap`/`list_map`
composition, plus the already-proved `list_bind_asc` for the one genuinely
new inductive step (concatMap-after-concatMap):

```ken
fn list_ap_pure_left (a : Type) (b : Type) (g : a -> b) (xs : List a) :
  Equal (List b) (list_ap a b (list_pure (a -> b) g) xs) (list_map a b g xs) =
  list_right_unit b (list_map a b g xs)

fn list_map_append_distrib (a : Type) (b : Type) (g : a -> b) (xs : List a) (ys : List a) :
  Equal (List b) (list_map a b g (list_append a xs ys)) (list_append b (list_map a b g xs) (list_map a b g ys)) =
  match xs {
    Nil ⇒ Refl ;
    Cons h t ⇒ cong (List b) (List b) (list_map a b g (list_append a t ys)) (list_append b (list_map a b g t) (list_map a b g ys)) (Cons b (g h)) (list_map_append_distrib a b g t ys)
  }

fn composeFG (a : Type) (b : Type) (c : Type) (f : b -> List c) (g : a -> b) (x : a) : List c = f (g x)

fn concatMap_map_fusion (a : Type) (b : Type) (c : Type) (f : b -> List c) (g : a -> b) (xs : List a) :
  Equal (List c) (concatMap b c f (list_map a b g xs)) (concatMap a c (composeFG a b c f g) xs) =
  match xs {
    Nil ⇒ tt ;
    Cons h t ⇒ cong (List c) (List c) (concatMap b c f (list_map a b g t)) (concatMap a c (composeFG a b c f g) t) (list_append c (f (g h))) (concatMap_map_fusion a b c f g t)
  }

fn mapAfter (a : Type) (b : Type) (c : Type) (g : b -> c) (f : a -> List b) (x : a) : List c = list_map b c g (f x)

fn list_map_concatMap_fusion (a : Type) (b : Type) (c : Type) (g : b -> c) (f : a -> List b) (xs : List a) :
  Equal (List c) (list_map b c g (concatMap a b f xs)) (concatMap a c (mapAfter a b c g f) xs) =
  match xs {
    Nil ⇒ tt ;
    Cons h t ⇒
      trans (List c)
        (list_map b c g (list_append b (f h) (concatMap a b f t)))
        (list_append c (list_map b c g (f h)) (list_map b c g (concatMap a b f t)))
        (list_append c (mapAfter a b c g f h) (concatMap a c (mapAfter a b c g f) t))
        (list_map_append_distrib b c g (f h) (concatMap a b f t))
        (cong (List c) (List c) (list_map b c g (concatMap a b f t)) (concatMap a c (mapAfter a b c g f) t) (list_append c (list_map b c g (f h))) (list_map_concatMap_fusion a b c g f t))
  }

fn concatMap_pointwise_eq (a : Type) (b : Type) (f : a -> List b) (g : a -> List b) (pf : (x : a) -> Equal (List b) (f x) (g x)) (xs : List a) :
  Equal (List b) (concatMap a b f xs) (concatMap a b g xs) =
  match xs {
    Nil ⇒ tt ;
    Cons h t ⇒
      trans (List b)
        (list_append b (f h) (concatMap a b f t))
        (list_append b (g h) (concatMap a b f t))
        (list_append b (g h) (concatMap a b g t))
        (cong (List b) (List b) (f h) (g h) (λz. list_append b z (concatMap a b f t)) (pf h))
        (cong (List b) (List b) (concatMap a b f t) (concatMap a b g t) (list_append b (g h)) (concatMap_pointwise_eq a b f g pf t))
  }
```

Assembling `ap_cmp` itself needs three more named accessors (again, the
lambda/`.field`-in-declared-type gap — `§6` Finding) and a three-part
`trans` chain: the FRONT (unfold `pure(compose)` via `list_ap_pure_left`,
lift through the outer `ap` via `cong`, fuse via `concatMap_map_fusion`),
the MIDDLE (`list_bind_asc` for the outer `concatMap`-after-`concatMap`,
`concatMap_map_fusion` again for the inner one), and the END (an inductive
reconciliation of the two remaining `concatMap`/`list_map` orderings,
needing `list_functor_fusion` — already landed — plus
`list_map_append_distrib`):

```ken
fn apMapV (a : Type) (b : Type) (c : Type) (v : List (a -> b)) (g1 : b -> c) : List (a -> c) = list_map (a -> b) (a -> c) (compose a b c g1) v

fn apMapW (a : Type) (c : Type) (w : List a) (h2 : a -> c) : List c = list_map a c h2 w

fn list_ap_cmp_front (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) :
  Equal (List (a -> c))
    (list_ap (a -> b) (a -> c) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v)
    (concatMap (b -> c) (a -> c) (apMapV a b c v) u) =
  trans (List (a -> c))
    (concatMap ((a -> b) -> (a -> c)) (a -> c) (λh. list_map (a -> b) (a -> c) h v) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u))
    (concatMap ((a -> b) -> (a -> c)) (a -> c) (λh. list_map (a -> b) (a -> c) h v) (list_map (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u))
    (concatMap (b -> c) (a -> c) (λg1. list_map (a -> b) (a -> c) (compose a b c g1) v) u)
    (cong (List ((a -> b) -> (a -> c))) (List (a -> c))
       (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u)
       (list_map (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u)
       (λp. concatMap ((a -> b) -> (a -> c)) (a -> c) (λh. list_map (a -> b) (a -> c) h v) p)
       (list_ap_pure_left (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u))
    (concatMap_map_fusion (b -> c) ((a -> b) -> (a -> c)) (a -> c) (λh. list_map (a -> b) (a -> c) h v) (compose a b c) u)

fn apCompH1 (a : Type) (b : Type) (c : Type) (v : List (a -> b)) (w : List a) (g1 : b -> c) : List c =
  concatMap (a -> b) c (λh1. list_map a c (compose a b c g1 h1) w) v

fn apThenBind (a : Type) (b : Type) (c : Type) (v : List (a -> b)) (w : List a) (g1 : b -> c) : List c =
  list_map b c g1 (concatMap (a -> b) b (λh1. list_map a b h1 w) v)

fn pfProbe (a : Type) (b : Type) (c : Type) (v : List (a -> b)) (w : List a) (g1 : b -> c) :
  Equal (List c) (apCompH1 a b c v w g1) (apThenBind a b c v w g1) =
  match v {
    Nil ⇒ tt ;
    Cons h0 t ⇒
      trans (List c)
        (list_append c (list_map a c (compose a b c g1 h0) w) (apCompH1 a b c t w g1))
        (list_append c (list_map b c g1 (list_map a b h0 w)) (apThenBind a b c t w g1))
        (list_map b c g1 (list_append b (list_map a b h0 w) (concatMap (a -> b) b (λh1. list_map a b h1 w) t)))
        (trans (List c)
           (list_append c (list_map a c (compose a b c g1 h0) w) (apCompH1 a b c t w g1))
           (list_append c (list_map b c g1 (list_map a b h0 w)) (apCompH1 a b c t w g1))
           (list_append c (list_map b c g1 (list_map a b h0 w)) (apThenBind a b c t w g1))
           (cong (List c) (List c) (list_map a c (compose a b c g1 h0) w) (list_map b c g1 (list_map a b h0 w)) (λz. list_append c z (apCompH1 a b c t w g1)) (list_functor_fusion a b c g1 h0 w))
           (cong (List c) (List c) (apCompH1 a b c t w g1) (apThenBind a b c t w g1) (list_append c (list_map b c g1 (list_map a b h0 w))) (pfProbe a b c t w g1)))
        (sym (List c)
           (list_map b c g1 (list_append b (list_map a b h0 w) (concatMap (a -> b) b (λh1. list_map a b h1 w) t)))
           (list_append c (list_map b c g1 (list_map a b h0 w)) (list_map b c g1 (concatMap (a -> b) b (λh1. list_map a b h1 w) t)))
           (list_map_append_distrib b c g1 (list_map a b h0 w) (concatMap (a -> b) b (λh1. list_map a b h1 w) t)))
  }

fn list_ap_cmp_mid1 (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) (w : List a) :
  Equal (List c)
    (concatMap (a -> c) c (apMapW a c w) (concatMap (b -> c) (a -> c) (apMapV a b c v) u))
    (concatMap (b -> c) c (apCompH1 a b c v w) u) =
  trans (List c)
    (concatMap (a -> c) c (λh2. list_map a c h2 w) (concatMap (b -> c) (a -> c) (λg1. list_map (a -> b) (a -> c) (compose a b c g1) v) u))
    (concatMap (b -> c) c (λg1. concatMap (a -> c) c (λh2. list_map a c h2 w) (list_map (a -> b) (a -> c) (compose a b c g1) v)) u)
    (concatMap (b -> c) c (λg1. concatMap (a -> b) c (λh1. list_map a c (compose a b c g1 h1) w) v) u)
    (list_bind_asc (b -> c) (a -> c) c u (λg1. list_map (a -> b) (a -> c) (compose a b c g1) v) (λh2. list_map a c h2 w))
    (concatMap_pointwise_eq (b -> c) c
       (λg1. concatMap (a -> c) c (λh2. list_map a c h2 w) (list_map (a -> b) (a -> c) (compose a b c g1) v))
       (λg1. concatMap (a -> b) c (λh1. list_map a c (compose a b c g1 h1) w) v)
       (λg1. concatMap_map_fusion (a -> b) (a -> c) c (λh2. list_map a c h2 w) (compose a b c g1) v)
       u)

fn list_ap_cmp_mid2 (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) (w : List a) :
  Equal (List c)
    (concatMap (b -> c) c (apCompH1 a b c v w) u)
    (list_ap b c u (list_ap a b v w)) =
  concatMap_pointwise_eq (b -> c) c (apCompH1 a b c v w) (apThenBind a b c v w) (pfProbe a b c v w) u

fn list_ap_cmp_mid (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) (w : List a) :
  Equal (List c)
    (concatMap (a -> c) c (apMapW a c w) (concatMap (b -> c) (a -> c) (apMapV a b c v) u))
    (list_ap b c u (list_ap a b v w)) =
  trans (List c)
    (concatMap (a -> c) c (apMapW a c w) (concatMap (b -> c) (a -> c) (apMapV a b c v) u))
    (concatMap (b -> c) c (apCompH1 a b c v w) u)
    (list_ap b c u (list_ap a b v w))
    (list_ap_cmp_mid1 a b c u v w)
    (list_ap_cmp_mid2 a b c u v w)

fn list_ap_cmp (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) (w : List a) :
  Equal (List c)
    (list_ap a c (list_ap (a -> b) (a -> c) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
    (list_ap b c u (list_ap a b v w)) =
  trans (List c)
    (list_ap a c (list_ap (a -> b) (a -> c) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
    (concatMap (a -> c) c (λh2. list_map a c h2 w) (concatMap (b -> c) (a -> c) (λg1. list_map (a -> b) (a -> c) (compose a b c g1) v) u))
    (list_ap b c u (list_ap a b v w))
    (cong (List (a -> c)) (List c)
       (list_ap (a -> b) (a -> c) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v)
       (concatMap (b -> c) (a -> c) (λg1. list_map (a -> b) (a -> c) (compose a b c g1) v) u)
       (λq. list_ap a c q w)
       (list_ap_cmp_front a b c u v))
    (list_ap_cmp_mid a b c u v w)
```

### 2.4 The `List` instances

`instance Applicative Option`/`instance Monad Option` are already declared
above (`§2.2`, immediately after their own laws — `Monad Option`'s
`bind_lid`/`bind_rid` need `Applicative_instance_Option` to exist first).
The `List` instances close out the same wiring pattern:

```ken
instance Applicative List {
  functor = Functor_instance_List ;
  pure = list_pure ;
  ap = list_ap ;
  ap_id = list_ap_id ;
  ap_hom = list_ap_hom ;
  ap_ich = list_ap_ich ;
  ap_cmp = list_ap_cmp ;
  map_coh = list_map_coh
}

instance Monad List {
  applicative = Applicative_instance_List ;
  bind = list_bind ;
  bind_lid = list_bind_lid ;
  bind_rid = list_bind_rid ;
  bind_asc = list_bind_asc
}
```

(`option_ap_id`/`option_ap_ich`/`option_ap_cmp`/`option_map_coh`/
`option_bind_rid`/`option_bind_asc` each dispatch, via a thin outer
`match`, to per-branch top-level lemmas rather than inlining the proof —
`§5` explains why.)

### 2.5 The `ITree` bridge — attested, not a surface instance

`Monad`'s fields and laws are satisfied by the landed interaction-tree
`bind` (`declare_bind`, `ken-elaborator/src/effects/state.rs:477`, a
single `Term::Elim` over `ITree e resp` whose `Ret` method is `λx. k x`):
`pure := Ret`; `bind_lid` is DEFINITIONAL (`ι` on `Ret` — the elimination
computes immediately, no induction needed); `bind_rid`/`bind_asc` hold by
induction on the tree, the same shape as `List`'s own `bind_rid`/
`bind_asc` above. This entry mints no second `bind` and writes no
`instance Monad (ITree e resp)` — `ITree e resp` is a parametric instance
head (free `e`, `resp`), and `elab_instance_decl` elaborates an instance
head in an EMPTY context, so a free head variable raises `UnresolvedCon`;
a general surface instance therefore does not elaborate today (the CAT-1
`55 §6.1` parametric-instance-head gap, still open with the Steward — not
reopened here). The effect system's denotation is a lawful monad BY
CONSTRUCTION, one denotation, not two.

## 3. Using it

```ken example
const listPureTwo : List Nat = list_pure Nat (Suc (Suc Zero))

const listApExample : List Nat =
  list_ap Nat Nat (Cons (Nat -> Nat) (Suc) (Nil (Nat -> Nat))) (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat)))

const listBindExample : List Nat =
  list_bind Nat Nat (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat))) (λx. Cons Nat x (Cons Nat x (Nil Nat)))
```

```ken example
const optionApSome : Option Nat = option_ap Nat Nat (Some (Nat -> Nat) Suc) (Some Nat Zero)
const optionApNone : Option Nat = option_ap Nat Nat (None (Nat -> Nat)) (Some Nat Zero)

const optionBindSome : Option Nat = option_bind Nat Nat (Some Nat Zero) (λx. Some Nat (Suc x))
const optionBindNone : Option Nat = option_bind Nat Nat (None Nat) (λx. Some Nat (Suc x))
```

## 4. Laws  proofs

Every `Applicative`/`Monad` law is already a real proof term inside the
instance declarations above (`§2.4`) — `class`'s own field-checking IS the
law-discharge mechanism, per the CAT-1/CAT-2 template. A handful of
computation facts about the concrete instances round out the picture:

```ken example
lemma listBindLidAtZero : Equal (List Nat) (list_bind Nat Nat (list_pure Nat Zero) (list_pure Nat)) (list_pure Nat Zero) = list_bind_lid Nat Nat Zero (list_pure Nat)

lemma optionApNoneShortCircuits : Equal (Option Nat) (option_ap Nat Nat (None (Nat -> Nat)) (Some Nat Zero)) (None Nat) = tt
```

`listBindLidAtZero` is `bind_lid` (already proved generically in `§2.3`)
instantiated at a concrete `x`/`k` — a direct application, not a fresh
proof; every catalog law is reusable this way at any concrete instance.
`optionApNoneShortCircuits` closes with `tt`: `mf = None` collapses
`option_ap`'s match immediately to the literal `None` constructor on both
sides.

```ken reject
-- Fails: `Refl` cannot close `list_ap_cmp`'s general statement directly
-- for ABSTRACT `u`/`v`/`w` -- neither side is a literal reduced value, and
-- unlike the concrete examples above, the composition law's proof
-- genuinely needs the induction this entry supplies (`§2.3`), not a bare
-- reflexivity check.
lemma listApCmpIsNotJustRefl (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) (w : List a) :
  Equal (List c)
    (list_ap a c (list_ap (a -> b) (a -> c) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
    (list_ap b c u (list_ap a b v w)) = Refl
```

## 5. Design notes

**Why the `sym`-around-a-`trans` shape kept failing, and why direct
induction was the fix.** The first attempt at the `pfProbe`-shaped
reconciliation (`§2.3`) tried to state the fact as `sym(trans(...))`,
composing already-proved fusion lemmas algebraically rather than
inducting directly. It failed twice, both times with a genuine bug (a
`sym` argument-direction mistake, and — more fundamentally — a missing
`list_map_append_distrib` step that the algebraic framing quietly assumed
away). Direct induction on the same carrier, following the SAME shape as
every other proof in this entry, surfaced both mistakes immediately via
the kernel's own error messages and was the reliable fix. The general
lesson: prefer inducting on the concrete carrier over composing several
already-proved lemmas algebraically when the algebraic route requires
tracking multiple direction/associativity choices by hand.

**Why several match-dispatching lemmas exist instead of inline proofs.**
A SELF-RECURSIVE proof whose match arms directly embed complex proof terms
(rather than dispatching to separately-declared, non-recursive lemmas)
repeatedly hit a kernel `TypeMismatch` that a structurally-identical
DISPATCHED version did not — confirmed for both a non-recursive case
(`option_ap_id`, `Option`'s laws) and a recursive one (`pfProbe`, `§2.3`).
The reliable shape throughout this entry: prove each match arm as its own
top-level, directly-ascribed lemma, then dispatch to it from a thin outer
`match`.

**Why `Refl` and `tt` land where they do.** For an ABSTRACT type parameter,
a constructor endpoint whose payload is itself abstract (e.g. `Some a x`
for abstract `x`) is STUCK, not collapsed — `Refl`, never `tt`; only a
NULLARY-constructor endpoint (`Nil`/`None`) genuinely collapses to `Top`
and wants `tt`. This is the guide's own `tt`-vs-`Refl` discriminator
(`catalog/guide/proof-techniques.ken.md §1`), applied dozens of times
across this entry's `Option` and `List` proofs.

## 6. Findings

- **Kernel-reduction defect:** none.
- **Sugar candidate → Ergo (parser):** `.field` projection on a class
  record and a bare `λ` BOTH fail to parse inside a `fn`'s own declared
  TYPE (`parse_atom_type` has no dot-continuation or lambda arm) — only
  in VALUE/body position. Hit repeatedly (`functorMapOf`,
  `applicativePureOf`, `composeKleisli`, `apMapV`, `apMapW`, `apCompH1`,
  `apThenBind`), each worked around with a named accessor function taking
  the dictionary/composed-function explicitly. This is the SAME class of
  gap DS-2 found for instance-value projection in type position;
  confirmed here to extend to bare lambdas too.
- **Abstraction candidate → Ergo/catalog follow-up:** `concatMap` is not
  landed anywhere in the catalog (only named in the spec chapter's own
  prose) despite `list_append`, `list_map`, and the other `List`
  operations it naturally sits alongside all being landed. Inlined here
  per the foundation-leader's explicit ruling (cross-file import isn't
  available yet regardless, so a `Collections.ken` landing would be a
  second, unconsumed copy today) — once cross-file import lands, this
  should move to a real `Collections.ken` addition so it is genuinely
  shared.
- **Naming hazard:** `concatMap`'s natural argument order (function
  first, matching its own `foldr`-shaped recursion) is the OPPOSITE of
  `bind`'s field order (container first) — `list_bind` is the necessary
  order-adjusted wrapper, not a stylistic renaming.
- **Kernel/elaborator quirk → Ergo:** `Axiom` as a top-level `fn`'s BODY
  fails with `VarOutOfScope` whenever the `fn`'s declared return type
  references one of the `fn`'s OWN parameters (`fn f (v : Bool) : Equal
  Bool v v = Axiom` fails; `const g : Equal Bool True True = Axiom`, and
  `Axiom` as a `class` instance FIELD value — e.g. `instance ProbeLaw Nat
  { trivial = Axiom }` — both succeed). Confirmed empirically; low
  severity (every landed `Axiom` use is already an instance field, the
  shape this doesn't break), but worth a fix or at least a documented
  limitation so the next author who reaches for a standalone Axiom-backed
  lemma doesn't lose time to it.

## 7. References

- **Wikipedia** — [Applicative
  functor](https://en.wikipedia.org/wiki/Applicative_functor) and
  [Monad (functional
  programming)](https://en.wikipedia.org/wiki/Monad_(functional_programming))
  — general orientation on the two classes' laws and intent.
- **Haskell base** — `Control.Applicative`/`Control.Monad`
  (`GHC.Base`, part of the `base` package, BSD-3-Clause) —
  <https://gitlab.haskell.org/ghc/ghc> — the canonical `pure`/`<*>`/`>>=`
  shapes and the list-monad's cartesian `ap`, consulted for shape only
  (`CLEAN-ROOM.md`, no source copied).
- **Lean 4 core** — `Applicative`/`Monad`
  (`Init/Prelude.lean`, part of the Lean 4 repository, Apache-2.0) —
  <https://github.com/leanprover/lean4> — the wired-superclass
  (`[Functor f]`/`[Applicative f]`) constructor-class chain this entry's
  `functor`/`applicative` fields mirror, consulted for shape only.

## 8. Trust  derivation

1. **Spec / WP.** `docs/program/wp/ds-7-applicative-monad.md` (this
   entry's build WP); the design contract is `spec/50-stdlib/
   56-effectful-classes.md` (CAT-2), `§3`/`§4`.
2. **Public API.** `Applicative`, `Monad`, `Applicative_instance_Option`,
   `Monad_instance_Option`, `Applicative_instance_List`,
   `Monad_instance_List`, plus every named helper in `§2`.
3. **Source map.**

   | Task | Section |
   |---|---|
   | See the shape | [Definition](#2-definition) |
   | Use it | [Using it](#3-using-it) |
   | Check the computation facts | [Laws  proofs](#4-laws--proofs) |
   | Why the proof shapes are what they are | [Design notes](#5-design-notes) |

4. **Derivation path.** `class Applicative`/`class Monad` — ordinary
   `class` declarations (`elab_class_decl`), a right-nested Σ record, the
   same mechanism `class Functor`/`class Ord` already use. Every instance
   field — a real `declare_def` term, kernel-rechecked. `Option`'s laws —
   finite case-split. `List`'s laws — structural induction + `cong`/
   `trans`/`sym` (`Core/Transport.ken`), reusing the landed
   `list_right_unit`/`list_assoc`/`list_functor_id`/`list_functor_fusion`
   (`Core/LawfulFunctors.ken`) throughout. The `ITree` bridge — attested
   correspondence only (`§2.5`), zero new code.
5. **`trusted_base()` delta.** **Zero.** Every law field in both
   instances is a real, kernel-checked proof term — no `Axiom`, no new
   `declare_primitive`/`declare_postulate`, no new `Term`/`Decl` variant,
   no new elaborator capability (the wired superclass field, nested
   projection, and instance mechanism all ride the landed `class`
   machinery — confirmed directly, `§2.1`). Confirmed by
   `crates/ken-elaborator/tests/ds7_applicative_monad_acceptance.rs`'s
   structural `trusted_base()` before-vs-after set-difference check
   (the DS-2-established pattern), not just a source grep.
6. **Proof families.** `Option` — finite case-split, no induction.
   `List` — structural induction throughout; `ap_cmp` is the deepest
   (induction inside `pfProbe`, composed with three non-recursive fusion
   lemmas and the already-proved `list_bind_asc`).
7. **Consumers.** None yet in this catalog; the ITree bridge (`§2.5`) is
   already a "consumer" in the sense that the effect system's `bind`
   already denotes a lawful monad, this entry just makes that
   correspondence explicit and checked.
8. **Validation evidence.**
   `crates/ken-elaborator/tests/ds7_applicative_monad_acceptance.rs` —
   the zero-`Axiom`/`trusted_base()` check, discriminating negative cases
   for AC8 (a non-cartesian/law-breaking wired `applicative` field, a
   masked `Axiom` inhabiting `Bottom`, the no-second-`bind` ITree
   discriminator), and elaborating this entry's `` ```ken ``/
   `` ```ken example ``/`` ```ken reject `` fences through the literate
   extractor.

## 9. `Traversable`

`Traversable` is the last item of the `Functor → Applicative → Monad →
Traversable` toolkit chain: it walks a container while threading an
arbitrary effectful (`Applicative`) action, producing the effect of "do
this container's worth of work, but let the effect happen once, in
order, and collect the results back into the same shape." `map` alone
cannot do this — it has nowhere to put the effect; `traverse` is `map`
generalized over an `Applicative` action.

### 9.1 Definition

`traverse`'s action argument returns an effectful value `g b` for an
ABSTRACT `g` — the class field wires `Applicative g` as an ordinary
EXPLICIT parameter (Fork C: an abstract `g` has no concrete head for
implicit instance search, so the implicit-constraint form `traverse
[Applicative g] ...` does not elaborate; the explicit-dictionary form
does, riding the same mechanism `list_traverse`'s own `apg` parameter
below uses). `functor`/`foldable` are WIRED superclass fields, supplied
whole — this entry does not re-prove `Functor List`/`Foldable Option` and
friends, all landed in `Core/LawfulFunctors.ken`.

```ken
class Traversable (f : Type -> Type) {
  functor : Functor f ;
  foldable : Foldable f ;
  proc traverse : (g : Type -> Type) -> Applicative g -> (a : Type) -> (b : Type) -> (a -> g b) -> f a -> g (f b)
}
```

`traverse`'s field carries SURF-2's `proc` marker: `g` is abstract, so
SURF-1's row-variable mechanism classifies the field's action fail-closed
as potentially effectful (`Unknown` codomain head → `RowVar` → `proc`,
`36 §1.5`/`§1.6`). A CONCRETE instance's traversal (`list_traverse`/
`option_traverse` below) is, for `List`/`Option`, a genuinely PURE
function of its explicit `Applicative g` dictionary — no real effect ever
fires. Assigning that pure `fn` into the `proc`-marked field needed the
DS-8b `∅ ⊆ proc` widening (`36 §1.6.2`) landed alongside this entry: a
class field's declared purity is an UPPER BOUND on what an instance may
do, not a requirement that every instance actually do it.

`List`'s traversal is the standard effect-sequencing fold — `pure Nil`
for the base case, then `ap` combines each mapped `Cons` with the
recursive traversal of the tail:

```ken
fn list_traverse (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (t : a -> g b) (xs : List a) : g (List b) =
  match xs {
    Nil ⇒ apg.pure (List b) (Nil b) ;
    Cons h u ⇒ apg.ap (List b) (List b) (apg.functor.map b (List b -> List b) (Cons b) (t h)) (list_traverse g apg a b t u)
  }

instance Traversable List {
  functor = Functor_instance_List ;
  foldable = Foldable_instance_List ;
  traverse = list_traverse
}
```

`Option`'s traversal short-circuits: `None` needs no effect at all,
`Some` runs the action once and re-wraps:

```ken
fn option_traverse (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (t : a -> g b) (mx : Option a) : g (Option b) =
  match mx {
    None ⇒ apg.pure (Option b) (None b) ;
    Some x ⇒ apg.ap b (Option b) (apg.pure (b -> Option b) (Some b)) (t x)
  }

instance Traversable Option {
  functor = Functor_instance_Option ;
  foldable = Foldable_instance_Option ;
  traverse = option_traverse
}
```

### 9.2 Coherence laws — identity and naturality (proved)

`§5.3`'s three coherence laws are stated and proved SEPARATELY from the
class (the class record itself carries no law fields for `traverse` —
unlike `Applicative`/`Monad` above, `§5.1`'s `class Traversable` shape has
none; the laws are standalone lemmas about a concrete instance's
`traverse`, not class-record obligations).

**Identity** — traversing with the trivial (`Identity`) applicative
changes nothing but the wrapper. `Identity` is built fresh (a bare
one-constructor wrapper; every one of its eight `Functor`/`Applicative`
laws closes immediately by the `tt`/`Refl` discriminator, no induction —
there is no real computation content to a wrapper that only wraps and
unwraps):

```ken
data Identity a = MkIdentity a

fn identity_pure (a : Type) (x : a) : Identity a = MkIdentity a x

fn identity_map (a : Type) (b : Type) (f : a -> b) (x : Identity a) : Identity b =
  match x { MkIdentity v ⇒ MkIdentity b (f v) }

fn identity_ap (a : Type) (b : Type) (mf : Identity (a -> b)) (mx : Identity a) : Identity b =
  match mf { MkIdentity f ⇒ match mx { MkIdentity x ⇒ MkIdentity b (f x) } }

fn identity_id_law (a : Type) (x : Identity a) : Equal (Identity a) (identity_map a a (idf a) x) x =
  match x { MkIdentity v ⇒ Refl }

fn identity_fusion_law (a : Type) (b : Type) (c : Type) (g : b -> c) (h : a -> b) (x : Identity a) :
  Equal (Identity c) (identity_map a c (comp a b c g h) x) (identity_map b c g (identity_map a b h x)) =
  match x { MkIdentity v ⇒ Refl }

fn identity_ap_id (a : Type) (v : Identity a) : Equal (Identity a) (identity_ap a a (identity_pure (a -> a) (idf a)) v) v =
  match v { MkIdentity x ⇒ Refl }

fn identity_ap_hom (a : Type) (b : Type) (g : a -> b) (x : a) :
  Equal (Identity b) (identity_ap a b (identity_pure (a -> b) g) (identity_pure a x)) (identity_pure b (g x)) =
  Refl

fn identity_ap_ich (a : Type) (b : Type) (u : Identity (a -> b)) (y : a) :
  Equal (Identity b) (identity_ap a b u (identity_pure a y)) (identity_ap (a -> b) b (identity_pure ((a -> b) -> b) (applyTo a b y)) u) =
  match u { MkIdentity f ⇒ Refl }

fn identity_ap_cmp (a : Type) (b : Type) (c : Type) (u : Identity (b -> c)) (v : Identity (a -> b)) (w : Identity a) :
  Equal (Identity c)
    (identity_ap a c (identity_ap (a -> b) (a -> c) (identity_ap (b -> c) ((a -> b) -> (a -> c)) (identity_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
    (identity_ap b c u (identity_ap a b v w)) =
  match u { MkIdentity g ⇒ match v { MkIdentity h ⇒ match w { MkIdentity x ⇒ Refl } } }

fn identity_map_coh (a : Type) (b : Type) (g : a -> b) (x : Identity a) :
  Equal (Identity b) (identity_map a b g x) (identity_ap a b (identity_pure (a -> b) g) x) =
  match x { MkIdentity v ⇒ Refl }

instance Functor Identity {
  map = identity_map ;
  id_law = identity_id_law ;
  fusion_law = identity_fusion_law
}

instance Applicative Identity {
  functor = Functor_instance_Identity ;
  pure = identity_pure ;
  ap = identity_ap ;
  ap_id = identity_ap_id ;
  ap_hom = identity_ap_hom ;
  ap_ich = identity_ap_ich ;
  ap_cmp = identity_ap_cmp ;
  map_coh = identity_map_coh
}
```

The identity law itself, for both instances, by induction on the carrier
(the `List` case needs `cong` to push the IH under `Cons`; the `Option`
case is a two-arm case-split, no induction):

```ken
fn list_traverse_identity_law (a : Type) (xs : List a) :
  Equal (Identity (List a)) (list_traverse Identity Applicative_instance_Identity a a (identity_pure a) xs) (identity_pure (List a) xs) =
  match xs {
    Nil ⇒ tt ;
    Cons h u ⇒ cong (Identity (List a)) (Identity (List a))
      (list_traverse Identity Applicative_instance_Identity a a (identity_pure a) u)
      (identity_pure (List a) u)
      (identity_map (List a) (List a) (Cons a h))
      (list_traverse_identity_law a u)
  }

fn option_traverse_identity_law (a : Type) (mx : Option a) :
  Equal (Identity (Option a)) (option_traverse Identity Applicative_instance_Identity a a (identity_pure a) mx) (identity_pure (Option a) mx) =
  match mx {
    None ⇒ tt ;
    Some x ⇒ Refl
  }
```

**Naturality** — an `Applicative` MORPHISM `η : g ⇒ h` (a family
`eta_map : (a:Type) → g a → h a` that commutes with `pure` and `ap`)
commutes with `traverse`: `η(traverse g apg t xs) = traverse h aph (η∘t)
xs`. `η`'s two defining properties are threaded as EXPLICIT parameters
(the same Fork C shape as every dictionary in this entry) rather than a
bundled morphism class — there is no landed `ApplicativeMorphism` class to
wire, and bundling one is out of this WP's scope. A reusable lemma first:
any such `η` also commutes with plain `functor.map` (derived from `η`'s
own two properties plus `map_coh`, applied identically to close both
instances' `Cons`/`Some` cases below):

```ken
fn applicativeApOf (g_ty : Type -> Type) (d : Applicative g_ty) (a : Type) (b : Type) (mf : g_ty (a -> b)) (mx : g_ty a) : g_ty b =
  d.ap a b mf mx

fn applicativeMapOf (g_ty : Type -> Type) (d : Applicative g_ty) (a : Type) (b : Type) (f : a -> b) (x : g_ty a) : g_ty b =
  d.functor.map a b f x

fn eta_natural_map_pure_term (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (eta_map : (a : Type) -> g a -> h a) (a : Type) (b : Type) (f : a -> b) : h (a -> b) =
  eta_map (a -> b) (applicativePureOf g apg (a -> b) f)

fn eta_natural_map_eq1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (eta_map : (a : Type) -> g a -> h a)
  (eta_pure : (a : Type) -> (x : a) -> Equal (h a) (eta_map a (applicativePureOf g apg a x)) (applicativePureOf h aph a x))
  (a : Type) (b : Type) (f : a -> b) :
  Equal (h (a -> b)) (eta_natural_map_pure_term g h apg eta_map a b f) (applicativePureOf h aph (a -> b) f) =
  eta_pure (a -> b) f

fn eta_natural_map (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (eta_map : (a : Type) -> g a -> h a)
  (eta_pure : (a : Type) -> (x : a) -> Equal (h a) (eta_map a (applicativePureOf g apg a x)) (applicativePureOf h aph a x))
  (eta_ap : (a : Type) -> (b : Type) -> (mf : g (a -> b)) -> (mx : g a) -> Equal (h b) (eta_map b (applicativeApOf g apg a b mf mx)) (applicativeApOf h aph a b (eta_map (a -> b) mf) (eta_map a mx)))
  (a : Type) (b : Type) (f : a -> b) (x : g a) :
  Equal (h b) (eta_map b (applicativeMapOf g apg a b f x)) (applicativeMapOf h aph a b f (eta_map a x)) =
  trans (h b)
    (eta_map b (apg.functor.map a b f x))
    (aph.ap a b (eta_natural_map_pure_term g h apg eta_map a b f) (eta_map a x))
    (aph.functor.map a b f (eta_map a x))
    (trans (h b)
       (eta_map b (apg.functor.map a b f x))
       (eta_map b (apg.ap a b (apg.pure (a -> b) f) x))
       (aph.ap a b (eta_natural_map_pure_term g h apg eta_map a b f) (eta_map a x))
       (cong (g b) (h b)
          (apg.functor.map a b f x)
          (apg.ap a b (apg.pure (a -> b) f) x)
          (eta_map b)
          (apg.map_coh a b f x))
       (eta_ap a b (apg.pure (a -> b) f) x))
    (sym (h b)
       (aph.functor.map a b f (eta_map a x))
       (aph.ap a b (eta_natural_map_pure_term g h apg eta_map a b f) (eta_map a x))
       (trans (h b)
          (aph.functor.map a b f (eta_map a x))
          (aph.ap a b (aph.pure (a -> b) f) (eta_map a x))
          (aph.ap a b (eta_natural_map_pure_term g h apg eta_map a b f) (eta_map a x))
          (aph.map_coh a b f (eta_map a x))
          (cong (h (a -> b)) (h b)
             (aph.pure (a -> b) f)
             (eta_natural_map_pure_term g h apg eta_map a b f)
             (λw. aph.ap a b w (eta_map a x))
             (sym (h (a -> b)) (eta_natural_map_pure_term g h apg eta_map a b f) (aph.pure (a -> b) f) (eta_natural_map_eq1 g h apg aph eta_map eta_pure a b f)))))
```

And the naturality law itself, for both instances (`List`'s `Cons` case
chains `eta_ap`, `eta_natural_map`, and the IH; `Option`'s `Some` case
needs only `eta_ap` and `eta_pure`):

```ken
fn list_traverse_nat_action (g : Type -> Type) (h : Type -> Type) (eta_map : (a : Type) -> g a -> h a) (a : Type) (b : Type) (t : a -> g b) (x : a) : h b =
  eta_map b (t x)

fn list_traverse_naturality (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (eta_map : (a : Type) -> g a -> h a)
  (eta_pure : (a : Type) -> (x : a) -> Equal (h a) (eta_map a (applicativePureOf g apg a x)) (applicativePureOf h aph a x))
  (eta_ap : (a : Type) -> (b : Type) -> (mf : g (a -> b)) -> (mx : g a) -> Equal (h b) (eta_map b (applicativeApOf g apg a b mf mx)) (applicativeApOf h aph a b (eta_map (a -> b) mf) (eta_map a mx)))
  (a : Type) (b : Type) (t : a -> g b) (xs : List a) :
  Equal (h (List b)) (eta_map (List b) (list_traverse g apg a b t xs)) (list_traverse h aph a b (list_traverse_nat_action g h eta_map a b t) xs) =
  match xs {
    Nil ⇒ eta_pure (List b) (Nil b) ;
    Cons hd u ⇒
      trans (h (List b))
        (eta_map (List b) (list_traverse g apg a b t (Cons a hd u)))
        (aph.ap (List b) (List b) (eta_map (List b -> List b) (apg.functor.map b (List b -> List b) (Cons b) (t hd))) (eta_map (List b) (list_traverse g apg a b t u)))
        (list_traverse h aph a b (list_traverse_nat_action g h eta_map a b t) (Cons a hd u))
        (eta_ap (List b) (List b) (apg.functor.map b (List b -> List b) (Cons b) (t hd)) (list_traverse g apg a b t u))
        (trans (h (List b))
           (aph.ap (List b) (List b) (eta_map (List b -> List b) (apg.functor.map b (List b -> List b) (Cons b) (t hd))) (eta_map (List b) (list_traverse g apg a b t u)))
           (aph.ap (List b) (List b) (aph.functor.map b (List b -> List b) (Cons b) (eta_map b (t hd))) (eta_map (List b) (list_traverse g apg a b t u)))
           (list_traverse h aph a b (list_traverse_nat_action g h eta_map a b t) (Cons a hd u))
           (cong (h (List b -> List b)) (h (List b))
              (eta_map (List b -> List b) (apg.functor.map b (List b -> List b) (Cons b) (t hd)))
              (aph.functor.map b (List b -> List b) (Cons b) (eta_map b (t hd)))
              (λw. aph.ap (List b) (List b) w (eta_map (List b) (list_traverse g apg a b t u)))
              (eta_natural_map g h apg aph eta_map eta_pure eta_ap b (List b -> List b) (Cons b) (t hd)))
           (cong (h (List b)) (h (List b))
              (eta_map (List b) (list_traverse g apg a b t u))
              (list_traverse h aph a b (list_traverse_nat_action g h eta_map a b t) u)
              (λw. aph.ap (List b) (List b) (aph.functor.map b (List b -> List b) (Cons b) (eta_map b (t hd))) w)
              (list_traverse_naturality g h apg aph eta_map eta_pure eta_ap a b t u)))
  }

fn option_traverse_nat_action (g : Type -> Type) (h : Type -> Type) (eta_map : (a : Type) -> g a -> h a) (a : Type) (b : Type) (t : a -> g b) (x : a) : h b =
  eta_map b (t x)

fn option_traverse_naturality (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (eta_map : (a : Type) -> g a -> h a)
  (eta_pure : (a : Type) -> (x : a) -> Equal (h a) (eta_map a (applicativePureOf g apg a x)) (applicativePureOf h aph a x))
  (eta_ap : (a : Type) -> (b : Type) -> (mf : g (a -> b)) -> (mx : g a) -> Equal (h b) (eta_map b (applicativeApOf g apg a b mf mx)) (applicativeApOf h aph a b (eta_map (a -> b) mf) (eta_map a mx)))
  (a : Type) (b : Type) (t : a -> g b) (mx : Option a) :
  Equal (h (Option b)) (eta_map (Option b) (option_traverse g apg a b t mx)) (option_traverse h aph a b (option_traverse_nat_action g h eta_map a b t) mx) =
  match mx {
    None ⇒ eta_pure (Option b) (None b) ;
    Some x ⇒
      trans (h (Option b))
        (eta_map (Option b) (option_traverse g apg a b t (Some a x)))
        (aph.ap b (Option b) (eta_map (b -> Option b) (apg.pure (b -> Option b) (Some b))) (eta_map b (t x)))
        (option_traverse h aph a b (option_traverse_nat_action g h eta_map a b t) (Some a x))
        (eta_ap b (Option b) (apg.pure (b -> Option b) (Some b)) (t x))
        (cong (h (b -> Option b)) (h (Option b))
           (eta_map (b -> Option b) (apg.pure (b -> Option b) (Some b)))
           (aph.pure (b -> Option b) (Some b))
           (λw. aph.ap b (Option b) w (eta_map b (t x)))
           (eta_pure (b -> Option b) (Some b)))
  }
```

### 9.3 Using it

```ken example
const listTraverseIdentity : Identity (List Nat) =
  list_traverse Identity Applicative_instance_Identity Nat Nat (identity_pure Nat) (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat)))

const optionTraverseSome : Identity (Option Nat) =
  option_traverse Identity Applicative_instance_Identity Nat Nat (identity_pure Nat) (Some Nat Zero)
```

### 9.4 Composition law — deferred for SIZE, not capability, to `DS-8c`

**This is a scheduling deferral, not a capability gap.** Everything
below is fully buildable TODAY with zero missing elaborator capability
and zero `§6.1`-style fork — it is ~40-60 more lemmas of ordinary proof
engineering at this entry's own granularity, with a concrete, written
closing plan (below), not an open-ended "later." It is deferred to a
named follow-on, `DS-8c` (traverse composition coherence law), because
holding the rest of `Traversable` — which does not depend on it — for
one more law's worth of bookkeeping was the worse tradeoff, not because
anything walls.

**Two things are deferred, precisely — read both claims below exactly as
scoped, not rounded up:**

1. **`Compose g h` is NOT YET a fully-proven-lawful `Applicative`.**
   `Compose` lands cleanly as a `fn`-level type synonym (`fn Compose (g :
   Type -> Type) (h : Type -> Type) (a : Type) : Type = g (h a)`,
   sidestepping `data`'s Type-0-only parameter hardcoding,
   `crates/ken-elaborator/src/data.rs:45`), and its full `Functor`
   instance (`id_law`/`fusion_law`) plus **three of `Applicative`'s four
   laws** — `ap_id`, `ap_hom`, `ap_ich` — plus `map_coh`, are proved:
   kernel-checked, zero `Axiom`, derived from `apg`/`aph`'s own laws via
   the same `trans`/`cong`/`sym` combinators as everywhere else in this
   entry, using the kernel's `Eq`-at-Pi unfold (`eq_at_pi`,
   `ken-kernel/src/obs.rs`) to promote a pointwise law to a
   function-level equality without a separate `funext` primitive. A
   reusable `ap_naturality` auxiliary (`apg.ap (apg.map (compose ψ) u) v
   = apg.map ψ (apg.ap u v)`, derived once from `apg`'s own `map_coh`/
   `ap_hom`/`ap_cmp`) further shortened the crossings this needed. The
   FOURTH law, `ap_cmp` (associativity), is NOT proved — `Compose`'s
   `Applicative` instance is therefore not assembled in this entry, and
   no `instance Applicative (Compose g h)` is declared here. What IS
   real, tested, and reusable toward `ap_cmp`: `ap_cmp`'s LHS reduces
   (via `compose_map_coh` then `apg.functor.fusion_law`, both already
   proved) to a single fused `apg` operation at each of its two nested
   levels — genuine forward progress, not a stub.
2. **`§5.3`'s composition coherence law itself — `traverse` composes
   through `Compose g h` — is separately deferred**, because it
   CONSUMES the missing `ap_cmp`. It is not claimed, asserted, or tested
   anywhere in this entry.

**Zero papering.** Every lemma above (Level1/Level2's reductions
included) is a real, fully-applied, kernel-checked proof term — no
`Axiom`, no `Refl`/`tt` forced where the goal does not actually collapse,
no stub. `ap_cmp` itself is simply absent — not declared with a
postulated body, not present under any name.

**`DS-8c`'s spec — the concrete closing plan, so the follow-on is a
scoped prerequisite, not an open "later":**

1. Rewrite the triple-composed crossing function (`Level2`'s fused
   `aph`-level composition) via `aph.map_coh`, applied pointwise, into
   pure `ap`/`pure` form.
2. Apply `aph.ap_cmp` itself as a TRIPLE-pointwise function equality
   (the SAME `eq_at_pi` promotion this entry already uses single- and
   double-pointwise for `ap_ich`/`ap_naturality`, one level deeper).
3. Lift that equality through three nested `apg` applications
   (`functor.map`, then two `ap`s).
4. Reconcile the result against the already-free RHS (`apg.ap_cmp`
   instantiated at `uP`/`vP`/`W`, proved in this entry) by splitting the
   triple application back into the `uP`/`vP` shape.

Once `DS-8c` lands `Compose`'s `ap_cmp`, `instance Applicative (Compose g
h)` assembles from pieces already in this entry, and the composition
coherence law follows by the same `list_traverse`/`option_traverse`
induction shape as `§9.2`'s identity/naturality proofs above.

**Scope the "lawful `Traversable`" claim precisely.** `List`/`Option`'s
`Traversable` instances (`§9.1`) satisfy the **identity** and
**naturality** coherence laws (`§9.2`, both proved) — they are not yet
claimed "fully lawful `Traversable`" in the sense of all three `§5.3`
laws; composition is the one outstanding law, tracked by `DS-8c`.

### 9.5 Findings

- **AC6 confirmed, with a landed-mechanism correction along the way.**
  `traverse`'s `proc` classification and the `∅ ⊆ proc` instance-field
  widening are two SEPARATE, both-necessary pieces — see DS-8b
  (`docs/program/wp/ds-8b-pure-into-proc-widening.md`). Before DS-8b
  landed, `proc traverse`'s class-field type parsed and classified fine,
  but NO concrete List/Option implementation (necessarily, honestly pure)
  could satisfy `check_instance_field_purity`'s then-strict `Proc`
  requirement — a genuine, grounded elaborator gap, escalated and fixed
  as its own WP rather than forced through with a workaround.
- **Sugar candidate → Ergo (parser), reconfirmed.** The `.field`
  projection / bare-`λ`-in-declared-type gap (`§6` above) recurred
  constantly building `Traversable`'s and `Compose`'s law proofs — every
  intermediate equality in a multi-step `trans`/`cong` chain needed a
  named accessor `fn` wrapping any dotted/lambda expression that appears
  in a DECLARED type (never in body position). Same gap, same workaround,
  now confirmed at much greater depth (dozens of accessors per law).
- **Reusable pattern → catalog/guide.** The `eq_at_pi`-promotion trick —
  a pointwise law `(x:A) -> Equal B (f x) (g x)`, partially applied one
  argument short of full, IS ALREADY (by kernel conversion, no extra
  proof step) a term of the function-level type `Equal (A -> B) f g` —
  closed every "lift a pointwise law to a `cong`-able function equality"
  step in this entry's `Compose`/naturality proofs, extending to
  MULTIPLE curried arguments at once (used for `aph.ap_ich`; the
  `ap_cmp` follow-on will need it three deep). Worth promoting into
  `catalog/guide/proof-techniques.ken.md` as a named technique — it
  recurs and is not obvious from the spec alone.
- **Proof-strategy consult, logged for the judgment log.** The `ap_cmp`
  scope call (grind vs. gate) went through a live Architect consult
  mid-build (the `ap_naturality` extraction) and a Steward-sanctioned
  valve (gate on a named follow-on once the remaining size was measured
  at ~40-60 lemmas, not the ~12-15 initially estimated). Both are
  judgment calls for the operator's log, not unilateral scope changes —
  flagged to the ring at each step before acting.

### 9.6 References

Same sources as `§7` (`Applicative`/`Monad`'s own references apply
identically to `Traversable`, which is the same family); additionally:

- **Haskell base** — `Data.Traversable`
  (`GHC.Base`, part of the `base` package, BSD-3-Clause) —
  <https://gitlab.haskell.org/ghc/ghc> — the canonical `traverse`/
  `sequenceA` shapes and the three coherence laws (identity, naturality,
  composition), consulted for shape only (`CLEAN-ROOM.md`, no source
  copied).
