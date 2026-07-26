---
id: KW-ORACLE-REMOVE
title: "Delete the whole-tree source-text oracle: it asserts facts about repository text, which is now a prohibited test subject"
status: merged
owner: language
size: S
gate: none
depends_on: []
blocks: []
github: 1035
origin: Operator ruling 2026-07-26 — "That test should not exist. Remove it. It should be a violation of our testing policies," followed by the general rule "Test oracles that assert facts about source code, catalog, or documentation lines are an invitation for failure and delay. Tests should focus on behavior." Surfaced when the librarian's DOC-CATALOG-CONTENTS preflight found the oracle blocking a doc-only WP. Steward-filed per COORDINATION §2.
---

> ## ✅ MERGED 2026-07-26 — PR #1035, `origin/main = 3b979305`
>
> ⭐ **Merged by the OPERATOR directly, not through the publisher** — ⛔ so there is
> no publisher log and no `wp/scripted-merge-*` branch for this one.
>
> **Verified by blob ABSENCE, with a control:**
> `git cat-file -e origin/main:crates/ken-elaborator/tests/kw_theorem_source_oracle.rs`
> → exit 1 (absent); the same probe at base `720b0e17` returned **present**. ⇒ The
> absence is this change landing, not a path that never existed.
>
> **Retros in, ring closed:** `evt_14tgfydxmg2f4` (leader),
> `evt_5h11p8gjjswmp` (implementer), `evt_780k8hnjjmp44` (QA).

> ## ⚠ THIS WP DID NOT CLEAR THE RED — and my claim that it would was WRONG
>
> **What was measured (by `language-implementer`, and it was correct):**
> `exact_candidate_has_no_unclassified_retired_occurrences` **FAILED** at `main`.
> The catalog change `95bc855c` moved lines in
> `catalog/packages/Core/Classes/EffectfulClasses.ken.md` and `Derived.ken.md`,
> both pinned by the oracle's frozen line-number allow-list
> (`kw_theorem_source_oracle.rs:93`, `:113`).
>
> ⛔ **What I then wrote here was:** *"Every non-doc-only merge is now blocked
> behind this deletion."* **True, and INCOMPLETE — necessary, not sufficient.**
> `95bc855c` broke **three** things; this oracle was the only one that *reported*.
> The other two (12 drifted cited-source attestations, a stale `measured_tokens`
> census) surfaced on PR #1035's own CI, where they read as this candidate's
> failure. They are [`LIB-GATE-DECOUPLE`](LIB-GATE-DECOUPLE.md).
>
> ⭐ **The generalizable defect: a diagnosis keyed on ONE reporting mechanism is
> blind to every other consumer of the same change.** ⇒ After any `--doc-only`
> merge, **enumerate consumers of the touched paths** — do not wait to be told
> which one shouts.
>
> ⭐ **A doc-only merge reddened a Rust test suite.** That is the coupling the
> operator was describing when he removed the library currency gate hours
> earlier — *"including it as a CI-type system induces coupling that causes just
> the sort of slowdown and waffling that we're dealing with now."* This node was
> already justified on principle; it is now justified by a stopped pipeline.

> ## ▶ DELETE ONE TEST FILE — the analysis is done, read the measurements
>
> This is an `S`: remove
> `crates/ken-elaborator/tests/kw_theorem_source_oracle.rs`. ⛔ **It is not a
> judgement call and there is nothing to redesign** — the operator has ruled the
> test's *subject* inadmissible. The measurements below exist so you do not
> re-derive them, and so you do not talk yourself into preserving something.

## Why this test is inadmissible

`agent/playbooks/build/qa.md` → **`PROHIBITED SUBJECT`**, and
`agent/playbooks/build/implementer.md` → **`NEVER TEST THE TEXT OF THE
REPOSITORY`**. The rule, in one question:

> ⭐ *"Does an edit that changes nothing about how any program behaves make this
> test fail?"*

For this oracle the answer is **yes, constantly**: it pins **64
`(path, line, count)` rows across 18 files in six top-level trees** and compares
the census for **exact equality**, so inserting a paragraph anywhere above a
pinned line reds CI — in files the author has never opened.

## ⭐⭐ THE MEASUREMENT THAT MAKES THIS SAFE — removal loses ZERO enforcement

