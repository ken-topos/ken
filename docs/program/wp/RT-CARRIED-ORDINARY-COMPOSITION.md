# RT-CARRIED-ORDINARY-COMPOSITION — continue the composed suffix

Owner: Runtime. Size: **M, provisional** — see Sizing.
Authority: Architect fourth-wall ruling `evt_63ae56tttz9pq` (2026-08-08) on the
[[RT-CARRIED-CONTINUATION-RESUME]] `D2` armed stop `evt_7qcgfbwgxh0qf`.

**Read `docs/program/16-recursive-descent-retirement.md` first** — the campaign
context and the five traps. **Traps 1, 2 and 3 are all live here.**

> # WHAT THIS NODE IS FOR, IN ONE SENTENCE
>
> A carried ordinary elimination **consumes exactly one frame**, so when a
> composed suffix stands behind it the port refuses rather than dropping it —
> and this node continues the suffix, or proves the shape unportable and refuses
> it lawfully.
>
> **No landed repair is reopened.** [[RT-PRODUCER-MATCH-PORT]] is merged and its
> guard is **correct**: it fails closed on a shape it never ported, exactly as
> its author documented. **You are completing it, not fixing it.**

## 1. Fixed inputs

**Measure every one of these yourself at your pinned base.** They were measured
at `cc736aaf30dbe210af01d9f7e93d8b24bacc0df0` and are **anchors to re-find,
never values to trust**.

| input | as measured at `cc736aaf` |
|---|---|
| exposed rows | **2** — `d8d_the_composed_binding_site_...` and `px8j_all_three_producer_paths_reach_real_consumers` |
| activation seam | **A**-only exclusion, the committed one-variant hook |
| refusal | `Unsupported(BoundaryCarrier, "a carried producer-call scrutinee reached an ordinary eliminator with further composed eliminators behind it; the carried elimination consumes exactly one frame, so the remainder would be silently dropped")` |
| owner | the `Carried x Ordinary` **pre-delegation guard family**, in front of the `lower_carried_match` delegation |
| suffix length measured | **one** trailing eliminator |
| suffix provenance | the successor `Active` frame **rebuilt from `active.pending`** by `resume_active_continuation` — **not** the explicit outer tail |
| the other two guards | `retained_scrutinee_index` and `deferred_constructor_case` — **neither fired**; population unmeasured |
| the new outer-tail guard | added by `cc736aaf`; **did not fire** |
| prior refusal | the continuation-frame `BoundaryCarrier` message is **gone from both rows** — that absence is the proof the `Active` route was reached |
| retained suite | **816 / 0 / 4**, the no-delta baseline |

**Coordinates: use function names and guard text.** At `cc736aaf` the
trailing-suffix guard sat near `core.rs:3761`. **This node moves that file**, so
any line number here rots against its own deliverable. Re-derive by name — and
if you record a trace, re-derive it **after** reverting instrumentation, because
instrumentation displaces the file and a displaced coordinate still resolves to
real code in a different function. That has already happened once on this chain.

## 2. What is owed

**A carried ordinary elimination that continues its composed suffix — or a proof
that a given suffix shape is unportable, with a lawful refusal for it.**

Both outcomes are legitimate, and the second is not a lesser result. Do not
assume every reachable suffix must be portable.

Not owed, and banned: reopening [[RT-PRODUCER-MATCH-PORT]]; reverting or
amending either landed carried repair; any work on rows 1-5
([[RT-LEXICAL-RECURSOR-CONSUMERS]]); any retirement or lane deletion.

## 3. Deliverables

### `D0` — close the population across the whole guard family

**The population is all three pre-delegation guards, not the one that fired.**

1. `retained_scrutinee_index = Some(_)`;
2. `deferred_constructor_case = Some(_)`;
3. nonempty `eliminators[1..]`, **with exact suffix length, kinds and
   provenance per member**.

**Census with denominators AND intersections.** A member may satisfy more than
one guard; the guards are ordered, so **only the first one reached is
observable** unless you measure the others independently. A census that records
only the first refusal per row cannot distinguish "the other guards have no
members" from "the other guards were never reached."

**The two A rows are the floor.** Enumerate from the production arm, per
fixture, by measurement. Record compilations that never reach the arm — the
denominator is what makes a zero mean anything, and it is what made
`PendingLet = 0` actionable at the sibling node instead of merely unobserved.

### `D1` — partition the three guards, before any repair

**A shared arm and a shared historical port do not prove one mechanism**
(Architect, explicit). Partition by owned fact, operand phase, required action
and fail-closed boundary.

**Keep the two suffix sources distinct in the evidence** — the explicit outer
tail guarded by `cc736aaf`'s new arm, and the successor frame rebuilt from
`active.pending`. They reach one guard and render one string. **Conflating them
misattributes the wall.**

Under **A**-only exclusion: each firing row reproduces its exact first refusal;
the retained run stays green; a same-family positive control that already works
stays green; and **activation denominators are recorded**.

### `D2` — repair only the proven cells

Consume or represent the fact **at its owner, before the guard**. Never teach a
downstream guard to accept a forbidden state.

**Prefer mirroring a landed representation over inventing one.** Both preceding
repairs on this chain succeeded exactly that way — `carried_join_arm` mirrored
the scalar lane's backedge representation, and the `Active` route mirrored what
the specialized path already did at two landed sites. Look for the equivalent
before building machinery.

**If only the trailing-suffix cell has members, repair that cell only.** Leave
the other two fail-closed as measured-at-base zeros, with the emptiness recorded
as a fact about this tree rather than a property of the design. Building a
mechanism for an empty cell is **Trap 3** — every control passes because there
is nothing to quantify over.

