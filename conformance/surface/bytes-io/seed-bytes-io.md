# L6 (`Bytes` + binary I/O) conformance — seed cases

Format: `../../README.md`. These pin the **L6 deliverable** (`docs/program/wp/
L6-bytes-io.md`, `spec/30-surface/38-ffi-io.md §1`): the **`Bytes` primitive**
(`14 §5` opaque constant, `41 §3a` kind tag `0x05`, immutable), its **core ops**
(registered reductions, `35 §3` partiality), **effect-tracked binary I/O** (each
op `visits` its exact row, untracked = type error), the **explicit `encode`/
`bytes_decode`** boundary (no hidden charset), and the **one-directional round-trip
law**. They extend — and must not regress — the on-`main` surface/effects
invariants (`../effects/seed-effects.md`, `../seed-surface.md`). **FFI/`foreign`
(`38 §2`–`§3`) is L7** and lives separately under `../ffi-io/`.

**CP0 safe-signature reconcile.** The coupled CP0 delivery contract
(`docs/program/wp/cp0-safe-bytes.md`, Deliverables B/C) fixes the formerly
loose core-operation seam to these exact shapes:

- `bytes_decode : Bytes → Result Utf8Error String`;
- `bytes_at : Bytes → Int → Option UInt8`;
- `bytes_slice : Bytes → Int(start) → Int(len) → Option Bytes`.

The cases below pin those names, result types, and invalid-input behavior. The
changed safe-signature outcomes are **RED-UNTIL-BUILT (CP0)** on the exact base;
they flip green only with the coupled implementation/spec/conformance landing.

**CP0 fixture re-anchor.** Retiring the placeholder `send`/`recv` primitives
removes the repository's sole registered `[Net]` producer. Conformance must not
manufacture one to preserve a historical test. The direct operation-row pair is
therefore re-anchored on the real landed producers `read_bytes : [FS]` and
`print_line : [Console]`; their declared rows live in `ElabEnv.effect_rows`.
The separate `append` name-hygiene fixture re-anchors on the real pure pair
`list_append`/`bytes_concat`, neither of which carries an I/O row.

**Grounded (content-verified against the landed targets, not heading numbers —
the `conformance-oracle-grounding-fallback` discipline):** `14 §5` (`Bytes` is
named a primitive type; opaque constant, registered reductions compute over
literals definitionally, trusted/audited — `18 §5`); `41 §3a` (`Bytes` =
kind-tag `0x05` interned compound; **`String` is NFC-normalized UTF-8 at
construction time** — the fact the round-trip law and its one-directionality
both rest on); `36 §1.4` (the escape check `ρ_inf ⊆ ρ_decl` accept / `ρ_inf ⊄
ρ_decl` EFFECT-ESCAPE — the **single soundness-relevant gate**, pinned in
L5); `31 §3` (`b"…"`/`0x[deadbeef]` ⇒ `Bytes`; bare `0xFF` ⇒ `Int`); `14 §8.4`
(W-style `ITree.Vis` admitted in K1.5 — so L6 adds **no** kernel rule and
carries no kernel-staging block). Cross-ref fidelity verified at each target; no
dangling forward-ref.

**Subsume-don't-proliferate — the escape-check gate has ONE home (L5).** The
`36 §1.4` mechanism (an undeclared effect escaping a declared row is a static
error, with the ≥2-distinct-effects discrimination) is pinned in
`../effects/seed-effects.md` (`eff-undeclared-escapes-rejected` + its accept-arm
flip, `eff-row-union-two-effects`). **L6 does not re-pin the gate.** AC2/AC3
pin only the **producer-specific** fact that rides it: real world-facing
operations carry mandatory rows in their signatures (`read_bytes visits [FS]`,
`print_line visits [Console]`). The bug CP0 preserves coverage for is distinct
from L5's — *L5*: "the gate fails to check `⊆`"; *CP0/L6*: "a real operation was
registered **without** its row (untracked I/O)." Under the latter bug a
perfectly-correct L5 gate still **accepts** a pure-context call (nothing
escapes), so the operation-row binding needs its own discriminator. The cases
cross-reference the L5 home rather than copy it.

**Reading disciplines (what makes a case here load-bearing):**
- **Structural, not "compiles."** AC1 asserts the **elaborated value/type** (the
  `Bytes` primitive, the `0x05` kind), and immutability as a
  **fresh-allocation** flip (`concat` yields a new slot; the input slot is
  unchanged), not the absence of a mutator alone (an absence that passes
  vacuously if the op is *coincidentally* missing — the `content-reconcile`
  absence-assertion gate).
