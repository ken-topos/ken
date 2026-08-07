# RT-CARRIER-BYTESPAN-OBSERVE — a total emitted byte-span observer over the carrier

**Every `BytesPointerLength` seat in the host-effect seat table is
`SPECIALIZED_ONLY`, and the carrier's emitted helper set has no way to read a
byte span's extent at all. So a byte-carrying value that arrives at an effect
seat as a carried boundary word is refused, per seat, at the `Need ⊆ Avail`
membership test. This node builds the missing observer and then flips the phase
for exactly the seats it has proved, one seat at a time.**

**Owner:** Team Runtime. **Size:** L.
**Node:** `docs/program/issues/RT-CARRIER-BYTESPAN-OBSERVE.md`.
**Risk:** medium-high. It adds a helper to a closed, pinned emitted inventory,
and its last deliverable is an activation whose gate this node inherits and did
not create.

**Authority:** Architect capability disposition `evt_4c26q24rp7xqb`; the
attribution in `evt_2qzwanx82m06r`; Steward ruling `evt_3pr04vk7zrd7c`, which
recut `AC-1` clause 1 out of `RT-CONTSRC-PRODUCER-LOCAL` into this node.

---

## 0. The base is now fixed

This frame's one open input was the governing base, which turned on the
operator's gate-readiness decision. **That decision landed on 2026-08-06: land
the candidate, with the five failing rows marked skipped and restored as work
allows.** Section 1a is settled accordingly and nothing in this frame is
outstanding.

## 1. Base and fixed inputs

### 1a. The governing base is `main`

**SETTLED 2026-08-07. Cut `wp/RT-CARRIER-BYTESPAN-OBSERVE-<slug>` from
`origin/main` at `b0a0a20c`.** `RT-CONTSRC-PRODUCER-LOCAL` has merged, along
with `RT-DECL-CLOSURE-PORT` and `RT-SRCBODY-BIND-ORDER`; all three nodes are
`merged`.

**Do not continue `wp/RT-DECL-CLOSURE-PORT-typed-units`.** The publisher
**squashes**, so neither `b914c7ff` nor `acfcc915` is an ancestor of `main`;
continuing that branch would re-offer already-merged commits. A branch that
dangles ahead of `main` with its content landed is the normal post-squash state
and is not unmerged work.

**The 1b anchors have been re-verified against `b0a0a20c` for you** — three
moved and three did not, and the table below now carries the current blobs. The
squash preserves content, not commit identity, so a moved blob here means the
file genuinely changed, not that the anchor drifted.

### 1a-i. THIRTY of this node's rows are SKIPPED, not four

**CORRECTED 2026-08-07, and the correction is nearly an order of magnitude.**
This section previously said four. Measured at `b0a0a20c` with the anchored
form `^[[:space:]]*#\[ignore` — which excludes doc-comment lines that merely
mention the attribute — **this node's id owns 30 `#[ignore]` attributes across
10 files:**

| file | rows |
|---|---|
| `crates/ken-verify/src/scenario.rs` | 9 |
| `crates/ken-cli/tests/rt_parity_native.rs` | 5 |
| `crates/ken-cli/tests/px4b_native_production.rs` | 4 |
| `crates/ken-cli/tests/rt_escape_second_resource_native.rs` | 2 |
| `crates/ken-cli/tests/px8ta_oriented_subcontinuation.rs` | 2 |
| `crates/ken-cli/tests/px7m_hostresult_computational_match.rs` | 2 |
| `crates/ken-cli/tests/px7l_checked_host_recursive_bind.rs` | 2 |
| `crates/ken-cli/tests/px7f_resource_native.rs` | 2 |
| `crates/ken-cli/tests/px8x_single_schema_observation.rs` | 1 |
| `crates/ken-cli/tests/px7p_constructor_field_composition.rs` | 1 |

**A skipped row measures nothing**, so at your `D0` the suite reports these
ignored rather than failed.

⇒ **`D0` must un-skip all 30 and record their live failure**, and `AC-1` is
discharged against that live baseline. **Removing all 30 `#[ignore]`
attributes is this node's deliverable**, and a green suite that still carries
any of them has discharged nothing for those rows.

**Do not read 30 rows as 30 units of work.** They are one mechanism — an
observer plus a helper plus per-seat phase flips — and the rows discharge
together as seats flip. What the count changes is the **verification surface**
and the sizing of `D0`, not the design. If it turns out the rows do not all
discharge from one mechanism, that is a finding and a recut, and the recut is
the Steward's.

