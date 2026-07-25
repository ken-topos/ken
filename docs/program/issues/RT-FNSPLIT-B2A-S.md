---
id: RT-FNSPLIT-B2A-S
title: "defunctionalize retained body selection — static-origin tag plus one closed consumer, replacing cloned-RuntimeExpr identity"
status: active
owner: runtime
size: M
gate: none
depends_on: [RT-FNSPLIT-B1R]
blocks: [RT-FNSPLIT-B2F]
github: null
origin: Architect ruling evt_6h5gw5c503n5z on RT-FNSPLIT-B2A hard-stop #6 (2026-07-25), gated behind research advisory evt_4w1rf45d4fkv3. Replaces the retired RT-FNSPLIT-B2A frame, whose Retain/Replace lists were inherited from the never-landed b077eb7a. Steward-filed; Steward owns the replacement frame and the full AC/control re-walk.
---

> ## ✅ ACTIVE — kicked to the Runtime ring 2026-07-25 (`evt_3wmhpdq7dm8d8`)
>
> Ring compaction verified on the **completion marker** for all three seats
> (implementer reached `ctx 0%`) before the mention went out. Fresh branch from
> `origin/main` = `70bd2c74`.
>
> ## ✅ FRAMED — 2026-07-25
>
> **The shovel-ready frame is
> `docs/program/wp/RT-FNSPLIT-B2A-S-selection-defunctionalization.md`.**
> Read that, not this file. It carries the Architect's seven ruled requirements as
> D1–D7, the ten ACs (including the two carried from the retired frame), the
> discriminating controls, and the anchors pinned on `origin/main` = `70bd2c74`.
>
> ⛔ **The single requirement most likely to be softened is D4:** cloned
> `RuntimeExpr` bodies must leave the covered population **in the same diff**.
> Replacing identity while keeping the old body carrier preserves two authorities
> and fails the slice. If D4 cannot be completed for any member, that is a
> **hard-stop, not a partial landing.**

## Why this node exists — the old B2a frame is RETIRED, not amended

`RT-FNSPLIT-B2A`'s `Retain`/`Replace` lists were **inherited from the held
`b077eb7a` branch, which is not an ancestor of `main`**. On the real base
(`7151ae58`) there is **one** production Cranelift function, **no**
`PartitionWorkItem`, and **no emitted-unit population to re-key** — so its D1/D2
described a construction while calling it a behaviour-preserving port. That is a
Steward framing defect. Confirmed independently by the ring, by research, and by
the Architect.

★ **The work splits into two nodes at a boundary neither the old frame nor the
ring's own three-slice proposal had right:**

- **`RT-FNSPLIT-B2A-S` (this node)** — close retained-body **selection identity**.
- **`RT-FNSPLIT-B2F`** — the atomic functionization + authority switch.

⛔ **The ring's proposed middle slice (functionize, defer removal) is ruled out.**
It would land a live second production path beside the old authority — exactly
what "carrier and removal land together" exists to prevent, one level up.

## ✅ THE ROUTED DESIGN QUESTION IS ANSWERED — YES, with a precise boundary

**A plan-retained `StaticOriginId → &RuntimeExpr` occurrence reference IS
compatible with *"never from pointer, content, clone order, or activation"* — but
only as a PAYLOAD SELECTED BY THE ORDINAL, never as identity.**

The identity is the preallocated `StaticOriginId`; the borrowed reference is
what that identity *selects*. ⛔ **No pointer equality, address hash, content
hash, traversal/clone order, or activation value may create or recover the ID.**

⚠ **This is a deliberately narrow intermediate representation. Be precise about
what it is NOT** — every item here is a claim the frame must refuse to make:

| it DOES | it does NOT |
|---|---|
| close retained-body **selection identity** | make B1R's semantic plane the emission source of truth |
| retire symptom-inventory **entry 1** (cloned-body identity) | create target functions |
| — | remove whole-configuration specialization (**entry 2 stays explicitly OPEN**) |
| — | constitute O(n) growth evidence |

★ **Entry 2 is the same already-named predicate, not a new defect.** Leaving it
open here is deliberate and must be stated, not glossed.

## The seven ruled requirements — a candidate is admissible only if ALL hold

Transcribed from `evt_6h5gw5c503n5z`. These are **settled inputs**; if one is
false against the landed code, **say so and escalate — do not build around it.**

