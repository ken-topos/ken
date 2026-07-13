# Bytes, I/O, and the FFI

> Status: **§1 impl-ready (L6); §2–§4 impl-ready (L7)** — elaborated to
> team-ready rigor; normative for the *trust and effect discipline*. **§1
> (`Bytes` + binary I/O, L6)** is built against the **landed** kernel/runtime
> (`14 §5` primitives, `41 §3a` encoding, `36 §1` effect rows). **§2–§4 (the
> `foreign` FFI + the trust boundary, L7)** are elaborated below: the `foreign`
> declaration + C-ABI marshalling (§2), the trust-boundary discipline
> (foreign-as-listed-postulate, `pure`-as-claim, runtime contracts at the edge,
> mandatory effects — §3), and capability gating (§4). `Bytes`, binary I/O, and
> the FFI are how a verified core meets the outside world; the **boundary
> discipline** matters more than the syntax.
>
> **L6 grounding (perishable-frame reconcile):** `Bytes` is the landed `41 §3a`
> content-addressed kind tag `0x05` (an interned compound, `41 §5`); its
> addressing is FNV-1a + `memcmp` (`41 §3`), **not** the serialization/Merkle
> hash (`§1.5`). Effect tracking rides L5's row system (`36 §1`); the one kernel
> admission L5's effect *denotation* needed — W-style (Π-bound) recursive
> inductives for the `ITree` `Vis` node (`14 §8.4`) — **has landed in K1.5**
> (`check_no_pi_bound_recursive` retired, `crates/ken-kernel/src/inductive.rs`),
> so **L6 carries no kernel-staging block** and adds **no new kernel rule**.
>
> **L7 grounding (perishable-frame reconcile):** a `foreign` rides the
> **landed** postulate machinery — `declare_postulate` → opaque constant (`11
> §4`: "how axioms, **FFI signatures**, and abstract interfaces are
> represented") recorded in `trusted_base()` (`18 §4.2`/`§5`) — and the
> **landed** B1 export, which projects every `trusted_base_delta` postulate
> (`25 §3`) into its `P` (assumptions) field under the boundary-label case
> (`71 §2.1`). Marshalling reuses L6's `Bytes`↔`(ptr,len)` (`§1.1`, `41 §1`);
> effects ride the `36 §1.4`
> escape check; capability gating couples Sec2 (`62`). **L7 adds no new kernel
> rule** — `foreign` is an existing postulate, not a new former. Pin against the
> landed code, not this line.

## 1. `Bytes` and binary I/O

`Bytes` is an **immutable, finite byte sequence** — the substrate for binary
protocols, hashing, serialization, and (in L7) FFI buffers. At the world
boundary, `read_bytes` reads bytes from files and `print_line` writes text to
the console. Every such operation is **effect-tracked** (`36`): an I/O
operation that does not carry its effect row is a **type error**. Text is
**never** an implicit reinterpretation of bytes — a
`String` is obtained from `Bytes` only through an **explicit, named**
`bytes_encode`/`bytes_decode` step (no hidden charset). Over that substrate sits
a **lawful serialization round-trip** (`§1.5`), the verified-component target
for G6.

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
| `bytes_at` | `Bytes → Int → Option UInt8` | indexed byte; **partial** (invalid index ⇒ `None`) |
| `bytes_slice` | `Bytes → Int(start) → Int(len) → Option Bytes` | `None` on an invalid span |
| `concat` | `Bytes → Bytes → Bytes` | total; `++` on `Bytes` |
| `empty` | `Bytes` | the zero-length value; `length empty ≡ 0` |

- **Partiality follows the `35 §3` / `43 §2` discipline.** Indexing and slicing
  are the **partial** operations (bounds), handled the same way `35` handles
  `div`/`mod` by zero. `bytes_at b i` returns `None` when `i` is negative or
  outside `b`. `bytes_slice b start len` returns `None` when `start` or `len`
  is negative, or when that span extends past `b`; its third argument is a
  **length**, not an end offset. Neither operation returns a neutral or a
  fabricated byte/empty slice for invalid bounds. For verified code, an
  **obligation-generating** total form over a refinement (`at_pf : (b : Bytes)
  → (i : Int) → { i ≥ 0 ∧ i < length b } → UInt8`, `34 §5`) is proven in-range
  ⇒ total and safe; unproven ⇒ a marked partial point that degrades to a runtime
  check (`unknown`/panic), **never** a silent out-of-bounds read. The two faces
  are the §`35 §3` "checked is the runtime face of an undischarged obligation,"
  not a separate mode.
- **A byte is `UInt8`** (`35`); `length`/indices are `Int` (the default integer,
  arbitrary-precision, `35 §2`).
- **Non-definitional laws are propositions** (`14 §5`), not assumed:
  `length (concat a b) == length a + length b`, `concat`-associativity,
  `concat a empty == a`, etc. — proved in the prelude (`50-stdlib/`,
  `20-verification/`), not baked into the kernel.
- **Exact surface spellings.** The primitive names `bytes_at` and
  `bytes_slice`, their signatures, their total failure behavior, and their
  registered reductions over literals are fixed here. Spellings of the
  remaining prelude conveniences (`++` vs `concat`, for example) remain a
  **`31`/prelude naming** detail (oracle-tagged for the build team).

### 1.3 Effect-tracked I/O

Every I/O operation carries its **exact effect row** (`visits`, `36 §1`). The
two landed, ordinary surface witnesses used by this clause are:

| Landed operation | Mandatory row |
|---|---|
| `read_bytes` | `visits [FS]` |
| `print_line` | `visits [Console]` |

Their complete result and capability types are defined by the landed FS and
Console floors; this clause fixes the operation-to-row association. Ken has no
landed `Net` operation in this round. A `Net`-specific operation-binding witness
is deferred until a real `Net` producer lands; conformance must not synthesize
one merely to preserve an old oracle.

- **The no-untracked-I/O guard is the `36 §1.4` escape check — L6 introduces no
  new gate.** Each I/O operation is a `perform_E` site (`36 §1.2`): it is the
  **one** place its label (`FS`/`Console`) enters the inferred row. A call to an
  I/O operation from a context whose declared row does **not** contain that label is
  an **EFFECT-ESCAPE static error** (`ρ_inf ⊄ ρ_decl`). Pure-by-default ⇒ a
  `view` with **no** `visits` has `ρ_decl = ∅`, so **any** I/O call escapes ⇒
  **untracked I/O is a compile error** (AC2/AC3). The accepting case carries the
  effect in its row; the rejecting case drops it — a **verdict flip**, exercised
  with the **≥2 distinct effects** `FS` and `Console` (`36 §1.4`).
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

#### 1.3.1 Typed FS capability API

Programs use a typed capability API over the unchanged, authority-polymorphic
FS producers. The coarse-v1 signatures are:

```ken
readFile : Cap APartial -> Bytes -> FS APartial (Result FileError Bytes)

writeFile : Cap AFull -> Bytes -> CreatePolicy -> Bytes
  -> FS AFull (Result FileError Unit)

attenuate : Cap AFull -> Cap APartial
```

`readFile` wraps `read_bytes`; `writeFile` wraps `write_file`. The wrappers do
not re-type or replace those raw producers. They make authority-level
sufficiency static at the program-facing API: a program holding only
`Cap APartial` cannot apply `writeFile`, so the attempt is ill-typed before the
driver runs. The driver's `CapabilityDenied` result remains a defense-in-depth
backstop rather than the primary program-validity gate.

The coarse-v1 `attenuate` is the Ken-callable `AFull`-to-`APartial`
specialization of the monotone-downward operation in `62 §3`. A read-and-write
program calls `readFile (attenuate fullCap) path`; it never passes its stronger
token directly to the read wrapper. This adds one explicit step but preserves
the fixed authority requirement in each operation's type and makes the
least-authority transition visible to a reader. There is no inverse operation
and no public capability constructor. The wrapper and attenuation function are
not new trusted machinery: the wrappers are ordinary checked Ken definitions,
and the exposed attenuation operation retains `62 §3`'s kernel-re-checked
downward-bound obligation. Neither adds a kernel rule or trusted primitive.

### 1.4 Text is explicit `bytes_encode` / `bytes_decode` — no hidden charset

A `String` is obtained from `Bytes` **only** through a named, visible boundary;
there is **no** implicit charset reinterpretation anywhere in the surface
(AC4).

```
bytes_encode : String → Bytes
bytes_decode : Bytes → Result Utf8Error String
```

- **`bytes_encode` is total and UTF-8 by contract.** It serializes a `String`
  (which is **NFC-normalized UTF-8** by construction, `41 §3a`) to its UTF-8
  bytes. The **charset is named in the operation**, not hidden: a non-UTF-8
  codec (if ever added) is a **different named function** (`encode_latin1`,
  …), never an implicit reinterpretation. There is no `Bytes`-to-`String`
  coercion, no "default charset," and no path that yields text from bytes
  without `bytes_decode`.
- **`bytes_decode` is total with explicit failure.** For valid UTF-8 it returns
  `Ok s`; invalid UTF-8 returns `Err e` for an `e : Utf8Error`. It never returns
  a neutral term or silently substitutes text. AC4's negative face: the
  **only** way to a `String` is this named, fail-visible step; an implicit or
  hidden-charset path is **rejected** (does not exist).

### 1.5 The serialization round-trip law

Over the `bytes_encode`/`bytes_decode` boundary, `BytesRoundTripLaw` is the
**one-directional** round-trip law, **provable** against `20-verification/`.
Its byte input is explicitly conditional on having been produced by
`bytes_encode`:

```
BytesRoundTripLaw :=
  ∀ (s : String). bytes_decode (bytes_encode s) == Ok s
```

- **Why it holds (and the direction matters).** `bytes_encode s` is the UTF-8
  bytes of `s`; `bytes_decode` parses valid UTF-8 (which `bytes_encode` always
  produces) and **re-constructs** a `String`, NFC-normalizing at construction
  (`41 §3a`). Because `s` is **already** NFC and NFC is **idempotent**, the
  reconstructed string equals `s` — so the law holds. The proof obligation is
  **dischargeable** (AC5 asserts the obligation is provable — a verified-
  component target — not merely that one sample round-trips; structural, per
  the untrusted-layer lesson).
- **The reverse is NOT a law — pin the silence so it is not over-claimed.**
  There is no unconditional inverse for arbitrary bytes: from
  `bytes_decode b == Ok s`, it does **not** follow in general that
  `bytes_encode s == b`. Invalid UTF-8 has no `String`, and even valid
  **non-NFC** bytes normalize on `String` construction (`41 §3a`), so
  `bytes_encode` after a successful `bytes_decode` is not the identity on every
  `Bytes`. Conformance must assert only the conditional
  `String → Bytes → Result Utf8Error String` direction above; a general
  `Bytes → String → Bytes` inverse is a **wrong** case (it would reject
  conforming implementations). This is a verdict/law-boundary silence resolved
  **at the source** so the conformance author does not fill it the other way.
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
check); the explicit **`bytes_encode`/`bytes_decode`** boundary (`§1.4`, no
hidden charset); and the **round-trip law** (`§1.5`, provable,
one-directional). **No new kernel
rule** (`§1.1`); **no `foreign`** (that is L7, `§2`–`§3`).

