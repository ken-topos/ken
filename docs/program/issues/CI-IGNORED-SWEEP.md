---
id: CI-IGNORED-SWEEP
title: "nothing in the repo ever re-runs an ignored row, so every skip is write-only and a landed repair ships with its own regression cover switched off"
status: draft
owner: verify
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: Adversary finding evt_4mwy8tmfmm7tw (F2), triaged and independently confirmed by the Steward against origin/main 533f7c06. Filed as its own node on the operator's ruling 2026-08-07, which kept the RT-SRCBODY-BIND-ORDER candidate's diff minimal rather than folding the sweep into it. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## THE FRAME IS OWED. `draft`, NOT startable.
>
> The measurements below are done and reproducible, so the frame is short.
> What it still owes is the job's exact shape, its non-blocking wiring, and
> the acceptance criterion that proves the sweep can actually report.

## The gap, measured

`--ignored`, `--run-ignored` and `include-ignored` appear **nowhere** under
`.github/`, `scripts/`, or `docs/program/` at `533f7c06`:

```sh
grep -rniE '\-\-ignored|run-ignored|include-ignored' .github/ scripts/ docs/program/
# empty
```

So the suppressed population is **write-only**. A row goes into it and no
mechanism ever asks whether it still belongs there.

## Why this is load-bearing now rather than someday

`RT-SRCBODY-BIND-ORDER` brings the suppressed population to **46 rows** — the
42-row authorized annotation set plus the four pre-existing `px4b` ignores.
**Eight** owner nodes are queued to land repairs against them
([[RT-CARRIER-BYTESPAN-OBSERVE]], [[RT-CARRIED-RESOURCE-SCALAR]],
[[RT-CLOSURE-BOUNDARY-LANE]], [[RT-COMPMATCH-TREE-SCRUTINEE]],
[[RT-FRAME-MARKER-ONCE]], [[RT-PROCESS-EXIT-STATUS]],
[[RT-WORKER-FIXTURE-DECODE]], [[RT-CARRIER-PRODUCER-OCCURRENCE]]).

**The last two sharpen this node's case rather than merely extending it.** Both
were found only because CI failed on them after a census that had stopped at 2
of 8 workspace members, and **both die at an `expect` before reaching the
assertions they exist to make.** So a sweep that merely re-runs the ignored
population and reports "still failing" would say nothing useful about them: the
question is not whether they fail but whether their fixtures can execute. **A
row that is red for a reason upstream of its own property is a distinct class**,
and the sweep's report needs to be able to say so rather than collapsing it into
a pass/fail bit.

**When one of those lands its repair, nothing reports that the row now
passes.** The `#[ignore]` persists, so the repair ships with its own
regression cover switched off — the node fixed the defect and simultaneously
guaranteed nobody would notice if it came back.

## The failure already happened once

`RT-SRCBODY-BIND-ORDER` `D11` ignored `px7o` on a false premise and **would
have switched off a working repair.** It was caught only because `D12`
happened to run a complete enumeration that included ignored rows — that is,
by luck of scope, not by any mechanism.

**A normal verification run cannot see this by construction.** `D13`'s
`120 passed / 0 failed / 34 ignored` is *disjoint* from the population it
suppresses: every row it reports on is a row that is not ignored.

## The check is cheap and has been run once, by hand

The ring ran it on request at `7d204438`:

```
ken-cli    --no-fail-fast -- --ignored   ->  0 passed / 34 failed
ken-verify --no-fail-fast -- --ignored   ->  0 passed / 10 failed
```

All 44 still fail, so there is **no over-annotation at that tip**. That is a
one-off measurement by hand, on request. This node makes it standing.

## What the frame owes

- **Non-blocking by construction.** A row that starts passing is *good news*
  needing routing, not a red gate. It must not become a fourth way for an
  unrelated candidate to be blocked.
- **It must be able to report.** A sweep that silently passes when it ran
  nothing is the same defect one layer up — the exact shape that produced
  `--no-tests` exit 4 on the two `px8f` jobs. Assert the **positive**: the
  expected suppressed-row count, and `$?`, never the absence of a failure
  token.
- **A positive control.** Un-ignore one known-failing row, observe the sweep
  reports the change, restore it. Without that the sweep passes for any
  reason, including never having run.
- **Name where the report goes.** A finding with no route is a finding nobody
  acts on; the owning node named in each `#[ignore]` string is the natural
  address.
