# Effects, capabilities, and state

> Status: **DRAFT v0**. Proposal-level for syntax; normative for the *model*.
> Effect tracking, capabilities, and the state/`space` escape hatch. **`OQ-8` /
> `OQ-8a` DECIDED** (operator, 2026-06-27): static, transitively-inferred effect
> **rows** (`visits`), pure by default; a **layered encoding**
> (authority/denotation/spec, §2) that keeps the kernel pure; capabilities as
> static value tokens (§3). The *stateful-effect verification methodology* is
> handed to `OQ-Space` (§4).

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

## 2. The encoding — three layers, one pure kernel (`OQ-8` DECIDED)

An effect row is **not** a kernel primitive. The surface `Eff [E] A` monad
(`../50-stdlib/`) elaborates into a **pure** dependent term through three
layers, each answering a different question — and the *same denotation* powers
verification, capabilities, information flow, and Ward's behavioral export:

1. **Authority — "who may perform this effect?"** A **capability-passing**
   translation: performing an effect requires a **capability token** (a value)
   in scope; at the `../10-kernel/` level this is ordinary Π over capability
   tokens (§3). Static and visible; no runtime gate.
2. **Denotation — "what does this computation *do*?"** The effectful computation
   denotes to an **interaction tree** (a free-monad-style *pure data structure*:
   `Ret a` | `perform e then continue with the response`). `Eff`'s bind is tree
   grafting. The kernel sees only this inductive datatype — it stays pure. One
   choice serves four masters: **handlers are folds** over the tree (§5);
   **Ward's event alphabet is the tree's `perform` nodes** (`../70-behavioral/
   §3`); **information-flow labels are labels on those nodes** (§3,
   `../60-security/61`); and **verification is predicates over the tree**.
3. **Specification — "what must it guarantee?"** `requires`/`ensures` on an
   effectful function are **WP/Hoare-style predicates over the denotation**. For
   *stateful* effects the pre/post relation is the genuinely hard part and is
   handed to **`OQ-Space`** (§4).

So effects are a **surface + elaborator + runtime** discipline; the kernel
reasons about a pure denotation and the runtime
(`../40-runtime/42-evaluation.md`) executes the real effects via the boundary.
The trusted base gains nothing — the same small-TCB invariant that governs the
rest of the kernel (ADR 0001/0004/0005). *(Precedents, one per layer: Koka rows
· Interaction Trees · F\* Dijkstra monads.)*

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

**`OQ-8a` DECIDED (operator, 2026-06-27): capabilities are first-class value
tokens, not a separate effect kind and not a runtime gate.** A capability is a
*value* (`c : Cap`) threaded explicitly or supplied by an enclosing handler (a
handler is a capability provider, §5); authority is **static and visible** in
the type, **attenuable** and **revocable** with use audited
(`../60-security/62`). It is kept distinct from the logical `requires φ` (a
proposition); conflating the two was the prototype's mistake.

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
- **Constant-time (`@ct`) — leakage-relevant operations as an effect sink.** A
  distinct, **opt-in** timing-sensitive label `@ct` (separate from `Secret`
  confidentiality) marks data whose *influence* must not reach a **leakage-
  relevant operation** — a secret-dependent **branch guard**, **memory index**,
  or **variable-time primitive**. Those operations are a distinguished **effect
  sink**, and the rule is the IFC rule reused: a `@ct` value reaching such a
  sink is a **type error** (you cannot leak by accident; no per-operation
  annotation). This **unary taint discipline soundly enforces the source-level
  constant-time (2-safety) property** — no relational/product-program machinery.
  The sensitive *range* is the `@ct` label's live span (intro → `declassify`),
  so there is **no `constant_time { … }` region**; a function carries a
  **signature-level CT promise** (constant-time in a parameter) for boundary
  checking and export. The *timing guarantee itself* is
  codegen/hardware-relative and **delegated to `Ward`** under a stated leakage
  model (`../60-security/61 §5a`, `64 §4.2`, `63 §5a`); a **policy** may require
  `@ct` for a data class (`../60-security/65`).

