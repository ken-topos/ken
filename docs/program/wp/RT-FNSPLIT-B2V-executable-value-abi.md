# `RT-FNSPLIT-B2V` — the executable boundary-value ABI

**Owner:** Team Runtime · **Size:** `L` · **Gate:** none · **Inert:** yes

| dependency | landed | PR |
|---|---|---|
| `RT-FNSPLIT-B2O` — validated `SemanticOwner` partition | `origin/main` = `e470ab65` | #963 |
| `RT-FNSPLIT-B2R` — slot order, width, declared ownership | `origin/main` = `c986d0a3` | #967 |

**This node blocks `RT-FNSPLIT-B2F`, which is HELD until this frame is on
`origin/main` and explicitly kicked** (Architect condition,
`evt_28cnmxf6ncghn`).

> ## ⛔ ANCHORS ARE PERISHABLE — re-derive at pickup
>
> Every citation below was re-derived at `origin/main` = **`164afa8a`**.
> **Verify against the landed code, not this line.** If a fixed input here is
> false against the code, **say so and escalate — do not quietly build around
> it.** This chain has twice had a frame go stale between authoring and pickup.

## Why this node exists — `B2R` declared the SLOT; nothing defined the VALUE

`B2O` and `B2R` give static code ownership, unit population, slot order and
width, and declared ownership. **They do not define what the bits of
`ValueWord` / `ResultWord` MEAN, nor how compiled code inspects a dynamic
aggregate.**

The landed lowering confirms the distinction: `Lowered` is a **compile-time
specialization lattice** (`lowering/mod.rs:415`); `ground_value` exports only
fully-constant aggregates, **through a Rust-side table**; `HostResult` and
dynamic aggregates have **no executable word representation**. ⇒ **A
compiled-once callee cannot consume the measured `Constructor` / `HostResult`
parameters.**

### ⭐ The escape hatch is closed — and this is the sharpest measurement

An aggregate result works **today** only because **the consumer is Rust**. The
callee returns an `iconst` token; the caller decodes it via `ResultDecoder` +
`result_table` (`lowering/mod.rs:290`, `:5824`) — **compile-time Rust objects
living in `CompiledModule`**. Under functionization the consumer is **emitted
code**, which holds no decoder and cannot read that table.

⇒ **The existing aggregate-result path is a Rust-side decode at the artifact
boundary, not a value representation.** It does not generalize from the root
boundary to an internal one.

### The measured population (ring's measurement, relayed — re-measure it)

| class | measured | carried by the declared 8-byte word? |
|---|---|---|
| `Constructor` | 29 `Parameter` transfers | **no** |
| `HostResult` | 4 | **no** |
| aggregates at the root `Result` slot | 26 of 154 (`Constructor` 22, `Record` 2, `String` 1, `Bytes` 1) | **no** |

⚠ The root-result figure is **explicitly root-only, not a per-unit census** —
no such census can exist while there is one emitted unit. ⛔ **A fail-closed
guard would reject ~33 of 41 source-valued transfers**, which cannot satisfy
`B2F`'s `D6` or `D7`. **The guard is sound and insufficient; that is the whole
finding.**

> ## ⛔⛔ THE ONE THING NOT TO DO — DO NOT SPLIT THIS NODE
>
> **Architect:** *"Do not split the value contract from its access interface: a
> second slot-only declaration would reproduce #9/#10 one layer down."*
>
> ★ **This is the chain's lesson as a construction rule.** `#9` produced `B2R`
> — a declaration. `#10` **is `#9` again**, one representation layer below,
> because that declaration had no executable meaning. **A third
> declaration-only node produces `#11` in the same shape.**
>
> ⇒ **The value representation and the emitted-code interface that reads it land
> TOGETHER or not at all.** If you find yourself proposing to defer the
> interface "until something consumes it" — that is the defect, not a
> simplification.

## The landed surface you build on — re-measured at `164afa8a`

| what | where |
|---|---|
| the `Lowered` specialization lattice — **21 variants** | `cranelift_backend/lowering/mod.rs:417` (`:415` is the `#[derive]`) |
| `Store` · `intern` · `slot_id` — the **encode half only**; see the `D2` amendment | `store.rs:343`, `:360`, `:400` |
| `AbiCarrier::ValueWord` · `GroundValueCarrier` · `ResultWord` | `planning/static_transition/abi.rs:64`, `:74`, `:76` |
| declared ownership per carrier (`OwnedByFrame` / `BorrowedForActivation` / `TransferredToCaller`) | `abi.rs:126`–`:131` |
| the Rust-side decode path that **does not count** | `lowering/mod.rs:290` (`result_table`), `emit_result` at `:5820` |

