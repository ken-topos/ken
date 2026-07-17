# PX8-X — Runtime V2 observation-transport exposure (thin, additive)

> **⚠ OPERATOR RULING (2026-07-17): Ward is a SEPARATE project, not part of Ken;
> Ken must not implement Ward's functionality.** This overrides the earlier
> Architect decomposition that placed a Ward *consumer* (PX8-W) in `ken-verify` —
> **PX8-W is STRUCK from Ken** (see its tombstone). Ken's job is the **one-way
> Ken→Ward export seam** ([[obligations-route-to-ward-not-into-ken]],
> `G-Ward-seam`): Ken **exports/exposes** the assumption boundary; the separate
> Ward project **consumes and judges** it, out of this repo. PX8-X is the
> **runtime-observation half of that export seam** and is **export-only — it
> builds no consumer, no monitor, no verdict.** (Operator confirmed 2026-07-17:
> build PX8-X now, export-only.)
>
> The V2 event trace is *produced* on every dispatch but is **dropped at every
> public observation boundary** (all routes return only `Vec<EffectEventV1>`).
> PX8-X exposes the already-produced V2 trace + a terminal-exit classification
> through a **real interpreter + linked-native observation route**, so the
> **external Ward project** can consume **real** V2 observations across the seam.
> **Additive-only; V1 stays byte-identical.**

- **ID:** PX8-X · **Owner:** **Team Runtime** (leader `agt_37reqrd72cg00` /
  implementer `agt_37reqg3nync00` / qa `agt_37reqvb6ce400`) · **Size:** S–M ·
  **Risk:** Medium (touches the linked-native wire codec — must extend it
  **without** changing the `LinkedEffectTraceV1`/`KETRACE2` magic, layout, bytes,
  or any existing decoder/caller; a mutation there breaks every PX4b/PX7 native
  test and the PX6 differential harness).
- **Branch:** `wp/px8x-v2-observation-exposure` off **`origin/main @ f2f60083`**.
- **Route:** **Architect §14** (observation-transport soundness — the additive
  boundary, the same-producer invariant, V1 byte-preservation). No `spec/`/
  `conformance/` change expected → CV only if the candidate touches those paths.
- **Parallel with PX8-F.** Runs concurrently; no dependency on PX8-F. Completes
  the runtime-observation half of the Ken→Ward export seam; **no in-Ken
  consumer** rides on it.

## Objective

Add a **real V2 observation route** on both the interpreter and the linked-native
lanes that carries the already-accumulated `EffectEventV2` trace (ordered,
role-labelled `resource_bindings`) plus a runtime **terminal-exit classification**
(`NormalReturn | ReturnedError | ControlledTrap`), sourced from the **same**
dispatch reply/settlement events that already populate the V1 trace — **without a
second event producer** and **without mutating** the V1 observation types, their
wire encoding, or their callers. This is the runtime-observation transport the
**external Ward project** reads across the one-way seam; today the V2 trace exists
internally but never crosses a public boundary. **Ken builds no consumer** — this
WP ends at exposing the observation.

## Fixed inputs — DO NOT REOPEN (settled; do not re-ask the operator)

- **Architect ruling `evt_3v3mtq4q085e2`** — the binding constraints:
  - **Additive only.** Do **not** mutate `EffectObservationV1`
    (`crates/ken-host/src/effect_v1.rs:2211-2218`), `LinkedEffectTraceV1`
    (`crates/ken-host/src/effect_wire_v1.rs:15-22`), their **wire bytes**
    (`KETRACE2` magic/layout), or **any existing caller** (inventory below). The
    V1 observation and every existing V1 consumer stay **byte-identical**.
  - **One producer, two views.** Both the V1 and V2 views come from the **same**
    dispatch reply/settlement event. Do **not** add a second event producer — the
    V2 trace already accumulates alongside V1 (anchors below); PX8-X only
    **exposes** it.
  - **The V2 observation is the transport for the external judge (the separate
    Ward project).** It carries observed runtime facts only; PX8-X does **not**
    validate the obligation, populate `Q`, or touch `status: delegated` — and Ken
    builds **no** consumer that does (that is the external Ward project's, not in
    this repo).
