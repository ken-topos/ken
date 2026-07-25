---
id: RT-FNSPLIT-B2O
title: "static body ownership — a total, validated occurrence → PredeclaredFunction mapping in the semantic plane, inert"
status: active
owner: runtime
size: M
gate: none
depends_on: [RT-FNSPLIT-B2A-S]
blocks: [RT-FNSPLIT-B2R, RT-FNSPLIT-B2F]
github: null
origin: Architect ruling evt_842spc7t6js1 on RT-FNSPLIT-B2F hard-stop #9 (2026-07-25), item 5 plus the one-owner half of item 6, gated behind research advisory evt_531c4k52mshrn per the armed #9 pull. Steward-filed under the ruling's explicit grant of re-slicing and sequencing ownership; Steward owns the frame, scope, and AC/control placement.
---

> ## ✅ `active` — FRAME IS WRITTEN AND KICKED (Steward, 2026-07-25)
>
> The shovel-ready frame is **`docs/program/wp/RT-FNSPLIT-B2O-body-ownership.md`**
> (fetchable on `origin/main`). It is **fully ruled — no held deliverable**;
> `D1` was released by the Architect. Construction authority is live for the
> Runtime ring.
>
> This node exists because `RT-FNSPLIT-B2F` was ruled **not buildable as one
> unit**. It is the **first** of two inert prerequisites.
>
> ⛔ **Read the frame, not this file, for the ruled seed set.** Any prose
> anywhere — including earlier drafts of this document — that describes the unit
> heads as "the root plus the `ClosureBody` heads" is **WRONG**:
> `TransitionKind::ClosureBody` is a body's **return successor**, not its head.
> The ruled seeds are `plan.entries` ∪ every `EdgeKind::StaticBody` **target**.

> ## ⛔⛔ RESPUN 2026-07-25 — `D6`'s ROUTE ORACLE IS OUT. `status` STAYS `active`.
>
> Five review folds across four candidate SHAs were spent on one defect family: a
> hand-written source-text parser asked to model Rust's grammar. Architect ruling
> `evt_5yxjd1zqnyvcq` (durable at `architect/work` `8bff7b7a`) removes the oracle
> and names the authority — **the occurrence's `StaticOriginId`, its validated
> `SemanticOwner`, and the planned edge kind.**
>
> **The frame carries the route** (`RESPIN ROUTE`, plus `AC-9` … `AC-12`). Three
> things about it that are load-bearing and easy to get backwards:
>
> 1. ⛔ **It is NOT a revert.** The production **executable** bytes never moved
>    across any fold — measured, the only production delta from `97db6f0b` to
>    `96627f2a` is `///` comment text. The respin **subtracts** from the test and
>    report surface.
> 2. ★ **The control INVERTS.** A Rust wrapper or nested-`fn` relocation must now
>    stay **GREEN** — a pin that reddens on one is measuring implementation
>    topology and reporting success.
> 3. ⚠ **The removal boundary is DECLARATION vs REACHABILITY, not
>    source-text-vs-not.** Three sound pins read source text and must survive.
>
> ★ **No AC in this frame ever required route-set closure** — verified: `cannot
> grow silently` appears **0** times in the frame and **0** times in the report at
> the first QA-approved tree, appearing only in the fold made in answer to the
> finding. A claim that outruns its evidence has two repairs, and **narrowing was
> free at fold 2 and was one deleted sentence.** The structural obligation that
> genuinely survives is **one-for-one consumption**, re-homed by ID into
> `RT-FNSPLIT-B2R` and `RT-FNSPLIT-B2F`, which inert `B2O` could never check.
>
> **Counters unmoved — a review fold is not a hard-stop.** Count of record **9**,
> next armed research pull **#12**; symptom inventory `ENTRIES = 3`, next
> predicate check **6th**.

## Why this node exists

`RT-FNSPLIT-B2F` asks for **one closed callable unit per static origin**. The
implementer raised hard-stop #9 before writing any code: that cannot be built
from the current plane, because the plane has no answer to two prior questions —
*where are the unit boundaries* and *what may cross them*. This node answers the
first. `RT-FNSPLIT-B2R` answers the second.

**Ownership precedes representation, and the reason is not taste.** The
ownership mapping is what *defines the cut*. You cannot enumerate "every value
that crosses a generated-function boundary" — the thing `B2R` must contract —
before you know where the boundary is. Sequencing them the other way would have
`B2R` deriving a cross-cut value population from a boundary that is still
path-dependent.

## The defect this closes

The ruling struck the frame's site-keyed disposition taxonomy as **unsound**,
not merely incomplete:

> Disposition is per **occurrence ownership and reaching path**, not one row per
> source site. The five provenance classes are evidence inputs, not the
> authority partition.

That is the Architect confirming the Steward's own withdrawal of `AC-5`. The
original `AC-5` demanded a two-way "migrated / not-a-body-emission"
classification of the call-site population, which **presupposed that disposition
is a function of the site**. For the 14 caller-dependent sites it is a function
of the *reaching path*, because the same parameter carries both retained bodies
and ordinary sub-expressions. A table keyed to sites could have been filled in
completely and still been wrong — the taxonomy had **no cell for the honest
answer**.

