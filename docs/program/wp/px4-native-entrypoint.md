# PX4 — Native entrypoint ABI beyond `ClosedNullary`

**Owner:** Team Runtime · **Size:** **M** · **Branch:** `wp/px4-native-entrypoint`
(cut fresh: `git branch wp/px4-native-entrypoint origin/main` at build time) ·
**Gate:** Runtime QA + **Architect §14** (soundness: the native process-boundary
is `tested`/`validated`, never promoted to proof; fail-closed exit/trap mapping;
the JIT transmute boundary stays confined and named). **No CV** (no
`spec/`+`conformance/` path). **FULL CI** (touches `crates/`).

> Measured against the operator's intent (`09-posix-linux-abi-campaign.md §3`):
> *"a real practical tool… in the safest way possible."* **NATIVE EARLY is the
> operator's ruling (FORK 3):** if the interpreter is the only implementation it
> teaches us the wrong lessons and we discover them after the design is
> load-bearing. PX4 is the first step of PX-B — it builds the native process
> shell so PX5/PX6 have something real to lower effects into and to differentially
> guard. *"Safest possible"* here = a wrong/failed entry **fails closed to a
> nonzero status with a trap report**, never a silent success.

---

## 0. Objective (one line)

Lift the native executable entrypoint from **`ClosedNullary`** (a nullary
`extern "C" fn() -> i64` with all effects rejected) to a **process-shaped
entrypoint ABI**: raw **argv** + **environment** in, **process exit status** out,
with **runtime init/teardown** and **runtime-level stdout/stderr/trap
reporting** — reaching interpreter parity **for the process boundary**, and
**stopping exactly there** (per-effect syscall lowering is PX5, not PX4).

## 1. The boundary PX4 owns — and the two it must NOT cross

**PX4 = the process shell.** It gets a native binary to (a) receive argv+env, (b)
run its (currently pure-bodied) entrypoint, (c) init/teardown the runtime, (d)
report traps and flush its own diagnostic streams, and (e) exit with the correct
status. **PX4 does NOT lower Ken effects** — that is **PX5**
(`RuntimeExpr::Effect`). Two hard "do not cross" lines:

1. **Do NOT lower `RuntimeExpr::Effect`.** It stays the explicit rejection at
   `crates/ken-runtime/src/cranelift_backend.rs:1438-1440` and
   `HostEffectExecution` stays the explicit **unavailable lane** at
   `platform_runtime_support.rs:327-331`. A native program body that calls a
   Console/FS/Clock effect must still hit a **stable unavailable lane — never a
   no-op, never a generic scalar call** (campaign §PX5 note). PX4 delivers the
   shell around the body, not the body's IO.
2. **Do NOT launder the JIT trampoline into `ken-host`** (Architect, Phase-A
   exit, campaign `:359-378`). The native code-execution `unsafe` at
   `cranelift_backend.rs:1059` (`mem::transmute::<_, extern "C" fn() -> i64>`) is
   its **own** named boundary — a native-code-execution boundary, *not* a host-ABI
   boundary. PX4 **changes its signature** (§3.B) but keeps it confined and named;
   it does not merge it with the `ken-host` syscall seam to make a checkbox green.

## 2. Fixed inputs — SETTLED, do not reopen

- **Native early (operator, FORK 3).** PX-B (`PX4→PX5→PX6`) lands before PX-C.
  PX4 is P0; it is the enabler, not a nice-to-have.
- **No OS/host-ABI `unsafe` outside `ken-host`; `ken-interp` forbids `unsafe`
  (Architect, corrected Phase-A exit).** The *only* native `unsafe` PX4 touches is
  the JIT transmute boundary above — an execution boundary, kept confined. PX4
  adds **no** host-ABI `unsafe`.
- **Successful OS/native execution is NEVER promoted to kernel proof.** The
  entrypoint's guarantees are `tested`/`validated`/`delegated`, and the
  disclosure lives **in the source** (not only this frame). Never an unearned
  `proved`.
