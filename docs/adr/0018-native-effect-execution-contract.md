# ADR 0018 — Native effect execution and differential observation

- **Status:** **Accepted** — Architect component and security ruling for PX5
  and PX6 (2026-07-15). The Steward owns transcription into the two work
  packages; implementation and terminal review remain separate gates.
- **Date:** 2026-07-15.
- **Grounded at:** PX4-landed `origin/main`, commit
  `513955fe781f696c9ebcc81873eac90b675cae53`.
- **Decider:** Architect.
- **Relates to:** ADR 0011 (platform-dependent code), ADR 0017 (scoped
  capabilities), `docs/program/09-posix-linux-abi-campaign.md` Phase PX-B,
  and `spec/40-runtime/42-evaluation.md` §6.

## Context

PX5 lowers `RuntimeExpr::Effect` to the native host boundary. PX6 compares the
interpreter and native executions by their external effects. They therefore
need one contract for operation identity, capability carriage, marshalling,
response lifetime, errors, and traces. Defining those independently would
create two implementations and then compare their disagreement.

The landed tree does not yet state this contract:

- `ken-host` is a safe Rust filesystem shell. It has no effect-operation
  catalog or dispatcher; Console and Clock live on `ken-interp::HostHandler`.
- `RuntimeExpr::Effect { effect, capability, args }` carries a string and an
  optional symbol. It carries neither a sealed operation identity nor a live
  capability value. The symbol cannot authorize a host call.
- PX4's borrowed ingress contains only Bytes and Constructor values. Native FS
  effects also need an opaque live capability and dynamically produced
  responses.
- the existing Runtime differential report observes a Runtime result, UTF-8
  stdout, and an exit status. That cannot distinguish reordered effects,
  stderr, denial identity, or filesystem mutation.

The campaign's original inventory is stale. The landed driven floor is
**fourteen operations**, not sixteen:

- Console: Read, Write, Flush, IsTerminal;
- Clock: WallNow;
- FS: ReadFile, WriteFile, AppendFile, Metadata, ReadDirectory,
  CreateDirectory, RemoveFile, RemoveDirectory, Rename.

This ADR fixes the contract. It does not claim that all fourteen operations are
immediately native-executable. An operation may remain a stable unavailable
lane until its complete request, response, and deterministic evidence path
exist.

## Decision

### 1. One sealed, versioned host-operation catalog

`ken-host` owns a sealed `HostOpV1` catalog for the fourteen operations above.
It also owns the request and reply types, semantic host errors, operation
availability, schema descriptor, and the one shared dispatch core. Numeric op
IDs are explicit and stable within version 1; declaration order is not an ABI.

There is exactly one policy switch:

```text
typed request
  -> validate operation and capability
  -> dispatch_host_op_v1
  -> existing safe host leaf operation
  -> typed reply
```

The existing flat `ken-host` filesystem functions remain leaf mechanisms. The
interpreter's Console, Clock, CaptureHost, and PosixHost implementations are
adapted behind the same catalog and shared dispatch semantics; they do not keep
an independent operation switch. A new native wrapper calls that same dispatch
core. Unsupported and unavailable operations return a named status before any
host action; they are never no-ops and never generic scalar foreign calls.

The produced executable links a private, versioned Rust static host runtime.
Its callable symbol is the moral equivalent of:

```text
ken_host_dispatch_v1(context, op_id, request, reply) -> status
```

The exact C-visible signature may use pointer/length pairs, but it must remain a
single versioned entry rather than one ad hoc symbol per operation. It is a
private Ken artifact ABI, not a public C embedding API and not general Rust
interop.

Because the produced starter does not run under Rust's standard `main`, it must
establish the same SIGPIPE posture before the first Console operation. A broken
pipe is mapped to the existing `IOError::BrokenPipe`; it must not terminate the
process by signal or bypass the effect trace.

The safe Rust API and the FFI decoder remain visibly distinct. `ken-host` may
replace crate-wide `forbid(unsafe_code)` with `deny(unsafe_code)` only to admit
one named, audited `abi_v1` module. That module alone may validate and decode
raw pointers. No OS or host-ABI unsafe moves into `ken-runtime`, generated
Cranelift code, the interpreter, or the CLI. The trusted-surface report must
enumerate every unsafe block and its lifetime, alignment, size, and aliasing
obligation.

