# Specification syntax

> Status: **DRAFT v0**. Normative for the forms and their meaning; concrete
> surface spelling cross-refs `../30-surface/`. Contract for WS-V **V1**. How a
> programmer or agent attaches a *correctness specification* to code — the
> surface the whole verification loop hangs off. This is the L1→L2 bridge the
> prototype lacked at the surface (digest §1, §8).

A **specification** is one or more **propositions** (`../10-kernel/12 §5`,
elements of Ω) attached to a definition, asserting how it must behave. Ken
offers three layered ways to state one, from lightest to most expressive (§1–3),
discharged through one smooth **gradient** — declarative contract → automatic
proof → typed hole → tactic/term (`23`), never a cliff between "automatic" and
"hand-written."

**The defining discipline (`OQ-spec`, DECIDED).** Because Ken is a *software
engineering* language — written by agents, read by humans — every claim carries
a **visible, exportable epistemic status**: **proved** (kernel-discharged),
**tested** (assumed, with a runtime/test obligation), **delegated** (a
temporal/behavioral property Ken can *state* but not close as a static
proposition — exported downstream), or **unknown** (an open typed hole). The
source distinguishes proof from test from model from gap *on its face* (§5); the
delegated/tested set is emitted as the **assumption boundary** consumed by the
behavioral sibling (`../70-behavioral/`, ADR 0006). This is the surface form of
"prove what can be proven and state what must be tested."

## 1. Function contracts — `requires` / `ensures`

The everyday form: pre- and post-conditions on a function.

```
view divide (n : Int) (d : Int) : Int
  requires  d ≠ 0
  ensures   result * d + (n % d) == n
= n / d
```

- **`requires φ`** — a **precondition**: a proposition `φ : Ω` over the
  function's parameters. Callers must establish it; inside the body it may be
  assumed.
- **`ensures ψ`** — a **postcondition**: a proposition `ψ : Ω` over the
  parameters *and* the special binder **`result`** (the return value). The body
  must establish it.
- Multiple `requires`/`ensures` clauses conjoin. Clauses may mention earlier
  parameters (they are checked in the telescope `../10-kernel/13 §3`).
- Contracts are **erasable**: they generate obligations (`22`) and assumptions
  but no runtime code by default (runtime-checked contracts are an opt-in, §5).

Semantically, a contract elaborates to a **refined function type**
(`../30-surface/39-elaboration.md`): `divide` above has the dependent type

```
(n : Int) (d : Int) → { d ≠ 0 } → (r : Int) × (r * d + (n % d) == n)
```

i.e. preconditions become extra (proof) arguments and postconditions become a Σ
pairing the result with a proof. The verification layer hides this encoding; the
programmer writes `requires`/`ensures`.

## 2. Refinement types — `{ x : A | φ x }`

A **refinement type** is the comprehension subobject (`../10-kernel/12 §5`,
`../30-surface/34-data-match.md`): the type of `x : A` *for which `φ x` holds*.

```
type Pos = { n : Int | n > 0 }
view head (xs : { l : List A | l ≠ nil }) : A = …      -- non-empty by type
```

- `{ x : A | φ x }` requires `φ x : Ω`. Its inhabitants are (by the
  comprehension/Σ reading) pairs `(x, proof-of-φx)`, but the proof component is
  a **mere proposition** (`12 §5.1`), so refinements carry *no runtime payload*
  and coerce silently to `A`.
- Refinements are the **route to L2 at the surface** the digest highlights:
  pushing a property into a type makes the checker enforce it at every use, with
  obligations generated where a plain `A` is used as `{x:A|φ}` (`22`).
- They compose with Π/Σ: function arguments, results, and record fields may all
  be refined; `requires`/`ensures` (§1) is sugar for refined argument/result
  types.

## 3. Propositional goals — `prove` / `law`

A **goal** is a standalone proposition to be discharged — a lemma, an invariant,
or an algebraic law — not attached to a single function's body.

```
prove  add_comm : (a b : Int) → a + b == b + a
law    Monoid (M) { assoc : … ; unit_l : … ; unit_r : … }   -- a property bundle
```

- **`prove name : φ`** registers `φ : Ω` as an obligation; on success `name`
  becomes a usable proof term of `φ` in the environment (`../10-kernel/11 §4`).
- **`law`** bundles related propositions (the algebraic-law form the digest
  notes the prototype supports via `law`/`verify`); proving a `law` for a type
  makes the bundle available as a record of proofs, usable by constraint
  resolution (typeclasses-as-subobjects, `../30-surface/33-declarations.md`).
- Goals are where Ken is used as a *proof assistant*, and where the REPL's
  "Little Prover" loop (`../30-surface/`, strategy T2) lives.

