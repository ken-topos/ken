# RTP1 — `ken-interp` call-by-need substitution sharing (perf)

**Steward frame → Team Runtime.** A Runtime/interp **performance** WP. Fixes an
exponential evaluation blowup surfaced by VAL2. **Design is settled** — the
Architect resolved the fix-approach fork from `docs/PRINCIPLES.md`
(`evt_6cq5whrbkw1hs`, ruling **(B)**); this frame pins that as a fixed input, it
is **not** to be reopened. Owner: **runtime-leader → runtime-implementer →
runtime-qa.** Gate: **Architect soundness (value-preservation on the corpus) +
Runtime QA + CI.** No spec/CV vote (no `/spec` touch, no primitive). Findings →
**Steward**.

## Why
VAL2's `natToDecimalFueled` (Nat→decimal-String) is **exponential in the printed
value** — `natToDecimal 10` = 28.5s, ~2.4–3.2× per +1 — so `factorial`=120 /
`fibonacci`=55 are unreachable (infeasible, not merely slow). The algorithmic
depth is only ~log10(n); the cost is in the **evaluator**, not the Ken source.
The discriminating evidence (already gathered, VAL2):

| shape | case | cost |
|---|---|---|
| bound `n` referenced **once** per body | `printLoop` @ n=20 | **3.6ms** |
| bound `n` referenced **twice** per body (`div10 n`, `mod10 n`) | `natToDecimalFueled` @ n=10 | **28.5s** |

~8000× apart at comparable recursion depth, the *only* structural difference
being reference-multiplicity of a bound value. That is the textbook signature of
**call-by-name without sharing**: each reference independently re-walks the bound
value; ≥2 references compound through the recursion → exponential. This is the
same "no-sharing" family as the parked L3-strings `O(n^3.5–4)` item — (B) should
subsume both.

## Settled inputs — DO NOT REOPEN
- **Approach = (B): call-by-need / memoised-thunk substitution sharing in
  `ken-interp`.** Ruled by the Architect from PRINCIPLES.md
  (`evt_6cq5whrbkw1hs`). **Not (A)** div/mod primitives — `div`/`mod` are already
  derivable (`div10Fueled`/`mod10Fueled` elaborate + SCT-pass today); their sole
  blocker is the perf (B) fixes. (A) grows `trusted_base()` and imports the
  div-by-0-under-totality question — both avoided by (B). **Do not relitigate the
  primitive path.**
- **Soundness posture: (B) is soundness-inert.** Call-by-need ≡ call-by-name on
  *values* in a pure, total, strongly-normalising language — sharing changes
  **cost, never result**. So no conformance value may change. This is *why* the
  gate is "verify value-preservation on the corpus," not a trust-root check.
- **Scope = `ken-interp` ONLY.** The kernel's conversion-checker reducer *may*
  share the same characteristic — that is a **separate tracked candidate**
  (kernel-perf, soundness-inert), explicitly **out of scope here** per the
  Architect's steer. Do not touch `ken-kernel`. Zero `trusted_base` delta.
- **No `/spec` touch, no new primitive, no new `data`.** Pure evaluator change.

## Deliverable 1 — CONFIRM the root cause FIRST (confirm-then-fix)
The no-sharing diagnosis is *plausible-not-confirmed at the interp level* (the
build teams read it from behavior, not internals). **Before implementing**,
confirm it inside `ken-interp`:
- Instrument the evaluator on the **doubly-referenced-`n`** case the VAL2
  implementer isolated (reconstruct `natToDecimalFueled` from
  `crates/ken-elaborator/tests/zzdebug.rs`-style scratch, or a minimal
  `f n = g n + h n` over `Suc`-peeling). Show the bound value is **re-walked once
  per reference**, and that the re-walk count compounds through the recursion
  (i.e. it *is* the exponential source).
- The single-ref `printLoop` (n=20, 3.6ms) vs double-ref (n=10, 28.5s) pair **is
  the discriminating probe** — it already points unambiguously at sharing; the
  confirm step is to *see it in the evaluator*, not re-derive it behaviorally.
- **If confirmation surfaces a DIFFERENT root cause** (not substitution
  re-walking) → **STOP, route back to Steward → Architect** with the evidence.
  Do **not** implement call-by-need blind against a mis-diagnosed cause.