- **Landed anchors on `f2f60083` (verify before editing; `git show origin/main:`):**
  - **Interpreter (`crates/ken-interp/src/eval.rs`):** recorder
    `EffectTraceRecorderV1` **4871-4877** with `events: Vec<EffectEventV1>` (4873)
    **and** `events_v2: Vec<EffectEventV2>` (4874); both accumulate on every
    dispatch in `record` **4884-4897**. `run_io_effect_observation_v1` **4910**
    passes **only `recorder.events`** at **4934** into `effect_observation_v1`
    **4937-4993**, which builds `EffectObservationV1` (4986-4992) —
    `recorder.events_v2` is dropped. Re-exported `ken-interp/src/lib.rs:13`.
  - **Native (`crates/ken-host/src/abi_v1.rs`):** `ProcessContext` **291** with
    `effect_trace: Vec<EffectEventV1>` (297) **and** `effect_trace_v2:
    Vec<EffectEventV2>` (298), accumulating at **1421-1427** + **903-913**.
    `write_observation_v1` **914-930** serializes `LinkedEffectTraceV1` **V1
    only** (919-926); called from `ken_host_invocation_v1_finish` **836**.
  - **Linked-native runner (`crates/ken-runtime`):** second producer
    `KenNativeInvocationV1` `native_effect_v1.rs:59/91/109-114` (also carries
    `effect_trace_v2`); `run_bound_process_effect_observation_v1`
    `object_linker_packaging.rs:240-298` returns `EffectObservationV1`, decoding
    the wire via `decode_linked_effect_trace_v1` (268) → **the gap is at the wire
    codec**, which has no V2 field.
  - **Types:** `EffectEventV2` `effect_v1.rs:2129-2136` with `resource_bindings:
    Vec<(ResourceBindingRoleV2, ResourceTraceIdentityV1)>` (2133);
    `ResourceBindingRoleV2 = File | Buffer | Target` (2122-2127);
    `effect_event_v2_from_dispatch` (2140-2153). **`EffectObservationV2` /
    `LinkedEffectTraceV2` do NOT exist** — PX8-X mints them.
  - **Terminal-exit today:** only `terminal_error: Option<TerminalErrorV1>` +
    `exit_status`/`terminal_value` (`effect_v1.rs:2215-2217`,
    `effect_wire_v1.rs:19-20`) exist; the **three-way
    `NormalReturn/ReturnedError/ControlledTrap` classifier does NOT exist in
    runtime code** (it lives only as static policy strings in
    `export.rs:191/1114-1116` and `§71:342-343`). PX8-X adds the runtime
    classifier additively from the existing `terminal_value` sign +
    `terminal_error` presence.
- **§71 §2.3 (on `f2f60083`)** — the monitor consumes ordered role-labelled
  `resource_bindings` and requires no live bracket-owned identity at
  `[NormalReturn, ReturnedError, ControlledTrap]`; that is **why** the V2
  observation must carry the trace **and** the terminal-exit class. PX8-X
  produces the observed facts; the **external Ward project** (out of Ken) judges
  them against the exported obligation. §71 is cited here only to fix the shape of
  the facts Ken must expose — Ken implements none of the monitor.

## Mandated deliverables (each ends in a concrete implementable choice)

- **X-D1 — V2 observation types (additive, minted).** Add `EffectObservationV2`
  (mirrors V1's observation fields **plus** `effect_trace: Vec<EffectEventV2>`
  and the terminal-exit class from X-D2) and, for the native lane, a
  `LinkedEffectTraceV2` transport. **New types beside the V1 ones — V1 unchanged.**
  Runtime owns the concrete representation (a parallel V2 wire record vs an
  additive trailer) **subject to** the additive constraint: the V1 record's
  magic/layout/bytes and every V1 decoder are untouched. State which you chose and
  why in the WP notes.
- **X-D2 — runtime terminal-exit classifier.** Add `TerminalExitClassV2 =
  NormalReturn | ReturnedError | ControlledTrap` (a **sealed enum, no `_ =>`
  catch-all** on any match that projects it — COORDINATION §7) computed
  additively from the existing `terminal_value` sign + `terminal_error` presence
  at the two classification sites (`eval.rs:4960-4985` interpreter,
  `object_linker_packaging.rs:278-289` native). Carry it in the V2 observation.
  Do **not** alter the existing V1 `terminal_error`/`exit_status` computation.
- **X-D3 — the two real V2 routes.** Add an interpreter V2 route (parallel to
  `run_io_effect_observation_v1`) that reads `recorder.events_v2` and returns an
  `EffectObservationV2`; and a native V2 route (parallel to
  `run_bound_process_effect_observation_v1`) that carries `effect_trace_v2` across
  the wire (X-D1) and returns an `EffectObservationV2`. **New functions beside the
  V1 ones** — do not change the V1 functions' signatures or bodies.
- **X-D4 — same-producer reachability proof.** The V2 route yields, on **both**
  lanes, the ordered role-labelled `resource_bindings` and the correct
  terminal-exit class, **derived from the same dispatch/settlement events** as the
  V1 view (assert V1 and V2 traces agree in count/order for the shared
  operations). This is self-validating and **gates PX8-X on its own** — no
  external consumer needed. The *route* is exercisable now with the landed
  `withBuffer`/`BufferAllocate` acquire event and any positioned op reachable now;
  the richer two-resource `FsReadAt` over `File`+`Buffer` becomes exercisable once
  the PX8-F surface lands, but PX8-X does **not** block on it. No second producer
  path.

## Required proofs / discriminators (each independently reaching; §7)

1. **V1 byte-identical (regression):** every existing V1 caller/test (inventory
   below) stays green; the V1 observation + `KETRACE2` wire bytes are unchanged.
