# ADR 0020 — Account-database effective-user home roots

- **Status:** Accepted.
- **Date:** 2026-07-16.
- **Deciders:** Architect, with operator sanction for the bounded libc/NSS
  dependency.
- **Relates to:** ADR 0017 (scoped-capability trust boundary), ADR 0019 §3
  (typed filesystem-root specifications), PX15 (bind-once root handles), and
  PX16 (implementation).

## Context

Tool authors need a `~/` filesystem-authority root. `$HOME` is forgeable
process input and cannot select authority. `/home/<name>` conventions and
manual account-file parsing do not implement the host account policy. The
effective user's configured home is supplied by the system account database,
which may delegate to NSS and is not a kernel fact.

ADR 0019 identifies `getpwuid_r(geteuid())` as the principled source but holds
the libc/NSS boundary for a dedicated decision. PX14 already snapshots the
effective UID once for startup posture. A second UID observation would permit
the posture and home-root decisions to disagree.

## Decision

`FsRootSpec` gains `EffectiveUserHome(suffix)`. The `~/suffix` declaration is
checked and plan-bound as this typed choice. At capability-table initialization,
both executor lanes consume PX14's immutable effective-UID snapshot, resolve
the account record once, open the returned home once, resolve the suffix
component by component without following symlinks, and retain only the ordinary
handle-backed `FsScope`. No home spelling or bytes enter `ProgramCaps`,
`ProcessInput`, operation paths, canonical observations, or artifact hashes.

The account lookup is confined to a private `ken-host::account_db_v1` module.
PX16's new libc/NSS unsafe allowance is confined to that module, which exposes
a safe owned-byte facade. The pre-existing audited `abi_v1` unsafe boundary
remains separate and unchanged. The Linux implementation uses exact-pinned
`libc::getpwuid_r`: a 1 KiB initialized buffer doubles only after `ERANGE` up to
a 1 MiB hard cap. Success requires the returned pointer to equal the supplied
record, matching UID, an in-buffer `pw_dir`, a bounded terminating NUL, a
nonempty value, and an absolute path. Bytes are copied before the buffer drops.
No libc type, pointer, or borrow crosses the module boundary.

Lookup and binding failures are exact
`HomeRootResolutionFailureV1` values. Native startup maps them to
`TerminalErrorV1::HomeRootResolutionFailed` through the existing no-context
startup writer and shared nonzero exit mapping, before capability mint or any
effect. NSS has no wall-clock bound; this decision adds no timeout claim.

Testing uses three complementary faces. A private in-process scripted lookup
drives the real initialization/resolution helper and every success/failure
branch. PX6 continues to compare post-resolution interpreter/native semantics
over lane-local handles. A separate linked-artifact integration invokes the
child's real libc/NSS lookup and accepts only successful startup or exact
`NoAccountRecord`. No scripted answer crosses `exec`; there is no test-only
authority ingress.

## Trust statement

This is the first direct libc/NSS trusted surface in Ken. The exact Rust crate
binding is pinned and checksum-locked, but the dynamically selected system
libc, NSS configuration, modules, and account-database answer are delegated
host policy. The boundary is runtime-trusted and discriminator-tested, never a
Ken or kernel proof. It adds no postulate, primitive, or affine/linear claim.

## Rejected alternatives

- `$HOME`, CLI, environment, or `ProcessInput` authority injection.
- A second `geteuid`, saved/fs UID substitution, or username lookup.
- Manual `/etc/passwd` parsing or a `/home/<name>` convention.
- A higher-level NSS crate, rustix libc backend, hand-written C, or unbounded
  `CStr::from_ptr` scanning.
- Passing a scripted home answer to a child through environment, artifact
  metadata, the observation sink, callback, or a new FFI field.
- A wall-clock timeout over NSS policy resolution.
