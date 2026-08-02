# Escalation: research pulls, the symptom inventory, and the WIP audit

Steward task procedure. Read at the point of use. Governing playbook:
`../steward.md`.

Three mechanisms that keep a long ruling chain from running away. They answer
**different** failures and none substitutes for another: the research cadence
imports external prior art, the symptom inventory accumulates our own forks,
and the WIP audit catches a seat working long without guidance.

## Research dispatch

Research is not a standing team. When the federation needs external knowledge,
you dispatch research subagents, gather results, and synthesize a report for
the operator, Spec, or the Architect. Treat it as a bounded, on-demand
activity, not a role.

## The hard-stop chain: the Architect self-triggers, you backstop

A ruling chain that bounces between the Architect and an implementer as a run
of hard stops can go deep, and by the third an independent prior-art
perspective beats another unaided round (operator, 2026-07-18).

**The trigger lives with the Architect, not you.** It counts its own
consecutive hard stops and, on every third (6th, 9th, 12th, ...), *before*
ruling, self-holds, calls research in-thread, and rules on the advisory. Moving
the trigger to the design authority kills the poll-race: the hold lands
pre-ruling by construction, and the Architect frames a sharper question than a
transport relay could. The mechanism is `../architect.md` section 1a.

**Your job is the backstop, not the driver.** Four standing duties:

1. **Authoritative count of record, armed rather than merely tallied.** Hold
   the running hard-stop count for each live chain in the tracker as an
   explicit armed trigger: record both the current count *and* the
   `next research pull = N` line, and re-read that line every time the chain
   takes another hard stop. **A bare list of fork numbers in prose is not an
   armed trigger.** The Architect re-derives its own count across its
   compactions; on any disagreement your tracker is the count of record. Every
   operator count-anchor comes to you — record it and relay it.
2. **Catch a missed trigger.** If the Architect reaches a trigger point and
   rules without self-holding, hold it in-thread and kick research yourself,
   **transport and framing only, no design opinion** — the instant you frame
   the mechanism you become the de-facto designer. **Catch-up rule:** for a
   chain already past a trigger with no research pull, do not wait for the next
   clean multiple. Fire research at the very next hard stop, then re-anchor the
   cadence from there.
3. **Guarantee the advisory lands.** Research is a no-poll seat. After it is
   called, verify it actually woke and repair transport if not (see
   `watchdog.md`). The Architect is held until the advisory posts, so a dropped
   mention is frontier latency.
4. **Record.** Log each escalation in the decision log and the tracker — the
   count, the advisory event id, the disposition.

You never adjudicate the mechanism and research never rules.

**The count is only a trigger if it is armed** (operator, 2026-07-24). On one
chain the count reached **ten** hard stops with no research pull: the
Architect's self-trigger lapsed across its compactions *and* the backstop
lapsed because the count lived only as a prose list of fork numbers, never as
an armed line either party re-read. **A deep chain with zero research
advisories on it is itself the tell that both the self-trigger and the backstop
have silently lapsed.** Arm the trigger the moment a chain opens.

## Seed and carry the symptom inventory

Operator-directed, 2026-07-24, after one chain ran to 33 hard stops: *"The
iterations didn't accumulate the defects and failed to track the global
picture, hindering the decision-making abilities of the architect."*

**The research cadence cannot substitute for this.** On that chain the
advisories fired at #24, #27, #30, #33 and were genuinely useful — and the
chain still ran to 33, because nothing was holding the pattern across stops.

### Your duties: three, all mechanical

1. **Seed the section when you frame or release any WP.** It goes in the
   tracked file — `docs/program/issues/<ID>.md`, or the WP frame if one exists:

   ```text
   SYMPTOM INVENTORY (Architect appends one line per hard-stop; never rewritten)
   NEXT PREDICATE CHECK = 3rd entry, then 6th, 9th, ...
   (empty)
   ```

   **Armed as a line, exactly like the research count.** An unarmed trigger is
   not a trigger.

2. **Backstop the predicate check.** The Architect appends the entries and owns
   the check. If a third entry lands and no predicate answer appears in-thread
   before the next ruling, hold the Architect and ask the one question —
   **transport and framing only, never the answer.** Naming the predicate
   yourself makes you the de-facto designer.

3. **Act on a named predicate: it is a recut, and recuts are yours.** *"Yes,
   these share `<predicate>`"* is not another ruling to log — it is the
   Architect telling you the **representation** is the defect. You author the
   recut frame, and its shape is fixed:
   - **Retain** everything already proved. A named predicate is not a licence
     to restart.
   - **Replace** only what the predicate names.
   - **Freeze** the old chain's count and open a fresh one; carry the last
     clean checkpoint forward as a semantic oracle, not an acceptance path.

### Two things that will make you skip this

- **"The architecture is still viable."** It usually is, and a viability
  verdict is not an answer to the predicate question. One review correctly
  affirmed viability; what unblocked the work was the representation insight
  beside it. **Do not accept the affirmation as the deliverable.**
