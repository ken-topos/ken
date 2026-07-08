---
scope: roles/steward
audience: (see scope README)
source: private memory `fleet-model-rollout-stagger-restarts`
---

# Stagger restarts when rolling the fleet onto a new model

**Live 2026-07-01: operator released Claude Sonnet 5, set all non-enclave roles
`model = "claude-sonnet-5"` (no `env` — Anthropic direct), asked to restart the
fleet "as available." I batch down+exec'd 15 idle agents back-to-back → all came
up on Sonnet 5 but hit a transient rate-limit error mid-onboarding
(`Server is temporarily limiting requests (not your usage limit)`), went
idle, and **never re-established convo presence**
(list_participants showed them `Offline`, last-seen = the restart moment; some
downgraded `[mcp]`→`[channel]`). kernel-qa reported "orientation() and every
convo tool unreachable." The MCP server didn't finish its handshake under the
burst. Fix that worked 100%: re-restart each ONE AT A TIME (~20s apart) — every
agent came back `[mcp] Online`.**

**Rules for a fleet model-swap / rollout:**
1. **Canary first.** Restart ONE idle, low-stakes agent, confirm the new slug
   launches healthy (`tmux capture-pane` → statusline shows the model, e.g.
   "Sonnet 5"; no API-error). A bad slug = API-error loop; a launched-alone
   canary proves the slug without fleet risk. (foundation-qa alone = clean; the
   15-burst = broken. The difference IS the burst.)
2. **STAGGER, never batch.** Restart one at a time, ~20s apart (down → exec →
   ~9s to the consent modal → Enter → ~11s settle → next). A brand-new model has
   tight launch-day capacity; 15 simultaneous fresh sessions trip a server-side
   429 that survives as broken MCP, not just a slow turn. `moot.toml`'s
   `launch_stagger_seconds` only paces `moot up`, NOT manual `moot exec`.
3. **Each `moot exec` shows a first-launch dev-channel consent modal**
   ("WARNING: Loading development channels … Enter to confirm · Esc to cancel",
   option 1 = local dev, preselected). The agent is frozen until you
   `tmux send-keys -t moot-<role> Enter`. Do it per agent after boot.
4. **Verify recovery via convo, not the pane.** `list_participants` `[mcp]` +
   `Online` + fresh `Last seen 0m` = truly reconnected. `[channel]`/`Offline`
   with a stale last-seen = the session registered once but its MCP isn't live
   (wedged) — needs a clean restart, a nudge won't fix it (tools are unreachable
   for the whole session).
5. **Re-verify each agent's LIVE state immediately before `moot down`.** A
   3-min-old `list_participants` snapshot goes stale: I killed **kernel-qa
   mid-K2c-QA** because it drifted idle→busy in the gap (K2c handoff arrived).
   Committed work (the branch) is safe — a fresh QA redoes it — but don't assume
   "idle" from a stale read. Sharpens pane suggestion text is not agent state.

**Who to defer:** never restart an agent mid-uncommitted-work (loses it) or a
coordinator mid-merge (drops the thread). Enclave stays Opus (no restart). Roll
active coordinators/implementers at their WP boundaries, staggered. Sonnet 5
early read: strong — careful QA (kernel-qa's rigorous K2c verdict) + good
situational awareness (the merge/publisher path caught a red main). Extends llm
proxy is build tier only anthropic runs direct (Anthropic-direct, no env) and the
`moot exec`-from-`/workspaces/ken` rule in bash cd main repo vs steward
worktree.
