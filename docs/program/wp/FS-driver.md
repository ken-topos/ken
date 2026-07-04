# FS-driver — real file I/O via a capability-carrying `[FS]` effect (VAL2 #9 / OQ-B)

**Steward frame → spec enclave (elaborate) → Runtime + Sec + conformance
(build).** Design-settled capability WP closing VAL2 finding #9
(`read-file-lines`: `[FS]` exists at the type level but `read_bytes` has **no**
`ken-interp` reduction — no Ken program can do file I/O). Product decision
**locked by the operator** (OQ-B, `VAL2-gap-design-OQs.md`, 2026-07-03). **This
is a multi-lane mini-workstream, not one small fix** — the enclave/Architect may
**decompose it into a short WP series** (driver / capability / conformance);
propose the decomposition back to Steward. Owner: spec-leader elaborates
semantics; **Runtime** (driver) + **Sec** (capability) + **conformance** build.
Gate: **/spec + /conformance touching** → spec enclave elaboration, **Architect**
(soundness: totality + kernel-untouched + capability model), **Spec review**
(conformance-validator), team QA, CI. Findings → **Steward**.

## The locked decision — DO NOT REOPEN

- **Real FS driver — NO mock.** File I/O runs through a **real driver** extending
  the existing `ITree`/Console effect-interpreter (the same free-monad Console
  runs on). The operator ruled a synthetic mock FS **wasted work** (we will
  ultimately need real FS interaction). Do **not** build a mock/virtual FS.
- **Conformance determinism via checked-in hermetic fixture files.** I/O is
  nondeterministic; determinism comes from **version-controlled fixture files**
  the test reads through the **real** driver (real code path, deterministic
  input) — **not** a throwaway mock and **not** by stubbing the read.
- **Capability-carrying effect.** `[FS]` carries the capability/paths a program is
  authorized to touch (aligns the **Sec / effect-capability** workstream). File
  access is **not** ambient authority — an undeclared access is rejected.
- **Totality preserved.** The effect is a **pure description** (an `ITree` node);
  nondeterminism lives in the **handler**; the program handles failure as
  `Option`/`Result` (file-not-found, etc.) in-language. The pure core stays
  total. **Kernel / `trusted_base` untouched** (outer-ring effect + runtime
  driver, like Console).

## ★ LANDED SUBSTRATE (fixed inputs — elaborate against THIS, not a survey)

Grounded against `origin/main` (2026-07-03; this `steward/work` worktree checks
out only stub crate `lib.rs`s — **cite via `git show origin/main:<path>`**). All
line numbers are **perishable — verify at pickup, do not trust this frame over
the landed code.** The headline: **the type-level `[FS]` side is already built
and tested; the gap is purely the runtime driver + capability thread + fixtures.**

- **The Console effect is the DIRECT precedent to MIRROR — not invent anew.**
  - Surface/prelude: `data ConsoleOp = Write String`, `console_resp : ConsoleOp →
    Type = Unit`, `IO A := ITree ConsoleOp console_resp A`, `print_line : String →
    IO Unit` (`crates/ken-elaborator/src/prelude.rs` ~`:194,693-724`). The `ITree`
    decl is already **generalized** off the old Console-hardwired 1-param form.
  - **The REAL runtime driver to mirror is `run_io`/`drive_h`** in
    `crates/ken-interp/src/lib.rs` (re-exports `:8-10`; `Vis (Write s) k →
    println!(s); run_io(k unit)`; test `val1_c2_write_prints_and_resumes`
    ~`:2270`). **The FS driver is the file-I/O analog of `run_io`.** (NB: there is
    **no `ken-runtime` crate** — the "Runtime lane" is `ken-interp`.)
