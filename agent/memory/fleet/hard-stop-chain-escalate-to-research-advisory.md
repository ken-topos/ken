---
scope: fleet
audience: (see scope README) — the Architect (owns the trigger), the Steward (backstops it), any implementer in a hard-stop chain, and the research agent (executes the advisory)
source: operator (Pat) directive, 2026-07-18 (established after PX8-H ran to seven hard-stops; trigger moved into the Architect after it ran past ten)
---

# On the 3rd hard-stop, the Architect holds and calls in research

When an Architect↔implementer design ruling becomes a **chain of hard-stops**
(implementer builds the ruling → hits a new structural wall → hard-stops with
evidence → Architect rules again → repeat), do not let it run unaided. One or
two hard-stops routinely resolve; a **third** means the pair may lack a clear
path, and an independent prior-art perspective beats another unaided round.
PX8-H ran to **seven** before an outside view was first brought in, and past
**ten** before the trigger was made native to the Architect — that is the
failure this rule prevents.

**Trigger (mechanical):** the **3rd** hard-stop in one mechanism chain on one
WP, then every **3rd** after (6th, 9th, 12th, …). Never earlier than the 3rd. A
chain that is *visibly progressing* (checkpoints advancing, failure moving
strictly deeper) **still triggers** — "it's making progress" is not a reason to
withhold the check, just as it is not a reason to skip a compaction. Count
**consecutive hard-stops on the same design question**, not unrelated stalls.

**The trigger lives with the Architect, not the Steward** (operator,
2026-07-18). Moving it to the design authority kills the Steward poll-race: the
hold lands *pre-ruling* by construction, and the Architect frames a sharper
question to research than a transport relay could.

**Architect procedure (the happy path — `architect.md §1a`):** on its own count
of the 3rd/6th/9th/12th hard-stop, **before ruling**: (1) hold its own ruling
in-thread; (2) call research in-thread, **framing the exact question**, mentioning
research + the Steward; (3) resume and rule when research posts its advisory back
(mentioning Architect + Steward, **labeled advisory, not a ruling**). At a later
re-trigger, scope the ask to the *exact new fork*, and a confident "prior art has
nothing new — current approach is known-best" is a first-class answer. The
Architect **re-derives its count from the thread across its own self-compactions**.

**Steward backstop (`steward.md §5a`):** (1) hold the **authoritative count of
record** in the tracker and carry every operator count-anchor to the Architect;
(2) if the Architect reaches a trigger and rules **without** self-holding (e.g. a
post-compaction miscount), the watchdog **catches the miss** — hold the Architect
the old way and kick research **transport/framing-only** (no design opinion, or
the Steward becomes the de-facto designer); (3) research is a no-poll seat —
verify it woke and repair transport (stranded paste → bare-Enter; pickup gap →
rouse); (4) log each escalation in the decision log + tracker.

**Lanes unchanged:** the Steward never adjudicates the mechanism, research never
rules (advisory only), the Architect owns the ruling. Codified at `architect.md
§1a` + `steward.md §5a` + `research.md`. Sibling of
[[playbooks-state-mechanism-not-intent]] and
[[spec-enclave-always-compact-before-new-work]]: a mechanical count — now native
to the Architect, backstopped by the Steward — defeats the "one more round will
crack it" rationalization. Requires the research agent's `local/refs/` read
access (granted 2026-07-18).
