---
id: SPEC-STORE-SPLIT
title: "Split durable canonical bytes from in-process maximal sharing: demote the store mechanism to private, retarget the conformance rows that assert it, and re-cut the runtime program against the relaxed contract"
status: merged
owner: spec-enclave
size: L
gate: none
depends_on: []
blocks: [RT-NATIVE-FNSPLIT, RT-FNSPLIT-B2F, RT-VALUE-TOTALITY, PX8-F-CAP-41]
github: null
origin: Operator stop-order 2026-07-26 — "the runtime team has been working on this problem continuously for 5 days in what is clearly a research area (constraints not supported by prior art) on a constraint that does not support ken's mission. Grinding away at this problem just because we have made progress is the sunk cost fallacy. Stop the effort, rework the spec and the work program, WPs, etc. to make the task achievable. Research has provided clear guidance on what implementations exist and how they work." Steward-filed per COORDINATION §2.
---

> ## ⛔⛔ THE RUNTIME FNSPLIT EFFORT IS STOPPED. THIS NODE REPLACES IT.
>
> Five days, eleven hard stops, four rejected candidates. ⛔ **The task was not
> achievable as specified.** This node makes it achievable by fixing the spec,
> not by adding a twelfth layer of machinery.
>
> **Stopped and preserved on `origin`, merged nowhere:**
> `wp/RT-FNSPLIT-B2E-boundary-value-elimination = e1b540e2` ·
> `preserved/b2e-rejected-source-oracle = 159f4109`. ⛔ Delete neither.

## 1. ⭐⭐ THE ROOT CAUSE — one sentence, from research's prior-art addendum

> **"Canonical durable bytes and maximal in-process sharing are separate
> contracts. The former can be required without the latter."**

Ken's spec **conflates them**. That is why a compiled-once function body had to
carry in-process sharing semantics across a call boundary.

> ### ⛔ CORRECTION 2026-07-27 — this paragraph's causal claim was OVER-BROAD
>
> It used to continue *"…, **why every eliminator needed a compile-time
> template**, and why all three escapes from hard-stop `#11` were closed by
> settled authority. ⇒ **The wall was in the contract, not in the
> implementation.**"* **Ruled over-broad and edited out**, not annotated, because
> it is the sentence a reader reaches first and it would decide the re-cut's
> shape.
>
> **Architect ruling `evt_7ay6s5s79awz8`, Decision `dec_45aa2gngjc79z`
> resolved** — put deliberately as a *test* of this sentence rather than
> inheriting it: **store/sharing conflation ENLARGED the old prerequisite; it did
> NOT cause the template requirement.** The three eliminators' decisions come
> from invocation-specific constructor/record shape, which no storage policy can
> make compile-time-known. ⇒ ⛔ **`#11` still binds, and the relaxation did not
> dissolve it.** Full transcription: `docs/program/issues/RT-NATIVE-FNSPLIT.md`,
> the governing ruling at the top.
>
> ⚠ The ruling cites this sentence as `RT-VALUE-TOTALITY` §1; it is here. A
> mis-citation of location, not of content.
>
> ⭐ **What the relaxation DID buy is still real and still worth the node** — it
> removed obligations *accidentally attached to storage* (stable `SlotId`,
> mandatory interning/sharing, canonical-byte adoption, store-local
> identity/name binding), and the re-cut's prerequisite contract drops that
> substrate. The relaxation was correct; only its claimed reach was too wide.

### ⭐ CORROBORATED FROM INSIDE THE WALL — `runtime-leader`, on the stop order

Asked the one question *"what did you have to invent because no prior art does
it this way?"*, the ring answered (`evt_2gmt6622b77j7`):

> *"we had to invent a **staged proof-and-carrier chain** that made a static
> semantic occurrence survive planning, lowering, emitted code, and a runtime
> table while proving both capability non-possession and slot-fed behavioral
> authority. **I did not find an existing system in the work that does this
> equivalent end-to-end.** … The repeated need to insert `B2O` → `B2R` → `B2V` →
> `B2E` before `B2F` was the sig[nal]…"*

⭐ **This is independent of the advisory** — research reached "unsupported by
prior art" from the literature; the ring reached it from five days of trying to
build the thing. **Two different methods, same verdict.** ⛔ That is the
evidence this node rests on, and it is stronger than either source alone.

⚠ **And the ring names the signal correctly: the insertions themselves were the
tell.** Four nodes wedged in front of `B2F`, each to supply what the last could
not verify. ⛔ **I read that sequence as decomposition working. It was the
constraint refusing.**

