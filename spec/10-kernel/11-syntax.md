# Core syntax

> Status: **DRAFT v0**. Normative. Defines the kernel's term language, contexts,
> the global environment, binding/de Bruijn representation, and substitution. The
> *typing* of these forms is in `13`–`16` and `18`; this chapter fixes only
> the grammar and its scoping.

The kernel operates on **core terms**: a small, fully-explicit language with no
implicit arguments, no notation, and no surface sugar. Elaboration
(`../30-surface/39-elaboration.md`) produces core terms; the kernel only ever
sees this language.

## 1. Syntactic categories

There are four mutually-referential categories:

- **Levels** `ℓ` — universe levels (`12-universes.md`).
- **Terms** `t, u, A, B` — the one category for both terms and types (types
  *are* terms of a universe; Ken is a pure type system in that sense).
- **Interval terms** `r, s` — elements of the interval `𝕀` (`16-cubical.md`).
- **Cofibrations** `φ, ψ` — face formulas constraining the interval
  (`16-cubical.md`).

```
ℓ  ::= 0 | suc ℓ | max ℓ ℓ | lvar                    -- universe levels (12)

t, u, A, B ::=
    Type ℓ                                            -- universe (12)
  | x                                                 -- variable (de Bruijn)
  | c                                                 -- global constant / definition
  | D | c_D                                           -- inductive type former / constructor (14)
  -- functions (Π) ---------------------------------------------------- (13)
  | (x : A) → B          | λ (x : A). t     | t u
  -- pairs (Σ) ------------------------------------------------------- (13)
  | (x : A) × B          | (t , u)          | t.1 | t.2
  -- identity / paths ------------------------------------------------ (15,16)
  | Path A t u           | ⟨i⟩ t            | t @ r        -- path type, abstr, app
  | refl t
  -- inductive elimination ------------------------------------------- (14)
  | elim_D M [c_k ↦ t_k]ₖ s            -- dependent eliminator (motive M, methods, scrutinee s)
  -- cubical operations ---------------------------------------------- (16)
  | transp (⟨i⟩ A) r t
  | hcomp {φ ↦ ⟨i⟩ u} t                | comp (⟨i⟩ A) {φ ↦ ⟨i⟩ u} t
  | Glue A {φ ↦ (T , e)}              | glue {φ ↦ t} u   | unglue t
  -- definitions / ascription ---------------------------------------- 
  | let x := t : A in u
  | (t : A)                            -- type ascription (erased after checking)

r, s ::= 0 | 1 | i | r ∧ s | r ∨ s | ~ r       -- interval, de Morgan algebra (16)
φ, ψ ::= (r = 0) | (r = 1) | φ ∧ ψ | φ ∨ ψ | ⊤ | ⊥    -- cofibrations (16)
```

Notes:

- **Type ascription** `(t : A)` is a checking hint only; it carries no runtime
  or conversion content and is erased once `t` checks against `A`.
- **Primitive types and literals** (`Int`, `Bytes`, …) are *not* primitive term
  formers here. They enter the kernel as ordinary global declarations — opaque
  constants plus reduction rules registered with the environment
  (`14-inductive.md §Primitives`, `../40-runtime/41-values.md`). This keeps the
  core grammar closed and small.
- The grammar is **intrinsically typed in spirit but extrinsically presented**:
  the same syntax can be ill-typed; typing (`18`) is what admits a term.

## 2. Binding and de Bruijn representation

Surface and spec examples use **named** binders for readability (`λ (x : A).
t`). The kernel representation is **de Bruijn indices**: a bound variable is a
natural number counting binders outward from its occurrence (`0` = nearest
enclosing binder). This makes α-equivalence syntactic identity and substitution
capture-free.

- **Two binder namespaces share one index space** is *not* used; instead the
  context (§3) is a single telescope whose entries are tagged with their kind
  (term variable, interval variable, or cofibration), and indices count entries
  of the *term* and *interval* kinds. A binder form and the context entry it
  introduces:

  | Binder form | introduces context entry |
  |---|---|
  | `λ (x : A). t`, `(x : A) → B`, `(x : A) × B`, `let` | term var `x : A` |
  | `⟨i⟩ t`, `transp (⟨i⟩ A) …`, `⟨i⟩ u` in `hcomp`/`comp` | interval var `i : 𝕀` |
  | system branch `{φ ↦ …}` | cofibration assumption `φ` |

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
    | Γ, i : 𝕀                -- interval variable
    | Γ, φ                    -- cofibration assumption (a face restriction)
