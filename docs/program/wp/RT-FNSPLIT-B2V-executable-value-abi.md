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

## The landed surface you build on — RE-ANCHORED at `a7d3e2b0`

⛔ **This table was re-measured by the Steward on 2026-07-26 against
`origin/main` = `a7d3e2b0`, after `RT-VALUE-TOTALITY-P1` merged (`8f677ebc`).**
The previous anchor was `164afa8a`. Every cell below was re-derived with
`git show origin/main:<path>`; the **verdict** column says what moved, so you can
see which rows you may still trust from your held branch.

| what | where — measured at `a7d3e2b0` | verdict |
|---|---|---|
| the `Lowered` specialization lattice — **21 variants** | `cranelift_backend/lowering/mod.rs:415` (`:414` is the `#[derive]`) | ⚠ **both line cells moved −2**; the 21 count HOLDS |
| `Store` · `intern` · `slot_id` — the **encode half only**; see the `D2` amendment | `store.rs:343`, `:360`, `:400` | ✅ exact, unchanged |
| `AbiCarrier::ValueWord` · `GroundValueCarrier` · `ResultWord` | `cranelift_backend/planning/static_transition/abi.rs:64`, `:74`, `:76` | ❌ **PATH was wrong from birth** — lines exact |
| declared ownership per carrier (`OwnedByFrame` / `BorrowedForActivation` / `TransferredToCaller`) | same file, `:126`–`:131` | ✅ lines exact; same path fix |
| the Rust-side decode path that **does not count** | `lowering/mod.rs:290` (`result_table`), `emit_result` at `:5820` | ✅ exact, unchanged |

⛔ **`store.rs`'s row no longer says "subsume" — read the `D2` amendment before
you use it.**

> ### ⛔ THE `abi.rs` PATH WAS WRONG FROM BIRTH, AND A CORRECTION PASS WALKED PAST IT
>
> The framed path `planning/static_transition/abi.rs` **does not exist at
> `a7d3e2b0` and did not exist at `164afa8a` either** (`git cat-file -e` fails at
> both). The real path has always been
> **`cranelift_backend/planning/static_transition/abi.rs`** — the frame dropped
> the `cranelift_backend/` component. Every line number in those two rows —
> `:64`, `:74`, `:76`, `:126`–`:131` — is **exact**. My defect; the frame is the
> Steward's.
>
> ⭐ **The tell worth keeping: this survived the ring's own locator-correction
> pass.** The note that used to sit here said *"two locators were off by a line or
> two and are corrected above."* The ring re-derived **line offsets** and the
> broken **path** came through untouched — because a locator with a wrong path and
> right lines reads as *correct* to anyone who navigates by symbol search rather
> than by opening the path. ⇒ ⛔ **A locator has two independent coordinates and
> re-deriving one is not evidence about the other.** Check the path with
> `git cat-file -e <base>:<path>` before you trust any line number in it.
>
> ⚠ It is also why the previous note's principle stands unchanged and is retained:
> *a locator one reader silently corrects is a locator the next reader
> re-derives.* Report the correction; do not absorb it.

### ⭐ WHAT `RT-VALUE-TOTALITY-P1` PUT UNDER YOU — new surface, same premises

P1 rewrote `canonical.rs` and `values.rs`, the two files `D2` builds on. **The
`D2` amendment's load-bearing premise SURVIVES** — re-measured, there is still
**no decoder anywhere in `ken-runtime`**: `canonical.rs`'s only non-test
functions are `encode_canonical`, the LE writers, `minimal_limbs`,
`child_positions::push` and `encode_header`, and a workspace grep for
`fn decode*` / `fn from_canonical` in `crates/ken-runtime/src/` hits only
unrelated types (`native_int`, `native_join_plan`,
`oriented_subcontinuation_plan`). ⇒ **`D2` is still correctly scoped and still
required.**

⚠ **But the amendment's LETTER is now stale, so read it for the premise, not the
inventory.** It says *"`canonical.rs` declares `encode_canonical` and nothing
else."* That was true at `aecdb001`. At `a7d3e2b0` the file also carries the
whole iterative encoder. ⛔ Do not read the stale sentence as a claim that the
file is otherwise empty — the true claim, unchanged, is **there is no decode
half**.

New machinery you should build **with**, not around:

| what | where at `a7d3e2b0` |
|---|---|
| `child_positions` — sealed-trait child enumeration, `pub(super) fn push` | `canonical.rs:139` |
| `OwnedChildren::push_steps` impls (the only two: `Vec<Value>`, `BTreeMap<Vec<u8>, Value>`) | `canonical.rs:111`, `:115`, `:124` |
| `encode_header` — the per-variant header/tag writer, exhaustive over `Value` | `canonical.rs:167` |
| `encode_canonical_recursive_reference` — the twin reference encoder, **`#[cfg(test)]`** | `canonical.rs:362` (attr at `:361`) |
| `detach_children` · `rebuild` — the iterative `Drop`/`Clone` worklist machinery | `values.rs:138`, `:179` |
| `Job::Visit` — `Clone`'s worklist arm | `values.rs:309` (`Job` enum at `:299`) |

⛔ **`encode_canonical_recursive_reference` is `#[cfg(test)]` — it is NOT
available to production code.** If a deliverable needs a recursive encoder
outside tests, that is a frame question for the Steward, not a `cfg` edit.

> ### ⛔⛔ DO NOT LEAN ON `AC-V1b`'s COVERAGE PIN — IT DOES NOT BIND
>
> P1's `ac_v1b_corpus_covers_every_value_variant` (`canonical.rs:750`) is the pin
> that establishes the iterative restructuring changed no bytes. **Its coverage
> guarantee is not real.** The doc at `:746`–`:749` says the count is *"counted
> from the corpus itself against the enum's own arm count, so adding a variant
> without extending the corpus reddens."* The body is
> `assert_eq!(kinds.len(), 25)` where `kinds` comes **solely** from
> `differential_corpus()`; the test body contains **no reference to `Value`'s
> cardinality**, so `kinds.len()` is invariant under adding a variant *by
> construction*.
>
> The adversary measured it (`evt_wv5fng3kt2yx`): a 26th variant plus only the
> five arms the compiler demanded, corpus untouched ⇒ all three `AC-V1b` tests
> **pass**, full `-p ken-runtime --lib` **371/371**. And the module doc at
> `:350`–`:355` already states honestly that the differential is *not* an
> independent byte oracle — so **coverage was its entire value**, and coverage is
> the part that does not bind.
>
> ⚠ **Honest scope: exhaustiveness IS genuine.** A variant cannot enter
> **unhandled** — the compiler named all five sites precisely. It enters
> **unverified**. ⇒ For B2V that means: **you inherit no coverage protection from
> `AC-V1b`.** If a B2V deliverable adds or changes anything `encode_header`
> dispatches on, your own controls must carry the coverage claim.
>
> ⭐ **`D4` already does this correctly, and is the pattern:** *"the proof is the
> exhaustive match over the 21 landed variants … so a new variant is a compile
> error, not a silent `ValueWord`."* Bind the number to the type; never restate it
> beside. `AC-V1b` is the same intent implemented as a frozen literal.
>
> ⛔ **This is NOT B2V's to fix** — it is tracked separately against
> `RT-VALUE-TOTALITY` (P1 is closed; the repair does not reopen it). It is here
> only so you do not build on a protection you do not have.

