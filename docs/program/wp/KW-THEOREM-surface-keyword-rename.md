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
   links into `spec/.../33-declarations.md#8-named-proof-claims--prop-lemma-and-attached-proof`
   and `#83-standalone-lemmas--lemma`. **Renaming a spec heading silently breaks
   every inbound link.** The librarian's stale-anchor mutation is the gate.
2. **Pluralisation.** `lemma` / `lemmas` / `` `lemma`s ``. A `\blemma\b` regex
   misses possessive and plural forms; a naive `lemma→theorem` mangles them.
3. **The formatter keyword list** (`kenfmt_c_capstone.rs:208`) is a **string
   list** — a canonicalization oracle that fails in **CI**, not in a targeted
   build.
4. **`catalog/**` sources are literate `.ken.md`** — all 698 declarations must
   still `ken check`, and the catalog is a **dependency-ordered** corpus.
5. **`RDeclKind::Lemma` is a surface enum variant** — renaming it breaks
   exhaustive matches in every consumer. That is a *feature* (the compiler
   enumerates them), but it means the change is not confined to the elaborator.
6. **Conformance seeds pin exact surface text** and run in CI.

## ⚠ Two triage counts were WRONG — both widen the work

Re-measured at `aecdb001` while framing:

1. **`library/` is 10 files, not 3**, and three are **not prose**:
   `library/manifest.toml`,
   `library/agents/evaluations/results-2026-07-24.toml`, and
   `library/agents/evaluations/fixtures/proof-terminals.txt`. **An
   evaluation-results file and a fixture are oracles, not documentation** —
   changing them changes what a check compares against.
2. **`conformance/` includes a raw `.ken` source** —
   `conformance/challenge/C6-lawful-ord-vs-stub/sound-ord-proved.ken` — not only
   literate `.ken.md`. **A glob written for `*.ken.md` misses it.**

Also re-counted: `catalog/` keyword-leading declarations are **698** (not 697)
plus 36 prose occurrences; `crates/` is **44** files (not 48), **all in
`ken-elaborator`**; `spec/` is 100 occurrences across 23 files.

⇒ **Do not trust these numbers either.** They measure `aecdb001`. Re-derive at
pickup and **escalate a discrepancy rather than building around it.**

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
3. **`catalog/` + `library/`** — the 698 mechanical declarations plus the prose
   classification (**doc ring; the librarian holds the anchor gate**).
4. **`conformance/`** — seed suites, CI-gated.
5. **`docs/` + `agent/`** — prose; largest file count, lowest risk, and where
   the leave-it-in-English class dominates.

## Acceptance criteria

**AC-1 — emit the fixed-base occurrence set.** `lemma` / `lemmas` / possessive
and plural forms, **plus** surface-derived identifiers and anchors. ⛔ **One glob
definition covering every Ken source root** — `catalog/**/*.ken.md`,
`conformance/**/*.ken`, `conformance/**/*.ken.md`, `examples/`, and the
evaluation fixtures. **Positive control: a deliberately planted `lemma`
declaration in each root class is SEEN by the sweep.** A sweep that grew one arm
per missed file has reproduced the bug it exists to prevent.

**AC-2 — classify every row** as (a) keyword-contract rename, (b) derived
identifier/anchor rename **with its consumers**, or (c) **intentional
ordinary-English leave**. Changes *and* leaves are review-visible.

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