⚠ **Two locators were off by a line or two and are corrected above** — the ring
re-derived them rather than silently adjusting, which is the right call: *a
locator one reader silently corrects is a locator the next reader re-derives.*
⛔ **`store.rs`'s row no longer says "subsume" — read the `D2` amendment before
you use it.**

## Deliverables

### `D1` — give the word a meaning

**One closed 64-bit boundary-value representation** used by `ValueWord` **and**
`ResultWord`, **reconciled explicitly with `GroundValueCarrier`**.

Because the plane has **no per-slot static type**, the permanent shape is a
**tagged word**: immediate payload where lawful, otherwise an **opaque handle**
into runtime-owned value storage.

⛔ **It must not specialize the representation from a JIT seed value or from
caller depth.** That is the `B2R` seed-environment lesson: a representation
chosen by inspecting a value describes a program that cannot be written.

### `D2` — subsume **and complete** the existing runtime machinery

> ## ⛔ AMENDED 2026-07-25 — THE ORIGINAL WORDING WAS A FALSE FIXED INPUT
>
> This deliverable said *"reuse the existing runtime value/store substrate,"*
> and the landed-surface table called `store.rs` *"the value substrate to
> **subsume**."* **The ring re-derived the anchors at `aecdb001` and the
> substrate does not have the half `D2` needs** (`evt_7vxre2rsm7xhk`, routed
> `evt_1npjzv3pt1976`). The Steward wrote that premise; the correction is the
> Steward's, and the ring was right to report it.
>
> | claim | measured at `aecdb001` |
> |---|---|
> | `Store` `:343`, `intern` `:360`, `slot_id` `:400` exist | ✅ true — anchors live |
> | it interns a value and returns a stable `SlotId`, with dedup | ✅ true |
> | **a `SlotId` can be resolved back to a value** | ❌ **no such path exists, for anyone** |
>
> There is **no `slot_id → value` lookup and no reverse index** — `slot_id` is a
> monotonic counter and the index is keyed by *hash of canonical bytes*. There
> is **no canonical decoder**: `canonical.rs` declares `encode_canonical` and
> nothing else. `Arena::get` is **private** and keyed by
> `(page_idx, offset, len)`, not by slot id.
>
> Three facts in the same direction:
> - `intern` **asserts `value.is_compound()`** — scalars panic, by design.
> - It types over `values::Value`, **not** `RuntimeGroundValue`:
>   `Value::Constructor { constructor_id: u32 }` vs
>   `RuntimeGroundValue::Constructor { constructor: RuntimeSymbol }`, and **no
>   symbol↔id interner exists in `ken-runtime`** (every workspace-wide
>   `constructor_id` hit is `ken_kernel::GlobalId`, a different namespace).
> - **`ken_runtime::store` has ZERO consumers workspace-wide** — established by
>   a field probe across all of `crates/`, not a narrow grep.
>
> ⇒ ★ **A handle whose payload is a `SlotId` is unprojectable by construction.
> The store is a one-way id-assignment index: it has the encode half and no
> decode half — precisely the half a handle needs.**
>
> ### ✅ This is a FRAME CORRECTION, not hard-stop `#11` — the ring's call, affirmed
>
> `D2` is **bigger than its wording, not unsatisfiable.** The missing piece is
> small, local to `ken-runtime`, and **is exactly `D2`'s stated work** — the
> inverse of an operation that already exists. Building it is the **opposite**
> of the parallel permanent heap `D2` forbids: it makes the store the single
> owner of record instead of routing around it. Nothing about it is a
> prerequisite another node must supply.
>
> ⭐ **The ring applied its own `#10` standard against its own interest and
> declined to stop.** After two consecutive nodes ending in hard-stops, the
> cheap move was a third; it measured instead. That discrimination — *missing
> prerequisite* vs *under-described in-scope work* — is the judgment the
> escalation path depends on, and it went the right way here.

