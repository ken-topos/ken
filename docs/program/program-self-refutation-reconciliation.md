# Program self-refutation reconciliation

This is the durable D1-D4 record for `DOC-PROGRAM-SELF-REFUTE`. It reconciles
current program law with the removal record already present in
`12-documentation-program.md` section 4. It does not revive a registry, add a
test oracle, or change a release-point artifact.

## D1 — derived authority without an invented runner

The load-bearing decision is unchanged. Before this work package it read:

> `library/` **must not introduce normative language.** Where a reference page
> restates a rule for usability it cites the exact spec section, and a drift
> gate verifies that section still exists.

It now reads:

> `library/` **must not introduce normative language.** Where a reference page
> restates a rule for usability it cites the exact spec section. Review verifies
> that the citation names a real section; no live runner checks it.

The derived-not-normative rule and exact-citation requirement retain their
force. Only the false enforcement mechanism changed. The boundary now agrees
with the dated measurement at `12-documentation-program.md:221-235`: the
registry is unreachable and migration verification is local rather than a
standing gate.

## D2 — release-point currency without a build-output channel

The currency rule still uses source revisions, generated `STATUS.md`, and the
prohibition on hand-editing currency into pages. It now says that generation
happens at release points. The removed words, "and build output," named no real
channel: the repository has no GitHub workflow reference to
`gen-doc-status.sh` or `library/REVISION`, and
`12-documentation-program.md:635-641` records that live documentation/content
CI coupling was removed.

## D3 — the eleven Findings sites

The classification authority is `07-catalog-style-guide.md:127-146`. Its
distinction is exact: the outsider-facing **Findings section** is retired, but
kernel-reduction defects, sugar candidates, and abstraction candidates remain
high-value signals that route through the live gap-escalation flow. The eleven
fixed `06-catalog-campaign.md` sites classify as follows.

| Base site | Classification and disposition | Section 5 grounding |
|---|---|---|
| line 78 | **live; left unchanged.** Findings remain a first-class dogfooding output, and the sentence points to routing rather than an entry section. | Lines 129-134 preserve the dogfooding instrument and migrate its function to the live flow. |
| lines 92-93 | **stale; corrected.** "captured in a standing section" became routing through the live gap-escalation flow. | Lines 130-136 retire the section as a stale, inward-facing duplicate. |
| line 212 | **live; left unchanged.** The heading identifies the campaign's routing and teaming home. | Lines 131-134 preserve the live flow and its durable routing. |
| line 219 | **live; left unchanged.** The Findings loop separates authors from fixers. | Lines 129-134 retain the dogfooding function through the live flow. |
| line 231 | **live; left unchanged.** This is team ownership for filing signals. | Lines 140-142 require uncaptured gaps to be routed before a retired section is removed. |
| lines 239-240 | **live; left unchanged.** This is filing discipline, not an entry-format section. | Lines 131-134 preserve escalation and durable capture. |
| lines 289-290 | **live; left unchanged.** The guide mirrors the package refinement and signal cadence. | Lines 129-134 keep writing real Ken as the instrument and move its function to a live channel. |
| line 301 | **live; left unchanged.** Acting on retros is the campaign's dogfooding purpose; it asserts no entry section. | Lines 129-134 preserve the instrument and durable flow. |
| line 310 | **live; left unchanged.** Language remains a destination in the routing loop. | Lines 145-146 say the high-value signals still route live. |
| line 351 | **stale; corrected.** Findings was removed from the standard entry-format list and the live route was stated separately. | Lines 138-143 forbid new Findings sections and preserve the signal through the channel. |
| line 424 | **live; left unchanged.** This names the Foundation filing skill. | Lines 140-142 require harvesting and routing uncaptured gaps. |

Nine sites therefore remain byte-unchanged. The two corrections remove only
claims about the retired per-entry section; they do not retire the loop.

## D4 — assurance-language reconciliation

### Method and current sites

The control was a claim-shape census, not a list of gate verbs. It examined
current and historical statements about the closed library-validation object
population: the validation registry and its runner, source-anchor and currency
checks, release-point generators, checked-example execution, and failures said
to reach CI or a build. Each assertion was read in context and classified by
whether it claims present execution, records an observed historical execution,
or preserves a superseded frame assumption.

The current assertions and their dispositions are:

