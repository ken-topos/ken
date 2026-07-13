# WP — I-2 · CA2 Console effect floor (Program I, Milestone B)

Owner: **Runtime** (host-ABI spine; may pair Language on the package helpers).
Reviewer: **Architect** (terminal — host-ABI). Size: **M**. Base:
`origin/main @ bd9e6c4b` (post-I-1). Deps: **I-1 merged (done)** — extends I-1's
composed `HostIO` tree. **NOT** dependent on §5.1 (FS placeholders — I-3) or §5.2
(`bytes_decode`, HELD): the floor is `Bytes`-in/`Bytes`-out and helpers use the
*encode* direction only.

## Frame status — SHOVEL-READY

The Console algebra is fixed by the Program I contract §2.2; the host-ABI
realization + six build-level decisions are **ruled by the Architect**
(`evt_6w0x364s4bvk8`, grounded on the landed `bd9e6c4b` spine) and pinned below
as **fixed inputs**. The implementer executes mechanically — **do not reopen the
design.** Any "current state" line is perishable: verify against the landed code
at pickup, not this doc.

## Headline (settled)

I-2 is an **extension of an already-composed tree, not a new spine.** `HostIO`
is already `ITree (Coproduct (FSOp APartial) ConsoleOp) …` (`prelude.rs:1175-1180`)
with **Console as the right (`InR`) summand** — today a placeholder
`data ConsoleOp = Write String` / `console_resp = \_.Unit`. I-2 grows that
summand to the real algebra. No new spine, no new combinator, one effect-row
entry (`Console`), one real spine-shape change (`console_resp` becomes
non-constant).

## Fixed inputs — the six decisions (DO NOT REOPEN)

