# Lawful structure classes — `Eq`, `DecEq`, `Ord`

> Status: **DRAFT v0 (ES4-classes).** The **first `packages/` catalog tranche**
> and the **pattern-setter** for every later ES4 package. An entry is *ordinary
> Ken* with a stated **derivation path from the built-ins** and a **zero
> `trusted_base()` delta**, and its core abstractions **carry their laws as
> propositions — proved, not postulated** (`README §2`,
> `../20-verification/21 §3`). **No new kernel feature:** a class is a
> **record** (`../30-surface/33 §5`, `../10-kernel/13 §3` Σ+η), a law is an
> **`Ω`** proposition (`../10-kernel/16 §1`); if the build adds a kernel
> rule/former it has over-built (`33 §5` line 150).

## 1. Why these lead ES4

`Ord` underpins `sort` and ordered `Map`/`Set` (`../30-surface/37 §6`/`§3.3`);
`DecEq` underpins membership and `nub` (`33 §5`); everything lawful downstream
builds on them. They are **structure classes** (`33 §5.1` — `Type`-valued
dictionaries with computational content, genuinely many per carrier,
coherence-gated), and they set the discipline every ES4 tranche follows: each
class + its instances is Ken over the built-ins, and on an **inductive carrier**
**every law field is inhabited by a real kernel proof** — the verification layer
may *assume the laws hold* because the instance proved them, with a **zero
`trusted_base()` delta**. (A *primitive* carrier like `Int` cannot prove its
∀-laws — no eliminator — so its lawful instance carries them as an **audited
delta**, not zero-delta; `§6`.) **K4 landed (`§6`):** the zero-delta
*real-proofs* path is realizable for the **live-`Eq`-conclusion** laws —
`Ω`-motive elimination (`3be0e30`) proves `refl`/`trans`/`total` + `Eq`'s
`refl`. The **concrete-equality** laws (`antisym`/`sound`/`complete`) needed the
**K5 + K7** kernel capability (`Top`-intro/`Bottom`-elim `1c84a30`;
`eq_at_inductive` operand-`whnf` `4ae2baf`, `../10-kernel/16 §8.1`) — both now
landed, so they are **real zero-delta proofs on main** (ES4-lawproofs-remainder,
`9a82745`). `Eq`'s `sym`/`trans` are likewise real zero-delta via
**case-split** (not K6; `§6`). The
first real zero-delta instances (`Ord Bool` `refl`/`trans`/`total`, `Eq Bool`
`refl`) are on main (**ES4-lawproofs**, `72e38a5`).

## 2. The three classes

Each class is a `class` declaration (`33 §5.2`) — a record of **operation
fields** (`Type`-valued) plus **law fields** (`Ω`-valued propositions). Bridge
notation, used throughout: `IsTrue b := Eq Bool b True : Ω` (`37 §6`; `Bool` is
real `data` since ES2, kernel `Eq` at `Bool` lands in `Ω`).

### 2.1 `Eq a` — decidable Boolean equality (an equivalence)

```
class Eq (a : Type) {
  eq    : a → a → Bool                                  -- the ergonomic `==`
  refl  : (x : a) → IsTrue (eq x x)
  sym   : (x y : a) → IsTrue (eq x y) → IsTrue (eq y x)
  trans : (x y z : a) → IsTrue (eq x y) → IsTrue (eq y z) → IsTrue (eq x z)
}
```

`eq` is the everyday Boolean equality; `refl`/`sym`/`trans` say it is an
**equivalence relation**. `Eq a` does **not** tie `eq` to the kernel's
propositional `Eq` — that is `DecEq`'s stronger promise (`§2.2`).

### 2.2 `DecEq a` — decides the kernel's propositional equality

```
class DecEq (a : Type) {
  eq       : a → a → Bool
  sound    : (x y : a) → IsTrue (eq x y) → Eq a x y     -- `eq` reflects kernel `Eq`
  complete : (x y : a) → Eq a x y → IsTrue (eq x y)     -- `eq` decides kernel `Eq`
}
```

