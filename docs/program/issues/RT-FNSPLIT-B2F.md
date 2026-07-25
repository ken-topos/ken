---
id: RT-FNSPLIT-B2F
title: "functionization and authority switch — per-static-origin Cranelift target functions, atomic with switch-over, equivalence evidence, and old-path removal"
status: active
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-B2A-S]
blocks: []
github: null
origin: Architect ruling evt_6h5gw5c503n5z plus amendment evt_25ynt8615r9sk answering Steward Q1-Q3 on merits (2026-07-25), gated behind research advisory evt_4w1rf45d4fkv3. Replaces the D1/D2 half of the retired RT-FNSPLIT-B2A frame. Steward-filed; Steward owns the replacement frame and AC/control placement.
---

> ## ✅ `ready` — SHOVEL-READY FRAME WRITTEN 2026-07-25, anchors on `0aa9e53f`
>
> **The frame is
> `docs/program/wp/RT-FNSPLIT-B2F-functionization.md` — BUILD FROM IT.**
> This file remains the durable home of the Architect's mechanism rulings (a
> ruling that lives only in a channel thread is not a deliverable); the frame
> carries the deliverables, ACs, and measured anchors.
>
> **The predecessor is clear:** `RT-FNSPLIT-B2A-S` merged at `origin/main` =
> `145fe915`, tree byte-identical to the approved `82356022`. This node closes
> symptom-inventory **entry 2 — the last open entry**, so it is the node that
> closes `RT-NATIVE-FNSPLIT`.
>
> ⛔ **EVERY ANCHOR IN THE PROSE BELOW IS STALE — use the frame's.** Re-derived
> on `0aa9e53f`: `lower_expr` is at `core.rs:4333` (**not** `:3847`; it has moved
> `:3847 → :4255 → :4333` across three re-frames), and the real production call
> count is **58** `self.lower_expr(` sites spanning `:310`–`:6743`, not "60".
>
> ⚠ **Three framing findings the draft below does not contain:**
>
> 1. **The pin `B2F` breaks first is already committed** —
>    `correspondence_adds_no_emitted_unit_to_the_production_census`
>    (`lowering/core/tests/control.rs:3336`) asserts an *exact* emitted-unit
>    census (`core.rs` 1 builder / 1 definition / 2 declarations; four other
>    files all zero). `B2F` must **re-baseline it to a PREDICTED number, not to
>    the observed output**, and must not weaken it — escape-clause condition (1)
>    rests on it.
> 2. ⭐ **The census population is narrower than "the backend."**
>    `crates/ken-runtime/src/native_int_clif.rs` is **production** (un-gated at
>    `lib.rs:23`) and holds **5** `FunctionBuilder::new` sites with its own
>    declare/define helpers, yet is in **neither** the N1 census nor
>    `BACKEND_PRODUCTION_SOURCES`. The landed pins are correctly scoped and say
>    so — **but this node owns a scaling verdict, and a verdict whose denominator
>    silently excludes a sibling production emitter measures the wrong
>    population.** New **AC-G0** requires the denominator be named and every
>    exclusion justified.
> 3. **`B2A-S`'s AC-4 pins the `origin -> expression` lookup count at exactly
>    one.** A second consumer reddens it **correctly**; route through
>    `retained_body_occurrence` or re-baseline explicitly.
>
> ⚠ **Not yet released.** Fleet is single-threaded; run the §2c gate before
> kicking.

## This is a CONSTRUCTION. The frame must say so in those words.

The retired `RT-FNSPLIT-B2A` called this a behaviour-preserving **port** because
its `Retain` list was inherited from `b077eb7a`, a branch that **never landed**.
On the real base there is **one** production Cranelift function and **no**
emitted-unit population to re-key. ⛔ **The frame must describe the target units
as NEW, never as retained ones.**

## ✅ Q1 RULED — shape (a), per-static-origin Cranelift functions, ON MERITS

**One closed Cranelift target function per static planned function/origin,
forward-declared as a bundle, with the fixed explicit activation frame.**

⚠ **This is explicitly NOT carried from `b077eb7a` or from the invalid frame** —
the Architect re-decided it from scratch, and the held branch contributes **no
authority**. The four stated merits, transcribed because they are the reason this
choice is not re-openable on taste:

