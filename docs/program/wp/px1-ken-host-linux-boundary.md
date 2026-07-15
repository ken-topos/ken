# WP — PX1: `ken-host`, the first-party Linux-ABI boundary over pinned `rustix`

**Owner:** Runtime ring. **Reviewer/Gate:** Runtime QA **+ Architect terminal
soundness/design gate (§14 — this moves the host trust boundary).**
**Size:** M. **Branch:** `wp/px1-ken-host`, cut fresh from `origin/main` at
kickoff. **CI:** ⛔ **FULL CI** — touches `crates/` (never `--doc-only`).
**Source:** charter `docs/program/09-posix-linux-abi-campaign.md` §4 PX1 +
FORK 2 ruling (`evt_7qqf827rr1jxk`) + Phase A exit correction; operator
Linux-direct reframe + PX1/PX2 pull-ahead (2026-07-15).

## Objective

Today the host boundary is **6 hand-declared `unsafe extern "C"` syscall
boundaries + call sites inlined in the 4,600-line pure evaluator**
(`ken-interp/.../eval.rs`), one of them hidden inside a function body, under
a `cfg` that PX0 has already narrowed to `target_os = "linux"`. There is
**no boundary crate**, and `ken-interp` does **not** forbid `unsafe`. That
is the §0 contract-expressibility defect in its rawest form.

PX1 **extracts every one of those raw boundaries into a new first-party
`ken-host` crate that wraps an exact-pinned, checksum-locked `rustix`
(`linux_raw` backend — direct to the Linux kernel, no libc, no POSIX layer).**
`ken-host` becomes the **only** public callable host boundary; `rustix` is a
**private** dependency behind it. When the extraction is complete, `ken-interp`
gets `#![forbid(unsafe_code)]`.

This is a **like-for-like consolidation of the existing surface** — the same 6
operations, the same observable behavior — re-homed behind an audited boundary
and bound to `rustix`'s typed API. **No new host operation is added** (those are
later PX). This is pure debt repayment and the enabler for all of PX-B…PX-E.

## Fixed inputs (SETTLED — do not reopen)

1. **FORK 2 is ruled (Architect, `evt_7qqf827rr1jxk`): option (b).** `ken-host`
   is a first-party **policy shell** over exact-pinned `rustix`;
   Ken-authored raw declarations are **retired**. A probe checks numbers,
   not signatures — so `rustix` (which owns the whole ABI surface:
   signatures, calling convention, errno, pointer/length coupling,
   per-target `cfg`, ownership) is the boundary. **This is settled
   component design; do not relitigate it.**
2. **`rustix` is ACCEPTED (operator, 2026-07-15) — SETTLED, never re-ask.**
   Exact-pinned, checksum-locked, private behind `ken-host`. `ken-kernel`
   stays `forbid(unsafe_code)`.
3. **Linux-ABI-direct (operator, 2026-07-15).** Target the Linux syscall
   ABI directly via `rustix`'s `linux_raw` backend. **No POSIX portability
   abstraction.** PX0 already re-gated the surface to
   `target_os = "linux"`; non-Linux targets already return a **named
   unavailable lane before any host call** — PX1 preserves that (do not
   regress it).
4. **The Phase A exit is CORRECTED (Architect, `evt_7qqf827rr1jxk`):** the
   AC is ***"no OS/host-ABI `unsafe` outside `ken-host`, and `ken-interp`
   forbids `unsafe`."*** The `cranelift_backend.rs:1059` JIT-entry
   `mem::transmute` is a **native-code-execution** boundary, **NOT** a
   host-call boundary — **it does NOT move into `ken-host`.** Laundering it
   in to make a checkbox green is the explicitly forbidden move (do-not
   guard below).
5. **ADR-0017 is settled:** `openat`-relative, handle-not-path, inode-keyed;
   resolve/operate split stays; **no byte-path bypass that can re-resolve
   after authorization** (`eval.rs:2245-2248`). `ken-host`'s API must
   preserve this — no raw path re-resolution escapes the boundary.
6. **Host guarantees are `tested`/`validated`, never `proved`** — disclosure
   in the source, per PRINCIPLES #14 and the charter.

## The backend seam — ✅ CONFIRMED (Architect, `evt_1t429wz5ehf42`)

**The seam is `ken-host`'s public SEMANTIC API — NOT a mirror of the six C
signatures.** Build **one** Linux implementation now; **do not add a
one-implementation `HostBackend` trait** (add an internal trait only when a
second real backend supplies evidence dispatch needs one). Precisely:

