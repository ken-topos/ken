# SPEC-IDENT-BLESSED — settle the identifier character set

**The spec promises a bounded blessed-Unicode-letter table that does not
exist, cites a security chapter carrying no such claim, and states a
confusable gate the landed lexer does not implement.**

**Owner:** spec enclave (spec-leader + spec-author + conformance-validator).
**Branch:** `wp/SPEC-IDENT-BLESSED`. **Size:** M. **Risk:** medium — a
normative surface contract with a security claim attached, and the security
chapter it cites does not carry that claim.

**Status:** Steward frame, shovel-ready. Released to the spec enclave.

---

## 1. Fixed inputs — measured at `origin/main = e23e5bc1`

⛔ These are **measurements, not recollections**. Blob ids are pinned so a
later reader can tell whether the input moved under them.

| path | blob |
|---|---|
| `spec/30-surface/31-lexical.md` | `736fa1f81559d03fc5601c489554fb625b3445c3` |
| `spec/60-security/64-trust-model.md` | `f3811a6992a5cbb122d67bbd650abb247f4aec37` |
| `crates/ken-elaborator/src/lexer.rs` | `5226bafd8a7ddb6ef30b4a2d5bf3a3fb27e6faad` |

**Immediate cause.** `SURF-IDENT-TR39-R1` merged today as **PR #1121**
(`origin/main` was `91b67c3e`). It recorded the lexer's **ASCII-only**
identifier boundary as an *explicit decision*, gave every otherwise-unblessed
alphabetic Unicode scalar a typed `NonAsciiIdentifierCharacter { character,
span }` refusal, and **deliberately declined to invent a blessed-identifier
table**. Its own statement of what it does not claim: *"does not implement a
TR39 identifier profile, does not supply a blessed Unicode identifier table."*
It routed the completeness question here. **This WP is that question.**

---

## 2. The measurement — four strands, and they do not compose

### Strand 1 — `§2` admits a set that is never defined

`31 §2` (line 366):

> **`ident`** — value/term names: lowercase-initial, `[a-z_][A-Za-z0-9_']*`
> plus **blessed Unicode letters**. Primes (`x'`) allowed (math-friendly).

⛔ **There is no table of blessed Unicode letters anywhere in the corpus.**

### Strand 2 — `§1a` principle 5 requires that table to exist and be bounded

`31 §1a` (lines 44–50):

> **Curated and confusable-resistant (a security property, not only
> legibility).** The blessed set is **bounded** (a fixed table, not "any
> Unicode"), and the lexer **normalizes/rejects Unicode confusables** (the TR39
> security profile: `⊔`/`U`, `∨`/`v`, `×`/`x`, `ℓ`/`l`, Cyrillic look-alikes).

⭐ **The only fixed table in the chapter is `§1b`, and it is the wrong one.**
`§1b` is the **notation-operator** table — glyph ↔ ASCII pairs with roles like
arrow, lambda, quantifier. It contains **no letters** and does not govern
identifiers. So principle 5's boundedness requirement is satisfied for the
*operator* surface and **unsatisfied for the identifier surface**.

### Strand 3 — `§1c`/`§1d` state a gate the implementation does not implement

`§1c` (lines 108–111): *"Confusable-resistance is a **hard lexer gate**
(principle 5). The blessed set is bounded; the lexer normalizes/rejects TR39
confusables."*
`§1d` (line 138): *"The lexer rejects **unblessed confusable** identifier
characters rather than repairing them into a different binding."*

⛔⛔ **THIS STRAND WAS FALSE AS ORIGINALLY WRITTEN. CORRECTED 2026-07-27
(Steward ruling `evt_5xqj5r5fbn7ry`), after the conformance validator and the
Architect blocked a candidate built on it. Read the correction, not the
retracted claim.**

⛔ ~~"The landed lexer rejects **every non-ASCII alphabetic scalar**. That is
**strictly stronger** on coverage…"~~ — **wrong.**

