# Adversary role memory scope

Lessons specific to the **Adversary** seat — the standing red-team that hunts
recent changes + their blast radius for flaws, gaps, leaky abstractions, and
undesirable behavior (see `agent/playbooks/federation/adversary.md`).

Read this scope plus `fleet/` and `enclave/` (the Adversary hunts
soundness-adjacent surfaces and reads references under the clean-room recheck, so
it sits on the enclave-facing edge). Record a lesson here when it is
Adversary-specific: a failure *shape* worth reusing across changes, a
false-alarm pattern to stop re-filing, a blast-radius-scoping trap, a
grounding/repro technique. A genuinely cross-cutting lesson belongs at the
broadest scope where every reader must apply it (much of the fleet corpus's
`verify-the-mechanism-not-a-proxy` family is exactly this), not here.

| Lesson | One-line |
|---|---|
| [a-repro-is-evidence-not-a-completion-oracle](a-repro-is-evidence-not-a-completion-oracle.md) | A repro answers *does this defect exist?* and is discharged once believed; a completion oracle answers *what would make this correct?* and binds until the fix merges — file the repro as evidence, let the ring build its own oracle, and leave the AC a weakening clause |
| [the-post-merge-yield-is-vantage-not-seat-quality](the-post-merge-yield-is-vantage-not-seat-quality.md) | When a post-merge pass finds what a careful ring missed, the cause is the VANTAGE (reading without a candidate in front of you, asking what the mechanism is *for*) not the seat — correct credit framed as seat quality, or the lesson won't transfer |

These are **lessons, not law** — verify a named file/flag/function still exists
before acting on one.