`sound` + `complete` together make `eq` a **decision procedure** for the
kernel's `Eq a x y : Ω` (`../10-kernel/15`/`16`) — i.e. `DecEq a` yields
`Decidable (Eq a x y)` for all `x y`. This is exactly what `Map`/`Set`
membership and `nub` require (a real decision of *propositional* equality, not
just a Boolean guess). `DecEq a` **semantically subsumes** `Eq a`: a decision
procedure for kernel `Eq` (itself an equivalence, `16`) is
reflexive/symmetric/transitive, so the `Eq` laws are derivable — recorded as a
fact, **not** wired as a superclass constraint (`§4`). This is the *abstract*
subsumption (kernel `Eq` is provably an equivalence, `16`); *constructing* a
concrete instance's `Eq.sym`/`Eq.trans` as real zero-delta proofs is done by
**case-split** (`§6`), **not** the `conv_struct` `Eq`-congruence (**K6**): each
branch computes its concrete answer, so the swap-congruence K6 would supply is
never exercised. `Eq.refl` was always provable; `sym`/`trans` are now real
zero-delta proofs via the `Eq Bool` case-split WP.

### 2.3 `Ord a` — total order (supplies the verified-`sort` comparator)

```
class Ord (a : Type) {
  leq     : a → a → Bool                                -- the ≤-decision `sort` threads
  refl    : (x : a) → IsTrue (leq x x)
  antisym : (x y : a) → IsTrue (leq x y) → IsTrue (leq y x) → Eq a x y
  trans   : (x y z : a) → IsTrue (leq x y) → IsTrue (leq y z) → IsTrue (leq x z)
  total   : (x y : a) → IsTrue (leq x y || leq y x)     -- Bool-`||`; see §3
}
```

`leq` is the **same** `a → a → Bool` comparator the landed `sort`/`isSorted`
thread explicitly (`37 §6`, ES2-remainder `2358b4d`); `Ord` *supplies* it via a
dictionary (`§4`). `antisym` concludes the kernel's `Eq a x y` (`Ω`); the other
three are `IsTrue`-of-`Bool` propositions. (An `Ordering`-valued
`cmp : a → a → Ordering` is a derivable convenience — `Ordering` is a package,
`30 §4` — but `leq` is the primitive field, matching the threaded surface.)

## 3. Law-field sorts — every law is `Ω`, and no truncation is needed (AC1)

A class's **sort is kernel-computed** over all its fields (`33 §5.1`, the
both-components-keyed `sort_sigma`, `13 §4`): an operation field
(`eq`/`leq : … → Bool`) is `Type`-valued, so each record lands in **`Type`** — a
**structure class**, never forced to `Ω` (the `§5.1` soundness note: forcing a
relevant dictionary to `Ω` would collapse its computational content). The **law
fields are `Ω`-valued** (`33 §5.2`): each is a `Π` into `IsTrue _`/`Eq _` (both
`Ω`), so it is a proof-irrelevant proposition.

**The decidable comparator keeps every law `Ω`-clean — the truncation catch does
not fire here.** Totality is the one law that is a *disjunction* in spirit
("`x ≤ y` **or** `y ≤ x`"), and a **bare propositional** disjunction
`IsTrue (leq x y) ∨ IsTrue (leq y x)` is **proof-relevant** — *which* side holds
is content — so at `Ω` it would require the truncation `‖·‖` (`16 §6`;
[[proof-relevant-inductive-cannot-be-declared-at-omega]]). We **avoid** that
entirely: because `leq` is **decidable** (`Bool`-valued), totality is stated as
the **Boolean equation** `IsTrue (leq x y || leq y x)` — one `Bool` (the
value-level `||`) lifted through `IsTrue`, i.e.
`Eq Bool (leq x y || leq y x) True : Ω` — proof-irrelevant, **no truncation**.
This is the general rule for these classes: **a decidable operation lets every
law be a `Bool`-equation / `Dec` form → `Ω`-clean**; the truncation obligation
bites only a law stated as a bare propositional `∨`/`∃`. The *algorithm* still
case-analyses the decidable `leq x y : Bool` directly (matchable); the `Ω`-law
`total` is for the *prover*.

## 4. `Ord` subsumes the explicit comparator — reflect-don't-extend (AC2)

The verified `sort` (`37 §6`) threads an **explicit** `leq : a → a → Bool`. A
constraint `where Ord a` **provides that same `leq`** from the resolved
dictionary: `where Ord a` desugars to an implicit instance argument
`{d : Ord a}` (`33 §5.4`), and inside the body `leq` is `d.leq` (projection). So
the `Ord`-constrained `sort` and the explicit-comparator `sort` are the **same
view** — one `sort`, whose comparator is either passed explicitly or supplied by
the dictionary:

