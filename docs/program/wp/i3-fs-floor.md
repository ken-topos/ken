# WP I-3 · CA3 — FS effect floor (current-capability)

**Owner:** Runtime (Language may pair on the Path/render packages) ·
**Terminal reviewer:** Architect (host-ABI lane) · **Size:** M · **Base:**
`origin/main @ 1f85ad9a` (post-I-2). Milestone B, Program I.

Transcribes the **Architect's ruled FS-floor design** (`evt_2sxd7pszs42af`,
Deliverable 1) into fixed inputs. **Do not reopen the design.** This is an
*enrichment of the existing `HostIO` tree* — the `FSOp` summand already exists
(I-2 left FS as real-passthrough default); you grow that placeholder algebra to
the real one, exactly as I-2 grew `ConsoleOp`.

## Headline

`HostIO` is already `ITree (Coproduct (FSOp APartial) ConsoleOp) …` with FS as
the left (`InL`) summand. I-2 landed the enabling seam (`trait HostHandler` with
real-FS passthrough default; one flat shared `IOError`). I-3 fills the FS arm
into that existing structure — **not a new spine.** Eight settled decisions,
all pinned below.

## Settled decisions (fixed inputs — do not relitigate)

**D1 · FSOp algebra — enrich the existing Auth-parameterized family in place.**
`FSOp : Auth→Type0` stays **hand-built via `declare_inductive`** (surface `data`
cannot express a `Cap a` field — the param kind is `Auth`, not `Type0`;
`prelude.rs:1224`). Keep `ReadFile`; add each new ctor as a hand-built
`CtorSpec` in the `[a]` context (`args=[Cap a, <typed args>]`), registering its
global name:
- `WriteFile (Cap a) Bytes(path) CreatePolicy Bytes(contents)`
- `AppendFile (Cap a) Bytes Bytes`
- `Metadata (Cap a) Bytes`
- `ReadDirectory (Cap a) Bytes`
- `CreateDirectory (Cap a) Bool(recursive) Bytes`
- `RemoveFile (Cap a) Bytes`
- `RemoveDirectory (Cap a) Bool Bytes`
- `Rename (Cap a) Bytes(from) Bytes(to)`

Paths stay **raw `Bytes`** (POSIX, no NFC) — same discipline as I-1 argv.

**D2 · `fs_resp` becomes a non-constant match** (mirrors what I-2 did to
`console_resp`):
- `ReadFile → Result FileError Bytes`
- `WriteFile/AppendFile/CreateDirectory/RemoveFile/RemoveDirectory/Rename →
  Result FileError Unit`
- `Metadata → Result FileError FileMetadata`
- `ReadDirectory → Result FileError (List DirEntry)`

`read_bytes`'s codomain migrates `FS a (Result IOError Bytes)` →
`FS a (Result FileError Bytes)` accordingly.

> **★ BUILD-RISK to verify first (Architect-flagged):** the scrutinee is an
> *Auth-parameterized* inductive (`fs_resp` at `prelude.rs:1263` is currently the
> constant `Result IOError Bytes`). Confirm the surface `match (op:FSOp a){…}`
> compiler accepts the value-kind param with the ctor names registered. **If it
> balks, hand-build `fs_resp` as an `Elim`** (the same escape hatch `FSOp` itself
> uses), constant motive `Type`. Decide by probing, not by assuming the surface
> match works.

**D3 · Error model — kind SHARED, structure FS-LOCAL.**
- **Kind stays the flat shared `IOError`** (`prelude.rs:232`) — *keep the name,
  no Console rename* — extended **additively**: `+AlreadyExists +InvalidInput
  +IsDirectory +NotDirectory +NotEmpty +Unsupported`, and `Other → Other Int`
  (raw errno). This arity change touches I-2's shared `io_error_kind_to_ctor`
  `_→Other` arm — supply `error.raw_os_error()`.
- **Structure is FS-only:** `data FileError = MkFileError FileOperation (Option
  Bytes) IOError` + `data FileOperation = OpReadFile | OpWriteFile | …`.
