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

The implementation is underway (~28,300 lines of Rust across 4 crates,
multi-agent federation). **Bold** marks are covered by landed kernel code;
*draft* marks have a written spec whose implementation is in progress or
not started; *deferred* marks are post-verification-loop.

| Workstream | Status | What |
|---|---|---|
| **Kernel trust-root (WS-K)** | **complete (K1→K-api)** | Π, Σ, inductive families, predicative universes, observational `Eq`/`cast`/`Ω`/quotients, W-style eliminators (K1.5), decidable conversion (NbE + SCT), the stable TCB API — the auditable trust boundary |
| **Verification spine (WS-V)** | **complete (V0→V4, T1)** | V0 elaborator, V1 spec syntax (`requires`/`ensures`/four-way status), V2 obligation generation (body-as-motive extraction), V3 automated prover (IPC tactic w/ kernel-re-checked certificates; Z3 oracle spec'd, not yet wired), V4 diagnostics (countermodels, typed holes, three-region decomposition), T1 machine-readable agent protocol |
| **Surface language (WS-L)** | **core landed (L1–L7)** | L1 numeric types (`Int`, `Decimal`, fixed-width), L2 sum types + `match` + exhaustiveness + refinements, L3 strings & collections, L5 effects (interaction-tree), L6 `Bytes` + binary I/O, L7 foreign FFI + trust boundary |
| **Security (WS-Sec)** | **tier-1 landed (Sec1, Sec1ct, Sec2)** | Sec1 IFC-by-typing (label lattice, declassification, non-interference), Sec1ct `@ct` constant-time discipline, Sec2 capabilities (PoLA, attenuation, revocation, audit) |
| **Behavioural seam (WS-B)** | **core landed (B1–B3)** | B1 assumption-boundary export emitter, B2 Temporal-as-data (Temporal Σ datatype), B3 trace/instrumentation contract; *B4 agentic boundary — in progress* |
| **Runtime (WS-X)** | **core landed (X1, X2)** | X1 strict-CBV interpreter (content-addressed store, effect evaluation), X2 runtime hardening (capacity conformance, NULL_SLOT fix) |
| Surface: L4 modules/pkg, L8 stdlib, L-fmt, typeclasses | draft | Spec drafted (§33, §39, ADR 0008); L-stream remaining |
| Native codegen (X3) | deferred | After the verification loop is proven |
| Self-hosting (S1/S2) | deferred | After native codegen |

## Origin

Ken was *inspired* by **Yon**, an AGPLv3 research prototype — but Yon is an
**excluded inspiration**: not mounted in any worktree, not consulted in any
environment, zero AGPLv3 source contact. Its core design (cubical paths/
cofibrations) was deliberately **not adopted** — Ken chose **observational
type theory** (OTT, ADR 0005) instead. Ken's design is its own; the MIT
license rests on the absence of AGPLv3 source contact. See
[`CLEAN-ROOM.md`](CLEAN-ROOM.md).

## Map

- **Spec:** [`spec/`](spec/) — the language specification, the authority
  implementation teams build from. Status backbone:
  [`spec/SPEC-PROGRESS.md`](spec/SPEC-PROGRESS.md).
- **Conformance:** [`conformance/`](conformance/) — black-box test seeds
  (kernel, verification, runtime, security, surface, behavioral).
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
