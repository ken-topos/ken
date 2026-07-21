# Q2b — triage the review queue

Track Q, item Q2, of
[`../11-test-suite-and-ci-remediation.md`](../11-test-suite-and-ci-remediation.md).

**Your queue is `Q2-<your-team>.md` in this directory.** Read this file first.

## What this is

`scripts/qa-risk-scan.py` (Q2a) scanned all 1909 tests for the risk patterns
in [`research/qa-conformance-to-rust-test-guidelines.md`](../../../research/qa-conformance-to-rust-test-guidelines.md)
§5 and flagged **428**. Your queue is a balanced slice of those.

> ### ⛔ THE SCAN FOUND QUESTIONS, NOT DEFECTS
>
> This is the advisory's own framing and it is load-bearing: **its scans are
> review queues, not defect counts.** Expect a large fraction of your queue
> to be **correct as it stands** — a count literal that genuinely *is* the
> contract, an `is_err()` whose variant honestly does not matter, a timing
> assert already living in a real perf lane.
>
> **Do not treat queue length as a defect count, and do not "fix" a test to
> clear it from the list.** A test you reclassify as a sound durable
> invariant is a *successful* triage outcome, not a miss.
>
> Equally: **a clean scan does not mean a test is well-formed.** These are
> syntactic smells only. Reachability and oracle-independence are invisible
> to source-text scanning — a test can be perfectly shaped and still assert
> nothing that matters.

## What you produce

**Output is a list, not edits. Do NOT change any test in this pass.**

Q2 is triage. Rework is Q3–Q7 and is not authorized yet. Editing now means
reworking tests before we know the distribution, which is the thing this
pass exists to measure.

Write your findings to `Q2-<your-team>-result.md` in this directory, one row
per test:

| test | class | confidence | note |
|---|---|---|---|

- **`class`** — one of `durable-invariant`, `compat-vector`,
  `transition-sentinel`, or `UNCLASSIFIABLE`.
- **`confidence`** — `high` or `low`. **`low` is a real and useful answer**;
  it routes the test to a second reader rather than burying a guess.
- **`note`** — one line. For `UNCLASSIFIABLE`, say *what you would need to
  know*. For a reclassification, say what the test actually promises.

## How to classify

The three classes and the discriminating question are in your **own
playbook** — `agent/playbooks/build/qa.md` (QA) and
`agent/playbooks/build/implementer.md` step 4 (implementer). Read that
first; it is the authority, and this file will not restate it.

The one question that settles most rows:

> **Which intended extensions keep this test green, and which turn it red?**

If both answers are "any change at all," it is a snapshot — not an
invariant. Label it `transition-sentinel` or mark it `UNCLASSIFIABLE`.

The full ten-step workflow is advisory §6; the Rust patterns are §7.

## Scope discipline

- **Stay in your queue.** It is balanced by test count, so it deliberately
  **does not follow crate ownership** — you will see files your team does
  not own. That is intended: this pass is procedural, not subject-area
  (operator, 2026-07-21). Classify what you are given.
- **If a test genuinely needs subject knowledge you lack, mark it
  `UNCLASSIFIABLE` with `low` confidence and say so in the note.** That is
  the correct outcome, not a failure — it routes the test to its owner.
  Guessing to fill the row is the only wrong answer.
- **Do not run the workspace.** You should not need to build at all; this is
  a reading pass. If you do build, `scripts/ken-cargo -p <crate>` only —
  **never `--workspace`** (`agent/COORDINATION.md` §12).

## When you are done

Commit your result file on your own branch and report to the Steward with
your counts per class. **Do not open a PR** — the Steward collects all six.
