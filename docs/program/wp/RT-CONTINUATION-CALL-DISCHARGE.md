# RT-CONTINUATION-CALL-DISCHARGE — a planned call nobody discharges

**Once the `Active` resume path goes live, `ContinuationClaimLedger::close`
refuses: one planned causal token is neither directly emitted nor
compositionally consumed. Every measured member has `pending_len=0`, so the
resume discharges nothing. This node decides WHO OWNS the discharge — it does
not assume.**

**Owner:** Team Runtime. **Branch:** `wp/RT-CONTINUATION-CALL-DISCHARGE`.
**Size:** `M` — **provisional; `D0`/`D1` may overturn it and is expected to.**
**Risk:** medium-high — the classification chooses between a planner-authority
correction, a producer/call-seat repair, and an evidence-plumbing repair.

**Read `docs/program/16-recursive-descent-retirement.md` first.** This node
exists because of that campaign's **Trap 2**, and this frame does not repeat the
traps.

> ### THIS FRAME DELIBERATELY DOES NOT NAME THE REPAIR
>
> The Architect ruled the planner **implicated but not convicted**
> (`evt_vxqa83y4z3nt`). A frame that named a repair here would be the fifth
> instance on this chain of reading a small-witness result as a class-wide
> property. **The deliverable is a classification backed by a trace**, and the
> repair follows from it.

---

## 1. Fixed inputs

**Measure every one yourself at your pinned base.** These are **anchors to
re-find, never values to trust** — your base is later than this frame by
construction, and this campaign moves the very files involved.

**Cite by grep-able phrase, not by line number.** Line coordinates on this chain
have rotted between a handback and its own review twice, most recently within a
single merge.

| input | anchor |
|---|---|
| the refusal | `"the discharged continuation call population is not the planned one"`, in `ContinuationClaimLedger::close`, `lowering/units.rs` |
| the law | `planned = direct-emitted ⊎ composed-consumed`, checked as **exact set equality**; `close` separately refuses an identity in **both** sets |
| the identity | `ContinuationCallIdentity` in `planning/static_transition.rs` — `token` plus `recursive_position`, exposing `target()` and `producer_owner()` |
| the `composed` feed | fed from `function_local.composed_discharges` **and nothing else**; the direct gate requires the recorded instruction to decode to `identity.target()`, and a composed instruction targets the **raw worker** |
| the `emitted` feed | accumulated per generated function **after that function's CLIF has been checked**, not at call-construction time |
| the projection note | `ContinuationClaimLedger::open` records that `planned == resolved` is **structural today** because `resolve_continuation_targets` walks the same projection |
| your base | **not fixed here.** Branch from `main` after [[RT-SPECIALIZED-ACTIVE-RESUME]]'s accepted partial lands, and pin it in your first checkpoint post |
| the end-state control | held evidence `65639a13` on `runtime-implementer/sar-lane-pair-evidence` — **read it, do not publish it** |

## 2. What is owed

### `D0` — trace the missing identity through BOTH lanes

Trace the **exact** missing `ContinuationCallIdentity` — construct origin,
continuation origin, alternative, recursive position, call-site sequence,
target, and emission owner — **through the same program in both lanes.**

> ### THE RETAINED-LANE CONTROL THIS FRAME PROMISED DOES NOT EXIST — MEASURED
>
> Steward, corrected 2026-08-08 at `evt_jzrg3nq5pggd` on the `D0` return.
>
> This frame claimed the retained lane's disposition for the identity was *"the
> single most informative fact available, and free because both lanes already
> run."* **`D0` measured it: the retained lane never opens a claim ledger for
> this program.** No `OPEN`, no `LEDGER`, compile `Ok`. The identity is not
> discharged directly, not compositionally, and **not planned** there.
>
> **The reasoning error was inferring from *both lanes run the same program* to
> *both lanes traverse the same mechanism*.** They lower by different
> strategies, so different causal populations are expected. **The shared input
> is the program, not the ledger** — and a lane pair is a control only over
> machinery both lanes actually enter.
>
> ⇒ **Any argument of the form "the retained lane discharges it as X" has
> nothing under it.** It is equally **not** evidence the activated plan is
> wrong.
>
> **What made this visible:** an `open` census, added because `close`
> structurally cannot distinguish *not planned* from *never reached*. Keep that
> instrument; the distinction is load-bearing for option 3.

Denominators, one disposition per arrival, zero orphans, and **committed
controls excluded from the denominator and named**. The **two independent A rows
are the floor, not the perimeter.**

**Do not extend the cross-crate census.** [[RT-SPECIALIZED-ACTIVE-RESUME]]
established that the activation seam is `#[cfg(test)]`, so a cross-crate run can
only ever witness the retained lane. That question is **retired**; re-running it
is not evidence-gathering, it is rediscovery.

### `D1` — classify EXACTLY ONE of three, with the other two refuted

1. **A real direct obligation was skipped.** Repair the actual producer/call
   seat and **retain finished-CLIF verification.**
2. **A real composed consumption occurred but its evidence was lost.** Restore
   the verified composed relation. **Do not claim it from the resume.**
3. **The activated path has no causal call obligation.** Correct the planner's
   issuance/projection **at planner authority**, proving why this exact path is
   not a member. **`pending_len == 0` alone does not establish this.**

