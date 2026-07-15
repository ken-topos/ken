# PX5 — Native effect lowering (Runtime)

- **ID:** PX5 · **Owner:** Team Runtime · **Size:** L · **Risk:** High
  (trusted-base growth; new artifact ABI + capability carriage).
- **Objective:** Lower `RuntimeExpr::Effect` to the native host boundary for an
  explicit, minimal, contract-complete subset of the 14-op driven floor, through
  one sealed versioned `ken-host` op-shim — matching interpreter semantics
  observably, and producing the canonical trace/error/observation PX6 consumes.
- **Depends on:** PX4 (merged, `origin/main @ 513955fe`). **Gate:** G-Sec /
  native-effect lane. **Sibling:** PX6 (Verify) — released concurrently; PX6
  consumes what PX5 produces.
- **Feeds:** Phase-B exit (the Milestone-C CLI runs native, observationally
  identical to interpreter).

## Fixed inputs — DO NOT REOPEN (cite, do not relitigate)

The component + security design is **ruled** in **ADR-0018**
(`docs/adr/0018-native-effect-execution-contract.md`, Decision
`dec_2pgkqkddt6eh3` RESOLVED by the Architect, grounded `513955fe`). It is the
controlling contract; this frame transcribes it into deliverables + ACs. Also
binding:

- **NATIVE-EARLY** (operator op ruling) — PX-B before PX-C.
- **rustix exact-pinned behind the ken-host policy shell** (FORK 2,
  `evt_7qqf827rr1jxk`). Ken-authored raw declarations stay retired.
- **Unsupported ops stay stable *unavailable lanes*, NEVER a no-op / generic
  scalar call.** Preserve BOTH the backend
  `CraneliftBackendError::Unsupported{construct:"Effect"}`
  (`cranelift_backend.rs:1594`) and the policy lane
  `NativeEffectForeignExecutableStatus` (`native_execution_differential.rs:237`);
  a newly-executable op moves to `NativeTested`, the lane is not deleted.
- **NO affine/linear types.** Confinement is runtime-trusted + discriminator-
  netted, **not kernel-proved** (ADR-0017 honesty boundary). Zero kernel rule,
  zero new Ken postulate, no proof-of-confinement claim.
- **Retain the landed target-manifest identity assertion** when the lane goes
  executable (`target_abi.rs:3`); the effect-ABI hash *supplements*, never
  replaces it.
- **Inventory is 14 driven ops, not the stale 16** (ADR-0018 §Context): Console
  Read/Write/Flush/IsTerminal; Clock WallNow; FS ReadFile/WriteFile/AppendFile/
  Metadata/ReadDirectory/CreateDirectory/RemoveFile/RemoveDirectory/Rename.

## Initial native-tested subset (Steward-pinned) — the ONLY ops PX5 makes executable

**Native-tested set (5 of 14):** `Console.Write`, `Console.Flush`,
`Console.IsTerminal`, `FS.WriteFile`, `FS.ReadFile`. This is minimal but
exercises the *entire* contract end-to-end: the ambient (no-capability) path,
the capability-bearing FS path (generational token, cap-check, marshalling), a
dynamic `Bytes` reply (response arena), stdout+stderr byte observation, FS
create/modify delta, capability + `FileError`/`IOError` identity, the ordered
effect trace, and exit status.

**Everything else stays a named unavailable lane** (explicit
`RepresentedUnavailable`/`Unsupported`, discriminated, never a no-op):
`Console.Read`, `Clock.WallNow`, `FS.AppendFile`, `FS.Metadata`,
`FS.ReadDirectory`, `FS.CreateDirectory`, `FS.RemoveFile`, `FS.RemoveDirectory`,
`FS.Rename`. The catalog (§1 below) still enumerates all 14; only availability
differs. Follow-on WPs light these up incrementally against the same contract.

## Scope

**In scope (PX5 owns, per ADR §"Consequences for framing"):** the sealed
`HostOpV1` catalog + shared dispatch core; the `RuntimeExpr::Effect`
expressibility repair; the live-capability route (generational token + host
table from `ProgramCaps`); the private static host shim; the typed per-op
wire ABI + `HostEffectAbiV1` manifest; the response arena; canonical
trace/error/observation *production*; and per-op native availability.

