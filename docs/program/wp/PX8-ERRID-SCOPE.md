# PX8-ERRID-SCOPE — reaching evidence for the five unreached PR-C rows

**PR-C's row is a *partial*: five of its ten arms already reach independently.
This closes the remaining five. The scoping question that blocked it is
answered — all five are inside, none outside — so this is ordinary evidence
work with an unusually exact specification.**

**Owner:** Team Verify (`verify-leader` + `verify-implementer` + `verify-qa`).
**Branch:** `wp/PX8-ERRID-SCOPE`. **Size:** L.
**Risk:** medium-high — four of the five need a *real* producer, and the
tempting shortcut is banned by name in `§4`.

**Status:** Steward frame, shovel-ready.
⛔ **Blocked on [[PX8-ERRID-ALLOC]]** (Foundation, in flight) — row 3 has no
identity to assert against until it merges.

⭐ **On the Linux ABI I critical path** — one of `PX8`'s three blockers. `PX8`
gates 15 of that program's 19 nodes.

---

## 1. Fixed inputs

| path | blob at `origin/main = 012aa56d` |
|---|---|
| `conformance/behavioral/buffer-io/seed-buffer-io.md` | `0364b230742e08f67fc59a2c2421221744b051e0` |
| `spec/30-surface/38-ffi-io.md` | `56c3b3d5f1090f8920cc66286e0d7ba3729f0113` |
| `crates/ken-host/src/effect_v1.rs` | `374356f36c69ffc7af0270c07efc86304850aee6` |

⚠ **`38-ffi-io.md` is LOCKED** and `conformance/` is the enclave's. Both are
**oracles**, not edit targets (`§7`).

⚠ **`effect_v1.rs` moves under `PX8-ERRID-ALLOC`** — it pins this same blob.
Re-derive at pickup.

---

## 2. ⭐ Premise correction — the row is a partial, and the locator was stale

The node cites `seed-buffer-io.md:619-645`. **That range is PR-B.** The real
row is **`buffer-io/transfer-failures-remain-errors` at `:653-676`**, and its
status is not "nothing reaches":

> `status: **RED-UNTIL-REMAINING-PR-C-ARMS — partial reaching evidence only**`

**Five arms already reach independently**, and the seed names where:

| arm | reaching evidence |
|---|---|
| `Closed`, `ResourceKindMismatch`, `RightNotHeld`, `BufferLimit` | `effect_v1.rs:3764` `bounded_positioned_io_reaches_progress_mismatch_and_ordered_bindings` |
| `Interrupted` (after a successful prefix) | `crates/ken-verify/tests/px8f_write_partition.rs:339` `checked_write_all_reaches_full_short_zero_progress_flip_and_error_prefixes` |

⇒ ⭐ **Read both before writing anything.** They are the shape this WP extends —
a real producer driven to a real failure, asserted at the exact identity. You
are adding five arms to an established pattern, not inventing one.

### ⛔ And the row flips GREEN only when ALL five land

The status string is `RED-UNTIL-REMAINING-PR-C-ARMS`. **Four of five leaves the
row exactly as red as it is today**, with no visible progress and a clause whose
whole point is universality quietly half-closed. ⇒ ⛔ **Do not deliver a
subset.** If one row proves unreachable, that is a `§9` hard stop with a named
mechanism — not a partial delivery.

---

## 3. The five rows — Architect `evt_6tzss92ckj2by`, ⛔ settled inputs

**Partition: inside = all five; outside = none.** *"There is no lawful normative
trim of these failure classes from PX8's clause-(a) closure. The boundary is
error-versus-progress, not 'errors convenient to induce.'"*

### 1. `MalformedResource`
A real `ResourceErrorV1` identity emitted by **resource resolution, before**
positioned backend I/O. ⭐ Because checked `BufferHandle`/resource constructors
are sealed, the honest reaching seat **may be the production runtime/ABI
boundary** rather than a forged checked Ken value — but it must still traverse
the real positioned operation and the real result reifier.
⛔ **A malformed ABI envelope dying as a terminal decode error is not this row.**

### 2. `InvalidBounds`
Must reach the **actual read/write consumer**, return exact `InvalidBounds`,
produce **no** progress value, and — where the fault is pre-I/O — **prove the
backend was not invoked.**
⛔ An unrelated buffer-allocation bounds rejection does not discharge the
positioned row. ⚠ `InvalidBounds` appears at several seed sites (`:477`, `:480`,
`:496`, `:617`); ⭐ only the **positioned** consumer discharges this one.

### 3. allocation failure ≠ `BufferLimit`
⛔ **Blocked on [[PX8-ERRID-ALLOC]]**, which delivers the nullary
`AllocationFailed` identity and the fallible allocation path. Assert the new
identity through the production route that WP builds.
⛔ Do not re-derive its precedence rule here — `BufferLimit` keeps precedence
for policy/representability rejection; only an *admitted* allocation that cannot
reserve storage reaches allocation failure.

### 4. unsupported / nonblocking posture
Inside **only** as the synchronous rejection identity: exact
`ResourceHostIO Unsupported` (`IoErrorIdentityV1::Unsupported`,
`effect_v1.rs:2086`) returned by the **real synchronous positioned backend** and
preserved as an error.
⛔ **PX8 gains no nonblocking mode and no retry/status result.**
⭐ **Also retain a NEGATIVE assertion** that `WouldBlock` is absent from PX8's
progress vocabulary (the seed states this at `:673`).
⛔ A test that manufactures `WouldBlock` or adds a nonblocking input belongs to
**PX12** and does not discharge this row.

