---
id: RT-FNSPLIT-B2R
title: "representation and call-ABI contract — a stable executable contract for every value that crosses a generated-function boundary, inert"
status: draft
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-B2O]
blocks: [RT-FNSPLIT-B2F]
github: null
origin: Architect ruling evt_842spc7t6js1 on RT-FNSPLIT-B2F hard-stop #9 (2026-07-25), items 1-4 plus the edge-agreement half of item 6, with the mechanical constraints of Architect addendum evt_t4fykh52ncb answering the Steward's four grounded facts (evt_34pvqr0vte0pr); gated behind research advisory evt_531c4k52mshrn per the armed #9 pull. Steward-filed under the ruling's grant of re-slicing and sequencing ownership; Steward owns the frame, scope, and AC/control placement.
---

> ## ⛔ `draft` — FRAME NOT YET WRITTEN. Do not start.
>
> The **second** of two inert prerequisites for `RT-FNSPLIT-B2F`, sequenced
> after `RT-FNSPLIT-B2O`. The shovel-ready frame will be
> `docs/program/wp/RT-FNSPLIT-B2R-representation-abi.md`. No construction
> authority exists until that frame is written and the Steward kicks it.

## Why this node exists, and what it is NOT

`RT-FNSPLIT-B2F` asks for one closed callable unit per static origin. Hard-stop
#9 established that the current plane cannot express such a unit: emitted
signature is `(pointer) -> i64`, `Lowered` is a **compile-time specialization
lattice rather than a value domain** (only scalar variants hold `ir::Value`;
`String`, `Bytes`, `Constructor { args }`, `Record { fields }`, `Closure` carry
host Rust data with no emitted representation), `CaptureSlot` carries only an
ordinal, and `PredeclaredFunction` has no signature, slot layout, ownership, or
calling convention.

⭐ **The prerequisite is NOT "build one universal boxed `Value` first".** Both
the research advisory and the ruling are explicit: what must exist first is *a
stable, executable representation contract for every value that crosses a
generated-function boundary*. That contract may be uniform, or a family of
statically typed per-origin layouts. Boxing is one admissible implementation of
the contract, not the contract.

⛔ And it is **not** the residual-specialization alternative. Carrying
configuration dependence in residual *function identity* conflicts with
one-unit-per-`PredeclaredFunction` and total Θ(n) unless the specialization
key's cardinality is separately proved bounded — which nothing here does.

## Ruled scope (Architect items 1-4 + the edge-agreement half of item 6)

1. **One fixed call-ABI scheme** — a common activation-frame header plus a
   statically declared payload layout per `PredeclaredFunction`. "Fixed frame"
   means one closed *layout language* and one common control/store/result/trap
   convention. It does **not** require every origin to have the same byte size.
2. **Closed slot layouts** — every parameter, capture, result, control value,
   trap value and persistent-store handle has a declared kind, width, alignment
   and ownership mode. Scalars may be unboxed. Aggregates may use statically
   typed layouts or closed tagged/handle carriers, but **no host
   `Lowered::String`, `Vec`, constructor, record or closure object may cross
   implicitly.**
3. **Explicit closure conversion** — every free variable becomes a typed frame
   slot. `CaptureSlot` names a layout; `PredeclaredFunction` names its signature
   and frame layout. Configuration dependence may not remain hidden in the
   caller's `Vec<Lowered>`.
4. **Explicit ownership** — strings, bytes, constructors, records, closure
   environments and persistent-store references have stated lifetime, aliasing,
   transfer/borrow and reclamation rules. **An opaque pointer without this
   contract does not discharge the prerequisite.**
5. **Edge agreement** — every dynamic edge agrees on caller/callee layout, every
   recursive bundle member is forward-declared, every cross-owner value is
   representable. Failure is a planner error **before** emission — never a
   fallback to the old specializer after partial emission.

Prefer deriving per-origin layouts from authoritative static semantic/type
facts. Where a layout cannot be derived statically, represent the value through
an explicit **closed handle/tag carrier**; do not residualize the observed host
value into function identity.

