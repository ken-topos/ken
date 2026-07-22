---
id: KW-THEOREM
title: "rename the surface keyword `lemma` to `theorem`"
status: draft
owner: spec
size: M
gate: none
depends_on: []
blocks: []
github: null
origin: operator directive 2026-07-22
---

**Operator directive (2026-07-22):** rename the keyword `lemma` to `theorem`
across the elaborator, catalog, library, spec, and docs.

## Measured footprint — 203 files, ~1,515 lines

| area | files | lines | character |
|---|---|---|---|
| `docs/` | 67 | 335 | mostly prose + WP frames |
| `crates/` | 48 | 227 | **the only place the keyword is *defined*** |
| `spec/` | 23 | 121 | **normative grammar + section anchors** |
| `catalog/` | 23 | 747 | **697 of 747 are keyword-leading declarations** |
| `conformance/` | 19 | 48 | seed suites pinning surface text |
| `agent/` | 17 | 24 | playbooks/memory — prose |
| `library/` | 3 | 9 | prose + **cross-doc anchors into `spec/`** |
| `tooling/` | 3 | 4 | — |

## ✅ `theorem` is FREE as a keyword — verified, no collision

`theorem` already occurs in 105 files, but **every occurrence is prose, doc
comment, or internal Rust naming** — never a Ken surface keyword and never an
identifier in `catalog/**/*.ken.md` or `examples/`.

★ **The internal vocabulary is already `theorem`:**

```
crates/ken-elaborator/src/lexer.rs:60   KwLemma, // "lemma" — standalone checked theorem
crates/ken-elaborator/src/ast.rs:217    /// `lemma name ... : φ = proof` — standalone checked proof theorem
crates/ken-elaborator/src/ast.rs:222    theorem: Type,          <- the AST field is ALREADY named `theorem`
crates/ken-elaborator/src/elab.rs:5691  fn elaborate_checked_theorem(
```

**⇒ This rename closes a naming seam rather than opening one.** The surface said
`lemma` while the implementation, the AST field, and the doc comments all said
*theorem*. That is the strongest available argument on intrinsic merits, and it
should be stated in the ADR rather than resting on the directive alone.

## Keyword definition sites (the mechanical core)

```
crates/ken-elaborator/src/lexer.rs:60    Token::KwLemma enum variant
crates/ken-elaborator/src/lexer.rs:459   "lemma" => Token::KwLemma
crates/ken-elaborator/src/ast.rs         RDeclKind::Lemma variant
crates/ken-elaborator/src/elab.rs:3846   RDeclKind::Lemma => elaborate_checked_theorem(...)
crates/ken-elaborator/tests/kenfmt_c_capstone.rs:208   formatter keyword STRING list
spec/20-verification/21-spec-syntax.md:180,403   lemma-decl ::= "lemma" ...
spec/30-surface/32-grammar.md:40                 grammar production
```

## ⛔ THE SCOPING TRAP — "lemma" in prose is NOT always the keyword

**This is the whole judgment content of the WP and a blind sweep will get it
wrong in both directions.**

*Lemma* is also an ordinary English/mathematical word meaning a helper result.
Prose like *"this lemma supports the main theorem"* is **correct English about
mathematics** and must NOT be rewritten to *"this theorem supports the main
theorem."* Conversely, `spec/30-surface/32-grammar.md:114`'s
`axiom N : T ⇒ lemma N : T = Axiom` **is** the keyword and must change.

**⇒ The acceptance criterion is a per-occurrence classification, not a
substitution count.** Every one of the ~1,515 lines is either:

| class | action |
|---|---|
| **keyword occurrence** (declaration, grammar production, keyword list, token) | rename |
| **English word** (prose about a helper result) | **leave**, and say so |
| **anchor/identifier derived from the keyword** (section slugs, test names) | rename **with its consumers** |

⛔ **A count of replacements is not evidence.** Cite the classification, and
make the leave-decisions as visible as the change-decisions — an unexplained
surviving `lemma` is indistinguishable from a missed one.

## ⚠ Couplings that a sweep will miss

1. **Cross-doc anchors.** `library/learn/reading-ken/02-types-contracts-and-proofs.md`
   links to `spec/.../33-declarations.md#8-named-proof-claims--prop-lemma-and-attached-proof`
   and `#83-standalone-lemmas--lemma`. **Renaming a spec heading silently breaks
   every inbound link.** The librarian's stale-anchor mutation is the gate.
2. **Pluralisation.** `lemma`/`lemmas`/`lemma`s (`library/…:53` writes
   *"as `lemma`s"*). A `\blemma\b` regex misses the possessive/plural forms; a
   naive `lemma→theorem` mangles them.
3. **The formatter keyword list** (`kenfmt_c_capstone.rs:208`) is a **string
   list** — a canonicalization oracle that fails in CI, not in a targeted build.
4. **`catalog/**` sources are literate `.ken.md`** — all 697 declarations must
   still `ken check` after the rename, and the catalog is a dependency-ordered
   corpus.
5. **`RDeclKind::Lemma` is a surface enum variant** — renaming it breaks
   exhaustive matches in every consumer; that is a *feature* (the compiler
   enumerates them), but it means the change is not confined to the elaborator.
6. **Conformance seeds pin exact surface text** (19 files) and run in CI.

## ⛔ OPEN FORK — needs an Architect/enclave ruling BEFORE decomposition

**Is this a hard rename or a transitional period?**

- **(A) Hard rename.** `lemma` ceases to lex; every existing `.ken` program
  using it breaks. Cleanest end state, one flip, no dual-spelling era.
- **(B) Accept both, `lemma` deprecated.** `lemma` continues to lex as a
  deprecated alias for some window. No corpus breaks, but the spec carries two
  spellings for one concept — which cuts against **subsume-don't-proliferate**.

The operator's wording (*"rename"*) reads as **(A)**, and the corpus is
first-party and fully in-repo, so (A) has no external-breakage cost that
normally motivates (B). **I lean (A) and am not deciding it** — it is a surface
contract question, and the answer determines whether this is one WP or a
two-phase campaign. Route before slicing.

## Suggested sequencing (only after the fork is ruled)

**Normative first, then implementation, then corpus** — the spec is the sole
authority (D1), so a catalog edit ahead of it would be unanchored.

1. **`spec/`** — grammar productions, keyword lists, section headings
   (enclave). Anchors change here, so inbound-link repair is scheduled with it.
2. **`crates/ken-elaborator`** — lexer token, `RDeclKind` variant, formatter
   keyword list (language ring). The compiler enumerates consumers.
3. **`catalog/` + `library/`** — 697 mechanical declarations + the prose
   classification (doc ring; librarian QA holds the anchor gate).
4. **`conformance/`** — seed suites, CI-gated.
5. **`docs/` + `agent/`** — prose; **largest file count, lowest risk**, and the
   place where the leave-it-in-English class dominates.

⛔ **Not ready to release.** Blocked on the (A)/(B) ruling, and the build side is
at its two-track cap (runtime + verify) per operator directive. This is filed
and sized, not queued.
