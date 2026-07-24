# RT-NATIVE-FNSPLIT — representation recut, planner-and-census first

**WP frame (Steward). Owning team: Runtime. Size: L (three phases, each its own
branch + merge Decision). Supersedes the per-hard-stop cadence.**

> ## ⛔ THIS IS A CONTINUATION, NOT A RESTART
>
> **Every semantic result from hard-stops #24–#33 is RETAINED.** The Architect's
> viability ruling (`evt_3m1g3v4m2bj51`) is explicit: *"Do not abandon bounded-
> function partitioning or the semantic mechanisms already proved."* The
> mechanism family is **viable and Θ(n) is reachable**. What is replaced is the
> **representation** — the identity that gets scheduled — not the semantics that
> identity was carrying.
>
> `b077eb7a` is the **semantic/diagnostic checkpoint**. It is a source of truth
> for *what the machine must do*, and a counter-example for *how it must be
> keyed*. Do not delete it, and do not treat this frame as licence to
> re-litigate settled semantics.

## Objective

Bound native per-function lowering growth to **O(n)** in nested resource-bracket
depth `n`, by factoring **static code identity** from **dynamic activation**, and
carry RT-NATIVE-FNSPLIT through the operator's scaling gate to merge.

## Fixed inputs — SETTLED, do not reopen

Each is a decided input. If one turns out **false against the landed code**,
say so and escalate — do not quietly build around it (see Perishability).

1. **Viability ruling `evt_3m1g3v4m2bj51`** — the factored machine, five points,
   reproduced in Phase 2 below.
2. **#33 ruling `evt_55c62m0anfyyk`** — `producer_head = W(selected, successor=T)`,
   `live_producer_tail = T`; W is the immediate callable wrapper, T the ultimate
   completion obligation; completion evidence is **for T only**; normal execution
   is exactly `W once → T once → CompletedTail(T)`.
3. **#34 disposition** — evidence, **not work to finish**. Its invariant carries
   forward as an explicit **source-return-owned resume edge/node**. ⛔ Do **not**
   overload `Terminal` (which means *no continuation*; this state has a live
   continuation owned indirectly by an exact source-return descriptor). Option 2's
   duplicate direct W is **rejected**.
4. **Operator scaling gate `evt_4btfhwqhah1ye`** — "tests complete under a
   timeout" is **not** acceptance. n=3..7 empirical + research-grounded analytical
   growth order + an explicit verdict.
5. **Research advisories** — #24 `evt_5gshpmyb2ta79`, #27 `evt_7s6b3zg82n7n5`,
   #30 `evt_1stmfwh0tj5gm`, #33 `evt_3vr382mrv99pe`, viability
   `evt_7p40c3x8cnwtm`. Prior-art grounding; advisory, never a ruling.
6. **The original `VReg::MAX` single-`Function` root cause is FIXED and GONE.**
   `b077eb7a` already emits an exported root plus one function per
   `PartitionWorkItem` (`core.rs:120–492`). ⛔ Do not re-solve it.

### Retain (do not rebuild, do not regress)

Exported root + bounded deferred Cranelift functions · the useful semantic
transition categories · exact normal/abrupt edges · trap sequencing and exact
trap identity/order · affine reservation/bind/spend authority · graph sealing ·
completion witnesses · the W/T distinction · linked cleanup/source topology.

### Replace

Whole-configuration specialization · vector-shaped and flattened residual keys ·
recursive `Debug` serialization as identity (`partition.rs:153–163`) · helper
identity coupled to environment / control / layout **contents**.

---

## ✅ Phase 1 — CLOSED, `could_not_determine`. AMENDED 2026-07-24 (recut hard-stop #1)

> ### ⛔ PHASE 1 IS RESOLVED. IT IS NOT PENDING WORK. DO NOT RE-RUN IT.
>
> **The original Phase 1 rested on a premise that is FALSE against the landed
> code**, and this section now records the outcome rather than asking for it. It
> is retained (not deleted) because the *result* is a fixed input to everything
> below. Architect ruling `evt_6dpb96kn1583f`; implementer report on clean WIP
> `82bd1f43`, tagged durably on origin as
> `rt-native-fnsplit-census-wip-82bd1f43`.

