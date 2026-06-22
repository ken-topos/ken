# ADR 0001 — Foundational architecture

- **Status:** Accepted
- **Date:** 2026-06-21
- **Deciders:** the operator

## Context

Ken is a new, distinct programming language whose goal is commercial-grade
*verified* development for agents (see `../program/01-strategy.md`). It draws on the design
of an AGPLv3 research prototype ("Yon") but cannot inherit its license, name, or
accreted constraints. The architecture must make the verification loop credible
quickly, keep the trust root small, and remain implementable by parallel agent
teams.

## Decisions

1. **New language, new name: Ken.** Not a fork of the prototype.
2. **MIT license.** A clean-room reimplementation; the prototype is a behavioral
   reference only (see `CLEAN-ROOM.md`).
3. **Host language: Rust.**
4. **Interpreter-first.** The initial backend is a tree-walking/bytecode
   interpreter that defines reference semantics. Native codegen (Cranelift or
   LLVM) comes later, *behind* the interpreter and differential-tested against
   it.
5. **Small permanent Rust trusted kernel.** The type/proof checker stays small
   and in Rust forever (de Bruijn criterion). The elaborator, prover, and codegen
   build on top and may self-host later; the kernel does not.
6. **Defer self-hosting** until the language is feature-complete enough to host a
   compiler.
7. **Kernel correctness from day one.** Universe stratification is checked (no
   `Type: Type`), `Sigma` is genuinely dependent, and conversion is decidable and
   termination-certified. The prototype's known soundness gaps are not
   reproduced.

## Open (tracked separately)

- **Content store:** whether to keep any hard per-store capacity ceiling.
  *Recommendation:* no — an unbounded/large content-addressed store; retain
  lattice math only for the roles that earn it (error-correction, set bitmaps).
  To be settled in a follow-up ADR (WP F3/F4).
- **Concrete syntax, effect model, Space/process model** — follow-up ADRs.

## Consequences

- The differentiating verification loop can be demonstrated long before any heavy
  codegen or self-hosting investment.
- The trust surface is one small Rust crate (`ken-kernel`), auditable and a
  candidate for later formal verification.
- Teams can parallelize against a stable kernel API early.
