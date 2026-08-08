---
id: RT-EFFECT-DIFF
title: "One reusable rich differential boundary over EffectObservation — interpreter vs native, first-divergence reporting, so backend-local tests can observe what only the CLI suites currently can"
status: draft
owner: runtime
size: L
gate: none
depends_on: []
blocks: []
github: null
origin: Architect ruling `dec_3tawbngh6k761` (2026-07-29), the "separate row-3 obligation" clause, on the RT-FNSPLIT-RECUR-PORT hard-stop #18 evidence; research advisory `evt_6980s92jgvf4h` row 3; Runtime Leader registration request `evt_3dxjc38x8w1sa`. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## STATUS CORRECTED `ready` TO `draft` — 2026-08-08, Steward
>
> **`ready` means shovel-ready: a written frame, dependencies merged.**
> This node was not, because
> no frame exists.
>
> **The correction is not a downgrade of the work.** A node advertising
> startable work it does not have makes the backlog read deeper than it
> is, and that depth is exactly what a Steward reads to decide whether a
> team is idle for want of work or for want of a lane.

> ## ▶ THE FRAME IS WRITTEN — read it, not this file
>
> `docs/program/wp/RT-EFFECT-DIFF-observation-boundary.md`

## ⭐⭐ THIS NODE'S REGISTRATION IS ITSELF A GATE DISCHARGE

`dec_3tawbngh6k761` closes with a condition, quoted exactly:

> *"The row-2 repair may merge independently; forcing that harness into this
> small fix would manufacture atomicity. But RECUR-PORT may not close, and
> RT-SCALE-B may not resume on a completed-representation empirical claim,
> until the row-3 work is durably registered as a Runtime dependency **or the
> harness lands**."*

⭐ **Registration discharges it — the harness does not have to land first.** This
file plus its frame is that registration. ⇒ [[RT-FNSPLIT-RECUR-PORT]] may close
on its row-2 repair, and [[RT-SCALE-B]] may run, without waiting on this node.

## ⛔ WHY THERE IS NO `depends_on` AND NO `blocks` EDGE

Both are deliberate, and both are the narrower reading of the ruling.

- **No `blocks`.** The ruling's gate is satisfied by registration, so a `blocks`
  edge would encode a constraint the Architect explicitly declined to impose and
  would stall the ABI critical path behind a testing-infrastructure build.
- **No `depends_on` on `RT-FNSPLIT-RECUR-PORT`.** The Architect ruled the row-2
  repair merges independently. The divergence populations this node seeds from
  are already measured and durable (six sites, `evt_3d41hdqe49pga`), so nothing
  here waits on that repair. ⚠ Runtime is single-threaded on one shared build
  turn — sequencing between the two is the ring's turn model, **not** a graph
  edge. ⛔ Do not add one to express "do row 2 first."

⚠ **The one thing that IS constrained**, and it is a constraint on a *claim*, not
on a start: `RT-SCALE-B` may not return a verdict asserting the representation is
**complete/verified** while this node is open. It may measure and it may run.
That caveat is recorded on `RT-SCALE-B` itself.

## The gap, stated as measured fact

`RuntimeObservation` is limited to returned ground values or traps
(`ir.rs:865-870`), and the packaged decoder handles only scalar `Int`/`Bool` with
trap decoding unavailable. The dependent suites compare against the far richer
`EffectObservation` surface — stdout, stderr, filesystem delta, terminal error,
canonical effect trace and order, terminal class, exit status.

⇒ ⭐ **The two suites are not asking the same observational question, and the
crate boundary is incidental.** That is why `ken-runtime --lib` returned 562
passed / 0 failed on a candidate that then failed seven CI checks at six sites.
⛔ A `-p ken-runtime` green is not evidence about the richer observation, and
this will recur on every future representation change until the boundary exists.

## ⛔ WHAT THIS NODE MAY NOT DO

- ⛔ **Do not copy the `ken-cli` assertions into a runtime-local corpus.** The
  research advisory warns specifically against a second corpus that drifts from
  the first. Build one comparator; feed it from both fixture sources.
- ⛔ **Do not weaken, filter, or re-baseline any existing CLI assertion.** The
  interpreter and the existing `ken-cli` parity suites remain the semantic
  oracle, and those suites remain an **independent packaging/integration
  backstop** — this node does not replace them.
- ⛔ **Do not fold this into the row-2 repair.** The Architect named that
  manufactured atomicity.

Seeded by the `RT-FNSPLIT-RECUR-PORT` divergence populations. Prior art surveyed
in the research advisory `evt_6980s92jgvf4h`, durable at
`local/rt-fnsplit-recur-port-hard-stop-18-differential-oracle-advisory.md`.
