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

These are **lessons, not law** — verify a named file/flag/function still exists
before acting on one.
