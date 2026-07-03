# State-effect-build — build the direct `[State s]` effect surface (VAL2 #10 / OQ-C·C2)

**Steward frame → Team Language (lead) + Team Runtime (pair).** The `/spec`
(`§4.5`) and `/conformance` (EFF6) are **already elaborated and merged** (PR #237
`5626038` + erratum #238 `2bed9da`); this is a **build-only** WP — no new spec or
conformance authoring. Implement the direct `get`/`put`/`runState` surface
against the merged §4.5 model and turn the **red-until-built** EFF6 corpus green
**through the real interpreter** (not hand-fed). Gate: **Architect approach-review
+ soundness** (kernel-untouched AC1 is load-bearing; `runState` derived-not-
primitive) + **Spec review** (conformance-validator: EFF6 driven for real) +
Language QA + Runtime QA + CI. Findings → **Steward**.

Base: **`origin/main@2bed9da`**. Branch (pre-staged by Steward):
**`wp/State-effect-build`**.

## The settled model — DO NOT REOPEN (cite `/spec`, do not relitigate)

All decisions below are **locked** in the merged spec — build to them, do not
re-derive:

- **OQ-C = C2** (operator, 2026-07-03; `36 §4.5` preamble). `[State s]` is an
  **effect over the existing `ITree`/handler machinery**. **C1** (explicit
  state-threading) is the **floor it desugars to** (`§4.5.6`). **C3** (real
  mutable refs/cells/regions) is **FORBIDDEN** — no memory is mutated anywhere on
  the value path.
- **No new denotation.** Both the imperative `space` door (`§4.1`) and this
  direct monadic door desugar to the **same** `State S` signature (`§2.1`) and
  are discharged by the **same** `runState` fold (`§4.2`). `§4.5` only *exposes*
  that machinery as named operations.
- **`get`/`put` denotations are normative** (`§4.5.2`), operations of `State s`
  (`§2.1`: `Op = Get | Put s`, `Resp Get = s`, `Resp (Put _) = Unit`):

  ```
  get : Unit →[{State s}] s    get ()  ⤳  perform (inj Get)     = Vis (inj Get)     (λ r. Ret r)
  put : s   →[{State s}] Unit  put s'  ⤳  perform (inj (Put s')) = Vis (inj (Put s')) (λ _. Ret tt)
  ```

  `Resp Get = s` is **non-`Unit`, `s`-typed** — the first effect whose response
  depends on a type parameter (Console's is always `Unit`). This is the forcing
  function for lift (a) below.
- **`runState` is §4.2's fold at `F = 𝟘`, NOT re-specified** (`§4.5.3`):

  ```
  runState : s → ITree (State s ⊕ F) a → ITree F (a × s)   -- §4.2, verbatim
  runState s₀ m  :  a × s   when F = ∅   -- pure collapse: ITree 𝟘 (a × s) ≅ a × s (§2.4)
  ```

  The result pair `a × s` is `(result, final-state)` — the **Σ-pair `R × S`**
  that §4.2 returns, realized at runtime by the interpreter's **`EvalVal::Pair`**
  (`ken-interp/src/lib.rs:525`). **NOT** the also-landed inductive `data Prod a b
  = MkProd a b` (`prelude.rs:160`) — that is a distinct construct (`§4.5.3`).
- **`runState` is an ordinary *total Ken definition*** — the §4.2 `elim_ITree`
  fold, structural on the sub-tree, **kernel-re-checked — NOT a trusted Rust
  primitive.** This is *precisely* what makes `[State s]` zero-`trusted_base`
  delta (AC1): the handler is **derived, never postulated.** The runtime merely
  *evaluates* it (call-by-need over the pure tree); it is **not** an I/O driver
  like Console's `run_io` (`§7.2`) — pure state threading performs no I/O.
- **Surface *spelling*** (`[State s]`, `get`, `put`, the pair constructor) is
  proposal-level `OQ-syntax` (`§4.5.2`) — **do NOT freeze a surface constructor
  name.** Assert the pair's **components, order, and Σ-pair denotation**; the
  stdlib pair name is not yet final (over-freezing a deferred spelling would
  falsely fail a valid build). The **operations and their denotations are
  normative**; the spelling is not.

## The build — three outer-ring lifts, no kernel change (`§4.5.6`)

The landed **runnable** `ITree` (`crates/ken-elaborator/src/prelude.rs:170`) is
**Console-hardwired** — `data ITree r = Ret r | Vis ConsoleOp (Unit -> ITree r)`:
a fixed `Unit` response, no effect parameter. `[State s]` is the **forcing
function** to lift three simplifications already flagged in-code, up to the §2/§4
model. **All three are §36-normative and admitted by K1.5's generic `elim_ITree`
(`crates/ken-kernel/tests/k1p5_wstyle.rs`) — the kernel is untouched (AC1):**

- **(a) Dependent response `E.Resp e`.** `State s` needs `Resp Get = s`
  (non-`Unit`). The runnable `ITree`'s `Vis` continuation must accept an
  effect-op-indexed response type, not a fixed `Unit`. *(Language: `prelude.rs`
  ITree decl + `effects/itree.rs` — currently a fixed-`u64`-response static
  stand-in.)*
- **(b) Container coproduct `⊕`** for `State s ⊕ F` (composition, `§4.5.4`). The
  effect-op position of `Vis` must range over a **coproduct** of effect
  signatures, so `State s` and `Console` (and any `F`) coexist in one tree.
  *(Language: `effects/*` — `row.rs`/`algebra.rs`/`lower.rs`.)*
- **(c) Named-effect dispatch** so `runState` **peels `State`** (its `inl`
  clauses) and **passes every other op through** (`Vis (inr o)`, §4.2's fourth
  clause). *(Meets at the interface: Language emits the dispatch in the derived
  `runState`; Runtime's `elim_reduce` must actually fold it over `Vis` nodes —
  `ken-interp/src/lib.rs:1693` placeholder "requires K1.5 IH in `elim_reduce`".)*

## Lane split (Steward's sequencing call — coordination mechanism is Architect's at approach-review)

Disjoint crates → **one branch, no file contention.** Both teams build to the
**spec-fixed interface** (`§4.5.2`/`§4.5.3`), so they develop in parallel and
integrate on `wp/State-effect-build`:

- **Language (LEAD) — `crates/ken-elaborator/`.** The ITree representation lift:
  `prelude.rs` (E-parameterize the `ITree` decl) + `src/effects/*` (dependent
  `Resp` (a), `⊕` coproduct (b), named-effect dispatch representation (c)) + the
  **derived stdlib** `get`/`put`/`runState` definitions (§4.5.2/§4.5.3, real Ken
  terms — NOT Rust primitives).
- **Runtime (PAIR) — `crates/ken-interp/`.** `elim_reduce` gets the **K1.5 IH**
  so the `runState` `elim_ITree` fold **actually reduces over `Vis` nodes**
  (`lib.rs:1693`/`:1736` placeholders → real), producing the **Σ-pair** result
  via `EvalVal::Pair` (`lib.rs:525`).
- **Interface (spec-fixed, both build to it):** the `ITree` term shape after lift
  (a)/(b) and the `get`/`put`/`runState` denotations of `§4.5.2`/`§4.5.3`.
  Language owns the emitted shape; Runtime folds over it. **No file overlap** —
  Language touches `ken-elaborator/*`, Runtime touches `ken-interp/*`.

**Architect confirms/revises at approach-review:** the lane split, the
coordination mechanism (parallel-one-branch vs serial), and that the `runState`
fold stays **derived-not-primitive** with the kernel untouched. This is the
"means to the end" the operator delegated to Architect — the *goal* (below) is
fixed; the *how* of the two-crate integration is Architect's call.

## Acceptance criteria — the EFF6 corpus, RED on `main` until built

**Do NOT hand-feed these green** (`conformance-hand-feeds-the-deliverable`). Each
must pass by driving the program **through the real interpreter**, not a harness
that pre-supplies the `(result, state)` pair. EFF6 is the **acceptance target**
authored with the spec.

- **AC1 — Kernel untouched (LOAD-BEARING, `direct-state-kernel-untouched`).**
  `git diff origin/main -- crates/ken-kernel/` **empty**; `trusted_base()`
  unchanged; **no** `State`/`Get`/`Put`/`runState` kernel `Term` or `Decl`
  variant; **no** new `declare_primitive`/`declare_postulate`. Verify by grep,
  not a test. If any build path needs a kernel variant for state, **that is the
  finding** (C2→C3 boundary breach — stop and escalate to Steward).
- **AC2 — `direct-state-next-post-increment`.** The direct-surface post-increment
  `next () = bind (get ()) (λ n. bind (put (n + 1)) (λ _. Ret n))`, driven
  through the **real interpreter**: `runState 0 next` reduces to **`(0, 1)`** and
  `runState 41 next` to **`(41, 42)`** — `(result, final-state)`: result = the
  **old** (pre-increment) value, second = the **final** state. Discriminating:
  a state-threading bug (`Put` not adopted) yields `(0, 0)`; a pair-order swap
  yields `(1, 0)`. Assert **components + order + Σ-pair denotation** (`EvalVal::
  Pair`), **not** a constructor spelling (deferred `OQ-syntax`).
- **AC3 — `direct-state-console-commute`.** A `[State Int, Console]` program
  `logged_next () = bind (get ()) (λ n. bind (put (n+1)) (λ _. bind (perform
  Console (Write "log")) (λ _. Ret n)))`, discharged two ways — `runState 0
  (handleConsole m)` and `handleConsole (runState 0 m)` — **both type-check and
  thread state identically**: pair `(0, 1)` and **one** `Console.Write "log"`,
  whichever peels first. `runState` passes Console ops through untouched
  (`Vis (inr o)`); the handlers **commute**. The **two orders are the net** — a
  single order would be green-vs-green under a pass-through bug.
- **AC4 — `direct-state-no-cross-run`.** The **same** pure tree `next` run twice
  from two initial states in one computation — `runState 0 next` then
  `runState 41 next`, no nesting between — yields **two independent** results
  `(0, 1)` and `(41, 42)` with **no cross-run state** (run-2 re-threads from `41`,
  not from run-1's final `1`). A shared mutable cell (forbidden C3) would leak
  and yield `(1, 2)` for run-2 — the **re-runnability flips.** This is the
  executable witness that no in-place mutation entered the value path.
- **AC5 — C3-forbidden grep face (`§4.5.5`).** No `RefCell`/`Cell`/`unsafe`/
  interior mutability introduced on the value path; the effect **erases** to its
  `ITree` description; the state is `runState`'s **parameter**, threaded
  functionally, never a cell.
- **AC6 — No regression.** `cargo test --workspace` green; **Console and EFF4's
  `space` door behave identically pre/post** (EFF6 and EFF4 exercise the *same*
  `§4.2` fold through different surfaces — the shared mechanism must not shift).

## Guardrails (do-not-reopen)

- **Kernel / `trusted_base` OFF-LIMITS** — outer-ring only (`ken-elaborator` +
  `ken-interp` + derived stdlib). A kernel `Term`/`Decl` need = escalate, don't
  patch.
- **`runState` is DERIVED, not a primitive** — a real total Ken def (the §4.2
  `elim_ITree` fold), kernel-re-checked. Do **not** shortcut it as a trusted Rust
  function; that would grow `trusted_base` and break AC1.
- **C3 forbidden** — no mutable cell/ref/region on the value path; state is a
  threaded parameter.
- **No surface-constructor freeze** — assert pair components/order/denotation,
  not a spelling (`OQ-syntax` deferred).
- **One effect-dispatch path** — reuse the `ITree`/Console machinery; this same
  path is shared with the future `[FS]` WP. Do **not** build a second effect
  system.
- **Drive EFF6 for real** — through the real interpreter, never hand-fed
  (`conformance-hand-feeds-the-deliverable`).

## Sequencing

- **Gate:** Architect approach-review + soundness (kernel-untouched + derived-
  not-primitive + C3-forbidden) + Spec review (conformance-validator: EFF6 driven
  for real, not hand-fed) + Language QA + Runtime QA + CI. Merge via Integrator.
- **Lane:** Language (lead, `ken-elaborator`) + Runtime (pair, `ken-interp`).
  Branch `wp/State-effect-build` off `origin/main@2bed9da`, pre-staged by Steward.
- **Relation to siblings:** shares the `ITree`/effect-dispatch machinery with the
  queued `[FS]` WP (`FS-driver.md`) — build the shared path **once**, cleanly, so
  `[FS]` extends it rather than forking a second effect system.
