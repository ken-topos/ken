# Sum types, pattern matching, and refinements

> Status: **DRAFT v0**. Proposal-level for syntax; **normative and high-priority
> for the feature.** Sum types, real constructors and eliminators, `match` with
> exhaustiveness, `Result`/`Option`/`Either`, and refinement types are
> first-class and fully checked from day one — each `data` declaration lowers to
> a genuine inductive type with real constructors and a real eliminator, never
> an opaque base.

## 1. Sum types (real, not stubbed)

```
data Option a = None | Some a
data Result e a = Err e | Ok a
data Color = Red | Green | Blue
data Tree a = Leaf | Node (Tree a) a (Tree a)
data Expr = Lit Int | Add Expr Expr | Neg Expr
```

- `data` declares an **inductive type** (`../10-kernel/14`): the type former,
  its **constructors**, and (generated) its **dependent eliminator** `elim_D`.
  Constructors are *real introduction forms* and there is a *real eliminator* —
  values can be built and taken apart, with computation (`elim_D … (Some x) ≡
  …`).
- Constructors may carry arguments (positional or named-record style, `32 §1`)
  and may be **recursive** (subject to strict positivity, `14 §2`).
- `Result`, `Option`, `Either` are ordinary `data` decls in the prelude
  (`../50-stdlib/`): fallibility and absence are honest sum types, not sentinel
  values.

## 2. Indexed families (GADT-like)

Constructors may target different **indices** (`../10-kernel/14 §1`), giving
length-indexed and well-typed-by-construction data:

```
data Vec a : Nat → Type {                 -- explicit-index form
  VNil  : Vec a 0
  VCons : {n : Nat} → a → Vec a n → Vec a (n+1)
}
```

This is the same power refinement types give, expressed in the data declaration;
a `view head {n} (v : Vec a (n+1)) : a` cannot be applied to an empty vector —
the non-emptiness is in the type.

## 3. Pattern matching

```
view area (s : Shape) : Decimal = match s {
  Circle r       => 3.14159d * r * r
  Rect   w h     => w * h
  Tri    b h     => 0.5d * b * h
}
```

- `match` scrutinizes one or more expressions and selects the first arm whose
  pattern matches; patterns are as in `32 §4` (constructors, variables,
  wildcards, literals, tuples, records, as-/or-patterns, optional guards).
- **Elaborates to `elim_D`** (`../10-kernel/14 §3`, `39`): nested matches
  compile to nested eliminators; the **dependent** motive is recovered so
  matching refines the result type per branch (essential for indexed families
  and the body-as-motive obligations of `../20-verification/22 §4`).
- In each branch, the scrutinee is **definitionally** the matched constructor
  (`s ≡ Circle r`), which is what the verification layer turns into a hypothesis
  (`../20-verification/22 §3`).

## 4. Exhaustiveness and reachability (required)

The checker MUST verify:

- **Exhaustiveness** — the arms cover every constructor (every value of the
  scrutinee type matches some arm); a missing case is a **compile error** with
  the unmatched pattern reported. (Indexed families: only the *type-possible*
  constructors at the given index must be covered — an impossible case need not
  be written.)
- **Reachability** — every arm is reachable; a redundant arm is a warning/error.

Ken requires this safety from day one. Exhaustive `match` over a closed `data`
needs no "default" and the compiler proves totality of the case analysis.

## 5. Refinement types

`{ x : A | φ x }` — the type of `x : A` such that `φ x : Ω` holds
(`../10-kernel/12 §5`, `../20-verification/21 §2`). At the surface:

```
type Nat   = { n : Int | n ≥ 0 }
type NonEmpty a = { xs : List a | xs ≠ Nil }
view head {a} (xs : NonEmpty a) : a = match xs { Cons x _ => x }
```

- Refinements **coerce to the underlying type** silently (`{x:A|φ} ≤ A`); using
  an `A` where `{x:A|φ}` is wanted **emits an obligation** `φ`
  (`../20-verification/22 §2`) the prover discharges or surfaces as a hole.
- The proof component is a **mere proposition** (`12 §5.1`) → **no runtime
  payload**; refinements are zero-cost at runtime and pure compile-time
  enforcement.
- Refinements compose with `data`, records, and function arguments/results, and
  are how `requires`/`ensures` desugar (`21 §1`).

## 6. Smart constructors & views (optional sugar)

Pattern synonyms / view patterns (matching through an abstraction) and smart
constructors (a `view` that enforces an invariant and returns a refined type)
are ergonomic sugar over §1–§5; whether to include them is **OQ-syntax**. The
semantic core is constructors + `elim_D` + refinements.

## 7. What WS-L must deliver here

Real sum types with constructors + eliminators (no opaque lowering); indexed
families; `match` → `elim_D` with dependent motives; **exhaustiveness +
reachability** checking; `Result`/`Option`/`Either` in the prelude; and
refinement types with coercion + obligation emission. Acceptance is part of
**G6** (real sum types end-to-end). Conformance:
`../../conformance/surface/data-match/` — including an exhaustiveness-failure
regression and a "construct then eliminate computes" test confirming
constructors and eliminators reduce as specified.
