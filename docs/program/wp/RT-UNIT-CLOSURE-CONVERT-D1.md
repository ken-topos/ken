# `RT-UNIT-CLOSURE-CONVERT` `D1` — inventory, and a hard stop

Inventory only. **No production edit.** Base: exact **`b3ba2820`** on
`wp/RT-DECL-CLOSURE-PORT-typed-units` (no rebase, merge, cherry-pick or new
branch). Released by `evt_1b5xqfd8j775k`.

All probes below were temporary and are reverted; the tree at this base is
unchanged by this checkpoint.

## The answer, first

**`CaptureSlot { ordinal: u32 }` is CONFIRMED**, and the merged substrate carries
**only counts and ordinals — never free-variable identities.**

```rust
// semantic_ir.rs:736
pub(super) struct CaptureSlot { pub(super) ordinal: u32 }
```

The derivation is the whole story:

```rust
// semantic_ir.rs:444 — the source side is a COUNT
capture_slots: match expr {
    RuntimeExpr::Closure       { captures, .. } => checked_len(captures.len())?,
    RuntimeExpr::LexicalClosure{ captures, .. } => checked_len(captures.len())?,
    _ => 0,
}
// semantic_ir.rs:1262 — the slot run is synthesized FROM that count
plane.capture_slots
    .extend((0..source.capture_slots).map(|ordinal| CaptureSlot { ordinal }));
```

⛔ `captures: Vec<RuntimeExpr>` (`ir.rs:443`) is the only place a free variable's
identity exists. It is consumed **for its length** and discarded at the
semantic-plane boundary. Nothing downstream — `CaptureLayout`, `AbiSlot`,
`AbiFrameHeader` — has a field that could name a captured value. `AbiSlot` has an
`ordinal`, a carrier, a width and an ownership mode, and no identity.

## The five `RT-FNSPLIT-B2R` mechanism elements

Measured at this base, not read from the frame table.

**D1 — layout language. PRESENT.** `AbiFrameHeader` `abi.rs:438`; `AbiSlot`
`:413`; `AbiSlotKind` `:243`, which has a `Capture` arm.

**D2 — descriptor construction. PRESENT.** A per-unit descriptor of header plus
slot run, each slot carrying kind, carrier, ownership, storage owner, width,
align and ordinal.

**D3 — closure conversion over both provenances. PRESENT AS CLASSIFICATION,
populated only from the IR's own capture list.** `AbiCaptureProvenance::{Lexical,
Seed}` at `abi.rs:261`; capture slots built at `abi.rs:1587` and `:1614`.

**D4 — ownership. PRESENT.** `AbiOwnership` `abi.rs:225` and `AbiStorageOwner`
`:209`, recorded **per slot** rather than per unit.

**D5 — the validator. PRESENT, with real laws.** `validate_slot_run`
`abi.rs:2508` refuses *"abi frame carries an implicit caller-environment tail"*;
capture-count agreement at `abi.rs:1511`, `:2020`, `:2368`.

⭐ **No element is a stub.** Each is reached on the ordinary planning path, and
the capture path is genuinely populated: across the lib corpus the closure
population is **90** `LexicalClosure` with one capture, **30** with two, **7**
`Closure` with one — alongside **365** and **49** with none.

⛔ So the honest reading is **not** "the mechanism is missing". It is that all
five elements are driven off one input — `captures.len()` — and for one
population that input is empty while the body still needs a binding.

⚠ **`D1b` CORRECTS THE FRAMING BELOW.** This section reads the failure as "a
free variable with no declared capture slot", which assumes the `Var` is valid
and the captures are wrong. That assumption was not measured. `D1b` measured it:
the failing units' lowering environment is **not** the frame's parameter/capture
prefix — it is `[StaticWorker, carried, ...]`, and the `StaticWorker` occupies de
Bruijn index 0. The hard stop stands, on a better-grounded mechanism; see
`RT-UNIT-CLOSURE-CONVERT-D1b.md` before acting on anything below.

## ⛔⛔ THE HARD STOP

Both `Var` failure sites (`lowering/core.rs:3661`, `:9638`) were instrumented at
this base and run across the whole lib corpus. Every failure:

```
index=2  env_len=2  unit=PredeclaredFunctionId(3)
  header     = { parameters: 1, captures: 0, frame_bytes: 40, align_bytes: 8 }
  definition = ClosureBody { defining_origin: 88, provenance: Lexical }