> ### ⚠ AND THE `"will not compile"` BOUND IS SCOPED TO FIVE POSITIONS
>
> `values.rs:14`–`:20` states that giving a recursive child position indirection
> *"will not compile"*. Verified, and **true for the five positions it names**:
> `child_positions::push` is bounded on a sealed trait implemented only for
> `Vec<Value>` and `BTreeMap<Vec<u8>, Value>`, so retyping `args` to
> `Vec<Rc<Value>>` fails at the call site.
>
> ⛔ **It is not a bound on positions that do not route through `push`.**
> `Step::Val` is constructible directly in the parent module — `canonical.rs:149`
> does exactly that for the root — so an arm can enumerate children without ever
> reaching the sealed bound, and the guarantee simply does not apply to it
> (adversary, `evt_wv5fng3kt2yx`). Making `Step::Val` constructible only inside
> `child_positions` would close it.
>
> ⇒ **Why B2V is told this:** `D2` adds the decode half in this exact
> neighbourhood. ⛔ If your work introduces a child position, do **not** treat
> `values.rs:14`–`:20` as covering it — that sentence is a claim about five
> call sites, not about the module. Route it through `child_positions::push` or
> say plainly in your evidence that a new position is outside the bound.
>
> ⚠ This is the **same defect class as the `AC-V1b` doc above and P1's
> `breadth-first` comment**: a *correct* mechanism argument written one scope
> wider than the mechanism it describes. All three were true of what the author
> checked and false as stated.

### ▶ YOUR HELD BRANCH — RE-ANCHORED; the pre-anchor tip is preserved on `origin`

⛔ **THE RE-ANCHOR THIS SECTION USED TO PRESCRIBE HAS HAPPENED. The text below was
edited on 2026-07-26 because it named `a7aa60eb` as the live tip and told you to
re-anchor — both were true when written and are now false.** Verified with
`git ls-remote origin` (read-only):

```
wp/RT-FNSPLIT-B2V-executable-value-abi     fed42481   <- live tip, HELD CHECKPOINT
preserved/rt-fnsplit-b2v-prereanchor-a7aa60eb  a7aa60eb   <- the old tip, on origin
preserved/rt-fnsplit-b2v-ab11a3d2          ab11a3d2   <- intermediate, on origin
re-anchored base                           69750fa3
```

⚠ **`fed42481` is a held checkpoint, NOT a QA candidate** — see `RULING R4`. It is
**four commits past `69750fa3`** and ⛔ **not** an ancestor-descendant continuation
of `ab11a3d2` (`git merge-base --is-ancestor ab11a3d2 fed42481` exits **1**). ⇒ A
commit *distance* is not a fast-forward; check any preservation claim against the
operation it is meant to protect against, before that operation runs.

⭐ **Both preserved tips are on `origin`, not merely local.** A single local ref is
zero off-box copies — a preservation claim that names one has not preserved
anything a force-move can't take.

**Why the pre-anchor re-anchor was needed at all**, retained because the reasoning
still governs your next rebase: the old base predated `RT-VALUE-TOTALITY-P1` and
both sides touched `canonical.rs` — the `D2` decode inverse added a region at
~`:259`, P1 rewrote the encoder. The Steward's read-only
`git merge-tree --write-tree` probe reported **exit 0, no textual conflict**
(merged tree `f26ba8d9`), against a synthetic same-line control that correctly
reported exit 1.

⛔ **THAT WAS A TEXTUAL RESULT AND NOTHING MORE — IT IS NOT "IT STILL BUILDS".**
A clean three-way merge means no hunk overlapped. It says nothing about whether
additions still *compile* against a rewritten encoder: a call to something P1
renamed, narrowed to `#[cfg(test)]`, or removed merges **silently clean** and then
fails to build. ⇒ **After any re-anchor, your first act is
`scripts/ken-cargo test -p ken-runtime`, and a merge probe is never a substitute
for it.**

> ⭐ **And note how that probe was checked, because the same trap is in your
> `D5`.** The first "positive control" the Steward ran was `8f677ebc` × `a7aa60eb`
> — a pair *assumed* to conflict because both touch `canonical.rs`. It returned
> exit 0, which proves nothing: **a negative result from a case you never
> established would fail is not a control.** The real control had to be a
> synthetic same-line divergence, where a conflict was *known* to be the right
> answer. ⇒ ⛔ **A control must be a case whose answer you already know** — this is
> exactly `D5`'s mutation table, where each row names the reason it must redden.

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

**One exhaustive, no-wildcard match over every `Lowered` variant that can reach
`Parameter`, `Capture`, or `Result`, assigning each variant exactly one STATIC
ENCODING POLICY.** Five policies; every variant gets exactly one:

| static policy | what it declares about EVERY value of the variant |
|---|---|
| **represented — immediate-only** | every value encodes in the tagged word; **no spill arm exists** |
| **represented — handle-only** | every value encodes as an opaque handle, **with explicit lifetime and referent owner** |
| **represented — immediate-with-declared-handle-spill** | values encode immediate **or**, on a declared closed condition, as a handle; the handle arm carries the **same** lifetime/referent-owner obligations as handle-only |
| **protocol-only** | never a source value at a boundary |
| **fail-closed forbidden** | rejected before emission, with an exact error |

⛔ **A policy is a claim about the whole variant, never about one sampled
value.** *Immediate-only* is the strong claim that **no** value of the variant
spills; do not assign it to a variant that has a spill arm. `Lowered::Int` →
`RepresentedImmediate { tag: ImmediateInt, spill: Some(Int) }` is the **third**
row, not the first — a small `Int` yields an immediate word and a wide one
yields a persistent handle, under **one** static policy.

⚠ **The runtime OUTCOME per input is a separate, finer classification** — that is
`AC-10`, and it is entailed by the policy rather than replacing it. Neither level
may absorb the other; see the `D4`/`AC-3` clarification in **RECUT 1**.

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

> ⛔⛔⛔ **TWO RECUTS AMEND THIS LIST. Read `## RECUT 2` (2026-07-26) FIRST,
> then `## RECUT 1` (2026-07-25), at the end of this file — BEFORE treating the
> list below as the bar.**
>
> ⛔ **RECUT 2 is a gate, not a note: no B2V acceptance candidate may bind on
> the old per-cell proof shape.** The predicate is **representation
> authority-to-execution closure** over blocks `#1`–`#6`, and ⛔ **a
> hand-maintained matrix that can drift from the production enums does not
> discharge it.** The `AC`→control map stays REQUIRED and stops being the proof.
>
> ⛔ **AND `RULING R4` (2026-07-26, inside `RECUT 2`) SETS THE EVIDENCE BAR FOR
> EVERY `AC` BELOW: causal coverage is PER-SITE.** A whole-graph differential is
> green while one consuming site is disconnected from the authority — measured,
> **439 passed / 0 failed** with one of five `class_guard` sites reverted to its
> literal. ⇒ Each consuming site needs its own behavioural differential or must be
> **named** probe-unreachable. Read `R3` **and** `R4` before treating any control
> below as discharging its `AC`.
>
> ⛔⛔ **RECUT 1 2026-07-25 — these `AC`s are AMENDED.** The Architect named a
> shared predicate across three production blocks: the defect is the **shape** of
> this list — each `AC` below pins one **facet** of an emitted round trip, and
> each block found a facet none of them named. `AC-10` (in **RECUT 1**) closes it
> structurally, and the three `NO CONTROL — open residual` rows are **promoted
> into scope**, not carried as residuals.

**AC-1 — the representation is closed and type-enforced.** One 64-bit tagged
word serves `ValueWord` and `ResultWord`; its relation to `GroundValueCarrier`
is stated. A new carrier or a new tag is a **compile error**, not a default.

