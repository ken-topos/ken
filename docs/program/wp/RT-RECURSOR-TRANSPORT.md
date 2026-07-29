# RT-RECURSOR-TRANSPORT — carry an active recursor across a unit boundary

**An active computational recursor's result carries invocation-local
scope/return-hole state. Two residual classes exist solely because that state
cannot cross a functionized unit boundary. This node builds that transport — or
proves the state need not cross — and retires both classes.**

**Owner:** Team Runtime. **Branch:** assembled on the **D7 lineage** — see §0.
**Size:** L. **Risk:** ⭐ **highest in the campaign.** This is the mechanism the
B2F migration stopped at.

⛔ **Read `docs/program/16-recursive-descent-retirement.md` first.** It carries
the campaign's three binding traps and the schedule. This frame does not repeat
them.

---

## 0. ⭐⭐ RECUT 2026-07-29 — `D1` ANSWERED, ORDER CHANGED, ASSEMBLY ATOMIC

**Architect ruling `evt_5zr53v2dp86md`, on exact `820d3e53`.** This node was
sixth in a seven-node campaign and reached by nobody. It is now **the reached
successor of `D7`**, and three things about it changed at once. ⛔ Read all three
before touching the deliverables below — each one invalidates a reading of this
frame that was correct yesterday.

### (1) ⭐ It is THIS node — not a new one, and not `RT-PRODUCER-MATCH-PORT`'s

The terminal refusal measured at the D7 seam is **this node's predicate
verbatim**:

```text
ObjectEmission / checked_process_object /
ComputationalMatch: a computational recursor closure names an in-flight
activation, not a transferable value
```

⇒ a computational recursor reached a separately emitted unit edge while its
invocation-local **activation / resume / return-hole** control was still embodied
in `ComputationalRecursorClosure`. ⛔ **Not a new node. Not another `D7`
disposition. Not the syntactic-residual retirement owned by
[[RT-PRODUCER-MATCH-PORT]].**

⭐ `ComputationalRecursorClosure` is correctly **`EscapeForbidden`** — admitting
it would publish compiler control. **`D7` did its ruled job**: it preserves the
phase-bearing known-constructor operands, performs the known-constructor
elimination in the source machine, and advances the edge to the next owner's
fail-closed admission check. That is a **lawful successor seam**, not a defect.

### (2) ⛔ THE `RT-PRODUCER-MATCH-PORT` DEPENDENCY IS RETIRED FOR THIS POPULATION

**Ruled:** *"The existing schedule dependency on `RT-PRODUCER-MATCH-PORT` is no
longer a mechanism prerequisite for this reached population: `D7` already
supplies enough producer-`Match` path to expose the recursor boundary."*

⇒ ⛔ **Do not wait for [[RT-PRODUCER-MATCH-PORT]].** Its `depends_on` edge to
this node is removed. ⚠ **What is NOT retired:** that node's own retirement of
the `ProducerMatchCall` syntactic residual. It remains separate, it remains
owed, and ⛔ **this node must not absorb it** (see `AC-16`).

### (3) ⛔⛔ ATOMIC ASSEMBLY ON THE `D7` LINEAGE — ONE CANDIDATE, TWO NODES

**Ruled:** this mechanism *"must be assembled **atomically on this D7 lineage**
because `D7` cannot satisfy its named parity gate without the reached successor,
and the successor has no reaching production witness on the pre-`D7` tree."*

| fact | value |
|---|---|
| preservation point | `820d3e53014899da50e7d8fab0584b8c267c5874` |
| tree | `5faee6ef816ce35369a2eadee5f4de305834ad85` |
| parent | `79029d4c` |
| `D7`-only adjustment | ⛔ **NOT AUTHORIZED** |

⇒ **[[RT-DECL-CLOSURE-PORT]] and this node land as ONE candidate, in ONE PR, and
both tracker nodes flip `merged` together.** Neither can go green alone: `D7`'s
parity gate needs the reached successor, and this node has no production witness
without `D7`.

