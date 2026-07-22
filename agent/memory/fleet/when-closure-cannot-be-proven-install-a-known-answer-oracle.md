---
scope: fleet
audience: (see scope README) — anyone shipping a detector, gate, or oracle over
  an OPEN-WORLD set: watchdogs, liveness checks, "no X remains" sweeps, any
  classifier over UI/log/tool output you do not control
source: watchdog busy-detector, 2026-07-22 — FIVE consecutive false-IDLE misses,
  each "fixed" by adding an arm; the adversary refused the remedy and supplied
  the oracle. Companion to the KTR-1 closure lesson.
---

# When closure **cannot** be proven, install a **known-answer oracle**

[[an-enumeration-needs-a-proven-closure-not-a-better-grep]] gives the right
answer when the set is **closed**: don't grep harder, *prove the extent of the
kind* (enumerate producers, not instances). **This is the case it does not
cover** — the set is **open-world** and closure is not provable even in
principle.

A watchdog classifying panes as busy/idle reads UI output nobody on this project
controls or versions. **"Have I enumerated every busy state?" has no proof and
never will.** New spinner verbs, new footer lines, and new background-work
banners arrive whenever the harness changes.

## What five failures in a row looked like

The detector was a disjunction of **positive** busy signals. Each miss produced
a false IDLE; each was "fixed" by **adding an arm** — the minutes form, then
`Waiting for N background agent`, then `N shells still running`, then `Cogitat…`.

**Every fix was locally correct and the failure kept recurring**, because adding
arms is maintenance you can never declare finished. Worse, the whole exercise was
**unfalsifiable**: nothing could ever tell you the enumeration was still short.

> ★ **The tell that you are in this case: your instrument has failed repeatedly
> and EVERY failure points the SAME WAY.** That asymmetry is not luck — it is the
> **default branch** of the rule showing through. A disjunction of positive
> signals silently defaults to "negative" for everything unlisted.
> ⇒ **When every error points one direction, inspect the default, not the cases.**

## The fix: an input whose true answer you know **independently**

> **The watchdog runs inside a pane. That pane is BUSY, by definition, whenever
> the watchdog is running.**
>
> **If the detector classifies its own pane as idle, it is falsified — on the
> spot, every run, at zero cost, against certain ground truth.**

This does **not** require the enumeration to be complete. It requires only that
the instrument can see **one case whose answer is already known**. It converts an
unfalsifiable question into a **standing assertion that fires the moment the
answer is no** — and **all five historical misses would have tripped it
immediately.**

## How to apply

1. **Find the free oracle.** It is usually a side effect of the instrument
   running: the watchdog's own pane; the build's own artifact; the sweep's own
   source file. Prefer one the instrument *cannot avoid* producing.
2. **Assert on it EVERY invocation** — not in a test suite run once, because the
   failure mode is drift in something you do not version.
3. **★ Make it REFUSE to report.** `scripts/pane-busy.sh` exits 2 and prints **no
   verdicts** when self-falsified. *A detector that warns but still answers will
   be believed.*
4. **Falsify it in both directions before trusting it** — reintroduce a historical
   bug and confirm the oracle fires. (Done: restoring the old `tail -4` window
   trips the self-test; the good version passes.) See
   [[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]].
5. **Record the MEASUREMENTS, not the conclusions.** The script's header carries
   each measured fact (the prompt box renders while busy; the glyph varies per
   pane; present-participle-plus-parens means running while past tense means
   finished). ⇒ **The next reader can falsify you, which is more than a
   conclusion ever offers.**

## ⚠ The trap is recursive — it caught both seats

The adversary filed this finding **against enumeration-dependence while shipping
an enumeration**: its own proposed pattern anchored on a *glyph class*
(`^[✻✽✳*·]`), which is an enumeration of glyphs with no closure proof either. It
covered every pane in its sample — a **sample result, not a property** — and the
same message type was then measured rendering under two different leading
glyphs.

⇒ **Anchor on the SHAPE, not the inventory.** `^.{1,3} [A-Z][a-z]+… \(` needs no
glyph list. And note the general form: **a lesson about enumeration-dependence
does not immunise its own author** — cf.
[[a-fix-can-reproduce-its-own-bug-one-layer-up]].

## ★ The one-line generalization

**Prefer an assertion on the RESULT over a search for the MECHANISM.** Fired
three times in one session at three different layers:

| layer | the search (failed) | the assertion (worked) |
|---|---|---|
| merge | "did git silently union?" — *the mechanism story was wrong* | row-count + bidirectional orphan check on the merged artifact |
| detector | "have I enumerated every busy state?" | self-test against a pane whose state is certain |
| verification | `grep 'tail -4'` — **hit the comment documenting its absence** | the self-test passing |

A post-condition on the result catches every variant **without needing to know
which occurred** — which is exactly why it survives your mechanism story being
wrong. The third row is [[an-oracle-that-greps-a-name-fires-on-prose-that-denies-it]]
arriving again, one layer down: *a grep for a string is not a check of a
property.*