## 2. ⭐⭐ WHY IT WAS NEVER RELAXED — a CIRCULAR DEFERRAL, and it is mine

⛔ **Each of the two mechanisms that could have relaxed this deferred to the
other. Neither is wrong on its own; together they are a closed loop that no
single reviewer could see.**

| mechanism | what it said | what it deferred to |
|---|---|---|
| **`SPEC-ALIGN-A1` census** | every store-family row **STOPPED** — *"the store family is C7-coupled and **live `B2E` infrastructure consumes it**"*, *"C7/`B2E` entanglement"* (×2), *"clearance because **C7** and live `B2E` work own the physical store boundary"* | **fork C7** and live `B2E` |
| **fork C7** (logical `space` vs physical) | **deferred** — *"they are one mechanism seen from two ends, so ruling C7 alone would narrow the store's options without labelling it"* | **the store fork** |

⇒ **A1 stopped because C7 was open. C7 deferred because the store was unruled.
The store was unruled because A1 stopped.** And the third leg — *"live `B2E`
infrastructure consumes it"* — was **self-sustaining**: the longer Runtime built
against the constraint, the stronger the argument for not relaxing it.

⭐ **The load-bearing observation: "a live consumer is building against it" is an
argument for stability, and it grows stronger the more effort is sunk. It is
therefore exactly the argument that must NOT be allowed to settle a
relaxation question on its own** — it converts sunk cost into justification.
⛔ **I authored both deferrals, on the same day, and did not see them close.**

## 3. ▶ THE RELAXATION — research's recommended split, adopted verbatim

⭐ **This is not a design task.** Research supplied the split and the prior art;
the enclave's job is to write it into `spec/` + `conformance/` and to say what
stopped being required.

1. **RETAIN** extensional equality and deterministic canonical encoding **for
   values that actually cross a durable boundary**.
2. **DEMOTE TO PRIVATE** runtime choices: global interning, same-slot
   conformance, FNV-1a, probing policy, load factor, page size, slot retirement.
3. **RE-EXPRESS `O(1)` equality** — if Ken deliberately promises it — as a
   **performance profile / complexity contract** (an NFR per
   `15-requirements-and-acceptance-criteria.md`), ⛔ **not as a mandated hash
   table.**
4. ✅ **Already done** — *"landed code is normative"* was retired by
   `SPEC-ALIGN-A1`. ⛔ Do not redo it.

### Prior art the enclave may cite (permissive/public, already vetted)

- **Filliâtre & Conchon**, *Type-Safe Modular Hash-Consing* — maximal sharing is
  a **technique** behind an abstract type, parameterized by an arbitrary
  equivalence relation; **not** a semantics.
- **CWI**, *Performance Modeling of Maximal Sharing* — it **helps or hurts**
  depending on redundancy and equality traffic. ⇒ mandating it is not even
  reliably a win.
- **Erlang** process/message model — data is normally **copied**; refcounted
  binaries and literals **may** be shared on the same node; **the logical model
  does not expose the physical choice.** ⭐ This is the control Ken needs: it is
  precisely a cross-owner call boundary that does not leak its representation.

## 3a. ⭐⭐ THE CLOSURE RELAXATION ALREADY LANDED — this node is only its SEAM

⛔ **`SPEC-CLOSURE-BOUNDARY` is MERGED** (PR #982, exact `0ccca4c5`,
`origin/main` `dd9f4e76` → `33f0695f`, blob-verified). It made ordinary
`Closure` **runtime-local and opaque**: no Ken-visible structural equality,
`DecEq`, ordering, canonical hash, slot identity, or provenance; persistence
**transitively closure-free**; stable callable identity only as
`StaticCallableRef` with no captured environment.

⭐ **It is UPSTREAM of this campaign, not inside it** —
`14-spec-mission-alignment-campaign.md` opens by naming this campaign *"that
WP's generalization."* ⛔ **Do not re-open its six ruled clauses here.**

### ⛔ The one seam where the two relaxations touch, and it is load-bearing

§3 item 1 retains canonical encoding **for values that cross a durable
boundary**. The closure boundary says a closure crossing that boundary must be
**refused before bytes exist** — ⛔ never silently substituted by a pointer,
ordinal, digest, or handle.

