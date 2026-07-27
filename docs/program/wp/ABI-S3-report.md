# `ABI-S3` — implementation report

Companion to `docs/program/wp/ABI-S3-monotonic-clocks-deadlines-entropy.md`.
Authored by `runtime-implementer`. This file exists to discharge **deliverable
6** — the design rulings this WP was built on must be durable artifacts rather
than channel messages — and to record the census corrections the frame's own
instrument could not produce.

## The rulings this WP was built on

⛔ These are settled. Do not re-litigate them; if one is wrong, that is a new
ruling, not a re-reading of this file.

### D2 — cancellation stays in `PX12`

**Ruled by the Architect**, decision **`dec_50pzvb14nnbt0`** (resolved).

`ABI-S3` lands an **uncancellable typed sleep** over a stable deadline value:

- the operation is `SleepUntil(Deadline)`;
- `Deadline` denotes an **absolute monotonic-clock deadline** — not a wall-clock
  instant, not an untyped scalar, not a comment on a duration field;
- an already-reached deadline **completes immediately**; ordinary
  interruption/error reporting stays distinct from cancellation;
- `ABI-S3` adds **no** placeholder cancellation field, token, status, or
  semantics.

`PX12` owns cancellation because it owns the readiness resources and the
operation lifecycle that make cancellation meaningful. It composes this same
`Deadline` value unchanged into its typed wait/cancellation control.

### D3 — secure kernel entropy is ambient at the host-dispatch layer

**Ruled by the Architect**, decision **`dec_50pzvb14nnbt0`** (resolved), in
answer to the Steward's escalation.

Entropy joins the V1 ambient host class: **no `EntropyCap`, no `ProgramCaps`
field, no capability token in the request.** The reason is structural rather
than convenient — kernel entropy exposes no pre-existing scoped object and has
no meaningful authority dimension to attenuate, so a Boolean capability would
duplicate the admission already carried by the declared effect row, which is the
proliferation `docs/PRINCIPLES.md` §7 rejects.

> **Ambient does not mean hidden or pure.** The operation remains an explicit
> `Entropy` effect in the program's type and trace. Code without that effect
> cannot perform it. The ruling grants no ambient FS/network/resource authority.

The trust boundary remains strict, and each clause is carried by code:

| clause | where it is enforced |
|---|---|
| kernel CSPRNG only | the process handler reads `/dev/urandom` directly |
| bounded requested byte count | `MAX_ENTROPY_REQUEST_BYTES_V1`; an over-large request is **refused, not truncated** |
| no userspace PRNG, seeded substitute, cached weak source, or silent fallback | the backend trait method **defaults to the unavailable outcome**, so an unwired backend cannot return a substitute buffer |
| unavailable yields the exact unavailable outcome and zero bytes | `ac4_entropy_reports_unavailable_rather_than_supplying_bytes_from_elsewhere` |
| catalog state stays `RepresentedUnavailable` | D4; `ac5_new_operations_are_unavailable_and_the_promotion_set_is_untouched` |

Returned bytes are a runtime observation, not a kernel proof of
unpredictability or a broader cryptographic guarantee.

### F-3 — `Entropy` joins the closed ambient sum (Option A)

**Ruled by the Architect**, decision **`dec_5vbz7tvc5dcdw`** (resolved), after
`runtime-implementer` surfaced that D3 settled the dispatch layer but did not
select whether `Entropy` belongs to the surface `AmbientOp` algebra.

`EntropyOp` and `entropy_resp` are defined and included in the **one closed
`AmbientOp` semantic sum** alongside `ConsoleOp` and `ClockOp`. `HostIO` remains
`FSOp` plus that single ambient sum. A nested binary coproduct is an acceptable
representation, but named family injection, response, and elimination helpers
are centralized: **production callers must not spread raw `InL`/`InR`
topology.**

The **Steward** sized the resulting whole scope as **L** and directed that the
topology land closed in `ABI-S3` — no deferral, no re-slicing.