- **Parity target is the interpreter's existing process ABI**, which is the
  authority PX4 mirrors natively (do not invent a second contract):
  - argv/env are staged by `crates/ken-cli/src/lib.rs`: `arguments: &[Vec<u8>]`,
    `environment: &[(Vec<u8>, Vec<u8>)]` (`:40-41`) → `process_input_value`
    (`:304-341`, argv list `:315-318`, env list `:320-334`).
  - exit status is the entrypoint's returned **`ExitCode` inductive → `i32`**,
    mapped **fail-closed on a malformed payload** (`exit_status()` `:346-364`,
    fail-closed at `:357/:362`; `ExitCode` id `:65`).
  The native path must reach the **same** argv/env value shape and the **same**
  `ExitCode→i32` fail-closed mapping — parity, not a parallel invention.
- **Unsupported/未reached lanes stay STABLE and EXPLICIT.** Every not-yet-native
  capability returns its named unavailable lane before doing anything, exactly as
  `ExecutableArgumentShape::Unavailable` / `platform_runtime_support` already do.

## 3. Mandated deliverable — each part ends in a concrete choice

### A. Widen the entrypoint argument shape (`executable_entrypoint_packaging.rs`)

Today `ExecutableArgumentShape` (`:87-102`) is `ClosedNullary |
UnsupportedRuntimeArguments{..} | Unavailable{..}`. Add a **process-shaped**
variant that carries raw **argv** and **environment**, marshalled to the same
value shape the interpreter builds (`process_input_value`). **Concrete choice:**
the native entry receives argv as a list of byte-strings and env as a list of
(byte-string, byte-string) pairs — **byte-accurate, not `String`** (argv/env are
OS bytes, not UTF-8) — and constructs the identical process-input Ken value the
interpreter uses, so the entrypoint contract (`ken-cli` `:435-505`) is unchanged.
`ClosedNullary` remains valid for a genuinely nullary entry; the new variant is
the argv/env case.

### B. Native entry/exit signature (`cranelift_backend.rs`)

Change the JIT trampoline (`:1059`) from `extern "C" fn() -> i64` to a signature
that **(i)** makes staged argv/env reachable by the entry (via a runtime-init
staging area — §3.C — or explicit params, whichever the backend supports
cleanly) and **(ii)** propagates the entrypoint's **`ExitCode`** result out.
**Concrete choice:** keep the transmuted function pointer as the **single named
native-execution boundary**; the entry returns a value the trampoline decodes to
an `ExitCode`, which is then mapped to the process `i32` status by the **same
fail-closed `exit_status` logic** as `ken-cli` (reuse it; do not fork the
mapping). A malformed/absent `ExitCode` payload → **nonzero fail-closed status +
a trap report**, never `0`.

### C. Runtime init/teardown + trap/diagnostic reporting

Add a runtime **init** (stage argv/env; set up the diagnostic streams) and
**teardown** (flush stdout/stderr; run any required cleanup) around the entry, and
a **trap/panic reporter**: an entrypoint trap (or a Ken `panic`/unrecoverable
runtime error) writes a **trap report to the runtime's stderr** and exits
**nonzero**. **Concrete choice:** "stdout/stderr" here is the **runtime's own
diagnostic plumbing** (final flush + trap report) — it is **NOT** the Ken
`Console` effect (that is a `RuntimeExpr::Effect`, deferred to PX5). Wire
`ExecutableTrapShape` (`:118`) / `ExecutableRuntimeSupport::TrapReporting`
(`:78`) to a real native reporter; the "trap reporting" support fact must become
**available and exercised**, not a placeholder.

### D. Keep the effect lane explicitly unavailable (PX5 boundary)

Confirm — with a test — that a native program body invoking a Console/FS/Clock
effect still reaches the **stable unavailable lane** (`HostEffectExecution` /
`RuntimeExpr::Effect` rejection), with a clear diagnostic, **not** a crash or
no-op. PX4's win is the shell; the body's IO is PX5.

## 4. Acceptance criteria (testable)

1. **AC1 — argv/env reach the native entry.** A native binary whose entrypoint
   consumes argv (e.g. exits with `min(argc, 255)` as its status, or echoes an
   argv element into its exit code) observes the **actual** argv/env passed by
   the OS, byte-accurate. Demonstrate with ≥2 distinct argv vectors (a
   non-degenerate pair, not one case — COORDINATION §7).