⭐⭐ **This is the resolution of a real circularity, not a convenience.** Before
the ruling, `D7`'s frame held it *"until its consumers are complete"* while the
tracker ran `RT-DECL-CLOSURE-PORT → RT-SEED-CALL-PORT →
RT-PRODUCER-MATCH-PORT → RT-RECURSOR-TRANSPORT`. That is a cycle, and it was not
merely a policy one — `rt_parity_native` is its own CI job, so the publisher gate
is **mechanical**. Atomic assembly breaks it.

⛔ **Atomic assembly does NOT relabel the mechanism as `D7`.** The recursor work
is reviewed, owned, and accepted as **this node**. ⛔ Do not fold its ACs into
`D7`'s, and ⛔ do not describe recursor code as a `D7` deliverable in the
candidate description.

## 1. Fixed inputs

| path | blob at the `D7` seam `820d3e53` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | ⚠ **re-pin at pickup** |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | ⚠ **re-pin at pickup** |
| `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs` | ⚠ **re-pin at pickup** |

⛔ **The former pins at `origin/main = 14c3c5f7` are RETIRED — do not use them.**
`D7` rewrote `core.rs`, `units.rs` and `static_transition.rs` on this lineage
(2658 insertions at `79029d4c`, plus the seam repair at `820d3e53`), so every
line anchor derived from `14c3c5f7` is stale. ⭐ **Re-pin against `820d3e53`
before deriving anything**, and ⛔ do not re-pin the numbers and call that a
re-measurement.

## 2. The two classes are one mechanism

**`MatchScrutineeRecursor`** — a `Match` whose scrutinee is a
`ComputationalMatch` with a non-empty `recursive_positions`.

**`LexicalCallArgumentRecursor`** — a `Call` whose callee is a `LexicalClosure`
and whose **argument** matches that same shape.

⭐ **The code states the shared mechanism itself**, in
`LexicalCallArgumentRecursor`'s doc comment:

> *"The recursive result still carries invocation-local scope/return-hole state.
> Passing it through a separately declared lexical unit is not one of the
> completed functionized ports, so the established recursive descent lane
> retains the whole call."*

⇒ Same predicate, same carried state, two syntactic positions: match scrutinee
and lexical call argument. ⛔ **Retiring one without the other builds the
transport twice.**

⭐ **The ruling confirms the fold:** *"This choice covers both existing
positions… The two reached `fs_*` rows are population for that shared mechanism,
**not permission to repair only two cells**."*

## 3. ⭐⭐ `D1` IS ANSWERED — OUTCOME **(b)**, THE STATE NEED NOT CROSS

⛔ **`D1` is no longer a probe to run. Do not re-run it to "confirm" the
answer.** It was pulled forward exactly as this frame's former §3 pre-authorized,
and the Architect answered it on the measured seam (`evt_5zr53v2dp86md` §2).

> **Outcome (b): prove the invocation-local state need not cross the runtime
> ABI.**

**Planning must split the recursor edge before emission**, in these five steps:

1. **decompose** the capsule into its **runtime residual operand** and its
   **checked compiler-only invocation segment**;
2. **consume/validate** the exact **oriented affine control obligation** in the
   caller-owned plan;
3. **embody the static continuation** in **graph-derived, out-of-line
   worker/continuation unit targets**;
4. **carry only ordinary** runtime residual / environment / result operands
   through **typed ABI slots**; and
5. use the **statically connected call/return edge as the return hole**.

**Per position:**

- **Match scrutinee** — the worker result returns to a **caller-owned `Match`
  continuation unit**.
- **Lexical call argument** — planning **splits/specializes the transparent
  call** around that recursor use, so **prefix, worker, and suffix units are
  statically connected**. ⛔ The recursor capsule is **not a parameter**.
- **Recursion** reuses an **interned static continuation definition** and a
  **direct unit target**.

**What may carry static identity:** the checked call/frame template, producer
origin, sibling/parameter position, recursive-body origin, and source call site.

⛔ **What stays compiler-local and is consumed affinely** — dynamic activation
IDs, resume cursors, unwind vectors, open obligations, splice handles. ⭐ **They
are neither ABI key material nor runtime data.** Putting any of them in a
descriptor is the failure this outcome exists to forbid.

## 4. Deliverables

