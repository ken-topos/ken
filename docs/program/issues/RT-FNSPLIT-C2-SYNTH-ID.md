---
id: RT-FNSPLIT-C2-SYNTH-ID
title: "closed synthesized-constructor-role identity capability, with the DynamicConstructor producer that consumes it — the identity source compiler-synthesized effect payloads have no occurrence to ask for"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-FNSPLIT-C1, RT-FNSPLIT-B2V, RT-FNSPLIT-B2R]
blocks: [RT-FNSPLIT-B2F]
github: null
origin: Architect ruling evt_xf4znbnb6vz9 on the HostResult producer obstruction raised by runtime-implementer evt_26sy7sknha9e6 and routed by runtime-leader evt_1fv8yq0krfn0, at exact B2F checkpoint fa1af614. The ruling found this a FRAME/PREREQUISITE defect rather than a local D9 lookup repair, because the capability is a planner addition that B2F's own no-widening boundary forbids the implementer from creating locally, and it directed the Steward by name to frame the prerequisite together with its DynamicConstructor consumer. Same shape as B2V (Architect ruling evt_28cnmxf6ncghn on B2F hard-stop #10) and C1 (evt_7ay6s5s79awz8 on #11): one prerequisite inserted ahead of B2F rather than a widening inside it. Steward-filed; Steward owns the frame and the AC/control placement.
---

# `RT-FNSPLIT-C2-SYNTH-ID`

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