> #### `AC-1` layout closure — Architect ruling `evt_1tdq9g139snay`, 2026-07-26
>
> **The node/header field inventory is the SOLE layout authority.** Any
> declared or exported extent is **mechanically derived** from that inventory
> **and consumed** by allocation/publication, **or it does not exist**.
> Publication emits **exactly** the derived extent, and every emitted
> reader/writer offset **plus field width** lies within it.
>
> ⛔ **A committed causal control must REDDEN when the field inventory, the
> published word count, the declared extent, or an emitted offset drifts
> independently.** ⛔ **Checking a hand-maintained constant against another
> hand-maintained constant does not discharge this.**
>
> ⚠ **The contract is one authority with real consumers** — **not** a preferred
> spelling and **not** a frozen byte count. The implementation stays free to
> delete the dead constant, derive it from the field inventory, or introduce a
> typed layout object.
>
> ⭐ **Why this is `AC-1` and NOT `AC-10`** — the Architect ruled the boundary
> explicitly, and it matters. `AC-10` closes an **admitted runtime value** under
> *emitted producer → valid word → separately compiled consumer*. The `fd4e7f08`
> header defect **did not falsify that round trip**: `BOUNDARY_REGION_HEADER_BYTES`
> had **no consumer**, and the published vector was large enough for every
> accessed field. ⛔ **Quietly widening `AC-10` to swallow every dead or drifting
> declaration would have destroyed the named predicate's boundary.** The real
> fault is that the representation claimed **one closed, type-enforced layout
> while carrying two inconsistent authorities, one of them unused** — an `AC-1`
> face. RETAIN's *"one derived layout"* points at the right mechanism, but
> **RETAIN is not an acceptance control**, which is why the requirement lives
> here.

**AC-2 — the representation cannot be value-specialized.** ⭐ **Prefer the
compiler over a test:** if `D1` is built so that no seed value or caller depth
is *in scope* at the construction site, that is a stronger discharge than any
assertion. State which mechanism enforces it. (`B2R` did exactly this for the
seed environment and it was the strongest thing in that node.)

**AC-3 — the `Lowered` disposition is exhaustive with no wildcard.** All 21
variants, **exactly one of `D4`'s five static encoding policies each**
(immediate-only · handle-only · immediate-with-declared-handle-spill ·
protocol-only · fail-closed forbidden), no `_` arm. `Constructor` and
`HostResult` are **live represented arms**. Assert the **exact** error for every
fail-closed arm — never `is_err`. ⛔ **A variant carrying a declared spill must
be assigned the spill policy, not immediate-only** — that misassignment is the
vacuity route `AC-10` exists to close.

**AC-4 — emitted code can construct, discriminate, and project.** Each `D3`
operation exercised **from emitted code**, not from Rust.

**AC-5 — observability controls redden under all three `D5` mutations.** Record
**which detector fired** per mutation — a mutation that reddens does not confirm
*your* detector caught it.

**AC-6 — referent owner and slot owner are distinguishable.** A control that
reddens if `ActivationFrame` is substituted for the referent owner.

> ### ⛔ AC-6 IS BOUND BY TWO ARCHITECT RULINGS — 2026-07-26. READ BOTH.
>
> These are **operative acceptance text**, not commentary. `AC-6` above is
> necessary and **no longer sufficient**: distinguishability alone does not
> discharge it. Blocked candidates `81a68435` and `fe7d8a08` are preserved and
> **neither is an acceptance candidate**.
>
> #### Ruling A — `evt_8851dkes0wmh`: `NoStoreIdentity` does NOT discharge
>
> ⭐ **The distinction is explicitness versus PRESERVATION.** A separately
> compiled consumer can recover *the fact that no store identity exists*; it
> cannot thereby recover **the same identity intact**. ⛔ **Renaming a residual
> as an outcome does not discharge it.** The candidate also declared referent
> owner `PersistentStore` while `NODE_SLOT` stayed null — and this ABI's own
> node-layout contract says a null slot **denotes invocation-arena ownership**,
> so the word was internally inconsistent. **Persistent-region reservation is
> storage governance, not store adoption or content-addressed identity.**
>
> 1. Emitted code may construct and seal a node without a `SlotId`, but that is
>    an internal **`PendingStoreAdoption`-equivalent** state — **not** an
>    admitted or published persistent `HandleWord` outcome.
> 2. Before publication or escape across the generated-function boundary, a
>    **trusted store-owned adoption/finalization** operation validates the
>    complete reachable value graph, canonicalizes/interns it, **mints or reuses
>    the real `SlotId`**, installs placement, and returns the canonical
>    store-backed word.
> 3. ⛔ **Only the store owns mint/reuse authority.** Emitted code remains unable
>    to write `NODE_SLOT`; that anti-forgery property carries **unchanged**.
> 4. A successful `PersistentStore` outcome must be **`StoreMinted` with a
>    non-`NULL_SLOT`**. `NoStoreIdentity` stays valid for **invocation-arena**
>    handles only.
> 5. Equal independently emitted values **converge** to the same store identity;
>    unequal values **never alias**. Adoption failure occurs **before**
>    publication, and ⛔ **no parent may publish with an unadopted reachable
>    child.**
>
> #### Ruling B — `evt_3cw3qtmxbvmc3`: `PersistentClosure` stays ADMITTED
>
> ⛔ **The conservative rejection is not a residual, and it may NOT be repaired
> by narrowing or reclassifying `Closure`/`DeclarationClosure`.** The bound `D4`
> policy deliberately assigns them `PersistentClosure`, and `AC-10` says **no
> admitted well-formed represented value may reject**. Moving the implementation
> gap into the classifier would contradict that authority and **recreate the
> higher-order wall the arm exists to prevent**.
>
> 1. ⛔ **Do not force `Closure` through `RuntimeGroundValue`/`read_ground`.**
>    Adoption needs a **closed canonical-image layer** covering both the existing
>    ground images and `Closure`. The normative image is
>    `Value::Closure { code_id, captured }` — authoritative static code identity
>    plus the **full ordered** canonical captured environment.
> 2. `code_id` derives from the **B2O/B2R authoritative callable-unit/body
>    identity**, in an **artifact-bound namespace**. ⛔ Never a runtime pointer,
>    a cloned `RuntimeExpr`, a symbol-only alias, or caller configuration. **Two
>    artifacts' equal local-origin ordinals must not collide merely because their
>    numbers match.** Identity recording only — **B2F dispatch/emission stays
>    inert and out of scope.**
> 3. Captures adopt **recursively and in order**. **Full capture values, not
>    hashes**, are the canonical content. Equal code identity **plus** equal
>    ordered captures converges; a different code identity, capture value, or
>    capture **order** does not alias.
> 4. Child canonicalization must **preserve the child's actual boundary
>    tag/word**. ⛔ The current hard-coded reconstruction as `PersistentGround`
>    is invalid for a nested `PersistentClosure`.
> 5. Extend closed canonical decode/readback for `Closure` **only as far as store
>    adoption and independent recovery need**. Malformed/unsupported images stay
>    **fail-closed**. ⚠ **If cyclic closure graphs are constructible, STOP and
>    route** — the bottom-up algorithm then needs an **explicit cycle contract**,
>    not accidental recursion.
> 6. `HostResult` and `BorrowedOpaque` are **different**: invocation-owned
>    represented arms. Persistent-store adoption must **continue to reject**
>    them, governed by their invocation-arena transfer/escape paths. ⛔ **Never
>    place either in the permanent store.**
>
> #### Required discriminators on the fresh descendant
>
> - emitted-created `Closure` with ordered captures → trusted adoption →
>   producer arena dropped → a **separately compiled** consumer recovers
>   `PersistentClosure`, **non-null** store identity, the same **artifact-scoped**
>   code identity, and capture **content and order**;
> - independent **equal** closures converge to one canonical `SlotId`/word;
> - different code identity, capture value, or capture **order** does not alias;
> - a **nested** `Closure` capture adopts bottom-up **without** retagging as
>   ground;
> - any reachable **invocation-owned** capture **rejects before** parent
>   publication;
> - canonical `Closure` decode **round-trips**, while malformed/unsupported
>   images **reject**;
> - emitted code **still cannot write `NODE_SLOT`**, and **no** B2F
>   dispatch/body-emission surface changes;
> - plus Ruling A's set: construct → seal → adopt → a separately compiled
>   consumer recovers real non-null identity and content **after the producer
>   arena is gone**; the equal/unequal adoption differential; unadopted publish
>   **fails closed**; emitted `NODE_SLOT` assignment stays **impossible while the
>   store-owned adoption path is positively exercised**; and owner/identity
>   **iff** for every admitted handle graph.
>
> ⛔ **CORRECTED 2026-07-26 — "ground adoption at `fe7d8a08` may carry
> unchanged" IS FALSIFIED. Do not rely on it.**
>
> That sentence was the Architect's, transcribed by the Steward in good faith,
> and **measurement overturned it within the hour.** `fe7d8a08`'s ground
> adoption recursed over `Constructor`/`Record` children **with no cycle
> guard**, and a reachable emitted input **hung it**. ⭐ **Mutation `M42` is the
> sharpest control on this node: remove the guard and it does not redden — it
> stack-overflows and aborts the test binary.** The defect is demonstrated, not
> argued. Repaired at `9b254fb9` (in-progress node-index set, exact fail-closed
> status at any cycle length).
>
> ⚠ **Read the shape of this, not just the fact.** An `AC-6` ruling carved out
> ground adoption as settled; the very next fold found a live defect inside the
> carve-out. ⛔ **A scope exclusion in a ruling is a statement about what the
> ruling ADDRESSES, never a warrant that the excluded region is correct.**
>
> ✅ Still carrying, and **not** touched by this correction: the classifier, the
> closed partition, `AC-1`, `AC-3`, and the three block-#4 repairs.

