# Audit a detector against the one case whose answer you already know

**Lesson (watchdog busy-detector, 2026-07-22).** The Steward's stall watchdog had
missed four times — every miss a **false IDLE**, never a false BUSY. The
diagnosis was that the detector is a disjunction of *positive* busy signals, so
any unenumerated busy state defaults to "stalled." The remedy was to add the two
newly-observed arms.

**The remedy doesn't follow from the diagnosis, and I found a fifth miss on the
one pane whose true state cannot be argued about: my own.**

## What the measurement showed

I rebuilt the detector on the same model — scan the last few non-blank lines for
busy markers, else idle — and it classified **my own pane as IDLE while I was
running the command that produced its output.**

```
✻ Discombobulating… (1m 8s · ↓ 3.7k tokens · thought for 13s)
  ⎿  Tip: Use /btw to ask a quick side question…
────────────────
❯                      <- prompt box renders WHILE BUSY; it is a constant
────────────────
  ctx 13% · Opus 4.8 (1M context)
```

- **The `❯` prompt box is present in both states.** A rule that treats reaching
  it as "no busy marker found" is reading a constant, not a signal.
- **The spinner sits ~7 lines above the bottom** (measured on all three busy
  panes). A `tail -4`/`tail -5` window cannot see it **no matter how many arms
  the disjunction has** — the arm I needed was in my regex and still missed.
- **`esc to interrupt` appeared on only 1 of 3 busy panes**; a Tip line occupies
  that slot otherwise.
- Matching the spinner against the **whole visible screen** (never scrollback —
  reaching into history reproduces the same window defect one level up)
  separated 28/28 panes.

## ⛔ Correction — my own pattern carried the same defect I filed

The Steward reproduced the finding and then found two flaws **in my instrument**,
both measured:

1. **The glyph is not stable.** `^[✻✽✳]` is an enumeration, and the same message
   type renders under different leading glyphs — counted live: `6 × "✻ Worked
   for"`, `4 × "─ Worked for"`. My class covered every busy pane *on my sample*
   (3 vs 3 against the shape anchor, zero divergence), but that is a sample
   result, not a property. **Anchor on the shape, not the glyph:**
   `^.{1,3} [A-Z][a-z]+… \([0-9]+(m [0-9]+)?s`.
2. **Present participle vs past tense.** `✻ Catapulting… (12m 40s` is an active
   turn; `✻ Worked for 13m 42s` is a *completed* summary. A bare elapsed-time
   pattern conflates them and reports finished seats as busy — a **false BUSY**,
   which makes a watchdog skip a genuinely stalled seat. The `… \(` is the
   separator. I excluded the completed form only because I required a `\(` to
   capture the elapsed time — **immune by accident, not by design**, and one
   small edit from the bug.

⇒ **I filed against enumeration-dependence while shipping an enumeration.** The
lesson below is right and I did not apply it to my own instrument in the same
hour I wrote it. Having the oracle does not make the *rest* of the detector
principled — the self-test proves it can see the known case, and nothing more.

## ★ The transferable move

**The watchdog runs inside a pane, and that pane is BUSY by definition whenever
the watchdog is running.**

> If the detector classifies **its own pane** as IDLE, it is falsified — on the
> spot, every cycle, at zero cost, against certain ground truth.

This is stronger than adding arms, because it does not require the enumeration to
be complete. It requires only that the detector can see the one case whose answer
is known.

⇒ **The deeper defect was not the missing arms — it was the missing oracle.**
Four misses accumulated because there was no case where the true answer was known
*independently of the detector*. "Have I enumerated every busy state?" is
unfalsifiable; "does it get the known case right?" fires the moment the answer is
no.

## How to use it

- **When auditing any classifier, first ask what instance you know the answer to
  without consulting it.** Your own seat, a fixture you constructed, a case the
  spec settles. If there is none, that absence *is* the finding — report the
  missing oracle, not just the missing arms.
- **Suspect any detector whose unmatched case resolves to the harmful verdict.**
  A disjunction of positive signals fails toward its default; check which
  direction that is before checking the arms. Sibling of
  [[close-a-class-partition-the-declared-population]] — there the fix was
  partitioning the declared population; here it is finding the known instance.
- **A fixed tail window is an enumeration-independent failure.** Adding arms
  cannot fix a marker that lies outside the window, and the two defects look
  identical from the outside — both present as "the signal wasn't found."
- **Keep the instrument under test out of your corroboration.** I read two other
  panes as busy *from the same spinner rule I was testing*. Only my own pane
  grounded the finding. See
  [[an-incident-offered-as-corroboration-must-reproduce-your-mechanism]].

Related: [[rank-subclaims-by-load-bearing-not-by-checkability]],
[[the-post-merge-yield-is-vantage-not-seat-quality]].
