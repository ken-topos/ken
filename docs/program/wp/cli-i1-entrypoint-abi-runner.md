# WP CLI I-1 — entrypoint ABI + runner

- **Program:** Ken CLI application tooling, **Program I** (host-ABI/effects).
  Milestone **A** — the foundation every later CLI WP builds on.
- **Owner:** Runtime ring (leader + implementer + QA). Core is the interpreter
  runner + `run_io` driver, no surface-syntax change in v1.
- **Reviewer:** Architect-terminal (owns the Program I / host-ABI contract).
- **Size:** M. **Risk:** medium (it changes the runner's result contract, but
  the blast radius is `ken-cli` + the driver's post-tree mapping — zero TCB).
- **Branch:** `wp/cli-i1-entrypoint-abi-runner` (off `origin/main @ 50d187bb`).
- **Deps:** none. Program I contract is published (`origin/main`). Ready now.

## Objective

Replace the three unsafe behaviors of today's runner with the fixed **entrypoint
ABI** from the contract: resolve `main` **by name** (not last-decl), pass it a
real **`ProcessInput`** (argv-after-`--`, environment, cwd as raw `Bytes`), and
map its returned **`ExitCode`** to process status **after** the tree returns —
**deleting `render_fs_result`** so the runner never again inspects an
application's result datatype.

## Fixed inputs (settled — do NOT re-decide)

The design is **ruled** in `docs/program/ken-cli-program-i-contract.md` §1
(published on `origin/main`). Treat it as the spec. The load-bearing decisions,
pinned:

1. **Named resolution.** Resolve the declaration **named `main`**. Missing `main`
   and duplicate `main` are each a **hard, named error** (not a fallback to
   last-decl). v1 uses the fixed name `main`; the N4 `program { entry = … }`
   header is a later fast-follow, **out of scope here**.
2. **`ProcessInput`.** `main` receives arguments, environment, and working
   directory as **raw `Bytes`** (`arguments : List Bytes`,
   `environment : List (Pair Bytes Bytes)`, `workingDirectory : Bytes`). POSIX
   argv/env/filenames are byte sequences — **do not** model them as `String`
   (that would NFC-normalize or reject legitimate input). UTF-8 decoding is an
   explicit **library** call, not part of this ABI. The record *spelling* may be
   refined at build time (defer spelling, not concept); the **semantic** shape is
   fixed.
3. **argv after `--`.** The runner separates its own options from the program's
   at `--`. Everything after `--` becomes `arguments`. **Any unexpected pre-`--`
   option is rejected with a named error** — never silently ignored (today's
   silent `args[3..]` drop is the exact hazard the gap report calls out).
4. **`ExitCode` is total, ordinary, mapped after the tree.**
   `data ExitCode = Success | Failure UInt8`. The program *returns* it; the
   runner maps **after** `run_io` returns — never terminates mid-tree. The total
   map is fixed: `Success → 0`; `Failure n → n` for `n ∈ 1..255`; **`Failure 0 →
   1`** (fail-closed — a failure with status 0 is a category error and must not
   read as success). `ExitCode` is a **fixed contract type**; the runner has no
   other result-shape knowledge.
5. **Delete `render_fs_result`.** All application output flows through Console ops
   *inside* the tree (I-2 lands the Console floor; until then, keep the existing
   `Write`/`ReadFile` driver arms working). The runner's only post-tree knowledge
   is the `ExitCode` map — it must never inspect, print, or branch on an app's
   result datatype again (`main.rs:200-240` `render_fs_result` is removed).

**Current-state anchors (verify against landed code, do not trust this line):**
today's runner does last-decl resolution (`ken-cli/src/main.rs:100`
`ids.last()`), ignores `args[3..]`, and maps status by inspecting the app value
(`render_fs_result`, `main.rs:230`). `run_io` (`ken-interp/src/eval.rs:1975`)
already trampolines with an exhaustive `_ => UnknownEffect` (`eval.rs:2076`) —
**keep that discipline**; driver failures stay loud non-zero exits.

## Mandated deliverable outline (each item ends in a concrete choice)

