---
id: SEC1-IFC-R3
title: "[Sec1-reduce] cannot be reified yet: NO production path can return Verdict::Disproved, so the verdict D5 requires is unreachable and every Disproved in sec1_acceptance is hand-rigged"
status: draft
owner: verify
size: M
gate: G-Sec
depends_on: []
blocks: []
github: null
origin: "verify-implementer authorized hard-stop on SEC1-IFC AC-R3 (2026-07-27), Steward-authorized evt_1g1tq7ybc92hj. R1+R2 landed as PR #1094 (main tree 8229a811). Measured by the Steward at origin/main 4d15002d: no crates/*/src/ path constructs Verdict::Disproved. Blocked on BOTH V3 (prover, a route that can refute) and V4 (diagnostics, whose DAG deliverables are countermodels+holes+unknown) -- prover.rs names V4 seven times; neither has a tracker node."
---

> ## ⛔ STATUS IS `draft` DELIBERATELY — THIS IS NOT RELEASABLE AND MUST NOT BE PULLED
>
> ⛔ **Do not release this to Team Verify.** It is blocked on infrastructure that
> does not exist, and the whole point of filing it now is that the gap is
> **durable and named** rather than living in a channel message.
>
> ⭐ It flips to `ready` only when a prover **refutation backend** exists — see
> §"The blocking dependency": **`V3` AND `V4`**, which carry different halves of
> it and are both required.

## What happened

`SEC1-IFC` (PR #1094) delivered `AC-R1` (`[Sec1-dual]`) and `AC-R2`
(`[Sec1-launder]`). `AC-R3` (`[Sec1-reduce]`) hit an authorized hard stop.

⭐ The frame anticipated this and said so: *"if `AC-R3` turns out to need prover
work that is not present, land `AC-R1`+`AC-R2` and re-raise `AC-R3` to the
Steward with what you measured — ⛔ do not stub it further."* That is exactly what
happened, so `[Sec1-reduce]` **correctly remains live** in `ifc.rs` and in the
suite's stub inventory. ⛔ It is an authorized deferral, ⛔ not a silent
completion claim.

## ⭐⭐ The measurement — and it is worse than "the reduction is stubbed"

Measured at `origin/main = 4d15002d`.

**No production code path can return `Verdict::Disproved`.**

`grep -rn 'Verdict::Disproved *{' crates/*/src/` returns **six** hits and **all
six are pattern-match arms**, not constructions:

| site | what it is |
|---|---|
| `ken-elaborator/src/prover.rs:70` | the **enum variant declaration** |
| `ken-elaborator/src/protocol.rs:89` | `⇒ ObligationStatus::Refuted` — a mapping |
| `ken-elaborator/src/protocol.rs:100` | `⇒ Some(WireVerdict::False)` — a mapping |
| `ken-elaborator/src/diagnostics.rs:238` | rendering a countermodel |
| `ken-elaborator/src/export.rs:435` | the refuse-to-export guard |
| `ken-cli/src/repl.rs:76` | a `match v` display arm |

And every route terminates in `Proved` or `Unknown`, never `Disproved`:

```
attempt_obligation  → classify → { Route::D, Route::FO, Route::HO }   -- exhaustive, no skip
  attempt_d   → attempt_ipc, else emit_unknown_hole
  attempt_fo  → attempt_ipc skeleton, else emit_unknown_hole
  attempt_ho  → attempt_ipc
  attempt_ipc → Verdict::Proved { cert }  (kernel-checked)  |  emit_unknown_hole
```

⇒ **The refutation half of the verdict trichotomy (`23 §1.2`) has no producer.**

### ⛔ Positive control on that negative claim — the grep key is not the wrong key

A zero-hit grep proves nothing by itself. **The identical pattern finds a genuine
construction** in the test tree —
`crates/ken-elaborator/tests/sec1_acceptance.rs:70`,
`verdict: Verdict::Disproved {` inside the helper documented at `:52` as
*"Synthetic `Disproved` result (for cases where the prover lacks the backend)"*.

⇒ The pattern **can** detect a construction. The absence in `src/` is a real
absence, not a mis-keyed probe.

## What that costs, in four places

1. ⭐⭐ **`ifc.rs:470–472` requires an unreachable verdict.** Its doc says
   *"Returns `true` iff the verdict is `Disproved` — the sole acceptable
   outcome."* ⇒ `check_reduction_faithfulness` is not merely a verdict-**shape**
   predicate over a synthetic obligation; **the outcome it demands cannot be
   produced by any program input.** That is a stronger and worse statement than
   the trigger's own comment makes.
2. **`D5` is `N2`'s sole net, and it is fed by hand.** `sec1_acceptance.rs:436–438`
   states this in-source: *"`matches!(v, Disproved)` is a verdict-SHAPE predicate;
   it asserts 'Disproved is Disproved.' The test feeds `synthetic_disproved(...)`
   — a hand-rigged `ProverResult::Disproved`. No `product(c,ζ)` construction."*
   ⇒ The `N2` failure mode — a too-weak `Φ_post` — remains undetectable.
3. **The conformance seed's `AC3` is partially vacuous.** Any seed row whose
   expected outcome is a refutation is satisfied today only through a synthetic
   verdict. ⚠ Sec1's **by-proof half has no executable producer**, and the
   seed does not say so.
4. **`export.rs:435`'s refusal arm can never fire in production.** The
   *"a refuted claim is never exported"* boundary (`71 §2.1`) is real code that no
   input reaches. ⚠ It fails **safe** — no false refutation can leak — but the
   arm is untested against a real refutation and must not be read as exercised.

⭐ **The honest part, which is why this is a gap and not a defect:** every one of
these is disclosed **in source**, at the point of work, in the trigger comments
and the stub-inventory test. ⛔ This is not a hidden over-claim; it is a declared
one, and the declaration is what makes it fixable.

## The blocking dependency — ⭐ `V3` **AND** `V4`, and the two carry different halves

⚠ **Two premises about this were wrong, in opposite directions, and both are
corrected here.** It was first recorded as blocked on `V3` alone; the source's
own labels say `V4`; ⛔ **neither alone is right.**

`prover.rs` carries **seven** `[placeholder — reifies in V4]` markers and **zero**
naming `V3`:

| route | what is deferred | named target |
|---|---|---|
| `attempt_d` | kernel `whnf` + decision procedure (`23 §3.1`), Z3-backed arithmetic search + `Decidable` constructor extraction (`23 §3.2`) | **`V4`** |
| `attempt_fo` | the Kripke embedding `φ ↦ φ#`, `World` sort, adequacy lemma `classically_valid(φ#) → φ`, `check_cert` soundness (`23 §4`) | **`V4`** |

**But the DAG (`docs/program/05-implementation-dag.md`) splits the work across
two WPs, and the split is meaningful:**

| WP | DAG scope | the half it supplies |
|---|---|---|
| **`V3`** | the **prover** (frame `V3-prover.md`, plus `V3-z3-throughput-evaluation.md`) | a route that can **reach a refutation** at all — the decision procedure and the Z3-backed search behind `attempt_d`/`attempt_fo` |
| **`V4`** | proof-failure **diagnostics** (`24`); DAG row `:167` lists its deliverables as ***"countermodels, holes, `unknown`"*** | the **`Countermodel`** a `Disproved` verdict must carry |

⇒ ⭐ **`Verdict::Disproved { countermodel }` needs both halves**: `V3` to decide
that `φ` is refutable, and `V4`'s countermodel machinery (a Kripke model forcing
`¬φ` at some world, `24 §1`) to be the payload. **`AC-R3` sequences after both,
and no amount of `ken-elaborator`-local work reaches it.**