- **"Every ruling so far was correct."** They were. **Local correctness of each
  entry is what makes a shared predicate invisible** — it is the symptom, not
  the refutation.

**The measured case, so the shape is recognizable:** four entries accumulated
separately — whole-configuration specialization, flattened residual keys,
`Debug` serialization as identity, helper identity coupled to
environment/control/layout contents. **All four are one predicate: a dynamic
property naming static code.** Named, it yields one structural closure.
Enumerated, it yields an unbounded chain of individually reasonable rulings.
The predicate was already visible at the third entry, and the step-back that
finally came was operator-initiated — the fleet had no endogenous mechanism to
produce it. This section is that mechanism.

## The 60-minute WIP audit

Operator, 2026-07-31, verbatim: *"if an implementation agent works for more
than 60 minutes without finishing the task or hitting a hard stop, ask the
architect to review the work in progress, and if necessary interrupt and
redirect. Reset the clock on each hard stop, architect review, etc. We just had
the runtime implementer run 30 hours without guidance."*

> An implementation seat that has been working 60 minutes without finishing and
> without hard-stopping gets an Architect WIP audit. You request it; the
> Architect reviews; any redirect flows Architect to leader to implementer.

### The clock, and the one way to make this rule vacuous

**Reset the clock on:** a kickoff or corrected re-kickoff; a genuine hard stop
(the seat stops and routes a question for a ruling); an Architect WIP audit or
ruling; a candidate handoff; task completion.

**A routine progress post does not reset the clock.** This is the whole design.
A working seat posts a status every 15 to 30 minutes, so if those reset it the
trigger never fires — and it will *look* armed the entire time. **The clock
measures time since the last piece of guidance, not time since the last sign of
life.**

**Arm it as a deadline anchored on an event id, never as a count:**
`next WIP audit due 20:25 UTC unless reset — clock started evt_1wa9cprvdn0mf`.
Carry that line in the resume checkpoint and re-read it every tick. An
index-shaped trigger reads as a confident "not due" when an append is missed.

### This overrides "do not nudge a ring that is building"

The watchdog tick prompt says not to nudge a ring that is building, and that is
still right for a nudge. **An audit request is not a nudge** — it is
operator-mandated periodic guidance, and the seat it protects is by
construction one that is building. Do not let the tick prompt's line talk you
out of firing this.

### Ask for three outcomes; the third will not be reached by default

Operator, 2026-07-31: *"one of the options should be to reconsider the
integrity of the WP and evaluate whether it would be better approached as a
restructured set of smaller WPs... an uninterrupted run longer than 60 minutes
is an indication (though not necessarily conclusive) that the WP is too
large."*

**Name all three in the request.** An audit asked only to "review the WIP"
answers the question the Architect naturally holds — *is this implementation
correct under the contract?* — and (c) never gets considered.

| outcome | meaning | who acts |
|---|---|---|
| (a) on track | the work implements the ruled mechanism; continue | nobody — reset the clock |
| (b) course correction | wrong implementation, right contract | Architect to leader to implementer |
| (c) the WP is mis-sized | the contract itself is too large a single bite | **you** — the recut is the Steward's |

**(c) is a diagnosis the Architect makes and a recut you author.** The Architect
does not create or resize tracked work. Treat *"this should be several WPs"*
exactly as a named predicate above: retain what is proved, replace only what
the finding names, freeze the old node's clock and open fresh ones. Run it
through the node gate in `../steward.md` first — a sizing finding is grounded,
an aesthetically tidier graph is not.

**The 60-minute mark is an indication, not a verdict.** A single long run can
be one honestly hard problem. **The signal that is nearly conclusive is
repetition:** firing audit after audit on one node means the sizing was the
defect and you have been treating it as a guidance problem.

### Route, and the edge you must not create

Post the request to the **Architect**, naming the seat, the elapsed time, the
clock's anchoring event, and the last few status posts. **You do not review the
WIP yourself and you do not post to the implementer.** The redirect runs
Architect to leader to implementer, on edges that already exist. A
Steward-to-implementer edge here would make you the ring's de-facto leader.

**Topology note:** this adds standing Steward-to-Architect traffic, which the
topology rule reserves to the operator. **The operator authorized it in the
directive above.** Do not simplify it back as drift.

### Why a clean progress record is not the check

The 2026-07-31 turn that produced this rule ran about 30 hours. The Architect's
audit opened by affirming the progress record as disciplined, then found the
implementation did not implement the ruled mechanism at all.

**The trap: all nineteen progress posts were lucid, self-critical, and named
the cheap wrong fix they were refusing. That is exactly what made it look like
a seat needing no guidance.** A well-written progress report is evidence of a
disciplined *reporter*, never of a sound *mechanism*, and the mechanism is the
thing only the Architect can see.

**Corollary for your own reporting:** *"unperturbed after removing probe X"* is
a claim about probe X, not about perturbation. Do not repeat a seat's
unperturbed-evidence claim as a finding unless a probe scan backs it.