## Why the D1 types are distinct

D1 says the wall and monotonic clocks are "not interchangeable at any layer."
That is taken literally rather than as prose:

```
data MonotonicInstant = MkMonotonicInstant Int
data Deadline         = MkDeadline MonotonicInstant
```

`MonotonicInstant` is deliberately **not** `Instant`. Had the monotonic read
answered `Instant` — the wall clock's response type — a wall reading would
type-check wherever a monotonic deadline is required, re-admitting at the type
layer precisely the defect D1 exists to prevent. The separation is carried at
four layers: the surface types, `CanonicalReplyV1`, the wire tag, and
`decode_deadline`, which fails closed on an `Instant`-shaped value.

## Census corrections — the frame's table is not a bound

The frame measured its 10-file/32-site surface by enumerating `ClockWallNow`,
and says to use it "as a checklist, not as a bound." That instruction was
load-bearing: **five required sites are outside it**, in three distinct
directions.

The obligation here was instead derived from the **compiler** — a throwaway
variant added to `HostOpV1` / `CanonicalRequestV1` / `CanonicalReplyV1`, with
scoped `check --all-targets` re-run until clean so masked sites surfaced.

**The complete compiler-policed surface for a new host op is five sites:**

| site | over |
|---|---|
| `ken-host/src/effect_v1.rs` `transfer_request_bound` | `CanonicalReplyV1` |
| `ken-host/src/effect_wire.rs` `put_request` | `CanonicalRequestV1` |
| `ken-host/src/effect_wire.rs` `put_reply` | `CanonicalReplyV1` |
| `ken-elaborator/src/export.rs` `host_operation_family` | `HostOpV1` |
| `ken-elaborator/src/export.rs` `canonical_host_perform_signature_v1` | `HostOpV1` |

⛔ **`ken-interp` has zero.** Every other match on `HostOpV1` is a wildcard or a
`matches!`. An operation added to the enum and wired nowhere else **compiles
clean and is silently inert** — every default is individually fail-closed, and
collectively that means `AC-5` passes *vacuously* for an operation that does not
work. `AC-5` is therefore not evidence of reachability; the dispatch test is.

### Direction 1 — visible to the grep, absent from the table

- **`crates/ken-host/effect_abi_v1.catalog`** is an 11th file and is
  load-bearing: `build.rs` parses it, asserts the op count, and hashes it into
  the `HOST_EFFECT_ABI_V1` attestation that `abi_v1.rs` binds to
  `HostOpV1::ALL`. Every new op needs a catalog row.
- The genuine op-count literal population is **four**, not the two the frame's
  deliverable 5 names: `effect_v1.rs` (`ALL` arity **and** the count assert),
  `build.rs`, and `abi_v1.rs`.
- Two `22`s are **false positives**, recorded so they are not re-chased:
  `effect_wire.rs` (`remaining()/22`, a record-size bound) and `lib.rs`
  (`"22-member producer"`, the ABI *fact* inventory).

### Direction 2 — structurally invisible to the grep

The surface vocabulary is spelled `WallNow` / `ClockOp` / `Clock`, never by Rust
identity, so these files contain **zero** `ClockWallNow` occurrences:

- **`ken-elaborator/src/prelude.rs`** — the Ken effect algebra itself.
- **`ken-elaborator/src/program_admission.rs`** — a closed granted effect row
  `["Console", "Clock", "FS"]`. Any program whose `main` declared `Entropy`
  would have been rejected `UnsupportedEffectRow`.

### Direction 3 — invisible to the compiler probe as well

**`ken-elaborator/src/compiler_driver.rs`** carries a constructor-name →
`HostOpV1` table that the erasure admission check consults. It is a `for` loop
over string literals, not a `match`, so **no exhaustiveness gate covers it** and
the compiler probe could not surface it either.

⚠ This shipped as a real regression and was caught only by a **`ken-verify`**
differential that builds a native artifact — the **4th crate**, outside the
frame's three-crate build scope. Scoping validation to the frame's crates would
have shipped it.

