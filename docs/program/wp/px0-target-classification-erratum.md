# PX0 — target-classification erratum: the host ABI is Linux, not `unix`

**Owner:** Runtime · **Size:** S · **Depends on:** nothing · **Blocks:** nothing
· **Precedes** the POSIX/Linux ABI campaign (`docs/program/09-posix-linux-abi-campaign.md`)

> **This is an ERRATUM on landed code, not campaign work.** It is released ahead
> of the campaign on the Architect's ruling (`evt_7qqf827rr1jxk`): *"the current
> wrong-target path should be quarantined before the campaign… do not wait behind
> CC9."* **PX1/PX2 will replace this code entirely. PX0 makes it HONEST in the
> meantime — it does not make it good.**

> **Treat every anchor below as PERISHABLE.** Each was verified against
> `origin/main @ e2a7ac75`. If a fixed input is false against the tree, **say so
> with exact anchors and ESCALATE** — do not quietly build around it.

## 1. The defect

`crates/ken-interp/src/eval.rs` gates its entire POSIX host boundary on
**`#[cfg(unix)]`** while the values it encodes are **Linux's**.

`#[cfg(unix)]` selects Linux, macOS, and every BSD. **Thirteen hand-asserted ABI
facts and six `unsafe extern "C"` declarations compile on all of them:**

| What | Where | Count |
|---|---|---|
| `*_KEN` flag constants (`O_RDONLY` … `AT_REMOVEDIR`) | `eval.rs:2355-2375` | **11** |
| `unsafe extern "C"` — `openat`, `mkdirat`, `unlinkat`, `renameat`, `readlinkat` | `eval.rs:2378-2394` | **5 fns** |
| `unsafe extern "C" { fn signal }` + `SIGPIPE = 13` + `SIG_IGN = 1`, **nested inside `mask_sigpipe()`** | `eval.rs:3714-3730` | **1 fn, 2 facts** |

**On a non-Linux Unix this code compiles, links, and hands those bits to real
syscalls.** Every test stays green, because the tests run where the numbers
happen to be right.

### ★ Why this is a SECURITY erratum and not a portability nit

**`O_NOFOLLOW_KEN` (`eval.rs:2371`) is the enforcement mechanism for
`SymlinkPolicy::NoFollow`** (`crates/ken-elaborator/src/capabilities.rs:89-92`)
— an **ADR-0017 capability-confinement property.** `O_DIRECTORY_KEN`,
`O_EXCL_KEN`, and `O_CLOEXEC_KEN` are load-bearing in the same path.

> **A capability-confinement guarantee is resting on a number nobody probed,
> under a `cfg` gate wider than the fact it encodes.** Whether the values are
> right on *today's* box is **not the point**. The point is that the artifact
> **cannot tell you**, and a wrong value does not fail — it **silently does not
> confine.**

**⛔ A green test run on this Linux box CANNOT discharge this.** The defect *is*
target-classification. Testing it on the one target where it is correct is the
same mistake in a different costume.

## 2. Deliverable — narrow, and only this

**Reclassify the boundary from `unix` to `linux`, and make every other target
fail LOUDLY and EARLY.**

1. **Re-gate** the constants, the two `unsafe extern "C"` blocks, and every host
   call site that consumes them from `#[cfg(unix)]` to
   **`#[cfg(target_os = "linux")]`**.
2. **Every non-Linux target returns an explicit `Unsupported`/unavailable result
   BEFORE any host call is attempted** — never a wrong-flag syscall, never a
   silent fallback, never a no-op. **Follow the existing unavailable-lane
   discipline** (`platform_runtime_support.rs` is the house pattern: a *named,
   stable* unavailable lane, not an error).
3. **Do NOT change any value, any signature, or any behavior on Linux.** This is
   a **classification** change. `git diff` must show no Linux-observable
   behavior delta.

## 3. Acceptance criteria

- **AC1 — the gate matches the fact.** No `#[cfg(unix)]` remains on any
  construct carrying a Linux-specific ABI value. Grep is the *candidate finder*;
  **enumerate the full inventory and state the count** (see the trap in §5).
- **AC2 — non-Linux fails closed, EARLY.** On a non-Linux target, a host FS
  operation returns a **named unavailable lane before any syscall**. Assert the
  **specific** variant, not `is_err()`.
- **AC3 — Linux behavior is BIT-IDENTICAL.** The existing FS/Console/Clock
  driver suites pass unchanged, and no expected value moved.
- **AC4 — `mask_sigpipe`'s FFI boundary is covered.** It is a *sixth* FFI
  declaration hiding **inside a function body** and it carries two more ABI
  facts. It is in scope. **Either re-gate it with the rest, or state plainly why
  it is exempt** — do not silently leave it on `cfg(unix)`.
- **AC5 — ZERO `trusted_base()` delta.** No new postulate, primitive, or `Axiom`.
  `git diff -- crates/ken-kernel Cargo.lock` is **empty**.

## 4. What this WP is NOT

- **⛔ NOT the `ken-host` extraction.** That is **PX1**. Do not move code between
  crates; do not create a crate.
- **⛔ NOT the ABI manifest.** That is **PX2**. **Do not "fix" a constant.** You
  are not being asked whether `0o400000` is right — you are being asked to stop
  claiming it on targets where we never checked. *Correcting a value you have not
  probed would repeat the exact defect with a fresher number.*
- **⛔ NOT a `rustix` migration.** The Architect ruled (b) for the campaign
  (`evt_7qqf827rr1jxk`); **that lands in PX1**, and it is pending Pat's
  dependency-risk acceptance. PX0 must not pre-empt it.
- **⛔ NOT macOS/BSD support.** If we ever want those, they are a **manifested
  backend with their own probed ABI facts** — a real WP, not a widened `cfg`.
  PX0's whole point is that the widened `cfg` was the bug.

## 5. ★★ The trap that already caught the Steward — read this before you grep

**I wrote this campaign's charter claiming *three* constants and *five* FFI
declarations. It is ELEVEN and SIX.** I was wrong by a factor of 3.6, in a
document whose entire thesis is *"the artifact cannot state its own contract."*

**Here is exactly how, because you are about to do the same thing:**

- I read the file from **line 2370**, because a prior audit had cited
  `AT_REMOVEDIR` at `:2375`. **The inventory starts at `:2355`.** I picked my
  window from *a citation of one instance* rather than from *the extent of the
  kind*. **A line number tells you where something IS. It does not tell you where
  its KIND begins and ends.**
- I grepped `unsafe extern` and got **two** hits. **I read one.** The second is
  **indented, inside a function body** — it does not look like a boundary at a
  glance.

> **⇒ ENUMERATE, then count, then state the count in your handoff.** A grep
> **selects candidates; it never decides.** If your handoff says *"I re-gated the
> constants,"* it is not done. It must say ***"there are N; here are all N; all N
> are re-gated"*** — and N must be a number you counted, not one you inherited.

## 6. Guardrails

- **⛔ Build/test TARGETED ONLY** — `scripts/ken-cargo -p ken-interp` /
  `--test <name>`. **Never `--workspace`** (`COORDINATION.md §12`, operator hard
  rule). CI owns workspace-green.
- **⛔ A behavior test on this Linux box cannot discharge AC2.** Prove the
  non-Linux lane **structurally** — by the `cfg`/dispatch shape and a
  target-classification unit test — not by an execution you cannot run here.
- **The honesty statement is the deliverable.** After PX0, "Ken's host boundary
  supports Linux" is a *true sentence we can defend*. Before PX0, it was a
  sentence nobody had checked.
