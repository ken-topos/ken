---
scope: build/leaders
audience: (see scope README)
source: private memory `leader-since-window-blindness-on-decision-votes`
---

# A leader must re-scan the full Decision vote-state, not just the latest vote

When a merge-Decision reviewer casts a vote, the notification/since-window that
wakes the owning leader often shows **only that one latest vote** — not the
earlier votes that already landed. A leader that reasons from the delta alone
concludes "still awaiting the other reviewers" and **ends its turn with a
fully-green Decision left unresolved** — the "merge Decision green+approved but
unmerged" stall (COORDINATION §13), because gate votes are cast into threads and
typically do **not** re-notify the leader, so nothing wakes it again.

**Live (2026-07-04, Map capstone `dec_72bq23xmx63mb`):** Architect APPROVE
(02:22) + foundation-qa APPROVE (02:23) landed first; CV's APPROVE (02:29) woke
foundation-leader, which saw only CV's vote and concluded "awaiting Architect +
foundation-qa" — when all three were in. It sat idle at a green Decision until
the Steward watchdog nudged it with the three vote event_ids to reconcile.

**How to apply:** (1) Build-leader rule — when a vote notification wakes you,
**re-scan the FULL Decision state** (`list_decisions` / read every reviewer's
vote), never conclude "pending" from the latest event alone. Resolve + send
`merge_ready` the instant all required votes are APPROVE. (2) Steward watchdog —
"green+approved but unmerged" with the leader idle is usually THIS: the leader
under-read its since-window. Diagnose via capture-pane (the pane shows the
mistaken "awaiting X" conclusion), then nudge with the concrete vote event_ids +
timestamps so it can self-verify all votes are in. Leader-side dual of the
Steward-side since-window false-nudge rule (playbook §7); both are
reconcile-against-full-state-not-the-delta. Fold into the build-leader playbook
at the next corpus-routing epoch.
