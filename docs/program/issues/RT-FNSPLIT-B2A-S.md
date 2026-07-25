---
id: RT-FNSPLIT-B2A-S
title: "defunctionalize retained body selection — static-origin tag plus one closed consumer, replacing cloned-RuntimeExpr identity"
status: active
owner: runtime
size: M
gate: none
depends_on: [RT-FNSPLIT-B2A-C]
blocks: [RT-FNSPLIT-B2F]
github: null
origin: Architect ruling evt_6h5gw5c503n5z on RT-FNSPLIT-B2A hard-stop #6 (2026-07-25), gated behind research advisory evt_4w1rf45d4fkv3. Replaces the retired RT-FNSPLIT-B2A frame, whose Retain/Replace lists were inherited from the never-landed b077eb7a. Steward-filed; Steward owns the replacement frame and the full AC/control re-walk.
---

> ## ⛔ SCOPE RULED (a) — 2026-07-25, `evt_2eap269sgnavm`
>
> **Covered population = `Lowered::Closure` + `Lowered::DeclarationClosure`.**
> ⛔ NOT the whole `OwnedSourceOccurrence` struct (17 sites, incl. source-machine
> frames). `ComputationalRecursorClosure` is out **structurally** — it carries no
> `OwnedSourceOccurrence`. Source-machine frames are entry-2 / `B2F` territory.
> ★ **D1's struct-level phrasing was a Steward defect and contradicted D5/AC-6**,
> which delegates the population decision; D5 was the correct half. Frame text
> corrected.
>
> ⚠ **AND A PROCESS DEFECT: the rewritten frame was never on `main`** — only on
> `steward/work` (`350f7b2d`), while the ring's base `4c5afda6` held the
> pre-rewrite draft **reusing the same identifiers for different deliverables.**
> ⇒ ★ **A kickoff must verify the frame is FETCHABLE AT THE RING'S BASE, not
> merely written.** "Written" and "readable by the ring" are different facts.
>
> ## ✅ ACTIVE — kicked to the Runtime ring 2026-07-25 (`evt_5jzpy3dgs8s67`)
>
> Full §2c gate ran: retros in 3/3, quiescent, **both** contention axes checked
> (no build WP contends; `px8ta` — which D8 touches — is not in the attestation
> ledger), and **each drop verified on the `Context compacted` marker.**
> ⚠ **Held ~70 minutes first:** Codex remote-compact returned 503
> (`circuit_open`) on both Codex seats across five retries. I escalated a waiver
> to the operator rather than self-authorize an exception to §2c; **the circuit
> recovered before they ruled, so the request was withdrawn as moot.**
>
> ## ✅ RE-FRAMED AND `ready` — 2026-07-25, anchors re-derived on `4c5afda6`
>
> **The frame is `docs/program/wp/RT-FNSPLIT-B2A-S-selection-defunctionalization.md`
> — REWRITTEN, not amended.** ⛔ Every anchor in the old draft was stale
> (`lower_expr` moved `:3847` → `:4255`; `define_function` `:202` → `:217`;
> builder `:140` → `:144`). D1–D3 are gone (they were `B2A-C`'s).
>
> ⭐ **D1 is now a ONE-FIELD statement, because `B2A-C` co-located the pair:
> `OwnedSourceOccurrence` drops `expr: RuntimeExpr`, leaving `static_origin` as
> the sole identity.** That is what turns provenance into a selector.
>
> ⛔ **THIS UNIT DELIBERATELY RETIRES A LANDED PIN.** `B2A-C`'s **N3** asserted *no*
> `origin -> expr` lookup from a lowering consumer. **B2A-S introduces exactly
> that lookup**, so N3 is deleted and replaced by a sole-dispatcher pin: the count
> goes **0 → 1**, never 0 → unbounded (frame AC-4). A reviewer checking the new
> lookup against `B2A-C`'s AC list without this will reject a correct diff.
>
> ⚠ **Not yet released.** Fleet is single-threaded; run the §2c gate before kicking.
>
> ## ⭐⭐ ADVERSARY-SUPPLIED TRIPWIRES — THIS UNIT DELIBERATELY CROSSES BOTH
>
> The adversary discharged both surfaces I flagged on B2A-C (`evt_7mve56d192pv6`)
> and, better, gave the **exact tripwire** for each. ⛔ **B2A-S's whole job is to
> cross tripwire 1 on purpose** — so the frame must say so explicitly rather than
> letting a reviewer read the crossing as a regression.
>
> **1. `OwnedSourceOccurrence` is provenance TODAY.** Verified: `static_origin`
> is **never compared, never a map key, never branched on** anywhere in production
> `lowering/`. The struct (`lowering/mod.rs:238-243`) holds `expr` **and**
> `static_origin` together, so the expression is *co-located*, not retrieved —
> there is nothing to look one up *with*.
> ⇒ **TRIPWIRE: the first read of the origin's VALUE in a decision.**
> ⭐ **That read IS this unit's D4/D5.** When B2A-S makes the tag a selector it
> crosses this line by design — and at that moment the retained-body carrier must
> leave **in the same diff**, because from then on two authorities really would
> coexist. **State the crossing in the frame and in the handoff.**
>
> **2. Nested `ComputationalMatch` sharing a scheduling entry is BENIGN today.**
> Verified: every production use of `.entry` is an **edge endpoint**
> (`self.edge(from, to, kind)`) or `next = planned.entry` for sequencing — never a
> map key, never an identity, never compared.
> ⇒ **TRIPWIRE: the first collection keyed by `.entry`.** Occurrences remain safe
> as keys because B1R enforces the `origin.0 == planned_node.0` bijection.
> ⛔ **So B2A-S must key its selector on `.occurrence`, NEVER on `.entry`** — and
> an AC should redden if a collection keyed by `.entry` appears.
>
> ⚠ **Carry N1 into this unit as a deliverable** (adversary, cost ≈ 3 lines): the
> AC-11 topology differential's provenance is **VERIFIED** — all 7 rows reproduce
> byte-for-byte from `70bd2c74` — but **the recipe is not in the tree**, so nobody
> else can re-verify it. ★ The deep reason it needed an outside check: **the
> asserted property is EQUALITY against committed constants, so a post-change
> re-capture would have produced byte-identical values — no observation
> distinguishes a genuine pre-change baseline from a re-recorded one.**
> ⇒ **Record the recipe in the comment**: base SHA, the two probe function names
> (`b2ac_topology_digest`, `b2ac_topology_fixtures`), and the
> `git worktree add --detach <path> 70bd2c74` + test invocation. Demonstrate the
> binding; do not testify to it.
>
> ## ⇢ RE-CUT 2026-07-25 — THIS NODE IS THE **SELECTION** UNIT, NOT RETIRED.
>
> **The Architect ruled the re-slice at `evt_1jdh8pn8y96z`.** This node survives
> with a **narrowed scope** and a **new predecessor**:
>
> ```
> B1R → RT-FNSPLIT-B2A-C → RT-FNSPLIT-B2A-S → RT-FNSPLIT-B2F
>       correspondence      THIS NODE          functionization
>                           (selection)
> ```
>
> - **D1–D3 LEAVE this node** — the dense table, its lifetime, and the visibility
>   widening belong to **`RT-FNSPLIT-B2A-C`**, which transports the origin to the
>   site. `5c7eae26` folds in *there*, not here.
> - **D4 + D5 STAY, and are now satisfiable** — because correspondence puts the
>   origin *in scope* at the two `lower_expr` closure arms. D4 was unsatisfiable
>   only because this frame assumed an origin nothing produced.
> - ⛔ **This unit remains ATOMIC, by ruling:** add the tag **as selector**,
>   **remove the retained body carrier**, and **install the sole dispatcher** —
>   *in the same diff*. That was always the right requirement; only its
>   prerequisite was missing.
>
> ⚠ **The frame at `docs/program/wp/RT-FNSPLIT-B2A-S-selection-defunctionalization.md`
> is STALE against this re-cut** — it still carries D1–D3 and still assumes the
> origin is available. ⛔ Do not build from it until the Steward re-cuts it.
> **Inventory:** this unit closes **entry 1**; entry 2 waits for `B2F`; entry 3 is
> closed by `B2A-C`. State them separately.
>
> ## ⛔ HALTED AT HARD-STOP #7 — 2026-07-25 (the stop that forced the re-cut)
>
> **D4 is unsatisfiable inside this frame's own boundary.** Raised by
> `runtime-leader` at `evt_2fvxkmfw8m1k8` after D1–D3 landed clean at
> `5c7eae26`; Steward re-verified the measurement on `origin/main` = `70bd2c74`.
> `lower_expr` (`core.rs:3847`) has **no origin in scope**, the only two
> production `Lowered::Closure` constructions are its `:4211`/`:4226` arms
> (`body: (**body).clone()`), and every non-threaded key is prohibited by this
> frame's own D6. Threading is the excluded source-machine scope. ⇒ **This
> frame requires its own prerequisite.**
>
> ⛔ **`5c7eae26` IS NOT MERGEABLE AS A STANDALONE UNIT.** The Architect's Q2
> permission (`evt_25ynt8615r9sk`) is **conditional** — "provided the complete
> tag-plus-sole-dispatch conditions hold." D4 *is* the sole-dispatch condition,
> so the proviso is unmet and the standalone-checkpoint permission never fires.
> This is the Architect's own conditional evaluating false on measured ground,
> not the Steward overriding it. The branch is retained as **durable input** to
> the successor, at `origin/wp/RT-FNSPLIT-B2A-S-selection-defunctionalization`.
>
> **The re-slice is gated on one measurement** — whether the planned-origin
> population is *total* over the closure occurrences reachable in `lower_expr`
> including the source-machine fallback. Totality ⇒ the correspondence is
> mechanical threading and becomes its own production unit sized by the
> **producer**; partiality ⇒ it drags static-authority scope and collapses into
> `RT-FNSPLIT-B2F`. See `RT-NATIVE-FNSPLIT` hard-stop #7 + inventory entry 3.
>
> ## ⛔ SUPERSEDED HISTORY BELOW
>
> Kicked to the Runtime ring 2026-07-25 (`evt_3wmhpdq7dm8d8`).
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
