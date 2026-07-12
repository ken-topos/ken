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
fn bool_or (a : Bool) (b : Bool) : Bool = match a { True ↦ True ; False ↦ b }
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
(`Proved`/`Refl`/`absurd`/direct hypothesis reuse under the restructured
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

#### Proof strategy: `Proved` versus `Refl`

Every branch whose goal reduces to a
TRUE `IsTrue`/`Equal` equation (e.g. `IsTrue (bool_leq True True)`) is
closed with `Proved` (K5 `Top`-intro), not `Refl`: the goal is
`Equal Bool (op x y) True` for an OPERATION (`bool_leq`/`bool_eq`/
`bool_or`), a redex — `eq_at_inductive` must `whnf` it to the literal `True`
before the same-nullary-constructor collapse to `Top` fires (K7, `obs.rs`).
`Refl` checks against a goal whose whnf is *still* `Eq`-shaped
(`ken-elaborator/src/elab.rs`); once the operand is reduced the goal is
`Top`, not `Eq`, so `Refl` no longer applies there — `Proved` is the
textbook-correct introduction for a `Top`-classified goal. Only genuine
hypothesis-reuse branches (the goal is syntactically a bound hypothesis's
own type, e.g. `q`/`p` in `trans` below) stay untouched by K7; they never
went through `Refl` at all.

`antisym`'s "same-value" branches (`x = y`) reduce the GOAL itself
(`Equal a x x`) past a live application into a BARE `Equal Bool True True`/
`Equal Bool False False` — which observationally collapses straight to the
kernel's `Top` proposition (`obs.rs::eq_at_inductive`, same-ctor nullary =>
`Top`), closed by `Proved` (K5 `Top`-introduction). `sound`/`complete`'s
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
REAL, kernel-checked, zero-delta proofs — `Proved` on the equal-value branches,
`absurd` on the contradictory branches (whose hypothesis now genuinely
collapses to `Bottom` under K7). No `Axiom` remains in either instance.

`Eq Bool`'s `sym`/`trans` are ALSO real, kernel-checked, zero-delta proofs
— via the SAME full case-split technique as `antisym`/`sound`/`complete`
above, no further kernel capability needed. Getting here took a real
correction, worth recording (`§6`).

