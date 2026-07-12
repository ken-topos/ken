# `Applicative`, `Monad`, and `Traversable` — effectful constructor classes

`Applicative` and `Monad` are the two constructor classes every effectful
computation over a container (`List`, `Option`, and the interaction-tree
effect denotation `ITree`) ultimately builds on: `Applicative` sequences
independent effectful values, `Monad` sequences effectful values where
later steps depend on earlier results. This entry defines both classes and
proves them lawfully for `List` and `Option`. `§9` extends the family with
`Traversable`, the class that walks a container while threading an arbitrary
`Applicative` effect.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws  proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [References](#6-references)
7. [Trust  derivation](#7-trust--derivation)
9. [`Traversable`](#9-traversable)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws  proofs](#4-laws--proofs)
- *Researcher* → [Laws  proofs](#4-laws--proofs) →
  [Design notes](#5-design-notes)
- *Porting from Haskell/Lean/Agda* → [Design notes](#5-design-notes)

## 1. Motivation

`class Functor` lets a container be mapped over, but `map` alone cannot
combine two independent effectful values
(there is no way to apply a function living inside the container to an
argument also living inside the container) or sequence a computation whose
NEXT step depends on a PREVIOUS result. `Applicative` adds `pure` (lift a
value in, effect-free) and `ap` (apply a contained function to a contained
argument); `Monad` adds `bind` (sequence, with the next step allowed to
depend on the previous result). Both are proved lawfully here for `List`
(the cartesian/list-monad reading — every combination of elements) and
`Option` (short-circuiting on `None`), and related to the interaction-tree
`bind` that denotes effect rows in the language.

## 2. Definition

### 2.1 The wired superclass chain

`Applicative f` carries an explicit `functor : Functor f` field, and
`Monad f` carries an explicit `applicative : Applicative f` field. Each
instance supplies the complete superclass dictionary rather than restating
its fields. Nested projection such as `d.applicative.functor.map` composes
these interfaces, so a `Monad List` instance can reuse the laws of
`Applicative List` and add only `bind` and its laws.

`ap_id`/`ap_hom`/`ap_ich`/`ap_cmp`/`map_coh`/`bind_lid`/`bind_rid`/
`bind_asc` are `Ω`-classified value equations (`Equal (f _) u v`), with one
canonical field per law:

```ken
fn apply_to (a : Type) (b : Type) (y : a) (g : a -> b) : b = g y

fn compose (a : Type) (b : Type) (c : Type) (g : b -> c) (h : a -> b) (x : a) : c = g (h x)

fn functor_map_of (g_ty : Type -> Type) (d : Functor g_ty) (a : Type) (b : Type) (h : a -> b) (x : g_ty a) : g_ty b = d.map a b h x

class Applicative (f : Type -> Type) {
  functor : Functor f ;
  pure : (a : Type) -> a -> f a ;
  ap : (a : Type) -> (b : Type) -> f (a -> b) -> f a -> f b ;
  ap_id : (a : Type) -> (v : f a) -> Equal (f a) (ap a a (pure (a -> a) (idf a)) v) v ;
  ap_hom : (a : Type) -> (b : Type) -> (g : a -> b) -> (x : a) -> Equal (f b) (ap a b (pure (a -> b) g) (pure a x)) (pure b (g x)) ;
  ap_ich : (a : Type) -> (b : Type) -> (u : f (a -> b)) -> (y : a) -> Equal (f b) (ap a b u (pure a y)) (ap (a -> b) b (pure ((a -> b) -> b) (apply_to a b y)) u) ;
  ap_cmp : (a : Type) -> (b : Type) -> (c : Type) -> (u : f (b -> c)) -> (v : f (a -> b)) -> (w : f a) ->
    Equal (f c)
      (ap a c (ap (a -> b) (a -> c) (ap (b -> c) ((a -> b) -> (a -> c)) (pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
      (ap b c u (ap a b v w)) ;
  map_coh : (a : Type) -> (b : Type) -> (g : a -> b) -> (x : f a) -> Equal (f b) (functor_map_of f functor a b g x) (ap a b (pure (a -> b) g) x)
}

fn applicative_pure_of (g_ty : Type -> Type) (d : Applicative g_ty) (a : Type) (x : a) : g_ty a = d.pure a x

fn compose_kleisli (g_ty : Type -> Type) (bindfn : (a : Type) -> (b : Type) -> g_ty a -> (a -> g_ty b) -> g_ty b)
  (a : Type) (b : Type) (c : Type) (k : a -> g_ty b) (h : b -> g_ty c) (x : a) : g_ty c = bindfn b c (k x) h

class Monad (f : Type -> Type) {
  applicative : Applicative f ;
  bind : (a : Type) -> (b : Type) -> f a -> (a -> f b) -> f b ;
  bind_lid : (a : Type) -> (b : Type) -> (x : a) -> (k : a -> f b) -> Equal (f b) (bind a b (applicative_pure_of f applicative a x) k) (k x) ;
  bind_rid : (a : Type) -> (m : f a) -> Equal (f a) (bind a a m (applicative_pure_of f applicative a)) m ;
  bind_asc : (a : Type) -> (b : Type) -> (c : Type) -> (m : f a) -> (k : a -> f b) -> (h : b -> f c) ->
    Equal (f c) (bind b c (bind a b m k) h) (bind a c m (compose_kleisli f bind a b c k h))
}
```

`functor_map_of`, `applicative_pure_of`, and `compose_kleisli` name the
dictionary fields and composed operation used in the law statements.

### 2.2 `Option` — finite case-split, zero induction

`pure = Some`; `ap` short-circuits to `None` on either side; `bind (Some
x) k = k x`, `bind None k = None`. Every law closes by direct case
analysis — no recursion, since `Option` has no recursive structure:

```ken
fn option_pure (a : Type) (x : a) : Option a = Some a x

fn option_ap (a : Type) (b : Type) (mf : Option (a -> b)) (mx : Option a) : Option b =
  match mf { None ↦ None b ; Some g ↦ match mx { None ↦ None b ; Some x ↦ Some b (g x) } }

fn option_bind (a : Type) (b : Type) (m : Option a) (k : a -> Option b) : Option b =
  match m { None ↦ None b ; Some x ↦ k x }
```

For an ABSTRACT type parameter, a `Some a x` endpoint (`x` itself
abstract) is STUCK, not collapsed — `Refl`, not `Proved` (`§5`). Inlining a
proof directly in a self-recursive match arm also repeatedly hit a kernel
`TypeMismatch` a dispatched version did not (`§5`) — every law below
proves each branch as its own top-level, directly-ascribed lemma, then
dispatches via a thin outer `match`:

```ken
lemma option_ap_id_none (a : Type) : Equal (Option a) (option_ap a a (option_pure (a -> a) (idf a)) (None a)) (None a) = Proved

lemma option_ap_id_some (a : Type) (x : a) : Equal (Option a) (option_ap a a (option_pure (a -> a) (idf a)) (Some a x)) (Some a x) = Refl

lemma option_ap_id (a : Type) (v : Option a) : Equal (Option a) (option_ap a a (option_pure (a -> a) (idf a)) v) v =
  match v { None ↦ option_ap_id_none a ; Some x ↦ option_ap_id_some a x }

lemma option_ap_hom (a : Type) (b : Type) (g : a -> b) (x : a) : Equal (Option b) (option_ap a b (option_pure (a -> b) g) (option_pure a x)) (option_pure b (g x)) = Refl

lemma option_ap_ich_none (a : Type) (b : Type) (y : a) :
  Equal (Option b) (option_ap a b (None (a -> b)) (option_pure a y)) (option_ap (a -> b) b (option_pure ((a -> b) -> b) (apply_to a b y)) (None (a -> b))) = Proved

lemma option_ap_ich_some (a : Type) (b : Type) (g : a -> b) (y : a) :
  Equal (Option b) (option_ap a b (Some (a -> b) g) (option_pure a y)) (option_ap (a -> b) b (option_pure ((a -> b) -> b) (apply_to a b y)) (Some (a -> b) g)) = Refl

lemma option_ap_ich (a : Type) (b : Type) (u : Option (a -> b)) (y : a) :
  Equal (Option b) (option_ap a b u (option_pure a y)) (option_ap (a -> b) b (option_pure ((a -> b) -> b) (apply_to a b y)) u) =
  match u { None ↦ option_ap_ich_none a b y ; Some g ↦ option_ap_ich_some a b g y }

lemma option_ap_cmp_none_u (a : Type) (b : Type) (c : Type) (v : Option (a -> b)) (w : Option a) :
  Equal (Option c)
    (option_ap a c (option_ap (a -> b) (a -> c) (option_ap (b -> c) ((a -> b) -> (a -> c)) (option_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) (None (b -> c))) v) w)
    (option_ap b c (None (b -> c)) (option_ap a b v w)) = Proved

lemma option_ap_cmp_some_u_none_v (a : Type) (b : Type) (c : Type) (g : b -> c) (w : Option a) :
  Equal (Option c)
    (option_ap a c (option_ap (a -> b) (a -> c) (option_ap (b -> c) ((a -> b) -> (a -> c)) (option_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) (Some (b -> c) g)) (None (a -> b))) w)
    (option_ap b c (Some (b -> c) g) (option_ap a b (None (a -> b)) w)) = Proved

lemma option_ap_cmp_some_u_some_v_none_w (a : Type) (b : Type) (c : Type) (g : b -> c) (h : a -> b) :
  Equal (Option c)
    (option_ap a c (option_ap (a -> b) (a -> c) (option_ap (b -> c) ((a -> b) -> (a -> c)) (option_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) (Some (b -> c) g)) (Some (a -> b) h)) (None a))
    (option_ap b c (Some (b -> c) g) (option_ap a b (Some (a -> b) h) (None a))) = Proved

lemma option_ap_cmp_all_some (a : Type) (b : Type) (c : Type) (g : b -> c) (h : a -> b) (x : a) :
  Equal (Option c)
    (option_ap a c (option_ap (a -> b) (a -> c) (option_ap (b -> c) ((a -> b) -> (a -> c)) (option_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) (Some (b -> c) g)) (Some (a -> b) h)) (Some a x))
    (option_ap b c (Some (b -> c) g) (option_ap a b (Some (a -> b) h) (Some a x))) = Refl

lemma option_ap_cmp (a : Type) (b : Type) (c : Type) (u : Option (b -> c)) (v : Option (a -> b)) (w : Option a) :
  Equal (Option c)
    (option_ap a c (option_ap (a -> b) (a -> c) (option_ap (b -> c) ((a -> b) -> (a -> c)) (option_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
    (option_ap b c u (option_ap a b v w)) =
  match u {
    None ↦ option_ap_cmp_none_u a b c v w ;
    Some g ↦ match v {
      None ↦ option_ap_cmp_some_u_none_v a b c g w ;
      Some h ↦ match w {
        None ↦ option_ap_cmp_some_u_some_v_none_w a b c g h ;
        Some x ↦ option_ap_cmp_all_some a b c g h x
      }
    }
  }

lemma option_map_coh_none (a : Type) (b : Type) (g : a -> b) :
  Equal (Option b) (functor_map_of Option Functor_instance_Option a b g (None a)) (option_ap a b (option_pure (a -> b) g) (None a)) = Proved

lemma option_map_coh_some (a : Type) (b : Type) (g : a -> b) (v : a) :
  Equal (Option b) (functor_map_of Option Functor_instance_Option a b g (Some a v)) (option_ap a b (option_pure (a -> b) g) (Some a v)) = Refl

lemma option_map_coh (a : Type) (b : Type) (g : a -> b) (x : Option a) :
  Equal (Option b) (functor_map_of Option Functor_instance_Option a b g x) (option_ap a b (option_pure (a -> b) g) x) =
  match x { None ↦ option_map_coh_none a b g ; Some v ↦ option_map_coh_some a b g v }

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
lemma option_bind_lid (a : Type) (b : Type) (x : a) (k : a -> Option b) :
  Equal (Option b) (option_bind a b (applicative_pure_of Option Applicative_instance_Option a x) k) (k x) = Refl

lemma option_bind_rid_none (a : Type) :
  Equal (Option a) (option_bind a a (None a) (applicative_pure_of Option Applicative_instance_Option a)) (None a) = Proved

lemma option_bind_rid_some (a : Type) (x : a) :
  Equal (Option a) (option_bind a a (Some a x) (applicative_pure_of Option Applicative_instance_Option a)) (Some a x) = Refl

lemma option_bind_rid (a : Type) (m : Option a) : Equal (Option a) (option_bind a a m (applicative_pure_of Option Applicative_instance_Option a)) m =
  match m { None ↦ option_bind_rid_none a ; Some x ↦ option_bind_rid_some a x }

lemma option_bind_asc_none (a : Type) (b : Type) (c : Type) (k : a -> Option b) (h : b -> Option c) :
  Equal (Option c) (option_bind b c (option_bind a b (None a) k) h) (option_bind a c (None a) (compose_kleisli Option option_bind a b c k h)) = Proved

lemma option_bind_asc_some (a : Type) (b : Type) (c : Type) (x : a) (k : a -> Option b) (h : b -> Option c) :
  Equal (Option c) (option_bind b c (option_bind a b (Some a x) k) h) (option_bind a c (Some a x) (compose_kleisli Option option_bind a b c k h)) = Refl

lemma option_bind_asc (a : Type) (b : Type) (c : Type) (m : Option a) (k : a -> Option b) (h : b -> Option c) :
  Equal (Option c) (option_bind b c (option_bind a b m k) h) (option_bind a c m (compose_kleisli Option option_bind a b c k h)) =
  match m { None ↦ option_bind_asc_none a b c k h ; Some x ↦ option_bind_asc_some a b c x k h }

instance Monad Option {
  applicative = Applicative_instance_Option ;
  bind = option_bind ;
  bind_lid = option_bind_lid ;
  bind_rid = option_bind_rid ;
  bind_asc = option_bind_asc
}
```

### 2.3 `List` — the cartesian instance, induction throughout

`pure x = [x]`; `ap` is the cartesian product-of-effects; `bind = concat_map`
(chapter `§3.3`/`§4.4`, Fork D — the shape coherent with `Monad List`).
`concat_map` is a straightforward structural recursion built from
`list_append`:

```ken
fn concat_map (a : Type) (b : Type) (f : a -> List b) (xs : List a) : List b =
  match xs { Nil ↦ Nil b ; Cons h t ↦ list_append b (f h) (concat_map a b f t) }

fn list_pure (a : Type) (x : a) : List a = Cons a x (Nil a)

fn list_ap (a : Type) (b : Type) (mf : List (a -> b)) (mx : List a) : List b =
  concat_map (a -> b) b (λg. list_map a b g mx) mf

fn list_bind (a : Type) (b : Type) (m : List a) (k : a -> List b) : List b = concat_map a b k m
```

`list_bind` exists because `concat_map`'s own natural argument order
(function first, matching its `foldr`-shaped recursion) is the OPPOSITE
of `bind`'s field order (container first, per the chapter's own `bind m k
= concat_map k m`) — a real argument-order mismatch, not a cosmetic one
(`§6` Finding).

`bind_lid`/`bind_rid`/`bind_asc` for `List`:

```ken
lemma list_bind_lid (a : Type) (b : Type) (x : a) (k : a -> List b) :
  Equal (List b) (list_bind a b (list_pure a x) k) (k x) = list_append::right_unit b (k x)

lemma list_bind_rid (a : Type) (m : List a) : Equal (List a) (list_bind a a m (list_pure a)) m =
  match m {
    Nil ↦ Proved ;
    Cons h t ↦ cong (List a) (List a) (list_bind a a t (list_pure a)) t (Cons a h) (list_bind_rid a t)
  }

lemma concat_map_append_distrib (a : Type) (b : Type) (f : a -> List b) (xs : List a) (ys : List a) :
  Equal (List b) (concat_map a b f (list_append a xs ys)) (list_append b (concat_map a b f xs) (concat_map a b f ys)) =
  match xs {
    Nil ↦ Refl ;
    Cons h t ↦
      trans (List b)
        (list_append b (f h) (concat_map a b f (list_append a t ys)))
        (list_append b (f h) (list_append b (concat_map a b f t) (concat_map a b f ys)))
        (list_append b (list_append b (f h) (concat_map a b f t)) (concat_map a b f ys))
        (cong (List b) (List b) (concat_map a b f (list_append a t ys)) (list_append b (concat_map a b f t) (concat_map a b f ys)) (list_append b (f h)) (concat_map_append_distrib a b f t ys))
        (sym (List b) (list_append b (list_append b (f h) (concat_map a b f t)) (concat_map a b f ys)) (list_append b (f h) (list_append b (concat_map a b f t) (concat_map a b f ys))) (list_append::assoc b (f h) (concat_map a b f t) (concat_map a b f ys)))
  }

lemma list_bind_asc (a : Type) (b : Type) (c : Type) (m : List a) (k : a -> List b) (h : b -> List c) :
  Equal (List c) (list_bind b c (list_bind a b m k) h) (list_bind a c m (compose_kleisli List list_bind a b c k h)) =
  match m {
    Nil ↦ Proved ;
    Cons h0 t ↦
      trans (List c)
        (concat_map b c h (list_append b (k h0) (concat_map a b k t)))
        (list_append c (concat_map b c h (k h0)) (concat_map b c h (concat_map a b k t)))
        (list_append c (compose_kleisli List list_bind a b c k h h0) (list_bind a c t (compose_kleisli List list_bind a b c k h)))
        (concat_map_append_distrib b c h (k h0) (concat_map a b k t))
        (cong (List c) (List c) (concat_map b c h (concat_map a b k t)) (list_bind a c t (compose_kleisli List list_bind a b c k h)) (list_append c (concat_map b c h (k h0))) (list_bind_asc a b c t k h))
  }
```

`ap_id`/`ap_hom`/`map_coh` for `List` compose with the
`list_append::right_unit`/`list_map::id` (`Core/LawfulFunctors.ken`) — zero new
induction needed for any of the three:

```ken
lemma list_ap_id (a : Type) (v : List a) : Equal (List a) (list_ap a a (list_pure (a -> a) (idf a)) v) v =
  trans (List a)
    (list_append a (list_map a a (idf a) v) (Nil a))
    (list_map a a (idf a) v)
    v
    (list_append::right_unit a (list_map a a (idf a) v))
    (list_map::id a v)

lemma list_ap_hom (a : Type) (b : Type) (g : a -> b) (x : a) :
  Equal (List b) (list_ap a b (list_pure (a -> b) g) (list_pure a x)) (list_pure b (g x)) =
  list_append::right_unit b (Cons b (g x) (Nil b))

lemma list_map_coh (a : Type) (b : Type) (g : a -> b) (x : List a) :
  Equal (List b) (functor_map_of List Functor_instance_List a b g x) (list_ap a b (list_pure (a -> b) g) x) =
  sym (List b) (list_append b (list_map a b g x) (Nil b)) (list_map a b g x) (list_append::right_unit b (list_map a b g x))
```

`ap_ich` needs one real induction. A self-recursive proof stated directly
via `list_ap` does not typecheck at the recursive call (`list_ap`'s own
unfolding to a `list_map` form needs `list_append::right_unit`, a PROOF, not a raw
reduction, so the induction hypothesis's type does not definitionally
match what `cong` needs at each step) — split into the true inductive
content, phrased directly over `concat_map`/`list_map`, and the outer
`list_ap`-phrased lemma composing it with the one `list_append::right_unit` step:

```ken
fn list_ap_inner (a : Type) (b : Type) (y : a) (g : a -> b) : List b = list_map a b g (list_pure a y)

lemma list_ap_ich_general (a : Type) (b : Type) (u : List (a -> b)) (y : a) :
  Equal (List b) (concat_map (a -> b) b (list_ap_inner a b y) u) (list_map (a -> b) b (apply_to a b y) u) =
  match u {
    Nil ↦ Proved ;
    Cons g0 t ↦
      cong (List b) (List b)
        (concat_map (a -> b) b (list_ap_inner a b y) t)
        (list_map (a -> b) b (apply_to a b y) t)
        (Cons b (g0 y))
        (list_ap_ich_general a b t y)
  }

lemma list_ap_ich (a : Type) (b : Type) (u : List (a -> b)) (y : a) :
  Equal (List b) (list_ap a b u (list_pure a y)) (list_ap (a -> b) b (list_pure ((a -> b) -> b) (apply_to a b y)) u) =
  trans (List b)
    (concat_map (a -> b) b (list_ap_inner a b y) u)
    (list_map (a -> b) b (apply_to a b y) u)
    (list_append b (list_map (a -> b) b (apply_to a b y) u) (Nil b))
    (list_ap_ich_general a b u y)
    (sym (List b) (list_append b (list_map (a -> b) b (apply_to a b y) u) (Nil b)) (list_map (a -> b) b (apply_to a b y) u) (list_append::right_unit b (list_map (a -> b) b (apply_to a b y) u)))
```

`ap_cmp` (composition) is the load-bearing law of the four — the standard
"every combination of three lists" associativity fact. `pure f`'s own
`ap` reduces to a plain `list_map` (a fact worth its own name,
`list_ap_pure_left`, since it generalizes both `ap_hom` and the front of
`ap_cmp`); the rest is three "fusion" facts relating `concat_map`/`list_map`
composition, plus the already-proved `list_bind_asc` for the one genuinely
new inductive step (concat_map-after-concat_map):

```ken
lemma list_ap_pure_left (a : Type) (b : Type) (g : a -> b) (xs : List a) :
  Equal (List b) (list_ap a b (list_pure (a -> b) g) xs) (list_map a b g xs) =
  list_append::right_unit b (list_map a b g xs)

lemma list_map_append_distrib (a : Type) (b : Type) (g : a -> b) (xs : List a) (ys : List a) :
  Equal (List b) (list_map a b g (list_append a xs ys)) (list_append b (list_map a b g xs) (list_map a b g ys)) =
  match xs {
    Nil ↦ Refl ;
    Cons h t ↦ cong (List b) (List b) (list_map a b g (list_append a t ys)) (list_append b (list_map a b g t) (list_map a b g ys)) (Cons b (g h)) (list_map_append_distrib a b g t ys)
  }

fn compose_f_g (a : Type) (b : Type) (c : Type) (f : b -> List c) (g : a -> b) (x : a) : List c = f (g x)

lemma concat_map_map_fusion (a : Type) (b : Type) (c : Type) (f : b -> List c) (g : a -> b) (xs : List a) :
  Equal (List c) (concat_map b c f (list_map a b g xs)) (concat_map a c (compose_f_g a b c f g) xs) =
  match xs {
    Nil ↦ Proved ;
    Cons h t ↦ cong (List c) (List c) (concat_map b c f (list_map a b g t)) (concat_map a c (compose_f_g a b c f g) t) (list_append c (f (g h))) (concat_map_map_fusion a b c f g t)
  }

fn map_after (a : Type) (b : Type) (c : Type) (g : b -> c) (f : a -> List b) (x : a) : List c = list_map b c g (f x)

lemma list_map_concat_map_fusion (a : Type) (b : Type) (c : Type) (g : b -> c) (f : a -> List b) (xs : List a) :
  Equal (List c) (list_map b c g (concat_map a b f xs)) (concat_map a c (map_after a b c g f) xs) =
  match xs {
    Nil ↦ Proved ;
    Cons h t ↦
      trans (List c)
        (list_map b c g (list_append b (f h) (concat_map a b f t)))
        (list_append c (list_map b c g (f h)) (list_map b c g (concat_map a b f t)))
        (list_append c (map_after a b c g f h) (concat_map a c (map_after a b c g f) t))
        (list_map_append_distrib b c g (f h) (concat_map a b f t))
        (cong (List c) (List c) (list_map b c g (concat_map a b f t)) (concat_map a c (map_after a b c g f) t) (list_append c (list_map b c g (f h))) (list_map_concat_map_fusion a b c g f t))
  }

lemma concat_map_pointwise_eq (a : Type) (b : Type) (f : a -> List b) (g : a -> List b) (pf : (x : a) -> Equal (List b) (f x) (g x)) (xs : List a) :
  Equal (List b) (concat_map a b f xs) (concat_map a b g xs) =
  match xs {
    Nil ↦ Proved ;
    Cons h t ↦
      trans (List b)
        (list_append b (f h) (concat_map a b f t))
        (list_append b (g h) (concat_map a b f t))
        (list_append b (g h) (concat_map a b g t))
        (cong (List b) (List b) (f h) (g h) (λz. list_append b z (concat_map a b f t)) (pf h))
        (cong (List b) (List b) (concat_map a b f t) (concat_map a b g t) (list_append b (g h)) (concat_map_pointwise_eq a b f g pf t))
  }
```

Assembling `ap_cmp` itself needs three more named accessors (again, the
lambda/`.field`-in-declared-type gap — `§6` Finding) and a three-part
`trans` chain: the FRONT (unfold `pure(compose)` via `list_ap_pure_left`,
lift through the outer `ap` via `cong`, fuse via `concat_map_map_fusion`),
the MIDDLE (`list_bind_asc` for the outer `concat_map`-after-`concat_map`,
`concat_map_map_fusion` again for the inner one), and the END (an inductive
reconciliation of the two remaining `concat_map`/`list_map` orderings,
needing `list_map::fusion` plus
`list_map_append_distrib`):

```ken
fn ap_map_v (a : Type) (b : Type) (c : Type) (v : List (a -> b)) (g1 : b -> c) : List (a -> c) = list_map (a -> b) (a -> c) (compose a b c g1) v

fn ap_map_w (a : Type) (c : Type) (w : List a) (h2 : a -> c) : List c = list_map a c h2 w

lemma list_ap_cmp_front (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) :
  Equal (List (a -> c))
    (list_ap (a -> b) (a -> c) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v)
    (concat_map (b -> c) (a -> c) (ap_map_v a b c v) u) =
  trans (List (a -> c))
    (concat_map ((a -> b) -> (a -> c)) (a -> c) (λh. list_map (a -> b) (a -> c) h v) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u))
    (concat_map ((a -> b) -> (a -> c)) (a -> c) (λh. list_map (a -> b) (a -> c) h v) (list_map (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u))
    (concat_map (b -> c) (a -> c) (λg1. list_map (a -> b) (a -> c) (compose a b c g1) v) u)
    (cong (List ((a -> b) -> (a -> c))) (List (a -> c))
       (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u)
       (list_map (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u)
       (λp. concat_map ((a -> b) -> (a -> c)) (a -> c) (λh. list_map (a -> b) (a -> c) h v) p)
       (list_ap_pure_left (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u))
    (concat_map_map_fusion (b -> c) ((a -> b) -> (a -> c)) (a -> c) (λh. list_map (a -> b) (a -> c) h v) (compose a b c) u)

fn ap_comp_h1 (a : Type) (b : Type) (c : Type) (v : List (a -> b)) (w : List a) (g1 : b -> c) : List c =
  concat_map (a -> b) c (λh1. list_map a c (compose a b c g1 h1) w) v

fn ap_then_bind (a : Type) (b : Type) (c : Type) (v : List (a -> b)) (w : List a) (g1 : b -> c) : List c =
  list_map b c g1 (concat_map (a -> b) b (λh1. list_map a b h1 w) v)

lemma pf_probe (a : Type) (b : Type) (c : Type) (v : List (a -> b)) (w : List a) (g1 : b -> c) :
  Equal (List c) (ap_comp_h1 a b c v w g1) (ap_then_bind a b c v w g1) =
  match v {
    Nil ↦ Proved ;
    Cons h0 t ↦
      trans (List c)
        (list_append c (list_map a c (compose a b c g1 h0) w) (ap_comp_h1 a b c t w g1))
        (list_append c (list_map b c g1 (list_map a b h0 w)) (ap_then_bind a b c t w g1))
        (list_map b c g1 (list_append b (list_map a b h0 w) (concat_map (a -> b) b (λh1. list_map a b h1 w) t)))
        (trans (List c)
           (list_append c (list_map a c (compose a b c g1 h0) w) (ap_comp_h1 a b c t w g1))
           (list_append c (list_map b c g1 (list_map a b h0 w)) (ap_comp_h1 a b c t w g1))
           (list_append c (list_map b c g1 (list_map a b h0 w)) (ap_then_bind a b c t w g1))
           (cong (List c) (List c) (list_map a c (compose a b c g1 h0) w) (list_map b c g1 (list_map a b h0 w)) (λz. list_append c z (ap_comp_h1 a b c t w g1)) (list_map::fusion a b c g1 h0 w))
           (cong (List c) (List c) (ap_comp_h1 a b c t w g1) (ap_then_bind a b c t w g1) (list_append c (list_map b c g1 (list_map a b h0 w))) (pf_probe a b c t w g1)))
        (sym (List c)
           (list_map b c g1 (list_append b (list_map a b h0 w) (concat_map (a -> b) b (λh1. list_map a b h1 w) t)))
           (list_append c (list_map b c g1 (list_map a b h0 w)) (list_map b c g1 (concat_map (a -> b) b (λh1. list_map a b h1 w) t)))
           (list_map_append_distrib b c g1 (list_map a b h0 w) (concat_map (a -> b) b (λh1. list_map a b h1 w) t)))
  }

lemma list_ap_cmp_mid1 (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) (w : List a) :
  Equal (List c)
    (concat_map (a -> c) c (ap_map_w a c w) (concat_map (b -> c) (a -> c) (ap_map_v a b c v) u))
    (concat_map (b -> c) c (ap_comp_h1 a b c v w) u) =
  trans (List c)
    (concat_map (a -> c) c (λh2. list_map a c h2 w) (concat_map (b -> c) (a -> c) (λg1. list_map (a -> b) (a -> c) (compose a b c g1) v) u))
    (concat_map (b -> c) c (λg1. concat_map (a -> c) c (λh2. list_map a c h2 w) (list_map (a -> b) (a -> c) (compose a b c g1) v)) u)
    (concat_map (b -> c) c (λg1. concat_map (a -> b) c (λh1. list_map a c (compose a b c g1 h1) w) v) u)
    (list_bind_asc (b -> c) (a -> c) c u (λg1. list_map (a -> b) (a -> c) (compose a b c g1) v) (λh2. list_map a c h2 w))
    (concat_map_pointwise_eq (b -> c) c
       (λg1. concat_map (a -> c) c (λh2. list_map a c h2 w) (list_map (a -> b) (a -> c) (compose a b c g1) v))
       (λg1. concat_map (a -> b) c (λh1. list_map a c (compose a b c g1 h1) w) v)
       (λg1. concat_map_map_fusion (a -> b) (a -> c) c (λh2. list_map a c h2 w) (compose a b c g1) v)
       u)

lemma list_ap_cmp_mid2 (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) (w : List a) :
  Equal (List c)
    (concat_map (b -> c) c (ap_comp_h1 a b c v w) u)
    (list_ap b c u (list_ap a b v w)) =
  concat_map_pointwise_eq (b -> c) c (ap_comp_h1 a b c v w) (ap_then_bind a b c v w) (pf_probe a b c v w) u

lemma list_ap_cmp_mid (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) (w : List a) :
  Equal (List c)
    (concat_map (a -> c) c (ap_map_w a c w) (concat_map (b -> c) (a -> c) (ap_map_v a b c v) u))
    (list_ap b c u (list_ap a b v w)) =
  trans (List c)
    (concat_map (a -> c) c (ap_map_w a c w) (concat_map (b -> c) (a -> c) (ap_map_v a b c v) u))
    (concat_map (b -> c) c (ap_comp_h1 a b c v w) u)
    (list_ap b c u (list_ap a b v w))
    (list_ap_cmp_mid1 a b c u v w)
    (list_ap_cmp_mid2 a b c u v w)

lemma list_ap_cmp (a : Type) (b : Type) (c : Type) (u : List (b -> c)) (v : List (a -> b)) (w : List a) :
  Equal (List c)
    (list_ap a c (list_ap (a -> b) (a -> c) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
    (list_ap b c u (list_ap a b v w)) =
  trans (List c)
    (list_ap a c (list_ap (a -> b) (a -> c) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
    (concat_map (a -> c) c (λh2. list_map a c h2 w) (concat_map (b -> c) (a -> c) (λg1. list_map (a -> b) (a -> c) (compose a b c g1) v) u))
    (list_ap b c u (list_ap a b v w))
    (cong (List (a -> c)) (List c)
       (list_ap (a -> b) (a -> c) (list_ap (b -> c) ((a -> b) -> (a -> c)) (list_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v)
       (concat_map (b -> c) (a -> c) (λg1. list_map (a -> b) (a -> c) (compose a b c g1) v) u)
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

`Monad`'s fields and laws are satisfied by the interaction-tree `bind`:
`pure := Ret`; `bind_lid` is DEFINITIONAL (`ι` on `Ret` — the elimination
computes immediately, no induction needed); `bind_rid`/`bind_asc` hold by
induction on the tree, the same shape as `List`'s own `bind_rid`/
`bind_asc` above. This entry mints no second `bind` and writes no
`instance Monad (ITree e resp)` — `ITree e resp` is a parametric instance
head (free `e`, `resp`), so a general surface instance is not expressed
here. The effect-system denotation is a lawful monad by construction.

## 3. Using it

```ken example
const list_pure_two : List Nat = list_pure Nat (Suc (Suc Zero))

const list_ap_example : List Nat =
  list_ap Nat Nat (Cons (Nat -> Nat) (Suc) (Nil (Nat -> Nat))) (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat)))

const list_bind_example : List Nat =
  list_bind Nat Nat (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat))) (λx. Cons Nat x (Cons Nat x (Nil Nat)))
```

```ken example
const option_ap_some : Option Nat = option_ap Nat Nat (Some (Nat -> Nat) Suc) (Some Nat Zero)
const option_ap_none : Option Nat = option_ap Nat Nat (None (Nat -> Nat)) (Some Nat Zero)

const option_bind_some : Option Nat = option_bind Nat Nat (Some Nat Zero) (λx. Some Nat (Suc x))
const option_bind_none : Option Nat = option_bind Nat Nat (None Nat) (λx. Some Nat (Suc x))
```

## 4. Laws  proofs

Every `Applicative`/`Monad` law is already a real proof term inside the
instance declarations above (`§2.4`) — class-field checking is the
law-discharge mechanism. A handful of
computation facts about the concrete instances round out the picture:

```ken example
lemma list_bind_lid_at_zero : Equal (List Nat) (list_bind Nat Nat (list_pure Nat Zero) (list_pure Nat)) (list_pure Nat Zero) = list_bind_lid Nat Nat Zero (list_pure Nat)

lemma option_ap_none_short_circuits : Equal (Option Nat) (option_ap Nat Nat (None (Nat -> Nat)) (Some Nat Zero)) (None Nat) = Proved
```

`list_bind_lid_at_zero` is `bind_lid` (already proved generically in `§2.3`)
instantiated at a concrete `x`/`k` — a direct application, not a fresh
proof; every catalog law is reusable this way at any concrete instance.
`option_ap_none_short_circuits` closes with `Proved`: `mf = None` collapses
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
induction was the fix.** The first attempt at the `pf_probe`-shaped
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
(`option_ap_id`, `Option`'s laws) and a recursive one (`pf_probe`, `§2.3`).
The reliable shape throughout this entry: prove each match arm as its own
top-level, directly-ascribed lemma, then dispatch to it from a thin outer
`match`.

**Why `Refl` and `Proved` land where they do.** For an ABSTRACT type parameter,
a constructor endpoint whose payload is itself abstract (e.g. `Some a x`
for abstract `x`) is STUCK, not collapsed — `Refl`, never `Proved`; only a
NULLARY-constructor endpoint (`Nil`/`None`) genuinely collapses to `Top`
and wants `Proved`. This is the guide's own `Proved`-vs-`Refl` discriminator
(`catalog/guide/proof-techniques.ken.md §1`), applied dozens of times
across this entry's `Option` and `List` proofs.

## 6. References

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

## 7. Trust  derivation

1. **Public API.** `Applicative`, `Monad`, `Applicative_instance_Option`,
   `Monad_instance_Option`, `Applicative_instance_List`,
   `Monad_instance_List`, plus every named helper in `§2`.
2. **Source map.**

   | Task | Section |
   |---|---|
   | See the shape | [Definition](#2-definition) |
   | Use it | [Using it](#3-using-it) |
   | Check the computation facts | [Laws  proofs](#4-laws--proofs) |
   | Why the proof shapes are what they are | [Design notes](#5-design-notes) |

3. **Derivation path.** `Applicative` and `Monad` are ordinary class
   declarations. `Option`'s laws use finite case splits; `List`'s laws use
   structural induction together with congruence, composition, and symmetry.
   The `ITree` discussion is a correspondence, not a second implementation.
4. **`trusted_base()` delta.** **Zero.** Every law field is a real,
   kernel-checked proof term; the entry introduces no `Axiom`, primitive, or
   postulate.
5. **Proof families.** `Option` — finite case-split, no induction.
   `List` — structural induction throughout; `ap_cmp` is the deepest
   (induction inside `pf_probe`, composed with three non-recursive fusion
   lemmas and the already-proved `list_bind_asc`).
6. **Consumers.** Effectful programs can use these classes to express
   independent sequencing, dependent sequencing, and traversal.
7. **Validation evidence.** The catalog checks the zero-`Axiom` trust
   posture, discriminating law failures, the single `ITree` binding, and all
   source, example, and rejection fences.

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
their related laws.

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
The purity relation permits a pure implementation in a `proc`-marked field:
a class field's declared purity is an upper bound on what an instance may do,
not a requirement that every instance actually do it.

`List`'s traversal is the standard effect-sequencing fold — `pure Nil`
for the base case, then `ap` combines each mapped `Cons` with the
recursive traversal of the tail:

```ken
fn list_traverse (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (t : a -> g b) (xs : List a) : g (List b) =
  match xs {
    Nil ↦ apg.pure (List b) (Nil b) ;
    Cons h u ↦ apg.ap (List b) (List b) (apg.functor.map b (List b -> List b) (Cons b) (t h)) (list_traverse g apg a b t u)
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
    None ↦ apg.pure (Option b) (None b) ;
    Some x ↦ apg.ap b (Option b) (apg.pure (b -> Option b) (Some b)) (t x)
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
laws closes immediately by the `Proved`/`Refl` discriminator, no induction —
there is no real computation content to a wrapper that only wraps and
unwraps):

```ken
data Identity a = MkIdentity a

fn identity_pure (a : Type) (x : a) : Identity a = MkIdentity a x

fn identity_map (a : Type) (b : Type) (f : a -> b) (x : Identity a) : Identity b =
  match x { MkIdentity v ↦ MkIdentity b (f v) }

fn identity_ap (a : Type) (b : Type) (mf : Identity (a -> b)) (mx : Identity a) : Identity b =
  match mf { MkIdentity f ↦ match mx { MkIdentity x ↦ MkIdentity b (f x) } }

lemma identity_id_law (a : Type) (x : Identity a) : Equal (Identity a) (identity_map a a (idf a) x) x =
  match x { MkIdentity v ↦ Refl }

lemma identity_fusion_law (a : Type) (b : Type) (c : Type) (g : b -> c) (h : a -> b) (x : Identity a) :
  Equal (Identity c) (identity_map a c (comp a b c g h) x) (identity_map b c g (identity_map a b h x)) =
  match x { MkIdentity v ↦ Refl }

lemma identity_ap_id (a : Type) (v : Identity a) : Equal (Identity a) (identity_ap a a (identity_pure (a -> a) (idf a)) v) v =
  match v { MkIdentity x ↦ Refl }

lemma identity_ap_hom (a : Type) (b : Type) (g : a -> b) (x : a) :
  Equal (Identity b) (identity_ap a b (identity_pure (a -> b) g) (identity_pure a x)) (identity_pure b (g x)) =
  Refl

lemma identity_ap_ich (a : Type) (b : Type) (u : Identity (a -> b)) (y : a) :
  Equal (Identity b) (identity_ap a b u (identity_pure a y)) (identity_ap (a -> b) b (identity_pure ((a -> b) -> b) (apply_to a b y)) u) =
  match u { MkIdentity f ↦ Refl }

lemma identity_ap_cmp (a : Type) (b : Type) (c : Type) (u : Identity (b -> c)) (v : Identity (a -> b)) (w : Identity a) :
  Equal (Identity c)
    (identity_ap a c (identity_ap (a -> b) (a -> c) (identity_ap (b -> c) ((a -> b) -> (a -> c)) (identity_pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w)
    (identity_ap b c u (identity_ap a b v w)) =
  match u { MkIdentity g ↦ match v { MkIdentity h ↦ match w { MkIdentity x ↦ Refl } } }

lemma identity_map_coh (a : Type) (b : Type) (g : a -> b) (x : Identity a) :
  Equal (Identity b) (identity_map a b g x) (identity_ap a b (identity_pure (a -> b) g) x) =
  match x { MkIdentity v ↦ Refl }

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
lemma list_traverse_identity_law (a : Type) (xs : List a) :
  Equal (Identity (List a)) (list_traverse Identity Applicative_instance_Identity a a (identity_pure a) xs) (identity_pure (List a) xs) =
  match xs {
    Nil ↦ Proved ;
    Cons h u ↦ cong (Identity (List a)) (Identity (List a))
      (list_traverse Identity Applicative_instance_Identity a a (identity_pure a) u)
      (identity_pure (List a) u)
      (identity_map (List a) (List a) (Cons a h))
      (list_traverse_identity_law a u)
  }

lemma option_traverse_identity_law (a : Type) (mx : Option a) :
  Equal (Identity (Option a)) (option_traverse Identity Applicative_instance_Identity a a (identity_pure a) mx) (identity_pure (Option a) mx) =
  match mx {
    None ↦ Proved ;
    Some x ↦ Refl
  }
```

**Naturality** — an `Applicative` MORPHISM `η : g ⇒ h` (a family
`eta_map : (a:Type) → g a → h a` that commutes with `pure` and `ap`)
commutes with `traverse`: `η(traverse g apg t xs) = traverse h aph (η∘t)
xs`. `η`'s two defining properties are threaded as EXPLICIT parameters
(the same Fork C shape as every dictionary in this entry) rather than a
bundled morphism class. A reusable lemma first:
any such `η` also commutes with plain `functor.map` (derived from `η`'s
own two properties plus `map_coh`, applied identically to close both
instances' `Cons`/`Some` cases below):

```ken
fn applicative_ap_of (g_ty : Type -> Type) (d : Applicative g_ty) (a : Type) (b : Type) (mf : g_ty (a -> b)) (mx : g_ty a) : g_ty b =
  d.ap a b mf mx

fn applicative_map_of (g_ty : Type -> Type) (d : Applicative g_ty) (a : Type) (b : Type) (f : a -> b) (x : g_ty a) : g_ty b =
  d.functor.map a b f x

fn eta_natural_map_pure_term (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (eta_map : (a : Type) -> g a -> h a) (a : Type) (b : Type) (f : a -> b) : h (a -> b) =
  eta_map (a -> b) (applicative_pure_of g apg (a -> b) f)

lemma eta_natural_map_eq1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (eta_map : (a : Type) -> g a -> h a)
  (eta_pure : (a : Type) -> (x : a) -> Equal (h a) (eta_map a (applicative_pure_of g apg a x)) (applicative_pure_of h aph a x))
  (a : Type) (b : Type) (f : a -> b) :
  Equal (h (a -> b)) (eta_natural_map_pure_term g h apg eta_map a b f) (applicative_pure_of h aph (a -> b) f) =
  eta_pure (a -> b) f

lemma eta_natural_map (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (eta_map : (a : Type) -> g a -> h a)
  (eta_pure : (a : Type) -> (x : a) -> Equal (h a) (eta_map a (applicative_pure_of g apg a x)) (applicative_pure_of h aph a x))
  (eta_ap : (a : Type) -> (b : Type) -> (mf : g (a -> b)) -> (mx : g a) -> Equal (h b) (eta_map b (applicative_ap_of g apg a b mf mx)) (applicative_ap_of h aph a b (eta_map (a -> b) mf) (eta_map a mx)))
  (a : Type) (b : Type) (f : a -> b) (x : g a) :
  Equal (h b) (eta_map b (applicative_map_of g apg a b f x)) (applicative_map_of h aph a b f (eta_map a x)) =
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

lemma list_traverse_naturality (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (eta_map : (a : Type) -> g a -> h a)
  (eta_pure : (a : Type) -> (x : a) -> Equal (h a) (eta_map a (applicative_pure_of g apg a x)) (applicative_pure_of h aph a x))
  (eta_ap : (a : Type) -> (b : Type) -> (mf : g (a -> b)) -> (mx : g a) -> Equal (h b) (eta_map b (applicative_ap_of g apg a b mf mx)) (applicative_ap_of h aph a b (eta_map (a -> b) mf) (eta_map a mx)))
  (a : Type) (b : Type) (t : a -> g b) (xs : List a) :
  Equal (h (List b)) (eta_map (List b) (list_traverse g apg a b t xs)) (list_traverse h aph a b (list_traverse_nat_action g h eta_map a b t) xs) =
  match xs {
    Nil ↦ eta_pure (List b) (Nil b) ;
    Cons hd u ↦
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

lemma option_traverse_naturality (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (eta_map : (a : Type) -> g a -> h a)
  (eta_pure : (a : Type) -> (x : a) -> Equal (h a) (eta_map a (applicative_pure_of g apg a x)) (applicative_pure_of h aph a x))
  (eta_ap : (a : Type) -> (b : Type) -> (mf : g (a -> b)) -> (mx : g a) -> Equal (h b) (eta_map b (applicative_ap_of g apg a b mf mx)) (applicative_ap_of h aph a b (eta_map (a -> b) mf) (eta_map a mx)))
  (a : Type) (b : Type) (t : a -> g b) (mx : Option a) :
  Equal (h (Option b)) (eta_map (Option b) (option_traverse g apg a b t mx)) (option_traverse h aph a b (option_traverse_nat_action g h eta_map a b t) mx) =
  match mx {
    None ↦ eta_pure (Option b) (None b) ;
    Some x ↦
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
const list_traverse_identity : Identity (List Nat) =
  list_traverse Identity Applicative_instance_Identity Nat Nat (identity_pure Nat) (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat)))

const option_traverse_some : Identity (Option Nat) =
  option_traverse Identity Applicative_instance_Identity Nat Nat (identity_pure Nat) (Some Nat Zero)
```

### 9.4 `Compose g h` — three of four `Applicative` laws, proved

`Compose g h` is the instrument the composition coherence law (`§5.3`)
needs. It lands cleanly as a `fn`-level type synonym, sidestepping
surface `data` declarations accept only `Type 0` parameters, while a `fn`
type synonym has no such restriction:

```ken
fn Compose (g : Type -> Type) (h : Type -> Type) (a : Type) : Type = g (h a)

fn compose_pure (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (x : a) : Compose g h a =
  apg.pure (h a) (aph.pure a x)

fn compose_ap (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (mf : Compose g h (a -> b)) (mx : Compose g h a) : Compose g h b =
  apg.ap (h a) (h b) (apg.functor.map (h (a -> b)) (h a -> h b) (aph.ap a b) mf) mx
```

**`ap_id`.** Every intermediate equality below needs a named accessor
`fn` wrapping any `.field` projection or bare `λ` that would otherwise
sit in a DECLARED type — the SAME parse restriction `§6`'s Finding
already documents, hit here at much greater depth (this is the recurring
shape throughout `§9.4`, not called out again per lemma):

```ken
fn compose_ap_id_ctx (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (v : Compose g h a) (w : g (h a -> h a)) : Compose g h a = apg.ap (h a) (h a) w v

fn compose_ap_id_fmap_ctx (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (v : Compose g h a) (w : h a -> h a) : Compose g h a = apg.functor.map (h a) (h a) w v

fn compose_ap_id_pure_pure (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) : g (h (a -> a)) =
  apg.pure (h (a -> a)) (aph.pure (a -> a) (idf a))

fn compose_ap_id_map_term (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) : g (h a -> h a) =
  apg.functor.map (h (a -> a)) (h a -> h a) (aph.ap a a) (compose_ap_id_pure_pure g h apg aph a)

fn compose_ap_id_ap_term (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) : g (h a -> h a) =
  apg.ap (h (a -> a)) (h a -> h a) (apg.pure ((h (a -> a)) -> (h a -> h a)) (aph.ap a a)) (compose_ap_id_pure_pure g h apg aph a)

fn compose_ap_id_pure_func (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) : g (h a -> h a) =
  apg.pure (h a -> h a) (aph.ap a a (aph.pure (a -> a) (idf a)))

fn compose_ap_id_func (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) : h a -> h a =
  aph.ap a a (aph.pure (a -> a) (idf a))

lemma compose_ap_id_eq1p (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (v : Compose g h a) :
  Equal (Compose g h a)
    (compose_ap_id_ctx g h apg a v (compose_ap_id_map_term g h apg aph a))
    (compose_ap_id_ctx g h apg a v (compose_ap_id_ap_term g h apg aph a)) =
  cong (g (h a -> h a)) (Compose g h a)
    (compose_ap_id_map_term g h apg aph a)
    (compose_ap_id_ap_term g h apg aph a)
    (compose_ap_id_ctx g h apg a v)
    (apg.map_coh (h (a -> a)) (h a -> h a) (aph.ap a a) (compose_ap_id_pure_pure g h apg aph a))

lemma compose_ap_id_eq2p (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (v : Compose g h a) :
  Equal (Compose g h a)
    (compose_ap_id_ctx g h apg a v (compose_ap_id_ap_term g h apg aph a))
    (compose_ap_id_ctx g h apg a v (compose_ap_id_pure_func g h apg aph a)) =
  cong (g (h a -> h a)) (Compose g h a)
    (compose_ap_id_ap_term g h apg aph a)
    (compose_ap_id_pure_func g h apg aph a)
    (compose_ap_id_ctx g h apg a v)
    (apg.ap_hom (h (a -> a)) (h a -> h a) (aph.ap a a) (aph.pure (a -> a) (idf a)))

lemma compose_ap_id_eq3 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (v : Compose g h a) :
  Equal (Compose g h a)
    (compose_ap_id_ctx g h apg a v (compose_ap_id_pure_func g h apg aph a))
    (compose_ap_id_fmap_ctx g h apg a v (compose_ap_id_func g h aph a)) =
  sym (Compose g h a)
    (compose_ap_id_fmap_ctx g h apg a v (compose_ap_id_func g h aph a))
    (compose_ap_id_ctx g h apg a v (compose_ap_id_pure_func g h apg aph a))
    (apg.map_coh (h a) (h a) (compose_ap_id_func g h aph a) v)

lemma compose_ap_id_eq4 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (v : Compose g h a) :
  Equal (Compose g h a)
    (compose_ap_id_fmap_ctx g h apg a v (compose_ap_id_func g h aph a))
    (compose_ap_id_fmap_ctx g h apg a v (idf (h a))) =
  cong (h a -> h a) (Compose g h a)
    (compose_ap_id_func g h aph a)
    (idf (h a))
    (compose_ap_id_fmap_ctx g h apg a v)
    (aph.ap_id a)

lemma compose_ap_id (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (v : Compose g h a) : Equal (Compose g h a) (compose_ap g h apg aph a a (compose_pure g h apg aph (a -> a) (idf a)) v) v =
  trans (Compose g h a)
    (compose_ap g h apg aph a a (compose_pure g h apg aph (a -> a) (idf a)) v)
    (compose_ap_id_ctx g h apg a v (compose_ap_id_pure_func g h apg aph a))
    v
    (trans (Compose g h a)
       (compose_ap g h apg aph a a (compose_pure g h apg aph (a -> a) (idf a)) v)
       (compose_ap_id_ctx g h apg a v (compose_ap_id_ap_term g h apg aph a))
       (compose_ap_id_ctx g h apg a v (compose_ap_id_pure_func g h apg aph a))
       (compose_ap_id_eq1p g h apg aph a v)
       (compose_ap_id_eq2p g h apg aph a v))
    (trans (Compose g h a)
       (compose_ap_id_ctx g h apg a v (compose_ap_id_pure_func g h apg aph a))
       (compose_ap_id_fmap_ctx g h apg a v (idf (h a)))
       v
       (trans (Compose g h a)
          (compose_ap_id_ctx g h apg a v (compose_ap_id_pure_func g h apg aph a))
          (compose_ap_id_fmap_ctx g h apg a v (compose_ap_id_func g h aph a))
          (compose_ap_id_fmap_ctx g h apg a v (idf (h a)))
          (compose_ap_id_eq3 g h apg aph a v)
          (compose_ap_id_eq4 g h apg aph a v))
       (apg.functor.id_law (h a) v))
```

**`ap_hom`** (a shorter, two-level bridge — both `ap` arguments start
pure, so no `map_coh`-then-fusion detour is needed):

```ken
fn compose_ap_hom_ctx (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (x : a) (w : g (h a -> h b)) : Compose g h b =
  apg.ap (h a) (h b) w (apg.pure (h a) (aph.pure a x))

fn compose_ap_hom_map_term (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) : g (h a -> h b) =
  apg.functor.map (h (a -> b)) (h a -> h b) (aph.ap a b) (apg.pure (h (a -> b)) (aph.pure (a -> b) f))

fn compose_ap_hom_ap_term (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) : g (h a -> h b) =
  apg.ap (h (a -> b)) (h a -> h b) (apg.pure ((h (a -> b)) -> (h a -> h b)) (aph.ap a b)) (apg.pure (h (a -> b)) (aph.pure (a -> b) f))

fn compose_ap_hom_func (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) : h a -> h b =
  aph.ap a b (aph.pure (a -> b) f)

fn compose_ap_hom_pure_term (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) : g (h a -> h b) =
  apg.pure (h a -> h b) (compose_ap_hom_func g h aph a b f)

lemma compose_ap_hom_eq1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (x : a) (f : a -> b) :
  Equal (Compose g h b)
    (compose_ap_hom_ctx g h apg aph a b x (compose_ap_hom_map_term g h apg aph a b f))
    (compose_ap_hom_ctx g h apg aph a b x (compose_ap_hom_ap_term g h apg aph a b f)) =
  cong (g (h a -> h b)) (Compose g h b)
    (compose_ap_hom_map_term g h apg aph a b f)
    (compose_ap_hom_ap_term g h apg aph a b f)
    (compose_ap_hom_ctx g h apg aph a b x)
    (apg.map_coh (h (a -> b)) (h a -> h b) (aph.ap a b) (apg.pure (h (a -> b)) (aph.pure (a -> b) f)))

lemma compose_ap_hom_eq2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (x : a) (f : a -> b) :
  Equal (Compose g h b)
    (compose_ap_hom_ctx g h apg aph a b x (compose_ap_hom_ap_term g h apg aph a b f))
    (compose_ap_hom_ctx g h apg aph a b x (compose_ap_hom_pure_term g h apg aph a b f)) =
  cong (g (h a -> h b)) (Compose g h b)
    (compose_ap_hom_ap_term g h apg aph a b f)
    (compose_ap_hom_pure_term g h apg aph a b f)
    (compose_ap_hom_ctx g h apg aph a b x)
    (apg.ap_hom (h (a -> b)) (h a -> h b) (aph.ap a b) (aph.pure (a -> b) f))

fn compose_ap_hom_purex (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (x : a) : h a =
  aph.pure a x

fn compose_ap_hom_singlewrap (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (b : Type) (y : h b) : Compose g h b =
  apg.pure (h b) y

lemma compose_ap_hom_eq3 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (x : a) (f : a -> b) :
  Equal (Compose g h b)
    (compose_ap_hom_ctx g h apg aph a b x (compose_ap_hom_pure_term g h apg aph a b f))
    (compose_ap_hom_singlewrap g h apg b (compose_ap_hom_func g h aph a b f (compose_ap_hom_purex g h aph a x))) =
  apg.ap_hom (h a) (h b) (compose_ap_hom_func g h aph a b f) (compose_ap_hom_purex g h aph a x)

lemma compose_ap_hom_eq4 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (x : a) (f : a -> b) :
  Equal (Compose g h b)
    (compose_ap_hom_singlewrap g h apg b (compose_ap_hom_func g h aph a b f (compose_ap_hom_purex g h aph a x)))
    (compose_pure g h apg aph b (f x)) =
  cong (h b) (Compose g h b)
    (compose_ap_hom_func g h aph a b f (compose_ap_hom_purex g h aph a x))
    (aph.pure b (f x))
    (compose_ap_hom_singlewrap g h apg b)
    (aph.ap_hom a b f x)

lemma compose_ap_hom (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) (x : a) :
  Equal (Compose g h b) (compose_ap g h apg aph a b (compose_pure g h apg aph (a -> b) f) (compose_pure g h apg aph a x)) (compose_pure g h apg aph b (f x)) =
  trans (Compose g h b)
    (compose_ap g h apg aph a b (compose_pure g h apg aph (a -> b) f) (compose_pure g h apg aph a x))
    (compose_ap_hom_ctx g h apg aph a b x (compose_ap_hom_pure_term g h apg aph a b f))
    (compose_pure g h apg aph b (f x))
    (trans (Compose g h b)
       (compose_ap g h apg aph a b (compose_pure g h apg aph (a -> b) f) (compose_pure g h apg aph a x))
       (compose_ap_hom_ctx g h apg aph a b x (compose_ap_hom_ap_term g h apg aph a b f))
       (compose_ap_hom_ctx g h apg aph a b x (compose_ap_hom_pure_term g h apg aph a b f))
       (compose_ap_hom_eq1 g h apg aph a b x f)
       (compose_ap_hom_eq2 g h apg aph a b x f))
    (trans (Compose g h b)
       (compose_ap_hom_ctx g h apg aph a b x (compose_ap_hom_pure_term g h apg aph a b f))
       (compose_ap_hom_singlewrap g h apg b (compose_ap_hom_func g h aph a b f (compose_ap_hom_purex g h aph a x)))
       (compose_pure g h apg aph b (f x))
       (compose_ap_hom_eq3 g h apg aph a b x f)
       (compose_ap_hom_eq4 g h apg aph a b x f))
```

**`Compose`'s full `Functor` instance** (`compose_map`, `id_law`,
`fusion_law` — needed both in its own right and as `ap_ich`/`map_coh`'s
supporting machinery below):

```ken
fn compose_map (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) (x : Compose g h a) : Compose g h b =
  apg.functor.map (h a) (h b) (aph.functor.map a b f) x

fn compose_map_ctx (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (x : Compose g h a) (w : h a -> h b) : Compose g h b =
  apg.functor.map (h a) (h b) w x

fn compose_map_aph_idmap (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) : h a -> h a =
  aph.functor.map a a (idf a)

lemma compose_map_id_eq1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (x : Compose g h a) :
  Equal (Compose g h a)
    (compose_map_ctx g h apg a a x (compose_map_aph_idmap g h aph a))
    (compose_map_ctx g h apg a a x (idf (h a))) =
  cong (h a -> h a) (Compose g h a)
    (compose_map_aph_idmap g h aph a)
    (idf (h a))
    (compose_map_ctx g h apg a a x)
    (aph.functor.id_law a)

lemma compose_map_id_law (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (x : Compose g h a) :
  Equal (Compose g h a) (compose_map g h apg aph a a (idf a) x) x =
  trans (Compose g h a)
    (compose_map g h apg aph a a (idf a) x)
    (compose_map_ctx g h apg a a x (idf (h a)))
    x
    (compose_map_id_eq1 g h apg aph a x)
    (apg.functor.id_law (h a) x)

fn compose_map_aph_compmap (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (p : b -> c) (q : a -> b) : h a -> h c =
  aph.functor.map a c (comp a b c p q)

fn compose_map_aph_mapcomp (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (p : b -> c) (q : a -> b) : h a -> h c =
  comp (h a) (h b) (h c) (aph.functor.map b c p) (aph.functor.map a b q)

lemma compose_map_fusion_eq1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (p : b -> c) (q : a -> b) (x : Compose g h a) :
  Equal (Compose g h c)
    (compose_map_ctx g h apg a c x (compose_map_aph_compmap g h aph a b c p q))
    (compose_map_ctx g h apg a c x (compose_map_aph_mapcomp g h aph a b c p q)) =
  cong (h a -> h c) (Compose g h c)
    (compose_map_aph_compmap g h aph a b c p q)
    (compose_map_aph_mapcomp g h aph a b c p q)
    (compose_map_ctx g h apg a c x)
    (aph.functor.fusion_law a b c p q)

lemma compose_map_fusion_law (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (p : b -> c) (q : a -> b) (x : Compose g h a) :
  Equal (Compose g h c)
    (compose_map g h apg aph a c (comp a b c p q) x)
    (compose_map g h apg aph b c p (compose_map g h apg aph a b q x)) =
  trans (Compose g h c)
    (compose_map g h apg aph a c (comp a b c p q) x)
    (compose_map_ctx g h apg a c x (comp (h a) (h b) (h c) (aph.functor.map b c p) (aph.functor.map a b q)))
    (compose_map g h apg aph b c p (compose_map g h apg aph a b q x))
    (compose_map_fusion_eq1 g h apg aph a b c p q x)
    (apg.functor.fusion_law (h a) (h b) (h c) (aph.functor.map b c p) (aph.functor.map a b q) x)
```

`compose_map`/`compose_map_id_law`/`compose_map_fusion_law` are
`Functor`'s three fields for `Compose g h` (a FIXED `g`/`h`) — no
`instance Functor (Compose g h)` is declared: probed directly
(`instance Box (Compose g h) { ... }` against a dict-free dummy class),
the head is then rejected because free `g` and `h` in an instance head are
kinded as `Type`, while `Compose` needs `Type -> Type`. Every use in this
entry goes through the explicit-dictionary form directly (`apg`/`aph`
threaded as ordinary parameters), never through instance search — the
same Fork C shape `class Traversable`'s own `traverse`
field uses.

**`ap_ich`** (the interchange law — the deepest of the three proved laws:
bridges `apg`'s own `ap_ich` at the outer level with `aph`'s own `ap_ich`
at the inner level via `map_coh` + `fusion_law`, closing the inner
bridge with `aph.ap_ich` applied POINTWISE and promoted to a
function-level equality by the kernel's `Eq`-at-Pi unfold — `eq_at_pi`,
`ken-kernel/src/obs.rs` — rather than a separate `funext` primitive: a
pointwise law, partially applied one argument short of full, IS ALREADY,
by kernel conversion, a term of the function-level equality type):

```ken
fn compose_ap_ich_uprime (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (mf : Compose g h (a -> b)) : g (h a -> h b) =
  apg.functor.map (h (a -> b)) (h a -> h b) (aph.ap a b) mf

fn compose_ap_ich_ctx (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (w1 : g (h a -> h b)) (w2 : h a) : Compose g h b =
  apg.ap (h a) (h b) w1 (apg.pure (h a) w2)

fn compose_ap_ich_mid1_ctx (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (w1 : g (h a -> h b)) (w2 : h a) : Compose g h b =
  apg.ap (h a -> h b) (h b) (apg.pure ((h a -> h b) -> h b) (apply_to (h a) (h b) w2)) w1

lemma compose_ap_ich_eq1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (w1 : g (h a -> h b)) (w2 : h a) :
  Equal (Compose g h b) (compose_ap_ich_ctx g h apg a b w1 w2) (compose_ap_ich_mid1_ctx g h apg a b w1 w2) =
  apg.ap_ich (h a) (h b) w1 w2

fn compose_ap_ich_mapctx1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (w1 : g (h a -> h b)) (w2 : h a) : Compose g h b =
  apg.functor.map (h a -> h b) (h b) (apply_to (h a) (h b) w2) w1

lemma compose_ap_ich_eq2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (w1 : g (h a -> h b)) (w2 : h a) :
  Equal (Compose g h b) (compose_ap_ich_mid1_ctx g h apg a b w1 w2) (compose_ap_ich_mapctx1 g h apg a b w1 w2) =
  sym (Compose g h b) (compose_ap_ich_mapctx1 g h apg a b w1 w2) (compose_ap_ich_mid1_ctx g h apg a b w1 w2)
    (apg.map_coh (h a -> h b) (h b) (apply_to (h a) (h b) w2) w1)

fn compose_ap_ich_compfuncfn (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (y : a) : h (a -> b) -> h b =
  comp (h (a -> b)) (h a -> h b) (h b) (apply_to (h a) (h b) (compose_ap_hom_purex g h aph a y)) (aph.ap a b)

fn compose_ap_ich_func3fn (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (y : a) : h (a -> b) -> h b =
  aph.ap (a -> b) b (aph.pure ((a -> b) -> b) (apply_to a b y))

fn compose_ap_ich_mapctx3 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (mf : Compose g h (a -> b)) (y : a) : Compose g h b =
  apg.functor.map (h (a -> b)) (h b) (compose_ap_ich_compfuncfn g h aph a b y) mf

lemma compose_ap_ich_eq3 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (mf : Compose g h (a -> b)) (y : a) :
  Equal (Compose g h b)
    (compose_ap_ich_mapctx1 g h apg a b (compose_ap_ich_uprime g h apg aph a b mf) (compose_ap_hom_purex g h aph a y))
    (compose_ap_ich_mapctx3 g h apg aph a b mf y) =
  sym (Compose g h b)
    (compose_ap_ich_mapctx3 g h apg aph a b mf y)
    (compose_ap_ich_mapctx1 g h apg a b (compose_ap_ich_uprime g h apg aph a b mf) (compose_ap_hom_purex g h aph a y))
    (apg.functor.fusion_law (h (a -> b)) (h a -> h b) (h b) (apply_to (h a) (h b) (compose_ap_hom_purex g h aph a y)) (aph.ap a b) mf)

lemma compose_ap_ich_pointwise (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (y : a) (q : h (a -> b)) :
  Equal (h b) (compose_ap_ich_compfuncfn g h aph a b y q) (compose_ap_ich_func3fn g h aph a b y q) =
  aph.ap_ich a b q y

lemma compose_ap_ich_funcs_eq (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (y : a) :
  Equal (h (a -> b) -> h b) (compose_ap_ich_compfuncfn g h aph a b y) (compose_ap_ich_func3fn g h aph a b y) =
  compose_ap_ich_pointwise g h aph a b y

fn compose_ap_ich_fmapctx (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (mf : Compose g h (a -> b)) (w : h (a -> b) -> h b) : Compose g h b =
  apg.functor.map (h (a -> b)) (h b) w mf

lemma compose_ap_ich_eq4 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (mf : Compose g h (a -> b)) (y : a) :
  Equal (Compose g h b)
    (compose_ap_ich_mapctx3 g h apg aph a b mf y)
    (compose_ap_ich_fmapctx g h apg a b mf (compose_ap_ich_func3fn g h aph a b y)) =
  cong (h (a -> b) -> h b) (Compose g h b)
    (compose_ap_ich_compfuncfn g h aph a b y)
    (compose_ap_ich_func3fn g h aph a b y)
    (compose_ap_ich_fmapctx g h apg a b mf)
    (compose_ap_ich_funcs_eq g h aph a b y)

fn compose_ap_ich_innermap (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (y : a) : g (h (a -> b) -> h b) =
  apg.functor.map (h ((a -> b) -> b)) (h (a -> b) -> h b) (aph.ap (a -> b) b) (apg.pure (h ((a -> b) -> b)) (aph.pure ((a -> b) -> b) (apply_to a b y)))

fn compose_ap_ich_innerap (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (y : a) : g (h (a -> b) -> h b) =
  apg.ap (h ((a -> b) -> b)) (h (a -> b) -> h b) (apg.pure ((h ((a -> b) -> b)) -> (h (a -> b) -> h b)) (aph.ap (a -> b) b)) (apg.pure (h ((a -> b) -> b)) (aph.pure ((a -> b) -> b) (apply_to a b y)))

fn compose_ap_ich_innerpure (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (y : a) : g (h (a -> b) -> h b) =
  apg.pure (h (a -> b) -> h b) (compose_ap_ich_func3fn g h aph a b y)

lemma compose_ap_ich_eq_inner1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (y : a) :
  Equal (g (h (a -> b) -> h b)) (compose_ap_ich_innermap g h apg aph a b y) (compose_ap_ich_innerap g h apg aph a b y) =
  apg.map_coh (h ((a -> b) -> b)) (h (a -> b) -> h b) (aph.ap (a -> b) b) (apg.pure (h ((a -> b) -> b)) (aph.pure ((a -> b) -> b) (apply_to a b y)))

lemma compose_ap_ich_eq_inner2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (y : a) :
  Equal (g (h (a -> b) -> h b)) (compose_ap_ich_innerap g h apg aph a b y) (compose_ap_ich_innerpure g h apg aph a b y) =
  apg.ap_hom (h ((a -> b) -> b)) (h (a -> b) -> h b) (aph.ap (a -> b) b) (aph.pure ((a -> b) -> b) (apply_to a b y))

fn compose_ap_ich_target_ctx (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (mf : Compose g h (a -> b)) (w : g (h (a -> b) -> h b)) : Compose g h b =
  apg.ap (h (a -> b)) (h b) w mf

lemma compose_ap_ich_eq_target1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (mf : Compose g h (a -> b)) (y : a) :
  Equal (Compose g h b)
    (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innermap g h apg aph a b y))
    (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerap g h apg aph a b y)) =
  cong (g (h (a -> b) -> h b)) (Compose g h b)
    (compose_ap_ich_innermap g h apg aph a b y)
    (compose_ap_ich_innerap g h apg aph a b y)
    (compose_ap_ich_target_ctx g h apg a b mf)
    (compose_ap_ich_eq_inner1 g h apg aph a b y)

lemma compose_ap_ich_eq_target2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (mf : Compose g h (a -> b)) (y : a) :
  Equal (Compose g h b)
    (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerap g h apg aph a b y))
    (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerpure g h apg aph a b y)) =
  cong (g (h (a -> b) -> h b)) (Compose g h b)
    (compose_ap_ich_innerap g h apg aph a b y)
    (compose_ap_ich_innerpure g h apg aph a b y)
    (compose_ap_ich_target_ctx g h apg a b mf)
    (compose_ap_ich_eq_inner2 g h apg aph a b y)

lemma compose_ap_ich_eq_target3 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (mf : Compose g h (a -> b)) (y : a) :
  Equal (Compose g h b)
    (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerpure g h apg aph a b y))
    (compose_ap_ich_fmapctx g h apg a b mf (compose_ap_ich_func3fn g h aph a b y)) =
  sym (Compose g h b)
    (compose_ap_ich_fmapctx g h apg a b mf (compose_ap_ich_func3fn g h aph a b y))
    (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerpure g h apg aph a b y))
    (apg.map_coh (h (a -> b)) (h b) (compose_ap_ich_func3fn g h aph a b y) mf)

lemma compose_ap_ich (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (mf : Compose g h (a -> b)) (y : a) :
  Equal (Compose g h b)
    (compose_ap g h apg aph a b mf (compose_pure g h apg aph a y))
    (compose_ap g h apg aph (a -> b) b (compose_pure g h apg aph ((a -> b) -> b) (apply_to a b y)) mf) =
  trans (Compose g h b)
    (compose_ap g h apg aph a b mf (compose_pure g h apg aph a y))
    (compose_ap_ich_mapctx3 g h apg aph a b mf y)
    (compose_ap g h apg aph (a -> b) b (compose_pure g h apg aph ((a -> b) -> b) (apply_to a b y)) mf)
    (trans (Compose g h b)
       (compose_ap g h apg aph a b mf (compose_pure g h apg aph a y))
       (compose_ap_ich_mapctx1 g h apg a b (compose_ap_ich_uprime g h apg aph a b mf) (compose_ap_hom_purex g h aph a y))
       (compose_ap_ich_mapctx3 g h apg aph a b mf y)
       (trans (Compose g h b)
          (compose_ap g h apg aph a b mf (compose_pure g h apg aph a y))
          (compose_ap_ich_mid1_ctx g h apg a b (compose_ap_ich_uprime g h apg aph a b mf) (compose_ap_hom_purex g h aph a y))
          (compose_ap_ich_mapctx1 g h apg a b (compose_ap_ich_uprime g h apg aph a b mf) (compose_ap_hom_purex g h aph a y))
          (compose_ap_ich_eq1 g h apg a b (compose_ap_ich_uprime g h apg aph a b mf) (compose_ap_hom_purex g h aph a y))
          (compose_ap_ich_eq2 g h apg a b (compose_ap_ich_uprime g h apg aph a b mf) (compose_ap_hom_purex g h aph a y)))
       (compose_ap_ich_eq3 g h apg aph a b mf y))
    (trans (Compose g h b)
       (compose_ap_ich_mapctx3 g h apg aph a b mf y)
       (compose_ap_ich_fmapctx g h apg a b mf (compose_ap_ich_func3fn g h aph a b y))
       (compose_ap g h apg aph (a -> b) b (compose_pure g h apg aph ((a -> b) -> b) (apply_to a b y)) mf)
       (compose_ap_ich_eq4 g h apg aph a b mf y)
       (trans (Compose g h b)
          (compose_ap_ich_fmapctx g h apg a b mf (compose_ap_ich_func3fn g h aph a b y))
          (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerpure g h apg aph a b y))
          (compose_ap g h apg aph (a -> b) b (compose_pure g h apg aph ((a -> b) -> b) (apply_to a b y)) mf)
          (sym (Compose g h b)
             (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerpure g h apg aph a b y))
             (compose_ap_ich_fmapctx g h apg a b mf (compose_ap_ich_func3fn g h aph a b y))
             (compose_ap_ich_eq_target3 g h apg aph a b mf y))
          (trans (Compose g h b)
             (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerpure g h apg aph a b y))
             (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerap g h apg aph a b y))
             (compose_ap g h apg aph (a -> b) b (compose_pure g h apg aph ((a -> b) -> b) (apply_to a b y)) mf)
             (sym (Compose g h b)
                (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerap g h apg aph a b y))
                (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerpure g h apg aph a b y))
                (compose_ap_ich_eq_target2 g h apg aph a b mf y))
             (sym (Compose g h b)
                (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innermap g h apg aph a b y))
                (compose_ap_ich_target_ctx g h apg a b mf (compose_ap_ich_innerap g h apg aph a b y))
                (compose_ap_ich_eq_target1 g h apg aph a b mf y)))))
```

**`map_coh`** (three steps: `apg`'s own `map_coh`, then `aph`'s `map_coh`
lifted pointwise, then `apg`'s `ap_hom` reconciling the result):

```ken
fn compose_map_coh_ctx (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (x : Compose g h a) (w : g (h a -> h b)) : Compose g h b =
  apg.ap (h a) (h b) w x

fn compose_map_coh_aphmapf (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) : h a -> h b =
  aph.functor.map a b f

fn compose_map_coh_pure1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) : g (h a -> h b) =
  apg.pure (h a -> h b) (compose_map_coh_aphmapf g h aph a b f)

fn compose_map_coh_func2 (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) : h a -> h b =
  aph.ap a b (aph.pure (a -> b) f)

fn compose_map_coh_pure2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) : g (h a -> h b) =
  apg.pure (h a -> h b) (compose_map_coh_func2 g h aph a b f)

lemma compose_map_coh_eq1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) (x : Compose g h a) :
  Equal (Compose g h b)
    (compose_map g h apg aph a b f x)
    (compose_map_coh_ctx g h apg a b x (compose_map_coh_pure1 g h apg aph a b f)) =
  apg.map_coh (h a) (h b) (compose_map_coh_aphmapf g h aph a b f) x

lemma compose_map_coh_eq_inner (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) :
  Equal (g (h a -> h b)) (compose_map_coh_pure1 g h apg aph a b f) (compose_map_coh_pure2 g h apg aph a b f) =
  cong (h a -> h b) (g (h a -> h b))
    (compose_map_coh_aphmapf g h aph a b f)
    (compose_map_coh_func2 g h aph a b f)
    (apg.pure (h a -> h b))
    (aph.map_coh a b f)

lemma compose_map_coh_eq2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) (x : Compose g h a) :
  Equal (Compose g h b)
    (compose_map_coh_ctx g h apg a b x (compose_map_coh_pure1 g h apg aph a b f))
    (compose_map_coh_ctx g h apg a b x (compose_map_coh_pure2 g h apg aph a b f)) =
  cong (g (h a -> h b)) (Compose g h b)
    (compose_map_coh_pure1 g h apg aph a b f)
    (compose_map_coh_pure2 g h apg aph a b f)
    (compose_map_coh_ctx g h apg a b x)
    (compose_map_coh_eq_inner g h apg aph a b f)

lemma compose_map_coh_eq3_inner (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) :
  Equal (g (h a -> h b)) (compose_map_coh_pure2 g h apg aph a b f) (compose_ap_hom_map_term g h apg aph a b f) =
  sym (g (h a -> h b))
    (compose_ap_hom_map_term g h apg aph a b f)
    (compose_map_coh_pure2 g h apg aph a b f)
    (trans (g (h a -> h b))
       (compose_ap_hom_map_term g h apg aph a b f)
       (compose_ap_hom_ap_term g h apg aph a b f)
       (compose_map_coh_pure2 g h apg aph a b f)
       (apg.map_coh (h (a -> b)) (h a -> h b) (aph.ap a b) (apg.pure (h (a -> b)) (aph.pure (a -> b) f)))
       (apg.ap_hom (h (a -> b)) (h a -> h b) (aph.ap a b) (aph.pure (a -> b) f)))

lemma compose_map_coh_eq3 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) (x : Compose g h a) :
  Equal (Compose g h b)
    (compose_map_coh_ctx g h apg a b x (compose_map_coh_pure2 g h apg aph a b f))
    (compose_ap g h apg aph a b (compose_pure g h apg aph (a -> b) f) x) =
  cong (g (h a -> h b)) (Compose g h b)
    (compose_map_coh_pure2 g h apg aph a b f)
    (compose_ap_hom_map_term g h apg aph a b f)
    (compose_map_coh_ctx g h apg a b x)
    (compose_map_coh_eq3_inner g h apg aph a b f)

lemma compose_map_coh (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (f : a -> b) (x : Compose g h a) :
  Equal (Compose g h b) (compose_map g h apg aph a b f x) (compose_ap g h apg aph a b (compose_pure g h apg aph (a -> b) f) x) =
  trans (Compose g h b)
    (compose_map g h apg aph a b f x)
    (compose_map_coh_ctx g h apg a b x (compose_map_coh_pure2 g h apg aph a b f))
    (compose_ap g h apg aph a b (compose_pure g h apg aph (a -> b) f) x)
    (trans (Compose g h b)
       (compose_map g h apg aph a b f x)
       (compose_map_coh_ctx g h apg a b x (compose_map_coh_pure1 g h apg aph a b f))
       (compose_map_coh_ctx g h apg a b x (compose_map_coh_pure2 g h apg aph a b f))
       (compose_map_coh_eq1 g h apg aph a b f x)
       (compose_map_coh_eq2 g h apg aph a b f x))
    (compose_map_coh_eq3 g h apg aph a b f x)
```

### 9.5 `ap_naturality`, and `ap_cmp`'s Level1/Level2 reductions

A general fact, provable ONCE from `apg`'s own laws alone (`map_coh` →
`ap_hom` → `ap_cmp` → `map_coh` back — no `aph` involved), reused for
several of the crossings above and cited by name in `§9.6`'s scope note:
pushing a `functor.map` of a post-composed function through an `ap` is
the same as `ap`-ing first and mapping after.

```ken
fn nat_aux_outer_ctx (g : Type -> Type) (apg : Applicative g) (a : Type) (c : Type) (v : g a) (w : g (a -> c)) : g c =
  apg.ap a c w v

fn nat_aux_lhs_inner (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) (u : g (a -> b)) : g (a -> c) =
  apg.functor.map (a -> b) (a -> c) (compose a b c psi) u

fn nat_aux_pure_composed (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) : g ((a -> b) -> (a -> c)) =
  apg.pure ((a -> b) -> (a -> c)) (compose a b c psi)

fn nat_aux_cmp_inner1b (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) (u : g (a -> b)) : g (a -> c) =
  apg.ap (a -> b) (a -> c) (nat_aux_pure_composed g apg a b c psi) u

fn nat_aux_cmp_inner1 (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) (u : g (a -> b)) : g (a -> c) =
  apg.ap (a -> b) (a -> c) (apg.ap (b -> c) ((a -> b) -> (a -> c)) (apg.pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) (apg.pure (b -> c) psi)) u

lemma nat_aux_eq_a (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) (u : g (a -> b)) :
  Equal (g (a -> c)) (nat_aux_lhs_inner g apg a b c psi u) (nat_aux_cmp_inner1b g apg a b c psi u) =
  apg.map_coh (a -> b) (a -> c) (compose a b c psi) u

fn nat_aux_pure_composed_via_cmp (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) : g ((a -> b) -> (a -> c)) =
  apg.ap (b -> c) ((a -> b) -> (a -> c)) (apg.pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) (apg.pure (b -> c) psi)

lemma nat_aux_eq_b (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) :
  Equal (g ((a -> b) -> (a -> c)))
    (nat_aux_pure_composed g apg a b c psi)
    (nat_aux_pure_composed_via_cmp g apg a b c psi) =
  sym (g ((a -> b) -> (a -> c)))
    (nat_aux_pure_composed_via_cmp g apg a b c psi)
    (nat_aux_pure_composed g apg a b c psi)
    (apg.ap_hom (b -> c) ((a -> b) -> (a -> c)) (compose a b c) psi)

lemma nat_aux_eq_c (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) (u : g (a -> b)) :
  Equal (g (a -> c)) (nat_aux_cmp_inner1b g apg a b c psi u) (nat_aux_cmp_inner1 g apg a b c psi u) =
  cong (g ((a -> b) -> (a -> c))) (g (a -> c))
    (nat_aux_pure_composed g apg a b c psi)
    (nat_aux_pure_composed_via_cmp g apg a b c psi)
    (λw. apg.ap (a -> b) (a -> c) w u)
    (nat_aux_eq_b g apg a b c psi)

lemma nat_aux_eq_lhs (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) (u : g (a -> b)) :
  Equal (g (a -> c)) (nat_aux_lhs_inner g apg a b c psi u) (nat_aux_cmp_inner1 g apg a b c psi u) =
  trans (g (a -> c))
    (nat_aux_lhs_inner g apg a b c psi u)
    (nat_aux_cmp_inner1b g apg a b c psi u)
    (nat_aux_cmp_inner1 g apg a b c psi u)
    (nat_aux_eq_a g apg a b c psi u)
    (nat_aux_eq_c g apg a b c psi u)

fn nat_aux_map_ap (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) (u : g (a -> b)) (v : g a) : g c =
  apg.functor.map b c psi (apg.ap a b u v)

fn nat_aux_ap_pure_ap (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) (u : g (a -> b)) (v : g a) : g c =
  apg.ap b c (apg.pure (b -> c) psi) (apg.ap a b u v)

lemma ap_naturality (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (psi : b -> c) (u : g (a -> b)) (v : g a) :
  Equal (g c)
    (nat_aux_outer_ctx g apg a c v (nat_aux_lhs_inner g apg a b c psi u))
    (nat_aux_map_ap g apg a b c psi u v) =
  trans (g c)
    (nat_aux_outer_ctx g apg a c v (nat_aux_lhs_inner g apg a b c psi u))
    (nat_aux_outer_ctx g apg a c v (nat_aux_cmp_inner1 g apg a b c psi u))
    (nat_aux_map_ap g apg a b c psi u v)
    (cong (g (a -> c)) (g c)
       (nat_aux_lhs_inner g apg a b c psi u)
       (nat_aux_cmp_inner1 g apg a b c psi u)
       (nat_aux_outer_ctx g apg a c v)
       (nat_aux_eq_lhs g apg a b c psi u))
    (trans (g c)
       (nat_aux_outer_ctx g apg a c v (nat_aux_cmp_inner1 g apg a b c psi u))
       (nat_aux_ap_pure_ap g apg a b c psi u v)
       (nat_aux_map_ap g apg a b c psi u v)
       (apg.ap_cmp a b c (apg.pure (b -> c) psi) u v)
       (sym (g c)
          (nat_aux_map_ap g apg a b c psi u v)
          (nat_aux_ap_pure_ap g apg a b c psi u v)
          (apg.map_coh b c psi (apg.ap a b u v))))
```

The arg-2 twin of `ap_naturality`, needed by `§9.6`'s final reconciliation
step: pushing a
`functor.map` of a *pre*-composed function through the SECOND `ap` argument
is the same as `ap`-ing against the raw value first and mapping via a
post-composed accessor after. Proved from `apg`'s own laws alone
(`map_coh` -> `ap_cmp` -> `ap_ich` -> `map_coh` -> `fusion_law`), an aux
lemma, not a claimed public law.

```ken
fn nat2_pure_compose (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (u : g (b -> c)) : g ((a -> b) -> (a -> c)) =
  apg.ap (b -> c) ((a -> b) -> (a -> c)) (apg.pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u

fn nat2_map_compose (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (u : g (b -> c)) : g ((a -> b) -> (a -> c)) =
  apg.functor.map (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u

lemma nat2_map_compose_eq (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (u : g (b -> c)) :
  Equal (g ((a -> b) -> (a -> c))) (nat2_map_compose g apg a b c u) (nat2_pure_compose g apg a b c u) =
  apg.map_coh (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u

fn nat2_pure_phi (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (phi : a -> b) : g (a -> b) =
  apg.pure (a -> b) phi

fn nat2_pure_compose_ap (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) : g (a -> c) =
  apg.ap (a -> b) (a -> c) (nat2_pure_compose g apg a b c u) (nat2_pure_phi g apg a b phi)

fn nat2_apply_map (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (x : g ((a -> b) -> (a -> c))) : g (a -> c) =
  apg.functor.map ((a -> b) -> (a -> c)) (a -> c) (apply_to (a -> b) (a -> c) phi) x

fn nat2_apply_map_ap_pure (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (x : g ((a -> b) -> (a -> c))) : g (a -> c) =
  apg.ap ((a -> b) -> (a -> c)) (a -> c) (apg.pure (((a -> b) -> (a -> c)) -> (a -> c)) (apply_to (a -> b) (a -> c) phi)) x

fn nat2_rhs_func (a : Type) (b : Type) (c : Type) (phi : a -> b) : (b -> c) -> (a -> c) =
  comp (b -> c) ((a -> b) -> (a -> c)) (a -> c) (apply_to (a -> b) (a -> c) phi) (compose a b c)

fn nat2_rhs_inner (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) : g (a -> c) =
  apg.functor.map (b -> c) (a -> c) (nat2_rhs_func a b c phi) u

lemma nat2_ich_eq (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) :
  Equal (g (a -> c))
    (nat2_pure_compose_ap g apg a b c phi u)
    (nat2_apply_map_ap_pure g apg a b c phi (nat2_pure_compose g apg a b c u)) =
  apg.ap_ich (a -> b) (a -> c) (nat2_pure_compose g apg a b c u) phi

lemma nat2_outer_map_coh (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) :
  Equal (g (a -> c))
    (nat2_apply_map g apg a b c phi (nat2_pure_compose g apg a b c u))
    (nat2_apply_map_ap_pure g apg a b c phi (nat2_pure_compose g apg a b c u)) =
  apg.map_coh ((a -> b) -> (a -> c)) (a -> c) (apply_to (a -> b) (a -> c) phi) (nat2_pure_compose g apg a b c u)

lemma nat2_step_ich_to_map (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) :
  Equal (g (a -> c))
    (nat2_pure_compose_ap g apg a b c phi u)
    (nat2_apply_map g apg a b c phi (nat2_pure_compose g apg a b c u)) =
  trans (g (a -> c))
    (nat2_pure_compose_ap g apg a b c phi u)
    (nat2_apply_map_ap_pure g apg a b c phi (nat2_pure_compose g apg a b c u))
    (nat2_apply_map g apg a b c phi (nat2_pure_compose g apg a b c u))
    (nat2_ich_eq g apg a b c phi u)
    (sym (g (a -> c))
      (nat2_apply_map g apg a b c phi (nat2_pure_compose g apg a b c u))
      (nat2_apply_map_ap_pure g apg a b c phi (nat2_pure_compose g apg a b c u))
      (nat2_outer_map_coh g apg a b c phi u))

lemma nat2_step_map_pure_swap (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) :
  Equal (g (a -> c))
    (nat2_apply_map g apg a b c phi (nat2_pure_compose g apg a b c u))
    (nat2_apply_map g apg a b c phi (nat2_map_compose g apg a b c u)) =
  cong (g ((a -> b) -> (a -> c))) (g (a -> c))
    (nat2_pure_compose g apg a b c u)
    (nat2_map_compose g apg a b c u)
    (λy. nat2_apply_map g apg a b c phi y)
    (sym (g ((a -> b) -> (a -> c)))
      (nat2_map_compose g apg a b c u)
      (nat2_pure_compose g apg a b c u)
      (nat2_map_compose_eq g apg a b c u))

lemma nat2_step_fusion (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) :
  Equal (g (a -> c))
    (nat2_apply_map g apg a b c phi (nat2_map_compose g apg a b c u))
    (nat2_rhs_inner g apg a b c phi u) =
  sym (g (a -> c))
    (nat2_rhs_inner g apg a b c phi u)
    (nat2_apply_map g apg a b c phi (nat2_map_compose g apg a b c u))
    (apg.functor.fusion_law (b -> c) ((a -> b) -> (a -> c)) (a -> c) (apply_to (a -> b) (a -> c) phi) (compose a b c) u)

lemma nat2_pure_compose_ap_eq (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) :
  Equal (g (a -> c))
    (nat2_pure_compose_ap g apg a b c phi u)
    (nat2_rhs_inner g apg a b c phi u) =
  trans (g (a -> c))
    (nat2_pure_compose_ap g apg a b c phi u)
    (nat2_apply_map g apg a b c phi (nat2_pure_compose g apg a b c u))
    (nat2_rhs_inner g apg a b c phi u)
    (nat2_step_ich_to_map g apg a b c phi u)
    (trans (g (a -> c))
      (nat2_apply_map g apg a b c phi (nat2_pure_compose g apg a b c u))
      (nat2_apply_map g apg a b c phi (nat2_map_compose g apg a b c u))
      (nat2_rhs_inner g apg a b c phi u)
      (nat2_step_map_pure_swap g apg a b c phi u)
      (nat2_step_fusion g apg a b c phi u))

fn nat2_canonical_lhs (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) (v : g a) : g c =
  apg.ap a c (nat2_pure_compose_ap g apg a b c phi u) v

fn nat2_rhs_apply (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) (v : g a) : g c =
  apg.ap a c (nat2_rhs_inner g apg a b c phi u) v

lemma nat2_canonical_lhs_eq (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) (v : g a) :
  Equal (g c) (nat2_canonical_lhs g apg a b c phi u v) (nat2_rhs_apply g apg a b c phi u v) =
  cong (g (a -> c)) (g c)
    (nat2_pure_compose_ap g apg a b c phi u)
    (nat2_rhs_inner g apg a b c phi u)
    (λy. apg.ap a c y v)
    (nat2_pure_compose_ap_eq g apg a b c phi u)

fn nat2_pure_phi_ap (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (phi : a -> b) (v : g a) : g b =
  apg.ap a b (nat2_pure_phi g apg a b phi) v

fn nat2_ap_u_pure_phi (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (u : g (b -> c)) (phi : a -> b) (v : g a) : g c =
  apg.ap b c u (nat2_pure_phi_ap g apg a b phi v)

lemma nat2_ap_cmp_eq (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) (v : g a) :
  Equal (g c) (nat2_canonical_lhs g apg a b c phi u v) (nat2_ap_u_pure_phi g apg a b c u phi v) =
  apg.ap_cmp a b c u (nat2_pure_phi g apg a b phi) v

fn nat2_map_phi (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (phi : a -> b) (v : g a) : g b =
  apg.functor.map a b phi v

fn nat2_ap_u_map_phi (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (u : g (b -> c)) (phi : a -> b) (v : g a) : g c =
  apg.ap b c u (nat2_map_phi g apg a b phi v)

lemma nat2_pure_phi_eq (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (phi : a -> b) (v : g a) :
  Equal (g b) (nat2_map_phi g apg a b phi v) (nat2_pure_phi_ap g apg a b phi v) =
  apg.map_coh a b phi v

lemma ap_naturality2 (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (phi : a -> b) (u : g (b -> c)) (v : g a) :
  Equal (g c)
    (nat2_ap_u_map_phi g apg a b c u phi v)
    (nat2_rhs_apply g apg a b c phi u v) =
  trans (g c)
    (nat2_ap_u_map_phi g apg a b c u phi v)
    (nat2_ap_u_pure_phi g apg a b c u phi v)
    (nat2_rhs_apply g apg a b c phi u v)
    (cong (g b) (g c)
      (nat2_map_phi g apg a b phi v)
      (nat2_pure_phi_ap g apg a b phi v)
      (λx. apg.ap b c u x)
      (nat2_pure_phi_eq g apg a b phi v))
    (trans (g c)
      (nat2_ap_u_pure_phi g apg a b c u phi v)
      (nat2_canonical_lhs g apg a b c phi u v)
      (nat2_rhs_apply g apg a b c phi u v)
      (sym (g c)
        (nat2_canonical_lhs g apg a b c phi u v)
        (nat2_ap_u_pure_phi g apg a b c u phi v)
        (nat2_ap_cmp_eq g apg a b c phi u v))
      (nat2_canonical_lhs_eq g apg a b c phi u v))
```

`ap_cmp`'s own LHS — three levels of nested `compose_ap` keyed on
`compose a b c` — reduces cleanly for its first two levels using
`compose_map_coh` (above) and `apg`'s `functor.fusion_law`; this was the
foundation for `ap_cmp`'s closing step, followed by the third-level
`aph.ap_cmp` pointwise reconciliation:

```ken
fn cmp_level1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) : Compose g h ((a -> b) -> (a -> c)) =
  compose_ap g h apg aph (b -> c) ((a -> b) -> (a -> c)) (compose_pure g h apg aph ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u

lemma cmp_level1_eq (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  Equal (Compose g h ((a -> b) -> (a -> c))) (cmp_level1 g h apg aph a b c u) (compose_map g h apg aph (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u) =
  sym (Compose g h ((a -> b) -> (a -> c)))
    (compose_map g h apg aph (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u)
    (cmp_level1 g h apg aph a b c u)
    (compose_map_coh g h apg aph (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u)

fn cmp_level2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) : Compose g h (a -> c) =
  compose_ap g h apg aph (a -> b) (a -> c) (cmp_level1 g h apg aph a b c u) v

fn cmp_level2_via_map (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) : Compose g h (a -> c) =
  compose_ap g h apg aph (a -> b) (a -> c) (compose_map g h apg aph (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u) v

lemma cmp_level2_step1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) :
  Equal (Compose g h (a -> c)) (cmp_level2 g h apg aph a b c u v) (cmp_level2_via_map g h apg aph a b c u v) =
  cong (Compose g h ((a -> b) -> (a -> c))) (Compose g h (a -> c))
    (cmp_level1 g h apg aph a b c u)
    (compose_map g h apg aph (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u)
    (λw. compose_ap g h apg aph (a -> b) (a -> c) w v)
    (cmp_level1_eq g h apg aph a b c u)

fn cmp_psi1 (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) : h (b -> c) -> h ((a -> b) -> (a -> c)) =
  aph.functor.map (b -> c) ((a -> b) -> (a -> c)) (compose a b c)

fn cmp_psi3 (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) : h (b -> c) -> (h (a -> b) -> h (a -> c)) =
  comp (h (b -> c)) (h ((a -> b) -> (a -> c))) (h (a -> b) -> h (a -> c)) (aph.ap (a -> b) (a -> c)) (cmp_psi1 g h aph a b c)

fn cmp_level2_raw_ctx (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (v : Compose g h (a -> b)) (w : g (h (a -> b) -> h (a -> c))) : Compose g h (a -> c) =
  apg.ap (h (a -> b)) (h (a -> c)) w v

fn cmp_level2_double_map (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) : g (h (a -> b) -> h (a -> c)) =
  apg.functor.map (h ((a -> b) -> (a -> c))) (h (a -> b) -> h (a -> c)) (aph.ap (a -> b) (a -> c)) (apg.functor.map (h (b -> c)) (h ((a -> b) -> (a -> c))) (cmp_psi1 g h aph a b c) u)

fn cmp_level2_fused_map (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) : g (h (a -> b) -> h (a -> c)) =
  apg.functor.map (h (b -> c)) (h (a -> b) -> h (a -> c)) (cmp_psi3 g h aph a b c) u

lemma cmp_level2_step2a (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) :
  Equal (Compose g h (a -> c)) (cmp_level2_via_map g h apg aph a b c u v) (cmp_level2_raw_ctx g h apg a b c v (cmp_level2_double_map g h apg aph a b c u)) =
  Refl

lemma cmp_level2_step2b (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  Equal (g (h (a -> b) -> h (a -> c))) (cmp_level2_double_map g h apg aph a b c u) (cmp_level2_fused_map g h apg aph a b c u) =
  sym (g (h (a -> b) -> h (a -> c)))
    (cmp_level2_fused_map g h apg aph a b c u)
    (cmp_level2_double_map g h apg aph a b c u)
    (apg.functor.fusion_law (h (b -> c)) (h ((a -> b) -> (a -> c))) (h (a -> b) -> h (a -> c)) (aph.ap (a -> b) (a -> c)) (cmp_psi1 g h aph a b c) u)

lemma cmp_level2_step2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) :
  Equal (Compose g h (a -> c)) (cmp_level2_raw_ctx g h apg a b c v (cmp_level2_double_map g h apg aph a b c u)) (cmp_level2_raw_ctx g h apg a b c v (cmp_level2_fused_map g h apg aph a b c u)) =
  cong (g (h (a -> b) -> h (a -> c))) (Compose g h (a -> c))
    (cmp_level2_double_map g h apg aph a b c u)
    (cmp_level2_fused_map g h apg aph a b c u)
    (cmp_level2_raw_ctx g h apg a b c v)
    (cmp_level2_step2b g h apg aph a b c u)

lemma cmp_level2_reduced (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) :
  Equal (Compose g h (a -> c)) (cmp_level2 g h apg aph a b c u v) (cmp_level2_raw_ctx g h apg a b c v (cmp_level2_fused_map g h apg aph a b c u)) =
  trans (Compose g h (a -> c))
    (cmp_level2 g h apg aph a b c u v)
    (cmp_level2_raw_ctx g h apg a b c v (cmp_level2_double_map g h apg aph a b c u))
    (cmp_level2_raw_ctx g h apg a b c v (cmp_level2_fused_map g h apg aph a b c u))
    (trans (Compose g h (a -> c))
       (cmp_level2 g h apg aph a b c u v)
       (cmp_level2_via_map g h apg aph a b c u v)
       (cmp_level2_raw_ctx g h apg a b c v (cmp_level2_double_map g h apg aph a b c u))
       (cmp_level2_step1 g h apg aph a b c u v)
       (cmp_level2_step2a g h apg aph a b c u v))
    (cmp_level2_step2 g h apg aph a b c u v)
```

### 9.6 Compose `ap_cmp` — proved

The final Compose crossing is discharged by the same explicit-dictionary
operations used above.  Its inner crossing is reduced pointwise by `aph`'s
composition law and then lifted through the outer `apg` applications.

```ken
fn compose_cmp_inner (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : h (b -> c)) (v : h (a -> b)) (w : h a) : h c =
  aph.ap a c (aph.ap (a -> b) (a -> c) (aph.ap (b -> c) ((a -> b) -> (a -> c)) (aph.pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w

fn compose_cmp_inner_rhs (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : h (b -> c)) (v : h (a -> b)) (w : h a) : h c =
  aph.ap b c u (aph.ap a b v w)

lemma compose_cmp_inner_eq (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : h (b -> c)) (v : h (a -> b)) (w : h a) :
  Equal (h c) (compose_cmp_inner g h aph a b c u v w) (compose_cmp_inner_rhs g h aph a b c u v w) =
  aph.ap_cmp a b c u v w

fn compose_cmp_level3_lhs (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : h (b -> c)) (v : h (a -> b)) (w : h a) : h c =
  aph.ap a c (cmp_psi3 g h aph a b c u v) w

fn compose_cmp_level3_mid (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : h (b -> c)) (v : h (a -> b)) (w : h a) : h c =
  aph.ap a c (aph.ap (a -> b) (a -> c) (aph.ap (b -> c) ((a -> b) -> (a -> c)) (aph.pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u) v) w

lemma compose_cmp_level3_map_coh (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : h (b -> c)) (v : h (a -> b)) (w : h a) :
  Equal (h c) (compose_cmp_level3_lhs g h aph a b c u v w) (compose_cmp_level3_mid g h aph a b c u v w) =
  cong (h ((a -> b) -> (a -> c))) (h c)
    (aph.functor.map (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u)
    (aph.ap (b -> c) ((a -> b) -> (a -> c)) (aph.pure ((b -> c) -> (a -> b) -> (a -> c)) (compose a b c)) u)
    (λz. aph.ap a c (aph.ap (a -> b) (a -> c) z v) w)
    (aph.map_coh (b -> c) ((a -> b) -> (a -> c)) (compose a b c) u)

lemma compose_cmp_level3_eq (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : h (b -> c)) (v : h (a -> b)) (w : h a) :
  Equal (h c) (compose_cmp_level3_lhs g h aph a b c u v w) (compose_cmp_inner_rhs g h aph a b c u v w) =
  trans (h c)
    (compose_cmp_level3_lhs g h aph a b c u v w)
    (compose_cmp_level3_mid g h aph a b c u v w)
    (compose_cmp_inner_rhs g h aph a b c u v w)
    (compose_cmp_level3_map_coh g h aph a b c u v w)
    (aph.ap_cmp a b c u v w)

fn compose_cmp_level3_lhs_func (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) : h (b -> c) -> h (a -> b) -> h a -> h c =
  compose_cmp_level3_lhs g h aph a b c

fn compose_cmp_level3_rhs_func (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) : h (b -> c) -> h (a -> b) -> h a -> h c =
  compose_cmp_inner_rhs g h aph a b c

fn compose_cmp_outer_psi (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (z : h (a -> b) -> h (a -> c)) :
  h (a -> b) -> (h a -> h c) =
  λx. λy. aph.ap a c (z x) y

fn compose_cmp_outer_lhs (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) : Compose g h c =
  compose_ap g h apg aph a c (cmp_level2 g h apg aph a b c u v) w

fn compose_cmp_outer_mid1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) : Compose g h c =
  compose_ap g h apg aph a c (cmp_level2_raw_ctx g h apg a b c v (cmp_level2_fused_map g h apg aph a b c u)) w

lemma compose_cmp_outer_step1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) :
  Equal (Compose g h c)
    (compose_cmp_outer_lhs g h apg aph a b c u v w)
    (compose_cmp_outer_mid1 g h apg aph a b c u v w) =
  cong (Compose g h (a -> c)) (Compose g h c)
    (cmp_level2 g h apg aph a b c u v)
    (cmp_level2_raw_ctx g h apg a b c v (cmp_level2_fused_map g h apg aph a b c u))
    (λx. compose_ap g h apg aph a c x w)
    (cmp_level2_reduced g h apg aph a b c u v)

fn compose_cmp_outer_inner (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) : g (h (a -> b) -> h (a -> c)) =
  cmp_level2_fused_map g h apg aph a b c u

fn compose_cmp_outer_mid2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) : Compose g h c =
  apg.ap (h a) (h c)
    (apg.ap (h (a -> b)) (h a -> h c)
      (apg.functor.map (h (a -> b) -> h (a -> c)) (h (a -> b) -> (h a -> h c))
        (compose_cmp_outer_psi g h aph a b c)
        (compose_cmp_outer_inner g h apg aph a b c u))
      v)
    w

lemma compose_cmp_outer_step2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) :
  Equal (Compose g h c)
    (compose_cmp_outer_mid1 g h apg aph a b c u v w)
    (compose_cmp_outer_mid2 g h apg aph a b c u v w) =
  cong (g (h a -> h c)) (Compose g h c)
    (apg.functor.map (h (a -> c)) (h a -> h c) (aph.ap a c)
      (apg.ap (h (a -> b)) (h (a -> c))
        (compose_cmp_outer_inner g h apg aph a b c u) v))
    (apg.ap (h (a -> b)) (h a -> h c)
      (apg.functor.map (h (a -> b) -> h (a -> c)) (h (a -> b) -> (h a -> h c))
        (compose_cmp_outer_psi g h aph a b c)
        (compose_cmp_outer_inner g h apg aph a b c u))
      v)
    (λx. apg.ap (h a) (h c) x w)
    (sym (g (h a -> h c))
      (apg.ap (h (a -> b)) (h a -> h c)
        (apg.functor.map (h (a -> b) -> h (a -> c)) (h (a -> b) -> (h a -> h c))
          (compose_cmp_outer_psi g h aph a b c)
          (compose_cmp_outer_inner g h apg aph a b c u))
        v)
      (apg.functor.map (h (a -> c)) (h a -> h c) (aph.ap a c)
        (apg.ap (h (a -> b)) (h (a -> c))
          (compose_cmp_outer_inner g h apg aph a b c u) v))
      (ap_naturality g apg (h (a -> b)) (h (a -> c)) (h a -> h c)
        (aph.ap a c) (compose_cmp_outer_inner g h apg aph a b c u) v))

fn compose_cmp_outer_func (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) :
  h (b -> c) -> h (a -> b) -> h a -> h c =
  λu. compose_cmp_outer_psi g h aph a b c (cmp_psi3 g h aph a b c u)

fn compose_cmp_outer_rhs_func (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) :
  h (b -> c) -> h (a -> b) -> h a -> h c =
  λu. λv. λw. aph.ap b c u (aph.ap a b v w)

lemma compose_cmp_outer_func_eq (g : Type -> Type) (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) (c : Type) :
  Equal (h (b -> c) -> h (a -> b) -> h a -> h c)
    (compose_cmp_outer_func g h aph a b c)
    (compose_cmp_outer_rhs_func g h aph a b c) =
  compose_cmp_level3_eq g h aph a b c

fn compose_cmp_outer_fused (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  g (h (a -> b) -> (h a -> h c)) =
  apg.functor.map (h (b -> c)) (h (a -> b) -> (h a -> h c))
    (comp (h (b -> c)) (h (a -> b) -> h (a -> c)) (h (a -> b) -> (h a -> h c))
      (compose_cmp_outer_psi g h aph a b c)
      (cmp_psi3 g h aph a b c)) u

fn compose_cmp_outer_unfused (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  g (h (a -> b) -> (h a -> h c)) =
  apg.functor.map (h (a -> b) -> h (a -> c)) (h (a -> b) -> (h a -> h c))
    (compose_cmp_outer_psi g h aph a b c)
    (compose_cmp_outer_inner g h apg aph a b c u)

lemma compose_cmp_outer_fusion_eq (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  Equal (g (h (a -> b) -> (h a -> h c)))
    (compose_cmp_outer_unfused g h apg aph a b c u)
    (compose_cmp_outer_fused g h apg aph a b c u) =
  sym (g (h (a -> b) -> (h a -> h c)))
    (compose_cmp_outer_fused g h apg aph a b c u)
    (compose_cmp_outer_unfused g h apg aph a b c u)
    (apg.functor.fusion_law (h (b -> c)) (h (a -> b) -> h (a -> c)) (h (a -> b) -> (h a -> h c))
      (compose_cmp_outer_psi g h aph a b c) (cmp_psi3 g h aph a b c) u)

fn compose_cmp_outer_rhs_u (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  g (h b -> h c) =
  apg.functor.map (h (b -> c)) (h b -> h c) (aph.ap b c) u

fn compose_cmp_outer_rhs_v (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (v : Compose g h (a -> b)) :
  g (h a -> h b) =
  apg.functor.map (h (a -> b)) (h a -> h b) (aph.ap a b) v

fn compose_cmp_outer_rhs (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) : Compose g h c =
  apg.ap (h b) (h c) (compose_cmp_outer_rhs_u g h apg aph b c u)
    (apg.ap (h a) (h b) (compose_cmp_outer_rhs_v g h apg aph a b v) w)

fn compose_cmp_outer_mapped_rhs (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  g (h (a -> b) -> (h a -> h c)) =
  apg.functor.map (h (b -> c)) (h (a -> b) -> (h a -> h c))
    (compose_cmp_outer_rhs_func g h aph a b c) u

lemma compose_cmp_outer_step3b (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  Equal (g (h (a -> b) -> (h a -> h c)))
    (compose_cmp_outer_fused g h apg aph a b c u)
    (compose_cmp_outer_mapped_rhs g h apg aph a b c u) =
  cong (h (b -> c) -> h (a -> b) -> h a -> h c)
    (g (h (a -> b) -> (h a -> h c)))
    (compose_cmp_outer_func g h aph a b c)
    (compose_cmp_outer_rhs_func g h aph a b c)
    (λx. apg.functor.map (h (b -> c)) (h (a -> b) -> (h a -> h c)) x u)
    (compose_cmp_outer_func_eq g h aph a b c)

fn compose_cmp_outer_apply_v (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (v : Compose g h (a -> b)) (w : Compose g h a) (x : g (h (a -> b) -> (h a -> h c))) : Compose g h c =
  apg.ap (h a) (h c) (apg.ap (h (a -> b)) (h a -> h c) x v) w

lemma compose_cmp_outer_step3a (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) :
  Equal (Compose g h c)
    (compose_cmp_outer_mid2 g h apg aph a b c u v w)
    (compose_cmp_outer_apply_v g h apg a b c v w (compose_cmp_outer_fused g h apg aph a b c u)) =
  cong (g (h (a -> b) -> (h a -> h c))) (Compose g h c)
    (compose_cmp_outer_unfused g h apg aph a b c u)
    (compose_cmp_outer_fused g h apg aph a b c u)
    (compose_cmp_outer_apply_v g h apg a b c v w)
    (compose_cmp_outer_fusion_eq g h apg aph a b c u)

lemma compose_cmp_outer_step3 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) :
  Equal (Compose g h c)
    (compose_cmp_outer_apply_v g h apg a b c v w (compose_cmp_outer_fused g h apg aph a b c u))
    (compose_cmp_outer_apply_v g h apg a b c v w (compose_cmp_outer_mapped_rhs g h apg aph a b c u)) =
  cong (g (h (a -> b) -> (h a -> h c))) (Compose g h c)
    (compose_cmp_outer_fused g h apg aph a b c u)
    (compose_cmp_outer_mapped_rhs g h apg aph a b c u)
    (compose_cmp_outer_apply_v g h apg a b c v w)
    (compose_cmp_outer_step3b g h apg aph a b c u)

-- The reconciliation uses `ap_naturality2`: `mapped_rhs`
-- is a single `map` over `u` of a function that internally routes BOTH `u`'s
-- and `v`'s content through `aph.ap`; splitting it back into the `uP`/`vP`
-- shape `apg.ap_cmp` needs takes a `u`-side `fusion_law` factoring (two
-- nested `map`s collapse to the one `mapped_rhs` already is, by construction)
-- plus a `v`-side `ap_naturality2` application, then one final `map_coh` to
-- match `ap_cmp`'s own `pure`-based statement.

fn cmp_v_ap_ab (h : Type -> Type) (aph : Applicative h) (a : Type) (b : Type) : h (a -> b) -> (h a -> h b) =
  aph.ap a b

fn cmp_v_bridge_mid (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) : g ((h a -> h b) -> (h a -> h c)) =
  nat2_map_compose g apg (h a) (h b) (h c) (compose_cmp_outer_rhs_u g h apg aph b c u)

fn cmp_v_bridge_inner_raw (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) : g ((h a -> h b) -> (h a -> h c)) =
  apg.functor.map (h (b -> c)) ((h a -> h b) -> (h a -> h c))
    (comp (h (b -> c)) (h b -> h c) ((h a -> h b) -> (h a -> h c)) (compose (h a) (h b) (h c)) (aph.ap b c)) u

lemma cmp_v_bridge_step1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  Equal (g ((h a -> h b) -> (h a -> h c)))
    (cmp_v_bridge_inner_raw g h apg aph a b c u)
    (cmp_v_bridge_mid g h apg aph a b c u) =
  apg.functor.fusion_law (h (b -> c)) (h b -> h c) ((h a -> h b) -> (h a -> h c)) (compose (h a) (h b) (h c)) (aph.ap b c) u

fn cmp_v_bridge_outer (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) : g (h (a -> b) -> (h a -> h c)) =
  nat2_rhs_inner g apg (h (a -> b)) (h a -> h b) (h a -> h c) (aph.ap a b) (cmp_v_bridge_mid g h apg aph a b c u)

fn cmp_v_bridge_outer_raw (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) : g (h (a -> b) -> (h a -> h c)) =
  apg.functor.map (h (b -> c)) (h (a -> b) -> (h a -> h c))
    (comp (h (b -> c)) ((h a -> h b) -> (h a -> h c)) (h (a -> b) -> (h a -> h c))
      (nat2_rhs_func (h (a -> b)) (h a -> h b) (h a -> h c) (aph.ap a b))
      (comp (h (b -> c)) (h b -> h c) ((h a -> h b) -> (h a -> h c)) (compose (h a) (h b) (h c)) (aph.ap b c)))
    u

lemma cmp_v_bridge_step2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  Equal (g (h (a -> b) -> (h a -> h c)))
    (cmp_v_bridge_outer_raw g h apg aph a b c u)
    (nat2_rhs_inner g apg (h (a -> b)) (h a -> h b) (h a -> h c) (cmp_v_ap_ab h aph a b) (cmp_v_bridge_inner_raw g h apg aph a b c u)) =
  apg.functor.fusion_law (h (b -> c)) ((h a -> h b) -> (h a -> h c)) (h (a -> b) -> (h a -> h c))
    (nat2_rhs_func (h (a -> b)) (h a -> h b) (h a -> h c) (aph.ap a b))
    (comp (h (b -> c)) (h b -> h c) ((h a -> h b) -> (h a -> h c)) (compose (h a) (h b) (h c)) (aph.ap b c))
    u

lemma cmp_v_bridge_step2b (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  Equal (g (h (a -> b) -> (h a -> h c)))
    (nat2_rhs_inner g apg (h (a -> b)) (h a -> h b) (h a -> h c) (cmp_v_ap_ab h aph a b) (cmp_v_bridge_inner_raw g h apg aph a b c u))
    (cmp_v_bridge_outer g h apg aph a b c u) =
  cong (g ((h a -> h b) -> (h a -> h c))) (g (h (a -> b) -> (h a -> h c)))
    (cmp_v_bridge_inner_raw g h apg aph a b c u)
    (cmp_v_bridge_mid g h apg aph a b c u)
    (λy. nat2_rhs_inner g apg (h (a -> b)) (h a -> h b) (h a -> h c) (aph.ap a b) y)
    (cmp_v_bridge_step1 g h apg aph a b c u)

lemma cmp_v_bridge_defeq (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  Equal (g (h (a -> b) -> (h a -> h c)))
    (compose_cmp_outer_mapped_rhs g h apg aph a b c u)
    (cmp_v_bridge_outer_raw g h apg aph a b c u) =
  Refl

lemma cmp_v_bridge_eq (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) :
  Equal (g (h (a -> b) -> (h a -> h c)))
    (compose_cmp_outer_mapped_rhs g h apg aph a b c u)
    (cmp_v_bridge_outer g h apg aph a b c u) =
  trans (g (h (a -> b) -> (h a -> h c)))
    (compose_cmp_outer_mapped_rhs g h apg aph a b c u)
    (cmp_v_bridge_outer_raw g h apg aph a b c u)
    (cmp_v_bridge_outer g h apg aph a b c u)
    (cmp_v_bridge_defeq g h apg aph a b c u)
    (trans (g (h (a -> b) -> (h a -> h c)))
      (cmp_v_bridge_outer_raw g h apg aph a b c u)
      (nat2_rhs_inner g apg (h (a -> b)) (h a -> h b) (h a -> h c) (aph.ap a b) (cmp_v_bridge_inner_raw g h apg aph a b c u))
      (cmp_v_bridge_outer g h apg aph a b c u)
      (cmp_v_bridge_step2 g h apg aph a b c u)
      (cmp_v_bridge_step2b g h apg aph a b c u))

fn cmp_v_bridge_mapped_rhs_apply_v (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) : g (h a -> h c) =
  apg.ap (h (a -> b)) (h a -> h c) (compose_cmp_outer_mapped_rhs g h apg aph a b c u) v

fn cmp_v_bridge_apply_v (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) : g (h a -> h c) =
  apg.ap (h (a -> b)) (h a -> h c) (cmp_v_bridge_outer g h apg aph a b c u) v

fn cmp_v_bridge_apply_v_mid (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) : g (h a -> h c) =
  apg.ap (h a -> h b) (h a -> h c) (cmp_v_bridge_mid g h apg aph a b c u) (compose_cmp_outer_rhs_v g h apg aph a b v)

lemma cmp_v_bridge_mapped_apply_v_eq (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) :
  Equal (g (h a -> h c))
    (cmp_v_bridge_mapped_rhs_apply_v g h apg aph a b c u v)
    (cmp_v_bridge_apply_v g h apg aph a b c u v) =
  cong (g (h (a -> b) -> (h a -> h c))) (g (h a -> h c))
    (compose_cmp_outer_mapped_rhs g h apg aph a b c u)
    (cmp_v_bridge_outer g h apg aph a b c u)
    (λy. apg.ap (h (a -> b)) (h a -> h c) y v)
    (cmp_v_bridge_eq g h apg aph a b c u)

lemma cmp_v_bridge_nat2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) :
  Equal (g (h a -> h c))
    (cmp_v_bridge_apply_v_mid g h apg aph a b c u v)
    (cmp_v_bridge_apply_v g h apg aph a b c u v) =
  ap_naturality2 g apg (h (a -> b)) (h a -> h b) (h a -> h c) (aph.ap a b) (cmp_v_bridge_mid g h apg aph a b c u) v

lemma cmp_v_bridge_full_v_eq (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) :
  Equal (g (h a -> h c))
    (cmp_v_bridge_mapped_rhs_apply_v g h apg aph a b c u v)
    (cmp_v_bridge_apply_v_mid g h apg aph a b c u v) =
  trans (g (h a -> h c))
    (cmp_v_bridge_mapped_rhs_apply_v g h apg aph a b c u v)
    (cmp_v_bridge_apply_v g h apg aph a b c u v)
    (cmp_v_bridge_apply_v_mid g h apg aph a b c u v)
    (cmp_v_bridge_mapped_apply_v_eq g h apg aph a b c u v)
    (sym (g (h a -> h c))
      (cmp_v_bridge_apply_v_mid g h apg aph a b c u v)
      (cmp_v_bridge_apply_v g h apg aph a b c u v)
      (cmp_v_bridge_nat2 g h apg aph a b c u v))

fn cmp_v_bridge_shape (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (v : Compose g h (a -> b)) (w : Compose g h a) (x : g ((h a -> h b) -> (h a -> h c))) : g (h c) =
  apg.ap (h a) (h c) (apg.ap (h a -> h b) (h a -> h c) x (compose_cmp_outer_rhs_v g h apg aph a b v)) w

fn cmp_v_bridge_map_form (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) : g (h c) =
  cmp_v_bridge_shape g h apg aph a b c v w (cmp_v_bridge_mid g h apg aph a b c u)

fn cmp_v_bridge_pure_form (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) : g (h c) =
  cmp_v_bridge_shape g h apg aph a b c v w (nat2_pure_compose g apg (h a) (h b) (h c) (compose_cmp_outer_rhs_u g h apg aph b c u))

lemma cmp_v_bridge_map_pure_eq (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) :
  Equal (g (h c))
    (cmp_v_bridge_map_form g h apg aph a b c u v w)
    (cmp_v_bridge_pure_form g h apg aph a b c u v w) =
  cong (g ((h a -> h b) -> (h a -> h c))) (g (h c))
    (cmp_v_bridge_mid g h apg aph a b c u)
    (nat2_pure_compose g apg (h a) (h b) (h c) (compose_cmp_outer_rhs_u g h apg aph b c u))
    (λy. cmp_v_bridge_shape g h apg aph a b c v w y)
    (nat2_map_compose_eq g apg (h a) (h b) (h c) (compose_cmp_outer_rhs_u g h apg aph b c u))

lemma cmp_v_bridge_ap_cmp (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) :
  Equal (g (h c))
    (cmp_v_bridge_pure_form g h apg aph a b c u v w)
    (compose_cmp_outer_rhs g h apg aph a b c u v w) =
  apg.ap_cmp (h a) (h b) (h c)
    (compose_cmp_outer_rhs_u g h apg aph b c u)
    (compose_cmp_outer_rhs_v g h apg aph a b v)
    w

lemma cmp_v_bridge_outer_cong (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) :
  Equal (g (h c))
    (compose_cmp_outer_apply_v g h apg a b c v w (compose_cmp_outer_mapped_rhs g h apg aph a b c u))
    (cmp_v_bridge_map_form g h apg aph a b c u v w) =
  cong (g (h a -> h c)) (g (h c))
    (cmp_v_bridge_mapped_rhs_apply_v g h apg aph a b c u v)
    (cmp_v_bridge_apply_v_mid g h apg aph a b c u v)
    (λy. apg.ap (h a) (h c) y w)
    (cmp_v_bridge_full_v_eq g h apg aph a b c u v)

lemma compose_cmp_outer_rhs_stage1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) :
  Equal (Compose g h c)
    (compose_cmp_outer_apply_v g h apg a b c v w
      (compose_cmp_outer_mapped_rhs g h apg aph a b c u))
    (compose_cmp_outer_rhs g h apg aph a b c u v w) =
  trans (g (h c))
    (compose_cmp_outer_apply_v g h apg a b c v w (compose_cmp_outer_mapped_rhs g h apg aph a b c u))
    (cmp_v_bridge_map_form g h apg aph a b c u v w)
    (compose_cmp_outer_rhs g h apg aph a b c u v w)
    (cmp_v_bridge_outer_cong g h apg aph a b c u v w)
    (trans (g (h c))
      (cmp_v_bridge_map_form g h apg aph a b c u v w)
      (cmp_v_bridge_pure_form g h apg aph a b c u v w)
      (compose_cmp_outer_rhs g h apg aph a b c u v w)
      (cmp_v_bridge_map_pure_eq g h apg aph a b c u v w)
      (cmp_v_bridge_ap_cmp g h apg aph a b c u v w))

lemma compose_ap_cmp (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (u : Compose g h (b -> c)) (v : Compose g h (a -> b)) (w : Compose g h a) :
  Equal (Compose g h c)
    (compose_cmp_outer_lhs g h apg aph a b c u v w)
    (compose_cmp_outer_rhs g h apg aph a b c u v w) =
  trans (Compose g h c)
    (compose_cmp_outer_lhs g h apg aph a b c u v w)
    (compose_cmp_outer_mid1 g h apg aph a b c u v w)
    (compose_cmp_outer_rhs g h apg aph a b c u v w)
    (compose_cmp_outer_step1 g h apg aph a b c u v w)
    (trans (Compose g h c)
      (compose_cmp_outer_mid1 g h apg aph a b c u v w)
      (compose_cmp_outer_mid2 g h apg aph a b c u v w)
      (compose_cmp_outer_rhs g h apg aph a b c u v w)
      (compose_cmp_outer_step2 g h apg aph a b c u v w)
      (trans (Compose g h c)
        (compose_cmp_outer_mid2 g h apg aph a b c u v w)
        (compose_cmp_outer_apply_v g h apg a b c v w
          (compose_cmp_outer_fused g h apg aph a b c u))
        (compose_cmp_outer_rhs g h apg aph a b c u v w)
        (compose_cmp_outer_step3a g h apg aph a b c u v w)
        (trans (Compose g h c)
          (compose_cmp_outer_apply_v g h apg a b c v w
            (compose_cmp_outer_fused g h apg aph a b c u))
          (compose_cmp_outer_apply_v g h apg a b c v w
            (compose_cmp_outer_mapped_rhs g h apg aph a b c u))
          (compose_cmp_outer_rhs g h apg aph a b c u v w)
          (compose_cmp_outer_step3 g h apg aph a b c u v w)
          (compose_cmp_outer_rhs_stage1 g h apg aph a b c u v w))))


```

The remaining outer lifting is assembled in the next checked block, using
`cong` over `functor.map` and `apg.ap_cmp`; no surface `Applicative`
instance for `Compose` is introduced.

### 9.7 The `traverse` composition law (proved)

`§5.3`'s composition law — `traverse (Compose ∘ map t2 ∘ t1) ≡ Compose ∘
map (traverse t2) ∘ traverse t1` — is stated over the SAME explicit
`compose_pure`/`compose_ap` operations as every other `Compose`-typed law
in this entry, never through instance search (no `instance Applicative
(Compose g h)` — that head stays kinding-blocked, `§9.4`). The composed
action and the LHS/RHS of the law are named directly against `compose_pure`
/`compose_ap` — `list_traverse`/`option_traverse` instantiated "at
`Compose g h`" is simply their own recursion re-run with `compose_pure`/
`compose_ap` substituted for `apg.pure`/`apg.ap`, which is exactly what
`list_traverse_composed`/`option_traverse_composed` below spell out
directly, sidestepping the kinding wall entirely (there is no dict to
construct):

```ken
fn cmp_traverse_action (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) : Compose g h c =
  apg.functor.map b (h c) t2 (t1 x)
```

**`Option`** — no induction, a two-arm case split (mirrors `§9.2`'s
`option_traverse_identity_law`/`option_traverse_naturality` shape). `None`
closes via `map_coh` then `ap_hom` (an outer `map` over a `pure` collapses
to a `pure`, then `option_traverse`'s own `None`-arm reduction on the
result matches `compose_pure` definitionally); `Some` closes via
`map_coh` (turning `ap(pure(Some b))(t1 x)` back into a plain `map`) then
`fusion_law` (fusing the two `map`s into one over `t1 x`), landing on a
function that is `option_traverse`'s own `Some`-arm reduction — matched
definitionally, no extra step:

```ken
fn option_traverse_composed (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (mx : Option a) : Compose g h (Option c) =
  match mx {
    None ↦ compose_pure g h apg aph (Option c) (None c) ;
    Some x ↦ compose_ap g h apg aph c (Option c) (compose_pure g h apg aph (c -> Option c) (Some c)) (cmp_traverse_action g h apg a b c t1 t2 x)
  }

fn option_traverse_composed_rhs (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (mx : Option a) : Compose g h (Option c) =
  apg.functor.map (Option b) (h (Option c)) (option_traverse h aph b c t2) (option_traverse g apg a b t1 mx)

fn otc_none_ap_pure_form (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) : Compose g h (Option c) =
  apg.ap (Option b) (h (Option c)) (apg.pure (Option b -> h (Option c)) (option_traverse h aph b c t2)) (apg.pure (Option b) (None b))

lemma otc_none_step1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) :
  Equal (Compose g h (Option c))
    (option_traverse_composed_rhs g h apg aph a b c t1 t2 (None a))
    (otc_none_ap_pure_form g h apg aph a b c t1 t2) =
  apg.map_coh (Option b) (h (Option c)) (option_traverse h aph b c t2) (apg.pure (Option b) (None b))

lemma otc_none_step2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) :
  Equal (Compose g h (Option c))
    (otc_none_ap_pure_form g h apg aph a b c t1 t2)
    (option_traverse_composed g h apg aph a b c t1 t2 (None a)) =
  apg.ap_hom (Option b) (h (Option c)) (option_traverse h aph b c t2) (None b)

lemma option_traverse_composition_none (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) :
  Equal (Compose g h (Option c))
    (option_traverse_composed g h apg aph a b c t1 t2 (None a))
    (option_traverse_composed_rhs g h apg aph a b c t1 t2 (None a)) =
  sym (Compose g h (Option c))
    (option_traverse_composed_rhs g h apg aph a b c t1 t2 (None a))
    (option_traverse_composed g h apg aph a b c t1 t2 (None a))
    (trans (Compose g h (Option c))
      (option_traverse_composed_rhs g h apg aph a b c t1 t2 (None a))
      (otc_none_ap_pure_form g h apg aph a b c t1 t2)
      (option_traverse_composed g h apg aph a b c t1 t2 (None a))
      (otc_none_step1 g h apg aph a b c t1 t2)
      (otc_none_step2 g h apg aph a b c t1 t2))

fn otc_some_map_pure (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (t1 : a -> g b) (x : a) : g (Option b) =
  apg.functor.map b (Option b) (Some b) (t1 x)

fn otc_some_ap_pure (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (t1 : a -> g b) (x : a) : g (Option b) =
  apg.ap b (Option b) (apg.pure (b -> Option b) (Some b)) (t1 x)

lemma otc_some_stepA (g : Type -> Type) (apg : Applicative g) (a : Type) (b : Type) (t1 : a -> g b) (x : a) :
  Equal (g (Option b))
    (otc_some_ap_pure g apg a b t1 x)
    (otc_some_map_pure g apg a b t1 x) =
  sym (g (Option b))
    (otc_some_map_pure g apg a b t1 x)
    (otc_some_ap_pure g apg a b t1 x)
    (apg.map_coh b (Option b) (Some b) (t1 x))

fn otc_some_rhs (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) : Compose g h (Option c) =
  apg.functor.map (Option b) (h (Option c)) (option_traverse h aph b c t2) (otc_some_ap_pure g apg a b t1 x)

fn otc_some_mapped (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) : Compose g h (Option c) =
  apg.functor.map (Option b) (h (Option c)) (option_traverse h aph b c t2) (otc_some_map_pure g apg a b t1 x)

lemma otc_some_stepB (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) :
  Equal (Compose g h (Option c))
    (otc_some_rhs g h apg aph a b c t1 t2 x)
    (otc_some_mapped g h apg aph a b c t1 t2 x) =
  cong (g (Option b)) (Compose g h (Option c))
    (apg.ap b (Option b) (apg.pure (b -> Option b) (Some b)) (t1 x))
    (otc_some_map_pure g apg a b t1 x)
    (λy. apg.functor.map (Option b) (h (Option c)) (option_traverse h aph b c t2) y)
    (otc_some_stepA g apg a b t1 x)

fn otc_some_fused (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) : Compose g h (Option c) =
  apg.functor.map b (h (Option c)) (comp b (Option b) (h (Option c)) (option_traverse h aph b c t2) (Some b)) (t1 x)

lemma otc_some_stepC (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) :
  Equal (Compose g h (Option c))
    (otc_some_mapped g h apg aph a b c t1 t2 x)
    (otc_some_fused g h apg aph a b c t1 t2 x) =
  sym (Compose g h (Option c))
    (otc_some_fused g h apg aph a b c t1 t2 x)
    (otc_some_mapped g h apg aph a b c t1 t2 x)
    (apg.functor.fusion_law b (Option b) (h (Option c)) (option_traverse h aph b c t2) (Some b) (t1 x))

-- `option_traverse_composed`'s own Some-arm goes through `compose_ap`
-- wrapping a `compose_pure`d function — an ABSTRACT `apg`/`aph`-level
-- reduction, never free for an opaque dict; bridged via `compose_map_coh`
-- (this entry's own law, `§9.4`), not assumed.
fn otc_some_compose_map_form (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) : Compose g h (Option c) =
  compose_map g h apg aph c (Option c) (Some c) (cmp_traverse_action g h apg a b c t1 t2 x)

lemma otc_some_step0 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) :
  Equal (Compose g h (Option c))
    (option_traverse_composed g h apg aph a b c t1 t2 (Some a x))
    (otc_some_compose_map_form g h apg aph a b c t1 t2 x) =
  sym (Compose g h (Option c))
    (otc_some_compose_map_form g h apg aph a b c t1 t2 x)
    (option_traverse_composed g h apg aph a b c t1 t2 (Some a x))
    (compose_map_coh g h apg aph c (Option c) (Some c) (cmp_traverse_action g h apg a b c t1 t2 x))

fn otc_some_raw_map (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) : g (h (Option c)) =
  apg.functor.map (h c) (h (Option c)) (aph.functor.map c (Option c) (Some c)) (cmp_traverse_action g h apg a b c t1 t2 x)

lemma otc_some_step0b (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) :
  Equal (Compose g h (Option c))
    (otc_some_compose_map_form g h apg aph a b c t1 t2 x)
    (otc_some_raw_map g h apg aph a b c t1 t2 x) =
  Refl

fn otc_some_fused_raw (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) : g (h (Option c)) =
  apg.functor.map b (h (Option c)) (comp b (h c) (h (Option c)) (aph.functor.map c (Option c) (Some c)) t2) (t1 x)

lemma otc_some_step1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) :
  Equal (Compose g h (Option c))
    (otc_some_raw_map g h apg aph a b c t1 t2 x)
    (otc_some_fused_raw g h apg aph a b c t1 t2 x) =
  sym (Compose g h (Option c))
    (otc_some_fused_raw g h apg aph a b c t1 t2 x)
    (otc_some_raw_map g h apg aph a b c t1 t2 x)
    (apg.functor.fusion_law b (h c) (h (Option c)) (aph.functor.map c (Option c) (Some c)) t2 (t1 x))

-- Pointwise: `option_traverse`'s own `Some`-arm reduction relates the
-- opaque `aph.ap(pure …)` form to the plain `aph.functor.map` form —
-- `aph.map_coh`, promoted to a function-level equality by kernel
-- conversion at the `Π` type.
fn otc_some_aph_map (h : Type -> Type) (aph : Applicative h) (b : Type) (c : Type) (t2 : b -> h c) (y : b) : h (Option c) =
  aph.functor.map c (Option c) (Some c) (t2 y)

lemma otc_some_ptwise (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (b : Type) (c : Type) (t2 : b -> h c) (y : b) :
  Equal (h (Option c))
    (otc_some_aph_map h aph b c t2 y)
    (option_traverse h aph b c t2 (Some b y)) =
  aph.map_coh c (Option c) (Some c) (t2 y)

lemma otc_some_step2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) :
  Equal (Compose g h (Option c))
    (otc_some_fused_raw g h apg aph a b c t1 t2 x)
    (otc_some_fused g h apg aph a b c t1 t2 x) =
  cong (b -> h (Option c)) (Compose g h (Option c))
    (comp b (h c) (h (Option c)) (aph.functor.map c (Option c) (Some c)) t2)
    (comp b (Option b) (h (Option c)) (option_traverse h aph b c t2) (Some b))
    (λf. apg.functor.map b (h (Option c)) f (t1 x))
    (otc_some_ptwise g h apg aph b c t2)

lemma option_traverse_composition_some (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (x : a) :
  Equal (Compose g h (Option c))
    (option_traverse_composed g h apg aph a b c t1 t2 (Some a x))
    (option_traverse_composed_rhs g h apg aph a b c t1 t2 (Some a x)) =
  sym (Compose g h (Option c))
    (option_traverse_composed_rhs g h apg aph a b c t1 t2 (Some a x))
    (option_traverse_composed g h apg aph a b c t1 t2 (Some a x))
    (trans (Compose g h (Option c))
      (option_traverse_composed_rhs g h apg aph a b c t1 t2 (Some a x))
      (otc_some_mapped g h apg aph a b c t1 t2 x)
      (option_traverse_composed g h apg aph a b c t1 t2 (Some a x))
      (otc_some_stepB g h apg aph a b c t1 t2 x)
      (trans (Compose g h (Option c))
        (otc_some_mapped g h apg aph a b c t1 t2 x)
        (otc_some_fused g h apg aph a b c t1 t2 x)
        (option_traverse_composed g h apg aph a b c t1 t2 (Some a x))
        (otc_some_stepC g h apg aph a b c t1 t2 x)
        (sym (Compose g h (Option c))
          (option_traverse_composed g h apg aph a b c t1 t2 (Some a x))
          (otc_some_fused g h apg aph a b c t1 t2 x)
          (trans (Compose g h (Option c))
            (option_traverse_composed g h apg aph a b c t1 t2 (Some a x))
            (otc_some_fused_raw g h apg aph a b c t1 t2 x)
            (otc_some_fused g h apg aph a b c t1 t2 x)
            (trans (Compose g h (Option c))
              (option_traverse_composed g h apg aph a b c t1 t2 (Some a x))
              (otc_some_raw_map g h apg aph a b c t1 t2 x)
              (otc_some_fused_raw g h apg aph a b c t1 t2 x)
              (trans (Compose g h (Option c))
                (option_traverse_composed g h apg aph a b c t1 t2 (Some a x))
                (otc_some_compose_map_form g h apg aph a b c t1 t2 x)
                (otc_some_raw_map g h apg aph a b c t1 t2 x)
                (otc_some_step0 g h apg aph a b c t1 t2 x)
                (otc_some_step0b g h apg aph a b c t1 t2 x))
              (otc_some_step1 g h apg aph a b c t1 t2 x))
            (otc_some_step2 g h apg aph a b c t1 t2 x)))))

lemma option_traverse_composition (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (mx : Option a) :
  Equal (Compose g h (Option c))
    (option_traverse_composed g h apg aph a b c t1 t2 mx)
    (option_traverse_composed_rhs g h apg aph a b c t1 t2 mx) =
  match mx {
    None ↦ option_traverse_composition_none g h apg aph a b c t1 t2 ;
    Some x ↦ option_traverse_composition_some g h apg aph a b c t1 t2 x
  }
```

**`List`** — by induction (mirrors `§9.2`'s `list_traverse_identity_law`/
`list_traverse_naturality` shape). `Nil` closes the same way `Option`'s
`None` does. `Cons` chains: unfold `compose_map`/`compose_ap`/
`cmp_traverse_action` (free — all concrete, transparent `fn`s), THREE
`fusion_law` steps to collapse the doubly-/triply-nested `map`s each
introduces into a single `map` over `t1 hd`, the IH, `ap_naturality2`
(the genuinely new content, pushing the recursive `map(traverse t2)(…)`
result through the outer `ap`), and a closing match against
`list_traverse`'s own `Cons`-arm reduction on both sides (definitional,
no extra law — `list_traverse`/`option_traverse` are concrete recursive
`fn`s, so THEIR unfolding is always free even though `apg`/`aph`'s own
operations never are):

```ken
fn list_traverse_composed (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (xs : List a) : Compose g h (List c) =
  match xs {
    Nil ↦ compose_pure g h apg aph (List c) (Nil c) ;
    Cons hd u ↦ compose_ap g h apg aph (List c) (List c) (compose_map g h apg aph c (List c -> List c) (Cons c) (cmp_traverse_action g h apg a b c t1 t2 hd)) (list_traverse_composed g h apg aph a b c t1 t2 u)
  }

fn list_traverse_composed_rhs (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (xs : List a) : Compose g h (List c) =
  apg.functor.map (List b) (h (List c)) (list_traverse h aph b c t2) (list_traverse g apg a b t1 xs)

fn ltc_nil_ap_pure_form (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) : Compose g h (List c) =
  apg.ap (List b) (h (List c)) (apg.pure (List b -> h (List c)) (list_traverse h aph b c t2)) (apg.pure (List b) (Nil b))

lemma ltc_nil_step1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) :
  Equal (Compose g h (List c))
    (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Nil a))
    (ltc_nil_ap_pure_form g h apg aph a b c t1 t2) =
  apg.map_coh (List b) (h (List c)) (list_traverse h aph b c t2) (apg.pure (List b) (Nil b))

lemma ltc_nil_step2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) :
  Equal (Compose g h (List c))
    (ltc_nil_ap_pure_form g h apg aph a b c t1 t2)
    (list_traverse_composed g h apg aph a b c t1 t2 (Nil a)) =
  apg.ap_hom (List b) (h (List c)) (list_traverse h aph b c t2) (Nil b)

lemma list_traverse_composition_nil (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) :
  Equal (Compose g h (List c))
    (list_traverse_composed g h apg aph a b c t1 t2 (Nil a))
    (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Nil a)) =
  sym (Compose g h (List c))
    (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Nil a))
    (list_traverse_composed g h apg aph a b c t1 t2 (Nil a))
    (trans (Compose g h (List c))
      (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Nil a))
      (ltc_nil_ap_pure_form g h apg aph a b c t1 t2)
      (list_traverse_composed g h apg aph a b c t1 t2 (Nil a))
      (ltc_nil_step1 g h apg aph a b c t1 t2)
      (ltc_nil_step2 g h apg aph a b c t1 t2))

fn ltc_x (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) : Compose g h (List c -> List c) =
  compose_map g h apg aph c (List c -> List c) (Cons c) (cmp_traverse_action g h apg a b c t1 t2 hd)

fn ltc_x_dbl (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) : g (h (List c -> List c)) =
  apg.functor.map (h c) (h (List c -> List c)) (aph.functor.map c (List c -> List c) (Cons c)) (cmp_traverse_action g h apg a b c t1 t2 hd)

lemma ltc_x_step0 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) :
  Equal (g (h (List c -> List c))) (ltc_x g h apg aph a b c t1 t2 hd) (ltc_x_dbl g h apg aph a b c t1 t2 hd) =
  Refl

