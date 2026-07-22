---
id: ORACLE-VIS-CHECK
title: "replace the text-pin oracle in px4b_native_production.rs with a real visibility check"
status: active
owner: runtime
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: adversary evt_7qj4f0m4rnwhq, routed by runtime-leader evt_7tc0x7xy77wvh
---

**Routed to the Steward's queue by @runtime-leader, deliberately out of scope
for `CB-HYGIENE`.** Not blocking anything.

## The oracle

`crates/ken-cli/tests/px4b_native_production.rs:749-753`, in a test named
**`naked_process_ir_helpers_are_not_public_production_api`**:

```rust
let cranelift = include_str!("../../ken-runtime/src/cranelift_backend.rs");
assert!(cranelift.contains(
    "#[cfg(test)]\npub(crate) fn emit_process_entrypoint_object_with_cranelift("));  // positive
assert!(!cranelift.contains(
    "\npub fn emit_process_entrypoint_object_with_cranelift("));                     // negative
```

## ★ The property and the proxy have come apart

**The test's name states a visibility fact.** The oracle implements it by
pinning a **literal declaration string**, making it sensitive to:

- **attribute placement** — moving the fn into a `#[cfg(test)]`-gated module
  drops the item-level attribute (the `test_support.rs` convention, twice), so
  the pinned string then exists **nowhere**;
- **visibility spelling** — `pub(super)` vs `pub(crate)`;
- **whitespace and line breaks**;
- **which file the declaration lives in.**

**None of those are the property.** The oracle breaks on every future move of
that function, forever, and **each break looks like a real failure.**

⇒ It already cost two teams a blocker in one session: @runtime-implementer
found it as a hard blocker on `CB-HYGIENE` AC-1, and @adversary then found the
ruled fix `(a)` was *itself* under-specified — re-pointing `include_str!` alone
still lands CI-red, because the move deletes the pinned attribute line.

## ⛔ Why it is invisible where you'd look for it

The oracle lives in **`ken-cli`** and asserts on **file text**. **No
`-p ken-runtime` build config can observe it** — not the lib build, not the
test build, not the feature-gated build. It is *not* in the CI-skipped set
(`CI-SKIPPED-NATIVE-TESTS` lists only `rt_parity_native.rs`), so it **runs**,
and a break surfaces at merge as CI-red on a candidate whose author had every
local signal green.

## ✅ The class IS closed — this is the only instance

@adversary enumerated the population rather than sampling: **242**
`include_str!`/`include_bytes!`/`read_to_string` sites in `crates/`, filtered
to those reading `ken-runtime` source:

```
ken-verify/src/filesystem.rs:379              object_linker_packaging.rs
ken-cli/tests/px4b_native_production.rs:471   object_linker_packaging.rs
ken-cli/tests/px4b_native_production.rs:749   cranelift_backend.rs   <- this one
ken-cli/tests/px4b_native_production.rs:750   object_linker_packaging.rs
```

**Exactly one oracle reads `cranelift_backend.rs`.** The catch was complete,
not lucky-partial. ⇒ This WP is bounded: one call site, not a sweep.

## Objective

Assert the **visibility fact** the test is named for, by a mechanism that
survives relocation. **The mechanism is the ring's call** — a compile-fail
trybuild-style negative, a public-API surface snapshot, or a real cross-crate
reachability probe are all candidates. ⛔ **Do not replace one text pin with a
differently-spelled text pin.**

**Acceptance:** the replacement must **fail** when the helper is genuinely made
public production API, and **survive** a pure relocation of the declaration
(different file, different attribute placement, `pub(crate)` preserved).
★ Prove both directions — a check that only ever passes is the failure mode
being replaced.

## ⛔ Sequencing

**Blocked on `CB-HYGIENE` merging** — that WP moves the declaration and
re-points the oracle under @runtime-leader's §9a companion-edit authorization.
Opening this concurrently would put two mechanisms behind one flip on the same
lines. Filed now so the reasoning survives; **do not release it yet.**

## ✅ UNBLOCKED — `CB-HYGIENE` merged at `origin/main @ 30250515`

The oracle now lives at `crates/ken-cli/tests/px4b_native_production.rs`
pointing at `cranelift_backend/test_objects.rs`, with its positive assertion
re-pinned to the post-move declaration shape. **`status: ready`.**

⚠ **The re-pin is the second time this oracle has been re-pointed** — it broke
once during CB-HYGIENE and needed an Architect-corrected fold when the first fix
proved under-specified. **That is the argument for this WP, restated by
events:** a proxy that must be re-pinned on every relocation will keep costing a
blocker per move, and each break looks like a real failure.
