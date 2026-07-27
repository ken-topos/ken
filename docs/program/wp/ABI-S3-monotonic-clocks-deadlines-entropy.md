# `ABI-S3` — monotonic clocks, sleep/deadlines, and secure kernel entropy

**Owner:** Runtime · **Size:** M · **Authority:**
`docs/program/10-linux-abi-completion.md` §4 Track S · **Node:**
`docs/program/issues/ABI-S3.md`

> ## ⭐ Why this one, now
>
> **`ABI-S3` has `depends_on: []`.** It is the only node in the Linux-ABI
> program that needs nothing from `PX8`, and it **gates `PX12`** (readiness —
> `timerfd`, deadlines, cancellation), which is one of the three inputs to the
> committed exit. It is critical-path work, not filler.

## Fixed inputs — measured on `origin/main = d359fb66`

⛔ **Verify each before you build on it; do not re-derive them from memory.**

| fact | site |
|---|---|
| ⭐ **`crates/ken-host/effect_abi_v1.catalog` is the CODEGEN AUTHORITY** — 22 `operation\|` rows; `build.rs:114` reads it and generates from it, `build.rs:14` is `rerun-if-changed` | `effect_abi_v1.catalog` |
| a new op **requires a catalog row** — e.g. `operation\|ClockWallNow\|0201\|unavailable\|UnitRequestV1\|0\|HostReplyV1\|1` | `effect_abi_v1.catalog:53` |
| `HostOpV1` holds **22** operations | `crates/ken-host/src/effect_v1.rs:16` |
| ⛔ the op count is pinned in **FOUR** literals, not one | see the table below |
| opcode families: `0x01` console, `0x02` clock, `0x03` fs, `0x04` resource | `effect_v1.rs:17`–`38` |
| ⭐ the clock family holds **exactly one** op — `ClockWallNow = 0x0201` | `effect_v1.rs:21` |
| availability is **two-state**: `NativeTested` \| `RepresentedUnavailable` | `effect_v1.rs:449`–`452` |
| `ClockWallNow` **is ambient** (`is_ambient()` — no capability token) | `effect_v1.rs:97` |
| wire tags `0`–`21` are taken; the next free `put_u8` tag is **22** | `crates/ken-host/src/effect_wire.rs:114`, `:621` |
| promotion-set membership is *"a plan, not evidence"* | `effect_v1.rs:103`–`108` |

### ⭐ The mechanical surface a new host op must touch — 10 files, 32 sites

Measured by enumerating every `ClockWallNow` occurrence, because it is the
existing member of the family you are extending and therefore the exact
template:

| file | sites |
|---|---|
| `ken-host/src/effect_v1.rs` | 8 |
| `ken-interp/src/eval.rs` | 5 |
| `ken-elaborator/tests/b2_acceptance.rs` | 4 |
| `ken-elaborator/src/export.rs` | 3 |
| `ken-interp/tests/b3_acceptance.rs` | 2 |
| `ken-host/src/effect_wire.rs` | 2 |
| `ken-elaborator/tests/b1_exact_denotation_alphabet.rs` | 2 |
| `ken-elaborator/tests/b1_acceptance.rs` | 2 |
| `ken-elaborator/src/erasure.rs` | 2 |
| `ken-elaborator/src/compiler_driver.rs` | 2 |

⛔ **Use this list as a checklist, not as a bound.** It is where `ClockWallNow`
appears, which is evidence about *that* op; a new op with different shape (a
blocking one — see D2) may require a site none of these name. **Sweep for the
obligation, then confirm you covered at least these.**

### ⛔ CENSUS CORRECTION — the table above is INCOMPLETE. Runtime, `evt_4rc0b25k59a6s`

Verified against `main` before recording. **Two omissions, both load-bearing:**

**1. `crates/ken-host/effect_abi_v1.catalog` is missing from the table and is
the codegen authority.** `build.rs` parses it (`:114`) and generates the host
effect surface from it. A new op **needs a catalog row** or `build.rs` fails with
*"HostOpV1 catalog is closed at 22"*.

**2. The op count is pinned in FOUR literals, not the one the table named:**

| site | literal |
|---|---|
| `crates/ken-host/build.rs:161` | `assert_eq!(operations.len(), 22, "HostOpV1 catalog is closed at 22")` |
| `crates/ken-host/src/abi_v1.rs:1749` | `assert_eq!(crate::HOST_EFFECT_ABI_V1_CATALOG.len(), 22)` |
| `crates/ken-host/src/effect_v1.rs:42` | `pub const ALL: [Self; 22]` |
| `crates/ken-host/src/effect_v1.rs:2613` | `assert_eq!(HostOpV1::ALL.len(), 22)` |

