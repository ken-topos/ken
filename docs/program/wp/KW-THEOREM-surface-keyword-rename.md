# `KW-THEOREM` — rename the surface keyword `lemma` to `theorem`

> **Operator directive, 2026-07-22.** Rename the Ken surface keyword `lemma` to
> `theorem` across the elaborator, catalog, library, spec, and docs.
>
> **Triage:** `docs/program/issues/KW-THEOREM.md` — the measured footprint, the
> per-area occurrence counts, and the fork history. This frame is the
> shovel-ready brief; the node is the evidence behind it.

## ✅ THE DESIGN FORK IS ALREADY RULED — do not re-open it

**Architect ruling `evt_5aem3ec5kmsg8`, Decision `dec_5bb4zsfafgkm5` RESOLVED:
(A) HARD RENAME.** `theorem` becomes the **sole** standalone checked-theorem
declaration keyword. **`lemma` is NOT retained as an accepted or deprecated
alias.**

Rationale as ruled — cite it, do not re-derive it: the semantic object is
already *theorem* in the AST field, in `elaborate_checked_theorem`, and in
implementation prose; `theorem` is collision-free in Ken source; every known
surface consumer is first-party and migrates in-repo. **An alias buys no
compatibility beneficiary and creates two normative spellings for one
construct.**

⛔ **No migration diagnostic is authorized either.** After the rename, `lemma`
is an ordinary identifier. `AC-4` requires you to **measure and record** what a
pre-rename source now does — **if that failure mode is actively misleading,
hard-stop and route it. It is not an implementer design call.**

## ⛔⛔ ONE WP, ONE ATOMIC MERGE CANDIDATE

The sequencing objection was acknowledged as **real but not sufficient to
justify language proliferation.** Therefore:

- **Spec-first authoring may happen first on the integration branch.**
- ⛔ **NO partial spec / lexer / corpus slice may land on `main`.**
- The exact candidate must contain **all** of: normative grammar + heading
  changes; lexer/parser/resolved-AST vocabulary (`KwTheorem`, `TheoremDecl`,
  `RDeclKind::Theorem` **and consumers**); formatter keyword strings; catalog;
  conformance; library / docs / agent / tooling surface references; **and every
  changed anchor plus its inbound consumers.**

⇒ **The flip is atomic at the only boundary users observe.** The authoring order
below is *authoring* order on the branch, **not** a landing sequence.

## ⛔ THE SCOPING TRAP — `lemma` in prose is NOT always the keyword

★ **This is the whole judgment content of the WP, and a blind sweep gets it
wrong in both directions.**

*Lemma* is also an ordinary English/mathematical word for a helper result. Prose
like *"this lemma supports the main theorem"* is **correct English about
mathematics** and must **NOT** become *"this theorem supports the main
theorem."* Conversely `spec/30-surface/32-grammar.md`'s
`axiom N : T ⇒ lemma N : T = Axiom` **is** the keyword and must change.

| class | action |
|---|---|
| **keyword occurrence** (declaration, grammar production, keyword list, token) | rename |
| **English word** (prose about a helper result) | **leave**, and say so |
| **anchor / identifier derived from the keyword** (section slugs, test names) | rename **with its consumers** |

⛔ **A count of replacements is not evidence.** Cite the classification, and make
the **leave**-decisions as review-visible as the change-decisions — an
unexplained surviving `lemma` is indistinguishable from a missed one.

★ **Ordinary mathematical English remains ordinary English.** What must disappear
is `lemma` **as Ken syntax**, and identifiers/anchors derived from that spelling.

## ⚠ Couplings a sweep will miss

1. **Cross-doc anchors.** `library/learn/reading-ken/02-types-contracts-and-proofs.md`
   links into `spec/.../33-declarations.md#8-named-proof-claims--prop-theorem-and-attached-proof`
   and `#83-standalone-theorems--theorem`. **Renaming a spec heading silently breaks
   every inbound link.** The librarian's stale-anchor mutation is the gate.
2. **Pluralisation.** `lemma` / `lemmas` / `` `lemma`s ``. A `\blemma\b` regex
   misses possessive and plural forms; a naive `lemma→theorem` mangles them.
3. **The formatter keyword list** (`kenfmt_c_capstone.rs:208`) is a **string
   list** — a canonicalization oracle that fails in **CI**, not in a targeted
   build.
4. **`catalog/**` sources are literate `.ken.md`** — every declaration must
   still `ken check`, and the catalog is a **dependency-ordered** corpus.
