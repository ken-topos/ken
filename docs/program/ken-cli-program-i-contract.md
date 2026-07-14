# Program I — Ken CLI host-ABI, effect, and capability contract

- **Status:** **Accepted** (Architect design ruling, 2026-07-13). Scope **A–C**
  of the operator-committed Ken CLI mega-effort (interpreter-first). Fixes the
  one semantic contract that the interpreter runner realizes now and the future
  native path adopts later without redefining it.
- **Deciders:** the Architect (component design); the operator (scope A–C
  committed; capability staging ruled; process toolkit **D** and native parity
  **E** out of scope for now). Framed against the Steward's Program I kickoff.
- **Scope:** the entrypoint ABI, the Console/FS/Env/Process effect families over
  the **existing** `ITree`/`Coproduct` spine, the staged capability model, and
  the driver architecture. **Zero `trusted_base()` delta** — host operations
  live in the untrusted interpreter driver; argument parsing and convenience
  APIs live in ordinary kernel-checked Ken packages. **No new kernel rules and
  no second effect system.**
- **Source:** `local/ken-cli-tooling-gap-report.md` (clean-room-clean gap report
  authored for us — read to understand; this contract is Ken's own design, not a
  lift). Every existing-spine fact below is grounded to emitted source.

## 0. The governing principle: reflect, don't extend

The gap report's decisive finding, which I independently confirmed against the
emitted source: **Ken already has the right effect and capability machinery.**
The CLI slice is not a new subsystem — it is more *constructors on families that
already exist*, more *arms on a driver loop that already trampolines*, and a
*runner that stops guessing the entrypoint and the result shape*. The whole
contract is built by the reflect-don't-extend and subsume-don't-proliferate
disciplines: add operations to the interaction-tree spine, keep OS behavior in
the untrusted driver, and make the convenience layer ordinary Ken the kernel
checks as normal code.

### The spine this contract reflects onto (grounded)

| Mechanism | What exists today | Reference |
|---|---|---|
| Interaction tree | `ITree (E:Type0) (Resp:E->Type0) (R:Type0)`, `Ret`/`Vis`; `Vis`'s continuation is dependently typed (`Resp op -> ITree …`) | `effects/state.rs:152-186`, reg. `prelude.rs:219-223` |
| A `Vis` effect family | op inductive `E` + response `fn e_resp (op:E):Type` + `proc` ops `= Vis E e_resp R (op…) (\r. Ret …)` | Console `prelude.rs:202,887,897`; FS `prelude.rs:968,1010,1043` |
| Composition | `Sum a b = InL a \| InR b`, reducing `resp_sum g h rg rh op`, coercion-free `injectL`/`injectR : ITree g rg a -> ITree (Sum g h) (resp_sum…) a` | `effects/state.rs:206,257,304,394` |
| Effect rows | `visits [E…]` is a **pure static** set-lattice pass (`EffectRow(BTreeSet<EffectName>)`), decoupled from the kernel tree; op-name→row map `io_effect_rows` | `effects/row.rs:15-51`, `bytes.rs:167-172` |
| Driver | `run_io` trampoline: per `Vis`, `peel_sum` the coproduct tag → match op ctor id (exhaustive, `_ => UnknownEffect`) → one host call → response value → `apply(k, resp)` | `ken-interp/src/eval.rs:1968-2085` |
| Capability | `data Auth = ANone\|APartial\|AFull`, opaque `Cap : Auth -> Type0`; Rust `Authority(u8)` + `Cap{authority_val, effect}`; `mint`/`attenuate`/`check_authority_sufficient`; **carried in the op** | `prelude.rs:922-945`, `capabilities.rs:30-189` |

Two facts from that table are load-bearing for the whole design:

1. **The capability is already carried *in* the operation, not read ambiently**
   (`ReadFile : (a:Auth) -> Cap a -> Bytes -> FSOp a`). Every new op keeps this.
2. **`authorizes(cap, path)` already receives the path but ignores it**
   (`eval.rs:1918`, path is `_`) — the scoped-capability seam is *pre-cut*; the
   fast-follow fills an argument the driver already has in hand.

