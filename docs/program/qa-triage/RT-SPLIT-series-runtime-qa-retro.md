# RT-SPLIT series — runtime-qa retro (trap / held / carry)

Committed directly (convo MCP client outbound still dead this session,
checked again just before writing this — same pattern as the three
per-slice verdicts this covers:
`RT-SPLIT-slice5-runtime-qa-verdict.md @ 53501ffe`,
`RT-SPLIT-slice6-runtime-qa-verdict.md @ a4473ab0`,
`RT-SPLIT-slice7-runtime-qa-verdict.md @ 54692e07`).

Scope: my own QA involvement covered slices 5, 6, and 7 (the tail of the
series) — this retro speaks to that span, not the full seven slices.

## trap

The slice-7 headline SHA lag: I fixed a self-caught out-of-scope `store.rs`
hunk and committed the fix, but left the verdict's **bold headline** naming
the pre-fix commit (`1b2d3e96`) while the correction sat in a lower
"addendum" paragraph — runtime-implementer caught it
(`evt_7jvyytqn48bk2`) before it reached a Decision. This is the same class
of defect as the fleet lesson *"a gating instruction must LEAD the
message"* — a truncated notification, or a reader who only reads the first
line, sees the stale/wrong SHA. I re-anchor the headline itself on every
correction now, most-recent-first, rather than appending. It recurred a
second time within the same slice (the `b6ed5445` → `f5e08452`-adjacent
re-anchor) and I applied the same discipline both times without being told
twice — worth naming as the corrected behavior, not just the miss.

## held

- Never trusted a "supersedes"/"blob-identical"/"comment-only" claim on
  report across any of the three slices — re-derived AC-2 (rustdoc symbol
  diff) fresh from a real build on both sides every time, independently
  re-extracted every visibility-widening line against the pre-existing
  declaration on base, and independently verified every AC-8 cross-tree
  import citation by reading the actual usage site rather than the cited
  line number alone.
- Ran a live mutation-proof on a genuinely load-bearing item each slice
  (the `RuntimeIrRunReport` identity guard on slice 6;
  `recursive_computational_result_depth` on slice 7, deliberately chosen
  because the implementer's own commit flagged it as a previously
  mis-derived placement — the sharper target, not an arbitrary pick) and
  confirmed the predicted failure before reverting cleanly.
- On the slice-7 re-anchor, independently re-verified all 9 claimed
  blob-identical paths via `git rev-parse <sha>:<path>` myself rather than
  accepting the implementer's own per-path verification as sufficient —
  costs little, closes the gap between "reported" and "confirmed."

## carry

When a final/closing slice arrives with a long chain of "the first pass was
wrong" self-corrections already in the commit history (slice 7 had three:
restricted-visibility items dropped by a glob enumeration, a consumer
outside the subtree, a parent unable to see a child's own private caller),
each admitted miss names a **category** worth specifically re-checking on
the final candidate — not just trusting that the commit's own fix closed
the whole category. Read the corrections as a checklist, not just history.

Separately: a bound verdict is a **living document** across re-anchors, not
a one-shot artifact. Every SHA correction belongs in the headline itself,
with the correction history kept visible (most-recent-first) so a reader
who stops at line one never acts on stale truth — this cost a
round-trip once this series; it shouldn't cost one again.
