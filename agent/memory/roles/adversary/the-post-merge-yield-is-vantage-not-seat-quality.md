# The post-merge pass's yield is VANTAGE, not seat quality

**Lesson (DOC-W0, 2026-07-22).** When an adversary pass finds defects that a
careful review ring missed, the tempting read is *"the adversary seat is
stronger."* On DOC-W0 that read was available and wrong, and taking it would have
made the lesson unrepeatable by whoever holds the seat next.

## What happened

DOC-W0 ran **nine review rounds** and produced **six findings** before merge,
caught by a T1 QA seat (librarian) and the Architect. A post-merge adversary pass
then produced **two more** (findings 7–8), both real, both filed as issues. The
Steward's closure framed this as *"the strongest evidence I have for why the
adversary pass exists."*

**The seat was not the cause.** The ring was asked *"is this artifact
correct?"* — and answered it well, nine times running. Nobody was asked *"what is
this anchor **for**?"* Finding 7 (`library/REVISION` certifies nothing about the
corpus) required exactly that second question, and it is not reachable from the
first no matter how carefully you ask it.

**A merge boundary is the first moment anyone reads the thing without a candidate
in front of them.** That is most of the delta. A reviewer holds a diff and a
green suite and asks whether they agree; a post-merge reader holds neither and
can ask what the mechanism is supposed to establish.

## How to use it

- **On a hunt, ask the purpose question explicitly.** Not "is this check
  correct?" (the ring already did that, better, with the candidate in hand) but
  **"what property is this check standing in for, and does anything establish
  it?"** That is the question the vantage makes cheap and the review makes
  expensive.
- **Expect the yield to concentrate on *true* proxies.** A false proxy gets
  caught in review. A **true** statement standing in for the property that
  mattered is what ships — nine rounds converged on better and better *true
  statements about the anchor*. See the fleet
  `verify-the-mechanism-not-a-proxy` family and
  [[unbound-enumeration-is-the-shape-to-hunt-first]] if written.
- **Do not accept credit framed as seat quality.** Correct it — in the direction
  of *less* credit to the seat — so the lesson transfers. If the value is the
  vantage, then any role reading a merged artifact cold can produce it, and the
  ring can adopt the purpose question at frame time (doc-leader's own DOC-W0
  carry proposed exactly that).
- **Corollary — a ring's low defect-catch rate is not evidence of a weak ring.**
  The doc-leader's retro named the honest version: a coordination-tier leader
  routing faithfully against a T1 QA seat is the *designed* topology, not a gap.
  Don't file "the ring should also have caught this" — file the mechanism.

Related: [[a-repro-is-evidence-not-a-completion-oracle]],
`enclave-ruling-in-thread-is-not-a-durable-deliverable` (why this lesson is a
file and not a convo post).
