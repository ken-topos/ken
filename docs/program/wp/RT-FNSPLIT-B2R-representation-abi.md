# `RT-FNSPLIT-B2R` — representation and call-ABI contract

> **Shovel-ready frame.** Authored by the Steward under the Architect's grant of
> re-slicing and sequencing ownership (ruling `evt_842spc7t6js1`, addendum
> `evt_t4fykh52ncb`). The ruled scope, the four mechanical constraints, and the
> inert-only scaffold live in `docs/program/issues/RT-FNSPLIT-B2R.md` and are
> **binding**; this document adds the re-derived anchors, the deliverables, and
> the acceptance criteria with their controls.
>
> **Base:** `origin/main` = `e470ab65` (`RT-FNSPLIT-B2O` merged, PR #963).
> **Branch:** `wp/RT-FNSPLIT-B2R-representation-abi`.

## Objective

Establish a **stable, executable representation contract for every value that
crosses a generated-function boundary** — declared, validated, and **inert**. No
new callable unit and no new emission edge land here; `RT-FNSPLIT-B2F` performs
the atomic switch-over.

⭐ **This is NOT "build one universal boxed `Value` first."** The contract may be
uniform *or* a family of statically typed per-origin layouts. Boxing is one
admissible implementation of the contract, never the contract itself.

## The population is `B2O`'s owner partition, consumed as data

`RT-FNSPLIT-B2O` landed the authority this node builds on. **The set of function
units is the validated `SemanticOwner` partition**, and the authority for "what
is a function unit" is the occurrence's `StaticOriginId`, its validated
`SemanticOwner`, and the planned edge kind.

⛔ **Never a Rust signature, name, visibility, or file.** Do not add `syn`, any
new dependency, or a source-parsing oracle. `B2O` spent **four candidate SHAs**
establishing this; the ledger is in
`docs/program/wp/RT-FNSPLIT-B2O-body-ownership.md` under `D6`.

### ★★ THE CONTROL INVERTS — this binds every pin in this node

> A Rust wrapper, a rename, a visibility change, or a `fn` moved between files
> **must leave semantic boundary classification GREEN.**

⛔ **A pin that reddens on any of those is measuring source topology and
reporting success.** Structural controls mutate **graph and owner axes**, never
source spelling.

## Anchors — re-derived at `e470ab65`, 2026-07-25

⚠ The issue file's anchor table was recorded at `52ded173` and **every line
number in it has moved**; two entries also named the wrong file. This table
supersedes it. Re-derive again if `main` moves before you start.

| fact | location at `e470ab65` |
|---|---|
| emitted signature `(pointer) -> i64` | `…/lowering/core.rs:47-50` |
| the one root `FunctionBuilder` | `…/lowering/core.rs:152` |
| the one `define_function` | `…/lowering/core.rs:225` |
| `root_static_origin` seed | `…/lowering/core.rs:46`, used `:192` |
| `Lowered` specialization lattice | `…/lowering/mod.rs:415-507` |
| `PredeclaredFunctionId` | `…/planning/static_transition/semantic_ir.rs:38` |
| **`SemanticOwner`** (new, from `B2O`) | `…/planning/static_transition/semantic_ir.rs:62` |
| `CaptureSlot` | `…/planning/static_transition/semantic_ir.rs:438` |
| `PredeclaredFunction` | `…/planning/static_transition/semantic_ir.rs:498` |
| `SemanticDescriptor` / its `owner` field | `…/planning/static_transition/semantic_ir.rs:508`, `:520` |
| `shared_exits` | `…/planning/static_transition/semantic_ir.rs:548` |
| `build_semantic_plane` | `…/planning/static_transition/semantic_ir.rs:735` |
| `lower_source_declaration_call` (the `C1` site) | `…/lowering/core.rs:4034`, env append `:4047-4048` |
| `lower_recursor_residual_call` (a **second** `C1` site) | `…/lowering/core.rs:270`, env append `:323-324` |
| lexical capture provenance | `…/lowering/mod.rs:2306` (`RuntimeExpr::LexicalClosure`) |
| seed capture provenance | `…/lowering/mod.rs:5093` (`lower_seed_capture`) → `:5107` (`lower_ground_value`) |
| `ImportedDeclarationRef` unsupported | `…/lowering/core.rs:4793-4798` |
| `LOCAL_HELPER_COUNT` = 6 | `…/artifact/tests.rs:56` |

## ⛔⛔ `C1` CANNOT BE PINNED BY AUDITING CALL SITES — measured, 2026-07-25

The issue file states `C1` against **one** site,
`lower_source_declaration_call`. **That framing does not survive measurement.**
The caller-environment append pattern occurs at **44 sites** in
`lowering/core.rs` + `lowering/mod.rs`, across **two spellings**:

| spelling | sites |
|---|---|
| `.extend_from_slice(captures\|env\|producer_env\|saved_producer_env)` | **29** |
| `.extend(captures\|env\|producer_env)` | **15** |

★ **And the site `C1` actually names is in the SECOND spelling** — `:4047-4048`
uses `.extend(...)`, so a sweep written against `extend_from_slice` **excludes
the very site the constraint cites.** That is not a hypothetical; it is what
happened while these anchors were being re-derived.

⇒ **`C1`'s control is a post-condition on the DESCRIPTOR, never a census of
append sites.** A site census is a spelling standing in for a population — the
defect family that cost `B2O` four SHAs and `B2A` two census misses. The
descriptor post-condition is **mechanism-independent**: it holds whether the
env is appended, cloned, threaded, or restructured, and it keeps holding when
someone adds a 45th site.

## Deliverables

**D1 — the layout language.** Declarative types for a common activation-frame
header plus a statically declared payload layout per `PredeclaredFunction`. One
closed layout language and one common control/store/result/trap convention.
"Fixed frame" does **not** require equal byte size across origins.

**D2 — descriptor construction from the owner partition.** For each
`PredeclaredFunction` in `B2O`'s validated partition, a descriptor enumerating
**exactly** its declared parameters plus explicit free-variable slots. Every
parameter, capture, result, control value, trap value and persistent-store handle
carries a declared kind, width, alignment and ownership mode. Scalars may be
unboxed; aggregates use statically typed layouts or **closed** tagged/handle
carriers.

**D3 — explicit closure conversion, over BOTH provenances.** Every free variable
becomes a typed frame slot. `CaptureSlot` names a layout; `PredeclaredFunction`
names its signature and frame layout. The two provenances differ in kind and
both are closed inputs:

| provenance | site | value source |
|---|---|---|
| lexical | `…/lowering/mod.rs:2306` | an **arbitrary source expression** |
| seed / declaration | `…/lowering/mod.rs:5093` → `:5107` | a **JIT-time `RuntimeGroundValue`** |

⛔ **The seed provenance must not choose a layout by inspecting the particular
runtime value.** Its static capture contract either determines a layout or
selects a fixed closed carrier able to represent the permitted ground-value
family. No `Lowered` variant and no runtime value shape may silently specialize
a frame.

**D4 — ownership.** Strings, bytes, constructors, records, closure environments
and persistent-store references get stated lifetime, aliasing, transfer/borrow
and reclamation rules. ⛔ **An opaque pointer without this contract does not
discharge the prerequisite.**

**D5 — the validator.** Rejects missing capture slots, extra capture slots, and
any implicit caller-environment tail. Every dynamic edge agrees on caller/callee
layout; every recursive bundle member is forward-declared; every cross-owner
value is representable. Failure is a **planner error before emission** — never a
fallback to the old specializer after partial emission.

**D6 — the evidence report**, `docs/program/rt-fnsplit-b2r-abi-report.md`:
predictions written **before** measurement, the pin classification of `AC-9`, and
every mutation outcome including the invalid ones.

## Acceptance criteria

**AC-1 — descriptor totality over the owner partition.** Every
`PredeclaredFunction` in the validated partition has **exactly one** descriptor;
every descriptor names a member of that partition. Both directions — a
one-directional check passes happily on an orphan.

**AC-2 — `C1`, as a descriptor post-condition.** Adding an **irrelevant
binding** to the caller must not change the callee descriptor, its slot count, or
its layout. ⛔ **Do not discharge this by auditing the 44 append sites** (see the
measurement above). Pin it as a property of the constructed descriptor.

**AC-3 — `C2`, both provenances.** A capture arriving by the lexical route and
one arriving by the seed route both produce a declared slot with a declared
layout. **Positive control required:** a seed capture whose ground value is a
`Constructor`/`Record`/`String` must still yield a **fixed** carrier, and the
test must observe that the descriptor does not vary with the value.

**AC-4 — `C3`, the discriminator.** *The transported payload may change; the ABI
may not.* Two controls, both required:

- caller depth changes ⇒ the per-origin descriptor is **identical**;
- a seed ground value changes **within its declared carrier class** ⇒ descriptor
  shape unchanged.

⚠ **A negative check passes for any reason.** "The validator rejects an implicit
caller-env tail" needs a **positive control that constructs one** and observes
the rejection — otherwise it passes because nothing ever reached the checker.

**AC-5 — `C4`, cross-module exclusion is CHECKED.** Imported edges receive **no**
callable descriptor and fail *before* emission with the existing
dependency-linking unsupported result (`…/lowering/core.rs:4793`). The exclusion
lives in the representation and the validator, **not in a comment.** Pair it with
a **positive intra-module recursion/bundle control** — otherwise the exclusion is
indistinguishable from a gap.

**AC-6 — inert.** Production retains **exactly** one root `FunctionBuilder`
(`core.rs:152`) and one `define_function` (`core.rs:225`). **Zero** new callable
target unit, call edge, dispatch edge, callback, flag, or alternate entry.
Executable probes are test-only. **Both** cfg configurations pin the unchanged
production unit census and zero executable edge into functionized emission.
⛔ No encoder/decoder or helper that creates a second live body-emission
authority lands here.

**AC-7 — no oracle, no dependency.** No `syn`, no new dependency, no
source-parsing oracle. `Cargo.toml`/`Cargo.lock` byte-identical; no
`syn`/`proc-macro2`/`quote` edge added to any `ken-*` crate.

**AC-8 — the control inverts.** A Rust wrapper, rename, visibility change, or
`fn` relocation leaves boundary classification **GREEN**. A pin that reddens on
one of those is a **defect in the pin**, and it is reported as such.

**AC-9 — classify every source-text-reading pin in the touched files, by an
enumeration you close over the files rather than by a list.** Each row carries a
real outcome:

| outcome | meaning |
|---|---|
| **reddens** | discharged |
| ⭐ **cannot compile** | the compiler enforces it — the strongest outcome, and usually the cheapest |
| ⚠ **stays green** | the pin is spelling-keyed: **a FINDING to report, explicitly NOT to fix** |

⛔ **`not attempted` is not an outcome.** ⭐ And **ask which mechanism already
enforces a property before building a detector for it** — in `B2O` three of eight
rows were discharged by compiler refusal, which was both the strongest evidence
and the cheapest to obtain.

⚠ **A verdict from a mutation that did not apply is not evidence, and it looks
exactly like evidence.** Report invalid mutations rather than discarding them: a
mutation that never applied, that broke the build, that inserted a comment with
nothing to catch, or that **edited the detector along with its subject** is not
an evasion. `B2O` produced all four kinds.

**AC-10 — predictions before measurement.** Write predicted values down first,
then measure. A count re-fit to what you observe measures nothing.

## Risks

- **`L` size, and the layout language is the hard judgment.** Prefer deriving
  per-origin layouts from authoritative static semantic/type facts; where a
  layout cannot be derived statically, use an explicit **closed** handle/tag
  carrier. ⛔ Do not residualize an observed host value into function identity —
  that conflicts with one-unit-per-`PredeclaredFunction` and with total Θ(n)
  unless the specialization key's cardinality is separately proved bounded, which
  nothing here does.
- **The inert boundary is the whole reason two prerequisites can precede an
  atomic switch.** Reading it loosely — *"an encoder is just data movement"* —
  collapses the sequencing back into the unsatisfiable single unit.
- **Growth verdict unchanged:** total target units may be Θ(n) while each
  function is bounded by its own static body/transition contract.
  `LOCAL_HELPER_COUNT` is accounted **separately** from the per-static-function
  population.

## Relationship to the chain

- Closes **no** symptom-inventory entry. Entry 2 is closed by `B2F`-proper.
  `RT-NATIVE-FNSPLIT` stays `active`.
- **Adversary P2 does not come here.** No container-spelling blacklist; that arm
  stays review-enforced until these closed ABI/body-owner structures admit an
  allowed-inventory structural pin **with a positive control** — and if they
  eventually do, that is a separate node, not a rider.
- `RT-FNSPLIT-B2F` depends on this node plus `B2O`.
