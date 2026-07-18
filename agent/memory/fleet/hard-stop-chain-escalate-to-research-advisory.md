---
scope: fleet
audience: (see scope README) — the Steward (drives it), the Architect + any implementer in a hard-stop chain, and the research agent (executes the advisory)
source: operator (Pat) directive, 2026-07-18, after PX8-H ran to seven hard-stops
---

# On the 3rd hard-stop, call in research and hold the Architect

When an Architect↔implementer design ruling becomes a **chain of hard-stops**
(implementer builds the ruling → hits a new structural wall → hard-stops with
evidence → Architect rules again → repeat), do not let it run unaided. One or
two hard-stops routinely resolve; a **third** means the pair may lack a clear
path, and an independent prior-art perspective beats another unaided round.
PX8-H ran to **seven** before an outside view was brought in — that is the
failure this rule prevents.

**Trigger (mechanical):** the **3rd** hard-stop in one mechanism chain on one
WP, then every **3rd** after (6th, 9th, …). Never earlier than the 3rd. A chain
that is *visibly progressing* (checkpoints advancing, failure moving strictly
deeper) **still triggers** — "it's making progress" is not a reason to withhold
the check, just as it is not a reason to skip a compaction. Count **consecutive
hard-stops on the same design question**, not unrelated stalls.

**Procedure (Steward-driven):**

1. **Hold the Architect fully** — in-thread mention: hold the next ruling; a
   research advisory is incoming and will mention you; resume on it. **Not
   timeboxed**, and the Architect does **not** keep working in parallel — its
   tokens are better spent *with* the advisory's context than on another unaided
   pass.
2. **Kick research, transport/framing only** — in-thread mention with the WP
   name, `thread_id`, the hard-stop `evt_…`s, the latest clean checkpoint SHA,
   and the one question the chain circles. Ask it to review the thread + work
   done and search prior art (`local/refs/`, both tiers, **and** the internet).
   **Nothing more** — no Steward design opinion, or the Steward/research becomes
   the de-facto design authority.
3. **Research posts its advisory back in the same thread**, mentioning the
   Architect + Steward, **labeled advisory, not a ruling**. That mention is the
   Architect's resume signal. Research is prompt because the Architect is held
   (its latency is frontier latency).
4. **Re-trigger only at 6th/9th** — refine the search focus given the prior
   advisory + new stops; never re-kick at the 4th/5th.

**Lanes unchanged:** the Steward never adjudicates the mechanism, research never
rules (advisory only), the Architect still owns the ruling. The Steward's part
is the transport + the hold; log the escalation in the decision log. Codified in
the [[playbooks-state-mechanism-not-intent]] corpus at `steward.md §5a` +
`research.md`. Sibling of [[spec-enclave-always-compact-before-new-work]]: a
mechanical count defeats the "one more round will crack it" rationalization.
Requires the research agent's `local/refs/` read access (granted 2026-07-18).
