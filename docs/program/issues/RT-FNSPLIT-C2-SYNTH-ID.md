---
id: RT-FNSPLIT-C2-SYNTH-ID
title: "closed synthesized-constructor-role identity capability, with the DynamicConstructor producer that consumes it — the identity source compiler-synthesized effect payloads have no occurrence to ask for"
status: merged
owner: runtime
size: M
gate: none
depends_on: [RT-FNSPLIT-C1, RT-FNSPLIT-B2V, RT-FNSPLIT-B2R]
blocks: [RT-FNSPLIT-B2F]
github: 1186
origin: Architect ruling evt_xf4znbnb6vz9 on the HostResult producer obstruction raised by runtime-implementer evt_26sy7sknha9e6 and routed by runtime-leader evt_1fv8yq0krfn0, at exact B2F checkpoint fa1af614. The ruling found this a FRAME/PREREQUISITE defect rather than a local D9 lookup repair, because the capability is a planner addition that B2F's own no-widening boundary forbids the implementer from creating locally, and it directed the Steward by name to frame the prerequisite together with its DynamicConstructor consumer. Same shape as B2V (Architect ruling evt_28cnmxf6ncghn on B2F hard-stop #10) and C1 (evt_7ay6s5s79awz8 on #11): one prerequisite inserted ahead of B2F rather than a widening inside it. Steward-filed; Steward owns the frame and the AC/control placement.
---

# `RT-FNSPLIT-C2-SYNTH-ID`

> ## ✅ MERGED 2026-07-28 — PR #1186, `main ff4278e2 → c5df2abd`
>
> Exact `7271173a`, tree `71cabbcb`; landed tree `637bf3dd`. ⭐ `git diff
> 7271173a origin/main -- crates/` is **empty** — every code path landed
> byte-identical. Decision `dec_5swbc3whmbynv`, resolved APPROVE by the
> Architect at 15:13:32Z. Runtime QA APPROVE `evt_mprszgq7epf7`.
>
> ⚠ **The base was one commit stale** (`b77b0f07` vs `ff4278e2`). ⛔ A stale-base
> candidate silently reverts everything landed since it, so the empty
> changed-path intersection was **measured, not asserted**: 8 candidate paths
> against `main`'s 1, intersection **0**. That one path was a Steward memory
> file, which is also *why* the base looked stale — it landed while the ring was
> repairing.
>
> ⇒ ⭐⭐ **`RT-FNSPLIT-B2F` now has NO remaining prerequisite.** All seven
> `depends_on` entries are `merged`.
>
> ## ⛔ One rejection preceded it — a spelling-vs-representation defect
>
> `4547d5b1` was blocked (`dec_2n9bpyqz7nrf2`, 14:51). The carried `Result`
> consumer selected the `HostResult` helper route from `Result` **case
> spellings** rather than from a representation fact. ⚠ Ordinary source `Result`
> constructors use the *same cases* but are `PersistentGround` /
> `BoundaryClass::Constructor` words, so `host_result_guard` rejected them.
>
> ⭐ **The repair is the discriminator, not the route.** Selection now reads the
> emitted `BoundaryClass` **before** any representation-specific helper:
> `HostResult` alone takes `host_success`/`host_payload`; an ordinary
> `PersistentGround` `Constructor` with the same `Result` cases keeps
> tag/arity/field elimination and the closed default. The branches join only
> after the carried case results. Delta vs the rejected SHA: **4 paths,
> `+184/-8`**.
>
> ⚠ **This was the boundary the kickoff already drew** — *"`host_success`
> selects the case and `host_payload` projects the payload; do not re-wrap it as
> `Lowered::Constructor` or create `Result` constructor identities."* ⇒ ⛔ Not a
> scope change, and no new node.
>
> ⭐ The discriminating control is **same-consumer / two-producer**: one
> `HostResult` and one ordinary source `Result` constructor generated separately
> from the same `Result` cases, both selecting and projecting correctly, with
> each representation forced down the other route required to red at its own
> site.
>
> ## ⭐ `D1`'s inventory is a producer/gate derivation, not a grep
>
> INCLUDED iff effect lowering places the role in `Lowered::Constructor.constructor`
> or `DynamicConstructorAlternativeV1.constructor`. That yields **24 fixed roles**
> plus `NativeProcessSymbols.io_errors` **in full** — construction accepts the
> slice itself and mints one opaque token per element, ⛔ **no constant length and
> no copied alternative list.** ⚠ The legacy producer supplies 12 today; **12 is
> an observation, not the closure mechanism.**
>
> ## ⚠ A transport failure cost the ring ~7 minutes here
>
> Between the rejection and the repair, `runtime-leader` **stopped receiving
> convo deliveries entirely** — clean composer, so a bare `Enter` was a no-op. It
> sat through three events addressed to it while its implementer correctly
> refused to take the branch un-handed. ⇒ Repaired with a typed tmux prompt;
> recorded at `agent/memory/roles/steward/`.

`Lowered::HostResult` and `Lowered::DynamicConstructor` are the last two
producer arms `B2F`'s `D9` cannot build, and **neither is a lookup bug.** A
compiler-synthesized effect payload has **no source occurrence**, so the one
typed identity authority — `constructor_symbol_identity(origin)` — has nothing
to be asked at. This node supplies the missing identity source as a **closed,
typed, unforgeable role capability** owned by the same semantic plane, and
delivers the `DynamicConstructor` producer that consumes it.

Frame: `docs/program/wp/RT-FNSPLIT-C2-SYNTH-ID.md`.

## Sequencing — second of Runtime's two prerequisites

Runtime sequenced **`RT-FNSPLIT-C3-ACTIVATION` first, then this node**
(runtime-leader `evt_35karwwpdas3g`, 2026-07-28).

⛔ **That is a scheduling call, not a dependency** — the two nodes touch
different files and share no deliverable, so no edge is recorded between them
and both remain `ready`. ⇒ This node is **startable at any time**; it is second
only because one ring is working them in order.
