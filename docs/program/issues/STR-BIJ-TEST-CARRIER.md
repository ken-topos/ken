---
id: STR-BIJ-TEST-CARRIER
title: "The AC2 reverse-direction test claims a universal inverse and its sole operand is an NFC fixed point — it is green under the correct law AND under the false one it pins"
status: ready
owner: language
size: S
gate: none
depends_on: [STR-BIJ]
blocks: []
github: null
origin: "conformance-validator block on STR-BIJ exact bcdd4548 (evt_35vfffa5pm230); Steward scope ruling 2026-07-27 narrowed STR-BIJ to prose/spec carriers and named this the explicit tracked exclusion. Frame: docs/program/wp/str-bij-overclaim-erratum.md (supplies the discriminating operand in its §2 repro)."
---

> ## ⛔ READ THIS FIRST — THE FIX IS **TWO** CHANGES AND EITHER ALONE MAKES THINGS WORSE
>
> This test has **two independent defects**: a **false universal carrier** (its
> name, header, doc, and assertion message) and a **vacuous operand** (its sole
> input is an NFC fixed point).
>
> ⛔ **A wording-only rename is strictly worse than leaving the test as it is.**
> The over-claiming name is currently the **only** signal that this test cannot
> discriminate. Renaming it removes the tell and leaves the test exactly as
> blind — the next reader finds an honestly-named test and no reason to look at
> its operand. ⭐ An over-claiming name on a vacuous test is doing accidental
> documentation work; do not delete it without fixing the vacuity **in the same
> act**.
>
> ⛔ Equally, adding the operand and leaving the name is a test that discriminates
> while still advertising a law that does not hold.
>
> ⇒ **Both halves, one commit.**

## Objective

Make `crates/ken-elaborator/tests/l3_strings_roundtrip_acceptance.rs:145–205`
say what is true and **test something that can fail**.

`STR-BIJ` corrected every prose/spec carrier of the retired
`String`/`List Char` "bijection" claim. This operational acceptance carrier was
its **named exclusion** — outside that WP's authorized seven paths and outside
its no-test/no-Rust guardrail — ⛔ **not** an accepted survivor and ⛔ not a
silent omission.

## The defect, measured at `origin/main` (blob `0eed416d`)

### (a) The false universal carrier — four coupled sites

| site | current text |
|---|---|
| `:145` section header | `AC2: round-trip identity (non-circular defining oracle)` |
| `:189` test name | `ac2_round_trip_s2l_l2s_identity` |
| `:182–187` doc | `string_to_list_char (list_char_to_string cs) ≡ cs` "for a well-formed `cs`" … "a genuinely **independent direction** from AC2's first test — a round-trip pinned from both ends nets the inverse-error pair a one-directional check would miss" |
| `:203` assert message | `"s2l(l2s cs) must reproduce cs's codepoints exactly"` |

⛔ **`s2l ∘ l2s ≡ id` is not a landed law and is not true.** `l2s` NFC-normalizes
(`spec/30-surface/37-strings-collections.md:90`), so it is **not injective** and
has no left inverse. The landed axiom is the **`String`-side retraction only** —
`l2s (s2l s) ≡ s` — from which `s2l` is **injective**, which is all that
`String` canonicity and later `DecEq`/`Ord` transport need.

⚠ The **first** AC2 test (`ac2_round_trip_l2s_s2l_identity`, `:150`) is the
landed direction and is **correct** — ⛔ do not touch its ten-string boundary
corpus. Only the header above it over-reaches by naming the pair.

### (b) ⭐⭐ The vacuous operand — this is the load-bearing half

The reverse test's **sole** operand is
`Cons Char 65 (Cons Char 66 (Nil Char))` → codepoints `[65, 66]` ("AB").

**That is an NFC fixed point.** ⇒ The test is **green under the correct
normalizing `l2s` and green under the false universal inverse it claims to
pin.** It is **green-vs-green on the normalization axis** — the one axis its own
name asserts.

⛔ **So the assertion is valid for its operand and establishes nothing about the
property.** A test that cannot fail when its stated law is false is not evidence
about that law. This is the same honesty over-claim `STR-BIJ` exists to remove,
in executable rather than prose form.

## Fixed inputs — ⛔ the discriminating operand is ALREADY DERIVED, do not re-derive it

`docs/program/wp/str-bij-overclaim-erratum.md §2` supplies it, and both elements
are valid `Char` per `37 §2.4` (both are Unicode scalar values):

