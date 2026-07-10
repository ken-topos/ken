# `Empty` and `Dec` — computational falsity and decidability

`Empty` is Ken's Type-sorted false, and `Dec P` is a decidability container
that lets you *compute* whether a proposition holds and recover the proof
or refutation. Together they let ordinary code case-split on a proposition
the way it already case-splits on a `Bool`, without losing the proof.

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
- *Practitioner* → [Using it](#3-using-it) → [Laws  proofs](#4-laws--proofs)
- *Researcher* → [Laws  proofs](#4-laws--proofs) → [Design notes](#5-design-notes)
- *Porting from Lean/Agda* → [Design notes](#5-design-notes)

## 1. Motivation

Ken already has two ways to state "false": the proof-irrelevant `Bottom :
Omega` (the logic's falsehood — two proofs of `Bottom` are always equal,
because propositions carry no information beyond their truth) and, for any
`Bool`-classified test, the ordinary `True`/`False` tags. Neither is enough
for *decidability*: given a proposition `P`, a decision procedure needs to
return **which** disjunct holds — a proof of `P`, or a proof that `P` is
impossible — and that answer has to be **inspectable**, so downstream code
can branch on it. A proof-irrelevant `Bottom`-based `Or` can't carry that
information (case-splitting on it can't tell you which side you're on); an
ordinary `Bool` can tell you which side, but throws the proof away.

`Empty` and `Dec` close this gap, mirroring Lean's `Decidable` and Agda's
`⊥`/`Dec` (consulted for shape only, `§7`). Every later catalog entry that
needs a real decision procedure — sorting by a decidable order, comparing
keys, checked pattern-refinement — builds on this pair.

## 2. Definition

`Empty` and the kernel-direct `Dec`/`Yes`/`No`/`decide` are Ken **prelude**
primitives (`crates/ken-elaborator/src/prelude.rs`), not declared by this
entry — surface `data` hardcodes every type parameter to `Type 0`
(`crates/ken-elaborator/src/data.rs:45`), so `Dec`'s `Ω`-sorted parameter
`P` cannot be spelled in surface syntax at all (`§5`, `§8`); both are
bootstrapped the same way `Nat`/`List`/`Bool` already are, so they are
globally available before this entry (or any `.ken` program) loads. (A
zero-constructor `data` — `Empty`'s own shape — now has a real surface
spelling, `data Empty : Type0 where { }`, landed by FR-1, `§6`; the
literal `Type0 =` legacy spelling below is still illustrative, since the
legacy `data D = …` form doesn't take a `:`-ascribed family type.)
Conceptually, as if spelled at the surface:

```ken ignore
-- Illustrative only — NOT re-declared here; `Empty` and `Dec` are already
-- global prelude names by the time this entry elaborates (`§8`).
data Empty : Type0 =

data Dec (P : Omega) : Type0 =
  Yes P
  | No (P -> Empty)

fn decide (P : Omega) (d : Dec P) : Bool =
  match d { Yes p ⇒ True ; No f ⇒ False }
```

What this entry *does* author, as real surface code:

```ken
-- The general Type-sorted eliminator for `Empty` — an uninhabited type
-- eliminates into anything. NOT named `absurd`: that identifier is
-- already checked-mode surface sugar for `Ω`-classified `Bottom`-
-- elimination (`crates/ken-elaborator/src/elab.rs:499`), and declaring a
-- real global under the same name would leave it permanently unreachable
-- (`§6` Finding) — every syntactic `absurd x` is intercepted by the sugar
-- before ordinary name resolution ever sees a user-declared `absurd`.
fn absurdEmpty (C : Type) (e : Empty) : C = match e { }

-- Ergonomic constructors — `Yes`/`No` already work directly, but a
-- lowercase smart-constructor pair reads better at call sites and mirrors
-- `yes`/`no` on the referenced Lean/Agda `Decidable`/`Dec` (`§7`).
fn yes (prp : Omega) (p : prp) : Dec prp = Yes prp p
fn no (prp : Omega) (f : prp -> Empty) : Dec prp = No prp f

-- `DecEq` — inlined from `catalog/packages/Core/LawfulClasses.ken`
-- (self-containment, same idiom `catalog/guide/proof-techniques.ken.md`
-- uses for `cong`/`bool_and`: `ken run` on a standalone entry has no
-- cross-package import mechanism today, `§6` Finding). `decEqDecides`
-- below is fully generic over ANY `DecEq a` instance, landed or local —
-- only the `§3` worked example needs a concrete one in scope.
class DecEq a {
  eq       : a -> a -> Bool ;
  sound    : (x : a) -> (y : a) -> IsTrue (eq x y) -> Equal a x y ;
  complete : (x : a) -> (y : a) -> Equal a x y -> IsTrue (eq x y)
}

fn bool_eq (a : Bool) (b : Bool) : Bool =
  match a { True ⇒ b ; False ⇒ match b { True ⇒ False ; False ⇒ True } }

instance DecEq Bool {
  eq = bool_eq ;
  sound =
    λx. match x {
      True  ⇒ λy. match y { True ⇒ λp. tt ; False ⇒ λp. absurd p } ;
      False ⇒ λy. match y { True ⇒ λp. absurd p ; False ⇒ λp. tt }
    } ;
  complete =
    λx. match x {
      True  ⇒ λy. match y { True ⇒ λp. tt ; False ⇒ λp. absurd p } ;
      False ⇒ λy. match y { True ⇒ λp. absurd p ; False ⇒ λp. tt }
    }
}

-- Reflects a stuck `Bool` value into an equation-carrying `Or`, so a
-- computed `Bool` result can be USED as a proof, not just branched on — a
-- plain `match (d.eq x y) {...}` cannot do this itself (the scrutinee is
-- an application, not a bound variable, so the dependent-motive machinery
-- that lets `match` refine a hypothesis has nothing to bind); this is the
-- same combinator `catalog/packages/Data/Collections/Map.ken` calls
-- `boolDichotomy`, inlined here for self-containment.
fn boolDichotomy (b : Bool) : Or (Equal Bool b True) (Equal Bool b False) =
  match b {
    True  ⇒ Inl (Equal Bool True True) (Equal Bool True False) tt ;
    False ⇒ Inr (Equal Bool False True) (Equal Bool False False) tt
  }

-- `sym`/`trans` — inlined from `catalog/packages/Core/Transport.ken`
-- (self-containment, same idiom `catalog/guide/proof-techniques.ken.md`
-- uses for `cong`) for the No-branch contradiction below.
fn sym (ty : Type) (x : ty) (y : ty) (p : Equal ty x y) : Equal ty y x =
  J (λy' _. Equal ty y' x) Refl p

fn trans (ty : Type) (x : ty) (y : ty) (z : ty)
         (p : Equal ty x y) (q : Equal ty y z) : Equal ty x z =
  J (λz' _. Equal ty x z') p q

-- The bridge: any `DecEq a` instance decides propositional equality.
-- `d.eq x y = True` (`Inl p`, `p : Equal Bool (d.eq x y) True`) → `sound`
-- hands back the proof directly. `d.eq x y = False` (`Inr q`) → assuming
-- a proof `pxy : Equal a x y` gives `d.complete x y pxy : IsTrue (d.eq x y)
-- = Equal Bool (d.eq x y) True`; combined with `q` via `sym`/`trans`, that
-- is `Equal Bool False True` — K7 (`spec/10-kernel/16-observational.md
-- §1`) makes THAT proposition definitionally `Bottom`, so the landed
-- `absurd` sugar (Bottom → any goal, INCLUDING a `Type`-sorted one, `16
-- §1` Bottom-Elim) discharges it into `Empty` directly — no new mechanism,
-- and no need for `absurdEmpty` here (this bridge is Ω → Type, not
-- Empty → C).
fn decEqDecides (a : Type) (d : DecEq a) (x : a) (y : a) : Dec (Equal a x y) =
  match boolDichotomy (d.eq x y) {
    Inl p ⇒ Yes (Equal a x y) (d.sound x y p) ;
    Inr q ⇒
      No (Equal a x y)
         (λpxy. absurd (trans Bool False (d.eq x y) True
                              (sym Bool (d.eq x y) False q)
                              (d.complete x y pxy)))
  }
```

## 3. Using it

```ken example
-- `DecEq_instance_Bool` is the synthesized dictionary value for `§2`'s
-- `instance DecEq Bool { ... }` — every `instance C T { ... }` registers a
-- real global `C_instance_T` (`crates/ken-elaborator/src/elab.rs:3386`),
-- not just a `where`-resolved implicit dictionary. (The landed
-- `catalog/packages/Core/LawfulClasses.ken` carries the SAME
-- shape, independently — `§2`'s note on why this entry inlines its own.)
const trueIsTrue : Dec (Equal Bool True True) =
  decEqDecides Bool DecEq_instance_Bool True True

const trueIsNotFalse : Dec (Equal Bool True False) =
  decEqDecides Bool DecEq_instance_Bool True False

-- `decide` recovers just the Bool tag, e.g. for an ordinary conditional.
const trueIsTrueTag : Bool = decide (Equal Bool True True) trueIsTrue
const trueIsNotFalseTag : Bool = decide (Equal Bool True False) trueIsNotFalse
```

```ken example
-- `yes`/`no` construct `Dec` values directly when you already have the
-- proof or refutation in hand — no `DecEq` needed.
const anyProofDecides : Dec (Equal Bool True True) = yes (Equal Bool True True) tt

fn refuteTrueFalse (p : Equal Bool True False) : Empty = absurd p

const refutationDecides : Dec (Equal Bool True False) =
  no (Equal Bool True False) refuteTrueFalse
```

```ken example
-- `absurdEmpty` — an inhabitant of `Empty`, however obtained, discharges
-- ANY goal.
fn contradictionImpliesAnything (e : Empty) : Bool = absurdEmpty Bool e
```

## 4. Laws  proofs

`Dec`/`Empty` are containers and an eliminator, not a lawful structure with
its own class — the "laws" here are the computation facts that justify
calling `decide` an honest reflection of `Dec`'s tag, stated as `lemma`s
over the concrete `DecEq Bool` instance from `§3` (the guide's `§7` named-
proof-claims form):

```ken example
lemma decideYesIsTrue : Equal Bool (decide (Equal Bool True True) trueIsTrue) True = tt

lemma decideNoIsFalse : Equal Bool (decide (Equal Bool True False) trueIsNotFalse) False = tt
```

Both close with `tt`: `decide`/`trueIsTrue`/`trueIsNotFalse` are all closed,
fully-applied terms, so both sides reduce to the same nullary `Bool`
constructor and the equality collapses to `Top` before `tt` is even checked
(the guide's `§1` `tt`-vs-`Refl` discriminator) — not `Refl`, since neither
side is stuck.

## 5. Design notes

**Why not `Ω`'s `Or`, and why not a homogeneous `Sum`.** A decision
procedure must be able to *large-eliminate* on which disjunct holds — `Or`
(`Ω`-sorted) is proof-irrelevant and cannot large-eliminate into `Type`,
and a homogeneous `Sum a b : Type` cannot mix an `Ω`-sorted proof payload
with a `Type`-sorted refutation payload (Ken is non-cumulative — `Ω` does
not inject into `Type 0`). `Dec`'s two constructors fix each payload's sort
independently, which is exactly the same move the landed refinement type
`{x : A | φ} = (x : A) × φ` already makes (an `Ω` field on a `Type`-sorted
family, `spec/10-kernel/13-pi-sigma.md:133`) — `Dec` has precedent, not a
new capability class.

**The `DecEq Int` caveat.** `DecEq Int.sound` is `Axiom`-backed (`Int` is
an opaque primitive, no induction) — `decEqDecides Int (DecEq Int) x y`
type-checks and is *usable*, but its `Yes` branch's proof rides that
`Axiom`, not a kernel-checked derivation. `§3`'s worked examples
deliberately use `DecEq Bool` (an inductive carrier, honest via K7/no-
confusion, zero trusted-base delta) so the showcase is not vacuous — see
`catalog/guide/` `§1.1` for the general opaque-primitive/`Axiom` pattern.

**Zero-constructor `data` now parses (FR-1, landed)** and **`absurd` is
reserved sugar** (still real, routed as a Finding, `§6`) — this entry's
own `Empty`/`Dec` remain the prelude-bootstrapped globals (`§2`), not
re-declared here, but the surface gap that forced that bootstrap is
closed: an explicit-family zero-constructor block now elaborates over an
independently-declared type, distinct from the prelude's `Empty`:

```ken example
data EmptyAttempt : Type where { }

fn absurdEmptyAttempt (C : Type) (e : EmptyAttempt) : C = match e { }
```

The literal `Type0 =` spelling (`§2`'s `` ```ken ignore `` block) still
isn't real surface syntax — the legacy `data D = …` form doesn't take a
`:`-ascribed family type, so that combination remains illustrative only;
the explicit-family `where { }` spelling above is the real one.

**A user-declared `absurd` is now a resolve-time hard error (FR-2,
landed).** Declaring `fn absurd (C : Type) (e : Empty) : C = match e { }`
used to elaborate successfully with the collision entirely silent — this
entry still uses `absurdEmpty` instead (the honest, reachable name), but
the footgun itself is now caught, not merely worked around:

```ken reject
-- Fails: 'absurd' collides with a reserved surface sugar identifier.
fn absurd (C : Type) (e : Empty) : C = match e { }
```

## 6. Findings

- **Kernel-reduction defect:** none.
- **Sugar candidate → Ergo — landed (FR-1):** the surface originally had
  no way to write a zero-constructor `data` declaration (`parse_data_decl`/
  `parse_explicit_data_decl` in `crates/ken-elaborator/src/parser.rs` both
  required at least one constructor), so `Empty` had to be bootstrapped via
  `data::elab_data_decl` called directly (the same technique
  `ElabEnv::empty()` already uses to bootstrap `Bool`), bypassing the
  parser entirely rather than the ordinary `elaborate_decl` source-text
  path every other prelude `data` uses. `docs/program/wp/
  ds-1-findings-remediation.md` FR-1 relaxed both parser gates for the
  zero-constructor case (`§5` now demonstrates it); `Empty`'s own literal
  pinned spelling (`data Empty : Type0 =`) still doesn't parse, since the
  legacy `data D = …` arm doesn't take a `:`-ascribed family type — the
  explicit-family `where { }` spelling is the real one.
- **Naming hazard — landed (FR-2):** `absurd` is checked-mode surface sugar
  keyed on the bare identifier (`elab.rs`, resolver emits `RCon` on scope
  miss) for `Ω`-classified `Bottom`-elimination. Declaring a real global
  named `absurd` used to elaborate with the collision entirely silent — a
  user-declared `absurd` became permanently unreachable via ordinary call
  syntax with no error at all (confirmed empirically: a probe declaring
  `fn absurd` then re-using the OLD `absurd h : Bottom` sugar shape still
  elaborated against the NEW declaration's unrelated signature). FR-2 makes
  this a resolve-time hard error (`resolve_decl`, guarding every declared
  name — and every `data` constructor name — against the reserved-sugar
  set `{Refl, Axiom, absurd}`, `§5` above now demonstrates the rejection —
  `J`/`Eq` are deliberately excluded, since their arity-3-gated sugar
  coexists with a lower-arity type-former/class of the same name, e.g. the
  landed `class Eq a`). This entry's `Empty`-eliminator stays named
  `absurdEmpty` regardless (the honest, reachable name).
- **Tooling candidate → Ergo (`ken-cli`):** `ken run` unconditionally
  executes the file's LAST declaration as an IO tree (`crates/ken-cli/src/
  main.rs`, `run_file`) — appropriate for a runnable program (the
  `examples/rosetta/*` entries), but a pure-library catalog entry like this
  one has no natural IO `main`, so `ken run` on it fails post-elaboration
  ("last definition is not an IO tree") even though every fence checks
  correctly. This entry's `` ```ken ``/`` ```ken example ``/`` ```ken
  reject `` fences are verified via `ElabEnv::elaborate_ken_md_file`
  directly (`crates/ken-elaborator/tests/ds1_empty_dec_acceptance.rs`) —
  the IDENTICAL fence-checking code `ken run` itself calls before its
  separate (and here, inapplicable) IO-execution step. `ken-cli` would
  benefit from a check-only mode (elaborate + verify fences, skip
  IO-execution when the last declaration isn't IO-shaped) for library
  entries.
- **Abstraction candidate:** none beyond what §2 already provides.

## 7. References

- **Wikipedia** —
  [Decidability (logic)](https://en.wikipedia.org/wiki/Decidability_(logic))
  — general orientation on decidable propositions.
- **Lean 4 core** — `Decidable` (`Init/Prelude.lean`, part of the Lean 4
  repository, Apache-2.0) — <https://github.com/leanprover/lean4> — the
  `Decidable p := isFalse (p → False) | isTrue p` shape this entry's `Dec`
  mirrors (consulted for shape only, `CLEAN-ROOM.md`; no source copied).
- **Agda standard library** — `Relation.Nullary.Decidable`
  (`src/Relation/Nullary/Decidable.agda`, MIT) —
  <https://github.com/agda/agda-stdlib> — a second reference point for the
  same `yes`/`no`-tagged decision shape, with Agda's `⊥` as the `Type`-
  sorted false this entry's `Empty` mirrors.

## 8. Trust  derivation

1. **Spec / WP.** `docs/program/wp/catalog-ds-1-empty-dec.md` (this entry's
   build WP); no dedicated spec chapter yet — `Dec`'s reflective-decision
   role is described at `spec/20-verification/23-prover.md §3`.
2. **Public API.** `Empty`, `absurdEmpty`, `Dec`, `Yes`, `No`, `decide`,
   `yes`, `no`, `decEqDecides`.
3. **Source map.**

   | Task | Section |
   |---|---|
   | See the shape | [Definition](#2-definition) |
   | Use it | [Using it](#3-using-it) |
   | Check the computation facts | [Laws  proofs](#4-laws--proofs) |
   | Why this shape, not `Or`/`Sum` | [Design notes](#5-design-notes) |

4. **Derivation path.** `Empty` — `declare_inductive` via
   `data::elab_data_decl` (zero ctors), the same admission machinery every
   other prelude `data` uses. `Dec`/`Yes`/`No` — `declare_inductive`
   directly (kernel-direct, `Ω`-sorted param), the same technique already
   landed for `Or`/`Perm_rel`. `decide` — ordinary surface `match` over the
   now-global `Dec`/`Yes`/`No`. `absurdEmpty`/`yes`/`no`/`decEqDecides` —
   ordinary surface `fn`, this entry's own code.
5. **`trusted_base()` delta.** Exactly two new inductive admissions
   (`Empty`, `Dec`), each an ordinary `declare_inductive` kernel recheck —
   **zero new trust category** (no `Axiom`, no `declare_primitive`, no new
   `Term`/`Decl` variant). Confirmed on the Rust emission: `git grep -n
   'declare_inductive\|declare_primitive\|declare_postulate' \
   crates/ken-elaborator/src/prelude.rs` shows `Empty`/`Dec` alongside
   every pre-existing prelude inductive, and neither appears in a
   `declare_primitive`/`declare_postulate` call. `decEqDecides Int`'s
   `Yes`-branch proof (when instantiated at `Int`) carries the
   pre-existing `DecEq Int.sound` `Axiom` delta — not a new one this entry
   introduces (`§5`).
6. **Proof families.** `decEqDecides` — one case-split on `d.eq x y`, two
   branches (`sound`-direct / K7-then-`Bottom`-elim), no induction.
7. **Consumers.** None yet — DS-1 is the pilot; later catalog entries
   (decidable-order-driven sorting, checked key comparison) are the
   intended consumers.
8. **Validation evidence.**
   `crates/ken-elaborator/tests/ds1_empty_dec_acceptance.rs` — the smoke
   test (`Dec` admits, `elim_Dec` large-eliminates into `Type0`), the
   `trusted_base()`-delta grep (AC3), and elaborating this entry's
   `` ```ken ``/`` ```ken example ``/`` ```ken reject `` fences through the
   literate extractor (AC5).
