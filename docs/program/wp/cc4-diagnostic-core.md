# WP CC4 — `Diagnostic.Core` (the origin-neutral located diagnostic)

Land the catalog's **origin-neutral** diagnostic: a checked value that knows
**where it came from** — a source byte range, an argv position, an environment
variable, a config key — and **does not know how to print itself**. Then
**subsume** the three location carriers that already exist, so Ken has *one*
diagnostic vocabulary rather than a per-package dialect.

**Program II (catalog closure), CC4.** Owner: **Foundation**. Reviewer:
**Architect** (soundness/design) — **CV only if `conformance/` is touched**.
Size: **L**. Base: `origin/main @ b91c9735`. Branch: `wp/cc4-diagnostic-core`.

**Zero trust delta** — ordinary kernel-checked catalog Ken; no kernel rule, no
primitive, no postulate, no `Axiom` in CC4's fences.

## Fixed inputs (settled — do NOT reopen)

Grounded against `origin/main @ b91c9735`. **Treat every anchor as perishable —
re-verify against the landed code at pickup, not against this frame. If a fixed
input turns out FALSE, say so and escalate; do not quietly build around it.**
(That clause has now caught a bad pin of mine three times. Use it.)

0. **The abstraction is EARNED — it has three real consumers, all landed.**
   - **CAT-5** (`Capability/Parsing`): `data SourceId = MkSourceId Nat` (`:50`),
     `Span`, and the `ValidSpan` laws.
   - **CC3** (`Parsing/Cursor`): `data ArgLocation = MkArgLocation Nat Nat Nat`
     (arg index, start, end).
   - **CC2** (`Text/Numeric`): `data NumericError = MkNumericError
     NumericErrorKind Nat` — whose own frame **explicitly named it "the minimal
     pre-CC4 carrier that CC4's `Diagnostic` will SUBSUME."** This WP is that
     promise coming due.
   Three consumers, not a speculative generalization. **Do not add a fourth
   origin "for symmetry"** — see guardrails.

1. **★ THE DEPENDENCY-DAG PIN — read this before you design anything.**
   `SourceId` is declared **inside CAT-5**. So the naive shape —
   `Diagnostic.Core` defines a `SourceOrigin` carrying CAT-5's `SourceId`, while
   CAT-5 consumes `Diagnostic` — is a **CYCLE** (Diagnostic → CAT-5 →
   Diagnostic). This is the *exact* trap CC3's frame fell into and you caught.
   **The resolution is pinned:**
   - **`Diagnostic.Core` depends on NOTHING.** It is the bottom of the stack.
   - **`SourceId` MOVES DOWN into `Diagnostic.Core`.** It is
     `MkSourceId Nat` — a bare identifier with **no CAT-5-dependent laws
     attached** — and identifying a source artifact is a **diagnostic** concept,
     not a parsing one. Its home in a parsing package is an accident of CAT-5
     being written first. CAT-5 then **consumes** it. (Verify the no-laws claim
     at pickup; if `SourceId` turns out to carry a Source-relative law, **stop
     and escalate** — the move is off and we inject instead.)
   - **`Span` does NOT move.** Its `ValidSpan` laws are **`Source`-relative**, so
     they belong with `Source`. CAT-5 keeps `Span` + `ValidSpan` and supplies an
     **injection** into the neutral range type.
   - **Every injection lives with its CLIENT, never with `Diagnostic.Core`** — an
     abstraction module must not depend on its clients. CAT-5 owns
     `span → ByteRange`/`Origin`; `Parsing/Cursor` owns `ArgLocation → Origin`;
     `Text/Numeric` owns its own.
   - **Load order (acyclic):** `… → Diagnostic.Core → Parsing.Cursor →
     Parsing.Decoder → Capability.Parsing (CAT-5) → Text.Numeric`.

2. **`Decoder` needs NO change — and that is the design working.** CC3's
   `Decoder` is already **loc-generic**:
   ```
   data DecoderError loc  = DecoderRejected loc | DecoderZeroProgress loc
                          | DecoderFuelExhausted loc
   data DecoderResult c loc a = Decoded a c | DecoderFailed (DecoderError loc)
   const Decoder (c : Type) (loc : Type) (a : Type) : Type = c → DecoderResult c loc a
   ```
   So a client simply **instantiates `loc = Origin`**. **Do NOT re-type
   `Decoder`, do NOT make it depend on `Diagnostic.Core`, and do NOT special-case
   `Origin` inside it.** If you find yourself editing `Decoder.ken.md`, stop —
   you have taken a wrong turn.

