# PX3 — Machine/ABI scalar types in Ken (`USize`/`ISize`/`CInt`)

**Owner:** Team Language · **Size:** **M** (grew from S on grounding — the PX2
manifest carries *no* width fact yet, so PX3 must add one before it can bind; see
§1) · **Branch:** `wp/px3-abi-scalars` (cut fresh: `git branch
wp/px3-abi-scalars origin/main` at build time — `origin/main` carries this frame)
· **Gate:** Language QA + **Architect §14** (trusted-base-delta + the
manifest-inventory closure axis — the same axis that caught the PX2 hole). **No
CV** unless you add a `conformance/` fixture (you should not). **FULL CI**
(touches `crates/`).

> Measured against the operator's intent (`09-posix-linux-abi-campaign.md §3`):
> *"a tool for doing real practical work in the safest way possible."* A machine
> integer whose width is **hand-asserted** is precisely the §0 defect this
> campaign exists to kill. PX3's width **must be probed and manifest-bound**, or
> it is a new instance of the disease. *"Safest possible"* here = a narrowing
> conversion that **cannot silently truncate** — it returns `Result`.

---

## 0. Objective (one line)

Introduce the machine/ABI scalar types **`USize`**, **`ISize`**, and the **`CInt`**
family to Ken as **opaque primitives whose widths are bound to (and cross-checked
against) the target-ABI manifest**, with **explicit, partial** conversions
to/from arbitrary-precision `Int` where **every narrowing returns `Result`, never
a silent truncation**.

## 1. The premise correction you must build on (grounded 2026-07-15 vs `origin/main`)

**The PX2 manifest today carries ONLY filesystem ABI facts** — `OFlags`,
`AtFlags`, `Mode`, the 5 `openat`-family syscall numbers, and 2 errnos
(`crates/ken-host/build.rs::linux_raw_facts()` `:108-147`). **There is no
pointer-width fact and no `c_int`-width fact.** `git grep pointer_width|c_int|CInt
crates/ken-host/` → zero. So the charter phrase *"USize bound to a manifest
value"* has **no field to bind to yet.**

⇒ **PX3's first deliverable is to ADD those width facts to the manifest**, exactly
as PX2 established: a value from `rustix`/`linux_raw` (or `core::mem::size_of`),
**cross-checked by a system-header probe query in `abi_probe.c`**, and covered by
the fail-closed `verify_inventory_closure` gate. **A width that is not probed and
inventory-closed is the PX2 defect reintroduced — Architect §14 will reject it.**

