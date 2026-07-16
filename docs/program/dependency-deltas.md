# Dependency-delta records (`spec/60-security/63`, ADR 0009)

Per-addition records for external crates curated into the tool-chain (ADR
0009's "select an industry-trusted component" rubric step 1). Each entry
documents the vetting the merge Decision checked, so the addition to the
tool-chain's own trusted computing base is legible and re-checkable on update.

## PX1 — Linux host boundary (`ken-host`)

PX1 retires six Ken-authored Linux ABI declarations. Five filesystem
facilities move behind a first-party policy shell over `rustix`'s typed
Linux-raw backend; the former signal mutation is replaced by the supported
Rust-standard-runtime entrypoint contract documented below. The dependency is
private to `ken-host`; no `rustix` type crosses the crate's public API.

### Closed production dependency graph

The population is defined by Cargo's target-selected normal-dependency graph
for `x86_64-unknown-linux-gnu`, not by a source-name search. With
`rustix = { version = "=1.1.4", default-features = false, features =
["std", "fs", "process"] }`, and without `rustix_use_libc` or Miri, the
governing target-selected Linux-raw compiled closure is **three crates**:
`rustix`, `bitflags`, and `linux-raw-sys`. The complete all-target union is
**seven crates, enumerated here for information**:

| crate | version | checksum | license |
|---|---:|---|---|
| `rustix` | `1.1.4` | `b6fe4565b9518b83ef4f91bb47ce29620ca828bd32cb7e408f0062e9930ba190` | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `bitflags` | `2.13.0` | `b4388bee8683e3d04af747c73422af53102d2bd24d9eadb6cbc100baef4b43f8` | MIT OR Apache-2.0 |
| `linux-raw-sys` | `0.12.1` | `32a66949e030da00e8c7d4434b251670a91556f4144941d37452769c25d58a53` | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `errno` | `0.3.14` | `39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb` | MIT OR Apache-2.0 |
| `libc` | `0.2.186` | `68ab91017fe16c622486840e4c83c9a37afeff978bd239b5293d61ece587de66` | MIT OR Apache-2.0 |
| `windows-sys` | `0.61.2` | `ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc` | MIT OR Apache-2.0 |
| `windows-link` | `0.2.1` | `f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5` | MIT OR Apache-2.0 |

`bitflags`, `linux-raw-sys`, `libc`, and `windows-link` have no normal
dependencies. `errno` reaches `libc` and `windows-sys`; `windows-sys` reaches
`windows-link`. This closes the all-target union. On the actual Linux-raw
target, Cargo selects only `rustix`, `bitflags`, and `linux-raw-sys`:

- `windows-sys` and `windows-link` are excluded by `cfg(windows)`;
- `libc` is the excluded rustix fallback backend; and
- `errno` is the excluded non-Linux-raw errno path.

Cargo nevertheless adds `errno` to this workspace's lockfile resolution, so
the lockfile delta contains **four** new packages even though only three
compile for the governing target.
The `std` feature supplies allocation and standard error integration, `fs`
supplies the five filesystem facilities, and PX14's `process` feature supplies
the single immutable `process::geteuid` startup observation. No other `rustix`
API feature is enabled. `linux_raw` is the resulting backend configuration,
not a Cargo feature; enabling `use-libc` or setting `rustix_use_libc` is
forbidden.

### Exercised upstream `unsafe` surface

PX1 and PX14 exercise exactly six typed `rustix` facilities:

- `fs::openat`, returning an owned descriptor and taking typed `OFlags` and
  `Mode`;
- `fs::readlinkat`, taking a borrowed descriptor and validated path argument;
- `fs::mkdirat`, `fs::unlinkat`, and `fs::renameat`, taking borrowed
  descriptors, validated path arguments, and typed flags/modes.
- `process::geteuid`, read once into an immutable startup snapshot for PX14's
  tested root-execution admission posture.

Those wrappers reach `rustix`'s private `linux_raw` syscall/assembly backend and
`linux-raw-sys`'s generated ABI types/constants. PX1 does not exercise
`ioctl`, networking, process mutation, memory mapping, io_uring, time, or the
libc backend. The host guarantees remain tested/validated, never proved.

SIGPIPE is not part of the `rustix` surface. Ken has exactly one supported
entrypoint: the standard-Rust `ken` binary. Rust standard binaries and Rust
test binaries set SIGPIPE to ignored before `main`, so a closed stdout pipe is
reported as an EPIPE-derived I/O error rather than terminating the process by
signal. Ken exposes no `cdylib`/`staticlib`/C embedding and does not opt out via
`#[unix_sigpipe]`. A future non-Rust-standard-runtime embedding must
re-establish SIGPIPE handling at its entrypoint before calling Ken.

**Proportionality result:** the governing Linux-raw compiled closure is **3
crates** and is proportionate to consolidating the evaluator's complete host
boundary (six raw ABI facilities, their nine unsafe call sites, thirteen
handwritten ABI facts, descriptor ownership, errno translation, and path-buffer
coupling). The all-target union's N=7 is informational rather than the stop
measure. The scope ruling therefore releases the dependency precondition.

## WP F1 — arbitrary-precision `Int` (`ken-interp`)

Sourced for `spec/10-kernel/18a §5.2.1` — the "iff bignum" delivery contract
for `add_int`/`sub_int`/`mul_int`/`eq_int`.

| crate | version (pinned) | license | `unsafe` status |
|---|---|---|---|
| [`num-bigint`](https://crates.io/crates/num-bigint) | `=0.4.6` | MIT OR Apache-2.0 | Not `forbid(unsafe_code)`; 12 `unsafe` blocks, all audited (below) |
| [`num-traits`](https://crates.io/crates/num-traits) | `=0.2.19` | MIT OR Apache-2.0 | Not `forbid(unsafe_code)`; 1 `unsafe` block, audited (below) — direct dependency, used only for `BigInt::to_i64` (the `Int`-fast-path narrowing check) |
| `num-integer` (transitive, via `num-bigint`) | `0.1.46` | MIT OR Apache-2.0 | not a direct dependency; standard integer-trait glue, no `unsafe` |

**Both crates are permissive/non-copyleft (clean-room-compatible, `CLEAN-ROOM.md`)
and are the de-facto standard, widely-adopted Rust bignum/numeric-trait crates**
(the ADR 0009 "earned industry trust" criterion).

**Mechanism: exact-pinned + checksum-locked, not a physical vendor tree.**
This repo has no `vendor/` directory; the addition is reproducible and
re-checkable via the exact-version pins above (`=0.4.6`/`=0.2.19`, never a
range) plus `Cargo.lock`'s SHA-256 content hash for each crate, which
`cargo build --locked` (CI) verifies on every build.

### `unsafe`-status audit

**`num-bigint` 0.4.6** (12 blocks):
- `biguint/addition.rs` (2), `biguint/subtraction.rs` (2): safe wrappers
  around `core::arch` add-with-carry/sub-with-borrow intrinsics
  (`_addcarry_u64`/`_subborrow_u64` and the `u32` variants) — register-only,
  no memory access, `unsafe` only for FFI-intrinsic API consistency.
- `biguint/division.rs` (1): an inline-asm `div` instruction on
  x86/x86_64 — register-only (`RDX:RAX`/`EDX:EAX` in, quotient/remainder
  out), no memory touched, documented `SAFETY` comment in the crate.
- `bigint.rs` / `biguint.rs` `to_str_radix` (2): `String::from_utf8_unchecked`
  over a buffer built exclusively from ASCII radix-digit characters (`0-9`,
  `a-z`) — always valid UTF-8 by construction.
- `bigrand.rs` (1): behind the crate's `rand` feature, which this project
  does **not** enable (only the default `std` feature is active) — dead code
  in our build, not compiled.
- Remaining blocks: same two intrinsic-wrapper/inline-asm shapes as above,
  repeated per width class.

**`num-traits` 0.2.19** (1 block): `cast.rs`, a `to_int_unchecked` float→int
cast used only by the crate's `ToPrimitive` impls for `f32`/`f64`. F1 only
calls `BigInt::to_i64()` (an integer→integer path); the float-cast arm is
never exercised by this addition.

**Verdict: no unaudited `unsafe` on any path F1 exercises.**

### Scope note

F1 adds **nothing** to `trusted_base()` and touches **no** `ken-kernel` file
(`spec/10-kernel/18a §5.2.1(5)`) — this is a tier-b tested-not-trusted
addition to the interpreter's outer ring, netted by the independent
differential oracle (`conformance/surface/numbers/seed-f1-bignum-int.md` AC2),
not a kernel-trusted dependency.

## PX2 — generated Target-ABI manifest (`ken-host` build only)

PX2 adds two exact-pinned build tools and reuses `linux-raw-sys = 0.12.1`
directly at build time. The new `cc` and `sha2` closures are not linked into
Ken's runtime artifact; `linux-raw-sys` remains in PX1's existing runtime
closure through `rustix`:

- `cc = 1.2.41`
  (`ac9fe6cdbb24b6ade63616c0a0688e45bb56732262c158df3c0c4bea4ca47cb7`)
  selects the target-qualified C compiler for the system-header observer. The
  observer is run only when host and target identities match; otherwise the
  manifest records an unavailable backend.
- `sha2 = 0.10.9`
  (`a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283`)
  hashes the canonical generated manifest payload with SHA-256.
- `linux-raw-sys = 0.12.1` keeps its PX1 checksum and permissive license. Its
  build-only features are exactly `std`, `general`, and `errno`; it supplies
  the generated numeric side of the fact-by-fact comparison and remains the
  runtime binding source through `rustix`.

The lockfile gains ten packages for the two new build tools: `cc`,
`find-msvc-tools`, `shlex`, `sha2`, `digest`, `block-buffer`, `crypto-common`,
`generic-array`, `typenum`, and `cpufeatures`. `cfg-if`, `libc`, and
`version_check` were already locked. This is build-time tooling only; PX1's
target-selected production closure remains N=3.

## Kernel-native Int literal (`ken-kernel`) — Int-decidable-equality
## value-reduction (ADR 0013 Layer 2)

`Term::IntLit(BigInt)` (`crates/ken-kernel/src/term.rs`) is the kernel-native
representation a checked `Int` literal reduces to; `obs.rs::eq_reduce` decides
`Eq Int (IntLit m) (IntLit n)` by BigInt value equality. This is the kernel
**binary's own first external dependency** — distinct from, and in addition
to, F1's `ken-interp`-layer trust above; `ken-kernel/Cargo.toml`'s
`[dependencies]` section was empty before this addition.

| crate | version (pinned) | license | `unsafe` status |
|---|---|---|---|
| [`num-bigint`](https://crates.io/crates/num-bigint) | `=0.4.6` | MIT OR Apache-2.0 | same crate/version as the F1 entry above — see that entry's `unsafe`-status audit, unchanged (no new code paths are exercised: this addition only calls `BigInt`'s `PartialEq`/`Clone`/`Hash`/`Debug`, none of which touch the audited `unsafe` blocks, which are all in arithmetic/division/radix-formatting paths this addition never calls) |

**Same exact pin as F1, reused, not a second vetting.** `num-bigint =0.4.6`
was already curated (ADR 0009) and audited above for `ken-interp`; this entry
records that the SAME crate/version now also enters `ken-kernel`'s compile-time
trusted computing base, per "small, auditable TCB" — honest-boundary, not
silent, even though no new vetting is required (identical crate, identical
version, and this addition's usage surface — value equality/clone/hash/debug
of `BigInt` — is a strict subset of what F1 already audited).

**Scope note:** unlike F1, this addition DOES touch `ken-kernel` (that is the
point — a kernel-native literal, not an interpreter-only value) but adds
**nothing** to `trusted_base()`: `Term::IntLit` is a term variant (a value
former), never a `Decl`, so no `trusted_base()` filter path reaches it, and
the BigInt-equality decision it enables is a kernel *decision* (structurally
analogous to constructor no-confusion), not a trusted assumption.

## Elaborator `IntLit` emission (`ken-elaborator`) — ADR 0013 Layer 2
## fast-follow

`elab_num_lit_infer`/`elab_num_lit_checked` (`crates/ken-elaborator/src/
elab.rs`) construct `Term::IntLit(BigInt)` directly for `Int` literals, so
`num-bigint` moved from `ken-elaborator/Cargo.toml`'s `[dev-dependencies]`
(added for the `checked_core.rs` round-trip test, prior entry above did not
cover this) to a real `[dependencies]` entry — production elaborator code
now constructs `BigInt` values, not just tests.

| crate | version (pinned) | license | `unsafe` status |
|---|---|---|---|
| [`num-bigint`](https://crates.io/crates/num-bigint) | `=0.4.6` | MIT OR Apache-2.0 | same crate/version as the F1/`ken-kernel` entries above — this addition's usage surface (`BigInt::from(i128)` construction, `Clone`/`PartialEq`) is the same construction/comparison subset already audited, no new code paths |

**Same exact pin, reused, no new vetting.** Third entry point for the
identical already-curated `num-bigint =0.4.6`, recorded per "small, auditable
TCB" for completeness of the accounting, not because this crossing is itself
TCB-critical: the elaborator is untrusted (its output is re-checked by the
kernel on every declaration), so this addition's trust weight is far lighter
than the `ken-kernel` entry above — but it's still an honest line, not a
silent one.

**Scope note:** adds nothing to `trusted_base()` (same reasoning as the
`ken-kernel` entry — `Term::IntLit` is a value former, not a `Decl`); the
`Cargo.lock` delta is empty (the crate was already resolved into the
workspace's dependency graph from `ken-kernel`'s and `ken-elaborator`'s own
prior dev-dependency use, so promoting it to a regular dependency changes
no resolved version).
