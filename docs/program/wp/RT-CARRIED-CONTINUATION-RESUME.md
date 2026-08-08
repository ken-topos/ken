# RT-CARRIED-CONTINUATION-RESUME — the carried continuation-frame resume path

Owner: Runtime. Size: **M, provisional** — see Sizing.
Authority: Architect sibling-authority ruling `evt_2pt95vbja6447` (2026-08-08)
on the `RT-MATCH-RECURSOR-CONSUMERS` `D2` hard stop `evt_397gfxdg45ncs`.

**Read `docs/program/16-recursive-descent-retirement.md` first** — the campaign
context and the five traps. **Trap 1 and Trap 2 are both live here**, and Trap 2
is why this node exists at all.

> # WHAT THIS NODE IS FOR, IN ONE SENTENCE
>
> The previous repair **worked**, and the refusal moved further in: a carried
> scrutinee now reaches a continuation frame that has **no resume path for a
> carrier value**, and this node builds one — or proves the pairing invalid.
>
> **Neither landed repair is reopened.** [[RT-RECURSOR-TRANSPORT]]'s `D2` at
> `resume_active_continuation` is sound. [[RT-MATCH-RECURSOR-CONSUMERS]]'s `D2`
> at `carried_join_arm` is correct and lands separately as an accepted partial.
> **You are building on both, not revisiting either.**

## 1. Fixed inputs

**Measure every one of these yourself at your pinned base.** They were measured
at `50808c11dcb6f054bfe02dd38f84f251e9638257` and are **anchors to re-find,
never values to trust**.

| input | as measured at `50808c11` |
|---|---|
| exposed rows | **2** — `d8d_the_composed_binding_site_...` and `px8j_all_three_producer_paths_reach_real_consumers` |
| activation seam | **A**-only exclusion, the committed one-variant hook |
| refusal | `Unsupported(BoundaryCarrier, "a carried scrutinee reached a continuation frame that resumes a compile-time value rather than eliminating one")` |
| owner | **`lower_computational_match_value_composed`**, `Active`/`PendingLet` arm |
| operand | `LoweringOperand::Carried(word)` — a **live dynamic value** |
| prior refusal | `RecursiveBackedge` is **gone from both rows** — that absence is the proof `carried_join_arm` was reached |
| retained suite | **816 / 0 / 4**, the no-delta baseline |

**Coordinates: use function names.** At `50808c11` the owner was `core.rs:3667`
and its arm `core.rs:3793`. **This node moves that file**, so any line number
written here rots against its own deliverable. Re-derive by name at your base
— and if you record a trace, re-derive it **after** reverting instrumentation,
because instrumentation displaces the file and a displaced coordinate still
resolves.

## 2. What is owed

**A resume path for a carried scrutinee meeting a continuation frame — or a
proof that a given frame/value pairing is invalid and a lawful refusal for it.**

Both outcomes are legitimate. The Architect's framing is *"resume a pending
computation over a carrier value, **or prove that frame/value pairing
invalid**."* Do not assume every reachable pairing must succeed.

Not owed, and banned: reopening [[RT-RECURSOR-TRANSPORT]] `D2`; reverting or
amending the `carried_join_arm` repair; any work on rows 1-5
([[RT-LEXICAL-RECURSOR-CONSUMERS]]); any retirement or lane deletion.

## 3. Deliverables

### `D0` — close the population from the production arm

**The population is the production arm `Carried(word)` x first eliminator
`{PendingLet, Active}` in `lower_computational_match_value_composed`. The owner
is the carried continuation-frame consumer, not either discovering fixture.**

**The two exposed rows are a floor, not the perimeter.** This campaign has twice
read a small-witness result as a class-wide property, and both times the
correction cost more than the census would have. Enumerate from the production
arm, per fixture, by measurement.

**Census both frame variants separately** and record the denominator, including
compilations that never reach the arm. A census that records only firing rows
cannot distinguish a closed population from a short one.

### `D1` — partition `PendingLet` from `Active`, before any repair

**A shared refusal arm is NOT evidence that `PendingLet` and `Active` require
one mechanism** (Architect, explicit). Treating the shared arm as a shared cause
is the exact inference this campaign has paid for repeatedly.

Under **A**-only exclusion: each firing row reproduces its exact first refusal;
the retained run stays green; a same-family positive control that already works
stays green; and **activation denominators are recorded**, so a refusal is never
credited to a path the selector did not reach.

Then trace each red to the **first missing or mis-consumed static fact** and
name its owner, **per variant**. Partition by owned fact, operand phase,
required action and fail-closed boundary — the four axes on which the Architect
separated this authority from its sibling.

### `D2` — repair only the proven roots

Consume or represent the fact **at its owner, before the guard**. Never teach a
downstream guard to accept a forbidden state.

**Prefer mirroring a landed representation over inventing one.** The
`carried_join_arm` repair succeeded precisely by mirroring how the scalar lane
already represents a backedge arm; look for the equivalent before building new
machinery.