```
view sort {a} (leq : a → a → Bool) (xs : List a) : { ys | isSorted leq ys ∧ Perm ys xs }
                          -- explicit comparator (37 §6), and:
view sort {a} (xs : List a) : { ys | isSorted (d.leq) ys ∧ Perm ys xs }  where Ord a
                          -- `d.leq` supplied by the dictionary — the SAME view
```

There is **no second `sort`** and **no new mechanism**: `where Ord a` is
ordinary implicit-dictionary insertion (`33 §5.4`), and a caller may still pass
a **named, non-canonical** `Ord` value explicitly (`33 §5.5`,
`sortBy byLength xs`) — the dependent-types escape hatch — without perturbing
canonical resolution. The refinement predicates (`isSorted`, `Perm`) are
unchanged; `Ord`'s `total` law is what lets a verified `sort` *discharge* the
sortedness obligation the explicit form could only *state*.

## 5. Laws PROVED, not postulated — the hard soundness gate (AC3)

An instance elaborates to a **record value** (`33 §5.3`) — a right-nested pair
of the operation implementations **and the law proofs** — admitted through the
real `declare_def` path and **re-checked by the kernel**:

```
inst_Ord_Bool : Ord Bool  ≡  (bool_leq , refl_pf , antisym_pf , trans_pf , total_pf)
```

(The exemplar uses the **inductive** carrier `Bool`, whose law proofs are real —
see `§6` on why a *primitive* carrier like `Int` cannot be zero-delta lawful.)
Each law proof `pⱼ` is a **real kernel proof** of its `Ω`-proposition, checked
at its Σ-Intro position (`33 §5.3`, `13 §2`) — **not** a `postulate`, **not** a
hole (`sorry`), **not** an empty stub. A verified algorithm that assumes "the
`Ord` dictionary carries the total-order laws" is sound **only** because those
proofs are real: a client cites `d.antisym`/`d.total` directly (`33 §5.2` η).

**This is the same coin as the ES1 zero-delta invariant, read from the law
side.** A law field inhabited by a `postulate` becomes an **`Opaque` entry in
`trusted_base()`** — so the instance's `trusted_base_delta` (`25 §3`) is
**non-empty**, violating AC1; a law field left as a hole **fails the kernel
re-check** outright. So — **for a carrier whose laws are provable (an
*inductive* carrier; `§6`)** — **"lawful instance" ≡ "zero-delta instance"**:
the laws are proved iff nothing enters the trust root by the back door. (A
*primitive* carrier can't prove its laws at all, so its lawful instance is
separately an **audited-delta** one — `§6`; the discriminating case below is
over an inductive carrier, where a postulate *is* a defect.) A "the dictionary
carries the laws" claim that passes **green-vs-green against a law-less
dictionary** is the predicate-definedness dual
([[lawful-class-instances-must-carry-law-proofs]]; the ES2 analog is
`isSorted`/`Perm` being *defined*, not postulated). **Discriminating obligation
(hard):** the conformance corpus must construct a **law-less** `Ord`-shaped
instance (a dictionary whose law fields are postulated/holed/stubbed) and show
it is **rejected as unlawful** (non-empty delta / re-check failure) **while**
the real law-carrying instance **accepts** — the verdict must **flip**. The
producer check is structural: **grep the instance's law fields for
`declare_postulate`/holes — their *absence* is the guarantee**
([[kernel-backed-claim-grep-the-emission-not-the-name]]); a test that merely
asserts "an `Ord Bool` resolves" is vacuous
([[conformance-hand-feeds-the-deliverable]]).

## 6. Derivation paths and zero `trusted_base()` delta (AC1)

The **classes** bottom out in the built-ins with **no new former**:
`Eq`/`DecEq`/`Ord` are `class` declarations = **record types** (`33 §5.2`,
right-nested Σ over `13 §3`), built from `Bool` (prelude, `30 §4`) + the
kernel's `Eq`/logic vocabulary (`15`/`16`, prelude) + the value-level `||`/`&&`
on `Bool`
(derived, `34`) + the Σ/record machinery. **No new former, zero delta.** The
**instances** are the subtle part.

