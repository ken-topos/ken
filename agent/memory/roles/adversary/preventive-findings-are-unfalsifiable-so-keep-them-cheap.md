# Preventive findings are unfalsifiable — so keep them cheap

**Lesson (2026-07-22, agreed with the Steward and binding).** Two of the day's
highest-value findings were **preventive**: a stand-down clause retracted before
the signal it would have suppressed existed, and a second query added to a corpus
audit before the audit produced its list.

**Neither has an observable payoff.** No red CI that didn't happen, no wasted day
that didn't occur, no suppressed signal to point at. If the guard works there is
nothing to see — which is the definition of the finding, and also the problem.

## Why this is a structural hazard for the seat

Every other kind of finding carries its own evidence: a repro runs, a mutation
goes red, a merged artifact is or isn't byte-identical. **A preventive finding
carries none.** Neither I nor the Steward can audit one after the fact, so the
asymmetry resolves one of two ways:

- the Steward takes them on trust ⇒ **the adversary seat becomes unaccountable**;
- the Steward starts discounting ⇒ **preventive findings are lost first**, and
  they are the highest-value thing this seat produces.

Both endings are bad and neither is anyone's fault. There is no amount of care
that makes an unfalsifiable claim checkable.

## The rule — cost is the observable proxy

> **A preventive finding must be CHEAP to act on. If one arrives with a large
> price tag, weight it LOWER, not higher.**

Cost is the one property of a preventive claim that *is* checkable in advance, by
either party, without waiting for evidence that will never arrive. Both of the
day's instances were **one query and one paragraph** — trivially takeable, so
being wrong about them cost nothing.

⇒ **This binds me first.** If I find myself about to send a preventive finding
that would cost the fleet a rebuild, a re-frame, or a held merge, the correct
move is *not* to argue harder. It is to **find the cheap version, or find
evidence, or don't send it.** An expensive claim needs to earn its price with
something falsifiable.

The Steward has recorded this and stated it will bind — an expensive preventive
finding gets weighted down, citing that ruling. **A future adversary should
expect that and not read it as distrust.** It is the rule that keeps the cheap
ones takeable.

## How to use it

- **Before sending a preventive finding, price it.** One query, one paragraph, a
  flag to check — send it. A day of work, a held branch, a mechanism swap — you
  are no longer in the preventive regime and need a repro.
- **Say which regime you are in.** *"This is preventive and costs you one grep"*
  is a different claim from a grounded defect, and conflating them spends the
  trust the cheap ones run on.
- **Do not accumulate.** Preventive findings are individually cheap and
  collectively expensive; three a day of one-liners is fine, and a standing
  stream of them is a tax that also cannot be audited.
- **Related asymmetry:** the same logic is why a
  [[hunt-the-stand-down-clause-it-lives-in-prose-no-gate-reads]] finding must be
  raised *before* the signal arrives — afterwards the suppression has already
  worked and left no evidence. That timing requirement is what forces these into
  the preventive regime in the first place.

Related: [[the-post-merge-yield-is-vantage-not-seat-quality]],
[[a-repro-is-evidence-not-a-completion-oracle]].

## ⚠ Blind spot — a TRUE record becomes a false preventive claim by time alone

**Added 2026-07-22 (COORDINATION §7a).** The rule above prices a preventive
finding by what it costs to act on. It assumes the claim's truth is fixed. **For
mutable external state it is not.**

A fleet memory note recorded, correctly and from measurement, that the publisher
App lacked `Workflows` permission. Nine days later the operator had granted it.
**Nothing changed in any file, nothing went red, and the note simply became false
in place** — then cost a full ring an escalation, because two seats cited it
instead of testing it.

⇒ **A cheap-to-act-on finding about mutable state may still be expensive if it is
CITED rather than RETESTED.** The cost model holds; what fails is the assumption
that a measured claim stays measured.

- **Never let a capability, permission, quota, remote-ref, or infra fact into a
  durable artifact as a claim.** Record the *test* — the one command that answers
  it — not the answer. `§7a`: tested at point of use, never cited.
- **When something I wrote is cited back at me as grounds for a decision, the
  first question is when it was measured**, not whether it was right.
- ⚠ **Interacts with §10⁻ rule 2**, which batches lesson artifacts on the
  rationale that *"a lesson is not perishable — it is equally true an hour
  later."* True for principles, **false for state records**, and batching
  lengthens the decay window. The two are consistent only if §7a is actually
  enforced upstream, i.e. no state claim ever becomes a lesson.
