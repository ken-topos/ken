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
