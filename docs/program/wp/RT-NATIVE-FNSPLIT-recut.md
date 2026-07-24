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

## ⚠ Phase 1 — CENSUS FIRST, ON THE HELD REPRESENTATION. This is the cheapest
## possible exit and it runs BEFORE any rewrite.

**Deliverable:** a permanent, bounded scaling harness that generates the minimal
nested resource-bracket family at **n = 3,4,5,6,7** and reports, **by state
kind**, against **`b077eb7a` unchanged**:

- static nodes, edges, helpers, emitted CLIF instructions/bytes
- descriptor bytes constructed **and** retained; exact-comparison bytes
- total frame fields **and** maximum frame fields
- maximum static-key bytes; maximum env / pending / path lengths
- compile wall-time and peak RSS under `prlimit` (fail-safe, never unbounded)

**Why this order, and it is the load-bearing decision in this frame.** The
Architect's own falsifier for the hold is: *the unchanged representation showing
constant key/frame/env/pending/path maxima, bounded K states per source node, and
stable first differences for graph, code AND descriptor work.* Measuring the held
representation therefore either:

- **falsifies the hold** — in which case **the entire rewrite is unnecessary**,
  we resume sealing, and Phase 1 cost is a harness we needed anyway; or
- **confirms it** — and we now hold the **baseline** every Phase-3 claim is
  measured against.

⇒ **Phase 1 is a genuine decision point, not a formality.** Report the table and
**stop for a Steward/Architect read** before starting Phase 2.

**Phase 1 acceptance:**

- **AC1.1** The harness is a permanent test, bounded by `prlimit`, and **fails
  safe** — a run that cannot complete reports *"could not determine"* as a
  **third outcome that FAILS**, never a silent pass. (A step that cannot reach an
  answer must not return the permissive one.)
- **AC1.2** The n=3..7 table is produced for **every** metric above, by state
  kind. Missing a metric is a failed AC, not a footnote.
- **AC1.3** Report **first and second finite differences**, not ratios.
  ⛔ A single ratio, or a fitted curve alone, does not discharge anything.
- **AC1.4** State explicitly, in the report, **which of the two Phase-1 outcomes
  obtains** — hold falsified, or hold confirmed — and do not proceed to Phase 2
  without that statement being read by the Steward + Architect.
- **AC1.5** ⛔ **Do not claim an exponent from few points.** `370n`, `93n²`, and a
  product that only switches on at n=5 all pass through the observed n=4 datum.
  Structural evidence (widths, K-per-node) is what discriminates; the table alone
  is corroboration.

---

## Phase 2 — the factored representation (only if Phase 1 confirms the hold)

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

- **AC3.1** The n=3..7 table shows **all** graph, code and descriptor totals
  **affine in n**, with bounded first differences — and constant maxima for key
  bytes, frame fields, env/pending/path lengths.
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

## Research cadence

**Count FROZEN at 33; the every-3rd hard-stop cadence is SUSPENDED** with the
machine it counted. When this frame opens a new chain, the Steward re-arms an
explicit `next research pull = N` line in `issues/RT-NATIVE-FNSPLIT.md`. **The
Steward holds the count of record**; the Architect's re-derivation loses on
disagreement. A deep chain carrying **zero** advisories is itself the tell that
both the self-trigger and the backstop have lapsed.

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