- **Console stays flat `Result IOError _`** — a Console error has no path;
  forcing it through a file-shaped record is dishonest padding. Kind is subsumed
  (one errno sum, both floors map onto it); structure lives only where
  meaningful.
- Rendering is a **package** (`renderIOError`/`renderFileError`), **never the
  driver**.
- Support types (all survey-confirmed free names): `data FileKind = KFile |
  KDirectory | KSymlink | KOther`; `data FileMetadata = MkFileMetadata Int
  FileKind` (size+kind; **defer mtime/perms** — clock/platform-dependent, tie to
  CA4); `data DirEntry = MkDirEntry Bytes FileKind` (raw-bytes name).

**D4 · Create/truncate/atomic-replace.** `data CreatePolicy = CreateNew |
CreateOrTruncate | CreateOrKeep`:
- `CreateNew` = `O_CREAT|O_EXCL` (AlreadyExists if present).
- **`CreateOrTruncate` = atomic-replace** (write sibling temp in same dir →
  fsync → `rename()` over target → best-effort dir fsync; no torn file on crash).
- `CreateOrKeep` = create-if-absent, else no-op success (idempotent) — **least
  essential; ship 2 and defer this if it complicates (non-blocking)**.
- `appendFile` = `O_APPEND`. Atomicity is per-file, **real-handler-only**; the
  virtual FS models the observable outcome, not syscall mechanics (tested
  observation, not a POSIX proof).

**D5 · Virtual-FS handler contract** — extend `trait HostHandler` with one FS
method per op, **each defaulting to real passthrough** (mirrors I-2's `fs_read`
default), so `PosixHost` inherits real behavior and only `CaptureHost`
overrides. Paths as `&[u8]`; unix builds `OsStr` via `OsStrExt::from_bytes`
(honest POSIX, no UTF-8 requirement — a small improvement over today's
`from_utf8`-or-`Other`; **v1 may keep UTF-8-path-only with a documented
limitation, non-blocking**). `CaptureHost` gains an in-memory virtual FS
(`BTreeMap<Vec<u8>, File|Dir>`, scripted initial state, `FsTrace` op sequence) —
same pattern as I-2's Console capture. Tests assert final FS state + trace +
**exact named error kinds** (the I-2 discipline: **never `is_err`**).

**D6 · Authority — coarse v1, write/delete behind AFull, LOUD caveat.** Driver
checks per-op required authority at the `authorizes` gate before syscall:
read/metadata/enumerate require `APartial`; write/create/delete/rename require
`AFull`.
- **Security caveat (state LOUDLY in runner + docs):** coarse `authorizes(cap,
  path)` *ignores path* — a write-granted program writes anywhere the process
  can. v1 write/delete are functional but **over-privileged**; ship behind
  `AFull` with the explicit "coarse authority — not path-confined" caveat.
  **Not** least-privilege until CA4/I-5 (scoped rights × scope × symlink ×
  openat-TOCTOU).
- **Sequencing (do NOT thread ProgramCaps in I-3):** composed `HostIO`
  monomorphizes FS at `FSOp APartial`, so end-to-end "a Full-granted program
  writes a real file" lights up only when **I-4** threads a Full `ProgramCaps`
  mint. I-3 tests each write/delete arm through `run_io`+virtual-FS with a
  **directly-minted `Cap AFull`** (as the i2 harness builds ops directly) —
  surface + driver + gate are fully I-3-testable without I-4.

**D7 · Zero-TCB** (same posture as I-1/I-2): all surface is ordinary
kernel-checked Ken (`declare_inductive`/`fn`) — no kernel rule/primitive/
postulate; driver arms are untrusted. **Require the `trusted_base()` before/after
equality harness AC** (the I-2 pattern). Retiring the bogus placeholders (owned
by CP0, not here) *shrinks* the trusted surface — net honesty gain.

