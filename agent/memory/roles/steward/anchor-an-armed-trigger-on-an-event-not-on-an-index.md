---
scope: roles/steward
audience: (see scope README)
source: 2026-07-30, RT-DECL-CLOSURE-PORT — the same armed block failed the
  same way twice, first standing at `21`/`12` for three stops, then at
  `#30`/`21st entry` for four more
---

# Anchor an armed trigger on an EVENT, not on an index

`steward.md §5a`/`§5a-ii` require two armed triggers on every long ruling
chain — a research pull every 3rd hard stop, a symptom-predicate check every
3rd inventory entry. I wrote both as **indices**: *"next pull `#30` · next
predicate check = 21st entry."*

**An index-shaped trigger can only be evaluated by someone who knows the
current count — so it silently converts an armed trigger into a bookkeeping
obligation.** Miss one append and the trigger is not late, it is
**unevaluable**: nobody can tell whether `#30` has been reached. The block
recorded its own first failure (*"this line stood at `21`/`12` for three
stops"*), I re-armed it with fresh indices, and it failed identically across
four more stops the next day.

**Worse, the two indices counted different populations.** `count of record`
tallies **Architect-admitted cells**; the pull cadence counts **hard stops**.
Stated adjacently as `count 26 · next pull #30` they read as one series, so
the line answered *"not due"* while the ring — counting its own hard stops —
pulled research correctly. **A reader who trusted my line would have skipped
a trigger the ring fired.**

**How to apply:**

- **Key the trigger to a named event id:** *"next pull = the 3rd hard stop
  after `evt_5ks9da0h0977w`."* Countable from the channel alone, by anyone,
  with no history to reconstruct and nothing to keep current.
- **Never place two counters over different populations in one line.** If
  both must appear, say what each counts and that they advance at different
  rates.
- **The tell that an anchor is index-shaped: evaluating it requires a
  second lookup.** If you cannot answer "is it due?" from the trigger plus
  the channel, it is not armed.
- **A trigger that lapsed twice is a DESIGN defect, not a diligence defect.**
  Re-arming the same shape is what produced the second lapse. Change the
  shape.

Sibling of [[a-frame-can-pin-against-a-derivation-not-a-number]]: both are
about pinning to something that stays true rather than to a value that rots.
Here the rot is silent, because a stale index reads as a confident *"not
due."*
