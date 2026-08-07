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

**Cut from current `origin/main`, after the `RT-CONTSRC-PRODUCER-LOCAL`
candidate merges.** Do not continue `wp/RT-DECL-CLOSURE-PORT-typed-units`: the
publisher **squashes**, so `b914c7ff` is not an ancestor of `main` once it
lands, and continuing that branch would re-offer 212 already-merged commits.

**The anchors in 1b were measured at `b914c7ff` and must be re-verified against
`main` before you rely on one.** The squash preserves content, not commit
identity, so the blobs should match and the commit will not.

### 1a-i. Four of this node's rows are SKIPPED, not passing

The four byte-span rows in section 2d ship marked `#[ignore]`, each carrying its
exact observed signature and this node's id. **A skipped row measures nothing**,
so at your `D0` the suite will report them ignored rather than failed.

⇒ **`D0` must un-skip the four rows and record their live failure**, and `AC-1`
is discharged against that live baseline, not against an ignored one. **Removing
the four `#[ignore]` attributes is this node's deliverable**, and a green suite
that still carries them has discharged nothing. The fifth row stays ignored — it
belongs to [[RT-ENTRY-TRAP-254]].

### 1b. Content anchors at `b914c7ff`

| path | blob |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs` | `7f4fa3c376be34402c4815e0706b896f4363e66d` |
| `crates/ken-runtime/src/cranelift_backend/lowering/mod.rs` | `69f6ea52361079f3b5432e0d9ff6759c034d03e9` |
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `f3c0c5e452b84e8492b61433c9621485ad8a502a` |
| `crates/ken-runtime/src/boundary_value_clif.rs` | `0ec07c6698aca67eb51084ecb4ab376efa5a6ed0` |
| `crates/ken-runtime/src/boundary_value.rs` | `ac0745763b2c71c07bb5205fad5edaa3a3718e17` |
| `crates/ken-cli/tests/rt_parity_native.rs` | `b2df2bbd00644b907cae5d05efa76edd9df1b3f2` |

### 1c. Expected baselines, to be re-measured at `D0` and not inherited

- `px4b_native_production`: **14 passed / 5 failed**. Four of the five are this
  node's subject; the fifth is the `explicit entry trap`, owned by
  [[RT-ENTRY-TRAP-254]] and **not** this node's to clear.
- `ken-runtime --lib`, targeted: **778 passed / 2 failed / 1 ignored**, the two
  reds standing and pre-existing.

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

`claim_host_effect_seat` (`lowering/mod.rs:7491` at the anchor) asks
`record.avail.admits(observed)` and, on failure, raises

```text
seat {slot:?} of {operation:?} needs {need:?}, which it cannot observe in {observed:?}
```

`EffectSeatAvail::admits` (`static_transition.rs`, near `:4834`) **is** the
`Need ⊆ Avail` test. The seat carries its own coordinates, so a failure names
the exact seat of the exact operation.

### 2d. The seat population is SIX, and only THREE have been measured failing

`host_effect_seat_contract` binds one `bytes` contract tuple
`(ProjectBytesSpan, BytesPointerLength, SPECIALIZED_ONLY)` and returns it at
exactly these `(operation, ordinal)` pairs:

| seat | measured at tip |
|---|---|
| `ConsoleWrite` `Argument(1)` | FAILING (1 row) |
| `FsReadFile` `Argument(0)` | FAILING (1 row) |
| `FsWriteFile` `Argument(0)` | FAILING (2 rows) |
| `FsWriteFile` `Argument(2)` | not measured |
| `FsChangeMode` `Argument(0)` | not measured |
| `FsOpen` `Argument(0)` | not measured |

**Four failing rows over three distinct seats, out of a population of six.**

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
   (`lowering/mod.rs`, near `:16745`) is the emitted reader that consumes it.
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
  still green, stated per row. **The `explicit entry trap` row stays ignored and
  that is correct** — it belongs to [[RT-ENTRY-TRAP-254]].

  **Report ignored separately from passed, always.** The suite lands with five
  rows ignored, so a bare `passed / failed` pair reads green while this node has
  changed nothing. `18 passed / 0 failed / 1 ignored` and
  `14 passed / 0 failed / 5 ignored` are the success and the no-op, and only the
  ignored count tells them apart.

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
- **Do not clear the `explicit entry trap` row here**, and do not fold
  [[RT-ENTRY-TRAP-254]] into this node on the strength of "bytes" appearing in
  its test name. That is the vocabulary inference the Architect already refuted
  once on this campaign. Whether the two share a root cause is unmeasured and
  must be measured.
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