index=3  env_len=3  unit=PredeclaredFunctionId(1)
  header     = { parameters: 1, captures: 0, frame_bytes: 40, align_bytes: 8 }
  definition = ClosureBody { defining_origin: 14, provenance: Lexical }
```

**The failing units are lexical closure bodies that declare `captures: 0` while
their bodies reference a free variable.** Their defining occurrences declare
`captures=0, params=1`.

⛔ **This is ABSENT, not inert.** There is no declared-but-unbound slot waiting
for a value. The descriptor states the frame has **zero** captures; the semantic
plane's capture run for that layout is **empty**; and the IR's `captures` list —
the only carrier of identity — is empty too. There is nothing anywhere in the
merged substrate that names the value `Var(2)` needs.

⛔ **Binding it would require fabricating a capture** — inventing a slot the
descriptor does not declare, for a value no plane records. That is one of the
four explicitly banned repairs, as are the alternatives: padding the frame,
shifting the `Var` indices, or copying the caller's environment tail. The
release directs a hard stop rather than a weakened proof, and this is that stop.

### The concrete missing route, named

**Nothing derives the free-variable set of a closure body.** The substrate
faithfully carries whatever the IR's `captures` list already holds and validates
it thoroughly — but no pass computes that list from the body, and no type could
hold the answer if one did, because `CaptureSlot` has only an ordinal.

⇒ The missing element is **upstream of all five**, and it needs two things that
do not exist at this base:

1. a free-variable analysis over a closure body, producing identities; and
2. a place to put them — `CaptureSlot` must gain an identity field, or an
   identity-bearing sibling table must exist, before D3 can bind a slot to a
   value rather than to a position.

⚠ **What this does NOT establish.** I have not shown *why* those closures reach
the planner with an empty capture list while 127 others do not. That is a
question about the IR's construction, one plane above this node, and answering it
is not D1's mandate. It is the first thing D2 sizing needs.

## Moved inputs, re-derived at `b3ba2820`

⚠ The `B2R` anchor table was re-derived at `e470ab65` and **every entry below has
moved again**. It says to re-derive if `main` moves; it has.

| input | `B2R` table (`e470ab65`) | **this base (`b3ba2820`)** |
|---|---|---|
| `CaptureSlot` | `semantic_ir.rs:438` | **`semantic_ir.rs:736`** |
| `PredeclaredFunction` | `semantic_ir.rs:498` | **`semantic_ir.rs:796`** |
| `SemanticDescriptor` | `semantic_ir.rs:508` | **`semantic_ir.rs:806`** |
| `PredeclaredFunctionId` | `semantic_ir.rs:38` | **`semantic_ir.rs:242`** |
| `SemanticOwner` | `semantic_ir.rs:62` | **`semantic_ir.rs:266`** |
| `shared_exits` | `semantic_ir.rs:548` | **`semantic_ir.rs:851`** |
| `build_semantic_plane` | `semantic_ir.rs:735` | **`semantic_ir.rs:1183`** |

⚠ `semantic_ir.rs:438` — the old `CaptureSlot` anchor — now lands **inside the
`capture_slots` derivation match** (`:444` is the arm itself). A reader following
the stale table lands six lines from the count that this checkpoint is about, and
would plausibly believe they were looking at the right thing.

Not in the `B2R` table, recorded because this checkpoint depends on them:

| input | this base |
|---|---|
| `CaptureLayout` | `semantic_ir.rs:238` |
| the `capture_slots` derivation | `semantic_ir.rs:444` |
| the slot-run synthesis | `semantic_ir.rs:1262` |
| `AbiFrameHeader` / `AbiSlot` / `AbiSlotKind` | `abi.rs:438` / `:413` / `:243` |
| `AbiCaptureProvenance` | `abi.rs:261` |
| `AbiOwnership` / `AbiStorageOwner` | `abi.rs:225` / `:209` |
| `validate_slot_run` | `abi.rs:2508` |
| the two `Var` resolution sites | `lowering/core.rs:3661`, `:9638` |
| `RuntimeExpr::LexicalClosure` | `ir.rs:442`, `captures` at `:443` |

## Suite

Unchanged at this base: `ken-runtime` lib **730 passed / 7 failed / 1 ignored**,
both `check -p ken-runtime` and `check --profile test` clean. Five of the seven
reds are this node's subject.

⛔ **No repair was attempted and none is authorized.** Next mover is the Steward,
for sizing before `D2`.
