---
name: skill-style
description: >-
  Author, edit, review, and retire entries in the agent/playbooks corpus. Sets
  what belongs in a skill, the structure budget, how to withdraw a rule without
  leaving debris, and the deterministic plus adversarial review to run before
  committing a change.
metadata:
  scope: tools
---

# Skill Style

Apply this to any file under `agent/playbooks/`. Use it when creating a skill,
editing one, or reviewing a change to one. It governs shape and upkeep. What
each seat does is still governed by that seat's playbook, federation law by
`agent/COORDINATION.md`, and the tier-to-model mapping by `agent/MODELS.md`.

A skill is read by an agent that is mid-task and about to act. Write for that
reader: the one who needs the next step, not one studying the domain. The test
before committing is the operator's, from `steward.md`: would a competent
colleague in a hurry get the same decision from half the words?

## What belongs in a skill

Seven kinds of content. A paragraph that is none of them should be cut.

1. **Judgement guidance.** Short rules for decisions the reader must make.
2. **Checklists.** Ordered, each item an action with an observable result.
3. **Mechanical procedure.** Exact commands, flags, paths, sequences.
4. **Facts stated nowhere else.** Ids, thresholds, filenames, conventions.
5. **Rationale, one clause.** Only where it changes how the rule is applied.
6. **Negative scope.** What this role does not own, and where that work goes.
7. **Failure-mode triggers.** "You will be tempted to skip this when X."

Rationale earns its place by naming the tell, not by retelling the incident.
"An idle build team is always your backlog" is worth its line. Ten lines
re-establishing that the rule was once broken are not.

Failure-mode triggers are the cheapest high-value lines in the corpus, because
they fire at the moment the reader is about to skip the rule. Keep them, and
add one whenever a discipline is skipped twice.

Negative scope is the cheapest way to keep a skill short, and it is the
category most often missing. Say what the seat does not own.

## What does not belong

- **Law restated from `COORDINATION.md`.** Cite the section instead. Applying
  law to a specific act is category 3 and belongs; repeating the law does not.
- **The same rule in two places.** State it once and reference it. Two copies
  drift, and the reader cannot tell which is current.
- **Withdrawn rules left in the body.** See Retiring a rule, below.
- **Rationale for a rule nobody disputes.**
- **Decorative icons.** Operator, 2026-08-01: plain text. Symbols that carry
  information stay: arrows in a derivation, Ken notation, terminal glyphs
  quoted as data. If deleting it would lose information, it is not decoration.

The substitutions that shorten a skill fastest:

| stop | start |
|---|---|
| three framings of one finding | one framing |
| a corpus-lesson citation on every point | a citation where it changes the next action |
| recounting how an error survived | the corrected text |
| an offer plus rationale plus alternatives | the recommendation and one line of why |
| a rule plus its history | the rule, and the tell that it is being broken |

What is not being relaxed: verify objects before naming them, read a decision
from the object, edit operative text rather than appending a correction, and
say plainly when something is unverified. Those are cheap and load-bearing.
The waste is in the commentary around them. Terse and verified, not terse and
guessed.

## Structure budget

Derived from the corpus as it stands, not invented. Excluding `steward.md`, the
playbooks run 47 to 597 lines with a median near 320.

- **Total under 400 lines.** Past that, split by task rather than trim by
  sentence.
- **Largest section under 20% of the file.** A section that dominates is the
  one the reader needs most and can navigate least.
- **At least three headings per 100 lines.** Long runs of unbroken prose are
  unnavigable regardless of quality.
- **One entry point.** A reader must not have to hold a priority ordering over
  sections. If a section outranks the rest, it is the only thing at the top.
- **Section identifiers unique and stable.** Two sections sharing a number
  makes every cross-reference to it unresolvable.

A budget that is exceeded is a signal to split, not a rule to argue with. Say
in the file why, if the answer is genuine.

## Retiring a rule

Withdrawing content is where skills accumulate debris, and it is the part no
other document covers.

- **A withdrawn rule leaves the body entirely.** Never leave it as a struck-out
  numbered step inside a live checklist: it costs a read every time, and a
  hurried reader may follow it.
- **Record the withdrawal once, in a Changelog at the end**, with the date and
  the authority. That preserves the audit trail without taxing the reader.
- **Renumbering breaks references.** Before changing a section identifier,
  `grep -rn '<identifier>' agent/` and fix the citations in the same commit.
- **Edit operative text rather than appending a correction.** A later note
  saying an earlier passage is wrong leaves the wrong passage operative, and
  the wrong passage is the one that gets read first.

## Measure before you commit

Run these against the file you changed. They are deterministic, they take
seconds, and they answer the questions a reviewer should not have to spend
judgement on. They are an authoring aid, not a gate: do not wire them into CI,
which would make them a test asserting facts about documentation lines.

```sh
f=agent/playbooks/<path>.md
wc -l "$f"
# largest section and its size
awk '/^#{2,3} /{if(n)print l" "n; n=$0; l=0; next}{l++}END{print l" "n}' "$f" \
  | sort -rn | head -3
# headings per 100 lines
echo "$(grep -c '^#\+ ' "$f") headings / $(wc -l < "$f") lines"
# duplicate section identifiers
grep -oE '^#+ (§?[0-9]+[a-z-]*)\.' "$f" | sed 's/^#* //' | sort | uniq -d
# withdrawn items still in the body
grep -nE '~~|WITHDRAWN|SUPERSEDED|RETIRED' "$f"
```

## Adversarial review

For anything beyond a typo fix, dispatch a **T2** subagent to review the skill
adversarially. Resolve T2 through `agent/MODELS.md`; never name a model in this
file or in the dispatch prompt.

Give the subagent the file, the numbers from the previous section, and this
rubric. Do not ask it to count anything: counting is the deterministic half and
models are unreliable at it.

1. Which paragraphs match none of the seven content kinds? Quote the first line
   of each.
2. Which instructions are not actionable, or have no observable result?
3. Which content is duplicated within the file, or restates
   `COORDINATION.md`? Quote both locations.
4. Where does rationale exceed one clause without changing how the rule is
   applied?
5. What would a reader do first, and can they tell from the top of the file?
6. Which withdrawn or superseded content is still in the body?

Require of the output: findings ranked by severity, a hard cap of ten, and an
explicit list of rubric items checked and found clean. The reviewer is
advisory and has no write access; it returns findings and the author decides.

The cap and the clean list exist because an adversarially prompted reviewer
will return findings against a good document. Without them the review produces
churn: edits that satisfy a reviewer rather than a reader, which is how a
short playbook becomes a long one.

## Calibrate the rubric before trusting it

A review that returns findings is not evidence the skill is bad, and one that
returns none is not evidence it is good. Before relying on a change to the
rubric, run it against a skill whose defects are already known and confirm it
recovers them. If it cannot, the rubric is decorative however well it reads.

Keep at least one known-bad case on hand for this. Adding a defect to a scratch
copy of a healthy skill, and confirming the rubric names it, is enough.

## Changelog

- 2026-08-02: created.
