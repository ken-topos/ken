# WP L6 — `Bytes` + binary I/O

**Owner:** Team Foundation (L-stream leaf, parallel to Language — operator-
assigned). **Branch:** `wp/L6-bytes-io` (cut from `origin/main`). **Stream /
gate:** L-stream → **G6**. **Depends on:** K1 (primitive-type machinery, `14
§5`)
— **merged**; L5 (effect rows for I/O tracking) — **merged**. **Spec source:**
`spec/30-surface/38-ffi-io.md §1` (+ `36` effects, `41` lowering, `31 §3`
literals).

> **Steward *frame*** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails; the spec enclave elaborates `38 §1` to team-ready
> rigor
> + conformance before Team Foundation builds. **Perishable:** pin `Bytes`'
> lowering to the **landed** `41`, not this line (K2c-series-2 stale-frame
> trap).

## 1. Objective (one line)

Deliver **`Bytes`** (an immutable, finite byte sequence as a kernel primitive) +
**effect-tracked binary I/O** (`read`/`write`/`send` over files/sockets/streams,
all `visits [...]`), with text I/O as `Bytes` + an **explicit** `String`
encode/decode (no hidden charset) and a **lawful round-trip** serialization
target.

## 2. Settled inputs — FIXED, do not reopen

Per `38 §1` (+ `36`, `41`):

1. **`Bytes` is a kernel primitive** (`14 §5`, `41`) — an **immutable, finite**
   byte sequence; literals `b"…"` / `0x[…]` (`31 §3`). The foundation for binary
   protocols, hashing, serialization, FFI buffers. **No new kernel rules** —
   primitive type with registered/audited reductions (the `14 §5` discipline,
   like the L1 numerics).
2. **All binary I/O is EFFECT-TRACKED** (`36`, riding L5's rows) — e.g.
   `read_bytes : Path → Bytes visits [FS]`, `send : Socket → Bytes → Unit visits
   [Net]`. **No untracked I/O** — every read/write/send carries its effect row.
3. **Text I/O = `Bytes` + an explicit `String` encode/decode — NO hidden
   charset.** Text is never an implicit reinterpretation of bytes; the
   encode/decode is a visible, named step.
4. **Serialization is a lawful round-trip facility over `Bytes`** — `decode
   (encode x) == Ok x` is **provable** (`20-verification/`), a natural
   verified-component target (G6). (The full derivable `encode`/`decode` for
   arbitrary types is **stdlib** (`L8`) — L6 provides the `Bytes` substrate +
   the
   round-trip *law/interface*, not the generic derivation.)
5. **FFI is OUT OF SCOPE — it is L7** (`38 §2/§3`: the `foreign` mechanism + the
   trust boundary). L6 stops at `Bytes` + binary I/O; do not build `foreign`.

## 3. Mandated deliverable outline (each item ends in an implementable choice)

Deliver in the surface/runtime + prelude:

1. **The `Bytes` primitive.** Pin its lowering to **landed `41`** (the
   byte-buffer
   representation — verify at pickup, don't guess), the `b"…"`/`0x[…]` literal
   forms (`31 §3`), immutability, and the core ops (length, index, slice,
   concat) as primitive/prelude — **no new kernel rule**.
2. **Effect-tracked binary I/O.** Pin the I/O surface — `read_bytes : Path →
   Bytes visits [FS]`, write/append, `send`/`recv : Socket → … visits [Net]`,
   stream ops — each with its **exact effect row** (L5). An I/O op with **no**
   effect annotation is a **type error** (the no-untracked-I/O guard).
3. **Text as explicit encode/decode.** Pin the `String ↔ Bytes` boundary as
   named total/partial functions (`encode : String → Bytes`, `decode : Bytes →
   Result String`) — **no implicit charset reinterpretation** anywhere.
4. **The round-trip law.** Pin the `decode (encode x) == Ok x` property as the
   serialization contract (the provable round-trip, `20-verification/`) — the
   *interface + law* live here; the generic derivation is the L8 follow-on.

## 4. Testable acceptance criteria

- **AC1 (`Bytes` primitive)** A `b"…"`/`0x[…]` literal elaborates to the `Bytes`
  primitive and is **immutable** (no mutating op exists); structural assertion
  on
  the elaborated value/type, not just "compiles".
- **AC2 (I/O is effect-tracked)** `read_bytes : Path → Bytes` **visits [FS]** —
  a
  call in a context lacking the `[FS]` capability is a **type error** (reject);
  the
  properly-tracked call accepts. *Verdict flips* (tracked accepts / untracked
  rejects) — not green-vs-green.
- **AC3 (`[Net]` tracked)** `send : Socket → Bytes → Unit` visits **[Net]** —
  same flip.
- **AC4 (no hidden charset)** Producing text from `Bytes` **requires** an
  explicit
  `decode` — an implicit/hidden-charset reinterpretation path is **rejected**
  (or
  does not exist). The named `decode` is partial (`Result`) for invalid input.
- **AC5 (round-trip law)** `decode (encode s) == Ok s` is **provable** — assert
  the obligation is dischargeable (a verified-component target), not merely that
  a sample round-trips (structural, per the untrusted-layer lesson).
- **Conformance:** `conformance/surface/bytes-io/` — AC1–AC5, per-case
  verdict/structural-flip + cross-case sweep (the effect-tracking class agrees).
  **QA gate:** the effect-tracking cases route a **real** I/O op through the
  **actual** effect-row check (a real untracked call → real reject), not a
  synthetic flag.

## 5. Do-not-reopen guardrails

- **`Bytes` is a kernel primitive** (`14 §5`) — no new kernel rule; ops are
  primitive/prelude (§2.1).
- **All I/O is effect-tracked** (`36`/L5) — no untracked read/write/send (§2.2).
- **No hidden charset** — text is always explicit `encode`/`decode` (§2.3).
- **FFI is L7, not L6** — do not build `foreign` or the trust boundary (§2.5).
- **Extend the landed `Bytes`/`41` representation** if any exists; verify at
  pickup (perishable-frame caveat).

## 6. Sequencing notes

- L6 is a **breadth-wave** WP (brings Team Foundation online, parallel to
  Language's L-stream). **L7 (FFI)** is the downstream follow-on (`38 §2/§3`,
  needs L6) — keep the `Bytes`-as-`(ptr,len)` marshalling boundary clean so L7
  rides it.
- The round-trip serialization law couples to the **verification** surface
  (`20-`) and the **stdlib** (`L8`, the generic derivation) — pin the law here,
  defer the derivation.
- Standard §2c: frame → spec-leader elaborates `38 §1` + conformance → merge
  (Architect + conformance-validator) → Team Foundation compacted, then kicked
  off.