## 1. Entrypoint ABI (deliverable 1)

Today's runner evaluates the **last declaration** (`eval.rs`→`ids.last()`,
`ken-cli/src/main.rs:70`), passes the program **no arguments** (only `args[2]`,
the source path, is read; `args[3..]` are silently ignored), and maps results to
status by **inspecting the app's result datatype** (`render_fs_result`,
`main.rs:200-240`). All three are unsafe for a tool runner and all three are
retired here.

### 1.1 The contract

```ken
data ExitCode = Success | Failure UInt8

class ProcessInput {
  arguments        : List Bytes
  environment      : List (Pair Bytes Bytes)
  workingDirectory : Bytes
}

proc main (input : ProcessInput) (caps : ProgramCaps a)
  : HostIO a ExitCode
  visits [Console, FS, Environment, Process]
```

The exact record/capability *spelling* may be refined at build time (defer
spelling, not concept), but these **semantic** choices are fixed:

- **Named resolution, not positional.** The runner resolves the declaration
  **named `main`**; missing or duplicate `main` is a hard, named error. The N4
  `program` header is anonymous, does not designate the entry point, and rejects
  `program App`. v1 uses the fixed name `main`.
- **Raw `List Bytes` argv.** POSIX `argv`, environment values, and filenames are
  byte sequences and need not be valid UTF-8. Modeling them as `String` would
  silently normalize (NFC) or reject legitimate input. **UTF-8 decoding is an
  explicit library call**, and the decode helper must be total
  (`Result DecodeError String`) — see §5's enclave ask, which this ABI depends
  on.
- **Arguments after `--`.** The runner separates its own options from the
  program's at `--`; **any unexpected pre-`--` option is rejected**, never
  silently ignored (the current silent-ignore is the dangerous behavior the gap
  report calls out).
- **`ExitCode` is total and ordinary.** The program *returns* an `ExitCode`; the
  runner maps it to process status **after the tree returns** — host termination
  is never performed mid-tree. The total map:
  `Success → 0`; `Failure n → n` for `n ∈ 1..255`; **`Failure 0 → 1`**
  (fail-closed: a failure with status 0 is a category error and must not read as
  success). This is the only result-shape knowledge the runner has — and
  `ExitCode` is a **fixed contract type**, not the app's arbitrary datatype.

### 1.2 Why this retires `render_fs_result`

`render_fs_result` couples the host to one application shape: it hard-codes that
a successful FS `main` returns `List String` and prints it, and that an `Err`
maps to a stderr line + exit 1 (`main.rs:200-240`). Under this contract, **all
program output flows through Console ops inside the tree** (the app prints its
own bytes via `Console.Write`), and the runner maps only the final `ExitCode`.
The runner therefore never inspects, prints, or branches on an application's
result datatype again — the exact decoupling deliverable 4 mandates.

### 1.3 Runner sequence

1. Resolve `main` by name; type-check its signature against §1.1 (domains
   `ProcessInput`, `ProgramCaps a`; codomain `HostIO a ExitCode`; `visits` row ⊆
   families declared by the program's capability clause).
2. Build `ProcessInput` from real argv-after-`--`, environment, and cwd (all
   `Bytes`).
3. Mint `ProgramCaps` from the program's declared capability clause (§3), one
   capability per declared mediated family.
4. `apply(main, input); apply(_, caps)`; drive the resulting
   `HostIO a ExitCode` through `run_io` (§4).
5. Map the returned `ExitCode` to process status (§1.1). Driver failures
   (`UnknownEffect`/`UnknownTree`/`NotAnIOTree`) remain loud non-zero exits.

## 2. Effect families over the existing ITree/Coproduct spine (deliverable 2)

Each family is the **standard three-part recipe** already used by Console and FS
(§0 table): an op inductive, a response `fn`, and total `proc` operations that
reduce to `Vis` nodes. Convenience (`printLine`, path ops, help) is **ordinary
package code**, never a driver primitive. Add the small algebra once; derive all
helpers in packages.

### 2.1 `HostIO` — the composed tree

`HostIO` is the interaction tree over the coproduct of the four op families,
built with the **existing** `Sum`/`resp_sum`/`injectL`/`injectR` — **no new
combinator**. Canonically right-nested:

```text
HostOp  =  ConsoleOp  +  FSOp  +  EnvOp  +  ProcessOp
                          |
                          v
HostIO a R = ITree (HostOp a) (resp_sum … chain …) R
```

Each family's ops are lifted into `HostIO` by `injectL`/`injectR` (library-side
helpers, kernel-checked). A program that uses only Console + FS still has type
`HostIO a ExitCode`; its **`visits` row** (`[Console, FS]`) is the record of
which families it actually touches. The row and the tree stay decoupled exactly
as today (`prelude.rs:1039-1042`): the row is the escape/capability annotation,
the tree is the runtime realization.