**The former "fifth row stays ignored, it belongs to `RT-ENTRY-TRAP-254`"
clause is DELETED because it is now false.** `RT-SRCBODY-BIND-ORDER` `D4`
**un-ignored** that row in the commit that greened it: the entry trap's root
cause was the source-body binding order, not a byte-span gap.
[[RT-ENTRY-TRAP-254]] is `closed`, `superseded_by: RT-SRCBODY-BIND-ORDER`.
**px4b now carries exactly four ignores and all four are yours.**

### 1b. Content anchors at `b0a0a20c`

Measured 2026-08-07 against `origin/main`. The `since b914c7ff` column says
whether the merges above changed the file — it is there so you know which reads
are still good and which are fresh.

| path | blob at `b0a0a20c` | since `b914c7ff` |
|---|---|---|
| `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs` | `7f4fa3c376be34402c4815e0706b896f4363e66d` | same |
| `crates/ken-runtime/src/boundary_value_clif.rs` | `0ec07c6698aca67eb51084ecb4ab376efa5a6ed0` | same |
| `crates/ken-runtime/src/boundary_value.rs` | `ac0745763b2c71c07bb5205fad5edaa3a3718e17` | same |
| `crates/ken-runtime/src/cranelift_backend/lowering/mod.rs` | `745f1cda7addbd447f2bbc9b1a2c8095b8bde768` | **MOVED** |
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `275bc0f9a0a0b006c86fe4abb092bfd16ebe1328` | **MOVED** |
| `crates/ken-cli/tests/rt_parity_native.rs` | `242a88fad14d90b944b6fa4ad76b5a0d0301e01c` | **MOVED** |

**The three substrate files this node builds on — `static_transition.rs`,
`boundary_value_clif.rs`, `boundary_value.rs` — are byte-identical to what
section 2e was written against.** The audit in 2e therefore still holds as
written; `BOUNDARY_LOCAL_HELPERS` has not moved.

**`rt_parity_native.rs` moved because its six `assert_narrowed_alike` rows were
`#[ignore]`d, five of them under this node's id.** That is the annotation
described in 1a-i, not a change to the rows' content.

### 1c. Expected baselines, to be re-measured at `D0` and not inherited

**RESTATED 2026-08-07 against `b0a0a20c`. The previous numbers were measured
before the quarantine and before the bind-order fix, and both moved.**

- `px4b_native_production`: **19 tests total; expect 15 passed / 0 failed / 4
  ignored.** The four ignored are this node's. The old reading — 14 passed / 5
  failed — predates both the annotation and `RT-SRCBODY-BIND-ORDER` `D4`, which
  greened the fifth (entry-trap) row and un-ignored it.
- `ken-runtime --lib`, targeted: **expect 783 passed / 0 failed / 3 ignored.**
  The old reading — 778 passed / 2 failed / 1 ignored — is superseded: the two
  standing reds are now `#[ignore]`d under [[RT-WORKER-FIXTURE-DECODE]] and
  [[RT-CARRIER-PRODUCER-OCCURRENCE]]. **Neither is yours**, and neither is an
  assertion failure — both die at an `expect` before reaching any assertion.
  Leave them ignored.

**These are expectations to falsify at `D0`, not inputs to inherit.** I derived
them from the annotations at `b0a0a20c`, not from a run.

**Assert the executed count, not just the exit status.** A filter that matched
nothing reports green and is indistinguishable from a pass. `0 passed` is a
failed measurement.

### 1d. Every anchor is perishable

**Treat every fixed input in this frame as perishable. If one turns out false
against the landed code, say so and escalate — do not quietly build around it.**
This clause is not boilerplate; on this campaign it is the mechanism that has
caught the most frame defects, including several of the Steward's own.

## 2. The gap, exactly

### 2a. The attribution, quoted rather than paraphrased

**Architect `evt_2qzwanx82m06r`:**

> The reachable `Constructor` predecessor of AC-1 match origin `268` projects
> a carried child into effect origin `264`; that child's
> `BytesPointerLength` seat lacks carried availability.

**It is not "HostResult payloads require this."** The probe did **not** identify
the concrete runtime class of the child, and did not identify a
`HostResult`-selected path. The `reps={Constructor}` component it reported is a
**compile-time selector predecessor** — the only selector edge that can enter
physical carried case 0 — not an observed runtime class. Under `Open` /
`OpaqueIngress` the planner maps every case to `Reachable` and validation
forbids eliminating one, so compilation must lower the reachable `Constructor`
leaf even if a later execution would select a different arm. A different runtime
arm cannot rescue a compile-time failure in an independently reachable leaf.
**Lawful decode, not a class gap.**

