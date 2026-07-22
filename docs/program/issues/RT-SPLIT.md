---
id: RT-SPLIT
title: decompose cranelift_backend.rs
status: merged
owner: runtime
size: L
gate: none
depends_on: []
blocks: []
github: null
origin: steward frame
---

`crates/ken-runtime/src/cranelift_backend.rs` is 22,081 lines in a single flat
module. Decompose it into coherent submodules without changing any behavior.
Maintainability only — feeds no G-gate. Frame authored; Phase 0 decomposition
ruling delivered by the Architect (transcribed in the brief §10) and is
binding.

## ▶ RELEASED 2026-07-22 — sequenced FIRST, ahead of BUDGET-EFF

**Operator, 2026-07-22:** *"From a dev efficiency point of view, I'd like
RT-SPLIT done first."* Correct, and it is the better order: `BUDGET-EFF`
modifies `cranelift_backend.rs:13081-13082`, so running it first would mean
editing a 22,081-line monolith and then splitting underneath that change.
Reversed, `BUDGET-EFF` and every later ABI item touching native land in a
decomposed file.

### The `F3-39` dependency was dropped — it is not grounded

This issue previously declared `depends_on: [F3-39]`, justified only as
*"per the current tracker's sequencing."* **Checked before dropping it:**

- `F3-39` closes two reducer defects in
  **`crates/ken-interp/src/eval.rs` (`prim_reduce`)**.
- `RT-SPLIT` decomposes **`crates/ken-runtime/src/cranelift_backend.rs`**.

**Different crates, different files, no contention, and no stated semantic
obligation** — the ordering was an inherited claim restated rather than
derived. `F3-39` remains real work (`status: draft`, `size: TBD` — size it
before release) and is unaffected by this running first.

**This does not change `PX8`'s closure property.** `RT-SPLIT` still discharges
no clause of it (`issues/PX8.md`); it is sequenced first for developer
efficiency, not because it gates anything.

Full brief: [`docs/program/wp/rt-split-cranelift-backend.md`](../wp/rt-split-cranelift-backend.md).

## ✅ SERIES COMPLETE — 7 of 7 slices merged

**Final slice landed `origin/main @ b9c23a6b` (PR #869, CI-gated).**
`crates/ken-runtime/src/cranelift_backend.rs`: **22,081 → 492 lines.**

Verified by content across all eleven paths; `artifact/api.rs`,
`lowering/core.rs`, and `lowering/mod.rs` all byte-identical to base on `main`.
Production visibility budget closed at **22/24**, zero new widenings.

★ **What made the series verifiable rather than merely sequenced.** Slice 6
scaffolded imports that slice 7 had to delete **with `api.rs` byte-unchanged**,
making slice 6 retroactively falsifiable by slice 7's diff — a property with a
live oracle five slices later, not an assertion. The no-re-touch rule on
`core.rs`/`lowering/mod.rs` held across all five governed slices for the same
reason.

★ **Defects were surfaced by their author, pre-review, in every case:** the
37-item ledger's two wrong rows (found by *compiling*, not analysis, and
reported as "assume more rows are wrong"); the first re-export enumeration
missing `pub(crate)` items behind a glob (caught by luck, and said so); the
out-of-scope `store.rs` rustfmt hunk (self-caught, with the `git diff -w`
false-empty trap named). That disclosure norm is the transferable artifact.

⇒ **Releases BUDGET-EFF's deferred native half** — `cranelift_backend/lowering/`
is no longer exclusive territory.
⚠ **And expires the disjointness premise:** `ken-runtime` had **zero**
`TransferCountV1` references, which is what made the two tracks safely
concurrent; the native half introduces the first.

⏳ Retros outstanding from the runtime ring — the WP is not closed until they
are in (COORDINATION §10).