**I-4 REV-2 §A.3 resolves the `Auth`-indexed FS choice to an
authority-monomorphic program.** `FSOp : Auth -> Type0`, so the coproduct
inherits that index. For each program, the anonymous header declares one
concrete authority `a`; `main` is checked with `ProgramCaps a`, and the runner
mints exactly that declared authority.

**v1 Console remains ambient process context.** A launched Ken program may
read stdin and write stdout/stderr as any process holding file descriptors
0/1/2 may do. `ProgramCaps` therefore has no Console field and the runner mints
no Console capability. This is not per-stream capability confinement; when
that model lands, Console joins the declared capability roster with a real
value that gates its operations.

- **Selected (a), authority-monomorphic.** `HostOp` fixes the header-declared
  authority. The header, `main`'s `ProgramCaps a`, and the body's demanded
  authorities must agree; disagreement is an ill-typed program, not a runtime
  grant comparison.
- **Not selected (b), authority-polymorphic.** `main` could instead be
  polymorphic in `a`, like `read_bytes`, with the runner instantiating it at
  mint time. v1 does not choose this form; it remains a reversible alternative
  only if a future need appears.

The scoped model (§3) replaces the scalar `Auth` index with a scoped capability
*value*, at which point the index question dissolves. **Do not lift each
family's authority to a separate type index** on the composed tree — carry each
capability as a *value* in its op (as FS already does), keeping `HostOp`
singly-indexed.

### 2.2 Console — generalize `Write String` to streams of bytes

Today: one op `Write String`, driven by `println!` (`prelude.rs:202`,
`eval.rs:2003-2011`). It cannot write without a newline, write bytes, select
stderr, flush, or read. Replace with:

```ken
data Stream    = Stdin | Stdout | Stderr
data ConsoleOp =
    Read   Stream Int      -- bounded read; response carries EOF explicitly
  | Write  Stream Bytes    -- byte-exact, newline-free
  | Flush  Stream
  | IsTerminal Stream
```

Response family (per-op codomain via a `match` in `console_resp`, exactly as
`fs_resp` is a `fn`): `Read → Result IOError (ReadResult)` where `ReadResult`
distinguishes bytes-read from EOF; `Write → Result IOError Unit`;
`Flush → Result IOError Unit`; `IsTerminal → Bool`. v1 may restrict `Read` to
`Stdin` and `Write` to `Stdout`/`Stderr` in the **driver**, but the **types**
must not hard-wire `println` as the only observation. `print`, `printLine`,
`eprint`, `eprintLine` are **package helpers** over `Write`, never primitives —
do not add a trusted `print` per convenience operation.

### 2.3 FS — the whole-file surface + structured `IOError`

Today `FSOp` has one constructor `ReadFile` and `IOError` is **field-less**
(`data IOError = NotFound | PermissionDenied | CapabilityDenied | Other`,
`prelude.rs:994`). Extend the family (new constructors, each carrying `Cap a` +
typed args, each with a `fs_resp` codomain and a driver arm) to the minimum CLI
floor:

