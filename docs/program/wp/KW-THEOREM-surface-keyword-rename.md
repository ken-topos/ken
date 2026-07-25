# `KW-THEOREM` — rename the surface keyword `lemma` to `theorem`

> **Operator directive, 2026-07-22.** Rename the Ken surface keyword `lemma` to
> `theorem` across the elaborator, catalog, library, spec, and docs.
>
> **Released 2026-07-25 as the fleet's SECOND implementation lane**, running
> concurrently with `RT-FNSPLIT-B2V` in `crates/ken-runtime`. See the contention
> section — the concurrency is measured, not assumed.

## Why this is worth doing on intrinsic merits, not just on the directive

★ **The rename closes a naming seam rather than opening one.** The surface said
`lemma` while the implementation, the AST field, and the doc comments already
said *theorem*:

```
crates/ken-elaborator/src/lexer.rs:60   KwLemma, // "lemma" — standalone checked theorem
crates/ken-elaborator/src/ast.rs:222    theorem: Type,          <- the AST field is ALREADY `theorem`
crates/ken-elaborator/src/elab.rs:5691  fn elaborate_checked_theorem(
```

`docs/PRINCIPLES.md` — *subsume-don't-proliferate*. One concept had two names
across a boundary; this collapses them. **State that argument in the `D6` ADR**;
do not rest the ADR on the directive alone.

✅ **`theorem` is free as a keyword — verified.** It occurs in 105 files today,
**every occurrence prose, doc comment, or internal Rust naming** — never a Ken
surface keyword, never an identifier in `catalog/**/*.ken.md` or `examples/`.

## ⛔ CONTENTION — measured at `aecdb001`, and it binds in ONE direction only

**Against `RT-FNSPLIT-B2V` (the other live lane): NONE.**

| axis | measurement | verdict |
|---|---|---|
| crates carrying `lemma` | **44 files, ALL in `ken-elaborator`** | **zero in `ken-runtime`** ✅ |
| active WP frames carrying `lemma` | `PX8-T`, `PX8-F` only — both `draft` | no live frame collides ✅ |

⛔ **Against the DOC TRACK: TOTAL, for this WP's whole duration.**

`library/SOURCE-ATTESTATIONS` carries **17 rows for `catalog/` sources**, and
this WP rewrites **698 keyword-leading declarations** across 23 catalog files.
**Every attested catalog source changes hash.** The doc track is the fleet's one
standing concurrency exception and it lives in `library/` + `agent/` — the same
ledger. ⇒ **The doc track is PARKED while this runs.** Two lanes, not three.

★ **This is the ledger axis, not the file axis.** Two WPs contend when one
mutates a source the other's domain attests, even with disjoint file scopes. A
`SOURCE-ATTESTATIONS` collision **merges as a silent union** — different rows,
no conflict, both halves independently correct and jointly wrong. Do not expect
git to catch it.

## Measured footprint — re-derive every number at pickup

| area | files | character |
|---|---|---|
| `catalog/` | 23 | **698 keyword-leading declarations** + 36 prose occurrences |
| `docs/` | 67 | prose + historical WP frames |
| `crates/` | **44, all `ken-elaborator`** | **the only place the keyword is DEFINED** |
| `spec/` | 23 files / 100 occurrences | **normative grammar + section anchors** |
| `conformance/` | 15 files / 41 occurrences | **14 `.md` + 1 raw `.ken`** — two oracle classes |
| `agent/` | 17 | playbooks/memory — prose |
| `library/` | **10** | ⚠ the node file says 3; **it is 10** — see below |
| `tooling/` + `examples/` | 3 + 1 | — |

⚠ **Two triage corrections found while framing, both of which widen the work:**

1. **`library/` is 10 files, not 3.** The extra ones are not prose: `library/manifest.toml`, `library/agents/evaluations/results-2026-07-24.toml`, and `library/agents/evaluations/fixtures/proof-terminals.txt`. **An evaluation-results file and a fixture are oracles, not documentation** — changing them changes what a check compares against.
2. **`conformance/` includes a raw `.ken` source**, `conformance/challenge/C6-lawful-ord-vs-stub/sound-ord-proved.ken`, not only literate `.ken.md`. A glob written for `*.ken.md` misses it.

⇒ **Do not trust this table either.** It is a measurement of `aecdb001`; re-derive
at pickup and **escalate a discrepancy rather than building around it.**

## Definition sites — the mechanical core

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

## Deliverables

### `D1` — the keyword, at its definition sites

Rename the token, the keyword map entry, the AST variant, the `RDeclKind`
variant, every dispatch arm, and the **formatter's keyword string list**.
Internal names already saying *theorem* stay as they are — they were right.

### `D2` — the normative grammar