### The result, and the proved cause

**`b077eb7a` cannot supply a complete empirical baseline at all.** On the
generated n=3 family *and* on the checkpoint's own pre-existing public depth-2
control, it fails with:

```text
NativeExitScopeTransitionV1: scope body return lost its parent producer tail
```

★ **That it reproduces on the checkpoint's existing depth-2 control — not only on
the new generator — is what makes this structural rather than a generator bug.**

⇒ **AC1.1 correctly returned `could_not_determine`; AC1.2 and the binary AC1.4
were unreachable.** The fail-closed third outcome did exactly the job it was
written for: a step that could not reach an answer **refused to return the
permissive one.** That is the mechanism working, not a failure of it.

There is also **no pre-existing planner boundary in the held representation** to
measure at: `core.rs:301–520` consumes `partition_queue`, lowers each
`PartitionWorkItem`, verifies and defines the functions, and the census is
emitted only afterward at `core.rs:523+`, with interner counts accumulated
*while* those work items are being materialized. **An earlier print is a
failure-frontier-dependent prefix, not a closed graph.**

### ⛔ What this DOES and DOES NOT falsify — the distinction is load-bearing

| falsified | NOT falsified |
|---|---|
| The premise that unchanged `b077eb7a` can supply a complete empirical baseline | **The representation hold** |

**The hold stands, established STRUCTURALLY** — variable-width composite helper
identity, and the absence of fixed-K / fixed-key-width invariants. It never
depended on curve-fitting, which is why losing the empirical baseline does not
disturb it. ⛔ **Do not read `could_not_determine` as "the rewrite is
unnecessary."** The cheap exit the original Phase 1 hoped for **does not exist**;
the recut proceeds.

### ⛔ Phase 1 landing rules

- **Preserve** `82bd1f43` and `rt-native-fnsplit-checkpoint-b077eb7a` as
  **diagnostic evidence**. Both are on origin. Do not delete either.
- ⛔ **Do NOT publish the held representation, nor its representation-specific
  census instrumentation.** Neither lands on `main`.
- ⛔ **Do NOT land a permanent test whose only honest explicit result is
  currently indeterminate.** A reusable fail-closed harness/schema may be split
  or ported **only when it measures a tree that can satisfy its declared
  contract.**
- ⚠ **This supersedes the Steward's earlier "Phase 1 merges the harness only"
  ruling (`evt_3zapwrrpkbq08`).** That ruling correctly kept the held
  representation off `main`, but it still assumed a *landable* harness. There is
  **no Phase-1 merge**; the harness ports forward into the recut instead.

### ⛔ Do not "finish" the old machine

- **Do NOT complete #34 in the held machine.** The durable viability ruling makes
  #34 **evidence, not work to finish** — its source-return-owned resume edge
  belongs in the **new** graph.
- **Do NOT accept partial pre-error counts** as a census.
- **Do NOT describe a newly-built dry-run path through `b077eb7a` as an
  "unchanged-representation" census.** Building a new path through it means it is
  no longer unchanged, and the claim would be false.

★ **Repairing the old composite-key machine would contradict a fixed input and
restart the very sealing cadence the viability ruling ended.**

### Historical — the original Phase-1 deliverable, for the record

*Superseded. A permanent bounded harness generating n = 3,4,5,6,7 and reporting,
by state kind, against `b077eb7a` unchanged:*

- static nodes, edges, helpers, emitted CLIF instructions/bytes
- descriptor bytes constructed **and** retained; exact-comparison bytes
- total frame fields **and** maximum frame fields
- maximum static-key bytes; maximum env / pending / path lengths
  ⛔ *(this metric was WRONG — see the width-metric correction below)*
- compile wall-time and peak RSS under `prlimit` (fail-safe, never unbounded)

---

## ⚠ THE COMPLETE n=3..7 EMPIRICAL GATE MOVES TO THE RECUT — two CLOSED boundaries

The operator's scaling gate is unchanged and still a merge condition. What
changes is **where** it is measured, because it can no longer be measured on the
held representation. It splits into **two closed boundaries**, and **neither may
stand in for the other.**

### Boundary A — PLANNER, before any semantic body emission

