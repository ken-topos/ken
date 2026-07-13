# WP CC2 — `Text.Codec` + `Text.Numeric` (+ gated lawful `String` keys)

Land the catalog's text-decoding and numeric-parsing packages over the safe
`Bytes` ops from CP0, with located parse errors and digit extraction that ride
landed primitives — plus, gated on an operator decision, lawful
`DecEq`/`Ord String` by transport through `List Char`.

**Program II (catalog closure), CC2.** Owner: **Foundation**. Reviewer:
**Architect** (soundness/design) + **CV** (conformance seed only). Size: **M**.
Base: `origin/main @ 6088e0b8`. Branch: `wp/cc2-text-codec-numeric`.

Built on the safe `Bytes` ops landed by **CP0** (`origin/main @ 6088e0b8`).
Thread all CC2 activity in its kickoff thread. **Zero NEW trust delta** — ordinary
kernel-checked catalog Ken; no kernel rule, no primitive, no postulate minted
inside CC2's own fences. (The sole trust question — a `String` injectivity
certificate — is homed OUTSIDE CC2, at the bijection layer, and is itself gated;
see fixed input 1.)

## Fixed inputs (settled — do NOT reopen)

These are grounded facts the Architect verified against `origin/main @ 6088e0b8`.
State them as fixed; the whole WP is built on them.

0. **Landed substrate (grounded @ 6088e0b8 — DO NOT rebuild).**
   - `Char := {c:Int | isScalar c}`; `charToInt c = c` is a **landed identity
     projection** (`decimal_char.rs:246`), surface-accessible. Digit chars are
     written as scalar `Int`s, e.g. `(48:Int)` (per `Console.ken.md:18` idiom
     `Cons Char (10:Int)`).
   - `Ord Char` / `DecEq Char` are landed, **by transport from `Int`**
     (`LawfulClasses §4.3/4.4`), riding `Ord Int`'s four bare `Axiom` laws
     (`LawfulClasses:267-272`) plus the `DecEq Int` certificate.
   - Lawful `Ord (List a)` (`LawfulClasses:2562`) and `DecEq (List a)`
     (`:2711`) are **already landed, `Axiom`-free** (lexicographic, via
     `lex_head_sound`). **The List-lifting layer is DONE** — it is a harness
     **dependency to load**, NOT a CC2 deliverable.
   - `string_to_list_char` / `list_char_to_string` are landed total.
     `string_eq` / `string_compare` are landed **functions** in Collections
     (functions-only; Collections §5 deferred proof-carrying instances as
     over-claiming trust).
   - **NO** `String↔List Char` retraction law is landed. **NO** `Bytes↔List
     UInt8` bridge, **NO** `DecEq/Ord Bytes`, **NO** bytes-eq cert exist.

1. **Lawful `DecEq`/`Ord String` = transport through `List Char`; ONE named
   injectivity certificate; GATED on an operator decision; `Bytes` DESCOPED.**
   - **Mechanism:** adopt structural transport through `List Char`. **REJECT any
     new comparator primitive** (redundant, larger trust surface). Because
     `Ord`/`DecEq Char` AND `Ord`/`DecEq (List a)` are already lawful+landed,
     `Ord String` / `DecEq String` are mostly `.`-projection + `cong`: the
     `complete` (DecEq) and `refl`/`trans`/`total` (Ord) laws are
     **zero-new-delta** — they never conclude `Equal String`. **ONLY** `sound`
     (DecEq) and `antisym` (Ord) conclude `Equal String x y` — from `Equal
     (List Char) (to_list x) (to_list y)` — and therefore **require injectivity
     of `string_to_list_char`**.
   - That injectivity is **the single irreducible trust question**: `String` is
     opaque, cannot be case-proved, is not landed, and is not derivable from
     landed facts (`BytesRoundTripLaw` gives `bytes_encode` injectivity — a
     **different** bridge, no path to `to_list`). It must be **ONE named
     certificate at the bijection layer** — `string_to_list_char` injectivity ≡
     the retraction `list_char_to_string (string_to_list_char s) = s` —
     registered as a **PREREQUISITE homed at the bijection layer** (exactly as
     `DecEqCert` lives at the numeric-tower layer and `BytesRoundTripLaw` at the
     bytes layer), **NEVER minted inside CC2's own instance fences.**
   - **GATING (critical):** lawful `DecEq`/`Ord String` is the **FINAL, GATED**
     CC2 deliverable, blocked on an **OPERATOR decision**. It is an irreducible
     conflict between two operator directives: "lawful `String` keys" (⟺ pay one
     injectivity postulate) vs the operator's stated "zero-trust-delta CC
     posture" (no new postulates).
     - **Option (i) [Steward + Architect RECOMMENDATION]:** land ONE small
       injectivity-cert prereq WP at the bijection layer, then the `String`-keys
       instance rides it with **zero-`Axiom` fences**.
     - **Option (ii):** descope `String` keys to **functions-only** (Collections'
       honest posture) until a future bijection-law WP.
     This deliverable does **NOT start** until the operator confirms (i) vs (ii).
     `Text.Codec` + `Text.Numeric` **proceed regardless**.
   - Lawful **`Bytes` keys are DESCOPED** to a separate fast-follow WP: no
     `Bytes↔List UInt8` substrate/cert exists, so it needs **both** a new
     structural bridge AND its extensionality cert. Out of CC2 scope.

