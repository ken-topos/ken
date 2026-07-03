# RTP1 — `ken-interp`: eliminate redundant per-reduction recomputation (perf)

**Steward frame → Team Runtime.** A Runtime/interp **performance** WP. Fixes an
exponential evaluation blowup surfaced by VAL2. **Design is settled** — the
Architect resolved the fix approach from `docs/PRINCIPLES.md` and **re-ruled it on
instrumented data** (`evt_7tgn3jz0z0kr4` + env-size amendment `evt_4snaecww34mnz`);
this frame pins that, it is **not** to be reopened. Owner: **runtime-leader →
runtime-implementer → runtime-qa.** Gate: **Architect soundness (value-preservation
on the corpus + kernel-untouched) + Runtime QA + CI.** No spec/CV (no `/spec` touch,
no primitive). Findings → **Steward**.

> **⚠ THIS FRAME SUPERSEDES the original "call-by-need substitution sharing"
> approach.** RTP1's D1 confirm (`evt_4egrcrt5srd7v`) **falsified** the no-sharing
> premise with instrumented `elim_reduce` call-counts. The corrected root cause and
> fix direction (B') are below. `single` (nothing to share) is *already* 2×
> exponential and `doubleLet` (the call-by-need target) is the *same* rate — so
> substitution sharing already works; call-by-need has no purchase. **Do not
> implement memoised substitution.**

## Why
VAL2's numeric-printing (`natToDecimalFueled`) and `mergeSort` are catastrophically
slow, and D1 instrumentation pinned the cause. Two observables, most plausibly **one
bug**:
- **Value-depth / redundant-walk (confirmed).** `elim_reduce` call-counts:
  `single` (`Suc m => Suc (single m)`, one self-ref) = **2.00×/+depth**; `doubleLet`
  (`let r = … in natAdd r r`) = **same 2.00×**; `double` (two explicit self-refs) =
  **3.00×**. `single` has zero reference-multiplicity yet is already exponential →
  the cost is a **structurally-duplicated recursive walk the mechanism always
  performs**, not a shared redex.
- **Environment-size amplification (VAL2 finding #6).** Prepending **3 small,
  semantically-unrelated** decls (`data OrdResult = Lt|Eq|Gt`, a `list_append`, a
  `concat`) to `gcd.ken` turns an otherwise-**55ms** `natToDecimal(4)` into a
  **60s+ timeout** (1000×+). The per-reduction redundant work also scales with
  **environment size** (weakening / GlobalId indexing / term-size-under-weakening
  over *all* globals).

## Corrected root cause (Architect-confirmed)
`elim_reduce`'s **eager IH computation** (`eval.rs` ~445-450) computes the
induction-hypothesis value for **every** recursive constructor position
**unconditionally** — on every reduction, whether or not the selected method body
consumes that IH binder. For surface recursion compiled as a self-call (not via the
IH binder), the eager IH is a **redundant walk whose result is discarded**
(`apply`'s `_ => Neutral`), while the body does its own recursion → the 2× baseline,
compounding with each additional explicit self-reference. The env-size amplification
is most plausibly the **same** bug's per-step cost scaling with env size
(`redundant-walk-count × O(env)/step`); eliminating the redundant walks collapses
the exponential multiplier and leaves the O(env) cost applying only to the *actual*
(linear) reductions — so **both dimensions collapse together**. *If* D-instrumentation
shows per-reduction work scales with env size **independently** of the IH redundancy,
that's a **second sub-fix within this same WP** (avoid re-weakening/re-walking over
the full env per step — still `ken-interp`-only, still value-preserving), **not** a
strategy change.

## Settled inputs — DO NOT REOPEN
- **Fix DIRECTION = (B'): eliminate the redundant per-reduction recomputation** —
  the eager `elim_reduce` IH walk (make it **lazy and/or conditional on the method
  body actually consuming the IH binder** — unconsumed IH costs nothing; a consumed
  one is computed once) **and** whatever env-scaled per-step work it multiplies. The
  Architect ruled the **direction + properties**, not the exact mechanism
  (conditional-skip vs lazy-thunk vs both, and the env-scaling sub-fix if any, are
  the WP's **engineering call** — same don't-over-specify discipline as the
  L3-strings floor).
- **NOT (A) `div_int`/`mod_int` primitives.** Ruled out and re-affirmed: growing the
  trust root to paper over an outer-ring evaluator inefficiency is backwards; `div`/
  `mod` are already derivable and their sole blocker is this perf.
- **Soundness posture — value-preserving, soundness-inert.** Not computing a
  discarded IH cannot be observed; computing a used IH lazily gives the identical
  value; in a pure total SN language eval-order/laziness never changes a value or
  termination. **Zero conformance-value change** — the regression net is corpus
  byte-identity.
- **Scope = `ken-interp` ONLY.** Kernel `Elim`/conversion checker **untouched**
  (load-bearing). Zero `trusted_base` delta. (Kernel conversion-checker perhaps
  shares the trait — a **separate** tracked candidate, out of scope here.)
- **Calibration — don't over-expect the fix.** (B') removes the *interpreter's*
  mechanism overhead, **not** algorithmic complexity. A source-level-exponential
  algorithm (naive `fib(n-1)+fib(n-2)`) correctly stays exponential. Expect the
  interp-strangled cheap algorithms (`natToDecimal` ~log₁₀n, `mergeSort` n log n) to
  **collapse dramatically**; measure how far, and label any residual as
  **algorithmic**, not a mechanism miss.

## Deliverable 1 — CONFIRM/pin the amplifier(s) FIRST (D1 gates D2)
The value-depth cause is confirmed (the call-count table). **Before implementing,
instrument to pin the env-size amplifier:** is it the **same** redundant-walk bug
paying `O(env)` per step, or an **independent** per-reduction env-scaling (e.g.
every reduction re-weakens terms over the full global env regardless of IH
redundancy)? Use the discriminating probe — the exact **3-line-prelude + `gcd.ken`**
repro (`OrdResult` + `list_append` + `concat` prepended). This tells you whether
(B') is one sub-fix or two. **If instrumentation surfaces a *third* surprise
(neither redundant-IH nor env-scaled-per-step) → STOP, route back to Steward.**

## Deliverable 2 — implement (B')
Eliminate the redundant per-reduction recomputation: lazy/conditional `elim_reduce`
IH computation, plus the env-scaling sub-fix **if D1 shows it's independent**.
Mechanism is your engineering call; the properties (value-preserving, ken-interp-
only) are fixed. Also sweep the stale `eval.rs` `ConsoleIds` doc comment
(`~1538`: "2 for the production `ITree E R`" → one-param `ITree r`) — Runtime carry,
one line, zero-behavior, same file.

## Acceptance criteria — the Architect's soundness gate + the two perf probes
- **AC1 — Kernel untouched (LOAD-BEARING).** `git diff origin/main --
  crates/ken-kernel/` **empty**; `trusted_base()` unchanged; `ken-interp` only. *If
  the fix would touch the kernel checker — STOP, escalate.*
- **AC2 — Value-preservation (the soundness gate).** `cargo test --workspace` green;
  the conformance corpus produces **byte-identical values pre/post** (a changed
  value = a bug, never an accepted delta).
- **AC3 — Perf probe #1 (value-depth).** The previously-exponential mechanism cases
  (`single`/`doubleLet`, `natToDecimalFueled`) **collapse to linear/near** — measure
  and report the before/after growth curve.
- **AC4 — Perf probe #2 (env-size).** The exact **3-line-prelude + `gcd.ken`** repro
  goes **timeout → fast**. *If (B') collapses AC3 but NOT AC4, that's the route-back
  signal (a second dimension to instrument) — not a merge.*
- **AC5 — Pinned regression tests** guard both previously-pathological shapes so the
  blowup can't silently return.

## Guardrails (do-not-reopen)
- Direction is **(B')** — do not reopen call-by-need substitution sharing (falsified)
  or div/mod primitives (ruled out).
- Kernel checker off-limits; fix the evaluator.
- Don't over-correct into an algorithmic-complexity claim — (B') is a mechanism fix.

## Gate & sequencing
- **Gate:** Architect soundness (AC1 kernel-untouched + AC2 value-preservation +
  AC3/AC4 both probes collapsing) + Runtime QA + CI. No spec/CV. **D1 confirm gates
  D2** — pin the env amplifier before implementing.
- **Lane:** Team Runtime (owns `ken-interp`). Branch `wp/RTP1-interp-sharing` (held
  at D1, zero diff). **First step on resume:** refresh this frame doc onto the branch
  (`git checkout steward/work -- docs/program/wp/RT-perf-sharing.md`) so the branch
  carries the corrected (B') contract, then proceed to D1's env-instrumentation.