fn ltc_x_raw (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) : g (h (List c -> List c)) =
  apg.functor.map b (h (List c -> List c)) (comp b (h c) (h (List c -> List c)) (aph.functor.map c (List c -> List c) (Cons c)) t2) (t1 hd)

lemma ltc_x_step1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) :
  Equal (g (h (List c -> List c))) (ltc_x_dbl g h apg aph a b c t1 t2 hd) (ltc_x_raw g h apg aph a b c t1 t2 hd) =
  sym (g (h (List c -> List c)))
    (ltc_x_raw g h apg aph a b c t1 t2 hd)
    (ltc_x_dbl g h apg aph a b c t1 t2 hd)
    (apg.functor.fusion_law b (h c) (h (List c -> List c)) (aph.functor.map c (List c -> List c) (Cons c)) t2 (t1 hd))

fn ltc_theta (h : Type -> Type) (aph : Applicative h) (b : Type) (c : Type) (t2 : b -> h c) (y : b) : h (List c) -> h (List c) =
  aph.ap (List c) (List c) (aph.functor.map c (List c -> List c) (Cons c) (t2 y))

fn ltc_xprime_raw1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) : g (h (List c) -> h (List c)) =
  apg.functor.map (h (List c -> List c)) (h (List c) -> h (List c)) (aph.ap (List c) (List c)) (ltc_x_raw g h apg aph a b c t1 t2 hd)

