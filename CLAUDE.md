# Working in `ken-topos/ken`

Guidance for any Claude Code session (and any agent) operating in this repo.

## Reference material is off-limits to code authors

`local/refs/` (gitignored) holds reference implementations. **Do not read them
to write Ken's code.** Two tiers, per `CLEAN-ROOM.md`:

- **`local/refs/yon/` is the AGPLv3 prototype — clean-room critical.** Only the
  **Spec enclave** may consult it (to write `/spec` + `/conformance` in its own
  words). It is **off-limits** to implementer agents and to anyone writing Ken's
  MIT-licensed code. Never copy or close-paraphrase it.
- **The permissive references** (Lean, Agda, cooltt, smalltt, cctt, …) may be
  **read to understand** by the Architect / Spec enclave to sharpen the spec,
  but **not copied** into the repo. Implementer agents build from `/spec`, never
  from `local/refs/`.

When unsure whether you may look at something under `local/refs/`, the answer is
no — ask the operator or the Spec enclave.

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
