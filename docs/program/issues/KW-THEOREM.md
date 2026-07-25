---
id: KW-THEOREM
title: "rename the surface keyword `lemma` to `theorem`"
status: merged
owner: language
size: M
gate: none
depends_on: []
blocks: []
github: null
origin: operator directive 2026-07-22. RELEASED 2026-07-25 as the fleet's SECOND implementation lane on the operator's directive to open one (OpenAI capacity reset to 100%). Owner moved spec -> language: the mechanical core is crates/ken-elaborator, so the Language ring implements; the spec enclave / conformance-validator retains the review vote on the normative grammar (D2) and the ADR (D6).
---

> ## ▶ THE FRAME IS WRITTEN — read it, not this file
>
> `docs/program/wp/KW-THEOREM-surface-keyword-rename.md`
>
> This file is the triage that produced the frame. **The frame carries the
> deliverables, the acceptance criteria, the settled inputs, and the
> do-not-reopen guardrails.**
>
> ### ✅ THE "HOLD UNTIL FNSPLIT CLOSE" PREMISE WAS UNMEASURED, AND IS FALSE
>
> This node was held on the Steward's assumption that a corpus-wide surface
> rename would contend with the live FNSPLIT lane. **Measured at `aecdb001`, it
> does not:** all 44 crate files carrying `lemma` are in `ken-elaborator` and
> **none is in `ken-runtime`**, and the only WP frames it touches (`PX8-T`,
> `PX8-F`) are `draft`. The hold is lifted.
>
> ### ⛔ IT DOES CONTEND — WITH THE DOC TRACK, ON THE LEDGER AXIS
>
> `library/SOURCE-ATTESTATIONS` carries **17 rows for `catalog/` sources**, and
> this WP rewrites **698 keyword-leading declarations** across 23 catalog files
> — so every attested catalog source changes hash. The doc track is the fleet's
> one standing concurrency exception and it lives in `library/` + `agent/`.
> **The doc track is PARKED for this WP's duration. Two lanes, not three.**

## ⚠ The footprint table below is SUPERSEDED — two counts were wrong

Re-measured at `aecdb001` while framing. **Both corrections widen the work:**

- **`library/` is 10 files, not 3** — and three of them are not prose:
  `library/manifest.toml`, `library/agents/evaluations/results-2026-07-24.toml`,
  and `library/agents/evaluations/fixtures/proof-terminals.txt`. **An
  evaluation-results file and a fixture are oracles, not documentation.**
- **`conformance/` includes a raw `.ken` source** —
  `conformance/challenge/C6-lawful-ord-vs-stub/sound-ord-proved.ken` — not only
  literate `.ken.md`. A glob written for `*.ken.md` misses it. That is a second
  oracle class, and the frame's `AC-3` requires one glob definition covering
  every Ken source root with a planted-declaration positive control per class.

Also re-counted: `catalog/` keyword-leading declarations are **698**, not 697,
with 36 further prose occurrences; `crates/` is **44** files, not 48.

## Measured footprint — 203 files (ORIGINAL TRIAGE; see corrections above)

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
   links to `spec/.../33-declarations.md#8-named-proof-claims--prop-theorem-and-attached-proof`
   and `#83-standalone-theorems--theorem`. **Renaming a spec heading silently breaks
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

## ✅ FORK RULED — (A) HARD RENAME. `dec_5bb4zsfafgkm5` RESOLVED

**Architect ruling `evt_5aem3ec5kmsg8`.** `theorem` becomes the **sole**
standalone checked-theorem declaration keyword. **`lemma` is NOT retained as an
accepted or deprecated alias.**

Rationale, as ruled: the semantic object is already *theorem* in the AST field,
`elaborate_checked_theorem`, and implementation prose, while `theorem` is
collision-free in Ken source; every known surface consumer is first-party and
migrates in-repo. **An alias buys no compatibility beneficiary and creates two
normative spellings for one construct.**

### ⛔ ONE WP, ONE ATOMIC MERGE CANDIDATE