### 2. Runtime IR carries the operation and the live capability

The landed `RuntimeExpr::Effect` shape is insufficient and must not be assigned
an ABI by reinterpretation. PX5 first gives it an explicit sealed operation and
an actual capability operand. The required information is:

```text
Effect {
    family: RuntimeSymbol,
    operation: HostOpV1,
    capability: Option<RuntimeCapabilityUse>,
    args: Vec<RuntimeExpr>,
}

RuntimeCapabilityUse {
    identity: RuntimeSymbol,
    value: RuntimeExpr,
}
```

Equivalent Rust ownership boxing is mechanical. The semantic distinction is
not: `identity` is stable provenance and trace identity; `value` evaluates to
the live opaque credential. Looking up authority by symbol, treating the
symbol as the credential, or consulting an ambient FS-capability map is
forbidden.

`capability = None` is valid only for catalog operations explicitly declared
ambient in V1: Console and Clock. Every FS operation requires exactly one live
capability. The effect's semantic `args` exclude erased type parameters and the
capability operand; the operation descriptor fixes their order and types.

Native startup extends PX4's borrowed ingress with one opaque capability kind.
The host context owns a capability table. A borrowed capability is only a
generational table token such as `{slot, generation}`; the table entry owns the
actual scoped capability, including rights, coarse authority, root handle,
symlink policy, and lineage. Generated code cannot inspect, construct,
arithmetically modify, ground, capture, store, return, or serialize the token.
It can only pass it in the capability position of a catalog operation.

The shim resolves the token and performs the same rights, authority, scope,
and symlink checks as the interpreter before calling the host. A wrong slot,
generation, kind, or operand shape is `MalformedCapability` before any host
operation. The capability table is initialized from the entrypoint's declared
`ProgramCaps`; there is no ambient native FS capability and no scalar
authority reminting.

The capability check and representation must have one implementation below the
two executors. PX5 may relocate the runtime capability representation and safe
check into a lower shared crate, with elaborator-facing types re-exported or
adapted, but it may not copy `check_fs_capability` into `ken-runtime` or the FFI
shim. This preserves ADR 0017's honesty boundary: confinement is
runtime-trusted and discriminator-netted, not kernel-proved.

### 3. Per-operation wire ABI and manifest identity

The native boundary uses typed request and reply records selected by
`HostOpV1`, never `Vec<scalar>`, `HostValue[]`, debug text, or a generic foreign
call. The V1 semantic schemas are:

| Operation | Request excluding erased types | Reply |
|---|---|---|
| Console.Read | stream, bounded limit | Result IOError ReadResult |
| Console.Write | stream, bytes | Result IOError Unit |
| Console.Flush | stream | Result IOError Unit |
| Console.IsTerminal | stream | Bool |
| Clock.WallNow | none | Instant |
| FS.ReadFile | capability, path | Result FileError Bytes |
| FS.WriteFile | capability, path, create policy, bytes | Result FileError Unit |
| FS.AppendFile | capability, path, bytes | Result FileError Unit |
| FS.Metadata | capability, path | Result FileError FileMetadata |
| FS.ReadDirectory | capability, path | Result FileError List-DirEntry |
| FS.CreateDirectory | capability, recursive, path | Result FileError Unit |
| FS.RemoveFile | capability, path | Result FileError Unit |
| FS.RemoveDirectory | capability, recursive, path | Result FileError Unit |
| FS.Rename | capability, source path, destination path | Result FileError Unit |

The table records semantic order. Each C-visible request/reply layout is a
separate named fixed-layout record. Tags and integral fields use explicit-width
integers where possible. Pointer/length pairs are checked for nullability,
overflow, target-width conversion, bounds, alignment, and the declared
lifetime before dereference. Every enum tag, stream direction, limit, create
policy, result tag, constructor arity, and payload length is validated.

`TargetAbi` remains necessary but is not sufficient: Linux constants cannot
describe the high-level effect wire schema. PX5 adds a generated
`HostEffectAbiV1` manifest from the single operation descriptor source. Its
canonical hash binds at least:

- schema version and all operation IDs and availability states;
- each request and reply size, alignment, field offset, tag, and arity;
- capability-token and borrowed-value layouts;
- response-arena lifetime version;
- semantic error IDs and trace schema version.

C-visible layout facts are independently C-probed. Pointer width and C integer
width come from the target manifest. Producer, registry, observer, and
per-operation consumer inventories must be equal in both directions, including
empty flag values where relevant. Producer-only, registry-only, observer-only,
and value-mutation discriminators are required.

Both the `TargetAbi` hash and `HostEffectAbiV1` hash are embedded in the object
and supplied to the host context. A mismatch, unavailable backend, unknown op,
wrong record size, or malformed field fails closed before dispatch. The landed
target-manifest identity assertion is preserved; the effect hash supplements
it rather than replacing it.

Host replies are host-neutral typed values. Generated lowering maps them to the
existing Ken response constructors and resumes the ITree continuation exactly
once. Dynamic Bytes and Constructor reply graphs live in one immutable,
append-only response arena owned by the process invocation. The arena lasts
until the Ken entry returns, so a reply remains valid across later
continuations; resetting it after each operation is forbidden. The arena is a
bounded effect-response lifetime mechanism, not a general Ken heap. Records,
closures, mutation, aggregate egress, and general allocation remain out of
scope.

### 4. Canonical observation, trace, and error identity

PX5 exposes one canonical observation vocabulary consumed unchanged by PX6.
It is byte-oriented:

```text
EffectObservationV1 {
    stdout: Bytes,
    stderr: Bytes,
    filesystem_delta: Vec<FsDeltaV1>,
    terminal_error: Option<TerminalErrorV1>,
    effect_trace: Vec<EffectEventV1>,
    exit_status: i32,
}

EffectEventV1 {
    sequence: u64,
    operation: HostOpV1,
    capability: Option<CapabilityTraceIdentity>,
    request: CanonicalRequestV1,
    outcome: CanonicalOutcomeV1,
}
```

Exactly one event is appended for each completed `Vis`, after its host response
or semantic denial is known and before its continuation resumes. A malformed
operation that cannot dispatch produces a terminal error, not a completed
effect event. The trace sink is launcher-owned, write-only from the native
entry, and out of band: it cannot contaminate stdout/stderr, consume the
program's FS capability, enter `ProcessInput`, or feed information back to Ken.

Canonical requests and responses preserve the typed semantic fields in the
table above. They contain raw bytes for paths and console data. They never
contain raw pointers, capability table slots, file descriptors, inode/device
numbers, absolute temporary-root paths, allocation addresses, or debug text.
`CapabilityTraceIdentity` is the stable declared ProgramCaps field identity,
not a credential.

Semantic error identity is an enum, not a message string. It preserves:

- exact capability denials: `RightNotHeld` with operation and held-right bits,
  `AuthorityInsufficient`, `ScopeEscape`, `SymlinkDenied`, and
  `MalformedCapability`;
- the existing Ken `IOError` identities, with `Other(raw_os_code)` only where
  the interpreter already exposes the raw code;
- FS operation identity, normalized relative raw-byte path, and the nested
  `IOError` or capability denial used to construct `FileError`;
- terminal driver failures: stable unknown family/operation identity,
  `UnknownTree`, malformed ITree shape, malformed host ABI field, and stable
  runtime trap code.

Error comparison uses this enum. OS prose and platform display strings are
diagnostics only. The interpreter and native paths must map the same host error
through one mapping function; two independently maintained maps are forbidden.

Filesystem deltas compare snapshots by relative raw-byte path, node kind, file
bytes, and symlink target bytes where present. They record created, removed,
and modified nodes. They ignore inode/device identity, absolute root names,
timestamps, and permissions not observable through the current Ken surface.
Directory order is canonical byte order. Stdout and stderr are raw byte
streams. Exit status uses the shared fail-closed `ProcessExitCode -> i32`
mapping already established by PX4.

No nondeterministic value is normalized away. In particular, two real
`Clock.WallNow` responses are not equal merely because both are clocks. The
operation is `NativeTested` only when both lanes consume the same scripted
clock response through the common host boundary; otherwise it remains
`RepresentedUnavailable` in V1.

