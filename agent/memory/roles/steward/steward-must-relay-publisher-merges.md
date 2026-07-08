---
scope: roles/steward
audience: (see scope README)
source: private memory `steward-must-relay-publisher-merges`
---

# The Steward must relay publisher-path merges

Publisher-path merge handling does not automatically wake every downstream
actor. Team leaders and the spec enclave may not be mentioned by a merge note,
and a scripted merge return is local to the Steward. So any actor who parked
with "**once it merges, I'll do X**" (collect retros, start the dependent WP,
un-stage a gated net) can be **stranded**: the merge fires, but they never learn
of it, and they sit idle at `❯` in their pre-merge state.

**The Steward is the relay node.** The instant a merge notification lands,
**check who is waiting on that merge to trigger their next step, and relay it to
them** — proactively, as part of processing the merge, **not** later when the
watchdog catches the idle stall.

**Live (2026-07-04, FS-driver Phase 1):** the merge note for `fd5451b` (PR
#280) reached only me. spec-leader had parked with "once it merges, I'll collect
retros" — and sat idle 40 min, Phase-1 retros uncollected, because it was never
notified. My watchdog caught it a tick later; I relayed (`evt_4cfhnqgshhc80`) and
it unblocked. Had I relayed *when I got the merge notification*, the 40-min retro
stall wouldn't have formed.

**How to apply:** merge-notification handling is a two-part act — (1) verify the
merge on `main` by content + update the tracker, **and (2) relay to every
downstream actor whose next move is gated on it** (retro collection → the WP's
owning leader; a dependent WP → its team; a gated-net un-stage → the owner). If
a leader's last posted intent was "once it merges I'll…", that's a relay you owe
them now. Sibling of re read latest events immediately before a stall nudge
(both about the Steward's role as the information hub the fast-moving federation
routes through) — but this one is a *proactive* relay, not a stall-diagnosis.