**1. Encoding + reflection — ONE `ConsoleOp` former (not per-stream).**
Extend today's ctor to:
```
data Stream    = Stdin | Stdout | Stderr
data ConsoleOp = Read Stream Int | Write Stream Bytes | Flush Stream | IsTerminal Stream
```
Plain surface `data` — Console is **not** Auth-indexed at the type level (unlike
`FSOp`; see #2). It lifts into `HostIO` via the **existing** `inject_r` (Console
is already `InR`) over `resp_coproduct` — **no new combinator.** `Console` stays
one effect-row entry under I-1's existing `⊆{Console,FS}` guard
(`main.rs:414-422`). The ops are total `proc`s reducing to `Vis` (same shape as
today's `print_line` `prelude.rs:980-984` and `read_bytes` `prelude.rs:1126-1131`).
**The one real spine-shape change:** `console_resp` goes from the constant
`\_.Unit` to a **non-constant large-elim** — ordinary kernel-checked Ken, the
same shape as `resp_state` (`state.rs:231-249`):
```
console_resp op = match op {
  Read _ _      ↦ Result IOError ReadResult
  Write _ _     ↦ Result IOError Unit
  Flush _       ↦ Result IOError Unit
  IsTerminal _  ↦ Bool
}
```

**2. Capability — v1 Console is AMBIENT; `ProgramCaps` stays UNCHANGED.**
A CLI process definitionally holds its own stdio, so at the current coarse
authority there is nothing to gate. `main (input) (caps : ProgramCaps) : HostIO
ExitCode` is unchanged; `ProgramCaps` stays `MkProgramCaps (Cap APartial)` (FS
authority only); `ConsoleOp` carries **no** per-op cap. The `visits [Console]`
row is the static "uses Console" record; the runner grants it unconditionally.
This does **not** violate contract §2.1 ("carry authority as a value, not a type
index") — there is simply no v1 Console authority to carry. **Scoped Console
authority (a `ProgramCaps` field + a per-op `ConsoleCap` *value*) is a
fast-follow, OUT OF SCOPE for I-2**, introduced only when per-stream restriction
(deny-stderr, stdin-redaction) becomes real. **Do NOT add a coarse no-op Console
`ProgramCaps` field now — a field that gates nothing is honesty-theater
(Architect explicitly rejected it).**

**3. Totality — no host exception crosses the ABI (mirror `fs_resp`).**
Via the #1 `console_resp` match:
- `Read → Result IOError ReadResult` with `data ReadResult = Chunk Bytes | Eof`.
  **EOF is a total value — the `Eof` variant, distinct from a zero-length
  `Chunk`.** Read past EOF → `Eof`.
- `Write / Flush → Result IOError Unit`. **Broken-pipe is a total `IOError`
  value, not a signal.**
- `IsTerminal → Bool` (isatty), total.

The driver arm **mirrors the FS arm's** `make_result` / `io_error_kind_to_ctor`
(`eval.rs:1943-1960, 2043-2070`) so it never panics. **Driver MUST mask SIGPIPE**
(`SIG_IGN`) and map `io::ErrorKind::BrokenPipe → BrokenPipe` value.

**4. Error type — share ONE `IOError` (subsume-don't-proliferate).**
I-2 extends today's `IOError = NotFound | PermissionDenied | CapabilityDenied |
Other` (`prelude.rs:1077`) **additively** with `BrokenPipe` (+`Interrupted`).
I-3 extends it further for FS. **Do NOT create a parallel `ConsoleError`.**

**5. Zero-TCB — CONFIRMED.** `Stream`/`ConsoleOp`/`ReadResult` = surface `data`;
`console_resp` = a `fn` (match); the ops = `proc`s reducing to `Vis`; the
`IOError` extension = additive ctors — **no new kernel rule / trusted primitive /
postulate.** `print`/`printLine`/`eprint`/`eprintLine` are ordinary Ken **package**
helpers over `Write`, e.g. `printLine s = write Stdout (bytes_encode (s ++ "\n"))`.
Today's `print_line` is *already* a derivable `proc`; the hardwired
`build_print_line_tree` primitive is now **dead code** (`prelude.rs:995-998`,
`eval.rs:1640-1669`) — **remove it.** The only untrusted code is the Rust driver
arms (same posture as I-1).

**6. Injectable capture seam — MUST BE BUILT (it doesn't exist on the CLI path).**
`run_file` calls the hardwired `run_io` (`println!`/`std::io` baked into arms);
the parametric `drive_h`/`drive_h_instrumented` (`eval.rs:1701, 2112`) exist but
are **unwired** to `run_io`. I-2 **parameterizes `run_io` over a host-handler
trait** — Console methods `console_write` / `console_read` / `console_flush` /
`console_is_terminal` — with two impls:
- the **real POSIX handler** (today's arms moved behind it, SIGPIPE masked);
- an **in-memory capture handler** (`Vec<u8>` stdout/stderr buffers,
  scripted-stdin cursor, fixed `IsTerminal`, asserted op-trace).

This generalizes `drive_h_instrumented` from observe-only → **provide+observe**
(contract §4.2). **I-2 defines the trait with the Console methods (FS passthrough
default); I-3 fills the FS virtual impl** — land the trait here as shared infra.

## Migration flags (in-package, small)

- (a) `Write String` → `Write Stream Bytes`; adjust the `print_line` helper.
- (b) `IOError` gains `BrokenPipe` / `Interrupted` — additive; I-3 inherits.
- (c) `run_io` handler-parameterization is shared infra I-3 extends — land the
  trait in I-2.
- (d) Remove the dead `build_print_line_tree` primitive.

## Mandated deliverable outline

1. **Prelude / algebra** — `Stream`, the grown `ConsoleOp`, `ReadResult`, the
   non-constant `console_resp` match, the additive `IOError` ctors; the ops as
   `proc`s reducing to `Vis` via existing `inject_r`. Zero kernel/TCB delta.
2. **Package helpers** — `print`/`printLine`/`eprint`/`eprintLine` in an ordinary
   Ken package over `Write` (stdout vs stderr routing correct; `printLine`
   appends exactly one `\n`). Not primitives.
3. **Driver** — Console arms mirroring the FS arm's total mapping; SIGPIPE
   masked; `BrokenPipe`/`Interrupted`/EOF as values; `IsTerminal` via isatty.
4. **Handler seam** — the host-handler trait (Console methods, FS passthrough
   default); real POSIX impl + in-memory capture impl; `run_io` parameterized
   over it.
5. **Cleanup** — delete the dead `build_print_line_tree`; grep-confirm gone.

## Acceptance criteria (testable)

- **AC1** — the `ConsoleOp` algebra + non-constant `console_resp` land as
  specified; whole thing kernel-checks; **zero TCB delta** (no new kernel
  rule/primitive/postulate; `ProgramCaps` unchanged; `build_print_line_tree`
  grep-clean gone).
- **AC2** — **byte-exact writes**: `write s bytes` emits exactly `bytes` (no added
  encoding); `printLine` appends exactly one `\n`; stdout vs stderr routing
  correct.
- **AC3** — bounded `Read Stream n` returns `Chunk Bytes` (≤ n bytes) or `Eof`;
  **EOF is the `Eof` variant** (not a zero-length `Chunk`); read past EOF → `Eof`.
- **AC4** — **broken-pipe is a value**: writing to a closed pipe yields
  `IOError BrokenPipe`, does not crash, SIGPIPE masked. (Assert the specific
  `BrokenPipe` ctor, not merely `is_err`.)
- **AC5** — `IsTerminal Stream → Bool` (isatty), total.
- **AC6** — `print`/`printLine`/`eprint`/`eprintLine` are **package** helpers over
  `Write` (grep: not kernel primitives); derivation is ordinary kernel-checked
  Ken.
- **AC7** — the **in-memory capture handler** captures stdout/stderr bytes
  exactly, scripts stdin, fixes `IsTerminal`, and asserts the op-trace; a
  `#[test]` uses it; the real POSIX handler sits behind the same trait.
- **AC8 — scope/zero-TCB.** No `crates/ken-kernel/**` delta, no `Cargo.lock`
  delta; `IOError` extension additive; `ProgramCaps` unchanged; effect-row guard
  unchanged (Console already an entry). If any `spec/`|`conformance/` byte moves
  (the Console algebra lives in the program contract, not `spec/` — expect none),
  **CV loops in**; otherwise none.
- **AC9** — literal locked workspace green on the exact SHA:
  `scripts/ken-cargo build --workspace --locked && … test --workspace --locked`.

## Do-not-reopen guardrails

- **One** `ConsoleOp` former (not per-stream). **Ambient** v1 Console — no per-op
  cap, `ProgramCaps` unchanged, **no honesty-theater no-op cap field.** **One
  shared** `IOError` (extend additively — no `ConsoleError`). EOF/broken-pipe are
  **total values**, never host signals. Scoped Console authority is a
  **fast-follow, not I-2.** §5.2 safe-decode is **not** a dependency. The handler
  trait is **built here** (shared infra for I-3), not deferred.

## Review & close

Runtime build (Language may pair on the package helpers) → Runtime QA (exact-SHA
gate: AC1–AC9, named-error assertions for `BrokenPipe`/`Eof`, capture-handler
byte-exactness, literal locked workspace) → **Architect** terminal re-confirm
(the six decisions landed as ruled; zero-TCB; handler seam generalizes
observe→provide+observe) → `git_request` to Steward → honesty-gate + CI-poll
publish. **On merge: I-2 CLOSED → I-3 (FS effect floor) next in Runtime ring**
(inherits the `IOError` extension + the handler trait's FS impl).
