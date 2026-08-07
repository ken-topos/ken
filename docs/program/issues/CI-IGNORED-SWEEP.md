---
id: CI-IGNORED-SWEEP
title: "nothing in the repo ever re-runs an ignored row, so every skip is write-only and a landed repair ships with its own regression cover switched off"
status: ready
owner: verify
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: Adversary finding evt_4mwy8tmfmm7tw (F2), triaged and independently confirmed by the Steward against origin/main 533f7c06. Filed as its own node on the operator's ruling 2026-08-07, which kept the RT-SRCBODY-BIND-ORDER candidate's diff minimal rather than folding the sweep into it. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## FRAMED — `ready`, size S
>
> Frame:
> [`CI-IGNORED-SWEEP`](../wp/CI-IGNORED-SWEEP.md)
>
> **The frame governs; this file is the origin record.** Where the two differ,
> the frame is later and was ground against `origin/main` `368ff87e`.
>
> **Two things the frame settles, which this record leaves open:**
>
> 1. **The classification carrier is DECIDED, not a deliverable to survey.** It
>    is an explicit checked-in registry keyed on **test path**, and it **fails
>    toward sweeping**: an unregistered row gets swept. A base-debt row omitted
>    from the registry costs noise; a policy row omitted costs a missed
>    regression, and the missed regression is the thing this node exists to
>    prevent. Frame `D1`.
> 2. **The two venues use different tools, and that decides the shape.** CI runs
>    nextest (`ci.yml:121`); **nextest is NOT installed locally** (`cargo nextest
>    --version` reports `no such command`), and there is no `.config/nextest.toml`.
>    So the hand-run `-- --ignored` measurement recorded below is libtest syntax
>    and **does not transfer to the CI job**, and the implementer cannot iterate
>    on the nextest invocation locally at all. Frame `1e` and `D0`.
>
> The frame also adds a control this record does not imply: the exemption
> mechanism must be structurally unable to deselect a row from the **main**
> `--workspace` gate (`AC-3`). A registry implemented as a default nextest
> profile would silently narrow every run.

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

`RT-SRCBODY-BIND-ORDER` brings the suppressed population to **50 rows, not the
46 this node first declared.**

**CORRECTED 2026-08-07** from Adversary `evt_2yxmdfhvt4fm0` (F2), re-measured
by the Steward at `b0a0a20c`. Use the anchored form — `^[[:space:]]*#\[ignore`
— which excludes doc-comment lines that merely mention the attribute; the
unanchored form inflates every file:

```
total 50 — ken-cli 34, ken-verify 10, ken-runtime 3, ken-interp 3
```

**The 46 was not a miscount of the quarantine; it was a scope error.** It
summed the 42-row authorized set and the four pre-existing `px4b` ignores —
both of which are base debt this program filed — and thereby **counted the
population this node AUTHORED rather than the population its mechanism will
SELECT.** A sweep selects on the `#[ignore]` attribute, not on provenance.
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

**Read that claim at its real width: it covers 44 of the 50 rows.** The command
ran `ken-cli` and `ken-verify` only, so the three `ken-runtime` and three
`ken-interp` ignores were never in it — including the 142-second cost row,
which is precisely the one a `--ignored` sweep would have made expensive to
discover. **The over-annotation question is open for those six**, and the
sweep this node builds is what closes it. A per-crate invocation list is
therefore part of what the frame owes.

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

- **A SECOND CUT, on reason-for-ignoring.** The four rows outside the declared
  46 are not base debt, and they are exactly the class that breaks the report:

  | row | reason class |
  |---|---|
  | `crates/ken-runtime/src/boundary_value_clif.rs:7473` — `"~142s of arena work; the fast instance at depth 3000 runs by default"` | **COST** |
  | `crates/ken-interp/tests/l1_acceptance.rs:242` — `"explicit conversions require L-classes or a separate conversion WP"` | **UNBUILT CAPABILITY** |
  | `crates/ken-interp/tests/l1_acceptance.rs:284` — `"integer division not yet in scope for L1; requires div op registration"` | **UNBUILT CAPABILITY** |
  | `crates/ken-interp/tests/l1_acceptance.rs:334` — `"Char literal syntax not yet in scope for L1"` | **UNBUILT CAPABILITY** |

  **Neither class awaits a repair from an owner node, and neither should ever
  be un-ignored by one.** A sweep whose question is *"does this row still
  belong in the quarantine?"* answers **yes** for all four, permanently. That
  is standing noise in the first report this node ever emits — **the report
  that decides whether anyone keeps reading it.**

  The cost row is worse than noise: **a sweep that re-runs the ignored
  population pays 142 seconds for it, unbudgeted**, on every run.

  ⇒ Classify on `base-debt-awaiting-repair` versus `ignored-by-policy`, and
  **have the sweep select only the first.** This is the node's second
  classification axis; the first is the one already recorded above — a row red
  for a reason upstream of its own property. They are independent: a row can be
  base debt *and* die before its assertion.

- **Do not enforce the classification by parsing the reason string.** These
  four are distinguishable today only by prose a human wrote. If the frame
  wants a machine-checkable cut it must say what carries it — a structured
  marker, a per-file allowlist, or an explicit `ignored-by-policy` registry —
  and **that choice is a deliverable, not an assumption.** A sweep that
  greps for "not yet in scope" is one reworded comment away from silently
  re-including a row.