**Zero-delta lawfulness requires an INDUCTIVE carrier — the load-bearing
precondition, two orthogonal axes.** An instance's law fields are
**∀-quantified** props (`∀ x. IsTrue (leq x x)`, …); to inhabit them with **real
kernel proofs** (zero axioms) an instance must satisfy **both**:

1. **The law's *sort*** — each law lands in `Ω`. A **decidable** (`Bool`) op
   keeps every law a Bool-equation → `Ω`-clean, no truncation (`§3`); a bare
   propositional `∨`/`∃` would need `‖·‖`.
2. **The carrier's *provability*** — the carrier has an **eliminator**. A ∀-law
   is proved by **case-analysis / induction on the carrier**, so the carrier
   must be **inductive**. `Bool` (real `data Bool = True | False`, `34`) proves
   every law by **finite case-split** (`bool_leq x x` reduces on each
   constructor) — zero axioms.

So the **zero-delta guarantee holds for inductive-carrier instances**
(`Ord Bool` — the exemplar — and any user `data`): `instance` = a
**`declare_def` record value** (`33 §5.3`) of ops + **real** law proofs,
re-checked, never
`Opaque`/`Primitive` — nothing new enters `trusted_base()`. (A `derive (DecEq)`
on a user `data`, `33 §5.6`, is likewise untrusted-generated then
kernel-re-checked over the type's constructors — zero delta, *because* the
carrier is inductive.)

**K4 landed — the live-`Eq` fragment is zero-delta; the concrete-equality laws
need more.** Constructing per-branch law proofs requires the kernel to
**dependently eliminate a `Type`-inductive into an `Ω`-motive**
(`λx. P x : Bool → Ω`) — to *prove* a per-branch `Ω`-proposition, not *select
which*. That is the **K4** capability (`14 §3` "Elimination into `Ω`", landed
`3be0e30`): an **inductive** carrier (`Bool` / user `data`) proves — by finite
case-split — the laws whose per-branch obligation stays a **live `Eq`** (`Ord`'s
`refl`/`trans`/`total`, `Eq`'s `refl`), which are **zero-delta now**. (A
*primitive* carrier still cannot — no eliminator — so it stays
**audited-delta**, below; the carrier axis is unchanged.)

