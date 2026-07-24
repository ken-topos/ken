---
id: RT-NATIVE-FNSPLIT
title: "Native backend: bound per-function lowering growth to O(n) — helper identity is a variable-width whole-configuration key (orig. single-Function VReg::MAX, since fixed)"
status: active
owner: runtime
size: TBD
gate: none
depends_on: []
blocks: []
github: null
origin: PX8-SPAN-PROV Phase 2 native reachability wall (runtime-implementer measured repro evt_7qhtk8w489am4; CV option-(c) ruling evt_77q2tc5dh1kzj; Steward scope ruling evt_7c160ej3bwz4; Architect means/layer ruling evt_7gkn3g4tsvgb9, 2026-07-23). Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## ⛔ READ THE VIABILITY RULING BELOW BEFORE THIS SECTION
>
> **The original root cause described immediately below — one Cranelift
> `Function` per process, hitting `VReg::MAX` — WAS FIXED and is GONE as of
> `b077eb7a`.** It is retained only as the WP's origin history. The live defect
> is a **variable-width whole-configuration helper key**, ruled 2026-07-24
> (`evt_3m1g3v4m2bj51`). ⇒ **Do not implement against this paragraph.**

### Origin (historical — superseded)

Discovered while closing PX8-SPAN-PROV Phase 2's native conformance matrices. The
native backend (`build_native_program`) **inlines the whole process ITree into one
Cranelift function**, so program size scales with the nested resource-bracket
structure. Measured wall (minimal repro, not assumed): **any program with four
nested resource brackets** fails to compile with Cranelift's
`Code for function is too large`, *before* producing any runtime outcome.

- 3-bracket programs lower fine (e.g. SP-A **freeze**, `px8f_buffer_native`
  writeAll).
- The 4th bracket is the wall. A native end-to-end write / precedence / slot-reuse
  discriminator inherently needs four brackets (readable source to mint spans +
  writable dest + two buffers, or release/realloc), so it cannot compile today.

This is a **general native-backend limitation**, independent of provenance — it
blocks *any* 4-resource-bracket Ken program on the native path, not just
PX8-SPAN-PROV's tests.

**Root cause (Architect, `evt_7gkn3g4tsvgb9`, independently reproduced on
`b717bf64` — 175.72 s lowering then object-emission failure):**
`compile_expr_into_module` lowers the whole checked process entrypoint into a
single Cranelift `Function` and calls `define_function` once. Cranelift 0.113.1's
virtual-register allocator emits `Code for function is too large` when that one
function reaches `VReg::MAX` (2²¹−1 virtual registers). So a small, valid Ken
process expands past a backend implementation limit **before execution** — a
native-codegen completeness defect, not a span-provenance / ABI / conformance-
harness defect.

## ⚠ SCALING GATE — operator directive 2026-07-23 (evt_4btfhwqhah1ye)

**"SP tests complete under a timeout" is NOT acceptance.** After the recut, a
4-resource-bracket program still costs ~103 CPU-s / ~4 GB to compile — the operator
ruled that unacceptable without understanding the scaling law. RT-NATIVE-FNSPLIT does
**not merge** until:
1. **Empirical scaling harness (Runtime, permanent tests):** minimal programs at
   **n = 3,4,5,6,7** nested resource brackets, each measured under a bounded
   harness (`prlimit`, fail-safe) for compile wall-time, peak RSS, and internal
   counts (distinct interned semantic states, defined helpers, total
   DFG/instr/blocks). Report the table + fitted growth curve.
2. **Analytical scaling model (Architect):** predicted order of growth vs. n;
   whether 103 s/4 GB @ n=4 is bad-constants-on-**O(n)** or residual
   super-linearity (→ further mechanism gap). Must be
   **research-grounded** (research dispatch `evt_62fqpe7pfvym4`).
3. **Verdict:** either (a) empirically+analytically **linear O(n)** + a plan
   to reduce the constants; or (b) a **research-supported** reason growth is
   inherently super-linear + an explicit operator ceiling/acceptability
   decision.

Gates the [[NATIVE-HANDLE-CARRIER]] fast-follow + [[PX8-F-CAP-41]] too.

## ⛔ ARMED §5a RESEARCH-CONSULT TRIGGER — the count of record

