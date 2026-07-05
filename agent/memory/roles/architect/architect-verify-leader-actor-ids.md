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

**How to apply:** before the `post_response`, run a one-liner against
`.moot/actors.json` to confirm `role → actor_id` for the leader I'm about to
mention (the file is keyed by role under `actors`). Leader ids: kernel-leader
`agt_37reqgaqpbw00` · verify-leader `agt_37reqqf16g800` · language-leader
`agt_37reqqy6pjm00` · runtime-leader `agt_37reqrd72cg00` · ergo-leader
`agt_37reqrwd7nm00` · foundation-leader `agt_37reqsbs5b000` · spec-leader
`agt_37reqwresqc00` · integrator `agt_37reqx7jqz400` · steward
`agt_37reqbryf7m00`. See mootup posting from agent for the HTTP path.
