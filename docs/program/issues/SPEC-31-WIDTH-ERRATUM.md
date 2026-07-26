---
id: SPEC-31-WIDTH-ERRATUM
title: "spec 31-lexical mandates a 96-column canonical width while the formatting conformance suite asserts 88 in 18 places and cites 31 §1d as its source — rule the exact value and reconcile"
status: draft
owner: spec
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: Found by conformance-validator during the SPEC-ALIGN-A1 census, reported evt_3jpxb2qhkx2d0 (2026-07-26), Steward-verified at exact origin/main=cf8924a8. Explicitly carved OUT of SPEC-ALIGN-A1: it is not an over-specification candidate and A1 is forbidden from moving a conformance row. Sibling of the closed SPEC-38-ERRATUM (spec 38-ffi-io self-contradicts on the transfer bound). Steward-filed per COORDINATION §2.
---

> ## ⛔ A LIVE SPEC↔CONFORMANCE CONTRADICTION — three artifacts, two values
>
> ⚠ **It is LATENT, not failing, and that is the hazard.** The affected
> conformance rows are `RED-UNTIL-BUILT (B3/B4/C)`, so nothing reddens today.
> **When the formatter gate goes live against the landed `CANONICAL_WIDTH`, the
> formatting gate fails — and it will present as an implementation bug rather
> than as a stale conformance value.**

## Measured at exact `origin/main=cf8924a8`

| artifact | value | site |
|---|---|---|
| **spec** | **96** Unicode display columns | `spec/30-surface/31-lexical.md:124`, in **§1d** (heading `:121`); also `:332` for the comment-fit rule |
| **landed code** | **96** | `crates/ken-elaborator/src/layout.rs:12` — `pub const CANONICAL_WIDTH: usize = 96;` |
| **conformance** | **88** | `conformance/surface/formatting/seed-canonical-format.md` — **18** occurrences of `88`, **0** of `96` |

⛔ **And the conformance suite cites the contradicting section as its own
source:** `:173` and `:384` both read `- spec: `31 §1d` (88 display columns …)`.
**`31 §1d` is exactly the section that says 96.**

**What is built on the wrong value:**

- **`FMT7`** (`:169`–`:187`) — *"deterministic 88-column property (gate 7)"*,
  with paired `88`/`89` fixtures across declaration header, arrow chain,
  application, match arm, effect row, contract, refinement, and
  record/class/instance field, including Unicode glyphs whose byte length differs
  from display width. ⚠ Its stated purpose is *"the 88/89 pair fixes both
  boundary orientation and display-width counting"* — a **boundary-orientation
  pin at the wrong boundary**.
- **the comment threshold** (`:610`–`:658`) — *"comments-pin-hard-lines-and-the-88-threshold"*,
  where *"`code + two spaces + comment` exactly 88 display columns"* is
  load-bearing, against a spec rule (`31:332`) that says **96**.
- `:479` — *"every line would fit within 88 columns."*

## What needs deciding, and by whom

⛔ **Which value is correct is not the enclave's to settle unilaterally**, because
the landed formatter already implements one of them. Two questions, in order:

1. **⭐ Which width is normative — 96 or 88?** ⚠ Note the asymmetry: **two
   artifacts say 96** (the spec section and the landed `layout.rs`), **one says
   88** (the conformance suite, 18 times). ⛔ A count is not an argument — the
   question is which was *derived* and which was *inherited*, and neither is
   evidenced here. **Architect.**
2. **Then the reconcile.** Whichever way it goes, one artifact family moves.
   Moving a conformance row is a **conformance-granularity** decision, not spec
   editing. **Architect rules; spec enclave executes.**

⚠ **Do not assume the 18-vs-1 count settles it.** A single deliberate spec
revision from 88 → 96 that never swept `conformance/` produces exactly this
picture — and so does a conformance suite written against a superseded draft.
⛔ Establish which happened before editing either side; the `git log` on both
files is the cheap discriminator and nobody has run it.

## Scope

✅ **In:** the ruling on the exact value; the sweep of **every** site on the
losing side, including `FMT7`'s fixture pairs and the comment-threshold
arithmetic; and the `spec:` citation lines at `:173`/`:384` which currently
attribute a value to a section that does not state it.

⛔ **Out:** whether one canonical width should be normative at all. It should —
one deterministic canonical form is mission-aligned, and `Go`'s precedent is that
a canonical formatter can be tool policy without making layout part of program
meaning. ⚠ That *separate* question (is the exact constant language conformance
or tooling policy?) is the campaign's formatter item and belongs to
`14-spec-mission-alignment-campaign.md`, **not here.** ⛔ This node fixes the
contradiction; it does not decide the constant's status.

## ⭐ Why this was found, and the transferable part

The `SPEC-ALIGN-A1` frame listed formatter width as a private-mechanism
relaxation candidate and cited `seed-canonical-format.md:10` as its conformance
consumer. **`:10` states only the `RED-UNTIL-BUILT` status — it asserts no
width.** Right file, wrong lines, and the wrong lines were topically adjacent
enough to read as supporting the claim.

⇒ ⛔ **A locator has two coordinates, and re-deriving the file is not evidence
about the lines.** The census found this only because it was required to check
what each row *asserts at its lines* rather than trust the citation. That
requirement earned its keep within an hour of the kickoff.
