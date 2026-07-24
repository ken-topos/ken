---
id: RT-PLANNER-DIAGNOSTIC-K
title: "Boundary A planner: report planner-invariant failures as planner defects, and assert fixed_k CONSTANT rather than merely affine"
status: merged
owner: runtime
size: S
gate: none
depends_on: []
blocks: []
github: https://github.com/ken-topos/ken/pull/929
origin: adversary findings H2 + H3 on landed RT-NATIVE-FNSPLIT Boundary A (647a2e5b), side thread thr_2seh2bm1kr5mh evt_4pb2t6ve1ysfr, 2026-07-24. Steward-filed (agents cannot create tracked work per COORDINATION §2); Steward triage = both CONFIRMED against landed code. H1 (zero K headroom) is NOT in this WP — it was routed to the Architect as an input to the hard-stop-#3 ruling.
---

> ## Both defects are on the LIVE native compile path today
>
> Boundary A wires `plan_static_transition_graph` into `build_native_program`
> and discards the result — but the `?` propagates. So a planner bug is a
> user-visible compile failure **right now**, for a plan nothing consumes.

## Scope

⛔ **This is not Boundary B and must not grow into it.** Two narrow changes in
`crates/ken-runtime/src/cranelift_backend/`. No semantic emission, no CLIF, no
census re-derivation, no change to what the planner computes.

⛔ **H1 is deliberately excluded.** The measured `fixed_k = 8` against a cap of
`MAX_HELPERS_PER_STATIC_SOURCE = 8` leaves **zero headroom**, and the hard-stop
#3 remedy is its most likely consumer. That is a mechanism question routed to
the Architect (`evt_yvz82a12b45h`); **do not change the cap in this WP.**

## H2 — a planner bug is reported to the user as an unsupported program

`static_transition.rs:194-196`:

```rust
fn planner_error(detail: impl Into<String>) -> CraneliftBackendError {
    unsupported("NativeStaticTransitionPlanner", detail)
}
```

`unsupported(...)` renders as **`unsupported runtime-IR lowering: …`** — the
channel whose meaning is *"your program uses something we cannot lower."* The
Adversary classified **all 49 production rejection messages**: apart from the
capacity family, **not one names a construct a program is forbidden to
contain.** They are internal-consistency assertions — *"planned helper inventory
is not exact for the closed graph"*, *"static node identity does not match its
closed position"*, *"persistent store depth does not match its child chain"*.

⇒ **A trip is a planner defect. The user is told their program is unsupported.**
There is no fallback path: `artifact/mod.rs:77,98` and `artifact/api.rs:262`
propagate, and `ken-cli` does not catch it.

### Deliverable H2a — honest diagnostic (required, ~2 lines)

Route planner-invariant failures through a distinct error identity that says
**the planner's own invariant failed and asks for a report**, rather than
attributing the failure to the user's program. Keep the capacity family (which
*is* a real statement about the input) distinguishable from the
self-consistency family.

★ This is **not** a trade-off. It is strictly better failure behavior with no
change in what compiles.

### Deliverable H2b — the `?` propagation question (ANSWER, do not decide alone)

Until Boundary B consumes the plan, propagating `?` converts any latent bug in
1899 lines of new, not-yet-exercised-in-anger code into a user-visible native
compile failure for **zero behavioural benefit**. The alternative is to gate
propagation (debug-only, or compute-and-log) so the planner stays exercised in
CI — where a trip is a bug report — while a release compile cannot regress.

⚠ **If the ring wired it unconditionally on purpose**, to shake out planner bugs
early, **that is a defensible trade and this WP should record it rather than
change it.** State the answer either way; an unstated risk window is the defect,
not the choice itself. Route the decision to the Architect if the ring did not
take it deliberately.

## H3 — `fixed_k` is the only census metric that gates production and the only one not asserted constant

`static_transition.rs:1481-1489` asserts six metrics **strictly constant** via
`values.windows(2).all(|pair| pair[0] == pair[1])`: `helper_key_bytes`,
`activation_frame_bytes`, `store_node_bytes`, `helper_key_schemas`,
`frame_schemas`, `store_node_schemas`.

**`fixed_k` is not among them.** It appears only in the *affine* list (`:1431`)
plus a `≤ MAX_HELPERS_PER_STATIC_SOURCE` bound (`:1491-1493`).

⛔ **Affine is satisfied by any linear sequence.** `fixed_k` could drift from a
flat `8,8,8,8,8` to `4,5,6,7,8` — affine ✅, `≤ 8` ✅, suite green ✅ — while the
production headroom is already **negative just past the measured window**.

★ This is the [[a-check-that-measures-a-proxy-passes-for-the-wrong-reason]]
shape: *affine* is a proxy, *constant* is the property, and the metric whose
constancy is load-bearing for a production hard-fail is the one whose constancy
is not asserted. The Adversary had to **run** the census to learn K is flat,
because the assertion does not say so.

### Deliverable H3 — one line

Move `fixed_k` (`max_helpers_per_static_source`) into the constant-asserted
list. **It passes today — measured `8,8,8,8,8`, first differences `[0,0,0,0]`.**

## Acceptance criteria

- **AC-1 — H3 asserts constancy, and the assertion discriminates.** `fixed_k` is
  in the pairwise-equal list. ⭐ **Prove it is not vacuous**: perturb one row's
  `max_helpers_per_static_source` to a still-affine, still-`≤8` sequence (e.g.
  `4,5,6,7,8`) and show the suite **goes red at the new assertion, named**.
  Restore byte-for-byte and re-verify green. *"It went red"* is not the claim;
  *"it went red at this assertion"* is.
- **AC-2 — H2a distinguishes the two families.** A planner self-consistency
  failure and a capacity failure produce **different, correctly-attributed**
  messages. Show both, verbatim. The self-consistency message must not claim the
  program is unsupported.
- **AC-3 — no behavioural change.** The set of programs that compile is
  unchanged by H2a and H3. `scripts/ken-cargo test -p ken-runtime` green.
- **AC-4 — H2b is ANSWERED in the handoff**, with the reasoning, whether or not
  the code changes. If unchanged, state explicitly that the risk window is
  accepted and why.
- **AC-5 — the cap is untouched.** `MAX_HELPERS_PER_STATIC_SOURCE` is still `8`.
  H1 belongs to the Architect's hard-stop-#3 ruling.

## Contention

Touches `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs`
— **the same file Boundary B edits** (B's delta is +215 lines there). ⛔ **These
two contend.** Sequence: land this WP first (it is S and B is stopped anyway),
then have Boundary B re-anchor onto the result. Do not run them concurrently.

⚠ Verify with **content**, not path overlap: after B re-anchors, this WP's
changes must appear exactly once. Path overlap alone proves nothing — see
`agent/memory/build/a-check-that-measures-a-proxy-passes-for-the-wrong-reason.md`.