> ### ✅ RULED — THE CYCLE CONTRACT, `evt_5pzxf6sm4z08`, 2026-07-26
>
> **CYCLIC PERSISTENT GRAPHS ARE MALFORMED/UNREPRESENTABLE IN B2V.** They must
> **fail closed before canonical-store publication or boundary escape.**
> `9b254fb9` has the **correct direction** but stays **preservation evidence,
> not an acceptance candidate**.
>
> ⭐ **The reason is the REPRESENTATION CONTRACT, not the guard's behaviour.**
> Ken's persistent `Value` is a **finite, well-founded recursive value**;
> `Constructor`/`Record` children and `Closure` captures are encoded **inline as
> full canonical `Value`s**, and store identity is the **hash + memcmp of that
> finite canonical byte image**. ⛔ **A back-edge has no finite `Value` and no
> canonical byte image under this contract.**
>
> ⛔ **Emitted mutators can construct a STAGING GRAPH THAT IS INVALID, and that
> ability does NOT enlarge the admitted represented-value domain.** Admitting
> cycles would require new graph/SCC/fixed-point identity semantics and a
> different canonical encoding — **out of B2V, and not implied by `AC-10`.**
>
> #### Required mechanism on the fresh descendant
>
> 1. Adoption begins **only after an exclusive seal/quiescence handoff**. The
>    published counts and reachable fields must be **one stable snapshot**;
>    emitted writers must no longer be able to mutate it.
> 2. Validate the complete reachable graph with an **iterative tri-colour /
>    worklist traversal**, scoped to the adopted persistent image. A **back-edge
>    to grey** fails with the stable malformed-shape status; a **repeated edge to
>    black is a legal shared DAG** and reuses the canonical child.
> 3. ⛔ **Do not use host recursion as the totality mechanism.** Every finite
>    acyclic graph admitted by the region bounds **remains admitted**; a deep
>    chain may **not** stack-overflow and may **not** be reclassified as
>    malformed.
> 4. **Complete validation precedes** canonical root publication / identity
>    installation. Then canonicalize in **postorder**, preserving each child's
>    own tag/word and the existing **store-only** slot-mint authority.
>    Invocation-owned reachable children still reject before parent publication.
> 5. ⚠ **The node-index key is sufficient ONLY because one adoption walk is
>    scoped to one persistent image.** If the walk can span images, the key
>    **must include image identity.**
>
> #### Required discriminators
>
> Direct **and multi-node** cycles reject deterministically · the **same-shape
> acyclic** graph and a **shared-child DAG** adopt · a **deep acyclic chain
> beyond the former recursive margin** adopts **without host-stack growth** · a
> **write after sealing** fails/no-ops with the exact state verdict and **cannot
> change the adopted image** · nested canonical children **retain their actual
> tag**.
>
> ⛔ **`Closure` work may resume only after this transcription AND the
> already-planned fresh-seat gate.** QA stays held for a fresh descendant
> closing `Closure` **plus** this cycle/depth/seal contract.
>
> ---
>
> ⭐ **How this ruling was reached matters — the precondition came back TRUE.**
> `ken_boundary_store_field_local` refuses only a *persistent parent with an
> invocation-owned child*, so emitted code can allocate two persistent nodes and
> write each as the other's child — and **both writes return `OK`** through
> bounds, tag, frozen-prefix and escape.
>
> ⭐ **The ring refused to infer the answer from the guard's behaviour, and that
> refusal is why the ruling is sound.** `AC-10` forbids rejecting an admitted
> well-formed represented value — so *"the guard rejects it"* was **the thing
> being ruled on, never evidence for the ruling.** ⛔ **Preserve that move:** a
> mechanism's behaviour is not testimony about whether that behaviour is
> correct.
>
> ⭐ **And the reusable lesson is the implementer's, sharper than the one I first
> wrote here** (`evt_3zmx1wa2qk1zw`): *"a precondition attached to NEW work is
> worth asking of the work you ALREADY SHIPPED, because the two share the
> mechanism."* The cycle precondition was written for `Closure`; asking it
> literally is what exposed the unguarded recursion in **already-landed ground
> adoption**. ⛔ **Do not narrow a precondition to the deliverable that
> occasioned it.**