```ken
-- new FSOp constructors (each : … -> Cap a -> <args> -> FSOp a)
readFile · writeFile(CreatePolicy) · appendFile · metadata · readDirectory
        · createDirectory(recursive?) · removeFile · removeDirectory
        · rename           -- documented same-filesystem behavior

data CreatePolicy = CreateNew | CreateOrTruncate | CreateOrKeep

data IOErrorKind =
    NotFound | PermissionDenied | AlreadyExists | InvalidInput
  | IsDirectory | NotDirectory | NotEmpty | Interrupted
  | BrokenPipe | Unsupported | Other Int      -- raw errno preserved, not the API

class IOError { operation : FileOperation; path : Option Bytes; kind : IOErrorKind }
```

Each op returns a structured `Result IOError _`. **Error *rendering* is a
package**, not the driver — the driver only *constructs* the structured value.
For v1 prefer **whole-file and atomic-replace** operations; **do not** add
streaming file handles until affine/linear resource tracking exists — if handles
come first, expose a `withFile` that makes close structural and reports
close failure (never rely on comments or GC for a required close). Path stays
`Bytes` in the op; a POSIX `Path` package (lexical join/parent/extension,
*non*-canonicalizing normalization; `canonicalize` is a distinct FS op) is
ordinary Ken.

### 2.4 Env and Process — visible because they make results host-dependent

`Environment` (read one var, enumerate if separately granted, cwd query) and
`Process` (pid/platform facts where genuinely needed; **monotonic and wall
clocks as distinct ops**) each follow the same recipe. They must be visible in
effect types precisely because env/clock access makes an otherwise pure result
host-dependent — and their handlers must be injectable (§4.4) so parser and
application logic stay deterministic under test. **Child processes, signals,
temp resources, shell execution, networking are Milestone D/P2 — out of scope
here**; keep them as *separate composable families later*, never by widening
`ConsoleOp`/`FSOp` into a monolith.

## 3. Capability model — STAGED (deliverable 3)

The operator ruled this staged: the FS floor **lands on the current authority-
level capability**, and I design the **scoped model as the gating fast-follow**
that must land before write/delete is *least-privilege*. Stated as a sharp
boundary, because the difference is a security property:

### 3.1 v1 — coarse, functional, honestly over-privileged

The current model, unchanged in mechanism: scalar `Authority` {`None`,`Partial`,
`Full`} + effect label, declared in the anonymous `program` header's
`capabilities` clause and minted **exactly** from that declaration, carried in
each op, and checked `required ⊑ held` by the driver **before** the syscall
(`authorizes`/`check_authority_sufficient`, `eval.rs:1918-1932`). `ProgramCaps`
is the record delivered to `main` carrying one such capability per declared
mediated family. This contract specifies what Ken accepts as a valid program.
CLI grants, OS sandboxing, and other constraints on a running process are a
separate concern and are out of scope.

**The honest caveat, stated loudly:** coarse authority confines *nothing to a
path*. `authorizes(cap, path)` still ignores `path`. So v1 read is acceptable
(read is already gated at `APartial`), but **v1 write/delete are functional yet
over-privileged** — a write-granted program can write *anywhere* the process
can. Therefore, on v1-coarse, **write/delete ship gated behind `AFull` with an
explicit "coarse authority — not path-confined" caveat in the runner/docs**, and
are **not** advertised as least-privilege until §3.2 lands. This keeps the
green-means-safe honesty the project requires: no silent over-claim of
confinement.

### 3.2 Scoped model — what the fast-follow must add (the gate)

Before write/delete is least-privilege, the capability must carry, and the
driver must enforce:

1. **Operation rights** — a set drawn from {read, write, create, delete,
   enumerate, metadata}; a right absent ⇒ deny that op.
2. **A directory/file scope** — a granted subtree (dir handle / path prefix);
   ops outside scope deny.
3. **Symlink policy** — whether traversal is allowed and **how it is checked**
   (a symlink out of scope must not escape).
4. **Attenuation** — narrowing to a smaller scope/right set only; **never
   widening**. Reuse the existing `attenuate → (Cap, AttenuationObligation)` +
   kernel `discharge_attenuation` machinery (`capabilities.rs:138-189`) — the
   monotone-narrowing law is already there; extend its `w` from a scalar to a
   (rights × scope) meet.
