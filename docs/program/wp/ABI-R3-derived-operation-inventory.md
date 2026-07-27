# ABI-R3 — an operation inventory derived from the enum, so a new operation is a build break

**The catalog's closure is real, and it is closed against `HostOpV1::ALL` — a
hand-maintained array that nothing ties to the `HostOpV1` enum. Adding a variant
to the enum alone changes no test, breaks no build, and makes the operation
invisible to every inventory at once.**

**Owner:** Team Runtime (`runtime-leader` + `runtime-implementer` +
`runtime-qa`). **Branch:** `wp/ABI-R3`. **Size:** M.
**Risk:** medium — small diff, but it edits the file every ABI consumer reads.

**Status:** Steward frame, shovel-ready.
⛔ **Blocked** — `depends_on: [PX8]`, and see `§8` for the live contention that
is the *real* constraint.

⭐ **`10-linux-abi-completion.md §7` calls this the load-bearing node of Track
R.** Every later WP in Tracks A, M, S, and T adds operations. Landing this first
means each of them extends a derived structure instead of re-litigating a
hand-maintained list.

---

## 1. Fixed inputs

| path | blob at `origin/main = 012aa56d` |
|---|---|
| `crates/ken-host/src/effect_v1.rs` | `374356f36c69ffc7af0270c07efc86304850aee6` |
| `crates/ken-host/build.rs` | `8357c0128469b0b32d05788c8485eec2e1386795` |
| `crates/ken-host/effect_abi_v1.catalog` | `b3da02d13e11838f95e2d975a3b18c8c5965bb9f` |
| `crates/ken-verify/src/catalog.rs` | `31936e8c5b4796e665c9c4d99833b2c78a5e98ca` |

⚠ **`effect_v1.rs` is being edited right now** by Foundation under
`PX8-ERRID-ALLOC` — it pins this exact blob. **Re-derive all four at pickup.**

---

## 2. ⭐⭐ The measurement — what is closed, and what the closure is closed *against*

**Give the landed work its due: the cross-checks are real.** `effect_v1.rs`'s
`generated_manifest_closes_catalog_observer_and_consumer_sets` compares the
build-script-generated `HOST_EFFECT_ABI_V1_CATALOG` (produced by `build.rs:288`
from the data file `effect_abi_v1.catalog`) against the Rust registry, and
`ken-verify/src/catalog.rs`'s `imported_catalog_partition_is_exact_and_closed`
proves the native/deferred partition is exact, disjoint, and covers everything.
Adding an operation to the data file but not to Rust **does** fail. The reverse
**does** fail. Those controls work.

⭐ **But every one of those sets is derived from `HostOpV1::ALL`**, and
`HostOpV1::ALL` is a hand-written array (`effect_v1.rs:44`):

```rust
pub const ALL: [Self; 25] = [ Self::ConsoleRead, /* …24 more, by hand… */ ];
```

⇒ **Add a variant to the `HostOpV1` enum and stop there.** The enum compiles.
`ALL` still has 25 elements and still type-checks. Every downstream set agrees
with every other downstream set — because they are all downstream of `ALL`, and
`ALL` never heard about the new variant. **Nothing anywhere fails.**

⭐ **The enum is the unwritten surface.** Two exhaustive searches, each complete
against its own notion of what it was enumerating.

### The three silent defaults this leaves

| site | shape | a new variant silently gets |
|---|---|---|
| `effect_v1.rs:73` `availability()` | `matches!(…) { … } else { … }` | `RepresentedUnavailable` |
| `effect_v1.rs:97` `is_ambient()` | `matches!(…)` | `false` |
| `effect_v1.rs:44` `ALL` | hand-written array | absent entirely |

⛔ **None of these is a compile error.** Each is a plausible-looking default,
which is what makes them survive review.

### ⭐ And the correct mechanism is already in the same file

`effect_v1.rs:574` — `FsOpenModeV1::required_right`:

```rust
pub const fn required_right(self) -> crate::RightSet {
    match self {
        Self::Read => crate::RightSet::READ,
        Self::Metadata => crate::RightSet::METADATA,
        Self::WriteCreate(_) => crate::RightSet::WRITE.union(…),
    }
}
```

