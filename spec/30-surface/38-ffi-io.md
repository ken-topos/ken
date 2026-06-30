# Bytes, I/O, and the FFI

> Status: **DRAFT v0** overall — proposal-level for syntax; normative for the
> *trust and effect discipline*. **§1 (`Bytes` + binary I/O) is impl-ready
> (L6)** — elaborated to team-ready rigor against the **landed** kernel/runtime
> (`14 §5` primitives, `41 §3a` encoding, `36 §1` effect rows); Team Foundation
> builds `Bytes` + binary I/O from §1. **§2–§4 (the `foreign` FFI + trust
> boundary) stay proposal-level — they are L7**, not elaborated here. `Bytes`,
> binary I/O, and the FFI are how a verified core meets the outside world; the
> **boundary discipline** matters more than the syntax.
>
> **L6 grounding (perishable-frame reconcile):** `Bytes` is the landed `41 §3a`
> content-addressed kind tag `0x05` (an interned compound, `41 §5`); its
> addressing is FNV-1a + `memcmp` (`41 §3`), **not** the serialization/Merkle
> hash (`§1.5`). Effect tracking rides L5's row system (`36 §1`); the one kernel
> admission L5's effect *denotation* needed — W-style (Π-bound) recursive
> inductives for the `ITree` `Vis` node (`14 §8.4`) — **has landed in K1.5**
> (`check_no_pi_bound_recursive` retired, `crates/ken-kernel/src/inductive.rs`),
> so **L6 carries no kernel-staging block** and adds **no new kernel rule**.

## 1. `Bytes` and binary I/O

`Bytes` is an **immutable, finite byte sequence** — the substrate for binary
protocols, hashing, serialization, and (in L7) FFI buffers. Binary I/O
reads/writes `Bytes` to files, sockets, and streams, and every such operation is
**effect-tracked** (`36`): an I/O operation that does not carry its effect row
is a **type error**. Text is **never** an implicit reinterpretation of bytes — a
`String` is obtained from `Bytes` only through an **explicit, named**
`encode`/`decode` step (no hidden charset). Over that substrate sits a **lawful
serialization round-trip** (`§1.5`), the verified-component target for G6.

### 1.1 The `Bytes` primitive

`Bytes` is a **primitive type** in the `14 §5` sense — an **opaque type
constant** with **no kernel-level constructors or eliminator**; it adds **no new
kernel rule** (guardrail). Its inhabitants are **literals** (introduced by the
elaborator as opaque primitive values) and the results of the primitive
operations (`§1.2`). This is the same discipline as the L1 numeric primitives
(`35`): the kernel gains an opaque constant plus a small, **audited** set of
registered reductions, listed in the trusted base (`18 §5`).

- **Immutable and finite.** A `Bytes` value never changes after construction;
  there is **no mutating operation** in the surface (the AC1 structural
  property). "Updating" bytes (e.g. `slice`, `concat`) **allocates a new value**
  and shares nothing observable with the old one — the `41 §2` append-mostly,
  immutable-heap discipline.
- **Runtime representation (landed `41`).** `Bytes` is a **content-addressed,
  interned compound** (`41 §5` table): stored once in the heap, keyed by the
  hash of its canonical bytes, with **O(1) structural equality** by slot-id
  (`41 §4`). Its canonical encoding (`41 §3a`) is the **kind tag `0x05`**
  followed by the raw byte sequence (length-determined); two `Bytes` values with
  identical content encode identically and share a slot. In-process addressing
  uses **FNV-1a + `memcmp`** (`41 §3`) — distinct from the cryptographic/Merkle
  hash used for serialization (`§1.5`); two hashes, two jobs.
- **The `(ptr, len)` boundary.** A `Bytes` value's machine face is a
  `(pointer, length)` pair (`38 §2` marshalling, `41 §1`). L6 keeps this
  boundary clean and explicit so **L7's** `foreign` marshalling (`Bytes` ↔ a C
  buffer) rides it without rework — but L6 builds **no** `foreign` mechanism.

**Literals (`31 §3`).** Two surface forms denote `Bytes`:

| Form | Example | Meaning |
|---|---|---|
| **byte string** | `b"GET / HTTP/1.1\r\n"` | a byte sequence from a string-like body with escapes (each `\xNN`/ASCII char ⇒ one byte) |
| **hex** | `0x[deadbeef]` | the bytes spelled as hex nibble pairs (`de ad be ef`) |

The **bracketed** `0x[…]` is the `Bytes` form; the **un-bracketed** `0xFF` is an
**`Int`** literal (`31 §3`, `35`) — the two are different tokens with different
types and must not be conflated. A `b"…"` literal elaborates **directly** to the
`Bytes` primitive (AC1), not via `String` (no decode round-trip at the literal).

### 1.2 Core `Bytes` operations (primitive / prelude)

The core operations are primitive operations with **registered reductions**
(`14 §5`) — they compute over literal `Bytes` **in the kernel's evaluator** (so
`length 0x[deadbeef] ≡ 4 : Int` holds definitionally and proofs reduce over
literals), and are neutral on stuck (non-literal) arguments. They introduce **no
new kernel rule** — only registered `prim` reductions, like `add` on `Int`.

| Op | Type | Notes |
|---|---|---|
| `length` | `Bytes → Int` | byte count; total, non-negative |
| `at` | `Bytes → Int → Option UInt8` | indexed byte; **partial** (out-of-range ⇒ `None`) |
| `slice` | `Bytes → Int → Int → Option Bytes` | `slice b start len`; `None` on an out-of-range window |
| `concat` | `Bytes → Bytes → Bytes` | total; `++` on `Bytes` |
| `empty` | `Bytes` | the zero-length value; `length empty ≡ 0` |

- **Partiality follows the `35 §3` / `43 §2` discipline.** Indexing and slicing
  are the **partial** operations (bounds), handled the same way `35` handles
  `div`/`mod` by zero: either an **explicit `Option`** result (shown above) or,
  for verified code, an **obligation-generating** total form over a refinement
  (`at_pf : (b : Bytes) → (i : Int) → { i ≥ 0 ∧ i < length b } → UInt8`, `34
  §5`) — proven in-range ⇒ total and safe; unproven ⇒ a marked partial point
  that degrades to a runtime check (`unknown`/panic), **never** a silent
  out-of-bounds read. The two faces are the §`35 §3` "checked is the runtime
  face of an undischarged obligation," not a separate mode.
- **A byte is `UInt8`** (`35`); `length`/indices are `Int` (the default integer,
  arbitrary-precision, `35 §2`).
- **Non-definitional laws are propositions** (`14 §5`), not assumed:
  `length (concat a b) == length a + length b`, `concat`-associativity,
  `concat a empty == a`, etc. — proved in the prelude (`50-stdlib/`,
  `20-verification/`), not baked into the kernel.
- **Exact surface spellings** of the prelude ops (`at` vs `get` vs `index`, `++`
  vs `concat`) are a **`31`/prelude naming** detail (oracle-tagged for the build
  team); the **signatures, totality/partiality treatment, and the registered
  reductions over literals** are fixed here.

### 1.3 Effect-tracked binary I/O

Every binary I/O operation carries its **exact effect row** (`visits`, `36 §1`).
The surface (signatures fixed; spellings oracle-tagged):

```
read_bytes  : Path   → Bytes            visits [FS]
write_bytes : Path   → Bytes → Unit     visits [FS]
append      : Path   → Bytes → Unit     visits [FS]
send        : Socket → Bytes → Unit     visits [Net]
recv        : Socket → Int   → Bytes    visits [Net]
```

Stream operations (open/read-chunk/write-chunk/close over a file or socket
handle) carry the **same** row as their underlying device (`[FS]` or `[Net]`).

- **The no-untracked-I/O guard is the `36 §1.4` escape check — L6 introduces no
  new gate.** Each I/O operation is a `perform_E` site (`36 §1.2`): it is the
  **one** place its label (`FS`/`Net`) enters the inferred row. A call to an I/O
  operation from a context whose declared row does **not** contain that label is
  an **EFFECT-ESCAPE static error** (`ρ_inf ⊄ ρ_decl`). Pure-by-default ⇒ a
  `view` with **no** `visits` has `ρ_decl = ∅`, so **any** I/O call escapes ⇒
  **untracked I/O is a compile error** (AC2/AC3). The accepting case carries the
  effect in its row; the rejecting case drops it — a **verdict flip**, exercised
  with the **≥2 distinct effects** `FS` and `Net` (`36 §1.4`).