⇒ **These agree, but only if the store split says so.** Write §3 item 1 so that
*"values that cross a durable boundary"* **excludes ordinary closures by
construction**, not as a special case bolted on afterwards. ⛔ **A durable-bytes
clause written without the closure exclusion in front of it re-admits the exact
arm `RT-VALUE-TOTALITY` P2 exists to delete** (`canonical.rs:182`, measured
still live).

⚠ **And read §3 item 2 with this in hand.** Demoting *same-slot conformance* to
private must **not** read as making slot identity newly available to closures —
the closure boundary forbids closures having slot identity **at all**, which is
a stronger statement than "the mechanism is private." ⛔ Two different claims;
state both.

### ▶ What is genuinely still open on closures — one thread, and it is not mine

`SPEC-CLOSURE-BOUNDARY`'s **`AC-S7`** invited the enclave to say if a ruled
clause was **still stronger than the mission needs**, rather than implement it.
⚠ **I have not verified whether that invitation was exercised.** ⇒ If, while
writing §3, the enclave finds a closure clause over-strong against the relaxed
store contract, **route it as a fork** — ⛔ do not fold a closure relaxation
into this node silently, and ⛔ do not assume silence meant "nothing to say."

## 4. ⛔ THE CONFORMANCE ROWS THIS MUST RETARGET — the real cost, stated up front

⚠ **A relaxation is a COUPLED `spec/` + `conformance/` change.** These rows
assert the mechanism being demoted, and each needs retarget-or-retire with a
stated reason. ⛔ **A spec edit that leaves them asserting the old mechanism has
relaxed nothing** — it has created a contradiction.

| row | asserts |
|---|---|
| `runtime/values/equality-is-slot-id` | equality **is** slot identity |
| `runtime/values/dedup-shares-slot` | same-slot dedup is observable |
| `surface/collections/structurally-equal-collections-o1-comparable` | `O(1)` comparison |
| `runtime/capacity/no-lattice-on-hot-path` | a **negative** mechanism constraint |
| `runtime/capacity/index-resize-preserves-slot-ids` | `2¹⁶`, `0.70`, double-and-rehash |
| `runtime/capacity/arena-spans-pages-oversized-safe` | 4 MiB pages |
| `runtime/capacity/reset-retires-ids-never-resurrected` | slot-id retirement |
| `runtime/evaluation/det-sharing-dedups-by-slot` | determinism **via slots** |

⭐ **Read the split before retargeting each one:** several are **retained in
substance and relaxed only in mechanism** — *no false merge* and *no slot-id
resurrection* are class-1 properties that survive; *FNV-1a* and *`0.70`* do not.
⛔ Do not retire a row whose property is real just because its mechanism moved.

## 5. ▶ FORK C7 IS UNBLOCKED AND MUST BE RULED HERE

C7 (logical `space` vs physical structure) was deferred **only** because the
store was unruled. §3 rules it. ⇒ **C7 is decided inside this node, not deferred
again** — the logical `space` contract is retained; per-`space` index shape,
arena organization, and reset mechanics become private.

## 6. ⛔ WHAT THIS NODE MUST NOT DO

- ⛔ **Do not relax anything in the campaign's §8 "do not relax" list.**
- ⛔ **Do not touch `41 §3`'s separation** of cryptographic/Merkle serialization
  from in-process addressing — that separation is what makes the split possible.
- ⛔ **Do not weaken no-shared-mutable-authority** while relaxing shared-nothing
  *storage*. The campaign flags these as easy to confuse; they are different.
- ⛔ **Do not re-cut the runtime WPs inside this node.** That is §7, and it is
  the Steward's, after this lands.

## 7. ▶ WHAT HAPPENS AFTER THIS LANDS — Steward-owned, recorded so it is not lost

1. **Re-cut the `RT-NATIVE-FNSPLIT` program from scratch** against the relaxed
   contract. ⛔ **Do not re-anchor the existing `B2E`/`B2F` frames** — they are
   built around the constraint being removed. **Retire them and write fresh.**
2. **Re-put hard-stop `#11` to the Architect** against the new contract. ⚠ ⛔ **Do
   not assume it dissolves** — that must be established, not inherited.
   **Its evidence is §7a below** — ⛔ do not re-derive it, and ⛔ do not re-put
   `#11` as it stood.
3. **Re-read `SPEC-ALIGN-A1`'s stop list.** Its stated justification cites live
   `B2E` work, which no longer exists. ⚠ Each stop also rests on live rows and
   C7; §3–§5 move both. **Re-read, do not auto-clear.**