**Measured on `lexer.rs` at `origin/main = 4fd80cd8`:** the operator match arms
at `:259-355` run **before** the `is_alphabetic()` rejection at `:483`, and
four of them return **identifier-class tokens**:

| glyph | token | enters a binding? |
|---|---|---|
| `Ω` | `Token::ConId("Omega")` | ✅ **yes** |
| `Σ` | `Token::ConId("Sigma")` | ✅ **yes** |
| `Π` | `Token::ConId("Pi")` | ✅ **yes** |
| `ℓ` | `Token::Ident("level")` | ✅ **yes** |
| `λ` | `Token::Lambda` | ⛔ no — dedicated token |
| `∀` `∃` `¬` | `Token::Ident(…)` | yes, but **not alphabetic**, so they never reach the gate |

⇒ ⭐ **`ℓ` *is* `level`; `Ω` *is* the ConId `Omega`.** Four alphabetic Unicode
scalars **do** enter binding positions, as their ASCII aliases. ⛔ Any claim of
"ASCII-only identifiers" or "no Unicode letter reaches a binding" stated
**absolutely** is false and will be blocked.

**The true statement: the identifier surface is ASCII-only *after alias
expansion*.** Every accepted identifier is an ASCII name; a fixed, closed table
of Unicode glyphs are alternate spellings **of specific ASCII names**.

⭐ **And this satisfies principle 5 rather than violating it.** The alias map
is total, closed, and single-valued — `ℓ` and `level` are **one** identifier,
not two that look alike. A homoglyph hole is two *distinct* bindings with
confusable spellings; this is its opposite. ⇒ The security property holds, via
a mechanism the chapter never described. **That gap is the actual defect.**

**What survives of the original strand:** for every non-ASCII alphabetic scalar
**outside** the alias table, the lexer *is* an ASCII wall rather than a
confusable rule, and `SURF-IDENT-TR39-R1`'s control proves it — Cyrillic `а`
(confusable) and `字` (plainly not) fail with the **same** error and span. ⇒
The spec describes a gate discriminating confusables from non-confusables; the
implementation discriminates *alias-table membership*, then ASCII from
non-ASCII.

### Strand 4 — ⛔ the security citation is dangling

`31` cites `../60-security/64` **twice** (lines 49, 111) as the authority for
the confusable-resistance security claim.

**Measured: `64-trust-model.md` contains no occurrence of `confusab`,
`homoglyph`, `blessed`, or `identifier`.** `64` is marked **Normative** and
fixes the TCB, authorship-independence, and the honest limits. It does not
carry the homoglyph claim the lexical chapter attributes to it.

### And the coverage fact that makes all four cheap to fix now

**Zero conformance rows cover identifier confusables.** A corpus-wide search
for `confusab` / `homoglyph` / `NonAscii` / `blessed Unicode` under
`conformance/` returns only `verify/protocol/false-unknown-non-confusable-
roundtrip`, which is about two protocol messages being distinguishable and is
unrelated.

⇒ ⭐ **Nothing has to be retracted.** There is no landed conformance assertion
to flip and no proof to withdraw. This is the cheapest moment this question
will ever have.

### `§2a` — the breaking-change question, discharged by the Steward

The obvious risk to Shape A/C is that ratifying ASCII-only breaks a source file
that already uses a non-ASCII identifier. **Measured, so the enclave inherits
the answer rather than the question.**

Across the 59 `.ken` / `.ken.md` files under `catalog/`, `examples/`, and
`prelude/`, **10 distinct non-ASCII alphabetic scalars** occur in 21 files:

| scalar | count | what it actually is |
|---|---|---|
| `λ` U+03BB | 894 | ⭐ **notation glyph**, not an identifier — `§1b` maps `λ` ↔ `\`; lexes to `Token::Lambda` |
| `Ω` `Σ` `Π` | 19 / 12 / 6 | ⛔ **CORRECTED** — these lex to `Token::ConId("Omega"/"Sigma"/"Pi")`, i.e. **identifier-class aliases**, not mere notation |
| `δ` `φ` `η` `ι` `ρ` | 7 / 6 / 6 / 2 / 1 | Greek metavariables, prose and comments — **not** in the alias table |
| `é` U+00E9 | 1 | prose |

⛔ **CORRECTION, 2026-07-27 (`evt_5xqj5r5fbn7ry`) — this table's original
reading was under-generalized and it is how the false Strand 3 survived.** I
labelled `λ` "a notation glyph, not an identifier" and stopped there. That
label is **correct for `λ` alone**, because `λ` is the only one returning a
non-identifier token. `Ω`, `Σ`, `Π` (and `ℓ`) are a **different class**: they
alias *to identifier tokens*. ⭐ **The census contained the answer and the
generalization was never made.** Do not repeat it: classify by the **token
returned**, not by the glyph's appearance in a notation table.

⭐ **The decisive fact is not this grep — it is the merge.**
`SURF-IDENT-TR39-R1` passed **full CI** as PR #1121, so ⇒ **no *checked* Ken
code uses a non-ASCII identifier outside the alias table**, because if it did
that merge would have gone red. ⚠ Note the added qualifier — the alias-table
glyphs are *accepted*, so CI greenness says nothing about their absence.

⚠ **Scoped honestly, in the direction it fails:** this establishes the property
for code that CI **elaborates**. A Greek letter sitting in an unchecked
markdown block or a comment is **not** covered and would not have reddened
anything. That residual is small (those are prose positions, which no shape
here affects) but it is real, and the enclave should not re-state my claim more
strongly than this.

⇒ **Shape A and Shape C are not breaking changes.** The `§8` hard-stop bullet
on this axis is **discharged**; do not re-run it.

---

## 3. The design judgment — front-loaded, so the author does not re-derive it

**The deliverable is a decision plus a corrected chapter. It is not a table
you invent.** The reason `SURF-IDENT-TR39` stopped and routed here is that
inventing a blessed-identifier set would have violated `§1a`'s own
fixed-table requirement (an invented table is not a *curated* one) and could
have opened the very confusable hole `§1a` names. That reasoning stands.

### `§3a` — three candidate shapes

**Shape A — ratify ASCII-only for identifiers.** Strike *"plus blessed Unicode
letters"* from `§2`; scope principle 5's confusable gate to the **notation
surface**, where the bounded table actually exists; state the identifier rule
as an explicit, reasoned security decision (a wall is a sound conservative
choice) with the extension point named as a non-goal. Matches the landed
lexer. Loses nothing the corpus uses.

**Shape B — supply the bounded table.** Adopt a **named external profile**
(e.g. UAX #31 identifier syntax restricted by TR39 `IdentifierStatus=Allowed`
and a declared script set), specify normalization (NFC) at lex time, and
specify confusable-skeleton rejection. Then the spec is satisfiable — but this
is a large normative addition **and** a build obligation Ergo has already
measured as absent.

**Shape C — ASCII-only now, extension point reserved.** Shape A's chapter
edits, plus an explicit reserved-extension clause stating what a future
profile must supply (a named profile, a normalization rule, a conformance
row), so a later widening is **additive** rather than a contract change.

⭐ **Recommendation, not a ruling: C.** It is A's cost with B's optionality,
and it makes the residual explicit rather than leaving a future reader to
infer that ASCII-only was an oversight. **The enclave decides**; if it picks A
or B it must say why C was rejected.

### `§3b` — the `64` citation must be dispositioned, not silently dropped

Two admissible outcomes, and the WP must **name which** and why:

1. **`64` gains the claim** — the homoglyph/reviewer-integrity property is
   genuinely a trust-model claim and belongs in `64 §4`'s honest limits, in
   which case `31`'s citation becomes correct.
2. **The citation is corrected** — the property is purely lexical and the
   cross-reference should point at the lexical chapter's own clause (or be
   removed).

⛔ **Leaving a normative chapter citing another normative chapter for a claim
it does not make is not an acceptable outcome of this WP.**

---

## 4. ⛔ Banned shapes

- ⛔ **Do not invent a blessed-identifier table** without naming an external
  profile it is drawn from. An invented set is not curated and re-opens the
  hole `§1a` exists to close.
- ⛔ **Do not add a CI checker or gate.** Operator test policy: *"Test oracles
  that assert facts about source code, catalog, or documentation lines are an
  invitation for failure and delay. Tests should focus on behavior."* ⚠ The
  weaker "only reports drift" form is **still a gate if it can go red**, and is
  equally banned.
- ⚠ **AMENDED 2026-07-27 (`evt_5xqj5r5fbn7ry`) — the notation-alias axis:
  DESCRIBE IT, DO NOT CHANGE IT.**

  ⛔ ~~"Do not touch the notation-alias axis… it is a **different axis**."~~ —
  **that ban was wrong and it blocked the WP.** Strand 3 shows the two axes
  **intersect**: `Ω`/`Σ`/`Π`/`ℓ` are alias-table entries *and* alphabetic
  scalars in identifier positions. ⇒ Forbidding the axis forbade stating the
  truth about the identifier surface, and the enclave could not choose a
  disposition by inference. That was my error, not a drafting slip.

  ✅ **You MAY describe** the alias mechanism exactly where it bears on the
  identifier surface — which glyphs alias to identifier tokens, and that the
  map is total, closed, and single-valued.

  ⛔ **You MAY NOT change it**: no adding to or removing from the alias table,
  no altering `§1d`'s protection of identifiers spelled `l`, `level`, `in`,
  `not`, no formatter change. The table is operator decision SURF-1 D3.
  ⛔ Reserving these glyphs from identifier positions is a **breaking change**
  and is a separate Ergo WP the Steward would frame — ⛔ **not** something this
  WP may do as a consequence.
- ⛔ **Do not edit `crates/`.** This WP is a spec decision. If it selects a
  shape whose implementation differs from the landed lexer, that is a
  follow-on build WP for Ergo — the Steward frames it, this WP does not do it.
- ⛔ **Do not re-open** the `§1c` accept-both / same-token behaviour or the
  formatter's canonicalization rules. They are settled and orthogonal.

---

## 5. Deliverables

- **`D1`** — the **decision**: which shape, and the rejected alternatives with
  the reason each was rejected. Recorded in the chapter, not only in channel.
- **`D2`** — `31 §2`'s `ident` production corrected so it admits exactly what
  the chapter can define.
- **`D3`** — `31 §1a` principle 5 and `§1c` corrected so the boundedness
  requirement and the confusable gate are stated over the surface each
  actually governs.
- **`D4`** — the `64` citation dispositioned per `§3b`, with the edit made on
  whichever chapter is wrong.
- **`D5`** — a **closed completeness report**: every site in the corpus that
  asserts or cites identifier-level confusable-resistance, each with its
  disposition. Closed over the whole corpus, complement named.
- **`D6`** ⭐ **(added by Steward ruling `evt_7m5mgy8ne2pxz`, 2026-07-27)** —
  `spec/90-open-decisions.md` reconciled, bounded to **exactly two repairs**.
  See `§7a`.

---

## 6. Acceptance criteria

- **`AC-1`** — `D1` names the selected shape **and** states, for each rejected
  shape, the specific reason. **Control:** a reader who disagrees with the
  choice can point at the sentence that would have to be false. ⛔ A bare
  "Shape C selected" does not discharge this.

- **`AC-2`** — after `D2`+`D3`, **no clause in `31` refers to a set the
  corpus does not define.** **Control:** the report cites, for every surviving
  use of "blessed", the exact table that bounds it. `§1b` bounds the notation
  uses; an identifier use with no table is a **failure**, not a residual.

- **`AC-3`** — `D4` leaves no dangling citation. **Control:** each of the two
  `../60-security/64` references in `31` resolves to text in `64` that makes
  the cited claim, **or** the reference is gone. Verified by reading `64`,
  not by assuming the edit landed.

- **`AC-4`** ⭐ **(the load-bearing one)** — `D5`'s inventory is **closed over
  the corpus and names its complement.** It must report the sites it examined
  **and** the sites it deliberately excluded with the reason. **Control:** the
  report must show at least the four strands in `§2` above and state whether
  any fifth exists. ⛔ **An empty or thin report is indistinguishable from a
  thorough one unless the hard cases are shown** — reporting "no other sites"
  while the notation-alias axis and the `verify/protocol` false hit both exist
  and were *correctly* excluded is a **failed measurement wearing the
  appearance of a clean one**. Name them as excluded, with why.

- **`AC-5`** — the WP states plainly what it does **not** settle. At minimum:
  whether the landed lexer's behaviour changes, and whether any conformance
  row is now owed. **Control:** a follow-on framer can read `AC-5` and know
  whether Ergo has work.

  ⭐ **AMENDED 2026-07-27 (Steward, `evt_7m5mgy8ne2pxz`), and this clause is
  now positive, not optional:** a conformance row **is** owed, and `AC-5` must
  say so at the spec's **locked granularity**.

  ⭐⭐ **RE-AMENDED the same day (`evt_5xqj5r5fbn7ry`): the obligation has
  THREE cases, not two.**

  1. ASCII identifier → **accepted**.
  2. non-ASCII alphabetic **not** in the alias table (Cyrillic `а`, `字`,
     fullwidth `Ｔ`) → **rejected**, typed error. Include a **non-confusable**
     control alongside the confusable one, so the row distinguishes an ASCII
     wall from a TR39 gate.
  3. non-ASCII alphabetic **in** the alias table (`ℓ`, `Ω`, `Σ`, `Π`) →
     **accepted as its ASCII alias**.

  ⛔ **Case 3 requires the identity control or the row is vacuous:** `ℓ` and
  `level` must resolve to **the same binding** — one identifier, not two.
  ⭐ That control is the only one of the three that can tell an alias mechanism
  apart from a confusable hole. ⚠ A row carrying cases 1 and 2 alone reads as
  complete, and is exactly the shape that passed review while the frame's
  premise was false.
  ⛔ The row is **staged, not authored here**, and ⛔ **do not invent a row
  id** — the Steward tracks the follow-on. ⚠ `crates/ken-elaborator/tests/
  surface_unicode.rs` is implementation evidence and does **not** discharge
  it; a crate test is not the corpus the build fleet codes against.

- **`AC-6`** ⭐ **(added with `D6`)** — the two `90` repairs are made **and
  bounded**. **Control:** show that the **claims** of `90`'s operator decision
  for principles 1–5 are unchanged apart from the removed `64` citation, and
  that the mechanism update is carried by a **`Resolved sub-item`** entry.
  ⛔ A diff that **reworks the operator's decision into different claims**
  fails this even if the resulting prose is more accurate. ⚠ Deliberately a
  test on **claims, not bytes** — a whitespace reflow to 80 columns is
  permitted and must not be reported as a violation.

---

## 7. Scope

**In:** `spec/30-surface/31-lexical.md`, `spec/60-security/64-trust-model.md`
(only if `§3b` outcome 1 is selected), **`spec/90-open-decisions.md` (bounded
— see `§7a`)**, and this frame's tracker node.

**Out:** ⛔ `crates/**`, ⛔ `conformance/**` (there is nothing to retract; a
*new* row may be **proposed** in `D5` but is not authored here), ⛔ the
formatter, ⛔ the notation table `§1b`, ⛔ `catalog/**`.

**Contention:** measured **zero**. Language holds `crates/ken-elaborator/**`
(`SURF-SPACE-CELLS-P1`, 13 paths, all under `crates/`); Runtime holds
`crates/ken-runtime/**`; Verify holds `crates/**` test paths. **No live ring
holds any `spec/` path.** The spec enclave is idle and clear as of
`e23e5bc1`.

---

## 7a. ⭐ `spec/90-open-decisions.md` — the atomic-boundary ruling

**Steward ruling, `evt_7m5mgy8ne2pxz`, 2026-07-27**, on a scope fork raised by
the conformance validator and routed by the spec leader. Recorded here because
a channel post is not a durable deliverable.

**Fixed input:** `spec/90-open-decisions.md`, blob
`bceabd9776126e93eba17404e90b18be9311b9a0` at `origin/main = e700b861`.

### Why it is atomic and not a follow-on

`90:205-208` asserts **both** defects this WP exists to fix, in the register
`31` itself names as governing — *"a bounded, confusable-resistant blessed set
with lexer normalization/rejection of Unicode look-alikes (TR39) …
`60-security/64`"*. ⇒ Correcting `31` alone **moves** the defect rather than
fixing it: the corpus still promises a blessed Unicode identifier set and
still dangles into `64`. ⚠ A follow-on is the worse shape specifically because
the two chapters would be **provably inconsistent for the whole gap**, which
is harder to review than either endpoint.

⭐ **My `§1` fixed inputs did not enumerate `90`. That census was incomplete**
— `AC-4` is what caught it, working as intended.

### ✅ Authorized — exactly two repairs

1. **Remove the `60-security/64` citation.** It was dangling the day it was
   written; `64` has never carried an identifier claim. Identical in kind to
   the repair already authorized in `31`. ⛔ Not a decision change.
2. **Record the mechanism as settled** using **the file's own idiom** — a
   `**Resolved sub-item (…)**` entry, as already used at `90:215` (SURF-1 D3)
   and `90:221` (ES3). Name `SURF-IDENT-TR39` and this WP, and state what
   landed: boundedness is preserved and binds the **§1b notation** set; the
   identifier surface is ASCII-only; there is no TR39 identifier profile,
   normalization, or repair.

### ⛔ Not authorized

- ⛔ **Do not edit, reword, or delete the operator's 2026-06-27 decision
  text.** Principles 1–5 stand. The operator's *property* — bounded,
  confusable-resistant, no homoglyph backdoor — is **preserved** by the
  selected shape, not overturned; ASCII-only is strictly stronger on it. Only
  the **mechanism** changed, and the resolved-sub-item idiom is precisely how
  this file records that.
- ⛔ Do not reopen the token-table iteration, the glyph set, or any other
  `OQ-syntax` item in `90`.
- ⛔ **`docs/PRINCIPLES.md:230-231` is excluded** — measured: it states the
  *property* ("confusable-resistant character set"), which the selected shape
  preserves, and it cites **`31 §1a`, not `64`**. Neither dangling nor
  falsified.
- ⛔ **`spec/20-verification/25-protocol.md:302,329` and
  `conformance/verify/protocol/false-unknown-non-confusable-roundtrip` are
  excluded** — "non-confusable **messages**" is verdict distinguishability, a
  different axis.

⇒ **The complete normative carrier set for this defect is `31` + `90`.**
Measured corpus-wide for `confusab|homoglyph|TR39|look-alike|blessed`; every
other hit is `docs/program/**` non-normative or the protocol axis above.

---

## 8. Hard stop

⛔ Stop and route to the Steward if any of the following is discovered:

- a **conformance row** asserting identifier-level confusable behaviour exists
  after all (the `§2` census says none does — if that is wrong, the
  cheapness argument in `§2` is false and the shape choice changes);
- ✅ ~~a consumer outside `spec/` using a non-ASCII identifier~~ —
  **DISCHARGED by the Steward in `§2a`.** Do not re-run it. If you believe
  `§2a` is wrong, say which of its two claims fails (the scalar census, or the
  full-CI inference) rather than repeating the search;
- the `64` disposition turns out to require a **substantive trust-model
  claim** rather than a citation repair. That is an Architect question, not a
  drafting one.