### `D3` — discriminating controls

Every row `D0` found stays enabled and unchanged in meaning, and runs green
under A-only exclusion at the pre-retirement base. **A mutation at each repaired
root recreates the attributed refusal while proving the detector was reached.**
Unaffected same-family controls stay green.

> ### THIS NODE FINALLY GETS THE END-TO-END CONTROL THE CHAIN HAS BEEN OWED
>
> Every predecessor's `D3` could only assert an **advance** — one refusal
> replaced by the next — because the rows never compiled. **If `D2` closes both
> A rows here, they compile end-to-end for the first time on this chain**, and
> that is the control to write. Say plainly in the handback whether they do.
>
> If a **fifth** authority appears instead, the advance-control shape carries
> over unchanged and the end-to-end control belongs to the successor.

## 4. Acceptance criteria

- **AC-1 — the population is closed by measurement across all three guards.**
  *Control:* the handback enumerates each member with its guard cell, suffix
  length/kind/provenance, and the denominator; and states, per non-firing guard,
  whether it has no members or was merely never reached. A grep list does not
  discharge this.
- **AC-2 — the three guards are partitioned on evidence.**
  *Control:* per guard, the owned fact and the first missing/mis-consumed static
  fact. **If one mechanism serves more than one, that is a measured finding with
  its evidence, not an inference from the shared arm.**
- **AC-3 — the two suffix sources are distinguished in the evidence.**
  *Control:* each firing member states which tail produced its suffix — the
  explicit outer tail or the `active.pending` successor frame — by measurement,
  not by the refusal string, which cannot tell them apart.
- **AC-4 — every repaired root has a committed discriminating control.**
  *Control:* it reds under a mutation at that root and greens without it, from
  the committed tree, with evidence the detector was reached. A hand-run
  mutation does not discharge this.
- **AC-5 — the landed guards are intact.** `emit_carrier_transfer` still refuses
  `RecursiveBackedge` as a source boundary value; the `carried_join_arm` repair
  and the `Carried x Active` route are unchanged; `PendingLet` stays
  fail-closed.
  *Control:* committed negative witnesses, each with a positive control proving
  the path is reached.
- **AC-6 — zero new `#[ignore]`, anywhere in this lineage.**
- **AC-7 — no retirement and no lane deletion.**
  *Control:* both residual variants, both insertions and the per-variant
  exclusion hook present and unchanged at the final SHA.
- **AC-8 — the candidate contains NO tracker `status:` change.** The flip is the
  Steward's, post-merge.
- **AC-9 — CI green** on the merge. Not a local `--workspace` run
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

1. **The three guards prove materially distinct authorities** requiring separate
   mechanisms. That is a partition to route, not a three-arm `match` to write.
2. **Any repair needs a new planner or ABI population.**
3. **Continuing the suffix requires `lower_carried_match` to express more than
   cases / default / origin / env** — that is a widening of the carried
   elimination's interface, which is a component-design call.
4. **The refusal advances again to a fifth authority.**

> ### THIS IS THE FOURTH WALL ON ONE CHAIN. EXPECT A FIFTH, AND ROUTE IT.
>
> `resume_active_continuation` → `carried_join_arm` →
> `lower_computational_match_value_composed` → this guard family. **Every one of
> those four repairs was correct, and each revealed the next.** That is the
> fail-closed machinery working on a newly reachable population (Campaign
> Trap 2), not a defect in the node that finds it.
>
> ⇒ **If a fifth authority appears, stop and route it — do not absorb it.** It
> will be legible and you will see roughly what it wants, and **that is exactly
> the condition under which absorbing feels efficient.** `#27` is the record of
> five instances of one shape absorbed one at a time, each individually
> reasonable.
>
> **A stop costs one turn. Absorbing costs the node's sizing and hides the
> population from everyone reading the tracker.** Two walls have now been
> returned cleanly on this chain; that is the standard.

## 7. Sizing

**`M`, provisional, and the re-size point is the `D0`/`D1` checkpoint** — post
it as its own exact SHA before starting `D2`.

Three things could move it: a population wider than the two exposed rows,
members in the other two guard cells, or a suffix that cannot be continued
within `lower_carried_match`'s existing interface. All three are measurements,
so none is yours to absorb.

**If `D2` comes in well under `M` because only one cell has members, that is not
a sizing error** — a provisional size bounded by stops is doing its job when the
stops do not fire.

## 8. Base

**Cut from the merged accepted partial carrying the `Carried x Active` route** —
without it the rows still refuse at the continuation frame and nothing here
reproduces.

> ### CUT FROM THE OBJECT THAT WILL MERGE, NOT THE ONE THIS FRAME NAMES
>
> **Standing rule on this chain; apply it without asking.** The accepted partial
> merges by **squash**, so `cc736aaf` never becomes an ancestor of `main`. A
> branch cut from it carries its lines in pre-merge form, and at merge that is
> either a conflict or a **silent re-introduction of something review already
> corrected**. The silent case is the dangerous one: a reverted comment reds no
> test, trips no gate, and passes a diff-scope check.
>
> **Cut from `origin/main` once `cc736aaf` has squashed onto it**, and **name
> your exact base in the handback** rather than relying on this line being
> current. Measure against the object under review, always.

Not `10369776`, ever.

## 9. Contention

`lowering/core.rs` and `core/tests/control.rs` — the same two files as
[[RT-LEXICAL-RECURSOR-CONSUMERS]] and [[RT-RECURSOR-TRANSPORT]] `D3`. **All
contend, which is why they are serialized rather than parallel.** This node is
ahead of both.