## 4. What the binders mean (precise)

| Binder / form | Scope | Type |
|---|---|---|
| parameters `x : A` | the whole contract + body | as declared |
| `result` | `ensures` clauses only | the function's return type |
| `old(e)` *(deferred → OQ-Space)* | `ensures`, for mutating ops | value of `e` in the pre-state |
| `φ` in `requires`/`ensures`/`{·|φ}`/`prove` | as above | **must be `: Ω`** |

- Every specification proposition MUST type-check at `Ω` (`12 §5`) in its scope;
  a `requires`/`ensures` whose body is not a proposition is a surface type
  error, not a verification failure.
- **`old(e)`** (referring to a pre-state value in a postcondition) is only
  meaningful for effectful/mutating operations (`../30-surface/36-effects.md`);
  for pure `view`s the pre/post states coincide. The proof interface is decided
  (`OQ-spec`), but **`old` and the state model are deferred to `OQ-Space`**: the
  DRAFT leans **explicit state** (name the pre/post state as values; no implicit
  heap, no `old`), adding `old`-style sugar only if a settled `space` model
  threads state *implicitly* — and never the framing/separation machinery unless
  forced (`90-open-decisions.md`).

## 5. Epistemic status — proved / tested / delegated / unknown

Every specification claim has one of four **statuses**, each visible in the
source and (for the latter three) carried in the **assumption-boundary export**
(`../70-behavioral/`). This four-way distinction is the heart of `OQ-spec` and
the feature that makes Ken a *software engineering* language rather than a
programming language: a reader sees, per claim, whether it is *proved*, merely
*tested*, *delegated* to behavioral checking, or still *open*.

- **`proved`** — the obligation (`22`) was discharged and the kernel re-checked
  the certificate (`23`, `../10-kernel/18 §4`). The default for a contract that
  goes through. No annotation; it simply holds.
- **`tested`** — a property that **cannot (yet) be proven** but is **asserted
  with a runtime/test obligation**: a first-class `assume`/`test`-tagged clause
  that lowers `requires`/`ensures` to a runtime assertion (boundaries, FFI,
  untrusted input) *and* registers a test/generator obligation. It is
  **visible** — a reader knows this guarantee rests on tests, not proof — and it
  is **exported** as part of the assumption boundary (the refinement predicate
  becomes a generator/oracle spec, `../70-behavioral/`, §2/Layer 2).
- **`delegated`** — a **temporal/behavioral** property (liveness, fairness,
  ordering, eventual consistency, an interleaving safety property) that is **not
  a static proposition over a pure function** and so cannot be closed in the
  kernel. Ken can *state* it — as ordinary **deeply-embedded temporal-logic
  data** (an inductive `Temporal`/μ-calculus value, *not* a kernel modality, so
  the TCB is untouched, `../70-behavioral/`) — and **exports** it to the
  behavioral sibling for model-checking and runtime monitoring. Stated here,
  discharged there.
- **`unknown`** — the obligation is *not* discharged and no test/delegation is
  given: the definition is admitted with a **typed hole** and the program
  **still runs**, the result carrying `unknown` where the unproven property is
  observed (`24-diagnostics.md §holes`). Verification is *incremental*, not
  all-or-nothing (Hazel-style); a hole is the honest "not done yet."

By default `proved` specs are static-only (erased); `tested` adds runtime code
by construction; `delegated` adds none to Ken (it is exported); `unknown` adds
none. The **assumption boundary** (`../70-behavioral/`) is precisely the
`tested` + `delegated` + open-`assume` set — what Ken could not guarantee
statically, handed to the sibling as the exact specification of what to model,
test, and monitor.

## 6. Interaction & elaboration

- Specs elaborate to refined types + obligations (`22`,
  `../30-surface/39-elaboration.md`); the kernel sees only the core encoding (Σ,
  Π, Ω, `Eq`).
- Refinement subtyping (`{x:A|φ} ≤ A` always; `A ≤ {x:A|φ}` generates an
  obligation `φ`) is the elaborator's, checked by emitting obligations — the
  kernel has no subtyping for refinements beyond the Σ encoding.
- A spec proposition may itself use earlier `prove`d lemmas, `law`s, and
  refinements — specs compose.

## 7. What WS-V must deliver here (V1)

The spec syntax (`requires`/`ensures`, `{x:A|φ}`, `prove`/`law`), its
type-checking at Ω, its elaboration to refined types + obligations, the `result`
binder, and the static/runtime/partial modes (§5). Acceptance ties to **G2**: a
real function with an `ensures` whose correct proof is accepted and whose wrong
proof is rejected. Conformance: `../../conformance/verify/spec-syntax/`.
