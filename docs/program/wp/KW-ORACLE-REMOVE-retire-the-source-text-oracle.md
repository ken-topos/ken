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

**AC-4 — the deletion is causally verified, not assumed.** ⭐ Run the same
command before and after and report both.

| | `kw_theorem_source_oracle` target | `-p ken-elaborator` overall |
|---|---|---|
| **before** | **runs 7 tests**, of which **1 FAILS** | **RED** |
| **after** | **absent** | **green** |

⚠ **The failing test is `exact_candidate_has_no_unclassified_retired_occurrences`,
and its red is the WP's justification — not an obstacle to it.** Do not
investigate it, do not repair it, do not treat it as a precondition to clear.

⭐ **The only number that discriminates is `runs 7 tests`.** It distinguishes
*"I deleted a file of 7 live tests"* from *"I deleted a file whose tests were
already not running."* ⛔ **If the target does not run exactly 7, stop and say
so.** If the pass/fail split differs from 6/1, note it and **continue** — that
is a census drifting further, which is expected and is not your problem.

> ### ⛔ CORRECTED TWICE. Both corrections were mine, and they had the same shape.
>
> **First (count):** this AC said **12**. `language-implementer` blocked and was
> right — `grep -c '^#\[test\]'` returns **7**. I had counted `^fn `
> declarations, sweeping in helpers (`candidate_inputs`, `classify`,
> `occurrence_lines`, `retired_findings`, …).
>
> **Second (pass state):** the corrected AC said **"7 passed → absent"**. Also
> false: at `f52b0f61` the target is **6 passed / 1 failed**, because the landed
> catalog change (`95bc855c`) moved lines in
> `catalog/packages/Core/Classes/EffectfulClasses.ken.md` and
> `Derived.ken.md` — **both pinned by the oracle's frozen line-number
> allow-list** (`kw_theorem_source_oracle.rs:93`, `:113`). The implementer
> blocked a second time and was right a second time.
>
> ⭐ **The shape both times: I wrote a fixed input I had not measured at the base
> I named.** The first was a count I derived from the wrong grep; the second was
> a pass/fail state I never observed at all — I assumed green because the suite
> had been green when I read the file. ⇒ **An AC that names an observable must
> be read out of the artifact at the stated base, exactly like any other claim.**
>
> ⭐ **And the second block is worth more than the correction it forced:** it is
> the measurement that proved **`main` itself is red on this oracle**, which is
> why this WP is now the fleet's merge-pipeline blocker rather than a cleanup.
> ⛔ **Do not read two blocks as friction.** Both stopped work that would
> otherwise have reported success against a false claim.

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

- ⭐ **AC-4's `runs 7 tests` count is the discriminating control** — see AC-4 for
  why the pass/fail split is deliberately *not* part of it.
- ⛔ **`git diff --stat` always exits 0** — it is not an emptiness test. Use
  `--quiet` (or read `--name-only`) when you need to assert a diff is empty.

## Handoff

Return **one exact candidate SHA** with the branch freed, plus the `--name-only`
diff and the before/after test counts. ⛔ **No Decision is opened by the ring** —
that is the Steward's.

⚠ **When this merges, post the landed SHA to the Steward promptly** — the doc
ring is held on it with a complete product delta waiting to rebase.
