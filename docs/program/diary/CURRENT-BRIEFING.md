# Current briefing (live — read this first on every Steward resume)

> ## ⛔ HOW TO READ THIS FILE, AND WHEN TO DISTRUST IT
>
> **`origin/main` outranks this file, always.** ⛔ If anything below tells you to
> do something `git fetch origin` shows as landed, **this file is stale and the
> repository is right.** Re-read fresh, in this order:
>
> 1. `git fetch origin && git rev-parse origin/main`
> 2. the LIVE block below — ⛔ **only** the LIVE block
> 3. the open tasks (⛔ do not re-derive priority from memory)
> 4. for what is HELD, DEFERRED, or WHOSE it is: **the node**
>    (`docs/program/issues/*.md`), its operative block — ⛔ never this file
>
> ⭐ **This file is a resume POINTER, not an archive. Git is the archive.** When a
> window closes its block is **deleted**, not demoted to a "superseded" section —
> ⛔ a superseded block left in the file gets read by someone, eventually.

> ### ⚠ REWRITTEN 2026-07-26 ~19:4xZ — 2866 lines → this. Read the bound.
>
> The prior content (~2700 lines of windows stacked back to 2026-07-21) is at blob
> **`c26ee67f29d42690f461d43fe15e21c2202a31df`** — `git show c26ee67f`. Nothing was
> lost; it was archived to git with this pointer.
>
> ⛔ **HONEST BOUND ON THE AUDIT: I did not read all 2866 lines.** I read every
> heading, the blocks claiming to be authoritative, and then **scanned** the
> remainder for sole-source markers, decision ids, held items, and preserved refs.
> ⇒ **That is a scan, not an exhaustive audit**, and its surface was my own idea of
> what "load-bearing" looks like. A reader who needs something from before
> 2026-07-26 should assume it is in `c26ee67f`, not that it was considered.
>
> ⭐ **What the scan found is why the rewrite was worth doing: two blocks that
> advertised themselves as authoritative were WRONG** (see *Corrections*), and a
> hand-maintained list of 6 preserved refs when origin held **26**.

## LIVE — 2026-08-05 ~11:2xZ · `D4a` rd 2 gated + run; lane cleared, shift not yet

**Verify `origin/main` before trusting anything below.**
`RT-CONTSRC-PRODUCER-LOCAL` is `active` in thread **`thr_6m43v75yndhtj`**.

### The one thing to do next

**`D4a` IS DONE — QA-APPROVED at exact `ac897a08`** (`evt_7yydatq78eqvg`), over
`52422da5`, lineage `97a4148b` (fixture + control) → `ac897a08` (record).
**Expect `runtime-leader`'s `D3b` disposition next**, which routes to the
Architect. ⛔ **Nothing is owed by me.**

⚠ **`runtime-leader` STALLED AT CAPACITY at ~11:47Z and I re-prompted it.** The
QA approval reached it, the model refused with *"Selected model is at capacity"*,
and ⛔ **the Codex turn ended SILENTLY** — approval unprocessed, ring frozen,
pane looking merely quiet. Re-prompt recovered it (`• Working`). ⭐ **Capacity is
transient and the seat does NOT self-recover; a re-prompt is the whole fix.**

⚠ **Earlier, the implementer was STRANDED, not idle** — composer held unsubmitted
text behind a finished turn. ⛔ **`handoff-gate-compact.sh` leads with a bare
`Enter`, which would have SUBMITTED that strand instead of compacting.** Hand-drove
one pane with a clear-first; `C-u` did not clear and the render stayed stale, so
a probe string proved the buffer empty before staging. ⭐ **The displayed line is
not the buffer.**

### `D4a` ROUND 2 — gated YES, run once, and where it actually stands

**The Architect gated my ruling YES** (`evt_65xkzqppdqdaj`), agreeing the fixture
supplies only the population. ⭐ **It added the half most easily lost: the `D4a`
mutation proves the INSTRUMENT; the `D3b` mutation proves the CONSUMER.** Once
the real `D3b` consumer exists, substituting the locator index for
`post_shift_index` must make the same fixture fail at the consumption boundary.
`D4a` passing does not discharge that. It also pinned condition 5 to my frame
correction: the fixture may add to `V`, **never to `R`**; re-census `C`/`V` at
the new base; any new `R` member or decline cause is a hard stop.

**Released (`evt_6cfjzp9yzvw8g`), run, nothing landed** — tree restored to
`52422da5`, `724/7` unchanged, branch free.

⭐ **THE ORIGINAL BLOCKER IS CLEARED.** Three lanes measured through the real
production path: `ConsoleRead` refused (the old hard stop), `ConsoleIsTerminal`
**visited but plans no seat at all** (it returns `Bool` before seat synthesis),
and **`ConsoleWrite` lowers, reaches the emission seam, and produces a
`CurrentLexical` record**. So a lawful lowerable shifted-population fixture is
reachable and the lane question is answered.

⛔⛔ **THE "NO SHIFT" READING WAS AN INSTRUMENT DEFECT, AND I PUBLISHED IT.** The
mid-round report said no nesting could shift the value, and I wrote its
scrutinee hypothesis into this file as the next bounded act. ⭐ **The first half
was true and the second half was wrong — the probe recorded ONE LINE PER SEAM,
but the seam carries a VECTOR of continuation inputs.** The shifted input was at
**ordinal 1** the whole time, in the shape already built:

| ordinal | binding | locator index | post-shift index |
|---|---|---|---|
| 0 | the enclosing `Match`'s case binder | 0 | 0 |
| 1 | the `Let`-bound host-effect result | 0 | **1** |

⭐ **No nesting search was needed and none should have been scheduled.** This is
exactly the recorded lesson *a short-circuiting probe measures the first cause,
not the set* — it reports one member of a vector and reads as a property of the
population. ⛔ **The `env_len` observations were the tell and I read them as the
obstacle:** growing to 5 while "the index" stayed 0 was the probe holding ordinal
0 fixed, not the program refusing to shift.

**MEASURED at the production planner and lowering path** (`recursive_port_process_compiles`):

```
post_shift_index = 1        locator.environment_index = 0
producer_env[1] = HostResult(v246, Ok, Err)   <- creation seat recorded v246
producer_env[0] = HostResult(v466, Ok, Err)   <- the decoy
```

