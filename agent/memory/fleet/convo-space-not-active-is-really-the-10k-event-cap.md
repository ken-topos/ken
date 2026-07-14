---
name: convo-space-not-active-is-really-the-10k-event-cap
description: convo `post_response` failing with "Space not active" is usually the space's hard 10,000-event cap (HTTP 409) — diagnose from the HTTP status in .moot/logs/mcp-<role>.log, never the adapter's error string.
metadata:
  type: reference
  scope: fleet
---

**`{"error":"Space not active"}` from `post_response` is an adapter mistranslation.**
The real failure is usually a backend **HTTP 409** on
`POST /api/spaces/<id>/response`:

```
{"code":"http_409","message":"Space has reached the maximum of 10000 events"}
```

A convo space has a **hard 10,000-event cap**. When it fills, **every**
participant's append 409s, fleet-wide and permanently — the federation's whole
coordination channel dies at once.

**The trap:** the symptom set actively misleads.

- `get_space_status` still says **`active`** (it reports `active | 10087 events`)
  and `list_spaces` shows the space healthy — so "is the space paused?" is a
  dead end. Toggling paused→active does nothing.
- **Reads keep working** (`orientation`, `get_recent_context`, `list_*`) and so
  does **`update_status`** — they hit different endpoints that don't check the
  cap. So the seat *feels* connected.
- Only the **append** path fails, so it reads like "my client is wedged," which
  invites the two natural-but-wrong fixes: **restart the backend** (there is no
  stuck lock) and **reconnect/recreate the MCP client** (a client reconnect
  cannot fix a server-side cap — and the `moot.adapters.mcp_runner` process is
  long-lived anyway, so a `/compact` or even a fresh Claude Code session
  re-attaches to the *same* runner and changes nothing).

**How to diagnose it in one step — read the HTTP status, not the error string:**

```sh
grep -E '/response|→ 4[0-9][0-9]' .moot/logs/mcp-<role>.log | tail
```

A `→ 409` is the cap. To see the server's actual message, probe the API directly
with the seat's key (from `/proc/<mcp_runner-pid>/environ`: `CONVO_API_KEY`,
`CONVO_AGENT_ID`) — `POST .../response` with `participant_id` + `agent_name` +
`text` + `message_type`, and read the body.

**Confirm it's fleet-wide before blaming your seat:** grep every
`.moot/logs/mcp-*.log` for `→ 409` and for the last *successful* append. If
several unrelated seats stop appending at the same instant, it is the space, not
you. (2026-07-14: the Ken space filled at CV's 23:27:41 post = event #10000;
steward, CV, spec-author, spec-leader and runtime-qa all 409'd after it. Two
sessions were spent on the wrong two theories before anyone read the status
code.)

**No agent can fix it.** Nothing exposed reduces the count — `archive_session`
archives a *session transcript*, not space events; `update_space` only sets
description/status/links; there is no create-space tool. It is an **operator**
action: raise/clear the cap on the backend (preserves the space and every
`thr_*`/`evt_*` id — strongly preferred), or provision a new space and re-point
`CONVO_SPACE_ID` for every seat (a full fleet restart, and every live thread id
dies with the old space).

**Posture while it's down:** the rings cannot turn (a leader cannot mention its
implementer), so **stay quiescent** — don't kick new work into a dead channel,
don't hammer `post_response`, and don't hand-relay the fleet over `tmux`
(that turns the Steward into a manual message bus). **Git, `gh`, and the
publisher path are convo-independent**, so merges in flight can still land.

Related: [[steward-coldstart-infra-checks]], [[pane-suggestion-text-is-not-agent-state]].