fn ltc_xprime_fused (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) : g (h (List c) -> h (List c)) =
  apg.functor.map b (h (List c) -> h (List c)) (comp b (h (List c -> List c)) (h (List c) -> h (List c)) (aph.ap (List c) (List c)) (comp b (h c) (h (List c -> List c)) (aph.functor.map c (List c -> List c) (Cons c)) t2)) (t1 hd)

lemma ltc_xprime_step (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) :
  Equal (g (h (List c) -> h (List c))) (ltc_xprime_raw1 g h apg aph a b c t1 t2 hd) (ltc_xprime_fused g h apg aph a b c t1 t2 hd) =
  sym (g (h (List c) -> h (List c)))
    (ltc_xprime_fused g h apg aph a b c t1 t2 hd)
    (ltc_xprime_raw1 g h apg aph a b c t1 t2 hd)
    (apg.functor.fusion_law b (h (List c -> List c)) (h (List c) -> h (List c)) (aph.ap (List c) (List c)) (comp b (h c) (h (List c -> List c)) (aph.functor.map c (List c -> List c) (Cons c)) t2) (t1 hd))

fn ltc_cons_raw (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) : Compose g h (List c) =
  apg.ap (h (List c)) (h (List c)) (ltc_xprime_raw1 g h apg aph a b c t1 t2 hd) (list_traverse_composed g h apg aph a b c t1 t2 u)