3. **The value knows its LOCATION, not its RENDERING.** `Diagnostic` carries a
   structured **origin** + a structured **kind/code** — **never a pre-rendered
   message string**, never a width, never a layout decision. Rendering is
   **CC5 (`Pretty.Doc`)**, and CC5 needs `Diagnostic` to be render-free to have
   anything to do. **No `String` formatting, no `show`, no message templating in
   CC4.**

4. **Constructibility — clean, and here is the proof so you don't re-derive it.**
   Every location this WP must carry is built from `Nat`: CAT-5's `Span`
   (`Nat`×`Nat`), `ArgLocation` (`Nat`×`Nat`×`Nat`), `NumericError`'s char index
   (`Nat`). **No opaque-`Int`/`Bytes` length hop is required anywhere in CC4** —
   unlike CC3, you do **not** need a cached-`Nat` carrier here. If you find
   yourself reaching for `bytes_length` or wanting an `Int → Nat` bridge, **stop
   and escalate** — it means the shape drifted, and minting `int_to_nat` is a TCB
   delta that goes to the operator, never into a build WP.

5. **The origin sum is CLOSED at the four the report names:** `SourceOrigin`
   (source id + byte range), `ArgumentOrigin` (arg index + byte range),
   `EnvironmentOrigin` (variable name), `ConfigKeyOrigin` (key path). **Ship all
   four** — `Environment`/`ConfigKey` have named consumers in **CC8** (the
   environment/config decoder), so they are committed, not speculative. But **add
   no fifth**, and give none of them a payload beyond what its consumer needs.

6. **Package model — unchanged from CC3.** No cross-file `import`/`pub`; the
   catalog has no disk loader. Dependency-bearing packages are elaborated **in
   order into ONE shared `ElabEnv`** (AC1). A standalone `ken check` of a
   dependent package is **expected to fail** — that is the known package-model
   gap, **not** a bug to route around. **Escalate; do not smuggle `import`.**

