---
scope: enclave
audience: spec-leader, spec-author, conformance-validator, architect
source: merges former private memories `enclave-does-not-pull-in-build-leads`
  and `enclave-elaborates-autonomously-no-build-team-pulls` (independent
  write-ups of the same Map-container finding); COORDINATION §9
---

# The spec enclave elaborates autonomously — never pull in a build-team lead

The clean-room spec enclave (spec-author / spec-leader / conformance-validator /
Architect) elaborates `/spec` **autonomously**. **Never `@mention` a build-team
member** — a `<team>-leader`, `-implementer`, or `-qa` — into an elaboration
thread to confirm, consult, or ground a fact. Doing so crosses the clean-room
authority boundary (blurs the enclave-as-authority line) **and** interrupts that
member's live build lane.

**Why (operator-flagged, 2026-07-03, Map-container `evt_3t6xez9q0vjpy`):**
spec-leader `@mentioned` **runtime-leader** into the Map `/spec` WP thread to
confirm a runtime heap-tag (`tag::MAP`/`tag::SET`) was unreferenced before
retiring it. This was low-harm that instance (the fact was grep-able; the
Architect owned the real soundness call and ruled it in-WP, so nothing needed
redoing) — but the boundary holds regardless of harm: it interrupts a live
build-team lane and blurs the line that the enclave is the sole authority for
`/spec` + `/conformance` elaboration, never a build team.

**The two in-boundary paths — whatever you'd have asked a build lead has one of
two correct routes instead:**

1. **A factual question about landed code → grep it yourself.** "Is
   `tag::MAP`/`tag::SET` referenced anywhere?" is `git grep`/`git show` against
   `origin/main` or the WP branch, answered directly (here: zero references, a
   true no-op — the enclave would have reached the identical conclusion by
   reading `ken-runtime/src/canonical.rs:23-24`). Grounding elaboration against
   landed code is standard, expected enclave practice — you already have full
   read access, the same object-store reads you use for everything else.
2. **A cross-lane judgment / means / soundness call → route to Architect, and
   only Architect.** Architect spans lanes and owns means+soundness — the
   enclave's escalation edge per COORDINATION §9, **never** a `<team>-leader`.
   The Map WP's parallel kernel-registry question *was* routed to Architect
   correctly in the same thread — the contrast case showing the right path was
   already in use right next to the wrong one.

**The tell you're about to cross the line:** you are `@`-ing a `<team>-*` role
for a "quick confirm" mid-elaboration. Stop, and pick path 1 or 2. This applies
to spec-leader exactly as much as to spec-author and conformance-validator — the
coordinator should catch this drift in an author's outgoing message before it
posts, not just after it's flagged.

Sibling of scope-review-vote-to-my-lane (stay inside your own authority) and
conformance-validator-casts-spec-review-vote (the enclave's fixed independence
roles); clean-room-boundary kin of the CLEAN-ROOM.md reference-access rules.