2. **Located errors = a minimal in-package located carrier, NOT generic-over-`e`.**
   - Do **NOT** overload `Utf8Error = InvalidUtf8` (nullary, decode-specific) for
     numeric parse failure. `Text.Numeric` (or a tiny `Text` error module) owns a
     **minimal located carrier**:
     ```
     data NumericError = MkNumericError NumericErrorKind Nat
     data NumericErrorKind = EmptyInput | InvalidDigit
     ```
     The `Nat` is a **char-index offset** into the input `List Char` (NOT a byte
     offset). No `Overflow` kind — `Int` is arbitrary-precision; add one only if a
     fixed-width variant is later in scope.
   - Frame it explicitly as the **minimal pre-CC4 carrier** that CC4's
     `Diagnostic` will **subsume** (reflect-don't-extend) — not location-free, and
     not a premature rich diagnostic.

3. **Digit extraction = landed `charToInt` + `leq_int`, NO new primitive.**
   ```
   char_to_digit (c:Char) : Option Int =
     let n = charToInt c in
     if leq_int 48 n && leq_int n 57 then Some (n - 48) else None
   ```
   Decimal; hex extends with `65..70` / `97..102`. Rides landed `charToInt` +
   `leq_int` — **zero new primitive, zero new trust**. **Return type is `Option
   Int`, NOT `Option UInt8`** (Architect note `evt_6zp37xvewdayd`): only
   `uint8_to_int` is landed, not the reverse `int_to_uint8`, so a `UInt8` return
   would need an unlanded cast; `n − 48 : Int` feeds the base accumulation
   (`acc·base + digit`) directly with no round-trip. (`Option Nat` is equally fine
   if the implementer prefers; AC4's `'0'..'9' → Some 0..9` tracks either.) Do **NOT** admit
   `char_to_int` / `to_digit` / `ord` primitives. `Text.Numeric` folds
   `char_to_digit` over `string_to_list_char` input with base accumulation,
   failing **located** (carrier of fixed input 2) on the first `None`.

4. **Bijection-debt confinement (Architect guard-rail `evt_49zk7vabtbfq8`) —
   CRITICAL.** Confine **ALL** `String↔List Char` bijection debt to the single
   gated keys cert (fixed input 1); do **NOT** let it leak into `Text.Numeric` /
   `Text.Codec` verified laws. Grounded @ 6088e0b8: **neither** bijection
   direction is a landed law — the retraction `list_char_to_string ∘
   string_to_list_char = id` **and** its section are only **tested-to-reduce on
   ground terms** (`l3_strings_roundtrip_acceptance.rs:172,199`), never proved
   universal. A round-trip stated `∀ (s:String). …` over the **opaque** `String`
   var **does not reduce** (neutral var) — so the ground-reduction test is **not**
   the universal law. Therefore:
   - **(a)** `Text.Codec` + `Text.Numeric` ship parse / format / view as
     **FUNCTIONS** — they reduce correctly on ground inputs, zero-trust.
   - **(b)** Any **VERIFIED round-trip LAW** is stated at the `List Char` / digit
     level — e.g. `parse_digits (format_digits n) = Ok n`, pure structural over
     the digit fold + landed `List`/`Int` facts — and **NEVER crosses into
     `String`.** The `String`-facing `parse` / `format` wrappers stay functions;
     the `String↔List Char` hop is the un-provable part, deferred wholesale to the
     fixed-input-1 keys cert.
   - **(c)** `Text.Codec`'s round-trip rides the **landed** `BytesRoundTripLaw`
     (`decode ∘ encode = Ok`, `Bytes↔String`; reverse explicitly not-a-law) — do
     **NOT** assert a `String↔List Char` crossing view law.
   `Text.Numeric` is confirmed **independent of** `DecEq`/`Ord String` (parse /
   format never compare `String` keys), so it is genuinely decoupled from the
   gated keys deliverable.

## Mandated deliverable outline

Each section ends in a concrete, implementable choice — not a survey.

1. **`Text.Codec`** — ASCII/UTF-8 views over `bytes_decode` / `bytes_at`. Define
   the ASCII view and its laws **in-package as ordinary Ken**. Exposed surface:
   a `decode_utf8 : Bytes -> Result Utf8Error String` re-export/wrap over landed
   `bytes_decode`, plus an `ascii_view` / byte-classification surface (e.g.
   is-ascii, byte-at classification via `bytes_at`). **NO new codec types** beyond
   what this surface needs — no speculative encoders/formats.

2. **`Text.Numeric`** — `char_to_digit` (fixed input 3); `parse_nat` / `parse_int`
   returning `Result NumericError Int`; and a format / `show_int` direction. Parse
   folds `char_to_digit` over `string_to_list_char` input with base accumulation,
   failing **located** on the first bad digit (`InvalidDigit` at the char index)
   and on empty input (`EmptyInput` at `0`).

3. **Lawful `DecEq`/`Ord String` (GATED, final — see fixed input 1).** The
   transport instance skeleton: `sound` (DecEq) and `antisym` (Ord) **cite the
   prereq injectivity cert BY NAME** (homed at the bijection layer); `complete` /
   `refl` / `trans` / `total` are **zero-delta** `.`-projection + `cong` off the
   landed `List Char` instances. Does **not** start until the operator confirms
   option (i) vs (ii).

## Acceptance criteria (testable)

- **AC1 — DS-7/8 ordered shared-`ElabEnv` acceptance harness.** Following the
  template `crates/ken-elaborator/tests/cc1_nonempty_validation_acceptance.rs`:
  ONE shared `ElabEnv`, dependency closure elaborated **IN ORDER** — Transport →
  Collections → LawfulClasses → **[(i) injectivity-cert prereq, IF the operator
  picks (i)]** → Text.Codec → Text.Numeric — then the CC2 entries **including
  every checked literate fence**; assert the checked globals are **real,
  transparent, kernel-checked terms**.
- **AC2 — zero-`Axiom` in CC2's OWN checked fences.** This gate is the backstop
  that **forces fixed input 1's injectivity anchor to the prereq bijection
  layer** — an in-fence `Axiom` mint (e.g. a smuggled `sound`/`antisym`) **must
  fail** the gate.
- **AC3 — `trusted_base()` before == after, measured on CC2's OWN fences.** This
  is **zero-NEW-delta-by-transport**, NOT zero-total-delta: CC2 legitimately
  rides pre-existing `Ord Int` `Axiom`s + the `DecEq Int` cert, and — under
  option (i) — the one prereq injectivity cert. The gate must **not** count
  landed upstream `Axiom`s against CC2.
- **AC4 — `Text.Numeric` located-error discriminators.** Parse of a valid decimal
  string → `Ok n`; empty input → `Err (MkNumericError EmptyInput 0)`; a bad digit
  at position `k` → `Err (MkNumericError InvalidDigit k)` with the **exact** `k`
  (defeats a location-free or off-by-one impl). `char_to_digit`: `'0'..'9'` →
  `Some 0..9`, non-digit → `None`. **Any VERIFIED round-trip law (fixed input 4b)
  is stated at the `List Char`/digit level** (`parse_digits (format_digits n) =
  Ok n`, structural) and **must NOT cross into `String`** — the `String`-facing
  `parse`/`format` wrappers are functions, not proof-carrying.
- **AC5 — `Text.Codec` elaborates and classifies.** `decode_utf8` wraps
  `bytes_decode` returning `Result Utf8Error String`; the `ascii_view` /
  byte-classification surface elaborates and its in-package ASCII-view laws are
  kernel-checked. Its round-trip rides the **landed `BytesRoundTripLaw`**; **no
  fence asserts a `String↔List Char` crossing view law** (fixed input 4c).
- **AC6 — lawful `String` keys (GATED).** IFF the operator picks option (i): the
  transport `DecEq String` / `Ord String` instances elaborate `Axiom`-free in
  CC2's fences, with `sound`/`antisym` discharged by the named prereq cert;
  `complete`/`refl`/`trans`/`total` as zero-delta projection+cong. IFF option
  (ii): no `String` instance ships; functions-only stands. This AC does not gate
  Text.Codec/Text.Numeric close.
- **AC7 — conformance seed for `Text.Numeric` parse semantics (→ CV vote).** A
  black-box conformance seed pins the parse semantics (valid parse, empty-input
  located reject, bad-digit located reject at the exact index). Because it touches
  `conformance/`, **CV is a required reviewer** for this seed.
- **AC8 — scope discipline.** Only the new `Text.Codec` / `Text.Numeric` catalog
  packages (+ the tiny `Text` error module for the `NumericError` carrier, +
  under option (i) the separately-homed prereq cert); no kernel/prelude/`Cargo`/
  lock delta; no new primitives; no speculative helpers.

## Do-not-reopen guardrails

- Do **NOT** let the implementer silently `Axiom`-mint `sound`/`antisym` inside
  CC2 fences. The anchor's **HOME = the prereq bijection layer**; pin it there so
  a block escalates correctly (and AC2 fails a smuggled mint).
- **List-lifting is landed** (lawful `Ord (List a)` / `DecEq (List a)`, `Axiom`-
  free) — load it as a dependency; do **NOT** rebuild it.
- **`Bytes` keys are out of scope** — a fast-follow WP; no `Bytes↔List UInt8`
  bridge or cert exists to ride.
- **No new primitives** — ride landed `charToInt`, `leq_int`, `bytes_decode`,
  `bytes_at`, `string_to_list_char`, `list_char_to_string`. Do not admit
  `char_to_int` / `to_digit` / `ord`, nor any new comparator primitive.
- `Text.Codec` / `Text.Numeric` are **ordinary catalog Ken with zero
  kernel/prelude delta** — no kernel rule, no prelude emission.
- **No `String`-crossing round-trip law in ANY `Text.Codec`/`Text.Numeric`
  fence** (fixed input 4). A verified round-trip law lives at the `List
  Char`/digit level only; the `String↔List Char` hop is the un-provable part,
  confined wholesale to the gated fixed-input-1 keys cert. This is the specific
  thing the Architect re-confirm checks: **(a)** no Numeric/Codec fence states a
  `String`-crossing round-trip law, **(b)** the fixed-input-1 injectivity cert is
  homed at the prereq bijection layer, not in a `Text` fence.
- Do **NOT** overload `Utf8Error` for numeric parse failure, and do **NOT** add an
  `Overflow` kind while `Int` is arbitrary-precision.

## Sequencing & review chain

`Text.Codec` and `Text.Numeric` proceed **immediately** (they do not depend on
the gate). In parallel, the Steward carries the option (i) vs (ii) decision to the
operator; the lawful-`String`-keys deliverable (and, under (i), the prereq
injectivity-cert WP at the bijection layer) **does not start** until that confirms.

Foundation builds → QA (Foundation) → **Architect** review (soundness/design of
the transport instances + the located-error carrier + the anchor's home) →
**CV** review of the AC7 conformance seed (required — it touches `conformance/`)
→ `git_request` to Steward → honesty-gate + CI-poll publish. CC2 closes once
Text.Codec + Text.Numeric land and retros are in; the gated `String`-keys
deliverable closes per the operator's option (i)/(ii) resolution.
