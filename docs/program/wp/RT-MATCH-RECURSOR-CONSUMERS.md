# RT-MATCH-RECURSOR-CONSUMERS — complete the Position A consumer repair

Owner: Runtime. Size: **M, provisional** — see Sizing.
Authority: Architect partition `evt_3r4j14fv1jtj2` (2026-08-08) on the
nine-expression census `evt_16cmej481q7ns`.

**Read `docs/program/16-recursive-descent-retirement.md` first** — the campaign
context and the five traps that bind every node in this arc. Trap 1 and Trap 2
are both live here.

> # WHAT THIS NODE IS FOR, IN ONE SENTENCE
>
> [[RT-RECURSOR-TRANSPORT]]'s `D2` closed **one** Position-A witness and its
> record said it had closed **the position**; `d8d` is a counterexample
> reachable at that same object, and this node closes the population for real.
>
> **The `D2` production mechanism is sound and is not being reverted.** It
> correctly closes the exact `D1` A witness at the exact
> `resume_active_continuation` seat. What was wrong was the *scope claimed for
> it*.

## 1. Fixed inputs

**Measure every one of these yourself at your pinned base.** The values below
were measured by the implementer at exact `D2`
`8efdfdb3fb39fc6e66708635cdf11269758d77ed` and are **anchors to re-find, never
values to trust** — your base is later than that by construction, because the
`D2` record correction lands first.

| input | as measured at `8efdfdb3` |
|---|---|
| the witness | `d8d` composed binding site |
| its complete residual set | **exactly `{MatchScrutineeRecursor}`** — never contained `LexicalCallArgumentRecursor` |
| lane, unexcluded | `RecursiveDescent` |
| lane, **A**-only exclusion | **`FunctionizedUnits`** |
| activation denominator | `Some({MatchScrutineeRecursor})`, a real compile through `px8j_capture_source_trace` |
| unexcluded compile outcome | `Ok` — the fixture was not already broken |
| refusal under A-only exclusion | `Unsupported(UnsupportedLowering { construct: "RecursiveBackedge", reason: "protocol machinery is never a source value at a boundary" })` |
| B-only exclusion on this row | **inapplicable** — the hook's own `debug_assert` refuses removing a variant that is not in the set |

## 2. What is owed

**A repair of the `MatchScrutineeRecursor` consumer population on the
functionized lane, proven on the pre-retirement tree.**

Not owed, and banned: any deletion of the `RecursiveDescent` lane, selector,
enum or authority (that is [[RT-DESCENT-RETIRE]]); any retirement of a residual
variant (that is [[RT-RECURSOR-TRANSPORT]] `D3`); any work on rows 1-5 (that is
[[RT-LEXICAL-RECURSOR-CONSUMERS]]).

## 3. Deliverables

### `D0` — close the population from the production predicate

**The population is defined by the production `MatchScrutineeRecursor`
predicate. `d8d` is a floor, not the perimeter.**

Sweep every compilation entry that can supply that predicate. Record every
firing fixture and every same-family green control.

**Helper spelling, snake_case fixture spellings and
`BodyEmissionAuthority::RecursiveDescent` assertions are candidate selectors,
not closure.** A grep tells you which fixtures might be in the family; it never
tells you what any one of them enumerates. **This node exists because exactly
that inference was made twice** — a class-wide claim from one green witness,
then a shared-helper claim over a fixture nobody opened. Do not make it a third
time: **enumerate, per fixture, by measurement.**

### `D1` — activate and attribute, before any repair

Under **A-only exclusion** — the existing one-variant hook, used as designed:

- each firing row reproduces its **exact first refusal**;
- the ordinary retained run stays **green**;
- at least one same-family control that already works on `FunctionizedUnits`
  stays green as a **positive control**;
- **exact activation denominators are recorded**, so a refusal cannot be
  credited when the selector or harness never reached the path.