⭐ **The decoy is a second `ConsoleWrite`** in the `Match` scrutinee's constructor
argument, matching on carrier, phase, lowering shape and constructor pair — **only
the SSA word differs**, which forces the oracle to be the SSA word rather than any
incidental discriminator.

⭐ **Oracle independence, the part `D3b` relies on:** lowering records the operand
it builds at the binder-creation seat, keyed by its own occurrence id with no
environment index in play; the seam half reads by index; the two join on
`binding_origin`, so **a wrong index breaks the join.** No planner re-walk, no
index arithmetic, no fixture-authored expected index, no direct construction.

**Mutations:** `UseLocatorIndex` and `SwapSlots` committed inside the control,
each asserting its own flip. ⭐ **`SwapSlots` is not redundant** — both indices
stay lawful and in bounds, so it survives a repair that merely bounds-checks.
Three more run by hand and reverted, including *drop the intervening binder*,
which reds loudly if the fixture stops being shifted — **act 1's gap, closed.**

⚠ **One trap a successor must not re-pay:** `ConsoleIsTerminal` looks like a free
win from the consumer list and is not — it returns `Bool` before seat synthesis
and plans no seat at all.

### THE `D4a` BIND — RULED, and round 1's stop. History; do not re-derive.

**The bind was:** the Architect required (`evt_tkzyc61rmd3`) a four-part proof at
one exact predeclared emission — a reaching `CurrentLexical` with
`post_shift_index != locator.environment_index`, the real operand at each index,
an **independent lowering-side** discrimination that does not re-run the planner
walk as its oracle, and a bounded wrong-index mutation. But the only durable
shifted fixture emits **zero seam records**: its `Let`-bound effect is
`HostOpV1::ConsoleRead`, absent from the fixed 13-element
`CRANELIFT_HOST_EFFECT_CONSUMERS_V1`, so lowering refuses it as an unavailable
lane before the seam. ⭐ **The fixture is shifted precisely BY the construct that
makes it unlowerable.** Every route to the seam was already prohibited, so the
required evidence was unobtainable and choosing what gives was a ruling.

**My ruling: a lowerable shifted fixture is AUTHORIZED**, as a second bounded
`D4a` extension round. ⛔ **No new node** — it folds. It lifts exactly one
prohibition (*"do not add a new population member"*) and nothing else. Full text
is now in the frame under checkpoint `D4a`; the three constraints are:

1. ⭐ **The fixture supplies the POPULATION; the MUTATION supplies the
   discrimination.** Building a fixture to exhibit the shift and then observing
   the shift measures nothing. ⛔ **No mutation row ⇒ no fixture**, and the
   outcome is a hard stop, not a green suite.
2. ⛔ **Do not inherit `D2b`'s effect lane.** The shifted value may be a case
   binder; those are already in `V`. Reaching for `ConsoleWrite` is analogy from
   the old fixture's shape, not derivation from the requirement.
3. ⛔ **`contsrc_d2_both_binding_kinds_fixture` is untouched.** Additive only.

**And `D4b`'s counts are now correct in the frame:** `C`=83 / `V`=80 are
**measurements at `e6d4f085`, not invariants** — the discharge condition already
said *post-admission* census. ⛔ **`R`'s three named causes are the invariant.**
A new fixture adding to `R` is a real finding; adding to `V` leaves the partition
intact.

⚠ **The grounding move worth repeating:** the prohibition I lifted was the
Architect's, and their own release named this outcome as *"the new boundary"* and
handed it back. A boundary is a scope call, so it was mine — but the soundness
axis stays theirs, which is why this is one confirming gate and not a
notification.

⛔ **`D4a` measurement is a distinct axis from `D4a` admission.** `52422da5`
already proves admission, a real depth-one predeclared emission, length
agreement and planner-side placement. **Equal indices make the pass-through
defect observationally identical**, and length agreement does not identify the
lowering value — which is why the Architect did not discharge it.

⛔ **THE BINDING ORDER IS FOUR, NOT MY TWO** (Architect `evt_7vc8zh0rvqyps`,
superseding my own `evt_11esqaep9awbs`):

1. **`D3a`** — non-lowering closure; both lowering consumers explicitly refuse;
   seam and pending population stay **visible**. **DONE, QA-approved.**
2. **`D4a`** — bounded admission and measurement. ⭐ **MAY BE DELIBERATELY RED.**
   It exists to produce real reaching producer-local emissions so nonzero-depth
   `CurrentLexical` correspondence can be measured. ⛔ A red here is the
   instrument working, **not** a regression to chase.
3. **`D3b`** — lowering closure, only after that evidence exists; seam deleted
   only when its closed population is empty.
4. **`D4b`** — closeout: `interned = V`, `declined = R`.

⛔ **My recut said `D3b` lands "with or after `D4`" and that was
under-specified where it counts** — `D4` as one unit cannot both *create* the
population and *prove* the partition, so it never named what produces `D3b`'s
evidence. `D4a` is that mechanism. ⭐ Same defect class as the `D1` clause: a
load-bearing sequencing term left ambiguous across two things.

⛔ **Option 2 is INVALID, not merely worse:** `D4` cannot safely admit before
the lowering consumers are explicitly fail-closed — hence `D3a` before `D4a`.

⛔ **The ABI ruling, so it is not re-litigated:** `AbiContinuationInputAuthority`
carries a **closed tagged provenance sum** — `EntryAbi { source_owner }` |
`ProducerLocal { binding_owner }` — keeping ordinal and affinity. A domain-total
bare owner was **rejected as lossy**: it collapses `EntryAbi { source_owner: X }`
and `ProducerLocal { binding_owner: X }` into the same value.

⛔ **"Any mismatch is a hard stop" is CORRECTED as overbroad**
(`evt_6p6vf0aqnjn3g`). Seam 1 must reject `CurrentLexical` at a specialization
emitter before indexing any operand run; a predeclared emitter must reject
`GeneratedContextCapture`. Applying the `CurrentLexical` comparison to a
specialization emitter is **itself a category error**. ⛔ Do not carry the old
phrasing forward from earlier posts in the thread.

⚠ **It woke on the mention, and that contradicts this file's own wake-asymmetry
claim below.** The standing note says a Claude implementer's mention push never
reaches the session. It did here. ⛔ Do not rely on either reading — **read the
pane before rousing**, which is what caught it. I did not establish the
mechanism, so this is an observation, not a retraction.

