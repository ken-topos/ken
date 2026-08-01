---
scope: roles/steward
audience: (see scope README) — anyone shipping a classifier/detector/oracle
  another seat's workflow will trust
source: 2026-07-22, `scripts/pane-busy.sh` watchdog — five consecutive
  false-IDLE readings, never a false BUSY, across one session
---

# Audit a detector against the case whose answer you already know

**Currency note:** the self-test mechanism this lesson is built around
(`scripts/pane-busy.sh` testing itself against its own pane, keyed off
`MOOT_ROLE`) is **no longer present in the current script** — it shipped in
an early revision and was later simplified away. The general principle below
still applies to any live detector; if you stand up a new watchdog check or
revive a self-test on `pane-busy.sh`, re-apply this against whatever
detector is actually in use, don't assume the old mechanism is still there.

## What happened

A busy-detector produced five consecutive false-IDLE readings and never a
false BUSY. Each time it was patched by adding an arm to the disjunction —
the minutes form, `Waiting for N background agent`, `N shells still
running`, `Cogitat…`. Naming the pattern ("I do not self-test the instrument
against a case I believe is positive") did not fix it, five times in a row.
The adversary refused the next patch-an-arm remedy: *the misses aren't an
enumeration problem — there was no oracle, no case with a true answer known
independently of the detector.*

## The oracle was in the room the whole time

**The watchdog runs inside a pane. That pane is BUSY, by definition,
whenever the watchdog is running.** If the detector classifies its own pane
as idle, it is falsified — on the spot, every run, at zero cost, against
certain ground truth. That converts "have I enumerated every busy state?"
(unfalsifiable) into a standing assertion that fires the moment the answer
is no, with no need for the enumeration to be complete. All five historical
misses would have tripped it immediately.

## Two ways the fix broke its own oracle — both found within the hour

**(a) Defaulting the identity the oracle rests on.** The self-check shipped
as `self="${MOOT_ROLE:-steward}"`. The env var was unset in every
environment, including the author's, where the default happened to be
right by coincidence. For every other caller the oracle silently tested
*someone else's* pane. **The one value you must never guess is the identity
the whole oracle rests on** — ask the system for it (e.g. `tmux
display-message -p '#S'`), and refuse to run if it can't be resolved.

> ### Run it from a seat that is not the author's
>
> Three quality gates passed this defect — authoring, a both-directions
> falsification, and review — because all three ran from the one seat where
> the wrong construct produces the right answer. Both-directions
> falsification is **not** sufficient when the axis you didn't vary is the
> one carrying the defect. The fix is not "remember to also vary caller
> identity, environment, cwd, privileges" — that's another unbounded
> enumeration, the same construct that cost the five original misses, one
> level up. **The durable form is: get the artifact executed by someone who
> is not you.** A second seat varies every vantage-dependent axis at once,
> including ones neither party would have listed.

**(b) The self-test validated the disjunction, not the arms.** It proves
*at least one* arm fires, which lets an unrelated arm rot silently — in
this case, the arm covering the seat with no spinner and no timer, the most
dangerous one to interrupt. Destroying it left the self-test green while a
live seat flipped BUSY→idle undetected. **Give every arm its own positive
AND negative control** over recorded fixtures, run before any verdict. The
negative controls are load-bearing: an arm matching nothing sails through a
positives-only suite.

## How to apply

1. **Before shipping any classifier/detector/oracle, find an input whose
   true label you know independently of it** — ideally one the instrument
   produces as a side effect of running. Assert on it every invocation, not
   in a test suite run once.
2. **Make the failure refuse to report**, not just warn. A detector that
   warns but still answers will be believed.
3. **Falsify it in both directions before trusting it** — reintroduce a
   known historical bug and confirm the self-test fires; confirm the good
   version passes. See
   [[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]].
4. **Prefer the standing assertion to another arm.** More arms is
   maintenance that never gets to be declared finished; the oracle is a
   fixed cost that catches the arms you didn't think of.

## Why the errors were all one direction — the structural tell

A detector built as a disjunction of positive signals has any
un-enumerated state default to "idle" — the asymmetry isn't luck, it's the
shape of the rule. **When every failure of an instrument points the same
way, look at its default branch, not its cases.**

## Measurements worth keeping (session-specific, treat as historical)

- A narrow `tail -N` is the real window defect, not the pattern — a
  spinner can render several lines above the bottom, so a narrow tail
  cannot see it no matter how many arms the pattern has; capturing through
  the true visible bottom (not a fixed relative offset) is what fixes it.
- Present-participle-plus-parens reads as running; past tense reads as
  finished (`Catapulting… (12m 40s` vs `Worked for 13m 42s`) — a bare
  elapsed-time pattern conflating the two produces a false BUSY, which
  makes a watchdog skip a genuinely stalled seat.
- A naive pattern can match its own text when echoed back into the pane
  under test, producing a false BUSY on itself; anchoring the shape (not a
  literal substring) is immune.
- A grep hit inside a comment documenting a bug is not evidence the bug is
  fixed — what settles a fix is the self-test passing, not a string search
  for the old pattern.

The meta-lesson worth keeping over the specific script: the session's
governing lesson was *prefer a post-condition to a mechanism story*, and
this detector was built to embody it — with a guess at its center. The
lesson didn't fail generally; it was applied to the layer being looked at
and not to the plumbing holding that layer up. When you harden a mechanism,
audit its plumbing separately — it does not get the same scrutiny the
interesting part does by default.
