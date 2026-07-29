---
id: RT-SCALE-B
title: "Boundary B — the full n=3..7 emission measurement, the research-grounded analytical model, and the operator scaling verdict that gates RT-NATIVE-FNSPLIT's merge"
status: merged
owner: runtime
size: L
gate: none
depends_on: [RT-SCALE-A, RT-FNSPLIT-B2F, RT-FNSPLIT-RECUR-PORT]
blocks: [RT-NATIVE-FNSPLIT]
github: null
origin: Operator scaling-gate directive 2026-07-23 (evt_4btfhwqhah1ye), requirements 1-3; relocated to the recut by `docs/program/wp/RT-NATIVE-FNSPLIT-recut.md` as Boundary B, whose metric list survived the 2026-07-24 B1/B2 split (Architect evt_49bnspfb74tne + addendum evt_3b2a75fcaegja) as B2's. Research dispatch for the analytical half: evt_62fqpe7pfvym4. Steward-filed 2026-07-26 (agents cannot create tracked work per COORDINATION §2) because the gate had acceptance criteria and no tracked node.
---

> ## ▶ THE FRAME IS WRITTEN — read it, not this file
>
> `docs/program/wp/RT-SCALE-B-emission-scaling-verdict.md`

> ## ⭐ `ready` 2026-07-28, AND IT SUBSUMES `RT-FNSPLIT-B2B`
>
> **`status: draft` → `ready`.** The frame is shovel-ready; the only thing this
> node waits on is `RT-FNSPLIT-B2F` merging, which the `depends_on` edge already
> expresses. ⇒ It now enters the releasable frontier **automatically** the moment
> `B2F` lands, instead of needing a Steward pass first.
>
> **[[RT-FNSPLIT-B2B]] is `closed` as subsumed into this node.** It was the same
> deliverable: its `AC1.1′`–`AC1.5′` map one-to-one onto `AC-B1`–`AC-B4`, its
> metric list is `D2`, its differential suite is `D3`, and its closing action is
> this node's `blocks` edge on `RT-NATIVE-FNSPLIT`.
>
> ⛔ **The origin line below contains a false premise, kept visible rather than
> rewritten:** *"the gate had acceptance criteria and no tracked node"* — `B2B`
> **was** the tracked node, filed by the Steward the day before. This node
> survives the fold because it has a written frame and the **analytical half**
> (`D4`, Architect) that `B2B` never carried; four things `B2B` carried and this
> frame lacked are folded into the frame.

## ⛔ THIS IS THE NODE THAT DECIDES THE EFFORT

`RT-NATIVE-FNSPLIT` **does not merge** until this node returns a verdict. Every
other node in the chain — `B1R`, `B2A-C`, `B2A-S`, `B2O`, `B2R`, `B2V`, `B2F`,
`B2O-CHECK` — is machinery **built to be measured here**.

⭐ **The whole reason the chain was recut is that this measurement did not
exist.** 33 hard-stops of correct, converging semantic work were accumulating on
a representation that provably could not reach the gate. **Every individual
ruling was right; the thing they were accumulating into was not.** The verdict
below is what stops that from happening a second time.

## Three requirements, from the operator directive verbatim in substance

1. **Empirical scaling harness** (Runtime, **permanent tests**) — n = 3..7, each
   under a bounded fail-safe harness, reporting compile wall-time, peak RSS, and
   internal counts.
2. **Analytical scaling model** (**Architect**) — predicted order of growth vs.
   n, and specifically whether ~103 s / ~4 GB at n=4 is **bad constants on an
   O(n) mechanism** or **residual super-linearity** (⇒ a further mechanism gap).
   Must be **research-grounded** (dispatch `evt_62fqpe7pfvym4`).
3. **Verdict** — either **(a)** empirically **and** analytically **linear O(n)**
   plus a plan to reduce the constants, or **(b)** a **research-supported**
   reason growth is inherently super-linear, plus an **explicit operator
   ceiling / acceptability decision.**

⛔ **Requirement 3 is not the Runtime ring's call to make alone.** Outcome (b)
routes to the **operator** through the Steward. Do not close this node with a
verdict that only the ring has read.

### ⚠ ONE CONSTRAINT ON THE VERDICT'S WORDING — Architect `dec_3tawbngh6k761`

⭐ **This node may run and may measure. It may not claim the representation is
complete or verified while [[RT-EFFECT-DIFF]] is open.**

That node is the row-3 observation boundary: `RuntimeObservation` is limited to
returned ground values or traps, so a backend-local run structurally cannot
observe the `EffectObservation` surface on which native/interpreter divergence
has twice been caught. ⇒ An empirical table gathered through the narrow
observation is a **real measurement of what it measures** and is ⛔ **not**
evidence that the emission port is semantically complete.

⚠ This is a constraint on a **claim**, not a dependency. There is deliberately no
`depends_on` edge — see `RT-EFFECT-DIFF.md` for why, and ⛔ do not add one.