fn ltc_cons_raw0 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) : Compose g h (List c) =
  apg.ap (h (List c)) (h (List c)) (apg.functor.map (h (List c -> List c)) (h (List c) -> h (List c)) (aph.ap (List c) (List c)) (ltc_x g h apg aph a b c t1 t2 hd)) (list_traverse_composed g h apg aph a b c t1 t2 u)

lemma ltc_cons_step0 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) :
  Equal (Compose g h (List c)) (list_traverse_composed g h apg aph a b c t1 t2 (Cons a hd u)) (ltc_cons_raw0 g h apg aph a b c t1 t2 hd u) =
  Refl

lemma ltc_x_full_eq (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) :
  Equal (g (h (List c -> List c))) (ltc_x g h apg aph a b c t1 t2 hd) (ltc_x_raw g h apg aph a b c t1 t2 hd) =
  trans (g (h (List c -> List c)))
    (ltc_x g h apg aph a b c t1 t2 hd)
    (ltc_x_dbl g h apg aph a b c t1 t2 hd)
    (ltc_x_raw g h apg aph a b c t1 t2 hd)
    (ltc_x_step0 g h apg aph a b c t1 t2 hd)
    (ltc_x_step1 g h apg aph a b c t1 t2 hd)

lemma ltc_cons_step0b (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) :
  Equal (Compose g h (List c)) (ltc_cons_raw0 g h apg aph a b c t1 t2 hd u) (ltc_cons_raw g h apg aph a b c t1 t2 hd u) =
  cong (g (h (List c -> List c))) (Compose g h (List c))
    (ltc_x g h apg aph a b c t1 t2 hd)
    (ltc_x_raw g h apg aph a b c t1 t2 hd)
    (λy. apg.ap (h (List c)) (h (List c)) (apg.functor.map (h (List c -> List c)) (h (List c) -> h (List c)) (aph.ap (List c) (List c)) y) (list_traverse_composed g h apg aph a b c t1 t2 u))
    (ltc_x_full_eq g h apg aph a b c t1 t2 hd)