## ★ The four mechanical constraints (Architect addendum `evt_t4fykh52ncb`)

These answer four facts the Steward grounded independently on the tree
(`evt_34pvqr0vte0pr`). They are the difference between item 3 being load-bearing
and being hygienic — **the frame must carry them as ACs with controls, not as
prose.**

### C1 — frame arity is ORIGIN-CONTRACT-determined, never caller-depth-determined

`lower_source_declaration_call` (`…/lowering/core.rs:4034-4050`) builds
`call_env = args ++ captures ++ env` — **the callee's environment is the
caller's entire environment appended wholesale.** So today the callee frame's
arity is a function of *caller depth*, not of the origin. That is not
configuration dependence merely *hidden* in a `Vec<Lowered>`; it is dependence
**structurally unbounded by the origin**, and it makes any per-origin frame
layout impossible until closure conversion removes it.

A `PredeclaredFunction` descriptor enumerates **exactly** its declared
parameters plus explicit free-variable slots. **No suffix of the caller
environment may cross implicitly.**

> **Control:** adding an **irrelevant binding** to the caller must not change
> the callee descriptor, its slot count, or its layout.

### C2 — BOTH capture provenances are closed inputs to layout construction

There are two, and they differ in kind. ⚠ A pin keyed to one of them is a
spelling standing in for a population — the exact defect that produced this
chain's two census misses:

| provenance | site | value source |
|---|---|---|
| lexical | `RuntimeExpr::LexicalClosure`, `…/core.rs:4772-4788` | `self.lower_expr(builder, capture, env)` — an **arbitrary source expression** |
| seed / declaration | `RuntimeExpr::Closure` `…/core.rs:4749`; declaration-closure `…/core.rs:6266` | `lower_seed_capture` (`lowering/mod.rs:5093`) → `lower_ground_value` (`:5107`) — a **JIT-time `RuntimeGroundValue`** |

The second provenance is the harder case and the one most likely to force the
closed carrier: its layout is a function of **a value seeded at JIT time, not of
syntax at all**. `lower_ground_value` returns `Lowered::String`, `Bytes`,
`Constructor { args }`, `Record { fields }` directly from the ground value.

⛔ **The seed provenance must not choose a layout by inspecting the particular
runtime value.** Its static capture contract must either determine a layout, or
select a **fixed closed tagged/handle carrier** capable of representing the
permitted ground-value family. No `Lowered` variant and no runtime value shape
may silently specialize the frame.

### C3 — pin the claim with a DISCRIMINATOR, not prose

The validator must **reject** missing capture slots, extra capture slots, and
any implicit caller-env tail. Two controls, both required:

> - caller depth changes ⇒ the per-origin descriptor is **identical**;
> - a seed ground value changes **within its declared carrier class** ⇒
>   descriptor shape unchanged.

⭐ **The transported payload may change; the ABI may not.** That sentence is the
property; everything else is evidence for it.

⚠ And a negative check passes for any reason. "The validator rejects an implicit
caller-env tail" needs a **positive control** that constructs one and observes
the rejection — otherwise it passes because nothing reached the checker.

### C4 — cross-module linking is a CHECKED exclusion, not a prose exclusion

`ImportedDeclarationRef` (`…/lowering/core.rs:4791`) is already
`unsupported("… requires dependency linking")`. So it may stay outside `B2R`,
and "the complete bundle" means **the complete intra-module callable bundle**.

But the exclusion has to be recorded **in the representation and in the
validator**: imported edges receive **no callable descriptor** and fail *before*
emission with the existing dependency-linking unsupported result. ⛔ No fallback
after partial emission, and the exclusion does not live in a comment. Pair it
with a **positive intra-module recursion/bundle control** — otherwise the
exclusion is indistinguishable from a gap.

## ⛔ Inert only — the already-ruled scaffold escape

- Declarative ABI/layout/ownership types, descriptor construction, and
  validators **may** be production code.
- Production retains **exactly** the existing one root `FunctionBuilder` and one
  `define_function`. **Zero** new callable target unit, call edge, dispatch
  edge, callback, flag, or alternate entry.
