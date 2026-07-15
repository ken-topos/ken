# WP CC7 — `ArgParse` (the Milestone-C exit criterion)

The payoff. Six work packages of substrate were built for this one: a
**multi-file subcommand tool with options, help, and diagnostics** — the
operator's stated north-star acceptance.

**Program II (catalog closure), CC7.** Owner: **Foundation**. Reviewer:
**Architect** — **CV only if `conformance/` is touched**. Size: **L**.
Base: `origin/main @ 576d223f`. Branch: `wp/cc7-argparse`.

**Zero trust delta** — ordinary kernel-checked catalog Ken; no kernel rule, no
primitive, no postulate, no `Axiom` in CC7's fences.

## Everything you consume already exists — and you built all of it

`Capability.Process.Arguments` (CC6a) · `Capability.Parsing.Decoder` + `Capability.Parsing.Cursor` (CC3) ·
`Capability.Diagnostics.Core` (CC4) · `Capability.Formatting.Doc` (CC5) · `Data.Sums.Validation` +
`Data.Collections.NonEmpty` (CC1) · `Data.Text.Codec` + `Capability.Parsing.Numeric` (CC2).
**Consume them. Do not rebuild any of them, and do not fork a second copy of
anything.**

## Fixed inputs (settled — do NOT reopen)

Grounded against `origin/main @ 576d223f`. **Treat every anchor as perishable.
If a fixed input is FALSE, say so with exact tree anchors and ESCALATE — do not
build around it.** (That clause has caught a bad pin of mine **five** times,
including one where I grepped the wrong file entirely. **I would rather be
corrected than believed.**)

1. **★ THERE IS NO `Bytes` EQUALITY. Option-name matching is BYTE-WISE over
   `ArgBytes`. This is the WP's central constructibility pin.**
   I ran the audit: **no `DecEq Bytes`, no `Ord Bytes`, no `bytes_eq` primitive**
   — CC2 explicitly **descoped** `Bytes` keys (no `Bytes ↔ List UInt8` bridge and
   no extensionality cert exists to ride).
   - **⇒ You CANNOT compare an argv `Bytes` to an option name with a landed
     equality.** Matching `--verbose` means **comparing bytes, one at a time.**
   - **The landed route is CC3's `ArgBytes`** (the proof-carrying cached-`Nat`
     carrier) — recurse on the cached `Nat`, index with `bytes_at`, compare with
     `uint8_to_int` + `eq_int`. **CC6a already consumes exactly this.** It is
     landed, it works, and it needs **no new primitive.**
   - **⇒ Do NOT mint a `bytes_eq` primitive, a `DecEq Bytes` instance, or a new
     carrier.** If you believe you need one, **STOP and escalate** — the
     `Bytes → Nat` bridge is an **open operator decision** and a build WP does not
     get to settle it.

2. **★ THE `Diagnostic → Doc` RENDERER IS A NEW, SEPARATE MODULE — and where it
   lives is a dependency-DAG pin, not taste.**
   It does not exist yet, and **CC5 deliberately forbade it in both base
   modules:**
   - **NOT in `Capability.Diagnostics.Core`** — that would give `Diagnostic` knowledge of
     rendering and **destroy CC4's AC4** (the value knows its *location*, not its
     *rendering*). CC4 landed render-free **on purpose**.
   - **NOT in `Capability.Formatting.Doc`** — that would make the abstraction depend on a
     client. Same cycle CC3 and CC4 both had to correct.
   - **⇒ It lives in a THIRD module: `catalog/packages/Capability/Diagnostics/Render.ken.md`**,
     depending on **both** `Capability.Diagnostics.Core` and `Capability.Formatting.Doc`. **It is a CC7
     deliverable**, but it is a **separate package** — do not bury it inside
     `ArgParse`, because CC8 (the env/config decoder) will want it too.

3. **Explicit specs — NO reflection, NO macros, NO derivation in v1.** The
   `CommandSpec` / `OptionSpec` / `PositionalSpec` datatypes are **written out**.
   The report pins this and it is settled. **A derivation mechanism is not in
   scope and is not a stretch goal.**

4. **`ArgParse` is a SPECIALIZATION of what exists, not a new universe.** Argv
   tokenization runs on `Capability.Parsing.Decoder` over an **`ArgCursor`** (CC3);
   failures are **`Diagnostic`s** (CC4) with **`ArgumentOrigin`** locations;
   multiple independent failures **accumulate via `Validation`** (CC1) rather
   than short-circuiting; usage/help **derives a `Doc`** (CC5). **If you find
   yourself writing a second parser, a second error carrier, or a second
   renderer, STOP — that is the failure this catalog exists to prevent.**

5. **Byte-preserving throughout.** Argv values are `Bytes`. **Never decode to
   `String`** to compare, match, or store. Decoding is the **caller's** explicit
   choice (`Data.Text.Codec`), never something `ArgParse` does for them. **A quiet
   decode passes every UTF-8 test and breaks only on real non-UTF-8 argv** —
   CC6a proved this guarantee holds; **do not be the WP that breaks it.**