### `D3` — discriminating controls

Every row `D0` found stays enabled and unchanged in meaning, and runs green
under A-only exclusion at the pre-retirement base. **A mutation at each repaired
root recreates the attributed refusal while proving the detector was reached.**
Unaffected same-family controls stay green.

## 4. Acceptance criteria

- **AC-1 — the population is closed by measurement, not by grep.**
  *Control:* the handback enumerates each fixture reaching the production arm
  with its variant and outcome, and states the denominator. A grep list does not
  discharge this.
- **AC-2 — `PendingLet` and `Active` are partitioned on evidence.**
  *Control:* the handback states, per variant, the owned fact and the first
  missing/mis-consumed static fact. **If one mechanism serves both, that is a
  measured finding with its evidence, not an assumption from the shared arm.**
- **AC-3 — every repaired root has a committed discriminating control.**
  *Control:* it reds under a mutation at that root and greens without it, from
  the committed tree, with evidence the detector was reached.
- **AC-4 — the landed guards are intact.** `RecursiveBackedge` stays
  protocol-only; `emit_carrier_transfer` still refuses it as a source boundary
  value; the `carried_join_arm` repair is unchanged.
  *Control:* committed negative witnesses, each with a positive control proving
  the path is reached.
- **AC-5 — zero new `#[ignore]`, anywhere in this lineage.**
- **AC-6 — no retirement and no lane deletion.**
  *Control:* both residual variants, both insertions and the per-variant
  exclusion hook are present and unchanged at the final SHA.
- **AC-7 — the candidate contains NO tracker `status:` change.** The flip is the
  Steward's, post-merge.
- **AC-8 — CI green** on the merge. Not a local `--workspace` run
  (`COORDINATION §12`).

## 5. Banned scope

- **No `#[ignore]`.** Ruled out for this arc; the ruling was not reopened.
- **No reshaping a fixture and no absorbing a refusal** to make a row pass.
- **No simultaneous exclusion of both variants, and no generalized hook.**
- **No reinterpreting a retained `RecursiveDescent` run as activation.**
- **No touching rows 1-5** or the `LexicalCallArgumentRecursor` population.
- **No resume or cherry-pick of `10369776252861e8b15e613576256a3682c70066`** —
  held evidence only.

## 6. Hard stops

**Return the partition before coding** if any fires:

1. **`PendingLet` and `Active` prove materially distinct authorities.** Two
   authorities is a partition to route, not a two-arm `match` to write.
2. **Any repair needs a new planner or ABI population.**
3. **The refusal advances again to a fourth authority.** See below — this is the
   likely one, and it is not a failure.

> ### EXPECT THE REFUSAL TO ADVANCE. STOPPING IS THE DESIGNED OUTCOME.
>
> This is the **third** wall on one chain: `resume_active_continuation` →
> `carried_join_arm` → this arm. Each repair was correct and each revealed the
> next. **That is the fail-closed machinery working on a newly reachable
> population** (Campaign Trap 2), not a defect in the node that finds it.
>
> ⇒ **If a fourth authority appears, stop and route it — do not absorb it.**
> It will be legible and you will be able to see roughly what it wants, and
> that is precisely the condition under which absorbing feels efficient.
> `#27` is the record of five instances of one shape absorbed one at a time,
> each individually reasonable.
>
> **A stop here costs one turn. Absorbing costs the node's sizing and hides
> the population from everyone who reads the tracker.**

## 7. Sizing

**`M`, provisional, and the re-size point is the `D0`/`D1` checkpoint** — post
it as its own exact SHA before starting `D2`.

Two things could move it: a population materially wider than the two exposed
rows, or `PendingLet`/`Active` proving distinct authorities. Both are
measurements, not judgment calls, so neither is yours to absorb.

## 8. Base

**Cut from the accepted partial carrying the `carried_join_arm` repair** — it is
the object that exposes this node's population, and without it the rows still
refuse on `RecursiveBackedge` and nothing here reproduces.

**As of 2026-08-08 that object is `24d585f8`, NOT `50808c11`.** QA superseded
`50808c11`; `24d585f8` is a strict descendant differing by two comment lines and
zero non-comment lines.

> ### CUT FROM THE OBJECT THAT WILL MERGE, NOT THE ONE THIS FRAME NAMES
>
> **Standing rule, and apply it without asking if `24d585f8` is superseded in
> turn.** The accepted partial merges by **squash**, so the SHA named here never
> becomes an ancestor of `main`. A branch cut from a superseded object carries
> the superseded lines in their pre-fix form, and at merge that is either a
> conflict or a **silent re-introduction of the exact defect review rejected**.
>
> **The silent case is the dangerous one, and comment-only fixes are its worst
> case:** a reverted comment reds no test, trips no gate, and passes a
> diff-scope check. It is the one class of correction every mechanical control
> we have is blind to.
>
> Measure against the object under review, and **name your exact base in the
> handback** rather than relying on this line being current.

Not `10369776`, ever.