## `F-2` — one exhaustive family authority

Three sites replicated a hand-rolled classifier:

```rust
if op == ClockWallNow   { clock_family }
else if op.is_ambient() { console_family }
else                    { fs_family }
```

It was correct only by coincidence of the landed op set: `is_ambient()` was true
for exactly the four console ops plus `ClockWallNow`, so after the special-case
the middle branch happened to mean "Console." **Every operation this WP adds
defeats it** — a monotonic read is misfiled whichever ambient answer it takes,
and D3 makes entropy ambient, so entropy would be misfiled as Console.

Replaced with a sealed `HostOpFamilyV1` matched **with no catch-all** at every
consumer, with `host_operation_family` derived from it so there is one authority
rather than two.

★ **It paid for itself inside this same WP.** Adding `Entropy` to the sealed
enum turned all three sites into compile errors naming the missing route. Under
the `if`/`else` cascade there would have been no error at all.

## Acceptance criteria

| AC | discharged by | notes |
|---|---|---|
| AC-1 | `ac1_every_new_request_survives_an_encode_decode_round_trip` | encodings asserted mutually distinct, so a codec collapsing every request cannot pass; wall ≠ monotonic on the wire |
| AC-2 | `ac2_monotonic_readings_survive_a_wall_clock_step_backwards` | positive control asserted **first**; **mutation-proven both halves** |
| AC-3 | `ac3_the_deadline_a_caller_passes_is_the_deadline_honoured` | demonstrated by use, with a discriminator |
| AC-4 | `ac4_entropy_reports_unavailable_rather_than_supplying_bytes_from_elsewhere` | **mutation-proven** against a silent downgrade |
| AC-5 | `ac5_new_operations_are_unavailable_and_the_promotion_set_is_untouched` | asserts `PX5_PLANNED_NATIVE_TARGETS` **contents**; carries an explicit vacuity note |
| AC-6 | CI | local runs are crate-scoped only (`COORDINATION §12`) |

### Mutation evidence

Each mutation was applied at its production site and restored byte-identically,
verified with `git diff --quiet` (⚠ `--stat` always exits 0):

| mutation | expected | observed |
|---|---|---|
| monotonic script made decreasing | AC-2 **property** fires | property assertion fired; control did **not** |
| wall clock no longer steps backwards | AC-2 **control** fires | control fired with its own message |
| unavailable entropy falls back to `unwrap_or_default()` | AC-4 fires | fired, naming the observed `Success(Bytes([]))` |

The first two matter as a **pair**: they show the control and the property are
independently live and *distinguishable*, so a green AC-2 cannot be green
because the harness silently lost its ability to perturb the wall clock.

## Scoped validation

Per `COORDINATION §12`, local runs are crate-scoped; the workspace build, the
`--locked` gate, and conformance run **in CI**, never on the dev box.

```
scripts/ken-cargo test -p ken-host          53 passed
scripts/ken-cargo test -p ken-elaborator  1076 passed
scripts/ken-cargo test -p ken-interp       168 passed
scripts/ken-cargo test -p ken-verify        24 passed
                                          ---- 
                                          1321 passed, 0 failed
```

⚠ **`ken-verify` is a 4th crate the frame's build guidance does not name.** It
implements the interpreter's `HostHandler`, and it is the only place the
constructor-admission table is exercised end to end. It must stay in this WP's
validation set.

## Contention

Unchanged from the frame: `crates/ken-host/`, `ken-interp/`, `ken-elaborator/`,
plus `ken-verify/`. The suspended `RT-NATIVE-FNSPLIT` track works in
`crates/ken-runtime/` and is **disjoint** — it was neither read nor touched.
`ken-runtime` references `HOST_EFFECT_ABI_V1_HASH` symbolically, so the
attestation change reaches it without a source edit.

⚠ **`ABI-A1`** promotes `ClockWallNow`'s availability and meets this WP in
`effect_v1.rs`'s availability match. They must not run concurrently.