The **concrete-equality-conclusion** laws — `Ord`'s **`antisym`** and `DecEq`'s
**`sound`**/**`complete`**, which conclude or hypothesize the kernel `Eq a x y`
— have per-branch obligations that reduce to a concrete **`Top`** (the
trivially-equal case → `Top`-**introduction** with `tt`) or **`Bottom`** (the
contradictory-hypothesis case → `Bottom`-**elimination** / ex-falso with
`absurd`). Closing them needs **two** kernel capabilities, not one:

1. **K5** — the **observational-fragment completion** (`../10-kernel/16 §1.4`;
   `tt`-intro / `absurd`-elim, the textbook unit/empty pair, sound because
   `Bottom` is *empty*, distinct from the K4-forbidden singleton-elim-*out*) —
   **landed** (`1c84a30`). K5 supplies the `tt` / `absurd` *terms*.
2. **K7** — the `eq_at_inductive` **operand-`whnf`** completeness fix
   (`../10-kernel/16 §8.1`: whnf the two `Eq` operands before the
   constructor-head compare, mirroring `eq_at_type`). K5's
   `tt` / `absurd` fire only once the goal / hypothesis `Eq` has actually
   **reduced** to `Top` / `Bottom`, and these three laws wrap the carrier
   through the instance's **own operation** (`bool_leq` / `bool_eq`), so their
   operands are **redexes**, not bare constructors: `antisym`/`sound`'s
   contradictory hypothesis `IsTrue (bool_leq/eq True False)` and `complete`'s
   **equal**-branch goal `IsTrue (bool_eq True True)` both stay a **neutral
   `Eq`** until the operand is whnf'd — exactly the K7 gap. Only the
   bare-constructor equal branches (`antisym`/`sound`'s `Equal Bool True True`)
   reduce under K5 alone.

So a **complete** zero-delta `Ord Bool` / `DecEq Bool` (`antisym` mandatory for
a total order; `sound`/`complete` for decidable equality) needed the **K5 + K7**
kernel capability (K5 `1c84a30`; K7's operand-`whnf` fix `4ae2baf`, `16 §8.1`) —
both now landed, and the three laws are **real, kernel-checked, zero-delta
proofs on main** (ES4-lawproofs-remainder `9a82745`: `antisym` closes via
`tt`/`absurd`, `sound`/`complete` via `absurd`; **no `Axiom` remains** in either
instance). **None needed K6** (no swapped-`Eq` hypothesis-reuse across a stuck
congruence); K4 alone realized only the live-`Eq`-conclusion fragment, and the
ES4-lawproofs build surfaced the K7 gap by pushing the real proofs to a wall
(Architect-ruled) before it landed. *(The staging ran the full three-state
lifecycle: the K5 un-stage `0feb2c8` over-claimed these "realizable now via K5"
— silently assuming the operation-wrapped operands reduce, the separate K7 step
— corrected to a K5+K7 park by `4466807`, then realized here once K7 `4ae2baf`
+ the wiring `9a82745` landed.)*

**`Eq`'s `sym`/`trans` realize by case-split — and K6 is grounded-but-
customerless.** Proving `Eq`'s **symmetry**/**transitivity** (from `Eq a x y`
derive `Eq a y x`, and compose) was first thought to need a **`conv_struct`
`Eq`-congruence** arm — call it **K6** — to reuse a hypothesis across a swap. It
does **not**: a **full case-split** on the carrier (for `Bool`, `sym` splits
`(x, y)`, `trans` splits `(x, y, z)`) closes every branch concretely — `tt` on a
reflexive-conclusion branch, `absurd` on a branch whose hypothesis reduces to
`Bottom` (`bool_eq` of mixed literals ⇝ `False` via K7 before the head-check) —
so **no hypothesis is ever reused across a swap** and the K6 congruence is
**never exercised** (the `Eq Bool` `sym`/`trans` case-split WP; Architect-ruled
`evt_78ntsfnyjdtq6`). These are **real, kernel-checked, zero-delta proofs**, no
`Axiom`. **K6 accordingly stands grounded-but-customerless:** a sound *forward
general-completeness* capability (comparing two genuinely-stuck `Eq`
propositions) that nothing in the lawful-classes corpus consumes — a *sound
positional* K6 fix would **not** have closed this pair; only an **unsound
cross-wise** arm could, which is a hard **NO**, never taken. So K6 is decoupled
from `Eq`'s realization: parked as a sound-but-unconsumed forward gap, **not** a
blocker on any instance. (`Eq`'s `refl` was always zero-delta — its goal routes
through an unresolved `bool_eq x x`, keeping the `Eq` live so `Refl` fires.)

**Realization status.** The **real, kernel-checked, zero-delta** law-carrying
instances are **on main** (Team Language, `packages/lawful-classes/`):
`Ord Bool` `refl`/`trans`/`total` + `Eq Bool` `refl` (`72e38a5`), and now
`Ord Bool`'s `antisym` + `DecEq Bool`'s `sound`/`complete` (`9a82745`,
ES4-lawproofs-remainder) — so `Ord Bool` and `DecEq Bool` are **complete
zero-delta lawful instances**, **no `Axiom` remaining**. `Eq Bool`'s
`sym`/`trans` are likewise **real zero-delta proofs** now — via case-split (the
`Eq Bool` `sym`/`trans` WP), not K6 — so **all three `Bool` instances
(`Ord`/`Eq`/`DecEq`) are complete
zero-delta lawful, no `Axiom` anywhere**. K6 remains a
sound-but-**customerless** forward capability. The discriminating shape — a
law-less *inductive* instance is an **avoidable** delta, hence a
defect — is **live**
([[soundness-ac-static-vs-runtime-face]]). The design is unchanged throughout;
only the gate states and build-time availability move.

**Primitive carriers (`Int`/`Float`/`String`/`Char`) fail the carrier axis — so
their lawful instances are NOT zero-delta.** A K1 primitive is **opaque to δ**
(`int_leq x x` on a *variable* `x` does not reduce — primitive reductions fire
on literals) and has **no induction principle**, so its total-order laws are
**not
kernel-provable**; the only inhabitant of a law field is a **`postulate`** → an
`Opaque` entry → a **non-empty `trusted_base_delta`**. The **operation** half is
still fine (wrapping the audited `Int` comparison, `30 §6` F2, adds no new
entry); the **law** half cannot be zero-delta. Two honest options, and the
instance must **declare which**:

