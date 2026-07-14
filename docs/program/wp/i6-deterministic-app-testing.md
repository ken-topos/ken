# I-6 — Deterministic application testing

**Owner:** Team Runtime · **Size:** S/M · **Base:** `origin/main @ c5f73b9c`
**Branch:** `wp/i6-deterministic-app-testing`

## 1. Objective

Make it possible for **an author of a Ken application** to write a
**deterministic test of their own program** — inject `argv`, environment, cwd,
stdin, and a filesystem; run the program; assert on what it printed, what it
touched, and how it exited.

**This WP builds no new runtime capability.** Every mechanism it needs is
already landed. Its entire job is to make the landed mechanism **reachable from
outside the `ken` binary**.

## 2. Fixed inputs — grounded against the tree, not against a report

I re-verified each of these against `origin/main @ c5f73b9c` myself. **Treat
them as perishable anyway** (§6).

### 2.0 ★ ERRATUM — §2.1's "complete" was FALSE. Read this first.

**The implementer hard-stopped this frame pre-edit and was right.** The original
§2.1 claimed the injectable seam was "public and complete." **It is not, for the
generic runner this WP mandates:**

- **`HostHandler` (`eval.rs:2189+`) has NO capability-mint operation.** It
  declares console ops, `fs_resolve`, and the handle-only `fs_*_at` family —
  and nothing else.
- **The runner mints through the CONCRETE type**: `run_file:297` calls
  `host.mint_fs_cap(declared_fs.authority)`, an **inherent** method on
  `PosixHost` (`eval.rs:2416`). `CaptureHost` carries its **own separate
  inherent** `mint_fs_cap` (`eval.rs:3058`).
- **⇒ A generic `run_program<H: HostHandler>` cannot mint the `ProgramCaps` at
  all.** The trait cannot express the one step the concrete path performs.

**RULING (Steward, on the Architect's soundness ruling `evt_7v76zwacxxn2q`):
`mint_fs_cap` MOVES ONTO THE `HostHandler` TRAIT. That is I-6's STEP 0.**

```rust
fn mint_fs_cap(&self, authority: capabilities::Authority) -> capabilities::Cap;
```

- **Required method, no default** — every host must consciously bind its own
  identity into the cap; a default risks an identity-less/unbounded cap. Both
  impls already provide it, so `required` is clean.
- **`&self`, not `&mut self`** — both mint read-only; the tighter borrow is
  correct.
- **`CaptureHost::mint_scoped_fs_cap` stays INHERENT (test-only).** The trait
  needs only the declaration-derived `mint_fs_cap`.
- **Write this invariant at the method**, so a future third impl inherits it:
  *"`mint_fs_cap` mints a cap rooted at the host's own resolved identity,
  bounded by the declared authority; it is runner-only and never reachable from
  Ken."*

**Why this grants no new authority:** a `HostHandler` impl already implements
`fs_resolve` (the `openat` walk) and the whole `fs_*_at` syscall family — **it
can already perform any FS operation directly.** A `Cap` is a strictly *weaker,
gating* token: it constrains what Ken's **program** may request and grants the
**host** nothing, because the host is the code doing the syscalls. **Authority
to mint a gating token is subsumed by authority to perform the ops it gates.**
Both mints are `&self` and root the cap in the host's own identity (`PosixHost`
→ `FsIdentity::Posix{device, inode}`; `CaptureHost` → `FsIdentity::Virtual`
node-0), and `fs_resolve` later resolves relative to that host-owned root —
**which is exactly why a caller-supplied cap, or a free-function `Cap::mint`
with no `&self`, would be a real bypass of the I-5 scoped ABI.**

**ADR-0017's unforgeability invariant is untouched:** Ken's surface cannot call
a Rust trait method; the cap enters Ken only as the `EvalVal::Cap` the runner
constructs. **Inherent-vs-trait is invisible to Ken.** Zero kernel-TCB.

**This is the I-5 precedent completed** — ADR-0017 §4a step 0 already moved
`fs_resolve`/`fs_*_at` onto the trait to parameterize the driver; `mint_fs_cap`
is the **one** host op it left concrete, because the mint call was not
parameterized yet. §5's "no semantics change" means *do not change what the
interpreter does* — and this does not. It changes only **where the operation is
declared.**

### 2.1 Already landed — DO NOT BUILD THESE