> The stable public surface is **Ken-owned opaque rooted-handle types, validated
> single-component/path inputs, operation enums, and typed results/errors** —
> the typed **semantic operations the call sites need** (e.g. create-policy and
> no-follow behavior), **not integer flags or C-shaped calls.** All `rustix`
> modules and types — `OFlags`, `AtFlags`, `Errno`, raw descriptors/pointers,
> backend handles — remain **private**. A private `linux` module is selected
> with `#[cfg(target_os = "linux")]`; the existing named unavailable lane stays
> explicit for non-Linux targets. *"Target-agnostic in shape"* means callers
> speak host semantics and need not change when a second backend arrives — it
> does **not** claim a non-Linux backend exists today.

This is settled framing input. The Architect's **terminal §14 design gate on the
PX1 implementation** still stands (below).

## Mandated deliverable outline (each ends in a concrete choice)

1. **⛔ PX1 PRECONDITION — the dependency-closure deliverable, BEFORE the
   `Cargo.toml` line.** Enumerate `rustix`'s **complete transitive
   dependency closure**, every license, the exact feature set (minimal —
   `linux_raw`, no `std`-libc backend, no unused features), and the
   **exercised upstream `unsafe` surface**, into
   `docs/program/dependency-deltas.md` **as a deliverable**. *Count it;
   state "there are N, here are all N"; do not inherit a number.* **If the
   closure is larger than the boundary it replaces, that is a finding — say
   so and escalate to the Steward before proceeding.** (This is the exact
   discipline PX0 exists to establish.)
2. **The `ken-host` crate + its public SEMANTIC API.** Create `crates/ken-host`.
   Expose the typed **semantic operations** the call sites need (create-policy,
   no-follow behavior, rooted-relative open/mkdir/unlink/rename/readlink, signal
   masking) — **Ken-owned opaque rooted-handle types, validated
   single-component/path inputs, operation enums, typed results/errors.** **Do
   NOT mirror the six C signatures as the public abstraction**, and **no
   `RawFd`, raw pointer, integer flag, `rustix` type (`OFlags`/`AtFlags`/
   `Errno`), or unrooted path escapes the boundary.** `rustix` is a **private**
   dependency (`pub(crate)` at most); a private `linux` module carries the impl
   under `#[cfg(target_os = "linux")]`.
3. **Move the 6 boundaries + their call sites** off inline FFI onto the
   `ken-host` API backed by `rustix`'s typed calls. Re-derive the exact
   sites from current source (they drift); the charter's grounding
   (`origin/main @ 26d5255e`) lists: the 5-fn `unsafe extern "C"` block
   (`openat`, `mkdirat`, `unlinkat`, `renameat`, `readlinkat`) at
   `eval.rs:2378-2394` with call sites
   `:2414/:2426/:2435/:2924/:2942/:2972/:2997`, and the
   `signal`/`SIGPIPE`/`SIG_IGN` block nested in `mask_sigpipe()` at
   `:3714-3730`. **Behavior-preserving:** same `openat`-relative semantics,
   same errno mapping, same `O_NOFOLLOW`/symlink policy — now via
   `rustix`'s typed `OFlags`/`AtFlags` (target-correct by `rustix`'s
   construction, so the ADR-0017 `O_NOFOLLOW` property is **source-correct**,
   not correct-only-by-prose).
4. **DELETE all 13 handwritten ABI facts — in PX1 (Architect correction,
   `evt_1t429wz5ehf42`).** Once the call sites move to `rustix`'s typed API, the
   **11 `*_KEN` constants + the local `SIGPIPE`/`SIG_IGN` facts have no valid
   consumer.** Keeping them as dead declarations until PX2 would preserve the
   very hand-authored ABI source FORK 2 retired and make the `O_NOFOLLOW` claim
   true only by prose. They are **companion spillover of the extraction —
   delete them in the same PX1 change; no production path may retain or consult
   them.** (PX2 then *measures and binds* the replacement source of truth — the
   `rustix`/system-header manifest — and does not need dead Ken constants to
   exist first.)
5. **`#![forbid(unsafe_code)]` on `ken-interp`.** After the extraction, add the
   attribute and make the crate build clean under it. If any residual
   host-ABI `unsafe` remains in `ken-interp`, it was missed in step 3 — the
   forbid attribute is the mechanical proof the extraction is complete. The JIT
   `mem::transmute` lives in **`ken-runtime/src/cranelift_backend.rs`** (verified
   on current `origin/main`), **not** `ken-interp` — it is a separate
   native-execution boundary; **do NOT move it into `ken-host`** (fixed input 4).
6. **Trust disclosure in the source.** `ken-host`'s module doc states plainly:
   this is the audited host-ABI TCB extension; its guarantees are `tested` /
   `validated` against `rustix` + the target, **never `proved`**; `ken-kernel`
   is unaffected and stays `forbid(unsafe_code)`.

## Acceptance criteria (testable)