**Steward holds the authoritative count** (steward playbook §5a duty 1). The
Architect re-derives its own count across compactions; **on any disagreement
this line wins.** Re-read this line on every hard-stop.

```text
RECUT CHAIN (live, from kickoff evt_2kgfmmeeh2x7w, 2026-07-24)
hard-stop count    = 3   ← PULL FIRED AND CONSUMED (advisory evt_rwqb8ear89wx)
  #1 = Architect amendment ruling evt_6dpb96kn1583f (2026-07-24) — Phase 1's
       held-checkpoint premise is FALSE; census returns could_not_determine;
       empirical gate moves to the recut in two closed boundaries. Frame
       amended by the Steward in response. NO research pull due (< #3).
  #2 = Boundary B static-to-semantic bridge, raised by runtime-implementer on
       WIP d4df9278, ruled by the Architect at evt_2jt1s5r7c1g2z (2026-07-24) —
       Boundary A's plan is closed and constant-width but retains NO static
       helper -> semantic-body association, while the retained emitter still
       allocates FuncId from PartitionSemanticStateKey (vectors/strings/
       recursive keys). Ruling: extend A with an OUT-OF-LINE semantic
       descriptor plane keyed by the existing planned node/edge IDs; planned
       IDs are the sole code identity; no discovery-order or hashed-key
       fallback. NO research pull due (< #3).
  #3 = Boundary B grounding found GENUINELY UNREPRESENTED activation-
       independent semantic classes, raised by runtime-implementer at
       evt_21yr288qkpb92 on clean checkpoint ed54b17e (2026-07-24).
       SourceKont is not uniformly R (PartitionSourcePrefixKey carries LetBody,
       ApplyRecursorSelection, UnwindRecursorSegment, checked recursive/IH
       returns, selected-case return, terminal steps — these transform
       value/control and may own a body, so they are neither R nor
       authority-only edges). ProducerKont is not classifiable by action-name
       mapping (OrientedInvocationReturn, CheckedComputationalIHReturn,
       ScopeBodyReturn have independent control semantics; A has only R/W/T/C,
       and R is ruled to own no body). SourceArm bodies lose exact occurrence
       identity before reserve (cloned RuntimeExpr). Definition scheduling is
       still state-owned per whole semantic key.
       ⇒ This hits the ESCAPE HATCH in the Architect's hard-stop-#2 ruling:
       "add an explicit planner node/transition kind and RETURN BOUNDARY A for
       amended census and fresh review." The ring correctly refused to overload
       R/W/T/C, assign by discovery order, or retain first-activation body
       selection.
       ⇒ RESEARCH PULL FIRED (§5a). Architect ruling is gated BEHIND the
       advisory, at the implementer's own request.
NEXT RESEARCH PULL = hard-stop #6, then #9, #12, …

⚠ The count is at 3; the pull FIRED and is CONSUMED. Advisory delivered at
   evt_rwqb8ear89wx (Danvy/Nielsen defunctionalization granularity; Agda
   TTerm / Lean FnBody / Cranelift IR as closed-IR precedent; Maranget on why
   small-n affine tables mislead). Re-arm: the next pull is #6.

⚠ BOUNDARY B1 KICKED 2026-07-24 (evt_784nkjqzzbxn) under the fork-(b) ruling;
   ring compacted, drops verified. RT-PLANNER-DIAGNOSTIC-K closed at 36dd61f6.

⚠ BOUNDARY A IS MERGED (647a2e5b, retros in) BUT THIS WP STAYS `active` —
   B1 is in flight and B2 has not started. Do not flip this parent to `merged`
   on a boundary landing; it closes only when B1 AND B2 land and the operator
   scaling gate above is satisfied.

HELD CHAIN (closed, historical — does NOT carry forward)
hard-stop count = 33            (FROZEN)
cadence           = SUSPENDED   (viability ruling 2026-07-24)
```

✅ **RE-ARMED 2026-07-24** when the recut frame was kicked to the Runtime ring.
The old count is **frozen and does not carry**: it counted the *held*
representation, and the recut replaces the machine those stops were about. The
recut chain starts at **zero** and pulls research on **every 3rd** hard-stop.

⛔ The frozen 33 is retained above as history only. #34 was raised but is
**evidence, not a ruled stop**, which is why the held count stayed at 33 rather
than advancing. Do not resume an every-3rd pull against the held
representation — that chain is closed.