⚠ **Why the source's labels read `V4` even for the `attempt_d` arithmetic search:**
those placeholders are written from the *verdict's* point of view — what they are
missing is the thing that lets a route answer "refuted" with evidence. ⛔ Do not
read the seven `V4` markers as evidence that `V3` is not also required; read the
DAG for sequencing and the markers for what is absent.

⚠ ⛔ **Neither `V3` nor `V4` has a tracker node**, so `depends_on` is `[]` — that
is a schema limitation, ⛔ **not** an assertion that nothing blocks this. The
blockers are stated here in prose and must be read as binding.

## Acceptance criteria — ⛔ apply only AFTER the backend lands

| AC | claim | control |
|---|---|---|
| `AC-R3a` | A production path can return `Verdict::Disproved` with a real `Countermodel`. | ⛔ **Re-run the census above and require it to change**: `Verdict::Disproved {` must appear as a **construction** in `crates/*/src/`. ⚠ Keep the test-tree positive control so the census cannot pass by a broken key |
| `AC-R3b` | `product(c, ζ)` exists — variable renaming, `lowEq_ζ`, the `coterminates_ζ` conjunct — and `D5` is tied to a genuine product-program reduction. | ⛔ `synthetic_disproved` must no longer be on `D5`'s path. A `Disproved` reaching `check_reduction_faithfulness` must originate in the prover |
| `AC-R3c` | ⭐⭐ **A too-weak `Φ_post` is DETECTED.** | Construct one deliberately; `D5` must report failure rather than a false pass. ⛔ This is the row the whole node exists for — a verdict-shape assertion cannot discharge it |
| `AC-R3d` | `[Sec1-reduce]` is removed from the deferred set **exactly** where it is reified. | `n1_n2_stub_gaps_carry_reify_triggers` is **updated, not deleted** — the trigger moves from "NOT yet delivered" to delivered. ⚠ `[Sec1-dual]`/`[Sec1-launder]` were already removed by PR #1094; ⛔ do not re-add them |
| `AC-R3e` | The seed's refutation-expecting rows are re-graded against a real backend, and any that were passing synthetically are named. | ⛔ Report which rows **changed evidence** without changing verdict. A row that was green synthetically and is green really is the case most likely to be missed |
| `AC-R3f` | `export.rs:435`'s refusal arm is exercised by a **real** refuted obligation. | ⛔ Until then it must not be described as tested. A guard no input reaches is not covered by the suite passing |

## Scope

**IN:** `crates/ken-elaborator/src/ifc.rs`'s reduction/faithfulness path and the
`sec1_acceptance` controls over it.

⛔ **OUT:**
- ⛔ **Building the prover backend itself** — that is the `V4` WP. If this node
  appears to require writing the Kripke embedding, it is **not ready**; stop.
- ⛔ `AC-R1`/`AC-R2` — **landed** in PR #1094 (`main` tree `8229a811`).
  Verified by discriminating control: `TRIGGER_SEC1_DUAL` and
  `TRIGGER_SEC1_LAUNDER` are **0** in `ifc.rs` on `main` while
  `TRIGGER_SEC1_REDUCE` is **1**.
- ⛔ **No kernel enlargement.** `proved` must remain believed **only** because
  `check(env, Γ, cert, φ)` accepts (`23 §1.5`); a refutation backend must not
  become a second trust root. If it seems to need one, **stop and re-raise** —
  that is a finding about the spec's premise.
- ⛔ Sec1ct / Sec2 / Sec4 / Sec5.

## Validation — ⛔ TARGETED ONLY

⛔ **NEVER `--workspace`** (operator, `agent/COORDINATION.md §12`).
`-p ken-elaborator --test sec1_acceptance`. Workspace, `--locked`, and
conformance run **in CI**.
