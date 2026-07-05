---
scope: fleet
audience: (see scope README)
source: private memory `compact-verify-survey-can-eat-the-compact-command`
---

# A Claude Code survey prompt can eat a `/compact` command

When I run the build-team compaction handoff gate
(`tmux send-keys -t moot-<role> "/compact"` → Enter, per member), a Claude Code
**feedback survey** ("How is Claude doing this session?
`1: Bad 2: Fine 3: Good 0: Dismiss`") can **pop up and silently intercept the
`/compact`** — the keystrokes go to the survey (which only accepts 0–3), the
command never runs, and the pane returns to an empty `❯` at the **original
ctx%**. If I only glance at "sent" or a transient progress bar, I'll wrongly
believe the member compacted → risk the exact stale-context kickoff Pat forbids.

**Live (2026-07-04, W-style-match-IH build kickoff):** leader (14%) and
implementer (19%) compacted to 0% first try, but **language-qa's `/compact` was
eaten by the survey twice** — it sat at 16% while I thought all three were done.
Only the ctx-drop verification (leader 0% / impl 0% / **qa still 16%**) caught
it.

**How to apply — this is why gate step 5 ("VERIFY THE DROP") is
non-negotiable:**
- **Dismiss the survey with its own `0: Dismiss`** (send `"0"` — surveys respond
  to the bare digit, no Enter), confirm it's gone, THEN send `/compact`.
- **Never trust the "sent" line or a momentary `▰▰▰ NN%` bar.** The only proof
  is **`ctx` actually falling to ~0%** on a `capture-pane` taken *after* the
  30–60s stale-display window. `Compacting conversation… NN%` climbing (with an
  elapsed timer) is the real in-progress signal; an unchanged ctx% with no
  compaction message ⇒ it didn't run ⇒ resend.
- A queued `/compact` behind a busy turn ("Press up to edit queued messages")
  fires at the turn boundary — fine; a redundant second one just reports "Not
  enough messages to compact." (harmless).
- **The slash-command autocomplete menu can swallow your Enter (2026-07-04, FS
  Runtime kickoff).** Typing `/compact` pops the `/`-command menu; a blind
  `send-keys Enter` fired too soon (even ~2s later, batched across 3 panes) left
  **all three at empty `❯` / unchanged ctx** — the submit didn't land. The
  reliable sequence: type `/compact`, `capture-pane` to confirm the input line
  reads `❯ /compact` (menu open), send Enter, then confirm the pane transitioned
  to **queued ("Press up to edit queued messages") or `Compacting…`** — only
  then is it accepted. Don't fire Enter and assume; confirm the transition, then
  poll ctx→0. (Same "surface signal lies" theme — the send returns success while
  nothing was submitted.)

Sibling of the compaction discipline in re read latest events immediately before
a stall nudge — same theme: the surface signal (sent / idle box / stale ctx)
lies; verify the underlying state.