⭐ **Why the census missed them, because the same mistake is easy to repeat: my
sweep was bounded twice over, and neither bound was visible from inside it.**

- It ran with **`--include=*.rs`**, so a `.catalog` file could not appear **by
  construction** — the authority was excluded by file extension, not by judgment.
- It keyed on the **op name**. The `build.rs` and `abi_v1.rs` pins never mention
  `ClockWallNow`, because they count the **collection**, not the op. ⇒ Enumerating
  one member's occurrences can **never** find a pin on the set.

⇒ **When censusing "everything a new X must touch", sweep by extension-free glob
AND search for the count/collection pins separately from the member name.** ⚠ And
note the direction of the error: an under-complete census reads as a *complete
checklist*, so it is silent — this one surfaced only because the frame said the
list was a checklist rather than a bound, and Runtime treated it as an in-scope
expansion instead of a stop.

### ⭐⭐ A THIRD site — same defect, in a direction the fix above does NOT cover

**`crates/ken-runtime/.../prelude.rs` is in the obligation surface and was
structurally invisible to my instrument** (`runtime-implementer`,
`evt_14xf98y408sm0`). It declares the **language-level** effect vocabulary —
`ConsoleOp`, `ClockOp`, `AmbientOp`, `clock_resp` — and it spells the operation
`WallNow`, never `ClockWallNow`. ⇒ **A grep for the Rust identity has zero hits
in the one file that defines the surface algebra a new op must extend.**

⛔ **Note carefully how this differs from the `.catalog` miss, because the
correction above would not have caught it.** The `.catalog` file was one the
grep *could* have seen and an extension filter excluded; `prelude.rs` is a
`.rs` file the extension-free glob **would** have included and the search term
**still** could not reach. Dropping `--include=*.rs` fixes the first and does
nothing for the second.

⇒ ⭐ **The general form: a census keyed on one plane's spelling of a concept is
blind to every other plane that names the same concept differently.** The Rust
enum says `ClockWallNow`, the catalog says `operation|ClockWallNow|…`, the
surface algebra says `WallNow` inside `ClockOp`, and the count pins say `22`
and name nothing at all. **Four spellings, one obligation.** ⇒ Enumerate the
**planes** a new op crosses — ABI enum, codegen catalog, wire, surface algebra,
count/collection pins, elaborator export — and search each in **its own**
vocabulary. ⛔ Do not sweep one term across all of them and read the result as
coverage.

⚠ **This is why the ring derived the obligation from the compiler instead**, by
adding a throwaway variant and patching errors to fixpoint. That method has no
vocabulary at all, which is exactly its advantage — and its own limit is
recorded in deliverable 5: `ken-interp` has **zero** compiler-policed sites, so
a compiler census cannot see what a wildcard arm silently absorbs. **Neither
instrument is complete; they fail in different directions, which is the reason
to run both.**

## The design judgments, front-loaded (§2c) — do not re-litigate these

### D1 — monotonic is a **separate operation**, never a mode of `ClockWallNow`

A deadline computed on a wall clock is wrong across any clock adjustment. The
node states this and it is settled: `ClockWallNow` (and `ABI-A1`'s availability
promotion of it) **must not** stand in for a monotonic read.

⇒ Add a distinct op in the `0x02` family. The two are not interchangeable at any
layer, and nothing may fall back from one to the other.

### D2 — ⭐ sleep is the **first blocking operation in the ABI** — the real risk here

Every one of the 22 landed ops completes without suspending the caller. A sleep
does not. **Treat "what does the host boundary do while a Ken program is
suspended?" as the load-bearing question of this WP, not as an implementation
detail** — it is the question `PX12` inherits.

⇒ **The deadline/timeout must be a value in the operation type, not prose.**
`PX12` requires *"cancellation and timeout in the operation type rather than in
prose"*, and `PX12` depends on this node — so a sleep whose duration is an
untyped scalar with a comment forces a breaking change there.

#### ✅ D2 RULED — Architect, `dec_50pzvb14nnbt0`, `resolved` 2026-07-27

**Cancellation is reserved for `PX12`.** `ABI-S3` lands, and lands only:

- a **stable typed absolute monotonic `Deadline`** value, and
- **`SleepUntil(Deadline)`**, **uncancellable**.

`PX12` must embed that deadline value **unchanged** inside its own typed
wait/cancellation control. ⛔ **`ABI-S3` must NOT add a placeholder cancellation
field, token, status, or semantics** — not even an unused one reserved for later.

⭐ **Absolute, not relative, is the load-bearing word.** A relative duration has
to be re-based every time it is composed, which is exactly the re-shaping `PX12`
must not have to do. An absolute monotonic deadline composes by being passed
along untouched.

### D3 — ✅ RULED: entropy is **host-ambient but effect-explicit**