5. **TOCTOU-safe enforcement** — resolve via `openat`-style **directory-relative
   handles**, **not** normalized-path-string comparison (which races and is
   escapable via `..`/absolute/symlink). This is a **driver/security design**,
   not a kernel feature — **zero TCB delta**; the enforcement lives where
   `authorizes` already sits, filling the `path` argument it already receives.

Fail-closed throughout: malformed op value → fail **before** any syscall;
missing/insufficient/out-of-scope capability → structured `CapabilityDenied`
value; unknown op → loud `UnknownEffect`.

## 4. Driver architecture (deliverable 4)

### 4.1 One arm per op, and the driver never inspects the app's result

The `run_io` loop is already the right shape (`eval.rs:1968-2085`). Every op arm
must, and only:

1. **decode** a fully typed operation value (ctor-id keyed, after `peel_sum`);
2. **check** the carried capability (§3);
3. perform **exactly one** host operation;
4. **map every outcome** to a Ken response value (success and each error kind);
5. **resume** the continuation (`apply(k, resp)`).

Dispatch stays **exhaustive with a loud `_ => UnknownEffect`** — no catch-all
success, no silent drop (`eval.rs:2069`). **Unknown ops, malformed payloads,
missing capabilities, and unsupported backend lanes fail loudly.** **No driver
arm inspects a particular application's result shape** — `render_fs_result` is
deleted (§1.2); the runner's only post-tree knowledge is the `ExitCode` map.

### 4.2 Injectable handlers for deterministic tests

Parameterize the driver over a **host-handler interface** — one method per op
family — with two implementations:

- the **real POSIX handler** (today's `std::fs`/`println!` arms behind the
  interface); and
- **in-memory handlers**: a virtual filesystem, captured stdout/stderr byte
  buffers, a scripted stdin, fixed clocks/randomness, and an asserted operation
  trace.

The existing `drive_h_instrumented` (a `Vis`-site callback + write-only trace
side-channel over an `ITreeIds` bundle, `eval.rs:2105+`) is the precedent — the
interface generalizes it from "observe" to "provide + observe." These are
**tested/delegated observations, not proofs of POSIX behavior**; pure parser and
transformation properties are still kernel-proved independently. A CLI app is
then tested with exact argv/env, a virtual FS, captured streams, scripted stdin,
and asserted traces — no real OS.

## 5. Enclave prerequisites (flag-don't-fix — what the contract needs)

Two prerequisites are the enclave's to fix; I state precisely what Program I
**requires** so the Steward scopes them correctly. **Neither should be wrapped
in a higher-level library that pretends the current shape works.**

### 5.1 Replace the placeholder FS primitives — do not wrap them

`write_bytes` and `append` are registered as `Bytes → Bytes → Bytes` primitives
(`bytes.rs:134-148`) — no path, no capability, no `Result`, **no driver arm**
(the loop only handles `Write`/`ReadFile`; any other tag is `UnknownEffect`).
They type-check bogus programs. **The contract requires them removed, replaced
by real `FSOp` constructors** (§2.3) mirroring `ReadFile`'s shape
(`… -> Cap a -> <args> -> FSOp a`, reducing to a `Vis` node with a real driver
arm). `send`/`recv` share the defect and stay out of scope (Net is P2) — but
should be removed or clearly quarantined so they cannot be imported as if
functional. **Ask:** land the extended `FSOp` constructor set + structured
`IOError` (§2.3); retire the `Bytes→Bytes→Bytes` placeholders.

### 5.2 Safe total signatures for the pure Bytes/text boundary

The entrypoint ABI's "argv is `List Bytes`, UTF-8 is an explicit library decode"
**depends on a total decode**. Today `bytes_decode : Bytes → String` goes
*neutral* on invalid UTF-8 (`bytes.rs:113-119`), and `BytesRoundTripLaw` already
presumes `decode(encode s) = Ok s` — i.e. the **spec expects a `Result`**
the registered primitive does not provide. Likewise `bytes_at : Bytes → Int →
Int` and `bytes_slice : Bytes → Int → Int → Bytes` go neutral out of bounds
(`bytes.rs:78-95`). **Ask:** land the safe `Option`/`Result` signatures the spec
specifies — `bytes_decode : Bytes → Result Utf8Error String`,
`bytes_at : Bytes → Int → Option UInt8`,
`bytes_slice : … → Option Bytes` — **before** the `Text.Codec`/`ArgParse`
packages standardize on the current unsafe shapes. CLI libraries must not freeze
the neutral-on-invalid signatures as permanent API.