- **AC1 — boundary consolidated.** `crates/ken-host` exists; all 6
  formerly-inline host-ABI `unsafe extern "C"` boundaries + their call
  sites now go through `ken-host`'s semantic API; `git grep 'unsafe extern'
  crates/ken-interp` returns **empty**. `rustix` appears in `ken-host`'s
  manifest only, private.
- **AC1b — the 13 handwritten facts are GONE.** No production path retains or
  consults the 11 `*_KEN` constants or the local `SIGPIPE`/`SIG_IGN` facts —
  `git grep` for them across `crates/**` (production) returns **empty**. (They
  are deleted in PX1, not deferred to PX2 — Architect `evt_1t429wz5ehf42`.)
- **AC2 — `ken-interp` forbids unsafe.** `ken-interp/src/lib.rs` carries
  `#![forbid(unsafe_code)]` and the crate builds clean under it
  (`scripts/ken-cargo build -p ken-interp`). The JIT `mem::transmute` in
  `ken-runtime/src/cranelift_backend.rs` is **untouched** (it is not in
  `ken-interp` and is not moved into `ken-host`).
- **AC3 — behavior-preserving.** The FS/console/signal host operations
  behave identically: existing host-op tests green (`scripts/ken-cargo
  test -p ken-interp` targeted suites); `openat`-relative resolve/operate
  split intact (ADR-0017); `O_NOFOLLOW`/symlink-policy path exercised and
  green, now sourced from `rustix`'s typed flag, not a magic number.
- **AC4 — no raw/rustix leakage.** No `RawFd`, raw pointer, bare integer flag,
  `rustix` type (`OFlags`/`AtFlags`/`Errno`/backend handle), or unrooted path
  crosses `ken-host`'s public API (types + a boundary test).
- **AC5 — dependency-closure deliverable present** in
  `docs/program/dependency-deltas.md`: full transitive closure, licenses,
  minimal feature set, upstream `unsafe` surface, counted and enumerated; the
  "larger-than-what-it-replaces?" question answered explicitly.
- **AC6 — no-regression in CI** (workspace-green **in CI**, never a local
  `--workspace` run) — full locked build + suite green on the publisher's
  CI gate.
- **AC7 — Linux-direct preserved.** `target_os = "linux"` gating intact;
  non-Linux targets still hit the named unavailable lane before any host
  call (PX0's property, not regressed). No POSIX-portability abstraction
  introduced.

## Do-not guards

- **Do NOT** move the `ken-runtime/src/cranelift_backend.rs` JIT
  `mem::transmute` (or any native-code-execution `unsafe`) into `ken-host` to
  satisfy AC2 — that merges two unrelated trust boundaries (fixed input 4; the
  forbidden move the Architect named).
- **Do NOT** add any new host operation, syscall, or capability — PX1 is
  extraction of the existing 6, nothing more. New ops are later PX.
- **Do NOT** add a `HostBackend` trait or any multi-backend abstraction — one
  Linux implementation today; the seam is `ken-host`'s semantic API (Architect
  `evt_1t429wz5ehf42`). A trait comes only when a 2nd real backend needs one.
- **Do NOT** expose `rustix` types or mirror the 6 C signatures as the public
  API — the facade is semantic (fixed input / seam above).
- **Do NOT** reintroduce string-path prechecking or any byte-path bypass
  that re-resolves after authorization (ADR-0017).
- **Do NOT** retain the 11 `*_KEN` constants / 2 `SIG*` facts as dead
  declarations — they are **deleted in PX1** (AC1b). The ABI **manifest** is
  PX2, not PX1 — don't add it here.
- **Do NOT** enable `rustix`'s libc backend or non-minimal features —
  `linux_raw` only.
- **Do NOT** run `cargo build/test --workspace` locally (COORDINATION §12);
  targeted `scripts/ken-cargo -p ken-host` / `-p ken-interp` only.
  Workspace-green is a CI property.

## Sequencing (Steward-owned)

- **Released now** (operator pulled PX1/PX2 ahead of CC9, 2026-07-15).
  Runtime `crates` lane, parallel to the language lane — no dependency on
  CC9/LET work.
- **PX2 follows PX1** (needs `ken-host` to exist; carries the clean-room
  gate for its system-header probe — that frame routes through the Spec
  enclave leakage recheck, not this one). PX1 has **no** clean-room gate
  (it consumes `rustix`'s public API, copies no source).
- **Architect terminal gate is required** (§14 — host trust boundary): the
  backend seam is already confirmed (`evt_1t429wz5ehf42`); the terminal gate
  verifies the FORK-2-aligned extraction has **zero host-ABI `unsafe` leak, the
  13 facts gone, no `rustix`/raw type leakage**, before merge.