### 5. host-I/O failure ≠ `Interrupted`
Reach a **different stable named** backend identity through the real positioned
path — prefer write-side `ResourceHostIO BrokenPipe`
(`IoErrorIdentityV1::BrokenPipe`, `:2079`) or another exact named `IOError`.
⛔ **Not** an undifferentiated harness failure.
⚠ If exercised after a successful `writeAll` prefix, the exact prefix is
preserved and no bytes past it are claimed.

---

## 4. ⛔ What the closure claim may NOT count — Architect, verbatim

> *enum construction, a hand-fed error, a terminal ABI-decode failure, or one
> engine's result as the other engine's oracle.*

⇒ **Each row must bind its real producer to the real interpreter/native
reification route and assert the exact locked-spec identity independently.**

⭐ **This list is the WP.** Every one of the four is a *cheaper* way to make a
test pass than the real route, and each produces a suite that looks discharged
and measures nothing. `AC-2` is the control that separates them.

---

## 5. Deliverables

- **`D1`–`D5`** — one production-reaching arm per row of `§3`, each asserting
  the exact identity, each proving no progress value is constructed.
- **`D6`** — the `WouldBlock`-absent negative assertion (row 4).
- **`D7`** — for each arm, a written statement of **how** the failure is
  induced and **why that route is the production route**. ⭐ This is what makes
  `§4` auditable by a reviewer who did not write the test.
- **`D8`** — a statement of whether the seed row's status string can now flip,
  and any locator drift found (⚠ the node's own `:619-645` was wrong — `§2`).
  ⛔ Report drift; do not repair `conformance/`.

---

## 6. Acceptance criteria

- **`AC-1`** — all five arms reach. **Control:** each arm names its exact
  identity in an assertion; ⛔ four-of-five fails this AC (`§2`).

- **`AC-2`** ⭐⭐ **(load-bearing — this is the whole WP)** — every arm's
  failure is **induced through the production path**. **Control:** for each
  arm, mutate the *production* code that produces the identity and show that
  arm reddens. ⛔ An arm that stays green when its producer is mutated is
  measuring a hand-fed value and fails, whatever it asserts.

- **`AC-3`** — no arm constructs progress. **Control:** each asserts the result
  is an error carrying its own identity, and that **no** `ReadProgress` /
  `WriteProgress` value exists on that path.

- **`AC-4`** — row 2's pre-I/O arm proves the **backend was not invoked**.
  **Control:** a backend-visit observation at zero, not merely an error
  assertion. ⭐ "It returned the right error" is compatible with having called
  the backend first.

- **`AC-5`** — row 4's boundary holds in both directions. **Control:**
  `Unsupported` is asserted positively from the real synchronous backend, **and**
  `WouldBlock` is asserted absent. ⛔ Adding a nonblocking input to produce
  either fails this AC and is PX12's scope.

- **`AC-6`** — row 5 is distinct from the already-green `Interrupted` arm.
  **Control:** the identity asserted is **not** `Interrupted`, and the existing
  `px8f_write_partition.rs:339` arm stays green and unmodified.

- **`AC-7`** — no oracle is another engine. **Control:** ⛔ no assertion
  compares an interpreter result to a native result as its expectation; each
  asserts against the locked `38` identity directly.

- **`AC-8`** — scope. **Control:** `git diff --name-only` shows no path under
  `spec/` or `conformance/`.

- **`AC-9`** — targeted green. **Control:** name the exact
  `scripts/ken-cargo test -p <crate>` invocations and pass counts. ⛔ No
  `--workspace` (`COORDINATION §12`) — workspace-green means **green in CI**.
  ⚠ Re-derive build-slot availability first; `ken-cargo` blocks silently for up
  to 30 minutes and `fuser -v /tmp/ken-build-locks/build.lock` names the
  holder — ⛔ don't pipe it through `head`.

---

## 7. ⛔ Banned scope

- ⛔ **No `spec/` edit** — `38-ffi-io.md` is LOCKED.
- ⛔ **No `conformance/` edit** — the seed is the enclave's. Report drift under
  `D8`; ⭐ editing a cited source moves its OID.
- ⛔ **No nonblocking mode, no retry, no status result** — PX12 (`§3` row 4).
- ⛔ **Do not re-litigate the partition.** All five are inside; the Architect
  ruled it and no operator narrowing is needed.
- ⛔ **Do not implement `AllocationFailed`** — that is `PX8-ERRID-ALLOC`.
- ⛔ **No `--workspace` run.**

---

## 8. Contention

`crates/ken-host/src/effect_v1.rs` — ⚠ **`PX8-ERRID-ALLOC` is live on this file
right now** and this WP is gated behind it anyway. Re-measure at pickup; ⛔ if
you find another live contender, stop and route rather than coordinating a
shared edit.

⭐ The rest of the fleet is winding down under an operator directive, so
contention should be falling.

---

## 9. Hard stop

⛔ Route to the Steward if:

- an identity **cannot be produced** by any real route — ⭐ say which, and name
  the mechanism that prevents it. **This is a full deliverable, not a failure:**
  it is the same class as `CONF-FMT8-LEVELTOK` and `CONF-SEC4-REFL-PAIR`, where
  a required operand could not be constructed. ⚠ A row whose operand is
  unconstructible is byte-identical, to any reader, to one not yet built —
  ⛔ so do not leave it silently undone; **or**
- row 4's `Unsupported` cannot be reached from the real synchronous backend on
  this platform without adding a nonblocking input — ⛔ do not add one; **or**
- row 1's honest reaching seat turns out to require a checked-value forgery —
  ⛔ that reopens the sealed-constructor boundary and is not yours to decide;
  **or**
- discharging a row would require editing `conformance/` or LOCKED `38`.
