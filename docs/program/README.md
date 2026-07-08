# Program of Work — Index

A plan to build **Ken**: a new, **MIT-licensed**, Rust-hosted, interpreter-first
**verified topos-oriented language for agentic development**, where
machine-checkable correctness — not just tests — is the deployable guarantee.

Ken is a **clean-room reimplementation**: Team Spec authors and extends the
spec from permissive references and first principles, and all implementation is
done from that spec — never from AGPLv3 or other copyleft source (see
`../../CLEAN-ROOM.md`).

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
4. **`07-compiler-program.md`** and **`08-compiler-continuation.md`** — the
   Rust bootstrap compiler campaign, its NC1-NC9 checkpoint, and the follow-on
   campaigns for broad Ken input, native artifacts, and stronger guarantees.
5. **`03-program-of-work.md`** — self-contained, team-sized work packages with
   spec sources, acceptance criteria, dependencies, and a fan-out plan across
   agent teams.
6. **`04-git-and-integration.md`** — the git workflow and coordination model:
   protected `main`, per-team PRs, the scripted publisher path that merges and
   notifies, and how it maps onto mootup spaces and Decisions. Realized by WP F1.

**Living status:** `IMPLEMENTATION-PROGRESS.md` is the build backbone — the
Steward-owned tracker of WP/gate state against the DAG (`05`), surviving
compaction (the analog of `spec/SPEC-PROGRESS.md`). Read it to see where the
build stands now.

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

MIT requires that Ken not be a derivative of any AGPLv3 source. The dedicated
**Spec enclave** (Opus, Anthropic-hosted) authors and maintains the spec and
conformance corpus from permissive references and first principles (WS-F/F2);
implementation teams work **from the spec**, with copyleft material kept out of
their context. Math and ideas are reusable; copyleft code is not.
`mmgroup` is BSD-2 (reusable with attribution).

## Open decisions for the user (pre–Phase 1)

- **Content store**: keep any hard capacity ceiling? (Recommended: no — an
  unbounded/large content-addressed store; retain the lattice only for
  error-correction and set-bitmap roles. See ADR in WS-F/F3.)
- **Syntax**, **effect model**, **Space/process model** — ADRs in Phase 0.

## Status

Draft v2 — early implementation underway. Skeleton crates exist (`ken-kernel`,
`ken-elaborator`, `ken-interp`, `ken-cli`). Multiple work packages have shipped
(kernel core, verification spine, diagnostics protocol, effects, IFC-by-typing).
The architecture (MIT, clean-room, Rust, interpreter-first, small permanent
kernel, deferred self-hosting) and the name are locked.