**Reuse AND COMPLETE the existing runtime value/store substrate** (`store.rs`)
for persistable Ken values — the read-back path (slot-keyed residency plus the
decode inverse) and the `RuntimeGroundValue ↔ Value` bridge with a store-owned
symbol interner are **in scope and required**. Add only the
**invocation-scoped** storage needed for **borrowed ingress** such as
`HostResult`.

⛔ **Do not create a parallel permanent heap by default.**

⛔ **If a word is a handle, name the referent owner and lifetime SEPARATELY from
the frame slot that stores the word.** `AbiStorageOwner::ActivationFrame` must
not silently stand in for an invocation-scoped or persistent **referent** owner.
⚠ These are two different questions — *who owns the slot* and *who owns the
thing the slot points at* — and `B2R`'s vocabulary only answers the first.

### `D3` — make the interface executable

Supply the **constant-width emitted-code interface** to **construct**,
**discriminate**, and **project** the representation. At minimum:

- scalar extraction
- constructor / result **tag**
- field **count** and **index**
- record field access
- `HostResult` **success/payload disposition**

⛔ **A Rust-side `result_table` token with no runtime lookup path does not
count.** ⛔ **The helper/symbol population is fixed Θ(1) per module** — never
per origin, never per runtime value. (This is the growth invariant the whole
`RT-NATIVE-FNSPLIT` program exists to protect; a per-value helper reintroduces
the defect the parent node is closing.)

### `D4` — close the transfer population STRUCTURALLY

**One exhaustive, no-wildcard disposition over every `Lowered` variant that can
reach `Parameter`, `Capture`, or `Result`.** Four dispositions, and every
variant gets exactly one:

| disposition | meaning |
|---|---|
| **represented immediate** | payload fits the tagged word directly |
| **represented handle** | opaque handle, **with explicit lifetime and referent owner** |
| **protocol-only** | never a source value at a boundary |
| **fail-closed forbidden** | rejected before emission, with an exact error |

⛔ **The 41-transfer histogram is corroboration, NOT the population proof.** The
proof is the exhaustive match over the **21 landed variants** at
`lowering/mod.rs:415` — so a new variant is a **compile error**, not a silent
`ValueWord`.

⛔ **`Constructor` and `HostResult` are REQUIRED LIVE ARMS**, not optional
follow-ups. A disposition that parks either in *forbidden* has not satisfied
this deliverable.

### `D5` — prove runtime OBSERVABILITY, not round-trip serialization

⭐ **This is the deliverable most likely to be discharged vacuously, so read the
controls literally.** Round-tripping a value through the representation and
reading it back **in Rust** proves nothing about emitted code.

Required controls:

1. **Non-constant `Constructor` through a `Parameter`**, inspected — tag and
   field — by a **separately compiled** body.
2. **`HostResult` across a boundary**, with the callee selecting the correct
   **success/error** payload.
3. **Nested aggregate** `Capture` / `Result` flow.

⛔ **Mutation obligations — each must REDDEN:**

| mutate | must redden because |
|---|---|
| callee reads a **compile-time template** | it is not reading the runtime value |
| callee reads a **constant table** | that is the Rust-side path, which does not generalize |
| callee reads the **wrong referent owner** | `D2`'s separation is real, not cosmetic |

⛔ **Borrowed ingress must FAIL CLOSED if it escapes the native invocation.**

### `D6` — remain INERT

**May add:** the representation, runtime support, declarations, pure codegen
helpers, isolated tests.

⛔ **Adds NONE of:** production generated-function population · production
cross-owner call · switch-over · a second body-emission authority.

**Existing root-function / definition / call censuses remain UNCHANGED.** ⚠ Any
constant helper declarations are **predicted before measuring** and
**re-baselined explicitly** — see the pin below, which this node *will* trip.

> ### ⛔ THE PIN YOU WILL TRIP — re-baseline it deliberately
>
> `D3` adds Θ(1) module-level helper declarations. The backend censuses count
> definitions and declarations per module, and `AC-G0` on `B2F` records the
> current answer as **6 definitions / 8 declarations** with the 6 pinned as
> `LOCAL_HELPER_COUNT`. **Your helpers move the declaration count.**
>
> ⇒ **Predict the new count in the frame's own terms BEFORE measuring**, state
> it, then re-baseline. ⛔ **Do not discover it as a red pin and adjust the
> number to match** — that converts a live oracle into a rubber stamp.
>
> ⚠ **And `B2R` already taught this exact lesson on this exact surface:** its
> `AC-9` predicted 0 and measured 13, because registering one file changed the
> **input** to every pin iterating `BACKEND_PRODUCTION_SOURCES`. **A predicted
> population must include registration-driven fan-out.**