- **Verdict-flip on the target bug** (`discriminating-conformance-verdict-must-
  flip`): AC2/AC3 each pair accept (rowed) vs reject (untracked) on the real
  `[FS]`/`[Console]` producers, so the verdict flips on a dropped/absent row;
  AC1's hex-vs-int and AC4's no-coercion flip on type.
- **The round-trip law is asserted as a dischargeable obligation over *all*
  strings** (`(property)`), not a single sampled round-trip (the untrusted-layer
  lesson) — AC5.
- **The reverse round-trip is pinned as a NON-law at the source** (`§1.5`): a
  `Bytes → String → Bytes` case asserting
  `bytes_decode b = Ok s ⇒ encode s = b` would be a **wrong case that rejects
  conforming implementations**, so the boundary is pinned with a **non-NFC
  distinguishing witness** (a witness off the degenerate already-NFC point —
  the `taint-axis`/off-grid-witness discipline) to show the asymmetry is real,
  not to require the reverse.
- **Safe partiality is exact, not an oracle.** CP0 closes the former
  `Option`-vs-refinement seam: `bytes_at` and `bytes_slice` return `Option`, and
  invalid indices/spans return `None`. The cases assert the result constructors
  and types, not merely the weaker "no silent out-of-bounds read" property.

**Tags.** `(soundness)` — a kernel **trusted-base** commitment: a registered
`Bytes` primitive reduction whose wrongness is a soundness bug (`14 §5`/`18 §5`,
the same class as the L1 `add 2 3 ⇓ 5` reductions). `(property)` — an invariant
over many inputs / an end-to-end law, not a single trace. `(oracle)` — confirmed
by the Spec enclave / at build time, safe as it is **not** kernel-normative:
the remaining pure-op spelling (`++`/`concat`), the `visits ρ` surface syntax
(L5 `OQ-syntax`), and error-kind strings. The producer names `read_bytes` and
`print_line` are landed declarations, not invented oracle stand-ins. CP0 fixes
`bytes_decode`, `bytes_at`, and `bytes_slice` by name and shape. The **`0x05`
encoding, the literal forms `b"…"`/`0x[…]`, the safe signatures, the registered
reductions, the partiality treatment, and every verdict** are **normative**, not
`(oracle)`.

---

## AC1 — `Bytes` is a primitive, immutable type

A `b"…"`/`0x[…]` literal elaborates **directly** to the `Bytes` primitive
(`14 §5` opaque constant, `41 §3a` `0x05`); there is **no mutating operation**.

### surface/bytes-io/bytes-literal-elaborates-to-primitive
- spec: `38 §1.1`, `14 §5`, `41 §3a` (`0x05`), `31 §3`
- given: the literals `b"GET"` and `0x[deadbeef]`.
- expect: each elaborates **directly** to a value of the **`Bytes` primitive
  type** — an opaque `14 §5` constant whose runtime form is content-addressed
  `0x05`-kinded interned compound (`41 §3a`/`§5`). Assert the **elaborated type
  is `Bytes`** (structural), **not** that it "compiles", and **not** via
  a `String` (a `b"…"` does not decode at the literal — no charset round-trip on
  introduction).
- why: AC1's introduction face as a **structural** assertion on the elaborated
  value/type. A bug that routes `b"…"` through `String` (then a
  `bytes_decode`) — or
  classifies the literal as anything but the `Bytes` primitive — is caught by
  the asserted type, where "compiles" would pass. (structural.)

### surface/bytes-io/bracketed-hex-is-bytes-bare-hex-is-int
- spec: `31 §3`, `38 §1.1`
- given: the two tokens `0x[ff]` and `0xFF`.
- expect: `0x[ff] : Bytes` (the **bracketed** form, a one-byte `Bytes`); `0xFF :
  Int` (the **un-bracketed** form, an arbitrary-precision `Int` literal). They
  are **different tokens with different types** and must not be conflated
  (`31 §3`).
- why: pins the lexer distinction the spec calls out explicitly. **Verdict/type
  flips:** a bug that lexes `0x[ff]` as `Int` (or `0xFF` as `Bytes`) gives the
  wrong static type and is caught by the asserted type on each. (type-flip.)