- **⭐ `D0` — the delta-free regression baseline, PER ROW.** ⛔ Not an aggregate.
  ⭐ **It already exists for the seven parity rows and it is binding** — measured
  on detached `483ef7ab` (`evt_3tc0zm7smx9h2`), exactly **1/7**:

  | row | base result |
  |---|---|
  | `buffer_freeze_malformed_span_is_unconstructible_at_the_landed_surface` | **PASS** |
  | `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` | FAIL — `BoundaryCarrier: a closure capture is a specialized-only surface…` |
  | `fs_read_at_malformed_offset_narrows_to_invalid_offset` | FAIL — `ComputationalMatch: tree-producing match scrutinee is not Bool or a constructor` |
  | `fs_read_at_malformed_offset_without_read_right_narrows_to_invalid_offset` | FAIL — same |
  | `fs_read_at_malformed_window_narrows_to_invalid_bounds` | FAIL — same |
  | `fs_write_at_malformed_offset_narrows_to_invalid_offset` | FAIL — same |
  | `fs_write_at_malformed_offset_without_write_right_narrows_to_invalid_offset` | FAIL — same |

  ⚠ **Why the row map and not the count.** `AC-1b` is a **per-row** property. A
  `1/7`-then vs `1/7`-now agreement is **count** agreement, and one of six
  contributors can defect underneath an aggregate that holds. ⭐ Extend this map
  to any suite you newly touch; ⛔ never replace it with a total.
- **`D2` — Full-residual enumeration** across the measured programs, including
  the population unmasked by [[RT-PRODUCER-MATCH-PORT]]'s `D4` **if that node has
  landed by pickup**. ⚠ It no longer gates this one — scope against what is
  measured on `820d3e53`, not against the pre-campaign number.
- **`D3` — The transport, per outcome (b)'s five steps**, covering **both**
  syntactic positions. ⛔ Not a runtime carrier; ⛔ not an ABI slot.
- **`D4` — Remove both `MatchScrutineeRecursor` and
  `LexicalCallArgumentRecursor`**, and only then re-run `AC-1a` and `AC-1b`.
- **`D5` — The atomic candidate.** One branch on the `820d3e53` lineage carrying
  `D7` **and** this mechanism, one PR, both nodes flipping `merged` together.

## 5. Acceptance criteria

- **`AC-1a` — the ceiling moved.** The selector reports
  `authority=FunctionizedUnits` / `residuals=none` on every program `D2` named
  as firing either class.
- **`AC-1b` — the objects still build.** Those programs **compile and pass**
  their existing suites, **and every row PASS in `D0`'s row map is still PASS**.
  ⛔ Not "the residuals are gone" — the objects build. ⚠ `AC-1a` does **not**
  discharge this: it quantifies over the firing set, and the regression
  population is its complement (campaign doc, Trap 2).
- **`AC-2`.** Outcome **(b)**'s proof obligation is discharged **by a control**,
  ⛔ not by prose.
- **`AC-3` — both positions, not one.** A control exercises the recursor in
  **match-scrutinee** position and another in **lexical-call-argument**
  position, and a mutation defeating the transport **reds** each. ⭐ One control
  covering one position would let the fold silently become a half-fold.