**`D2b` is QA-APPROVED at exact `7316e13a`** (`evt_3w4s25ta13hc4`), lineage
`e6d4f085` (base) → `2bd724cd` (record) → `7316e13a`. `D3` was released on top
(`evt_7rk80sgaq07fg`), the implementer scoped it and **made no edits**
(`evt_5pqxd21sw5m57`) — branch free, tree clean. ⛔ No merge is owed by me: the
node is mid-flight and the branch accumulates until the WP completes.

⭐ **This ring keeps stopping clean rather than half-applying, twice now.** Both
times it posted the scoping instead of holding it in context, so compaction cost
none of it. Read the stops as the frames working, not as under-delivery.

**`D3b`, `D4b`, candidate, `D6` closure, `#27`/case-emission, the call-result
SCC and downstream `D7` all remain held.** ⛔ `D3a` is DONE (QA-approved) and
`D4a` is ruled-but-ungated, not building — neither is merely held. WIP clock:
derive it from the latest reset event in the thread, never from a stamped
deadline.

### Where the node stands

| deliverable | state |
|---|---|
| `D0` `12d9612a` (zero delta), `D1` `77a24320` | accepted `evt_5zkydewv5kspb` |
| `D2` identity + value contract | accepted preservation at exact `e6d4f085` |
| `D2b` immediate availability | **QA-APPROVED** exact `7316e13a` — `evt_3w4s25ta13hc4` |
| `D3a` | **QA-APPROVED** exact `14b111ae` — `evt_62g4pganvk6f6` |
| `D4a` | **QA-APPROVED** exact `ac897a08` — `evt_7yydatq78eqvg`. `V` admitted `52422da5`; rd 1 hard stop `evt_7xwdw87mgf1q3`; rd 2 ruled `evt_28xx7t69z7j76`, gated `evt_65xkzqppdqdaj`, shifted fixture landed |
| `D3b` / `D4b` | held, in that order |

⛔ **There is no undivided `D4` any more.** The SET EQUALITY definition below
still governs — it is what **`D4b`** discharges; **`D4a`** admits `V` to create
the population `D3b` needs to measure.

**`D2`'s route was blocked once and corrected twice, and both stops were
sound.** `a5a6ce9b` stamped one blanket `ValueWord` contract across a
`ComputationalMatch` binder run that is **not homogeneous** — it is ordered
`[recursive IH binders, constructor argument binders, outer environment]`.
`5377d2ab` fixed the argument half by **reading** the carrier from the
scrutinee's shape instead of choosing one, and hard-stopped on the IH half
rather than defaulting. `e6d4f085` added the census and the fidelity
correction.

### `D4` is now SET EQUALITY, not closure

Unit: **one call to `exact_continuation_source_environment`**, identified by
program fingerprint + consumer owner + continuation origin + producer construct
origin + recursive position + closure origin.

| set | contents |
|---|---|
| `C` | all **83** `(identity, full required vector)` instances |
| `V` | the **80** fully closed under the current value-slot authority |
| `R = C \ V` | exactly **3**: `OPEN[ih-binder]`, `OPEN[let-value:Construct]`, `AMBIG2[let-value:If]` |

`D4` discharges when `interned = V` and `declined = R`. ⭐ **All 17 parity
instances are in `V`** — the population behind the six failing `D0` rows, and
the critical-path fact. ⛔ Call the three **outside-this-contract-domain
residuals**, never "unrepresentable" — the Architect corrected my wording, and
nothing claims a future authority cannot represent them.

⛔ **The program fingerprint is load-bearing.** `StaticOriginId`s are per-compile,
so without it edges from different fixtures collide and the census silently
undercounts: a first pass reported 58 identities of which six were collisions.

### `D2b` — why `D3` hard-stopped, and it is MY frame this time

`D3` reached the real emission seam and measured that **a producer-local value
has no member in the run the seam indexes.** Resolving its arm would need one of
three exits this node bans: widening the emitting function's input ABI run,
giving the seam a second non-ABI environment, or reusing a convention slot.

⛔ **The root cause is a `D1` clause I wrote.** It promised "an exact
emission-time locator into the environment that actually contains it" and
**never said which environment**. `D2` read it as the semantic environment and
populated a scope-relative `(environment_origin, environment_index)`; the seam
indexes a different space. **A load-bearing term left spanning two coordinate
spaces** — a different defect class from this campaign's earlier four, which
were false laws. Nobody could have discharged it as written. `D1`'s wording is
now corrected in place.

⛔ **Do not pin "`producer_env` is always the ABI operand run."** The Architect's
precision correction: the 61 records prove the **currently admitted** population
only, and there are **two** consumers — the retained-frame seat passes the
current `LoweringEnvironmentBinding` run, the detached/generated-context seats
read a function-local ABI operand run.

⭐ **The implementer withheld six of nine consumer sites** rather than land a
partial that would leave the seam no longer naming its own remaining work. That
is now frame law: no partial `D3`. It also **withdrew its own parity remark**
unprompted — it had reverted the probe while the parity run was in flight, so
the empty result could not distinguish "no parity emission reaches this seam"
from "the probe was not compiled in". An absence presented as corroboration,
caught and retracted by its author before anyone read the stop through it.

### The lesson from my own fork, because it will recur

I put a **binary** fork to the ring: zero IH-bearing edges ⇒ no node; nonzero ⇒
substrate first and `D3` waits. The census returned **1 of 83**, and the leader
applied my rule correctly — but the decisive fact had no cell in it: **the IH
edge is one of three non-closed positions from three causes, and a callable
contract closes exactly one.** So "every environment closed" was unreachable by
any node in the graph, with or without the substrate. That is a deadlock, not a
gate — the same thing checkpoint `1f` retired.

⇒ **State a fork by what would DECIDE it, not by the shape of the number you
expect back.** The census was still the right call; the defect was in how I
pre-committed to reading it.

### `RT-CONTSRC-CALLABLE-CONTRACT` — filed, `draft` on purpose

A real capability gap: production continuation inputs have **no callable domain
at all** (`BoundaryUseAvail::Callable` is `#[cfg(test)]`-only), and a recursive
IH is a compiler-only `StaticWorker` with no word, tag, descriptor or carrier.
Grounded in the Architect's ruling plus three source measurements.

⛔ **Held `draft` deliberately — it is NOT framing debt.** It closes 1 of 83
instances and 0 of 6 failing rows, and the one-release-ahead policy is already
satisfied by the six framed successors of `RT-DECL-CLOSURE-PORT`. Promoting it
would put an off-critical-path node in front of a reader looking for the next
kickoff.