So a function's effect-and-capability type is simultaneously its **capability
manifest** and its **flow manifest**. Details, the label lattice,
declassification, and the constant-time discipline are in `../60-security/`.

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
  ordered by the effect discipline. Semantically, `becomes` denotes to a
  **state-passing fold** of the interaction tree (§2.2) — imperative surface,
  functional denotation.
- A `space` is the **only** place identity-bearing mutable state exists; pure
  values are immutable and content-addressed (`../40-runtime/41-values.md`).

**State verification — bounded Hoare, no separation logic (`OQ-Space`
DECIDED).** Because each `space`'s cells are **encapsulated and non-aliased**
(state is partitioned per space, with no shared mutable heap across spaces),
reasoning about a space operation is **local, bounded Hoare** over its own cells
— Ken needs **no separation logic / frame rules** (the machinery a big aliasable
heap would force). `requires`/`ensures` on a space operation relate pre- and
post-state; **`old(e)` is admitted, scoped to the operation's `ensures`** (a
cell's pre-call value, well-defined because the denotation is state-passing) —
*not* a global `\old` (`../20-verification/21 §4`). Ken proves **local,
sequential, per-space** correctness; **concurrent/distributed/temporal**
correctness is delegated to Ward (below, `../70-behavioral/`).

**Concurrency & isolation — shared-nothing message-passing (`OQ-Space`
DECIDED).** For *in-Ken* communication, spaces are **shared-nothing**: they
share no mutable memory and communicate only by **passing immutable,
content-addressed values** (actor-style). Isolation is therefore a
**guarantee**, not a discipline — no shared mutable state ⇒ no data races — on
which capability revocation and confinement rest (`../60-security/62 §4`, ADR
0004). This pairs with the rest of the model: a space handle is a **capability**
(§3), send/receive are **effects** (§2), messages carry **IFC labels** (§3), and
message events are **Ward's behavioral alphabet** (`../70-behavioral/`). The
**runtime realization** — process, thread, green-thread, or distributed — is
deferred to `../40-runtime/` (the *model* is shared-nothing; the *mapping* is an
implementation choice, distribution-ready). *(FFI is the exception: a `foreign`
boundary may use shared memory, but it is already an explicitly unsafe/untrusted
boundary, `38-ffi-io.md §3`, so it does not weaken the in-Ken isolation
property.)*

## 5. Handlers — tail-resumptive only (`OQ-8`; multishot → `OQ-9`)

A **handler** interprets an effect — operationally, a **fold over the
interaction tree** (§2.2). Handlers are how user-defined effects are given
meaning and how capabilities are provided (a handler is a capability provider,
§3).

Ken's core admits **tail-resumptive** handlers only: the continuation is invoked
**at most once, in tail position** — which keeps the fold well-founded,
preserves **totality** (the SCT/termination story, `../10-kernel/17 §4`), and
keeps effectful code tractable to verify. **Reified, multi-shot continuations**
(the analysis's `shift`/`reset`, `with multishot`) are **not** promised — they
break totality and are hard to verify — and stay **research track** (`OQ-9`, `02
§7`; the prototype "parses but ignores" them). Genuinely reactive/nonterminating
interaction (coinductive trees) is Ward's domain (`../70-behavioral/`) and
touches `OQ-coinduction`.

## 6. What WS-L must deliver here (L5)

The static effect row (`visits`) with transitive inference + checking; the
pure-by-default discipline; a capability/authority story (static, visible); and
the `space` state model (cells, `becomes`, effect-tracked) with the
concurrency/isolation model deferred to OQ-Space. Acceptance: a pure-typed
`view` is provably effect-free (the verification layer may treat it as a
function); performing an undeclared effect is a compile error. Conformance:
`../../conformance/surface/effects/`.