```ken
fn bool_leq (a : Bool) (b : Bool) : Bool = match a { False ↦ True ; True ↦ b }
fn bool_eq (a : Bool) (b : Bool) : Bool = match a { True ↦ b ; False ↦ match b { True ↦ False ; False ↦ True } }

instance Ord Bool {
  leq = bool_leq ;
  refl = λx. match x { True ↦ Proved ; False ↦ Proved } ;
  antisym =
    λx. match x {
      True ↦ λy. match y {
        True ↦ λp.λq. Proved ;
        False ↦ λp.λq. absurd p
      } ;
      False ↦ λy. match y {
        True ↦ λp.λq. absurd q ;
        False ↦ λp.λq. Proved
      }
    } ;
  trans =
    λx. match x {
      True ↦ λy. match y {
        True ↦ λz. match z { True ↦ λp.λq. Proved ; False ↦ λp.λq. q } ;
        False ↦ λz. match z { True ↦ λp.λq. Proved ; False ↦ λp.λq. p }
      } ;
      False ↦ λy.λz.λp.λq. Proved
    } ;
  total =
    λx.λy. match x {
      True ↦ match y { True ↦ Proved ; False ↦ Proved } ;
      False ↦ match y { True ↦ Proved ; False ↦ Proved }
    }
}

instance Eq Bool {
  eq = bool_eq ;
  refl = λx. match x { True ↦ Proved ; False ↦ Proved } ;
  sym =
    λx. match x {
      True ↦ λy. match y {
        True ↦ λp. Proved ;
        False ↦ λp. absurd p
      } ;
      False ↦ λy. match y {
        True ↦ λp. absurd p ;
        False ↦ λp. Proved
      }
    } ;
  trans =
    λx. match x {
      True ↦ λy. match y {
        True ↦ λz. match z { True ↦ λp.λq. Proved ; False ↦ λp.λq. absurd q } ;
        False ↦ λz. match z { True ↦ λp.λq. absurd p ; False ↦ λp.λq. absurd p }
      } ;
      False ↦ λy. match y {
        True ↦ λz. match z { True ↦ λp.λq. absurd p ; False ↦ λp.λq. absurd p } ;
        False ↦ λz. match z { True ↦ λp.λq. absurd q ; False ↦ λp.λq. Proved }
      }
    }
}

instance DecEq Bool {
  eq = bool_eq ;
  sound =
    λx. match x {
      True ↦ λy. match y {
        True ↦ λp. Proved ;
        False ↦ λp. absurd p
      } ;
      False ↦ λy. match y {
        True ↦ λp. absurd p ;
        False ↦ λp. Proved
      }
    } ;
  complete =
    λx. match x {
      True ↦ λy. match y {
        True ↦ λp. Proved ;
        False ↦ λp. absurd p
      } ;
      False ↦ λy. match y {
        True ↦ λp. absurd p ;
        False ↦ λp. Proved
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
  match a { True ↦ b ; False ↦ False }

proof intro for bool_and (a : Bool) (b : Bool)
  : IsTrue a → IsTrue b → IsTrue (bool_and a b) =
  match a {
    True ↦ λha.λhb. hb ;
    False ↦ λha.λhb. absurd ha
  }

proof left for bool_and (a : Bool) (b : Bool)
  : IsTrue (bool_and a b) → IsTrue a =
  match a {
    True ↦ λh. Proved ;
    False ↦ λh. absurd h
  }

proof right for bool_and (a : Bool) (b : Bool)
  : IsTrue (bool_and a b) → IsTrue b =
  match a {
    True ↦ λh. h ;
    False ↦ λh. absurd h
  }

fn compare_bool_cases (b : Bool) : Or (Equal Bool b True) (Equal Bool b False) =
  match b {
    True ↦ Inl (Equal Bool True True) (Equal Bool True False) Proved ;
    False ↦ Inr (Equal Bool False True) (Equal Bool False False) Proved
  }

fn compare_second_result (b : Bool) : OrdResult =
  match b { True ↦ ord_eq ; False ↦ ord_lt }

fn compare_result_of (a : Bool) (b : Bool) : OrdResult =
  match a { True ↦ compare_second_result b ; False ↦ ord_gt }

fn compare_raw (a : Type) (leq : a → a → Bool) (x : a) (y : a) : OrdResult =
  compare_result_of (leq x y) (leq y x)

fn compare_with (a : Type) (d : Ord a) (x : a) (y : a) : OrdResult =
  compare_raw a (d.leq) x y

fn compare (a : Type) (d : Ord a) (x : a) (y : a) : OrdResult =
  compare_with a d x y

lemma compare_eq_sound_raw_second_true (a : Type) (leq : a → a → Bool)
  (antisym_law : (x : a) → (y : a) → Equal Bool (leq x y) True → Equal Bool (leq y x) True → Equal a x y)
  (x : a) (y : a) (hxy : Equal Bool (leq x y) True) (hyx : Equal Bool (leq y x) True)
  : Equal OrdResult (compare_second_result (leq y x)) ord_eq → Equal a x y =
  λp. antisym_law x y hxy hyx

lemma compare_eq_sound_raw_second_false (a : Type) (leq : a → a → Bool)
  (x : a) (y : a) (hyx : Equal Bool (leq y x) False)
  : Equal OrdResult (compare_second_result (leq y x)) ord_eq → Equal a x y =
  λp. absurd (J (λb _. Equal OrdResult (compare_second_result b) ord_eq) p hyx)

lemma compare_eq_sound_raw_second_dispatch (a : Type) (leq : a → a → Bool)
  (antisym_law : (x : a) → (y : a) → Equal Bool (leq x y) True → Equal Bool (leq y x) True → Equal a x y)
  (x : a) (y : a) (hxy : Equal Bool (leq x y) True)
  (choice : Or (Equal Bool (leq y x) True) (Equal Bool (leq y x) False))
  : Equal OrdResult (compare_second_result (leq y x)) ord_eq → Equal a x y =
  match choice {
    Inl hyx ↦ compare_eq_sound_raw_second_true a leq antisym_law x y hxy hyx ;
    Inr hyx ↦ compare_eq_sound_raw_second_false a leq x y hyx
  }

lemma compare_eq_sound_raw_first_true (a : Type) (leq : a → a → Bool)
  (antisym_law : (x : a) → (y : a) → Equal Bool (leq x y) True → Equal Bool (leq y x) True → Equal a x y)
  (x : a) (y : a) (hxy : Equal Bool (leq x y) True)
  : Equal OrdResult (compare_result_of (leq x y) (leq y x)) ord_eq → Equal a x y =
  J (λb _. Equal OrdResult (compare_result_of b (leq y x)) ord_eq → Equal a x y)
    (compare_eq_sound_raw_second_dispatch a leq antisym_law x y hxy (compare_bool_cases (leq y x)))
    (sym Bool (leq x y) True hxy)

lemma compare_eq_sound_raw_first_false (a : Type) (leq : a → a → Bool)
  (x : a) (y : a) (hxy : Equal Bool (leq x y) False)
  : Equal OrdResult (compare_result_of (leq x y) (leq y x)) ord_eq → Equal a x y =
  λp. absurd (J (λb _. Equal OrdResult (compare_result_of b (leq y x)) ord_eq) p hxy)

lemma compare_eq_sound_raw_dispatch (a : Type) (leq : a → a → Bool)
  (antisym_law : (x : a) → (y : a) → Equal Bool (leq x y) True → Equal Bool (leq y x) True → Equal a x y)
  (x : a) (y : a)
  (choice : Or (Equal Bool (leq x y) True) (Equal Bool (leq x y) False))
  : Equal OrdResult (compare_raw a leq x y) ord_eq → Equal a x y =
  match choice {
    Inl hxy ↦ compare_eq_sound_raw_first_true a leq antisym_law x y hxy ;
    Inr hxy ↦ compare_eq_sound_raw_first_false a leq x y hxy
  }

proof eq_sound for compare_raw (a : Type) (leq : a → a → Bool)
  (antisym_law : (x : a) → (y : a) → Equal Bool (leq x y) True → Equal Bool (leq y x) True → Equal a x y)
  (x : a) (y : a)
  : Equal OrdResult (compare_raw a leq x y) ord_eq → Equal a x y =
  compare_eq_sound_raw_dispatch a leq antisym_law x y (compare_bool_cases (leq x y))

proof eq_complete for compare_raw (a : Type) (leq : a → a → Bool) (x : a) (y : a)
  (hxy : Equal Bool (leq x y) True) (hyx : Equal Bool (leq y x) True)
  : Equal OrdResult (compare_raw a leq x y) ord_eq =
  J (λb _. Equal OrdResult (compare_result_of b (leq y x)) ord_eq)
    (J (λc _. Equal OrdResult (compare_second_result c) ord_eq) Proved
      (sym Bool (leq y x) True hyx))
    (sym Bool (leq x y) True hxy)

proof eq_sound for compare_with (a : Type) (d : Ord a) (x : a) (y : a)
  : Equal OrdResult (compare_with a d x y) ord_eq → Equal a x y =
  compare_raw::eq_sound a (d.leq) (d.antisym) x y

proof lt_sound for compare_raw (a : Type) (leq : a → a → Bool) (x : a) (y : a)
  : Equal OrdResult (compare_raw a leq x y) ord_lt → Equal Bool (leq x y) True =
  match compare_bool_cases (leq x y) {
    Inl hxy ↦ λp. hxy ;
    Inr hxy ↦ λp.
      absurd (J (λb _. Equal OrdResult (compare_result_of b (leq y x)) ord_lt) p hxy)
  }

lemma compare_lt_reverse_false_when_first_true (a : Type) (leq : a → a → Bool)
  (x : a) (y : a)
  (choice : Or (Equal Bool (leq y x) True) (Equal Bool (leq y x) False))
  : Equal OrdResult (compare_second_result (leq y x)) ord_lt → Equal Bool (leq y x) False =
  match choice {
    Inl hyx ↦ λp.
      absurd (J (λb _. Equal OrdResult (compare_second_result b) ord_lt) p hyx) ;
    Inr hyx ↦ λp. hyx
  }

proof lt_reverse_false for compare_raw (a : Type) (leq : a → a → Bool)
  (x : a) (y : a)
  : Equal OrdResult (compare_raw a leq x y) ord_lt → Equal Bool (leq y x) False =
  match compare_bool_cases (leq x y) {
    Inl hxy ↦
      J (λb _. Equal OrdResult (compare_result_of b (leq y x)) ord_lt → Equal Bool (leq y x) False)
        (compare_lt_reverse_false_when_first_true a leq x y (compare_bool_cases (leq y x)))
        (sym Bool (leq x y) True hxy) ;
    Inr hxy ↦ λp.
      absurd (J (λb _. Equal OrdResult (compare_result_of b (leq y x)) ord_lt) p hxy)
  }

proof left_false_elim for bool_or (a : Bool) (b : Bool)
  (ha : Equal Bool a False) (hor : Equal Bool (bool_or a b) True)
  : Equal Bool b True =
  J (λw _. Equal Bool (bool_or w b) True) hor ha

lemma compare_gt_sound_raw_first_true (a : Type) (leq : a → a → Bool)
  (x : a) (y : a)
  (choice : Or (Equal Bool (leq y x) True) (Equal Bool (leq y x) False))
  : Equal OrdResult (compare_second_result (leq y x)) ord_gt → Equal Bool (leq y x) True =
  match choice {
    Inl hyx ↦ λp. hyx ;
    Inr hyx ↦ λp.
      absurd (J (λb _. Equal OrdResult (compare_second_result b) ord_gt) p hyx)
  }

proof gt_sound for compare_raw (a : Type) (leq : a → a → Bool)
  (total_law : (x : a) → (y : a) → Equal Bool (bool_or (leq x y) (leq y x)) True)
  (x : a) (y : a)
  : Equal OrdResult (compare_raw a leq x y) ord_gt → Equal Bool (leq y x) True =
  match compare_bool_cases (leq x y) {
    Inl hxy ↦
      J (λb _. Equal OrdResult (compare_result_of b (leq y x)) ord_gt → Equal Bool (leq y x) True)
        (compare_gt_sound_raw_first_true a leq x y (compare_bool_cases (leq y x)))
        (sym Bool (leq x y) True hxy) ;
    Inr hxy ↦ λp.
      bool_or::left_false_elim (leq x y) (leq y x) hxy (total_law x y)
  }

lemma compare_second_result_not_gt (b : Bool)
  (p : Equal OrdResult (compare_second_result b) ord_gt) : Bottom =
  match compare_bool_cases b {
    Inl h ↦ absurd (J (λc _. Equal OrdResult (compare_second_result c) ord_gt) p h) ;
    Inr h ↦ absurd (J (λc _. Equal OrdResult (compare_second_result c) ord_gt) p h)
  }

lemma compare_gt_forward_false_when_true (a : Type) (leq : a → a → Bool)
  (x : a) (y : a)
  : Equal OrdResult (compare_result_of True (leq y x)) ord_gt → Equal Bool (leq x y) False =
  λp. absurd (compare_second_result_not_gt (leq y x) p)

proof gt_forward_false for compare_raw (a : Type) (leq : a → a → Bool)
  (x : a) (y : a)
  : Equal OrdResult (compare_raw a leq x y) ord_gt → Equal Bool (leq x y) False =
  match compare_bool_cases (leq x y) {
    Inl hxy ↦
      J (λb _. Equal OrdResult (compare_result_of b (leq y x)) ord_gt → Equal Bool (leq x y) False)
        (compare_gt_forward_false_when_true a leq x y)
        (sym Bool (leq x y) True hxy) ;
    Inr hxy ↦ λp. hxy
  }

proof leq_sound for compare_raw (a : Type) (leq : a → a → Bool)
  (x : a) (y : a)
  : IsTrue (ord_result_leq (compare_raw a leq x y)) →
    Equal Bool (leq x y) True =
  match compare_bool_cases (leq x y) {
    Inl hxy ↦ λp. hxy ;
    Inr hxy ↦ λp.
      absurd
        (J (λb _. IsTrue (ord_result_leq (compare_result_of b (leq y x))))
          p hxy)
  }

lemma compare_leq_complete_when_true (a : Type) (leq : a → a → Bool)
  (x : a) (y : a)
  (choice : Or (Equal Bool (leq y x) True) (Equal Bool (leq y x) False))
  : IsTrue (ord_result_leq (compare_result_of True (leq y x))) =
  match choice {
    Inl hyx ↦
      J (λb _. IsTrue (ord_result_leq (compare_second_result b))) Proved
        (sym Bool (leq y x) True hyx) ;
    Inr hyx ↦
      J (λb _. IsTrue (ord_result_leq (compare_second_result b))) Proved
        (sym Bool (leq y x) False hyx)
  }

proof leq_complete for compare_raw (a : Type) (leq : a → a → Bool)
  (x : a) (y : a) (hxy : Equal Bool (leq x y) True)
  : IsTrue (ord_result_leq (compare_raw a leq x y)) =
  J (λb _. IsTrue (ord_result_leq (compare_result_of b (leq y x))))
    (compare_leq_complete_when_true a leq x y (compare_bool_cases (leq y x)))
    (sym Bool (leq x y) True hxy)

fn ord_leq_at (a : Type) (d : Ord a) (x : a) (y : a) : Bool =
  d.leq x y

proof true_of_equal for ord_leq_at (a : Type) (d : Ord a) (x : a) (y : a)
  (p : Equal a x y) : Equal Bool (ord_leq_at a d x y) True =
  J (λy2 _. Equal Bool (ord_leq_at a d x y2) True) (d.refl x) p

lemma bool_true_false_absurd (z : Bool)
  (ht : Equal Bool z True) (hf : Equal Bool z False) : Bottom =
  absurd (J (λw _. Equal Bool w True) ht hf)

proof left_true_intro for bool_or (x : Bool) (y : Bool)
  (hx : Equal Bool x True) : Equal Bool (bool_or x y) True =
  J (λw _. Equal Bool (bool_or w y) True) Proved
    (sym Bool x True hx)

proof right_true_intro for bool_or (x : Bool) (y : Bool)
  (hy : Equal Bool y True) : Equal Bool (bool_or x y) True =
  match compare_bool_cases x {
    Inl hx ↦ J (λw _. Equal Bool (bool_or w y) True) Proved
      (sym Bool x True hx) ;
    Inr hx ↦ J (λw _. Equal Bool (bool_or w y) True) hy
      (sym Bool x False hx)
  }

fn pair_ord_leq (a : Type) (b : Type) (da : Ord a) (db : Ord b)
  (x : Pair a b) (y : Pair a b) : Bool =
  ord_result_leq
    (pair_compare a b
      (compare a da) (compare b db)
      x y)

proof refl for pair_ord_leq (a : Type) (b : Type) (da : Ord a) (db : Ord b)
  (x : Pair a b) : IsTrue (pair_ord_leq a b da db x x) =
  J (λr _. IsTrue (ord_result_leq r)) Proved
    (sym OrdResult
      (pair_compare a b (compare a da) (compare b db) x x)
      ord_eq
      (pair_compare::eq a b
      (compare a da) (compare b db)
      x x
      (compare_raw::eq_complete a (da.leq)
        (pair_fst a b x) (pair_fst a b x)
        (da.refl (pair_fst a b x))
        (da.refl (pair_fst a b x)))
      (compare_raw::eq_complete b (db.leq)
        (pair_snd a b x) (pair_snd a b x)
        (db.refl (pair_snd a b x))
        (db.refl (pair_snd a b x)))))

lemma pair_ord_leq_transport_head (a : Type) (b : Type)
  (da : Ord a) (db : Ord b) (x : Pair a b) (y : Pair a b)
  (r : OrdResult)
  (p : Equal OrdResult
    (compare a da (pair_fst a b x) (pair_fst a b y)) r)
  (h : IsTrue (pair_ord_leq a b da db x y))
  : IsTrue
      (ord_result_leq
        (pair_compare_result_of
          (compare b db (pair_snd a b x) (pair_snd a b y)) r)) =
  J (λq _.
      IsTrue
        (ord_result_leq
          (pair_compare_result_of
            (compare b db (pair_snd a b x) (pair_snd a b y)) q)))
    h p

lemma pair_ord_leq_untransport_head (a : Type) (b : Type)
  (da : Ord a) (db : Ord b) (x : Pair a b) (y : Pair a b)
  (r : OrdResult)
  (p : Equal OrdResult
    (compare a da (pair_fst a b x) (pair_fst a b y)) r)
  (h : IsTrue
    (ord_result_leq
      (pair_compare_result_of
        (compare b db (pair_snd a b x) (pair_snd a b y)) r)))
  : IsTrue (pair_ord_leq a b da db x y) =
  J (λq _.
      IsTrue
        (ord_result_leq
          (pair_compare_result_of
            (compare b db (pair_snd a b x) (pair_snd a b y)) q)))
    h
    (sym OrdResult
      (compare a da (pair_fst a b x) (pair_fst a b y)) r p)

lemma pair_ord_head_sound (a : Type) (b : Type) (da : Ord a) (db : Ord b)
  (x : Pair a b) (y : Pair a b)
  (h : IsTrue (pair_ord_leq a b da db x y))
  : Equal Bool
      (ord_leq_at a da (pair_fst a b x) (pair_fst a b y)) True =
  ord_result_elim
    (λr.
      Equal OrdResult
        (compare a da (pair_fst a b x) (pair_fst a b y)) r →
      Equal Bool
        (ord_leq_at a da (pair_fst a b x) (pair_fst a b y)) True)
    (compare a da (pair_fst a b x) (pair_fst a b y))
    (λp. compare_raw::lt_sound a (da.leq)
      (pair_fst a b x) (pair_fst a b y) p)
    (λp. ord_leq_at::true_of_equal a da
      (pair_fst a b x) (pair_fst a b y)
      (compare_with::eq_sound a da (pair_fst a b x) (pair_fst a b y) p))
    (λp. absurd (pair_ord_leq_transport_head a b da db x y ord_gt p h))
    Refl

lemma pair_ord_tail_sound (a : Type) (b : Type) (da : Ord a) (db : Ord b)
  (x : Pair a b) (y : Pair a b)
  (hyx : Equal Bool
    (ord_leq_at a da (pair_fst a b y) (pair_fst a b x)) True)
  (h : IsTrue (pair_ord_leq a b da db x y))
  : Equal Bool
      (ord_leq_at b db (pair_snd a b x) (pair_snd a b y)) True =
  ord_result_elim
    (λr.
      Equal OrdResult
        (compare a da (pair_fst a b x) (pair_fst a b y)) r →
      Equal Bool
        (ord_leq_at b db (pair_snd a b x) (pair_snd a b y)) True)
    (compare a da (pair_fst a b x) (pair_fst a b y))
    (λp. absurd
      (bool_true_false_absurd
        (ord_leq_at a da (pair_fst a b y) (pair_fst a b x))
        hyx
        (compare_raw::lt_reverse_false a (da.leq)
          (pair_fst a b x) (pair_fst a b y) p)))
    (λp. compare_raw::leq_sound b (db.leq)
      (pair_snd a b x) (pair_snd a b y)
      (pair_ord_leq_transport_head a b da db x y ord_eq p h))
    (λp. absurd (pair_ord_leq_transport_head a b da db x y ord_gt p h))
    Refl

lemma pair_ord_complete_head_strict (a : Type) (b : Type)
  (da : Ord a) (db : Ord b) (x : Pair a b) (y : Pair a b)
  (hxy : Equal Bool
    (ord_leq_at a da (pair_fst a b x) (pair_fst a b y)) True)
  (hyx : Equal Bool
    (ord_leq_at a da (pair_fst a b y) (pair_fst a b x)) False)
  : IsTrue (pair_ord_leq a b da db x y) =
  ord_result_elim
    (λr.
      Equal OrdResult
        (compare a da (pair_fst a b x) (pair_fst a b y)) r →
      IsTrue (pair_ord_leq a b da db x y))
    (compare a da (pair_fst a b x) (pair_fst a b y))
    (λp. pair_ord_leq_untransport_head a b da db x y ord_lt p Proved)
    (λp. absurd
      (bool_true_false_absurd
        (ord_leq_at a da (pair_fst a b y) (pair_fst a b x))
        (ord_leq_at::true_of_equal a da
          (pair_fst a b y) (pair_fst a b x)
          (sym a (pair_fst a b x) (pair_fst a b y)
            (compare_with::eq_sound a da
              (pair_fst a b x) (pair_fst a b y) p)))
        hyx))
    (λp. absurd
      (bool_true_false_absurd
        (ord_leq_at a da (pair_fst a b x) (pair_fst a b y))
        hxy
        (compare_raw::gt_forward_false a (da.leq)
          (pair_fst a b x) (pair_fst a b y) p)))
    Refl

lemma pair_ord_complete_tail (a : Type) (b : Type)
  (da : Ord a) (db : Ord b) (x : Pair a b) (y : Pair a b)
  (hxy : Equal Bool
    (ord_leq_at a da (pair_fst a b x) (pair_fst a b y)) True)
  (hyx : Equal Bool
    (ord_leq_at a da (pair_fst a b y) (pair_fst a b x)) True)
  (htail : Equal Bool
    (ord_leq_at b db (pair_snd a b x) (pair_snd a b y)) True)
  : IsTrue (pair_ord_leq a b da db x y) =
  ord_result_elim
    (λr.
      Equal OrdResult
        (compare a da (pair_fst a b x) (pair_fst a b y)) r →
      IsTrue (pair_ord_leq a b da db x y))
    (compare a da (pair_fst a b x) (pair_fst a b y))
    (λp. pair_ord_leq_untransport_head a b da db x y ord_lt p Proved)
    (λp. pair_ord_leq_untransport_head a b da db x y ord_eq p
      (compare_raw::leq_complete b (db.leq)
        (pair_snd a b x) (pair_snd a b y) htail))
    (λp. absurd
      (bool_true_false_absurd
        (ord_leq_at a da (pair_fst a b x) (pair_fst a b y))
        hxy
        (compare_raw::gt_forward_false a (da.leq)
          (pair_fst a b x) (pair_fst a b y) p)))
    Refl

fn pair_deceq_eq (a : Type) (b : Type) (da : DecEq a) (db : DecEq b)
  (x : Pair a b) (y : Pair a b) : Bool =
  bool_and (da.eq (pair_fst a b x) (pair_fst a b y))
           (db.eq (pair_snd a b x) (pair_snd a b y))

lemma pair_deceq_cong (a : Type) (b : Type)
  (x1 : a) (x2 : a) (y1 : b) (y2 : b)
  (p : Equal a x1 x2) (q : Equal b y1 y2)
  : Equal (Pair a b) (mk_pair a b x1 y1) (mk_pair a b x2 y2) =
  J (λx2' _. Equal (Pair a b) (mk_pair a b x1 y1) (mk_pair a b x2' y2))
    (cong b (Pair a b) y1 y2 (mk_pair a b x1) q)
    p

lemma compare_lt_lt_absurd (a : Type) (d : Ord a) (x : a) (y : a)
  (hxy : Equal OrdResult (compare a d x y) ord_lt)
  (hyx : Equal OrdResult (compare a d y x) ord_lt) : Bottom =
  bool_true_false_absurd (ord_leq_at a d x y)
    (compare_raw::lt_sound a (d.leq) x y hxy)
    (compare_raw::lt_reverse_false a (d.leq) y x hyx)

lemma compare_lt_eq_absurd (a : Type) (d : Ord a) (x : a) (y : a)
  (hxy : Equal OrdResult (compare a d x y) ord_lt)
  (hyx : Equal OrdResult (compare a d y x) ord_eq) : Bottom =
  bool_true_false_absurd (ord_leq_at a d y x)
    (ord_leq_at::true_of_equal a d y x (compare_with::eq_sound a d y x hyx))
    (compare_raw::lt_reverse_false a (d.leq) x y hxy)

lemma compare_eq_lt_absurd (a : Type) (d : Ord a) (x : a) (y : a)
  (hxy : Equal OrdResult (compare a d x y) ord_eq)
  (hyx : Equal OrdResult (compare a d y x) ord_lt) : Bottom =
  bool_true_false_absurd (ord_leq_at a d x y)
    (ord_leq_at::true_of_equal a d x y (compare_with::eq_sound a d x y hxy))
    (compare_raw::lt_reverse_false a (d.leq) y x hyx)

proof eq_sound for pair_compare (a : Type) (b : Type) (da : Ord a) (db : Ord b)
  (x : Pair a b) (y : Pair a b)
  (h : Equal OrdResult
    (pair_compare a b (compare a da) (compare b db) x y) ord_eq)
  : Equal (Pair a b) x y =
  pair_deceq_cong a b
    (pair_fst a b x) (pair_fst a b y)
    (pair_snd a b x) (pair_snd a b y)
    (compare_with::eq_sound a da (pair_fst a b x) (pair_fst a b y)
      (and_fst
        (Equal OrdResult (compare a da (pair_fst a b x) (pair_fst a b y)) ord_eq)
        (Equal OrdResult (compare b db (pair_snd a b x) (pair_snd a b y)) ord_eq)
      (pair_compare::eq_cases a b (compare a da) (compare b db) x y h)))
    (compare_with::eq_sound b db (pair_snd a b x) (pair_snd a b y)
      (and_snd
        (Equal OrdResult (compare a da (pair_fst a b x) (pair_fst a b y)) ord_eq)
        (Equal OrdResult (compare b db (pair_snd a b x) (pair_snd a b y)) ord_eq)
        (pair_compare::eq_cases a b (compare a da) (compare b db) x y h)))

proof lt_asym for pair_compare (a : Type) (b : Type) (da : Ord a) (db : Ord b)
  (x : Pair a b) (y : Pair a b)
  (hxy : Equal OrdResult
    (pair_compare a b (compare a da) (compare b db) x y) ord_lt)
  (hyx : Equal OrdResult
    (pair_compare a b (compare a da) (compare b db) y x) ord_lt)
  : Bottom =
  match pair_compare_lt_cases a b (compare a da) (compare b db) x y hxy {
    Inl hax ↦
      match pair_compare_lt_cases a b (compare a da) (compare b db) y x hyx {
        Inl hay ↦ compare_lt_lt_absurd a da
          (pair_fst a b x) (pair_fst a b y) hax hay ;
        Inr hay ↦ compare_lt_eq_absurd a da
          (pair_fst a b x) (pair_fst a b y) hax
          (and_fst
            (Equal OrdResult (compare a da (pair_fst a b y) (pair_fst a b x)) ord_eq)
            (Equal OrdResult (compare b db (pair_snd a b y) (pair_snd a b x)) ord_lt)
            hay)
      } ;
    Inr hbx ↦
      match pair_compare_lt_cases a b (compare a da) (compare b db) y x hyx {
        Inl hay ↦ compare_eq_lt_absurd a da
          (pair_fst a b x) (pair_fst a b y)
          (and_fst
            (Equal OrdResult (compare a da (pair_fst a b x) (pair_fst a b y)) ord_eq)
            (Equal OrdResult (compare b db (pair_snd a b x) (pair_snd a b y)) ord_lt)
            hbx)
          hay ;
        Inr hby ↦ compare_lt_lt_absurd b db
          (pair_snd a b x) (pair_snd a b y)
          (and_snd
            (Equal OrdResult (compare a da (pair_fst a b x) (pair_fst a b y)) ord_eq)
            (Equal OrdResult (compare b db (pair_snd a b x) (pair_snd a b y)) ord_lt)
            hbx)
          (and_snd
            (Equal OrdResult (compare a da (pair_fst a b y) (pair_fst a b x)) ord_eq)
            (Equal OrdResult (compare b db (pair_snd a b y) (pair_snd a b x)) ord_lt)
            hby)
      }
  }

proof antisym for pair_ord_leq (a : Type) (b : Type) (da : Ord a) (db : Ord b)
  (x : Pair a b) (y : Pair a b)
  : IsTrue (pair_ord_leq a b da db x y) →
    IsTrue (pair_ord_leq a b da db y x) →
    Equal (Pair a b) x y =
  ord_result_elim2
    (λr.λs.
      Equal OrdResult
        (pair_compare a b (compare a da) (compare b db) x y) r →
      Equal OrdResult
        (pair_compare a b (compare a da) (compare b db) y x) s →
      IsTrue (ord_result_leq r) → IsTrue (ord_result_leq s) →
      Equal (Pair a b) x y)
    (pair_compare a b (compare a da) (compare b db) x y)
    (pair_compare a b (compare a da) (compare b db) y x)
    (λpx.λpy.λhx.λhy. absurd (pair_compare::lt_asym a b da db x y px py))
    (λpx.λpy.λhx.λhy.
      sym (Pair a b) y x (pair_compare::eq_sound a b da db y x py))
    (λpx.λpy.λhx.λhy. absurd hy)
    (λpx.λpy.λhx.λhy. pair_compare::eq_sound a b da db x y px)
    (λpx.λpy.λhx.λhy. pair_compare::eq_sound a b da db x y px)
    (λpx.λpy.λhx.λhy. absurd hy)
    (λpx.λpy.λhx.λhy. absurd hx)
    (λpx.λpy.λhx.λhy. absurd hx)
    (λpx.λpy.λhx.λhy. absurd hx)
    Refl Refl

lemma pair_ord_trans_with_heads (a : Type) (b : Type)
  (da : Ord a) (db : Ord b)
  (x : Pair a b) (y : Pair a b) (z : Pair a b)
  (hxy : IsTrue (pair_ord_leq a b da db x y))
  (hyz : IsTrue (pair_ord_leq a b da db y z))
  (haxy : Equal Bool
    (ord_leq_at a da (pair_fst a b x) (pair_fst a b y)) True)
  (hayz : Equal Bool
    (ord_leq_at a da (pair_fst a b y) (pair_fst a b z)) True)
  (haxz : Equal Bool
    (ord_leq_at a da (pair_fst a b x) (pair_fst a b z)) True)
  : IsTrue (pair_ord_leq a b da db x z) =
  match compare_bool_cases
    (ord_leq_at a da (pair_fst a b z) (pair_fst a b x)) {
    Inl hazx ↦
      pair_ord_complete_tail a b da db x z haxz hazx
        (db.trans
          (pair_snd a b x) (pair_snd a b y) (pair_snd a b z)
          (pair_ord_tail_sound a b da db x y
            (da.trans
              (pair_fst a b y) (pair_fst a b z) (pair_fst a b x)
              hayz hazx)
            hxy)
          (pair_ord_tail_sound a b da db y z
            (da.trans
              (pair_fst a b z) (pair_fst a b x) (pair_fst a b y)
              hazx haxy)
            hyz)) ;
    Inr hazx ↦ pair_ord_complete_head_strict a b da db x z haxz hazx
  }

proof trans for pair_ord_leq (a : Type) (b : Type) (da : Ord a) (db : Ord b)
  (x : Pair a b) (y : Pair a b) (z : Pair a b)
  (hxy : IsTrue (pair_ord_leq a b da db x y))
  (hyz : IsTrue (pair_ord_leq a b da db y z))
  : IsTrue (pair_ord_leq a b da db x z) =
  pair_ord_trans_with_heads a b da db x y z hxy hyz
    (pair_ord_head_sound a b da db x y hxy)
    (pair_ord_head_sound a b da db y z hyz)
    (da.trans
      (pair_fst a b x) (pair_fst a b y) (pair_fst a b z)
      (pair_ord_head_sound a b da db x y hxy)
      (pair_ord_head_sound a b da db y z hyz))

lemma pair_ord_total_head_both (a : Type) (b : Type)
  (da : Ord a) (db : Ord b) (x : Pair a b) (y : Pair a b)
  (haxy : Equal Bool
    (ord_leq_at a da (pair_fst a b x) (pair_fst a b y)) True)
  (hayx : Equal Bool
    (ord_leq_at a da (pair_fst a b y) (pair_fst a b x)) True)
  : IsTrue
      (bool_or
        (pair_ord_leq a b da db x y)
        (pair_ord_leq a b da db y x)) =
  match compare_bool_cases
    (ord_leq_at b db (pair_snd a b x) (pair_snd a b y)) {
    Inl hbxy ↦
      bool_or::left_true_intro
        (pair_ord_leq a b da db x y)
        (pair_ord_leq a b da db y x)
        (pair_ord_complete_tail a b da db x y haxy hayx hbxy) ;
    Inr hbxy ↦
      bool_or::right_true_intro
        (pair_ord_leq a b da db x y)
        (pair_ord_leq a b da db y x)
        (pair_ord_complete_tail a b da db y x hayx haxy
          (bool_or::left_false_elim
            (ord_leq_at b db (pair_snd a b x) (pair_snd a b y))
            (ord_leq_at b db (pair_snd a b y) (pair_snd a b x))
            hbxy
            (db.total (pair_snd a b x) (pair_snd a b y))))
  }

lemma pair_ord_total_head_forward (a : Type) (b : Type)
  (da : Ord a) (db : Ord b) (x : Pair a b) (y : Pair a b)
  (haxy : Equal Bool
    (ord_leq_at a da (pair_fst a b x) (pair_fst a b y)) True)
  : IsTrue
      (bool_or
        (pair_ord_leq a b da db x y)
        (pair_ord_leq a b da db y x)) =
  match compare_bool_cases
    (ord_leq_at a da (pair_fst a b y) (pair_fst a b x)) {
    Inl hayx ↦ pair_ord_total_head_both a b da db x y haxy hayx ;
    Inr hayx ↦
      bool_or::left_true_intro
        (pair_ord_leq a b da db x y)
        (pair_ord_leq a b da db y x)
        (pair_ord_complete_head_strict a b da db x y haxy hayx)
  }

proof total for pair_ord_leq (a : Type) (b : Type) (da : Ord a) (db : Ord b)
  (x : Pair a b) (y : Pair a b)
  : IsTrue
      (bool_or
        (pair_ord_leq a b da db x y)
        (pair_ord_leq a b da db y x)) =
  match compare_bool_cases
    (ord_leq_at a da (pair_fst a b x) (pair_fst a b y)) {
    Inl haxy ↦ pair_ord_total_head_forward a b da db x y haxy ;
    Inr haxy ↦
      bool_or::right_true_intro
        (pair_ord_leq a b da db x y)
        (pair_ord_leq a b da db y x)
        (pair_ord_complete_head_strict a b da db y x
          (bool_or::left_false_elim
            (ord_leq_at a da (pair_fst a b x) (pair_fst a b y))
            (ord_leq_at a da (pair_fst a b y) (pair_fst a b x))
            haxy
            (da.total (pair_fst a b x) (pair_fst a b y)))
          haxy)
  }

instance Ord (Pair a b) where Ord a, Ord b {
  leq     = pair_ord_leq a b da db ;
  refl    = pair_ord_leq::refl a b da db ;
  antisym = pair_ord_leq::antisym a b da db ;
  trans   = pair_ord_leq::trans a b da db ;
  total   = pair_ord_leq::total a b da db
}

proof sound for pair_deceq_eq (a : Type) (b : Type) (da : DecEq a) (db : DecEq b)
  (x : Pair a b) (y : Pair a b)
  : IsTrue (pair_deceq_eq a b da db x y) → Equal (Pair a b) x y =
  λp.
    pair_deceq_cong a b
      (pair_fst a b x) (pair_fst a b y)
      (pair_snd a b x) (pair_snd a b y)
      (da.sound (pair_fst a b x) (pair_fst a b y)
        (bool_and::left (da.eq (pair_fst a b x) (pair_fst a b y))
          (db.eq (pair_snd a b x) (pair_snd a b y)) p))
      (db.sound (pair_snd a b x) (pair_snd a b y)
        (bool_and::right (da.eq (pair_fst a b x) (pair_fst a b y))
          (db.eq (pair_snd a b x) (pair_snd a b y)) p))

proof complete for pair_deceq_eq (a : Type) (b : Type) (da : DecEq a) (db : DecEq b)
  (x : Pair a b) (y : Pair a b)
  : Equal (Pair a b) x y → IsTrue (pair_deceq_eq a b da db x y) =
  λp.
    bool_and::intro
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
  sound    = pair_deceq_eq::sound a b da db ;
  complete = pair_deceq_eq::complete a b da db
}

fn lex_result_leq (tail : OrdResult) (head : OrdResult) : Bool =
  ord_result_leq (pair_compare_result_of tail head)

lemma lex_transport_head (tail : OrdResult) (head : OrdResult)
  (r : OrdResult) (p : Equal OrdResult head r)
  (h : IsTrue (lex_result_leq tail head))
  : IsTrue (lex_result_leq tail r) =
  J (λq _. IsTrue (lex_result_leq tail q)) h p

lemma lex_untransport_head (tail : OrdResult) (head : OrdResult)
  (r : OrdResult) (p : Equal OrdResult head r)
  (h : IsTrue (lex_result_leq tail r))
  : IsTrue (lex_result_leq tail head) =
  J (λq _. IsTrue (lex_result_leq tail q)) h
    (sym OrdResult head r p)

lemma lex_head_sound (a : Type) (d : Ord a) (x : a) (y : a)
  (tail : OrdResult)
  (h : IsTrue (lex_result_leq tail (compare a d x y)))
  : Equal Bool (ord_leq_at a d x y) True =
  ord_result_elim
    (λr. Equal OrdResult (compare a d x y) r →
      Equal Bool (ord_leq_at a d x y) True)
    (compare a d x y)
    (λp. compare_raw::lt_sound a (d.leq) x y p)
    (λp. ord_leq_at::true_of_equal a d x y
      (compare_with::eq_sound a d x y p))
    (λp. absurd (lex_transport_head tail (compare a d x y) ord_gt p h))
    Refl

lemma lex_tail_sound (a : Type) (d : Ord a) (x : a) (y : a)
  (tail : OrdResult)
  (hyx : Equal Bool (ord_leq_at a d y x) True)
  (h : IsTrue (lex_result_leq tail (compare a d x y)))
  : IsTrue (ord_result_leq tail) =
  ord_result_elim
    (λr. Equal OrdResult (compare a d x y) r →
      IsTrue (ord_result_leq tail))
    (compare a d x y)
    (λp. absurd
      (bool_true_false_absurd (ord_leq_at a d y x) hyx
        (compare_raw::lt_reverse_false a (d.leq) x y p)))
    (λp. lex_transport_head tail (compare a d x y) ord_eq p h)
    (λp. absurd (lex_transport_head tail (compare a d x y) ord_gt p h))
    Refl

lemma lex_complete_head_strict (a : Type) (d : Ord a) (x : a) (y : a)
  (tail : OrdResult)
  (hxy : Equal Bool (ord_leq_at a d x y) True)
  (hyx : Equal Bool (ord_leq_at a d y x) False)
  : IsTrue (lex_result_leq tail (compare a d x y)) =
  ord_result_elim
    (λr. Equal OrdResult (compare a d x y) r →
      IsTrue (lex_result_leq tail (compare a d x y)))
    (compare a d x y)
    (λp. lex_untransport_head tail (compare a d x y) ord_lt p Proved)
    (λp. absurd
      (bool_true_false_absurd (ord_leq_at a d y x)
        (ord_leq_at::true_of_equal a d y x
          (sym a x y (compare_with::eq_sound a d x y p)))
        hyx))
    (λp. absurd
      (bool_true_false_absurd (ord_leq_at a d x y) hxy
        (compare_raw::gt_forward_false a (d.leq) x y p)))
    Refl

lemma lex_complete_tail (a : Type) (d : Ord a) (x : a) (y : a)
  (tail : OrdResult)
  (hxy : Equal Bool (ord_leq_at a d x y) True)
  (hyx : Equal Bool (ord_leq_at a d y x) True)
  (htail : IsTrue (ord_result_leq tail))
  : IsTrue (lex_result_leq tail (compare a d x y)) =
  ord_result_elim
    (λr. Equal OrdResult (compare a d x y) r →
      IsTrue (lex_result_leq tail (compare a d x y)))
    (compare a d x y)
    (λp. lex_untransport_head tail (compare a d x y) ord_lt p Proved)
    (λp. lex_untransport_head tail (compare a d x y) ord_eq p htail)
    (λp. absurd
      (bool_true_false_absurd (ord_leq_at a d x y) hxy
        (compare_raw::gt_forward_false a (d.leq) x y p)))
    Refl

fn list_ord_leq (a : Type) (d : Ord a)
  (xs : List a) (ys : List a) : Bool =
  ord_result_leq (list_compare a (compare a d) xs ys)

proof refl for list_ord_leq (a : Type) (d : Ord a)
  (xs : List a) : IsTrue (list_ord_leq a d xs xs) =
  match xs {
    Nil ↦ Proved ;
    Cons x xs2 ↦
      lex_complete_tail a d x x
        (list_compare a (compare a d) xs2 xs2)
        (d.refl x) (d.refl x) (list_ord_leq::refl a d xs2)
  }

lemma list_ord_cons_cong (a : Type)
  (x : a) (y : a) (xs : List a) (ys : List a)
  (ph : Equal a x y) (pt : Equal (List a) xs ys)
  : Equal (List a) (Cons a x xs) (Cons a y ys) =
  J (λy2 _. Equal (List a) (Cons a x xs) (Cons a y2 ys))
    (cong (List a) (List a) xs ys (Cons a x) pt)
    ph

proof antisym for list_ord_leq (a : Type) (d : Ord a)
  (xs : List a) : (ys : List a) →
    IsTrue (list_ord_leq a d xs ys) →
    IsTrue (list_ord_leq a d ys xs) →
    Equal (List a) xs ys =
  match xs {
    Nil ↦ λys. match ys {
      Nil ↦ λhxy.λhyx. Proved ;
      Cons y ys2 ↦ λhxy.λhyx. absurd hyx
    } ;
    Cons x xs2 ↦ λys. match ys {
      Nil ↦ λhxy.λhyx. absurd hxy ;
      Cons y ys2 ↦ λhxy.λhyx.
        list_ord_cons_cong a x y xs2 ys2
          (d.antisym x y
            (lex_head_sound a d x y
              (list_compare a (compare a d) xs2 ys2) hxy)
            (lex_head_sound a d y x
              (list_compare a (compare a d) ys2 xs2) hyx))
          (list_ord_leq::antisym a d xs2 ys2
            (lex_tail_sound a d x y
              (list_compare a (compare a d) xs2 ys2)
              (lex_head_sound a d y x
                (list_compare a (compare a d) ys2 xs2) hyx)
              hxy)
            (lex_tail_sound a d y x
              (list_compare a (compare a d) ys2 xs2)
              (lex_head_sound a d x y
                (list_compare a (compare a d) xs2 ys2) hxy)
              hyx))
    }
  }

lemma list_ord_trans_cons (a : Type) (d : Ord a)
  (x : a) (xs : List a) (y : a) (ys : List a)
  (z : a) (zs : List a)
  (ih : (ys2 : List a) → (zs2 : List a) →
    IsTrue (list_ord_leq a d xs ys2) →
    IsTrue (list_ord_leq a d ys2 zs2) →
    IsTrue (list_ord_leq a d xs zs2))
  (hxy : IsTrue (list_ord_leq a d (Cons a x xs) (Cons a y ys)))
  (hyz : IsTrue (list_ord_leq a d (Cons a y ys) (Cons a z zs)))
  (haxy : Equal Bool (ord_leq_at a d x y) True)
  (hayz : Equal Bool (ord_leq_at a d y z) True)
  (haxz : Equal Bool (ord_leq_at a d x z) True)
  : IsTrue (list_ord_leq a d (Cons a x xs) (Cons a z zs)) =
  match compare_bool_cases (ord_leq_at a d z x) {
    Inl hazx ↦
      lex_complete_tail a d x z
        (list_compare a (compare a d) xs zs) haxz hazx
        (ih ys zs
          (lex_tail_sound a d x y
            (list_compare a (compare a d) xs ys)
            (d.trans y z x hayz hazx) hxy)
          (lex_tail_sound a d y z
            (list_compare a (compare a d) ys zs)
            (d.trans z x y hazx haxy) hyz)) ;
    Inr hazx ↦
      lex_complete_head_strict a d x z
        (list_compare a (compare a d) xs zs) haxz hazx
  }

proof trans for list_ord_leq (a : Type) (d : Ord a)
  (xs : List a) : (ys : List a) → (zs : List a) →
    IsTrue (list_ord_leq a d xs ys) →
    IsTrue (list_ord_leq a d ys zs) →
    IsTrue (list_ord_leq a d xs zs) =
  match xs {
    Nil ↦ λys.λzs. match zs {
      Nil ↦ λhxy.λhyz. Proved ;
      Cons z zs2 ↦ λhxy.λhyz. Proved
    } ;
    Cons x xs2 ↦ λys. match ys {
      Nil ↦ λzs.λhxy.λhyz. absurd hxy ;
      Cons y ys2 ↦ λzs. match zs {
        Nil ↦ λhxy.λhyz. absurd hyz ;
        Cons z zs2 ↦ λhxy.λhyz.
          list_ord_trans_cons a d x xs2 y ys2 z zs2
            (list_ord_leq::trans a d xs2) hxy hyz
            (lex_head_sound a d x y
              (list_compare a (compare a d) xs2 ys2) hxy)
            (lex_head_sound a d y z
              (list_compare a (compare a d) ys2 zs2) hyz)
            (d.trans x y z
              (lex_head_sound a d x y
                (list_compare a (compare a d) xs2 ys2) hxy)
              (lex_head_sound a d y z
                (list_compare a (compare a d) ys2 zs2) hyz))
      }
    }
  }

lemma list_ord_total_cons_head_both (a : Type) (d : Ord a)
  (x : a) (xs : List a) (y : a) (ys : List a)
  (ih : IsTrue
    (bool_or (list_ord_leq a d xs ys) (list_ord_leq a d ys xs)))
  (hxy : Equal Bool (ord_leq_at a d x y) True)
  (hyx : Equal Bool (ord_leq_at a d y x) True)
  : IsTrue
      (bool_or
        (list_ord_leq a d (Cons a x xs) (Cons a y ys))
        (list_ord_leq a d (Cons a y ys) (Cons a x xs))) =
  match compare_bool_cases (list_ord_leq a d xs ys) {
    Inl htxy ↦
      bool_or::left_true_intro
        (list_ord_leq a d (Cons a x xs) (Cons a y ys))
        (list_ord_leq a d (Cons a y ys) (Cons a x xs))
        (lex_complete_tail a d x y
          (list_compare a (compare a d) xs ys) hxy hyx htxy) ;
    Inr htxy ↦
      bool_or::right_true_intro
        (list_ord_leq a d (Cons a x xs) (Cons a y ys))
        (list_ord_leq a d (Cons a y ys) (Cons a x xs))
        (lex_complete_tail a d y x
          (list_compare a (compare a d) ys xs) hyx hxy
          (bool_or::left_false_elim
            (list_ord_leq a d xs ys) (list_ord_leq a d ys xs)
            htxy ih))
  }

lemma list_ord_total_cons_head_forward (a : Type) (d : Ord a)
  (x : a) (xs : List a) (y : a) (ys : List a)
  (ih : IsTrue
    (bool_or (list_ord_leq a d xs ys) (list_ord_leq a d ys xs)))
  (hxy : Equal Bool (ord_leq_at a d x y) True)
  : IsTrue
      (bool_or
        (list_ord_leq a d (Cons a x xs) (Cons a y ys))
        (list_ord_leq a d (Cons a y ys) (Cons a x xs))) =
  match compare_bool_cases (ord_leq_at a d y x) {
    Inl hyx ↦ list_ord_total_cons_head_both a d x xs y ys ih hxy hyx ;
    Inr hyx ↦
      bool_or::left_true_intro
        (list_ord_leq a d (Cons a x xs) (Cons a y ys))
        (list_ord_leq a d (Cons a y ys) (Cons a x xs))
        (lex_complete_head_strict a d x y
          (list_compare a (compare a d) xs ys) hxy hyx)
  }

lemma list_ord_total_cons (a : Type) (d : Ord a)
  (x : a) (xs : List a) (y : a) (ys : List a)
  (ih : IsTrue
    (bool_or (list_ord_leq a d xs ys) (list_ord_leq a d ys xs)))
  : IsTrue
      (bool_or
        (list_ord_leq a d (Cons a x xs) (Cons a y ys))
        (list_ord_leq a d (Cons a y ys) (Cons a x xs))) =
  match compare_bool_cases (ord_leq_at a d x y) {
    Inl hxy ↦ list_ord_total_cons_head_forward a d x xs y ys ih hxy ;
    Inr hxy ↦
      bool_or::right_true_intro
        (list_ord_leq a d (Cons a x xs) (Cons a y ys))
        (list_ord_leq a d (Cons a y ys) (Cons a x xs))
        (lex_complete_head_strict a d y x
          (list_compare a (compare a d) ys xs)
          (bool_or::left_false_elim
            (ord_leq_at a d x y) (ord_leq_at a d y x)
            hxy (d.total x y))
          hxy)
  }

proof total for list_ord_leq (a : Type) (d : Ord a)
  (xs : List a) : (ys : List a) →
    IsTrue (bool_or (list_ord_leq a d xs ys) (list_ord_leq a d ys xs)) =
  match xs {
    Nil ↦ λys. match ys {
      Nil ↦ bool_or::left_true_intro
        (list_ord_leq a d (Nil a) (Nil a))
        (list_ord_leq a d (Nil a) (Nil a)) Proved ;
      Cons y ys2 ↦ bool_or::left_true_intro
        (list_ord_leq a d (Nil a) (Cons a y ys2))
        (list_ord_leq a d (Cons a y ys2) (Nil a)) Proved
    } ;
    Cons x xs2 ↦ λys. match ys {
      Nil ↦ bool_or::right_true_intro
        (list_ord_leq a d (Cons a x xs2) (Nil a))
        (list_ord_leq a d (Nil a) (Cons a x xs2)) Proved ;
      Cons y ys2 ↦ list_ord_total_cons a d x xs2 y ys2
        (list_ord_leq::total a d xs2 ys2)
    }
  }

instance Ord (List a) where Ord a {
  leq     = list_ord_leq a d ;
  refl    = list_ord_leq::refl a d ;
  antisym = list_ord_leq::antisym a d ;
  trans   = list_ord_leq::trans a d ;
  total   = list_ord_leq::total a d
}

fn list_deceq_eq (a : Type) (da : DecEq a) (xs : List a) (ys : List a) : Bool =
  list_eq a da.eq xs ys

fn list_deceq_head_eq (a : Type) (da : DecEq a) (x : a) (y : a) : Bool =
  da.eq x y

fn list_deceq_cons_result (a : Type) (da : DecEq a) (xs : List a) (ys : List a)
  (b : Bool) : Prop =
  IsTrue (match b { True ↦ list_eq a da.eq xs ys ; False ↦ False })

lemma list_deceq_sound_cons (a : Type) (da : DecEq a)
  (x : a) (xs : List a) (y : a) (ys : List a)
  (ih : (ys : List a) → IsTrue (list_deceq_eq a da xs ys) → Equal (List a) xs ys)
  : list_deceq_cons_result a da xs ys (list_deceq_head_eq a da x y)
    → Equal (List a) (Cons a x xs) (Cons a y ys) =
  match (list_deceq_head_eq a da x y) eqn: h {
    True ↦ λp.
      J (λy' _. Equal (List a) (Cons a x xs) (Cons a y' ys))
        (cong (List a) (List a) xs ys (Cons a x) (ih ys p))
        (da.sound x y h) ;
    False ↦ λp. absurd p
  }

lemma list_deceq_complete_cons (a : Type) (da : DecEq a)
  (x : a) (xs : List a) (y : a) (ys : List a)
  (head_true : IsTrue (list_deceq_head_eq a da x y))
  (tail_true : IsTrue (list_deceq_eq a da xs ys))
  : IsTrue (list_deceq_eq a da (Cons a x xs) (Cons a y ys)) =
  J (λb _. IsTrue (match b { True ↦ list_eq a da.eq xs ys ; False ↦ False }))
    tail_true
    (sym Bool (list_deceq_head_eq a da x y) True head_true)

proof sound for list_deceq_eq (a : Type) (da : DecEq a)
  (xs : List a) : (ys : List a) → IsTrue (list_deceq_eq a da xs ys) → Equal (List a) xs ys =
  match xs {
    Nil ↦ λys. match ys {
      Nil ↦ λp. Proved ;
      Cons y ys2 ↦ λp. absurd p
    } ;
    Cons x xs2 ↦ λys. match ys {
      Nil ↦ λp. absurd p ;
      Cons y ys2 ↦ list_deceq_sound_cons a da x xs2 y ys2
        (list_deceq_eq::sound a da xs2)
    }
  }

lemma list_deceq_complete_nil (a : Type) (da : DecEq a)
  : IsTrue (list_deceq_eq a da (Nil a) (Nil a)) =
  Proved

lemma list_deceq_complete_refl_cons (a : Type) (da : DecEq a)
  (x : a) (xs : List a)
  (ih : IsTrue (list_deceq_eq a da xs xs))
  : IsTrue (list_deceq_eq a da (Cons a x xs) (Cons a x xs)) =
  list_deceq_complete_cons a da x xs x xs
    (da.complete x x Refl)
    ih

lemma list_deceq_complete_refl (a : Type) (da : DecEq a)
  (xs : List a) : IsTrue (list_deceq_eq a da xs xs) =
  match xs {
    Nil ↦ list_deceq_complete_nil a da ;
    Cons x xs2 ↦ list_deceq_complete_refl_cons a da x xs2
      (list_deceq_complete_refl a da xs2)
  }

proof complete for list_deceq_eq (a : Type) (da : DecEq a)
  (xs : List a) : (ys : List a) → Equal (List a) xs ys → IsTrue (list_deceq_eq a da xs ys) =
  λys.λp.
    J (λys' _. IsTrue (list_deceq_eq a da xs ys'))
      (list_deceq_complete_refl a da xs)
      p

instance DecEq (List a) where DecEq a {
  eq       = list_deceq_eq a da ;
  sound    = list_deceq_eq::sound a da ;
  complete = list_deceq_eq::complete a da
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
with `Proved` (both sides reduce to the same literal) or `absurd` (a hypothesis
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
   | The `Proved`-vs-`Refl`/K7 story, the restructuring discipline | [Laws & proofs](#4-laws--proofs) |
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
   `elim_Bool`-into-`Omega` case-split proofs (K4), using `Proved`/`absurd` (K5)
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
   law field depending on arity, each closing with `Proved` (collapsed-`Top`
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