**Architect, `dec_50pzvb14nnbt0`, read `resolved` from the object 2026-07-27.**
This was the one open question gating the entropy slice; it is closed.

**Ambient at the host-dispatch layer, like `Clock`** — because it *"exposes no
pre-existing scoped resource and has no meaningful authority lattice to
attenuate; a capability would duplicate the already-static effect-row
admission."*

⭐ **This settles which predicate `is_ambient()` tracks, so the next op in this
family does not re-ask:** ambient means **"names no pre-existing scoped resource
and has no authority lattice to attenuate"** — ⛔ **not** *"is not
security-relevant."* Security relevance alone does not earn a capability; an
attenuable resource does.

⛔ **But it is NOT a pure or hidden ambient read.** It remains an **explicitly
declared `Entropy` effect**, visible in the program type. Concretely:

- ⛔ no `EntropyCap`, and **no `ProgramCaps` field**;
- ⛔ no capability token in the host request;
- ✅ the `Entropy` effect **does** appear in the program's effect row.

⚠ **Ambient does not weaken any source or honesty requirement** — the ruling is
explicit on this, and it is the half most likely to be dropped:

- **kernel CSPRNG only** — no userspace PRNG, no seeding a userspace generator
  and calling that the op;
- **bounded request**;
- ⛔ **no userspace or weak fallback**;
- **exact represented unavailability** — an unavailable secure source is
  `RepresentedUnavailable`, never a silent downgrade;
- ⛔ **no cryptographic or proof claim beyond the observed bytes.** Do not
  document the op as providing a security guarantee it does not demonstrate.

### D4 — every new op lands `RepresentedUnavailable`

The two-state availability model has no provisional value, and the code says
membership in a promotion set *"is a plan, not evidence."* Promotion is a
separate, evidence-bearing act (that is what Track A is for). ⛔ Do not add any
new op to `PX5_PLANNED_NATIVE_TARGETS` in this WP.

## ⭐ SIZING CALL — the node is **L**, and it is NOT re-sliced. Steward, 2026-07-27

`runtime-implementer` flagged that `F-3`/Option A *"materially exceeds Size M"*
and asked for a Steward size call (`evt_39mcgsvjqcqk`). **Answering it: the node
moves M → L, the scope stays whole, and nothing is deferred out.**

**What the Architect's Option A ruling (`dec_5vbz7tvc5dcdw`) actually costs**, as
the ring measured it: a new opcode family byte `0x05` (the landed families are
`0x01` console, `0x02` clock, `0x03` fs, `0x04` resource, and entropy belongs to
none); `EntropyOp` + `entropy_resp` + `EntropyIO` in the prelude; an `Entropy`
effect label; an `entropy_family` spine field in both spine structs; and the
ambient elimination in `erasure.rs` going from a two-arm `InL`→console /
`InR`→clock match to a three-way one.

**Why the answer is "grow it", not "slice it":**

1. ⛔ **The topology change has no seam that leaves the tree consistent.**
   `AmbientOp` is the *closed* ambient operation sum; a slice that lands the
   family byte and the prelude type without the three-way elimination ships a
   coproduct member nothing eliminates. **That is verbatim the failure this
   fleet spent five days on in `RT-NATIVE-FNSPLIT`** — a representation whose
   consumer is the next node's problem. ⛔ Not again, and not for a saved letter
   of size.
2. **Option A is ruled.** Re-slicing to avoid the topology would be re-litigating
   `F-3` by the back door — the Architect rejected Option B precisely because it
   creates a second ambient semantic lane.
3. **The cost is bounded and already surveyed.** The list above is a closed
   enumeration made by the compiler, not an estimate.

⚠ **Size is a coordination signal, not an authorization limit.** The M was my
estimate before three census findings and one topology ruling landed; ⭐ **an
estimate that has been overtaken is a stale input, and correcting it is cheaper
than defending it.** No AC changes, no deliverable is dropped, and the ring does
not stop.

## Deliverables

1. **A monotonic clock read** in the `0x02` family (D1), with its wire encode +
   decode, its canonical request/response, and its availability = 
   `RepresentedUnavailable`.
2. **A stable typed absolute monotonic `Deadline`** plus **uncancellable
   `SleepUntil(Deadline)`** (D2). ⛔ No cancellation field, token, status, or
   placeholder of any kind — `PX12` composes this deadline unchanged.
3. **Secure kernel entropy** as a **host-ambient but effect-explicit** op (D3):
   no `EntropyCap`, no `ProgramCaps` field, no capability token in the host
   request, **and** an `Entropy` effect visible in the program's effect row.
   Kernel CSPRNG, bounded request, no fallback.
4. **The full 10-file sweep** for each new op, using the table above as the
   checklist.