**Then trace each red to the first missing or mis-consumed static fact, and name
its owner.** Partition by correlated continuation owner/origin, pending suffix,
operand phase/kind, source-machine/composed consumer seat, and boundary reached.

**A rendered refusal string is a symptom, not a cause**, and a shared refusal
string is not proof of a shared mechanism.

### `D2` — repair only the proven root boundary or boundaries

Reuse planner-owned continuation specialization / call identity and ordinary
typed-value transport **where they already name the edge**. Do not infer one
mechanism from shared syntax.

**The lawful shape of the fix:** make the protocol or fact get **consumed or
represented at its owner, before the guard**. Never teach a downstream guard to
accept a forbidden state.

### `D3` — discriminating controls

Every row `D0` found stays **enabled and unchanged in meaning**, and runs green
under A-only exclusion at the pre-retirement base. **A mutation at each repaired
root recreates the attributed refusal while proving the detector was reached.**
Unaffected same-family controls stay green.

> ### `D3` OWNS GIVING THE `D2` COUNTERS A CONSUMER — THEY CURRENTLY HAVE NONE
>
> **Added 2026-08-08 from an Adversary finding on merged `3061a645`; no
> correctness defect, and the sequencing is not being reversed.**
>
> The accepted partial landed four `#[cfg(test)]` accessors and a mutation knob
> — `mrc_d2_backedge_arms_seen`, `mrc_d2_inert_words`, `reset_mrc_d2_counts`,
> `set_mrc_d2_suppress_inert_word` — with **zero callers anywhere in the repo**.
> No test reads either counter, nothing ever sets the suppression, so the
> `if !suppress_inert_word` guard has one reachable side. **No lint catches
> this**: `dead_code` does not fire on `#[cfg(test)]` items at that visibility,
> and CI is green.
>
> **The risk is citation, not correctness.** The denominator counter carries a
> careful anti-vacuity rationale — counted *before* the representation arm, so
> suppression cannot drive it to zero and "no inert word" reads as "the arm
> declined" rather than "the seat was never reached." That reasoning is right
> and currently **inert**. A future reader greps the counter, finds a documented
> denominator with an explicit anti-vacuity argument beside it, and reasonably
> concludes the property is **measured**. It is only **measurable**, and only
> the first is what a reader takes away.
>
> ⇒ **`D3` either gives the accessors their consumer — which it needs anyway —
> or states at the declaration site that they are unread until it does.** Either
> discharges this; leaving them silent does not.

## 4. Acceptance criteria

- **AC-1 — the population is closed by measurement, not by grep.**
  *Control:* the handback enumerates each fixture in the production-predicate
  population with its complete residual set, and states which selectors were
  used as candidates. A grep list alone does not discharge this.
- **AC-2 — every repaired root has a committed discriminating control.**
  *Control:* the control reds under a mutation at that root and greens without
  it, **from the committed tree**, with evidence the detector was reached. A
  hand-run mutation does not discharge this.
- **AC-3 — the `RecursiveBackedge` guard is intact.**
  *Control:* `RecursiveBackedge` remains protocol-only; a committed negative
  witness shows it is still refused as a source boundary value, with a positive
  control proving that path is reached.
- **AC-4 — no banned mechanism.** No fallback to `RecursiveDescent`, no
  `BoundaryUse`, no `PlannedEffectSeat` widening, no lowering-minted token, and
  no invocation-local activation/resume/return-hole state in ABI data.
  *Control:* name the ABI payload at each new crossing and show its fields are
  ordinary typed values; `BoundaryUse` stays at zero production hits.
- **AC-5 — zero new `#[ignore]`, anywhere in this lineage.**
  *Control:* `git diff` on the candidate contains no added `#[ignore]`.
- **AC-6 — no retirement and no lane deletion in this candidate.**
  *Control:* both residual variants, the classifier insertions, the collector
  insertions and the per-variant exclusion hook are **present and unchanged** at
  the final SHA.
