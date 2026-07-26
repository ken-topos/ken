# Ken mission

Ken's mission is to make agent-written software understandable, verifiable,
and safe to deploy at commercial scale.

Ken makes contracts, proofs, authority, and assumptions first-class parts of
human-readable programs. It independently re-checks provable claims with a
small, permanent, auditable kernel, and clearly identifies everything else as
tested, delegated, or unknown.

## Why Ken exists

Agents can produce software faster than humans can review it. The scarce
resource is therefore not writing code, but understanding what the code
promises and deciding whether those promises are justified.

Ken shifts human attention from reconstructing implementations to judging
intent, specifications, trust boundaries, and evidence. Programs state what
they require and guarantee alongside what they do. The toolchain proves what
it can, reports failures in a form agents can act on, and preserves the exact
boundary of what remains unproved.

## Commitments

Ken pursues this mission by:

- optimizing its permanent source form for human reading and review;
- providing propositional correctness with software-engineering ergonomics;
- grounding proof in a small kernel that re-checks every certificate,
  independently of who or what produced it;
- making effects, capabilities, information flow, provenance, and trust
  assumptions explicit and auditable;
- keeping totality and predictability as defaults while marking partiality and
  foreign boundaries;
- separating static proof from testing, behavioral assurance, and monitoring
  without presenting one as another; and
- stating its limits honestly rather than silently weakening a guarantee.

## The boundary

Ken does not claim that verification can determine whether a specification
captures human intent. That judgment remains a human responsibility.

Ken's job is to make the specification legible, make its evidence independently
checkable, and make every remaining assumption visible. In short:

> **Agents write. Humans judge. The kernel verifies.**