> ### ✅ THE TWO RESIDUALS FROM `9b254fb9` — AC OWNERSHIP RULED, AND THEY SPLIT
>
> Reported by the implementer rather than discovered later. ⛔ **Neither is
> discharged**, and the Architect ruled they belong to **DIFFERENT `AC`s** —
> ⭐ **I had guessed both were `AC-10`, and that was wrong.**
>
> 1. **Unbounded depth ⇒ an `AC-10` DOMAIN-TOTALITY face.** `adopt` is
>    cycle-safe but has **no depth bound**; a deep *acyclic* aggregate can
>    overflow and the margin is unmeasured. ⛔ **Finite deep acyclic values are
>    ADMITTED, so crashing OR rejecting them is forbidden.** Cycle-safety and
>    depth-safety are different properties; the guard closes only the first.
> 2. **Adoption against a live writer ⇒ an `AC-6` OWNERSHIP-TRANSFER face.**
>    Store adoption requires an **exclusive sealed handoff** before it may
>    absorb counts, validate, mint identity, or publish. ⛔ **Rust's `&mut` is
>    not by itself a proof** once emitted code retains a raw region base.
>
> ⭐ **Why the split matters more than the labels.** Had both been filed under
> `AC-10`, the ownership-transfer face would have been discharged by a
> *totality* control that never exercises a concurrent writer — a green row
> covering the wrong question. **Raising them as questions instead of folding
> them into the nearest `AC` is what produced the split.**

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
>
> #### ⛔ MANDATORY ROW added 2026-07-26 — `AC-1` **layout closure**
>
> The map's `AC-1` row must name the control discharging the **layout-closure
> clause** (Architect `evt_1tdq9g139snay`), not only tag/carrier closure. That
> control **reddens when the field inventory, the published word count, the
> declared extent, or an emitted offset drifts independently.**
> ⛔ **A row citing a constant-vs-constant equality assertion does NOT discharge
> it** — that is two hand-maintained authorities agreeing, which is the defect.
> ⚠ `fd4e7f08`'s map was complete and honest **and had no such row**, because
> the clause did not exist yet; that is precisely how a 136-vs-144 mismatch
> passed a full `AC`→control review.

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
  `#11`**, armed. ⛔ **`#10` is recorded under symptom-inventory entry 2 / the
  prerequisite chain — it is NOT a fourth entry.** Inventory stays at 3 entries;
  next predicate check at the **6th**.
  > ⚠ **`#11`, not `#12` — corrected by the Steward 2026-07-25.** Three tracked
  > files carried `#12` (the generic next-multiple-of-3 after the consumed `#9`);
  > the steward playbook carries an **operator override** of *"catch-up set to
  > `#11`, then `#15`, `#18`, `#21`"* (2026-07-24). The two cannot be reconciled
  > from the dates, so it is settled by dominance rather than by guess: a pull at
  > `#11` is **required** under one reading and merely **early** under the other,
  > and early is explicitly fine — a cadence threshold is a floor on when to ask,
  > not a bar on asking sooner. `#12` is wrong under one reading, so `#11` wins.
- ### ⛔ ARMED — consecutive Architect PRODUCTION blocks on this node
  ```text
  CONSECUTIVE ARCHITECT PRODUCTION BLOCKS = 6
    #1 78a57d90  #2 657f60a0  #3 ddff2fae  #4 fd4e7f08 (dec_7sd3enk81maws
                                              REJECTED on the object, evt_4bs6scfmt5ax0)
    #5 81a68435 (evt_8851dkes0wmh)  #6 fe7d8a08 (evt_3cw3qtmxbvmc3)
  PREDICATE CHECK AT 3 -> FIRED 2026-07-25, ANSWERED YES -> RECUT 1.
  PREDICATE CHECK AT 6 -> FIRED 2026-07-26, ANSWERED YES -> RECUT 2.
  NEXT CHECK = block #9 on this node.
  ```
  ✅ **THE `#6` CHECK IS ANSWERED: YES, one predicate over ALL SIX BLOCKS**
  (`evt_17v000g4gmppp`) — **representation authority-to-execution closure**. See
  **RECUT 2**. ⛔ **No B2V acceptance candidate may bind on the old per-cell
  proof shape.**

  ⭐ **Both checks this counter has fired have come back YES.** That is the
  argument for the counter existing: `§5a-ii` counts **hard-stops**, and a
  review block is correctly **not** a hard-stop — so six Architect production
  blocks moved neither the hard-stop count nor the symptom inventory, and every
  armed line in the repo read correct and current throughout. ⛔ **A backstop
  that depends on someone remembering to look is not operative**, which is why
  it lives here as a counted line the next block must walk past.

  ⚠ **On the two readings I recorded at `#6` and deliberately did not choose:**
  I framed them as *convergence* versus *a predicate the frame fails to name*,
  and noted `#5`/`#6` were narrower than `#1`–`#4`. **The Architect answered
  that this is NOT merely `#5`/`#6` convergence** — the predicate reaches back
  to `#1`. ⛔ **The narrowing I observed was real and was not the point**;
  had the Steward concluded from it, the answer would have been wrong.
  `COORDINATION §5a-ii` reserves naming the predicate to the Architect, and this
  is the case that shows why.
  ⚠ **Block #4 arrived on a candidate QA had APPROVED with a complete
  AC→control map and a passing mutation proof.** Read that before treating a
  green QA map as coverage: the map was honest and its residual accounting was
  honest, and three production defects still sat outside it.

  ⭐ **All three of block #4's defects have one shape — the Rust side states the
  law and the emitted side does not enforce it.** (1) `BOUNDARY_REGION_HEADER_BYTES`
  declares a layout no consumer checks, against an 18-word publish; (2) the
  emitted `store_int_limbs` admits `len=0` / leading-zero / negative-zero
  magnitudes that `RuntimeIntV1::canonical_sign_and_limbs` forbids; (3) the
  emitted reader wraps `at + region_len` where the Rust oracle uses
  `checked_add`. ⛔ **That is this node's own founding diagnosis one layer down**
  — `B2V` exists because the aggregate-result path was *a Rust-side decode, not a
  value representation*.

  ✅ **RULED 2026-07-26 (`evt_1tdq9g139snay`), and the boundary is the point.**
  The recut predicate reaches **(2)** and **(3)** — construct a *valid* word,
  keep it *resolvable*. **(1) is NOT an `AC-10` face**: the header constant had
  no consumer and the published vector was large enough for every accessed
  field, so the round trip was never falsified. It is an **`AC-1` layout-closure
  face** and is folded there. ⛔ **Widening `AC-10` to absorb it would have
  destroyed the named predicate's boundary** — the reason to raise an uncovered
  face as a *question* rather than patch it into the nearest `AC`.
  ⭐ **This counter exists because §5a-ii's could not see this.** §5a-ii counts
  **hard-stops**, and a review block is correctly **not** a hard-stop — so three
  Architect production blocks moved neither the hard-stop count nor the symptom
  inventory, and the chain could have gone arbitrarily deep with every armed line
  in the repo reading correct and current. The trigger fired only because the
  Steward had armed it by hand in a watchdog payload. ⛔ **A backstop that depends
  on someone remembering to look is not operative** — so it now lives here, at the
  point of work, as a counted line the next block has to walk past.
- Read `agent/playbooks/tools/pin-a-property.md` before writing any assertion.

## ⛔⛔⛔ RECUT 2 — 2026-07-26. A predicate over ALL SIX BLOCKS. `evt_17v000g4gmppp`

> ### ⛔ NO B2V ACCEPTANCE CANDIDATE MAY BIND ON THE OLD PER-CELL PROOF SHAPE.
>
> That is the gate. Everything below explains it. **The `#6` §5a-ii check fired
> and the Architect answered YES** — blocks `#1`–`#6` share one predicate, and
> ⛔ **this is NOT merely `#5`/`#6` convergence**, which was the reading I
> recorded as live and correctly did not choose.

### The predicate — the Architect's words, `evt_17v000g4gmppp`

**Representation authority-to-execution closure:**

> Every B2V representation authority — layout inventory, static policy, value
> partition, canonical form, owner/lifetime/identity rule — must be the **sole
> authority actually consumed by the production path it governs**; and every
> admitted partition must have **one total executable lifecycle** from emitted
> construction through validation/sealing/adoption/publication to separately
> compiled recovery. ⛔ **A declaration, classifier row, Rust oracle, or residual
> label with no production consumer does not discharge the predicate.**

### The six blocks are successive counterexamples — one predicate, six faces

| # | SHA | the authority that had no closed execution |
|---|---|---|
| 1 | `78a57d90` | persistent **policy** existed; stable persistent resolution and emitted handle construction did not |
| 2 | `657f60a0` | content, store-mint authority and tag×class legality were **declared but not enforced** by the emitted/store path |
| 3 | `ddff2fae` | Big-`Int` spill and immediate validity were **declared** but lacked total executable encodings/checks |
| 4 | `fd4e7f08` | one **layout authority had no consumer**, and the Rust canonicality/non-wrapping laws **were not the emitted laws** |
| 5 | `81a68435` | `PersistentStore` was an **outcome without the adoption/mint lifecycle** that makes it true |
| 6 | `fe7d8a08` | `PersistentClosure` was **admitted without a canonical image/adoption arm** |

