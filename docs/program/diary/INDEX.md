# Ken program diary — index

This is the merged, dated narrative history of the Ken program. It replaces
two overlapping stores that used to hold the same kind of content in
different places — `docs/program/progress-archive/` (an 11-segment split of
the old `IMPLEMENTATION-PROGRESS.md` chronicle) and
`docs/program/STEWARD-DECISION-LOG.md` (the Steward's decision narrative).
Both are now folded into one dated diary, `docs/program/diary/YYYY/Mon/DD.md`,
so "what happened on day X" has exactly one place to look.

**Looking for current, non-historical state?**
- Live operator briefing + Steward resume state:
  [`CURRENT-BRIEFING.md`](CURRENT-BRIEFING.md) — read this first on any
  Steward resume.
- Current build/spec status: `docs/program/IMPLEMENTATION-PROGRESS.md` (owned
  by a different in-flight reorganization; not touched by this merge).

Each day file may carry two subsections where both source stores covered the
same day: **Progress chronicle** (from `progress-archive/`) and **Steward
decisions** (from `STEWARD-DECISION-LOG.md`). They are sectioned, not
interleaved — read chronicle first, then decisions, for that day.

Content that could not be reliably dated from the source text lives in
[`undated.md`](undated.md) rather than being force-fit to a guessed day (see
that file — as of this merge it is empty; everything in both source stores
carried a derivable date).

## Days, newest first

| Date | Summary |
|---|---|
| [2026-07-21](2026/Jul/21.md) | SPAN-SEAL and RT-PARITY both merged and closed (9 Decisions total, 7 spent on the Steward's own defects, not the rings'). Adversary's RT-PARITY hunt found **BUDGET-EFF**: `TransferCount.remaining` is computed from the raw request length, not the capped effective one, on both interpreter and native — fail-closed but wrong against locked spec `38`. ADR-0010 amended (STR-BIJ) reasoning from PRINCIPLES alone; RT-ESCAPE filed; provider-refusal routing rule set (route to Opus, then gpt-5.6); MODELS.md tier-table fleet-wide correction identified. |
| [2026-07-20](2026/Jul/20.md) | PX8-J-ERR (#30) hard-stopped to the Architect, ruled, and merged — closing out the PX8 series; issue #30 closed and the operator briefed. PX8-F (#11) also merged, CI green, §10 retros in. |
| [2026-07-19](2026/Jul/19.md) | PX8-TA oriented-subcontinuation redesign hits three hard-stops; a research-advisory pull produces a strong convergence signal on hard-stop #3. PX8-H finally merges after the marathon Architect ruling chain (see 07-17/07-18); PX8-DS starts. |
| [2026-07-18](2026/Jul/18.md) | The Adversary role is authored, wired, and launched (standing red-team, advisory-only). PX8-L merges after a CI-red respin. PX8-H's Architect↔implementer hard-stop chain runs to roughly its 15th ruling; the operator codifies a hard-stop→research-advisory escalation policy (§5a), then moves its trigger into the Architect itself. An operator-requested audit finds convo thread-discipline breakdowns (message-type mislabeling, durable-state noise in work threads). |
| [2026-07-17](2026/Jul/17.md) | The native capability-model campaign (PX13→PX14→PX15→PX16) completes and merges. PX8-H/PX8-P/PX8-V chain continues; `PRINCIPLES` transient-T change lands. Carries forward the 2026-07-16 PX5/PX5B/PX5C history as embedded "prior" recap blocks. |
| [2026-07-16](2026/Jul/16.md) | PX5, PX5B, and PX5C all close (merged, three retros each). PX6 blocks on a cross-lane capability-identity fork, routed to the Architect, then resumes. |
| [2026-07-15](2026/Jul/15.md) | PX3 and PX4 both merge and close. PX5 + PX6 released after the Architect's PX-B design ruling; the vocab-split hard-stops resolve; CAT-TAX closes out fully. A dedicated "Research" seat is provisioned at the operator's request. Architect rules the PX5 boundary should be a predecessor WP. |
| [2026-07-14](2026/Jul/14.md) | The LET chain completes (LET-3/5/6) through two AX-1/AX-2 Architect rulings. Milestone C (CC5–CC7) and Program I (I-4/I-5) both complete; ADR-0017 lands. The SEC2 honesty erratum catches and fixes a false security claim that had reached `main`. CAT-TAX closure begins; PX4B (native production spine) is blocked twice by the Architect on real structural catches. |
| [2026-07-12](2026/Jul/12.md) | ADR 0014 (package-abstraction ruling) fully settles, operator-ruled option C; ADR 0015 (remove `use M` open-import) is operator-directed. The F3b fail-closed design arc opens. Catalog housekeeping issues #26/#28 close. |
| [2026-07-11](2026/Jul/11.md) | proof-vocabulary A/B/C merges and closes (#19). |
| [2026-07-09](2026/Jul/09.md) | DS-1 merges, then the operator calls a **hold** on the campaign for a process review. Catalog reframed into a data-structures program; Sections/Domains + retro discipline land. SURF DEF / SURF NAMED PROOF close out. |
| [2026-07-08](2026/Jul/08.md) | NC27 closes, completing the NC1–NC27 compiler-campaign build-out. SURF DEF fully closes and is cleaned up; catalog work becomes the next frontier. |
| [2026-07-07](2026/Jul/07.md) | NC15 closes; the integrator watchdog playbook closes out. |
| [2026-07-06](2026/Jul/06.md) | The CAT-2/CAT-3/CAT-4 catalog campaign progresses through its D1–D3 decision sequence; KM (dependent-match, then KM-sigma) mechanism WPs merge. Continues the Steward's Opus→Codex role handoff from the previous night. |
| [2026-07-05](2026/Jul/05.md) | Operator directive: fix a kernel `Term::Let` bug — root-caused and fixed same session. CAT-2 D3 ruled; SURF-1/SURF-2 progress. Steward hands off from Opus to Codex/gpt-5.5 at ~22:30. |
| [2026-07-04](2026/Jul/04.md) | Roadmap decision: catalog-led is the next campaign. effect-composition elaboration merges; CAT-1 merges; the VAL2 Rosetta corpus reaches 16/0, closing the filesystem workstream. Credit-window strategy discussed. |
| [2026-07-03](2026/Jul/03.md) | Operator-return summary after an overnight autonomous-mandate window; program-roadmap decisions locked. |
| [2026-06-29](2026/Jun/29.md) | The earliest chronicle in the archive — the 2026-06-29 Steward session log. |

## Month indexes
- [2026 / Jun](2026/Jun/INDEX.md)
- [2026 / Jul](2026/Jul/INDEX.md)