**D8 · Path package** — ordinary Ken over Bytes: lexical join/parent/basename/
extension + **non-canonicalizing** normalization (`.`/`..`/dup-slash collapse
*without* touching the FS); `canonicalize` is a distinct FS op. **Depends on
§5.2 (CP0) safe `bytes_at`/`bytes_slice`** — so **sequence D8 after CP0**, or
ship with clamped-total local helpers. **Not I-3-core** — the FS core (D1–D7)
does not block on CP0.

## Scope boundary — I-3 vs CP0 (pipelined, no shared-file churn)

- **I-3 owns:** prelude `FSOp` + `fs_resp` + driver arms + virtual FS +
  render/Path packages. (Path D8 may lag for CP0's safe bytes ops.)
- **CP0 (separate enclave WP) owns:** `crates/ken-elaborator/src/bytes.rs`
  registration (placeholder retirement + safe Bytes/text sigs) + the consumer
  sweep. **Do NOT touch `bytes.rs` here.**
- They meet at the honesty check "no bogus FS-write primitive remains" — that
  deletion is **CP0's**, not I-3's.

## Acceptance criteria

- **AC1** — `FSOp` grows the 8 ctors (D1) via `declare_inductive` hand-built
  `CtorSpec`s; `fs_resp` is the non-constant match/`Elim` (D2); the crate builds
  and the composed `HostIO` still monomorphizes at `FSOp APartial`.
- **AC2** — one shared `IOError` extended additively with the 6 kinds + `Other
  Int` (D3); I-2's `io_error_kind_to_ctor` `_→Other` arm supplies
  `raw_os_error()`; Console's flat `Result IOError _` is **unchanged**.
- **AC3** — `FileError`/`FileOperation`/`FileKind`/`FileMetadata`/`DirEntry` +
  `CreatePolicy` land as ordinary `data`; render is a package.
- **AC4** — driver arms for all 8 ops on `PosixHost` (real, atomic-replace for
  CreateOrTruncate per D4) + `CaptureHost` virtual FS (D5).
- **AC5** — per-op authority gate (D6): read-class requires `APartial`,
  write/delete-class requires `AFull`, checked before syscall; the coarse
  "not path-confined" caveat is stated in runner + docs.
- **AC6** — discriminating exact tests through `run_io` + virtual FS: final FS
  state + `FsTrace` order + **named error kinds** (e.g. `AlreadyExists` on
  CreateNew-over-existing, `NotEmpty` on RemoveDirectory-nonempty) — **never
  `is_err`**; write/delete arms tested with a **directly-minted `Cap AFull`**
  (no ProgramCaps threading).
- **AC7** — `trusted_base()` before/after **equality** harness AC (zero-TCB);
  no kernel/`Cargo.lock`/`spec`/`conformance`/`.github` delta; **no `bytes.rs`
  delta** (that's CP0).
- **AC8** — formatter clean on any new `.ken`/package; literal
  `scripts/ken-cargo build --workspace --locked && test --workspace --locked`
  green on the exact SHA.
- **AC9 (D8, may defer)** — Path package lexical ops + non-canonicalizing
  normalize; if shipped in I-3, use clamped-total local helpers pending CP0's
  safe `bytes_at`/`slice`.

## Do-not-reopen guardrails

- Design is settled (`evt_2sxd7pszs42af`). Don't redesign the algebra, error
  model, or authority scheme.
- **No scoped/path-confined authority** — that's CA4/I-5, explicitly out.
- **No ProgramCaps threading** — that's I-4.
- **No `bytes.rs` edits, no placeholder deletion** — that's CP0.
- Don't add mtime/perms to `FileMetadata` (tie to CA4).
- Ping the Steward only at a genuine boundary the frame doesn't cover (e.g. the
  D2 build-risk needs the `Elim` escape hatch and the shape isn't obvious).

## Review chain

Runtime build (Language may pair on render/Path packages) → Runtime QA
(exact-SHA; named-error assertions; virtual-FS byte/trace exactness; directly-
minted `Cap AFull` write/delete arms; literal locked workspace) → **Architect**
terminal re-confirm → `git_request` to Steward → honesty-gate + CI-poll publish.