- **`AC-4` (no-regression).** Workspace green **in CI** — ⛔ never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-5`.** The exhaustive-match fail-closed property in the residual
  classifier is preserved. ⛔ No wildcard arm.
- **`AC-6`.** Emitted function count and per-function code-size distribution
  recorded for the affected programs. ⛔ Report; do not tune, do not pin a
  threshold.

### ⭐⭐ The eight required discriminator ACs (Architect `evt_5zr53v2dp86md` §3)

⛔ **All eight. They are the ruling's price for outcome (b)**, and each one names
the mutation that must **red** it.

- **`AC-9` — two-position closure.** One **real** Match-scrutinee witness and one
  **real** lexical-call-argument witness traverse the static split. ⛔ **Omitting
  either position fails the closed edge census BEFORE emission.**
- **`AC-10` — continuation identity.** The same runtime residual under **two
  distinct** checked continuation/frame identities yields **two distinct**
  expected results. ⛔ Collapsing or swapping their static continuation keys must
  **red** the oracle.
- **`AC-11` — zero-control ABI.** Descriptor slots contain **exactly** ordinary
  residual / environment / result convention slots. ⛔ Adding activation, cursor,
  frame, splice, capsule, or selector material must **fail the ABI census**.
- **`AC-12` — owner-exact return hole.** Returning into the **callee**, the
  **wrong caller suffix**, or the **wrong sibling/parameter position** must
  **red** the result/control oracle.
- **`AC-13` — affine control.** Duplicate, omit, reorder, transplant, or reuse a
  dynamic splice / open-obligation token, and planning must **reject before
  CFG/object emission**.
- **`AC-14` — finite recursion.** Repeated dynamic invocation of one static
  recursor site reuses **one interned continuation graph**. ⛔
  Clone-on-invocation must fail the unit census / fixed-point check.
- **`AC-15` — fail-closed escape.** Direct **whole-capsule** transfer remains
  rejected **before** descriptor / function / object allocation. ⭐ The outer
  capsule must continue to fail transfer before allocation or publication.
- **`AC-16` — population and non-absorption.** Preserve the exact `D0` row map
  and `D7`'s controls, and **prove the reached rows advance through this
  mechanism without absorbing** the separate syntactic-residual retirement owned
  by [[RT-PRODUCER-MATCH-PORT]].

## 6. ⛔ Banned scope

- ⛔ **Retiring only one of the two classes.** They are folded for a stated
  mechanism reason; half is a worse state than neither, because it hides that
  the transport is incomplete.
- ⛔⛔ **Every item on the ruling's forbidden list, at every depth:** a
  capsule / activation / cursor / frame-vector **ABI slot**; a **tag, selector,
  function pointer, trampoline, side table, or codec**; a **value-selected
  continuation**; **reconstruction of `Lowered::ComputationalRecursorClosure`**;
  **inlining as transport**; **validator or admission weakening**.
- ⛔ **Absorbing [[RT-PRODUCER-MATCH-PORT]]'s `ProducerMatchCall` retirement.**
  The dependency is retired; the sibling work is not.
- ⛔ **Repairing `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds`.**
  It remains **unruled** at `Match: scrutinee is not a constructor value`, and
  ⛔ **nothing in the ruling attributes or authorizes repair of it.**
- ⛔ **Counting or repairing the transient ordinary-`Closure` refusal** — see §8.
- ⛔ **A `D7`-only adjustment to `820d3e53`.** Not authorized.
- ⛔ **Deleting the selector or the `RecursiveDescent` lane** — that is
  [[RT-DESCENT-RETIRE]].
- ⛔ **Weakening the `recursive_positions` predicate** to shrink the population.

## 7. Hard stop

Stop and report if outcome **(b)** cannot be discharged as specified — ⛔ **do
not fall back to a runtime carrier**, and ⛔ do not soften an infeasibility into
a described-but-unbuilt mechanism. ⭐ If the two positions turn out to need
different transports, that **falsifies this node's fold** and is the Steward's
re-cut, not the ring's to absorb. ⚠ Per Trap 2, this node exposes the largest
newly reachable population in the campaign; **expect a fail-closed invariant to
fire and route it as its own node.**

## 8. ⚠ The transient ordinary-`Closure` refusal — UNATTRIBUTED, do not count it

An earlier refusal on the write-readonly row is recorded in
`/tmp/rt-d7-parity.log`:

```text
ObjectEmission / checked_process_object /
Closure: a closure cannot cross the boundary: it is runtime-local and
live-domain only, and it has no durable lane
```

⛔ **That log is NOT sufficient causal evidence** to assign it to `D7`, to
recursor transport, or to a new node. ⭐ **The text proves only that the
admission walk encountered a whole closure at `checked_process_object`** — the
later terminal variant does **not** identify its former parent edge or
disposition. ⚠ **A changed visible refusal is not causal attribution**, which is
the discipline the implementer correctly applied when it declined to classify it.

⇒ ⛔ **Do not count it and do not repair it now.** **If it recurs on a preserved
exact tree**, take **one diagnostic-only bounded witness before transfer**,
recording: source origin · owner · parent/child role · disposition token ·
**complete nested `Lowered` variant path** · pre-allocation side-effect counters.
Run **one named row**, remove the diagnostic, preserve the exact tree. ⚠ If the
transient tree cannot be reproduced exactly, **retain the log as unattributed
evidence and wait for recurrence.**