- **`read_bytes` is ALREADY registered at the type level with `[FS]` — and
  tested.** `crates/ken-elaborator/src/bytes.rs` registers `read_bytes`/`write_bytes`/
  `append`/`send`/`recv` as prims (`~:125-130`) with `io_effect_rows` mapping
  `read_bytes|write_bytes|append → EffectRow::singleton("FS")`, `send|recv →
  "Net"` (`~:161-171`). The escape/capability check REJECTS an untracked call —
  `tests/l6_acceptance.rs::read_bytes_untracked_is_type_error`. Note `bytes.rs`
  ~`:120-123`: the arg type is a **placeholder `Bytes → Bytes`** ("real Path/Socket
  types are an L7/FFI concern; the EFFECT ROW is what matters").
- **THE GAP (VAL2 #9, crisp):** `read_bytes` has **no `ken-interp` reduction** —
  at runtime it is an **inert `EvalVal::CtorPending`** (`crates/ken-interp/src/
  eval.rs` ~`:1318-1322`); the saturated-prim path (~`:1461-1467`) wires only
  `print_line`/`string_to_list_char`/… and otherwise falls to a `Neutral`
  catch-all. So `[FS]` type-checks and tracks but **drives nothing when run.**
  This WP adds: the reduction dispatching to a real host-I/O handler, the
  capability thread to it, and the fixture conformance — **onto a working
  type-level substrate.**
- **Effect machinery to REUSE (do not fork a second effect system):**
  `crates/ken-elaborator/src/effects/` — `itree.rs` (`ITree` Ret/Vis, `perform`,
  `bind`, `handler_fold`, dependent response type landed via the `[State]` WP),
  `row.rs` (`EffectRow(BTreeSet<EffectName>)`, `is_subset_of` for `ρ_inf ⊆
  ρ_decl`; `EffectName`'s doc lists `FS` first), `check.rs::check_capabilities`
  (~`:127-155`, `EffectError::MissingCapability`), `algebra.rs::CapParam`
  (`using name : Cap E`).
- **Capability tokens (Sec lane) are ALREADY landed** —
  `crates/ken-elaborator/src/capabilities.rs`: unforgeable `CapToken`, `mint`
  (root), **`attenuate`** (monotone-downward, "one directory not the filesystem"),
  **revocation handle**, combined cap+flow gate. Spec: `spec/60-security/
  62-authority.md` (no ambient authority; `Cap_FS`/`Cap_Net` tokens; attenuation;
  revocation). The AC3 discriminating pair (with-cap succeeds / without-cap
  rejected) threads THIS machinery to the FS op at **runtime** (currently the
  token exists but no `Cap_FS` is minted/threaded through an FS op).
- **Cross-effect dependency to account for:** `[FS]` shares the `Vis (inr o)`
  pass-through dispatch path with `[State]`, and **EFF6 console-commute is tagged
  DEFERRED** (`#245 5d723e3`, pending a Console-lift). The elaboration should note
  whether the FS driver needs that lift or is independent (State-effect-build.md
  ~`:86-87,171-173`).

## Means are the enclave's + Architect's call

Fixes the **goal + properties + acceptance**, not the mechanism. The exact op set
(`read_bytes`/`read`/`write`/`open`/…), the effect-row + capability
representation, how a program *declares* its FS capability, and the driver
implementation are the **enclave's elaboration + Architect's** call. Illustrative
signatures are tagged *verify/decide against the landed effect + capability
system, not this line.*

## Mandated deliverable outline (each item resolves to a concrete choice)

1. **Effect ops + real reduction.** Give `read_bytes` (and the enclave-chosen op
   set) a **real `ken-interp` reduction** dispatching to the driver. Reuse the
   `ITree`/Console effect node machinery — **no second effect system** (coordinate
   with the `[State]` WP so both share one effect-dispatch path).
2. **Runtime driver.** The handler that performs the actual file I/O in the
   runtime (real syscalls, outside the pure core). Runtime lane. Specify where it
   lives and how it surfaces failure as `Option`/`Result` to the program.
3. **Capability model.** How `[FS]` carries the authorized paths/capabilities and
   how an **undeclared access is rejected** (Sec lane) — the discriminating case:
   a program without the capability must **fail**, one with it must succeed.
4. **Conformance — real driver + checked-in fixtures.** Tests drive the **real**
   driver against **version-controlled hermetic fixture files** (deterministic,
   real code path). The `read-file-lines` shape (VAL2 #9) is the canonical
   example. **No mock.** Grounded against the landed producer
   ([[conformance-hand-feeds-the-deliverable]]).
5. **Totality/capability statement.** Explicit spec + a check that the effect is
   erasable to its description, the pure core stays total, and the capability is
   enforced (not decorative).

## Phase 1 — enclave elaboration (deliverables 1/2/3/5)

*Front-loaded semantics for the Phase-2 build. The design **mirrors the landed
Console effect** (`print_line` → `build_print_line_tree` → `Vis (Write s) k` →
`run_io`) at every step — the FS driver is the file-I/O analog. Grounded against
`origin/main@7b5eb3c` (line numbers perishable — verify shapes at pickup).
Deliverable 4 (conformance fixtures + AC3 fixture pair) is
`conformance-validator`'s companion, gated together. Illustrative Ken signatures
are tagged **[verify against the landed effect + capability system]** per the
frame's "means are the enclave's call".*

### Scope of the op set (this WP)

`read-file-lines` (VAL2 #9) needs exactly **one op driven end-to-end: read**.
This WP wires **`read_bytes`** (read) through reduction + driver + capability +
fixtures. `write_bytes`/`append` (`[FS]`) and `send`/`recv` (`[Net]`) stay
**registered at the type level but undriven** (a named follow-on, not silently
in scope, not silently dropped) — the driver's op dispatch is written so adding
their arms later is additive, no re-architecture.

### D1 — Effect ops + real reduction (the *pure description*)

The Console prelude is the exact template (`prelude.rs` ~`:194-213`,
`ConsoleOp`/`ITree`/`IO`). Mirror it for FS, **reusing the generalized
`ITree ρ Resp R`** (already lifted off the Console-hardwired form — *do not fork
a second effect system*):

```
data FSOp = ReadFile Path                       -- Path = the bytes.rs placeholder
fs_resp : FSOp -> Type                           -- the driver's response type…
fs_resp (ReadFile _) = Result Bytes IOError      -- …carries failure (see D2)
FS A := ITree FSOp fs_resp A                      -- reuse the generalized ITree
read_bytes : Cap FS -> Path -> FS (Result Bytes IOError)   -- [verify …]
```

**Re-type `read_bytes` (deliverable-1 decision — flag the interaction).** Today
`read_bytes : Bytes -> Bytes` with the `[FS]` effect tracked in a **side table**
(`bytes.rs::io_effect_rows`, `read_bytes -> EffectRow::singleton("FS")`). A
`Bytes -> Bytes` prim **cannot reduce to an `ITree Vis` node** without a type
mismatch, so a driver-interpretable op **must** carry the effect **in its return
type** — the `FS (…)` monad — exactly as `print_line : String -> IO Unit` does.
So Phase 1 re-types `read_bytes` to the `FS`-monad form above. **Interaction to
preserve:** the landed static escape/row check
(`tests/l6_acceptance.rs::read_bytes_untracked_is_type_error`,
`io_effect_rows` + `check_capabilities`) is the **static capability face** (D3)
and must **stay green** — the return-type effect is *added* for driver
interpretability, it does not replace the row tracking. If the re-type would
regress that test, reconciling the two faces (row table vs. `FS`-in-return-type)
is a pinned Phase-2 step, **not** a silent drop of the static check.

**Real reduction (the gap).** `read_bytes` is intercepted in the saturated-prim
path (`eval.rs` ~`:1461`, *exactly* where `print_line` /
`string_to_list_char` are — they are intercepted before the generic
`prim_reduce` because they need `store` + ctor `GlobalId`s) → a new
`build_read_file_tree(cap, path, fs_ids, store)` that mirrors
`build_print_line_tree` (`eval.rs:1528`) and returns

```
Vis (ReadFile cap path) (λ r. Ret r)   -- a pure ITree value; NO syscall here
```

This reduction is **pure and total** — it constructs an `ITree` description, it
performs no I/O (AC5). An `FSIds` struct (like `ConsoleIds`/`ITreeIds`) carries
the `ret_id`/`vis_id`/`readfile_id`/`params_len` plus the `Result`/`IOError`
ctor ids the driver needs to *build the response*; obtain them from the
`GlobalEnv` after prelude registration.

### D2 — Runtime driver + failure-surfacing

The top-level driver is `run_io` (`eval.rs:1737`), invoked from the CLI
(`ken-cli/src/main.rs:110`, `run_io(tree, &console_ids, …)`). It loops the
`ITree`: `Ret r` → return `args[m]`; `Vis op k` → dispatch on `op` **exhaustive,
no catch-all** (`42 §6.5`), compute `resp`, `apply(k, resp)`, loop. Extend the
dispatch with an **FS arm** alongside the existing Console `Write` arm:

```
Vis (ReadFile cap path) k ->
    if !authorizes(cap, path) { resp = Err(CapabilityDenied) }      -- D3, runtime face
    else match std::fs::read(path) {                                -- the ONLY new syscall
        Ok(bytes) => resp = Ok(Bytes(bytes)),
        Err(e)    => resp = Err(io_error_kind_to_IOError(e)),       -- NotFound / Permission / Other
    }
    apply(k, resp) ; loop                                            -- resp is a total Result value
```

- **Where it lives:** `ken-interp` (mirroring `run_io`) — **there is no
  `ken-runtime` crate** (AC1). The `std::fs` syscall is the sole new host-effect
  surface, **confined to the driver** (outer ring), never in `eval`/the pure
  core.
- **Failure-surfacing (deliverable 2):** file-not-found / permission /
  capability denial become a **total in-language `Result` value** handed back
  through `k` — **never a panic** — so the program matches the failure and
  the pure core stays total (AC5). `IOError` is a small in-language sum
  (`data IOError = NotFound | PermissionDenied | CapabilityDenied | Other`); the
  driver maps `std::io::ErrorKind → IOError`.
- **Multi-effect programs** (FS + Console): route each op through the landed
  `[State]` `Vis (inr o)` pass-through dispatch — **one driver, two arms**,
  not a second dispatcher. See the Console-lift note in D3.

### D3 — Capability model (static face + runtime face)

The capability has **two faces** — deliver both, and state which each AC3 arm
pins (the static-vs-runtime split is the recurring effect-AC shape):

- **Static face (LANDED — keep it).** `check.rs::check_capabilities`
  (~`:127-155`): a decl performing `[FS]` must have `using cap : Cap FS` (a
  `CapParam`, `algebra.rs`, `§2.5`) or an enclosing handler, else
  `EffectError::MissingCapability` at **elaboration**. This is the "is `[FS]`
  authority *declared*" gate — already tested. Nothing to build; preserve it
  through the D1 re-type.
- **Runtime face (NEW — the load-bearing thread this WP adds).** The `Cap_FS`
  token is **carried in the effect** (capability-*carrying*, per the operator
  lock): the op node encodes the capability, `ReadFile cap path`, so
  `read_bytes` takes the `Cap FS` (supplied by the `using cap : Cap FS` param).
  The driver's FS arm, **before any syscall**, checks `authorizes(cap, path)`
  and refuses otherwise — this makes access **non-ambient** at runtime. Reuse
  `capabilities.rs` (`Cap`, `attenuate`, `authority`); the token is
  **unforgeable** — `mint` is handler/runtime-only, not surface-callable
  (`capabilities.rs:66`, `spec 62 §2.2`) — so a program can only **narrow**
  (`attenuate`, "one directory not the filesystem"), never widen.

**Path-scope representation — flag for Architect / Phase 2.** `spec 62 §2.1`
says `Cap_FS`'s authority *is* "a set of paths"; the landed
`capabilities.rs::Authority(u8)` is a **coarse stand-in** with no path field.
The runtime `authorizes(cap, path)` check needs a concrete path-scope (an
authorized directory prefix, narrowed by `attenuate`). Realizing it — extend
`Authority`/`Cap` with a path-scope vs. an FS-specific scope carried alongside —
is the **one open representation choice**, Architect's + Phase-2's call. The
**contract is fixed**: `authorizes(cap, path)` gates the syscall, `attenuate`
only narrows, unauthorized ⇒ `CapabilityDenied`. (This is not a fork PRINCIPLES
can't settle — it reuses the landed attenuate/mint skeleton; if Architect wants
it decided up front, it routes to Steward per the defer rule, but I read it as
delegated.)

**AC3 discriminating pair (both faces; CV pins the concrete fixtures in D4,
`FS-driver-conformance.md` §2a/§2b — S1/S2 static, R1/R2/R2′ runtime):**

- *static arm (S1/S2)* — `read_bytes … path` in a decl with **no**
  `using cap : Cap FS` ⇒ `MissingCapability` (elaboration rejects); with the
  param ⇒ elaborates.
- *runtime arm (R1/R2)* — **same op**, `using cap : Cap FS` present (so it
  clears the static gate and actually *reaches* the driver): a **sufficient**
  cap ⇒ the driver reads the fixture; an **insufficient** cap ⇒
  `CapabilityDenied` at the driver, **no read**. Outcome flips on the capability
  ⇒ the check is
  **load-bearing, not decorative** (AC3); a no-op `authorizes` (always-true) =
  ambient authority = fails AC3, so the negative must **reach** the refusal —
  not be a static reject in disguise (R2 declares `using cap`).
- *Phase-1 vs Phase-2 form (honest, matches CV §2b).* The R1/R2 discriminator
  Phase 1 can express is **authority-level** (attenuate to an insufficient
  `Authority` ⇒ `CapabilityDenied`, via the scalar `Authority(u8)`). The
  **path-exclusion** form (a `dir1`-cap refusing `dir2`) is **R2′**, deferred
  with the path-scope realization above — CV pins it as a Phase-2
  known-gap-with-reason. The *contract* (`authorizes` gates, `attenuate`
  narrows, unauthorized ⇒ `CapabilityDenied`) is fixed now; only the path-scope
  *spelling* `(oracle)`-defers.

**Console-lift / EFF6 dependency (the frame asks).** The FS-only path (AC2,
`read-file-lines`) is **independent** of the deferred EFF6 console-commute
(`#245`): `read-file-lines` *sequences* FS then processes lines; it asserts no
console-commute *equation*. A program mixing FS **and** Console rides the landed
`[State]` `Vis (inr o)` pass-through (its `direct-state-console-commute` AC3
path); only a program that needs the specific commute *law* would need EFF6.
**So this WP is not blocked on the Console-lift.**

### D5 — Totality / capability statement (AC5, checkable)

- **Pure description.** `read_bytes cap path` reduces *in the pure core*
  (`eval`) to a total `Vis (ReadFile cap path) (λ r. Ret r)` `ITree` value — no
  syscall, no partiality. **Checkable by grep:** the `eval.rs` FS interception
  builds an `ITree` and calls **no** `std::fs`.
- **Erasability.** The effect is erasable to its `ITree` denotation (`36 §2.4`),
  a pure core term `eval` already handles. All nondeterminism/partiality
  (file-not-found, I/O error, capability denial) is **confined to the driver**
  (outer ring) and surfaced as a **total `Result`** the program matches — the
  pure core stays **total** (AC5).
- **Kernel untouched (AC1).** The whole delta is outer-ring: prelude decls
  (`ken-elaborator`), prim reduction + driver (`ken-interp`), capability thread
  (`ken-elaborator`/`ken-interp`), fixtures (`conformance/`). **Zero
  `ken-kernel/`, zero `trusted_base` delta, no new `Term`/`Decl` variant** —
  grep-verified, not a test.
- **Capability enforced, not decorative.** The driver's `authorizes(cap, path)`
  is load-bearing — AC3's runtime arm flips on it. Its absence (always-admit) is
  ambient authority and fails AC3 by construction.
- **Trust level — honest, not "kernel-backed" (matches CV §2c).** The FS
  driver's **runtime** capability gate (`authorizes(cap, path)`, and the
  `authority_flows_to`/`is_satisfied()` it rests on — a plain Rust `bool`) is
  **trusted Rust-level** logic in the outer ring, **conformance-netted, not
  kernel-backed** — so AC3 is a *tested-not-trusted* posture
  ([[kernel-backed-claim-grep-the-emission-not-the-name]],
  [[tested-not-trusted-posture-needs-reachability-precondition]]). This is
  **distinct from** `attenuate`'s *static* refinement obligation, which **is**
  kernel-re-checked (`discharge_attenuation` → `declare_postulate`, `62 §3`) —
  that governs monotone-downward attenuation soundness at elaboration, **not**
  the runtime path gate; do not conflate them or let the runtime gate borrow the
  static obligation's kernel-backing. Honest for an outer-ring effect
  (Sec1-level trust); the doc must not label the FS runtime capability
  "kernel-backed."

### What Phase 2 builds (maps to the proposed bundled build branch)

| Lane | Deliverable | Files (perishable) |
|---|---|---|
| Runtime | D1 reduction + D2 driver arm + `IOError`/`FSOp` prelude | `ken-interp/{eval,lib}.rs`, `ken-elaborator/src/prelude.rs`, `bytes.rs` re-type |
| Sec (Verify) | D3 runtime `authorizes` thread + path-scope on `Cap_FS` | `ken-elaborator/src/capabilities.rs`, effect capability thread |
| Conformance | D4 hermetic fixtures + AC3 pair (CV) | `conformance/fs/…` (CV's companion doc) |

Bundled into **one** Phase-2 branch (spec-leader's decomposition): the runtime
driver must never land on `main` **without** its capability gate live, else an
ambient file-read sits on `main` transiently between merges.

## Acceptance criteria

- **AC1 — Kernel untouched (load-bearing).** `git diff origin/main --
  crates/ken-kernel/` empty; `trusted_base()` unchanged; no new kernel variant.
  `[FS]` is an outer-ring effect + runtime driver + capability surface — the touch
  is `ken-interp` (driver reduction) + `ken-elaborator` (capability thread, if any)
  + `conformance/` (fixtures); **verify by grep, not a test.** (There is no
  `ken-runtime` crate — the driver lives in `ken-interp`, mirroring `run_io`.)
- **AC2 — `[FS]` drives real I/O end-to-end.** `read_bytes` has a real reduction;
  `read-file-lines` reads a **checked-in fixture** through the real driver and
  produces the correct content; failure surfaces as `Option`/`Result`.
- **AC3 — Capability enforced.** A discriminating pair: an access **with** the
  declared capability succeeds; the same **without** it is **rejected** (the
  capability is load-bearing, not decorative). [[taint-axis-orientation-needs-distinguishing-pair]]-style: both arms on the same op.
- **AC4 — Determinism, no mock.** Conformance uses checked-in fixture files +
  the real code path; grep confirms **no mock/virtual FS** was introduced.
- **AC5 — Totality preserved.** The pure core stays total; the effect is a pure
  description; nondeterminism is confined to the handler.
- **AC6 — No regression.** `cargo test --workspace` green; Console and other
  effects unaffected.

## Guardrails (do-not-reopen)

- **NO mock FS** — real driver + checked-in hermetic fixtures.
- **Reuse the `ITree`/Console effect machinery** (shared with `[State]`) — one
  effect-dispatch path, not two.
- **Capability-carrying, not ambient authority** — undeclared access rejected.
- **Kernel / `trusted_base` off-limits**; totality preserved in the pure core.
- **Multi-lane** — if too large for one WP, **decompose and propose to Steward**;
  do not silently narrow scope.

## Sequencing

- **Gate:** /spec + /conformance touching → spec enclave elaborates semantics +
  capability model on this WP branch (may split into a WP series), merges to
  `main` via the Integrator, then Runtime/Sec are kicked. Architect (totality +
  kernel-untouched + capability soundness) + Spec review (conformance-validator) +
  team QA + CI.
- **Suggested decomposition (enclave's call — propose the series back to
  Steward):** the landed substrate makes three lanes natural — **(a) Runtime
  driver** (`ken-interp` reduction dispatching `read_bytes` → a real host-I/O
  handler, mirroring `run_io`; failure as `Option`/`Result`) — the AC2 core;
  **(b) Sec capability thread** (mint/thread a `Cap_FS` from surface `[FS]` to the
  driver so an undeclared access is rejected at runtime — the AC3 discriminating
  pair, reusing the landed `CapToken`/`attenuate`); **(c) conformance** (checked-in
  hermetic fixtures + real driver, the `read-file-lines` shape). (a) is the
  spine; (b)/(c) layer on it. If the enclave keeps it as one WP, that is fine too —
  the point is **don't silently narrow scope**.
- **Lane:** Runtime (`ken-interp` driver) + Sec (`ken-elaborator` capability) +
  conformance. Branch off `origin/main`.
- **Relation to siblings:** shares the effect-handler mechanism with `[State]`
  (both extend the Console `ITree` interpreter) — build the shared effect-dispatch
  path once. Sequence after `[State]` if they'd contend on the same effect-system
  files.
