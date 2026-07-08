---
scope: fleet
audience: (see scope README)
source: private memory `thin-flow-directive-dont-cc-the-room`
---

# Thin-flow directive: one reviewer per lane, don't cc the room

Steward posted a binding federation directive after the F1 / 44-capacity-restate
/ decimal-char-demote tranche, where retros had "quietly thickened [the flow]
below the edge level" — extra parties cc'd, verbatim relays, "flagging in
parallel," multi-party ruling threads where one decider suffices, pre-confirming
what a gate already checks. Each move was locally sensible (more eyes catch
more) but collectively expensive, paid on every WP, on a serial enclave.

**The baseline (landed in COORDINATION §9/§10d):**
- **spec:** leader → author → CV (Spec) + Architect (soundness), one pass each →
  publisher path.
- **build:** leader → implementer → QA → Architect (soundness) + CV
  (conformance) → publisher path.
- **A mid-WP fork goes to the ONE owner of its lane** — soundness→Architect,
  conformance→CV, scope/process→Steward — who rules alone. Don't cc the room;
  don't relay verbatim (summarize + route to one); don't pre-confirm a gate; a
  clean ack is terminal (no re-confirm fan-in).

**How to apply (my own role, spec-leader).** This session I habitually:
mentioned both Architect AND CV on routine kickoffs where only one gate was
actually load-bearing for a given fork, posted my own "confirmed independently"
verification atop an author's already-independent re-derivation, and let
retro-collection threads run 4-deep (mine, spec-author, CV, Architect) when a
simpler ack would do. None of that was wrong per WP, but it's exactly the
pattern flagged. Going forward: route a fork to its ONE lane owner, don't
duplicate a check the gate already performs, and treat "retro landed, thanks" as
a terminal ack rather than a launch pad for a fourth reply.

**Retros: sharpen WHAT a reviewer checks, never add WHO else is looped in.** A
carry that adds a party/relay/gate/confirm-hop is topology-touching and
therefore operator-consent-only — Steward default-rejects it without that
consent. If I (or an author) am tempted to propose "and also loop in X for
this," that's a scope question for the Steward, not something to just do.

**Timing:** binds from the WP *after* the one in flight when the directive
landed (the Decimal/Char build finished on its pre-directive path). Don't
retroactively second-guess an already-in-flight WP's thicker flow — apply the
thin default to new WPs from here on.