## Acceptance criteria

**AC-1 — the representation is closed and type-enforced.** One 64-bit tagged
word serves `ValueWord` and `ResultWord`; its relation to `GroundValueCarrier`
is stated. A new carrier or a new tag is a **compile error**, not a default.

**AC-2 — the representation cannot be value-specialized.** ⭐ **Prefer the
compiler over a test:** if `D1` is built so that no seed value or caller depth
is *in scope* at the construction site, that is a stronger discharge than any
assertion. State which mechanism enforces it. (`B2R` did exactly this for the
seed environment and it was the strongest thing in that node.)

**AC-3 — the `Lowered` disposition is exhaustive with no wildcard.** All 21
variants, one disposition each, no `_` arm. `Constructor` and `HostResult` are
**live represented arms**. Assert the **exact** error for every fail-closed arm
— never `is_err`.

**AC-4 — emitted code can construct, discriminate, and project.** Each `D3`
operation exercised **from emitted code**, not from Rust.

**AC-5 — observability controls redden under all three `D5` mutations.** Record
**which detector fired** per mutation — a mutation that reddens does not confirm
*your* detector caught it.

**AC-6 — referent owner and slot owner are distinguishable.** A control that
reddens if `ActivationFrame` is substituted for the referent owner.

**AC-7 — borrowed ingress fails closed on escape.** Exact error.

**AC-8 — the node is INERT.** Root-function / definition / call censuses
unchanged; helper declaration delta **predicted, stated, then re-baselined**.

**AC-9 — helper population is Θ(1) per module.** Demonstrated over ≥2 module
sizes, not asserted.

> ### ⛔ AMENDED 2026-07-25 — `AC-8`/`AC-9` HAD NO STANDING ORACLE
>
> The frame told the ring it would trip the declaration-count pin. **That
> premise is false for the proposed placement, measured before building**
> (`evt_7vxre2rsm7xhk`).
>
> Every landed backend census is scoped to `cranelift_backend/**`:
> `BACKEND_PRODUCTION_SOURCES` enumerates 13 files, and
> `correspondence_adds_no_emitted_unit_to_the_production_census` counts
> `FunctionBuilder::new(` / `.define_function(` / `.declare_function(`
> **per listed file**. `native_int_clif.rs` already declares **8** functions and
> appears in **neither** — it is a *sibling* of `cranelift_backend/`, not inside
> it. Sibling-placed value-ABI helpers are therefore **invisible to all three
> pins**, and the ring predicted, before measuring, that every existing row
> stays unchanged.
>
> ⇒ ★★ **That is a problem, not a relief. If no census counts the helpers,
> `AC-8` has nothing to re-baseline and `AC-9`'s Θ(1) claim has no standing
> oracle — a green that measures nothing.** A pin whose silence is scoped to a
> directory it does not cover is not evidence about a helper outside it.
>
> **⇒ REQUIRED, promoted from the ring's own proposal into an AC:** add a
> **boundary-helper census** pinning the **exact permitted SET of helper
> names**, so *any* addition reddens — **including one nobody imagined**.
> ⛔ **Pin the allowed inventory, not a forbidden list**; a forbidden list is
> open at the top and grows silently. `AC-9`'s Θ(1) demonstration must run
> against **that** census, not against the `cranelift_backend/**` rows.
>
> ⚠ `D4`'s disposition lands in `lowering/mod.rs` (`Lowered` is module-private)
> — an existing census row at `0/0/0`, which it **keeps**, since a `match`
> declares and defines nothing. Predict that row explicitly rather than
> assuming it.

> ### ⛔ PER-PIN EVASION ATTEMPT — AN AC, NOT A HAZARDS NOTE
>
> For **each** pin above, attempt a **compile-preserving evasion** and record
> the result **per pin, in a table with one row per pin**. Name the positive
> control that would fire if the attempt were skipped.
>
> ⚠ **The evasion must vary the axis the pin NAMES.** On `B2R` two witnesses for
> an edge-**layout** law both mutated the same field and both landed on
> **identity** detectors — a green row, named for layout, testing identity.
> **Failing to find a witness is evidence about the witnesses you could think
> of, never about the property.**

