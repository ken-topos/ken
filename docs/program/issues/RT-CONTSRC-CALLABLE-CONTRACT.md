---
id: RT-CONTSRC-CALLABLE-CONTRACT
title: "Closed callable-contract arm for continuation sources — a recursive IH is a compiler-only static worker with no value carrier, and the enclosing slot authority is unconditionally a value contract, so its environment sits outside the domain RT-CONTSRC-PRODUCER-LOCAL owns"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSRC-PRODUCER-LOCAL]
blocks: []
github: null
origin: Architect ruling evt_4h2v01dc7g8s4 (2026-08-05) stating the required component boundary, on the D2 correction at exact 5377d2ab; sized by the IH-requirement census at evt_qttaeebtzjkt (exact e6d4f085). Steward-filed per COORDINATION §2 (agents cannot create tracked work).
---

## The gap, exactly

`RT-CONTSRC-PRODUCER-LOCAL`'s `D1` gives a producer-local value an exact
coordinate, but the coordinate is enclosed by `ContinuationSourceSlotAuthority`,
whose contract is unconditionally a **value-slot** contract: carrier, ownership,
storage owner, referent affinity.

A recursive induction-hypothesis binder is not such a value. Lowering already
represents it as a compiler-only `LoweringEnvironmentBinding::StaticWorker` —
no runtime word, tag, descriptor or carrier — and its only lawful use is an
exact-`Var` callee.

**The coordinate can name the IH; the enclosing contract has nowhere lawful to
write "static callable, no value carrier."** That is an expressibility failure,
not a missing enum arm, and it is why the position stays `Open`.

Three measurements agree on the boundary, all in current source at
`e6d4f085`:

1. **No `ResultPhase` to `AbiCarrier` map exists anywhere.** The vocabularies
   are disjoint by construction — one records a representation phase, the other
   an ABI transport. `slot_referent_affinity`, the authority for which carriers
   a continuation source environment admits, accepts `ValueWord` and
   `GroundValueCarrier` and refuses the rest; nothing proves an IH is either.
2. **The IH's phase is not edge-local.** It is `carrier()` or `SPECIALIZED`
   depending on `functionized_units`, a whole-plan argument to
   `plan_static_transition_graph_with_symbols` that is **not a field of
   `StaticTransitionPlan`**. So `(case body, ordinal)` does not determine the
   IH contract, and the walk cannot reach the fact that would.
3. **Production continuation inputs have no callable domain at all.** As
   measured when this node was filed, `BoundaryUseAvail::Callable` and
   `BoundaryUseNeed::PreserveCallableIdentity` existed solely as `#[cfg(test)]`
   mutations; every production projection was `Value` / `PreserveValue`.
   **`RT-CONTSPEC-LEDGER` (recut 2026-08-08) deletes those enums outright**, so
   the measurement holds a fortiori once that node lands. The frame's `D0` says
   how to record which world you measured in; the shape of this node is
   unaffected either way.

## Why this is a successor and not a precursor

**Measured at `e6d4f085` (`evt_qttaeebtzjkt`): exactly one edge of 83 requires
a recursive-IH binder**, and it is a `ken-runtime` lib edge:

```
prog=10/2 consumer=fn0 cont=origin10 construct=origin19 pos=0 closure=origin18
  required=2   0:OPEN[ih-binder]   1:local[case-arg]
```

All **17 parity edges** — the population behind the six failing `D0` rows in
[[RT-CONTSRC-PRODUCER-LOCAL]] — are all-closed and IH-free, including all four
closures the `1e` hard stop was about.

⇒ This node closes **1 of 83 edges and 0 of 6 failing rows.** It is real work
on a real capability gap, and it is not on the critical path.

**It is also not the thing that would let `D4` close every environment.**
`D4`'s declined set `R` is exactly three instances, from three causes:

| supplying construct | verdict | closed by this node? |
|---|---|---|
| `ih-binder` | `OPEN` | yes |
| `let-value:Construct` | `OPEN` | no — a different contract domain |
| `let-value:If` | `AMBIG2` | no — joins two distinct exact sources |

**These are "outside-this-contract-domain residuals", NOT "unrepresentable"**
(Architect, `evt_38yd5sd1ht0kk`). Nothing here claims a future authority cannot
represent `Construct` or a joined `If`; they are simply not authorized by
[[RT-CONTSRC-PRODUCER-LOCAL]] and not by this node either.

⇒ A completion standard of "every environment closed" would be unreachable by
any node currently in the graph, with or without this one. `D4` is therefore
stated as **set equality** — `interned = V`, `declined = R` — not as closure.

## Status: `ready`, and the priority is written into the frame

**Frame:** `docs/program/wp/RT-CONTSRC-CALLABLE-CONTRACT.md`.

This node was held at `draft` on the argument that it closes 1 of 83 instances
and 0 of 6 failing `D0` rows, so promoting it would put an off-critical-path
node in front of a reader looking for the next kickoff. **That cost is real and
the remedy was wrong.** Withholding `status` to express priority makes the node
invisible to `gen-progress.sh`, which reads status, not intent — a framed node
kept at `draft` is indistinguishable from unstarted work. Priority now lives in
section 0 of the frame, where it is stated rather than encoded.

The same argument also claimed *"the content above is the frame."* It is not.
A node is not a frame: what it lacked was fixed inputs measured at a base, ACs
with named controls, and banned scope — which is the whole of what a ring
discharges. The frame now carries those, and makes naming the base its own
`D0`, because this node's base does not exist yet.

**`ready` here means framed and shovel-ready, not startable.** Its dependency
[[RT-CONTSRC-PRODUCER-LOCAL]] is unmerged, with `D3c`, a possible `D3b` re-cut,
`D4b` and its candidate ahead. It enters the frontier when that node merges.

## What it must deliver

Preserve the coordinate and locator from `D1`. Split the **contract beside it**
as a **closed sum**:

1. **Value source** — the existing carrier / ownership / storage /
   referent-affinity contract, **unchanged**.
2. **Static-callable source** — an exact planner-owned callable/worker identity
   and its declared parameter/capture contract, with **no** value carrier, **no**
   ownership and **no** storage projection.

Constraints, all from the Architect's ruling:

- The callable arm comes from the **existing planned static-worker/member
  authority** — never from `ResultPhase`, from source syntax, or from a new
  lookup in lowering.
- **Every** downstream source / projection / view consumer must **exhaustively
  distinguish** the two contracts. No default arm.
- A callable source creates **no ordinary ABI slot**. Its lawful consumer is
  callee-only, and every value-producing use must **fail closed**.
- The **production** need/availability vocabulary must be able to state that
  fact. A `#[cfg(test)]` callable arm cannot be its authority — that is
  measurement 3 above, and reusing it would make the test suite the source of a
  production contract.
- Whole-plan result representation stays a **separate later decision** about
  what calling the worker returns. It must not be folded into coordinate
  identity or used to manufacture an input carrier.

## The witness this node must close

The single IH edge above. Its environment closing is the deliverable; a
report that the arm exists without that edge's vector going all-closed does not
discharge it.