2. **V2 route carries the real trace:** the interpreter and native V2 routes each
   return an `EffectObservationV2` whose `effect_trace` is the **real**
   `Vec<EffectEventV2>` with the ordered `resource_bindings` (grep the emitted
   order/roles, not the type name).
3. **Terminal-exit three arms:** `NormalReturn` on a normal return,
   `ReturnedError` on a returned error, `ControlledTrap` on a controlled trap —
   each **reaching** (a distinguishing case per arm, not one positive).
4. **Same-producer agreement:** for the operations shared between views, the V1
   and V2 traces agree in sequence count/order (proving one producer, two views).
5. **Additive wire:** an existing V1-only decoder reads a PX8-X-encoded stream
   unchanged (the V2 data is additive and ignorable to V1 readers).

## Acceptance criteria (testable)

- **AC1** — X-D1..X-D4 landed: real interpreter + native V2 routes returning
  `EffectObservationV2` with the real `Vec<EffectEventV2>` trace + terminal-exit
  class; new types/functions **beside** the V1 ones.
- **AC2** — **additive-only:** `EffectObservationV1`, `LinkedEffectTraceV1`, the
  `KETRACE2` wire bytes, and **every existing caller** are byte-identical/behavior-
  identical (discriminator 1). The differential harness (`ken-verify/scenario.rs`)
  and PX4b/PX7 native tests stay green.
- **AC3** — the V2 routes carry the **real** ordered `resource_bindings` from the
  same producer (discriminators 2 + 4); terminal-exit classifies all three arms
  (discriminator 3).
- **AC4** — the `TerminalExitClassV2` match(es) are exhaustive with **no `_ =>`**
  (§7 completeness-by-construction).
- **AC5** — **no-regression = GREEN IN CI** (never a local `--workspace` run;
  COORDINATION §12). Build/test **targeted only** (`scripts/ken-cargo -p
  ken-interp …` / `-p ken-host …` / `-p ken-runtime …` / `--test <name>`), **plus
  run the ken-cli integration + `ken-verify` differential suites** the wire change
  implicates before release (the additive-wire boundary is exactly what full CI
  guards).

## Do-not-reopen guard

- Do **not** mutate `EffectObservationV1`/`LinkedEffectTraceV1`/their wire
  bytes/`KETRACE2` magic/any existing caller — additive V2 beside V1 only.
- Do **not** add a second event producer — expose the existing
  `events_v2`/`effect_trace_v2`; both views from the same event.
- Do **not** validate the obligation, populate `Q`, or touch `status:
  delegated` — PX8-X carries observed facts only; the **external Ward project**
  judges them. Ken builds **no** consumer/monitor/verdict in this repo.
- Do **not** wait on PX8-F — the route + terminal-exit + wire are buildable now
  against the landed producer; only the *two-resource positive fixture* fully
  exercises once PX8-F's surface lands, and that is **not** a PX8-X gate.
- If exposing the V2 trace cannot be done additively without touching the V1 wire
  layout, **HARD-STOP to the Steward** (the additive premise is wrong) rather than
  mutating V1.

## Additive-constraint caller inventory (must stay byte/behavior-identical)

- `run_io_effect_observation_v1` — `ken-interp/lib.rs:13`; callers
  `ken-cli/src/lib.rs:225`, `eval.rs:5795`.
- `run_bound_process_effect_observation_v1` — callers `ken-verify/scenario.rs:153,
  230`; CLI tests `px4b_native_production.rs`, `px7f_resource_native.rs:21`,
  `px7l/px7m/px7n/px7o/px7p_*`.
- `write_observation_v1` — `abi_v1.rs:836` (+ test `:1538`).
- `EffectObservationV1` — `ken-cli`, `ken-runtime/{lib,native_effect_v1,
  object_linker_packaging}.rs`, `ken-host/effect_v1.rs`, `ken-interp/eval.rs`,
  `ken-verify/{canonical,catalog,host,lib,scenario}.rs`.
- `LinkedEffectTraceV1` — `ken-host/{abi_v1,effect_wire_v1}.rs` (`KETRACE2` codec).

## Sequencing

`{ PX8-F ∥ PX8-X } → Phase-C exit` (Ken side). PX8-X and PX8-F are independent and
parallel; each merges on its own Decision (Architect §14). PX8-X is
**self-contained** — its acceptance (X-D4 same-producer agreement + additive-wire
regression) is provable without any consumer, so it does **not** wait on PX8-F or
anything downstream.

The V2 observation type + terminal-exit class this WP mints are Ken's **exported**
interface across the one-way Ken→Ward seam. The **external Ward project** (out of
this repo) is the consumer that reads them and discharges the exported obligation;
per the operator ruling (2026-07-17) Ken implements **none** of that monitor —
the earlier PX8-W consumer is struck. Keep the exported types clean and documented
so an external consumer can bind to them, but build no consumer here.
