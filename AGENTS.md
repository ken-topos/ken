# Working in `ken-topos/ken`

Guidance for any Claude Code session (and any agent) operating in this repo.

## Load your role playbook first (every agent, every session)

You are **one role** in a multi-agent federation, and your operating
instructions live in a role-specific **skill**. Before doing any work — and
again **after every context compaction** — orient yourself:

1. Call **`orientation()`** (convo MCP) to learn your **role** (e.g. `steward`,
   `kernel-leader`, `spec-author`) and focus space.
2. Read **`agent/COORDINATION.md`** (federation law) and **`agent/MODELS.md`**
   (model tiers) — binding on every role.
3. **Invoke the skill for your role** (the Skill tool) and follow it as your
   standing playbook — route from your `orientation()` role:

| Your role (from `orientation()`) | Skill to load |
|---|---|
| `steward` | `ken-steward` |
| `architect` | `ken-architect` |
| `integrator` | `ken-integrator` |
| `librarian` | `ken-librarian` |
| `spec-leader` | `ken-spec-leader` |
| `spec-author` | `ken-spec-author` |
| `conformance-validator` | `ken-conformance-validator` |
| `<team>-leader` — kernel/verify/language/runtime/ergo/foundation | `ken-build-leader` |
| `<team>-implementer` | `ken-build-implementer` |
| `<team>-qa` | `ken-build-qa` |

Build-team roles share the `ken-build-*` archetype skills — your team is the
prefix on your role name (`kernel-leader` → `ken-build-leader`). The skills are
the `agent/playbooks/` corpus, surfaced as skills via `.claude/skills/` (Claude
Code) and `.agents/skills/` (Codex) — both symlink into `agent/playbooks/`;
editing a playbook edits its skill. If a team-specific overlay
exists (`agent/teams/<team>/<role>.md`), load it after the archetype skill. The
Steward owns this corpus and its routing.

**If the `Skill` tool reports your skill is unknown:** the skill registry loads
at **session start**, so a skill added or updated mid-session (e.g. you just
`git rebase`d onto a `main` that added it) is not registered for the `Skill`
tool until your next restart. Don't block on it — **`Read` the file directly at
`.claude/skills/<skill>/SKILL.md`** (or its `agent/playbooks/` target) and follow
it; it is the identical playbook. A fresh session start registers every skill
for the `Skill` tool. This makes playbook updates self-heal on rebase without a
forced restart.

## Load your memory scopes (every agent, every session)

The federation's hard-won operational lessons live in **`agent/memory/`** — a
curated, scoped corpus (see `agent/memory/README.md`). After loading your
playbook, **read the memory scopes for your role**: your `fleet` scope plus the
narrower scopes on your path (its path + ancestors).

| Your role | Memory scopes to read (the dir's files + its `README.md`) |
|---|---|
| _any role_ | `agent/memory/fleet/` |
| `steward` | `fleet` + `agent/memory/enclave/` + `agent/memory/roles/steward/` |
| `architect` | `fleet` + `enclave` + `agent/memory/roles/architect/` |
| `spec-leader` / `spec-author` / `conformance-validator` | `fleet` + `enclave` + `agent/memory/roles/<role>/` |
| `integrator` | `fleet` + `agent/memory/roles/integrator/` |
| `librarian` | `fleet` + `agent/memory/roles/librarian/` |
| `<team>-leader` | `fleet` + `agent/memory/build/` + `agent/memory/build/leaders/` + `agent/memory/teams/<team>/` |
| `<team>-implementer` | `fleet` + `build/` + `build/implementers/` + `teams/<team>/` |
| `<team>-qa` | `fleet` + `build/` + `build/qa/` + `teams/<team>/` |

These are **lessons, not law** — recall aids that reflect what was true when
written; verify a named file/flag/function still exists before acting on one.
Record a new lesson at the broadest scope where every reader must apply it; a
genuinely cross-cutting one gets a `scope:` frontmatter tag rather than a copy.
This corpus is the source of truth — Codex's generated `~/.codex/memories/` (if
ever enabled) is supplemental only, never canonical.

## Reference material is off-limits to code authors

`local/refs/` (gitignored) holds reference implementations. **Do not read them
to write Ken's code.** Per `CLEAN-ROOM.md`:

- **The AGPLv3 prototype (`yon`) is NOT mounted in this environment.** It
  is the *excluded inspiration* — Ken's design is its own; `yon` is not a
  consultable reference. There is zero AGPLv3 contact, which is strictly
  cleaner. **No agent should go looking for it.**
- **The permissive references** (Lean, Agda, cooltt, smalltt, cctt, …) may be
  **read to understand** by the Architect / Spec enclave to sharpen the spec,
  but **not copied** into the repo. Implementer agents build from `/spec`,
  never from `local/refs/`.
- **Copyleft references** (GPL/AGPL/CeCILL — e.g. smtcoq, spot, jif) are
  **Spec-enclave-only** for approach and behavior only, under the leakage
  recheck. Never consulted by implementer agents, never vendored.

When unsure whether you may look at something under `local/refs/`, the answer
is no — ask the operator or the Spec enclave.

## Conventions

- **Read `docs/PRINCIPLES.md`** — the project's reasoning charter (agents-write/
  humans-read, decide on intrinsic merits not effort, small auditable TCB,
  reflect-don't-extend, subsume-don't-proliferate, honesty about the boundary).
  When the spec does not settle a choice, reason from it.
- **Wrap markdown at 80 columns.**
- **Use Mermaid for diagrams and charts** — dependency graphs, flows, state
  machines, sequence diagrams — in fenced ` ```mermaid ` blocks, **not** ASCII
  art (it renders, diffs, and edits better). Mermaid/code fences are **exempt**
  from the 80-column rule. Keep node labels plain (avoid parentheses inside
  labels; spell out symbols like `Omega` if a renderer is finicky).
- The spec is in `spec/` (`spec/SPEC-PROGRESS.md` is the status backbone); open
  design decisions are in `spec/90-open-decisions.md`; architecture decisions in
  `docs/adr/`; the clean-room policy in `CLEAN-ROOM.md`.
- Agent-team coordination law: `agent/COORDINATION.md`. Git/merge model:
  `docs/program/04-git-and-integration.md`.
