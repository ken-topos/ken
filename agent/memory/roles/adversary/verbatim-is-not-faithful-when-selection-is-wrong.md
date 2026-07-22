---
name: verbatim-is-not-faithful-when-selection-is-wrong
description: Relay safeguards govern transformation, not selection — a perfectly verbatim quote of the wrong block is invisible to every downstream reader.
metadata:
  type: feedback
---

A seat's outbound convo client failed while its inbound stayed healthy, so the
Steward relayed its QA verdict from its tmux pane. The relay was **verbatim,
explicitly attributed, and carried no added judgment** — every safeguard we have,
correctly applied. It was still wrong: it transcribed the seat's earlier short
acknowledgment instead of the **bound verdict with SHA and base** posted below it.

I spent a full analysis filing an ambiguity ("was QA's pass against `origin/main`
or against `102f4644`?") that the untranscribed block answered in its first line.

**Why:** every relay safeguard governs **transformation** of content — quote
exactly, attribute the author, add nothing. **None governs selection.** And
selection is *structurally* invisible downstream: a reader sees a faithful
quotation and has no way to tell it is the wrong paragraph, because the evidence
that would reveal it is exactly what was left out. Absence of the resolving
sentence is not detectable from the text that is present.

Note the sharper distinction I got wrong at first: the relay did not fail to
*clarify* an unclear statement — the original was complete. It **removed the
evidence that already resolved the question.** Only the second failure is
invisible to the reader, and they call for different fixes.

This is the day's dominant family on a new axis: **a true statement about a
moment, accurately quoted, standing in for a later one** — kin to
[[forecasting-a-merge-is-not-evidence-about-it]] (time) and the scope-window
cases, but here the narrowing happens in the *transport*, not in the instrument.

**How to apply:**

- **Relaying is not transport — selecting a block is authorship.** If you relay,
  confirm with the source that you have the right block. A `send-keys`
  round-trip works even when the seat's outbound posting is dead, because the
  inbound channel usually survives independently.
- **Diagnose which direction failed.** Inbound-healthy / outbound-dead is a
  dropped MCP client, not a space outage; it means the seat can still *read* and
  confirm, which is what makes the round-trip possible at all.
- **A relayed attestation is one seat reporting, not two seats agreeing** — the
  record will not distinguish them later. Put "source re-posts in its own voice"
  on the close checklist as **mandatory**, before the thread becomes the record.
- **As the adversary:** when a claim arrives through a relay and looks
  ambiguous, suspect the *selection* before you suspect the author. Ask for the
  surrounding blocks rather than reasoning harder about the words you were given
  — the resolving sentence is often one paragraph away and costs one message.
- Corollary for any evidence chain: **"quoted accurately" and "quoted
  completely" are different properties**, and only the first one is checkable
  from the quote.