**Advisories on record:** #24 `evt_5gshpmyb2ta79` · #27 `evt_7s6b3zg82n7n5` ·
#30 `evt_1stmfwh0tj5gm` · #33 `evt_3vr382mrv99pe` (requested by Architect
16:33:14Z; **transport-repaired by Steward `evt_d2b3vahe7khj`** — the request
carried no `mentions` array, so research was never notified; research ack
`evt_74gympwyk8q67`).

⚠ **Why this section exists.** This chain ran to **10 hard-stops with zero
research pulls** (operator, 2026-07-24) because the count lived only as a prose
list of fork numbers in the Steward's resume state and in a status string —
never as an armed `next pull = N` line either party re-read. The Architect's
self-trigger lapsed across its compactions and the Steward backstop lapsed with
it. A deep chain with no advisories on it is itself the tell that **both**
mechanisms have silently lapsed.

⚠ **Transport lesson, #33 (2026-07-24):** the Architect's own advisory request
is not self-routing. A request posted with an empty `mentions` array reaches
**no one** — research is a no-poll seat, so it re-oriented to "awaiting
dispatch" while the Architect sat `blocked-on-Research` indefinitely. **Steward
duty 3 is not optional: after any pull, confirm the research pane actually went
`Working`.** Delivery ≠ engagement (COORDINATION §2, §13).


## ⛔⛔ VIABILITY RULING 2026-07-24 — HOLD + REPRESENTATION RECUT (`evt_3m1g3v4m2bj51`)

**Operator-directed viability review (`evt_98j3z2n49bpg`), Architect ruling on a
research advisory (`evt_7p40c3x8cnwtm`). The hard-stop cadence STOPS HERE.**
Runtime is **held at clean `b077eb7a`** — the semantic/diagnostic checkpoint —
**until the Steward authors the recut frame.** No #34/#35 option is implemented
in the current machine.

### ⚠ THE ROOT CAUSE STATED AT THE TOP OF THIS FILE IS STALE — DO NOT BUILD ON IT

The "**inlines the whole process ITree into one Cranelift function** →
`VReg::MAX`" premise **is already fixed and gone.** At `b077eb7a`,
`core.rs:120–492` emits an exported root plus a **separate Cranelift function
per queued `PartitionWorkItem`**. **The 1,482-state / 1,525-edge red is NOT the
single-function failure.** Anyone reading this file top-down would otherwise
re-solve a solved problem.

### The ruling, in three parts

1. **Single-`Function` inlining — dead, already replaced.**
2. **Defunctionalized lowering into bounded helpers — VIABLE, and Θ(n) is
   reachable** for n nested well-bracketed scopes. Normal/abrupt control, affine
   authority, trap order, joins and cleanup add **constant-factor** node kinds;
   none forces super-linearity. A linked predecessor stack/DAG shares suffixes
   instead of enumerating paths.
3. **The whole-configuration specialization REPRESENTATION — no route to O(n)
   through more sealing.** Sealing is linear only in the graph it *receives*; it
   cannot undo state products or variable-width identity already materialized.

⇒ **The mechanism family is viable; this representation of it is not.**

### Why the representation cannot claim O(n)

One helper per distinct **composite semantic-state key**, where keys remain the
Cartesian tuple `(program point × environment × selected suffix × join/path ×
layout × control heads)` with **variable-width** members:
`PartitionSelectedScopeKey.outer_env` / selected `pending` are `Vec`s
(`partition.rs:1262–1348`); producer actions embed eliminator vectors
(`:1350–1401`); SourceArm/SourceKont keys combine residual body, env,
declaration stack, active recursion, control heads, selected state, cleanup,
field types and field maps (`:2700–3050`); `PartitionContinuationKey` carries
the exact checked join plus `field_types`/`field_map` (`:1413–1422`); the
interner exact-compares the **entire retained key** (`:3088–3148`, `:4500–4635`).

★ **Hash-consing children is necessary but INSUFFICIENT while the outer key is
still the Cartesian tuple** — it shares equal subterms, it cannot merge two
different tuples merely because their components are shared. **Θ(n) states each
carrying Θ(n)-wide data already permits Θ(n²)** descriptor, comparison, frame
and emitted-interface work.

