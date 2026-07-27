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

> ## ⛔⛔ RE-SCOPED 2026-07-27 AFTER A BLOCK — `AC-C1` AND `AC-C4` WITHDRAWN
>
> **They were unachievable at this base.**
>
> **@language-leader blocked this node on its own fixed inputs and was right.**
> Measured on the real elaborated/evaluated path: `l2s([101,769])` yields
> `EvalVal::Str("e\u{301}")`, **not** `Str("é")`. ⇒ **The implementation does not
> NFC-normalize.** No normalization exists in `ken-elaborator`/`ken-runtime`.
>
> ⛔ **The spec contradicts itself, and it names this exact trap:**
> `spec/30-surface/37-strings-collections.md:89` states `List Char → String` as
> *"encode UTF-8, then **NFC-normalize**"* — normative, total behavior — while
> `:810–811` says `String` NFC normalization is *"**a deferred behavior —
> currently stubbed**"* and warns, verbatim, against *"**the
> over-pin-a-deferred-behavior trap**."*
>
> ⇒ ⛔ **`AC-C1`'s expected `[233]` was inherited from the contract and never run
> through the executable path.** The spec warns against pinning the *pre*-NFC
> behavior; the withdrawn AC pinned the *post*-NFC behavior. Same error, opposite
> direction.
>
> ### ⭐⭐ AND THE ORIGINAL DIAGNOSIS WAS TOO WEAK — no operand can fix this test
>
> With NFC stubbed, `s2l (l2s cs) ≡ cs` **genuinely holds for every `cs`**. ⇒ The
> test is **not** vacuous because its operand happens to be an NFC fixed point —
> it is vacuous because ⛔ **every input is one while normalization is absent.**
> There is no operand that discriminates at this base.
>
> ⭐ **The correct statement of the defect: the test is green because a deferred
> behavior is MISSING. It pins the STUB, and its name claims the CONTRACT.** That
> is strictly more serious than the NFC-fixed-point framing this node was filed
> with, and it is what the node now targets.
>
> ⚠ **Open, and NOT this node's to settle** — routed to the Architect: is stubbed
> NFC a **contract defect**? Either `:89` is aspirational and must be marked
> deferred everywhere it is stated as behavior, or the implementation owes NFC as
> a **behavioral WP**. ⛔ Do not resolve it here; ⛔ do not wait on it either —
> everything below is independent of the ruling.

> ## ⛔ THE ORIGINAL FRAMING, RETAINED FOR THE RECORD — but see the re-scope above
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

## Fixed inputs

⛔ **The `cs₁`/`cs₂` operand below belongs to the WITHDRAWN `AC-C1`.** It is
retained because it is the evidence that the contract and the implementation
disagree — ⛔ not as an input to build against.

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

## Acceptance criteria — ⛔ RE-SCOPED. `AC-C1` and `AC-C4` are WITHDRAWN.

> ⛔ **`AC-C1` (assert `[233]`) and `AC-C4` (the normalizer positive control) are
> WITHDRAWN as unachievable at this base** — there is no normalizer to reach and
> no operand that discriminates. ⚠ Recorded rather than deleted so they cannot be
> re-read as still owed. ⇒ They return **with the behavioral NFC WP**, not before.

| AC | claim | control |
|---|---|---|
| `AC-C2` | All four carriers state the **landed** property: the retraction `l2s (s2l s) ≡ s` and `s2l`'s derived injectivity. ⛔ No "identity", no "inverse", no "both ends", no "reproduce exactly" on the reverse direction. | grep the four sites; each must name **retraction** or **injectivity**, ⛔ never a two-sided claim. The `:145` header must stop naming the **pair** as an identity |
| `AC-C3` | The **existing** landed-direction test and its corpus are unchanged. | `ac2_round_trip_l2s_s2l_identity` (`:150`) and its ten-string corpus are **byte-identical**. ⛔ This WP removes an over-claim; it must not remove coverage |
| `AC-C5` | No behavior change anywhere: no axiom, law, instance, spec, or catalog edit. | ⛔ **Test-only** Rust. ⚠ It is now *known* that a spec change is implied — that is the Architect's open ruling, ⛔ **not** a licence to make it here |
| ⭐ `AC-C6` **(NEW — replaces `AC-C1`)** | The test states **explicitly** that the reverse direction holds **only under the current NFC stub**, citing `37 §9` and `dec_ppakqc11kffh`. | ⛔ The carrier must name the **stub** as the reason it passes, and ⛔ must **not** present it as normative `String` semantics. ⭐ The honest version of what `AC-C1` reached for: the test cannot demonstrate the contract, so it must **say which weaker thing it demonstrates** |
| ⭐⭐ `AC-C7` **(NEW — the durable half)** | **Expect the decomposed `[101, 769]` SOLELY as a loud transition tripwire**: when real NFC lands, this test must **FAIL LOUDLY** rather than silently keep passing. | ⛔ Positioned so a future NFC implementation **breaks this test on purpose**, with a message naming `dec_ppakqc11kffh` and this node. ⚠ Today it would silently continue passing once NFC lands — the inverse of the *"literal-level pin would falsely fail"* hazard `37 §9` describes. ⇒ **The one thing the current test cannot do, and the reason this node exists** |

> ### ⚖️ AMENDED by Architect Decision `dec_ppakqc11kffh`
>
> (`resolved` 2026-07-27T13:58:05Z)
>
> **Disposition (b): NFC-at-construction REMAINS the normative contract.**
> `37 §2.1`/`§2.3`, `41 §3a`, and the existing K3 canonical encoder all agree on
> it; `37 §9`'s *"deferred behavior — currently stubbed"* is an **honest staging
> disclosure, not authority to weaken the permanent contract.** ⇒ The measured
> decomposed result is a **known implementation gap**, not intended behavior.
>
> ✅ *"Hold/withdraw the released `[233]` ACs at base `f1f626f7`"* — the withdrawal
> above is confirmed.
>
> ⚠ **Correction to this node, which had it backwards:** I wrote ⛔ *"do not weaken
> `[233]` to the observed decomposed result."* The ruling **requires** expecting
> `[101, 769]` — **solely as the transition tripwire**, never as normative
> semantics. `AC-C7` now says so. ⇒ Expecting the decomposed value is **mandatory
> and bounded**, not forbidden.
>
> ⛔ **This carrier must be replaced/flipped when the behavioral WP lands, and
> ⛔ must never be cited as normative `String` semantics.**

⚠ **No verdict flip is required and the suite stays green.** ⛔ Do not report
"tests still pass" as evidence for this node — that was true before any of it.

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