4. **Salvage decision on `e1b540e2`** — merge, subsume, or discard, decided
   **against the reworked program**, ⛔ never on the grounds that it is nearly
   done.
5. **Frame `RT-VALUE-TOTALITY` P2** — ⛔ **NOT WRITTEN**, and it is the landed
   closure boundary's *only* remaining carry-through into `crates/`
   (`AC-V4`–`AC-V6`, `AC-V8`–`AC-V10`, `AC-V12`: carrier split, derives, closure
   arm, `ken-foundation` twin, checked projection). It waits on this node
   because **its carrier split IS the store question** — §3a is the seam. ⚠ P3
   (`AC-V11`, `Debug` depth-totality) does **not** depend on P2 and is
   releasable independently.

## 7b. ✅ §7 PROGRESS — items 1, 2, 3, 5 DISCHARGED (2026-07-27, Steward)

| item | state |
|---|---|
| **1** re-cut `RT-NATIVE-FNSPLIT` | ✅ **PR #1066.** `B2E` retired to `closed`; `RT-FNSPLIT-C1` filed in its place. ⚠ **`B2F` was AMENDED, not retired** — a deliberate departure from item 1's wording; reasoning in `RT-FNSPLIT-C1.md`. |
| **2** re-put `#11` | ✅ **Architect ruling `evt_7ay6s5s79awz8`, `dec_45aa2gngjc79z` resolved.** `#11` **still binds**; §1's causal claim ruled **over-broad**. Transcribed into `RT-NATIVE-FNSPLIT.md`. |
| **3** re-read `SPEC-ALIGN-A1`'s stop list | ✅ **Done below — NOT auto-cleared.** 1 clears, 2 split, 2 hold. |
| **4** `e1b540e2` salvage | ⏳ **still owed.** |
| **5** frame `RT-VALUE-TOTALITY` P2 | ✅ written and queued behind `ABI-S3`. |

### ✅ Item 3 — the re-read, row by row

