# Ken language specification (`/spec`)

This directory is the **normative specification** for Ken — the MIT-licensed,
Rust-hosted, interpreter-first verified topos language. Ken's implementation
(the kernel, elaborator, prover, interpreter, stdlib) is built **against this
spec**, not against the AGPLv3 prototype. The prototype is at most a behavioral
oracle for the Spec enclave; this spec is the authority (`../CLEAN-ROOM.md`).

Start with **[`00-overview.md`](00-overview.md)** — the thesis, the system
shape, the design principles, scope/non-goals, and the glossary.

## Map

| Area | Directory | Covers |
|---|---|---|
| Overview | `00-overview.md` | Thesis, L0/L1/L2, north star, principles, scope |
| **Trusted kernel** | `10-kernel/` | Core type theory: syntax, universes, Π/Σ, inductives, identity/`J`, cubical, conversion, the typing judgment + kernel API |
| **Verification** | `20-verification/` | Spec syntax, obligation generation, the prover (Z3 + Kripke), diagnostics, the machine-readable protocol |
| Surface language | `30-surface/` | Lexer, grammar, modules, data/`match`, numbers, effects, FFI, elaboration |
| Runtime | `40-runtime/` | Value model, content-addressed heap, reference operational semantics, termination, capacity |
| Stdlib | `50-stdlib/` | Prelude and core library shape |
| **Security** (tier-1) | `60-security/` | Threat model, information-flow control, capabilities/authority, supply-chain, the trust model + honest limits |
| Open decisions | `90-open-decisions.md` | Unresolved design forks (for the operator) |

Each chapter is self-contained and cites the kernel rules it relies on.
Executable **conformance** cases live in `../conformance/` and cite the spec
section they pin.

## Reading by role

- **Kernel team (WS-K):** `10-kernel/` is your contract — start there.
- **Verify team (WS-V):** `20-verification/` (+ `10-kernel/17,18`).
- **Language team (WS-L):** `30-surface/` (+ `10-kernel/13,14` and
  `30-surface/39-elaboration.md`).
- **Runtime/execution (WS-X):** `40-runtime/`.
- **Everyone (security is tier-1):** `60-security/` — its README first; IFC
  (`61`) and authority (`62`) bind WS-L + WS-V; supply-chain (`63`) + trust
  model (`64`) bind tooling + WS-K.

## Status & conventions

This spec is being drafted incrementally. **Current status, the per-file
checklist, and the resume protocol are in
[`SPEC-PROGRESS.md`](SPEC-PROGRESS.md).** Normative keywords (MUST / SHOULD /
MAY), the `(oracle)` and `(OQ-n)` tags, and notation conventions are defined in
`00-overview.md §7`. Markdown is wrapped at 80 columns.