Two invariants are absent, and both are required for an analytical O(n) claim:
**(a)** a fixed **K** helpers/transitions per static source/control node;
**(b)** a **constant maximum key/frame width**, independent of nesting depth.

### ★ What n=4 does and does not prove — the honest reading

`(states, edges) = (1482, 1525)`; `E/S ≈ 1.029` says only that the realized
graph is **nearly chain-shaped on average**. **One point cannot establish an
exponent** — it cannot separate `370n` from `93n²` from a product that only
switches on at n=5. ⛔ **We do NOT claim n=4 proves quadratic growth.** The hold
rests on **code inspection rejecting an O(n) proof**, not on curve-fitting one
datum. Both things are true at once: the growth order is still unknown, *and*
more local sealing is the wrong next move.

### Retain vs replace (permanent architecture)

**Retain** — exported root + bounded deferred functions; the useful semantic
transition categories; and **all of #24–#33's proved semantics**: exact
normal/abrupt edges, trap sequencing, affine reservation/bind/spend authority,
graph sealing, completion witnesses, the **W/T producer-wrapper vs ultimate-tail
distinction**, linked cleanup/source topology.

**Replace** — whole-configuration specialization; vector-shaped/flattened
residual keys; recursive `Debug` serialization as identity
(`partition.rs:153–163`); helper identity coupled to env/control/layout contents.

**The factored machine:** (1) static transition graph, one constant-width node
per syntax/control transition, one helper per static node/edge; (2) dynamic
activation passed through a **fixed ABI frame** — *dynamic activation identity
must not create code identity*; (3) persistent constant-width cons/DAG stores
(syntax, env extension, eliminator, selected context/lineage, source, cleanup,
continuation) — no flattened suffix/ancestry/declaration-stack/occurrence-path
in any helper key; (4) evidence attached **out of line** to stable node/edge IDs;
(5) normal and abrupt successors share one persistent continuation/cleanup graph
— mutually exclusive runtime paths must not become a static subset product.

`PartitionWorkItem` survives **only** if each item names one static transition.

### #34 is EVIDENCE, not work to finish

#34 confirms #33's semantic ruling worked: the path constructs `W = site 4`,
`T = site 2`, leaves W solely descriptor-owned, passes strict STOP, and reaches
the nested-exit resume seam. **Carry the invariant, do not patch it now.** In the
new graph this is an explicit **source-return-owned resume edge/node** — ⛔ do
**not** overload `Terminal` (which means *no continuation*, whereas this state
has a live continuation owned indirectly by an exact source-return descriptor).
Option 2's duplicate direct W is **rejected**.

### ⛔ THE FIRST IMPLEMENTATION UNIT IS A PLANNER/CENSUS RECUT — NOT #35

Generate the minimal nested family for **n = 3…7 BEFORE lowering full bodies**,
and report **by state kind**: static nodes, edges, helpers, emitted CLIF
instructions/bytes; descriptor bytes constructed/retained; exact-comparison
bytes; total and maximum frame fields; maximum static-key bytes; maximum
env/pending/path lengths.

**Acceptance needs BOTH** — empirical **bounded first differences** (use first
*and second* finite differences, not ratios) **and** structural assertions:
fixed K transition/helper nodes per static source/control node; constant maximum
key and frame schema widths in n; all graph/code/descriptor totals affine in n.
Then port semantics and rerun the table **plus** exact normal return, every
abrupt exit, trap identity/order, joins, and affine single-spend differentials.
**n=4 alone never discharges the gate.**

**Falsifier for this hold:** the unchanged representation showing, across
n=3…7, constant key/frame/env/pending/path maxima, bounded K states per source
node, and stable first differences for graph, code **and descriptor** work —
*not merely state/edge counts*.

### ✅ THE CHECKPOINT IS DURABLE — `b077eb7a` is tagged ON ORIGIN

```
refs/tags/rt-native-fnsplit-checkpoint-b077eb7a  ->  b077eb7a   (verified by ls-remote)
```

⚠ **When found, `b077eb7a` lived on ONE local branch
(`wp/RT-NATIVE-FNSPLIT-native-partition`) with ZERO refs on origin** — no copy
anywhere off this box, in a repo where `handoff-gate-compact.sh` hard-resets
branches and where a `git branch -f` would have orphaned it silently. It carries
the proved semantics of #24–#33 that **Phase 1 must measure and Phases 2–3 must
port from**; losing it would have cost the recut its reference implementation.

