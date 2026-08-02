# Documentation wave reconciliation

This is the durable D1–D4 record for `DOC-PROGRAM-WAVE-RECONCILE`, measured
against base `678a8c62ece7f315ea25daafb6e0ea41a70e8204`. It reconciles the
program's summary surfaces with the dated measurements already embedded in
`12-documentation-program.md`. It does not remeasure those blocks or author a
residual page.

## D1–D3 — summary-surface reconciliation

The status line now records 89 registered documents and gives a separate
item-derived state for Waves 3–6. The section 4 table and section 4b headings
use those same states.

| Wave | Section 4b heading | Section 4 table | Newest dated block | Agreement |
|---|---|---|---|---|
| 3 | `LANDED · 9 of 9` | `LANDED — 9 of 9` | Six subjects were already delivered; `DOC-W3-DEPDATA` was the one residual. The how-tos and migration also landed. | Yes: every Produces item is landed. |
| 4 | `PARTIAL · 5 of 10` | `PARTIAL — 5 of 10 landed` | The generation-capability block requires authored labels when facts cannot be generated. The terminal residual then landed five surfaces and named five missing mechanisms. | Yes: delivered work is retained and mechanism-limited items remain deferred. |
| 5 | `PARTIAL · 6 of 10` | `PARTIAL — 6 of 10 landed` | The checked-format capability report permits cards plus five indexes and holds four fact classes. | Yes: the six supported products landed and the four held indexes remain gated. |
| 6 | `PARTIAL · 1 of 4` | `PARTIAL — agent-pack evaluation landed; other 3 of 4 deferred or foreclosed` | The dated table disposes three items as deferred or foreclosed and names agent-pack evaluation as the live component. | Yes: `DOC-W6-AGENT-EVAL` landed and no removed mechanism is revived. |

The state is derived from the item register below, not from the number of
merged nodes.

### Findings classification control

The authority is `07-catalog-style-guide.md` section 5. It says that the
"Findings section is retired from the outsider-facing catalog entry" while
its function has moved to the "live gap-escalation flow." It also says that
kernel-reduction defects, sugar candidates, and abstraction candidates remain
high-value signals and "route live, not through a catalog section."

Against that distinction, the eleven fixed sites in
`06-catalog-campaign.md` classify as follows. The two retired entry-section
claims were corrected by `DOC-PROGRAM-SELF-REFUTE`; all live flow, filing, and
ownership claims remain byte-unchanged here.

| Base site | Classification and current disposition |
|---|---|
| line 78 | **Live, unchanged.** Findings are a dogfooding output routed for action, not a catalog-entry section. |
| lines 92–93 | **Retired entry-section claim, already corrected.** The standing-section claim now routes through the live gap-escalation flow. |
| line 212 | **Live, unchanged.** The heading names the campaign's routing and teaming home. |
| line 219 | **Live, unchanged.** The loop separates authors from fixers. |
| line 231 | **Live, unchanged.** This is team ownership for filing signals. |
| lines 239–240 | **Live, unchanged.** This is filing discipline, not an entry-format section. |
| lines 289–290 | **Live, unchanged.** The guide participates in the package-refinement and signal cadence. |
| line 301 | **Live, unchanged.** Acting on retros is dogfooding purpose, not an entry section. |
| line 310 | **Live, unchanged.** Language remains a destination in the routing loop. |
| line 351 | **Retired entry-section claim, already corrected.** Findings were removed from the standard entry format and the live route was stated separately. |
| line 425 | **Live, unchanged.** This names the Foundation filing skill. |

This closes at nine live sites left unchanged and two stale entry-section
claims already corrected. In particular, the routing claim at line 78, the
filing claim at lines 239–240, and the team-ownership claim at line 231 are
preserved and grounded by section 5's live-channel rule.

## D4 — exhaustive Produces-item residual register

The Produces lists contain 9 items for Wave 3, 10 for Wave 4, 10 for Wave 5,
and 4 for Wave 6. The register has exactly 9, 10, 10, and 4 rows.

### Wave 3 — 9 of 9

| # | Produces item | Disposition |
|---|---|---|
| 3.1 | Contracts | **landed** — `library/learn/reading-ken/02-types-contracts-and-proofs.md` |
| 3.2 | Dependent data | **landed** — `library/guide/dependent-data.ken.md`; merged `DOC-W3-DEPDATA` |
| 3.3 | Proofs | **landed** — `library/guide/proof-techniques.ken.md` and the reading spine's chapter 02 |
| 3.4 | Effects | **landed** — `library/learn/reading-ken/04-effects-capabilities-and-authority.md` |
| 3.5 | Security | **landed** — `library/learn/reading-ken/04-effects-capabilities-and-authority.md` |
| 3.6 | Packages | **landed** — `library/learn/reading-ken/05-packages-and-provenance.md` |
| 3.7 | Execution | **landed** — `library/learn/reading-ken/06-execution.md` |
| 3.8 | Diagnostic-driven how-to recipes | **landed** — merged `DOC-W3-HOWTO`; five pages under `library/how-to/` |
| 3.9 | `catalog/guide/` migration | **landed** — merged `DOC-W3-GUIDE`; checked pages under `library/guide/` |

