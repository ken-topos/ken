# PX8-ERRID-ALLOC — an allocation-failure identity that production can emit

**`ResourceErrorV1` is a closed sum with no allocation-failure identity, and
buffer allocation is `vec![0; capacity]` — infallible, aborting on exhaustion.
PX8's clause-(a) closure requires a row for allocation failure *distinct from*
`BufferLimit`. That row is not untested; it is unproducible.**

**Owner:** Team Foundation (`foundation-leader` + `foundation-implementer` +
`foundation-qa`). **Branch:** `wp/PX8-ERRID-ALLOC`. **Size:** M.
**Risk:** medium — widens a closed sum that crosses the wire/ABI surface.

**Status:** Steward frame, shovel-ready, released.
⭐ **On the Linux ABI I critical path** — prerequisite to `PX8-ERRID-SCOPE`,
one of `PX8`'s three blockers. `PX8` gates 15 of that program's 19 nodes.

---

## 1. Fixed inputs

| path | blob at `origin/main = e754508b` |
|---|---|
| `crates/ken-host/src/effect_v1.rs` | `374356f36c69ffc7af0270c07efc86304850aee6` |
| `spec/30-surface/38-ffi-io.md` | `56c3b3d5f1090f8920cc66286e0d7ba3729f0113` |
| `conformance/behavioral/buffer-io/seed-buffer-io.md` | `0364b230742e08f67fc59a2c2421221744b051e0` |

> ✅ **RE-VERIFIED CURRENT at `origin/main = dca1b793` (Steward, 2026-07-28).**
> All three blobs above are **byte-identical** on `main` — ⭐ this frame has not
> rotted, and `§2`'s line anchors are measured against a live tree. ⛔ You still
> re-derive on your own base and yours wins; this stamp only means **no
> re-framing pass is owed before you start.**

⚠ `38-ffi-io.md` is **LOCKED**. ⛔ Editing it is `§4`-banned; if the identity
needs a normative home, that is a routed spec change, not this WP's edit.

---

## 2. The measurement

`effect_v1.rs:592-613` — the closed sum:

```
Closed · MalformedResource · ResourceKindMismatch · RightNotHeld
ReleaseFailed · BufferLimit · InvalidOffset · InvalidBounds · NoProgress
```

- `:661` — `bytes: vec![0; capacity]`. **Infallible.** On exhaustion the process
  aborts; no value is ever returned.
- `:829`, `:834` — `BufferLimit` is returned for per-buffer and live-capacity
  **policy/width admission**.

⇒ **Two separate gaps, and both must close for the row to be producible:**
an identity to return, and a path that can return it.

---

## 3. ⭐ Steward-discharged — the Architect ruled the semantics, not just the gap

`evt_6tzss92ckj2by`. ⛔ **Do not re-litigate these; they are settled inputs.**

### 3a. The shape

> *"the direct shape is a nullary `AllocationFailed`, subject to the normal
> Spec/CV spelling lane"*

⭐ **Nullary.** ⛔ Do not attach an errno, a size, or a context payload — that
turns an engine-neutral identity into a host-specific one, which is the thing
prohibition (1) below rules out.

### 3b. ⛔ Three named prohibitions

1. ⛔ **Not** `ResourceHostIO Other(errno)`. Allocation failure is not host I/O.
2. ⛔ **Not** aliased to `BufferLimit`. Policy refusal is not exhaustion.
3. ⛔ **Not** a synthetic error production cannot emit. ⭐ This is the whole
   point of the WP — an identity that only tests can construct leaves the row
   exactly as unproducible as it is today, while *looking* discharged.

### 3c. ⭐ Precedence is ruled, and it is testable

> *"`BufferLimit` retains precedence for deterministic policy/representability
> rejection; only an admitted allocation that cannot reserve storage reaches
> allocation failure."*

⇒ A request that violates policy **must** still return `BufferLimit`, even
under memory pressure. **That ordering is `AC-3` and it is the AC most likely
to be got wrong**, because the natural implementation checks allocation first.

### 3d. Ordering against resource minting

> *"fallible allocation that returns it **before minting a resource or
> incrementing live capacity**"*

⇒ A failed allocation must leave the resource table and the live-capacity
accounting **byte-identical to the pre-call state**. ⭐ That is observable and
is `AC-4`.

---

## 4. ⛔ Banned shapes

- ⛔ **Do not edit `spec/`.** `38-ffi-io.md` is LOCKED. If you conclude the
  identity needs normative text, **stop and route** (`§8`) — that is the
  Spec/CV lane, not yours.