fn ltc_cons_mid (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (y : Compose g h (List c)) : Compose g h (List c) =
  apg.ap (h (List c)) (h (List c)) (ltc_xprime_fused g h apg aph a b c t1 t2 hd) y

lemma ltc_cons_step1 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) :
  Equal (Compose g h (List c))
    (ltc_cons_raw g h apg aph a b c t1 t2 hd u)
    (ltc_cons_mid g h apg aph a b c t1 t2 hd (list_traverse_composed g h apg aph a b c t1 t2 u)) =
  cong (g (h (List c) -> h (List c))) (Compose g h (List c))
    (ltc_xprime_raw1 g h apg aph a b c t1 t2 hd)
    (ltc_xprime_fused g h apg aph a b c t1 t2 hd)
    (λf. apg.ap (h (List c)) (h (List c)) f (list_traverse_composed g h apg aph a b c t1 t2 u))
    (ltc_xprime_step g h apg aph a b c t1 t2 hd)

lemma ltc_cons_step2 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) (ih : Equal (Compose g h (List c)) (list_traverse_composed g h apg aph a b c t1 t2 u) (list_traverse_composed_rhs g h apg aph a b c t1 t2 u)) :
  Equal (Compose g h (List c))
    (ltc_cons_mid g h apg aph a b c t1 t2 hd (list_traverse_composed g h apg aph a b c t1 t2 u))
    (ltc_cons_mid g h apg aph a b c t1 t2 hd (list_traverse_composed_rhs g h apg aph a b c t1 t2 u)) =
  cong (Compose g h (List c)) (Compose g h (List c))
    (list_traverse_composed g h apg aph a b c t1 t2 u)
    (list_traverse_composed_rhs g h apg aph a b c t1 t2 u)
    (λy. apg.ap (h (List c)) (h (List c)) (ltc_xprime_fused g h apg aph a b c t1 t2 hd) y)
    ih