**Do not justify this node from the historical `c7410b79` `BoundaryCarrier: a
host-effect operand is a specialized-only surface` signature.** The Architect
refuted that attribution and the refutation stands (`evt_7v61ed5pn9q3t`):
`claim_host_effect_seat` did not exist at that commit, the refusal fires in
`specialized_at` before any seat key or need is consulted, and the wording is
the generic phase-boundary vocabulary every specialized-only leaf uses.

### 2b. The scope boundary, and it is the sentence that sizes this node

The *same leaf*, the *same `CarriedWord` phase*, a *different slot of the same
operation* is **satisfied**:

```text
Argument(0)  need=BytesPointerLength    avail={specialized:true, carried:FALSE} REFUSED
Capability   need=CapabilityTokenScalar avail={specialized:true, carried:true}  SATISFIED
```

**Availability is per seat, never a blanket phase ban.** This node is *give
`BytesPointerLength` a total emitted observer over the carrier*. It is **not**
"make `CarriedWord` observable" and **not** a phase relaxation. A deliverable or
AC opening *"carried words cannot satisfy this operation"* would be false.

### 2c. Where the refusal is raised

`claim_host_effect_seat` (`lowering/mod.rs:7558` at `b0a0a20c`; it was `:7491`
at `b914c7ff` — the file moved, see 1b) asks
`record.avail.admits(observed)` and, on failure, raises

```text
seat {slot:?} of {operation:?} needs {need:?}, which it cannot observe in {observed:?}
```

`EffectSeatAvail::admits` (`static_transition.rs:4839` at `b0a0a20c`; that file
is byte-identical to the `b914c7ff` anchor) **is** the
`Need ⊆ Avail` test. The seat carries its own coordinates, so a failure names
the exact seat of the exact operation.

### 2d. The seat population is SIX, and only THREE have been measured failing

`host_effect_seat_contract` binds one `bytes` contract tuple
`(ProjectBytesSpan, BytesPointerLength, SPECIALIZED_ONLY)` and returns it at
exactly these `(operation, ordinal)` pairs:

**CORRECTED 2026-08-07. `FsChangeMode` was listed here as "not measured" and it
is not — three quarantined rows name it.** Re-derived from the 30 `#[ignore]`
reason strings at `b0a0a20c`, tallied by the operation each names:

| operation | quarantined rows naming it | was stated as |
|---|---|---|
| `FsReadFile` | 20 | FAILING (1 row) |
| `FsWriteFile` | 6 | FAILING (2 rows) |
| `FsChangeMode` | 3 | **not measured** |
| `ConsoleWrite` | 1 | FAILING (1 row) |
| `FsOpen` | **0** | not measured |

**Thirty rows over four of the five distinct operations. `FsOpen` is the only
operation with no quarantined row, and it remains genuinely unmeasured.**

> **READ THE UNIT OF THIS TABLE CAREFULLY — it is not the unit `AC-4` asks
> for.** The `#[ignore]` reason strings name an **operation**
> (`the FsWriteFile byte-span seat ...`); they do **not** name an **ordinal**.
> So this table cannot tell you whether `FsWriteFile`'s six rows sit at
> `Argument(0)`, at `Argument(2)`, or at both — and `AC-4` is keyed on the
> **six `(operation, ordinal)` seats**, not on five operations.
>
> ⇒ **Resolving operation-tally into per-seat evidence is `D0`/`D1` work, and
> it is exactly the "first failure is not a population oracle" hazard this
> frame already warns about.** Do not carry a row's operation into a claim
> about its seat. The six-seat population in `host_effect_seat_contract` is
> unchanged and is still what `AC-4` must disposition.

⇒ **This is the node's governing population hazard and it cuts both ways.**
Repairing only the three seats that fail leaves an identical seat refusing for
an identical reason, discovered later by whoever first routes a carried word to
it. Flipping all six because they share a contract tuple asserts a capability
for three seats nobody measured. **Neither is acceptable.** The disposition is
per seat, with its own evidence, and `AC-4` is where that is recorded.

### 2e. The substrate audit — what the carrier can and cannot do today

`BOUNDARY_LOCAL_HELPERS` (`boundary_value_clif.rs:66`) is a **closed, pinned
inventory of emitted helper names**. Its content-access half is:

```text
ken_boundary_byte_local        (arena, word, index, out) -> status
ken_boundary_int_sign_local    (arena, word, out)        -> status
ken_boundary_int_len_local     (arena, word, out)        -> status
ken_boundary_int_limb_local    (arena, word, index, out) -> status
ken_boundary_int_view_local    (arena, word, out_view)   -> status
```