⇒ ⛔ **The frame still permits A TABLE TO CLOSE BEFORE ITS PRODUCTION MECHANISM
CLOSES.** That sentence is the defect. Every one of the six was a correct local
ruling on a real cell, and none of them reached it.

### The subsuming repair

**One mechanically closed artifact** over the **finite structural partition**,
spanning every phase:

```text
authority -> producer -> validator -> canonicalizer/adopter -> publisher -> consumer
```

- Every admitted row must **name or derive ALL phases**.
- A missing consumer, canonical image, lifecycle phase, or authority use must be
  a **construction/compile failure** or a **named causal red control**.
- ⛔⛔ **A HAND-MAINTAINED MATRIX THAT CAN DRIFT FROM THE PRODUCTION ENUMS IS NOT
  ENOUGH.** This is the clause that retires the per-cell `AC`→control map as a
  *sufficient* proof shape. The map stays required; it stops being the proof.

> ⭐ **This is the `fd4e7f08` lesson stated as law.** That candidate shipped a
> map that was complete, honest, and mutation-proved, with `ken-runtime` at
> 398/0 — and three production defects sat outside it **because no `AC` asked
> the closure question, so no row was missing.** A matrix cannot report a cell
> it does not have. Derivation from the production enums can.

### ⚠ What RECUT 2 does NOT do

- ⛔ **It does not undo the six local rulings.** Each stands. It names the
  structural proof they are instances of.
- ⛔ **It adds no new constraint to the in-flight fold**, and **does not stop the
  ring** — they continue under the Steward's standing instruction.
- ⛔ **It is not a verdict on the ring.** Six blocks landed on work that was
  honestly reported and correctly escalated every time; the counter measures the
  frame's proof shape, not the ring's competence.

### ✅ Transcription verified by the Architect

`0525a206`/blob `3266f280` faithfully transcribed rulings A and B; `37f67afe`/
blob `43efb676` preserves them **and correctly retracts the falsified "ground
adoption may carry unchanged" carve-out.** ⭐ **The Architect checked the
Steward's transcription against what they meant, because I asked them to** —
a frame is what the implementer obeys, and a faithful-looking transcription is
not self-verifying.

### ⛔⛔⛔ RULING R3 — 2026-07-26. `RECUT 2` vs `D6`: WIRING IS IN SCOPE AND REQUIRED

**`dec_r09576dypk6e`** · Architect `evt_7nkbf495pg54h` · verified `resolved` from
the object, `resolved_by` = the Architect's actor, `resolved_at`
`2026-07-26T08:29:38Z`. Raised by `runtime-implementer` `evt_387scrzz83p0b`,
escalated by `runtime-leader` `evt_55dsdwygrb4r6`, routed by the Steward
`evt_5t8pd2gf6kgtq`.

⛔ **This is transcribed here because an in-thread ruling is not a durable
deliverable.** The frame is what the implementer obeys.

**What was asked.** `RECUT 2` rejects an authority with no production consumer;
`D6` requires the node to remain **INERT**. Measured on the non-`cfg(test)` lib
build, `emit_boundary_value_local_graph` is production-reached while
**every** type in the classification layer reported *never used* — so the emitted
path ran in production without consulting the disposition governing it. The ring
declined to resolve it on its own reading.

**The ruling — the Architect's words, `evt_7nkbf495pg54h`:**

> `RECUT 2` and `D6` govern **different boundaries**:
>
> - `RECUT 2` requires the fixed boundary-helper artifact to be **generated from
>   the sole representation authority actually governing it**.
> - `D6` keeps that artifact **inert at the semantic call graph**: no generated
>   semantic-origin function, no semantic-body call to a boundary helper, no
>   cross-owner call, no switch-over, no second body emitter, and no
>   helper-population/census change.
>
> So production **codegen consumption is not `B2F` activation.**
> `emit_boundary_value_local_graph` is already the non-test production codegen path
> for the fixed helper graph. **`B2V` must make that path causally consume the
> representation authority**; `B2F` later performs only the switch-over that makes
> semantic bodies call the already-emitted helpers.

⭐ **That distinction — *codegen* consumption vs *semantic-call-graph* activation —
is the whole ruling.** `D6`'s inertness was never about whether production code
consults the authority; it is about whether semantic bodies call the helpers. Both
clauses were satisfiable at once and the frame did not say so.

