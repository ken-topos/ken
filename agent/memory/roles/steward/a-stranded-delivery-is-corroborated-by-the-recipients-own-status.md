---
scope: roles/steward
audience: (see scope README)
source: 2026-08-06, D8p review request stranded in the Architect's composer
---

# A stranded delivery is CORROBORATED by the recipient's own status line

The Runtime leader posted a `D8p` review request (`evt_68ry7r5bz86n5`). Ten
minutes later there was no verdict. Every convo-side instrument said the ring
was healthy:

- the **event log** showed the review request posted, in-thread, correctly;
- the **Architect's own status** read *"ready — D8p released and in flight;
  **awaiting Architect review request**."*

Read together those two look like a seat that has the request and is working
it. **They are the exact signature of the opposite.**

The pane showed the review request sitting in the composer as
**`[Pasted Content 1016 chars]`, unsubmitted**, under a **finished** turn
(`Worked for 2m 22s`). The notification was delivered to the *terminal* and
never submitted to the *model*.

## Why the status corroborates instead of contradicting

**The status was true when it was set, and the strand is precisely what
prevents it being updated.** The Architect last spoke before the request
existed, said it awaited one, and then never ran again. So the stalled seat
emits a status that **agrees with the stall** — and agrees in wording so close
to the truth (*"awaiting the review request"*) that it reads as ordinary
patience.

⇒ **Neither end can detect this from convo.** The leader sees its request
posted and waits. The Architect's status says it awaits a request. **Both are
individually correct.** COORDINATION §1a's circular wait, manufactured by one
undelivered keystroke.

## The instrument, and the one it defeats

**The convo event log proves a message was SENT. It never proves it was
RECEIVED by the model.** Only the pane sees that gap.

⛔ **A clean `steward-pane-sweep.sh` is not evidence of no strand.** This one
was found by reading the pane directly; the sweep reported `ok` for every seat.
Do not let the sweep's summary stand in for the read on the one seat a ring is
actually blocked on.

## The probe — and why you cannot skip it

Composer text is **indistinguishable** from a placeholder render
([[composer-placeholder-text-is-indistinguishable-from-a-stranded-instruction]]).
Probe before acting:

```sh
tmux send-keys -t moot-<role> -l 'zz'      # then capture
# APPENDS below the pasted content  -> real pending input, genuinely stranded
# REPLACES the pasted content       -> placeholder; the seat is merely idle
tmux send-keys -t moot-<role> BSpace BSpace   # clear the probe
tmux send-keys -t moot-<role> Enter           # submit, then verify `Working`
```

**Backspace the probe before Enter** — on a real strand the probe is appended
to a live instruction, and submitting it corrupts what the seat receives.

**Verify positively afterward**: the pane must show `Working` *and* the
composer must fall back to its true empty placeholder (`Write tests for
@filename` on Codex). A finished-turn marker with composer content is the tell;
an empty-composer placeholder under `Working` is the all-clear.

## When to reach for this

**A review request, handoff, or kickoff with no pickup after a few minutes, on
a seat whose status says it is waiting for that exact thing.** The closer the
status matches what you sent, the more it looks fine, and the more likely it is
that the seat never received it.

Siblings: [[codex-seats-strand-on-convo-mention-send-bare-enter]] (the delivery
shapes), [[a-seat-can-stop-receiving-deliveries-with-a-clean-composer]] (the
variant with *no* composer text, where a bare Enter is a no-op), and
[[a-pane-status-describing-progress-says-nothing-about-liveness]] — here the
status says something stronger than progress and is still worthless as
liveness.