> ### ⛔ AMENDED 2026-07-25 (second) — EVERY QA VERDICT CARRIES AN `AC` → CONTROL MAP
>
> The Architect blocked `78a57d90` (`evt_2c6f3natxvwcm`), and its second finding
> is **`AC-4` verbatim, undischarged.** `make_immediate` is the only constructor
> the emitted interface exposes; every live `PersistentGround` /
> `PersistentClosure` / `InvocationBorrowed` / `InvocationHostResult` word is
> materialized **Rust-side** by `BoundaryArenaBuilder::push_node` /
> `materialize_*` before `publish`, and the emitted probes never call
> `make_immediate` at all. What was demonstrated is that emitted code can
> **inspect a Rust-built fixture** — never that it can **construct** one.
>
> ⚠ **`AC-4` is not amended. It was right, and it said exactly this.**
>
> **That candidate cleared QA.** ⛔ Not through inattention: QA verified every
> control the candidate shipped and independently reproduced both false-green
> mutations, and its verdict is sound on the properties it asks. The gap is
> structural — **nothing in the loop ever asked which `AC` each control
> discharges, or whether an `AC` had a control at all.**
>
> ★★ **An `AC` with zero controls is invisible to a review that examines
> controls.** It yields no red, no gap, and no anomaly; it is simply not in the
> frame of view. This is the sibling of the `AC-8`/`AC-9` amendment above, one
> layer out — that one was *an `AC` whose oracle could not see the deliverable*;
> this is *an `AC` with no oracle at all*. **Both are green-compatible with the
> deliverable not existing.**
>
> **⇒ REQUIRED of every QA verdict on this node:** a table with **one row per
> `AC` in this frame**, naming the **control** that discharges it and the
> evidence. An `AC` whose row cannot name a control is recorded as
> **`NO CONTROL — open residual`**, in the verdict, in that spelling.
> ⛔ **Omitting the row is the defect this exists to stop** — a verdict that
> lists only what it checked cannot distinguish *discharged* from *never
> asked*, and the two read identically.
>
> ⚠ **This is content added to a message QA already sends** — no new party, no
> new hop, no new gate, no change to the reviewer set.

## Do-not-reopen guardrails

1. ⛔ **Do not split this node** (see the box above). This is the ruling's
   central constraint.
2. ⛔ **Do not re-open `B2O`'s owner partition or `B2R`'s slot shapes.** `B2R`
   is **not defective within its declared scope** — it simply never covered the
   value half.
3. ⛔ **Do not build the switch-over.** That is `B2F`, and it stays atomic.
4. ⛔ **Do not create a parallel permanent heap** (`D2`).
5. ⛔ **Do not let the helper population scale with origins or values** (`D3`).
6. **Every anchor is perishable.** Escalate a false fixed input; do not build
   around it.

## What this does to `B2F`

`B2F` remains the same atomic boundary. ⭐ **Its `AC-11` is re-scoped by this
ruling** to **enforcement of this prerequisite on every `Parameter` / `Capture`
/ `Result` transfer** — **not** rejection of common aggregates, and **not**
inheritance from `C4`.

## Standing

- ⛔ **Local builds/tests are TARGETED ONLY** — `scripts/ken-cargo -p
  ken-runtime`, or `--test <name>`. **Never `--workspace`** (`COORDINATION §12`,
  operator hard rule). Workspace-green and `--locked` mean **green in CI**.
- ⚠ **A change to `eval.rs`'s reifier or to `store.rs` needs the full
  `-p ken-interp` / `-p ken-runtime` suite**, not a single targeted test.
- **Report an unpushed ref and KEEP GOING.** Build seats have no GitHub
  credential by design; the Steward pushes. Raising it is not gating on it.
- **Hard-stop protocol.** Count of record is **10**; **next research pull =
  `#12`**, armed. ⛔ **`#10` is recorded under symptom-inventory entry 2 / the
  prerequisite chain — it is NOT a fourth entry.** Inventory stays at 3 entries;
  next predicate check at the **6th**.
- Read `agent/playbooks/tools/pin-a-property.md` before writing any assertion.