- **Audited-delta** (the pragmatic default for the arithmetic/ordering
  primitives Ken ships — you cannot simply lack `Ord Int`): provide the instance
  with the primitive's laws **postulated and transparent in
  `trusted_base_delta`** — the same trusted-by-audit surface as the primitive op
  itself (the honest FFI/primitive posture,
  [[tested-not-trusted-posture-needs-reachability-precondition]]). Lawful and
  *usable by the prover*, but **not zero-delta** — the assumption is
  **visible**, never hidden.
- **Deferred:** omit the instance until `Int` gains reduction rules + an
  induction principle that make the laws provable (then it becomes zero-delta).

`Int` is therefore **illustrative-only** in this catalog — a primitive carrier
whose lawful instance carries an audited delta — **not** a zero-delta exemplar.
The zero-delta exemplar is the **inductive** `Bool` (and user `data`).

## 7. Placement, `packages/` layout, and the un-defer (AC4/AC5)

**Catalog placement.** These entries realize the `README §2` "Lawful classes"
row; the catalog index (`README`) points here, and the **realized Ken source**
lives under the in-repo **`packages/`** tree (`../../packages/README.md`) — the
layout established by this WP as the pattern for every later ES4 tranche: one
package per module unit (`33 §3`), each carrying its **derivation-path +
`trusted_base()` delta** declaration (zero on an inductive carrier; an audited
delta for a primitive carrier, `§6`). This spec chapter is the **contract**; the
Team-Language build follow-on lands the `.ken` source (classes + canonical
law-carrying instances) + wires `where Ord a` to supply the comparator.

**Un-defer (AC5).** ES2-remainder deferred the lawful `Ord` and CV task #27
descoped two conformance cases to `(deferred, gated-on-WP)` because the landed
`Ord` was an empty stub. **This WP is that gate.** On its build, the two cases —
`surface/collections/user-ord-instance-drives-verified-sort` and
`user-ord-sort-emits-both-conjuncts` — **un-defer**, re-pointed from the
`33 §5.4` desugared-path placeholder to the **real `Ord` here**: a user
`instance Ord K` (law-carrying) drives the verified `sort` via `where Ord a`
(`§4`), and its VC carries both conjuncts identically to the explicit form. (The
conformance edit names the target; the actual un-defer rides the build.)

## 8. What the build delivers + acceptance

**Build (Team Language, named follow-on — not this WP):** `Eq`/`DecEq`/`Ord` +
**canonical instances carrying real law proofs** under `packages/`, wired into
`ken-elaborator`; `where Ord a` supplies the `sort` comparator (`§4`); the CV
#27 cases un-deferred against the real classes.

**Acceptance (spec-design piece):**

- **AC1 (signatures + zero-delta).** The three classes are structure classes
  (`33 §5.1`) with `Ω`-valued law fields (`§3`), each with a stated derivation
  path; **zero `trusted_base()` delta on an inductive carrier** (`§6`) — no new
  postulate, no kernel former. (A primitive carrier fails the carrier axis and
  carries an **audited delta**, `§6`.)
- **AC2 (`Ord` subsumes the comparator).** `where Ord a` supplies the same
  `leq : a → a → Bool` the explicit `sort` threads (`§4`) — **same view, no
  second `sort`, no new mechanism**.
- **AC3 (laws PROVED — hard, soundness).** A canonical instance **carries real
  proofs** of its law fields (`§5`); a **law-less** dictionary is **rejected as
  unlawful** (non-empty delta / re-check failure) — the discriminating case
  **flips** against it, verified by grepping the law fields for
  `declare_postulate`/holes (absence = the guarantee).
- **AC4 (catalog + `packages/` layout).** `packages/` layout established
  (`../../packages/README.md`), catalog placement in `README`, derivation paths
  stated — the pattern for every later ES4 tranche.
- **AC5 (un-defer).** The two CV #27 `(deferred, gated-on-WP)` cases named as
  un-deferring on this WP's build, re-pointed to the real `Ord` (`§7`).

**Conformance:** `../../conformance/surface/collections/` (the un-deferred
`user-ord-*` cases) + the discriminating **law-proof-present flips against a
law-less dictionary** net. **QA (build):** producer-grep the instance law fields
for `declare_postulate`/holes — their **absence** is the check
([[kernel-backed-claim-grep-the-emission-not-the-name]]).
