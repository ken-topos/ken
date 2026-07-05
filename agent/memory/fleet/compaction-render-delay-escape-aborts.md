---
scope: fleet
audience: (see scope README)
source: private memory `compaction-render-delay-escape-aborts`
---

# `/compact` has a render delay; Escape aborts it, don't send it

Running the §2c compact-gate (send `tmux send-keys /compact` + a separate
`Enter` to each enclave/team seat, then verify the ctx drop), there is a **real
render delay of several seconds** between the `Enter` and the
`✻ Compacting conversation…` progress bar appearing. So a capture taken
immediately after `Enter` shows an **empty prompt + unchanged ctx% + no
Compacting marker** — which looks identical to a swallowed/no-op command **even
when compaction is about to start**.

**The trap (mine, CAT-3 kickoff gate):** on that ambiguous read I reflexively
sent **Escape to "clear state" and re-typed `/compact`** — and **Escape aborts
an in-flight compaction** (`AbortError: Compaction canceled.` appeared 3× in
CV's scrollback; each retry started compaction, then the next Escape killed it).
I burned ~4 cycles fighting a compaction that was firing fine each time.

**Fix / procedure:**
1. After `/compact`+`Enter`, **wait a full tool round-trip** (re-capture is
   enough delay) and **re-capture the FULL pane** (`tail -14`, not `tail -6`)
   before concluding anything — look for `Compacting conversation…` OR the
   `AbortError`/scrollback history.
2. **Never send Escape as a reset** during the gate — Escape = abort-compaction.
   If a seat truly didn't fire, just re-send `/compact`+`Enter` (a second
   `Enter` if the autocomplete palette ate the first), no Escape.
3. **Read the LIVE bottom `ctx N%` line specifically.**
   `grep 'Compacting' | head -1` catches a **stale scrollback** Compacting frame
   (top-to-bottom order); `grep 'ctx N%' | tail -1` can catch the mid-compaction
   status line (ctx doesn't update until compaction completes). Confirm with
   `tail -6` of the live pane.
4. A Sonnet seat (e.g. spec-leader) with the full playbook loaded has a
   **post-compaction floor ~16%**, not 0% — it still counts as compacted if it
   visibly ran the bar; don't chase it to 0.

Sibling of compact verify survey can eat the compact command (a survey modal
silently eats the send-keys) — both are "the send-keys didn't do what the pane's
first frame suggests"; verify by re-reading the live pane, not by assuming and
re-sending destructively.
