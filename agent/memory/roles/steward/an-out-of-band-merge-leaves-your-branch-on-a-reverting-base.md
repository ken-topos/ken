---
scope: roles/steward
audience: (see scope README) — anyone who publishes (the Steward), and any leader
  assembling a candidate after someone else moved `main`
source: 2026-07-26. The operator merged PR #1035 by hand, out of band. `steward/work`
  stayed on the pre-merge base, so the tracker update I was about to publish would
  have re-added the test file the operator had just deleted. Caught by a diff-scope
  step added an hour earlier for an unrelated reason.
---

# An out-of-band merge leaves your branch on a base that silently reverts it

⛔ **When `main` moves by a path your workflow does not observe — a hand merge, a
teammate's publish, an operator fixing something directly — your branch keeps its
old base, and nothing tells you. The next candidate you build from it can carry a
revert of that change inside a PR titled as something else entirely.**

## What happened

The operator merged #1035, which **deleted**
`crates/ken-elaborator/tests/kw_theorem_source_oracle.rs`. No publisher ran, so
there was no merge log, no `wp/scripted-merge-*` branch, and no mootup relay —
**every signal my pipeline watchdog keys on was absent.** `main` simply moved.

`steward/work` still sat on the pre-merge base. So:

```
git diff --name-only origin/main..HEAD
  docs/program/IMPLEMENTATION-PROGRESS.md
  crates/ken-elaborator/tests/kw_theorem_source_oracle.rs   <- the operator's deletion
```

⇒ Publishing would have **re-added the deleted file**, inside a PR whose title
said *tracker update*. Nobody reviewing that title would look for it.

## The one-command test

```sh
git merge-base --is-ancestor origin/main HEAD    # exit 1 ⇒ your base is stale
```

⇒ **After any merge you did not perform: `git reset --hard origin/main` before
building a candidate**, then re-cherry-pick. ⛔ Never rebase or force-move without
checking ancestry first.

## ⛔⛔ THE CONTROL I WROTE FOR THIS WAS AIMED AT THE WRONG OBJECT

I added a standing control — *"after every publish, re-probe that recently deleted
paths are still absent on `main`"* — because verifying my **own** blobs landed
cannot detect a revert of **someone else's** change. Those are different questions
and only the first was ever being asked.

⚠ **Then, the same day, the control fired FALSE and I nearly acted on it.**
Reviewing a candidate whose base predated the deletion:

```sh
git cat-file -e <candidate>:crates/ken-elaborator/tests/kw_theorem_source_oracle.rs
  ⇒ PRESENT      # looks like the candidate re-adds the operator's deletion
```

**It does not.** The file is present in the candidate's **tree** simply because the
base predates the deletion; it is absent from the candidate's **diff**, and this
repo squash-merges (merge-base → branch), so the merge never touches that path.

⭐ **The probe has to be aimed at the MERGE RESULT, not at your tree:**

```sh
MT=$(git merge-tree --write-tree origin/main <candidate>)
git cat-file -e "$MT:<recently-deleted-path>"   # exit 0 ⇒ REAL revert; exit 1 ⇒ safe
git rev-parse "$MT:<the-candidate's-own-file>"  # positive control: the change IS in the result
```

⇒ ⛔ **"Is it in my tree?" and "will it be in `main`?" are different questions, and
the tree-level probe answers the wrong one with a confident yes.** It returns a
false alarm on *every* candidate whose base predates *any* deletion — which is
most of them.

## How to apply

- **Before building any candidate:** `git merge-base --is-ancestor origin/main HEAD`.
  Non-zero ⇒ reset first.
- **Before passing `--doc-only`:** `git diff --name-only origin/main..HEAD` and read
  it. A `--doc-only` merge skips CI, so this is the only gate left.
- **Before publishing anyone's candidate:** compute the merge result with
  `git merge-tree --write-tree` and probe it for recently deleted paths — with a
  positive control that the candidate's own change *is* in that result. ⛔ A probe
  with no positive control cannot distinguish "safe" from "I measured nothing".
- ⚠ **Do not use `git diff origin/main <sha>` as a staleness detector** — it fires
  identically on safe and unsafe candidates. The discriminator is the
  **intersection** of the two change sets against the merge base; empty ⇒
  immaterial, do not rebase.

## Positioning

- ⭐ **Why the catch was luck, and what that argues for.** The diff-scope step that
  caught this was added an hour earlier to keep a `scripts/` change out of a
  doc-only merge. It caught a **worse, unrelated** hazard instead. ⇒ **A check
  placed at a COMMAND catches what its author did not have in mind; a check aimed
  at a known failure does not.** Prefer step-shaped rules at the point of work over
  advisory prose about being careful.
- [[stale-base-candidate-silently-reverts-everything-landed-since]] — ⛔ that title
  overstates it and `COORDINATION §14` carries the correction: a squash applies
  merge-base → branch, so merely *lacking* what `main` gained deletes nothing. The
  real hazard is the narrow one measured here.
- [[committed-is-not-reachable-publish-then-verify-on-main]] — the verify half.
- [[steward-work-is-stale-immediately-after-every-publish]] — the routine case; this
  is the case where nothing announced the move.