**Three facts follow, and they size the work:**

1. **A carried byte value can be read one byte at a time, by index.** There is
   a `store_bytes_len` **writer**, and no reader of a byte span's length
   anywhere in the inventory. **The extent cannot be observed at all.**
2. **`int_view` is the exact precedent for what is missing.** It returns a
   canonical `{sign, len, limbs}` triple through an out-pointer, its own guards
   are the authority, and `narrow_carried_int_u64`
   (`lowering/mod.rs:16810` at `b0a0a20c`; `:16745` at the old anchor) is the
   emitted reader that consumes it.
   Read both before designing anything; the byte-span analogue is
   `{ptr, len}`.
3. **A per-index byte reader existing does NOT establish that a pointer can be
   produced.** `BytesPointerLength` needs a **contiguous address**. Whether the
   carrier stores byte content contiguously, and under which tag / class /
   owner / extent, is precisely the `D1` measurement. **Do not assume it, in
   either direction.**

**Two landed precedents show the phase flip is a real, previously-exercised
move, not a novel one:** `(BufferAllocate, 0)` carries `carried_exact_int` and
`(BufferFreeze, 0 | 3)` carry `phase_bearing_resource`, both `EITHER_PHASE`.
Each was flipped only once its route emitted the helper that makes the
observation total. That order is the one this node repeats.

## 3. Deliverables

Sized so each reaches a releasable increment or a genuine hard stop in about an
hour. **If `D3` or `D4` turns out to be two units, say so and return it — the
recut is the Steward's and a split is expected, not a defect.**

### `D0` — the delta-free baseline

Record, **before any delta**, the per-row state of `px4b_native_production` and
the targeted `ken-runtime --lib` count, and the per-seat refusal for each of the
four failing rows: exact operation, slot, need, observed phase.

**A measurement carrying your own delta cannot produce this.** Per row and per
seat, never as a total; a pass/fail count reads identically before and after.

### `D1` — the representation census, and nothing is frozen here

**Enumerate every legal runtime representation that can reach each of the six
`BytesPointerLength` seats, and measure the exact tag / class / owner / extent
shape of each.**

State the unit each census is taken in. Answer *what does this seat require*,
never *where did it first stop* — a first-failure classification is not a
population oracle, and reading one as such produced a false minimality ruling on
the predecessor node.

**`D1` gates `D2` through `D5`.** No representation may be frozen, and no
`Avail` row touched, before it lands.

### `D2` — normalize at the producer

Convert invocation-owned byte sources into a self-evidencing bytes
representation at their producer. **Freeze the exact invocation-owned `Bytes`
row, or an explicit byte-span subtype, only after `D1`.**

### `D3` — one emitted helper

Add a `bytes_view`-style carrier helper returning pointer and length **only
after** tag / class / owner / extent and arena-bounds checks. Its own guards are
the authority; do not re-derive them in the caller.

Follow the inventory's naming convention (`ken_boundary_*_local`) and the
`IntPart::View` shape. **The convention is a default, not a pin** — if the
measurement says the right shape is different, say so.

**Adding the name reddens `BOUNDARY_LOCAL_HELPERS`'s inventory pin.** That is
expected and it is the positive control that the helper actually landed: the
failure message names the added helper. Update the pin in the same commit, and
quote the redden in the handoff.

### `D4` — one lowering observer

It consumes the exact `PlannedEffectSeat` record and emits the `D3` helper call,
returning SSA pointer and length. **It never constructs `Lowered::Bytes` and
never decodes at Rust or JIT time.**

Separate the outcomes the way `narrow_carried_int_u64` separates them: a
well-formed span that fails a bounds rule and a word that never denoted a byte
span are **different answers**, and a caller must not be able to read one off
the other.

### `D5` — flip the phase, per seat, last

Only after producer and reader close together may a `BytesPointerLength` seat
become `EITHER_PHASE`. **Per seat, each with its own evidence** (`AC-4`).
**Every seat left `SPECIALIZED_ONLY` needs an explicit proof of why**, recorded
in the disposition table, not an omission.

### `D6` — the activation-gate discharge pass

**This node inherits an activation gate it did not create (section 6). `D5` is
the activation.** So, after `D5` and before any merge:

**Re-run the per-family register** (`evt_5tzqtkgw02gxg`) and report, per family,
exactly one of:

- **BUILDABLE** — the exact fixture, the exact assertion, **and a demonstrated
  RED before green, committed.** A control is not buildable until its assertion
  has been *seen to fail*, and a red observed once in a turn and not committed
  guards nothing.
- **BLOCKED** — the exact producer it still requires, named.

**Every family that became buildable ships its control in this node.** A family
that stayed blocked keeps its gate entry with its producer named.

**`D6` is a deliverable and not a note.** It is here because a sentence in a
frame that tells someone to do something is an acceptance criterion, and this
campaign has already paid for putting one under a heading whose grammatical mood
is advice.

## 4. Acceptance criteria

Every AC names its owning deliverable. **If one cannot, that is the finding.**

- **`AC-1` (`D5`) — the four measured rows reach successful lowering**, each
  stated per row against its `D0` entry, and each naming the exact seat that
  now admits `CarriedWord`.

  > **MEASURED:** the four rows pass.
  > **CLAIMED:** carried byte spans are observable at their seats.
  > **THE GAP:** the rows pass *for this reason*. A row can go green because a
  > different arm was selected. **Close it with `AC-2`, not with prose.**

- **`AC-2` (`D5`) — a mutation that removes the byte-span observation restores
  the exact original refusal, at the exact original seat.** The assertion is on
  the **exact** message — operation, slot, need, observed phase — never
  `is_err`.

  **Report which operand moved.** This control is detector-side: it proves the
  observer is load-bearing for these rows. It does **not** prove reach, and must
  not be reported as if it did.

- **`AC-3` (`D0`, `D5`) — per row, never a total.** Every row green at `D0` is
  still green, stated per row.

  **Report ignored separately from passed, always.** A bare `passed / failed`
  pair reads green while this node has changed nothing, because an ignored row
  is not a failing one. **Arithmetic restated 2026-08-07** — `px4b` carries 19
  tests and 4 ignores, all four this node's:

  | reading | meaning |
  |---|---|
  | `19 passed / 0 failed / 0 ignored` | success — the four un-skipped and green |
  | `15 passed / 0 failed / 4 ignored` | **the no-op** — nothing un-skipped |

  **Only the ignored count tells them apart**, and the old `18/0/1` versus
  `14/0/5` pair is superseded: it assumed a fifth ignored entry-trap row that
  no longer exists.

  **Apply the same discipline to the other nine files in 1a-i.** `px4b` is one
  of ten, and a per-file ignored count is the only thing that distinguishes 30
  un-skips from 4.

- **`AC-4` (`D5`) — the per-seat disposition table, over all SIX seats.**
  For each of the six seats in section 2d, exactly one of:

  1. **`EITHER_PHASE`**, with the `D1` representation it was proved against and
     a witness that reaches **that seat**; or
  2. **`SPECIALIZED_ONLY`**, with an explicit proof of what the observer cannot
     yet provide for it.

  **A seat may not be left out, and "same contract tuple as a proved seat" is
  not evidence.** The tuple is shared by construction; that is what makes it a
  bad discriminator.

  **Pin the allowed inventory, not a forbidden list:** assert the exact set of
  `EITHER_PHASE` `BytesPointerLength` seats, so that any addition reddens —
  including one nobody imagined.

- **`AC-5` (`D3`) — the helper's guards refuse a wrong tag, class, owner or
  extent with ZERO host dispatch**, each refusal asserted on its exact error and
  each reached by a witness that **varies that axis and holds the neighbours
  fixed**.

  **Two witnesses that mutate the same field are one witness.** Perturb the
  wrong axis and an earlier guard fires first: you observe a rejection, assert
  its exact message, and have measured a different law. Any guard with no
  witness is reported as *"no witness via ⟨the routes tried⟩"*, naming them —
  never silently counted as a law.

- **`AC-6` (`D4`) — phase equivalence.** For a value reaching the same seat as a
  specialized template and as a carried word, the observed span is the same.
  Persistent **and** invocation-owned `Bytes` are both exercised.

- **`AC-7` (`D3`) — the emitted helper inventory pin is updated and still
  closed**, and its redden on the addition is quoted in the handoff.

- **`AC-8` (`D6`) — the activation gate is discharged or restated, per family**,
  in the BUILDABLE / BLOCKED form above. **Route entry, an emitted counter, or
  plausible IR is not evidence.** A control is admissible only if the entire
  claimed property is decided **before** any independent abort, and a mutation
  of it has been observed to make the exact assertion RED before the run reaches
  that same abort.

  **Every claim in this AC states its population: in which build, on which rig.**