### Two things about this node that must not be misread later

**None of `D2`'s stops was a sizing defect.** The heterogeneous-run defect was
caught at the gate; the IH boundary was found by building, which is the only way
it could have been found. Four *earlier* stops on this campaign were my frames
asserting laws the measured plane does not support — do not count these with
them.

⭐ **The implementer retired its own invented `ValueWord` blanket before the gate
ruled on it**, reported the IH half as three measured grounds rather than picking
a default, and declined to choose the graph. That is what these frames are
written to produce.

> ### The mistake I made at 07:36Z, because the shape recurs
>
> My frame-correction post `evt_270c4gk9trrmv` carried the line "`D2`-`D4`
> remain unreleased pending the Architect's gate". **It was already false when
> it landed** — the gate and `D2`'s release had posted 80 and 57 seconds
> earlier, and I had composed against the state I last measured. The Architect
> caught it in under a minute (`evt_1g2ssacct76tq`); corrected at
> `evt_7rbseqb0xnsaq`.
>
> ⇒ **A post whose subject is NOT release state must not assert release state.**
> A status claim carried as background inherits the message's authority and
> escapes its scrutiny. Re-read the channel immediately before posting anything
> that names what is released.

### The three frame corrections, so they are not re-litigated

From `evt_1srfqjmkp5eh8`, all published:

1. **`D3`'s consumer count was 3 in prose, 10 measured.** Frame now sizes `D3`
   from the in-tree seam function `entry_abi_pending_producer_local`, not from
   frame prose. **A frame-side count of a code-side population goes stale the
   moment the code moves.**
2. **`AC-1`'s six red rows are TWO populations** — the `AC-1` row refuses at
   `Match: scrutinee is not a constructor value`, the other five at
   `ComputationalMatch: ...`. Greening the five does not discharge `AC-1`, or
   the reverse. Invisible in `1 passed / 6 failed`.
3. **`AC-5` is pinned to `D4`.** It guards broad admission changing the interned
   population; before `D4` admits anything that condition is unreachable, so an
   earlier "controls green" report is true and meaningless while reading as
   cleared risk.

⚠ **A live-verb grep nearly reported this ring idle.** The implementer's footer
read `✻ Actualizing… (1m 9s)` — a verb absent from the tick's pattern list, so
the sweep printed a blank status. **The busy-check is wrong in both directions
and the verb list is open-ended: a missing verb reads exactly like idle.**
Resolve any blank or `(no-footer)` status by reading the pane, never by
extending the pattern and trusting it.

⭐ **Confirmed open-ended, twice more: `Baked for 5m 51s` and `Grooving…` both
printed `(no-footer)`.** ⛔ Do not chase the list. ⭐ **The cheap independent
instrument is `ctx`, which the tick already captures: a ctx that ROSE between
two reads is work, whatever verb is rendering.** It resolved `Grooving` without
a second pane read.

- Kick (fresh root, its own thread): **`evt_7h92n2tr7pbrm`**.
- `D7` rescope-in-place, posted in `thr_3rx07jfewhjhf`: `evt_14a9cee7fkv2s`.
- Handoff gate ran on all three seats (all 0 ahead, 0 dirty, so the
  `reset --hard` was safe). **Confirmed:** implementer ctx 0% with skills
  restored, both Codex seats show `Context compacted`.

**The wake asymmetry is the thing to watch.** `runtime-leader` and `runtime-qa`
are **Codex** (`gpt-5.6-terra`) and woke on the mention via the tmux backend —
the leader was Working within a minute. **`runtime-implementer` is Claude
(Opus 5) and its mention push never reaches the session.** So the leader's
dispatch to it will not wake it either. If it sits idle at an empty composer,
rouse mechanically: `tmux send-keys -t moot-runtime-implementer -l "<one line:
run get_recent_context, pick up evt_7h92n2tr7pbrm; re-orient per CLAUDE.md>"`
then a **separate** `Enter`. A wake is not task routing and does not breach
Steward-never-to-implementer.

### The branch trap — RESOLVED 07:1xZ, kept because it recurs every release

**Confirmed clear by an independent instrument:** the handoff gate's own
post-compaction worktree read shows `runtime-implementer` at
**`179af863 (wp/RT-DECL-CLOSURE-PORT-typed-units)`**, so it is building on the
proved lineage, not bare `main`. No `preserved/` refs were created — nothing was
ahead. The description below is the standing hazard, not an open item.

**`179af863` is contained by exactly ONE ref —
`wp/RT-DECL-CLOSURE-PORT-typed-units` — and NO worktree has it checked out.**
All three runtime seats sit on their own `*/work` branches at 0 ahead of `main`.
⇒ The implementer must **explicitly check out that branch** before touching the
new node. If it starts on `runtime-implementer/work` it builds on bare `main`
without checkpoints 1/`1b`/`1c`, and a grep for its own `D7` symbols comes back
empty — which reads as missing work rather than as a wrong branch.

### What `1e` got wrong, and the defect is reusable

`1e` ruled the minimal scope was the host-effect-result slot alone. **Falsified**
(`evt_5ngh190h9b1k5`) and the design rejected by the Architect
(`evt_75k8cydbj5127`): every effect-bearing closure needs **two** `Open` inputs,
ordinal 0 an effect result and ordinal 1 a case binder, so the
effect-result-only population is **zero** and closing it moves no row.

**The defect: `1d`'s census recorded the DECLINING ordinal — the first `Open` —
and I read it as a REQUIREMENT census.** "6 effect edges = the 6 failing rows"
was a pair count short-circuited at the first `Open`, compared against a `161`
in a different unit. Corrected closure-edge census: **34 case-binder-only, 4
mixed, 1 `Construct`-only.** A first-failure statistic is silent about every
input after the one that failed, so it cannot support a minimality claim.

**Also settled:** there is no lawful ABI seat for a mid-body value — the
Architect closed all five exits. A producer-local value is a **third
availability class**, which is why this is a representation boundary and not a
missing enum arm.

### The rulings now standing

- **BROAD admission.** All newly representable candidates may intern, not the 4
  `D0` edges alone — the narrow option needs a real edge-selection authority
  with every cheap substitute forbidden. This **dissolves** route modality.
