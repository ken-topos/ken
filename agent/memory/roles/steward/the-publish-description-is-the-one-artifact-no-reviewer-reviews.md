---
scope: roles/steward
audience: (see scope README)
source: adversary G1 on DOC-GATE-RECORD-AXIS (64b0811f), thr_2seh2bm1kr5mh
  evt_4j8fschh7v4vx, 2026-07-24 — the Steward's own PR description claimed a
  positive control the commit does not contain, and it landed permanently in
  the git log as the squash commit message.
---

# The publish description is the **one artifact no reviewer reviews**

*…and it lands on `main` forever, as the squash commit message.*

Every other artifact in a work package has a reader positioned to catch it. QA
reads the code. The Architect reads the mechanism. The Steward reads the
candidate. **The publish description has nobody**, because the Steward authors
it *after the last gate has closed*, and the scripted publisher turns it into
the **squash commit message on `main`** — the most durable prose the project
produces.

> ⛔ It is written at the exact moment when everyone has stopped looking, by the
> person whose job that moment was to end.

## The instance

`DOC-GATE-RECORD-AXIS` closed two adversary findings. The Steward's PR
description said:

> *"This binds coverage on the record axis, **with a positive control that fails
> when the binding is removed**."*

Measured against the tree, with probe controls:

```text
added   #[test] lines: 0
removed #[test] lines: 0        <- control: the diff direction works
total   #[test] in file: 22     <- control: the grep does find tests
```

**Zero tests.** Both new checks were bare assertions inside test bodies, each
deletable without anything reddening. And the second half of the sentence was
false too: the mechanism pins **one instance** through one hard-coded path
literal; it does not bind an axis.

★ **The claim was specifically the property the parent WP existed to enforce.**
`DOC-VALIDATION-BINDING` was written to kill *"deleting a gate leaves its token
orphaned while the suite stays green."* The Steward asserted that cure in the
sentence describing the change that lacked it. **The remedy for the orphaning
defect was itself orphanable, and the prose covered the gap.**

## Why the usual defenses do not fire here

- It is **not** caught by review: it is authored after review ends.
- It is **not** caught by CI: prose is not compiled.
- It is **not** caught by the retro: the Steward writes the retro questions too,
  and here the Steward asked QA *"did the positive control look redundant?"* —
  **building a retro question on the same false premise**, which would have had
  QA reconstruct a memory of something that never happened.
- It is **not** caught by the author rereading: the sentence describes the
  change as *intended*, and intent is what the author has in mind.

## How to apply

- ⭐ **Before publishing, verify the description against the tree the way a
  reviewer would — every verification-flavored claim in it is a check you are
  asserting exists.** Grep the diff for the artifact each clause names. "With a
  positive control" ⇒ `git diff <base> <sha> | grep -c '^+.*#\[test\]'`. A claim
  about a test, a proof, a control, or a gate is a **factual claim about the
  diff**, not a summary of it.
- **Prefer what the mechanism PINS to what it EVOKES.** "Pins the one
  status-kind record the hard-coded runner can serve" is true and leaves the
  next reader looking. "Binds coverage on the record axis" is reassuring and
  stops them. The reassuring sentence is the defect — same disease as
  [[dont-publish-derived-code-measurements-state-the-property-let-the-ring-measure]].
- **The fix for a landed overclaim is to make the claim TRUE, not to correct the
  prose.** The commit message cannot be amended once squashed onto `main`. File
  the follow-on WP (here: `DOC-GATE-CONTROL-BINDING`).
- ⚠ **Then re-attack your own correction.** A fix that adds a claim adds a fresh
  overclaim — see
  [[mutation-proof-injection-point-is-a-reachability-tell]]. Write the new WP's
  publish text under the same rule.

## The generalization worth carrying

**A fix to a finding is the highest-yield target in the federation** — same
author, same topic, same reassuring register, and everyone is relieved that the
problem is handled. That is the Adversary's stated reason for hunting this one,
and it paid. Apply it to your own corrections first: the moment you have just
closed something, your prose about it is at its least trustworthy and least
scrutinized.

Related: [[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]],
[[audit-a-detector-against-the-case-whose-answer-you-already-know]],
[[a-claimed-executable-inventory-needs-a-reversible-deletion-proof]].