7. **Home:** `Diagnostic.Core` → `catalog/packages/Diagnostic/Core.ken.md`
   (§13's identity map: N dotted components → N−1 directories + a leaf).

## Mandated deliverable outline

1. **`Diagnostic.Core`** — `SourceId` (moved down); a neutral `ByteRange`
   (`Nat`×`Nat`); the closed `Origin` sum (fixed input 5); the `Diagnostic` value
   (origin + structured kind/code, **no rendering**); and the **validity
   predicates** the value must satisfy (a well-formed range has `start ≤ end`;
   an origin's payload is well-formed) — as plain checkable predicates, mirroring
   CAT-5's `ValidSpan` posture (checkable per value, not enforced by the type).

2. **The three subsumptions — this is the WP's real objective.**
   - **CAT-5**: consumes the moved `SourceId`; keeps `Span` + `ValidSpan`
     (Source-relative); supplies `span → ByteRange` and a `SourceOrigin`
     injection. Its landed laws and the Boolean grammar **must survive** (AC2).
   - **`Parsing/Cursor`**: supplies an `ArgLocation → ArgumentOrigin` injection.
     `ArgLocation` may remain as the cursor's own loc type — but **prove the
     injection is faithful** (index and byte range survive it exactly, AC4).
   - **`Text/Numeric`**: **`NumericError` is re-homed onto `Diagnostic`** — this
     is the subsumption CC2's frame promised. Its `EmptyInput`/`InvalidDigit`
     kinds become `Diagnostic` kinds; its `Nat` char index becomes an origin.
     **Do not leave a parallel `NumericError` alive** — a surviving second
     carrier is the "two universes" failure this WP exists to prevent.

## Acceptance criteria (testable)

- **AC1 — DS-7/8 ordered shared-`ElabEnv` harness.**
  `crates/ken-elaborator/tests/cc4_diagnostic_core_acceptance.rs`, following
  `cc3_parsing_cursor_decoder_acceptance.rs`: ONE shared `ElabEnv`, dependency
  closure elaborated **IN ORDER** — Transport → Collections → LawfulClasses →
  **Diagnostic.Core** → Parsing.Cursor → Parsing.Decoder → **CAT-5** →
  **Text.Numeric** — then every checked fence; assert the checked globals are
  real, transparent, kernel-checked terms. **NOT a standalone `ken check`.**
- **AC2 — anti-regression: `cat5_parsing_package.rs` (19 assertions) and
  `cc3_parsing_cursor_decoder_acceptance.rs` and
  `cc2_text_codec_numeric_acceptance.rs` all stay GREEN.** Re-point where a
  signature genuinely moved; a re-point that **drops or weakens** an assertion is
  a regression, not a re-point. CAT-5 still parses `(and true (not false))`,
  still rejects `true and false`, spans still exact.
- **AC3 — origin faithfulness, at NON-DEGENERATE positions.** For each of the
  three injections, the round-trip preserves the location **exactly**: a CAT-5
  span at a non-zero, non-empty range; an `ArgLocation` at **arg 2, bytes 3–3**
  (non-zero index **and** non-zero offset — the CC3 discriminator, still live);
  a `NumericError` at char index **k ≠ 0**. A location-free, zero-defaulting, or
  off-by-one injection **must fail** these.
- **AC4 — the diagnostic does NOT render.** No `String`-producing function on
  `Diagnostic`, no `show`, no width/layout parameter anywhere in CC4's fences.
  (Grep-able and mechanical: this is what leaves CC5 something to do.)
- **AC5 — no surviving second carrier.** `Text.Numeric` exposes **no**
  standalone `NumericError` after the subsumption. **This is the objective a WP
  can pass every test while quietly failing** (CC3's CAT-5 lesson): a green suite
  with the old carrier still alive underneath is a FAILED CC4.
- **AC6 — zero trust delta.** No `Axiom` in CC4's fences; `trusted_base()` before
  == after; no kernel/prelude/`Cargo`/lock delta; no new primitives.
- **AC7 — corpus-wide oracles (BOTH of them — this is the CC3 red-CI lesson).**
  CC4 adds a new catalog file, so it must satisfy **every** test that globs
  `catalog/`:
  - `crates/ken-cli/tests/ken_fmt.rs` (catalog-wide `ken fmt` sweep), and
  - `crates/ken-elaborator/tests/kenfmt_c_capstone.rs` (the live-corpus
    fixed-point arm — the new file must be a `ken fmt` fixed point).
  Run **both** targeted before release. **`FRAME_LINE_COUNTS` is a discharged
  historical baseline — add NO row for `Diagnostic/Core.ken.md`** (a file created
  after the frame has no honest pre-frame count; the row would be fabricated and
  its check vacuous). CC3 already re-scoped that oracle to a coverage check, so
  a new file no longer breaks it — **keep it that way.**
- **AC8 — scope discipline.** Only: `Diagnostic/Core.ken.md`, the three consumer
  packages, the AC1 harness, and re-points of the three existing harnesses.
  **`Parsing/Decoder.ken.md` should NOT need to change at all** (fixed input 2).

## Do-not-reopen guardrails

- **No rendering** — `Doc`/formatting/width is **CC5**. A `Diagnostic` that can
  print itself has stolen CC5's job and coupled the value to a presentation.
- **No fifth origin**, and no payload beyond a named consumer's need. The four
  are committed (CC8 consumes Environment/ConfigKey); a fifth would be
  speculative.
- **Do NOT re-type or re-home `Decoder`** — it is already loc-generic; clients
  instantiate at `Origin` (fixed input 2).
- **Do NOT move `Span`** — its `ValidSpan` laws are `Source`-relative.
- **No `import`/`pub` smuggling** — escalate package-model gaps.
- **No `int_to_nat` / `Int` destructor** — not needed here (fixed input 4); if
  you think it is, the shape drifted. Escalate.
- **No parallel carriers left alive** (AC5) — subsume, don't proliferate.

## Sequencing & review chain

Foundation builds → Foundation QA → **Architect** (soundness/design; he should
press hardest on **whether the three subsumptions are genuine or wrappers with
the old carriers still alive underneath** — the CC3 question, which is the one
that actually distinguishes this WP from a rename) → **CV only if
`conformance/` is touched** (CC4 need not touch it; adding a seed pulls a
required CV vote — do it deliberately or not at all) → `git_request` to the
Steward → honesty gate + CI-poll publish. CC4 closes when it lands **and** its
§10 retros are in.

**Carried from CC3 (apply, don't re-derive):** the ordered shared-`ElabEnv`
harness is AC1, not an afterthought; AC discrimination must be **literal and
exact** (exact index, exact range — not "an error occurred"); a **historical
ledger is never a live-corpus identity contract**; and if a fixed input is false
against the landed code, **escalate with exact tree anchors** rather than
building around it — that is what saved CC3 twice, and it is why this frame ran
a dependency-DAG check and a constructibility audit before pinning anything.