1. **`ExitCode` + `ProcessInput` + `ProgramCaps` types.** Introduce them as
   ordinary kernel-checked Ken declarations (prelude or a CLI package — builder's
   call; cite where). `ProgramCaps` is the record delivered to `main`, one
   capability per granted family; for I-1 with only Console/FS reachable, it
   carries the current-authority `Cap` mint (unchanged mechanism — I-4 does the
   real per-family threading). Keep `HostOp` **singly-indexed** — carry the cap
   as a *value* in the op, do not lift authority to a tree type index (contract
   §2.1).
2. **Named entrypoint resolution** in the runner: look up the decl named `main`;
   missing/duplicate → distinct named errors. Type-check its signature against
   the ABI (domains `ProcessInput`, `ProgramCaps`; codomain `HostIO ExitCode` —
   or the v1 monomorphic-authority spelling per contract §2.1(a), **recommended**;
   `visits` row ⊆ granted families).
3. **`ProcessInput` construction** from real argv-after-`--`, process
   environment, and cwd — all as `Bytes`. **`--` splitting + pre-`--` option
   rejection** with a named error.
4. **`ExitCode` → process status** mapping applied **after** `run_io` returns,
   with the fixed total map incl. `Failure 0 → 1`. Driver failures
   (`UnknownEffect`/`UnknownTree`/`NotAnIOTree`) remain loud non-zero exits.
5. **Delete `render_fs_result`** and every branch that inspected the app value;
   re-point the existing FS/Console example programs/tests onto the new ABI (a
   `main : … → HostIO ExitCode` that writes via the existing driver arms). If any
   in-tree example relied on the old `List String`-printed shape, migrate it.

## Acceptance criteria (testable — assert specific variants, not `is_err`)

- **AC1 — named resolution.** A program whose `main` is not the last decl still
  runs; a program with **no** `main` and one with **two** `main`s each fail with
  the **distinct named error** (assert the specific error, not just failure).
- **AC2 — argv passthrough incl. non-UTF-8.** Args after `--` arrive verbatim as
  `List Bytes`, including a **non-UTF-8 byte sequence** (round-trips unchanged, no
  normalization/rejection). Args before `--` that are unknown options are
  **rejected with the named error**; the source-path arg is still consumed.
- **AC3 — ExitCode map (all arms).** `Success → 0`; `Failure 3 → 3`;
  `Failure 255 → 255`; **`Failure 0 → 1`** (fail-closed). Assert the exact
  process status for each.
- **AC4 — render_fs_result gone.** `render_fs_result` (and any app-result
  inspection) no longer exists in `ken-cli`; grep-clean. Output observed in AC2/AC5
  flows through the driver's Console/Write arm, not a runner-side print of the
  app value.
- **AC5 — driver failures stay loud.** An unknown effect op still yields a
  non-zero exit via `UnknownEffect` (not a silent success). The exhaustive
  `_ => UnknownEffect` arm is intact.
- **AC6 — locked workspace CI.** Literal `cargo build --workspace --locked &&
  cargo test --workspace --locked` green on the exact final SHA (first-class QA
  gate — a rosetta/.ken fixture may live in a wrapper-skipped crate).

## Do-not-reopen guardrails

- **No new kernel rules, no second effect system, zero `trusted_base()` delta.**
  Host ops live in the untrusted interpreter driver; `ExitCode`/`ProcessInput`/
  `ProgramCaps` are ordinary kernel-checked Ken. If you find yourself adding a
  trusted primitive, **stop and flag the Steward** — that is a contract breach.
- **Do not** design the Console or FS op surface here (I-2/I-3), the coarse or
  scoped capability model (I-4/I-5), or the injectable handler interface (I-6).
  I-1 keeps the **existing** driver arms working and only fixes the
  runner/entrypoint/exit contract.
- **Do not** re-decide any §1 semantic (named-main, `Bytes` argv, `--` split,
  the exit map). Spelling refinements are fine; concept changes come back to the
  Steward.
- **Do not** silently ignore any input. Every rejected option / malformed
  entrypoint / driver failure is a **named, loud** error.

## Flow

Runtime builds one atomic `ken-cli` (+ driver post-tree mapping) lane →
QA gates (incl. literal locked CI on the exact SHA, specific-variant asserts) →
**Architect-terminal** review (owns the Program I contract) → `git_request` to
the Steward → honesty-gate + CI-poll publish → **I-1 CLOSED**. Then **I-2**
(Console floor) follows in the same ring.