⚠ **The obvious worry is "deleting this un-enforces the keyword retirement."**
It does not, and here is the proof, measured at `origin/main`:

| question | probe | result |
|---|---|---|
| is the replacement keyword live in the lexer? | `grep '"theorem"' crates/*/src/` | ✅ `lexer.rs:60` `KwTheorem`, `:459` `"theorem" => Token::KwTheorem` |
| **is the retired token a token anywhere in the lexer/parser?** | `grep '"<retired>"' crates/*/src/` | ⛔ **ZERO hits** |
| any behavioural handling of it in `src/`? | `grep -ri` over `crates/*/src/` | only **ordinary prose in doc comments** (`diagnostics.rs`, `prover.rs`, `protocol.rs`, `trace.rs`) |

⇒ **The retirement is enforced by construction, in the lexer's keyword table.**
Source using the retired form is a **parse error** — it cannot elaborate. That is
behavioural enforcement, it is stronger than any text census, and it is
completely untouched by this WP.

⭐ **So what was the oracle actually enforcing? Prose hygiene.** It caught the
retired token in *English sentences* and inside *identifier substrings*. It never
had anything to do with whether a program is well-formed. ⛔ **Removing it loses
no property the language actually has.**

## The population, stated precisely (⚠ narrower than first reported)

`candidate_inputs()` = `git rev-parse HEAD` → `git ls-tree -r` → whole committed
tree, but `classify(path, source)` returns `Option<SourceClass>` and the loop
**skips `None`**. ⇒ The population is **files carrying Ken content** —
`FencedKen` (markdown *with a Ken fence*), `RawKen`, `EvaluationResults`,
`EvaluationFixture` — **not** all markdown.

⚠ **For a markdown file that IS in the population, the scan then covers the whole
file, prose included** (`occurrence_lines` enumerates every line; the fence walk
runs only to *validate* structure and its result is discarded). That asymmetry —
in-population by fence, then scanned entirely as prose — is what produced 64
`OrdinaryEnglish` rows and is the heart of why the test is unmaintainable.

⛔ **Do not "fix" this by narrowing the scan.** Prose scanning is deliberate and
control-pinned (`occurrence_scan_reaches_every_population_class_beyond_declaration_heads`
plants a prose line plus a fence line and asserts **2** findings). The ruling is
removal, not repair.

## What goes

**The entire file.** All 7 test functions and every helper are scaffolding for
the census — `candidate_inputs`, `classify`, `markdown_ken_lines`,
`occurrence_lines`, `retired_occurrences`, `retired_occurrence_offsets`,
`retired_findings`, `occurrence_census`, `allowed_occurrence_census`,
`ALLOWED_OCCURRENCES`, `PopulationExclusion`, plus the tests
`exact_candidate_has_no_unclassified_retired_occurrences`,
`occurrence_scan_reaches_every_population_class_beyond_declaration_heads`,
`occurrence_probe_is_case_insensitive_plural_possessive_and_identifier_reaching`,
`new_markdown_with_a_ken_fence_enters_without_registration`,
`population_exclusions_are_closed_named_and_causal`,
`spaced_ken_fence_info_is_classified`,
`whole_tree_population_reaches_outside_adversary_roots`.

⛔ **Nothing is salvaged into another file.** Every one of those tests exists to
verify the machinery of a prohibited oracle; keeping a "useful piece" reintroduces
the subject through a smaller door.

## Blast radius

- ⛔ **No `src/` change.** No production code references this test.
- ▶ **Unblocks `DOC-CATALOG-CONTENTS`**, which is held with a complete 24-file
  product delta at `wp/DOC-CATALOG-CONTENTS-index-to-contents = 1e36a37d`.
- ⇒ It also unfreezes `spec/30-surface/33-declarations.md`, five WP frames, a
  diary entry, and an `agent/memory/fleet/` file — all currently line-frozen.

## The honest residual

⚠ **After this lands, the retired token may appear in prose anywhere with no
check.** That is the intended consequence: prose hygiene is not a test's job.
⛔ **Do not propose a replacement checker inside this WP.** If the operator later
wants prose hygiene enforced, it is a lint or a review convention, not a test,
and it is a separate decision.
