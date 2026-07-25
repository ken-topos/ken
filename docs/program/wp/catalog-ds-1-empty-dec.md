# WP — DS-1: `Empty` + `Dec` (catalog data-structures program, first WP + `.ken.md` format pilot)

**Owned by the Steward** (frame); **design pinned by the enclave** (Architect
ruling on `spec/10-kernel/14`, `spec/20-verification/23 §3`, and the landed
prelude; spec-author fidelity co-review on the sort/surface calls, and
pen-holder of this brief); **built by Foundation**. This is the **first WP of
the catalog data-structures program** and the **`.ken.md` format pilot** — a
deliberately small vertical slice that exercises the full reframed machinery
(the `.ken.md` format, the just-landed `catalog/guide/` incl. its §2 `def` / §7
named-proof sections, fence roles, this design→build→gate pipeline, the retro
loop). Keep it **small and exemplary, not exhaustive**.

Per the operator, this is the **one** WP we run before a process review; DS-2…DS-9
stay parked.

## Objective

Deliver the three false-and-decidability primitives every later data-structure
entry leans on, in Ken's own terms and grounded in landed Ken:

- **`Empty`** — the Type-sorted false, with `absurd : (C : Type) → Empty → C`.
- **`Dec P`** — a decidability container with `decide`, `yes`, `no`.
- **The bridge** `DecEq a → (x y : a) → Dec (Equal a x y)`.

## The pinned design (forks 1–4, enclave-resolved — do not re-open)

The design is fully pinned; the build implements it. One **build-time
confirmation** (the step-1 smoke test, below) fixes a single admittance fact —
it is not a design fork.

### Fork 1 — `Empty` + `absurd` (surface-authorable)

`Empty` is a **fresh Type-sorted zero-constructor inductive**, exactly the
`spec/10-kernel/14-inductive.md` canonical ⊥ (`14`, "Canonical examples":
`data Empty : Type 0 where` — no constructors):

```
data Empty : Type0 =
```

`absurd : (C : Type) → Empty → C` is its **empty eliminator** — the dependent
eliminator `elim_Empty` over zero cases, i.e. large elimination into any `Type`
(`14 §3` permits large elimination under the predicative universe rules). No
methods, since `Empty` has no constructors.