**Exhaustive match, no wildcard.** Add an `FsOpenModeV1` variant and the build
breaks. ⇒ **Same file, same crate, two enums, opposite discipline.** This WP
applies the discipline that is already here to the enum that lacks it.

---

## 3. ⛔ The existing count assertion cannot fail — do not extend it, delete it

`effect_v1.rs:2916`, inside `catalog_is_closed_and_availability_is_exact`:

```rust
assert_eq!(HostOpV1::ALL.len(), 25);
```

`ALL` is declared `[Self; 25]`. **`.len()` on a fixed-size array is a
compile-time constant**, so this asserts `25 == 25`. It is a **tautology** — it
has never been able to fail and cannot be made to fail by any edit to the
catalog.

⚠ `abi_v1.rs:1765` has the same shape against `HOST_EFFECT_ABI_V1_CATALOG.len()`
(a slice, so that one *can* fail — but it is still a count).

⇒ ⛔ **Both are the exact anti-pattern `§4` of the program names:** *"Tests
assert **named** memberships and properties, never total counts."* ⭐ A count is
a proxy, and a compensating duplicate defeats it: an `ALL` that lists one
operation twice and omits another has length 25 and passes.

**`D3` removes them and replaces them with named-membership assertions.**

---

## 4. Deliverables

- **`D1`** ⭐ **the deliverable** — `HostOpV1::ALL` derived from the enum, so
  that adding a variant without extending the inventory **fails to compile**.
  ⭐ Recommended route, zero new dependencies: a `const fn` or associated
  constant built from an **exhaustive `match self`** (the `FsOpenModeV1:574`
  pattern), or a small local derive. ⛔ Any route is acceptable **provided it
  satisfies `AC-1`'s mutation**; the mechanism is not the deliverable, the
  build break is.
- **`D2`** — `availability()` and `is_ambient()` converted to exhaustive
  `match` with **no wildcard and no `else` fallback**, so each new operation
  must be classified explicitly.
- **`D3`** — the two count assertions of `§3` replaced by named-membership
  assertions over the five axes (`§5`).
- **`D4`** — the same treatment for **rights** and **request/reply schema** per
  operation, and the **differential fixture** binding, so all five axes named in
  `10-linux-abi-completion.md §4` are derived rather than restated.
- **`D5`** — a written statement of which enums in `ken-host` **still** carry
  the hand-maintained shape after this WP, with the count. ⛔ An empty answer is
  a failed census. ⭐ This is what lets the next WP be framed against a measured
  surface.

---

## 5. The five axes (program `§4`, verbatim scope)

Operation **identity**, **availability**, **rights**, **request/reply schema**,
and **differential fixture per operation**. ⇒ Each must be reachable *from the
enum*, and each must be a build break to omit.

⚠ Some axes may not be expressible as a build break without restructuring more
than this WP should touch. ⭐ **If so, say which axis and what blocks it** —
that is a real deliverable, not a shortfall. ⛔ Do not silently deliver three of
five; see `§9`.

---

## 6. Acceptance criteria

- **`AC-1`** ⭐⭐ **(load-bearing — this is the whole WP)** — **adding an
  operation is a build break.** **Control:** add a throwaway 26th variant to
  `HostOpV1`, change nothing else, and show `scripts/ken-cargo build -p
  ken-host` **FAILS**. Then remove it and show green.
  ⭐ **Run this control against the CURRENT tree first and report that it
  PASSES** — that is the defect, measured, and it is what proves your control
  discriminates. ⛔ A control you only ever ran against the fixed tree cannot
  tell a real repair from a no-op.

- **`AC-2`** — the silent defaults are gone. **Control:** with the 26th variant
  present, the failure is a **`non-exhaustive patterns` compile error** naming
  `availability` and `is_ambient` — ⛔ not a runtime assertion, and ⛔ not a
  default classification.

- **`AC-3`** — no count assertions survive on the inventory. **Control:**
  `grep -n 'ALL.len()\|CATALOG.len()' crates/ken-host/src/` returns no
  assertion site. ⚠ `effect_v1.rs:166` uses `CATALOG.len()` to compute
  `operation_count` — that is a **value**, not an oracle, and stays.

