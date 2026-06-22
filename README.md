# Ken

**Ken** is a verified, topos-oriented programming language for agentic
development: a language an agent can write **and prove correct**, with a small
auditable trust root and a permissive license. Machine-checkable correctness —
not just tests — is the deployable guarantee.

- **Host:** Rust. **Initial backend:** an interpreter (reference semantics);
  native codegen comes later, behind it.
- **Trust root:** a small, permanent Rust **kernel** (type theory + proof
  checker) — the de Bruijn criterion: the thing you must trust is small enough to
  audit.
- **License:** MIT (see `LICENSE`).

> **Status: pre-implementation scaffold.** No language code exists yet. This repo
> currently holds the program of work and the team workflow. Start at
> [`docs/program/`](docs/program/README.md).

## Map

- **Plan:** [`docs/program/`](docs/program/README.md) → strategy, roadmap,
  program of work, and the git/integration model.
- **Workflow:** [`CONTRIBUTING.md`](CONTRIBUTING.md),
  [`docs/program/04-git-and-integration.md`](docs/program/04-git-and-integration.md),
  and the per-role agent playbooks under [`agent/playbooks/`](agent/playbooks/).
- **Clean room:** [`CLEAN-ROOM.md`](CLEAN-ROOM.md) — Ken is a clean-room
  reimplementation; it uses the AGPLv3 "Yon" prototype only as a behavioral
  reference, never as a code basis.
- **Code:** `crates/` — `ken-kernel`, `ken-elaborator`, `ken-interp`, `ken-cli`
  (skeletons).
- **Decisions:** `docs/adr/`.

## Build

```
cargo build --workspace
```

Agents (and anyone on the shared dev box) build/test through the machine-wide
build lock instead, scoped to the touched crate:

```
source scripts/ken-env.sh           # shared sccache + registry (once per shell)
scripts/ken-cargo build -p ken-kernel
```

See [`docs/ops/compute-budget.md`](docs/ops/compute-budget.md) — the build is the
binding resource on the current hardware.

## License

MIT. Programs you write in Ken are yours under any license you choose.