6. **Package model — unchanged.** No cross-file `import`/`pub`; dependency-bearing
   packages elaborate **in order into ONE shared `ElabEnv`** (AC1). A standalone
   `ken check` of a dependent package is **expected to fail** — the known
   package-model gap. **Escalate; do not smuggle `import`.**

7. **Homes:** `ArgParse` → `catalog/packages/Application/CommandLine/ArgParse.ken.md` (or
   `Application/CommandLine/ArgParse/Core.ken.md` if you split);
   `Capability.Diagnostics.Render` →
   `catalog/packages/Capability/Diagnostics/Render.ken.md`.

## Mandated deliverable outline

1. **`Capability.Diagnostics.Render`** (fixed input 2) — `Diagnostic → Doc`. The renderer
   decides *presentation*; the `Diagnostic` still knows only its origin. Keep it
   small and general: **CC8 will consume it.**

2. **The spec datatypes** — `CommandSpec` / `OptionSpec` / `PositionalSpec`,
   explicit (fixed input 3). Enough to express: subcommands, long/short options,
   options with values, flags, positionals, and required-vs-optional.

3. **Tokenization + decoding** — over `ArgCursor` via `Capability.Parsing.Decoder` (fixed
   input 4), byte-wise option matching (fixed input 1), values validated through
   `Validation` so **multiple errors accumulate** with **exact `ArgumentOrigin`
   locations** (arg index + byte range).

4. **Usage/help as a `Doc`** — derived from the `CommandSpec`, rendered
   deterministically at a caller-chosen width. **The spec is the single source of
   truth**: help text is *derived*, never hand-written alongside the spec, or it
   will drift.

## Acceptance criteria (testable)

- **AC1 — DS-7/8 ordered shared-`ElabEnv` harness** (`cc7_argparse_acceptance.rs`),
  full dependency closure elaborated **IN ORDER**, then every checked fence.
  **NOT a standalone `ken check`.**
- **AC2 — ★ THE MILESTONE ACCEPTANCE: a multi-file subcommand tool.** A worked
  example with **≥2 subcommands**, **options with values**, **flags**, and
  **positionals**, that: parses a real argv; **renders its own help from its
  own spec**; and on bad input produces a **located diagnostic**. **This is the
  operator's north star — if this example is not convincing, CC7 is not done.**
- **AC3 — errors ACCUMULATE, and locate exactly.** Two independent bad arguments
  produce **TWO** diagnostics (not one — `Validation`, not first-error
  short-circuit), each with the **exact** `(arg index, byte range)`. **Assert
  both, at non-zero index and non-zero offset.**
- **AC4 — byte-preserving, discriminated.** A **genuinely invalid UTF-8** option
  *value* survives parsing **byte-identically** (fixed input 5). A UTF-8-only
  test cannot catch a quiet decode — **use real invalid bytes**, as CC6a did.
- **AC5 — help is DERIVED, not duplicated.** Changing the `CommandSpec` changes
  the rendered help **with no other edit**. (Discriminating: add an option to the
  spec and assert it appears in the rendered help.)
- **AC6 — no second universe.** No new parser, no new error carrier, no new
  renderer, no new cached-`Nat` carrier, **no `bytes_eq`/`DecEq Bytes`**. `ArgParse`
  consumes CC1–CC6a. **Grep-able, and it is the WP's real test** (fixed inputs
  1, 4).
- **AC7 — zero trust delta.** No `Axiom` in CC7's fences; `trusted_base()` before
  == after; no kernel/prelude/`Cargo`/lock delta; **no new primitives.**
- **AC8 — corpus-wide oracles (BOTH).** New catalog files ⇒ run **both**
  `crates/ken-cli/tests/ken_fmt.rs` **and**
  `crates/ken-elaborator/tests/kenfmt_c_capstone.rs`. **Add NO row to
  `FRAME_LINE_COUNTS`.**

## Do-not-reopen guardrails

- **No `bytes_eq` / `DecEq Bytes` / new carrier** — byte-wise over `ArgBytes`
  (fixed input 1). The `Bytes → Nat` bridge is an **open operator decision**;
  **a build WP does not settle it.**
- **`Capability.Diagnostics.Render` is its OWN module** — not inside `Capability.Diagnostics.Core`
  (breaks CC4's render-free property) and not inside `Capability.Formatting.Doc` (cycle).
- **No reflection / macros / derivation** — explicit specs (fixed input 3).
- **No decoding argv to `String`** (fixed input 5).
- **No second parser / error carrier / renderer** (AC6).
- **No `import`/`pub` smuggling** — escalate package-model gaps.

## Sequencing & review chain

Foundation builds → Foundation QA → **Architect** (he should press hardest on
**AC6 — is `ArgParse` genuinely a specialization of CC1–CC6a, or has a second
universe grown inside it?** That is the difference between the catalog compounding
and the catalog fragmenting) → CV only if `conformance/` is touched →
`git_request` to the Steward → honesty gate + publish. CC7 closes when it lands
**and** its §10 retros are in.

**On CC7's merge, Milestone C is met.** Remaining after: CC8 (env/config decoder
— the *second* description-driven decoder, which is what finally justifies
extracting a shared `Schema`, and **not before**) and CC9 (`Resource`/`Bracket` +
`Tooling.Testing.Property`).
