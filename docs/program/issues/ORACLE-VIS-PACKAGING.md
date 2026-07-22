---
id: ORACLE-VIS-PACKAGING
title: "replace the text-pin visibility oracle on build_process_starter_executable_artifact"
status: ready
owner: runtime
size: XS
gate: none
depends_on: [ORACLE-VIS-CHECK]
blocks: []
github: null
origin: runtime-implementer, found while implementing ORACLE-VIS-CHECK; routed by runtime-leader
---

**Filed, not opened.** `ORACLE-VIS-CHECK` was deliberately bounded to the one
`cranelift_backend.rs` instance, and this is a second, same-class instance at a
different site. Recorded so the reasoning survives for whoever picks it up.

## The oracle

`crates/ken-cli/tests/px4b_native_production.rs`, in the **same test** that
`ORACLE-VIS-CHECK` fixed —
`naked_process_ir_helpers_are_not_public_production_api`:

```rust
let packaging = include_str!("../../ken-runtime/src/object_linker_packaging.rs");
assert!(packaging.contains("#[cfg(test)]\nfn build_process_starter_executable_artifact("));
assert!(!packaging.contains("\npub fn build_process_starter_executable_artifact("));
assert!(!packaging.contains("\npub(crate) fn build_process_starter_executable_artifact("));
```

## Why it is the same defect

Identical shape to the one already fixed: **the test's name states a visibility
fact and the implementation pins a literal declaration string.** It is
therefore sensitive to attribute placement, visibility spelling, whitespace,
and which file the declaration lives in — **none of which are the property** —
and it will break on any future relocation of the helper, with each break
looking like a real failure.

★ **The test's name is plural — "helpers".** After `ORACLE-VIS-CHECK`, one
half of that plural is asserted by the compiler and the other half is still
asserted by substring match. That split is the argument for closing this.

⛔ Note the **third** assertion here that the cranelift instance did not need:
a separate `!contains("\npub(crate) fn ...")`. This helper is **bare-private**,
not `pub(crate)`, so its correct state is narrower — any replacement must
preserve that distinction rather than copy the cranelift conjuncts across.

## ✅ Mechanism is already proven — reuse, don't re-derive

`ORACLE-VIS-CHECK` established and mutation-proved the pattern. Reuse it:

```rust
//! ```compile_fail
//! use ken_runtime::object_linker_packaging::build_process_starter_executable_artifact as _;
//! ```
```

Points to carry over, each of which cost something to learn:

- **Cover every reachable path, not just the module path.** `lib.rs:39` is
  `pub use cranelift_backend::*` and `lib.rs:57` is
  `pub use object_linker_packaging::*` — both public globs — so a widened item
  surfaces at the **crate root** as well as under its module path. Enumerate
  the globs for this module before writing the snippets.
- **Use `use ... as _;`, never `let _ = path;`.** The bare `use` form resolves
  a path and checks visibility and nothing else. A snippet that *uses* the item
  also fails to compile on a type-inference error, so for a generic helper the
  `compile_fail` block keeps passing **even after the item is made public** —
  a vacuous pass of exactly the kind being removed. This was caught live on
  `ORACLE-VIS-CHECK` because the cranelift helper takes `impl Into<String>`.
- **Include a positive control.** A `compile_fail` block passes when the
  snippet fails for *any* reason, a typo in the path included. A plain
  (non-`compile_fail`) block naming a genuinely public item in the same import
  form is what distinguishes "correctly unreachable" from "harness broken".
- **The CI `--doc` lane already exists** as of `ORACLE-VIS-CHECK` — a
  `Doctests` step in `ci.yml`, gated to `matrix.shard == 1`. Doctests are not
  partitionable, so an ungated step would run the identical set four times.
  ⛔ Keep the step **inside** `test-shard` rather than giving it its own job:
  `build-test` is what branch protection reads, and a new job absent from its
  `needs:` reports **green no matter how it failed**. `ci.yml` warns about this
  in its own comments.

## Acceptance

Same as `ORACLE-VIS-CHECK`, and **both directions must be proved, not asserted**:

1. **Fails** when the helper is genuinely made public production API.
2. **Survives** a pure relocation — different file, different attribute
   placement, visibility preserved.

⚠ When constructing direction 1, check the crate still **compiles** under the
mutation. On `ORACLE-VIS-CHECK` the first attempt — ungating the module and
making the item `pub` — made the crate fail to build outright, so the
`compile_fail` blocks "passed" for the wrong reason. A mutation that breaks the
build proves nothing.

## Out of scope

The remaining assertions in that test are **genuinely about generated C source
text** (`.capability = host_init.capability`, `host_init.capability == 0`,
`process_environment_count`) and are correctly expressed as substring matches.
They are not part of this class and should stay as they are.
