---
id: SPAN-SEAL
title: seal the BufferSpan producer surface
status: merged
owner: foundation
size: M
gate: none
depends_on: []
blocks: [RT-PARITY]
github: null
origin: steward frame, Architect ruling + PX8-F provenance (2026-07-21)
---

Restored the locked `BufferSpan` abstraction: no public declaration may
produce a `BufferSpan`. A public producer had escaped into source globals
(`crates/ken-elaborator/src/prelude.rs:2076`) and was absent from the
private-name closure, letting checked source mint a `BufferSpan` without
naming `PrivateBufferSpan` — a breach of `spec/30-surface/38-ffi-io.md:356-365`.
Replaced the escaped producer with a public proposition + checked lemma, kept
the advance step private, and pointed the published `writeAll` exact-prefix
law at the mechanism `writeAll` actually executes.

Original size estimate was S–M; recorded here as M (see the brief for the
finer-grained call).

**Merged** `cd4184b8` (PR #798). Verified by content; adversary hunt
`evt_74mjc4txd9y1e` afterward confirmed **the seal holds** (findings S1+S2
folded into the follow-on `SEAL-2`).

Full brief: [`docs/program/wp/SPAN-SEAL-buffer-span-producer-closure.md`](../wp/SPAN-SEAL-buffer-span-producer-closure.md).