### surface/bytes-io/bytes-immutable-concat-allocates-fresh
- spec: `38 §1.1`, `41 §2` (append-mostly, immutable heap)
- given: `a = 0x[dead]`, `b = 0x[beef]`, then `c = concat a b` (`++`).
- expect: `c` is a **new** value `0x[deadbeef]` occupying a **fresh slot**
  distinct from `a`'s and `b`'s slot-ids; **`a` is unchanged** after (same
  slot-id, same content `0x[dead]`). No surface operation mutates a `Bytes` in
  place — "updating" **allocates** and shares nothing observable with the old
  value.
- why: AC1's immutability face as a **fresh-allocation structural flip**, not a
  vacuous "no mutator exists". A hypothetical in-place-mutate bug (`concat`
  growing `a`'s buffer) would change `a`'s slot-id/content — caught by asserting
  `a` is unchanged **and** `c` is a distinct slot. Grounds immutability in the
  `41 §2` append-mostly heap. (structural; the absence-assertion gate met by a
  positive flip.)

---

## AC1-support (§1.2) — core ops: registered reductions + partiality

### surface/bytes-io/bytes-prim-reduces-over-literals (soundness)
- spec: `38 §1.2`, `14 §5` (registered reductions), `18 §5` (trusted base)
- given: `length 0x[deadbeef]`, and `length b` for an abstract `b : Bytes`.
- expect: `length 0x[deadbeef] ≡ 4 : Int` **definitionally** — the registered
  `prim` reduction computes the literal result **in the kernel's evaluator**, so
  the equality closes by `refl` and proofs reduce over literals (same discipline
  as `add 2 3 ≡ 5`, `35`). On the **stuck** argument `length b` is a **neutral**
  term (no reduction fires).
- why: pins the `Bytes` core ops as **registered reductions in the trusted
  base** — `(soundness)` because a wrong primitive reduction (e.g. `length 0x[
  deadbeef] ⇝ 3`) is a kernel soundness bug (`14 §5`/`18 §5`). The reduces-over-
  literals / neutral-on-stuck pair is the primitive-reduction discipline made
  executable. (reduces-to + neutral, trusted-base.)

### surface/bytes-io/bytes-at-some-in-range-none-out-of-range
- spec: CP0 Deliverable B/AC2, `38 §1.2`, `43 §2` (partiality)
- given: `b = 0x[00ff]`; evaluate `bytes_at b 1`, `bytes_at b 2`, and
  `bytes_at b (-1)`.
- expect: `bytes_at : Bytes → Int → Option UInt8`; `bytes_at b 1 = Some u`,
  where `u : UInt8` has numeric value 255, while the upper-bound and
  negative-index calls both equal `None`. The out-of-range result is **never**
  a bare/neutral `0` and never an unchecked read.
- why: discriminates the CP0 safe shape from the old
  `Bytes → Int → Int`/neutral-on-invalid shape on both the static and dynamic
  axes. An implementation retaining the old result type fails the asserted
  `Option UInt8`; one returning the old neutral `0` fails the `None` result.
  The in-range arm prevents a vacuous implementation that returns `None` for
  every index. (type/value flip; controlled in-range arm.)

### surface/bytes-io/bytes-slice-third-argument-is-length-invalid-is-none
- spec: CP0 Deliverable B, `38 §1.2`, `43 §2` (partiality)
- given: `b = 0x[00112233]`; evaluate `bytes_slice b 1 2`,
  `bytes_slice b 3 2`, `bytes_slice b (-1) 1`, and `bytes_slice b 1 (-1)`.
- expect: `bytes_slice : Bytes → Int(start) → Int(len) → Option Bytes` and
  `bytes_slice b 1 2 = Some 0x[1122]`. The third argument is a **length**, not
  an end offset. The overlong span, negative start, and negative length each
  return `None`; none yields a truncated, empty, neutral, or unchecked slice.
- why: the positive arm is a direct discriminator for the length-vs-end-offset
  trap: interpreting `2` as an end offset would produce only `0x[11]`, not
  `0x[1122]`. The invalid arms distinguish the safe `Option` contract from the
  old neutral-on-invalid `Bytes` result, while the valid arm prevents
  always-`None` from passing. (type/value flip; argument-semantics flip.)

---

## AC2 — `[FS]` binary I/O is effect-tracked (operation-row binding)

The real landed `read_bytes` declaration is authority-polymorphic and returns
an `FS` interaction tree; this seed does not re-pin I-3's argument/result shape.
It pins the shared operation-row fact: the declaration **carries**
`visits [FS]`, so an untracked call is a type error via the L5-pinned `36 §1.4`
gate (this case does not re-pin the gate — see the subsume note).

### surface/bytes-io/read-bytes-untracked-is-type-error
- spec: `38 §1.3`, `36 §1.4` (gate home: `../effects/seed-effects.md`
  `eff-undeclared-escapes-rejected`)
- given: the real landed `read_bytes … visits [FS]` called from (a) a `view`
  declaring **no** row (`ρ_decl = ∅`, pure by default); (b) a `view` declaring
  `visits [FS]`. Derive its row from `ElabEnv.effect_rows`, populated from the
  actual declaration; do not hand-feed an `FS` literal.
- expect: (a) **static error** EFFECT-ESCAPE (`EffectEscapes(FS)`, kind
  `(oracle)`) — `read_bytes`'s latent `[FS]` is in the inferred row, `[FS] ⊄ ∅`;
  (b) **accepts** — `[FS] ⊆ [FS]`. A **verdict flip** keyed on whether the I/O
  operation's row is honored.
- why: AC2 — the **producer operation-row binding**, not the gate (the gate is
  L5's, cross-referenced). Registering `read_bytes` **without** `visits [FS]`
  makes (a) wrongly **accept** even under a correct `36 §1.4` check — so the row
  must be on the real declaration. Verdict flips (a reject / b accept) on the
  row's presence. (verdict-flip; references L5 gate, does not duplicate it.)

---

## AC3 — `[Console]` output is effect-tracked (the ≥2nd distinct effect)

### surface/bytes-io/print-line-untracked-is-type-error
- spec: `36 §1.4` (gate home as above), CP0 fixture-re-anchor disposition
- given: the real landed
  `print_line : String → IO Unit visits [Console]` called from (a) a `view`
  with **no** row; (b) a `view` declaring `visits [Console]`. Derive the callee
  row from `ElabEnv.effect_rows`, populated by the actual prelude declaration;
  do not hand-feed a synthetic effect declaration.
- expect: (a) **static error** EFFECT-ESCAPE (`EffectEscapes(Console)`) —
  `[Console] ⊄ ∅`; (b) **accepts** — `[Console] ⊆ [Console]`. The **same flip**
  as AC2 on a **distinct real effect**, exercising the `36 §1.4`
  ≥2-distinct-effects discrimination at the producer-registration layer.
- why: AC3 — the operation-row binding on the second real producer. A bug
  registering `print_line` without `visits [Console]` makes (a) wrongly accept;
  the row's presence flips the verdict. `[Console]` is distinct from AC2's
  `[FS]`, so the two cases cannot collapse to one label. No `[Net]` producer is
  assumed or synthesized. (verdict-flip, real distinct effect.)

---

## AC4 — text from bytes via named, partial `bytes_decode` (no hidden charset)

### surface/bytes-io/text-from-bytes-requires-named-bytes-decode
- spec: `38 §1.4`, `41 §3a` (NFC at construction)
- given: a `Bytes` value used where a `String` is expected with **no**
  `bytes_decode` step.
- expect: **static error** — there is **no** implicit `Bytes → String` coercion
  and **no** "default charset"; the only path to a `String` is the named
  `bytes_decode : Bytes → Result Utf8Error String` followed by matching its
  `Ok` arm.
- why: AC4 — text is **explicit**. A bug adding an implicit-charset coercion
  would wrongly **accept** the un-decoded use. **Named absence guard:** the
  elaborator has no `Bytes`-to-`String` coercion; the sole producer from
  `Bytes` is `bytes_decode`. (verdict flip + named-absence.)

### surface/bytes-io/bytes-decode-valid-ok-invalid-utf8-err
- spec: CP0 Deliverable B, `38 §1.4`, `41 §3a` (NFC at construction)
- given: call `bytes_decode` on (a) valid UTF-8 `0x[41]` (`"A"`) and (b) invalid
  UTF-8 `0x[ff]` (`0xFF` is not a valid UTF-8 lead byte).
- expect: `bytes_decode : Bytes → Result Utf8Error String`;
  `bytes_decode 0x[41] = Ok "A"`, while `bytes_decode 0x[ff] = Err e` for some
  `e : Utf8Error`. Invalid input never yields a neutral or fabricated `String`.
- why: the valid/invalid pair holds the operation fixed and varies only UTF-8
  validity. It distinguishes CP0's exact `Result` shape from the old
  `Bytes → String` neutral-on-invalid behavior, and the `Ok` arm prevents an
  always-`Err` implementation from passing. It also proves why conformance does
  **not** assert that arbitrary bytes decode. (type/value flip; controlled
  validity pair.)

---

## AC5 — the round-trip law (one-directional, dischargeable)

### surface/bytes-io/bytes-decode-encode-roundtrip-provable (property)
- spec: `38 §1.5`, `41 §3a` (NFC idempotent at construction), `20-verification/`
- given: the named `BytesRoundTripLaw` obligation
  `∀ (s : String). bytes_decode (encode s) = Ok s`.
- expect: the obligation is **dischargeable** (provable) against
  `20-verification/` — `encode s` is valid UTF-8 (so `bytes_decode` succeeds,
  `Ok`), `bytes_decode` rebuilds a `String` **NFC-normalizing at construction**
  (`41 §3a`), and `s` is **already** NFC with NFC **idempotent**, so the
  reconstruction **equals** `s`. Assert the **obligation is provable over all `s`**
  (`(property)`), **not** that one sampled string round-trips.
- why: AC5 — the serialization contract as a **verified-component** target. The
  structural assertion is "the obligation discharges", per the untrusted-layer
  lesson; a single-sample case would pass even if the law fails for some `s`.
  The proof rests on NFC-idempotence (`41 §3a`), re-derived here, not assumed.
  (property/obligation.)

### surface/bytes-io/reverse-roundtrip-is-not-a-law
- spec: `38 §1.5` (the silence pinned at source), `41 §3a` (renormalization)
- given: the **non-NFC but valid** UTF-8 witness `b = 0x[65 cc 81]` — the NFD
  spelling of `"é"` (`U+0065` `e` + `U+0301` combining acute). Consider the
  successful `Ok s` arm of `bytes_decode b`, followed by `encode s`.
- expect: `bytes_decode b = Ok "é"`, but the reconstructed `String`
  **NFC-normalizes** at construction (`41 §3a`) to `U+00E9`, so encoding the
  successful result yields `0x[c3 a9]` (the NFC bytes) **≠**
  `b = 0x[65 cc 81]`. Therefore the successful
  `Bytes → String → Bytes` reverse trip does **NOT** preserve every valid input
  byte sequence, and **conformance must NOT assert it as a law** — doing so
  would reject conforming (renormalizing) implementations.
- why: pins the **law-boundary silence at source** (`verdict-mapping-silence`
  + `38 §1.5`): the round-trip is **one-way**. The witness is **non-NFC**
  — deliberately **off** the degenerate already-NFC point, where the successful
  decode-then-encode path *would* preserve the input and the asymmetry would
  hide (green-vs-green, the off-grid-witness discipline). A
  **negative/guard** case: it asserts what conformance must **not** require, so
  a future author does not "complete" the round-trip the wrong way.
  (boundary-guard; non-NFC witness.)

---

## Coverage map (AC → cases)

- **AC1** (`Bytes` primitive + immutable): `bytes-literal-elaborates-to-
  primitive`, `bracketed-hex-is-bytes-bare-hex-is-int`, `bytes-immutable-concat-
  allocates-fresh`.
- **§1.2 core ops** (registered reductions + safe partiality):
  `bytes-prim-reduces-over-literals` (soundness),
  `bytes-at-some-in-range-none-out-of-range`,
  `bytes-slice-third-argument-is-length-invalid-is-none`.
- **AC2** (`[FS]` tracked): `read-bytes-untracked-is-type-error`.
- **AC3** (`[Console]` tracked): `print-line-untracked-is-type-error`.
- **AC4** (no hidden charset):
  `text-from-bytes-requires-named-bytes-decode`,
  `bytes-decode-valid-ok-invalid-utf8-err`.
- **AC5** (round-trip law, one-directional):
  `bytes-decode-encode-roundtrip-provable`
  (property), `reverse-roundtrip-is-not-a-law`.

## Cross-case consistency sweep

- **Effect-tracked producer class (`36 §1.4` operation-row binding).** AC2
  (`read_bytes`/`FS`) and AC3 (`print_line`/`Console`) are **one metatheory
  class** — they must **agree**: each **accepts** under its correct row and
  **rejects** the untracked call, and both flip the **same** way under the bug
  "a real operation was registered without its mandatory row." A divergence
  would be an inconsistency in operation-row binding. This sweep uses two real
  landed producers and derives both rows from `ElabEnv.effect_rows`; the gate
  itself is swept in L5 (`../effects/seed-effects.md`).
- **Safe partiality class.** `bytes-at-some-in-range-none-out-of-range` and
  `bytes-slice-third-argument-is-length-invalid-is-none` agree on the CP0
  invalid-input contract: each has a successful `Some` arm and invalid `None`
  arms; neither may return an old neutral payload. The positive arms also make
  always-`None` fail, and the slice arm independently pins length semantics.
- **Decode partiality class.**
  `text-from-bytes-requires-named-bytes-decode` rejects the absent named step;
  `bytes-decode-valid-ok-invalid-utf8-err` then holds that step fixed and flips
  `Ok`/`Err` only on UTF-8 validity. The round-trip case quantifies over
  `encode` outputs, not arbitrary `Bytes`, so it cannot contradict the invalid
  witness.
- **Round-trip directionality class.**
  `bytes-decode-encode-roundtrip-provable`
  (forward, **provable**) and `reverse-roundtrip-is-not-a-law` (reverse, **not a
  law**) must not contradict: the forward law holds for all `s : String`; the
  reverse fails on a non-NFC witness. Both rest on the **same** fact (`String`
  NFC-normalizes at construction, `41 §3a`) — NFC-idempotence makes the forward
  hold and renormalization makes the reverse fail. A case asserting the reverse
  as a law would contradict this class.
- **`Bytes`-from-text introduction is singular.** AC1's `b"…"`-is-not-via-
  `String` and AC4's only-`bytes_decode`-yields-`String` are duals:
  introduction of a `Bytes` literal never routes through `String`, and
  production of a `String` from `Bytes` never routes through anything but
  `bytes_decode` — no hidden charset on either side of the boundary.

## Subsumed / not-duplicated (one home per property)

- **The `36 §1.4` escape-check gate** (undeclared effect escapes; the ≥2-effect
  discrimination; pure-by-default ⇒ any effect escapes) is pinned in
  `../effects/seed-effects.md` (`eff-undeclared-escapes-rejected`,
  `eff-declared-matches-used-accepted`, `eff-row-union-two-effects`). L6
  **references** it; AC2/AC3 add only the **operation-row binding** for the L6
  I/O operations. No gate case is copied here.
- **`ITree`/`Vis` denotation, capability gating, `space` state** (`36 §2`–`§5`)
  are L5's (`../effects/seed-effects.md`). L6 I/O operations are `Vis` nodes at
  the world frontier (`38 §1.3`), but the `Vis`-shape property has its
  home in L5 (`eff-denotes-to-interaction-tree`); L6 does not re-pin it.
- **`foreign`/FFI + the trust boundary** (`38 §2`–`§3`, postulate-as-listed,
  `pure`-as-claim, runtime contracts) are **L7**, under `../ffi-io/` — out of L6
  scope; not authored here.
- **The serialization/integrity (Merkle/BLAKE3) hash** (`41 §3b`) is distinct
  from in-process FNV-1a addressing, selected **downstream** (L8/transport),
  not pinned by L6.

## Build-sequencing note

L6 builds on **landed** kernel/runtime: `Bytes` is the `41 §3a` `0x05` interned
compound (K3 store) and a `14 §5` primitive (no new kernel rule — the W-style
`ITree.Vis` admission already **landed in K1.5**, `14 §8.4`). The effect-row
machinery (`36 §1.4` gate, `ITree` denotation) is **L5, on `main`**. So every
case here drives **real** values/signatures through **landed** mechanisms: AC2/
AC3 route a real I/O signature through the real `36 §1.4` escape check (a real
untracked call → a real reject, per the QA gate — not a synthetic flag); AC5's
obligation discharges against the real `20-verification/` pipeline. The
Language's CP0 build half delivers the safe **operations**
(`bytes_decode`/`bytes_at`/`bytes_slice`) and their native lowering over the
landed substrate. The pre-existing `read_bytes` operation-row case stays live;
the retired `send`/`[Net]` case is replaced by the real landed
`print_line`/`[Console]` producer, never by a synthetic Net declaration.