5. **`RDeclKind::Lemma` is a surface enum variant** — renaming it breaks
   exhaustive matches in every consumer. That is a *feature* (the compiler
   enumerates them), but it means the change is not confined to the elaborator.
6. **Conformance seeds pin exact surface text** and run in CI.

> ## ⛔⛔ EVERY COUNT IN THIS FRAME IS NON-AUTHORITATIVE — DERIVE, DO NOT READ
>
> **Amended 2026-07-25, third correction.** Three separate counts I published
> here were wrong, each caught by a ring re-deriving instead of trusting me:
>
> | count | I said | actual | caught by |
> |---|---|---|---|
> | `library/` occurrence files | 3, then **10** | **11** | `doc-leader` `evt_ksfe2xjyp0q1` |
> | `AC-1` Ken source roots | hand list | wrong 3 ways | `spec-leader` `evt_6zkdcmsrrxy9k` |
> | `catalog/` declarations | 697, then **698** | **696 / 22 files** | `spec-leader` `evt_4vw8nb08s7nz1` |
>
> ★ **The third one is the tell, and it is damning.** The file I missed is
> `library/learn/reading-ken/06-execution.md`, which says *"the five **lemmas**
> above"*. My regex was `\blemma\b` — **which cannot match `lemmas`**, because
> the trailing `s` is not a word boundary. **That is coupling #2 in this very
> frame:** *"Pluralisation … a `\blemma\b` regex misses the possessive/plural
> forms."* **I wrote the warning and then measured with the exact regex the
> warning names.**
>
> ⇒ **The defect is not any one number — it is that I published derived
> measurements at all.** A count in a frame reads as settled and suppresses the
> re-derivation that would catch it. **State the property and the derivation;
> let the ring measure.**
>
> **The authoritative derivations, against the exact candidate:**
>
> ```sh
> # every tracked Ken source, literate and not
> git ls-tree -r --name-only <sha> | grep -E '\.ken(\.md)?$'
> # every occurrence file, case-insensitive, plural-safe
> git grep -il -E 'lemma|lemmas' <sha> -- <path>
> ```
>
> ⛔ **Use `-i` and cover the plural/possessive.** The numbers below are a
> snapshot of `c2c1ba9f` that has **already been proven unreliable three times**
> — they are context for sizing, **never** an input to a completeness check.

## ⚠ Two triage counts were WRONG — both widen the work

Re-measured at `aecdb001` while framing:

1. **`library/` occurrence files: I said 3, then 10; it is 11.** Three are
   **not prose**:
   `library/manifest.toml`,
   `library/agents/evaluations/results-2026-07-24.toml`, and
   `library/agents/evaluations/fixtures/proof-terminals.txt`. **An
   evaluation-results file and a fixture are oracles, not documentation** —
   changing them changes what a check compares against.
2. **`conformance/` includes a raw `.ken` source** —
   `conformance/challenge/C6-lawful-ord-vs-stub/sound-ord-proved.ken` — not only
   literate `.ken.md`. **A glob written for `*.ken.md` misses it.**

3. ⛔ **A keyword-leading count in a literate `.ken.md` MUST be fence-scoped.**
   My "698 keyword-leading declarations" was an unscoped line population and is
   **not a Ken-syntax population at all** — it admits Markdown prose that merely
   begins a line with the word, e.g.
   `Data/Numeric/Nat/Order.ken.md:309` (*"lemma is unavoidable…"*). Toggling on
   fences and counting `^lemma[[:space:]]` **only inside a fence** gives **696
   across 22 files** (`spec-leader`, `evt_4vw8nb08s7nz1`).

   ⭐ **This is a hazard for YOUR sweep, not just for my count.** In literate
   Ken, prose and source share a file, so *any* population you derive over
   `.ken.md` must know whether it is inside a fence. An unscoped `^lemma` sweep
   will rename English.

⇒ **Do not trust any number in this frame.** Re-derive at pickup — fence-scoped
where the file is literate — and **escalate a discrepancy rather than building
around it.**

## Definition sites — the mechanical core, re-derived at `aecdb001`

```
crates/ken-elaborator/src/lexer.rs:60     Token::KwLemma enum variant
crates/ken-elaborator/src/lexer.rs:459    "lemma" => Token::KwLemma
crates/ken-elaborator/src/ast.rs:219      Decl::LemmaDecl { .. }   (also :446, :468)
crates/ken-elaborator/src/elab.rs:3846    RDeclKind::Lemma => elaborate_checked_theorem(...)
crates/ken-elaborator/src/elab.rs:5146    RDeclKind::Lemma => ensure_omega_type(...)
crates/ken-elaborator/src/elab.rs:5673    kind: RDeclKind::Lemma
crates/ken-elaborator/tests/kenfmt_c_capstone.rs:208   formatter keyword STRING list
spec/20-verification/21-spec-syntax.md:180, :403       lemma-decl ::= "lemma" ...
spec/30-surface/32-grammar.md:40                       grammar production
```