2. **AC2 — exit status parity + fail-closed.** The native `ExitCode→i32` mapping
   is the **same** mapping as `ken-cli::exit_status`; a well-formed `ExitCode`
   maps correctly, and a **malformed payload fails closed to nonzero + trap
   report**, never `0`. Show both arms.
3. **AC3 — trap reporting is real.** An entrypoint trap/panic writes a trap
   report to the runtime stderr and exits **nonzero**; the `TrapReporting`
   support fact is **available and exercised**, not a placeholder. Show a trap
   run vs. a clean run (distinguishing pair).
4. **AC4 — effect lane still explicitly unavailable.** A native body invoking a
   Ken effect hits the **stable named unavailable lane** (not a no-op, not a
   crash); `RuntimeExpr::Effect` remains rejected and `HostEffectExecution`
   remains the explicit unavailable fact. (Guards the PX4/PX5 boundary.)
5. **AC5 — the transmute boundary stays confined + named.** No OS/host-ABI
   `unsafe` is added outside `ken-host`; the native-execution `unsafe` remains the
   single named JIT boundary (`ken-interp` keeps `forbid(unsafe_code)`; the JIT
   trampoline is **not** folded into `ken-host`). §14 confirms structurally.
6. **AC6 — honest disclosure in source.** The native entrypoint's guarantees are
   labelled `tested`/`validated` **in the source**, never `proved`; a wrong-target
   manifest still fails closed before any syscall (unchanged from PX2). No
   unearned proof language.
7. **AC7 — no regression.** Workspace-green **in CI** (never a local
   `--workspace` run — COORDINATION §12); existing native/interp suites pass.

> **Full interpreter/native observational parity is PX6's gate, not PX4's.** PX4
> is verified by **direct native tests** of the process ABI (argv/exit/trap);
> the differential harness that proves external-delta identity comes in PX6
> (Verify). Do not block PX4 on a harness that does not exist yet — and do not
> build a bespoke differential harness here (that is PX6's lane).

## 5. Guardrails (do-not-reopen)

- ⛔ **Do not lower `RuntimeExpr::Effect`** — PX5's job. Effects stay an explicit
  unavailable lane.
- ⛔ **Do not launder the JIT trampoline into `ken-host`** (Architect Phase-A
  exit). It is its own named native-execution boundary.
- ⛔ **No host-ABI `unsafe` outside `ken-host`; `ken-interp` keeps
  `forbid(unsafe_code)`.**
- ⛔ **Never promote native execution to kernel proof.** `tested`/`validated`,
  disclosed in source.
- ⛔ **No silent-success failure mode.** A wrong/failed entry fails closed to
  nonzero + trap report.
- ⛔ **`crates/` ⇒ FULL CI**, never `--doc-only`. Build TARGETED only
  (`ken-cargo -p <crate>`), never `--workspace` (operator hard rule).
- ⛔ **A genuine need for a new kernel/trusted rule is a scope fork → Architect**,
  not an implementer judgment call (campaign §7).

## 6. Dependencies & context

- **Depends on:** PX1 (`609dd600`) + PX2 (`626b38dd`) — merged (the `ken-host`
  boundary + manifest the native artifact binds and the target-identity assert).
- **Feeds:** **PX5** (native effect lowering — fills the unavailable lane PX4
  keeps explicit) → **PX6** (Verify: interp/native differential harness — proves
  the parity PX4 begins). DAG: `PX2 → PX4 → PX5 → PX6` (campaign §6), sibling of
  PX3.
- **Phase-B exit (PX4+PX5+PX6 together, NOT PX4 alone):** the Milestone-C CLI
  tool runs as a native executable, observationally identical to its interpreter
  run. PX4 delivers the process shell that makes that reachable.
- **Perishable anchors:** grounded against `origin/main` 2026-07-15; the
  `ken-runtime`/cranelift anchors were stable, the `eval.rs` host-op anchors had
  drifted — **re-verify every `file:line` at build time** (campaign §0a).
