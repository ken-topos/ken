---
scope: roles/spec-leader
audience: (see scope README)
source: private memory `leader-relays-frame-citations-must-reverify-too`
---

# A leader relaying a WP frame must independently reverify its citations

On WP 44-capacity-restate (2026-07-02), Steward's frame asserted
"`OQ-systems-target` closed 2026-07-02 (operator ruling)" as a fixed input. I
relayed that frame to spec-author verbatim in my kickoff message, without
grepping `90-open-decisions.md` myself first. spec-author baked it into `/spec`
in good faith; Architect's independent grep caught that the id was fabricated
(zero hits anywhere on `origin/main`) — Steward owned the root cause (they wrote
the id without ever registering it).

**Why this is mine to fix, not just spec-author's.** I am a relay point in the
citation chain (Steward → me → spec-author), and laundered citation authority
says every adoption point must re-ground, not just the terminal author. I
treated "Steward's fixed input" as inherently trustworthy because it came from a
leader frame, which is exactly the "kickoff frame is a secondary artifact" trap
the memory already names — I knew the rule and still didn't apply it to my own
hop.

**How to apply.** Before relaying a Steward (or any) WP frame downstream, grep
any cited `OQ-`/decision-id/ADR reference against the live register
(`90-open-decisions.md`, `docs/adr/`) myself — a 10-second check — before it
leaves my hands. This doesn't replace the author's own re-verify at pickup
(defense in depth is the point), but catching it at the relay hop is strictly
cheaper than catching it after a full elaboration + review cycle. Sibling of
laundered citation authority, applied to the leader-as-relay role specifically
rather than the author-as-adopter role the parent memory already covers.