*(Everything below is true — but read §2.0: the seam is complete for **effects**,
and was **not** complete for the **mint**.)*

- **The injectable seam is public for effects.** `crates/ken-interp/src/lib.rs`
  (lines 9–15) re-exports `HostHandler`, `run_io`, `apply`, `CaptureHost`,
  `PosixHost`, `ConsoleIds`, `ConsoleStream`, `ConsoleTrace`, `CoproductIds`,
  `FSIds`, `FsTrace`, `FsOpKind`, `HostCreatePolicy`, `HostFileKind`,
  `HostFileMetadata`, `HostRead`, `VirtualFsNode`, `VfsNodeId`, `Resolution`,
  `ResolveError`, `CapabilityDenied`, `check_fs_capability`, `RunIoError`.
  `run_io<H: HostHandler>` is the public injection point (`eval.rs:4239`).
- **The virtual filesystem** — landed by I-5 step 0 (inode-keyed, with a
  `Symlink` node).
- **Captured console streams** — landed by I-2.
- **Scripted stdin** — landed. `CaptureHost::new(stdin)` (`eval.rs:2940`); I-2
  drives `abcde → abc, de, Eof` through the real `run_io`.
- **A "fixed clock" is NOT part of this WP.** There is **no clock effect in
  Ken** — zero registered clock/time/now operations in the prelude, zero in the
  interpreter. You cannot inject determinism into an effect that does not
  exist. **Clocks belong to I-7.** If a checklist told you otherwise, the
  checklist was wrong.

### 2.2 The actual gap — `ken-cli` is a BINARY-ONLY crate

This is the finding the WP turns on, and it is **larger than "write some
docs"**:

- `crates/ken-cli/Cargo.toml` declares **`[[bin]] name = "ken"` and no
  `[lib]`**. `crates/ken-cli/src/` contains only `main.rs` and `repl.rs`.
- **⇒ Nothing can `use ken_cli::…`.** Not a downstream crate, not an
  integration test, not an application author. There is no library surface.
- The entire run pipeline is **private inside the binary**:
  `os_bytes:145/151` · `elaborate_cli_file:161` · `run_file:239` ·
  `resolve_main:382` · `constructor_value:436` · `list_value:494` ·
  `process_input_value:506` · `exit_status:552`.
- `run_file` **hard-wires the host**: `ken_interp::PosixHost::new()`
  (`main.rs:297`).
- Process input is read from **the real process**: `std::env::vars_os()`
  (`main.rs:522`) and `std::env::current_dir()` (`main.rs:538`).

*(All anchors above re-derived at `origin/main @ c5f73b9c` via
`git show origin/main:<file>` — not from a worktree. Verify them the same way.)*

So an application author who wants a deterministic test must hand-rebuild
elaboration, entrypoint resolution, Console/FS id harvesting, `ProgramCaps`
application, `ProcessInput` assembly, and the eval store — reimplementing, from
scratch, a pipeline that already exists and is already correct.

**That reconstruction is the gap. Closing it is the whole WP.**

### 2.3 `ProcessInput` can already carry injected values (constructibility)

`MkProcessInput : List Bytes → List (Prod Bytes Bytes) → Bytes` — argv,
environment pairs, and cwd. All three are plain `Bytes`, so all three are
**injectable as `Vec<u8>`**. Nothing new is needed to *carry* a fake
environment; the only reason the binary uses the real one is that it calls
`std::env` directly.

## 3. Mandated deliverable

### 3.0 STEP 0 — lift `mint_fs_cap` onto the `HostHandler` trait

Per §2.0. Both impls already have it inherently; this is a **move to the trait**,
carrying the stated invariant as a doc comment. Do this **first** — the generic
runner in §3.2 is not expressible until it lands.

### 3.1 Give `ken-cli` a library

Add `[lib]` to `crates/ken-cli/Cargo.toml` (`src/lib.rs`) and move the pipeline
into it. **Do NOT create a new crate.** `ken-cli` already depends on
`ken-kernel`, `ken-elaborator`, and `ken-interp`, so a `lib` target adds
**zero new crate edges** — the dependency-DAG audit is clean, and a new crate
would add an edge for nothing.

### 3.2 One parameterized entry point

Expose a single driver in which **every source of nondeterminism is a
parameter**:

```rust
pub fn run_program<H: HostHandler>(
    source: /* the Ken program */,
    argv:   &[Vec<u8>],
    env:    &[(Vec<u8>, Vec<u8>)],
    cwd:    &[u8],
    host:   &mut H,
) -> Result<ProgramOutcome, RunError>
```

The exact spelling — argument order, `ProgramOutcome`'s shape, whether `source`
is a path or bytes, how the console/FS traces come back — **is yours to choose**
and is not pinned. **The contract is pinned:** `argv`, `env`, `cwd`, and the
host are **all injected**, and the library **never reads the ambient process**.

### 3.3 `main.rs` becomes a thin wrapper

The binary supplies `PosixHost` plus the real `std::env` argv/env/cwd and calls
the same driver. **The extraction must be behavior-preserving** — the `ken`
binary is the reference, and its observable behavior may not drift.

### 3.4 One worked, copyable, application-level deterministic test

The artifact an application author reads and copies: a Ken program run under
`CaptureHost` with fixed argv/env/cwd/stdin, asserting on the console trace, the
FS trace, and the exit status. **One** is enough. It must use **only the public
surface** — no private hooks, no test-only escape hatch. If it needs something
that isn't public, that something is part of this WP.

## 4. Acceptance criteria

- **AC1 — the library exists and is reachable.** An integration test outside
  `src/` does `use ken_cli::…` and drives the program end to end.
- **AC2 — nondeterminism is fully parameterized (structural discriminator).**
  `rg 'std::env' crates/ken-cli/src/lib.rs` is **EMPTY**. Grep the emission, not
  the name: the ambient reads must be *gone from the library*, not merely
  wrapped.
- **AC3 — no behavior drift.** The `ken` binary behaves exactly as before; the
  existing CLI tests stay green.
- **AC4 — determinism is actually demonstrated.** The worked test runs the same
  program with the same injected inputs **twice** and gets a **byte-identical**
  console trace, FS trace, and exit status.
- **AC5 — behavioral reuse, not declared reuse** (this is the AC that matters —
  it is CC7's AC6 in a new suit). The test must **DRIVE the landed
  `CaptureHost` through the real `run_io`**. A bespoke harness that reproduces
  the pipeline privately — or a test that constructs a `CaptureHost` and then
  never routes the program's effects through it — **fails this AC even if it is
  green.** Assert that the observed traces come from the *injected* host.
- **AC6 — zero deltas.** No new crate. No new primitive. No new handler, virtual
  FS, console stream, stdin mechanism, or clock. `trusted_base()` before ==
  after.

## 5. Guardrails — do not reopen

- **Do not build a clock.** It is I-7's, and no clock effect exists to inject
  into. *(A mostly-done WP is good news. Do not manufacture scope.)*
- **Do not build a second handler, VFS, stream, or stdin path.** They are landed
  (§2.1). If one seems inadequate, say so with tree anchors — do not fork it.
- **Do not change interpreter or kernel semantics.** This is an extraction.
- **Corpus-oracle pin (audit (c)).** The worked example should be a **Rust
  integration test driving an inline Ken program**, adding **no new file** under
  `catalog/` or `examples/rosetta/`. If you judge a real corpus file is
  genuinely necessary, **STOP and escalate**: adding one trips
  `crates/ken-cli/tests/ken_fmt.rs`, `crates/ken-elaborator/tests/kenfmt_c_capstone.rs`,
  and the live 39-file fixed-point oracle — and it must **never** get a
  `FRAME_LINE_COUNTS` row (that table is a discharged historical baseline; a
  file created after it has no honest pre-frame line count, and a fabricated row
  makes its check vacuous forever).
- **Do not settle the `Bytes → Nat` substrate question.** It is an open operator
  decision. If you think you need `bytes_eq`, a `DecEq Bytes`, or an `Int → Nat`
  bridge, **STOP and escalate.**

## 6. The clause that has caught five bad pins of mine

**Treat every anchor above as perishable.** I grounded §2.2 by reading the
landed tree — but I have written five wrong pins in this program, and each was
caught only because an implementer refused to take my word for it. **If a fixed
input is false against the code, say so with exact tree anchors and escalate. Do
not quietly build around it.** I would rather be corrected than believed.

Module layering, instance homing, and scope questions route to the **Steward**.
Soundness questions route to the **Architect**.