★ **Same failure family as the QA bound-verdict attestations:** *a workaround (or
a hold) that leaves load-bearing state on one local ref in one clone.* A "frozen"
checkpoint is only frozen if something outside this machine holds it. ⛔ Do not
delete this tag.

### Steward duties from this ruling

- **Author the recut frame.** Runtime is held until it exists. ⇒ status is
  `active` but the ring is **parked**, not building.
- **Hard-stop cadence is SUSPENDED with the machine.** Count of record stays
  **33** (#34 was raised but is evidence, not a ruled stop). The every-3rd
  research cadence resumes **only** against the recut chain; re-arm the trigger
  line when the new frame opens.

## Contract (Architect-specified — state the NEED, do not freeze the mechanism)

Per `evt_7gkn3g4tsvgb9`, the follow-up must:
- make native compilation **accept the minimal four-bracket discriminator and the
  actual SP-A-write / SP-B / SP-C programs without source contortions**;
- **bound per-function lowering growth generically** — outlining,
  continuation/function partitioning, or an equivalent owner-chosen design is
  admissible — while **preserving process semantics, effect order, trap/error
  identity, join/subcontinuation accounting, and the public native ABI**;
- **prove the boundary** with the minimal three-vs-four-bracket reproducer, then
  run the exact native SP matrices currently blocked.

**⛔ Rejected non-solutions (Architect):** test-only special cases; merely raising
Cranelift's cap; disabling a verifier/check; interpreter fallback presented as
native execution; or asking Ken authors to reshape otherwise-valid source. The
mechanism ("function splitting" etc.) is the owning team's to choose — this WP
states the need, not the design.

## Why this is its own WP (not in the PX8-SPAN-PROV fence)

PX8-SPAN-PROV's fence is buffer-span provenance (elaborator/interp/runtime/host
admission). Native codegen function-splitting / process-ITree size relief is a
distinct backend-capability concern. Per the Steward scope ruling
(`evt_7c160ej3bwz4`), PX8-SPAN-PROV Phase 2 lands its **sound, mutation-proven
mechanism** now with **honest partial-status** native conformance rows
(interpreter GREEN; native SP-A-write / SP-B / SP-C marked
**BLOCKED-ON-NATIVE-REACHABILITY**, pointing here). This WP is the named
follow-up that lifts the wall.

✅ **Architect means/layer ruling delivered (`evt_7gkn3g4tsvgb9`): out-of-fence for
PX8-SPAN-PROV, a separate Runtime backend WP** — concurring with the Steward scope
split. Size is **TBD** because the repair mechanism is the owning team's to choose
(the Contract above admits outlining, continuation/function partitioning, or an
equivalent design), not because any ruling is outstanding.

## What "done" unblocks

Once the 4-bracket wall is lifted, the deferred native conformance matrices become
runnable:
- SP-A **write** absolute native+interpreter discriminator (foreign-write reject →
  `InvalidBounds`, zero backend; own-write success control);
- SP-B per-engine combined foreign / stale / overflow / negative-offset precedence
  arms, native end-to-end;
- SP-C old / foreign / fresh **write** controls (not just freeze), native
  end-to-end.

At that point CV flips the PX8-SPAN-PROV native SP rows from
BLOCKED-ON-NATIVE-REACHABILITY to GREEN on the landed capability (a small
conformance-only follow-up fold), completing the Phase-1-locked engine matrices.

## Sequencing (Steward)

**RELEASED + `active` 2026-07-23** — this is **Track 1** of two concurrent impl
tracks (operator "the plan sgtm" + returning to two active impl teams with the
codex reseat). Kicked to the Runtime ring (design-bearing: Architect design
consult before heavy implementation). Track 2 is [[PX8-F-CAP-41]] (Foundation) —
genuinely contention-free (disjoint crates: this = `ken-runtime`/Cranelift; A1 =
`ken-elaborator`/`ken-host`; disjoint ledger; different team). Size stays **TBD**
until the ring proposes its codegen approach in the design sketch.
Sibling of [[PX8-SPAN-PROV]] (whose native conformance completion it unblocks);
root [[PX8]].