The repair is not a better table. It is a **mapping in the semantic plane** that
the compiler itself can validate, against which the 59-call population is
dispositioned as a *derived report* rather than a hand-authored claim.

## Ruled scope (Architect item 5 + the one-owner half of item 6)

- A **total** mapping from every source occurrence to its owning
  `PredeclaredFunction`. Total means every reachable occurrence, and **exactly
  one** owner each.
- The **synthesized root occurrence owns the root unit.** This is the occurrence
  behind `core.rs:188` (`compiler.lower_expr`, taking `root_static_origin`) —
  the 59th call, the one the `self.`-spelled census missed, and the one that
  cannot be classified as traversal because it *seeds* the descent.
- **Intra-owner syntax traversal stays local; crossing to another owner is a
  static call.** This is the invariant that gives `D6` its boundary: not "remove
  recursion", but "no caller recursively emits another owner's body".
- **Validation:** every occurrence has exactly one owner; the mapping is checked
  after every transforming pass; failure is a planner error **before** emission,
  never a fallback to the old specializer after partial emission.

### ⛔ Inert only — the already-ruled scaffold escape

- Declarative owner tables, their construction, and their validators **may** be
  production code.
- Production retains **exactly** the existing one root `FunctionBuilder` and one
  `define_function`. **Zero** new callable target unit, call edge, dispatch
  edge, callback, flag, or alternate entry.
- Executable probes are **test-only**.
- **Both** cfg configurations pin the unchanged production unit census and zero
  executable edge into functionized emission.
- ⛔ No encoder/decoder or helper that creates a **second live body-emission
  authority** lands here. If executable transport is needed by a production
  call, it travels in `B2F`'s atomic live boundary.

## Anchors — ⚠ RE-DERIVE BEFORE THE FRAME IS WRITTEN

Every anchor in this chain has moved at least once (`lower_expr` alone went
`:3847 → :4255 → :4333`). The frame must re-measure on its own base. Recorded
here as of `52ded173` for orientation only:

| fact | location |
|---|---|
| `lower_expr` definition | `crates/ken-runtime/src/cranelift_backend/lowering/core.rs:4333` |
| synthesized root call | `…/lowering/core.rs:188` |
| tokenized production call population | **59** (1 definition + 59 calls = 60 whole-token occurrences, less 5 in comments) |
| semantic carrier | `…/cranelift_backend/planning/static_transition/semantic_ir.rs` |
| cross-owner re-emission exhibit | `…/lowering/core.rs:4034-4050` |

⚠ **`crates/ken-backend-native` does not exist.** The research advisory
(`evt_531c4k52mshrn`) cites every path under that prefix. Its **line numbers are
accurate** — the derivation was done against the real tree — but the paths are
not. Do not copy them into a frame. Research confirmed the defect against the
held object `3891b7aa` and issued erratum **`evt_3k9xam3ws9pgz`** with the
corrected roots.

## The cleanest exhibit of what `D6` removes

`lower_source_declaration_call` (`…/lowering/core.rs:4034-4050`), non-recursive
branch: it emits **no call**. It builds `call_env = args ++ captures ++ env` and
continues the source machine with `expr: body` in that environment. That is
cross-owner re-emission in four lines, and it is a better `D6` exhibit than any
call-site census — the census counts *opportunities*, this shows the *authority*.

## Relationship to the rest of the chain

- **Closes no symptom-inventory entry.** The inventory is append-only; entry 2
  is closed by `B2F`-proper and by nothing before it. `RT-NATIVE-FNSPLIT` stays
  `active`.
- **`RT-FNSPLIT-B2B`** reports the *measured* census; this node and `B2F` supply
  the *structural* invariant it reports against.
- **Adversary P2 (entry-keyed-store residual) does not come here.** The ruling
  directs: no container-spelling blacklist; leave that arm review-enforced until
  the new closed ABI/body-owner structures admit an allowed-inventory structural
  pin **with a positive control**.

## Standing hazards for whoever builds this

- **Totality is not closure under parent→child reachability.** Hard-stop #8 was
  predictable from a totality question that the mechanism did not need:
  `ComputationalMatch` filed its occurrence on a different node from the entry
  its parent pointed at, so `TOTAL` was true and composition still failed. A
  one-owner-per-occurrence check is a *totality* check. The frame must **also**
  pin that the mapping composes: an owner boundary crossed on the way *down*
  must be the same boundary seen on the way *back up*.
- **A structural pin that enumerates spellings is not a proof of the property.**
  Two censuses in this chain were keyed to a spelling standing in for a
  population — `self.lower_expr(` for the call population, and
  `RuntimeExpr::Closure` for the capture population (the second arm,
  `LexicalClosure`, behaves differently). Pin the property, and attempt a
  compile-preserving evasion of each pin.
- **A negative check passes for any reason.** "Zero new call edges" needs a
  positive control that *would* fire.