- **Every I/O act is a `Vis` node (`36 §3.1` contract).** The I/O operations are
  the tree's `perform` sites: a function's effects are recoverable from its
  **type** (the latent row `A →[ρ] B`), never hidden between nodes. This is the
  interface Sec/B (`60-`/`70-`) read labels and capabilities off of — L6 must
  not perforate it (no effectful I/O that bypasses a `Vis` node).
- **I/O operations are not kernel primitives.** Unlike the **pure** `Bytes` ops
  (`§1.2`, which carry `14 §5` registered reductions and compute in the kernel),
  an I/O operation is an **effect-performing surface operation** whose
  denotation is an `ITree` `Vis` node (`36 §2`), **not** a kernel reduction — it
  computes no literal in the kernel and adds **nothing** to the TCB. The kernel
  admission this denotation needs (W-style recursive inductives) **landed in
  K1.5** (banner + `14 §8.4`).
- **Capabilities (`36 §3`) are the authority face of the same row.** An I/O
  operation may additionally take a capability token (`using c : Cap FS`) to
  express least authority; L6 specifies the **effect-row** obligation (the
  type-error-if-untracked guarantee) and leaves capability *threading* as the
  `36 §3` mechanism it already is — no new machinery here.

### 1.4 Text is explicit `encode` / `decode` — no hidden charset

A `String` is obtained from `Bytes` **only** through a named, visible boundary;
there is **no** implicit charset reinterpretation anywhere in the surface
(AC4).

```
encode : String → Bytes          -- total; UTF-8 serialization (the named charset)
decode : Bytes   → Result String  -- partial; UTF-8 parse, Err on invalid input
```

- **`encode` is total and UTF-8 by contract.** It serializes a `String` (which
  is **NFC-normalized UTF-8** by construction, `41 §3a`) to its UTF-8 bytes. The
  **charset is named in the operation**, not hidden: a non-UTF-8 codec (if ever
  added) is a **different named function** (`encode_latin1`, …), never an
  implicit reinterpretation. There is no `Bytes`-to-`String` coercion, no
  "default charset," and no path that yields text from bytes without a `decode`.
- **`decode` is partial (`Result`).** Not every `Bytes` is valid UTF-8, so
  `decode` returns `Err` on invalid input (`Result String`, `36`) — the partial,
  fail-visible boundary for untrusted bytes. AC4's negative face: the **only**
  way to a `String` is this named, partial step; an implicit/hidden-charset path
  is **rejected** (does not exist).

### 1.5 The serialization round-trip law

Over the `encode`/`decode` boundary, the **serialization contract** is the
**one-directional** round-trip law, **provable** against `20-verification/`:

```
∀ (s : String).  decode (encode s) == Ok s          -- (L6 law, provable; AC5)
```

- **Why it holds (and the direction matters).** `encode s` is the UTF-8 bytes of
  `s`; `decode` parses valid UTF-8 (which `encode` always produces) and
  **re-constructs** a `String`, NFC-normalizing at construction (`41 §3a`).
  Because `s` is **already** NFC and NFC is **idempotent**, the reconstructed
  string equals `s` — so the law holds. The proof obligation is
  **dischargeable** (AC5 asserts the obligation is provable — a verified-
  component target — not merely that one sample round-trips; structural, per the
  untrusted-layer lesson).
- **The reverse is NOT a law — pin the silence so it is not over-claimed.**
  `encode (decode b) == Ok b` does **not** hold for all `b : Bytes`: (1)
  `decode` is partial (invalid UTF-8 has no `String`), and (2) even valid
  **non-NFC** bytes normalize on `String` construction (`41 §3a`), so
  `encode ∘ decode` is
  **not** the identity on `Bytes`. Conformance must assert the law in the
  `String → Bytes → String` direction only; a `Bytes → String → Bytes` case is a
  **wrong** case (it would reject conforming implementations). This is a
  verdict/law-boundary silence resolved **at the source** so the conformance
  author does not fill it the other way.
