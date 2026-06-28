# Core syntax

> Status: **DRAFT v0**. Normative. Defines the kernel's term language, contexts,
> the global environment, binding/de Bruijn representation, and substitution.
> The *typing* of these forms is in `13`–`16` and `18`; this chapter fixes only
> the grammar and its scoping.

The kernel operates on **core terms**: a small, fully-explicit language with no
implicit arguments, no notation, and no surface sugar. Elaboration
(`../30-surface/39-elaboration.md`) produces core terms; the kernel only ever
sees this language.

## 1. Syntactic categories

There are two mutually-referential categories:

- **Levels** `ℓ` — universe levels (`12-universes.md`).
- **Terms** `t, u, A, B` — the one category for both terms and types (types
  *are* terms of a universe; Ken is a pure type system in that sense).

```
ℓ ::= 0 | suc ℓ | max ℓ ℓ | lvar    -- universe levels (12)

t, u, A, B ::=
    Type ℓ                           -- universe (12)
  | Ω                                -- strict proposition universe (12, 16)
  | x                                -- variable (de Bruijn)
  | c                                -- global constant / definition
  | D | c_D                          -- inductive former / constructor (14)
  | (x : A) → B | λ (x : A). t | t u           -- functions Π (13)
  | (x : A) × B | (t , u) | t.1 | t.2          -- pairs Σ (13)
  | Eq A t u | refl t                  -- observational equality (15,16)
  | cast A B e t                       -- cast along Eq Type A B (16)
  | J M d e                            -- eq eliminator (derived) (15)
  | elim_D M [c_k ↦ t_k]ₖ s            -- inductive elim (14)
  | A / R | [t] | elim_/ M f s         -- quotient: type / class / elim (16)
  | ‖ A ‖ | |t|                        -- propositional truncation (16)
  | let x := t : A in u
  | (t : A)                          -- ascription (erased after check)
```

Notes:

- **Type ascription** `(t : A)` is a checking hint only; it carries no runtime
  or conversion content and is erased once `t` checks against `A`.
- **Primitive types and literals** (`Int`, `Bytes`, …) are *not* primitive term
  formers here. They enter the kernel as ordinary global declarations — opaque
  constants plus reduction rules registered with the environment
  (`14-inductive.md §5`, `../40-runtime/41-values.md`). This keeps the core
  grammar closed and small.
- The grammar is **intrinsically typed in spirit but extrinsically presented**:
  the same syntax can be ill-typed; typing (`18`) is what admits a term.

## 2. Binding and de Bruijn representation

Surface and spec examples use **named** binders for readability (`λ (x : A).
t`). The kernel representation is **de Bruijn indices**: a bound variable is a
natural number counting binders outward from its occurrence (`0` = nearest
enclosing binder). This makes α-equivalence syntactic identity and substitution
capture-free.

- The context (§3) is a single telescope of **term variables** `x : A` only —
  there is no interval or cofibration namespace (those were cubical; ADR 0005).
  The binders `λ (x : A). t`, `(x : A) → B`, `(x : A) × B`, and `let` each
  introduce one term-variable entry; indices count them.

- The kernel MUST provide capture-avoiding **substitution** and **weakening**
  (§5). Implementations MAY use any internal representation (indices, levels,
  locally-nameless) provided the observable judgments are identical; de Bruijn
  indices are the reference.

- **Definitional unfolding** of a global constant `c` (δ-reduction) replaces `c`
  by the body recorded in the environment (§4). Constants are *not* de Bruijn
  variables; they are names resolved in the global environment.

## 3. Contexts

A **context** `Γ` is an ordered telescope of entries:

```
Γ ::= ·                       -- empty
    | Γ, x : A                -- term variable of type A
```

- Entries bind to the right: later entries' types may mention earlier variables.
- A term-variable entry `x : A` requires `Γ ⊢ A : Type ℓ` (or `A : Ω`) for some
  `ℓ` (well-formedness, `18`).
- There are **no interval or cofibration entries** — the context is term
  variables only (ADR 0005; observational equality needs no dimension context).

Context well-formedness `⊢ Γ ctx` and the judgments over Γ are in `18`.

## 4. The global environment

Type-checking is relative to a **global environment** `Σ` (distinct from the
local context Γ) recording top-level declarations in dependency order:

```
Σ ::= ·
    | Σ, c : A := t          -- transparent definition (δ-unfoldable)
    | Σ, c : A               -- opaque constant / postulate (no unfolding)
    | Σ, data D …            -- inductive family declaration (14-inductive.md)
    | Σ, c : A := prim p     -- primitive: opaque const + reduction (41)
```

- A **transparent definition** `c : A := t` requires `· ⊢ t : A` in the
  environment so far, and `t` to pass the **termination check**
  (`17-conversion.md §SCT`) before admission. δ-reduction may unfold `c` to `t`.
- An **opaque constant** `c : A` (postulate) blocks δ; it is how axioms, FFI
  signatures (`../30-surface/38-ffi-io.md`), and abstract interfaces are
  represented. Postulating a closed term of `⊥` would break consistency, so
  introducing opaque constants of empty types is a soundness-relevant action the
  kernel records but does not itself forbid — policy is the elaborator's (`18
  §4`).
- **Inductive declarations** and **primitives** are detailed in `14` and `41`.
- The environment is **append-only and acyclic**: a declaration may reference
  only earlier ones. This is what makes δ-unfolding well-founded and is a
  precondition of the termination argument.

## 5. Substitution and weakening (reference operations)

The kernel MUST implement, and conversion/typing are defined in terms of:

- **Single substitution** `t[u/x]` — replace the variable bound by the nearest
  relevant binder with `u`, capture-avoiding. Used by β (`13`), ι (`14`), and
  the `Eq`/`cast` computations (`15`, `16`).
- **Weakening / context extension** — transport a term into a larger context.
- **Simultaneous substitution / explicit substitutions** MAY back the
  implementation for efficiency, but the observable result MUST equal the
  reference single substitution.

Substitution interacts with the environment only through variables; global
constants are invariant under substitution (they are closed in `Σ`).

## 6. Well-formedness vs typing

This chapter defines only **raw well-formedness** — that a term is built by the
grammar and that every variable index resolves to an in-scope term-variable
entry. It does **not** decide typing. A raw-well-formed term may still be
ill-typed (e.g. applying a non-function). The kernel's `check` and `infer`
(`18`) decide typing; raw well-formedness is their precondition and is what the
parser/elaborator must guarantee before calling the kernel.

## 7. Notation used in the rest of `10-kernel/`

- `Γ ⊢ t : A` — `t` has type `A` in context Γ (and ambient `Σ`).
- `Γ ⊢ a ≡ b : A` — `a` and `b` are definitionally equal at `A` (`17`).
- `Γ ⊢ A` abbreviates `Γ ⊢ A : Type ℓ` for some `ℓ`.
- `t ⇓ v` — `t` evaluates to (whnf or full) normal form `v` (`17`).
- `t[u/x]` — substitution (§5).
- Inference rules are written with premises over a line and the conclusion
  below; side conditions are noted to the right.