### 5. PX6 lives in `ken-verify` and uses twin real roots

PX6 creates the workspace crate `crates/ken-verify`, owned by Team Verify. It
contains the independent executor orchestration, comparator, mismatch report,
and mutation discriminators. Canonical operation, error, trace, and observation
types live below it in `ken-host` or `ken-runtime`; `ken-verify` consumes them
and does not redefine them.

The existing `ken-runtime` native-execution differential/provenance report
remains Runtime's artifact and policy substrate. PX6 may call it, but must not
turn Runtime into the owner and judge of its own effect equivalence.

The passing FS lane runs the interpreter and the produced native executable
against two byte-identical real temporary-directory roots. Each lane receives
the same raw ProcessInput, scripted stdin and ambient inputs, declared
ProgramCaps shape, and scenario. The interpreter uses the real Posix host and a
capability minted for root A; the native host context owns the equivalent
capability for root B. PX6 compares the two canonical observations and final
root deltas.

`CaptureHost` remains valuable for unit tests, malformed requests, and negative
controls, but it is not the passing differential substrate: a virtual host
cannot prove that the produced artifact performs the real Linux filesystem
operation. Clock uses an identical scripted response or stays unavailable.

PX6 must include discriminators for silent skip, duplicated resume, reordered
events, stdout/stderr swap, path-byte normalization, weakened error identity,
wrong capability token, denied-before-host-action, filesystem mutation without
trace, trace without mutation, target/effect manifest mismatch, and each
operation status transition. A lane moves from
`RepresentedUnavailable`/`Unsupported` to `NativeTested` only when its exact
artifact plus external-delta evidence passes. The backend's existing
`Unsupported { construct: "Effect" }` and
`NativeEffectForeignExecutableStatus` remain stable for operations outside the
native-tested set.

## Security and trust posture

This design adds a native host-runtime TCB surface. It does not add a kernel
rule, Ken postulate, or proof of confinement. The security claim is exactly:

- the checked program cannot forge or inspect a native capability token;
- the shared runtime host boundary validates the token, rights, authority,
  scope, symlink policy, ABI identity, and request shape before host action;
- the interpreter and native artifact are tested against the same canonical
  external observation contract.

Those are tested and target-validated runtime properties. They are not Ken
proofs. PX5 and PX6 must report the trusted delta and retain the campaign's
honest unavailable lanes.

## Rejected alternatives

- **Treat `Option<RuntimeSymbol>` as the credential.** A name proves no live
  authority and permits ambient reminting.
- **Pass authority, path root, fd, or pointer as an ordinary scalar.** This
  makes credentials forgeable or leaks host identity into generated code.
- **Keep separate interpreter and native dispatch switches.** The
  differential harness would compare two policies instead of two executions.
- **Use a generic scalar/value-vector FFI.** It cannot state per-operation
  arity, type, ownership, or result lifetime and weakens fail-closed checking.
- **Reset response storage after each effect.** Continuations may retain the
  preceding response, creating dangling borrowed values.
- **Compare error strings, UTF-8 output, return values, or normalized clock
  reads.** Each loses a load-bearing external distinction.
- **Use CaptureHost as the PX6 passing lane.** It does not exercise the linked
  artifact's Linux host boundary.
- **Place PX6 in `ken-runtime`.** Runtime would own both the implementation and
  the supposedly independent equivalence judgment.
- **Build a general native heap or public C embedding in PX5.** Neither is
  required for the fixed effect floor.

## Consequences for framing

PX5 owns the catalog, IR expressibility repair, live capability route, shared
dispatch, private static host shim, typed ABI and manifest, response arena,
canonical trace/error production, and per-operation native availability.

PX6 owns `ken-verify`, twin-root execution, canonical comparison, mismatch
diagnostics, and the external-delta/mutation net. PX6 may not repair PX5 by
inventing comparator-side normalization.

The frames must use the fourteen-operation landed inventory, not the stale
sixteen-operation campaign count. They must state the initially native-tested
subset explicitly. Every other catalog operation remains a named unavailable
lane until its full contract is executable and discriminated.
