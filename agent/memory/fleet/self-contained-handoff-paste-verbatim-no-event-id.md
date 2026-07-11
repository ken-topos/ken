---
scope: fleet
audience: (see scope README)
source: private memory `self-contained-handoff-paste-verbatim-no-event-id`
---

# Hand a ruling/artifact VERBATIM in-thread — never "see evt_XXXX"

When you deliver a technique ruling, a helper, a spelling, or any artifact a
recipient must **apply**, put the **exact text in the message**. Do **not**
refer them to a prior event by ID (*"apply the helper from evt_XXXX"*, *"per my
ruling above"*).

**Why:** the build/enclave terra (Codex/GPT) seats are event-driven and their
**event-by-ID retrieval is unreliable** — a seat handed only a pointer often
**cannot fetch `evt_XXXX`** from its own session search, so it stalls and
re-asks, and you re-paste. On a no-poll fleet that is **two full rouse
round-trips** wasted on a pointer. (2026-07-11: an Architect ruling referenced
`evt_4q77x9434bb38`; the Foundation implementer could not retrieve it, held
correctly rather than guess, and only moved once the Architect **re-posted the
probe-verified helper signature + body verbatim** in-thread. It happened twice
in one afternoon.)

**How to apply (any role that hands work down — Architect, leaders, Steward):**
- Paste the **exact, probe-verified signature + body** (or spelling/diff), plus
  the precise **call site** and the byte-for-byte spots to reconcile. The
  recipient should apply it from the **single message, zero further lookups**.
- If you must cite a prior event for provenance, still **inline the content** —
  the ID is a footnote, never the payload.
- Same shape when **relaying**: a leader passing an Architect ruling to its
  implementer re-pastes the artifact; it does not forward a pointer.

Sibling of [[playbooks-state-mechanism-not-intent]] (hand the mechanism, not a
reference to it) and the no-poll rouse discipline. If a seat *cannot* fetch by
ID, that is the environment, not the seat's fault — design the handoff so it
never needs to.
