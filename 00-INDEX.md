# Program of Work — Index

A plan to build **Ken**: a new, **MIT-licensed**, Rust-hosted, interpreter-first
**verified topos-oriented language for agentic development**, where
machine-checkable correctness — not just tests — is the deployable guarantee.

Ken is a **clean-room reimplementation**. The AGPLv3 Yon fork (this repo's
sibling at `../yon`) and the exploratory
`../yon/Yon Programming Language Analysis.md` are **reference prototype + idea map
only** — studied, specified from, and tested against, but never copied. The
analysis was written from public 1.0 docs and is factually unreliable about the
prototype; the reality check corrects it against source.

## Read in this order

1. **`01-reality-check.md`** — what the prototype actually does, where the
   analysis is wrong (the f64 thesis is refuted; the prototype kernel is already
   partway to L2), evidence-backed. A **knowledge artifact**, not implementation
   material (clean-room note inside). **Start here.**
2. **`02-strategy.md`** — the thesis, the clean-room ground rules, the locked
   architecture (Rust host, interpreter-first, small permanent Rust kernel,
   deferred self-hosting), success criteria G1–G8, and the eight workstreams.
3. **`03-roadmap.md`** — phases 0–7, each with an objective exit gate; the
   work-package dependency graph.
4. **`04-program-of-work.md`** — self-contained, team-sized work packages with
   spec sources, acceptance criteria, dependencies, and a fan-out plan across
   agent teams.
5. **`05-git-and-integration.md`** — the git workflow and coordination model:
   protected `main`, per-team PRs, a single Integrator who merges and notifies,
   and how it maps onto convo/mootup spaces and Decisions. Realized by WP F1.

## The one-paragraph version

The Yon prototype proved the hard ideas are buildable — a computing dependent +
cubical kernel, content-addressing, mechanical Yoneda checking — but it is
AGPLv3, named, and carries soundness debt (unchecked universes, non-dependent
Sigma) and accreted constraints (a hard Λ₂₄ slot ceiling, a float-only numeric
model) that a commercial language should not inherit. Ken takes the validated
*design*, not the code: a small permanent **Rust trusted kernel** (the de Bruijn
trust root) with **correct universes and dependent Sigma from day one**, an
**interpreter** as reference semantics, surface **specification syntax** wired to
the kernel so an agent can state and check a correctness property, an automated
prover backend that uses Z3 as an **oracle** with a **Kripke embedding** to keep
the intuitionistic logic sound, and **legible, machine-readable proof-failure
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
`mmgroup` is BSD-2 (reusable with attribution). Confirm with IP counsel before
Phase 1. (Not legal advice.)

## Open decisions for the user (pre–Phase 1)

- **Content store**: keep any hard capacity ceiling? (Recommended: no — an
  unbounded/large content-addressed store; retain the lattice only for
  error-correction and set-bitmap roles. See ADR in WS-F/F3.)
- **Syntax**, **effect model**, **Space/process model** — ADRs in Phase 0.
- **Name**: decided — **Ken**. A trademark / registry (crates.io) sanity-check is
  still advisable before any public launch.

## Status

Draft v2 (reframed from fork-enhancement to greenfield new language after the
2026-06-21 decisions: name **Ken**, MIT, clean-room, Rust interpreter-first,
small permanent kernel, deferred self-hosting). Planning artifacts only; no Ken
code written. Architecture decisions locked and the name chosen; design ADRs
remain open.