- **~34 edges newly intern**, changing emitted code on green programs. Expected;
  the per-row `D0` and `718/2` baselines are the control.
- **`D7` retired the clause** blocking candidate/QA/`D6`/`AC-4` "while the row
  stands unreached" — it could never be discharged by the node holding it.

### Four stops on this node were MY framing, not Runtime

checkpoint 1 (mislocalized), `1b` (`1/1/1`), `1c` (forward reading), `1e`
(first-`Open` as requirement). **The instruction to measure rather than comply
caught every one.** Keep writing frames that way. This is not a sizing problem
and should not be read as one later.

## SUPERSEDED — 2026-08-05 ~06:4xZ · D7 `1d` answered NEGATIVE; `1e` released

**`origin/main` at last check: `3eeeb5ed`** (the `1e` ruling, PR #1410; D7 frame
blob `b5c240e6`). Verify it; do not trust this line.

### The one thing to do next

**Nothing, until Runtime returns `1e`'s answer.** I ruled at `evt_2tsq017qgvtgh`
(06:41Z); `runtime-leader` released `1e` from exact `179af863` and the Architect
picked up the confirming gate — **both confirmed by pane transition, not just a
posted mention.**

**WIP audit clock: armed from the ~06:43Z leader release, so due ~07:43Z.**

### What `1d` settled, and what `1e` is

`1d` came back **negative over 1110 candidate records** (`evt_5kws532ac99c9`) —
no existing authority both proves the closure-`381` edge mandatory and supplies
an exact edge-local closed environment. Three results outlive the checkpoint:

- **`member=true` on all 612 declines AND all 489 interns** — `1c`'s finding at
  1101-row scale. Closure-level membership is retired as an edge-local
  predicate permanently, not provisionally.
- **`case_emission` is INAPPLICABLE, not insufficient** —
  `build_case_emission_plan` never iterates `ComputationalMatch`. A later node
  reaching for it finds nothing, and now knows why.
