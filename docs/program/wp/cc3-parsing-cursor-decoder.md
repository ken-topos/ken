# WP CC3 — `Parsing.Cursor` + progress-safe `Decoder` (and the CAT-5 refactor)

Land the catalog's cursor abstraction and its progress-safe decoder
combinators, and **refactor CAT-5 to consume them** — so Ken has *one* parsing
universe, not two. Two real instances justify the abstraction: `ByteCursor`
(over CAT-5's `Source`) and `ArgCursor` (over `List Bytes`, preserving arg
index + byte range).

**Program II (catalog closure), CC3.** Owner: **Foundation**. Reviewer:
**Architect** (soundness/design) — **CV only if `conformance/` is touched**.
Size: **L**. Base: `origin/main @ e22f5688`. Branch:
`wp/cc3-parsing-cursor-decoder`.

Thread all CC3 activity in its kickoff thread. **Zero trust delta** — ordinary
kernel-checked catalog Ken; no kernel rule, no primitive, no postulate, no
`Axiom` inside CC3's fences.

## Fixed inputs (settled — do NOT reopen)

Grounded against `origin/main @ e22f5688`. **Treat every line/anchor below as
perishable — re-verify against the landed code at pickup, not against this
frame.**

0. **Landed substrate (grounded — DO NOT rebuild).**
   - **CAT-5** (`catalog/packages/Capability/Parsing/Parsing.ken.md`) already
     defines: `Source` (a `SourceId` + immutable `Bytes` + length + a UTF-8
     proof), `Span`, `Located a`, `ParseError`, `ParseResult a = Parsed a Span
     Nat | Failed ParseError`, and
     ```
     const Parser (a : Type) : Type =
       (s : Source) → (start : Nat)
       → LessEqNat start (source_length s) → ParseResult a
     ```
     (`:250-251`) — note the **in-bounds proof is a caller obligation**, not a
     runtime check. Its well-formedness laws (`ParserValid` / `ParserTotal` /
     `ParserSourceLocal` / `ParserLaws`) are **plain predicates over a
     `Parser a`**, checkable per concrete parser — not enforced by the type.
     Base combinators: `parser_pure`, `parser_fail`.
   - **CAT-5 already uses fuel-bounded recursive descent**
     (`parse_bool_expr_at_fuel`, `skip_spaces_fuel`, `:324-329`, `:492`), and
     its own prose states exactly why: the real termination measure —
     unconsumed input — *"is not syntactically a subterm of the fuel itself."*
     Critically, **the fuel is seeded from `source_length s`** — a real upper
     bound on remaining bytes, **not** an arbitrary caller-supplied budget.
   - **CAT-5's acceptance test is `crates/ken-elaborator/tests/
     cat5_parsing_package.rs`.** It is the anti-regression net (AC2).
   - **CC1** (`Data/NonEmpty`, `Data/Validation`) and **CC2** (`Text/Codec`,
     `Text/Numeric`, `Text/StringKeys`) are landed and may be depended on.

1. **THE TERMINATION PIN — fuel seeded from the cursor's `remaining`, PLUS a
   stated progress law. This is the WP's central design decision; it is
   SETTLED.** The report left "a progress proof **or** explicit fuel" open. The
   landed code settles it, and the pin is the *combination*, not either half:
   - **Mechanism = fuel** (a `Nat` decremented per step). A genuine
     well-founded descent on "elements remaining" is **not** syntactically
     structural — `remaining − k` is not a subterm of `remaining` — so the
     termination checker cannot see it. CAT-5 hit this exact wall and its
     comment records the finding. **Do not spend the WP re-litigating it.**
   - **Fuel is SEEDED FROM the cursor's `remaining` count**, never from a
     caller's guess. An arbitrary budget makes a parse fail for a
     **non-semantic** reason (budget exhaustion on legal input) — that is a
     defect, not a design.
   - **The progress law is what makes the seeded fuel SUFFICIENT.** State it:
     *a successful decoder step strictly advances the cursor position*
     (consumption ≥ 1). Given that, a repetition can succeed at most
     `remaining` times, so **fuel seeded from `remaining` can never run out
     before the input does.** Fuel stops being a budget and becomes a *derived*
     bound.
   - **⇒ The observable contract (this is the AC, not the mechanism):
     repetition NEVER fails for lack of fuel.** AC3 is the net.
   - **Escalation (pre-authorized):** if you find a formulation the termination
     checker accepts as a *genuine* well-founded descent with **no fuel
     parameter at all**, that is strictly better — take it. But only if it
     needs **zero** new trust and preserves the same observable contract.
     **Report which one landed** either way; do not silently substitute.

2. **The cursor abstraction = an explicit operations RECORD (a dictionary), NOT
   a type class.** Ken has no associated types, so the element type and the
   location type must be **explicit parameters**; and where a class parameter
   would be an abstract type variable, the settled preference is the explicit
   dictionary. So, in shape (exact field list is yours):
   ```
   data CursorOps c el loc =
     MkCursorOps (c → Nat) (c → Option el) (c → c) (c → loc)
   --             remaining  peek            advance  locate
   ```
   Across their two homes, the instances are two **values** of this record —
   `byte_cursor_ops` and `arg_cursor_ops` — **not** `instance` declarations.
   Each instance lives with the carrier it is over: `arg_cursor_ops` lives in
   `Parsing.Cursor`, while
   `byte_cursor_ops` lives downstream in CAT-5 beside CAT-5's `Source` and
   `Span`. This acyclic home split supersedes the original frame wording that
   placed both values in `Parsing.Cursor`. If the explicit-dictionary shape
   genuinely cannot be made to work, **escalate to the Architect**; do not
   reach for a class with a higher-kinded or associated-type parameter.

3. **Location stays PARAMETERIZED — CC3 does NOT build the origin-neutral
   diagnostic.** That is **CC4** (`Diagnostic.Core`, which generalizes
   `SourceId+Span` to `SourceOrigin`/`ArgumentOrigin`/…). In CC3:
   `ByteCursor`'s `loc` is CAT-5's existing `Span`; `ArgCursor`'s `loc` is its
   own carrier (**arg index + byte range**). CC4 **subsumes** both later
   (reflect-don't-extend). **Do not pre-empt it** by inventing an origin-neutral
   diagnostic here — you would be designing CC4 blind, with one consumer.

4. **`ArgCursor` is a CURSOR INSTANCE ONLY.** No argv tokenization, no
   `CommandSpec`/`OptionSpec`, no usage/help rendering, no `--`-handling policy
   — those are **CC6/CC7**. Because `bytes_length : Bytes → Int` has no landed
   `Int → Nat` bridge, CC3's carrier is a proof-carrying wrapper whose payload
   is `List Bytes`: it caches each argument's length as `Nat` and carries
   `SourceLength`-style evidence that every cached length agrees with the
   opaque byte length. Concretely, the carrier is a list of length-certified
   byte values, so the remaining bound is the sum of the cached per-argument
   lengths minus the current byte offset. `ArgCursor` preserves **arg index +
   byte range** in its location. This cached-length carrier supersedes the
   original raw-`List Bytes` wording; it adds no primitive or trusted
   conversion.

5. **Package model — NO cross-file `import`/`pub` smuggling.** The catalog has
   **no disk loader** (`07-catalog-style-guide.md §13`, final bullet): a
   dependency-bearing catalog package is **elaborated in dependency order into
   ONE shared `ElabEnv`** (the CC1/CC2 pattern — see AC1). A standalone `ken
   check` of a *dependent* package is **EXPECTED to fail**; that is the known
   package-model gap, **not a bug to work around**. If the model genuinely
   blocks you, **ESCALATE to the Steward** — do not invent `import`/`pub` to
   route around it.
   - **This makes CAT-5 dependency-bearing for the first time** (it is
     currently a self-contained leaf by the §13 self-containment choice). That
     is **intended** — it is the whole point of "don't build a second parsing
     universe" — and it costs nothing in CI: there is **no standalone catalog
     check gate**; `crates/ken-cli/tests/ken_fmt.rs:91-92` sweeps
     `catalog/**.ken.md` for **formatting only** (AC6).

6. **Homes (pinned, per §13's identity map — N dotted components → N−1
   directories + a leaf):** `Parsing.Cursor` → `catalog/packages/Parsing/
   Cursor.ken.md`; `Parsing.Decoder` → `catalog/packages/Parsing/Decoder.ken.md`.
   CAT-5 **stays put** at `Capability/Parsing/Parsing.ken.md` — do **not** move
   or rename it (pure churn, and it would touch every reference in the corpus).
   If `06-catalog-campaign.md` carries a Section registry, add `Parsing` to it
   (doc-only).

## Mandated deliverable outline

Each section ends in a concrete, implementable choice — not a survey.

1. **`Parsing.Cursor`** — the `CursorOps` record (fixed input 2); the
   **progress law** and the bounds/validity laws as plain predicates over a
   cursor (mirroring CAT-5's `ParserValid`-style posture: checkable per
   instance, not enforced by the type); and **`arg_cursor_ops`** over the
   proof-carrying cached-length `List Bytes` wrapper — position is **(arg
   index, byte offset within that arg)**; `locate` → an arg-index + byte-range carrier.
     Crossing an arg boundary is an ordinary advance, not a special case.

2. **`Parsing.Decoder`** — progress-safe combinators over a `CursorOps`:
   `pure`, `fail`, `map`, `bind`/`seq`, `alt`, `satisfy`/`token`, and the
   repetition family (`many`/`some`) with **fuel seeded from `remaining`**
   (fixed input 1). State the progress obligation on the step decoder, and
   state the repetition's own law (it consumes the whole input when its step
   always progresses).

3. **The CAT-5 refactor — the subsumption.** Re-express
   `Capability/Parsing`'s worked Boolean grammar as a **`Decoder` over
   `byte_cursor_ops`**, defined in CAT-5 beside the `Source`/`Span` carrier it
   is over (`remaining = source_length s − pos`, `peek` via the landed
   `bytes_at` path, `locate` → a CAT-5 `Span`). `Parser a` becomes a
   *specialization* of the decoder
   rather than a parallel universe. CAT-5's landed laws and its worked grammar
   **must survive** (AC2). Delete CAT-5's now-subsumed bespoke recursion
   (`parse_bool_expr_at_fuel` / `skip_spaces_fuel`) **only** to the extent the
   decoder genuinely replaces it — a leftover second mechanism is the failure
   this WP exists to prevent.

## Acceptance criteria (testable)

- **AC1 — DS-7/8 ordered shared-`ElabEnv` acceptance harness.** New test
  `crates/ken-elaborator/tests/cc3_parsing_cursor_decoder_acceptance.rs`,
  following `cc2_text_codec_numeric_acceptance.rs`: ONE shared `ElabEnv`, the
  dependency closure elaborated **IN ORDER** — Transport → Collections →
  LawfulClasses → [NonEmpty/Validation if used] → **Parsing.Cursor** →
  **Parsing.Decoder** → **Capability.Parsing (refactored CAT-5)** — then every
  checked literate fence; assert the checked globals are **real, transparent,
  kernel-checked terms**. **This is AC1 — NOT a standalone `ken check`.**
- **AC2 — the CAT-5 anti-regression net: `cat5_parsing_package.rs` stays
  GREEN.** Every existing *discriminating* assertion must survive the refactor:
  the Boolean grammar still parses `(and true (not false))`, still **rejects**
  `true and false`, and its spans are still **exact**. If the refactor forces a
  signature change, the test may be re-pointed — but a re-point that drops or
  weakens an assertion is a **regression**, not a re-point.
- **AC3 — the progress discriminator (LOAD-BEARING — this is what proves the
  termination pin).** Two arms, both required:
  - **(a) Zero-consumption under repetition must fail LOUDLY.** `many` of a
    step that consumes nothing (`pure`, or the moral equivalent) must either be
    **statically unconstructible** (the progress obligation cannot be
    discharged) or fail with a **NAMED** error variant — **assert the named
    variant, never `is_err`**. It must not silently loop, silently truncate, or
    quietly return `[]`.
  - **(b) Fuel never exhausts before the input does.** Run a repetition whose
    step consumes ≥1 over a **long** input and assert it consumes the **whole**
    input and lands on the **exact** end position. This is the arm that
    discriminates *fuel-seeded-from-`remaining`* from *an arbitrary budget* — a
    budget-based impl fails it. A short input would pass either way, so **the
    input must be long enough that any plausible fixed budget would run out.**
- **AC4 — two real instances, both exercised, at NON-DEGENERATE positions.**
  - **ByteCursor:** CAT-5's grammar runs end-to-end through it (AC2 covers the
    semantics).
  - **ArgCursor:** decoding a **multi-arg** `List Bytes` reports a failure at
    **arg 2, byte 3** (or equivalent) with the **exact** `(arg index, byte
    offset)` — a **non-zero index AND a non-zero offset**, so a location-free,
    arg-0-only, or off-by-one implementation fails. (A single-arg or
    offset-0 case would pass a broken impl — that is the trap.)
- **AC5 — zero trust delta.** No `Axiom` inside CC3's own fences;
  `trusted_base()` before == after; no kernel/prelude/`Cargo`/lock delta; no
  new primitives.
- **AC6 — kenfmt-canonical.** The new catalog files pass the catalog-wide
  formatting gate (`crates/ken-cli/tests/ken_fmt.rs:91-92` sweeps **all** of
  `catalog/**.ken.md`). Run `ken fmt` on them before release.
- **AC7 — scope discipline.** Only: the two new `Parsing/` catalog packages,
  the CAT-5 refactor, the AC1 harness test (+ the optional Section-registry doc
  line). Nothing else.

## Do-not-reopen guardrails

- **ONE parsing universe.** The whole point is subsumption — if CAT-5 ends up
  with *both* a decoder client path and its own surviving bespoke recursion,
  the WP has failed its objective even if every test is green.
- **No origin-neutral `Diagnostic`** — that is CC4, and it needs *two*
  consumers to design against. Keep locations parameterized (fixed input 3).
- **No argv tokenization / `CommandSpec` / usage rendering** — CC6/CC7 (fixed
  input 4).
- **No arbitrary fuel budget** — fuel is seeded from `remaining` (fixed input
  1). A parse that fails on legal input for lack of budget is a defect.
- **No `import`/`pub` smuggling** — escalate package-model gaps (fixed input 5).
- **Do not move or rename `Capability/Parsing`** (fixed input 6).
- **Do not de-duplicate CAT-5's local `list_append` copy.** It is a deliberate
  §13 self-containment choice. De-duping it widens the harness and the diff for
  no gain in this WP — leave it, and note it as a fast-follow if the refactor
  makes it incoherent.
- **No speculative combinators.** Ship what the two instances + the CAT-5
  grammar actually need. No `Schema`, no generic parser-combinator zoo.

## Sequencing & review chain

Foundation builds → Foundation QA → **Architect** review (soundness/design: the
`CursorOps` shape, the progress law, and — the one he should press hardest —
whether the CAT-5 refactor is a **genuine subsumption** or a wrapper with the
old universe still alive underneath) → **CV only if `conformance/` is touched**
(CC3 need not touch it; adding a conformance seed pulls a required CV vote, so
do that deliberately or not at all) → `git_request` to the Steward →
honesty-gate + CI-poll publish. CC3 closes when it lands **and** its §10 retros
are in.

**Carried from CC2's retros (apply, don't re-derive):** the ordered
shared-`ElabEnv` harness is AC1, not an afterthought; AC discrimination must be
**literal and exact** (exact index, exact position — not "an error occurred");
and keep the frame in sync with what actually lands — if a fixed input turns out
false against the landed code, **say so and escalate**, don't quietly build
around it.

**A signal worth reporting:** two explicit-dictionary instances (`byte_` /
`arg_cursor_ops`) is the **second occurrence** of the pass-the-dictionary idiom
in the catalog. If you find yourself wanting a third, that is a
**language-feature signal** (the ergonomics of instance resolution), not a cue
to hand-roll more plumbing — **flag it to the Steward** rather than absorbing
it.