- Executable probes are **test-only**.
- **Both** cfg configurations pin the unchanged production unit census and zero
  executable edge into functionized emission.
- ⛔ **No encoder/decoder or helper that creates a second live body-emission
  authority lands here.** If executable representation transport is needed by a
  production call, it travels in `B2F`'s atomic live boundary.

⚠ This escape is the whole reason two prerequisites can land before an atomic
switch. Reading it loosely — "an encoder is just data movement" — collapses the
sequencing back into the unsatisfiable single unit.

## Anchors — ⚠ RE-DERIVE BEFORE THE FRAME IS WRITTEN

Recorded as of `52ded173` for orientation only; every anchor in this chain has
moved at least once.

| fact | location |
|---|---|
| emitted signature `(pointer) -> i64` | `crates/ken-runtime/src/cranelift_backend/lowering/core.rs:44-46` |
| one root builder / one definition | `…/lowering/core.rs:152`, `:188`, `:225` |
| `Lowered` specialization lattice | `…/cranelift_backend/lowering/mod.rs:415-507` |
| `CaptureSlot { ordinal: u32 }`, `PredeclaredFunction` | `…/planning/static_transition/semantic_ir.rs` |
| `AC-G0` native-int helper constant | `LOCAL_HELPER_COUNT` = **6** definitions / **8** declarations, Θ(1) per module |

⚠ **`crates/ken-backend-native` does not exist.** The research advisory
(`evt_531c4k52mshrn`) cites every path under that prefix; its **line numbers are
accurate** but the paths are not. Do not copy them into a frame. Erratum with
the corrected roots: **`evt_3k9xam3ws9pgz`**.

## Relationship to the rest of the chain

- **Closes no symptom-inventory entry.** Entry 2 is closed by `B2F`-proper.
  `RT-NATIVE-FNSPLIT` stays `active`.
- **Adversary P2 does not come here.** No container-spelling blacklist. That arm
  stays review-enforced until these closed ABI/body-owner structures admit an
  allowed-inventory structural pin **with a positive control** — and if they
  eventually do, that is a separate node, not a rider.
- **The growth verdict is unchanged:** total target units may be Θ(n) while each
  function is bounded by its own static body/transition contract. `AC-G0`'s
  native-int constant is accounted **separately** from the per-static-function
  population.

## ⛔⛔ RE-HOMED FROM `B2O` — 2026-07-25 (Architect `evt_5yxjd1zqnyvcq`)

**`B2O`'s `D6` source-route oracle was ruled OUT and its structural obligation
re-homed here and into `B2F`.** This section is the Steward's discharge of the
condition set on that split: *narrowing a claim MOVES its acceptance criteria;
it does not delete them.* If an obligation below cannot be traced to a
successor, the split dropped coverage.

**Ruled scope added to this node:**

> `B2R` derives **ABI/layout population from the function units and the owner
> cut** — not from any source-text census of Rust methods.

⇒ The population `B2R` lays out is the **validated `SemanticOwner` partition
`B2O` produces**, consumed as data. The authority for "what is a function unit"
is the occurrence's `StaticOriginId`, its validated `SemanticOwner`, and the
planned edge kind — **never** a Rust signature, name, visibility, or file.

### ★★ THE CONTROL INVERTS — this binds every pin in this node

> *"A Rust wrapper or nested function relocation must remain **GREEN** for
> semantic boundary classification, proving source topology is not authority."*

⛔ **A pin here that reddens because someone added a Rust wrapper, renamed a
method, changed a visibility, or moved a `fn` between files is measuring the
wrong thing and reporting success.** Structured controls mutate **graph/owner
axes**. `B2O` spent **four candidate SHAs** discovering this; do not re-derive
it. See `docs/program/wp/RT-FNSPLIT-B2O-body-ownership.md` `D6` for the full
ledger, including the finding that the claim being defended was never required
by any AC.

⛔ **Do NOT add `syn`, any new dependency, or a source-parsing oracle.**