**State the evidence against the two you did not pick.** A classification that
only argues for its choice is a preference.

**Option 3 is not a free relabelling.** Because `planned == resolved` is
structural today, a projection-level correction **moves the set `close` checks
against** — so it must be argued at planner authority and measured, not asserted.

### `D2` — the repair, at whichever authority `D1` named

Bounded to the classified authority. **Hard stop and route** if the repair would
require any of the following; all four are Architect-forbidden:

- discharging the token in the empty resume;
- weakening the set-equality law, or the both-sets refusal;
- bulk-claiming the token;
- manufacturing a composed discharge, or treating an identity return as a call.

### `D3` — the end-state control

**The activated lane compiles `Ok` on the program the retained lane compiles
`Ok`.** That is the assertion held at `65639a13`, and this node inherits it as
its acceptance control.

**Read that as compilation, not as discharge parity.** `D0` measured that the
retained lane never opens a claim ledger for this program at all, so there is no
retained-lane disposition for this identity to agree with. The corrected
statement of both `AC-5` and `AC-6` is in §3.

**Assert the mechanism, not merely the absence of the refusal.**
[[RT-CARRIED-ORDINARY-COMPOSITION]]'s `D3` established the trap: asserting the
absence of the refusal you just repaired reads as exactly the right test, and is
worthless when the repair deleted that refusal from production. Check whether
your own `!contains` is free **before** you write it.

**Exclude this control from population evidence.** A control that is a member of
the population it observes proves the hook is reachable, not that a program has
the shape.

**Do not assert cross-run count equality between the normal and mutated runs.**
That is the defect the Architect retired in `coc_d3` and in `sar_d3`: once
lowering lawfully continues, the normal run revisits the hook while the
suppressed run stops at its first restored refusal. Equality asserts the repair
has **no downstream effect**. Assert `> 0` for the mutated denominator and
**equality only where the control owns the relation**.

## 3. Acceptance criteria

| AC | criterion | control |
|---|---|---|
| `AC-1` | The missing identity is traced in both lanes, all seven fields recorded | `D0` record, with denominators and excluded committed controls named |
| `AC-2` | Exactly one of the three classifications is chosen, with the other two refuted on evidence | `D1` record |
| `AC-3` | The repair sits at the authority `D1` named, and nowhere else | diff scope, stated against **both** the merge base and the `D0`/`D1` checkpoint |
| `AC-4` | The ledger law is unchanged: exact set equality, both-sets refusal intact, `composed` still fed only from `function_local.composed_discharges` | verbatim check at the three sites |
| `AC-5` | The activated lane **compiles `Ok`** on the program the retained lane compiles `Ok` | the `65639a13` assertion, committed green. **Not "discharges the same identity"** — the retained lane discharges nothing here |
| `AC-6` | The repair does not make the retained lane acquire a ledger obligation or stop compiling | lane-pair, both directions. A **guard**, not a discriminator |
| `AC-7` | `D3` is non-vacuous on this candidate — suppressing the mechanism reds it | mutation, with the reached-count proven `> 0` |
| `AC-8` | No `#[ignore]` added; `issues/` untouched; the five landed repairs intact | mechanical |
| `AC-9` | Workspace green **in CI** | CI, never a local `--workspace` run |

**`AC-5` and `AC-6` are the pair, and `AC-6` is the one that can fail
silently.** A repair that makes the activated lane close by changing what the
retained lane does has not validated the campaign premise — it has hidden the
divergence.

**Both were re-stated on the `D0` return** (`evt_jzrg3nq5pggd`), because the
retained lane turns out never to open a ledger for this program. `AC-6` survives
as a **guard** — it still catches a repair that gives the retained lane an
obligation it does not have — but it is **not** the discriminator this frame
originally claimed, and it must not be cited as one.

**The genuine free control `D0` produced instead** is the 213-identity
population: `DIRECT 170`, `COMPOSED 34`, and — with all nine
undischarged-or-double rows identified as committed controls — **no independent
program reaching `close` undischarged**. Both discharge forms are well
populated, so neither mechanism is exotic and this identity is the outlier.

## 4. Contention

**None expected.** Runtime is single-threaded and this is the only active
Runtime node once [[RT-SPECIALIZED-ACTIVE-RESUME]]'s partial lands.

- `lowering/units.rs` and `planning/static_transition.rs` are this node's
  primary surfaces and no other ready node names them.
- [[RT-FNUNIT-RESULT-TOKEN]] is `ready` and queued behind this chain; its pinned
  surfaces are `cranelift_backend/surface.rs`, `compiled.rs` and
  `artifact/api/tests.rs` — **disjoint**, but re-check at release rather than
  trusting this line.
- The doc track runs concurrently by standing exception and touches `library/`
  and `agent/`, not `crates/`.

**Re-run the check yourself at kickoff.** A contention statement written at
framing time is a claim about a tree that no longer exists.

## 5. If the classification does not close

**Route it, do not absorb it.** Six walls on this chain have each been a distinct
authority, and every one was resolved by routing rather than by widening the
node in front of it. A seventh is a normal outcome, not a failure — and the
campaign's own record is that the expensive mistake has always been treating a
new authority as a defect in the previous repair.
