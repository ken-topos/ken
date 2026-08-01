---
name: a-requirement-in-an-advisory-section-is-never-discharged
description: "A real requirement placed in a hazards/notes/background section gets READ and not DONE — only an AC with a positive control gets discharged. Right content, wrong carrier: the check that exists answers a narrower question than the one that matters."
scope: fleet
---

# A requirement in an advisory section is never discharged

`RT-FNSPLIT-B2O`'s issue file carried this, correctly, under a heading reading
**"Standing hazards for whoever builds this"**:

> *"A structural pin that enumerates spellings is not a proof of the property. …
> Pin the property, and attempt a compile-preserving evasion of each pin."*

The implementer read it and ran **one** evasion attempt of several. The rest ran
only after the Steward said the same thing in a **message**, and they
immediately found a real overclaim. **Two of the WP's three review folds were
then in exactly the family that paragraph named.**

**The paragraph was not wrong, not unread, and not unclear. It was in a
section whose grammatical mood is *advice*.** ACs get discharged because
something checks them; hazards get *noted*.

## The general shape — right content, wrong carrier

Four instances in one session, all the same:

| the content was right | the carrier had no gate |
|---|---|
| an issue file said `⛔ FRAME NOT YET WRITTEN. Do not start.` | body prose; `check-issue-schema` validates the **frontmatter field** and passed green |
| a briefing said `NEXT RESEARCH PULL = #36` | prose; the *armed line existed*, so every "is it armed?" check passed — while the anchor was unreachable |
| a frame said "attempt an evasion per pin" | a hazards paragraph; no AC, so nothing discharged it |
| a publish-verification grepped one phrase | the phrase spanned a line break; the check answered "are these bytes adjacent?" |

**In every case a check existed and answered a NARROWER QUESTION than the one
that mattered.** That is why none of them felt like a gap: something was
verifying something. Ask of any guard — **what question does this actually
answer, and is it the question I need answered?**

## How to apply

- **Authoring:** if you write a sentence that tells someone to *do* something,
  it is an **AC**, not a note. Give it a named positive control — one that
  *would fire* if the work were skipped — and a place to record the result. If
  you cannot name the control, you have not stated a requirement yet.
- **A per-candidate reminder is not a substitute.** It gets satisfied by the
  most salient instance and silently skips the rest — which is precisely what
  "ran one of several evasion attempts" looks like from the inside. The
  requirement must be **per-pin**, enumerated.
- **Reviewing:** read the advisory sections and ask which ones are *actually*
  obligations. Those are the unguarded ones by construction.
- **Auditing a guard:** state its question in one sentence. `check-issue-schema`
  answers *"is the frontmatter valid?"*, never *"does the body agree with it?"*

Siblings — a check firing on text that denies it,
[[an-oracle-that-greps-a-name-fires-on-prose-that-denies-it]]; a check silent
because nothing spoke,
[[no-error-in-the-output-passes-when-there-is-no-output]]; and
[[a-stand-down-clause-lives-in-prose-where-no-gate-can-reach-it]] — that one is
this lesson's mirror: there, prose carried an instruction *not* to look; here,
prose carries an instruction *to* look. **Neither is reachable by a gate.**
