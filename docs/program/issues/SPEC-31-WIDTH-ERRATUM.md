---
id: SPEC-31-WIDTH-ERRATUM
title: "spec 31-lexical mandates a 96-column canonical width while the formatting conformance suite asserts 88 in 18 places and cites 31 §1d as its source — rule the exact value and reconcile"
status: active
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
- **the comment threshold** (`:610`–`:658`) —
   *"comments-pin-hard-lines-and-the-88-threshold"*,
  where *"`code + two spaces + comment` exactly 88 display columns"* is
  load-bearing, against a spec rule (`31:332`) that says **96**.
- `:479` — *"every line would fit within 88 columns."*

## ✅ RULED BY THE OPERATOR, 2026-07-26 — **96 IS NORMATIVE**

> **Operator:** *"re 88 v 96. 96 is what it should be. It was an incomplete
> revision, apparently."*

⇒ ⛔ **The value question is CLOSED. Do not re-ask it, and do not re-argue it
from the 18-vs-1 count.** The count was never the argument; the operator's
ruling names the **cause** — an incomplete revision — which is exactly the
derived-vs-inherited question this node said was unevidenced. **`conformance/` is
the side that carries a superseded value.**

⇒ **`spec/30-surface/31-lexical.md` and `crates/ken-elaborator/src/layout.rs` are
correct and DO NOT CHANGE.** The reconcile moves `conformance/`.

### ⛔ AND IT IS NOT A `sed` — the arithmetic has to be re-derived

**This is the one thing most likely to be got wrong**, because `88` → `96` looks
like a substitution and is not:

1. ⛔ **`FMT7`'s fixtures are a BOUNDARY PAIR, not a constant.** `:169`–`:187`
   builds paired fixtures *"at display widths 88 and 89"* across eight syntactic
   forms, whose stated purpose is *"the 88/89 pair fixes both boundary
   orientation and display-width counting."* ⇒ The pair becomes **96/97**, and
   **every fixture must be re-authored to those widths** — a fixture whose text
   is 88 columns wide is simply *flat* at a 96 limit and tests nothing. ⚠ A
   number-only edit leaves eight fixtures **silently vacuous while reading as
   updated.**
2. ⛔ **The comment threshold is derived arithmetic.** `:639` — *"makes `code` +
   two spaces + `comment` exactly 88 display columns"* — is a computed
   construction against a spec rule (`31:332`) that says **96**. The code and
   comment lengths must be re-derived so the sum is exactly 96, and the `89`
   partner becomes `97`.
3. ⛔ **Fix the false attributions too.** `:173` and `:384` read
   ``- spec: `31 §1d` (88 display columns …)`` — they attribute a value to a
   section that states 96. **Those citation lines are part of the defect, not
   collateral.**
4. ⚠ **The Unicode cases are the reason the pair exists.** `FMT7` deliberately
   includes *"Unicode glyphs whose UTF-8 byte length differs from display
   width"*, because a byte-counting implementation flips exactly one arm. ⇒ The
   re-authored fixtures must preserve that property at the new boundary, or the
   pin stops discriminating byte-counting from display-counting — which is
   **most** of what it was for.

⇒ ⭐ **Required control:** for each re-authored pair, show the **`96` arm stays
flat and the `97` arm breaks.** A pair where both arms break, or both stay flat,
is not a boundary pin. ⛔ `18` occurrences changed is not evidence; a passing
discriminating pair is.

## Sequencing — ✅ PRECONDITION DISCHARGED, RELEASED 2026-07-27

⛔ **Not concurrent with `SPEC-ALIGN-A1`** — same ring (spec enclave). ✅ **A1 is
`merged`** (PR #1028, `origin/main = 4c2d9529`, verified on `main`), so the
condition *"releases when A1 closes"* is met and **this node is released to the
spec enclave** — kickoff `evt_629qvns1n7j7d`, at `origin/main = a1e29284`.
⚠ A1 was forbidden from touching it: A1 may not move a conformance row, which is
why this was carved out in the first place.

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