fn ltc_step3_rhs (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) : Compose g h (List c) =
  apg.ap (List b) (h (List c)) (nat2_rhs_inner g apg (List b) (h (List c)) (h (List c)) (list_traverse h aph b c t2) (ltc_xprime_fused g h apg aph a b c t1 t2 hd)) (list_traverse g apg a b t1 u)

lemma ltc_step3 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) :
  Equal (Compose g h (List c))
    (ltc_cons_mid g h apg aph a b c t1 t2 hd (list_traverse_composed_rhs g h apg aph a b c t1 t2 u))
    (ltc_step3_rhs g h apg aph a b c t1 t2 hd u) =
  ap_naturality2 g apg (List b) (h (List c)) (h (List c)) (list_traverse h aph b c t2) (ltc_xprime_fused g h apg aph a b c t1 t2 hd) (list_traverse g apg a b t1 u)

fn ltc_nat2_fused (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) : g (List b -> h (List c)) =
  apg.functor.map b (List b -> h (List c))
    (comp b (h (List c) -> h (List c)) (List b -> h (List c))
      (nat2_rhs_func (List b) (h (List c)) (h (List c)) (list_traverse h aph b c t2))
      (comp b (h (List c -> List c)) (h (List c) -> h (List c)) (aph.ap (List c) (List c)) (comp b (h c) (h (List c -> List c)) (aph.functor.map c (List c -> List c) (Cons c)) t2)))
    (t1 hd)

