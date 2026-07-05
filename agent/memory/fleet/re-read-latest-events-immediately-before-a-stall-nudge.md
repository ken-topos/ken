---
scope: fleet
audience: (see scope README)
source: private memory `re-read-latest-events-immediately-before-a-stall-nudge`
---

# Re-read latest events immediately before a stall-nudge

Before posting a watchdog stall-nudge, **re-read the latest convo events
immediately before the mention** — not the read you took minutes earlier at the
top of the pass. The federation moves fast: a handoff/vote/Decision can post in
the gap between your initial `get_recent_context` and your nudge, so a "no
handoff event exists / X is idle-waiting" conclusion drawn from the earlier read
is **stale**, and nudging on it is a false stall.

**Live (2026-07-04, Map capstone-close reconcile):** I read recent context, then
did git checks (confirmed `wp/map-capstone-close @ b2ba86a` committed), then
nudged spec-leader to "open the Decision" on a supposed silent handoff. But
spec-author's handoff HAD posted in the gap, and spec-leader had already
reviewed it and caught a real whole-doc-sweep miss and was correctly holding for
the fix — healthy gate movement, not a stall. spec-leader corrected me. No harm
(it ignored the premature nudge), but it was noise. Second false-stall of the
program (after the K7 one).

**Live (2026-07-04, FS-driver deliv-4 companion) — 3rd false-stall, pane-reading
variant:** I nudged CV as "parked" on the `FS-driver-conformance.md` companion
spec-author was holding the Decision for. CV was **actively committing it**
(`9c7c1b9`, same minute as my nudge). My `capture-pane` even SHOWED the truth —
the live line was a running shell command `git cat-file -t 9c7c1b9` (the exact
companion SHA) verifying its just-made commit — but I anchored on the **stale
chrome** (an old "Grounding depth axes" spinner, the previous task's task-list,
and the empty `❯` at the bottom) and read "idle/parked." **The live signal wins:
a running command referencing the very artifact you think is un-started is the
agent DOING it — read the active command line, not the surrounding chrome.**
Compounded by my own prior "you're clear until the run" message, which built a
plausible parked-narrative I then *confirmed* instead of *falsified*. When
you've authored a narrative that predicts a stall, actively look for the pane
evidence that REFUTES it before nudging.

**Live (2026-07-04, CAT-3 close) — 4th false-stall, limit-truncation variant:**
I DID re-read at nudge time, but used
`get_recent_context(since=<cursor>, limit=6)`. A `since` read returns
**oldest-first**, so `limit 6` returned the 6 *oldest* events after the cursor
and **truncated the newest** — which was exactly spec-leader's "retros in"
handoff (`evt_606…` @22:34:31). My read's latest event was 22:34:04 ("holding
for the retro"), so I concluded spec-leader was parked and nudged — but the
handoff had landed 27s later, outside my truncated window. The survey overlay on
its pane compounded the misread. Low harm (dismissing the survey was real
hygiene; spec-leader just re-confirmed done). **Rule: a nudge-time re-read must
reach the actual TAIL. A small `limit` on an oldest-first `since` hides the
newest events — the ones that answer "did the handoff happen yet." Use a large
limit, or read newest-first, or confirm the latest event's timestamp is within
seconds of now before trusting "X hasn't posted."**

**How to apply:** the git-truth (a branch is committed) is necessary but NOT
sufficient to declare a handoff silent — the convo-truth ("the author never
posted the handoff") must be checked against a read taken *at nudge time*, with
no intervening tool calls that could let the state advance. If you did git work
between the read and the nudge, re-read first. Corollary of the K7 false-stall
rule (verify against ground truth over a deep window; when panes/reads disagree
with the live state, the live state wins) and sibling of leader since window
blindness on decision votes (both: reconcile against the FULL current state,
never a delta or a stale snapshot). Also: a committed branch with the author
idle is more often "authored, in review, author idle by design" than "silent
handoff" — the enclave author hands off and goes idle normally.