## Deliverable 2 — implement call-by-need substitution sharing
Give the evaluator's substitution/environment **memoised thunks**: a bound value
is forced **at most once**, and the forced result is **shared** across every
reference. Standard lazy evaluation (call-by-need). Target the substitution/
closure-application path in `ken-interp`'s evaluator (`eval.rs` — **verify the
exact mechanism against the landed code at pickup; this line is perishable**).
The change must be **semantics-preserving by construction** (memoise the result
of forcing, do not change *what* is forced or *whether* a value is forced in a
total language where all bindings are used-or-erased identically).

## Deliverable 3 — the regression proof (the soundness net writes itself)
1. **Perf collapse.** The previously-exponential double-ref case now tracks the
   single-ref cost curve: `natToDecimalFueled` at n=10, 20, and a stretch value
   (e.g. 55 / 120 — the VAL2 oracle values) completes in well-under-1s.
2. **Value-preservation (the gate).** `cargo test --workspace` **fully green**,
   and every conformance-corpus value is **byte-identical** pre/post. Call-by-
   need changes cost, not results — any value that changes is a **memoisation
   bug**, not an acceptable outcome. Spell this out in the merge Decision so the
   Architect can gate on it directly.
3. **Pinned perf-regression test.** A test that exercises the previously-
   exponential shape and asserts it completes within a generous bound, so the
   blowup **cannot silently return**.
4. **Bonus check (report, don't gate):** the L3-strings `O(n^3.5–4)` deep-Nat
   `slice 0 99` case should also improve (same family). Note the before/after;
   it's confirmation (B) subsumed the family, not an acceptance gate.

## Minor in-scope cleanup (fold-in — Runtime carry from console-harvest-fix)
While in `ken-interp`'s `eval.rs`, sweep the stale `ConsoleIds`/`build_print_line_tree` doc comment
(`eval.rs:~1538`: "2 for the production `ITree E R`" / "`ITree Console Unit`") → it should read the
landed **one-param `ITree r`** (`params_len = 1`). Cosmetic, zero behavior/trust impact — both
runtime-implementer and runtime-qa flagged it in the console-harvest-fix retros. In-scope only because
it's the same crate/file and zero-behavior; it keeps the diff `ken-interp`-only. Do **not** let it
expand the WP beyond that one doc-comment line.

## Acceptance criteria
- **AC1** Root cause **confirmed** as substitution no-sharing with interp-level
  evidence (or routed back to Steward if the cause differs). Confirm precedes
  fix.
- **AC2** `natToDecimalFueled` (VAL2 oracle values, incl. `factorial`=120 /
  `fibonacci`=55) evaluates in well-under-1s — the wall is gone.
- **AC3** `cargo test --workspace` green; **conformance values byte-identical
  pre/post** (value-preservation — the soundness gate).
- **AC4** Diff-scope: `ken-interp` only. **Zero** `ken-kernel` touch, **zero**
  `trusted_base` delta (grep-confirmed, not asserted).
- **AC5** A pinned perf-regression test guards the previously-exponential shape.

## Guardrails (do-not-reopen)
- Approach is **(B)**; do not reopen the div/mod-primitive path.
- Do **not** expand to the kernel reducer (separate tracked candidate).
- Do **not** change any observable value. A changed corpus value ⇒ a bug to fix,
  never an accepted delta. The whole point is *same values, faster*.

## Gate & sequencing
- **Gate:** Architect **soundness** (value-preservation on the corpus; trust root
  untouched by construction) **+ Runtime QA + CI**. No spec/CV (no `/spec` touch).
  The **confirm** step (D1) gates the **implement** step (D2) — a mis-diagnosis
  routes back before any evaluator change lands.
- **Lane:** Team Runtime (owns `ken-interp`).
- **Sequencing:** Team Runtime is mid-WP (`wp/console-harvest-fix`, in QA).
  **Released at that WP's close seam** (retros in → Handoff-Gate compaction →
  kickoff), one-WP-per-team. Branch cut off the `origin/main` that includes the
  merged console-fix. Staged now; fires when the console fix closes.