This makes PX3 **cross a lane boundary** into `crates/ken-host` (Runtime's crate).
Per COORDINATION §9a this is **owner-assigned spillover**: Team Language owns PX3's
branch and does this edit, following the PX2 pattern verbatim (§3.A). It is a
**mechanical extension of an established mechanism**, not a redesign of the gate.

## 2. Fixed inputs — SETTLED, do not reopen

- **The kernel does not grow.** `USize`/`ISize`/`CInt` are **opaque primitive
  types** registered through the **existing** `ken_kernel::declare_primitive`
  API with `PrimReduction::OpaqueType` — the identical mechanism that already
  introduces `Int`, `Int8…UInt64`, `Bytes`, `String`
  (`crates/ken-elaborator/src/numbers.rs:230-300`; `bytes.rs:55-60`). **No new
  trusted typing rule.** A WP that reaches for one is a scope fork → Architect,
  not an implementer call (campaign §7).
- **No ABI fact without a probe (campaign §7).** Every width fact is manifest-
  generated **and** cross-checked by `abi_probe.c`; disagreement **fails the
  build closed**. This is non-negotiable — it is the entire point of PX0–PX2.
- **`rustix` is SETTLED (operator) — never re-ask.** It is the private
  `linux_raw` binding source inside `ken-host`; state it as fact when you touch
  `ken-host`, do not re-surface it.
- **V1 support is `target_os = "linux"`, never `unix`.** Any non-Linux target
  gets the existing **explicit-unavailable** lane; a broad `cfg(unix)` must not
  imply a width contract we never manifested (campaign §3, the §0 bug).
- **`Result` is the prelude `data Result e a = Err e | Ok a`**
  (`crates/ken-elaborator/src/prelude.rs:224`; **constructor order `Err` then
  `Ok`**). Narrowing conversions return `Result` over a Ken-source range error,
  **not** `Option` (the existing `conversions.rs` narrowings return `Option` —
  PX3 does **not** migrate those; it adds `Result`-returning conversions for the
  new ABI scalars — §3.C).
- **Trusted-base-delta is bounded and accounted.** The existing pattern already
  budgets exactly-one retract postulate per width and asserts it
  (`conversions.rs:116-166`, `uint8_int_retract`). PX3 states and bounds its own
  delta the same way; a QA/§14 check confirms the delta set is exactly what the
  frame declares (§4 AC4).

## 3. Mandated deliverable — each part ends in a concrete choice

### A. Add the width facts to the `ken-host` manifest (spillover; PX2 pattern)

Add **two** `AbiFact`s to `linux_raw_facts()` (`crates/ken-host/build.rs:108-147`):
`POINTER_WIDTH` (bits; 64 on the V1 target) and `C_INT_WIDTH` (bits; 32 on the V1
target). For **each**:

1. Emit the value from the same authority PX2 uses (`linux_raw`/`core::mem` for
   pointer width; the C ABI for `int`), **not** a literal typed from memory.
2. Add the **matching query to `abi_probe.c`** (`FACT=INTEGER` protocol, values
   only — the clean-room constraint from `spec-leader evt_5fx7gmprrk07b` binds
   here: no header text, identifiers-as-queries only, target-qualified).
3. Extend **`verify_inventory_closure`** (`build_support.rs:13`) so the new facts
   are in the **bidirectional** producer↔registry↔observer set — a missing OR
   extra width fact **fails the build closed** (this is the PX2 §14 lesson;
   do not add a presence-only needle).

**Concrete choice:** the two facts are named `POINTER_WIDTH` and `C_INT_WIDTH`,
carry **bit widths** (not byte widths), and join the existing `AbiFact` vector so
the manifest hash covers them. If the `CInt` family needs more than `c_int` (e.g.
`c_long`), add each as its own probed fact — never derive one width from another
by assumption.

### B. Register the scalar types (Language crate, `numbers.rs` pattern)

Register `USize`, `ISize`, and the `CInt` family as **opaque primitives** via
`reg_ty!`/`declare_primitive` alongside the existing width types
(`numbers.rs:292-300`). **Concrete choice:** `USize`/`ISize` are the
pointer-width unsigned/signed machine words; `CInt` is the C `int` type. Their
representation is opaque (like `UInt8`); they are **distinct nominal types**, not
aliases of `UInt64`/`Int64`, so a Ken program cannot silently confuse a machine
word with a fixed-width integer.

### C. The conversions — widening total, narrowing `Result`-partial

Follow the `conversions.rs` split (native floor primitive in Rust + derived Ken
source via `elaborate_decl`):

- **Widening to `Int`** — total: `usize_to_int : USize → Int`, `isize_to_int :
  ISize → Int`, `cint_to_int : CInt → Int`. Native `PrimReduction::Op`, identity
  reduction, tested-not-trusted (`conversions.rs:88-113`).
- **Narrowing from `Int`** — **partial, returning `Result`**:
  `intToUSize : Int → Result RangeError USize` (and `ISize`, `CInt`), generated
  as ordinary Ken source, checking `min ≤ n ≤ max` where **`min`/`max` derive
  from the manifest width fact** (not a literal). On out-of-range → `Err
  (rangeError …)`; in-range → `Ok (int_to_…_raw n)`. **Concrete choice:** define
  a small Ken `RangeError` payload (the offending value + the target type's
  name/bounds) so the error is actionable, and place it in the same catalog/
  prelude location as the conversions. `min`/`max` for the signed/unsigned
  machine words are computed from `POINTER_WIDTH`; for `CInt` from `C_INT_WIDTH`.
- **Do NOT** provide a silent/unchecked public narrowing. The `_raw` cast stays
  **private/internal** to the checked wrapper (as `int_to_uint8_raw` is today).

### D. Where the code lives

Scalars + native conversion floors: `crates/ken-elaborator/src/` (extend
`numbers.rs`/`conversions.rs`). Derived Ken-source conversions + `RangeError`:
prefer a literate catalog package (the `bytes_nat_length` model,
`catalog/.../Collections.ken.md`) **or** `elaborate_decl` in the elaborator,
matching the existing `conversions.rs` approach. **Concrete choice:** keep the
public conversion API in **one** location; do not scatter it.

## 4. Acceptance criteria (testable)

1. **AC1 — probed, not asserted.** `POINTER_WIDTH` and `C_INT_WIDTH` are in the
   generated manifest AND queried by `abi_probe.c`; flipping either the producer
   value or the probe value **fails the build closed** with an
   inventory-closure error (demonstrate both arms — producer-only drift and
   registry-only drift — per the PX2 bidirectional gate).
2. **AC2 — types present, distinct, opaque.** `USize`/`ISize`/`CInt` elaborate,
   are nominally distinct from the fixed-width types, and add **no** kernel
   conversion rule (`ken check` on a program using them passes; the kernel sees
   opaque primitives).
3. **AC3 — narrowing cannot truncate.** `intToUSize`/`intToISize`/`intToCInt`
   return `Result`; an out-of-range input yields `Err`, an in-range input `Ok`;
   there is **no** public unchecked narrowing. Include a boundary pair per type
   (max in-range `Ok`, max+1 out-of-range `Err`) — a non-degenerate pair, not a
   single positive case (COORDINATION §7).
4. **AC4 — bounded trusted-base delta, stated.** The `trusted_base()` delta is
   exactly the declared set (the new opaque type ids + the accounted retract
   postulates, budgeted as in `conversions.rs:116-166`), and QA/§14 confirms the
   delta set equals what this frame declares — no unaccounted trusted addition.
5. **AC5 — widths bind to the manifest.** The narrowing bounds are **derived from
   the manifest width facts**, not literals; changing the manifest width would
   change the bounds (structurally, e.g. the bound expression references the
   fact, or a build-time substitution ties them — demonstrate the linkage, don't
   just assert it).
6. **AC6 — no regression.** Workspace-green **in CI** (never a local
   `--workspace` run — COORDINATION §12); existing numeric/conversion suites pass.

## 5. Guardrails (do-not-reopen)

- ⛔ **No hand-asserted width.** Every width is probed + inventory-closed (§7
  campaign). A literal width in the boundary is a **defect**, not a shortcut.
- ⛔ **No kernel growth / new typing rule.** Opaque primitives + Ken-source
  conversions only. A genuine need for a kernel rule is a **scope fork →
  Architect**, halt and escalate.
- ⛔ **No silent narrowing.** Public narrowing is `Result`-partial, full stop.
- ⛔ **Do not migrate the existing `Option`-returning conversions** — out of
  scope; PX3 adds the ABI scalars only.
- ⛔ **`catalog/`·`crates/` ⇒ FULL CI**, never `--doc-only`. Build TARGETED only
  (`ken-cargo -p <crate>`), never `--workspace` (operator hard rule).
- ⛔ **rustix is settled** — state as fact, never re-ask.

## 6. Dependencies & context

- **Depends on:** PX2 (merged, `origin/main` `626b38dd` — the manifest + probe +
  `verify_inventory_closure` you extend) and PX1 (merged, `609dd600` — `ken-host`).
- **DAG:** `PX2 → PX3` (`09-posix-linux-abi-campaign.md §6`, sibling of PX4).
- **Perishable anchors:** every `file:line` here was grounded against
  `origin/main` on 2026-07-15; **re-verify at build time** (the campaign's own
  §0a discipline — enumerate, count, re-pin; a line number tells you where
  something IS, not where its KIND begins/ends).