| Site | Disposition |
|---|---|
| `12-documentation-program.md:70-72` | **corrected.** Exact citation remains mandatory; review verifies the named section and no live runner is claimed. |
| `12-documentation-program.md:124-126` | **corrected.** Generated `STATUS.md` remains a release-point record; the nonexistent build-output channel was removed. |
| `06-catalog-campaign.md:29` | **corrected.** RQ-3 still requires byte preservation, but its conformance cell now names candidate-time comparison and the absence of a standing runner. |
| `12-documentation-program.md:213-219` | **dated live evidence; left unchanged.** One ordinary detector test really runs in CI and mutation-proves checked-example classification. It is not the unreachable registry runner. |
| `12-documentation-program.md:221-245` | **current control; left unchanged.** It measures the registry as unreachable and selects migration-local extraction rather than standing coupling. |
| `12-documentation-program.md:635-641` | **current removal policy; left unchanged.** It records release-point drift after removal of live documentation/content CI coupling. |
| `13-documentation-migration-ledger.md:30-37` | **current migration-local rule; left unchanged.** It requires destination extraction without claiming a standing library-wide CI gate. |
| `agent-citation-routing.md:304-383` | **current reconciliation record; left unchanged.** It distinguishes corrected artifacts, historical evidence, and the two program-law claims corrected here. |
| `issues/DOC-PROGRAM-SELF-REFUTE.md:14-35` | **defect record; left unchanged.** It quotes Sites A and B as contradictions to repair, not as current enforcement. |
| `wp/DOC-PROGRAM-SELF-REFUTE.md:8-58` | **work-package evidence; left unchanged.** It quotes the false clauses, the dated registry measurement, and the judgments that govern this correction. |

### Population closure

The population was derived from the assurance objects, not from spellings such
as "gate checks". The five objects are (1) the registered-record runner,
(2) the cited-source comparison performed by `gen-doc-status.sh`, (3) the
attestation ledger and its `REVISION`/`STATUS.md` outputs, (4) publisher or CI
consumers of those results, and (5) release-point use of the retained
generators. At rejected candidate `de5c7491`, repository-wide searches for the
object names and their producers selected 484 candidate lines in 101
`docs/program/**/*.md` files. That count is a reproducible candidate-selection
measurement, not the answer. Reading the surrounding assertion reduced the
candidate set to the dispositions below. The closure control was the removal
record:
`LIB-GATE-DECOUPLE.md:102-120` identifies both the removed publisher check and
the stale success sentence, while `12-documentation-program.md:635-641` states
the surviving release-point policy. Every candidate assertion must therefore
classify as current no-live-coupling law, dated execution evidence, a
superseded/retired assumption, or release-point shorthand. None is silently
dropped because it avoids a particular verb.

### Exact runner-name history

The exact historical runner-name population approved by `DOC-AGENT-CITE` is
unchanged:

- `diary/2026/Jul/25.md:1116,1234` records observed implication and failure
  while the runner was live.
- `issues/LIB-GATE-DECOUPLE.md:66,164,169,200` records the failure, classifier
  trap, former test body, and removal finding.
- `wp/LIB-GATE-DECOUPLE-remove-the-ci-coupling.md:51,56,104` preserves the
  classifier trap, former test body, and failing control in the removal frame.
- `issues/DOC-ATTEST-LIVING.md:11,20,55` records the former failure and then
  retires its live premise after decoupling.
- `wp/RT-FNSPLIT-B2A-S-selection-defunctionalization.md:346` and
  `wp/RT-FNSPLIT-B2A-C-plan-lowering-correspondence.md:369` retain superseded
  warnings from the period when the runner still fired in CI.
- `wp/DOC-CATALOG-CONTENTS-preflight.md:190-201` is a superseded frame
  assumption that describes the former targeted suite and CI inclusion. It is
  historical planning evidence, not current program law.

These six dated/removal files and the preflight frame are intentionally left
unchanged. The literal runner-name population is only one subset of the
claim-shaped population; it is not used as the closure argument.

### Same assurance object, different language

The following assertions do not use the runner's name, but speak about the same
publisher, currency, ledger, or release-point objects. Each is explicit here so
historical prose cannot be mistaken for current law.

