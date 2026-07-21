---
id: RT-SPLIT
title: decompose cranelift_backend.rs
status: draft
owner: runtime
size: L
gate: none
depends_on: [F3-39]
blocks: []
github: null
origin: steward frame
---

`crates/ken-runtime/src/cranelift_backend.rs` is 22,081 lines in a single flat
module. Decompose it into coherent submodules without changing any behavior.
Maintainability only — feeds no G-gate. Frame authored; Phase 0 decomposition
ruling delivered by the Architect (transcribed in the brief §10) and is
binding.

Execution waits only on the Runtime ring becoming free — it must follow
`F3-39` per the current tracker's sequencing, and Runtime is single-threaded
against the rest of the fleet's active queue.

Full brief: [`docs/program/wp/rt-split-cranelift-backend.md`](../wp/rt-split-cranelift-backend.md).