**Acceptance (AC1–AC5).**

- **AC1 — `Bytes` primitive + immutable.** A `b"…"`/`0x[…]` literal elaborates
  to the `Bytes` primitive (structural assertion on the elaborated value/type,
  not "compiles"); **no mutating op exists**.
- **AC2 — `[FS]` tracked.** `read_bytes` (`visits [FS]`) called from a context
  lacking `FS` is a **type error** (reject); the properly-rowed call **accepts**
  — a **verdict flip**.
- **AC3 — `[Console]` tracked.** `print_line` (`visits [Console]`) — the
  **same** flip on a **distinct**, real effect (the ≥2-effect discrimination of
  `36 §1.4`). A `Net` witness remains deferred until a real producer lands.
- **AC4 — no hidden charset.** Producing text from `Bytes` **requires** the
  named `bytes_decode`; an implicit/hidden-charset path is **rejected** (or
  absent), and invalid input produces `Err`.
- **AC5 — round-trip law.**
  `bytes_decode (bytes_encode s) == Ok s` is **provable** (the obligation is
  dischargeable — structural), not merely sampled; the reverse is **not**
  asserted (`§1.5`).

**Conformance:** `../../conformance/surface/bytes-io/` — AC1–AC5 with per-case
**verdict/structural flip** and the **cross-case sweep** (the effect-tracking
cases `FS`/`Console` agree as one metatheory class). The **QA gate**: the
effect-tracking cases route a **real** I/O signature through the **actual** `36
§1.4` escape check (a real untracked call → real reject), not a synthetic flag.
(The L7 `foreign`/trust-boundary cases live separately under
`../../conformance/surface/ffi-io/`, `§5`.)