**Ruled mechanism seam (the Architect's, verbatim in effect):** derive **one
crate-private emission plan** directly from the exhaustive `LoweredVariant` static
policy and the finite `BoundaryInput → BoundaryOutcome → PhaseClosure` authority.
Compute and pass that plan **once** at the existing `lowering/core` →
`emit_boundary_value_local_graph` seam, or an equivalent **single-owner** route.
The emitter must use it to construct the helper bodies' legal
tag/class/owner/identity and runtime-partition behaviour.

⛔ **The following explicitly DO NOT COUNT:**

- duplicating the policy in `boundary_value_clif`;
- a `let _ = plan`, warning-suppression read, or **assertion-only** validation;
- **another hand-maintained table beside the helper bodies**;
- specializing from a JIT seed or a sampled runtime value.

⚠ Magnitude, reachability and adoption **remain runtime distinctions emitted from
the same authority.** The plan **may change helper-body contents**, but the fixed
Θ(1) helper set, the semantic generated-function population, the calls, the
ownership topology, and **every `D6` census** remain unchanged. ⛔ **If any of those
move, stop and route.**

⛔ **Required evidence is CAUSAL, not structural:** mutate or bypass the authority
and the captured/emitted helper graph **must change or reject**; an emitter that
ignores the plan **must redden.**

⚠ **`5e6b0945` is a valid completeness checkpoint and is NOT an acceptance
candidate** until the authority-to-emitter edge is real. ⭐ The implementer had
already said exactly this about its own artifact before the ruling — that it was a
seventh declaration in the same consumer-less layer — and declined to present green
tests as closure.

**Mechanism latitude, as ruled:** the Architect fixed the **component seam**; the
**exact Rust carrier and CLIF spelling inside it are the ring's.**

### ⛔⛔⛔ RULING R4 — 2026-07-26. Immediate tag→class IN SCOPE; `R3`'s bar is PER-SITE

Architect `evt_51xk9sxqdtzgt`, answering **two** questions at once — the
implementer's standing non-blocking question (`define_class:715`) and its own
offer to sequence the class-axis repair separately. Raised by
`runtime-implementer` `evt_3kpwxwrs8bty0`; checkpoint record corrected by
`runtime-leader` `evt_1kedhmyapcvf7`.

⛔ **Transcribed here for the same reason `R3` was: an in-thread ruling is not a
durable deliverable, and the ring's next compaction will not carry the channel.**

**What was asked.** (1) `define_class`'s immediate result is computed by a
hand-written `is_bool ? Bool : Int` branch, because `ImmediateWord` carries no
class — **in scope, or a named residual?** (2) The implementer then *measured*
that the class axis has the same per-site hole as the tag axis, offered
`fed42481` as a clean boundary, and asked whether to sequence the repair
separately.

**The ruling — the Architect's words, `evt_51xk9sxqdtzgt`:**

> 1. `define_class`'s immediate result is observable helper behaviour governed by
>    `R3`'s sole representation authority. The current `is_bool ? Bool : Int`
>    branch is **a second hand-maintained mapping beside the helper body, so it
>    cannot remain as a named residual.** The exhaustive authority must carry
>    enough information to derive the returned class for **every admitted
>    `ImmediateWord`**, and the emission plan must deliver that projection to the
>    helper.
> 2. Keep that projection **separate from `BOUNDARY_TAG_CLASS_RELATION`.** That
>    relation governs node `NODE_CLASS` legality and **correctly** excludes
>    immediate tags because immediates have no node. The new datum is the uniform
>    `class` helper's **boundary-value classification**, not a fictional immediate
>    node class. ⛔ **Name that distinction in source and evidence so a later
>    reader cannot merge the two contracts.**
> 3. The measured one-site disconnect answers the evidence question: **`R3`
>    requires per-site causal coverage.** Four remaining consumers changing an
>    aggregate graph **cannot prove the fifth consumes authority.** The wiring at
>    `720f301c` was real, but **my earlier statement that the class axis was
>    closed is withdrawn**; it is only *structurally* wired pending the per-site
>    behavioural differentials. **Repair this now in the same `B2V` increment** —
>    this is the already-required acceptance evidence for one mechanism, **not a
>    separate WP and not a reason to preserve `fed42481` as a QA candidate.**

⭐ **Clause 1 is `R3`'s "another hand-maintained table beside the helper bodies"
clause reaching a case nobody had named.** The residual was not a table — it was
a two-arm `if` — and it still counts. ⇒ Read that `DO NOT COUNT` bullet as being
about **who maintains the mapping**, not about its shape or size.

⛔ **Clause 3 STRENGTHENS the `R3` evidence bar, and it is retroactive.** `R3`
says an emitter that ignores the plan must redden; the frame's generic pin
language (*"a named causal red control"*) permitted reading that as satisfied by
a whole-graph differential. It is not — the standing pin

```
recut2_the_emitted_helper_graph_changes_when_the_authority_changes
```

is an **aggregate** differential, and the implementer measured that disconnecting
**one** of five `class_guard` sites back to its literal leaves it at
**439 passed / 0 failed**. ⇒ ⛔ **Every consuming site needs its own behavioural
differential, or must be NAMED as probe-unreachable.** An aggregate differential
cannot answer a per-site question — it is green while a site is disconnected.

⚠ **`fed42481` is a HELD CHECKPOINT, NOT a QA candidate**, and — per
`evt_1kedhmyapcvf7` — it is **four commits past re-anchored `69750fa3`**, *not* an
ancestor-descendant continuation of `ab11a3d2`. `ab11a3d2` and `a7aa60eb` are
preserved independently on `origin`. ⛔ **A commit distance is not a
fast-forward:** `git merge-base --is-ancestor ab11a3d2 fed42481` exits **1**.
Verify any such claim against the operation it is protecting.

⭐ **Clause 3 is the Architect withdrawing its OWN confirmation on measured
evidence supplied by the ring it had confirmed to.** The wiring it read was
accurate; the pin standing behind it was weaker than either party treated it as.
⇒ A confirmed axis can carry an overclaim, and the seat best placed to find it is
the one that already proved the same shape elsewhere.

**Unchanged:** `D6`, the fixed helper population, the census set, and the
`RECUT 2` subsuming repair. **Ring latitude, as ruled:** an
`ImmediateWord { tag, value_class }` field **or** an equivalently **total**
derived tag→value-class relation is sound.

**Next move, as ruled:** Runtime Implementer completes the immediate-class
derivation **plus** per-site class evidence; Runtime Leader reviews the resulting
clean checkpoint **before** any QA routing.

## ⛔⛔ RECUT 1 — 2026-07-25. The Architect NAMED the shared predicate.

⛔ **Read this before reading the acceptance criteria above.** The `AC` set above
is the thing this recut acts on.

> ### ⛔ HOW THIS RECUT AMENDS — edit the OPERATIVE text, never append a correction
>
> **A later note saying an earlier deliverable is false does not replace the
> deliverable.** `D4` is the construction authority an implementer reads *first*;
> a clarification 300 lines below it is read *second, if at all*. Two consecutive
> Architect blocks on this recut were caused by exactly this — folding a
> correction in as a new paragraph while the contradicted `D`/`AC`/RETAIN text
> stayed operative and unedited. Both readings then live in the frame, and the
> **wrong one is the one positioned to be obeyed.**
>
> ⇒ Every fold in this recut **edits the operative deliverable in place**, and
> the clarification blocks below **explain** that text rather than contradicting
> it. ⛔ Before returning any folded ref, re-read `D1`–`D6`, `AC-1`–`AC-10`, and
> RETAIN **as a whole** and confirm none of them still states the superseded
> contract. A whole-frame reconcile is the fold, not a step after it.

### The predicate — the Architect's words, `evt_2zxt6m9bg43r2`

> For every boundary-reachable `Lowered` value that the exhaustive disposition
> admits, a generated producer must be able to construct a **valid, lossless,
> ownership-correct** fixed-width boundary word; that word must remain resolvable
> for its declared lifetime; and a separately compiled consumer must recover the
> same value/identity through the emitted ABI, while malformed or unrepresentable
> inputs fail **before** publication rather than truncate, alias, forge identity,
> or defer failure to projection.

> So these are **not independent defects**. They are successive exposed faces of
> one incomplete claim: **the admitted disposition is not yet closed under
> emitted producer → boundary word → separately compiled consumer round trip.**

⭐ **All three blocks were individually correct.** Local correctness of each is
precisely what made the shared predicate invisible — it is the symptom, not the
refutation. Likewise, *"the architecture is still viable"* is true here and is
**not** an answer to the predicate question; the answer above is.

### RETAIN — a named predicate is NOT a licence to restart

⛔ **Everything below is PROVED and stands. Do not rebuild it, do not re-review
it, do not let a recut become a restart.**

- the tag × class relation, closed over the whole product
- region selection / threshold agreement with referent owner
- the native exact-`Int` **normalization dependency** — the canonical
  sign/limb contract is authoritative wherever a word is built, including from
  emitted code
- **one derived layout** (a single authority computes node/header extents; no
  second hand-maintained copy) and a **distinct content table**
- ⛔ **NOT the byte counts.** The earlier "64/112 layout change" is
  **superseded** and is *not* retained. The recut promotes persistent
  arbitrary-precision `Int` into scope, and region-owned magnitude storage
  necessarily widens node/header extents; **a sound persistent-wide-`Int`
  representation cannot both satisfy that promoted `AC-10` obligation and hold
  64/112 unchanged.** (Nor is the successor "80/136" retained: the exact-SHA
  review of `fd4e7f08` found the declared 136 was actually a 144-byte publish
  with no consumer — which is why the *pin*, not the *number*, is the retained
  property.)
- ⭐ **A reviewed layout delta required by an `AC-10` outcome is PREDICATE DELTA,
  NOT RESTART.** Do not read a changed byte count as the recut having reopened
  proved architecture; read it as the promoted obligation doing exactly what it
  was promoted to do.
- removal of the caller-supplied store-identity writer
- the emitted String/Bytes reachability controls and their causal mutation
  (`M14`, exact `BOUNDARY_ERR_CLASS = -4`)
- the **sealed exhaustive / no-wildcard disposition MECHANISM**, and every
  already-proved **classification outside `AC-10`'s implicated domain**.
  ⛔ **Not the classifications wholesale.** A classification or value-band that is
  **narrowed or reclassified in order to close `AC-10`** is reviewed as **the
  predicate delta**: it is neither prohibited nor a restart. `Constructor` and
  `HostResult` remain **required live represented arms**.
  > ⚠ **This item read *"the exhaustive `Lowered` disposition"* wholesale in the
  > first draft — self-contradictory**, because the same recut names a *narrowed
  > disposition* as a permissible mechanism, and narrowing necessarily changes at
  > least one current classification. A RETAIN list that forbids the mechanism the
  > frame leaves open is a trap for the implementer, not a protection.

The last clean checkpoint carries forward as a **semantic oracle, not an
acceptance path** — it tells you what the answer looks like; it does not
discharge anything.

### REPLACE — only what the predicate names

**What is defective is the SHAPE of the `AC` set, not any single `AC`.** `AC-1`,
`AC-2`, `AC-4`, `AC-6` and `AC-7` each pin **one facet** of the round trip. Each
block found a facet no `AC` had named. Enumerated, that yields an unbounded chain
of individually-reasonable blocks; **named, it yields one closure.**

> ### ⛔ `D4` / `AC-3` CLARIFICATION — static POLICY vs runtime OUTCOME
>
> The exhaustive no-wildcard match assigns every `Lowered` **variant** exactly one
> **static disposition policy**. A *represented* policy declares a **closed
> encoding policy**: **immediate-only**, **handle-only**, or
> **immediate-with-declared-handle-spill**. The existing
> `RepresentedImmediate { spill: Some(…) }` is the **third** of those — ⛔ **not a
> claim that every runtime value of that variant is immediate.** `ProtocolOnly`
> and `FailClosedForbidden` remain terminal static policies.
>
> ⚠ **Why this had to be said.** `Lowered::Int` maps to
> `RepresentedImmediate { tag: ImmediateInt, spill: Some(Int) }` — **one static,
> variant-level policy**. A small runtime `Int` yields an immediate word; a wide
> one yields a **persistent handle**. `D4`'s old wording defined *represented
> immediate* as *"payload fits the tagged word directly"*, which is **false for
> that same arm whenever its declared spill fires**. ⛔ Calling the whole `Int`
> population *immediate* lets a proof attach handle evidence to **one sampled
> spill** while never establishing that **every** spill partition carries the
> handle obligations. Forcing the value-level `AC` to say *handle* instead would
> contradict `AC-3`'s one static disposition per variant. **Both levels are real;
> neither may absorb the other.**

**`AC-10` — total classified-domain closure.** The exhaustive disposition assigns
every boundary-reachable **variant** one **static policy**, and the closed
value-dependent partition assigns every boundary **input** exactly one **actual
outcome entailed by that policy**: *immediate word*, *handle word with declared
class/owner/lifetime*, *protocol-only*, or *fail-closed forbidden*.

1. Every **well-formed value admitted by a represented policy** is constructible
   by emitted code and recovered by a **separately compiled** consumer with
   content/value, identity, owner, and lifetime intact.
2. A **protocol-only** case cannot enter a source-valued slot.
3. A **malformed, forbidden, or unrepresentable** input rejects **before
   emission/publication** with its exact status.

⛔ **No admitted well-formed represented value may reject, and no input *or
encoding outcome* may remain unclassified.**

> ### ⛔ Why `AC-10` is phrased this way — the Steward got it wrong first
>
> My first draft read *"for every value the disposition admits, **either** round
> trip **or** fail closed."* The Architect blocked it, correctly: that puts the
> failure arm **inside the admitted subset**, which makes admission
> **non-semantic** and is **satisfied vacuously by an implementation that rejects
> every represented value.** ⭐ **Classification happens first; the behavior is
> then *entailed by the class*.** The predicate's failure arm is load-bearing —
> but it belongs to the *unrepresentable* class, not to the represented one.
>
> ⚠ **The instructive part:** I was guarding against the opposite error — the day
> before, I over-strengthened a correct mechanism into a post-condition that
> failed on correct work. Steering away from *too strong*, I landed on *vacuously
> satisfiable*. **Both failures come from writing the predicate on the wrong
> domain**, and neither is fixed by tuning strength. Fix the domain first.

⚠ **Three further things about `AC-10`:**

- ⛔ **It quantifies over the DISPOSITION, not over tags.** The predicate is
  explicitly *stronger than "all tags are enumerated"* and *stronger than
  Rust-side materialization*. A sweep over 21 arms is not a sweep over admitted
  values: magnitude bands and lifetime bands are part of the domain. **Total over
  nodes is not closed under parent → child reachability.**
- ⛔ **"One control total over every value" is NOT an executable oracle** — the
  admitted domains include unbounded integers, byte/string contents,
  lifetime/ownership states, and recursive parent → child reachability. **No
  finite runtime test enumerates them.** Demanding one would produce either an
  impossibility or *a finite case sweep wearing a universal name*, which is worse
  than an honest sweep because it reads as total. **Totality is therefore proved
  STRUCTURALLY:**
  > a sealed exhaustive no-wildcard disposition, **plus** a closed finite
  > partition of every value-dependent representation discriminator — at least
  > **variant**, **magnitude/shape**, **lifetime/owner**, and **parent → child
  > reachability / aggregate recursion**. Every value-dependent partition maps to
  > exactly one **actual outcome permitted by its static policy**. ⛔ **A handle
  > outcome — including an immediate policy's SPILL ARM — must discharge the
  > handle class, referent-owner, identity, and lifetime obligations.** Each
  > partition **boundary** carries a **nondegenerate witness pair** and a
  > **causal mutation** driven through an emitted producer and a separately
  > compiled consumer.

  ⭐ **The demanded unit is one property / one `AC` — not one test function.**
  QA's `AC-10` row names the structural closure artifact **and the complete
  control family**; it need not pretend one dynamic test enumerates an infinite
  domain.
- ⛔ **`AC-10` requires an `AC` → control row like every other.** An `AC` that
  ships **zero** controls is invisible to a review that examines controls:
  *discharged* and *never asked* read identically in a green verdict.

**The three `NO CONTROL — open residual` rows are PROMOTED into `AC-10`'s scope.**
QA recorded them honestly and they were correct to record; the Architect's third
block says the first of them *"is not an optional test residual."* They are not
residuals — **they are the predicate's uncovered faces**, and they are the recut:

| promoted residual | why it is in scope |
|---|---|
| `AC-4` Big at the persistent boundary | the disposition promises a `PersistentGround` spill for every `Int`; only `Small` can materialize |
| `AC-1` tag reachability | review-only reachability is not a sweep over the admitted domain |
| `AC-6` persistent content addressing | the emitted node stays `NULL_SLOT`, so identity recovery is unproved |

⛔ **The `AC-6` row above is RULED, not open — 2026-07-26.** Two Architect
rulings (`evt_8851dkes0wmh`, `evt_3cw3qtmxbvmc3`) bind the mechanism that closes
it, and they are folded into the **operative `AC-6` text above**, which is where
you must read them. ⚠ **Do not discharge this row from the sentence in this
table** — the table states the *gap*; the `AC-6` block states the *obligation*,
and the obligation is strictly larger. Two closed candidates (`81a68435`,
`fe7d8a08`) already read as satisfying the table row while failing the ruling.

### FREEZE

The enumerated-`AC` chain is **closed at three production blocks**. The counter
in *Standing* opens fresh. ⛔ The three blocked candidates (`78a57d90`,
`657f60a0`, `ddff2fae`) stay reachable on origin as durable checkpoints and are
**never publishable**.

### ⚠ What this recut does NOT do

⛔ **It does not stop the Runtime ring, and it adds no new constraint to the fold
in flight** — the Architect said so explicitly, and I am taking that at face
value. @runtime-implementer keeps folding.

⛔ **It does not choose a mechanism.** *How* the disposition is closed — a real
persistent `Big` representation, a narrowed disposition, or a grounded
frame-boundary impossibility — is the **Architect's** call, not the frame's. This
frame states the property and where the control must be total; it does not pick
the implementation, and the Steward must not.
