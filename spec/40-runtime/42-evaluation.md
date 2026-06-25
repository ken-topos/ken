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
(definition unfolding), `Path`-β, the cubical computations, and **prim** (the
audited primitive reductions, `../10-kernel/14 §5`). The kernel evaluates lazily
to *weak-head* normal form for conversion (NbE); the interpreter evaluates
programs to **values** (full normal forms of closed, ground computations). The
two MUST agree on results — the interpreter is "the kernel's evaluator, run to
completion, with primitives and effects."

- **Determinism.** Evaluation of a closed, effect-free term is a function: same
  term → same value. This is what makes the interpreter a usable oracle.
- **Canonicity.** A closed term of an inductive type evaluates to a constructor
  form; cubical operations on closed terms compute (`../10-kernel/16 §11`). No
  closed program "gets stuck" on a well-typed ground computation.

## 2. Evaluation order

- The reference order is **call-by-value with sharing** for the pure fragment
  (arguments evaluated once, results shared via the content-addressed heap,
  `41`), except where laziness is semantically required (e.g. `if`/`match`
  evaluate only the taken branch; short-circuit `&&`/`||`). Strictness vs.
  laziness for `let` and data fields is **OQ-eval-order** — the *observable
  values* are fixed; the *evaluation strategy* (and thus space/time, not
  meaning) is the choice.
- **Sharing via interning** means a value computed twice is stored once;
  repeated subcomputations over equal data are deduplicated (`41 §4`).

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
`../../conformance/runtime/evaluation/` — canonicity of closed inductive/cubical
computations, determinism, short-circuit/branch laziness, and `unknown`
propagation.