## 2. The FFI surface — the `foreign` declaration

A **`foreign`** declaration binds a Ken name to an external (C-ABI) symbol,
giving it a **Ken type** and a **declared effect row**. It is the one surface
form that crosses out of the checked language, so it is built to make that
crossing **typed, effect-rowed, and visible** — never an untyped escape hatch.

```
foreign c_sqrt : Float → Float
  = symbol "sqrt"  library "m"  pure

foreign os_write : Int32 → Bytes → Int  visits [FS]
  = symbol "write"  library "c"
```

### 2.1 The declaration form (grammar + elaboration entry)

The `foreign` form extends the V0/`33` declaration grammar:

```
foreign-decl ::= "foreign" ident ":" type effect-row? "=" foreign-body
foreign-body ::= "symbol" string ("library" string)? "pure"?
effect-row   ::= "visits" "[" label ("," label)* "]"      -- 36 §1
```

- A `foreign` decl carries four things: a Ken **type** `T`, a **declared effect
  row** `ρ` (`visits [...]`; `pure` is the surface spelling of the **empty row**
  `ρ = ∅`), a C **symbol** name, and an optional **library**.
- `pure` and a non-empty `visits ρ` are **mutually exclusive**: `pure ≡ visits
  []`. A decl with neither defaults to `pure` (empty row) — and is then subject
  to §3.4 (a world-touching symbol with no row is the named residual, not a
  silent pass).