lemma ltc_step4 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) :
  Equal (g (List b -> h (List c)))
    (nat2_rhs_inner g apg (List b) (h (List c)) (h (List c)) (list_traverse h aph b c t2) (ltc_xprime_fused g h apg aph a b c t1 t2 hd))
    (ltc_nat2_fused g h apg aph a b c t1 t2 hd) =
  sym (g (List b -> h (List c)))
    (ltc_nat2_fused g h apg aph a b c t1 t2 hd)
    (nat2_rhs_inner g apg (List b) (h (List c)) (h (List c)) (list_traverse h aph b c t2) (ltc_xprime_fused g h apg aph a b c t1 t2 hd))
    (apg.functor.fusion_law b (h (List c) -> h (List c)) (List b -> h (List c))
      (nat2_rhs_func (List b) (h (List c)) (h (List c)) (list_traverse h aph b c t2))
      (comp b (h (List c -> List c)) (h (List c) -> h (List c)) (aph.ap (List c) (List c)) (comp b (h c) (h (List c -> List c)) (aph.functor.map c (List c -> List c) (Cons c)) t2))
      (t1 hd))

fn ltc_xi_func (h : Type -> Type) (aph : Applicative h) (b : Type) (c : Type) (t2 : b -> h c) (y : b) : List b -> h (List c) =
  λl. list_traverse h aph b c t2 (Cons b y l)