## Authoring order on the branch — NOT a landing sequence

**Normative first, then implementation, then corpus** — the spec is the sole
authority, so a catalog edit ahead of it would be unanchored.

1. **`spec/`** — grammar productions, keyword lists, section headings
   (**enclave**). Anchors change here, so inbound-link repair is scheduled with
   it.
2. **`crates/ken-elaborator`** — lexer token, `RDeclKind` variant, formatter
   keyword list (**language ring**). The compiler enumerates consumers.
3. **`catalog/` + `library/`** — the fence-scoped mechanical
   declarations plus the prose classification (**doc ring; the librarian holds the anchor gate**).
4. **`conformance/`** — seed suites, CI-gated.
5. **`docs/` + `agent/`** — prose; largest file count, lowest risk, and where
   the leave-it-in-English class dominates.

## Acceptance criteria

**AC-1 — emit the fixed-base occurrence set.** `lemma` / `lemmas` / possessive
and plural forms, **plus** surface-derived identifiers and anchors.

> ## ⛔ AMENDED 2026-07-25 — MY HAND-ENUMERATED ROOT LIST WAS WRONG THREE WAYS
>
> Reported by `spec-leader` (`evt_6zkdcmsrrxy9k`) and **re-derived independently
> by the Steward** against `c2c1ba9f`. The original AC listed
> `catalog/**/*.ken.md`, `conformance/**/*.ken`, `conformance/**/*.ken.md`,
> `examples/`, and the evaluation fixtures. Measured, that list:
>
> 1. **Missed `catalog/packages/Tooling/Verification/ProofErasureBoundaryChecker.ken`**
>    — `catalog/` is **42 × `.ken.md` + 1 × `.ken`**, and that one file is
>    live and consumed: `include_str!`'d at
>    `crates/ken-interp/src/proof_erasure_checker.rs:15` and enumerated by the
>    formatter corpus gate at `crates/ken-cli/tests/ken_fmt.rs:101`.
> 2. **Named `conformance/**/*.ken.md`, which matches ZERO files.** Conformance
>    holds **15 × `.ken`** and plain `.md` seed docs — no literate Ken at all.
> 3. **Omitted `tooling/highlight-js/sample.ken` entirely** — a tracked Ken
>    source in a root I never listed.
>
> ⇒ **Repo-wide there are 33 non-literate `.ken` sources** (catalog 1,
> conformance 15, examples 16, tooling 1) **and 42 literate `.ken.md`**, all in
> `catalog/`.
>
> ⚠ **The missed catalog file carries ZERO `lemma` occurrences**, so no rename is
> missed *today*. **That is exactly why it is dangerous:** the AC would have gone
> green on an unclosed population and the defect would have surfaced on the next
> corpus-wide migration instead.
>
> ★ **The failure is mine and it is the one this AC exists to prevent.** I wrote
> *"enumerate every Ken source root"* as the rule — and then **hand-enumerated
> the roots.** A hand list is the thing that is never closed. I even flagged the
> `.ken`-vs-`.ken.md` split for `conformance/` and did not re-check `catalog/`.

⛔ **DERIVE the population structurally from the tracked tree — never hand-list
roots:**

```sh
git ls-tree -r --name-only <candidate-sha> | grep -E '\.ken(\.md)?$'
```

**One expression, evaluated against the exact candidate.** Plus the non-source
oracles that are not `.ken` at all — the conformance seed `.md` docs and
`library/agents/evaluations/{results-*.toml,fixtures/proof-terminals.txt}`.

**Positive control: a deliberately planted `lemma` declaration in EACH class —
literate `.ken.md`, non-literate `.ken`, and each non-source oracle — is SEEN by
the sweep.** A sweep that grew one arm per missed file has reproduced the bug it
exists to prevent.

**AC-2 — classify every row** as (a) keyword-contract rename, (b) derived
identifier/anchor rename **with its consumers**, (c) **intentional
ordinary-English leave**, or (d) **intentional RESIDUAL** (below). Changes *and*
leaves are review-visible.