| Site | Disposition |
|---|---|
| `issues/VIS-BR-LITERAL.md:91-103` | **superseded issue guidance; left unchanged.** It says a cited-source edit trips CI and therefore requires a same-PR re-attestation. That was the operating premise when PR 908 exposed the coupling; `LIB-GATE-DECOUPLE` later removed it. |
| `issues/DOC-VALIDATION-BINDING.md:28-36` | **superseded issue guidance; left unchanged.** Its post-merge publisher freeze describes the former publisher contract, not the current release-point policy. |
| `wp/ABI-R1-capability-prose-currency.md:145-148` | **dated acceptance criterion; left unchanged.** It required both retained generators to be green on that merge result. It is evidence about that candidate, not a promise that CI still runs them. |
| `issues/DOC-CURRENCY-ANCHOR.md:124-170` | **historical closure record; left unchanged.** The byte-comparison gate and its CI controls really landed, then were retired by the later decoupling ruling. Its present-tense wording is scoped by the dated `CLOSED — 2026-07-22` heading. |
| `issues/SPEC-CLOSURE-BOUNDARY.md:126-146` | **dated publication evidence; left unchanged.** It records a candidate that the then-live publisher gate actually refused and the consumer omission that refusal exposed. |
| `issues/DOC-ASBUILT-LEDGER.md:14-29` | **release-point shorthand; left unchanged.** "Closes" and "installs" describe the terminal re-stamp at that release point. Lines 53-68 in the same node explicitly accept later drift and reject a perpetual-green reading. |
| `wp/DOC-ASBUILT-LEDGER.md:7-10` | **release-point shorthand; left unchanged.** "Turns the currency gate green" is the measured terminal candidate result. Lines 78-88 explicitly say a later red `--check` is expected and forbid a stays-green AC. |
| `issues/DOC-ATTEST-LIVING.md:174-180` | **retired assumption; left unchanged.** The claim that every publish polls the gate is below the file's lines 14-34 retirement notice, which says all following live premises are false after `LIB-GATE-DECOUPLE`. |

The object sweep also found these sibling families:

| Sites | Disposition |
|---|---|
| `issues/DOC-W0.md:121-134`, `issues/DOC-W1.md:14-26`, and `issues/DOC-CURRENCY-ANCHOR.md:124-170` | **dated design and closure evidence.** These explain why the content-currency mechanism was built and what it proved before decoupling. They do not override the later removal record. |
| `issues/DOC-W2.md:14-20`, `issues/ABI-S3.md:14-21`, and `issues/RT-VALUE-TOTALITY.md:171-182` | **dated merge evidence.** Each records a checker result observed for a named historical publication. The RT record also says publisher verification did not run and that its skipped clauses were discharged by hand. |
| `issues/SRC-ATTEST.md` and `wp/SRC-ATTEST-currency-substrate.md` | **historical mechanism design.** They define the attestation substrate and merge-result authorization that preceded the operator's later removal of per-merge coupling. The ledger format survives; the old publisher consumer does not. |
| `issues/DOC-ASBUILT-AUDIT.md:25-34`, the other `issues/DOC-ASBUILT-*` nodes, and the `wp/DOC-ASBUILT-*` frames | **release-point measurements and instructions.** They use `--check` locally to expose accumulated drift, forbid unreviewed or incremental re-stamping, and culminate in one reviewed terminal re-stamp. They do not claim a live CI consumer. |
| `issues/LIB-GATE-DECOUPLE.md:102-120`, `issues/STR-BIJ.md:35-46`, `issues/DOC-W6-AGENT-EVAL.md:50-55`, and `wp/DOC-W6-AGENT-EVAL.md:14-23` | **current controls; left unchanged.** They independently state that no currency check runs in publisher/CI and that release-point drift is accepted. |

The claim-shaped census therefore finds no unqualified current assertion that a
library-validation, drift, attestation, or registry gate executes in CI or the
publisher. The apparent survivors are all explicitly bounded above as dated
evidence, retired assumptions, or release-point operations. The three current
program-law sites remain the corrections in D1, D2, and RQ-3.

### Scope and release-point artifacts

Only `docs/program/` paths change. In particular,
`library/SOURCE-ATTESTATIONS`, `library/STATUS.md`, and `library/REVISION`
remain byte-untouched. Their accepted release-point drift is not repaired
piecemeal; the corrected generators will restamp the full population together
at the next release point.
