# Ken

**Ken** is a clean-room, agents-write/humans-read **verified language**: a
language an agent can write *and prove correct*, with a small auditable trust
root and a permissive license. Machine-checkable correctness — not just
tests — is the deployable guarantee.

**The thesis** ([`spec/00-overview §1`](spec/00-overview.md)): an L2 guarantee
(kernel-checked proof) delivered with L1 ergonomics — write a function, state
a property, get back a verdict an agent can act on without reading the kernel.

- **License:** MIT (see `LICENSE`). Programs you write in Ken are yours under
  any license you choose.
- **Host:** Rust, `#![forbid(unsafe_code)]`.
- **Trust root:** a small, permanent Rust **kernel** — dependent type theory
  with observational equality, a decidable conversion checker, and a proof
  checker that re-checks every certificate. The **de Bruijn criterion**: the
  thing you must trust is small enough to audit
  ([`docs/PRINCIPLES.md §5`](docs/PRINCIPLES.md)).
- **Reference semantics:** an interpreter (strict CBV); the interpreter is the
  oracle — native codegen (X3) is deferred and is correct iff it agrees.

## What's built

The implementation is underway (~28,000 lines of Rust across 4 crates,
multi-agent federation). **Bold** marks are covered by landed kernel code;
*draft* marks are in active development or have a written spec but whose
implementation has not started.

| Area | Status | What |
|---|---|---|
| **Kernel trust-root** | **built (K1→K-api)** | Π, Σ, inductive families, predicative universes, observational `Eq`/`cast`/`Ω`/quotients, decidable conversion (NbE + SCT), the TCB API |
| **Verification spine** | **built (V0→V4)** | V0 elaborator, V1 spec syntax (`requires`/`ensures`/four-way status), V2 obligation generation, V3 automated prover (IPC tactic, Z3 oracle w/ kernel-re-checked certificates), V4 diagnostics (countermodels, typed holes, three-region decomposition) |
| **Diagnostic protocol** | **built (T1)** | Machine-readable, round-trippable agent contract for diagnostics |
| **Reference interpreter** | **built (X1)** | Strict CBV evaluation, content-addressed store, effect evaluation (`drive_h`) |
| **IFC by typing** | **built (Sec1)** | Label lattice, declassification, non-interference — security tier-1 started |
| Surface language | draft | Spec drafted (§30–§39); L-stream implementation not started |
| Effects (ITree) | draft | Spec drafted (§36); L5 interaction-tree denotation landed in X1 |
| Constant-time | draft | Spec drafting (`@ct` discipline); Sec1ct WP |
| Behavioural seam | draft | Ken → Ward export; spec drafted (§70–§74) |
| Native codegen | deferred | After the verification loop is proven |
| Self-hosting | deferred | After native codegen |

## Origin

Ken's design grew out of studying **Yon**, an AGPLv3 research prototype that
showed the core ideas are buildable (content-addressed heap, dependent +
cubical kernel). Ken reuses that *design*, never its source — it is a
clean-room reimplementation and its own language from here on. See
[`CLEAN-ROOM.md`](CLEAN-ROOM.md).

## Map

- **Spec:** [`spec/`](spec/) — the language specification, the authority
  implementation teams build from. Status backbone:
  [`spec/SPEC-PROGRESS.md`](spec/SPEC-PROGRESS.md).
- **Conformance:** [`conformance/`](conformance/) — black-box test seeds
  (kernel, verification, runtime, security, surface).
- **Plan:** [`docs/program/`](docs/program/README.md) — strategy, roadmap,
  work-package DAG, and the git/integration model.
- **Decisions:** [`docs/adr/`](docs/adr/) — architecture decision records.
- **Principles:** [`docs/PRINCIPLES.md`](docs/PRINCIPLES.md) — the reasoning
  charter; when the spec does not settle a choice, reason from this.
- **Workflow:** [`CONTRIBUTING.md`](CONTRIBUTING.md) +
  [`docs/program/04-git-and-integration.md`](
  docs/program/04-git-and-integration.md)
  + per-role playbooks under [`agent/playbooks/`](agent/playbooks/).
- **Code:** [`crates/`](crates/) — `ken-kernel` (trusted kernel),
  `ken-elaborator` (elaboration + verification surface), `ken-interp`
  (reference interpreter), `ken-cli` (CLI driver).

## Build

```bash
source scripts/ken-env.sh           # shared sccache + registry (once per shell)
scripts/ken-cargo build -p ken-kernel
scripts/ken-cargo test -p ken-kernel
```

The machine-wide build lock (`scripts/ken-cargo`, default 1 slot) keeps the
shared dev box from swapping. Scope to the touched crate; full-workspace and
`--release`/LTO builds run in CI. See
[`docs/ops/compute-budget.md`](docs/ops/compute-budget.md).

## License

MIT.
