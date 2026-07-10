# Dependency-delta records (`spec/60-security/63`, ADR 0009)

Per-addition records for external crates curated into the tool-chain (ADR
0009's "select an industry-trusted component" rubric step 1). Each entry
documents the vetting the merge Decision checked, so the addition to the
tool-chain's own trusted computing base is legible and re-checkable on update.

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
