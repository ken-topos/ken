---
id: SPEC-IDENT-BLESSED
title: "Settle the identifier character set: 31-lexical promises a bounded blessed-Unicode-letter table that does not exist, cites a security chapter that carries no such claim, and states a confusable gate the landed lexer does not implement"
status: merged
owner: spec-enclave
size: M
gate: none
depends_on: [SURF-IDENT-TR39]
blocks: []
github: https://github.com/swe-toolkit/ken/pull/1147
origin: "Routed by SURF-IDENT-TR39-R1 (merged PR #1121, origin/main 91b67c3e), which recorded the lexer's ASCII-only identifier boundary as an explicit decision and deliberately declined to invent a blessed-identifier table. Steward-framed 2026-07-27 at origin/main e23e5bc1."
---

Frame: [`../wp/SPEC-IDENT-BLESSED.md`](../wp/SPEC-IDENT-BLESSED.md) — shovel-ready,
inputs pinned by blob at `origin/main = e23e5bc1`.

> ## ✅ MERGED 2026-07-28 — PR #1147, exact `2f057e64a1fb876bfb4793ac3d8f55d9388049c0`
>
> ⚠ **The defect statement below is written in the present tense and is now
> HISTORICAL.** It describes `31-lexical.md` as it stood at `e23e5bc1`; it is not
> a live claim about `main`.
>
> Landed at `origin/main = 1a75836ddda64ffde8d07586916be5b423c772e8`, blob-verified:
> `spec/30-surface/31-lexical.md` = `bfcbce267733c3ae82a8172260118a2d43c1df85`,
> `spec/90-open-decisions.md` = `bfed05e9580c7535e82c7cbe770d37f093d8dae7`
> (independently confirmed by `spec-leader`).
>
> **How each numbered complaint was discharged — by `§1e`, `SPEC-IDENT-BLESSED`
> DECIDED, Shape C:**
>
> 1. **No blessed-Unicode-letter table** → none was invented. *Blessed* now
>    qualifies **notation**, bounded by the fixed `§1b` table; **identifiers** are
>    the stored ASCII productions of `§2` after closed alias expansion.
> 2. **`§1a` principle 5 / `§1c` BL3 demanded a fixed table** → both now bind
>    blessed **notation** to `§1b` and stored **identifiers** to `§1e`/`§2`.
>    Neither names an undefined identifier set or an unimplemented TR39 gate.
> 3. **A confusable gate the lexer does not implement** → Shape B is explicitly
>    rejected *because* the landed lexer implements no confusable policy. The
>    homoglyph path closes instead through a total single-valued alias map plus an
>    ASCII wall, so `ℓ`/`level` and `Ω`/`Omega` are each **one** binding.
>
> ⭐ Four candidates were required, and three died on **frame** defects, not on the
> ring's work: an amendment whose banned scope forbade the only discharge of its
> own authorization, then the `§1b` table-role scope gap the Architect caught, then
> the row-and-footnote split (one claim in two positions). ⛔ The last surviving
> stale carrier — the `FMT8` `l-identifier-is-not-a-level-token` row — was
> deliberately left byte-unchanged and routed to filed `CONF-FMT8-LEVELTOK`.
>
> ⭐ The spec enclave is **PARKED** under the operator's wind-down. ⛔ No successor,
> ⛔ no retro.

## Why this is a real defect and not tidying

`spec/30-surface/31-lexical.md` asserts four things that do not compose:

1. **`§2`** — `ident` admits `[a-z_][A-Za-z0-9_']*` **plus blessed Unicode
   letters**. ⛔ No table of blessed Unicode letters exists anywhere in the
   corpus.
2. **`§1a` principle 5** — requires that the blessed set be **bounded by a
   fixed table**. ⭐ The only fixed table in the chapter is `§1b`, which is the
   **notation-operator** table: glyph ↔ ASCII pairs for arrow, lambda, and
   quantifiers. It contains **no letters** and does not govern identifiers. So
   boundedness holds for the operator surface and fails for the identifier
   surface.
3. **`§1c`/`§1d`** — state confusable-rejection as a **hard lexer gate** that
   discriminates *unblessed confusable* characters. The landed lexer instead
   rejects **every** non-ASCII alphabetic scalar: strictly stronger on
   coverage, and a **different mechanism**. `SURF-IDENT-TR39-R1`'s load-bearing
   control exists to prove exactly that — Cyrillic `а` and `字` fail with the
   same error and span.
4. ⛔ **The security citation is dangling.** `31` cites `../60-security/64`
   **twice** as the authority for confusable-resistance. Measured:
   `64-trust-model.md` — a **Normative** chapter — contains no occurrence of
   `confusab`, `homoglyph`, `blessed`, or `identifier`.

## What makes now the cheapest moment

**Zero conformance rows cover identifier confusables.** A corpus-wide search
returns only `verify/protocol/false-unknown-non-confusable-roundtrip`, which is
about protocol-message distinguishability and is unrelated. ⇒ Nothing is
retracted and no proof is withdrawn — unlike the same fix after a conformance
row lands.

## Steward-discharged, so the enclave does not re-derive it

**The breaking-change question is answered (`§2a` of the frame).** 10 distinct
non-ASCII alphabetic scalars occur in 21 Ken files, but `λ` (894 of them) is a
**notation glyph**, not an identifier, and the rest are prose/metavariables.
⭐ The decisive fact is the merge, not the grep: the landed lexer rejects
non-ASCII identifier characters and PR #1121 passed **full CI**, so no
*checked* Ken code uses one. ⚠ Scoped in the direction it fails — this covers
code CI elaborates, not an unchecked markdown block or comment.

⇒ **Shape A and Shape C are not breaking changes.**

## The deliverable is a decision, not a table

⛔ **Do not invent a blessed-identifier table** without naming an external
profile it is drawn from — that is precisely what `SURF-IDENT-TR39` refused to
do, correctly, because an invented set is not *curated* and re-opens the hole
`§1a` exists to close.

Three shapes are laid out in `§3a` (ratify ASCII-only · adopt a named external
TR39/UAX-31 profile · ASCII-only with the extension point reserved). The
Steward recommends the third and says why; **the enclave decides**, and must
state why the others were rejected.

⛔ No CI checker or gate — including the weak "reports drift" form, which is
still a gate if it can go red (operator test policy).

⛔ Do not touch the notation-alias axis (`§1d`'s protection of identifiers
spelled `l`, `level`, `in`, `not`). That is a different axis, `SURF-IDENT-TR39`
explicitly excluded it, and conflating the two is the most likely way to
manufacture a defect here.

⛔ Do not edit `crates/`. If the selected shape diverges from the landed lexer,
that is a follow-on build WP for Ergo — the Steward frames it.
