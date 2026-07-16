# ADR 0019 — Capability evolution and process admission

- **Status:** Accepted design; implementation staged through PX13, PX14, and
  PX15.
- **Date:** 2026-07-16.
- **Deciders:** Architect, with the Steward accepting and routing the resulting
  work packages.
- **Relates to:** ADR 0017 (scoped-capability trust boundary), ADR 0018
  (native-effect execution contract), PX13 (mode mutation), PX14 (effective-root
  admission), and PX15 (execution-start working-directory roots).

## Context

ADR 0017 fixes the authority model and its honesty boundary: capability
confinement is trusted runtime behavior, covered by discriminating tests, and is
not a kernel proof. ADR 0018 fixes native effect execution, ABI binding, and the
canonical differential-observation contract. Neither should become a grab-bag
for each later capability or process-startup extension.

This decision names the extension points shared by filesystem mode mutation,
root-execution admission, and path-root resolution. It does not add a kernel
rule, a Ken postulate, a confinement proof, or linear or affine types. Every
claim here is runtime-trusted and discriminator-tested, never kernel-proved.

## Decision

### 1. Operation-right evolution

A new operation that changes access-control metadata receives a distinct
capability right. It does not reuse a content-write right or a metadata-read
right. Rights attenuation remains narrowing-only and intersects new rights in
the same way as existing rights.

PX13 therefore adds `CHANGE_MODE` for `chmod`. `WRITE` does not imply
`CHANGE_MODE`, and `CHANGE_MODE` does not imply `WRITE`. Full authority includes
the right; partial authority does not. The operation resolves its target through
the existing component-by-component no-follow walk and applies `fchmod` to the
already-authorized handle. It performs no path lookup after authorization and
accepts only mode values whose bits are contained in `0o7777`.

An observation field is added only when the Ken surface exposes the
corresponding mutation. The mode observation is `st_mode & 0o7777` for regular
files and directories and is absent for symlinks and unsupported node kinds.
It excludes file-type bits. Numeric owner and group identifiers, timestamps,
ACLs, xattrs, inode identities, device identities, and absolute-root spellings
remain unobserved. Without a portable principal model, numeric owner data would
leak a host namespace rather than describe Ken semantics.

`chown` and `chgrp` are not part of PX13. Any future ownership mutation requires
a separate `CHANGE_OWNER` right, a principal and user-namespace model, a
privilege contract, and a separately versioned observation decision.

### 2. Process-admission posture

Effective-root admission is a startup declaration check, not a capability
token. The v1 privilege predicate is exactly `geteuid() == 0`. Real, saved, and
filesystem user IDs, Linux capability sets, user namespaces, securebits, and
other privilege models are deferred.

The declaration surface is:

```text
program capabilities FS <authority>, RootExecution Allow
```

Omission means deny. The marker is checked declaration metadata. It is not a
field of the Ken-visible `ProgramCaps a` value, not an FS right, not a scalar
that generated code may forge, and not authority escalation. The compiler binds
it into the native entrypoint plan and its hash. No command-line flag or
environment variable may add it.

One shared startup-admission function consumes an immutable effective-UID
snapshot and the checked declaration. Interpreter and native execution call it
before constructing `ProcessContext`, before minting or inserting any
capability-table grant, and before the first effect. The resulting posture
witness records completion of both this admission and the required SIGPIPE
posture.

When the effective UID is zero and the declaration omits the allowance,
startup produces the unit terminal outcome
`TerminalErrorV1::RootExecutionDenied`. It is not a synthetic host effect:
effect trace, filesystem delta, stdout, and stderr are empty, and the status is
mapped through the one shared `ProcessExitCode` mapping. Native startup must be
able to emit that canonical terminal observation without constructing a live
`ProcessContext`.

### 3. Path-root resolution

An execution-start working-directory root is a typed root specification. `./`
captures the working directory once during capability-table initialization,
opens the root handle then, and retains no ambient working-directory dependency
afterward. Suffixes resolve component by component beneath that handle using the
existing `ScopeEscape` and `SymlinkDenied` policy. Canonical observations remain
relative to the resolved capability root and never expose its spelling.

`~/` is not admitted by this decision. `$HOME` is forgeable ambient input and
must not select an authority root. The principled effective-user-home source is
`getpwuid_r(geteuid())`, snapshotted once, but that is a libc/NSS
account-database boundary rather than a Linux syscall. It conflicts with the
settled rustix/linux-raw-only host boundary and therefore requires a
prerequisite work package that owns the dependency and trusted-surface delta,
startup snapshot, and injectable differential seam. Manual `/etc/passwd`
parsing and `/home/<name>` conventions are also rejected.

ADR 0020 supplies that bounded account-database boundary and is the normative
owner of effective-user home resolution; this section remains the semantic
owner of the typed root-specification choice.

## Consequences

Capability evolution remains additive and fail-closed: new operation IDs,
rights, request and reply layouts, canonical observations, manifest hashes,
registries, producers, consumers, and differential tests move as one closed
inventory. Existing operation IDs and meanings do not change, and an old
inventory or hash is rejected.

PX13 ships only mode mutation and mode observation. PX14 implements the shared
effective-root startup check. PX15 implements only the execution-start
working-directory root. Effective-user-home resolution remains held behind the
account-database boundary.

These mechanisms enlarge trusted runtime behavior, not the kernel TCB. Their
security posture remains explicit: runtime-trusted, target-validated, and
discriminator-tested, with no claim that the kernel proves capability
confinement or process admission.

## Rejected alternatives

- Reusing `WRITE` or metadata-read authority for mode mutation.
- Adding ownership mutation or numeric owner observations without a principal
  model.
- Reopening a path after authorization instead of operating on the authorized
  handle.
- Modeling root execution as a forgeable capability token, CLI flag, or
  environment override.
- Using `$HOME`, parsing `/etc/passwd` manually, or assuming `/home/<name>` for
  an authority root.
- Extending ADR 0017 or ADR 0018 into a collection of unrelated future
  capability and startup decisions.
