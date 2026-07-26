# WP `KW-ORACLE-REMOVE` — retire the whole-tree source-text oracle

**Node:** `docs/program/issues/KW-ORACLE-REMOVE.md` · **Owner:** language ·
**Size:** S · **Gate:** none · **Blocks:** `DOC-CATALOG-CONTENTS`

> ## ▶ ONE DELETION. THE ANALYSIS IS DONE — DO NOT REDO IT.
>
> Delete `crates/ken-elaborator/tests/kw_theorem_source_oracle.rs`. That is the
> whole change. **Read the node first** — it carries the measurements that make
> this safe, and they are the reason this is an `S` rather than a design task.

## Objective

Remove a test whose subject is the *text of the repository* rather than the
behaviour of a program, per the operator's ruling of 2026-07-26 and the
now-standing prohibition in `agent/playbooks/build/qa.md` (review gate) and
`agent/playbooks/build/implementer.md` (authoring gate).

## Scope

**In:** delete `crates/ken-elaborator/tests/kw_theorem_source_oracle.rs` in full.

**Out — ⛔ hard stops, route to Steward rather than deciding:**
- ⛔ **No `src/` change of any kind.** If you believe production code must move,
  stop and route it. (It does not: no production code references this test.)
- ⛔ **No replacement checker, lint, or "narrowed" oracle.** Not in this WP, not
  as a follow-on you add yourself.
- ⛔ **No edits to `catalog/`, `docs/`, `library/`, `spec/`, or `agent/`.** The
  doc-side work belongs to `DOC-CATALOG-CONTENTS`, which is held and will resume
  after this lands.
- ⛔ **Do not salvage helpers into another test file.** Every helper serves the
  prohibited oracle.

## Acceptance criteria

**AC-1 — the file is gone.** `crates/ken-elaborator/tests/kw_theorem_source_oracle.rs`
does not exist at the candidate SHA.

**AC-2 — nothing else changed.** The candidate's diff against its base touches
**exactly one path**, and that path is the deleted file. ⭐ State the diff as
`--name-only` output, not a summary — *"only the test was removed"* is a claim,
the file list is evidence.

**AC-3 — the crate still builds and its suite is green.** Targeted only:
```sh
scripts/ken-cargo test -p ken-elaborator
```
⛔ Never `--workspace`; workspace-green means **green in CI** (`COORDINATION §12`).

**AC-4 — the deletion is causally verified, not assumed.** ⭐ Report the test
count **before and after** from the same command. The file holds **7** `#[test]`
functions, so the `kw_theorem_source_oracle` target should go from **7 passed**
to **absent**. ⚠ **If the delta is not 7, say so and stop** — that means the
file's tests were not all running, or something else changed, and either way
your AC-2 claim needs re-examining.

> ⛔ **CORRECTED 2026-07-26 — this AC originally said 12, and 12 was wrong.**
> `language-implementer` blocked on it and was right: `grep -c '^#\[test\]'`
> returns **7**. I had counted `^fn ` declarations, which includes helpers
> (`candidate_inputs`, `classify`, `occurrence_lines`, `retired_findings`, …).
> ⭐ **The block is the AC working as intended** — a fixed input stated as a
> number is checkable, so a wrong one gets caught instead of silently reshaping
> the work. The "what goes" list in the node names the 7 correctly; only this
> count was wrong.

**AC-5 — no production reference is left dangling.** Confirm no `src/` or other
test file referenced the deleted module. The build succeeding is *evidence* for
this, not proof of intent — state which you are relying on.

**AC-6 — state the residual explicitly in the handoff.** After this lands the
retired token may appear in prose anywhere, unchecked. That is intended. ⛔ Do
not report it as a defect and do not propose a fix.

## ⚠ The one trap in this WP, and it is a real one

⛔ **Do not write the retired token into any file you touch.**

Until your deletion lands, the oracle is **still live in CI**, and it scans the
whole committed tree. A markdown file **containing a Ken fence** is in its
population and is then scanned **in full, prose included** — so a commit message
body is fine, but a new or edited `.md` carrying both a Ken fence and the token
in prose would red CI on its own.

⭐ For this WP the exposure is essentially nil (you touch one `.rs` file and
delete it), but it is the reason to **write your handoff in the channel, not into
a repo file**, and to refer to *"the retired token"* if you must mention it.

⚠ This trap **disappears the moment your candidate merges** — that is the point
of the WP.

## Evidence bar

This is a deletion, so the usual "does the pin discriminate?" questions do not
apply. What does apply:

- ⭐ **AC-4's count delta is the discriminating control.** It is the one cheap
  check that distinguishes *"I deleted a file of 12 live tests"* from *"I
  deleted a file whose tests were already not running,"* and those are different
  facts about what the repo had.
- ⛔ **`git diff --stat` always exits 0** — it is not an emptiness test. Use
  `--quiet` (or read `--name-only`) when you need to assert a diff is empty.

## Handoff

Return **one exact candidate SHA** with the branch freed, plus the `--name-only`
diff and the before/after test counts. ⛔ **No Decision is opened by the ring** —
that is the Steward's.

⚠ **When this merges, post the landed SHA to the Steward promptly** — the doc
ring is held on it with a complete product delta waiting to rebase.
