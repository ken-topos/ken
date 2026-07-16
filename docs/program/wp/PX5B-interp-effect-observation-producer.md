# PX5B — interp-lane canonical effect observation producer (Runtime)

- **ID:** PX5B · **Owner:** Team Runtime · **Size:** S · **Risk:** Medium
  (touches the interpreter host-dispatch seam; the observation must be a
  *write-only, one-way* instrumentation that changes no evaluation behavior).
- **Objective:** Give the interpreter lane a **real** canonical effect
  observation producer — the oracle counterpart to PX5's native
  `run_bound_process_effect_observation_v1`. Record, at the actual interpreter
  canonical host-dispatch seam, the true `{operation, CanonicalRequestV1,
  reply.capability_identity, reply.outcome}` for each dispatched host op into an
  ordered trace, and expose it as a consumable producer that yields a
  `ken_host::EffectObservationV1` reflecting **what the interpreter actually
  dispatched** — never a caller-supplied expectation. This closes the PX6 §14
  producer-seam block so the differential harness's oracle lane is trustworthy.
- **Depends on:** PX5 (merged, `origin/main @ 049628f8`) for the canonical
  observation/trace/error/capability types (`ken_host::EffectObservationV1`,
  `EffectEventV1`, `CanonicalRequestV1`, `CapabilityTraceIdentity`,
  `CanonicalOutcomeV1`/`SemanticErrorV1`, `HostDispatchReplyV1`).
- **Blocks:** **PX6** (Verify) — the differential harness consumes this producer
  for the interpreter lane's promotable observation. PX6 is parked on PX5B.
- **Gate it feeds:** G-Ward-seam / native-effect lane (with PX6).

## Fixed inputs — DO NOT REOPEN (cite, do not relitigate)

The mechanism is **pinned by the Architect §14 verdict on PX6 `b7e7a14d`**
(`evt_6q01xcfrmzz16`, thread `thr_74tys5r5e39s4`). It is a fixed input here:

- **The bug being fixed** (in PX6, not here): PX6's interpreter lane built its
  promotable trace from `Scenario.expected_fs` (a caller-supplied expectation)
  rather than from observed dispatch. That lets an interpreter-side raw-path
  normalization bug (`dir/./x` executed as `dir/x`) pass a differential falsely
  when native preserves `dir/./x`. The oracle must reflect **actual** dispatch.
- **Ground truth already exists** at `ken-interp/src/eval.rs:4153-4244` (the exact
  `CanonicalRequestV1` decoded from the evaluated Ken operation) and `:4258-4269`
  (the real `HostDispatchReplyV1` carrying capability identity + outcome). Today
  that request/reply is **discarded after `reify_host_reply_v1`**. PX5B records it
  instead of discarding it.
- **The instrumentation is write-only and one-way.** It records; it does not
  change evaluation, does not invent a response, does not consult the recorded
  data to make any decision. Removing the hook must leave interpreter behavior
  byte-identical.
- **Authorization stays in `HostHandler`.** Do NOT move path authorization,
  capability resolution, or denial handling out of the handler / dispatcher. Host
  leaves remain descriptor-only.
- **Capability identity and error identity come from the reply**, never
  reconstructed from the configured capability. Record
  `reply.capability_identity` and the reply outcome/`SemanticErrorV1` enum
  identity verbatim.
- Trust: this is interpreter instrumentation for a tested/target-validated
  harness — **zero kernel rule, zero new Ken postulate, no proof-of-confinement
  claim.** It is a trusted-base change (interpreter), so it takes the §14
  Architect soundness gate.

## Scope

**In scope (PX5B owns):** the write-only observation hook at every interpreter
canonical host-dispatch seam (FS grounded at `eval.rs:4153-4269`; the
Console/Clock canonical dispatch seams likewise — enumerate them); an ordered
in-run trace accumulator keyed to real dispatch order; and a consumable interp
producer API that runs a checked program and returns a real
`ken_host::EffectObservationV1` (six fields) whose `effect_trace`, terminal error
identity, and per-op capability identity are sourced from actual dispatch.

**Out of scope:** the comparator, the twin-root harness, the discriminator net,
and dropping `expected_fs` as an observation source — those are **PX6 (Verify)**.
Do NOT change native production (PX5, merged). Do NOT change evaluation
semantics, path authorization, or the sealed op catalog.

## Mandated deliverable outline — each section ends in a concrete choice

