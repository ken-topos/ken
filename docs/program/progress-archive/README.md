# Progress archive — index

This directory is the chronicle half of `docs/program/IMPLEMENTATION-PROGRESS.md`.
That file was split on 2026-07-21: the live file now carries **current state
only** (what a cold resume needs), and every prior "live state" snapshot —
the full build history — moved here, split into time-based segments so no
single file is unmanageable.

**No content was dropped in the split**, with one narrow, explicitly
authorized exception: the ~13 repeated "prior live state, kept for history"
scaffolding headers (pure boilerplate marking where one snapshot was
demoted beneath the next) were collapsed to a plain `---` divider. A few
lines that had grown to 10,000-56,000 characters (headers that accumulated
nested parentheticals across dozens of edits — see `2026-07-19.md`'s note)
were mechanically reflowed into wrapped prose / bullet lists at their own
existing sentence boundaries; no words were changed, added, or removed.

Segments are listed newest-first, matching the order they appear when
walking from the live file backward through time. Read them in this order
to reconstruct the project history session-by-session.

| Segment | Date range | Size | Covers |
|---|---|---|---|
| [`2026-07-20-to-07-21.md`](2026-07-20-to-07-21.md) | 2026-07-20 to 07-21 | 262 KB | RT-PARITY remediation through merge; SPAN-SEAL defect+fix; STR-BIJ framed; RT-ESCAPE filed; F1 reclassified soundness-relevant; a provider content-refusal incident; the MODELS.md tier-table correction; tail of the PX8-series close (PX8-J-ERR). |
| [`2026-07-19.md`](2026-07-19.md) | 2026-07-19 | 157 KB | PX8-TA oriented-subcontinuation redesign (hard-stops 1-3); PX8-DS; start of PX8-F; the PX8-H/PX8-I heterogeneous-continuation landing saga (17 Architect hard-stops, reflowed from one 56,461-character line — see the file's editorial note). |
| [`2026-07-16-to-07-18.md`](2026-07-16-to-07-18.md) | 2026-07-16 to 07-18 | 220 KB | PX8-H/PX8-I/PX8-P/PX8-V/PX8-R/PX8-T/PX8-F chain; close of the native capability-model campaign (PX13->PX16). |
| [`2026-07-14-to-07-15.md`](2026-07-14-to-07-15.md) | 2026-07-14 (late) to 07-15 | 136 KB | CAT-TAX closure; PX0-PX6 POSIX/Linux ABI campaign kickoff; LET chain completion; kenfmt; axiom-keyword fossil sweep; PX1/PX2 close. |
| [`2026-07-14-part1.md`](2026-07-14-part1.md) | 2026-07-14 (later hours) | 179 KB | LET chain completion; AX-1/AX-2; KTR-1/KTR-2; resource-management design rulings; POSIX/Linux ABI campaign framing. Continues in part 2. |
| [`2026-07-14-part2.md`](2026-07-14-part2.md) | 2026-07-14 (earlier hours) | 245 KB | Milestone C (CC5-CC7); Program I (I-4/I-5); ADR-0017; the SEC2 honesty erratum; ends at the transition into 07-12. |
| [`2026-07-09-to-07-13.md`](2026-07-09-to-07-13.md) | 2026-07-09 to 07-13 | 102 KB | ADR 0014/0015; the F3b fail-closed design arc; package-abstraction ruling; proof-vocabulary work; the DS-1 operator-directed hold. |
| [`2026-07-06-to-07-08.md`](2026-07-06-to-07-08.md) | 2026-07-06 to 07-08 | 309 KB | SURF DEF / SURF NAMED PROOF; the NC1 through NC27 compiler-campaign build-out. |
| [`2026-07-03-to-07-06.md`](2026-07-03-to-07-06.md) | 2026-07-03 to 07-06 | 222 KB | ken-md-literate; the Codex fleet cutover; CAT-1->CAT-4 catalog build campaign; effect-composition; the VAL2 Rosetta corpus / filesystem workstream. |
| [`2026-07-03.md`](2026-07-03.md) | 2026-07-03 | 232 KB | Operator-return summary; locked program-roadmap decisions; the overnight autonomous-mandate window. |
| [`2026-06-29-and-07-05-07-06-log.md`](2026-06-29-and-07-05-07-06-log.md) | 2026-06-29, then 2026-07-05 to 07-06 | 145 KB | The 2026-06-29 Steward session log; a stale "RE-KEYED 2026-07-14" status-table set stranded at the file's tail by an earlier restructuring; the 2026-07-05 kernel `Term::Let` bug fix through the 2026-07-06 forward session log (including the Steward's Opus-to-Codex role handoff). |

**Total archived: ~2,264 KB across 11 segments** (vs. 2,247 KB of source
material moved out of the live file — the small increase is entirely the
mega-line reflow adding wrapped-line overhead, not new content).

## Using this archive

- Looking for **current status**? Don't start here — read
  `docs/program/IMPLEMENTATION-PROGRESS.md` at the repo root of this
  directory's parent.
- Looking for **why** a past decision was made, or the **detailed evidence
  trail** for a merged WP? Find the date range above and read that segment;
  each one is a straight, chronologically-ordered (mostly reverse-chron,
  newest-first within the segment) excerpt of the original tracker, with
  only cosmetic reflowing applied.
- Each segment file states its own exact source-line range from the
  pre-split `IMPLEMENTATION-PROGRESS.md`, for anyone auditing the split
  against `git log` history.