Build the new factored static transition graph for **n = 3..7** and report:

- static nodes · edges · **planned** helpers
- persistent-store nodes · out-of-line evidence records
- **fixed K** per static source/control node
- fixed key schemas · fixed frame/store-node schemas

**This is the recut's FIRST empirical acceptance gate.** ⛔ It is **not** a
measurement of unchanged `b077eb7a` and must not be described as one.

⛔ **CLIF instructions/bytes and full compile wall/RSS CANNOT be required here** —
there is no lowering yet. Requiring them of a pre-lowering planner census is a
category error.

### Boundary B — FULL EMISSION, after the semantic port

Report emitted helpers · CLIF instructions/bytes · descriptor
construction/comparison work · compile wall + peak RSS · the same structural
counts — **plus** the exact normal/abrupt/trap/join/affine differential suite.

⛔ **A post-failure prefix cannot substitute for either boundary.**

### ⛔ WIDTH METRIC — CORRECTED. The original frame's version was WRONG.

**The original Phase-1 metric list demanded constant maxima for "env / pending /
path lengths". That is the wrong invariant and would have rejected a CORRECT
design.**

The constant invariants are maximum **inline identity / frame / store-node
widths**. **Logical persistent-chain DEPTH for environment / pending / path may
grow Θ(n), and that is SOUND** — because the helper/frame carries **one
constant-width ID** into the persistent store rather than the chain itself.

⇒ The gate requires, precisely:

1. **No flattened env / pending / path member in helper identity.**
2. **Constant ID / node payload width.**
3. **Affine total persistent nodes.**
4. **At most affine logical chain depth.**

⛔ **Do NOT require the logical chain length itself to be constant.**

### ⛔ The first planner graph must carry #34 explicitly

The **source-return-owned resume transition** must be present **from the first
accepted planner shape** — not retrofitted. `Terminal` stays un-overloaded (it
means *no continuation*; this state has a live continuation owned indirectly by an
exact source-return descriptor), and Option 2's duplicate direct W remains
rejected.

★ **This is topology planning in the new representation, not a semantic patch to
the old one** — which is what keeps it out of the ended cadence.

### Acceptance

- **AC1.1′** Fail-closed is retained verbatim for **both** boundaries: a run that
  cannot complete reports `could_not_determine` as a **third outcome that FAILS**,
  never a silent pass.
- **AC1.2′** Each boundary reports **every** metric in *its own* list. Missing one
  is a failed AC, not a footnote. ⛔ Do not borrow Boundary B's metrics into A.
- **AC1.3′** Report **first and second finite differences**, not ratios. ⛔ A
  single ratio, or a fitted curve alone, discharges nothing.
- **AC1.4′** Boundary A is a **genuine stop**: report and **hold for a Steward +
  Architect read** before the semantic port. The binary
  hold-falsified/hold-confirmed question is **closed** above and is not re-asked.
- **AC1.5′** ⛔ **Do not claim an exponent from few points.** `370n`, `93n²`, and
  a product switching on at n=5 all pass through the historic n=4 datum. **The
  structural invariants (1)–(4) are what discriminate**; the table corroborates.

---

## Phase 2 — the factored representation

> ### ⛔ ENTRY CONDITION — AMENDED 2026-07-24
>
> The original entry condition was *"only if Phase 1 confirms the hold."* **That
> condition can never be met as written**, because Phase 1 returned
> `could_not_determine` and the binary question is closed. The amended entry
> condition is:
>
> 1. **This amended frame is authoritative** (it is — that is what makes the
>    planner-first sequence the live unit), **and**
> 2. **Boundary A (the planner census) is the first thing built** — the factored
>    static transition graph for n=3..7, reported and **held for a Steward +
>    Architect read** before any semantic body emission.
>
> **The hold is established structurally, not empirically** (see Phase 1). Phase 2
> proceeds on that basis. ⛔ Do not wait for an empirical confirmation that cannot
> be produced, and do not re-open the viability question to get one.

Replace the scheduled identity with:

1. **Static transition graph.** One constant-width node per syntax/control
   transition, keyed by `(transition-kind, static node ID)` plus bounded tags.
   **One emitted helper per static node/edge — never per whole semantic
   configuration.**