1. **The observation hook.** At the FS canonical-dispatch seam
   (`eval.rs:4258-4269`) and the Console/Clock canonical-dispatch seams, after
   `dispatch_host_op_v1` returns the reply and before/around
   `reify_host_reply_v1`, record one ordered trace event carrying `{operation:
   HostOpV1, request: CanonicalRequestV1, capability_identity:
   Option<CapabilityTraceIdentity> from reply, outcome: CanonicalOutcomeV1 from
   reply}`. **Concrete choice:** the event type is `ken_host::EffectEventV1` (the
   same canonical event PX5's native trace emits), so both lanes yield
   comparable `EffectObservationV1`; if any needed field is absent from
   `EffectEventV1`, extend the *consumer-facing observation assembly*, not the
   sealed native wire ABI.
2. **The trace accumulator.** A per-run, append-only ordered sink threaded
   through the interpreter host-dispatch path (e.g. on the `HostHandler` /
   backend, or an explicit run-scoped recorder). **Concrete choice:** ordering is
   the real dispatch order (append at the seam); no reordering, dedup, or
   normalization. Recording is unconditional for every dispatched op incl.
   denials (a denied op is a real event with its reply outcome).
3. **The consumable producer API.** A `pub` entry — e.g.
   `run_io_effect_observation_v1(program, host, …) ->
   Result<ken_host::EffectObservationV1, _>` in `ken-interp` — that runs a
   checked program through the interpreter with a real `PosixHost` and returns
   the six-field observation: `stdout`/`stderr` from the interpreter's console
   capture, `filesystem_delta` observed from the real root (or left to the PX6
   harness's root-A snapshot if that is the existing division — state which),
   `terminal_error`/`exit_status` from the run, and `effect_trace` from the
   accumulator. **Concrete choice:** the `effect_trace`, terminal error identity,
   and capability identities MUST come from the hook, never from any
   caller-supplied expectation; `ken-verify` imports this producer and does not
   reimplement it.
4. **Behavioral-inertness proof.** A test (or structural argument) that the hook
   is one-way: interpreter evaluation results, denials, and existing
   `ken-interp` suites are unchanged with the hook present. **Concrete choice:**
   assert the existing interp acceptance suites stay green and add a direct
   equality check that a run's reified results are identical with/without
   observation enabled.

## Acceptance criteria (testable)

- **AC1 — real trace, not expectation.** The producer's `effect_trace` is built
  only from values recorded at the dispatch seam. A test proves that for a
  program dispatching a raw path `dir/./x`, the recorded request retains the
  **actual dispatched bytes** decoded from the Ken operation, independent of any
  caller-supplied string.
- **AC2 — descriptor-collision discriminator.** Two raw paths with the same
  resolved components (`dir/./x` vs `dir/x`) are shown to be distinguishable in
  the recorded trace: the trace retains the actual dispatched raw bytes, so the
  two do **not** collide in the observation. (This is the interp-producer half of
  the PX6 discriminator; PX6 completes the two-lane bite.)
- **AC3 — reply-sourced identities.** For a malformed / wrong-token dispatch, the
  recorded `capability_identity` is `None` and the error identity is the reply's
  `MalformedCapability` (or the exact reply enum), proving identities come from
  the reply, not from configured-cap inference.
- **AC4 — six-field observation.** The producer returns a real
  `ken_host::EffectObservationV1` (imported, not redefined) with all six fields
  populated from actual run/dispatch state, usable directly by `ken-verify`.
- **AC5 — behavioral inertness.** Interpreter evaluation is byte-identical with
  the hook present vs. absent; authorization stays in `HostHandler`; host leaves
  stay descriptor-only; no evaluation-semantics change.
- **AC6 — CI-green, honesty.** No-regression = **green in CI** (full locked
  workspace), never a local `--workspace` run (build/test targeted only via
  `scripts/ken-cargo -p ken-interp`, plus any directly-touched crate). Disclosure
  tested/validated, not proved; no kernel rule, no new postulate.

## Do-not-reopen guards

- The mechanism is the Architect's `b7e7a14d` verdict — do NOT redesign it; do
  NOT move authorization out of `HostHandler`; do NOT let the hook influence
  evaluation; do NOT reconstruct capability/error identity from configured caps.
- Do NOT redefine `ken_host` canonical types (import them); do NOT touch PX5
  native production or the sealed op catalog.
- Do NOT absorb PX6's comparator / discriminator-net / `expected_fs`-demotion
  work — that is Verify's, on the fresh SHA after PX5B lands.
- A genuinely new fixed boundary (only) hard-stops and routes to Steward/Architect.

## Grounding anchors (origin/main `049628f8`)

- **Interp FS canonical-dispatch seam:** `ken-interp/src/eval.rs:4153-4244`
  (`CanonicalRequestV1` decode per op), `:4246-4257` (capability table insert with
  `CapabilityTraceIdentity("interpreter:FS")`), `:4258-4269`
  (`dispatch_host_op_v1` → `reply`, denial routing, `reify_host_reply_v1`, reply
  discarded). Console/Clock dispatch seams: enumerate the sibling
  `dispatch_host_op_v1` call sites in `eval.rs`.
- **Canonical types (PX5, on main):** `ken_host::EffectObservationV1`
  (`ken-host/src/effect_v1.rs:896`), `EffectEventV1`, `CanonicalRequestV1`,
  `CanonicalOutcomeV1`/`SemanticErrorV1`, `CapabilityTraceIdentity`,
  `HostDispatchReplyV1`, `dispatch_host_op_v1`.
- **Native counterpart (the shape to mirror):**
  `ken-runtime::run_bound_process_effect_observation_v1`
  (`object_linker_packaging.rs:196`) — the native lane's six-field producer.
- **Consumer:** PX6 `crates/ken-verify` (frame
  `docs/program/wp/PX6-effect-differential-harness.md`).
