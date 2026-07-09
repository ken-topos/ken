# State-effect — a pure `[State s]` effect (VAL2 #10 / OQ-C)

**Steward frame → spec enclave (elaborate) → Runtime + Language (build).** This
is a **design-settled** capability WP closing VAL2 finding #10
(`accumulator-factory`: no way to express stateful computation). The product
decision is **locked by the operator** (OQ-C · C2, `VAL2-gap-design-OQs.md`,
2026-07-03) — this frame pins it and it is **not to be reopened**. Owner:
spec-leader elaborates the semantics; **Runtime** (handler/driver) + **Language**
(surface) build. Gate: **/spec + /conformance touching** → spec enclave
elaboration, **Architect soundness** (purity + totality + kernel-untouched),
**Spec review** (conformance-validator), team QA, CI. Findings → **Steward**.

## The locked decision — DO NOT REOPEN

The operator settled OQ-C on the merits, separating **mutation** (not required,
forbidden) from **stateful computation** (required, and expressible purely):

- **C2 is chosen:** a `[State s]` **effect** — `get`/`put` threaded by a
  **handler** over the **existing `ITree` free-monad effect machinery** (the same
  interpreter the **Console** effect already runs on). "Mutation" is a
  *description* interpreted in the handler; **no memory is mutated**.
- **C1 (state-threading idiom) is the floor** — always available, and a `[State]`
  program must be explainable as sugar over it. C2 does not replace C1; it
  packages it.
- **C3 (real mutable refs / ST regions / `IORef`-style cells) is FORBIDDEN.** It
  is the imperative escape hatch `docs/PRINCIPLES.md` rules out (largest TCB, real
  mutation, breaks purity). Do **not** introduce a mutable cell, region, or
  reference anywhere. If any design step seems to *need* real mutation, that is
  the signal you have left C2 — **STOP and route to Steward**, do not reach for
  C3.

**Why this is sound (the posture the build must preserve, not relitigate):** the
effect is a pure value (an `ITree` node); state lives in the **handler's
parameter**, threaded functionally by structural recursion over the tree;
`runState` is total. `[State s]` is therefore purity- and totality-**preserving
by construction**, exactly like Console/`[FS]` — an **outer-ring** effect +
stdlib handler, **zero kernel / `trusted_base` delta.**

## Means are the enclave's + Architect's call (operator, 2026-07-03)

This frame fixes the **goal + properties + acceptance**, not the mechanism. The
**exact** effect-row representation, the `get`/`put` signatures, how the state
type `s` is carried in the row, and the handler's implementation are the
**enclave's elaboration + Architect's** call — the same don't-over-specify
discipline used on the L3-strings floor. Where this frame sketches a signature it
is **illustrative**, tagged *verify/decide against the landed effect system, not
this line*.

## Mandated deliverable outline (each item resolves to a concrete choice)

1. **Effect declaration.** Add `[State s]` to the effect system with its
   operations — illustratively `get : [State s] s` and `put : s -> [State s]
   Unit` (enclave fixes the exact surface + how `s` parameterizes the row).
   Reuse the existing effect-row + `ITree` node machinery; **no second effect
   system.**
2. **Handler.** `runState : s -> ([State s] a) -> Pair a s` (or the enclave's
   chosen shape) — the pure state-threading interpreter over the `ITree`,
   structurally recursive, carrying `s` as its parameter. Specify where it lives
   (Runtime interpreter + any `catalog/packages/` surface).
3. **Surface + interaction.** How a Ken program writes `get`/`put` and runs
   `runState`, and how `[State s]` composes with other effect rows (e.g. a
   program that is both `[State s]` and `[Console]`). Enclave specifies row
   composition.
4. **Conformance — drive the REAL producer.** A test that runs a `get`/`put`
   program under `runState` **through the real interpreter** (not a hand-fed
   harness that pre-supplies the threaded value — that is green-vs-green) and
   checks the correct final `(result, state)`. The `accumulator-factory` shape
   (VAL2 #10) is the canonical example. Grounded against the landed producer, per
   [[conformance-hand-feeds-the-deliverable]].
5. **Purity/totality statement.** An explicit argument (spec + a discriminating
   check) that `runState` is total and that **no mutable cell is introduced** —
   the effect is erasable to its `ITree` description; the state is a function
   parameter, not a store.

## Acceptance criteria

- **AC1 — Kernel untouched (load-bearing).** `git diff origin/main --
  crates/ken-kernel/` empty; `trusted_base()` unchanged; **no new kernel
  `Term`/`Decl` variant** for state. `[State s]` is an outer-ring effect +
  handler, exactly like Console — verify by grepping the kernel is not in the
  diff, not by a test. (Sibling posture: [[abstraction-visibility-feature-soundness-gate]].)
- **AC2 — `[State s]` drives end-to-end through the real interpreter.** A
  `get`/`put` program under `runState` produces the correct threaded
  `(result, state)` via the actual eval path; `accumulator-factory` becomes
  expressible and correct.
- **AC3 — Purity + totality preserved.** `runState` is total (structural on the
  `ITree`); **no mutable reference/cell/region exists in the diff** (grep:
  no `RefCell`/`Cell`/`unsafe`/interior-mutability introduced into the value
  path); the effect is a pure description. A discriminating check distinguishes
  "state threaded in the handler parameter" from "state mutated in place."
- **AC4 — No regression.** `cargo test --workspace` green; Console and every
  other effect row behave identically pre/post.

## Guardrails (do-not-reopen)

- **C3 is forbidden** — no mutable refs/cells/ST/regions; if a step needs real
  mutation, STOP → Steward.
- **Reuse the `ITree`/Console effect machinery** — do not invent a second effect
  interpreter.
- **Kernel / `trusted_base` off-limits** — the effect + handler live in the
  outer ring.
- Means (row representation, signatures, handler impl) are the **enclave's +
  Architect's** call — do not treat this frame's illustrative signatures as
  fixed.

## Sequencing

- **Gate:** /spec + /conformance touching → spec enclave elaborates the
  semantics on this WP branch, merges to `main` via the Integrator, then the
  build teams are kicked. Architect soundness (purity/totality + kernel-untouched)
  + Spec review (conformance-validator) + team QA + CI.
- **Lane:** Runtime (owns the `ITree` interpreter/handler) + Language (surface).
  Branch off `origin/main`.
- **Relation to siblings:** shares the effect-handler mechanism with the `[FS]`
  driver WP (both extend the Console `ITree` interpreter) — coordinate so the two
  reuse one effect-dispatch path, not two.
