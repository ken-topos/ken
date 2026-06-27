# Operational semantics (the reference interpreter)

> Status: **DRAFT v0**. Normative for *what evaluation computes* and its role as
> the reference; the interpreter's internal strategy is implementation latitude
> so long as it realizes these results. Contract for WS-X **X1**.

The interpreter defines **the meaning of a Ken program**. It evaluates **core
terms** (`../10-kernel/11`) — the elaborator's output — to **values** (`41`).
Everything downstream (a native backend, X3) is judged correct by agreement with
it (`../00-overview.md §3`).

## 1. Relationship to the kernel's reduction

Evaluation realizes the same reductions the kernel uses for conversion
(`../10-kernel/17 §1`): β, Σ-projection, ι (eliminators on constructors), δ
(definition unfolding), the observational computations (`Eq`-by-type, `cast`,
quotient elim), and **prim** (the audited primitive reductions, `../10-kernel/14
§5`). The kernel evaluates lazily to *weak-head* normal form for conversion
(NbE); the interpreter evaluates programs to **values** (full normal forms of
closed, ground computations). The two MUST agree on results — the interpreter is
"the kernel's evaluator, run to completion, with primitives and effects."

- **Determinism.** Evaluation of a closed, effect-free term is a function: same
  term → same value. This is what makes the interpreter a usable oracle.
- **Canonicity.** A closed term of an inductive type evaluates to a constructor
  form; `Eq`/`cast` on closed terms compute (`../10-kernel/16 §9`). No closed
  program "gets stuck" on a well-typed ground computation.

## 2. Evaluation order (`OQ-eval-order` DECIDED)

**Totality makes evaluation order meaning-preserving.** Because every Ken
program terminates (SCT, `../10-kernel/17 §4`), there is no ⊥/divergence to
distinguish strict from lazy — both compute the *same value*. So evaluation
order is a purely **operational** choice (space, time, legibility), **not**
semantics — unlike a partial language, where laziness is observable. That frees
the choice to favor predictability.

**Decision (operator, 2026-06-27): call-by-value (strict) with sharing, strict
by default, lazy only where required or annotated.**

- **Strict CBV is the default** — arguments and `let` bindings are evaluated
  eagerly, left-to-right; results are shared via the content-addressed heap
  (`41`), so equal subcomputations are deduplicated (CBV's predictability + the
  store's space efficiency, no recomputation). Chosen because, the choice being
  meaning-preserving, strict is the most **legible and predictable**: a cost
  model you can reason about, an order that matches how the code reads, and no
  thunk/space-leak footguns. **Predictability is also a precondition for the
  time/space reasoning security depends on** — the `@ct` timing discipline
  (`../60-security/61 §5a`) and worst-case bounds need a non-data-dependent
  "when"; lazy-by-default would undermine them.
- **Laziness where semantically required** — `if`/`match` evaluate only the
  taken arm; `&&`/`||` short-circuit. (CBV evaluates the scrutinee, then only
  the taken branch.) The coinductive fragment, if added (`OQ-coinduction`,
  deferred), brings its own *local* lazy/guarded evaluation — opt-in for that
  type class, no conflict with the strict inductive core.
- **Laziness by explicit annotation** — an opt-in **`Lazy a`** (thunk) type
  defers an expensive, possibly-unused computation, **forced** on demand and
  memoized (call-by-need *locally*). Laziness is thus **visible in the type**,
  never a pervasive implicit default — strict-by-default, lazy-by-annotation.
- **Distinct from the kernel's conversion (`OQ-eval-strategy`).** The kernel
  decides definitional equality by **lazy WHNF** (`§1`, `../10-kernel/17`); the
  *runtime* executes **CBV-with-sharing**. Different layers, allowed to differ —
  as Lean's kernel reduces lazily for defeq while compiled code runs strictly.
  The two need only agree on final values (`§1`).

## 3. Effects

Effectful operations (`../30-surface/36`) have their operational meaning here:

- A primitive effect (`FS`, `Net`, `Clock`, `Console`, `Rand`, …) performs its
  world-interaction when evaluated, in the order the effect discipline imposes.
  The effect row in a function's type bounds *which* effects can occur (`36
  §1`).
- A `space` (`36 §4`) holds mutable cells; `becomes` updates a cell; reads
  observe the current cell value. Cell state is **not** content-addressed (it
  has identity); the ordering of cell operations follows the effect sequencing.
- Pure evaluation is **referentially transparent** and reproducible; effectful
  evaluation is reproducible only relative to the world. The verification layer
  reasons over the pure fragment (`36 §1`), which is exactly the part with a
  mathematical semantics.

## 4. `unknown` at runtime

Evaluating a term that depends on an **open verification hole** (`41 §6`,
`../20-verification/24 §2`) yields `unknown`, propagated by the Kleene/Heyting
rules (`41 §6`). This is the operational face of partial verification: the
program runs, and `unknown` marks exactly where an unproven property bears on a
result. A hole-free program never yields `unknown`.

## 5. The interpreter as oracle (and the REPL)

- **Differential oracle.** The native backend (X3) is validated by running a
  differential corpus through both and requiring identical values (`44`/X4). The
  interpreter is the reference; on disagreement, the interpreter is right by
  definition.
- **REPL.** The interpreter makes an interactive **REPL** natural (strategy T2):
  evaluate expressions, run `prove`/`assume` (`../20-verification/21 §3`),
  inspect values — the "Little Prover" loop. Incremental re-checking is the only
  non- trivial REPL piece (`../30-surface/39`).

## 6. What WS-X must deliver here (X1)

A reference interpreter that evaluates core terms to values realizing the
kernel's reductions + primitives + effects, deterministically and with
canonicity for closed ground programs; `unknown` propagation; sharing via the
content-addressed heap; and the oracle role for later backends. It runs the
**G1** vertical slice end-to-end. Conformance:
`../../conformance/runtime/evaluation/` — canonicity of closed
inductive/observational computations, determinism, short-circuit/branch
laziness, and `unknown` propagation.