- **AC-7 — the candidate contains NO tracker `status:` change.** The flip is the
  Steward's, post-merge.
  *Control:* `git diff` over `docs/program/issues/` is empty of `status:` lines.
- **AC-8 — CI green** on the merge. Not a local `--workspace` run, which is
  banned (`COORDINATION §12`).

## 5. Banned scope

- **No `#[ignore]`.** Quarantine was ruled out for this population and the
  ruling was not reopened.
- **No reshaping of a fixture, and no absorption of a refusal**, to make a row
  pass.
- **No simultaneous exclusion of both variants, and no generalized hook** — that
  fabricates a lane no program has.
- **No reinterpreting a retained `RecursiveDescent` run as activation.**
- **No touching rows 1-5** or the `LexicalCallArgumentRecursor` population.
- **No resume or cherry-pick of `10369776252861e8b15e613576256a3682c70066`** —
  held evidence only.

## 6. Hard stops

**Return the partition before coding** if either fires:

1. **`D1` finds materially distinct authorities** rather than downstream
   symptoms of one root.
2. **Any repair needs a new planner or ABI population.**

**And a third, specific to this node's relationship with its sibling:** if the
causal partition here and [[RT-LEXICAL-RECURSOR-CONSUMERS]]'s appear to share
one exact production root, **route a subsumption proposal — do not fold.** A
shared root may not be inferred from shared retirement timing or shared syntax.

> ### FOURTH HARD STOP, added by the Steward on the `D0`/`D1` checkpoint `bcf3218b`
>
> **If closing both rows requires BUILDING the reduced-predecessor merge shape,
> stop and return before building it.**
>
> `D1` attributed the owner to **`carried_join_arm`** in `core.rs` — the
> function name is the handle, and it is the only handle this frame states,
> because `D2` moves that file and any line number here rots on contact. (`D1`
> first reported `core.rs:10842`, read off an instrumented worktree that
> displaced the file by a uniform +78; corrected to **`10764` at `89aa1550`** in
> `cf70db86`, with the attribution unchanged.) `D1` flagged
> that its existing `Trap` arm documents that shape as **not built by this
> route**. `Trap` can decline it because `Trap` **refuses**; a `RecursiveBackedge`
> arm has to **succeed**, so it cannot borrow that escape.
>
> **This is a checkpoint, not a ban.** I am not forbidding the shape — I am
> refusing to let it be discovered mid-implementation and absorbed silently into
> an `M`. Building a new control-flow shape in the lowering route is a component-
> design call that belongs to the Architect and a sizing call that belongs to me,
> and neither is dischargeable from inside a coding turn.
>
> **If the repair closes both rows by re-routing or re-representing the operand
> at `carried_join_arm` — the lawful shape `D2` already names — no stop fires
> and no routing is owed.** That is the expected path. Post the exact SHA and
> keep going.

## 7. Base

**Post-`D2`-record-correction `main`.** Not `8efdfdb3` — that object's record
overclaims and it is not landing as-is. Not `10369776`, ever.

Cut `wp/RT-MATCH-RECURSOR-CONSUMERS` from `origin/main` after the correction
merges. Keep **both** residual variants and the per-variant exclusion hook for
the whole node.

## 8. Contention

`lowering/core.rs` and `core/tests/control.rs` — the same two files as
[[RT-LEXICAL-RECURSOR-CONSUMERS]] and [[RT-RECURSOR-TRANSPORT]] `D3`. **All
three contend, which is why they are serialized rather than parallel.** This
node is first of the three.

## 9. Sizing

**`M`, provisional, and the provisional part is real.** `d8d` is one measured
expression; `D0` may find the A population materially wider, and `M` is a
scoping figure taken from a symptom count.

Checkpoints, exact SHA posted at each: `D0` population closure · `D1` activation
and causal partition · `D2` repair · `D3` controls.

**Post the `D0`/`D1` outcome as its own checkpoint before starting `D2`** —
that is where the Steward re-sizes if needed.