5. **A catalog row per new op** in `crates/ken-host/effect_abi_v1.catalog`, and
   **all FOUR op-count literals** updated to the new arity — `build.rs:161`,
   `abi_v1.rs:1749`, `effect_v1.rs:42`, `effect_v1.rs:2613` — with the wire tags
   continuing from **22**. ⛔ Bumping an assert without adding the catalog row, or
   the row without the asserts, is the failure mode here.
6. ✅ **Done — the D2/D3 ruling is transcribed above** from
   `dec_50pzvb14nnbt0` (read `resolved` from the object, `resolved_by` the
   Architect). Nothing about it lives only in the channel; build from the
   sections above, not from the decision text in a notification.

## Acceptance criteria

⚠ **Behavioral, per operator policy (2026-07-26): oracles that assert facts
about source lines invite failure and delay. Assert what the ops DO.**

- **AC-1 — wire round-trip, per new op.** Encode → decode returns the identical
  canonical request, for every new op. ⛔ The existing count assert is *not* this
  test: arity is not round-trip fidelity.
- **AC-2 — ⭐ the monotonic/wall distinction is OBSERVABLE, with a negative
  control.** A test that only reads the monotonic clock passes whether or not
  D1 was honoured. Required shape: **monotonic readings are non-decreasing
  across a simulated wall-clock adjustment that moves `ClockWallNow` backwards.**
  Positive control: the same probe over `ClockWallNow` **must** observe the
  backward step — otherwise the harness cannot perturb the wall clock and the
  monotonic result is vacuous.
- **AC-3 — the deadline is a value, demonstrated by use.** A caller constructs an
  absolute monotonic `Deadline`, passes it to `SleepUntil`, and observes it
  honoured. ⛔ Do not discharge this by showing a field exists.
- **AC-3b — ⛔ NO cancellation surface exists** (D2). The `Deadline` type and the
  `SleepUntil` request carry **no** cancellation field, token, or status —
  including unused or reserved ones. ⚠ This is a **negative** claim, so it needs
  a positive control: show the probe **does** find the fields that *are* present,
  otherwise "found no cancellation surface" is indistinguishable from a probe
  that inspects nothing.
- **AC-3c — the `Entropy` effect is VISIBLE in the program type** (D3), while
  **no** `EntropyCap` / `ProgramCaps` field and **no** capability token appear in
  the host request. ⭐ Both halves are required: ambient-at-dispatch and
  explicit-in-the-effect-row is the ruling, and showing only one half is
  compatible with a hidden ambient read, which the ruling forbids.
- **AC-4 — entropy is the kernel source, and unavailability is REPRESENTED.**
  Two distinct requests do not return identical bytes, **and** when the secure
  source is unavailable the op reports unavailable rather than returning bytes
  from anywhere else. ⚠ The second half is the one that matters — the first
  passes for a userspace PRNG too.
- **AC-5 — every new op reports `RepresentedUnavailable`** (D4), and
  `PX5_PLANNED_NATIVE_TARGETS` is unchanged — assert its contents, not its
  length.
- **AC-6 — no regression, green in CI.** ⛔ Build and test **locally only
  scoped** (`scripts/ken-cargo -p ken-host`, `-p ken-interp`, `-p
  ken-elaborator`). ⛔ **Never `--workspace` on this box** — the full build, the
  `--locked` gate, and conformance run **in CI on GitHub** (`COORDINATION §12`).
  "No regression" here means **green in CI**, never a local workspace run.

## Contention check

- **Files:** `crates/ken-host/` + `ken-interp/` + `ken-elaborator/`. The
  suspended FNSPLIT track works in `crates/ken-runtime/` — **disjoint.**
- **Ledger axis:** this WP touches no attested source under
  `library/SOURCE-ATTESTATIONS` and no `library/` page, so it carries **no
  currency obligation.** (Ledger is generated at release points, not enforced per
  merge — `LIB-GATE-DECOUPLE`.)
- ⚠ **`ABI-A1`** promotes `ClockWallNow`'s availability and so touches the same
  family. If `ABI-A1` is released while this is live, the two meet in
  `effect_v1.rs`'s availability match. **Do not run them concurrently.**

## Do-not-reopen guardrails

- ⛔ Whether the ABI should offer a monotonic clock at all — Track S settles it.
- ⛔ Whether to reuse `ClockWallNow` for deadlines — ruled out by D1.
- ⛔ Whether a userspace PRNG is acceptable — ruled out by D3.
- ⛔ Entropy ambient-vs-capability-gated, and whether cancellation lands here —
  **both settled by `dec_50pzvb14nnbt0`.** Runtime proceeds under the resolved
  decision; no design choice remains open in this WP.
- ⛔ Promotion of any new op to `NativeTested` — that is Track A, with its own
  evidence gate.
