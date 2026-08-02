# WP frame — DOC-PROGRAM-SELF-REFUTE

Node: `docs/program/issues/DOC-PROGRAM-SELF-REFUTE.md`. Owner: doc ring
(`doc-leader` + `doc-author`, Librarian as QA). Size M.

## Fixed inputs, measured at `origin/main = 1d70335a`

These are measurements, not estimates. Re-measure only if you believe one is
wrong; do not re-derive them as a matter of course.

1. `docs/program/12-documentation-program.md:70-72` — D1's closing clause:
   *"and a drift gate verifies that section still exists."*
2. `docs/program/12-documentation-program.md:124-126` — *"Currency is a source
   revision, recorded by generated `STATUS.md` and build output, never
   hand-edited into pages."*
3. `docs/program/12-documentation-program.md:221-235` — the refutation, dated
   `Measured 2026-08-01 at f31e8d94`: `VALIDATION_GATES` appears exactly twice
   in `crates/ken-cli/tests/library_documentation_gates.rs`; all eleven gate
   functions occur exactly twice each (definition + registry row); no test
   iterates the registry.
4. `grep -rn 'gen-doc-status\|library/REVISION' .github/` returns **0**.
5. `docs/program/07-catalog-style-guide.md:127` — *"## 5. Findings — RETIRED
   from the catalog entry (2026-07-11)"*, with `:138` *"Do not add a Findings
   section to new entries"* and `:139` *"Existing Findings sections are
   removed"*.
6. `docs/program/06-catalog-campaign.md` — Findings appears at lines 78, 92,
   212, 219, 231, 239, 289, 301, 310, 351, 424.

## Judgments, front-loaded

These are settled. They are inputs, not questions to escalate.

**J1 — D1's decision stands; only its mechanism clause is false.** D1 says
`library/` is derived and must cite the exact spec section. That is confirmed
and load-bearing, and this WP does not touch it. The false part is the trailing
promise that a drift gate *verifies* the cited section still exists. Correct
the clause, keep the rule. State what is actually true: the citation
requirement is enforced by review, and no live runner checks it.

**J2 — this is a text repair. Re-arming is banned scope.** The registry was
retired by operator ruling (`LIB-GATE-DECOUPLE`, `f84e4804`), and lines 233-235
of the same program record that "restoring the gate" would mean building *new*
coupling, which is that ruling in reverse. ⇒ **Zero changes under `crates/`,
`.github/`, or `scripts/`.** A refusal worded as "the honest fix is to make the
gate real" is out of scope and routes back as a hard stop, not a repair.

**J3 — the Findings SECTION is retired; the Findings LOOP is not.** `07` §5
retires the section from the entry format and says the dogfooding instrument
continues through a live channel. So in `06`, a sentence about a Findings
*section inside a catalog entry* is stale, and a sentence about Findings
*routing, filing, or team ownership* may well be current. ⛔ **Do not delete
every occurrence of the word.** Classify each of the eleven sites against `07`
§5's actual text, and correct only what asserts the retired section.

**J4 — site B is half true.** Generated `STATUS.md` is real. Only the "build
output" channel is false. Correct the channel, keep the rule that currency is a
source revision and is never hand-edited.

**J5 — no new test oracles.** Operator policy: test oracles asserting facts
about source, catalog, or documentation lines invite failure and delay. The
controls below are measurements and reviews, not new tests. Do not add a test
that greps documentation.

## Deliverables

- **D1.** Site A corrected per J1.
- **D2.** Site B corrected per J4.
- **D3.** The eleven `06-catalog-campaign.md` Findings sites classified per J3
  — each recorded as *stale (corrected)* or *live (left unchanged)*, with the
  `07` §5 sentence each classification rests on.
- **D4.** A reconcile note in the WP's report recording, per site, what was
  corrected and what was deliberately left standing and why.

## Acceptance criteria

- **AC-1 — no surviving sentence in `docs/program/` asserts that a
  library-validation gate, drift gate, or registry runner executes.**
  *Control:* search by **claim shape** — present-tense assertions that a check
  runs, a property is enforced or guaranteed, validation executes, or a failure
  reaches the build — and report **every** hit with its disposition:
  corrected · dated evidence · superseded frame assumption.
  ⛔ **Do not search by a list of gate verbs.** That exact method failed on
  2026-08-02: a verb enumeration matched nothing and the empty result was read
  as "no sites remain" rather than "the pattern did not match." Reporting the
  hits you classify as *not a defect* is what makes an empty defect list
  distinguishable from a non-matching search.

- **AC-2 — D1's decision is unchanged in force.** *Control:* quote D1's
  decision before and after. The derived-not-normative rule and the
  cite-the-exact-section requirement must be unchanged in meaning; only the
  mechanism clause differs.

- **AC-3 — the diff touches `docs/program/` and nothing else.** *Control:*
  `git diff --name-only <base>..HEAD` lists only `docs/program/` paths. Use
  `--quiet` for any emptiness test; `--stat` always exits 0.

- **AC-4 — the Findings classification does real work.** *Control:* at least
  one of the eleven sites is classified **live and left byte-unchanged**, with
  its `07` §5 grounding quoted. ⚠ If all eleven are corrected or all eleven are
  deleted, the classification is vacuous and AC-4 fails — J3 exists precisely
  because "Findings" names two different things in this corpus.

## Contention check

Doc ring edits `docs/program/`. The runtime ring's in-flight `RT-WORKER-BIND`
candidate touches six `crates/ken-runtime` paths. Disjoint — no shared path,
no shared build turn. The doc track's standing concurrency exception (operator,
2026-07-21) applies.

## What this WP does not do

- It does not re-arm the registry or add CI coupling (J2).
- It does not edit `library/` — `DOC-AGENT-CITE` already corrected the
  `library/` half at `428ee50f`, and `library/STATUS.md`, `SOURCE-ATTESTATIONS`
  and `REVISION` are frozen release-point artifacts that lag by design.
- It does not touch the six files D5 classified as dated evidence, removal
  records, or superseded frame assumptions. Those are correct as history.