`Empty` **coexists with the Ω-valued `Bottom`** already in the prelude — it does
**not** replace it (subsume-don't-proliferate / coexist-when-roles-differ):

- `Bottom : Ω₀` is the **proof-irrelevant** Ω-false (the logic's falsehood;
  `spec/10-kernel/16-observational.md §1`).
- `Empty : Type0` is the **computational** false — needed so `Dec`'s refutation
  branch sits at **Type** and `decide` large-eliminates into Type.

This is the standard `False : Prop` vs `Empty : Type` split (Lean; Agda `⊥`
per universe). Both are correct to have.

**Surface status:** `Empty` has **no parameters**, so `data Empty : Type0 =` is
plain surface `.ken.md` `data`. `absurd`, `decide`, and the bridge are likewise
surface-authorable. See "The surface / kernel-direct split" below for the one
piece that is **not**.

### Fork 2 — `Dec` is Lean's `Decidable` shape (kernel-direct), NOT `Or` and NOT `Sum`

```
data Dec (P : Omega) : Type0 = Yes P | No (P -> Empty)
```

`spec/20-verification/23-prover.md §3` is decisive: `Decidable P` is the
**derived** sum `P + (P → Empty)` — explicitly *"not a kernel primitive"* — and
the reflective-decision mechanism requires the kernel to **compute** `dec a` to
`inl proof` / `inr refutation` (canonicity C6) and branch on the result. That
demands a **computationally-relevant** (Type-sorted, large-eliminable) container.
Two landed candidates were rejected, and the reason is a spec-fidelity fact:

- **NOT the Ω `Or`** (`crates/ken-elaborator/src/prelude.rs`, declared with
  `[a : Ω₀, b : Ω₀]`): it is **proof-irrelevant** and **cannot large-eliminate
  into Type** — you cannot recover *which* disjunct, which is the entire content
  a decider needs.
- **NOT the homogeneous `Sum : Type → Type → Type`** (the landed
  `state_eff::declare_sum`, `prelude.rs:270`): the bridge target is
  `Dec (Equal x y)` and **`Equal a x y : Ω`** (`16 §1`,
  `16-observational.md:224` — `Eq A a b : Omega_l`), while **Ω does not inject
  into Type0** (Ken is non-cumulative, `12 §3` / `OQ-2`; `12 §5` +
  `16 §1.1`: "no cumulativity for Omega… `Omega_l : Type (suc l)`… lifting is
  explicit", and no `El`/coercion Ω→Type exists). So a *homogeneous* `Type`
  sum cannot carry an **Ω-proof** left payload and a **Type** refutation right
  payload at once.

A **declared inductive fixes each field's sort independently** — the principled
answer, not a workaround: `Yes` takes the Ω-proof `p : P`, `No` takes the Type
refutation `f : P → Empty`, the family sits at `Type0` with a **Type-relevant
Yes/No tag**, so `decide` large-eliminates into Type. This is precisely Lean's
`Decidable p : Type`; the spec's `P + (P → Empty)` is *genuinely mixed-sort*
once Ω is a separate universe (Lean consulted for the shape only — no source
copied; `CLEAN-ROOM.md`).

**Ω-field on a Type container is admissible — landed precedent.**
`spec/10-kernel/13-pi-sigma.md:133`: the refinement `{x : A | φ} = (x : A) × φ`
"stays in `Type (max l_A l_φ)`" — an **Ω second component on a Type-sorted Σ**.
`Dec` is the same move (an Ω-erasable payload) plus a retained two-constructor
tag.

### Fork 3 — the `DecEq a → (x y : a) → Dec (Equal a x y)` bridge

Constructible from the **landed** `DecEq` (`catalog/packages/Core/Classes/`,
`class DecEq a { eq : a → a → Bool ; sound : … → Equal a x y ;
complete : Equal a x y → IsTrue (eq x y) }`). Case on `eq x y : Bool`:

- **`True`** (`IsTrue (eq x y)`): `sound x y` yields `Equal a x y` → `Yes …`.
- **`False`**: build the refutation `Equal a x y → Empty` (→ `No …`) via the
  cross-sort ex-falso in Fork 4.

### Fork 4 — cross-sort ex-falso `Bottom(Ω) → Empty(Type)` (available)

`spec/10-kernel/16-observational.md §1` **Bottom-Elim** explicitly permits the
motive `C : Type l` (as well as `C : Omega_l`), so a vacuous large-elimination
from the Ω-`Bottom` into the Type-`Empty` is sound and landed-spec-backed.

The refutation's obligation: in the `eq x y = False` branch,
`IsTrue (eq x y) = Equal Bool False True`, which must be turned into `Bottom`
(then `absurd`-ed into `Empty`). Per the same rule's caveat — *"an impossible
equality proposition does not by itself synthesize a closed `Bottom`"* — this
must be **produced**, via Bool no-confusion / K7:

- **Honest for an inductive carrier** (`DecEq Bool`): `eq` reduces and K7 gives
  `Equal Bool False True → Bottom` with a real proof, **zero trusted-base delta**.
- **`Axiom`-backed for `DecEq Int`** (`lawful_classes.ken`: `sound = Axiom`,
  because `Int` is a primitive with no induction) — this is exactly the
  `catalog/guide/` §1.1 opaque-primitive / `Axiom` gotcha, at the `DecEq`/`Int`
  boundary (DS-6 hits this again).

**Demonstrate the bridge over an inductive carrier (`DecEq Bool`), not only
`DecEq Int`** — an `Axiom`-only showcase is a vacuous "verified" claim.

## The surface / kernel-direct split (load-bearing scope fact)

Surface `data` **hardcodes every parameter to `Type 0`**
(`crates/ken-elaborator/src/data.rs:45`) — there is **no way to spell an
Ω-sorted parameter** in surface `.ken.md`. So `data Dec (P : Omega)` is **not**
a surface declaration: it must be built **kernel-direct** via `declare_inductive`
(the same one-level-below-the-surface-sugar technique the landed `Or`, `Sum`,
and `Perm_rel` already use), which is **zero new trusted-base category** (an
ordinary `declare_inductive` admission, kernel-rechecked).

Consequently DS-1 is a **surface `.ken.md` entry over a minimal kernel-direct
prelude `Dec`**, and its merge carries a **`crates/` delta → CI-gated, not
doc-only**:

| Piece | Where | Surface or kernel-direct |
|---|---|---|
| `Dec`, `Yes`, `No`, `decide` (+ `elim_Dec`) | prelude (`prelude.rs`) | **kernel-direct** (`declare_inductive` + `globals.insert`) |
| `Empty`, `absurd` | surface `.ken.md` entry | surface `data` / eliminator |
| `yes`/`no`/`decide` ergonomic wrappers | surface entry | surface `fn`/`const` |
| the `DecEq → Dec (Equal x y)` bridge | surface entry | surface `fn` |

## Build step 1 (the one confirmation — do this first)

Before writing the entry, a ~3-line smoke test on the kernel-direct `Dec`:

1. `data Dec (P : Omega) : Type0 = Yes P | No (P -> Empty)` **admits** —
   positivity is trivial (`Dec` does not occur in its own constructor
   arguments), and the universe check passes with the Ω-field per the
   `13-pi-sigma.md:133` Σ precedent.
2. `elim_Dec` **large-eliminates into a `Type` motive** (the content `decide`
   needs).

**If it admits → build straight through.** **If it does not**, the fallback is
the spec's *own* reflective-decision mechanism (`23 §3`): do not store the
Ω-proof — decide on the `Bool` tag and recover proof/refutation **per-branch**
via `DecEq`'s `sound`/`complete` (Type-sorted `Bool`, no Ω-storage). Either way
DS-1 stays small and the fallback needs **no kernel move**. The Architect
confirms the smoke-test result at the build gate.

## Deliverables

1. **Prelude (kernel-direct):** `Dec : Omega → Type0` with `Yes`/`No`
   constructors + generated `elim_Dec`, and a `decide` accessor — registered in
   `elab.globals`, kernel-rechecked, `trusted_base()` delta accounted (an
   ordinary inductive admission, no new trust category).
2. **Surface `.ken.md` catalog entry** (authored against `catalog/guide/`, using
   `data`/`fn`/`const` and the `def`/`prop`/`theorem` surface as apt): `Empty` +
   `absurd`; ergonomic `yes`/`no`/`decide` surface; the
   `DecEq a → (x y : a) → Dec (Equal a x y)` bridge — with worked, **checked**
   `` ```ken example `` fences and at least one `` ```ken reject `` where a
   discriminator earns its place.

## Acceptance criteria

- **AC1 — smoke test passes:** `Dec` admits and `elim_Dec` large-eliminates into
  a `Type` motive (or the `23 §3` Bool-tag fallback is used, with a one-line note
  why). Architect confirms at the gate.
- **AC2 — `Empty`/`absurd` surface-authored** and elaborate; `absurd`'s large-elim
  motive typechecks.
- **AC3 — `Dec`/`Yes`/`No`/`decide` prelude piece** kernel-rechecked; the
  `trusted_base()` delta is exactly the new inductive's admission (no new trust
  category); grep the emission, not just a `.ken` view.
- **AC4 — bridge builds and is demonstrated over an inductive carrier**
  (`DecEq Bool`, honest via no-confusion/K7), not only `DecEq Int` (`Axiom`) —
  the showcase must not be vacuous.
- **AC5 — `.ken.md` fences check:** every `` ```ken example ``/`` ```ken reject ``
  behaves as authored; `ken run` exits 0 on the entry.
- **AC6 — clean-room attested:** authored from `/spec` + `catalog/guide/` +
  landed Ken; Lean's `Decidable` consulted for shape only, no source copied.
- **AC7 — scoped to the pilot:** small and exemplary; name what DS-1 defers
  (e.g. general decidable relations, `Dec`-combinators) rather than sprawling.

## Sources and the clean-room boundary

Landed Ken (`spec/10-kernel/14`, `16 §1`, `23 §3`; the prelude `Or`/`Sum`
precedents; landed `DecEq`), `catalog/guide/`, and first principles. General
dependent-type practice (Lean/Agda `Decidable`/`⊥`) may be **enclave-consulted**
for shape; the entry is written in Ken's terms and **copies no reference source**
(`CLEAN-ROOM.md`). Foundation builds from this brief + landed Ken, not from
`local/refs/`.

## Cadence and gate

Steward frame → **enclave design pin (done; this brief transcribes it)** →
Architect fidelity-gates this committed text → merge to `main` → Steward pulls
Foundation → build (smoke test first) → Architect (soundness + smoke-test
confirm) + CV (conformance) at the build gate → publisher path. Because DS-1
carries a `crates/` delta (the kernel-direct `Dec`), the entry merge is
**CI-gated, not doc-only**. On merge, the catalog data-structures program has its
validated pilot; everything past DS-1 waits for the operator's process review.
