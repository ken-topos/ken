---
id: SURF-IDENT-TR39
title: "The lexer's confusable-resistance is satisfied VACUOUSLY by an ASCII-only identifier rule — spec 31 §2's blessed Unicode letters are unimplemented, and the test that looks like the TR39 gate cannot see the difference"
status: active
owner: ergo
size: S–M
gate: none
depends_on: []
blocks: []
github: null
origin: Steward measurement 2026-07-27 at `origin/main = 78f1f74b`, filed per COORDINATION §2.
---

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
