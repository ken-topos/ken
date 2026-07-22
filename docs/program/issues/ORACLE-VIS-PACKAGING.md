---
id: ORACLE-VIS-PACKAGING
title: "replace the text-pin visibility oracle on build_process_starter_executable_artifact"
status: in-review
owner: runtime
size: XS
gate: none
depends_on: [ORACLE-VIS-CHECK, SRC-ATTEST]
blocks: []
github: null
origin: runtime-implementer, found while implementing ORACLE-VIS-CHECK; routed by runtime-leader
---

## ⏸ CURRENT STATE (2026-07-22) — built, QA-clean, HELD on `SRC-ATTEST`

`wp/ORACLE-VIS-PACKAGING @ 8dad30de` is **QA-clean after three rounds** and the
Librarian's content verdict is **APPROVED / benign**. It cannot merge yet: it
edits a manifest-cited source, and until `SRC-ATTEST` lands there is **no valid
`library/REVISION` it can carry** (Librarian impossibility proof; Architect
ruling `dec_7q3kes0jcx1kn`). ⛔ **The hold is the Steward's sequencing call —
there is no defect in the runtime ring's candidate.** The sections below are the
original framing and remain accurate as the build contract.

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

## ⛔ CORRECTION (steward, at framing) — the section below described the REJECTED candidate

**The original filing pointed at a `compile_fail` doctest lane. That design was
rejected and never landed.** It was written while that candidate was live;
`ORACLE-VIS-CHECK` then changed mechanism entirely mid-WP and this file was
never re-derived against what merged. Verified against `origin/main @ dd715950`:

| the filing claimed | what is actually on `main` |
|---|---|
| "the CI `--doc` lane already exists … a `Doctests` step in `ci.yml`" | **no such step.** `ci.yml` carries only a comment saying the workspace has no doctests to lose |
| reuse the `//! ```compile_fail` block form | **one** `compile_fail` occurrence in the entire test file; the landed oracle is not doctest-based |

⛔ **Rebuilding the doctest form re-enters the exact rejection that cost a full
review cycle**: it requires editing `.github/workflows/ci.yml`, and **the
publisher credential lacks workflow-write**, so such a branch is rejected at
push before a PR exists. No local signal can see that boundary.

★ @runtime-leader's own carry from `ORACLE-VIS-CHECK`, adopted here as a
framing-time check: *"does this diff touch a path the publisher can push"* is a
**distinct axis** from *"which reviewer lane does this diff need"*. Ask it
before building against a mechanism, not after a rejection.

## ✅ The mechanism that actually landed — reuse THIS one

A **`rustc` subprocess probe compiled against the built `ken_runtime` rlib**
(`assert_helper_is_not_reachable_from_outside_ken_runtime`, same file). It
touches `crates/` only, so it is deliverable. Three load-bearing parts:

- **Freshest rlib wins** — `sort_by_key(Reverse(mtime))`. `deps/` accumulates
  one rlib per build (15 spanning a day, when written). Selecting by filename
  hash makes the probe report on hours-old source **while every signal looks
  healthy, the positive control included**. Freshness is a third axis,
  independent of the other two.
- **The positive control also SELECTS the rlib** — a probe form that cannot
  resolve at all would make every negative probe fail for its own reasons.
  ⚠ See the F6 note below: that dual role is also a live defect.
- **Assert on error CODES** (`E0432`/`E0433`/`E0603`), never diagnostic prose.

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
- ⛔ **There is no CI `--doc` lane and you must not add one** — see the
  correction above. The landed mechanism needs no `ci.yml` change at all.

## ★★ F7 (@adversary, at framing) — the third conjunct is INEXPRESSIBLE here

This is the substance of the framing and it upgrades the warning above. It is
not that the third conjunct is *easy to forget* — **remembering does not help,
because a cross-crate probe has nowhere to put it.**

Confirmed at `object_linker_packaging.rs:653` — `#[cfg(test)] fn`, no `pub`.
The surviving pins are:

