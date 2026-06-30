# WP T2 — the REPL (the *Little Prover* loop)

**Owner:** Team Ergo. **Branch:** `wp/T2-repl` (cut from `origin/main`).
**Stream
/ gate:** WS-T (ergonomics) → **G7** (the agent write→verify→repair loop).
**Depends on:** V4 (the verification spine / diagnostic) — **merged**; X1
(interpreter) — **merged**; T1 (the diagnostic protocol) — **merged**.
**Crates:**
`ken-cli` (the scaffold already anticipates a `repl` subcommand) + the landed
`ken-elaborator` / `ken-kernel` / `ken-interp`.

> **⚠ Process — this is a DIRECT Steward→Ergo build (NO enclave step).** T2 is
> *tooling over existing interfaces* — it authors **no** `/spec` and **no**
> `/conformance` (it adds no language feature; the kernel + V-spine are
> unchanged
> and still gate everything). So there is no spec-elaboration ring: **build
> directly from this frame.** Ergo's own tests are the acceptance bar; the merge
> Decision is **Architect-only** (crates-only, soundness-reviewed at merge).
> **Perishable:** drive the **landed** crate APIs (`ken_elaborator`,
> `ken_kernel`,
> `ken_interp`, the V-spine, T1's verdict/witness types) — read them at pickup,
> don't invent signatures.

## 1. Objective (one line)

Turn the `ken` CLI scaffold into an interactive **REPL** that runs the *Little
Prover* loop — **define → state a property → get a verdict** (proved / tested /
refuted / unknown, with witness/countermodel) and **evaluate expressions** —
over
the already-built kernel, verification spine, and interpreter.

## 2. Settled inputs — FIXED, do not reopen

1. **The REPL DRIVES the existing pipeline; it does not reimplement or weaken
   it.**
   Elaboration is `ken-elaborator` (V0), checking is `ken-kernel`, verification
   is
   the V-spine (V0–V4), evaluation is `ken-interp` (X1). A REPL bug can only
   mis-display or crash — **it cannot make an unsound result pass** (the kernel
   check is the same trusted gate). Do not add a second checker/evaluator.
2. **Verdicts use the T1 diagnostic protocol — do NOT invent a verdict format.**
   The proved/tested/refuted/unknown trichotomy + witness/countermodel are T1's;
   render them, don't re-author them. A refuted goal shows its **countermodel**;
   a
   proved goal shows **proved (Q)**; an open/abstract goal shows
   **unknown/tested**
   — the verdicts must be **distinct and correct** (the discriminating-verdict
   discipline applies to what the REPL prints).
3. **Build over the capability that EXISTS NOW, grow as the surface lands.** The
   kernel + V-spine + X1 are built; the surface language (L1–L8) is landing
   incrementally. The REPL works over whatever the elaborator currently accepts
   and
   **gets richer as L-WPs merge** — do **not** block T2 on the full surface.
4. **Scope is the interactive loop, NOT the G7 automation.** T2 is the
   human/agent-driven REPL. The automated write→verify→repair *agent loop* (and
   any
   IDE/LSP) is a later WS-T layer — not here.

## 3. Mandated deliverable outline (each ends in an implementable choice)

Deliver in `ken-cli` (a `repl` module/subcommand; factor a small `ken-repl` if
it
helps, your call):

1. **The `repl` subcommand + the read-eval-print loop.** `ken repl` enters an
   interactive prompt. Each input is classified — a **definition**, an
   **expression**, a **goal/property**, or a **`:`-command** — and dispatched.
   Multi-line input (a block) is supported; decide the line/block boundary rule
   and state it.
2. **The command set (pin the exact tokens).** At minimum: `:def` (or bare
   definition) to elaborate+check+register a definition; `:check`/`:prove
   <goal>`
   to run verification and print the verdict; `:eval`/bare-expression to
   evaluate
   and print the value; `:type <expr>` to infer+print the type; `:list` /
   `:reset` for session state; `:help`; `:quit`. (Also expose the
   non-interactive
   forms — `ken check <file>` / `ken eval <expr>` — as the same machinery the
   scaffold's comment anticipates, if cheap.)
3. **The verify path → verdict display.** A stated goal routes through the
   V-spine
   and prints the **T1 verdict**: `proved` (Q) / `tested` (P, with the test
   evidence) / `refuted` (with the countermodel/witness) / `unknown` — in the
   model's vocabulary. **The verdict must reflect the real prover result**, not
   a
   placeholder.
4. **Session state.** A growing context of registered definitions usable by
   later
   inputs (define `f`, then `:check` a property of `f`). Pin how a redefinition
   is
   handled (shadow / error — your call, state it).
5. **Diagnostics, never panics.** A parse error, type error, or elaboration
   failure prints a **T1-style diagnostic** (location + message) and returns to
   the
   prompt — the REPL never crashes on bad input.

## 4. Testable acceptance criteria (Ergo's own tests — no `/conformance` seed)

- **AC1 (define + type)** Defining a well-typed function registers it and
  `:type`
  prints its inferred type; an ill-typed definition prints a diagnostic and is
  **not** registered.
- **AC2 (the verdict trichotomy — discriminating)** A **provable** goal prints
  `proved`; a **refutable** goal prints `refuted` **with a countermodel**; an
  **abstract/open** goal prints `unknown` (or `tested` if it routes to testing).
  The three are **distinct** and each is the **real prover verdict** — a test
  that
  swaps a provable goal for a refutable one must flip the printed verdict (not
  green-vs-green).
- **AC3 (eval)** Evaluating an expression prints the value computed by
  `ken-interp`
  (a real reduction, e.g. `2 + 3` → `5` once L1 lands; a kernel/V-spine term
  today).
- **AC4 (diagnostics not panics)** Malformed input, an unbound name, and a type
  error each print a diagnostic and return to the prompt — **the process does
  not
  panic or exit**.
- **AC5 (session state)** A definition entered on one line is usable by a
  `:check`
  or `:eval` on a later line.
- **AC6 (drives the REAL pipeline)** The verify path goes through the **actual**
  V-spine + kernel (grep-trace: the verdict originates from the real prover
  call,
  not a hardcoded string); eval goes through the **actual** `ken-interp`.

## 5. Do-not-reopen guardrails

- **Drive, don't reimplement** — one kernel, one V-spine, one interpreter; the
  REPL is a front-end (§2.1).
- **T1 verdict format** — render it, never invent a second (§2.2).
- **No G7 automation / no IDE** in T2 — the interactive loop only (§2.4).
- **No new `/spec` or `/conformance`** — this is tooling; if you find yourself
  wanting to *change* language behavior, that's a different WP — flag it to the
  Steward, don't fold it in.
- **Verdicts must be honest** — `proved` only when the kernel certified it; a
  test-only result is `tested`, never shown as `proved` (the project's honesty
  charter; mirrors B1's Q/P discriminator at the UI).

## 6. Sequencing / process notes

- **Direct build:** implementer builds from this frame on `wp/T2-repl` → Ergo-QA
  verifies against §4 → **merge Decision Architect-only** (crates-only). §14:
  `merge_ready` states `status: resolved` + a real @mention to the Architect
  (`agt_37reqftfe6g00`). No spec-leader / conformance-validator step.
- **Grows with the surface:** as L1 (numbers, in QA now), L2 (sum/match), …
  land,
  the REPL automatically handles more — no T2 rework needed; later WS-T WPs (T3
  test framework, T4 docs, T5 ecosystem) build on this loop.
- The REPL is the visible face of **G7** (the agent write→verify→repair loop) —
  honest verdicts + good diagnostics here are what make the loop usable.