fn ltc_target_fused (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) : g (List b -> h (List c)) =
  apg.functor.map b (List b -> h (List c)) (ltc_xi_func h aph b c t2) (t1 hd)

lemma ltc_step5 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) :
  Equal (g (List b -> h (List c))) (ltc_nat2_fused g h apg aph a b c t1 t2 hd) (ltc_target_fused g h apg aph a b c t1 t2 hd) =
  Refl

fn ltc_compose_u (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) : g (List b -> List b) =
  apg.functor.map b (List b -> List b) (Cons b) (t1 hd)

fn ltc_map_compose_u (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) : g (List b -> h (List c)) =
  apg.functor.map (List b -> List b) (List b -> h (List c)) (compose (List b) (List b) (h (List c)) (list_traverse h aph b c t2)) (ltc_compose_u g h apg aph a b c t1 t2 hd)

fn ltc_map_compose_u_raw (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) : g (List b -> h (List c)) =
  apg.functor.map b (List b -> h (List c)) (comp b (List b -> List b) (List b -> h (List c)) (compose (List b) (List b) (h (List c)) (list_traverse h aph b c t2)) (Cons b)) (t1 hd)

lemma ltc_step7 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) :
  Equal (g (List b -> h (List c))) (ltc_map_compose_u_raw g h apg aph a b c t1 t2 hd) (ltc_map_compose_u g h apg aph a b c t1 t2 hd) =
  apg.functor.fusion_law b (List b -> List b) (List b -> h (List c)) (compose (List b) (List b) (h (List c)) (list_traverse h aph b c t2)) (Cons b) (t1 hd)

lemma ltc_step8 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) :
  Equal (g (List b -> h (List c))) (ltc_target_fused g h apg aph a b c t1 t2 hd) (ltc_map_compose_u_raw g h apg aph a b c t1 t2 hd) =
  Refl

lemma ltc_step9 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) :
  Equal (g (List b -> h (List c))) (ltc_target_fused g h apg aph a b c t1 t2 hd) (ltc_map_compose_u g h apg aph a b c t1 t2 hd) =
  trans (g (List b -> h (List c)))
    (ltc_target_fused g h apg aph a b c t1 t2 hd)
    (ltc_map_compose_u_raw g h apg aph a b c t1 t2 hd)
    (ltc_map_compose_u g h apg aph a b c t1 t2 hd)
    (ltc_step8 g h apg aph a b c t1 t2 hd)
    (ltc_step7 g h apg aph a b c t1 t2 hd)

fn ltc_ap_over_v (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (u : List a) (f : g (List b -> h (List c))) : Compose g h (List c) =
  apg.ap (List b) (h (List c)) f (list_traverse g apg a b t1 u)

lemma ltc_step10 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) :
  Equal (Compose g h (List c))
    (ltc_ap_over_v g h apg aph a b c t1 t2 u (ltc_target_fused g h apg aph a b c t1 t2 hd))
    (ltc_ap_over_v g h apg aph a b c t1 t2 u (ltc_map_compose_u g h apg aph a b c t1 t2 hd)) =
  cong (g (List b -> h (List c))) (Compose g h (List c))
    (ltc_target_fused g h apg aph a b c t1 t2 hd)
    (ltc_map_compose_u g h apg aph a b c t1 t2 hd)
    (λf. apg.ap (List b) (h (List c)) f (list_traverse g apg a b t1 u))
    (ltc_step9 g h apg aph a b c t1 t2 hd)

lemma ltc_step11 (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) :
  Equal (Compose g h (List c))
    (ltc_ap_over_v g h apg aph a b c t1 t2 u (ltc_map_compose_u g h apg aph a b c t1 t2 hd))
    (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Cons a hd u)) =
  ap_naturality g apg (List b) (List b) (h (List c)) (list_traverse h aph b c t2) (ltc_compose_u g h apg aph a b c t1 t2 hd) (list_traverse g apg a b t1 u)

lemma list_traverse_composition_cons (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (hd : a) (u : List a) (ih : Equal (Compose g h (List c)) (list_traverse_composed g h apg aph a b c t1 t2 u) (list_traverse_composed_rhs g h apg aph a b c t1 t2 u)) :
  Equal (Compose g h (List c))
    (list_traverse_composed g h apg aph a b c t1 t2 (Cons a hd u))
    (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Cons a hd u)) =
  trans (Compose g h (List c))
    (list_traverse_composed g h apg aph a b c t1 t2 (Cons a hd u))
    (ltc_cons_raw0 g h apg aph a b c t1 t2 hd u)
    (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Cons a hd u))
    (ltc_cons_step0 g h apg aph a b c t1 t2 hd u)
    (trans (Compose g h (List c))
      (ltc_cons_raw0 g h apg aph a b c t1 t2 hd u)
      (ltc_cons_raw g h apg aph a b c t1 t2 hd u)
      (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Cons a hd u))
      (ltc_cons_step0b g h apg aph a b c t1 t2 hd u)
      (trans (Compose g h (List c))
        (ltc_cons_raw g h apg aph a b c t1 t2 hd u)
      (apg.ap (h (List c)) (h (List c)) (ltc_xprime_fused g h apg aph a b c t1 t2 hd) (list_traverse_composed g h apg aph a b c t1 t2 u))
      (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Cons a hd u))
      (ltc_cons_step1 g h apg aph a b c t1 t2 hd u)
      (trans (Compose g h (List c))
        (apg.ap (h (List c)) (h (List c)) (ltc_xprime_fused g h apg aph a b c t1 t2 hd) (list_traverse_composed g h apg aph a b c t1 t2 u))
        (apg.ap (h (List c)) (h (List c)) (ltc_xprime_fused g h apg aph a b c t1 t2 hd) (list_traverse_composed_rhs g h apg aph a b c t1 t2 u))
        (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Cons a hd u))
        (ltc_cons_step2 g h apg aph a b c t1 t2 hd u ih)
        (trans (Compose g h (List c))
          (apg.ap (h (List c)) (h (List c)) (ltc_xprime_fused g h apg aph a b c t1 t2 hd) (list_traverse_composed_rhs g h apg aph a b c t1 t2 u))
          (apg.ap (List b) (h (List c)) (nat2_rhs_inner g apg (List b) (h (List c)) (h (List c)) (list_traverse h aph b c t2) (ltc_xprime_fused g h apg aph a b c t1 t2 hd)) (list_traverse g apg a b t1 u))
          (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Cons a hd u))
          (ltc_step3 g h apg aph a b c t1 t2 hd u)
          (trans (Compose g h (List c))
            (apg.ap (List b) (h (List c)) (nat2_rhs_inner g apg (List b) (h (List c)) (h (List c)) (list_traverse h aph b c t2) (ltc_xprime_fused g h apg aph a b c t1 t2 hd)) (list_traverse g apg a b t1 u))
            (apg.ap (List b) (h (List c)) (ltc_nat2_fused g h apg aph a b c t1 t2 hd) (list_traverse g apg a b t1 u))
            (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Cons a hd u))
            (cong (g (List b -> h (List c))) (Compose g h (List c))
              (nat2_rhs_inner g apg (List b) (h (List c)) (h (List c)) (list_traverse h aph b c t2) (ltc_xprime_fused g h apg aph a b c t1 t2 hd))
              (ltc_nat2_fused g h apg aph a b c t1 t2 hd)
              (λf. apg.ap (List b) (h (List c)) f (list_traverse g apg a b t1 u))
              (ltc_step4 g h apg aph a b c t1 t2 hd))
            (trans (Compose g h (List c))
              (apg.ap (List b) (h (List c)) (ltc_nat2_fused g h apg aph a b c t1 t2 hd) (list_traverse g apg a b t1 u))
              (apg.ap (List b) (h (List c)) (ltc_target_fused g h apg aph a b c t1 t2 hd) (list_traverse g apg a b t1 u))
              (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Cons a hd u))
              (cong (g (List b -> h (List c))) (Compose g h (List c))
                (ltc_nat2_fused g h apg aph a b c t1 t2 hd)
                (ltc_target_fused g h apg aph a b c t1 t2 hd)
                (λf. apg.ap (List b) (h (List c)) f (list_traverse g apg a b t1 u))
                (sym (g (List b -> h (List c))) (ltc_nat2_fused g h apg aph a b c t1 t2 hd) (ltc_target_fused g h apg aph a b c t1 t2 hd) (ltc_step5 g h apg aph a b c t1 t2 hd)))
              (trans (Compose g h (List c))
                (apg.ap (List b) (h (List c)) (ltc_target_fused g h apg aph a b c t1 t2 hd) (list_traverse g apg a b t1 u))
                (apg.ap (List b) (h (List c)) (ltc_map_compose_u g h apg aph a b c t1 t2 hd) (list_traverse g apg a b t1 u))
                (list_traverse_composed_rhs g h apg aph a b c t1 t2 (Cons a hd u))
                (ltc_step10 g h apg aph a b c t1 t2 hd u)
                (ltc_step11 g h apg aph a b c t1 t2 hd u))))))))

lemma list_traverse_composition (g : Type -> Type) (h : Type -> Type) (apg : Applicative g) (aph : Applicative h) (a : Type) (b : Type) (c : Type) (t1 : a -> g b) (t2 : b -> h c) (xs : List a) :
  Equal (Compose g h (List c))
    (list_traverse_composed g h apg aph a b c t1 t2 xs)
    (list_traverse_composed_rhs g h apg aph a b c t1 t2 xs) =
  match xs {
    Nil ↦ list_traverse_composition_nil g h apg aph a b c t1 t2 ;
    Cons hd u ↦ list_traverse_composition_cons g h apg aph a b c t1 t2 hd u (list_traverse_composition g h apg aph a b c t1 t2 u)
  }
```

### 9.8 References

Same sources as `§7` (`Applicative`/`Monad`'s own references apply
identically to `Traversable`, which is the same family); additionally:

- **Haskell base** — `Data.Traversable`
  (`GHC.Base`, part of the `base` package, BSD-3-Clause) —
  <https://gitlab.haskell.org/ghc/ghc> — the canonical `traverse`/
  `sequenceA` shapes and the three coherence laws (identity, naturality,
  composition), consulted for shape only (`CLEAN-ROOM.md`, no source
  copied).