- **Spelling deferral (`OQ-syntax`, defer-spelling-not-concept).** The exact
  surface tokens (`foreign`/`symbol`/`library`/`pure`/`visits`) are reserved
  keywords whose literal spelling the build team finalizes; what is **locked
  here** is the **structure** (type + row + symbol + library), the **type+row
  obligation**, and the **postulate wiring** (§2.3). Conformance pins the
  structure and `(oracle)`-tags the literal keyword (assert-at-locked-
  granularity).

### 2.2 C-ABI marshalling (reuses L6)

A call to a `foreign` marshals Ken values to/from their C-ABI representations
following the primitive lowering (`41 §1`):

| Ken type | C-ABI representation | Source |
|---|---|---|
| scalars `Int*`/`UInt*`/`Float*`/`Bool`/`Char` | the named machine type (`i64`/`f64`/`i1`/`u32`/…) | `41 §1` typed immediates |
| `Bytes` | **`(ptr, len)`** — a pointer + length pair | L6 `§1.1`, `41 §1` |
| handle / `section` | tagged pointer / slot-id (transport convention) | `41 §1`/`§3`, `44` |

- `Bytes` ↔ `(ptr, len)` is the **L6 boundary** (`§1.1`) — L7 **rides it, does
  not re-derive** (guardrail). L6 keeps that boundary clean and explicit
  precisely so this marshalling needs no rework.