```

- Entries bind to the right: later entries' types may mention earlier variables.
- A term-variable entry `x : A` requires `Γ ⊢ A : Type ℓ` for some `ℓ`
  (well-formedness, `18`).
- An interval entry `i : 𝕀` adds a dimension; `𝕀` is **not** a type in any `Type
  ℓ` (you cannot abstract over `𝕀` with `Π`, only with path/interval binders),
  which keeps the interval from leaking into the ordinary type hierarchy
  (`16-cubical.md §Interval-is-not-a-type`).
- A cofibration entry `φ` restricts the remaining context to the faces where `φ`
  holds; under `Γ, φ` more definitional equalities may hold (`16-cubical.md
  §Partial-elements`).

Context well-formedness `⊢ Γ ctx` and the judgments over Γ are in `18`.

## 4. The global environment

Type-checking is relative to a **global environment** `Σ` (distinct from the
local context Γ) recording top-level declarations in dependency order:

```
Σ ::= ·
    | Σ, c : A := t          -- transparent definition (δ-unfoldable)
    | Σ, c : A               -- opaque constant / postulate (no unfolding)
    | Σ, data D …            -- inductive family declaration (14-inductive.md)
    | Σ, c : A := prim p      -- primitive: opaque constant + registered reduction p (41)
```

- A **transparent definition** `c : A := t` requires `· ⊢ t : A` in the
  environment so far, and `t` to pass the **termination check**
  (`17-conversion.md §SCT`) before admission. δ-reduction may unfold `c` to `t`.
- An **opaque constant** `c : A` (postulate) blocks δ; it is how axioms, FFI
  signatures (`../30-surface/38-ffi-io.md`), and abstract interfaces are
  represented. Postulating a closed term of `⊥` would break consistency, so
  introducing opaque constants of empty types is a soundness-relevant action the
  kernel records but does not itself forbid — policy is the elaborator's (`18
  §Postulates`).
- **Inductive declarations** and **primitives** are detailed in `14` and `41`.
- The environment is **append-only and acyclic**: a declaration may reference
  only earlier ones. This is what makes δ-unfolding well-founded and is a
  precondition of the termination argument.

## 5. Substitution and weakening (reference operations)

The kernel MUST implement, and conversion/typing are defined in terms of:

- **Single substitution** `t[u/x]` — replace the variable bound by the nearest
  relevant binder with `u`, capture-avoiding. Used by β (`13`), ι (`14`), and
  path application (`15`).
- **Interval substitution** `t[r/i]` — replace an interval variable by an
  interval term `r ∈ {0,1,i,∧,∨,~}`. Used by path application and the cubical
  operations (`16`). Interval substitution can change which cofibrations hold
  and therefore which definitional equations fire (`16 §Boundary`).
- **Weakening / context extension** — transport a term into a larger context.
- **Simultaneous substitution / explicit substitutions** MAY back the
  implementation for efficiency, but the observable result MUST equal the
  reference single/interval substitutions.

Substitution interacts with the environment only through variables; global
constants are invariant under substitution (they are closed in `Σ`).

## 6. Well-formedness vs typing

This chapter defines only **raw well-formedness** — that a term is built by the
grammar and that every variable index resolves to an in-scope entry of the right
*kind* (term vs interval). It does **not** decide typing. A raw-well-formed term
may still be ill-typed (e.g. applying a non-function). The kernel's `check` and
`infer` (`18`) decide typing; raw well-formedness is their precondition and is
what the parser/elaborator must guarantee before calling the kernel.

## 7. Notation used in the rest of `10-kernel/`

- `Γ ⊢ t : A` — `t` has type `A` in context Γ (and ambient `Σ`).
- `Γ ⊢ a ≡ b : A` — `a` and `b` are definitionally equal at `A` (`17`).
- `Γ ⊢ A` abbreviates `Γ ⊢ A : Type ℓ` for some `ℓ`.
- `t ⇓ v` — `t` evaluates to (whnf or full) normal form `v` (`17`).
- `t[u/x]`, `t[r/i]` — substitution (§5).
- Inference rules are written with premises over a line and the conclusion
  below; side conditions are noted to the right.
