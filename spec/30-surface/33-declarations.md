# Declarations, modules, and constraints

> Status: **DRAFT v0**. Proposal-level for syntax; normative for the *features*.
> Modules/imports, definitions, records, visibility, and Ken's constraint
> mechanism — **typeclasses-as-subobjects-of-the-universe** (the digest's
> elegant idea, genuinely greenfield since the prototype has no open user
> typeclasses).

## 1. Definitions

- **`view f … = e`** — a (pure) function; elaborates to a global Π/λ definition,
  gated by SCT (`../10-kernel/17 §4`). Effectful variants carry an effect row
  (`36`).
- **`let x : A = e`** — a value definition (a nullary `view`).
- **`type T … = …`** — a type alias or a refinement/`Σ`/Π type abbreviation
  (transparent; unfolds by δ).
- All top-level definitions are **mutually recursive within a module** if the
  SCT check accepts the group; otherwise the offending recursion is reported
  (`17`).
- Definitions may be **generic** (implicit type/level parameters, `39`): `view
  id {A : Type} (x : A) : A = x`.

## 2. Records (products)

```
record Point { x : Int, y : Int }
record User  { name : String, age : { n : Int | n ≥ 0 } }   -- refined field
```

- Elaborates to right-nested Σ with definitional η (`../10-kernel/13 §3`), so
  field access `p.x`, record literals `{ x = 1, y = 2 }`, punning `{ x, y }`,
  and **functional update** `{ p | y = 3 }` all have their expected definitional
  behaviour.
- Fields may be **dependent** (a later field's type mentions an earlier field)
  and **refined** (carry a proposition) — records are Σ, so this is free.

## 3. Modules and imports

- **`module M { … }`** groups declarations under a namespace; a file is an
  implicit module named by its path.
- **`import M`** brings `M` into scope qualified (`M.foo`); **`import M as N`**
  aliases; **`import M (foo, Bar)`** selectively; **`use M`** opens `M`
  unqualified (use sparingly).
- **Cross-package imports** resolve through the package manager
  (`38`/`../50-stdlib/`): a manifest (`ken.toml`) + lockfile (`ken.lock`) with
  content-addressed, pinned dependencies (the prototype's git-pinned model,
  generalized; content-addressing makes builds reproducible — a marketable
  feature, digest §9).
- A package/module is itself an environment fragment; the kernel sees a single
  flattened, append-only `Σ` (`../10-kernel/11 §4`).

## 4. Visibility

- Top-level names are **module-private by default**; `pub` exports. (Or the
  inverse default — OQ-syntax.) Exported names form the module's interface.
- A type may be exported **abstractly** (name only, constructors hidden) — the
  surface form of an opaque constant / abstract interface (`../10-kernel/11
  §4`), giving information hiding without a new kernel feature.

## 5. Constraints — typeclasses as subobjects of the universe

Ken's constraint/trait mechanism is **structure on types**. A "class" is a
structure `C : Type → Type` (its members carve out a subobject of the universe,
`../10-kernel/12 §5`); an "instance" exhibits that a given type carries it. This
is the digest's "typeclasses-as-subobjects" idea — the most category-faithful
account — and is greenfield (the prototype has none).

Two kinds of class, split by where the dictionary lives — the distinction that
governs coherence (`OQ-classes`, decided):

- **Property classes** (Ω-valued: `Decidable p`, `IsHom f`). Proof-irrelevant
  (`../10-kernel/16 §1`), so **any two instances are definitionally equal** —
  the subobject framing is literal, and **coherence is free** (the kernel
  guarantees it; no resolver convention applies).
- **Structure classes** (`Type`-valued, dictionary with computational content:
  `DecEq`, `Monoid`, `Ord`). Genuinely *many* can exist on one carrier (ℤ under
  `+` and under `×`), so the subobject reading is "∃ a dictionary" and coherence
  is a **resolver convention** (below), not a theorem.

```
class DecEq (A : Type) {              -- a record of operations + their laws
  eq    : A → A → Bool                 -- (the propositional equality is the
  ok    : (x y : A) → eq x y == true → Eq A x y   --  kernel's Eq, 10-kernel/15)
}
instance DecEq Int { eq = int_eq, ok = … }

view nub {A : Type} (xs : List A) : List A  where DecEq A = …   -- a constraint
```

- A `class` elaborates to a **record type** (a Σ of operations *and* their law
  propositions); an `instance` is a value of that record — including **proofs of
  the laws** (`law`/`verify`, `../20-verification/21 §3`).
  Lawful-by-construction classes are a verification win: `Monoid A` carries
  proofs of associativity and unit, usable by the prover.
- A constraint `where C A` is an **implicit instance argument** the elaborator
  resolves by instance search (`39`) — exactly an implicit `Π` over the class
  record. Resolution is proof search for subobject membership. **Coherence
  policy** (`OQ-classes`, decided): for structure classes, **one canonical
  instance per (class, head-type)** participates in implicit search, so "the
  `Ord A`" is a *function of `A`* — stable program-wide, which the prover relies
  on (the dictionary carries law proofs it *uses*). **Orphan instances are a
  hard error**: an instance MUST be declared with its class or with its
  head-type, keeping canonicity decidable and per-module-checkable. **No
  overlapping instances** in search; **ambiguity is a compile error** naming
  both candidates, never a silent pick. Property classes need none of this —
  proof irrelevance makes any instance do.
- **Named instances are first-class values, passed explicitly.** Because an
  instance *is* a record value, you may define `byLength : Ord String` and pass
  it (`sortBy byLength xs`) — the dependent-types escape hatch Haskell lacks (no
  `newtype` gymnastics). *Implicit* search stays canonical and predictable;
  *explicit* passing is unrestricted. That split is the whole point: the
  resolver may pick only one, canonical thing silently; you may deliberately use
  any value.
- **Search terminates.** Instance search is bounded by a structural-decrease
  rule on the instance graph (the SCT family, `../10-kernel/17 §4`); no infinite
  instance chains.
- `derive (DecEq, Show)` requests an elaborator-generated instance for a `data`/
  `record` (structural); generation is untrusted (the kernel checks the result).

## 6. Fixity and operators

`infixl N op` / `infixr N op` / `infix N op` declare operator fixity (`32 §6`).
Operators are ordinary `view`s with symbolic names; there is nothing special
about them semantically.

## 7. What WS-L must deliver here

Definitions (incl. generic + mutually recursive under SCT), records (dependent +
refined fields, update/pun), the module/import system + package manager with
content-addressed lockfiles, visibility/abstraction, and the
class/instance/constraint mechanism with **lawful** instances and `derive`.
Conformance: `../../conformance/surface/declarations/`.