Both `lemma-decl` productions in `spec/20-verification/21-spec-syntax.md` and
the one in `spec/30-surface/32-grammar.md`, plus **section anchors and
cross-document links into them**. ⚠ `library/` contains cross-doc anchors into
`spec/`; a renamed anchor breaks them silently.

### `D3` — the catalog corpus, and its attestations

All 698 keyword-leading declarations, plus the 36 prose occurrences. **Then
regenerate `library/SOURCE-ATTESTATIONS` and reconcile `library/STATUS.md`.**
⛔ A migrated catalog with a stale ledger is a broken deliverable, not a
follow-up.

### `D4` — the oracle classes

Conformance seed suites (14 `.md`) **and** the raw `.ken` challenge source, plus
`library/agents/evaluations/results-2026-07-24.toml` and
`library/agents/evaluations/fixtures/proof-terminals.txt`. ⛔ **A sweep glob must
enumerate every Ken source root** — `*.ken.md`, `*.ken`, and the fixture files.

### `D5` — prose currency

`docs/` (67), `agent/` (17), `library/` prose, `tooling/`, `examples/`.
⛔ **Do not rewrite history.** Closed WP frames, retros, and diary entries that
describe what was true when written stay as written; ADRs are amended, not
edited. Migrate *live* prose only, and say in the retro where you drew that line.

### `D6` — the ADR

Record the rename with the *subsume-don't-proliferate* argument above, the
measured footprint, and the alias decision below.

## ⛔ Settled inputs — these are NOT yours to re-open

1. **No compatibility alias.** `lemma` is removed, not deprecated. The entire
   corpus migrates atomically in-tree and Ken has no external consumers yet; an
   alias would install two spellings for one concept permanently — the exact
   thing this WP exists to remove.
2. **No migration diagnostic is in scope.** After `D1`, `lemma` becomes an
   ordinary identifier. ⭐ **`AC-4` requires you to MEASURE and RECORD what a
   pre-rename source now does** — it does not authorize you to design a nicer
   error for it. **If the measured failure mode is actively misleading, that is a
   hard-stop to route, not an implementer design call.**
3. The rename target is `theorem`. Settled by directive; the ADR justifies it,
   it does not re-decide it.

## Acceptance criteria

**AC-1 — the new spelling works, positively.** A catalog source declaring a
`theorem` **elaborates and checks**. ⛔ Not a parse test — full elaboration.

**AC-2 — the old spelling no longer declares a theorem.** ⚠ **A negative check
passes for any reason**, so this AC is discharged only alongside `AC-1`'s
positive control on the same harness. Assert the **exact** diagnostic, never
`is_err`.

**AC-3 — no surface `lemma` survives, by structural closure.** ⛔ **Not a grep
for one spelling.** Enumerate every Ken source root (`catalog/**/*.ken.md`,
`conformance/**/*.ken`, `conformance/**/*.ken.md`, `examples/`, the evaluation
fixtures) from **one** glob definition, and assert the population is zero.
**Positive control: a deliberately planted `lemma` declaration in each root
class is SEEN by the sweep.** A sweep that grew one arm per missed file has
reproduced the bug it was written to prevent.

**AC-4 — the pre-rename failure mode is measured and recorded**, per settled
input 2. Record the exact diagnostic in the retro; do not improve it.

**AC-5 — the attestation ledger is regenerated and consistent.** Every one of
the 17 catalog rows in `library/SOURCE-ATTESTATIONS` reflects the migrated
source, and `library/STATUS.md` agrees. ⭐ **Predict the row count before
regenerating, then compare** — a silently-unioned ledger is the failure this
lane's contention analysis exists to prevent.

**AC-6 — spec anchors resolve.** Every cross-document link into a renamed
`spec/` anchor still resolves. Assert it; do not eyeball it.

**AC-7 — the formatter round-trips.** `kenfmt` emits `theorem` and its capstone
keyword list has no stale entry.

**AC-8 — no regression.** Green in **CI** — `--workspace` and `--locked` and the
conformance suite run on GitHub, never on this box.

## Standing

- ⛔ **Local builds/tests are TARGETED ONLY** — `scripts/ken-cargo -p
  ken-elaborator` / `--test <name>`. **Never `--workspace`** (`COORDINATION §12`,
  operator hard rule). Workspace-green and `--locked` mean **green in CI**.
- **Report an unpushed ref and KEEP GOING.** Build seats have no GitHub
  credential by design; the Steward pushes. Raising it is not gating on it.
- **The other lane is live.** `RT-FNSPLIT-B2V` is active in `crates/ken-runtime`.
  ⛔ **Do not touch `crates/ken-runtime`.** If this WP appears to need a change
  there, that is a frame-boundary fact — **hard-stop and route it**, do not
  reach across.
- Read `agent/playbooks/tools/pin-a-property.md` before writing any assertion.
- **Every anchor above is perishable.** Escalate a false fixed input; do not
  build around it.
