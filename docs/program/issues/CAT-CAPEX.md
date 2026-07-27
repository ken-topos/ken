---
id: CAT-CAPEX
title: "catalog exhibits no checked capability/authority exemplar — write one against the landed Cap/Auth surface"
status: merged
owner: ergo
size: M
gate: none
depends_on: []
blocks: []
github: null
origin: "evt_2dgcc89s1yapn (DOC-W1-3 census). Ordering question discharged by Steward measurement 2026-07-27 at origin/main e700b861; frame written the same pass."
---

Frame: [`../wp/CAT-CAPEX.md`](../wp/CAT-CAPEX.md) — shovel-ready, inputs
pinned by blob at `origin/main = e700b861`.

## ⭐ The parked ordering question is answered — this is now releasable

The node previously read **"⛔ Not ready, and deliberately unassigned"** on one
unresolved question: is a capability exemplar blocked on `ABI-R3` and the
membrane, or writable against the landed contract today?

**It is writable today.** Measured, three ways:

1. `Cap : Auth -> Type0` is registered in the elaborator's globals as a real
   surface type; `data Auth = ANone | APartial | AFull` is an ordinary checked
   inductive.
2. Four capability-parameterized `proc`s (`read_bytes`, `write_file`,
   `append_file`, `file_metadata`) **already elaborate** and are green in CI.
3. `crates/ken-interp/tests/i3_fs_floor.rs` already loads the **catalog**
   fragment `Capability/Filesystem/Errors.ken.md` and drives `read_bytes` with
   a real `Authority` — the catalog path and the capability path already meet
   in a passing test.

⇒ Both blockers named on the old node are discharged: ordering (above) and
sizing (**M** — `catalog/` only, additive).

⚠ The old node's **"build side is capped at two implementation tracks"**
premise is stale: five rings are active, and per the operator's 2026-07-27
directive an idle build ring is the Steward's backlog. Ergo holds no other
node.

## ⚠ The original census searched for the wrong thing

The grep that surfaced this looked for `Cap_FS`, `: Cap `, `CapParam`,
`cap_set`, `attenuate`. But `attenuate` is **required by spec to be unbound**
(`38 §1.3.1`), and `Cap_FS` is a **retired** spelling — the landed one is the
authority-indexed `Cap a`. ⭐ A census that enumerates spellings does not
measure the property. **The gap is real; its stated cause was not.**

## ⛔ The single most likely defect

`spec/60-security/62-authority.md §7` is the obvious place to copy from, and it
is **stale on three axes** — the retired `view` keyword, the retired `Cap_FS`
spelling, and `write_at` for the landed `write_file`. Copying it yields a
fragment that cannot check. Write against the prelude signatures the frame
quotes.

That staleness is tracked separately as [`SPEC-AUTH-EX`](SPEC-AUTH-EX.md).
⛔ It is **not** Ergo's to fix and does **not** gate this WP.
