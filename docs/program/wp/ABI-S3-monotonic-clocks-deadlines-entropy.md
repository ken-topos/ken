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
| `HostOpV1` holds **22** operations | `crates/ken-host/src/effect_v1.rs:16` |
| `pub const ALL: [Self; 22]` | `effect_v1.rs:42` |
| the count is pinned by `assert_eq!(HostOpV1::ALL.len(), 22)` | `effect_v1.rs:2613` |
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
untyped scalar with a comment forces a breaking change there. Author the deadline
type so `PX12` can add cancellation **without re-shaping it**.

⚠ **What is NOT decided here and must go to the Architect:** whether
cancellation itself lands in this WP or in `PX12`. Frame the type so either is
possible; do not implement cancellation semantics on your own authority.

### D3 — entropy: settle **ambient vs capability-gated** before writing the op

`ClockWallNow` is ambient. Reading the clock and reading the kernel CSPRNG are
**not** obviously the same trust class, and `is_ambient()` is a one-line decision
that is very hard to reverse once Ken programs depend on it.

⇒ **State your answer and its reasoning in the WP, and get the Architect's
ruling before landing.** Weigh it from `docs/PRINCIPLES.md` (small auditable
TCB), not from convenience.

⛔ **"Secure kernel entropy" means the kernel CSPRNG.** No userspace PRNG, no
seeding a userspace generator and calling that the op, no fallback to a weaker
source when the primary is unavailable — an unavailable secure source is
`RepresentedUnavailable`, never a silent downgrade.

### D4 — every new op lands `RepresentedUnavailable`

The two-state availability model has no provisional value, and the code says
membership in a promotion set *"is a plan, not evidence."* Promotion is a
separate, evidence-bearing act (that is what Track A is for). ⛔ Do not add any
new op to `PX5_PLANNED_NATIVE_TARGETS` in this WP.

## Deliverables

1. **A monotonic clock read** in the `0x02` family (D1), with its wire encode +
   decode, its canonical request/response, and its availability = 
   `RepresentedUnavailable`.
2. **Sleep / deadline** with the deadline expressed as a **type** (D2),
   extensible to `PX12`'s cancellation without re-shaping.
3. **Secure kernel entropy**, ambient-vs-gated per the D3 ruling, no fallback.
4. **The full 10-file sweep** for each new op, using the table above as the
   checklist.
5. **The count pin at `effect_v1.rs:2613` and `ALL: [Self; N]` updated** to the
   new arity, with the wire tags continuing from **22**.
6. **A short note in the WP recording the D3 ruling** and who gave it, so the
   trust-class decision is durable rather than living in the channel.

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
- **AC-3 — the deadline is a value, demonstrated by use.** A caller constructs a
  deadline, passes it, and observes it honoured. ⛔ Do not discharge this by
  showing a field exists.
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
- ⛔ Promotion of any new op to `NativeTested` — that is Track A, with its own
  evidence gate.
