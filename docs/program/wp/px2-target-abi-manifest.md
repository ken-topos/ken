# WP — PX2: the Target-ABI manifest + system-header cross-check

**Owner:** Runtime ring (leader → implementer → QA).
**Reviewer/Gate:** Runtime QA **+ Architect terminal soundness/design gate
(§14 — this hardens the host trust boundary with a fail-closed identity
check).** **No CV Spec-vote at merge** (no `spec/`+`conformance/` delta), **but
a distinct pre-release clean-room clearance is required — see the RELEASE
box.**
**Size:** M · **Risk:** medium (build-time probe + fail-closed mechanism +
artifact hash-binding + clean-room gate). **Branch:** `wp/px2-target-abi-manifest`,
cut fresh from `origin/main` at kickoff. **CI:** ⛔ **FULL CI** — touches
`crates/` (never `--doc-only`).
**Source:** charter `docs/program/09-posix-linux-abi-campaign.md` §4 PX2 + FORK-2
required shape (`evt_7qqf827rr1jxk`) + Phase-A-exit correction; PX1 landed
(`ken-host` on `origin/main @ 609dd600`).

> ### ✅ RELEASE STATUS — clean-room CLEARED; PX2 releasable
> The operator authorized PX2 (pulled PX1/PX2 ahead of CC9, 2026-07-15). The
> **system-header probe is CLEARED by the Spec enclave's leakage recheck**
> (`evt_5fx7gmprrk07b`, spec-leader front-desking the enclave's ruling): a
> Linux-target probe may compile Ken-authored C that `#include`s the platform
> headers and emits **numeric ABI values only** — that is fact-observation, not
> vendoring header source; `CLEAN-ROOM.md` bars copying/close-paraphrase of
> protected *expression*, not independently observing a build-time *fact*. The
> clearance is **subject to 5 binding constraints** — see **fixed input 9** and
> **AC8**; they are settled and must be honored verbatim. With those, the probe
> half is releasable alongside the manifest + hash-binding halves.

## Objective

PX1 consolidated the host boundary into `ken-host` over exact-pinned `rustix`
(`linux_raw`), deleting all 13 hand-authored ABI facts. The ABI numbers Ken now
depends on live in `linux-raw-sys` (rustix's machine-generated backend) — a
single, machine-generated source of truth, but still **one** source, invisible
in the artifact and unverified against the actual target.

PX2 makes that trust **legible, cross-checked, and artifact-bound.** It:

1. **Generates a `TargetAbi` manifest** recording the exact ABI identity the
   host boundary was built against — target, selected backend, pinned
   dependency set (version + checksum + features), the **complete ABI-fact
   inventory** the boundary actually depends on, a schema version, and an
   **output hash** over the whole manifest.
2. **Cross-checks every ABI fact against an INDEPENDENT second source** — a
   build-time system-header probe. `linux-raw-sys`'s value and the probe's value
   for each fact must agree; **any disagreement fails the build CLOSED.** Two
   independent derivations of each number, not one trusted set.
3. **Binds the manifest hash into both the interpreter and the native
   artifacts**, so each artifact declares the exact ABI identity it was built
   against and a **wrong-target manifest fails closed before any host syscall
   runs** (the Phase-A-exit acceptance criterion).

This is **pure hardening of the PX1 boundary** — no new host operation, no new
syscall, no kernel change. It is the enabler that makes the guardrail *"no ABI
fact without a probe"* (charter §7) true, and PX-B…PX-E build on it.

## Fixed inputs (SETTLED — do not reopen)

1. **`ken-host` is the boundary and owns the manifest.** Subsume-don't-
   proliferate: the `TargetAbi` machinery lives in / adjacent to `ken-host`
   (its `build.rs` + a generated module), not a new standalone crate. `rustix`
   stays a private dependency; no `rustix`/raw type crosses the public API
   (PX1 property, not regressed).
2. **The manifest is GENERATED, never hand-maintained.** A hand-written ABI
   constant in the boundary is a **defect** after PX2 (charter guardrail §7 —
   *"no ABI fact without a probe"*). The facts are read from `linux-raw-sys` +
   cross-checked; none is typed by hand. Do **not** reintroduce the deleted
   `*_KEN` constants in any form.
3. **The probe is an INDEPENDENT CROSS-CHECK, not a replacement constant set**
   (FORK-2 required shape). It emits the same facts from the real system headers
   as a second source of truth; the build **compares** the two and **fails
   closed on any disagreement.** It never becomes the source Ken reads at
   runtime (that stays `linux-raw-sys`, typed, via `rustix`).
4. **Grounded inventory — enumerate from the LANDED `ken-host`, NOT the
   charter's stale "13 facts."** PX1 removed the signal op entirely, so the **2
   former `SIG*` facts are ABSENT** and must **not** be resurrected (SIGPIPE is
   Rust-std's, per the PX1 dependency record). The inventory is exactly the
   **filesystem** ABI facts the boundary depends on — re-derive them from
   `crates/ken-host/src/lib.rs` at build time (they drift as `ken-host` evolves).
   As of `609dd600` that is: the `OFlags` set (`RDONLY`, `WRONLY`, `RDWR`,
   `APPEND`, `CREATE`, `EXCL`, `TRUNC`, `DIRECTORY`, `NOFOLLOW`, `CLOEXEC`),
   `AtFlags::REMOVEDIR`, the `Mode` bit space, the `*at` / directory syscall set
   (`openat`, `mkdirat`, `unlinkat`, `renameat`, `readlinkat`, plus the
   `std`-backed `read_dir`/`create_dir`/`remove_dir`/`remove_dir_all`/`symlink`
   facilities), and the errno space. **Count them; state "there are N, here are
   all N"; do not inherit a number** (the PX1 dependency-precondition discipline,
   applied to facts).
5. **Zero kernel / `trusted_base()` delta.** PX2 lives entirely in the
   **runtime** trust boundary (like PX1). `ken-kernel` is untouched and stays
   `#![forbid(unsafe_code)]`; `ken-interp` and `ken-host` stay
   `#![forbid(unsafe_code)]`. If any change appears to touch the kernel, a
   trusted typing rule, or `trusted_base()`, **STOP — it is a scope fork to the
   Architect** (charter guardrail §7), not an implementer call.
6. **Host guarantees are `tested` / `validated`, never `proved`** — the
   manifest and the module doc disclose this plainly (PRINCIPLES #14).
7. **Linux-direct preserved.** `target_os = "linux"` gating intact; a non-Linux
   target records the **unavailable backend** in the manifest, runs **no probe**,
   and still hits the named unavailable lane **before any host call** (PX0's
   property — do not regress it).
8. **The JIT `mem::transmute` in `ken-runtime/src/cranelift_backend.rs` is
   untouched.** Binding the manifest hash into the native artifact does **not**
   move or launder that native-execution boundary into `ken-host` (Phase-A-exit
   correction — the explicitly forbidden move).
9. **The 5 clean-room constraints (SETTLED — enclave clearance
   `evt_5fx7gmprrk07b`; honor verbatim).** The probe is cleared *only* under
   these; a violation voids AC8:
   - **(a) No header expression enters the tree.** Do not copy or paraphrase
     macro bodies, declarations, comments, layout/order, generated header
     fragments, or preprocessor output. The probe emits only a fixed, reviewed
     `FACT=INTEGER` protocol; generated Rust/manifest code carries **numeric
     values only**.
   - **(b) Interface names are permitted only as the identifiers needed to query
     the fact.** `#include <…>` paths and macro/function/fact names (e.g.
     `O_NOFOLLOW`) may appear in Ken-authored probe code and manifest labels. Do
     **not** reproduce macro definitions or stringify/dump header text.
   - **(c) The probe is an observer, never a source Ken consumes at runtime.**
     `linux-raw-sys` remains the runtime/binding source; the header result is
     compared fact-by-fact and any mismatch fails closed. No copied C
     declaration, signature, or host-header-derived calling convention may enter
     the public boundary.
   - **(d) Target identity must be honest.** Run the probe only through the
     selected Linux target toolchain/headers **for the target being
     manifested**. If that target-qualified probe cannot run (including an
     unsupported cross-build), **fail closed or record the explicitly unavailable
     backend** — never treat build-host headers as evidence for a different
     target.
   - **(e) Keep the probe inventory closed to PX2's enumerated ABI facts** (fixed
     input 4). No general header dump, discovery scan, or unrelated API
     extraction; the probe's own C is Ken-authored and minimal.

## Mandated deliverable outline (each ends in a concrete choice)

1. **The `TargetAbi` manifest + its generator (`ken-host/build.rs` + a generated
   module).** At build time, emit a manifest value recording:
   - **target identity** — the target triple / `cfg(target_os)` the boundary was
     built for, and the selected backend (`linux_raw`);
   - **pinned dependency set** — `rustix`, `bitflags`, `linux-raw-sys` with exact
     version + checksum + the enabled feature set (`["std","fs"]`), i.e. the
     governing target-selected N=3 closure the PX1 dependency record already
     established (`docs/program/dependency-deltas.md` §PX1);
   - **the complete ABI-fact inventory** (fixed input 4), each fact carrying its
     **numeric value as resolved from `linux-raw-sys`**;
   - a **schema version** (integer) for the manifest format; and
   - an **output hash** (e.g. SHA-256) over the canonical serialization of all of
     the above.
   Concrete choice: generate a `const TARGET_ABI: TargetAbi` (+ `const
   TARGET_ABI_MANIFEST_HASH: [u8; 32]`) into `OUT_DIR` and `include!` it — no
   checked-in manifest file to drift.
2. **The system-header probe (INDEPENDENT cross-check) — clean-room-gated.**
   A `build.rs` probe that `#include`s the Linux system/UAPI headers
   (`<fcntl.h>`, `<sys/stat.h>`, `<unistd.h>`, …) and **emits the numeric value**
   of every fact in the inventory. The build then compares, fact by fact, the
   probe value against the `linux-raw-sys` value; **any mismatch is a hard build
   error (`cargo:` build failure), never a warning.** Concrete choice: a tiny C
   translation unit compiled by the probe (via the `cc` crate or a direct
   `cc` invocation — record the tool in the dependency delta if a crate is
   added) that prints `FACT=VALUE` lines; `build.rs` parses and diffs them.
   **✅ CLEARED (`evt_5fx7gmprrk07b`) — build to the 5 constraints in fixed input
   9.** The `FACT=INTEGER` protocol is fixed and reviewed; the probe copies **no
   header source** (values only); it runs only through the
   target-being-manifested's toolchain/headers and **fails closed** (or records
   the unavailable backend) if it cannot; and its C is Ken-authored, minimal, and
   closed to the enumerated inventory.
3. **Fail-closed manifest-hash binding into interpreter AND native artifacts.**
   Compile `TARGET_ABI_MANIFEST_HASH` into the interpreter (`ken-interp`/`ken-cli`)
   and the native-execution artifact (`ken-runtime`), and add a **fail-closed
   identity assertion**: before any host syscall, the artifact confirms the
   compiled-in hash matches `ken-host`'s `TARGET_ABI_MANIFEST_HASH`; a mismatch
   **aborts before the syscall** (charter Phase-A-exit: *"a wrong-target manifest
   fails closed before any syscall runs"*). Concrete choice: the interpreter
   checks at host-boundary entry (it has live FS ops today); the **native
   artifact does no host syscalls yet** (PX-B adds them), so its PX2 binding is
   the **compiled-in hash + a native-entry identity assertion** — the per-syscall
   check goes live when PX-B wires native host ops. State this scope in the
   source so PX-B inherits a manifest-bound native path, and do **not** over-build
   a native syscall gate that has nothing to gate yet.
4. **Trust disclosure in the source.** `ken-host`'s module doc + the manifest
   state: this is the audited host-ABI TCB extension; the ABI facts are
   dual-sourced (`linux-raw-sys` + the system-header probe) and **cross-checked at
   build time, fail-closed**; guarantees are `tested` / `validated`, never
   `proved`; `ken-kernel` is unaffected.

## Acceptance criteria (testable)

- **AC1 — manifest generated & complete.** A `ken-host` build emits a
  `TargetAbi` manifest recording target identity, selected backend (`linux_raw`),
  the exact `rustix`/`linux-raw-sys`/`bitflags` version+checksum+features (the
  N=3 governing closure), the complete FS ABI-fact inventory **enumerated from
  the landed boundary** (fixed input 4 — "there are N, here are all N"), a schema
  version, and an output hash. The 2 former `SIG*` facts are **absent**.
- **AC2 — probe cross-check present & FAIL-CLOSED (discriminator pair).** The
  build emits each ABI fact from the real system headers and compares it against
  the `linux-raw-sys` value. A **non-degenerate pair**: with the true values the
  build **succeeds**; with a deliberately tampered expected value the build
  **fails closed** (hard error, not a warning). A single green build does not
  prove the net — exercise the mismatch arm.
- **AC3 — hash binding & fail-closed identity (discriminator pair).**
  `TARGET_ABI_MANIFEST_HASH` is compiled into the interpreter and native
  artifacts; a **matching** hash lets a host op proceed, a **mismatched** hash
  **aborts before any host syscall**. Both arms tested.
- **AC4 — no hand-written ABI fact remains or is reintroduced.** `git grep`
  across `crates/**` shows zero hand-authored ABI numeric constants in the
  boundary; the deleted `*_KEN` / `SIG*` facts are not resurrected (charter
  guardrail §7).
- **AC5 — Linux-direct preserved.** A non-Linux target records the unavailable
  backend in the manifest, runs no probe, and hits the named unavailable lane
  before any host call (PX0 property, not regressed). No POSIX-portability
  abstraction introduced.
- **AC6 — no trust growth.** Zero `trusted_base()` delta; `ken-kernel`,
  `ken-interp`, and `ken-host` all remain `#![forbid(unsafe_code)]`; the JIT
  `mem::transmute` in `ken-runtime` is byte-untouched; guarantees disclosed
  `tested`/`validated`, never `proved`.
- **AC7 — no-regression in CI** (workspace-green **in CI**, never a local
  `--workspace` run — §12): full locked build + suite green on the publisher's CI
  gate. Any new build dependency (`cc`) is recorded in
  `docs/program/dependency-deltas.md`.
- **AC8 — clean-room cleared, constraints honored.** The probe carries the Spec
  enclave's leakage-recheck clearance `evt_5fx7gmprrk07b`, and **all 5 binding
  constraints (fixed input 9) are demonstrably met**: no header expression in the
  tree (only a fixed `FACT=INTEGER` protocol + numeric manifest values); interface
  names appear only as query identifiers/labels; the probe is a build-time
  observer, never a runtime source; the probe is target-honest (fails closed / an
  unavailable backend if it cannot run for the manifested target); and the probe
  inventory is closed to the enumerated ABI facts.

## Do-not guards

- **Do NOT** reintroduce hand-written ABI constants in any form — the manifest is
  generated + probe-checked (fixed input 2; charter guardrail §7).
- **Do NOT** resurrect the `SIG*` facts or any signal operation — PX1 removed
  them; SIGPIPE is the Rust-std runtime's guarantee (PX1 dependency record).
- **Do NOT** add a new host operation, syscall, or capability — PX2 is
  identity/manifest/probe only; new ops are PX-B+.
- **Do NOT** copy system-header **source** into the tree, and do NOT write probe
  code before the Spec-enclave leakage recheck clears it (RELEASE box / AC8).
- **Do NOT** make a fact disagreement a warning — it **fails the build CLOSED**
  (AC2).
- **Do NOT** grow the kernel or `trusted_base()`, or add a trusted typing rule —
  that is a scope fork to the Architect (fixed input 5).
- **Do NOT** move or launder the `ken-runtime` JIT `mem::transmute` into
  `ken-host` (fixed input 8 — the Phase-A-exit forbidden move).
- **Do NOT** enable `rustix`'s libc backend or any non-minimal feature —
  `linux_raw` only (PX1 pin).
- **Do NOT** run `cargo build/test --workspace` locally (§12); targeted
  `scripts/ken-cargo -p ken-host` / `-p ken-interp` / `-p ken-runtime` only.
  Workspace-green is a CI property.

## Sequencing (Steward-owned)

- **Released after PX1** (needs `ken-host` to exist — it does, `origin/main @
  609dd600`). Runtime `crates` lane, parallel to the Language lane.
- **The probe half is HELD** behind the Spec-enclave leakage recheck (RELEASE
  box). The manifest + hash-binding halves (deliverables 1, 3, 4) are unblocked;
  the frame is shovel-ready except that the implementer must not begin the probe
  (deliverable 2) until the clearance lands. The Steward routes the clean-room
  query to the enclave and holds the Handoff-Gate/kickoff until it clears.
- **Architect terminal §14 gate is required** (host trust boundary + fail-closed
  soundness): verifies the manifest is generated (no hand-facts), the cross-check
  fails closed, the hash-binding fails closed before any syscall, and the
  Linux-direct + `forbid(unsafe)` + JIT-untouched invariants hold, before merge.
- **PX3 (Language: `USize`/`ISize`/`CInt` bound to the manifest) depends on
  PX2** — the manifest is the binding target.
