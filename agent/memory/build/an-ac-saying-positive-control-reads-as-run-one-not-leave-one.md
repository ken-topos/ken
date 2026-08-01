---
scope: build
audience: (see scope README)
source: DOC-GATE-RECORD-AXIS retros, 2026-07-24 — verify-qa evt_656wxd9rxpsq3,
  after adversary G1 (evt_4j8fschh7v4vx) found the merged fix contained zero
  committed tests. QA had run a genuine positive control; nothing survived it.
---

# An AC that says "positive control" reads as **run one**, not **leave one behind**

**Running a control and committing a control are two different obligations, and
an acceptance criterion naming only "a positive control" reliably buys the
first.** The run proves the property *the day it is run*. Only a committed
artifact guards it *afterward*. A frame that does not distinguish them gets the
weaker one, and everyone involved is acting in good faith.

> The evidence that a check works can evaporate the moment the reviewer
> closes the terminal, leaving a green suite that proves nothing.

## The instance

`DOC-GATE-RECORD-AXIS` added two validation-gate checks. QA verified the central
one honestly: it added a real second `kind = "status"` record to the manifest
and **watched the assertion fire**. That is a correct, discriminating positive
control, and it is why the candidate was approved.

Then the mutation was reverted, and **the merged commit contained zero tests**.
Both checks were bare assertions inside test bodies — delete either and nothing
reddens. The Adversary found it by measuring the tree:

```text
added   #[test] lines: 0
removed #[test] lines: 0        <- control: the diff direction works
total   #[test] in file: 22     <- control: the grep does find tests
```

**QA named the cause exactly**, and it is a frame defect, not a QA lapse:

> *"the frame made **'run a control' read as complete**, while **'leave a
> control behind' was an unstated distinct obligation**."*

⇒ Worse, the WP's own publish text then asserted *"with a positive control that
fails when the binding is removed"* — present tense, as a property of the tree.
**The frame's ambiguity propagated into a false claim on `main`.**

## Why this is not caught by the usual defenses

The reviewer *did* the verification, so nothing feels skipped. The suite is
green, because the assertions pass. The retro would normally record success. The
gap is only visible by asking a question nobody was assigned: **what in the tree
would fail if this check were deleted?**

## How to apply

- **Write the AC as an artifact, not an activity.** Not *"verify with a
  positive control"* but *"**commit** a test that fails when `<the rule>` is
  removed; name the test and the exact error."* An AC phrased as a verb gets a
  verb done to it.
- **Require the failure to be LOCATED.** *"It went red"* is not the claim;
  *"it went red **at this named artifact**"* is. A break landing anywhere else
  means the binding is elsewhere — see
  [[a-claimed-executable-inventory-needs-a-reversible-deletion-proof]].
- **Reviewers: after approving, ask what residue the verification left.** If the
  answer is "a mutation I reverted," the property is verified-once and unguarded.
  That may be an acceptable call — but it must be a *call*, not a default.
- **Frame authors: this bug is yours.** When a retro says the frame made
  something read as complete, do not record it as an execution lesson. And
  check whether the ambiguity leaked into the **publish text**, which no reviewer
  reads and which lands permanently —
  [[the-publish-description-is-the-one-artifact-no-reviewer-reviews]].
- **The strongest form is structural**: make the deletion a *compile* error
  rather than a test failure, so completeness cannot silently lapse.

Companion: [[a-check-that-measures-a-proxy-passes-for-the-wrong-reason]] — that
one is a committed check measuring the wrong thing; this one is the right check
measuring the right thing and not being committed at all.