- **`AC-4`** — the replacements assert **named** memberships. **Control:** each
  new assertion names specific operations (e.g. `FsReadAt`, `BufferAllocate`)
  and would still be meaningful if the catalog grew. ⛔ An assertion whose only
  content is a number fails this AC.

- **`AC-5`** — the landed closure is preserved, not replaced. **Control:**
  `generated_manifest_closes_catalog_observer_and_consumer_sets` and
  `imported_catalog_partition_is_exact_and_closed` stay green. ⭐ They are
  correct as far as they go; this WP closes the gap **beneath** them.

- **`AC-6`** — a genuine mutation, not a build break. **Control:** flip one
  operation's `availability` classification and show a **named** test reddens,
  identifying that operation. ⛔ If only a count moves, `D3` is incomplete.

- **`AC-7`** — no behavior change. **Control:** the ABI wire identities
  (`0x0101`…`0x0501`) and `operation_count` are unchanged; `git diff` shows no
  edit to `effect_abi_v1.catalog`'s rows.

- **`AC-8`** — targeted green. **Control:** name the exact
  `scripts/ken-cargo test -p <crate>` invocations and pass counts across
  `ken-host`, `ken-verify`, and `ken-elaborator` (`export.rs:1583` and
  `erasure.rs:6519` both consume `HostOpV1::ALL`). ⛔ No `--workspace`
  (`COORDINATION §12`) — workspace-green means **green in CI**.

---

## 7. ⛔ Banned scope

- ⛔ **Do not add or remove an operation.** This WP changes how the inventory is
  *derived*, never what is in it. `AC-7` is the control.
- ⛔ **Do not change any ABI numeric identity.** They are the wire contract.
- ⛔ **No `spec/` edit, no `conformance/` edit.**
- ⛔ **Do not pull in a new crate dependency** for enum iteration without
  routing it — `ken-host` sits near the trust boundary, and `D1` is achievable
  with an exhaustive match. ⭐ If you conclude a derive macro is genuinely
  better, **stop and route** with the reason.
- ⛔ **No `--workspace` run.**

---

## 8. ⚠ Contention — this is the real constraint, and it is not the DAG edge

**`crates/ken-host/src/effect_v1.rs` is live.** Foundation is editing it under
`PX8-ERRID-ALLOC` right now, widening `ResourceErrorV1`, and that WP pins the
same blob this frame does. ⇒ ⛔ **Do not start while that is in flight**, and
re-measure at pickup.

⭐ **Steward note on the `PX8 -> ABI-R3` edge.** `§7` of the program argues
every sequencing edge it asserts — `ABI-R3` before A1–A3/M1, `ABI-REVOKE`
between, `PX9` before PX10/PX11 — **except this one**. No stated rationale
links PX8's clause-(a) closure to the operation inventory, and this WP adds no
operations and needs none of PX8's behavior.

⚠ **That is an observation, not a re-wiring.** The DAG edge stands until the
Architect or the operator moves it. It costs nothing today: Runtime has
`RT-NATIVE-FNSPLIT` and then `NATIVE-HANDLE-CARRIER` queued ahead of this
regardless, and `effect_v1.rs` is contended until `PX8-ERRID-ALLOC` lands.
⇒ **Raise it only if this becomes the thing Runtime is waiting on.**

⚠ Re-derive build-slot availability at pickup. `ken-cargo` blocks silently for
up to 30 minutes on lock contention; `fuser -v /tmp/ken-build-locks/build.lock`
names the holder — ⛔ don't pipe it through `head`.

---

## 9. Hard stop

⛔ Route to the Steward if:

- an axis in `§5` cannot be made a build break without restructuring beyond
  `ken-host` — ⭐ name the axis and the mechanism that blocks it; **or**
- `AC-1`'s mutation **already fails on the current tree**, which would mean my
  measurement in `§2` is wrong and the frame's premise is false; **or**
- making `availability`/`is_ambient` exhaustive forces a classification call on
  an operation whose correct value is not obvious — ⛔ that is a design call,
  not yours to default; **or**
- a new crate dependency looks necessary for `D1` (`§7`).
