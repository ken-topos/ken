---
scope: fleet
audience: (see scope README) — anyone authoring a universal claim, a measured
  constant, or a residual: implementers, QA, frame authors, and the Steward
  routing any of them into a durable document
source: THREE seats, same day, 2026-07-26. foundation-implementer's ABI-R1 retro
  carry (`evt_7pdp6bprr77z8`) supplied the rule; runtime-implementer's B2V
  handoff supplied the mechanism that makes it fire. Both self-reported. The
  third is the Steward's, on SPEC-STORE-SPLIT: a carried CURRENT-STATE claim
  (not a constant) told the spec enclave to close an already-DECIDED operator
  decision; caught by the ring, not the author.
---

# A scope exclusion says who may EDIT it — not that you need not VERIFY it

⛔ **Before you author a universal claim or carry a measured constant forward,
label its provenance: MEASURED HERE, or INHERITED.** An inherited universal needs
a producer/consumer closure sweep across **every** production lane. If that sweep
is not required by the deliverable or cannot be run cheaply, **omit the universal**
and state only the common grounded claim.

That is the rule. The rest of this entry is about **why it does not fire**, which
is the part neither seat knew before measuring it.

## ★★ THE MECHANISM: a do-not-touch list is read as a do-not-check list

`RT-FNSPLIT-B2V` carried a depth constant measured **before** `RT-VALUE-TOTALITY-P1`
landed. The implementer then re-anchored onto a base **containing** P1 and carried
the number across. It was low by **~4×** against P1's own bisection (9032 / 10074 /
65486 at 8 MiB); the walk now adopts at 3000, 10000 and 30000.

⭐ **Its own root cause, verbatim, and it is the reusable part:**

> *"P1 was on this WP's do-not-touch list and I read 'not mine to change' as 'not
> relevant to re-check'."*

⇒ ⛔ **Those are different questions and a scope exclusion only answers the
first.** "Out of scope to modify" and "out of scope to re-derive against" feel like
one boundary because they are written as one line in the frame. **A frame's
excluded-scope list is about EDIT AUTHORITY. It says nothing about whether your
inputs are still true at your base** — and re-anchoring onto a base that contains
the excluded work is exactly the moment those inputs go stale.

⚠ **The re-anchor is the trigger, not the original measurement.** The number was
correct when taken. Nothing about carrying it forward looks like an error, because
the operand did not change — **the base under it did.**

## ★★ AND AN HONEST, WELL-FORMED CLAIM SUPPRESSES RE-DERIVATION HARDEST

The stale constant arrived inside a residual that did **everything right**: it
named a mechanism, gave a **measured interval** rather than a point, refused to
narrow itself into a green, and routed its own fix as out-of-scope.

⇒ ⛔ **Every one of those is a virtue, and together they made the claim read as
SETTLED — so nobody re-derived it, including its own author, across a re-anchor
and a compaction.** A hedged or sloppy claim invites a check. A disciplined one
closes the question in the reader's mind. **The better the residual, the more
load-bearing its provenance label becomes**, because its quality is doing the work
a citation should be doing.

## The sibling instance — same rule, inverted direction, same day

`ABI-R1` had to correct a filesystem security-boundary paragraph and took **three**
candidates. Two were blocked for **opposite** universal claims about one mechanism:

| candidate | claimed | blocked by |
|---|---|---|
| `0c8b77fc` | the resolver **enforces** the scope's `SymlinkPolicy` | Architect |
| `f93a81bd` | resolution **does not consult** it | QA |

**Neither universal was true.** The second was supplied by the Steward from a
truncated probe, and the implementer adopted it and wrote its inverse. Its retro
named why the borrowed premise was invisible: *"I treated the routing evidence as
already closed instead of labelling it inherited, not re-derived."*

⇒ The landed prose stopped at the **true** common statement — `SymlinkPolicy` is a
carried, per-scope, two-state mechanism — and claimed nothing about enforcement in
either direction. ⭐ **Omitting the universal was the correct deliverable, not a
retreat from one.** The lane divergence became a tracked node
(`docs/program/issues/RT-SYMLINK-LANE.md`) instead of a sentence nobody could
ground.

## ★★★ A THIRD INSTANCE, AND IT SHOWS THE RULE IS NOT ONLY ABOUT NUMBERS

**2026-07-26, `SPEC-STORE-SPLIT`, the Steward — the carried input was a
CURRENT-STATE CLAIM, not a measured constant.** The frame's `AC-5` instructed the
spec enclave to *close* open decision `OQ-Space`. At the frame's own released
base, `spec/90-open-decisions.md:502` had read **`DECIDED`** for five weeks
(operator, 2026-06-27), and the summary row agreed.

⭐ **Provenance, exactly:** the campaign doc's §6.7 said *"`OQ-Space` remains
open … do not close it on the strength of this deferral."* I took that sentence
and **never opened `spec/90`.** ⇒ **An inherited premise presented as a derived
one** — and because the frame is an instruction, it propagated as an order to
overwrite an operator decision.

⛔ **The generalization that matters: "label its provenance" applies to every
load-bearing premise, not just constants and universals.** A number *looks* like
it has a source; a sentence about the current state of a document reads as
background. Both are operands. ⚠ And §6.7 was **already false when written** — a
stale carrier does not announce itself, so *"I read it in a project doc"* is a
provenance, never a verification.

⭐ **What caught it:** not me. The frame's own perishability clause told the ring
to escalate a false fixed input, and two seats reproduced the contradiction at the
released base within minutes. ⇒ **Ship that clause in every frame** — it is the
cheapest control there is for exactly this class, and it works because it invites
the reader to disbelieve you.

## How to apply

- ⛔ **Label EVERY load-bearing premise, not only numbers** — a current-state
  claim about a document ("X is open", "Y is the carrier", "no consumer remains")
  is an operand and needs *measured here at `<base>`* or *inherited from
  `<source>`*. ⭐ **The tell that you are about to skip this: the premise arrived
  as prose in a document you trust.**
- **Write the label in the artifact, not in your head.** One clause per universal
  or constant: *measured here at `<base>`* / *inherited from `<source>`, not
  re-derived*. An unlabelled number is indistinguishable from a measured one, and
  a coordinator cannot tell your guess from your measurement.
- ⛔ **Re-anchoring is a provenance event.** When you rebase or re-anchor onto a
  base containing work your frame excluded, **re-derive every constant and every
  universal that touches it.** The exclusion protected you from editing it; it
  never protected your inputs.
- **If the closure sweep is not affordable, delete the universal.** The common
  grounded claim plus a tracked node beats a lane-universal that is false in one
  lane, and it is a *smaller* deliverable, not a weaker one.
- ⚠ **When you receive a claim whose form is impressive — named mechanism,
  measured interval, self-limited scope — that is when to ask which operand was
  inherited.** Its polish is not evidence about its currency.

## Positioning

- [[agreement-is-not-corroboration-when-a-premise-was-inherited]] — the *receiving*
  side: two seats agree because one re-used the other's operand. **This entry is
  the authoring side**, and it names the boundary (excluded scope) that makes an
  operand feel unavailable to re-check.
- [[a-probe-truncated-before-the-grep-is-not-a-measurement]] — how the `ABI-R1`
  universal was manufactured in the first place. That one is about producing a bad
  input; this one is about *carrying* it.
- [[an-enumeration-needs-a-proven-closure-not-a-better-grep]] — a producer/consumer
  closure sweep is the mechanism this rule demands, and it has its own failure
  mode: an unstated domain.
- [[deriving-from-the-contract-cannot-detect-a-defective-contract]] — a layer can
  be correct against an input that was already wrong.