- ⛔ **Do not edit `conformance/`.** The seed is the enclave's.
- ⛔ **Do not write the five production-reaching evidence rows.** That is
  `PX8-ERRID-SCOPE` and it depends on this WP. ⭐ Delivering the identity is
  the whole job; proving all five identities reachable is the next one.
- ⛔ **Do not make allocation fallible by adding a test-only injection point
  that production never takes.** See `§3b(3)` — the mechanism must be the real
  one, with the test controlling its input, not a parallel path.
- ⛔ **No `--workspace` run.** Targeted only (`COORDINATION §12`).

---

## 5. Deliverables

- **`D1`** — `AllocationFailed` (nullary) added to `ResourceErrorV1`, with its
  checked-Ken binding following the landed `generated_binding("error",
  "resource.BufferLimit")` pattern at `effect_v1.rs:397`.
- **`D2`** — buffer allocation made **fallible**: the `vec![0; capacity]` at
  `:661` replaced by a reservation that can fail and return `D1`'s identity
  **before** any resource is minted or live capacity incremented.
- **`D3`** — a **production-reachable** test that drives `D2`'s real failure
  path and observes `AllocationFailed` surface as a checked value. ⭐ Say in
  the report **how** the test induces failure and why that route is the
  production route.
- **`D4`** — a written statement of the wire/ABI consequence of widening a
  closed sum: what a decoder that predates the variant does with it, and
  whether any version/schema field needed to move. ⛔ An empty answer is a
  failed measurement, not a clean result.

---

## 6. Acceptance criteria

- **`AC-1`** — `AllocationFailed` is nullary and engine-neutral. **Control:**
  show the variant declaration; ⛔ any payload fails this AC.

- **`AC-2`** ⭐ **(load-bearing)** — the identity is emitted by **production**,
  not by a test. **Control:** the `D3` test must reach the identity through the
  same code path a real exhaustion would take. ⛔ A test that constructs
  `ResourceErrorV1::AllocationFailed` and asserts on it measures nothing and is
  exactly prohibition `§3b(3)`.

- **`AC-3`** ⭐ **(the one most likely to be got wrong)** — **precedence
  holds.** **Control:** a request that violates per-buffer or live-capacity
  policy returns `BufferLimit`, **not** `AllocationFailed`, and this must hold
  even when allocation would also fail. ⛔ If the implementation checks
  allocation before policy, this AC fails and the two identities become
  order-dependent.

- **`AC-4`** — a failed allocation is **atomic**. **Control:** assert the
  resource table and live-capacity accounting are unchanged from the pre-call
  state after a failure. ⭐ A leaked capacity increment on the failure path is
  the defect this AC exists to catch.

- **`AC-5`** — no existing `ResourceErrorV1` behavior changed. **Control:**
  the landed `BufferLimit` rows stay green, and `git diff` shows no edit to the
  other variants' producers.

- **`AC-6`** — scope. **Control:** `git diff --name-only` shows no path under
  `spec/` or `conformance/`.

- **`AC-7`** — targeted green. **Control:** name the exact
  `scripts/ken-cargo test -p <crate>` invocations you ran and their pass counts.
  ⚠ Re-derive build-slot availability first; `ken-cargo` blocks silently for up
  to 30 minutes on lock contention, and `fuser -v /tmp/ken-build-locks/build.lock`
  names the holder — ⛔ don't pipe it through `head`.

---

## 7. Contention

`crates/ken-host/src/effect_v1.rs` — ⚠ **re-measure at pickup.** The Kernel
ring's `KERNEL-NESTED-IND` work has touched dependent-host code this session.
⛔ If you find a live contender on this file, **stop and route**; do not
coordinate a shared edit yourself.

⭐ The rest of the fleet is winding down under an operator directive, so
contention should be falling, not rising.

---

## 8. Hard stop

⛔ Route to the Steward if:

- the identity appears to need normative text in LOCKED `38` — that is the
  Spec/CV lane and I will route it; **or**
- allocation cannot be made fallible without a test-only injection point — ⭐
  say so with the mechanism that blocks it rather than adding one; **or**
- `AC-3`'s precedence cannot be maintained without reordering existing
  admission logic in a way that changes a landed `BufferLimit` result; **or**
- widening the closed sum turns out to break a decoder or schema contract that
  `D4` cannot resolve inside this WP's scope.