> ### ⛔ AMENDED 2026-07-25 — CELL (d) WAS MISSING, AND ITS ABSENCE IS UNSAFE
>
> `doc-leader` (`evt_16nahb3tmybn1`), grounded from the doc-author's fixed-base
> classification: the original three cells **cannot classify a deliberate
> mention of the retired spelling kept for historical, diagnostic, or control
> purposes.** Such a row is neither a rename target nor ordinary mathematical
> English.
>
> **(d) intentional residual — RETAIN, and mark it so it is distinguishable
> from a miss.** Known members:
>
> - `docs/program/IMPLEMENTATION-PROGRESS.md` and
>   `docs/program/diary/CURRENT-BRIEFING.md` — status narrative *about* the
>   rename.
> - The `KW-THEOREM` issue file and **this frame**, whose entire subject is the
>   old spelling. ⚠ **A sweep that renames its own frame has destroyed the
>   record of what it did.**
> - ⭐ **`AC-4`'s negative control, which MUST contain `lemma name …` by
>   construction** — it exists to prove that spelling is *rejected*. **A sweep
>   that renames it silently converts the negative control into a second
>   positive one and `AC-4` then proves nothing.**
>
> ★ **This is the failure mode where a taxonomy with no cell for the honest
> answer reads as complete.** With only three cells, every residual must be
> mis-filed as (a) — destroying the control — or as (c), asserting that
> `lemma name : P = pf` is ordinary English, which is false. **Neither
> mis-filing is visible in a green run.**
>
> ⇒ Cell (d) rows are **enumerated explicitly in the classification**, not left
> to survive as unexplained hits. The four old-slug locators in the issue and
> frame are a **separate** class — they are derived-anchor consumers under
> **(b)** and must move with their anchors.

**AC-3 — re-emit against the exact merge candidate**, so no newly introduced or
unclassified hit escapes.

**AC-4 — positive and negative, on the same harness.** Positively prove
`theorem` **parses, elaborates, and formats** (⛔ not a parse test — full
elaboration). Negatively prove `lemma name …` is **rejected, not aliased**,
asserting the **exact** diagnostic, never `is_err`. ⚠ **A negative check passes
for any reason** — it is discharged only alongside the positive control. Record
the measured pre-rename failure mode per the settled input above.

**AC-5 — exhaustive enum breakage is ONE detector, not the net.** Also run the
catalog corpus, conformance/CI, the formatter oracle, and the stale-anchor
detector.

**AC-6 — anchors resolve.** Every cross-document link into a renamed `spec/`
anchor still resolves. Assert it; do not eyeball it.

**AC-7 — the attestation ledger is regenerated and consistent.** All 17
`catalog/` rows in `library/SOURCE-ATTESTATIONS` reflect the migrated sources
and `library/STATUS.md` agrees. ⭐ **Predict the row count before regenerating,
then compare.** A ledger merges as a **silent union** — different rows, no
conflict, both halves independently correct and jointly wrong.

**AC-8 — no regression.** Green in **CI** — `--workspace`, `--locked`, and the
conformance suite run on GitHub, never on this box.

## Merge authority

⛔ **The single merge Decision requires Spec/conformance AND Architect
authority**, because normative grammar and implementation surface change
together. **This is NOT a `§14a` doc-only path.**

## Contention — measured at `aecdb001`

**Against `RT-FNSPLIT-B2V` (the other live lane): NONE.** All 44 crate files
carrying `lemma` are in `ken-elaborator`; **zero in `ken-runtime`**. The only WP
frames touched (`PX8-T`, `PX8-F`) are `draft`.

⚠ **The doc ring is ABSORBED into this WP, not parked** — authoring step 3 is
its work and the librarian holds the anchor gate. ⛔ **It must not also run an
independent doc WP**: `library/SOURCE-ATTESTATIONS` carries 17 `catalog/` rows
that this WP rewrites wholesale, so any concurrent doc work collides on the
ledger axis — **and a ledger collision merges clean and wrong.**

## Standing

- ⛔ **Local builds/tests are TARGETED ONLY** — `scripts/ken-cargo -p
  ken-elaborator` / `--test <name>`. **Never `--workspace`** (`COORDINATION §12`,
  operator hard rule). Workspace-green and `--locked` mean **green in CI**.
- ⛔ **Do not touch `crates/ken-runtime`** — `RT-FNSPLIT-B2V` is live there. If
  this WP appears to need a change in it, that is a frame-boundary fact:
  **hard-stop and route it**, do not reach across.
- **Report an unpushed ref and KEEP GOING.** Build seats have no GitHub
  credential by design; the Steward pushes. Raising it is not gating on it.
- Read `agent/playbooks/tools/pin-a-property.md` before writing any assertion.
- **Every anchor above is perishable.** Escalate a false fixed input; do not
  build around it.
