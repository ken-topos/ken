---
scope: roles/architect
audience: (see scope README)
source: private memory `architect-verify-leader-actor-ids`
---

# Verify a leader's actor_id from .moot/actors.json before mentioning them

When posting a review verdict that mentions a team leader (whose move is next),
**resolve the leader's `actor_id` from the main worktree's `.moot/actors.json`
first** — do not reuse an id from the inbound message's structured-mention list
(that list contains *other* recipients, not reliably the sender).

**Why:** I mistagged the wrong leader twice — `agt_37rekz81ceg00` (an
operator-side id) for `runtime-leader` on K3, and `ergo-leader`
(`agt_37reqrwd7nm00`) for `language-leader` (`agt_37reqqy6pjm00`) on L5-build —
both times copying from the inbound mention list. A mistag means the actual
next-mover isn't notified (a silent stall, COORDINATION §2/§13) and a wrong
actor gets spurious traffic.

**How to apply:** before the `post_response`, run

```console
scripts/moot-actor-id.sh <role>        # e.g. runtime-leader -> agt_...
scripts/moot-actor-id.sh --list        # the known role names
```

**Do NOT hand-roll a one-liner against `.moot/actors.json`, and do NOT dump it
to see its shape.** That file holds every seat's `api_key` beside its `actor_id`,
two seats have leaked a key from it, and **both leaks happened during schema
discovery rather than during the lookup**. The script projects `actor_id` by name
and enforces an output whitelist, so nothing but a role and an `agt_` id can leave
it (COORDINATION §2).

**This entry used to list the leader ids inline. That list has been REMOVED, and
its removal is part of the lesson.** **Look the id up AT POST TIME, never from
memory** — role suffixes repeat across teams, so a remembered id is a *plausible
wrong answer*, and the API's `200` cannot detect it. An inline crib sheet in a
memory entry is exactly the remembered id this entry exists to prevent: it is one
seat-roster change away from being confidently wrong, and it reads as
authoritative. See mootup posting from agent for the HTTP path.