1. **Allocate the occurrence reference in the same planner source walk, at the
   same moment, as its already-preallocated `StaticOriginId`.** Store an exact
   **dense, compile-local table** keyed by that ID. The table must be **total and
   one-to-one** with the planned occurrence population; lookup is
   **range-checked** and **verifies the entry's stored origin matches the index.**
2. **Keep the table on a compile-local `StaticTransitionPlan<'src>`** (or an
   equivalent private companion owned by that plan). References **may** borrow the
   root/declaration source trees, because the plan is consumed entirely inside
   `compile_expr_into_module` and **cannot escape into `CompiledModule` or runtime
   state.** ⚠ That non-escape property is the whole licence for the lifetime —
   make it checkable.
3. **Widen `StaticOriginId` only to `pub(in crate::cranelift_backend)`**, and
   spell the carrier field **`static_origin`** — as already ruled. ⛔ Never bare
   `origin` beside `RecursorProducerOriginId`.
4. **⭐ IN THE SAME DIFF, remove raw/cloned `RuntimeExpr` bodies from every
   retained closure/work-item representation the slice covers.** In particular
   **`Lowered::Closure { body: RuntimeExpr, … }` cannot survive for that
   population beside the new tag.** ⛔ **Replacing identity while retaining the old
   body carrier preserves two authorities and FAILS the slice.** This is the
   requirement most likely to be softened under schedule pressure; it is the one
   that must not be.
5. **Install ONE closed consumer** — conceptually
   `lower_static_origin(static_origin, dynamic_env)`. It range-checks the dense
   table, selects the borrowed occurrence **solely by the tag**, then invokes the
   existing recursive lowering as the **temporary payload semantics**. **Every**
   retained closure application/resume in scope routes through it; **none**
   selects a body directly. ⚠ Research's recognition criterion: the consumer is
   the **sole point of consumption** — that is what makes this a complete
   defunctionalization rather than a tag with a leak.
6. **Move D0's carrier controls here, and add the discriminators that matter at
   this boundary:**
   - swap two equal-shaped origin entries → reject/wrong-body control **reddens**;
   - duplicate / missing / out-of-range origin → **loud planner failure**;
   - perturb the borrowed **address** without changing the ordinal mapping →
     **no identity change**;
   - each mutation restored **byte-identically** (`git diff --quiet`).
7. **⛔ Re-enumerate the carrier construction/pattern sites against the NEW
   consumer population.** The inherited **29/28** census (`core.rs` 13 E0063 + 16
   E0027; `mod.rs` 14 + 14) is **evidence, not authority** — the consumer boundary
   has changed. ⚠ It was also measured on a compiler probe against `5015bc71`;
   `lowering/` is byte-identical across B1R, so it carries forward as a *starting
   estimate* only.

## Naming — the honesty requirement is part of the ruling

⛔ **Do not call this a semantic-plane emission port.** The milestone is
**"defunctionalize retained body selection."** The ring itself accepted this
correction (`evt_2t9jdvmrtmhmf`): an `origin → &RuntimeExpr` table closes
selection identity while leaving `RuntimeExpr` owning semantics — an *abstract
machine over source terms*, not a *virtual machine over compiled units*. Bundling
those two properties under one name is the same overclaim shape that cost B1R a
round trip.

## Steward's own re-walk obligations before this flips to `ready`

- ⚠ **Re-walk EVERY AC and control from the retired B2a frame.** Several were
  defined on deliverables that have moved to `RT-FNSPLIT-B2F`; scoping a
  deliverable out strands the ACs defined on it. Specifically: old `AC-1` (D4
  differential suite), `AC-2` (old-path removal), `AC-3` (the four D3 width
  invariants), and `AC-8` (no growth claim) all belong to **B2F**, not here —
  ⛔ do not copy them across without deciding each one.
- Old `AC-4` (`fixed_k` still `8,8,8,8,8` vs cap `8`) and `AC-5` (no new opcode,
  no wildcard arm) **do** still apply here and should be carried.
- Re-derive the contention check against `origin/main` at release, on **both**
  the file axis and the **ledger** axis. ⚠ The ledger attests
  `crates/ken-runtime/src/cranelift_backend.rs`; if this slice edits that file
  itself, the consumer population must be re-derived before landing.
