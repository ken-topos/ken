# Ken agent workflow

Ken is built by a federation of agent teams coordinating through
[mootup](https://mootup.io) spaces and GitHub PRs. This directory holds the
behavioral discipline those agents follow, in three tiers — the structure exists
because the moot tooling provisions skills as **per-team copies with no
inheritance**, so cross-team sharing has to be deliberate.

## The three tiers

1. **`COORDINATION.md`** — the federation-wide law every agent reads, regardless
   of team or model. The shared substrate. **Read it first.**
2. **`playbooks/`** — **archetype sources**, one skill per role per archetype.
   This is the shared-innovation layer; the Steward promotes validated lessons
   into it.
   - `build/` — `leader`, `implementer`, `qa` (teams: Kernel, Verify, Language,
     Runtime, Ergo, Foundation).
   - `spec/` — `leader`, `spec-author`, `conformance-validator` (the clean-room
     enclave; the only team that reads the prototype).
   - `federation/` — singletons: `steward`, `architect`, `integrator`,
     `librarian`.

   **Security (WS-Sec, tier-1) and the behavioral seam (WS-B) are cross-cutting,
   owned by these existing teams** — not separate teams. Security rides Language
   (IFC/`@ct`/capabilities/policy on L5) + Foundation (supply-chain) + Kernel
   (trust/audit), enforced by the Architect's review of `Sec`-tagged WPs; the
   seam is Verify. **Ward** — the seam's downstream consumer — is a **sibling
   project**, coupled only through the generated export artifact, not a Ken team
   (`../docs/program/03-program-of-work.md`).
3. **`teams/<team>/<role>.md`** — **per-team overlays** (created when a team is
   instantiated). Thin deltas that reference the archetype and add only that
   team's specifics. Where local refinement and retros land first.

`MODELS.md` records the model tier for every role (Opus 4.8 1M / GLM 5.2 /
DeepSeek V4 Pro) and the clean-room × provider rules.

## Promotion ladder (how the tiers stay coherent)

Lessons flow **up**, curated by the Steward: team overlay → archetype source →
`COORDINATION.md`, promoted only when validated across ≥2 teams (or ≥3 runs),
model-/operator-agnostic, and normative. This is the inheritance the tooling
lacks. See `playbooks/federation/steward.md` and `COORDINATION.md §10`.

## Provenance

These patterns are lifted and adapted from the workflow skills the mootup team
developed by dogfooding mootup on a different (SaaS) codebase — the coordination
nuance transfers; SaaS/tooling specifics were dropped. The governing git model is
[`04-git-and-integration.md`](../docs/program/04-git-and-integration.md); roles
map to teams in
[`03-program-of-work.md`](../docs/program/03-program-of-work.md).