### Wave 4 — 5 landed, 5 deferred

| # | Produces item | Disposition |
|---|---|---|
| 4.1 | Language reference | **landed** — `library/guide/surface-reference.ken.md`; merged `DOC-W4-LANGUAGE` found no named gap |
| 4.2 | Verification reference | **landed** — `library/guide/proof-techniques.ken.md` and the reading spine; `DOC-W4-RESIDUAL` closed the row as already delivered |
| 4.3 | Toolchain reference | **landed** — `library/reference/toolchain/`; merged `DOC-W4-TOOLCHAIN` |
| 4.4 | Runtime reference | **landed** — `library/learn/reading-ken/06-execution.md`; `DOC-W4-RESIDUAL` closed the row as already delivered |
| 4.5 | Platform reference | **landed** — `library/reference/platform/README.md`; merged `DOC-W4-RESIDUAL` |
| 4.6 | Diagnostics reference | **deferred** — gate: a unified public diagnostic registry or derivation interface spanning kernel, runtime, and host identities |
| 4.7 | Symbol index | **deferred** — gate: a maintained visibility-filtered projection joining public exports to stable checked-core symbols, plus an exporter |
| 4.8 | Keyword index | **deferred** — gate: a maintained extraction/export path over the normative and lexer inventories |
| 4.9 | Diagnostic index | **deferred** — gate: the same unified public diagnostic registry or derivation interface as 4.6 |
| 4.10 | Glossary index | **deferred** — gate: a maintained extraction path from `spec/00-overview.md` section 8 into `library/` |

### Wave 5 — 6 landed, 4 deferred

| # | Produces item | Disposition |
|---|---|---|
| 5.1 | One card per live package | **landed** — 39 cards under `library/reference/catalog/`; merged `DOC-W5A-CARD-FORMAT`, `DOC-W5B-CARDS-APP-DATA`, and `DOC-W5C-CARDS-CAPABILITY` |
| 5.2 | Subject index | **landed** — `library/reference/catalog/subjects.md` |
| 5.3 | Declaration/type index | **landed** — `library/reference/catalog/declarations.md` |
| 5.4 | Law index | **landed** — `library/reference/catalog/laws.md` |
| 5.5 | Effect/capability index | **landed** — `library/reference/catalog/effects-and-capabilities.md` |
| 5.6 | Assurance index | **landed** — `library/reference/catalog/assurance.md` |
| 5.7 | Platform index | **deferred** — gate: per-package instantiation of the reserved `platform` facet; the catalog decision remains open |
| 5.8 | Maturity index | **deferred** — gate: per-package instantiation of the reserved `maturity` facet; the catalog decision remains open |
| 5.9 | Dependency index | **deferred** — gate: a package-level checked dependency projection for literate catalog leaves |
| 5.10 | Reverse-dependency index | **deferred** — gate: the complete package dependency projection in 5.9 plus a maintained inversion over it |

### Wave 6 — 1 landed, 3 deferred or foreclosed

| # | Produces item | Disposition |
|---|---|---|
| 6.1 | Static searchable HTML and offline artifact | **deferred** — gate: a real reader population that needs an artifact beyond repository Markdown |
| 6.2 | Versioned snapshots and migration notes | **deferred** — gate: Ken's first public release; `library/releases/` remains absent until then |
| 6.3 | Post-merge changes wired to an as-built queue | **deferred (foreclosed under current policy)** — gate: a new operator policy; `f52b0f61` and `LIB-GATE-DECOUPLE` removed the post-merge alarm and live documentation/content CI coupling |
| 6.4 | Measurement set | **landed** — agent-pack evaluation results landed as merged `DOC-W6-AGENT-EVAL`; the dated block records the user-dependent and removed-gate components without reviving them |

### Next-release answer

There is **no releasable-now Produces item** in Waves 3–6. Wave 3 is complete;
the Wave 4 and Wave 5 residuals each require a named mechanism or fact source;
and Wave 6's residuals require readers, a public release, or a new operator
policy. The doc ring therefore has no page to frame directly from this
register.

## Controls

- **AC-1:** the four-row comparison above shows the heading, table, and newest
  dated block agree for every wave.
- **AC-2:** Produces-list/register counts are Wave 3 `9/9`, Wave 4 `10/10`,
  Wave 5 `10/10`, and Wave 6 `4/4`.
- **AC-3:** the candidate scope is restricted to `docs/program/`.
- **AC-4:** the explicit next-release answer is no releasable-now item, with
  every residual tied to its gate.
- **AC-5:** the three dated blocks are claim-unchanged: Wave 3's
  `c777d2d4` seven-subject measurement, Wave 4's `7fa65b20` generation
  precondition, and Wave 6's `5a0fd8e6` gating table. This work changes only
  summary surfaces around them.