- **`AC-9` (no-regression).** Workspace green **in CI** — never a local
  `cargo test --workspace` (`COORDINATION §12`).

- **`AC-10` (`D3`, `D4`) — the `ResponseBytes`-validity invariant is enforced
  structurally, not by convention.**

  **STEWARD AMENDMENT, 2026-08-07, after `D2` merged at `4f9f0987`.** Added
  from Adversary `evt_5xqw6xsbm4v8b`, whose central claim I verified in source
  before folding it. **This is preventive. There is no defect and no repro —
  the closure holds at `4f9f0987`.**

  > **MEASURED:** a newly added construction of `Lowered::ResponseBytes` that
  > does not go through the masking helper is **refused by a mechanism**, and
  > the refusal is demonstrated by adding one and observing it.
  > **CLAIMED:** every `ResponseBytes` reaching the carrier is a valid span.
  > **THE GAP:** today that is enforced by **two call sites and a comment.**
  > `masked_reply_response_bytes` (`core.rs:64`) is the only production
  > construction; the two typed producers call it; the second site's comment
  > says a mask at only one of them "leaves the other dereferencing an
  > unestablished pointer on its failure path." **That sentence is the author
  > identifying a closure requirement and discharging it by convention.**

  **Why `D2` raised the stakes rather than creating a bug.** Before `D2`, an
  unmasked `ResponseBytes` published the host pointer as a borrowed word — wrong,
  but nobody dereferenced it. After `D2`, the disposition at `mod.rs:9880` maps
  **every** `Lowered::ResponseBytes` to `(PersistentGround, Bytes)`, so it
  reaches `emit_carrier_bytes_runtime_span`, whose loop dereferences `pointer`.
  **The obligation moved from "is this word right" to "is this pointer safe to
  read", and the type is an ordinary struct variant constructible anywhere in
  the module.** A third producer added later that builds the literal instead of
  calling the helper is a dereference of an unestablished pointer, and nothing
  structural stops it.

  **The mechanism is the ring's call** — private constructor, helper-only path,
  a debug assert, or something else. **State which and why.**

  **The obstacle to name up front, because it is what makes the obvious fix
  harder than it looks:** two test sites construct `Lowered::ResponseBytes`
  directly and legitimately —
  `core/tests/constructors.rs:2864` and `:8748`, the latter being the `D2` edge
  test that takes `declared_len` separately from `source.len()` **precisely so
  the guards are reachable.** A mechanism that forbids direct construction
  outright disarms the controls that prove the guards work. **Say how the chosen
  mechanism keeps those two reachable**; if it cannot, that is a finding, not a
  reason to weaken the tests.

  **Positive control:** the added construction must fail. If it compiles and
  runs clean, the mechanism is not doing anything and this AC is not
  discharged. Record what you added and what refused it.

  **If the honest answer is that no structural mechanism is available without a
  disproportionate change, say so and record it as a residual** — `AC-10` then
  discharges as *"guarded by review, not by mechanism"*, explicitly, with the
  reason. **An AC with nowhere to record the honest answer gets recorded as
  guarded.**