```rust
:906  assert!(!packaging.contains("\npub fn ..."));
:907  assert!(!packaging.contains("\npub(crate) fn ..."));   // <- this one
```

⛔ **From `ken-cli`, bare-private and `pub(crate)` are the same observation.**
Both are unreachable, and both yield `E0432`/`E0433`/`E0603`. The mechanism
asserts on error *codes* rather than prose — which is correct, and is precisely
what removes any power to separate the two.

⇒ Converting by copying the cranelift form does not merely *miss* a conjunct:
**it drops a property the mechanism cannot state, and drops it invisibly.** The
converted test goes green, and a widening from bare-private to `pub(crate)` is
no longer caught by anything.

**Two mechanisms can express it. Either is acceptable; silently losing it is
not:**

1. A probe from **inside `ken-runtime`** — a sibling-module reference under
   `#[cfg(test)]`. `pub(crate)` resolves; bare-private does not.
2. **Keep that one conjunct as a source-text pin**, with the reason written
   down in-file so the next reader knows it is deliberate residue and not an
   instance the WP failed to convert.

⚠ This is an **expressibility** question, and framing time is the cheap moment
for it — cf. the contract-expressibility discipline. **The ring picks the
mechanism; the frame only forbids losing the property.**

## Acceptance

Same as `ORACLE-VIS-CHECK`, and **both directions must be proved, not asserted**:

1. **Fails** when the helper is genuinely made public production API.
2. **Survives** a pure relocation — different file, different attribute
   placement, visibility preserved.
3. ★ **Fails when the helper is widened to `pub(crate)`** — mutate it to
   `pub(crate) fn`, run, and watch the check go red. This is the F7 property
   and it is stated as a **post-condition on the result, deliberately naming no
   mechanism**: whichever of the two routes above the ring picks, this mutation
   must fail. If it passes, the property was lost.

⚠ When constructing directions 1 and 3, check the crate still **compiles**
under the mutation. On `ORACLE-VIS-CHECK` the first attempt — ungating the
module and making the item `pub` — made the crate fail to build outright, so
the checks "passed" against rubble. **A mutation that breaks the build proves
nothing.**

⚠ And per the same WP's hardest lesson: **when a mutation proof passes where it
should fail, suspect a stale input before you doubt the mutation.** If AC-3's
mutation does not go red, check which rlib the probe actually compiled against
before concluding the property holds.

## ▪ F6 (@adversary) — adjacent defect in the code this WP edits

Same file, same function. The rlib loop is newest-first, but the control loop
takes **the first candidate that compiles `CONTROL`** and silently falls back
to older ones when the newest fails:

```rust
for candidate in &candidates {          // newest → oldest
    if compiled { selected = ...; break; }
    control_failures.push(...);         // discarded unless ALL fail
}
```

Two invariants in one loop — *use the freshest* and *use one the control
resolves against* — and where they conflict the **safety-critical one loses,
silently**. On success nothing reports which rlib was chosen. The false-pass
shape is this WP's own defect class: helper made public in current source →
newest rlib fails the control for an unrelated reason → older rlib selected →
probes hit stale source → `E0603` → **green**.

**Grounding, honestly:** read from control flow; the precondition was not
reproduced (the adversary's worktree had 1 rlib, the implementer's 15). But the
code's own comment establishes that candidates fail the control *by design* —
that is why the loop exists — so it is unmeasured, not hypothetical.

⇒ **In scope, at the observable-first level: report the selected rlib, and say
so explicitly when it is not `candidates[0]`.** Make it visible before making
it fatal. Escalating to a hard failure is a judgment call for the ring — if you
make it fatal, say why in the commit.

## Out of scope

The remaining assertions in that test are **genuinely about generated C source
text** (`.capability = host_init.capability`, `host_init.capability == 0`,
`process_environment_count`) and are correctly expressed as substring matches.
They are not part of this class and should stay as they are.
