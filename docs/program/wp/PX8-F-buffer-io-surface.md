# PX8-F — Foundation `System.Buffer`/`System.IO` surface + derived `writeAll` proof

> **▶ RESUME NOW (2026-07-20) — ALL PX8-F PREREQUISITES HAVE LANDED; current
> `origin/main = 8d761bc5`.** Full native effect-lowering chain merged: PX8-N
> `ace72db7`, PX8-X train `a97b4304`, PX8-L `e74e935f`, PX8-H `aab1f831`, PX8-I
> `38ed8223`, PX8-J `a9db5a17`, PX8-TA `e45ca05e`, PX8-DS `765c73ac`, and
> PX8-TR `8d761bc5`. Frozen candidate = **`376773b6` on the local
> `wp/px8f-buffer-io-surface` branch** (base `765c73ac`).
> **⚠ SEMANTIC REBASE, not delta-apply:** replay `376773b6`'s three PX8-F
> commits (catalog Buffer/IO + prelude + interp eval + surface/native fixtures)
> ONTO current `8d761bc5`; NEVER raw-diff-and-apply (that can revert the
> native-lowering chain). Re-derive every anchor against the rebased tree. The two
> test-only fixture riders are on `origin/steward/work`. Resume per the sequence
> below** (Architect rulings
> `evt_291b8gcwde32v` + `evt_22y0emyj6cwf2` + `evt_5v6jrc6rnva`, 2026-07-17).
> PX8-F hard-stopped on a real substrate gap:
> the checked surface needs `BufferSpan` to carry a **constructor-private
> structural `Nat`** budget that transparent `writeAll` eliminates, but the host
> reply is scalar-only and the native lanes are represented-unavailable. The
> preserved candidate is durable at **`preserved/px8f-held-60a481b5`** (= branch
> `wp/px8f-buffer-io-surface @ 60a481b5`, parent frame `abf99596`), with the
> minimized `px8f_buffer_native` fixture. **Both now-landed predecessors (rebase
> onto their combined result, current `origin/main @ ace72db7`):**
> - **PX8-N** (`docs/program/wp/PX8-N-*.md`, Team Runtime) supplies the native
>   **bounded-`Nat` reply-lowering carrier** so the structural-`Nat` budget is
>   constructible on the native lane (fixes this hard-stop).
> - **PX8-X train** (`docs/program/wp/PX8-X-single-schema-unification-train.md`,
>   Runtime+Verify+Spec) collapses the observation/obligation schema to ONE
>   unversioned `ResourceLifetimeObligation` — the obligation PX8-F emits.
>
> On resume, Foundation **rebases the preserved candidate once** onto the combined
> main, **amends the frame to the sole unversioned obligation** (drop all V1/V2
> obligation assertions), **rebaselines resource-producing export evidence once**,
> **retains the no-acquire negative control unchanged**, **authors the FIRST real
> linked checked `writeAll` native fixture** (constructible only now via PX8-N —
> none existed before; the preserved `px8f_buffer_native.rs` is a positioned-READ
> fixture, not `writeAll`, per Architect `evt_4kh6gz18tvzs6`), and only then takes
> fresh **Foundation QA + Architect §14 + CV**.
>
> **⚠ Ward is a SEPARATE project, not part of Ken; Ken does not implement Ward's
> monitor** ([[obligations-route-to-ward-not-into-ken]], `G-Ward-seam`): the
> earlier PX8-W Ward-consumer WP is **STRUCK from Ken** (its monitor + RB-A…RB-O
> behavioral cases belong to the external Ward project). PX8-F emits the surface +
> the honest `status: delegated` obligation via the landed export; it does **NOT**
> discharge it, and neither does any other Ken WP. It still carries its focused
> **Verify short-write/`NoProgress` differential companion** (Ken's own conformance
> differential, not Ward discharge — folded on the rebased branch). See Sequencing.