**Out of scope (belongs elsewhere):** the differential harness, twin-root
execution, comparison, mutation net → **PX6 / `ken-verify`**. No general native
heap, mutation, closures, records, or aggregate egress. No public C embedding /
general Rust interop. No new kernel rule or Ken postulate. Do not build the 9
deferred ops' executable paths.

## Coordination note (concurrent with PX6 — read this)

PX6 (`crates/ken-verify`, Team Verify) is released concurrently and **imports the
canonical observation types** (`EffectObservationV1`, `EffectEventV1`, the
error-identity enum, `FsDeltaV1`) that PX5 defines. **Land that canonical
vocabulary early** in `ken-host`/`ken-runtime` (deliverable 7 below), before the
full native-effect production, so Verify can code its comparator against the
concrete types rather than only the ADR shapes. The two campaigns meet at the
5-op native-tested subset; coordinate the type-placement handoff with the Verify
leader through the Steward.

## Mandated deliverable outline — each section ends in a concrete, implementable choice

1. **Sealed versioned op catalog (`ken-host`).** Add one `HostOpV1` catalog for
   all 14 ops with explicit stable numeric IDs (declaration order is NOT an ABI),
   typed request/reply/semantic-error vocabulary, per-op availability, and a
   schema descriptor. Add the single shared dispatch core:
   `typed request → validate op + capability → dispatch_host_op_v1 → existing
   safe host leaf → typed reply` (ADR §1). The existing flat FS functions
   (`open_at`/`read`/`write_new`/…) remain **leaf mechanisms** called by the
   core. **Adapt** interp `PosixHost`/`CaptureHost`/Console/Clock backends and
   the new native wrapper to this one core — no second semantic switch anywhere.

2. **`RuntimeExpr::Effect` expressibility (`ir.rs`).** Replace the landed
   `Effect{effect, capability:Option<RuntimeSymbol>, args}` with
   `Effect{family:RuntimeSymbol, operation:HostOpV1,
   capability:Option<RuntimeCapabilityUse>, args:Vec<RuntimeExpr>}` and
   `RuntimeCapabilityUse{identity:RuntimeSymbol, value:RuntimeExpr}` (ADR §2).
   `identity` = stable provenance/trace identity only; `value` evaluates to the
   live opaque credential. `capability=None` is valid ONLY for the V1-ambient
   ops (Console, Clock); every FS op carries exactly one live cap. Semantic
   `args` exclude erased type params and the capability operand.

3. **Live-capability native carriage.** Extend PX4 borrowed ingress with one
   opaque capability kind: a generational `{slot, generation}` token indexing a
   host-owned capability table initialized from the entrypoint's declared
   `ProgramCaps`. Generated code may only *pass* the token in an op's capability
   position — never inspect/construct/modify/ground/capture/store/return/
   serialize it. The shim resolves the token and runs the SAME rights/authority/
   scope/symlink checks as the interpreter before the host call. Wrong
   slot/generation/kind/shape → `MalformedCapability` before any host action.
   **Relocate the one capability representation + safe check into a shared crate
   below both executors** (elaborator-facing types re-exported/adapted); do NOT
   copy `check_fs_capability` into `ken-runtime` or the FFI shim.