2. **Dynamic activation.** Environment, normal-continuation, abrupt-continuation,
   scope and affine-authority pointers/dense IDs pass through a **fixed ABI
   frame**. ⛔ **Dynamic activation identity must not create code identity.**
3. **Persistent constant-width stores.** Syntax, environment extension,
   eliminator, selected context/lineage, source, cleanup and continuation nodes
   are cons/DAG nodes: local payload + child ID, interned once. ⛔ **No flattened
   suffix, ancestry, declaration stack, environment, capture schema, or occurrence
   path in any helper key.**
4. **Out-of-line evidence.** Contracts, affine ledgers, exact edge witnesses and
   sealing validators are **kept**, but keyed by stable node/edge IDs, with
   growing evidence stored out of line rather than embedded in state identity.
5. **Shared suffixes.** Normal and abrupt successors point into the **same**
   persistent continuation/cleanup graph. ⛔ Mutually exclusive runtime paths must
   not become a static subset product.

`PartitionWorkItem` may survive **only** as an emission queue in which each item
names **one static transition**. If it still means "one distinct full
configuration," that decomposition is replaced too.

★ **The trap to design against, stated once:** hash-consing children is
**necessary but insufficient** while the outer key remains the Cartesian tuple
`(program point × environment × selected suffix × join/path × layout × control
heads)`. Interning shares equal subterms; it **cannot** merge two distinct tuples
merely because their components overlap. **Calling the vectors "interned" does
not reduce the product-state count.**

**Phase 2 acceptance:**

- **AC2.1 — structural, and it is a compile-time property where possible.** Every
  helper key schema has a **fixed maximum width independent of n**. Demonstrate
  by construction (fixed-arity key types, no `Vec`/`String`/path members), not by
  measurement alone.
  ⚠ **Read this with the corrected width metric above.** "Fixed width" governs
  **inline identity / frame / store-node width** — *not* logical
  persistent-chain depth, which **may grow Θ(n)** and is sound because the
  helper/frame carries one constant-width ID. The four precise requirements are
  (1) no flattened env/pending/path member in helper identity, (2) constant
  ID/node payload width, (3) affine total persistent nodes, (4) at most affine
  logical chain depth. ⛔ Do not assert or demand a constant chain length.
- **AC2.2** Each static source/control node owns **at most a fixed K**
  transition/helper nodes. **Name K** and assert it in a test.
- **AC2.3** No `Debug`-serialization, recursive walk, or variable-width member
  appears in any identity/bucket path. Grep-able and asserted.
- **AC2.4** The #33 W/T semantics and the #34 source-return-owned resume node are
  represented **explicitly** in the new graph, with `Terminal` **not** overloaded.
- **AC2.5** Scalar ABI bytes, dynamic cell widths and `SourceKont`-return width
  are **byte-identical** to `b077eb7a`. All summary/planning metadata is erased
  after CFG finalization.

---

## Phase 3 — semantic port, differential re-verification, and the gate verdict

Port the retained semantics onto the factored representation and re-run **both**
the Phase-1 table **and** the differential suite.

**Phase 3 acceptance:**

> ### ⛔ BASELINE HONESTY — AMENDED 2026-07-24. THERE IS NO BASELINE.
>
> **No apples-to-apples complete wall/RSS baseline exists at `b077eb7a`** — it
> cannot complete even the depth-2 public control (Phase 1). So:
>
> - **Report the recut's ABSOLUTE n=3..7 values.** They stand on their own.
> - ⛔ **Label the historic n=4 `1,482 states / 1,525 edges` comparison
>   NON-COMPARABLE** unless it came from the **identical source, phase boundary,
>   and metric schema**. It did not.
> - ★ **Operator review decides the constants from the new complete
>   measurements. It must NOT inherit a fabricated baseline.** Writing a
>   comparison against a number that was never produced by the same measurement
>   is the failure mode here — an honest absolute table beats a flattering delta
>   against an incommensurable figure.

- **AC3.1** The n=3..7 table shows **all** graph, code and descriptor totals
  **affine in n**, with bounded first differences — and constant maxima for
  **inline identity/frame/store-node widths**.
  ⚠ **Corrected:** *not* "env/pending/path lengths" — logical chain depth may be
  Θ(n) (see the width-metric correction). Require affine total persistent nodes
  and at most affine logical chain depth instead.