## 6. Native adoptability (D/E out of scope, contract shared)

Process toolkit (**D**) and native parity (**E**) are **not designed here**. But
the contract is deliberately backend-neutral so native adopts it without
redefining what a Ken program is:

- one **checked entrypoint contract** (`ProcessInput`/`ExitCode`/`HostIO`/
  `ProgramCaps` + the required-effect and requested-capability records) that the
  interpreter runner consumes now and native packaging consumes later;
- effect families composed **as data** (`ITree`/`Sum`) — native lowers the same
  op tree; and
- the honesty rule the native path inherits: **native codegen stays unavailable
  for an effect until its runtime support and differential evidence exist**;
  interpreter success is **never** relabeled native support (Cranelift already
  rejects `RuntimeExpr::Effect`, `cranelift_backend.rs:1437-1441`). Native
  observations must compare stdout/stderr/exit/filesystem deltas, not a scalar
  return.

## 7. Suggested decomposition (Steward owns final WP scoping)

A dependency-ordered slicing that keeps each WP shovel-ready and independently
green:

- **I-1 Entrypoint ABI + runner** (Milestone A): named `main`, `ProcessInput`,
  `ExitCode`, `List Bytes` argv, `--` separation + pre-`--` rejection, exit map,
  **delete `render_fs_result`**. Acceptance: echo exact/`non`-UTF-8 args, `--`,
  exit 0/non-zero, ignored-arg rejection, missing/duplicate `main`.
- **I-2 Console floor** (Milestone B.1): `Stream×{Read,Write,Flush,IsTerminal}`
  over `Bytes`; driver arms; `print`/`printLine`/`eprint*` helper package.
- **I-3 FS floor** (Milestone B.2): extended `FSOp` + structured `IOError` +
  driver arms; POSIX `Path` package. **Depends on §5.1 (enclave placeholder
  replacement).**
- **I-4 Coarse capability threading** (Milestone B.3): `ProgramCaps` mint +
  per-family caps; write/delete gated behind `AFull` with the coarse caveat.
- **I-5 Scoped capability model** (§3.2 fast-follow): rights × scope × symlink ×
  attenuation × `openat` enforcement. **Gates least-privilege write/delete.**
- **I-6 Injectable handlers** (§4.2): host-handler interface + in-memory FS /
  captured streams / scripted stdin / fixed clock; deterministic app tests.
- **I-7 Env/Process families** (P1): env read/enumerate, cwd, clocks.
- **Enclave prereqs** (§5): parallel, independently shippable.
- **Package floor** (Milestone C — `ArgParse`, codecs, diagnostics, `Doc`,
  `Validation`, `Cursor`): **hard-blocked on the N2 cross-file loader**
  (`docs/program/wp/n2-in-repo-loader.md`) — until N2 lands, these check
  independently but cannot form the imported dependency graph. Route the N2
  dependency as the critical path for Milestone C.

## Revisit if

- **N2 cross-file loading** lands — unblocks the Milestone C package floor and
  turns the catalog closure (`ArgParse` over `Cursor`/`Validation`/`Diagnostic`/
  `Doc`) into a real imported graph rather than inlined code.
- **The N4 program header** grows an `entry`/entrypoint field — the named-`main`
  convention (v1) migrates to a header-declared entrypoint name.
- **Affine/linear resource tracking** lands — streaming file handles become
  expressible with a close obligation, relaxing the v1 whole-file-only
  restriction (§2.3).
- **Scoped capability model (§3.2)** is built — write/delete drop the `AFull`
  coarse gate and become least-privilege; the `Auth`-index composition tension
  (§2.1) dissolves into a scoped capability *value*.
- **Native parity (E)** is scheduled — it consumes this contract; it does not
  redefine it. Any effect lowered needs runtime support + differential evidence
  first.
