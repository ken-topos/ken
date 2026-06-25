# Bytes, I/O, and the FFI

> Status: **DRAFT v0**. Proposal-level for syntax; normative for the *trust and
> effect discipline*. `Bytes`, binary I/O, and the foreign-function interface —
> three things the prototype genuinely lacks (general FFI; `Bytes`/binary I/O;
> the prototype's `File` is text-only — digest §7/§9). These are how a verified
> core meets the unverified outside world, so the **boundary discipline** matters
> more than the syntax.

## 1. `Bytes` and binary I/O

- **`Bytes`** — an immutable, finite byte sequence; a primitive type
  (`../10-kernel/14 §5`, `../40-runtime/41`). Literals `b"…"` / `0x[…]` (`31
  §3`). The foundation for binary protocols, hashing, serialization, and FFI
  buffers.
- **Binary I/O** — reading/writing `Bytes` to files, sockets, and streams, all
  **effect-tracked** (`36`): `read_bytes : Path → Bytes visits [FS]`, `send :
  Socket → Bytes → Unit visits [Net]`. Text I/O is `Bytes` + an explicit
  `String` encode/decode (no hidden charset).
- **Serialization** is a stdlib facility over `Bytes` (the prototype's
  `serialize`/`deserialize` + Merkle generalized): a derivable, *lawful*
  `encode`/`decode` with the round-trip property `decode (encode x) == Ok x`
  provable (`../20-verification/`) — a natural verified-component target (G6).

## 2. The FFI surface

A **`foreign`** declaration binds a Ken name to an external (C ABI) symbol:

```
foreign c_sqrt : Float → Float
  = symbol "sqrt"  library "m"  pure

foreign os_write : Int32 → Bytes → Int  visits [FS]
  = symbol "write"  library "c"
```

- A `foreign` decl gives the external function a **Ken type** and an **effect
  row** (`pure` ≡ empty row). Marshalling between Ken values and C ABI types
  follows the primitive lowering (`../40-runtime/41`): scalars pass as their
  machine types, `Bytes` as `(ptr, len)`, etc. The general C/BLAS FFI the
  prototype lacks (its externals are a fixed prefix-gated allowlist) is the L7
  deliverable.
- The FFI replaces the prototype's narrow allowlist with a **general** but
  **explicitly-trusted** mechanism (§3).

## 3. The trust boundary (the load-bearing part)

FFI and I/O are where Ken's guarantees stop and must be marked honestly:

- **A `foreign` function is a postulate** (`../10-kernel/11 §4`, `18 §5`): the
  kernel cannot check C code, so a `foreign` decl's type is *assumed*, and it
  appears in the **trusted base** / `trusted_base_delta` (`../20-verification/25
  §3`). A verified artifact's trust base therefore lists exactly which foreign
  functions (and which of their assumed contracts) it relies on — visible, not
  hidden.
- **`pure` on a `foreign` is a claim, not a check.** Declaring `c_sqrt` pure
  asserts referential transparency the kernel cannot verify; it is part of the
  trusted boundary and SHOULD be used only for genuinely pure externals. A wrong
  `pure` is a soundness bug confined to that postulate — and it is *listed*.
- **Contracts at the boundary.** `requires`/`ensures` on a `foreign` become
  **runtime-checked** assertions (`../20-verification/21 §5`) where statically
  unprovable — the place runtime contracts earn their keep (untrusted input, FFI
  results). This converts an unverifiable boundary into a fail-fast one.
- **Effects are mandatory at the boundary.** Any `foreign`/I/O that touches the
  world MUST carry the appropriate effect row (`36`); a `pure`-but-effectful
  foreign is the one thing the discipline cannot catch and the reviewer must.

## 4. Capabilities and least authority

I/O effects (`FS`, `Net`, …, `36 §3`) gate which foreign/IO functions a
computation may call; a function gets exactly the I/O capabilities its type
declares. This keeps the unverified surface area of any given program explicit
and minimal — the verified core stays pure, and the trusted boundary is a small,
enumerated set of `foreign` postulates + capabilities.

## 5. What WS-L must deliver here (L6, L7)

`Bytes` + binary I/O (effect-tracked, encode/decode to `String`); lawful,
derivable serialization with a provable round-trip; a **general** `foreign` FFI
with typed/effect-rowed bindings and C-ABI marshalling; the trust-boundary
discipline (foreign-as-listed-postulate, `pure`-as-claim, runtime contracts at
the edge). Acceptance is part of **G6** (≥1 FFI call in the verified component,
with the trust base showing exactly what is assumed). Conformance:
`../../conformance/surface/ffi-io/` — a serialization round-trip proof and a
`foreign` decl whose postulate shows up in `trusted_base_delta`.
