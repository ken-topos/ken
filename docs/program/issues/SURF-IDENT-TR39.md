---
id: SURF-IDENT-TR39
title: "The lexer's confusable-resistance is satisfied VACUOUSLY by an ASCII-only identifier rule — spec 31 §2's blessed Unicode letters are unimplemented, and the test that looks like the TR39 gate cannot see the difference"
status: merged
owner: ergo
size: S–M
gate: none
depends_on: []
blocks: []
github: null
origin: Steward measurement 2026-07-27 at `origin/main = 78f1f74b`, filed per COORDINATION §2.
---

> ## ✅ MERGED 2026-07-27 — PR #1121, `origin/main` = `91b67c3e`
>
> Squash of the exact approved SHA `a741061b` (tree `5e7c1061`). All three paths
> blob-verified identical on `main`: `error.rs`, `lexer.rs`,
> `tests/surface_unicode.rs`. Architect Decision `dec_5ch6fb4kvbqs2` resolved
> APPROVE for that exact SHA/tree; QA approved the same SHA. CI green.
>
> **Shape B was chosen before any `lexer.rs` edit** and the reasoning is the
> durable part: `§1b`'s bounded table governs **notation operators**, while `§2`
> supplies **no bounded blessed identifier-character table**. Inventing one would
> have violated `§1a-5`'s own fixed-table requirement and could have opened the
> confusable hole the spec names. So the ASCII boundary is recorded as an
> explicit **decision**, not as an accident of implementation.
>
> ⭐ **`AC-T3` is what makes this reviewable.** Cyrillic `а` **and**
> non-confusable `字` must fail with the *same specific* error and span — which
> makes the rejection **attributable to the ASCII rule** rather than merely
> coincident with it. The superseded test could not do that: all its cases died
> at the ASCII wall before any confusable rule could have been consulted, so it
> was vacuous as a security property. It is preserved under a truthful
> ASCII-boundary name with exact typed assertions.
>
> ⛔ **What this does NOT claim** (Architect's resolution, verbatim in substance):
> not Shape A, not a TR39 identifier profile, not a blessed Unicode identifier
> table, and **not lexical conformance coverage**.
>
> ⚠ **Residual — the `§2` completeness gap is enclave-routed, not closed.** The
> open question is whether `§2` should acquire a real blessed-character table or
> whether the blessed-letter clause should be retired for an explicit ASCII-only
> contract. ⛔ Do not read this merge as having settled it.
>
> ⚠ **`AC-T4` addressed the `ℓ`/`l` operator axis by EXCLUSION**, stated as such:
> the lexer maps the blessed operator glyph and its ASCII spelling to the same
> token, and this WP neither changes that nor claims TR39 separation on it.

> ## ⭐ RELEASED 2026-07-27 to **Ergo** as
> **[`SURF-IDENT-TR39-R1`](../wp/SURF-IDENT-TR39-R1.md)**.
>
> ⛔ This is **not** "add Unicode identifiers." Read §3 before any code: the
> obvious completeness fix is the one action that converts a vacuous pass into a
> real security hole.

## 1. The measurement

Taken at `origin/main = 78f1f74b`. ⛔ Re-derive at point of use.

| layer | what the corpus says | what is built |
|---|---|---|
| `spec/30-surface/31-lexical.md §2` | `ident` is `[a-z_][A-Za-z0-9_']*` **plus blessed Unicode letters** | ✗ **ASCII only** |
| `31 §1a` principle 5 | *"the lexer **normalizes/rejects** Unicode confusables (the TR39 security profile)"* | ✗ **no TR39 machinery exists** |
| `31 §1c` | *"**Confusable-resistance is a hard lexer gate** (principle 5)"* | ✗ no gate |
| `31 §1d` | *"The lexer rejects unblessed confusable identifier characters rather than repairing them into a different binding"* | ✗ nothing rejects *as confusable* |
| `31 §7` | conformance target `conformance/surface/lexical/` | ✗ **the directory does not exist** |
| Unicode **operators** (`λ → ∧ × Ω ∀`) | blessed, §1b | ✅ built — `lexer.rs:259–329` |

The whole identifier rule, verbatim (`crates/ken-elaborator/src/lexer.rs`):

```rust
fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '\''     // :148–150
}
...
if c.is_ascii_alphabetic() || c == '_' {                    // :420
```

There is no confusable table, no script analysis, no normalization step, and no
error variant naming a confusable. Unicode reaches the *operator* surface and
stops there.

## 2. ⭐⭐ The false green

`crates/ken-elaborator/tests/surface_unicode.rs:52` —
`surf1_d3_rejects_unbounded_unicode_identifiers` — asserts that three inputs are
rejected:

```rust
"fn surf1_bad (а : Type) : Type = Type",   // Cyrillic small a
"fn surf1_bad (xа : Type) : Type = Type",  // ASCII start, Cyrillic continuation
"fn Ｔ : Type = Type",                     // fullwidth capital T
```

It is green, its name reads as the TR39 gate, and **it passes because of an
ASCII wall.** Every one of those three is rejected by `is_ascii_alphabetic`
before any notion of "confusable" is consulted.

⚠ **State the direction of the weakness precisely — it is not simply blind.**

- ✅ It **would** catch the sloppiest widening: an implementer who admits *any*
  Unicode alphabetic would see all three cases start passing, and the test
  reddens. That is real and worth keeping.
- ⛔ It has **no positive control.** Nothing anywhere asserts that a *blessed*
  Unicode identifier character is **accepted**. So `§2`'s "plus blessed Unicode
  letters" capability is **absent and invisible** — the suite is equally
  consistent with the feature existing and with it never having been written.
- ⛔ It cannot distinguish **an ASCII wall from a confusable gate**, because
  every input it tries is rejected by both. Three rejections that share a single
  short-circuit are one control, not three.
- ⛔ It says nothing about the confusables `§1a-5` actually names — `⊔`/`U`,
  `∨`/`v`, `×`/`x`, `ℓ`/`l`. Those are **blessed operator glyphs colliding with
  ASCII identifier characters**, a different and nastier axis than a Cyrillic
  homoglyph, and `§1d` singles it out by name.

## 3. ⛔⛔ WHY THE OBVIOUS FIX IS THE DANGEROUS ONE

The security property in `§1a-5` is currently satisfied **by accident** — it is
a consequence of the under-implementation of `§2`, not of any gate.

⇒ **The two clauses are load-bearing on each other in one direction only.** A
future engineer reading `§2` sees a plain completeness gap ("identifiers should
accept blessed Unicode letters; they don't") and fixes it. That single change
**removes a security property that was never implemented**, because there is no
TR39 gate behind it to catch what was just admitted — and the corpus will not
protest, since `§1c` already asserts the gate exists and the test named after it
is green.

⭐ **This is why the node exists as a node.** The order is not negotiable: **the
gate is built before the admission, or the admission does not happen.**

## 4. Scope

**IN:** `crates/ken-elaborator/src/lexer.rs` and its tests; optionally seeding
`conformance/surface/lexical/`.

⛔ **OUT:**
- ⛔ **The formatter.** `ken fmt`, `lossless.rs`, `layout.rs`, `literate.rs` and
  the `§1d` token-kind canonicalization are **built and landed**. This node is
  the *lexer* half of `L-fmt` only. Do not touch the formatter.
- ⛔ **Editing `spec/`.** If the outcome needs a spec amendment (§5 Shape B), the
  WP **proposes** it as a document and routes it to the Spec enclave. Ergo does
  not edit the normative corpus.
- ⛔ **String-value normalization.** `STR-NFC-CONSTRUCTION` is a live enclave
  item about `String` *values*. This node is about *identifiers*. Same word, two
  concepts — do not merge them.
- ⛔ **Widening the blessed operator table.** `§1b` is settled.

## 5. The two acceptable outcomes

Both are honest; the status quo is not.

- **Shape A — build the gate, then admit.** A bounded blessed
  identifier-character table plus a specific confusable-class rejection error;
  only then does `§2`'s Unicode-letter admission land. Spec-complete.
- **Shape B — narrow deliberately.** Identifiers stay ASCII-only, but by
  *decision* rather than by accident: a specific error that names the rule, and
  `§2`'s blessed-letter clause recorded as an explicit, cited, unimplemented
  completeness gap routed to the enclave.

⛔ **Forbidden: the status quo** — the capability absent, the security property
vacuous, and a green test named for a gate that does not exist making both look
handled.
