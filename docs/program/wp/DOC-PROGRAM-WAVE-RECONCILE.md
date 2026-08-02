# WP frame — DOC-PROGRAM-WAVE-RECONCILE

Node: `docs/program/issues/DOC-PROGRAM-WAVE-RECONCILE.md`. Owner: doc ring
(`doc-leader` + `doc-author`, Librarian as QA). Size M.

## Fixed inputs, measured at `origin/main = 70441007`

These are measurements, not estimates. Re-measure only if you believe one is
wrong; do not re-derive them as a matter of course.

1. `12-documentation-program.md:3-6` — *"Waves 0, 1 and 2 are LANDED —
   `library/` carries 26 documents … Wave 3 and beyond remain map only."*
2. `library/` holds **89** markdown documents and `library/manifest.toml`
   holds **89** entries. By subtree: `learn/` 10, `guide/` 4, `how-to/` 5,
   `reference/` 54, `agents/` 12, top level 4.
3. §4's wave table gives waves **3, 4, 5 and 6** the state *"map only — §4b"*.
4. §4b's headings read: Wave 3 *"(MAP · fence precondition RECONCILED)"*,
   Wave 4 *"(MAP)"*, Wave 5 *"(MAP · re-check D4 FIRST)"*, Wave 6 *"(MAP)"*.
5. Merged nodes by wave — Wave 3: `DOC-W3-GUIDE`, `DOC-W3-HOWTO`,
   `DOC-W3-DEPDATA`. Wave 4: `DOC-W4-TOOLCHAIN`, `DOC-W4-LANGUAGE`,
   `DOC-W4-RESIDUAL`. Wave 5: `DOC-W5-CAPABILITY`, `DOC-W5A-CARD-FORMAT`,
   `DOC-W5B-CARDS-APP-DATA`, `DOC-W5C-CARDS-CAPABILITY`, `DOC-W5D-INDEXES`.
   Wave 6: `DOC-W6-AGENT-EVAL`. Twelve in total, all `merged`.
6. Three dated Steward measurement blocks sit **inside** those same sections
   and are current: Wave 3's seven-subject block at `c777d2d4` (2026-08-01),
   Wave 4's generation-precondition block at `7fa65b20` (2026-08-01), Wave 6's
   four-item gating table at `5a0fd8e6` (2026-08-02).
7. Every doc-owned tracker node is now `merged` or `closed`. There is no
   `draft` or `ready` doc node behind this one.

## Judgments, front-loaded

These are settled. They are inputs, not questions to escalate.

**J1 — the bodies are right and the headers are wrong.** Each dated block in
input 6 was measured and is current. The defect is that the status line, the
table row, and the heading a reader meets *first* still assert the state those
blocks superseded. ⇒ **Repair the headers to match the bodies.** Do not
re-measure the bodies, and do not "reconcile" by weakening a block.

**J2 — "map only" and "landed" are not the only two dispositions.** A wave
whose nodes all merged can still owe Produces items, and a wave can owe items
that are *deferred behind a named gate* rather than pending. Dispose each wave
by walking its **Produces** list item by item. ⛔ **Do not infer a wave's state
from its node count** — twelve merged nodes is what prompted this WP, not what
settles it.

**J3 — Wave 6's gating table is current and authoritative.** It was measured
2026-08-02 and disposes all four Produces items: three deferred or foreclosed,
one (agent-pack evaluation) landed as `DOC-W6-AGENT-EVAL`. Only Wave 6's
**table row** under-describes this. ⛔ Do not re-derive Wave 6's gates; quote
that block and move on.

**J4 — deferred is a real disposition and it must name its gate.** Wave 1b
(needs one complete real catalog program to read), the `ffi-and-platform` task
module (PX8), and Wave 6's three foreclosed items each already carry a gate in
the text. Carry the gate with the disposition; *"deferred"* alone is what lets
an item silently become backlog nobody owns.

**J5 — this is a text repair, and the banned scope is inherited verbatim from
`DOC-PROGRAM-SELF-REFUTE` J2/J5.** Zero changes under `crates/`, `.github/`,
`scripts/`, or `library/`. No re-armed registry, no CI coupling, and **no new
test oracle that greps documentation** — operator policy is that oracles
asserting facts about source, catalog or documentation lines invite failure and
delay. The controls below are measurements and reviews.

**J6 — do not write any missing page here.** If `D4` finds a Produces item that
is releasable today, it is recorded as releasable and framed as its own node.
Widening this WP into authoring is how a reconciliation becomes a wave.

## Deliverables

- **D1.** The status line at `:3-6` corrected: the measured document count, and
  a per-wave state that matches §4b.
- **D2.** §4's wave table rows for waves 3, 4, 5 and 6 given dispositions
  grounded in each wave's Produces list, not in its node count (J2).
- **D3.** §4b's headings for waves 3, 4, 5 and 6 relabeled to agree with the
  dated blocks already inside those sections (J1).
- **D4.** A **residual register**: every Produces item across waves 3-6,
  each disposed as one of
  **landed** (naming a `library/` path or a merged node id) ·
  **releasable now** · **deferred** (naming its gate, per J4).

## Acceptance criteria

- **AC-1 — no heading, table row, or status line in
  `12-documentation-program.md` contradicts a dated measurement block in the
  same file.** *Control:* for each of waves 3-6, quote three things side by
  side — the §4b heading, the §4 table row, and the newest dated block in that
  section — and show they agree. ⚠ Report all four waves including any you
  change nothing on; a wave omitted from the table is indistinguishable from a
  wave you did not check.

- **AC-2 — D4's register is exhaustive over the Produces lists.** *Control:*
  count the Produces items in each of waves 3-6 from the section text, state
  the count, and show the register has the same number of rows for that wave.
  A row disposed **landed** names a `library/` path or a merged node id; a row
  disposed **deferred** names its gate.

- **AC-3 — the diff touches `docs/program/` and nothing else.** *Control:*
  `git diff --name-only <base>..HEAD` lists only `docs/program/` paths. Use
  `--quiet` for any emptiness test; `--stat` always exits 0.

- **AC-4 — the register answers the question that motivated the WP: what, if
  anything, can the doc ring be released next?** *Control:* the register either
  names at least one **releasable now** item, or states that waves 3-6 have
  none and grounds that in the per-item dispositions. ⚠ **Both outcomes pass;
  silence fails.** This is the Steward's next framing input, and an empty
  answer that is *stated* is usable where an omitted one is not.

- **AC-5 — no dated measurement block loses or changes a claim.** *Control:*
  list the three blocks in input 6 and state for each whether its claims are
  unchanged. Reflowing to 80 columns is not a change; deleting a sentence,
  softening a disposition, or re-dating a block is. ⛔ If a block looks wrong,
  that is a hard stop and a new measurement, not an edit under this WP.

## Contention check

The doc ring edits `docs/program/`. The runtime ring's in-flight
`RT-CONTSPEC-ACTIVATE` touches `crates/ken-runtime` plus, at most, its own
`docs/program/wp/RT-CONTSPEC-ACTIVATE.md` and
`docs/program/issues/RT-CONTSPEC-ACTIVATE.md`. ⚠ `docs/program/` is therefore a
**shared directory but not a shared file** — no path this WP touches is one the
runtime ring touches. Verify the intersection at candidate time rather than
assuming it; the doc track's standing concurrency exception (operator,
2026-07-21) is contention-free-ness, not priority.