The sequencing objection was acknowledged as **real but not sufficient to
justify language proliferation**. Therefore:

- **Spec-first authoring may happen first on the integration branch.**
- ⛔ **NO partial spec/lexer/corpus slice may land on `main`.**
- The exact candidate must contain **all** of: normative grammar + heading
  changes; lexer/parser/resolved-AST vocabulary (`KwTheorem`, `TheoremDecl`,
  `RDeclKind::Theorem` **and consumers**); formatter keyword strings; catalog;
  conformance; library/docs/agent/tooling surface references; **and every
  changed anchor plus its inbound consumers.**

⇒ **The flip is atomic at the only boundary users observe.** The five-area
ordering below is *authoring* order on the branch, **not** a landing sequence.

### Acceptance — the classification made falsifiable

1. **Emit the fixed-base occurrence set** for `lemma`/`lemmas` plus
   surface-derived identifiers and anchors.
2. **Classify every row** as (a) keyword-contract rename, (b) derived
   identifier/anchor rename **with its consumers**, or (c) **intentional
   ordinary-English leave**.
3. **Make both changes AND leaves review-visible**, then **re-emit against the
   exact candidate** so no newly introduced or unclassified hit escapes.
4. **Positively prove** `theorem` parses, elaborates, and formats.
   **Negatively prove** `lemma name ...` is **rejected, not aliased.**
5. **Exhaustive internal enum breakage is ONE detector, not the net** — also run
   the catalog corpus, conformance/CI, the formatter oracle, and the
   stale-anchor detector.

★ **Ordinary mathematical English remains ordinary English** — a helper result
may still be *called* a lemma. What must disappear is `lemma` **as Ken syntax**,
and identifiers/anchors derived from that surface spelling.

### Merge authority

The single merge Decision requires **Spec/conformance AND Architect** authority,
because normative grammar and implementation surface change together. This is
**not** a §14a doc-only path.

## Authoring order on the branch (NOT a landing sequence — the merge is atomic)

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

## 📍 QUEUE POSITION — ⚠ TWO OPERATOR DIRECTIVES, SAME DAY, IN TENSION

> ### ⚠ UNRESOLVED AS OF 2026-07-25 — the Steward must not kick on its own read
>
> **Directive A (earlier, 2026-07-25):** *"after RT-NATIVE-FNSPLIT closes, run
> kw-theorem"* — recorded below, with an explicit **⛔ do not release it early
> to fill idle enclave time.**
>
> **Directive B (later, 2026-07-25):** *"add a second implementation team
> lane"*, followed by *"KW-THEOREM?"* — the operator naming this node as that
> lane, after an OpenAI capacity reset to 100%.
>
> **The Steward's read, offered and NOT acted on:** B supersedes A's *letter*
> while preserving A's *purpose*. A's stated rationale was that FNSPLIT is the
> standing priority and KW-THEOREM must not displace it. A second lane does not
> displace FNSPLIT — Runtime is working `RT-FNSPLIT-B2V` concurrently and is
> untouched by this WP (measured: zero shared files).
>
> ⛔ **But A is same-day, explicit, and says "do not release early" in terms.
> The Steward does not get to retire that on inference.** The frame is written
> and the contention is measured so the decision is cheap to make — but the
> §2c gate compacts three seats, so **no kickoff until the operator confirms B
> supersedes A.**
>
> ⚠ **One thing B changes that A did not contemplate:** A routed this to the
> **spec enclave**. As a build lane it routes to the **Language ring** instead
> (the mechanical core is `crates/ken-elaborator`), with the spec enclave
> retaining the review vote on `D2`/`D6`. `owner:` has been moved accordingly.

### (superseded framing) NEXT AFTER `RT-NATIVE-FNSPLIT` CLOSES (operator, 2026-07-25)