- **L6 delivers the interface + the law; the generic derivation is L8.** The
  derivable `encode`/`decode` for **arbitrary** types (the `Serialize`-class
  machinery) is the stdlib follow-on (`L8`); L6 fixes the `Bytes` substrate, the
  `String` boundary, and the round-trip **law/interface** — not the generic
  derivation. The serialization/integrity hash referenced for verified transport
  (a cryptographic/Merkle hash, BLAKE3 recommendation, `41 §3b`) is **separate**
  from in-process addressing (`§1.1`) and is selected downstream, not by L6.

### 1.6 What L6 delivers + acceptance

Team Foundation delivers, in the surface + runtime + prelude: the **`Bytes`
primitive** (`§1.1`, `14 §5` opaque constant + `41 §3a` `0x05` encoding,
immutable, `b"…"`/`0x[…]` literals); the **core ops** (`§1.2`, registered
reductions, `35 §3` partiality); the **effect-tracked I/O surface** (`§1.3`,
each op `visits` its exact row, untracked = type error via the `36 §1.4` escape
check); the **explicit `encode`/`decode`** boundary (`§1.4`, no hidden charset);
and the **round-trip law** (`§1.5`, provable, one-directional). **No new kernel
rule** (`§1.1`); **no `foreign`** (that is L7, `§2`–`§3`).

**Acceptance (AC1–AC5).**

- **AC1 — `Bytes` primitive + immutable.** A `b"…"`/`0x[…]` literal elaborates
  to the `Bytes` primitive (structural assertion on the elaborated value/type,
  not "compiles"); **no mutating op exists**.
- **AC2 — `[FS]` tracked.** `read_bytes` (`visits [FS]`) called from a context
  lacking `FS` is a **type error** (reject); the properly-rowed call **accepts**
  — a **verdict flip**.
- **AC3 — `[Net]` tracked.** `send` (`visits [Net]`) — the **same** flip on a
  **distinct** effect (the ≥2-effect discrimination of `36 §1.4`).
- **AC4 — no hidden charset.** Producing text from `Bytes` **requires** the
  named `decode`; an implicit/hidden-charset path is **rejected** (or absent),
  and `decode` is **partial** (`Result`) on invalid input.
- **AC5 — round-trip law.** `decode (encode s) == Ok s` is **provable** (the
  obligation is dischargeable — structural), not merely sampled; the reverse is
  **not** asserted (`§1.5`).

**Conformance:** `../../conformance/surface/bytes-io/` — AC1–AC5 with per-case
**verdict/structural flip** and the **cross-case sweep** (the effect-tracking
cases `FS`/`Net` agree as one metatheory class). The **QA gate**: the
effect-tracking cases route a **real** I/O signature through the **actual** `36
§1.4` escape check (a real untracked call → real reject), not a synthetic flag.
(The L7 `foreign`/trust-boundary cases live separately under
`../../conformance/surface/ffi-io/`, `§5`.)

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
  machine types, `Bytes` as `(ptr, len)`, etc. A general C/BLAS FFI is the L7
  deliverable.
- The FFI is a **general** but **explicitly-trusted** mechanism (§3), not a
  fixed allowlist of externals.

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

`Bytes` + binary I/O (effect-tracked, encode/decode to `String`) — **elaborated
in §1, the L6 deliverable**; lawful, derivable serialization with a provable
round-trip (the **law/interface in §1.5**; the generic derivation is L8); a
**general** `foreign` FFI with typed/effect-rowed bindings and C-ABI marshalling
(**L7**, §2–§3); the trust-boundary discipline (foreign-as-listed-postulate,
`pure`-as-claim, runtime contracts at the edge — **L7**). Acceptance is part of
**G6** (≥1 FFI call in the verified component, with the trust base showing
exactly what is assumed). Conformance:
- `../../conformance/surface/bytes-io/` — the L6 `Bytes`/I/O/encode-decode/
  round-trip cases (`§1.6`).
- `../../conformance/surface/ffi-io/` — (L7) a serialization round-trip proof
  and a `foreign` decl whose postulate shows up in `trusted_base_delta`.
