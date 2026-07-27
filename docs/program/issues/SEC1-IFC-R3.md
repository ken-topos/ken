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

> ## ⛔ STATUS IS `draft` DELIBERATELY — STILL NOT RELEASABLE, BUT THE BLOCK HAS MOVED
>
> ⛔ **Do not release this to Team Verify yet.** ⚠ **But the premise below is now
> PARTLY STALE and must be re-derived before anyone acts on it.**
>
> ### ⭐⭐ MEASURED UPDATE 2026-07-27 — `V3-RESIDUAL` merged (PR #1103)
>
> At `main = 7725272c`.
>
> ⛔ **The central claim of this node — *"No production code path can return
> `Verdict::Disproved`"* — IS NO LONGER TRUE.** Verified on `main`:
>
> ```
> crates/ken-elaborator/src/prover.rs:220
>     Ok(()) => Verdict::Disproved { countermodel },
> ```
>
> in `attempt_with_refutation`, gated on `check(env, ctx, &refutation, &not_φ)`
> where `not_φ = φ → Bottom`. ⭐ **The cardinal rule (`23 §1.5`) is intact** — a
> refutation is believed only because the **kernel accepted `q : φ → Bottom`**;
> an invalid refutation yields an honest `Unknown`. ⇒ The backend did **not**
> become a second trust root, which was `AC-R3d`'s / `AC-V3r4`'s whole worry.
>
> `disproved_carries_countermodel` is now a **real** test: it asserts the
> countermodel names the failing input class, carries both failing inputs, and
> that `trusted_base()` is **unchanged** by a checked empty-context refutation.
>
> ⇒ ⭐ **`AC-R3a` is now satisfiable and must NOT be re-derived as blocked.**
>
> ### ⛔ What is still owed — re-scope to THIS before releasing
>
> ⚠ **Do not read "a `Disproved` exists" as "`Sec1`'s reduction is unblocked."**
> The landed arm refutes an **`Int`-literal disequality**. `AC-R3b` needs
> `product(c, ζ)` — variable renaming, `lowEq_ζ`, the `coterminates_ζ` conjunct —
> and `AC-R3c`, ⭐ **the row this whole node exists for**, needs a deliberately
> too-weak `Φ_post` to be **DETECTED**. ⛔ Neither follows from a literal
> refutation.
>
> ⇒ **Before release, someone must measure the residual**: which of `AC-R3a`–`R3f`
> the landed refutation arm actually reaches, and how much of `V4`'s countermodel
> machinery (`24 §1`) the landed `Countermodel` type genuinely supplies versus a
> description string. ⛔ **Until that measurement exists, this stays `draft`** —
> and it is a *measurement*, not another ruling.
>
> ⚠ **`SPEC-PROGRESS.md` cannot answer this** (47/48 rows `DRAFT`, `REVISED`
> never used). Measure the code.