- The ring **declined a near-miss discriminator** ("`Open` because of an effect
  result") as confounded with corpus identity: `Effect` occurs in 0 of 1057 lib
  ancestor chains and 60 of 60 parity chains, so it separates two test suites.

**`1e` is folded into `D7` — there is NO new node.** `1d`'s "requires a separate
substrate node" was my prose, not a ruling; the preference order is relax, fold,
then cut, and `179af863` is not on `main`, so a separate node would branch from
an unmerged branch for no independent mergeability.

**Scope is minimal by the inventory's own counts:** the host-effect-result
`ContinuationInputSource` variant plus its ABI position — 6 edges, exactly the 6
failing `D0` rows. **The case-binder slot is OUT** — 161 edges in a corpus at
718/2, no failing row demanding them.

**I refused the second minting.** A route-modality authority's only source is
`1d`'s own distinguish-before-interning requirement, which was a constraint of
the projection-only framing that `1e` retires. Stated to the ring as a question
to measure, not a law — three of this node's stops were exactly that error. **If
they report it IS genuinely required, that one gets a real node.**

**`1d` hard-stops TO ME.** If the inventory finds that satisfying it would mint
a new representation, population, identity, or planner/ABI authority, that needs
a **separate substrate node** which the Architect expressly did not authorize —
and **the graph shape is my call, decided FROM the inventory.** Do not cut that
node before the answer arrives; deciding beforehand is creating a node on
speculation.

### Checkpoint lineage — each an accepted parent of the next

`6a09ed68` (population) → `727b51a1` (per-visit claim group) → `69c68e6e` (body
close) → `f6958b95` (operation-arm claim consumption) → `ae64f687` (lazy
exact-`SiteOperand` + carried exact-`Int`) → `50092c59` (ckpt 1: phase-bearing
capture edges + pre-emission gate) → `ca1c4418` (ckpt `1b`: two arity
coordinates) → `179af863` (ckpt `1c`: the interned-to-member converse).

`4ec5362c` is **preservation-only partial progress**, not an accepted `1b`.

### The standing shape of this node's stops — READ BEFORE RECUTTING AGAIN

**Three stops were MY frame asserting a law the measured plane does not
support**, not Runtime sizing or execution:

1. **checkpoint 1** — I localized the repair to the `#23` producer; the producer
   was correct and the refusal was at the generic `LexicalClosure` value arm.
2. **`1b`** — I demanded a nullary Host-`Vis` be `1/1/1`; the honest relation is
   source-seed `0` / emitted-template `1` / marker-consumer `1`, and forcing the
   seed to `1` moved five non-injecting rows off their `D0` text.
3. **`1c`** — I read generic member status as a planning omission; `Open`
   environment means *do not commit this specialization*, and the forward law
   would falsely reject 23 green programs.

⭐ **The per-row-never-a-count requirement caught #2 and #3.** A pass/fail total
reads `1/7` before and after, identically, and hides both. Keep it as a frame
requirement, never a convention.

**Still held:** checkpoint 2, candidate, QA, `D6` closure, `AC-4`, the
call-result SCC, and the `#27` / case-emission populations.

**WIP audit clock — DERIVE IT, do not read a stamped deadline.** A fixed
timestamp here goes stale on every reset event and then fires a spurious audit;
it needed rewriting twice in the first hour. At tick time, take the **latest**
of these in the WP thread and add 60 minutes:

- a kickoff or re-kickoff (leader release),
- an Architect audit, ruling, or review verdict,
- a candidate or checkpoint handoff,
- a genuine hard stop, or completion.

**A routine progress post is not a reset event.** Counting those makes the
trigger fire never while looking armed.

On this ring the resets have been arriving every 5 to 15 minutes, so **the
60-minute trigger has not come close to firing and probably will not while the
cadence holds.** That is the healthy case, not a broken detector — but it is
also why a stamped deadline was pure noise here.

**Governing base, do not let it drift:** continue only from the `70887529`
lineage. Rebase, merge, or cherry-pick of `fb8fd881`, `430798bf`, `548682c3`,
`42ccd8ec` is banned — they are competing historical implementations, and
importing them reintroduces the role/disposition-derived schema the host-effect
ruling ruled false (Architect `evt_` in `thr_3rx0`, 01:06).

### Frontier: one release ahead is SATISFIED

Every node whose `depends_on` names `RT-DECL-CLOSURE-PORT` is `ready` and has a
frame file in `docs/program/wp/`:

| successor | other unmet deps |
|---|---|
| `RT-SEED-CALL-PORT` | none — this is the immediate next release |
| `PX8-ERRID-ALLOC` | `RT-NATIVE-FNSPLIT` |
| `NATIVE-HANDLE-CARRIER` | `RT-NATIVE-FNSPLIT`, `RT-JOIN-DISPOSITION` |
| `RT-CONTSPEC-LEDGER` | `RT-CONTSPEC-ACTIVATE` |
| `RT-DESCENT-RETIRE` | `RT-SEED-CALL-PORT`, `RT-PRODUCER-MATCH-PORT`, `RT-RECURSOR-TRANSPORT` |

`RT-SEED-CALL-PORT`'s fixed-input blobs are pinned at `origin/main = 14c3c5f7`
(2026-07-29) and are stale by construction — D7 rewrites `core.rs` in front of
it. **The frame says so itself and instructs re-pin at pickup.** That is
shovel-ready, not framing debt; do not re-pin the numbers and call it a
re-measurement.

### Lane state

| ring | state |
|---|---|
| **Runtime** | building — `RT-DECL-CLOSURE-PORT` D7 effect-seat slice 2 |
| **Kernel · Verify · Language · Ergo · Foundation · Spec** | idle, awaiting Steward kickoff — the fleet's single-threaded posture, not a stall |
| **Doc** | stood down after `DOC-PROGRAM-WAVE-RECONCILE` merged |
| **Architect** | serving the Runtime ring; last act was the identity acceptance of `70887529` |

**Tracker statuses reconciled 2026-08-05 ~07:45Z** — four were wrong against the
generator's own legend, where `active` means **a team is building**:

| node | was | now | why |
|---|---|---|---|
| `RT-CONTSRC-PRODUCER-LOCAL` | `ready` | `active` | it IS in flight |
| `KERNEL-NESTED-IND` | `active` | `ready` | deps met, framed inline, no seat |
| `SPEC-MISSION-GROUNDING` | `active` | `ready` | three ACs open, no seat |
| `SURF-SPACE-CELLS` | `active` | `draft` | P1 landed, P2 residual unframed |

**Both of the two nodes that argued for `active` in their own prose used it to
mean "not merged".** `SURF-SPACE-CELLS` said it stays `active` "so a reader
cannot mistake a merged phase for a merged node"; `SPEC-MISSION-GROUNDING` said
the AC reconciliation "is the reason it is `active` rather than `merged`." The
anti-merged signal those blocks wanted is carried by the blocks themselves. Both
operative sentences are rewritten, not appended to.

⇒ The releasable-frontier list now shows `KERNEL-NESTED-IND` and
`SPEC-MISSION-GROUNDING`. **That is accurate and is not a kick order** — the
single-threaded hold is a release decision, not a dependency, and the tracker is
generated so it cannot carry the hold. `SPEC-MISSION-GROUNDING` in particular is
**not** releasable by me: `AC-M3` names a pass `COORDINATION §10⁻a` forbids the
Steward to request.

### Unlanded finished work — research, 4 days old

`wp/research-kernel-extension-assessment` @ `0c450267` (2026-08-01) carries
`research/kernel-extension-assessment.md`, 746 lines, absent from `main`. No
`git_request` for it reached me. **Its path is neither `library/` nor a Steward
route, so the fail-closed predicate sends it to the Architect** — who is
currently the Runtime ring's reviewer. Do not spend that seat on it mid-slice;
bundle the routing question with the next Architect contact, or ask research
whether anything is owed.

### My own transport — fixed by the 01:48 restart

The seat now runs the original flagged process
(`--dangerously-load-development-channels server:convo-channel`), not the
unflagged `bg-pty-host --fork-session --resume` fork that silently dropped every
mention. Channel subscription confirmed on `spc_4q7g0se87rgje`. **The
generalization I posted at 01:35 — that `runtime-implementer` shared the
defect — was wrong and is retracted at `evt_` 01:45; route to it normally.**

### ⛔ OWED TO THE OPERATOR — four items, none self-resolvable

1. **`MAP-TRANSPORT-CODEC` candidate 3** — a wire format for a **non-Ken peer**.
   Candidates 1 and 2 were measured and answered *no*; this one is a **roadmap
   call and is not answerable from the repository.** The node is closed
   `not-needed`; if this comes back *yes* it reopens with a fresh frame.
2. ⛔ **`SPEC-MISSION-GROUNDING` `AC-M3` names a pass I am forbidden to request.**
   The AC says the adversary refutation pass is owed; `COORDINATION §10⁻a`
   forbids the Steward from asking the adversary to hunt anything. **Two
   operator-authored artifacts conflict.** Needs the operator to dispatch the
   adversary directly or re-route the AC. **Raised, unanswered.**
3. **T3 / `Property`** — there is **no `ken test` subcommand** (`ken-cli`
   dispatches `repl|run|check|native-build|fmt|version|help`) and **no spec
   chapter for the CLI at all** (task `#143`). `Tooling/Testing/Property.ken.md`
   exists but is deterministic finite checks — no randomness, shrinking, or
   seeds, deliberately. ⇒ T3 is blocked on a **design input**, not on code.
4. **Linux ABI** — `ABI-S3`'s three ops landed `RepresentedUnavailable` by
   design and **no Track-A node promotes them**, so `§6`'s exit condition is
   unreachable through `ABI-A1/A2/A3`.

### ⛔ Still operator-HELD

**`DOC-ATTEST-LIVING`** — ⛔ **do not release, do not re-ask.** Node:
`docs/program/issues/DOC-ATTEST-LIVING.md`.

### ⛔⛔ THE `integrator` GHOST — do NOT chase it again

**The seat was RETIRED by PR #1052** (50 files). It has **no tmux session, no
entry in `.moot/actors.json`, and no playbook** — `agent/playbooks/federation/`
holds only adversary, architect, librarian, research, steward.

⚠ **But `orientation()` and `list_participants` still show it**, carrying a
**stale stored status** — *"PR #365 green on head `befc2dc4`, awaiting Steward
routing."* ⭐ **That reads exactly like a live seat blocked on you, and it is
not.** I treated it as a real open query, investigated it, and posted a routing
reply to a seat that cannot read. No one was waiting; nothing was owed.

⇒ ⛔ **A retired seat's last status is indistinguishable from a live seat's
current one.** Before acting on any participant status, check for a tmux session
**and** an `actors.json` entry. Convo has no tool to remove a participant, so
this ghost persists — it is operator/convo-admin item, already raised.

For the record on its content: `befc2dc4` is **on no ref at all**, and
`scripts/scripted-pr-automerge.sh` **is** on `main` (blob `76afaf31`) — the
capability landed and I run it every publish. ⛔ Do not re-propose that commit;
its `COORDINATION.md` / `04-git-and-integration.md` / `steward.md` versions are
from 2026-07-08 and re-landing them would revert weeks of work.

### ⭐ Traps measured this window — positional, so they will recur

1. ⛔ **The decisions read path field is `decision_id`, NOT `id`.** `d.get('id')`
   returns `None` for **every** record, so a lookup reports **NOT FOUND for a
   decision that exists**. I was one step from blocking a merge on this. ⇒ Always
   run a positive control against a decision you know exists.
2. ⛔ **Any non-doc-only publish MUST be `run_in_background: true`.** Full-CI
   polling exceeds the Bash tool's 600000ms cap and the tool kills the publisher
   (exit 143). Doc-only finishes in ~2 min and is safe in the foreground.
3. ⭐⭐ **"Awaiting merge" may already BE merged** — twice this window (Ergo
   `a85c0dc5`, Kernel `5396f9a7`). The publisher **squashes**, so every ancestry
   check says unmerged. Only a blob diff of the candidate's **own** paths against
   `origin/main` discriminates. ⚠ And a path-drift check *actively misleads*: both
   Kernel paths read as "changed on main since the approval base," which normally
   means a stale base — here the thing that changed them was the candidate's own
   already-landed work.
4. ⛔ **Require the exact BRANCH as well as the exact SHA.** Kernel had
   `wp/KERNEL-NESTED-IND` (D1a, `e685570c`) and `wp/KERNEL-NESTED-IND-D3` (the
   approved D3a) live at once, and the approved SHA was **not** an ancestor of the
   branch matching the node name.
5. ⚠ **A node annotation written at merge time can gate another team's frontier.**
   I wrote *"`RT-VALUE-TOTALITY` stays `active` for its remaining scope"* — it had
   none, and that stale `active` was the last unmet `depends_on` of
   `RT-FNSPLIT-C1`. It would have idled Runtime behind a complete node. It
   surfaced only because `runtime-leader` refused to infer a branch and asked.

## ⛔ CORRECTIONS — two claims the old file made that were FALSE

⭐ Both were **time-varying state wearing a permanent-looking hat** — the exact
failure the heartbeat prompt bans. Recorded so the *shape* is recognisable, not
just the instances.

### 1. ⛔ "ARMED COUNTERS — the SOLE count of record" was stale AND retired

It read `RT-NATIVE-FNSPLIT: hard-stop 10 · next research pull #11` and `Architect
production blocks: 6 · next check #9`. **Both numbers were behind**, and the chain
they counted **is retired** — the operator stopped the FNSPLIT effort on
2026-07-26 and `SPEC-STORE-SPLIT` replaces it.

⛔ **A counter calling itself "the SOLE count of record" is the worst thing to
leave stale**: it invites a reader to trust it *instead of* measuring. ⇒ **There
are no armed counters now.** When the re-cut program exists, its node owns its
counts.

### 2. ⛔ "TRANSPORT — convo MCP mostly DEAD" is FALSE

The old block claimed only `set_interval`/`subscribe` survived and routed all
reads through scratchpad HTTP scripts. **Measured across this entire session:
`orientation`, `list_decisions`, `post_response`, `list_participants` all work
over MCP.** Tracked as task `#110` because **the heartbeat prompt still repeats
the claim.**

**What IS true — the part worth keeping:**

- ⛔ **NEVER call `mcp__convo__get_transcript`.** Its `limit` does not bound the
  response and it takes the stdio connection down with it. Operator prohibition;
  fleet law in `AGENTS.md`.
- ⚠ **Mentions arrive TRUNCATED** — a doorbell, not a message. Fetch full text via
  the HTTP read path, with **your own** credential.
- ⚠ **`list_decisions` can exceed the result cap** and spill to a file — grep the
  file rather than retrying the call.
- ⛔ `claude mcp list` reporting `convo: ✔ Connected` **is not evidence** — it
  health-checks a fresh process.

## ▶ Preserved refs — ⛔ QUERY LOCALLY. `origin` carries `main` ONLY.

> ### ⛔ THIS SECTION WAS FALSE AS WRITTEN. Both halves.
>
> It said *"Origin holds 26"* and gave `git ls-remote origin
> 'refs/heads/preserved/*'` as the query. **Operator ruling, 2026-07-26:** *"clean
> up all of the non-main branches at origin."* ⇒ **All 63 non-`main` origin
> branches are deleted.** That `ls-remote` now returns **nothing**, and a reader
> running it would conclude the work was lost.

**Measured 2026-07-27 — the query is local, and the population is larger, not
smaller:**

```sh
git for-each-ref 'refs/heads/preserved/*'    # 78 refs
git ls-remote --heads origin                 # refs/heads/main — and nothing else
```

⭐ **A branch on one local ref is the NORMAL state of preserved work, not an
exposure.** ⛔ Do not raise an unpushed ref as a finding.

⛔ **AND THE "EXISTS NOWHERE ELSE" CLAIM WAS WRONG ON EVERY ITEM IT NAMED.** Each
was checked at `origin/main = a1e29284`:

| the old claim | measured |
|---|---|
| `preserved/b2e-rejected-source-oracle` = `159f4109` | ✅ **present locally at that exact SHA** |
| `wp/RT-FNSPLIT-B2E-boundary-value-elimination` = `e1b540e2` | ✅ **present locally at that exact SHA** — ⛔ delete neither |
| `preserved/rt-fnsplit-b2f-hardstop-{9,10,11}-evidence` | ⛔ **no local ref of that name exists** — and it does not need to. Hard-stops #9/#10/#11 are all on `main`, across **12** files (`RT-FNSPLIT-B2{E,F,O,R,V}.md`, `RT-NATIVE-FNSPLIT.md`, `RT-VALUE-TOTALITY.md`, the B2O report + predictions, two WP frames, `diary/2026/Jul/25.md`). `bce75fec` is literally *"make hard-stop #11's evidence durable"*. |
| `preserved/architect-state-*` | ⛔ **wrong prefix** — the refs are `preserved/architect-work-*` (5 locally). A ref name you cannot resolve is not a backup. |

⭐ **The transferable part: a "this exists nowhere else" note is a claim about a
population you did not enumerate, and it decays in both directions at once** — the
copy you were protecting had already landed in the repo, while the ref name you
recorded it under never existed. ⇒ **Re-derive from `for-each-ref` and `git grep`
on `main`; never from a hand-kept list of what is precious.**

## Operator rulings — 2026-07-21 ~12:45Z. ⛔ SETTLED, do not reopen.

⭐ Kept inline deliberately: this is law, and a settled ruling is a **fixed input,
never a question to re-ask.**

- **No "ratification."** The Linux ABI II charter is a **planning document, not a
  commitment.** Nothing outside the project depends on our timelines. ⛔ Do not
  re-raise status-correction as a decision.
- **Where anticipated and done diverge, fill the gap first** — hence
  `docs/program/10-linux-abi-completion.md`.
- **L2-1: no cross-compilation. CROSS-PLATFORM IS INDEFINITELY DEFERRED**
  (restated 2026-07-21 after I re-raised it). Manifest v2 is family-scoped and
  generated, **not** cross-target.
  ⛔ **This ruling ALREADY ANSWERS any non-linux finding** — do not route one back
  as a scoping question. Record such findings as *observations against a deferred
  lane* and stop.
- **L2-0: all desirable, nothing deferred.** All nine `RepresentedUnavailable`
  operations get promoted.
- **Timing, timelines, and budget are the OPERATOR'S domain.** ⛔ Do not reason
  about schedule or cost.
- ★ **My lane is token efficiency in terms of delivered work.** That is the axis
  to optimize and the one to report on.

**Standing test policy (operator, 2026-07-26):** *"Test oracles that assert facts
about source code, catalog, or documentation lines are an invitation for failure
and delay. Tests should focus on behavior."* ⇒ Executable form: **"does an edit
that changes nothing about how any program behaves make this test fail?"**

**Standing gate policy (operator, 2026-07-26):** the library currency ledger is
generated **at version release points**, ⛔ **not enforced per merge.**

**⛔ `origin` CARRIES `main` ONLY (operator, 2026-07-26; restated 2026-07-28).**
A branch living on one local ref is **normal** and is never a finding. ⛔ No
durability sweeps, no pushes of WP or seat branches, no ring reporting an
unpushed ref. The publisher's own candidate-branch push stays — that is how it
opens a PR.

**⛔ THE `integrator` SEAT IS RETIRED (operator, 2026-07-26).** *"remove any
references to the integrator. that seat was retired weeks ago."* ⇒ Every operative
reference is gone as of PR #1052 (`a1e29284`, 50 files) — PR template, CODEOWNERS,
`ci.yml`, four devcontainer files (including a **functional** `ctx-nudge.sh` case
arm), `COORDINATION.md`, `04-git-and-integration.md`, 40 WP frames, the roster
(29→28), git refs, worktrees. ⭐ **The chronicles keep the word deliberately** —
`docs/program/diary/`, `agent/memory/MIGRATION-LOG.md`,
`docs/program/ds-campaign-judgment-log.md` (17 files, 501 occurrences): there it is
a true account of what the process **was**. **Instructions get corrected; records
stay records.** ⚠ One residual is not mine to clear — the convo **participant**
still exists; see the LIVE block's operator-owed list.

**Canonical width: 96 (operator, 2026-07-26).** *"re 88 v 96. 96 is what it should
be. It was an incomplete revision, apparently."* ⇒ `spec/30-surface/31-lexical.md`
and `CANONICAL_WIDTH` are correct; `conformance/` is the stale side.
`SPEC-31-WIDTH-ERRATUM` reconciles it. ⛔ Do not re-argue the value.

## ▶ Where durable law lives — ⛔ do not restate it here

⭐ **The old file's real defect was restating durable rules inside a diary.** A
rule copied into a briefing drifts from its source and then contradicts it. ⇒
**Point, never copy.**

| what | where |
|---|---|
| federation law, §2c handoff gate, §14 merge gate | `agent/COORDINATION.md` |
| my playbook, publish discipline | `agent/playbooks/federation/steward.md` |
| hard-won operational lessons | `agent/memory/` (`fleet` + `enclave` + `roles/steward/`) |
| model tiers | `agent/MODELS.md` |
| reasoning charter | `docs/PRINCIPLES.md` |
| ⛔ no local `--workspace` builds — CI only | `agent/COORDINATION.md §12` |
| build status against the DAG | `docs/program/IMPLEMENTATION-PROGRESS.md` |
| spec status | `spec/SPEC-PROGRESS.md` |

## ⚠ Standing traps — only the POSITIONAL ones

⭐ Each is here because it fires **at a specific command**. That is the whole test
for belonging in this file rather than in `agent/memory/`.

- ⛔ **Verify landed content by BLOB IDENTITY, never ancestry.** The publisher
  squashes, so an approved SHA is correctly *never* an ancestor of `main`.
- ⛔ **Verify every object you NAME exists at the base you NAME** —
  `git cat-file -e <base>:<path>`, and quote the blob (§2c step 5b).
- ⛔ **`git diff --stat` always exits 0.** Use `--quiet` for an emptiness test.
- ⛔ **The publisher's exit code is the LAUNCHER's** — confirm it exited *and* that
  `main` moved.
- ⛔ **Never `git fetch` while the publisher is inside its merge→verify window** —
  `refs/remotes/origin/main` is shared across ~70 worktrees.
- ⛔ **Never `pkill -f`** (matches your own shell) · **never `git stash`**
  (`refs/stash` is shared) · **never `git checkout <ref> -- .`** (reverts
  uncommitted edits worktree-wide).
- ⛔ **A probe truncated before its filter is not a measurement.** Search the full
  stream; truncate the RESULT.
- ⛔ **Never dump `.moot/actors.json`** to learn its shape — use
  `scripts/moot-actor-id.sh <role>`; the schema-discovery step is what leaks a
  key. Look up a participant id **at post time**, never from memory.
- ⛔ **`steward/work` is stale immediately after every publish** — reset onto the
  squashed `main` before writing anything new.
- ⛔ **A `--doc-only` merge can redden `main` and is structurally unable to notice.**
  After one, **enumerate consumers of the touched paths** — attestation ledger,
  measured-token censuses, source-text oracles. This is how `95bc855c` broke three
  things and reported none.
