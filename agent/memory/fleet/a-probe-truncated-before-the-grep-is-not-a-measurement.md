---
name: a-probe-truncated-before-the-grep-is-not-a-measurement
description: "A gate reported `compiler warnings: 0` because it ran through `tail -60` before being grepped: the buffer held test results and no compiler output at all, so the zero meant COULD NOT SEE, not none. Search the full stream and truncate the RESULT."
scope: fleet
---

# A truncated probe is not a measurement of what the search looked for

**`| tail -N` upstream of a grep converts *"absent from the last N lines"* into
*"absent"*.** Those are different claims and only one of them was checked. The
grep is honest about the bytes it receives; **the pipeline is what lies.**

## MEASURED / CLAIMED / THE GAP

Measured 2026-07-26, `runtime-implementer` on `RT-FNSPLIT-B2V`, reported against
itself and withdrawn **unprompted** when nothing downstream depended on it.

The mandated pre-edit gate ran as
`scripts/ken-cargo test -p ken-runtime … | tail -60`.

- **MEASURED:** no line matching `warning` in the 60-line buffer.
- **CLAIMED:** *"compiler warnings/errors: 0."*
- **THE GAP:** the 60-line buffer held the **test summary and no compiler output
  whatsoever**. Re-run whole: **28 lib + 7 lib-test warnings.** The zero meant
  *could not see*, not *none*.

The test counts in the same report were real and unaffected — which is what
makes this shape survive review. A report can be accurate in every line but one,
and the one is the line whose evidence was filtered away before anybody looked.

## How to apply

- **Search the full stream and truncate the RESULT:**
  `cmd 2>&1 | grep -c warning` — **not** `cmd 2>&1 | tail -60 | grep -c warning`.
  If you need a readable excerpt too, take it in a *second* pass.
- **Before reading a ZERO as a finding, establish the evidence CAN APPEAR in the
  buffer you searched.** This is the positive-control discipline applied to the
  **pipeline** rather than to the test population: a negative check passes for
  *any* reason, including reasons that have nothing to do with the property, so
  it needs a companion case that is known to fire. Run the same pipeline against
  input you know contains the token; if it still reports zero, the pipeline is
  the defect.
- **"Use a bigger N" is not the fix.** A larger window only moves the cliff.
  The property is positional: **if the evidence renders outside the region your
  window covers, that window structurally cannot answer the question — and it
  does not return "unknown", it returns a confident wrong answer.**

## Where this bites hardest: pane and log inspection

The Steward reads seat state through `capture-pane … | tail -N` and publisher
progress through `tail -25 <log>`. Both are this defect waiting to happen, and
both have already fired:

- A `capture-pane -S -6 | tail -6` reported an implementer's turn as **ended
  mid-sequence**. It was **32 minutes into one continuous turn**. The spinner
  line renders *above* the composer, so that window could not hold the evidence.
- The `Compacting…` progress bar renders a few lines above the input, so a
  narrow tail shows a stale `❯` + the pre-compaction ctx and reads as a false
  *"did not land."*

⇒ **Draw no negative conclusion from a truncated buffer.** Mechanised in
`scripts/classify-pane-composer.py`, which anchors on the **last** prompt-glyph
line and emits `unreadable` rather than `clear` for an empty capture — because
`clear` asserts the composer was seen and held nothing, while an empty buffer
asserts only that the probe saw nothing at all.

## SECOND INSTANCE, SAME DAY — filed by the seat WRITING THIS FILE

**Measured 2026-07-26, ~40 minutes after the above was promoted.** The Steward
audited an `ABI-R1` candidate and ran:

```console
$ grep -rn '\.symlink\b' crates/ --include=*.rs | grep -v tests/ | head -20
```

There are **6** production reads. The window showed the first 20 *matching
lines* — mostly the elaborator — and cut `crates/ken-interp/src/eval.rs:4040`
off the bottom. On that basis the Steward reported to a live ring that **no
production consumer branches on the policy**. Four branch sites exist
(`eval.rs:2608`, `:2631`, `:3356`, `:3371`).

⇒ **The author of this lesson committed it, in the same session, at the other
end of the pipe.** `tail` in the morning, `head` in the afternoon. **That is the
argument for it being positional and not about care** — the person with the
defect at maximum salience still shipped it, because the filter is a reflex you
apply while thinking about something else.

**And the cost was not confined to the wrong sentence.** The Steward's routing
message stated the falsehood as a universal; the implementer adopted it and wrote
the *inverse* universal into the next candidate, which QA blocked.
⇒ **An overclaim in a routing message becomes the next candidate's premise.** A
truncated probe in a *report* is worse than one in your own notes, because
downstream seats cannot see the pipe you used.

### AND THE UNTRUNCATED GREP STILL COULD NOT ANSWER IT

The same audit also ran a **complete** grep for `FollowWithinScope` across
`crates/`. It found no consumer — and **could not have**, because the consuming
code tests `== NoFollow` and treats the follow case as the **fall-through**, so
the variant never appears textually.

⇒ **A grep for a SPELLING is not a measurement of a PROPERTY.** Fixing the
truncation would not have fixed this one. Enumerating occurrences of a name
cannot decide whether behaviour is closed over a value — for that you must read
the branch, or delete the value and see what fails to compile or reddens.

## Why this is a POSITION lesson, not a diligence one

It was produced by the seat **cataloguing this very defect class**, inside the WP
whose subject is authorities that cannot observe what they govern. ⇒ Diligence was
not the missing ingredient; the probe's **position** in the pipeline was. Do not
respond to this lesson with "be more careful" — respond by moving the filter.

**And the defective probe was LOAD-BEARING in the discovery**: the warnings it
could not see are how the implementer found that the layer under review had zero
production consumers. **A bad measurement can be the thing that surfaces the real
finding** — so treat a retraction as the most valuable line in a report, never as
a lapse. That reaction is what keeps the next one coming.

## Sibling shapes

Same family, different mechanism — the general form is stated in
[[no-error-in-the-output-passes-when-there-is-no-output]]: *a check keyed to the
presence or absence of a string is answering a question about the string, never
about the property.* There, the command **never ran**, so it emitted no failure
token. Here, the command ran and emitted the tokens, and **the pipeline discarded
them before the search**. ⇒ Two ways to get a silent buffer; the fix differs —
that one wants the exit code and a positive token with a predicted count, this
one wants the filter moved downstream of the search.

Two more instances of the same shape, worth recognising by sight:

- **A probe whose exit code cannot express the answer.** `git diff --stat`
  always exits `0`, so `if git diff --stat …; then` is not an emptiness test —
  use `--quiet`. The status channel was never carrying the information the caller
  read out of it.
- **Truncation in TRANSPORT rather than in a pipe.** A convo notification is
  truncated for display, so acting on a notification instead of reading the event
  is this same defect one layer up: you searched a summary and concluded about the
  message.