4. **Private static host shim + `abi_v1` unsafe boundary.** The produced
   executable links a private versioned Rust static host runtime whose single
   callable entry is morally `ken_host_dispatch_v1(context, op_id, request,
   reply) -> status` (one versioned entry, not one symbol per op; C-visible
   signature may use pointer/length pairs). `ken-host` may switch crate-wide
   `forbid(unsafe_code)` → `deny(unsafe_code)` to admit exactly ONE named,
   audited `abi_v1` module that validates/decodes raw pointers; no OS/host-ABI
   unsafe moves into `ken-runtime`, generated Cranelift, interp, or CLI. Restore
   **SIGPIPE → `IOError::BrokenPipe`** before the first Console op (the starter
   isn't under Rust `main`); signal termination must not bypass the trace.

5. **Per-op wire ABI + `HostEffectAbiV1` manifest.** Each op gets a named
   fixed-layout request/reply record in the semantic field order of the ADR §3
   table (Console.Write = {stream, bytes} → Result IOError Unit; FS.WriteFile =
   {capability, path, create-policy, bytes} → Result FileError Unit; FS.ReadFile
   = {capability, path} → Result FileError Bytes; etc.). Explicit-width integer
   tags/fields; pointer/length pairs checked for null/overflow/target-width/
   bounds/alignment/lifetime before deref. Generate `HostEffectAbiV1` from the
   single catalog descriptor; its canonical hash binds schema version, op
   IDs+availability, every request/reply size/align/offset/tag/arity, capability
   + borrowed layouts, response-arena lifetime version, semantic error IDs, and
   trace schema version. C-visible facts are **C-probed**; producer↔registry↔
   observer↔per-op-consumer inventories close **bidirectionally** with
   producer-only/registry-only/observer-only/value-mutation discriminators.
   Embed BOTH `TargetAbi` and `HostEffectAbiV1` hashes in the object + supply to
   the host context; mismatch/unavailable/unknown-op/wrong-size/malformed fails
   closed **before dispatch**.

6. **Response arena + resume-once.** Host replies are host-neutral typed values;
   generated lowering maps them to existing Ken response constructors and
   resumes the ITree continuation **exactly once**. Dynamic `Bytes`/`Constructor`
   reply graphs live in ONE immutable append-only response arena owned by the
   whole entry invocation (lasts until the Ken entry returns — a later
   continuation may retain a prior reply; per-effect reset is FORBIDDEN). Not a
   general heap.

7. **Canonical trace/error/observation production (what PX6 consumes).** Produce
   `EffectObservationV1{stdout:Bytes, stderr:Bytes, filesystem_delta,
   terminal_error, effect_trace, exit_status:i32}`. Append exactly one ordered
   `EffectEventV1{sequence, operation:HostOpV1, capability:Option<trace-identity>,
   request, outcome}` per completed `Vis`, after response/denial and before
   resume. The native trace sink is launcher-owned, write-only, out-of-band — it
   cannot touch stdout/stderr, consume the FS cap, enter `ProcessInput`, or feed
   Ken. Error identity is an **enum** (exact cap denials `RightNotHeld`(op+held
   bits)/`AuthorityInsufficient`/`ScopeEscape`/`SymlinkDenied`/
   `MalformedCapability`; Ken `IOError` incl `Other(raw_os_code)` only where the
   interpreter already exposes it; FS-op + relative raw-byte path + nested error;
   stable driver/ABI/trap failures) mapped by the SAME function both lanes. FS
   deltas: relative raw-byte path, node kind, file bytes, symlink-target bytes;
   exclude inode/device/timestamps/absolute roots/non-observable perms;
   directory order canonical byte order. Exit status via PX4's shared
   `ProcessExitCode → i32` mapper.

8. **Explicit availability + honesty.** All 14 ops in the catalog; only the
   5-op subset is `NativeTested`; the other 9 are explicit
   `RepresentedUnavailable`/`Unsupported` lanes. Source discloses tested/target-
   validated, never proved. Enumerate the trusted-surface delta (every unsafe
   block + its lifetime/alignment/size/aliasing obligation).

## Acceptance criteria (testable)

- **AC1 — subset executes natively, matches interp observably.** For each of
  Console.Write/Flush/IsTerminal, FS.WriteFile, FS.ReadFile: a Ken entry run
  through the produced native artifact yields the SAME `EffectObservationV1`
  (stdout/stderr bytes, FS delta, error identity, ordered trace, exit) as the
  interpreter on identical ProcessInput/ProgramCaps. (Interp is the oracle.)
- **AC2 — capability route is real + fail-closed.** FS.WriteFile/ReadFile go
  through the generational token + host table; a wrong slot/generation/kind/shape
  → `MalformedCapability` **before** any host action; a right-not-held / scope-
  escape / symlink-denied case reproduces the interp's exact denial enum. No
  ambient FS cap; generated code cannot forge/inspect the token (structural).
- **AC3 — one shared dispatch + one cap-check + one error-map.** Structurally
  assert there is no second semantic op-switch and no second cap-check/error-map
  (interp and native both route through the shared core); `check_fs_capability`
  is NOT copied into `ken-runtime`/FFI.
- **AC4 — manifest closure.** `HostEffectAbiV1` producer↔registry↔observer↔
  per-op-consumer inventories close bidirectionally; producer-only, registry-only,
  observer-only, and one field-layout mutation each fail the build closed. Both
  `TargetAbi` + `HostEffectAbiV1` hashes embedded; a hash/size/field mismatch
  fails closed before dispatch. C-visible layout is C-probed.
- **AC5 — response arena lifetime.** A reply retained across a later
  continuation stays valid (arena lives to entry return); a probe that would
  observe per-effect reset fails. No general heap/mutation/aggregate egress
  introduced (structural).
- **AC6 — unavailable lanes preserved.** Each of the 9 deferred ops is an
  explicit named unavailable lane: backend `Unsupported{construct:"Effect"}` +
  policy `NativeEffectForeignExecutableStatus` remain; the PX4 test
  `host_effect_execution_remains_a_named_unavailable_lane`
  (`native_process_entrypoint.rs:512`) is preserved/extended (deferred ops still
  fail explicit + named on the artifact, exit 1, stderr-reported — never silent
  success). SIGPIPE→`BrokenPipe` verified (no signal-kill bypass of the trace).
- **AC7 — confined unsafe + honesty.** Exactly the named `abi_v1` unsafe module
  in `ken-host`; `ken-interp` keeps `forbid(unsafe_code)`; no host-ABI unsafe in
  `ken-runtime`/generated code/CLI. Trusted-surface delta enumerated; disclosure
  is tested/validated, not proved. No kernel/spec/conformance movement unless a
  reviewer directs it. **No-regression = green in CI** (full locked workspace),
  never a local `--workspace` run.

## Do-not-reopen guards

- Do NOT re-derive the contract — ADR-0018 is controlling; a genuinely new fixed
  boundary (only) hard-stops and routes to the Steward/Architect.
- Do NOT treat `Option<RuntimeSymbol>` as the credential, pass authority/root/fd/
  pointer as a scalar cap, keep a second dispatch switch, use a generic
  scalar/vector FFI, reset the arena per-effect, or normalize away
  nondeterminism (all explicitly REJECTED in ADR §"Rejected alternatives").
- Do NOT widen the native-tested subset beyond the pinned 5 without Steward
  re-scoping.
- Do NOT copy `check_fs_capability`; relocate to one shared impl.

## Grounding anchors (landed `513955fe`)

Effect stub `cranelift_backend.rs:1594` (+ ctor `:2645`, err enum `:186`,
`Lowered` `:1365`); sibling rejections `runtime_ir_evaluator.rs:970,1286`,
`platform_runtime_support.rs:735`, `artifact_validation.rs:754`. Interp
reference `eval.rs:run_io:4216 → fs_dispatch:3900 → check_fs_capability:3736 →
ken_host::* → apply(k):4389`. ken-host flat FS `lib.rs:425–571`; manifest
`TargetAbi:40`/`assert_target_abi_identity:72`; `build.rs:9 SCHEMA_VERSION`. IR
`ir.rs:376`. Capability `eval.rs:189` + `capabilities.rs`. Policy lane
`native_execution_differential.rs:237`; corpus-exclusion `HostEffectExecution
:116`. Manifest note `target_abi.rs:3`. Marshalling precedent
`conversions.rs:67`. **Test homes (ken-runtime = inline `#[cfg(test)]`, no
`tests/` dir):** `native_process_entrypoint.rs mod tests:234` (lane test `:512`);
`cranelift_backend.rs` tests (`unsupported_effect_is_distinct_from_backend_failure
:4053` to extend); `native_execution_differential.rs` inline. Interp oracle:
`crates/ken-interp/tests/` + inline `eval.rs`.
