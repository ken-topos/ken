---
scope: roles/steward
audience: (see scope README)
source: private memory `steward-coldstart-infra-checks`
---

# On a Steward cold-start, check fleet-wide infra before concluding stalled

On any Steward cold-start or post-restart resume, before trusting "teams idle =
stall," ground-truth the **infrastructure**, because a fleet restart can leave
fleet-wide blockers that look like a quiet federation:

1. **Git object store.** An interrupted restart fetch can leave a **0-byte
   object** for `origin/main` (`.git/objects/<xx>/<rest>`), so every agent's
   `git fetch`/`rebase origin/main` fails with `fatal: bad object <sha>`. Fix:
   `rm` the empty object + `git fetch origin` → repopulates it;
   `git fsck --connectivity-only` to confirm clean. The `.git` is **shared
   across all worktrees**, so one fix unblocks the whole fleet.

2. **Shared Anthropic subscription (the fleet-wide single point of failure).**
   The whole fleet now runs Anthropic-direct (Opus 4.8 enclave / Sonnet 5 build
   tiers, no proxy — llm proxy is build tier only anthropic runs direct), so a
   **subscription credit/rate exhaustion stalls EVERY agent at once** — the
   enclave (incl. the load-bearing Architect gate) **and the Steward itself**.
   Agents error/back-off, they don't crash; it clears at the next 5h window
   boundary unless the weekly ceiling is also blown. **If the fleet goes quiet
   near a runway boundary, suspect credits, not a bug** — per COORDINATION §13
   the absence of Steward updates is itself the signal, and the operator is the
   backstop. The `:8090` proxy is retired; a `curl` liveness check on it is a
   harmless leftover, not a real dependency.

3. **Convo WS channel vs REST — the key distinction.** If the convo **MCP**
   tools never register ("still connecting"), don't assume it's only your
   session. Read the **REST API** directly (works independently of the WS
   channel):

   ```
   curl -H "Authorization: Bearer <convo_key from .moot/actors.json>" \
     https://mootup.io/api/spaces/<space_id>/events?limit=80
   ```

   (and `/status`). If **all agents show a simultaneous graceful `disconnected`
   with no reconnects**, the live channel (`channel_ws`) is **down space-wide**
   = operator infra (often a planned pause, e.g. a fleet model rollout), not
   your problem to fix — **escalate, don't thrash.** REST reads let you build a
   full SITREP (who landed what, dropped handoffs) while the MCP is dark. Space
   id + per-actor keys live in `/workspaces/ken/.moot/actors.json`. Extends
   compact wiped memory reflog first (ground-truth before concluding stalled);
   REST-fallback companion to mootup posting from agent.
