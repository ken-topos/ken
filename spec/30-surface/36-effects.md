# Effects, capabilities, and state

> Status: **DRAFT v0**. Proposal-level for syntax; normative for the *model*.
> Effect tracking, capabilities, and the state/`space` escape hatch. Ken adopts
> the prototype's **proven** shape — statically-checked, transitively-inferred
> effects (`visits`) — over the analysis's hypothetical Kleisli scheme (digest
> §7, OQ-8).

## 1. Effects as a static row

A `view` is **pure** by default. A function that performs an effect declares an
**effect row**:

```
view read_config (path : String) : Config  visits [FS] = …
view now () : Instant                       visits [Clock] = …
view greet (name : String) : Unit           visits [Console] = …
```

- An **effect** (`FS`, `Clock`, `Console`, `Net`, `Rand`, …) is a named
  capability a computation may use. The row `visits [E₁, …]` is part of the
  function's type.
- **Statically checked + transitively inferred:** calling an effectful function
  *requires* its effects to be in the caller's row; the checker **infers** a
  function's effects from its body (transitive closure of what it calls), and
  reports a mismatch where a declared row omits an effect actually used. So
  effects cannot be silently performed — a pure-typed function is pure (the
  verification layer relies on this: a `view` with no row is a mathematical
  function and its `ensures` are about values, not world-state).
- **Purity is the default and the common case;** only boundary functions carry
  rows, which keeps the verification core (`../20-verification/`) reasoning over
  pure terms.

## 2. Effects in the type / kernel view

An effect row elaborates to an **indexing of the result** by the capabilities
required — the simplest sound encoding is a parametrized monad `Eff [E] A`
(`../50-stdlib/`) or an explicit capability-passing translation
(`../10-kernel/`-level it is ordinary Π over capability tokens). Either way the
kernel sees a pure dependent term; effects are a surface discipline, not a new
kernel primitive (OQ-8 fixes the encoding). The reference semantics
(`../40-runtime/42-evaluation.md`) gives each primitive effect its operational
meaning.

## 3. Capabilities (`requires`-as-capability)

Distinct from logical preconditions (`../20-verification/21 §1`), a
**capability** is an authority token a computation must be *given* to act (open
a file, hit the network). The prototype conflates two readings of `requires`;
Ken separates them:

- **Logical `requires φ`** — a proposition, discharged by proof (`../20-`).
- **Capability `using c : Cap`** — a value-level authority, passed explicitly or
  via the effect row, enabling the corresponding effect. Capabilities make the
  *principle of least authority* expressible: a function gets exactly the
  capabilities it needs, visible in its type.

Whether capabilities are a separate construct or just specific effects in the
row is **OQ-8a**; the requirement is that authority is **static and visible**,
not the prototype's runtime-only FNV-gate.

**Security extension (tier-1, `../60-security/`).** The effect/capability
discipline is the host for two security mechanisms (ADR 0004):

- **Information-flow labels.** Effect channels (`Net`, `FS`, a log, a `space`
  cell) carry a **clearance label**, and data carries a security label; writing
  data `@ ℓ` to a sink of clearance `κ` type-checks only when `ℓ ⊑ κ`. The same
  indexed-effect machinery that indexes capabilities here indexes **labels** —
  this is how Ken gets **intrinsic information-flow control** without a new
  kernel primitive (`../60-security/61-information-flow.md`).
- **Attenuation + revocation.** Capabilities are **attenuable** (derive a
  strictly weaker token for a child) and **revocable** at a boundary, with use
  audited — the principle-of-least-authority story
  (`../60-security/62-authority.md`).

So a function's effect-and-capability type is simultaneously its **capability
manifest** and its **flow manifest**. Details, the label lattice, and
declassification are in `../60-security/`.

## 4. State — the `space` model

Pure code cannot mutate. Genuine mutable state and process isolation live in a
**`space`** — Ken's analog of the prototype's `Space` (kept as the *concept*,
not the implementation; the digest notes the prototype's `Space`/`spawn` is
`fork()` + POSIX shared memory, logical isolation only):

```
space Counter {
  mut n : Int = 0
  view inc () : Unit  visits [Counter] = n becomes n + 1
  view get () : Int   visits [Counter] = n
}
```

- A `space` encapsulates **cells** (`mut`) with identity; operations on it carry
  the space as an effect. Mutation is `becomes` (cell update). Reads/writes are
  ordered by the effect discipline.
- A `space` is the **only** place identity-bearing mutable state exists; pure
  values are immutable and content-addressed (`../40-runtime/41-values.md`).
- **Concurrency / isolation model is OQ-Space.** Whether spaces map to OS
  processes, threads, or logical regions; the message/transport model; and the
  isolation guarantee are deliberate design choices, not inherited (the
  prototype's actual model — `fork` + shared memory — is *not* a spec
  commitment). The DRAFT fixes only that mutable state is *encapsulated,
  effect-tracked, and identified*, leaving the runtime model to `../40-runtime/`
  + OQ-Space.

## 5. Algebraic effects / handlers (research)

Reified, multi-shot continuations and general algebraic-effect handlers (the
analysis's `shift`/`reset`, `with multishot`) are **research track** (`02 §7`,
digest §7/§10). The prototype "parses but ignores" multishot; Ken does **not**
promise them. The DRAFT effect model is the simpler static-row + handler-as-
tail-resumptive form; richer handlers are a possible future extension (OQ-9).

## 6. What WS-L must deliver here (L5)

The static effect row (`visits`) with transitive inference + checking; the
pure-by-default discipline; a capability/authority story (static, visible); and
the `space` state model (cells, `becomes`, effect-tracked) with the
concurrency/isolation model deferred to OQ-Space. Acceptance: a pure-typed
`view` is provably effect-free (the verification layer may treat it as a
function); performing an undeclared effect is a compile error. Conformance:
`../../conformance/surface/effects/`.
