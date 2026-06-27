# Program of Work — Index

A plan to build **Ken**: a new, **MIT-licensed**, Rust-hosted, interpreter-first
**verified topos-oriented language for agentic development**, where
machine-checkable correctness — not just tests — is the deployable guarantee.

Ken is a **clean-room reimplementation**: Team Spec turns the prototype's
*behavior* into a written spec, and all implementation is done from that spec —
never from prototype source (see `../../CLEAN-ROOM.md`).

## Read in this order

0. **`../PRINCIPLES.md`** — the reasoning charter: the project's values made
   explicit (agents-write/humans-read, intrinsic-merits-not-effort, small
   auditable TCB, reflect-don't-extend, honesty about the boundary). Strategy
   and every design decision flow from it; read it first.
1. **`01-strategy.md`** — the thesis, the clean-room ground rules, the locked
   architecture (Rust host, interpreter-first, small permanent Rust kernel,
   deferred self-hosting), success criteria G1–G8, and the eight workstreams.
2. **`02-roadmap.md`** — phases 0–7, each with an objective exit gate (the phase
   narrative; the WP dependencies are refreshed in `05`).
3. **`05-implementation-dag.md`** — the **post-design-freeze** work-package DAG:
   the settled-design dependency graph (incl. the OTT kernel, the tier-1
   security workstream, and the behavioral seam) that the Steward decomposes
   into the catalog. Supersedes `02`'s dependency summary where the design
   changed it.
4. **`03-program-of-work.md`** — self-contained, team-sized work packages with
   spec sources, acceptance criteria, dependencies, and a fan-out plan across
   agent teams.
5. **`04-git-and-integration.md`** — the git workflow and coordination model:
   protected `main`, per-team PRs, a single Integrator who merges and notifies,
   and how it maps onto mootup spaces and Decisions. Realized by WP F1.

## The one-paragraph version

Ken is a small permanent **Rust trusted kernel** (the de Bruijn trust root) with
**correct universes and dependent Sigma from day one**, an **interpreter** as
reference semantics, surface **specification syntax** wired to the kernel so an
agent can state and check a correctness property, an automated prover backend
that uses Z3 as an **oracle** with a **Kripke embedding** to keep the
intuitionistic logic sound, and **legible, machine-readable proof-failure
diagnostics** (countermodels, typed holes, three-region decomposition) as the
agentic differentiator. Real types (`Int`, sum types, `Bytes`, FFI) are designed
in because they are also the self-hosting substrate. Native codegen and
self-hosting are **deferred** until the verification loop is proven. The
coalgebraic-layer, linear-types, and continuations material is real but
research-grade and kept off the critical path.

## Clean-room boundary (load-bearing)

MIT requires that Ken not be a derivative of the AGPLv3 prototype. A dedicated
**Team Spec** reads the prototype and produces a written spec + black-box
conformance corpus (WS-F/F2); implementation teams work **from the spec**, with
AGPL source kept out of their context. Math and ideas are reusable; code is not.
`mmgroup` is BSD-2 (reusable with attribution).

## Open decisions for the user (pre–Phase 1)

- **Content store**: keep any hard capacity ceiling? (Recommended: no — an
  unbounded/large content-addressed store; retain the lattice only for
  error-correction and set-bitmap roles. See ADR in WS-F/F3.)
- **Syntax**, **effect model**, **Space/process model** — ADRs in Phase 0.

## Status

Draft v2 — planning artifacts only; no Ken code written. The architecture (MIT,
clean-room, Rust, interpreter-first, small permanent kernel, deferred
self-hosting) and the name are locked; design ADRs (content store, syntax,
effect/Space model) remain open.