> **Operator directive, 2026-07-25: *"after RT-NATIVE-FNSPLIT closes, run
> kw-theorem."*** This **supersedes the 2026-07-22 "LAST" placement below.
>
> **Trigger:** `RT-FNSPLIT-B2B` lands ⇒ Steward flips `RT-NATIVE-FNSPLIT` to
> `merged` ⇒ **this is the next WP released.** No further priority check needed;
> the call is made and is a fixed input.
>
> ⚠ **Owner is `spec` — so the receiving unit is the spec enclave** (spec-leader
> + spec-author + conformance-validator), **not** the runtime ring that will
> just have closed FNSPLIT. Compact the enclave unconditionally before the
> kickoff (§2c); it was already compacted once on 2026-07-25 while idle, so
> re-verify rather than assume.
>
> ⛔ **Do not release it early to fill idle enclave time.** The directive is
> ordered *after* FNSPLIT, and FNSPLIT is the standing priority. If the enclave
> is idle in the meantime that is expected, not a stall to fix by promoting this.

### (historical) LAST placement — operator directive, 2026-07-22

**Operator: *"put it at the end of the current work queue."*** This sits behind
every currently-`ready` item:

```
BUDGET-EXHAUST (verify) · F1-37 (runtime) · Q-CLAIM-CLOSURE (runtime)
STR-BIJ (spec-enclave) · DOC-VALIDATION-BINDING (doc)
PUB-VERIFY · MODELS-TIER · CI-SKIPPED-NATIVE-TESTS (steward)
```
…and behind the in-flight `active` set (PX8, RT-SPLIT, DOC-W1, BUDGET-EFF).

~~✅ **Now `ready`**~~ — superseded. The (A)/(B) fork was ruled
(`dec_5bb4zsfafgkm5`); the node was released as the fleet's second lane and has
since **merged**.

## ✅ MERGED 2026-07-25 — `origin/main` = `c72be0b0`, PR #977

Landed on exact **`305dc6d5`**, a corrective descendant of CI-red `963d36ac`.

| | |
|---|---|
| Decision | `dec_74fwejgv6hda0` — `resolved`, read **off the object** |
| authorities | Librarian PASS `evt_524fj8c43q7jg` · CV APPROVE `evt_11tsr3hhmxfbj` · Architect APPROVE `evt_6hk6m7x8xmsn4` — all **fresh exact-SHA** |
| landed tree | **`6f7cf51c`** — identical to the tree asserted **before** publishing |
| void | `dec_286hqjak5kjq8` and every approval on dead `963d36ac` |

⭐ **The lesson this node bought, and it is the one worth keeping.** The first
candidate passed **four independent exact-SHA reviews** and every targeted local
run, then went **red in CI** on `ken_fmt strict_frozen_corpus_gate_is_green`.
⛔ **This very file predicted it by name** — *couplings* item: the formatter
keyword list *"is a canonicalization oracle that fails in CI, not in a targeted
build."* It still happened, because the couplings section is read **once, at
kickoff**, and nothing at candidate-assembly time re-read it. **Presence of a
warning is not evidence it is operative.**

Root cause was layout, not semantics: `layout.rs` `CANONICAL_WIDTH = 96` governs
**declaration-signature** layout, and `theorem` is two characters longer than
`lemma`, so exactly one signature per affected file crossed the boundary while
the migration swapped lines without re-emitting through `ken fmt`.

✅ **Operator directive (2026-07-25): `lemma` is retired from the language
entirely; it may remain in comments and documentation.** Verified satisfied on
landed `main` by **positive control**, not by absence: `Lexer::lex("theorem
lemma")` yields `Token::Ident("lemma")` (an ordinary identifier, not a keyword),
a `lemma` declaration head is rejected, the normative `31-lexical.md` /
`32-grammar.md` have zero occurrences, and a corpus-wide source oracle permits
prose while forbidding Ken source. ⚠ **One residue, deliberately left:**
`provide_lemma` remains a normative **protocol enum value**
(`25-protocol.md`, `SuggestedAction::ProvideLemma`) — an API token, not a
language construct; the mathematical concept *lemma* is not retired.

**Retros owed:** language ring, doc ring, spec enclave.