```
cs₁ = [U+0065, U+0301]   -- "e" + COMBINING ACUTE ACCENT  (NOT an NFC fixed point)
cs₂ = [U+00E9]           -- "é" precomposed

l2s cs₁  ≡  l2s cs₂          -- NFC identifies them  ⇒  l2s is NOT injective
s2l (l2s cs₁) ≡ cs₂ ≠ cs₁    -- the List-Char-side round trip is NOT the identity
```

⇒ In codepoints: input `[101, 769]`, correct output **`[233]`**.

| input | pin |
|---|---|
| test file | `crates/ken-elaborator/tests/l3_strings_roundtrip_acceptance.rs`, blob **`0eed416d`** at `origin/main` — carrier at **`:145–205`** |
| spec | `spec/30-surface/37-strings-collections.md:90` (the `List Char → String` row states the NFC normalization) |
| landed axiom | `catalog/packages/Data/Text/StringBijection.ken.md:13–14` — `string_to_list_char_retraction`, the `String` side only |
| frame | `docs/program/wp/str-bij-overclaim-erratum.md` (§2 repro = the operand above) |

## Acceptance criteria

| AC | claim | control |
|---|---|---|
| `AC-C1` | The reverse test's operand set includes a **non-NFC-fixed-point** `List Char` — `[U+0065, U+0301]` — and asserts the **normalized** result `[233]`. | ⭐⭐ **The discrimination control, and this AC's whole point:** assert `[233]`. ⛔ Asserting `[101, 769]` (the input, i.e. what a true inverse would return) must **FAIL**. Show both readings and which one the test takes — a green test whose expectation you cannot state in two distinguishable ways has not discriminated anything |
| `AC-C2` | All four carriers in (a) state the **landed** property: the retraction `l2s (s2l s) ≡ s` and `s2l`'s derived injectivity. ⛔ No "identity", no "inverse", no "both ends", no "reproduce exactly" on the reverse direction. | grep the four sites; each must name **retraction** or **injectivity**, ⛔ never a two-sided claim. The `:145` header must stop naming the **pair** as an identity |
| `AC-C3` | The **existing** landed-direction test and its corpus are unchanged. | `ac2_round_trip_l2s_s2l_identity` (`:150`) and its ten-string corpus are **byte-identical**. ⛔ This WP removes an over-claim; it must not remove coverage |
| `AC-C4` | ⭐ **A positive control that the new operand is actually reaching `l2s`'s normalizer** — not being rejected, silently truncated, or lost in `Char` literal construction. | ⛔ Required, because *"the assertion passed"* and *"the operand never got built"* are the same green. Show the intermediate `String` (or its codepoints) differs from a naive UTF-8 encoding of the input — i.e. that normalization **happened** |
| `AC-C5` | No behavior change anywhere else: no axiom, law, instance, spec, or catalog edit. | ⛔ This is **test-only** Rust work. If it appears to need a spec or axiom change, **stop and re-raise** — that would mean the landed law is wrong, which is a different and much larger finding |

⚠ **`AC-C1` does not require a verdict flip in the suite.** The suite stays green.
What must change is that it **could** go red. ⛔ Do not read "the tests still
pass" as evidence for this node — that was true before the fix.

## Scope

**IN:** `crates/ken-elaborator/tests/l3_strings_roundtrip_acceptance.rs:145–205`
only.

⛔ **OUT:**
- ⛔ The prose/spec carriers — **`STR-BIJ` landed those** (exact `bcdd4548`,
  seven paths). ⛔ Do not re-correct them and do not treat their wording as open.
- ⛔ `catalog/packages/Data/Text/StringBijection.ken.md`'s **filename** and the
  `Data.Text.StringBijection` module path, plus the hard-coded module/test
  identifiers naming it. ⚠ That is a **separately deferred rename** with ten
  coupled `library/` consumers — an explicitly accepted survivor, ⛔ not part of
  this node.
- ⛔ Any change to `l2s`/`s2l` **behavior**, the NFC normalization, or the
  retraction axiom.

## Validation — ⛔ TARGETED ONLY

⛔ **NEVER `--workspace`** (operator, `agent/COORDINATION.md §12`).
`scripts/ken-cargo test -p ken-elaborator --test l3_strings_roundtrip_acceptance`.
Workspace, `--locked`, and conformance run **in CI**.

⚠ `ken-cargo` is a single machine-wide `flock`, slots == 1 — coordinate a
seat-to-seat yield **in-thread**, ⛔ never by sampling `ps`.

## Reporting

Return exact SHA/tree/base and, specifically: **the failing-expectation evidence
for `AC-C1`** (that asserting `[101, 769]` reddens), and **the normalization
positive control for `AC-C4`**.