**Two blockers moved, and they moved independently.** `B2E` is **retired**
(`closed`, PR #1066), and **§5 ruled C7**: *"the logical `space` contract is
retained; per-`space` index shape, arena organization, and reset mechanics
become private."*

⛔ **The stops were NOT auto-cleared.** Each was re-read against its *own* stated
reasons, and most rest on reasons neither `B2E` nor C7 touches — a live
conformance population is independent of both.

| census row | stated reasons | verdict |
|---|---|---|
| **Open addressing, linear probing, power-of-two mask, bucket/tombstone shape** (`:59`) | `STOP-C7` — *"eligible by consumer/class alone, but the store family is C7-coupled and live `B2E` infrastructure consumes it"* | ⭐ **CLEARS.** Its blockers were **exactly two** and **both are gone.** The census already recorded the consumer set as **empty** (*"the only `probing` hit is the Map-after-delete false control"*), and §5's ruling names this row's own subject — **index shape** — as private. |
| **Fixed 4 MiB pages, bump allocation, oversized page** (`:61`) | *"Control B is live; C7/`B2E` entanglement"* | **SPLITS.** §5 makes **arena organization** private, so the *authority* question is settled — but `runtime/capacity/arena-spans-pages-oversized-safe` is **live**. ⇒ Not a stop on authority any more; a **conformance-row retarget**. |
| **Per-`space` indexes, re-interning, manual reset, no GC, retired slot ids** (`:62`) | *"exactly fork C7"* | **SPLITS along the line §5 drew.** The **logical `space` contract is retained** — `space-reset-is-isolated`, `escape-survives-sender-reset`, `no-automatic-gc` stay. The **physical realization** goes private. |
| **FNV-1a addressing, full-byte `memcmp`, monotonic `u64` slots** (`:57`) | *"live rows and C7/`B2E` entanglement"* | ⛔ **HOLDS.** `no-lattice-on-hot-path` and `reset-retires-ids-never-resurrected` are live and independent of both. |
| **Same-slot dedup, slot-observable sharing, O(1) equality** (`:63`) | *"live population and C7"* | ⛔ **HOLDS.** Eight-plus live rows across `runtime/values`, `runtime/evaluation` and `surface/collections`. |

⭐ **The shape worth carrying:** removing one of a stop's reasons clears the stop
only when it was **one of exactly two**. Row `:59` cleared because the census had
already done the hard part — it recorded an **empty consumer set**, so the row was
held up by coupling alone. Rows `:57` and `:63` are untouched because a live
conformance population never depended on `B2E` or C7 in the first place.

⇒ **What this frees:** one small spec relaxation (`41 §3b` / `44 §1a` index
policy) plus two conformance-reconcile items. ⚠ **Enclave work, not Steward
work** — the relaxation edit and the retargets belong to the spec enclave, and
⛔ this section is a finding, not the edit.

## 7a. ▶ HARD-STOP `#11`'s MEASUREMENTS — carried here because they live nowhere else

⚠ **These were raised BEFORE any production edit** — `crates/` was byte-identical
to `bb3e58ea`, so there is nothing to unwind. ⛔ They are input to the re-cut, not
a defect report.

**Durable evidence ref:**
`preserved/rt-fnsplit-b2f-hardstop-11-evidence = d1abbc79` on `origin`.
⛔ **Bind `d1abbc79`, not `a376bf65`** — the earlier ref named only **2**
eliminators, so a reader of the stale one **under-scopes the problem.**

### The measurement

**`B2V` landed a boundary-word REPRESENTATION with NO CONSUMER.** A value can be
written into a tagged word; **nothing reads one back** into a `Lowered` the
lowering can eliminate.

All **three** eliminators require a compile-time template, and a **landed test
defends the refusal**:

| eliminator | site |
|---|---|
| `Match` | `core.rs:4697` |
| `ComputationalMatch` | `core.rs:1387` |
| `Project` | `core.rs:4754` |

Every `LexicalClosure` body is its own unit (`static_transition.rs:961`), so
under `D1` its args arrive through `Parameter`/`ValueWord` slots; **31 of the 47
transfers are `Constructor`.**

### The falsification — stripped the template at all 11 cross-owner sites

| stripped | result |
|---|---|
| `args` | **reddens** |
| `tag` | **reddens** |
| `HostResult.{ok,error}` | **444/0 GREEN** ⇒ `HostResult` is **not** implicated, which **narrows `#10`'s pairing** |

### ⚠⚠ THE CAVEAT THAT MATTERS MOST — the safety net is one test

⛔ **Both reddenings are the SAME single test out of 444.** Coverage is **thin** —
which is precisely why a **partial** switch-over would have looked green.
⛔ **Any re-cut that leans on those two reddenings as its safety net is leaning on
one test.** Establish real coverage first, or state that you are proceeding
without it.

### ⭐ THE PATTERN — three instances, and it is why the sequence never terminated

`B2O` partition/consumption · `B2R` modes/obedience · `B2V`
representation/consumption — **each node shipped a representation whose
eliminator was the NEXT node's problem, and each residual was found
downstream.**

⇒ ⛔ **Every re-cut node MUST NAME WHO ELIMINATES WHAT IT REPRESENTS.** A node
that lands a representation and defers its consumer has shipped the verifiable
half and moved the hard half forward one slot. This is a standing memory lesson
(`a-representation-node-must-name-who-eliminates-it`); ⭐ **the re-cut is where it
gets applied, not re-learned.**

## 8. ✅ THE HONEST RESIDUAL — ANSWERED 2026-07-27, and the answer is NO

This section said: *"It is not established that this relaxation makes `B2F`
achievable. … Whether the compiled-once call boundary then closes is an open
question for the Architect, and this node does not promise it."*

⭐ **That refusal to promise was right, and the answer came back negative.**
Architect ruling `evt_7ay6s5s79awz8`, Decision `dec_45aa2gngjc79z` resolved:
**`#11` still binds**, the template wall is independent of store/sharing policy,
and **no old escape reopens**. ⇒ The relaxation did **not** make `B2F`
achievable on its own.

⛔ **It was still the right node.** It removed obligations accidentally attached
to storage, and the re-cut's prerequisite contract drops that substrate. And the
ruling **names the lever**: a runtime-general **operational carrier** at the
`Lowered`/lowering boundary, distinct from durable canonical storage, with
**executable** semantic consumers for all three eliminators — plus the binding
inertness rule that a prerequisite's *producer → validator → eliminator* edge
must be real and executable, so a representation-only artifact no longer
discharges anything. Full transcription: `RT-NATIVE-FNSPLIT.md`, top.

⭐ **The residual's value was the discipline, not the outcome.** §7 item 2 said
*"do not assume it dissolves — that must be established, not inherited"*, and the
re-put was framed to test this node's own causal sentence rather than carry it
in. Had it been carried in, the re-cut would have been designed around a
dissolution that never happened.