1. **The operator gate is PER-FUNCTION growth.** Ken's original failure was one
   Cranelift `Function` accumulating the whole program's lowering state. A
   direct-label/CFG machine inside one Cranelift function **still** grows that
   function's IR/VReg population with every static transition — it changes the
   control representation without establishing a bounded per-function unit.
2. A data-driven bytecode/instruction VM *could* keep one interpreter function
   bounded, but needs new instruction semantics, a decoder/dispatcher, a code
   store, and a runtime machine — **a larger new abstraction** than the
   already-planned semantic programs plus Cranelift module declarations, and it
   moves execution off the backend's current direct-code contract **with no
   demonstrated need**.
3. **`cranelift_module::Module` already supplies the right bundle boundary** —
   declare all signatures/IDs first, then define each body. The landed plane
   already has exact static origins, program IDs, capture layouts, and
   `PredeclaredFunction` records. ⚠ **Those records are evidence of FIT, not proof
   that functions already exist** — they make the function bundle the *smaller*
   construction.
4. One closed function per static unit gives the wanted invariant directly:
   dynamic environment/control/store state crosses a **fixed ABI**, code identity
   is **static**, each body has **bounded helper vocabulary**, and **total units
   may be Θ(n) while each function is bounded by its own static
   body/transition contract.** ★ That last clause is the precise scaling claim —
   the frame must state it this way and not as a blanket bound.

## ✅ Q3 RULED — ONE atomic review/merge boundary

**Functionization + live switch-over + differential equivalence + removal of the
old authority are ONE boundary.** ⛔ The ring's proposed live `ii`/`iii` split is
**rejected**: it would leave two live production authorities, which is what
"carrier and removal land together" exists to prevent. **At every landed point
there is exactly one production authority.**

The boundary must include the whole connected mechanism: target code-unit
population · declarations/signatures · the fixed dynamic-frame ABI · persistent-
store transport · static dispatch/call edges · behaviour-equivalence evidence ·
switch-over of **every** live consumer · **removal** of the recursive
whole-configuration body-emission authority.

### ⭐ The ONE permitted escape, as a checkable graph property

A preparatory merge is acceptable **only** when unreachability is mechanically
shown by **all four**:

1. Production still has **exactly** the pre-existing one `FunctionBuilder::new`
   and one root `define_function` path; **no** new production
   `Module::declare_function` / `define_function`, indirect call, dispatch, or
   compiled-module output is reachable.
2. Executable scaffold is **`#[cfg(test)]` only**; production additions are
   **declarative types / validation / data layout only**.
3. **No** feature flag, runtime branch, optional callback, unused function
   pointer, or alternate entry can activate it.
4. The compile-entry reachability census has **zero** production references to
   the scaffold consumer, and a **committed structural test/grep pins that zero
   edge** plus the unchanged one-function census.

⛔ **If preparation needs a production call edge, or emits even one callable
target unit, it is not scaffold** and must travel in the atomic live boundary.
★ **This makes unreachability a checkable graph property, not prose** — and the
committed pin in (4) is what stops it decaying into an assertion.

⚠ Note the cfg(test) asymmetry cuts both ways: a `#[cfg(test)]`-only scaffold is
invisible to a production build *and* a production-only path is invisible to a
test build. Whatever pins condition (4) must be verified in **both**
configurations.

## What this node inherits from the retired frame — decided, not copied

Old `AC-1` (D4 five-category differential suite), old `AC-2` (old-path removal,
⚠ re-scoped: the "whole-configuration emission path" is **not** a separable path
— it is `lower_expr`'s entire recursive-descent inliner, `core.rs:3847`, 60 call
sites), old `AC-3` (the four D3 width invariants, each independently falsifiable),
and old `AC-8` (**no growth claim** — superseded here, since this node *is* where
the scaling verdict belongs) all land on **this** node.

⚠ Old `AC-7`'s scope was right and stays: the **full** `scripts/ken-cargo test -p
ken-runtime`, no filter. ⛔ Workspace, `--locked`, and conformance are CI's.

## Open: what remains of RT-FNSPLIT-B2B

`RT-FNSPLIT-B2B` was framed as "the full emission census + finite differences +
explicit growth verdict." With the scaling verdict now belonging to this node's
atomic boundary, **B2B must be re-derived or subsumed** — do not release it
against its current frame. ⚠ Its premise ("a census taken while the emitter is
still moving measures a moving target") is still sound; what changed is *which*
node the verdict attaches to.