## What this node does NOT own

- ⛔ **Boundary A's planner metrics.** Those are [[RT-SCALE-A]]. **Neither
  boundary may stand in for the other.**
- ⛔ **`B2F`'s `AC-G0` helper-count denominator.** That is a **Θ(1)-per-module
  growth invariant** (6 definitions / 8 declarations) and is already answered;
  it is **not** the n=3..7 empirical table and does not discharge any part of
  this node.

Gates the [[NATIVE-HANDLE-CARRIER]] fast-follow and [[PX8-F-CAP-41]] too.

## ⭐ SYMPTOM INVENTORY

The Architect appends **one line per hard-stop**; entries are never rewritten.
The frame's *"armed at release"* sentence is a promise, not an arming — these two
lines are the arming, and they are what either party re-reads.

```text
SYMPTOM INVENTORY (Architect appends one line per hard-stop; never rewritten)
COUNT OF RECORD      = 19  (Steward, 2026-07-29; §5a backstop holds it)
ENTRIES              = 10
NEXT PREDICATE CHECK = 12th entry   (9th CONSUMED — answered below)
NEXT RESEARCH PULL   = #21, then #24, #27, …   (#18 FIRED 2026-07-29 — spent)
```

⭐ **#18 fired and is spent.** Hard-stop #18 was the `04cdce4e` CI rejection —
seven red checks, every one in `ken-cli` and none reachable from the
`ken-runtime` validation set that had just returned 562/0. The pull was
dispatched (`evt_65xt5kzp8t704`) and the advisory returned
(`evt_6980s92jgvf4h`), durable at
`local/rt-fnsplit-recur-port-hard-stop-18-differential-oracle-advisory.md`.
⚠ **The advisory did not settle the fork and was not asked to** — it stated the
discriminator (*is the required semantic fact erased before observation, or
present and mishandled?*) and left the ruling to the Architect.

⭐ **Its one durable finding, independent of which way the fork goes:** the
`ken-runtime` differential and the `ken-cli` parity suites **are not asking the
same observational question.** `RuntimeObservation` is limited to returned
ground values or traps, and the packaged decoder handles only scalar Int/Bool,
while the dependent suites compare against the full `EffectObservation` surface
(stdout, stderr, filesystem delta, terminal error, canonical effect trace,
terminal class, exit status). ⇒ ⛔ **The crate boundary is incidental**; a
`-p ken-runtime` green is not evidence about the richer observation, and
copying CLI assertions into a second runtime-local corpus is the wrong repair.

**Entries carried forward from the held chain (count of record = 8):**

1. whole-configuration specialization
2. flattened residual keys
3. `Debug` serialization as identity
4. helper identity coupled to env/control/layout contents
5. (5–8) the four entries accumulated after the recut, per the Architect's
   in-thread record — re-derive from the thread if a specific one is needed;
   ⛔ the **count**, not the prose, is what arms the trigger.

**Entries appended after arming:**

9. `DeclarationCall` source validation treated `StaticOriginId` as an index
   into raw walk-ordered `semantic_sources`, even though the exact source,
   origin, owner, target, and typed edge were already present.
10. trap-exit selection treated absent optional unit-frame handles as root
    authority, so a missing unit lane named the root's static exit path even
    though every emitted unit already had its fixed `TrapWord` ABI slot.

⭐ **The shared predicate already named for entries 1–4: a dynamic property
naming static code.** ⚠ If a new entry reduces to that same predicate, the
**emission port** is incomplete — it routes back to the port work and ⛔ does
**not** become a verdict of super-linearity.

### ✅ THE 9TH-ENTRY PREDICATE CHECK — MIXED; ENTRY 10 RECURS

**Entry 9 does not reduce to the shared predicate.** Its static name and all
facts needed to validate it already existed. The validator mishandled that name
as a position in a differently ordered collection. That is the
present-but-misindexed case: a localized validator representation defect, not a
dynamic surrogate for static code and not evidence of scaling behavior.

**Entry 10 does reduce to the shared predicate.** Absence of optional
unit-frame handles is an ambient construction property, not a generated
function's static role. Using it to choose the root exit makes that dynamic
absence name static root code and fails open: a malformed unit silently
acquires root authority. The repair therefore belongs to the existing
`RT-FNSPLIT-RECUR-PORT` emission-port work: bind an explicit closed root-versus-
unit trap-exit authority at function construction. It is ⛔ **not** a
super-linearity verdict and does not create an `RT-EFFECT-DIFF` obligation.

⚠ **Why both counters start mid-sequence.** This chain ran to **33 hard-stops
with the research trigger never fired**, because the count lived only as prose in
a resume state that neither party re-read. The catch-up rule (steward §5a duty 2)
re-anchored it at #11, then #15, #18. ⛔ Do not "reset to 3" because this node is
new — it inherits the held chain's count, and a reset would disarm both triggers
a second time.