> ## ⭐⭐ THE MEASUREMENT WAS TAKEN — 2026-07-27, Steward, at `main = d6df571e`
>
> Both `V3-RESIDUAL` and `V4-RESIDUAL` are now **merged** (`V4-RESIDUAL`
> confirmed by `merge-tree` against `origin/main`: identical tree
> `2d9153c3`). ⛔ **That does NOT unblock this node**, and the reason is
> narrower and more decisive than the "blocked on V3 AND V4" prose above.
>
> ### `AC-R3a` — ✅ SATISFIED, and the census it prescribes now passes
>
> `crates/ken-elaborator/src/prover.rs:264` constructs
> `Verdict::Disproved { countermodel }`, and there **is** a production route:
> `attempt_obligation → attempt_d (:281) → attempt_with_refutation (:305)`.
> ⭐ The cardinal rule (`23 §1.5`) is intact — the verdict is returned only
> after the kernel accepts `q : φ → Bottom`; an invalid refutation yields
> `Unknown`. The backend did **not** become a second trust root.
>
> ### ⛔ `AC-R3b` / `AC-R3c` — UNREACHABLE, and the census cannot see it
>
> ⭐⭐ **This is the finding that matters.** `attempt_d`'s refutation arm is
> gated on exactly one syntactic shape (`prover.rs:298-300`):
>
> ```rust
> if let Term::Eq(_, lhs, rhs) = phi {
>     if let (Term::IntLit(left), Term::IntLit(right)) = (lhs.as_ref(), rhs.as_ref()) {
>         if left != right {   // ← the ONLY production route to Disproved
> ```
>
> Every other `φ` falls through to `emit_unknown_hole` at `:320`.
>
> ⇒ **A `product(c, ζ)` faithfulness obligation is not an `Int`-literal
> disequality.** Building `product(c, ζ)` would make `D5`'s
> `check_reduction_faithfulness` receive `Unknown`, not `Disproved` — so the
> WP would *fail*, not deliver. ⛔ `AC-R3c` (a too-weak `Φ_post` must be
> DETECTED) is the same: detection **is** a refutation of the faithfulness
> obligation, and no route produces one.
>
> ⚠ **`AC-R3a`'s control is a census of a spelling, and it now passes while
> the property it stands for is still false for every obligation this node
> actually cares about.** A "`Disproved` construction exists in `src/`" is
> true and does not entail "`D5` can be fed by the prover."
>
> ### ⛔ The ACs transitively require this node's own EXCLUDED scope
>
> `## Scope` bans *"building the prover backend itself."* But `AC-R3b`/`R3c`
> cannot be discharged without exactly that. ⇒ **The frame is unsatisfiable as
> written** — the same ban-vs-AC intersection that blocked two WPs on
> 2026-07-27, at whole-node scale.
>
> ### The real blocker, named precisely
>
> `prover.rs:317-318` — *"[placeholder — reifies in V4]: kernel whnf +
> decision procedure (`23 §3.1`) + Z3-backed arithmetic search + `Decidable`
> constructor extraction (`23 §3.2`)"*. Seven such markers remain.
>
> ⛔⛔ **`z3` is not a dependency of this workspace at all** — zero hits across
> the root and every crate manifest. The DAG's `V3` row (`05` `:166`) names
> *"classifier + Z3 + Kripke embedding + reflective certificate."*
>
> ⇒ **Adding an SMT backend is a product/architecture decision (build, CI,
> licensing, throughput — the DAG already names
> `V3-z3-throughput-evaluation.md`), not a call this node can make.**
> ⭐ **It is escalated to the operator and it gates the entire by-proof half
> of `Sec1`.**
>
> ### ⛔ And the Z3-free widening is VACUOUS — do not frame it
>
> ⚠ Recorded so nobody re-derives it as available work. `attempt_d` hardcodes
> `Term::IntLit` where the **kernel is already generic**: `obs.rs:84` gates on
> `env.deceq_cert(*id).is_some()` and says in-source *"general opt-in gate …
> not hardcoded to any specific primitive."* But
> `declare_deceq_certificate` has **exactly one caller** —
> `crates/ken-elaborator/src/numbers.rs:363`, registering `Int`.
>
> ⇒ Generalizing the prover off `IntLit` has **no second registered type to
> generalize to**. It would change no observable behavior and produce a green
> WP over nothing. ⛔ Not work.

## What happened

`SEC1-IFC` (PR #1094) delivered `AC-R1` (`[Sec1-dual]`) and `AC-R2`
(`[Sec1-launder]`). `AC-R3` (`[Sec1-reduce]`) hit an authorized hard stop.

⭐ The frame anticipated this and said so: *"if `AC-R3` turns out to need prover
work that is not present, land `AC-R1`+`AC-R2` and re-raise `AC-R3` to the
Steward with what you measured — ⛔ do not stub it further."* That is exactly what
happened, so `[Sec1-reduce]` **correctly remains live** in `ifc.rs` and in the
suite's stub inventory. ⛔ It is an authorized deferral, ⛔ not a silent
completion claim.

## ⚠ HISTORICAL — the measurement AS IT STOOD AT `4d15002d`. ⛔ SUPERSEDED.

> ⛔ **This whole section is a historical record, not a live claim.** Its central
> assertion was **falsified** by `V3-RESIDUAL` (PR #1103, `main = 7725272c`) —
> see the correction block at the top of this node. It is retained because the
> *reasoning* about what an unreachable verdict costs is still the reason the
> node exists, and because the census method below is the one to re-run.
>
> ⛔ **Do not quote any sentence from this section as current state.**

Measured at `origin/main = 4d15002d`.

**No production code path can return `Verdict::Disproved`.** *(⛔ FALSE as of
`7725272c` — `prover.rs:220` constructs it.)*

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