- Marshalling is a **runtime-lowering** concern (`41`), **not a kernel rule**:
  the kernel sees only the `foreign`'s assumed Ken type (§2.3); the (ptr,len) /
  scalar lowering happens at the call in the runtime, below the trusted
  boundary.
- The FFI is **general** — any C-ABI symbol — **not** a fixed allowlist of
  externals. Generality is safe because every use is an explicit, *listed*
  concession (§3).

### 2.3 Elaboration — a `foreign` is a postulate

A `foreign f : T visits ρ = symbol "…"` elaborates to a **postulate** of the
rowed Ken type. The defensive elaboration entry:

```
elabForeign(Σ, ⟨ foreign f : T visits ρ = symbol s library l [pure] ⟩):
  T'  := elabType(·, rowed(T, ρ))      -- the latent-row type A →[ρ] B (36 §1); pure ⇒ ρ=∅
  id  := declare_postulate(Σ, [], T')  -- opaque constant (11 §4); enters trusted_base() (18 §4.2/§5)
  recordForeign(id, symbol = s, library = l, row = ρ)   -- elaborator-side link/marshalling record
  bind f ↦ id
```

- The kernel admits `f` as an **opaque constant** `f : T'` (`11 §4`: an opaque
  constant blocks δ and "is how axioms, **FFI signatures**, and abstract
  interfaces are represented") — there is **no body to unfold**, and it is
  **recorded in `trusted_base()`** (`18 §4.2`, the `declare_postulate` contract:
  "`id` admitted opaque; recorded in the trusted base").
- **No new kernel rule** (guardrail). `declare_postulate` is the **existing**
  kernel API (`18 §4.1`); the type+row is ordinary `36 §1` machinery; the
  symbol/library/marshalling is an **elaborator-side** record consulted by the
  runtime, never by the kernel. L7 adds **zero** kernel formation rules (the
  level-discipline reconcile is therefore trivially a pass — no new universe or
  former appears).

## 3. The trust boundary (the load-bearing part)

FFI is where Ken's guarantees **stop**. The discipline marks that frontier
**honestly and structurally**: every foreign is assumed, *listed*, effect-rowed,
and capability-gated — and the one soundness gap the type system cannot
mechanically close is **named** (§3.4), never hidden.

### 3.1 Foreign-as-listed-postulate → the trusted base (the honesty headline)

This is the load-bearing AC2 mechanism — the honesty-about-the-boundary
principle made structural.

- A `foreign` postulate (§2.3) is in `trusted_base()` (`18 §5`).
- A verified **artifact's** `trusted_base_delta` (`25 §3`) is the set of
  `trusted_base()` postulates its checked content **transitively depends on** —
  its **dependency cone**. The B1 export (`71`) computes it from
  `trusted_base()` membership over the artifact's verified content.
- **Resolve the silence — "relies on" is by *use*, not *declaration*.** A
  `foreign` decl in scope is **not** itself a reliance; what lists `os_write` in
  an artifact's delta is a verified definition that **calls** `os_write`. A
  `view` that calls it has `os_write` in its delta; a `view` that does not is
  **absent** from it — even with the `foreign` decl visible in the same module.
  The honesty property ranges over **the foreign functions the verified content
  reaches**, exactly as `18 §5` reads `trusted_base()` ("the assumptions a given
  program *depends on*") and `71 §2.1` reads the boundary ("which foreign
  functions it *relies on*"). (Pinning this at the source so the conformance
  author nets reliance-by-call, not the vacuous decl-lists-itself.)
- The B1 export projects every `trusted_base_delta` postulate into the **`P`
  (assumptions)** field, tagged `tested`, under the **FFI boundary-label** case
  (`71 §2.1` names "FFI" among the boundary labels feeding `P`; invariant I2,
  `71 §3.1`). So **a verified artifact's exported contract lists exactly the
  foreign functions it relies on — visible, not hidden.**
- **AC2 flips on dependency:** relied-on ⇒ the foreign's postulate is a `P`
  entry; not-relied-on ⇒ absent (and the export hash changes, `71 §3.3`).
  Conformance routes a **real** `foreign` decl through the **real** B1 export
  and observes the foreign in `P` — never a synthetic trust-base literal. This
  is the **FFI instance** of B1's assumption-visibility (subsumes
  `export/removing-assume-shrinks-P`, `71 §3.1` I2 — subsume-don't-proliferate).

### 3.2 `pure` is a claim, not a check — projects to *trusted*, never `Q`

The kernel **cannot check C**. A `foreign`'s type — including a `pure`
(empty-row) claim of referential transparency — is **assumed**, never verified;
it is part of the trusted boundary, and is the AC3 mechanism.

- **The no-over-claim projection (`71` I1).** A `foreign`'s assumed guarantee is
  a **postulate** of its type, so its goal **is** a member of `trusted_base()`.
  By the structural discriminator (`21 §5.4`, `71 §2.1`: a claim is `Q` **iff**
  its certificate `check`s **and** its goal is **not** a `trusted_base()`
  postulate) it can **never** project to `Q`. A `pure foreign`'s guarantee lands
  in **`P`/trusted** (`tested`), **never** kernel-certified `Q`. **Under-claim
  is the safe direction**: filing the assumed claim as `Q` would over-claim a
  kernel certification that does not exist.
- This is the FFI instance of the **trusted-by-typing-is-not-`Q`** discipline
  (cf. Sec1's declassify edge, B1's I1): a guarantee resting on a **trusted
  meta-claim** ("the C symbol is referentially transparent") is **not
  kernel-proved**, so it projects to *trusted*, not `Q`.
- A **wrong `pure`** is a soundness bug **confined to that postulate — and it is
  *listed*** (§3.1): even an incorrect purity claim cannot masquerade as proved,
  and the assumption itself is visible in `P`.
- **Discriminating, not green-vs-green.** *Nothing* about a foreign is ever in
  `Q`, so "the foreign is absent from `Q`" alone is vacuous. The net is the
  **pair on the same artifact**: a genuinely kernel-proved Ken-side
  postcondition (a `view` wrapping the foreign, whose `ensures` over the
  *marshalling* — not the C body — discharges) projects to **`Q`** **while** the
  foreign's assumed `pure`/type claim projects to **`P`** — the field tracks
  **kernel-provedness**, not the `pure` keyword. The bug (trust the `pure`
  annotation, bucket it as a guarantee) lands the assumption in `Q` → red.
  (Reuses B1 EX-A's proved↔assumed pair, here
  `kernel-proved`↔`foreign-assumed`.)

### 3.3 Boundary contracts → runtime-checked assertions

A `requires`/`ensures` on a `foreign` is a proposition over an **opaque**
postulate (the AC4 mechanism).

- Where it is **statically unprovable** (the kernel has no body to reason
  about), it lowers to the **`tested`** epistemic status (`21 §5.2`): a
  **runtime-checked, fail-fast assertion** at the call boundary, **plus** a
  visible `P`/`tested` entry in the assumption boundary. This is **the** place
  runtime contracts earn their keep (`21 §5.2` names exactly "boundaries, FFI,
  untrusted input"): an unverifiable boundary becomes a **fail-fast** one rather
  than a silently-assumed one.
- **Structural, not "accepts."** Conformance observes the **emitted runtime
  check** (the lowered assertion exists at the call site) + the `tested` `P`
  entry — never merely that the program compiles (the discriminating-output
  discipline). A `foreign` `ensures` that **is** statically discharged (proved
  over the Ken-side marshalling) needs no runtime check and may reach `Q`
  (§3.2); the **unprovable** one **must** emit the runtime check — the two faces
  flip on static provability.

### 3.4 Effects are mandatory — and the one residual the discipline cannot catch

The AC5 mechanism: a catchable flip plus the single named gap (the honest limit,
`64`).

- **The catchable flip (the net).** A world-touching `foreign` carries its
  effect row (§2.1). A caller that performs the foreign's effect **without**
  declaring the matching row in its own signature is an **EFFECT-ESCAPE** static
  error (`36 §1.4`: `ρ_inf ⊄ ρ_decl`) — the **same** escape check L6 I/O rides
  (`§1.3`), **not** a new gate. A `view` calling `os_write` (`visits [FS]`)
  without `[FS]` in its declared row is **rejected**; with `[FS]`, it
  **accepts** — a **verdict flip** on a **real** foreign through the **real**
  escape check.
- **The residual (the honest limit).** A `foreign` declared **`pure`** (empty
  row) whose C symbol **actually performs I/O** is the **one** gap the type
  discipline **cannot mechanically catch**: the kernel sees an empty row, no
  caller is forced to declare an effect, and the real I/O is invisible to
  typing. This is a **reviewer-surfaced flag**, **not** a verdict the type
  system flips. It is **mitigated, not eliminated**, by §3.1 — the mis-declared
  foreign is still a *listed* postulate, so the wrong claim is at least
  **visible** in `P`. Name it as the residual; **never** silently treat a
  `pure`-but-effectful foreign as sound (the conformance case asserts it is
  *flagged*, not accepted as a netted verdict).

## 4. Capabilities and least authority (couples Sec2)

A `foreign` world-action requires **two independent concessions**, and dropping
**either** rejects (the AC6 composition with Sec1/Sec2):

1. its **effect row** (`36 §1.4`, §3.4) — *may this code perform this effect*;
   and
2. the **gating capability** — a `Cap_FS`/`Cap_Net`/… token passed at the call
   (`using c : Cap E`, `36 §3`; the authority face, Sec2 `62`) — *is this code
   authorized to perform it*.

- The capability gate is **Sec2's** discipline (`62`): no ambient authority,
  least by default, the cap a real Π value the call needs (`36 §2.5`). A
  `foreign os_write` call in a context holding **no** `Cap_FS` is rejected
  **even if** its `[FS]` row is declared (the authority is missing); a context
  with `Cap_FS` but **no** `[FS]` row is rejected by the escape check (§3.4).
  **Both** are required — the composition keeps the unverified surface of any
  program **explicit and minimal**.
- **The trusted boundary is a small, enumerated set** (guardrail): the verified
  core stays pure; every FFI use is a *listed* foreign postulate (§3.1) **plus**
  an explicit capability — never an ambient escape.

## 5. Honest limits — kernel-checked vs trusted vs the named residual

Where Ken's guarantee sits, per boundary fact — the honest accounting (`64`):

| Boundary fact | Status | Netted by |
|---|---|---|
| The foreign's **Ken type** (arg/result shape, effect row) | **assumed** (postulate, `11 §4`) | listed in `trusted_base_delta` → `P` (§3.1) |
| A **statically-proved** Ken-side contract over the marshalling | **`Q`** (kernel-certified) | kernel `check` (`18 §4.5`) |
| A **statically-unprovable** `requires`/`ensures` on the foreign | **`tested`** | runtime-checked assertion (§3.3) + `P` |
| The **`pure`** / referential-transparency claim | **trusted**, never `Q` (§3.2) | listed; never kernel-certified |
| The **effect row** on the foreign, at call sites | **enforced** | `36 §1.4` escape check (§3.4) — the flip |
| A **`pure`-but-effectful** foreign | **the residual** | reviewer flag only (§3.4) — listed, not caught |

The single honest gap is the last row: the discipline **names** it but cannot
mechanically close it. Everything else is either kernel-checked (`Q`),
runtime-checked (`tested`), or **listed-as-assumed** (`P`) — never hidden. The
goal of the whole chapter is that the unverified surface of any program is
**small, explicit, and enumerable** — not zero (FFI is necessary), but *honest*.

## 6. What WS-L must deliver here (L6, L7)

`Bytes` + binary I/O (effect-tracked, encode/decode to `String`) — **elaborated
in §1, the L6 deliverable**; lawful, derivable serialization with a provable
round-trip (the **law/interface in §1.5**; the generic derivation is L8). And
(**L7**, §2–§5): a **general** `foreign` FFI with typed/effect-rowed bindings
and C-ABI marshalling (§2); the trust-boundary discipline — foreign-as-listed-
postulate (§3.1), `pure`-as-claim-not-`Q` (§3.2), runtime contracts at the edge
(§3.3), mandatory effects + the named residual (§3.4); and capability gating
(§4). **No new kernel rule** (§2.3).

**L7 acceptance (AC1–AC5; ties to G6** — ≥1 FFI call in a verified component,
with the trust base showing exactly what is assumed):

- **AC1 — `foreign` binds + marshals.** A `foreign` decl elaborates to a typed,
  effect-rowed postulate (§2.3); a call marshals `Bytes`↔`(ptr,len)` + scalars↔
  machine types (§2.2) — a **structural** assertion on the binding/marshalling,
  not "compiles".
- **AC2 — foreign-as-listed-postulate (honesty headline, discriminating).** An
  artifact that **relies on** `foreign f` (calls it) has `f`'s postulate in its
  `trusted_base_delta` → `P`; one that does **not** is **absent** (§3.1). Routed
  through the **real** B1 export; the verdict flips on **dependency**.
- **AC3 — `pure` projects to *trusted*, never `Q`.** A `pure foreign`'s
  guarantee is assumed → `P`/`tested`, **never** `Q` (§3.2); under-claim is the
  safe direction. A genuinely-proved Ken-side claim in the *same* artifact
  reaches `Q` — the field tracks kernel-provedness, not the `pure` keyword.
- **AC4 — boundary contracts runtime-checked.** A statically-unprovable
  `requires`/`ensures` on a `foreign` lowers to a **runtime-checked** fail-fast
  assertion + a `tested` `P` entry (§3.3) — observe the **emitted** check, not a
  silent assumption.
- **AC5 — effects mandatory (discriminating).** A world-touching `foreign` whose
  effect escapes the caller's declared row is **rejected** (`36 §1.4`); the
  properly-rowed call **accepts** — a verdict flip (§3.4). The **`pure`-but-
  effectful** case is the **named residual** (flagged, not silently accepted as
  sound).

Conformance:
- `../../conformance/surface/bytes-io/` — the L6 `Bytes`/I/O/encode-decode/
  round-trip cases (`§1.6`).
- `../../conformance/surface/ffi-io/` — (L7) AC1–AC5 with per-case verdict/
  structural flip, the **AC2 honesty pair** (relied-on listed / not-relied-on
  absent, through the real export) and the **AC3 pure-not-`Q` pair**, the **AC5
  effects-mandatory flip + the named residual**, the **capability+effect
  composition** (`§4`), and a **G6 serialization round-trip proof** in an
  FFI-using verified component (a `foreign` decl whose postulate shows up in
  `trusted_base_delta`). **QA gate:** AC2/AC3 route a **real** `foreign` through
  the **actual** export/trust-base machinery (postulate listed, never `Q`); no
  synthetic trust-base literal.
