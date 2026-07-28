---
id: RT-FNSPLIT-C3-ACTIVATION
title: "the opaque activation owner — one Rust representation authority in ken-runtime that constructs, publishes and tears down per-invocation boundary storage, with the deployment-supplied capacity profile and the one-argument public adapter seam"
status: ready
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-C1, RT-FNSPLIT-B2R, RT-FNSPLIT-B2V]
blocks: [RT-FNSPLIT-B2F]
github: null
origin: Architect corrected ruling evt_2yjg12pyqqjdv, bound to exact B2F checkpoint 7ce4198f, which supersedes the owner/capacity/scope parts of the arena ruling evt_1m082dp6xf0mw. Two premises of that ruling were falsified by measurement — runtime-implementer evt_5nc20xj7h9e31 (no production activation owner exists on either launcher path; ken-runtime is rlib-only so BoundaryArenaV1::publish is unreachable from the linked C stub) and Steward evt_53gz05jhsz2fp (the capacity numbers are caller-supplied parameters, so the "existing store/plan capacity authority" has no referent). "Fold it into S6/D6" and "capacity comes from the existing authority" are both retracted, and the former fail-closed clause is withdrawn because applying it would turn every linked executable from working to non-starting. The Architect directed the Steward by name to frame and register this node. Same shape as B2V (evt_28cnmxf6ncghn on B2F hard-stop #10), C1 (evt_7ay6s5s79awz8 on #11) and C2-SYNTH-ID (evt_xf4znbnb6vz9): one prerequisite inserted ahead of B2F rather than a widening inside it. Steward-filed; Steward owns the frame and the AC/control placement.
---

# `RT-FNSPLIT-C3-ACTIVATION`

`B2F`'s `S6`/`D6` switch-over needs the generated root and every unit to take
`(frame_ptr, services_ptr)`. Nothing in production can supply that second
argument: no launcher constructs or publishes a `BoundaryArenaV1`, and the
object-linked starter cannot be taught to — `ken-runtime` is rlib-only, so the
publishing authority is unreachable from the C stub it links.

This node creates the missing owner as **one Rust representation authority in
`ken-runtime`**, exported as an object-linkable static library behind an opaque
activation handle, together with the deployment-supplied capacity profile that
sizes it and the one-argument public adapter seam that lets `B2F` change the
internal convention without touching the public C representation.

⛔ `B2F` is **held at `7ce4198f`** until this node is durable.

Frame: `docs/program/wp/RT-FNSPLIT-C3-ACTIVATION.md`.