- **AC3.2 — differential, against `b077eb7a` behavior:** exact normal return;
  **every** abrupt exit; trap identity **and order**; joins; affine single-spend.
  Each unchanged. ⛔ One case per guard position, plus one where the problem hides
  behind indirection — an obvious-case pass is not the property.
- **AC3.3** The originally-blocked 4-bracket programs compile and produce correct
  runtime outcomes, unblocking `PX8-SPAN-PROV`'s native SP rows
  (SP-A write/precedence, SP-B precedence arms, SP-C write controls).
- **AC3.4** Compile wall-time and peak RSS at n=4 are reported against the
  Phase-1 baseline. If growth is linear but constants remain unacceptable, that
  is an **explicit operator decision**, surfaced — not absorbed silently.
- **AC3.5** No-regression means **green in CI**, never a local `--workspace` run
  (COORDINATION §12). Local runs are scoped `-p ken-runtime` / named tests.

---

## Do-not-reopen guardrails

- ⛔ The single-`Function` `VReg::MAX` root cause — **fixed, gone, historical.**
- ⛔ The #33 W/T ownership ruling and the #34 Option-2 rejection.
- ⛔ Whether bounded-function partitioning is viable — **ruled: it is.**
- ⛔ The operator's scaling gate as a merge condition.
- ⛔ Growing the scalar ABI, minting a trusted primitive, or widening the TCB to
  make a phase pass. Any of those is an **operator** call, not a build call.

### Added by the 2026-07-24 amendment (hard-stop #1)

- ⛔ **Phase 1's `could_not_determine` is CLOSED.** Do not re-run the
  held-checkpoint census, and do not re-ask the binary
  hold-falsified/hold-confirmed question.
- ⛔ **The representation hold is established STRUCTURALLY.** Do not treat the
  missing empirical baseline as evidence against it, and do not reopen viability
  to manufacture a confirmation.
- ⛔ **Do not repair, complete, or dry-run the held machine** — not #34, not a new
  path through `b077eb7a`, not a partial pre-error count presented as a census.
- ⛔ **Do not land the held representation or its representation-specific census
  instrumentation**, and do not land a permanent test whose only honest result is
  currently indeterminate.
- ⛔ **Do not require constant logical persistent-chain depth.** Θ(n) depth with
  constant-width IDs is the *correct* design; demanding otherwise rejects it.
- ⛔ **Do not compare Phase-3 numbers to the historic n=4 `1,482 / 1,525` figures**
  as if commensurable, and do not hand the operator a baseline that was never
  measured under the same schema.

## Research cadence

**The held chain's count is FROZEN at 33 and does NOT carry forward** — it counted
the machine this frame replaces.

**The recut chain is ARMED and live:**

```text
RECUT CHAIN: hard-stop count = 1   (#1 = the Architect's 2026-07-24 amendment ruling)
NEXT RESEARCH PULL = hard-stop #3, then #6, #9, #12, …
```

**The Steward holds the count of record**; the Architect's re-derivation loses on
disagreement (it explicitly deferred on #1). The armed line lives in
`issues/RT-NATIVE-FNSPLIT.md` and is the one to re-read on every hard-stop. **No
research pull is due before #3.** A deep chain carrying **zero** advisories is
itself the tell that both the self-trigger and the backstop have lapsed — that
already happened once on the held chain (10 hard-stops dry).

## ⚠ Perishability — this clause binds

Every current-state claim in this frame — line numbers, type names, which
structures are vector-shaped — was written against **`b077eb7a`** on 2026-07-24
and is **perishable**. Re-verify each against the landed code at pickup. **If a
fixed input is false, say so and escalate; do not build around it.** The stale
premise this WP already carried for days (the `VReg::MAX` root cause, fixed long
before anyone noticed the file still asserted it) is exactly the cost of
skipping this step.

## Escalation

Soundness / mechanism → **Architect**. Conformance rows → **CV**. Scope,
sequencing, research cadence, and the operator's scaling gate → **Steward**.
Route a fork to the **one** owner of its lane; do not convene the room.
