# `lawful-classes` — `Eq`, `DecEq`, `Ord`

The first `catalog/packages/` catalog tranche and the pattern-setter for
every later ES4 package: three structure classes for decidable equality and
total order, each an ordinary record built from `Bool` and the kernel's own
equality vocabulary — no new kernel former.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws & proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [Findings](#6-findings)
7. [References](#7-references)
8. [Trust & derivation](#8-trust--derivation)

The Definition introduces the public class vocabulary and the small supporting
operations it needs. Laws & proofs is grouped by carrier and proof family:
the audited primitive `Int` boundary, finite `Bool` case splits, then the
projection transports for `Char`. Trust & derivation records the corresponding
trusted-base posture and validation evidence.

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws & proofs](#4-laws--proofs)
- *Researcher* →
  [Laws & proofs](#4-laws--proofs) → [Design notes](#5-design-notes)
- *Porting from Haskell, Lean, or Agda* → [Motivation](#1-motivation) →
  [Definition](#2-definition) → [Design notes](#5-design-notes)

## 1. Motivation

`spec/50-stdlib/51-lawful-classes.md` gives Ken decidable Boolean equality
(`Eq`), a decision procedure for the kernel's propositional equality
(`DecEq`), and a total order (`Ord`) — the vocabulary every later catalog
entry that sorts, compares, or deduplicates needs, stated once as ordinary
checked Ken rather than re-derived per entry.

## 2. Definition

A class is a record (`33 §5.2`, right-nested Σ over `13 §3`); a law is an
`Omega` proposition (`16 §1`). Bridging notation (`51 §2`):
`IsTrue b := Equal Bool b True : Prop` (`Bool` is real inductive data since
ES2; `Prop` is the prelude's surface-nameable alias for `Omega_0`).

```ken
fn IsTrue (b : Bool) : Prop = Equal Bool b True
```

`Eq a` is decidable Boolean equality, an equivalence (`51 §2.1`). `eq` is
the everyday `==`; `refl`/`sym`/`trans` say it is an equivalence relation.
It does NOT tie `eq` to the kernel's propositional `Equal` — that is
`DecEq`'s stronger promise.

```ken
class Eq a {
  eq    : a → a → Bool ;
  refl  : (x : a) → IsTrue (eq x x) ;
  sym   : (x : a) → (y : a) → IsTrue (eq x y) → IsTrue (eq y x) ;
  trans : (x : a) → (y : a) → (z : a) → IsTrue (eq x y) → IsTrue (eq y z) → IsTrue (eq x z)
}
```

`DecEq a` decides the kernel's propositional equality (`51 §2.2`):
`sound`+`complete` together make `eq` a decision procedure for `Equal a`.
It semantically subsumes `Eq a` (recorded as a fact here, not wired as a
superclass constraint — `51 §2.2`/`33 §5.4`).

```ken
class DecEq a {
  eq       : a → a → Bool ;
  sound    : (x : a) → (y : a) → IsTrue (eq x y) → Equal a x y ;
  complete : (x : a) → (y : a) → Equal a x y → IsTrue (eq x y)
}
```

**Public surface and supporting operations.** The stable class vocabulary is
`Eq`, `DecEq`, and `Ord`; their fields are the public operations and laws a
consumer uses through a dictionary. `IsTrue` is the common bridge from a
Boolean result to a proposition, and `bool_or` is deliberately a supporting
definition for `Ord.total`'s law shape rather than an additional ordering
operation. Carrier-specific operations below exist to define or realize those
class fields; the public API and its compatibility posture are collected in
Trust & derivation.

`bool_or` is a REAL (transparent, match-based) `Bool` "or", used ONLY for
`total`'s law-field TYPE (never for an operation field of any instance).
It is deliberately NOT the `or_bool` PRIMITIVE (`numbers.rs`): a primitive
never reduces regardless of argument concreteness (K1's `whnf` only unfolds
`Decl::Transparent`), which would make `total` permanently unprovable for
EVERY carrier, inductive or not. A transparent `bool_or` lets an inductive
carrier's `total` instance be proved by case-split while costing nothing
extra — it's ordinary Ken, not a new kernel feature.

```ken
fn bool_or (a : Bool) (b : Bool) : Bool = match a { True ⇒ True ; False ⇒ b }
```

`Ord a` is a total order, supplying the comparator the verified `sort`/
`is_sorted` thread explicitly (`51 §2.3`/`§4`, ES2-remainder `2358b4d`).
`total`'s Bool-EQUATION form — `IsTrue (bool_or (leq x y) (leq y x))`, the
value-level `bool_or` lifted through `IsTrue` — keeps the law `Omega`-clean
with no truncation (`51 §3`): a BARE propositional "`x<=y` or `y<=x`" would
be proof-relevant (which side holds is content) and need `||.||` to reach
`Omega`; the decidable `Bool` `bool_or` sidesteps that entirely.

```ken
class Ord a {
  leq     : a → a → Bool ;
  refl    : (x : a) → IsTrue (leq x x) ;
  antisym : (x : a) → (y : a) → IsTrue (leq x y) → IsTrue (leq y x) → Equal a x y ;
  trans   : (x : a) → (y : a) → (z : a) → IsTrue (leq x y) → IsTrue (leq y z) → IsTrue (leq x z) ;
  total   : (x : a) → (y : a) → IsTrue (bool_or (leq x y) (leq y x))
}
```

## 3. Using it

`where Ord a` desugars to an implicit `{d : Ord a}` (`33 §5.4`); the
resolved dictionary is bound under the surface name `d` for the duration of
that one declaration's elaboration (never leaks to sibling decls), so the
body/refinement can project its fields (`d.leq`) exactly as the spec's own
illustration shows — ordinary implicit-dictionary insertion, the same
`sort`/`is_sorted` view as the explicit-comparator form, no second mechanism.

Once a concrete instance is registered, its fields project directly off the
synthesized `C_instance_T` dictionary — `§4`'s `Ord Char` instance is a
worked example: every one of its own fields is a `.`-projection off
`Ord_instance_Int` (`(Ord_instance_Int).leq`, `(Ord_instance_Int).refl`,
…), the same projection form a resolved `where Ord a` dictionary uses
internally.

## 4. Laws & proofs

The public law shape lives in the three class declarations above; this section
shows how each carrier inhabits it. Read the families in order when auditing the
trust boundary: `Int` reuses one named certificate and visibly audited ordering
laws, `Bool` proves finite cases in the kernel, and `Char` introduces no new
assumption because it projects the already-existing `Int` dictionaries.

### 4.1 Canonical `Int` instances

`Int` is a K1 primitive: opaque to
δ (`leq_int x x` on a variable `x` does not reduce — primitive reductions
fire on literals only, `ken-kernel`'s `conv.rs::whnf` only unfolds
`Decl::Transparent`, never `Decl::Primitive`) and has no induction
principle, so `DecEq Int`'s universally-quantified `sound`/`complete` laws
are NOT kernel-provable from first principles — proving them would need a
trusted assumption regardless of how the law is phrased. Rather than mint
that assumption fresh in THIS instance (a per-package `Axiom`), `sound`/
`complete` reference `int_eq_sound`/`int_eq_complete` — the ONE named
kernel decidable-equality certificate for `Int` (`ken-kernel::env::
DecEqCert`, registered once against `eq_int` during numeric-tower
bootstrap, `docs/adr/0013-int-decidable-equality-kernel-posture.md` Layer
1), not a fresh `Decl::Opaque` minted by elaborating this file. `Ord Int`'s
own laws are untouched here (still `Axiom`, out of scope — `Int`'s
ordering is not part of this certificate).

`Eq Int`'s `refl`/`sym`/`trans` are then ordinary, REAL, kernel-checked
proofs — not postulates at all — derived from `DecEq Int`'s `sound`/
`complete` via the kernel's own `J`-eliminator over `Equal`, the same
`sym`/`trans`-by-`J` idiom `catalog/packages/Core/Transport.ken.md` uses
(inlined here rather than referenced by name, to avoid a same-named-`sym`/
`trans` cross-file collision — see the `Transport.ken.md`/`EmptyDec.ken.md`
precedent of each inlining its own copy for the same reason): `sound`
converts a `Bool`-equation hypothesis to a propositional one, `J` transports
it to the swapped/composed shape, `complete` converts back. This is a
genuine "5→2 Axiom" collapse for the `Int` equality vocabulary — `Eq Int`
contributes ZERO postulates, `DecEq Int` contributes zero NEW ones (its
`sound`/`complete` are the SAME two kernel-registered entries `Eq Int`
derives from, not per-instance duplicates).

The `eq` field on both instances is `eq_int` DIRECTLY (the raw primitive),
not a `fn`-wrapped alias: the certificate's own type is built kernel-side
against `eq_int` literally (`crates/ken-elaborator/src/numbers.rs`, before
any catalog wrapper exists to reference). Spelling every law field's
hypothesis as `eq_int x y` verbatim makes it match the certificate's own
type on the nose, sidestepping any operand-congruence question entirely —
the same "same literal expression" discipline `Ord Char` (`§5` below) uses
for exactly this reason. (This is a SUFFICIENT choice, not a demonstrated
NECESSARY one: whether a `fn`-wrapped alias like `int_eq` would actually
fail here — the `§5`/K6 `conv_struct` gap — was not independently
reproduced through this specific class-instance-record projection path;
raw `eq_int` avoids the question rather than resolving it.)

```ken
fn int_leq (x : Int) (y : Int) : Bool = leq_int x y

instance DecEq Int {
  eq       = eq_int ;
  sound    = int_eq_sound ;
  complete = int_eq_complete
}

instance Eq Int {
  eq    = eq_int ;
  refl  = λx. int_eq_complete x x Refl ;
  sym   =
    λx.λy.λp.
      int_eq_complete y x
        (J (λy' _. Equal Int y' x) Refl (int_eq_sound x y p)) ;
  trans =
    λx.λy.λz.λp.λq.
      int_eq_complete x z
        (J (λz' _. Equal Int x z') (int_eq_sound x y p) (int_eq_sound y z q))
}

instance Ord Int {
  leq     = int_leq ;
  refl    = Axiom ;
  antisym = Axiom ;
  trans   = Axiom ;
  total   = Axiom
}
```

### 4.2 Canonical `Bool` instances — the zero-delta exemplar

`Bool`
is a real inductive (`data Bool = True | False`, ES2), so its laws ARE
kernel-provable by finite case-split (`elim_Bool` into an `Omega`-motive, K4
`3be0e30`). Every law field below is a REAL, kernel-checked proof
(`tt`/`Refl`/`absurd`/direct hypothesis reuse under the restructured
signature form described next) — NOT `Axiom`, anywhere in any of the three
`Bool` instances.

#### Proof strategy: stage binders before case splits

A law
field bound as `(x:a)(y:a)(p:P x y)(q:Q x y) -> Concl` would check `p`/`q`
against their DECLARED (unnarrowed) types even while case-splitting `x`/`y`
inside the body — `match x {...}` only narrows the GOAL (via the `Elim`'s
motive), never a SIBLING hypothesis bound before it, so `p`/`q` stay
symbolic in `x`/`y` and can't be reused where the branch needs them
concretely. Binding each variable-under-case-split as its OWN Pi layer
(`(x:a) : (y:a) -> P x y -> Concl`, case-splitting `x` FIRST via a `match`
whose ARMS are themselves further `\y. match y {...}`-nested functions, and
only introducing `p`/`q`'s LAMBDA *after* the relevant match) makes each
hypothesis's binder-time type ALREADY concrete in the case-split variables
— so a hypothesis that becomes exactly the (also-concrete) goal in an
"impossible" branch (e.g. both reduce to `Eq Bool False True`) can be reused
directly, no ex-falso needed. This is *why* `refl`/`trans`/`total` are
provable today without any further kernel capability.

#### Proof strategy: `tt` versus `Refl`

Every branch whose goal reduces to a
TRUE `IsTrue`/`Equal` equation (e.g. `IsTrue (bool_leq True True)`) is
closed with `tt` (K5 `Top`-intro), not `Refl`: the goal is
`Equal Bool (op x y) True` for an OPERATION (`bool_leq`/`bool_eq`/
`bool_or`), a redex — `eq_at_inductive` must `whnf` it to the literal `True`
before the same-nullary-constructor collapse to `Top` fires (K7, `obs.rs`).
`Refl` checks against a goal whose whnf is *still* `Eq`-shaped
(`ken-elaborator/src/elab.rs`); once the operand is reduced the goal is
`Top`, not `Eq`, so `Refl` no longer applies there — `tt` is the
textbook-correct introduction for a `Top`-classified goal. Only genuine
hypothesis-reuse branches (the goal is syntactically a bound hypothesis's
own type, e.g. `q`/`p` in `trans` below) stay untouched by K7; they never
went through `Refl` at all.

`antisym`'s "same-value" branches (`x = y`) reduce the GOAL itself
(`Equal a x x`) past a live application into a BARE `Equal Bool True True`/
`Equal Bool False False` — which observationally collapses straight to the
kernel's `Top` proposition (`obs.rs::eq_at_inductive`, same-ctor nullary =>
`Top`), closed by `tt` (K5 `Top`-introduction). `sound`/`complete`'s
"consistent" branches close identically.

#### K7: swapped and contradictory branches

These branches were
EXPECTED (per the Architect's per-obligation re-derivation) to reduce a
HYPOTHESIS to a bare `Equal Bool True False`/`Equal Bool False True`,
collapsing to `Bottom` and closed by `absurd`. THIS DID NOT HOLD ON THE
KERNEL AS IT STOOD THEN, mechanism-grounded (not just structurally
observed): `antisym`/`sound`/`complete`'s hypotheses are
`IsTrue (leq x y)`/`IsTrue (eq x y)` = `Equal Bool (bool_leq x y) True` —
the CARRIER value is wrapped through the instance's OWN operation
(`bool_leq`/`bool_eq`), not a bare case-split variable. Even after BOTH
`x`/`y` are substituted to literal constructors by the case-split,
`bool_leq True False` stays a SYNTACTIC application (`App(Const,lit,lit)`)
until something forces its OWN iota-reduction — and
`ken-kernel/src/obs.rs::eq_at_inductive` (reached via `conv.rs::whnf`'s
`Term::Eq` case / `obs::eq_reduce`) used to call `peel_app` on its two VALUE
operands WITHOUT first WHNF-ing them, so a wrapped-but-literal operand
(`bool_leq True False`) was never recognized as constructor-headed and the
`Eq` stayed neutral — it did NOT collapse to `Bottom`, so `absurd` could not
discharge it. (Confirmed empirically: a DIRECT, unwrapped literal hypothesis
like `p : Equal Bool True False` DID collapse and `absurd` DID close it —
isolated in a scratch repro before this entry was written.) This was
DISTINCT from K6 (about `conv_struct` lacking an `Eq`×`Eq` congruence arm
for comparing two STUCK propositions) — this was about `eq_at_inductive` not
WHNF-ing its OWN operands before checking constructor-headedness, a
narrower defect one level upstream.

Architect-confirmed and named ("K7", `evt_1w8r8qey52qvt`): a genuine kernel
INCOMPLETENESS, not murky — `eq_at_inductive`'s sibling `eq_at_type` (same
file) already whnfs its two value operands before head-matching;
`eq_at_inductive` was simply missing that same step. The fix was a safe,
airtight-sound two-line whnf mirroring `eq_at_type` verbatim (cannot
over-accept: whnf is the kernel's own sound reduction, so a newly-recognized
constructor head was always definitionally true; no regression on
genuinely-neutral operands, which whnf to themselves). Landed as a small
trust-root kernel WP (`obs.rs`-only, `conv.rs` untouched, `4ae2baf`) —
explicitly NOT an elaborator-side transport/`cast` workaround (rejected:
that would have grown the TCB to route around a kernel-completeness gap
that belongs in the kernel). K7 is now on `main`, and this entry wires
`Ord Bool`'s `antisym` and `DecEq Bool`'s `sound`/`complete` (below) as
REAL, kernel-checked, zero-delta proofs — `tt` on the equal-value branches,
`absurd` on the contradictory branches (whose hypothesis now genuinely
collapses to `Bottom` under K7). No `Axiom` remains in either instance.

`Eq Bool`'s `sym`/`trans` are ALSO real, kernel-checked, zero-delta proofs
— via the SAME full case-split technique as `antisym`/`sound`/`complete`
above, no further kernel capability needed. Getting here took a real
correction, worth recording (`§6`).

```ken
fn bool_leq (a : Bool) (b : Bool) : Bool = match a { False ⇒ True ; True ⇒ b }
fn bool_eq (a : Bool) (b : Bool) : Bool = match a { True ⇒ b ; False ⇒ match b { True ⇒ False ; False ⇒ True } }

instance Ord Bool {
  leq = bool_leq ;
  refl = λx. match x { True ⇒ tt ; False ⇒ tt } ;
  antisym =
    λx. match x {
      True ⇒ λy. match y {
        True ⇒ λp.λq. tt ;
        False ⇒ λp.λq. absurd p
      } ;
      False ⇒ λy. match y {
        True ⇒ λp.λq. absurd q ;
        False ⇒ λp.λq. tt
      }
    } ;
  trans =
    λx. match x {
      True ⇒ λy. match y {
        True ⇒ λz. match z { True ⇒ λp.λq. tt ; False ⇒ λp.λq. q } ;
        False ⇒ λz. match z { True ⇒ λp.λq. tt ; False ⇒ λp.λq. p }
      } ;
      False ⇒ λy.λz.λp.λq. tt
    } ;
  total =
    λx.λy. match x {
      True ⇒ match y { True ⇒ tt ; False ⇒ tt } ;
      False ⇒ match y { True ⇒ tt ; False ⇒ tt }
    }
}

instance Eq Bool {
  eq = bool_eq ;
  refl = λx. match x { True ⇒ tt ; False ⇒ tt } ;
  sym =
    λx. match x {
      True ⇒ λy. match y {
        True ⇒ λp. tt ;
        False ⇒ λp. absurd p
      } ;
      False ⇒ λy. match y {
        True ⇒ λp. absurd p ;
        False ⇒ λp. tt
      }
    } ;
  trans =
    λx. match x {
      True ⇒ λy. match y {
        True ⇒ λz. match z { True ⇒ λp.λq. tt ; False ⇒ λp.λq. absurd q } ;
        False ⇒ λz. match z { True ⇒ λp.λq. absurd p ; False ⇒ λp.λq. absurd p }
      } ;
      False ⇒ λy. match y {
        True ⇒ λz. match z { True ⇒ λp.λq. absurd p ; False ⇒ λp.λq. absurd p } ;
        False ⇒ λz. match z { True ⇒ λp.λq. absurd q ; False ⇒ λp.λq. tt }
      }
    }
}

instance DecEq Bool {
  eq = bool_eq ;
  sound =
    λx. match x {
      True ⇒ λy. match y {
        True ⇒ λp. tt ;
        False ⇒ λp. absurd p
      } ;
      False ⇒ λy. match y {
        True ⇒ λp. absurd p ;
        False ⇒ λp. tt
      }
    } ;
  complete =
    λx. match x {
      True ⇒ λy. match y {
        True ⇒ λp. tt ;
        False ⇒ λp. absurd p
      } ;
      False ⇒ λy. match y {
        True ⇒ λp. absurd p ;
        False ⇒ λp. tt
      }
    }
}
```

### 4.3 `Ord Char` — transport from `Ord Int`

Re-homed from the Decimal/Char DEMOTE
(`docs/program/wp/lawful-classes-lane.md`), this instance uses refinement
erasure rather than a carrier-specific proof. Under
refinement erasure `Char = { c : Int | isScalar c }` (`decimal_char.rs`),
`Char.toInt` (`charToInt`) is the IDENTITY projection and
`leqChar a b = leq_int a b` (same file) — so `Char`'s order IS `Int`'s
order, verbatim, not a fresh structure. The law fields do NOT re-postulate
(that would mint a NEW `Decl::Opaque`, growing `trusted_base()`); they
TRANSPORT — reference `Ord Int`'s own existing, already-visible fields via
`.`-projection (`(Ord_instance_Int).refl` etc, `33 §5.2` eta-projection,
parenthesized so the parser takes it as a projection and not a `M.foo`
qualified-name token — `Ord_instance_Int` alone is a `ConId` and
`parse_dotted` would otherwise swallow the whole `Ord_instance_Int.refl` as
one qualified reference). So this instance's own `trusted_base_delta` mints
nothing new: zero-NEW-delta by transport, NOT zero-delta (the referenced
`Axiom`s are still there, honestly, on `Ord Int`) — see `§5` for why `leq`
is also transported rather than using the separately-defined `leqChar` view.

```ken
instance Ord Char {
  leq     = (Ord_instance_Int).leq ;
  refl    = (Ord_instance_Int).refl ;
  antisym = (Ord_instance_Int).antisym ;
  trans   = (Ord_instance_Int).trans ;
  total   = (Ord_instance_Int).total
}
```

### 4.4 `DecEq Char` — the same transport

ADR 0013's Layer 1 states its intended consequence: "someone writes the
trivial `instance DecEq Char`." The shape is identical to `Ord Char`
above — every field is a direct `.`-projection off `DecEq_instance_Int`,
zero-Char-specific kernel work, zero-NEW-delta (the projected fields
themselves resolve to the shared `int_eq_sound`/`int_eq_complete`
certificate, not a fresh postulate).

```ken
instance DecEq Char {
  eq       = (DecEq_instance_Int).eq ;
  sound    = (DecEq_instance_Int).sound ;
  complete = (DecEq_instance_Int).complete
}
```

### 4.5 Structural `DecEq` liftings for `Pair` and `List`

`Pair` and `List` are prelude carriers, so their canonical `DecEq` instances
home with the `DecEq` class. Both lift the supplied element dictionaries
directly: their comparisons remain neutral at abstract elements, and their
proofs therefore route through the supplied `sound` and `complete` fields
rather than treating a dictionary operation as if it reduced by itself.

```ken
fn bool_and (a : Bool) (b : Bool) : Bool =
  match a { True ⇒ b ; False ⇒ False }

fn bool_and_intro (a : Bool) (b : Bool)
  : IsTrue a → IsTrue b → IsTrue (bool_and a b) =
  match a {
    True ⇒ λha.λhb. hb ;
    False ⇒ λha.λhb. absurd ha
  }

fn bool_and_left (a : Bool) (b : Bool)
  : IsTrue (bool_and a b) → IsTrue a =
  match a {
    True ⇒ λh. tt ;
    False ⇒ λh. absurd h
  }

fn bool_and_right (a : Bool) (b : Bool)
  : IsTrue (bool_and a b) → IsTrue b =
  match a {
    True ⇒ λh. h ;
    False ⇒ λh. absurd h
  }

fn bool_dichotomy (b : Bool) : Or (Equal Bool b True) (Equal Bool b False) =
  match b {
    True ⇒ Inl (Equal Bool True True) (Equal Bool True False) tt ;
    False ⇒ Inr (Equal Bool False True) (Equal Bool False False) tt
  }

fn pair_deceq_eq (a : Type) (b : Type) (da : DecEq a) (db : DecEq b)
  (x : Pair a b) (y : Pair a b) : Bool =
  bool_and (da.eq (pair_fst a b x) (pair_fst a b y))
           (db.eq (pair_snd a b x) (pair_snd a b y))

fn pair_deceq_cong (a : Type) (b : Type)
  (x1 : a) (x2 : a) (y1 : b) (y2 : b)
  (p : Equal a x1 x2) (q : Equal b y1 y2)
  : Equal (Pair a b) (mk_pair a b x1 y1) (mk_pair a b x2 y2) =
  J (λx2' _. Equal (Pair a b) (mk_pair a b x1 y1) (mk_pair a b x2' y2))
    (cong b (Pair a b) y1 y2 (mk_pair a b x1) q)
    p

fn pair_deceq_sound (a : Type) (b : Type) (da : DecEq a) (db : DecEq b)
  (x : Pair a b) (y : Pair a b)
  : IsTrue (pair_deceq_eq a b da db x y) → Equal (Pair a b) x y =
  λp.
    pair_deceq_cong a b
      (pair_fst a b x) (pair_fst a b y)
      (pair_snd a b x) (pair_snd a b y)
      (da.sound (pair_fst a b x) (pair_fst a b y)
        (bool_and_left (da.eq (pair_fst a b x) (pair_fst a b y))
          (db.eq (pair_snd a b x) (pair_snd a b y)) p))
      (db.sound (pair_snd a b x) (pair_snd a b y)
        (bool_and_right (da.eq (pair_fst a b x) (pair_fst a b y))
          (db.eq (pair_snd a b x) (pair_snd a b y)) p))

fn pair_deceq_complete (a : Type) (b : Type) (da : DecEq a) (db : DecEq b)
  (x : Pair a b) (y : Pair a b)
  : Equal (Pair a b) x y → IsTrue (pair_deceq_eq a b da db x y) =
  λp.
    bool_and_intro
      (da.eq (pair_fst a b x) (pair_fst a b y))
      (db.eq (pair_snd a b x) (pair_snd a b y))
      (da.complete (pair_fst a b x) (pair_fst a b y)
        (and_fst
          (Equal a (pair_fst a b x) (pair_fst a b y))
          (Equal b (pair_snd a b x) (pair_snd a b y))
          p))
      (db.complete (pair_snd a b x) (pair_snd a b y)
        (and_snd
          (Equal a (pair_fst a b x) (pair_fst a b y))
          (Equal b (pair_snd a b x) (pair_snd a b y))
          p))

instance DecEq (Pair a b) where DecEq a, DecEq b {
  eq       = pair_deceq_eq a b da db ;
  sound    = pair_deceq_sound a b da db ;
  complete = pair_deceq_complete a b da db
}

fn list_deceq_eq (a : Type) (da : DecEq a) (xs : List a) (ys : List a) : Bool =
  list_eq a da.eq xs ys

fn list_deceq_head_eq (a : Type) (da : DecEq a) (x : a) (y : a) : Bool =
  da.eq x y

fn list_deceq_sound_cons_true (a : Type) (da : DecEq a)
  (x : a) (xs : List a) (y : a) (ys : List a)
  (ih : (ys : List a) → IsTrue (list_deceq_eq a da xs ys) → Equal (List a) xs ys)
  (peq : Equal Bool (list_deceq_head_eq a da x y) True)
  (h : IsTrue (list_deceq_eq a da (Cons a x xs) (Cons a y ys)))
  : Equal (List a) (Cons a x xs) (Cons a y ys) =
  J (λy' _. Equal (List a) (Cons a x xs) (Cons a y' ys))
    (cong (List a) (List a) xs ys (Cons a x)
      (ih ys
        (J (λb _. IsTrue (match b { True ⇒ list_eq a da.eq xs ys ; False ⇒ False })) h peq)))
    (da.sound x y peq)

fn list_deceq_sound_cons_false (a : Type) (da : DecEq a)
  (x : a) (xs : List a) (y : a) (ys : List a)
  (qeq : Equal Bool (list_deceq_head_eq a da x y) False)
  (h : IsTrue (list_deceq_eq a da (Cons a x xs) (Cons a y ys)))
  : Equal (List a) (Cons a x xs) (Cons a y ys) =
  absurd
    (J (λb _. IsTrue (match b { True ⇒ list_eq a da.eq xs ys ; False ⇒ False })) h qeq)

fn list_deceq_sound_cons_dispatch (a : Type) (da : DecEq a)
  (x : a) (xs : List a) (y : a) (ys : List a)
  (ih : (ys : List a) → IsTrue (list_deceq_eq a da xs ys) → Equal (List a) xs ys)
  (h : IsTrue (list_deceq_eq a da (Cons a x xs) (Cons a y ys)))
  (choice : Or (Equal Bool (list_deceq_head_eq a da x y) True) (Equal Bool (list_deceq_head_eq a da x y) False))
  : Equal (List a) (Cons a x xs) (Cons a y ys) =
  match choice {
    Inl peq ⇒ list_deceq_sound_cons_true a da x xs y ys ih peq h ;
    Inr qeq ⇒ list_deceq_sound_cons_false a da x xs y ys qeq h
  }

fn list_deceq_complete_cons (a : Type) (da : DecEq a)
  (x : a) (xs : List a) (y : a) (ys : List a)
  (head_true : IsTrue (list_deceq_head_eq a da x y))
  (tail_true : IsTrue (list_deceq_eq a da xs ys))
  : IsTrue (list_deceq_eq a da (Cons a x xs) (Cons a y ys)) =
  J (λb _. IsTrue (match b { True ⇒ list_eq a da.eq xs ys ; False ⇒ False }))
    tail_true
    (sym Bool (list_deceq_head_eq a da x y) True head_true)

fn list_deceq_sound (a : Type) (da : DecEq a)
  (xs : List a) : (ys : List a) → IsTrue (list_deceq_eq a da xs ys) → Equal (List a) xs ys =
  match xs {
    Nil ⇒ λys. match ys {
      Nil ⇒ λp. tt ;
      Cons y ys2 ⇒ λp. absurd p
    } ;
    Cons x xs2 ⇒ λys. match ys {
      Nil ⇒ λp. absurd p ;
      Cons y ys2 ⇒ λp.
        list_deceq_sound_cons_dispatch a da x xs2 y ys2 (list_deceq_sound a da xs2) p
          (bool_dichotomy (da.eq x y))
    }
  }

fn list_deceq_complete_nil (a : Type) (da : DecEq a)
  : IsTrue (list_deceq_eq a da (Nil a) (Nil a)) =
  tt

fn list_deceq_complete_refl_cons (a : Type) (da : DecEq a)
  (x : a) (xs : List a)
  (ih : IsTrue (list_deceq_eq a da xs xs))
  : IsTrue (list_deceq_eq a da (Cons a x xs) (Cons a x xs)) =
  list_deceq_complete_cons a da x xs x xs
    (da.complete x x Refl)
    ih

fn list_deceq_complete_refl (a : Type) (da : DecEq a)
  (xs : List a) : IsTrue (list_deceq_eq a da xs xs) =
  match xs {
    Nil ⇒ list_deceq_complete_nil a da ;
    Cons x xs2 ⇒ list_deceq_complete_refl_cons a da x xs2
      (list_deceq_complete_refl a da xs2)
  }

fn list_deceq_complete (a : Type) (da : DecEq a)
  (xs : List a) : (ys : List a) → Equal (List a) xs ys → IsTrue (list_deceq_eq a da xs ys) =
  λys.λp.
    J (λys' _. IsTrue (list_deceq_eq a da xs ys'))
      (list_deceq_complete_refl a da xs)
      p

instance DecEq (List a) where DecEq a {
  eq       = list_deceq_eq a da ;
  sound    = list_deceq_sound a da ;
  complete = list_deceq_complete a da
}
```

## 5. Design notes

### 5.1 Why `Eq Bool`'s `sym`/`trans` need a real correction, not only K7

The
ORIGINAL (never-shipped) proof attempt tried to REUSE a hypothesis
`p : IsTrue (eq x y)` directly for the swapped goal `IsTrue (eq y x)`,
WITHOUT case-splitting `x`/`y` — i.e. `p` itself, unchanged, as the answer.
With `x`/`y` left as free (symbolic) variables, this needs the kernel to see
`Equal Bool (bool_eq x y) True` and `Equal Bool (bool_eq y x) True` as the
SAME type; `bool_eq x y` and `bool_eq y x` don't reduce (both args
symbolic), so both stay stuck `Term::Eq` propositions, and
`ken-kernel/src/conv.rs`'s `conv_struct` has no congruence case comparing
two `Term::Eq(...)` nodes component-wise — this is a real, confirmed kernel
gap ("K6", Architect-ruled).

But K6 — even a SOUND, POSITIONAL fix to it — would NOT have closed this
pair anyway (Architect's sharpening, `evt_78ntsfnyjdtq6`): positional
congruence compares `bool_eq x y` and `bool_eq y x` argument-by-argument in
place — `x` vs `y` — and for genuinely distinct free variables that is
FALSE, not true. The two applications are only PROPOSITIONALLY equal (via
`bool_eq`'s commutativity, a fact about its VALUE), never DEFINITIONALLY
equal — closing this specific swap-reuse needs a cross-wise congruence arm,
which is the unsound one (smuggles propositional symmetry into definitional
equality, collapses directed `Eq`, enables unproven-symmetry transport via
`cast`) and stays a hard NO. So the hypothesis-reuse-without-case-split
TECHNIQUE was simply the wrong tool here, independent of whether K6 ever
lands.

**The fix:** apply the SAME full case-split `antisym`/`sound`/`complete`
already use. Case-splitting `x` then `y` down to concrete constructors makes
`bool_eq`'s application a REDEX that K7 whnfs before the constructor-head
check — each of the 4 (`sym`)/8 (`trans`) branches then independently closes
with `tt` (both sides reduce to the same literal) or `absurd` (a hypothesis
reduces to `Bottom`). The swap-congruence K6 would have supplied is never
exercised — no branch reuses a hypothesis across a swap; each computes its
own concrete answer. Zero `conv.rs` diff, zero new capability,
K6-independent.

### 5.2 Why `Ord Char` transports `leq` via `.`-projection

`leq` is transported as `(Ord_instance_Int).leq`
— not `leqChar`, though both reduce to the identical `leq_int a b`
(confirmed: `leqChar`/`int_leq` are both `Decl::Transparent` wrappers over
the same `leq_int` primitive, so both fully whnf to the same normal form).
This is a REAL correction, not cosmetic: using `leqChar` here made every
later field's kernel re-check FAIL, even though each field's OWN
transported proof is individually real and each ingredient (Char~Int by
refinement erasure; `leqChar`~`int_leq` by shared unfolding) converts in
isolation. Root cause (isolated by a patch-and-revert scratch repro,
`ken-kernel/src/conv.rs` untouched): `refl`'s expected codomain
`IsTrue (leqChar x x)` and the transported term's own inferred codomain
`IsTrue ((Ord_instance_Int).leq x x)` both whnf to a STUCK
`Term::Eq(Bool, <leq-app-on-a-free-var>, True)` — `leq_int` only fires on
literals, so a free-variable `x` leaves it neutral — and `conv_struct` has
NO congruence arm comparing two `Term::Eq(...)` nodes component-wise, so it
falls to its structural-equality-only path and rejects two operands that
are SYNTACTICALLY different (`leqChar x x` vs `(Ord_instance_Int).leq x x`)
even though both fully reduce to the identical `leq_int x x`. This is the
SAME missing-arm shape as the `Eq Bool` `sym`/`trans` K6 gap above — but
here a SOUND, POSITIONAL congruence arm (comparing type/lhs/rhs pairwise,
each recursively convertible) WOULD have closed it, unlike K6's swap case
(which needed the unsound cross-wise arm). Flagged to the Architect as K6's
first real customer — forward kernel work, not blocking. The fix used here
needs NO kernel change: transport `leq` itself via the SAME `.`-projection
as every other field, so every later field's expected type and the
transported proof's own inferred type share the LITERALLY IDENTICAL
projection term (`(Ord_instance_Int).leq`, not two different names that
happen to co-reduce) — the two sides become syntactically equal after whnf
(`a == b` fires directly, `conv.rs`'s first structural-equality check),
never reaching the missing `Eq`-congruence arm at all. Still zero-NEW-delta
transport, still reduces via `leq_int` (through one more projection layer)
— not a new proof technique, a more literal transport.

## 6. Findings

- **Kernel-reduction defect, LANDED (K7):** `eq_at_inductive`
  (`ken-kernel/src/obs.rs`) failed to WHNF its two value operands before
  checking constructor-headedness, unlike its sibling `eq_at_type` in the
  same file — so an operation-wrapped literal hypothesis
  (`bool_leq True False`) never collapsed to `Bottom`, blocking `absurd`.
  Fixed by a two-line whnf mirroring `eq_at_type` (`4ae2baf`,
  Architect-confirmed `evt_1w8r8qey52qvt`); airtight-sound (whnf is the
  kernel's own reduction), `conv.rs` untouched. `§4` above.
- **Kernel-completeness gap, PARKED (K6):** `conv_struct`
  (`ken-kernel/src/conv.rs`) has no congruence arm comparing two
  `Term::Eq(...)` nodes component-wise. A SOUND, POSITIONAL arm would help
  `Ord Char`-shaped transport proofs (`§5`) but was not needed once `leq`
  transports via `.`-projection instead of a separately-defined view; the
  UNSOUND cross-wise arm that would have helped `Eq Bool`'s original
  proof attempt is a hard no (`§5`). Currently CUSTOMERLESS — no live
  proof obligation in this codebase needs the sound positional arm — so
  this is forward priority, not blocking.
- **Sugar/tooling candidate:** none.
- **Abstraction candidate:** none beyond what `§2` already provides.

## 7. References

- **Type class — Wikipedia** — <https://en.wikipedia.org/wiki/Type_class> —
  general orientation for the record-and-dictionary vocabulary used here.
- **Order theory — Wikipedia** — <https://en.wikipedia.org/wiki/Order_theory>
  — general orientation for reflexivity, antisymmetry, transitivity, and
  totality before reading this entry's Boolean formulation.

These links are reader orientation only. The `IsTrue`-bridged
Boolean/propositional split and the proof strategies in this entry are
Ken-native; no external reference implementation informed its source.

## 8. Trust & derivation

1. **Spec / WP.** `spec/50-stdlib/51-lawful-classes.md`; `wp/ES4-classes-
   build` (Architect ruling `evt_68ppz77ysh5ne`), the ES4-lawproofs reopen
   post-K4 (`3be0e30`), the ES4-lawproofs-remainder reopen post-K5+K7, and
   `wp/eq-bool-sym-trans`'s closure post-K5+K7 (Architect ruling
   `evt_78ntsfnyjdtq6`); `docs/program/wp/lawful-classes-lane.md` (the
   `Ord Char` re-homing); `docs/adr/0013-int-decidable-equality-kernel-
   posture.md` + `docs/program/wp/DS-6a-int-deceq-certificate.md` (the
   `Eq`/`DecEq Int` certificate collapse + `DecEq Char`).
2. **Public API.** `IsTrue`, `class Eq`, `class DecEq`, `bool_or`,
   `class Ord`, `instance Eq Int`, `instance DecEq Int`,
   `instance Ord Int`, `instance Ord Bool`, `instance Eq Bool`,
   `instance DecEq Bool`, `instance Ord Char`, `instance DecEq Char`.
3. **Source map.**

   | Task | Section |
   |---|---|
   | Choose a class or inspect its public field shape | [Definition](#2-definition) |
   | See the three classes | [Definition](#2-definition) |
   | Project a field off a dictionary | [Using it](#3-using-it) |
   | Audit the `Int`, `Bool`, or `Char` proof family | [Laws & proofs](#4-laws--proofs) |
   | The `tt`-vs-`Refl`/K7 story, the restructuring discipline | [Laws & proofs](#4-laws--proofs) |
   | Why `Eq Bool`/`Ord Char` needed the fixes they did | [Design notes](#5-design-notes) |
   | Check assumptions, consumers, and validation evidence | [Trust & derivation](#8-trust--derivation) |

4. **Derivation path.** The three classes are `class` declarations = record
   types (`33 §5.2`, right-nested Σ over `13 §3`), built from `Bool`
   (prelude, `30 §4`) + the kernel's `Eq`/logic vocabulary (`15`/`16`) + the
   Σ/record machinery. No new kernel former. `Ord Int` wraps the audited
   `leq_int` primitive with visible `Axiom` law fields (untouched by the
   Int-equality certificate below, out of scope). `Eq Int`/`DecEq Int` wrap
   `eq_int` and derive their laws from the ONE named kernel decidable-equality
   certificate (`ken-kernel::declare_deceq_certificate`, registered once
   against `eq_int` in `crates/ken-elaborator/src/numbers.rs`, ADR 0013 Layer
   1) — `DecEq Int`'s `sound`/`complete` reference the certificate directly;
   `Eq Int`'s `refl`/`sym`/`trans` are real `J`-derived proofs built FROM
   it, no postulate of their own. `Bool` instances are real
   `elim_Bool`-into-`Omega` case-split proofs (K4), using `tt`/`absurd` (K5)
   over operation-wrapped equations that require K7's operand-whnf to
   collapse. `Ord Char`/`DecEq Char` transport every field via
   `.`-projection off `Ord_instance_Int`/`DecEq_instance_Int`.
5. **`trusted_base()` delta.** `Ord Int`: 4 `Axiom` entries (`refl`/
   `antisym`/`trans`/`total`), each a real, grep-able `Decl::Opaque` —
   illustrative-only, not claimed zero-delta, untouched by the Int-equality
   certificate. `Eq Int`/`DecEq Int`: **zero catalog `Axiom`** — `sound`/
   `complete` reference the
   pre-existing kernel certificate (registered once during numeric-tower
   bootstrap, BEFORE this file is ever elaborated, so elaborating this file
   contributes nothing new to `trusted_base()` for either instance);
   `refl`/`sym`/`trans` are genuine kernel-checked proofs, not postulates.
   The certificate itself is exactly 2 kernel `Decl::Opaque` entries,
   audited once (`ken-kernel/src/check.rs::declare_deceq_certificate`), not
   duplicated per catalog package — this is the "5→2 Axiom, relocated not
   eliminated" honest accounting ADR 0013 describes. `Bool` instances:
   **zero** — every law field is a genuine kernel-checked proof, no `Axiom`
   anywhere in `Ord Bool`/`Eq Bool`/`DecEq Bool`. `Ord Char`/`DecEq Char`:
   **zero-NEW-delta** — mint no new postulate, transport `Ord Int`'s
   `Axiom`s / `DecEq Int`'s certificate reference via projection.
6. **Proof families.** `Bool` instances: full case-split on every
   quantified variable (`x`, `y`, and for `trans`, `z`), 4–8 branches per
   law field depending on arity, each closing with `tt` (collapsed-`Top`
   branches) or `absurd` (collapsed-`Bottom`, contradictory branches) —
   `§4`'s restructuring discipline is why each hypothesis stays reusable
   at binder time. `Eq Int`: `J`-eliminator composition over `Equal Int`,
   converting through `DecEq Int`'s `sound`/`complete` at each end — no
   case-split (`Int` has none to do). `Ord Char`/`DecEq Char`: no
   case-split, pure `.`-projection.
7. **Consumers.** `catalog/packages/Core/EmptyDec.ken.md` inlines its own
   `DecEq Bool` for self-containment (same idiom, independently); the
   sort/comparison threads across `Data/Collections/Collections.ken` and
   `Data/Collections/Map.ken` depend on `Ord`'s `leq` field.
8. **Validation evidence.**
   `crates/ken-elaborator/tests/es4_classes_acceptance.rs` — confirms all
   three `Bool` instances are complete zero-`Axiom` lawful instances (every
   law field kernel-checked, none postulated), and that `Eq Bool`'s
   `sym`/`trans` specifically closed via the corrected full-case-split
   technique, not the original (never-shipped) hypothesis-reuse attempt.
   `crates/ken-elaborator/tests/ds6a_int_deceq_acceptance.rs` — confirms
   `Eq Int`/`DecEq Int`'s zero-catalog-`Axiom` posture, `DecEq Char`'s
   transport, and the certificate's soundness/neutrality conformance arms.