- **ID:** PX8-F · **Owner:** **Team Foundation** (leader `agt_37reqsbs5b000` /
  implementer `agt_37reqg89sn800` / qa `agt_37reqw9dhdg00`) · **Size:** L ·
  **Risk:** High (first Buffer *use*-surface; the derived `writeAll` must be
  **real kernel-checked terms — no `Axiom`, no postulated theorem**; a new
  prelude/catalog surface that can break checked-program exhaustiveness consumers
  only full-workspace CI catches — the PX8-R/PX8-P lesson).
- **Branch:** `wp/px8f-buffer-io-surface` — held at `60a481b5` (preserved at
  `preserved/px8f-held-60a481b5`). On resume it **rebases once onto the combined
  main** after the PX8-X train + PX8-N land (NOT off `f2f60083` any more — the
  obligation schema and the native `Nat` lowering both change underneath it).
  Foundation reconstructs anchors from the combined main at unfreeze (a held-WP
  unfreeze is a semantic rebase — re-derive every anchor; do not trust the frozen
  line numbers below without re-grepping the rebased tree).
  **(Live-state note: the hold point advanced to `c8b8cdb7` during the
  PX8-L/H/I/J native-lowering chain; `60a481b5` above is stale — reconcile the
  held SHA at unfreeze, Task #11.)**
- **⚑ UNFREEZE RIDERS (added while held — apply at unfreeze):**
  - **`PX8-F-fixture-rider-pwrite64.patch`** (this dir) — a **test-only** repair
    to `crates/ken-cli/tests/px8f_buffer_native.rs`: adds a `pwrite64` LD_PRELOAD
    wrapper (`dlsym RTLD_NEXT "pwrite64"`, `off64_t`, same `min(count,2)` cap)
    beside the existing `pwrite`, so the short-write interposer bites the symbol
    the host `FileExt::write_at` actually calls (`pwrite64@GLIBC_2.2.5`), not the
    unreached `pwrite`. **Why held here:** PX8-J proved its terminal-answer
    continuation mechanism against this overlay (real ordered `FsWriteAt`, exact
    `abcdef`, exit 0, interpreter agreement), but the frozen `writes.len()==3`
    oracle was mechanically unreachable because the shim missed `pwrite64`. The
    repair is a **PX8-F fixture fix, not a PX8-J change** — it was kept off the
    PX8-J production branch and custodied here. **Provenance:** Architect ruling
    (refined-b, thr_618c 2026-07-19 07:41Z) + Steward ownership ruling
    `evt_3mv6g5v5ksnyy` + implementer `code_share` `evt_6a1syfq0a2cv3`. Base blob
    `6378c31d` (hunk `@@ -161`). **At unfreeze:** `git apply` it (or re-derive the
    hunk against the rebased fixture), then confirm the discriminator is live —
    removing only the `pwrite64` wrapper must collapse the native trace to one
    write and fail the three-write assertion. **No** `pwritev` (the vectored API
    is imported but unreached; interposing it is not minimal evidence).
  - **`PX8-F-fixture-rider-posix-interp-leg.patch`** (this dir) — a **test-only**
    repair to the same file: the interpreter differential leg was mechanically
    unreachable because `InterpreterHostBackend::fs_open_resource` accepts only
    `FsHandle::Posix` and rejects `CaptureHost`'s virtual root as
    `Capability(ScopeEscape)` (exit 81, zero writes) **before** any `input.bin`
    lookup — a host-KIND mismatch, not a relative-path miss. The rider **keeps**
    the old `CaptureHost` leg as a control (asserting the pre-lookup `FsOpen` +
    `Capability(ScopeEscape)`/exit 81), then runs the real differential through
    `PosixHost::new_at(&dir)` with the real execution-start cwd and reads real
    `dir/output.bin`; the native/interpreter exit + terminal-error comparison is
    preserved. **Provenance:** Architect ruling (`evt_azmharf1p667`, 08:30Z) +
    Steward custody (`evt_4smfxym8ef2h3` routing) + implementer `code_share`
    `evt_298an89139jt9`. Base blob `6378c31d` (hunks `@@ -189`, `@@ -241`).
  - **⚠ Both riders share base blob `6378c31d` (the `c8b8cdb7` fixture) and touch
    DISJOINT regions** (pwrite64 C-shim ~L161; interp leg ~L189-283) — they
    compose (implementer proved `ken-cargo test -p ken-cli --test
    px8f_buffer_native --no-run` green with both applied over PX8-J `dcfae3e1`).
    At unfreeze apply **both**, then run BOTH legs green as the Task #11
    fixture-revalidation pass — not a bare rebase. **✅ APPLY-VERIFIED
    (2026-07-19): both `git apply --check` clean against `c8b8cdb7`'s fixture,
    individually AND composed (pwrite64 → posix-interp-leg), exit 0** — no
    line-offset issue in practice. Re-derive only if the unfreeze rebase moves the
    surrounding lines.
- **Route:** **Architect §14** (surface soundness — the `writeAll` proof is
  real-checked, the progress partition matches §1.7.2, no in-language mutation) —
  **+ CV** iff the candidate touches `spec/`/`conformance/` (it will: the buffer
  seed cases turn GREEN and a new `.ken.md` catalog source lands). One branch,
  one Decision on the combined tip; retains Foundation QA evidence.

## Objective

Add the Ken **use-surface** for the bounded mutable buffer floor over the
**landed** PX8-R substrate + PX8-P `withBuffer` acquire/settle bracket:
positioned single transfers (`readAt`/`writeAt`) returning the exact §1.7.2
progress partition, the immutable `BufferWindow`/`BufferSpan` views +
`freeze`/`spanBytes`, and the **derived, kernel-checked `writeAll`** with its
five-part theorem (§1.7.3). This is **transcription of a settled surface**
(Architect `evt_2brnz8wg3ecth` fork-2/fork-3 + the contract-pinned
`spec/30-surface/38-ffi-io.md §1.7`), not new design. It wires the four host ops
that PX8-R landed as `RepresentedUnavailable` into real checked Ken calls, and
supplies the `writeAll` proof as **real terms the kernel checks**.

## Fixed inputs — DO NOT REOPEN (settled; do not re-ask the operator)

Read these as the authoritative contract before writing code:

- **`spec/30-surface/38-ffi-io.md §1.7` (contract-pinned by PX8-T, on
  `f2f60083`)** — the normative surface. Binding sub-parts:
  - **§1.7 (buffer floor):** `Buffer` is a second runtime resource kind; opaque
    copyable handle via the public `withBuffer` bracket (**landed**, PX8-P);
    **fixed strictly-positive non-growing capacity**; escaped copies invalidated
    on settle; allocation consumes **no ambient right**; admitted against the
    one deterministic `BufferLimitsV1 { per_buffer_max_capacity,
    invocation_max_live_capacity }` policy bound into the plan. Reading a limit
    from env, silently growing, or placing buffers in the capability table is
    **non-conforming**.
  - **§1.7 (`ResourceKindMismatch`):** closed inventory `FsHandle | Buffer`; a
    live token of the wrong kind → the **distinct** fail-visible
    `ResourceKindMismatch { expected, actual }` (own surface constructor +
    canonical wire discriminator, **not** `MalformedResource`). Buffer→Fs-only
    op reports `{expected: FsHandle, actual: Buffer}`; the reverse reports the
    reversed payload. **The two rejecting directions + two same-kind accepting
    controls are one non-degenerate conformance unit** (§7 discriminator).
  - **§1.7.1 (views + positioned transfers):** Ken observes only the opaque
    handle, an immutable `BufferWindow` request descriptor, a **constructor-
    private** immutable `BufferSpan` for the current live subrange, and scalar
    projections (incl. span length). **No pointer/slice/fd/mutable-ref/backing
    region crosses the boundary.** `spanBytes` (a.k.a. `freeze`) validates a
    current span and may copy that bounded span to immutable `Bytes` — it cannot
    expose the backing region. A `BufferSpan` carries a **constructor-private
    structural `Nat` attempt budget equal to its byte length**; user code can
    neither forge nor rechoose it. The primitive floor is **exactly one
    explicitly positioned transfer per direction**: `readAt file fileOffset
    buffer window` / `writeAt file fileOffset buffer span`. `fileOffset`
    nonnegative; offset+length overflow-checked before host I/O; **no file
    cursor**. Window start `0 ≤ start ≤ capacity` beyond the tail is **capped**
    (ordinary short progress); a negative/over-capacity start is an
    invalid-bounds error; a zero-length effective request is a **derived
    wrapper** that never invokes the positive-length primitive. A mutating read
    invalidates the prior window before the host op; EOF/error leaves the window
    empty.
  - **§1.7.2 (the exact progress partition) — spelling is normative:**
    ```text
    data ReadProgress  = ReadSome BufferSpan TransferCount | ReadEof
    data WriteProgress = Wrote TransferCount
    ```
    `TransferCount` is **constructor-private, strictly positive, bounded by the
    effective request, with an `Int` projection**; `ReadSome`'s span + count are
    minted together with `length span = count`. `Complete`/`Partial` are
    **derived comparisons**, not constructors. The count carries its
    positivity/upper-bound **witnesses** (the `writeAll` proof eliminates the
    checked result, never postulates about an arbitrary `Int`). Exact partition
    for a positive effective request: read-zero → `ReadEof`; read `0<n≤req` →
    `ReadSome span n` (incl. short); write `0<n≤req` → `Wrote n` (incl. short);
    **write-zero → error `NoProgress`, never `Wrote`.** Errors (never progress):
    `Closed`, `MalformedResource`, `ResourceKindMismatch`, `RightNotHeld`,
    invalid offset/window/bounds, buffer-limit/alloc failure,
    unsupported/nonblocking posture, host-I/O failure; `Interrupted` is a named
    error; `WouldBlock` is PX12. The four **primitive contracts** (write
    positivity/bounds, read positivity/bounds, position/overflow, tail-capping)
    are **fixed audited guarantees, not a fresh `Axiom` per caller** — the
    `writeAll` proof *consumes* write positivity.
  - **§1.7.3 (`writeAll` + its theorem) — the load-bearing proof.** `writeAll`
    is **transparent checked Ken code**, taking a positioned handle + `BufferSpan`,
    deriving **structural `Nat` fuel from the span's constructor-private attempt
    budget** (never caller-supplied), calling `writeAt` until the span is empty
    or the first transfer error. Required theorem (conjunction; `B`=initial
    bytes, `L`=length, `N`=Σ successful counts): **(1) termination** (≤`L`
    primitive calls, always returns); **(2) exact-prefix invariant** (after every
    successful prefix `0≤N≤L`, offset+span-start advanced by `N`, remaining
    `L-N`, bytes written exactly `B[0..N)`); **(3) success completeness** (success
    only when `N=L`); **(4) first-error preservation** (next error `e` incl.
    `NoProgress` → return that same first `e` after exactly prefix `B[0..N)`, no
    claim past it); **(5) all-success corollary** (every call succeeds → success,
    all of `B` written). **PX8-F must supply real kernel-checked terms consuming
    the checked positivity/bounds witnesses to prove strict decrease, fuel
    sufficiency, success⇒full-transfer, error-prefix preservation. NONE may be an
    `Axiom` or a postulated `writeAll` theorem.** Fuel exhaustion with bytes
    remaining is excluded by the positivity lemma — not a user-visible error.
    **The external Ward project owns the §71 §2.3 lifetime property; it is NOT the
    termination proof** (that boundary is the one-way Ken→Ward seam: Ken proves
    termination in-kernel and *exports* the lifetime obligation; the separate Ward
    project discharges it, out of Ken — see Sequencing).
- **Architect surface ruling `evt_2brnz8wg3ecth` fork-2 / fork-3** — the design
  behind §1.7: positioned (no cursor), operation-specific progress sums,
  `writeAll` proved by **structural `Nat` fuel from the bounded `BufferSpan`
  (CC3 pattern)**, sequential read/write/copy/seek are **derived Ken** threading
  `fileOffset + transferred` (no seek primitive, no vectored IO), and
  **`withResource` gains a capability-gated write/create mode** (truncate once at
  acquisition, **never** on a positioned write).
- **R2 closed (operator 2026-07-15):** no in-language affine/linear machinery; no
  in-language mutation of Ken values. The buffer is an **opaque runtime-backed
  handle**; all mutation is a runtime effect; liveness is runtime-enforced, never
  type-enforced. **A sub-WP proposing an in-language mutable reference, an
  all-or-nothing `write` primitive, or an affine buffer handle has misread the
  frame.**
- **Landed anchors on `f2f60083` (verify before editing; do not fabricate;
  source `git show origin/main:<path>` — the surface is NOT in a stale worktree):**
  - `crates/ken-elaborator/src/prelude.rs`: `withBuffer` public entry
    **1828-1837** (signature: `withBuffer (a:Auth)(e r:Type)(capacity:Int)(body:
    Resource Buffer -> HostIO a (ResourceBodyResult e r)) : HostIO a (Result
    ResourceError (ResourceBracketResult e r)) visits [FS]`); `FSOp` datatype
    decl **~1385-1440**; private-op id zip/fetch **1464/1474**; `fs_resp`
    large-elimination decl **1476**, `PrivateBufferAllocate` arm **1494**;
    `ResourceError` ctors (incl. `ResourceKindMismatch`, `BufferLimit`,
    `InvalidOffset`, `InvalidBounds`, `NoProgress`) **~1361**; finalizer
    `resource_settle_result_for` **1708-1717** / `release_if_live` **1755-1768**
    / `private_with_buffer_after_allocate` **1805-1827**; **private-name lockdown
    list 1842-1859** (add every new private op here). Extend the checked `FSOp`
    family + `fs_resp` with the Buffer-*use* ops here.
  - `crates/ken-interp/src/eval.rs`: `BufferAllocate` dispatch **4552**;
    positioned trait methods `fs_read_at`/`fs_write_at` **2287-2288** (defs
    2683/2695, 3415/3426) driven at **4150/4166**; `ResourceKindMismatch` Ken
    carrier **4744**; `NoProgress` carrier **4764**. Add the Buffer-use dispatch
    arms alongside `BufferAllocate`.
  - `crates/ken-host/src/effect_v1.rs`: **host ops already landed** —
    `FsReadAt=0x030D` (L34), `FsWriteAt=0x030E` (L35), `BufferAllocate=0x0402`
    (L37), `BufferFreeze=0x0403` (L38); real `pread`/`pwrite` dispatch
    **1606-1648** / **1664-1693** (write-zero→`NoProgress` at **1693**);
    `BufferRegionV1` **616**; **the four ops sit `RepresentedUnavailable` in the
    ABI manifest 2599-2603** — PX8-F flips them to reachable by wiring the Ken
    use-surface. **Consume `ken_host` types; never redefine.**
  - **Catalog is greenfield:** no `catalog/packages/System/` dir; `System.Resource`
    lives at `catalog/packages/Capability/System/Resource.ken.md` (38 lines,
    `withResource` only). PX8-F's `System.Buffer`/`System.IO` catalog source is
    **new** under `catalog/packages/Capability/System/`.

## Mandated deliverables (each ends in a concrete implementable choice)

- **F-D1 — positioned single-transfer primitives.** Add the two checked `FSOp`
  Buffer-use ops `readAt`/`writeAt` with the exact §1.7.1 signatures
  (`readAt file fileOffset buffer window` / `writeAt file fileOffset buffer
  span`), their `fs_resp` arms returning `Result <err> ReadProgress` /
  `Result <err> WriteProgress`, private id registration mirroring
  `PrivateBufferAllocate` (add raws to the **lockdown list 1842-1859**), lowered
  to the landed `HostOpV1::FsReadAt=0x030D` / `FsWriteAt=0x030E`; interpreter
  dispatch arms at `eval.rs` alongside 4552 → the landed `fs_read_at`/`fs_write_at`
  progress path. Overflow-checked nonnegative `fileOffset`; **no cursor**.
- **F-D2 — the progress partition types + views.** Define the §1.7.2 closed sums
  `ReadProgress = ReadSome BufferSpan TransferCount | ReadEof` and `WriteProgress
  = Wrote TransferCount` with **constructor-private** `TransferCount` (strictly
  positive, request-bounded, `Int` projection, positivity/upper-bound witnesses
  attached); the immutable `BufferWindow` request descriptor; the
  **constructor-private** immutable `BufferSpan` (carrying the structural `Nat`
  attempt budget = byte length) with scalar projections (span length); and
  `spanBytes`/`freeze` (validate current span → copy bounded span to immutable
  `Bytes`; **never** expose the backing region) lowering through
  `BufferFreeze=0x0403`. Enforce the tail-cap / invalid-bounds / zero-length-
  derived-wrapper rules from §1.7.1.
- **F-D3 — `ResourceKindMismatch` surface unit.** Surface the distinct
  `ResourceKindMismatch { expected, actual }` (own constructor + wire
  discriminator, not `MalformedResource`) on wrong-kind tokens through the
  Buffer-use ops, with the reversed-payload directions and same-kind accepting
  controls as **one non-degenerate conformance unit** (KM-A). (The runtime
  identity landed in PX8-R; F-D3 is the **checked-surface reachability** of both
  directions.)
- **F-D4 — derived `writeAll` + its kernel-checked five-part theorem.**
  Transparent checked Ken `writeAll` (positioned handle + `BufferSpan`,
  structural `Nat` fuel from the span's private attempt budget, never
  caller-supplied) + **real kernel-checked** proof terms for §1.7.3 (1)-(5)
  consuming the write-positivity/bounds witnesses. **No `Axiom`, no postulated
  `writeAll` theorem** — Foundation authors the transparent defs + Omega proofs
  as real terms the kernel elaborates. Empty input succeeds with no host call;
  the `Suc fuel` branch does one positive `writeAt`, `Wrote n` advances by `n`
  and strictly decreases remaining, error returns unchanged. **★ On the native
  lane the span's structural-`Nat` budget is constructed by PX8-N's bounded-`Nat`
  reply-lowering carrier** (minted at the validated response boundary) — this is
  why PX8-F is held until PX8-N lands. The interpreter lane already mints the `Nat`
  by Rust recursion; after PX8-N, the native `writeAll` recursion reaches native
  execution and agrees with the interpreter. **★ Terminal obligation (Architect
  `evt_4kh6gz18tvzs6`): PX8-N's evidence is `FsReadAt`-only — Foundation must
  AUTHOR the first real linked checked `writeAll` native fixture here (invoke the
  actual `writeAll`, force ≥1 positive short write so recursion/decrement is
  observed, assert exact completion/prefix behavior + interpreter↔native
  agreement). This is the first layer where the `writeAll` path is reachable — a
  new fixture, NOT a "rerun" of one that never existed.**
- **F-D5 — `withResource` write/create mode + catalog surface + honesty.**
  Extend `withResource` with the capability-gated write/create mode (truncate
  once at acquisition, never on a positioned write). Author the greenfield
  `System.Buffer`/`System.IO` catalog source(s) under
  `catalog/packages/Capability/System/` (mirroring `Resource.ken.md`), with
  **source honesty statements** (`tested`-not-`proved`) for every runtime-enforced
  invariant Ken cannot state (fixed capacity, one live window, exactly-once
  settlement) — routed to the Ward seam exactly as PX7's obligation was, **not**
  restated as a Ken `proved`. **Any catalog `.ken.md` MUST pass the frozen-corpus
  `ken fmt` gate** (`-p ken-cli --test ken_fmt` / `ken fmt --check <path>`) in QA
  — that gate is full-CI-only and invisible to targeted `-p` semantic suites
  (the PX7-F lesson).

## Required proofs / discriminators (each independently reaching; §7)

1. **Non-degenerate partition pair per direction** (§1.7.3 obs. 1-2): a **full**
   write returns success with the whole span written **and** at least one
   **positive short** write still reaches success with exact prefix accounting —
   on the same shape (grep the reached branch, not a suite-green).
2. **Write-zero `NoProgress` load-bearing** (obs. 3): a write-syscall-zero
   **reaches** and returns `NoProgress`, never `Wrote 0` (assert the specific
   `NoProgress` error variant, not `is_err`).
3. **Mid-stream error prefix preservation** (obs. 4): a transfer error returns
   that **exact** error after the **exact** written prefix `B[0..N)`.
4. **Read partition:** read-zero → `ReadEof`; read `0<n≤req` →
   `ReadSome span n` with `length span = n` (short read is success).
5. **`ResourceKindMismatch` unit:** both reversed rejecting directions + both
   same-kind accepting controls (KM-A), reaching through the checked surface.
6. **`writeAll` proof is real:** the five parts are **kernel-checked terms**, not
   `Axiom`/postulate (grep the elaborated proof, not the theorem name); a
   mutation that drops fuel-sufficiency or strict-decrease fails to elaborate.
7. **No backing region escapes:** no path exposes a pointer/slice/fd/mutable-ref;
   `spanBytes` copies a bounded span only.

## Acceptance criteria (testable)

- **AC1** — F-D1..F-D5 landed: positioned `readAt`/`writeAt` on the checked
  surface lowering to `FsReadAt=0x030D`/`FsWriteAt=0x030E` (grep the emission);
  the §1.7.2 progress sums with constructor-private `TransferCount`/`BufferSpan`;
  `freeze`/`spanBytes` via `BufferFreeze=0x0403`.
- **AC2** — the derived `writeAll` five-part theorem is **kernel-checked real
  terms** (discriminator 6); **no `Axiom`, no postulated theorem** anywhere in
  the proof.
- **AC3** — the four §1.7.3 conformance observations each **reach their named
  branch** (discriminators 1-3 + read partition 4); a short write reaches success
  with exact prefix; write-zero reaches `NoProgress`; mid-stream error preserves
  prefix. Seeds `WA-A/B/C/D`, `PR-A/B/C` turn GREEN.
- **AC4** — `ResourceKindMismatch` is the distinct surface identity on both
  reversed directions with same-kind accepting controls (discriminator 5); seed
  `KM-A` GREEN.
- **AC5** — no PX7/PX8-P regression: the `FsHandle` bracket + `withResource` +
  the no-acquire export fixture `ken-export-v0:6360c2cb74f78f7e` stay
  byte-identical; a read/write on a `Closed` handle returns `Closed`, insufficient
  authority returns `RightNotHeld`.
- **AC6** — the new `System.Buffer`/`System.IO` `.ken.md` catalog source passes
  the **frozen-corpus `ken fmt`** gate (full-CI); no in-language mutable surface.
- **AC7** — **no-regression = GREEN IN CI** (never a local `--workspace` run;
  COORDINATION §12). Build/test **targeted only** (`scripts/ken-cargo -p
  ken-elaborator …` / `-p ken-interp …` / `-p ken-cli --test ken_fmt` /
  `--test <name>`), **plus run the ken-cli integration suite the new surface
  constructors implicate before release** (the PX8-R CI-red lesson: a new surface
  ctor can break checked-program exhaustiveness consumers only full CI catches).

## Do-not-reopen guard

- Do **not** add an in-language mutable reference, an all-or-nothing `write`
  primitive, an affine handle, a file cursor, a seek primitive, or vectored IO —
  all out of scope (§1.7.1 / R2 / `evt_2brnz8wg3ecth`).
- Do **not** redefine the landed PX8-R runtime vocabulary or the four host ops —
  **consume** `ken_host`; wire the checked surface to them.
- Do **not** expose a pointer/slice/fd/mutable-ref/backing region; `spanBytes`
  copies a bounded span only.
- Do **not** change the `FsHandle` bracket / `withResource` acquire behavior
  observably. The **no-acquire negative control** stays exactly
  `ken-export-v0:6360c2cb74f78f7e` (it has no resource obligation, so the schema
  collapse doesn't touch it). **Resource-producing export hashes DO rebaseline
  once** onto the sole unversioned `ResourceLifetimeObligation` (the old V1-byte
  preservation is superseded — PRINCIPLES transient T; PX8-X train) — that is
  expected, not a regression.
- Do **not** postulate the `writeAll` theorem or introduce an `Axiom` — real
  kernel-checked terms only.
- Do **not** author any Ward runtime-observation consumer/monitor here **or
  anywhere in Ken** — Ward is a **separate project** (operator ruling 2026-07-17;
  the old in-Ken PX8-W is struck). PX8-F emits the surface + the honest
  `status: delegated`-bearing obligations via the landed export; it does **not**
  discharge them, and neither does any other Ken WP.
- If the surface surfaces a genuinely new blocker (e.g. `writeAll`'s fuel cannot
  be derived from the landed `BufferSpan` budget without a runtime change),
  **HARD-STOP to the Steward** — do not paper over it.

## Verify short-write/`NoProgress` companion (folded on this branch)

Architect fork-3 (confirmed `evt_3v3mtq4q085e2`) assigns the **short-write
differential + the zero-write mutation proving the `NoProgress` branch
load-bearing** to **Team Verify**, folded as a **Verify companion on the PX8-F
branch** (mirroring PX8-V's conformance companion) so **Architect §14 + CV** gate
the combined tip. This is the write-partition differential (`WA-B` short-write to
success with exact prefix; `WA-C`/`PR-A` write-zero reaching `NoProgress`) proving
the runtime `NoProgress` branch is load-bearing — **distinct** from the Ward
lifetime monitor (which is the external Ward project's, out of Ken). This
companion is a Ken conformance differential over Ken's own short-write/`NoProgress`
runtime behavior — not Ward discharge. Steward coordinates the fold at kick;
Verify authors it against the landed PX8-F surface tip.

## Sequencing (Architect `evt_291b8gcwde32v` + `evt_22y0emyj6cwf2` +
`evt_5v6jrc6rnva`, 2026-07-17)

**Implementation order:** PX8-T✅ (`d69819ca`) → PX8-R✅ (`1640fd48`) →
PX8-P✅+PX8-V✅ (`f2f60083`) → **{ PX8-X train ∥ PX8-N } →
PX8-F rebased terminal gate (this) → Phase-C exit (Ken side).**

- **PX8-F is HELD until BOTH predecessors land** (frozen at `60a481b5`,
  `preserved/px8f-held-60a481b5`):
  - **PX8-N** (Team Runtime, `docs/program/wp/PX8-N-*.md`) — the bounded-`Nat`
    reply-lowering carrier that makes the native structural-`Nat` budget
    constructible (fixes PX8-F's hard-stop).
  - **PX8-X train** (Runtime+Verify+Spec,
    `docs/program/wp/PX8-X-single-schema-unification-train.md`) — the single-schema
    collapse that changes the obligation PX8-F emits (sole unversioned
    `ResourceLifetimeObligation`).
- **PX8-X and PX8-N run in parallel**, each with its own gate/Decision (PX8-N:
  Runtime QA + Architect §14; PX8-X: Runtime QA + Verify QA + Architect §14 + CV,
  one combined SHA). PX8-F must **not** merge on today's obligation and then be
  converted — it rebases **once** onto the combined main after both land.
- **Ward discharge is OUT of Ken.** The old PX8-W (in-Ken `ken-verify` monitor)
  is **struck** (operator ruling 2026-07-17: Ward is a separate project). Ken's
  seam-side is complete once it **exports** the assumption boundary — the static
  obligation (PX8-V, landed) + the runtime observation (PX8-X). The §71 §2.3
  monitor that reads those and renders a verdict lives in the **external Ward
  project**, out of this repo; Ken never builds it, and the RB-A…RB-O behavioral
  cases go with it.
- **Phase-C exit** (`cat`/`cp`/`wc` native over a larger-than-memory file,
  interpreter↔native external-delta equality via the PX6 harness) needs **the
  PX8-X train + PX8-N + PX8-F (rebased) all landed** — the Ken buffer-IO floor. It
  does **not** wait on any in-Ken Ward monitor (there is none).