- **`AC-11` (`D4`) — the byte view's VALIDITY WINDOW is written down, because
  store-ownership is not address-stability.**

  **STEWARD AMENDMENT, 2026-08-07, after `D3` merged at `7c2587e6`.** Added from
  Adversary `evt_2pfr5k60epyta`. **Every claim below I re-derived in source
  before folding.** This is preventive: **there is no reachable violation
  today** and no repro.

  > **MEASURED:** `define_bytes_view`'s doc states the returned pointer's
  > validity window, and `D4`'s observer is written against that stated window
  > rather than against the owner guard's framing.
  > **CLAIMED (by the owner guard's doc, `boundary_value_clif.rs:2203`):** *"a
  > class says what a payload IS, an owner says how long it LIVES. Handing a
  > caller a pointer into storage that dies with the invocation is the failure
  > this refuses."*
  > **THE GAP:** that reads as a lifetime guarantee on the **returned address**,
  > and the guard only establishes it on the **referent**. `region_data_base`
  > forms `ARENA_DATA + at` and `define_bytes_view` stores that raw interior
  > pointer to `*out` — a live address into the persistent image's data table.
  > **The referent outlives the invocation; the address does not necessarily
  > outlive the next reservation.**

  **The codebase already says so, in the one place a reader of `bytes_view` will
  not look.** `BoundaryValueStore::publish_persistent` (`boundary_value.rs:2122`)
  carries: *"Invalidated by any later materialization or reservation — those can
  move the tables. Materialize, reserve, then publish."* That is exact.
  `reserve_persistent` reaches `BoundaryRegion::reserve` (`:1532`), whose body
  includes `self.data.resize(self.live_data + data, 0)` on a `data: Vec<u8>`.
  **A resize reallocates and moves the table.**

  **The ordering invariant is enforced only by `debug_assert!`** (`:1533`,
  *"reserve before publish: growing a table moves it under the pointer"*). It is
  the right invariant and it is stated well, and it is **compiled out under
  `--release`.**

  **What I measured that the report explicitly did not, and it bounds the
  severity.** Every `reserve_persistent` call site in the crate is immediately
  followed by `publish_persistent` — one reserve, then publish, never a reserve
  after: the production activation path (`boundary_activation.rs:130-131`), the
  clif harness (`boundary_value_clif.rs:3202-3208`), and two test fixtures.
  **So the discipline holds by construction at every site today.** The exposure
  is a future site, and `D5` is what creates consumers who could hold a view
  across one.

  **Why this is filed at `D4` rather than left for the consumer to settle.**
  `D3` activates nothing, so nobody holds a view across anything and there is no
  dangling read to demonstrate. That is exactly why the window is now: **`D4`
  and `D5` will be written against the guard's current framing, and once a
  consumer exists the two properties get reconciled to whatever that consumer
  happened to need** — at which point the over-claim survives as the part nobody
  re-derived.

  **Note the direction of travel.** `D2` went to real trouble to avoid
  publishing a borrowed address — it copies rather than retagging the host
  pointer. **`D3` reintroduces a published raw pointer**, into region storage
  instead of host storage. The lifetime is genuinely longer. It is **not
  unbounded**, and only the first half of that is currently written down.

  **The `D4` deliverable is the CONTRACT SENTENCE, not a mechanism.** Say in
  `define_bytes_view`'s doc what the view's validity window is — valid until the
  next materialization or reservation of the persistent image — and ground
  `D4`'s observer on it. That is cheap and it is in `D4`'s path.

  **The MECHANISM question is explicitly NOT `D4`'s and must not be improvised
  here.** Whether this additionally wants a debug-only generation counter
  checked at the view, a rule that views are consumed before any further
  reservation, or nothing at all is **a design question for the Architect.**
  Raise it; do not decide it inside the turn. If `D4` cannot land without an
  answer, that is a hard stop, not a licence.

## 5. Banned scope

- **This is not `Carried -> Lowered`.** That inverse is withheld by design;
  reintroducing it here is the same wall wearing a different name.
- **Do not dereference an arbitrary `BorrowedOpaque` scalar.** That class also
  represents capability and resource tokens. A byte-span reader that accepts the
  **class** rather than a **measured row** is a confused-deputy hole.
- **No widening of `Avail`, no descriptor weakening, and no planner or ABI
  authority change ahead of `D5`.**
- **No representation frozen ahead of `D1`.**
- **No blanket phase relaxation.** Setting `BytesPointerLength` to
  `EITHER_PHASE` need-wide, rather than seat-wise, asserts a capability that
  does not exist. Availability is recorded per seat for exactly this reason.
- **The `explicit entry trap` ban is DISCHARGED, and the question behind it is
  now answered.** The row was cleared by `RT-SRCBODY-BIND-ORDER` `D4`, and the
  two do **not** share a root cause: the entry trap was the source-body
  parameter run reaching its body in ABI descriptor order, not a byte-span gap.
  [[RT-ENTRY-TRAP-254]] is `closed`. Nothing here is yours to clear or to fold.

  **The reason the ban existed still binds.** It was written to stop a
  vocabulary inference — "bytes" appearing in a test name — of exactly the kind
  the Architect refuted once already on this campaign. That the guess would
  have landed on the wrong cause **is the argument for the discipline, not
  against it**: the shared word was `bytes` and the shared cause was binding
  order. Do not infer a seat, a class, or an owner from a name anywhere else in
  this node.
- **Do not touch the carried source-match class dispatch.** It stays in
  [[RT-CONTSRC-PRODUCER-LOCAL]].

## 6. The activation gate this node inherits

**Architect `evt_7qfayjcebxv5y`, ratified by the Steward `evt_4nabbpm2crz82`.**

[[RT-CONTSRC-PRODUCER-LOCAL]] lands `lower_source_carried_match` as a **reviewed
dormant partial mechanism with an unmeasured runtime residual.** That was
allowed **only because no completing Runtime rig executes the carried
source-Match route at all** — one fact, which bounds the present risk and the
present claim together.

**NARROWED by the Architect, `evt_m36y2zegby7m`, 2026-08-06.** Say **"no
ENUMERATED completing rig executes `lower_source_carried_match`"**. The census is
explicitly the `ken-runtime` crate plus `rt_parity`; `px4b_native_production` is
a **third rig that completes lowering** and was in neither. It was checked and
ruled a **negative by path** — a functionized declaration-unit call whose
`Carried` scrutinee dispatches to the *generic* `lower_carried_match`, never
constructing or resuming `SourceContinuation::MatchScrutinee`, which is
`lower_source_carried_match`'s only caller on that path. **So it is an
enumerated negative, not a universal proof about every possible rig.**


**This node is one of the two visible activation routes.** The moment a carried
source-Match path becomes successfully executable, the dormancy argument
expires. That is what `D6` and `AC-8` exist to catch.

**The gate has two entries and they are independent:**

1. **The shared cross-unit carried producer** — a cross-unit carried word
   reaching a source-machine `Match` **in a unit that does not feed a byte-span
   effect seat**. Families 1, 2b, 3, 6 and 7 all name it.
2. **The `cfg(test)` seam-visibility boundary** — the control seam must be
   visible to the build that reaches the arm. It exists only in the
   `ken-runtime` **lib** build; the rig that exercises the carried route,
   `rt_parity_native`, lives in **`ken-cli`** and links a non-`cfg(test)` build.

**Byte-span observation does not fix entry 2.** The two are kept separate
deliberately so that a successor cannot read one discharge as both. Feature-
gating the seam across that boundary is a real change to what ships and belongs
to the gate, not to a bounded control child.

**Family 7 is UNREACHED, not UNREACHABLE.** It was never measured unreachable.
Do not compress that: unreached is a producer gap someone closes, unreachable is
a closed question, and they license opposite decisions.

**Family 2a already ships a transition sentinel rather than a promise** — it
asserts `applications == 0` and reddens the moment the arm becomes reachable
under `cfg(test)`, which is exactly when 2a becomes writable. **Prefer that
shape for any remaining gate entry where it is cheap.** A blocked family that
asserts its own unreachability is a mechanism that fires by itself, instead of a
note someone must read at the right moment.

**Measured dead ends, so nobody re-spends them:** a `Construct`-bound closure
parameter, a `RuntimeDeclaration` parameter fed a `Construct`, and the borrowed
process input **all arrive `Specialized`** — each completes and returns a
plausible value from a route that is not this one. The abort boundary is exact:
`PHASE 2 selector graph COMPLETE` fires and `ALL LEAVES LOWERED` never does.

## 7. Governing hazards

**State the population in the same sentence as the claim.** Three times on this
node's predecessor a true fact about one population was transferred to a
neighbouring one, and none of the three was carelessness — each read as
obviously transferable:

| true of | falsely transferred to |
|---|---|
| the route is **entered** 10 times on `rt_parity` | a **control** can be written on it |
| a closure parameter is carried at an **effect argument** | carried at a **Match scrutinee** |
| the mutation reddens on **`rt_parity`** | it reddens in the **lib build where the seam lives** |

⇒ **"Buildable" is meaningless without "in which build, on which rig."**

**Gate-shaped is not gate-satisfying.** A control whose red can only be
demonstrated by an **uncommitted** production mutation is gate-shaped. The gate
must be re-runnable by someone who was not present for the demonstration — that
is the only moment it exists for.

**Removing the first blocker is not a promise that byte-span alone completes
every row.** It may reveal a later one. The register is the checklist, not this
paragraph.

**Every mutation carries a provenance check.** Count the anchor **before** the
edit and compare against a **predicted** post-count; do not re-match a needle
the replacement may still contain. Restore byte-identically and verify with
`git diff --quiet` — `git diff --stat` always exits 0 and is not an emptiness
test. Commit the real fix before any mutation-proof reset. **Report the invalid
mutation rows; that is what makes the valid ones trustworthy.**

## 8. Hard stop

Stop and report, with the concrete seat or edge, if:

- `D1` finds that no legal runtime representation reaching a
  `BytesPointerLength` seat can produce a contiguous address — that is a
  representation boundary, and it goes to the Architect, not into a workaround;
- a seat cannot be dispositioned either way under `AC-4` without asserting a
  capability the measurement does not support;
- `D5` activates the carried source-Match route while a family that became
  buildable under `D6` cannot get a **committed** control in this unit —
  **do not merge the activation**; or
- the base line in section 1a was never filled in.

**Do not absorb any of these and do not work around them.**
