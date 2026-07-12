# ADR 0015 — Remove the open-import form `use M`

- **Status:** Accepted (operator, 2026-07-12). Spec + conformance edit routed to
  the Spec enclave; grammar/keyword retirement is a build fast-follow.
- **Date:** 2026-07-12
- **Deciders:** the operator (2026-07-12); framed by the Architect.
- **Relates to:** `spec/30-surface/33-declarations.md` §3.2 (import forms) and
  §3.3 (name resolution); `spec/90-open-decisions.md` **OQ-syntax** (concrete
  syntax — "token table iterates with the team"); `docs/PRINCIPLES.md`
  (agents-write / humans-read; subsume-don't-proliferate; reflect-don't-extend).

## Context

§33 §3.2 offers **four** import forms:

- `import M` — qualified (`M.foo`)
- `import M as N` — aliased (`N.foo`)
- `import M (foo, Bar)` — selective: exactly those names, **unqualified**
- `use M` — open: **all** of `M`'s exports, unqualified

The first three preserve **provenance** — a reader (or `grep`) recovers any
symbol's origin from the file's import block. `use M` does not: a bare `foo` in
the body may come from any opened module, and adding an export to `M` can
silently capture or shadow a name in a downstream file that never changed. The
spec already hedges it ("use sparingly — it maximizes the ambiguity surface").

The operator questioned whether `use M` earns its place in a language whose
charter is **humans-read**, where a symbol's origin must be locally recoverable.

## Decision

**Remove `use M` from the normative surface.** Keep the three
provenance-preserving forms (`import M`, `import M as N`, `import M (…)`).

Rationale:

1. **Subsumed by selective import.** The only legitimate need `use M` serves —
   unqualified access to names — is covered by `import M (…)`, which lists the
   names once, at the top of the file, `grep`-able. `use M`'s sole delta over it
   is *not listing them*: its distinguishing feature is the provenance loss.
2. **The operator-ergonomics defense does not apply here.** *Which* instance
   satisfies a constraint is discharged by **instance resolution** (§39 §6),
   ambient and coherent — not by name-import. Common method names belong in the
   **injected prelude**. Neither path needs `use`.
3. **It costs more than it earns.** The "Open ambiguity" rule in §3.3
   (must-qualify-unless-same-decl; re-export-is-not-ambiguous) exists *solely*
   to arbitrate `use` clashes. Removing `use` collapses §3.3 to
   "qualified/aliased/selective are unambiguous by construction; local shadows
   imported." One conformance case (`use-open-ambiguity`) is retired.
4. **Reversibility asymmetry.** Adding an import form later is a **pure
   widening** (breaks no existing code); removing one later is a **breaking
   change**. With
   no persuasive champion today, *reflect-don't-extend* favors the smaller
   surface now.
5. **No in-tree dependence.** Zero `use M` occurrences in `catalog/`,
   `examples/`, or the emitted prelude (Architect swept 2026-07-12) — a clean
   removal with no
   migration. The Spec enclave confirms with the authoritative catalog sweep
   before the removal lands.

The large-domain-vocabulary steelman (a numeric tower, a DSL where enumerating a
selective import is genuinely painful) is acknowledged. If wholesale unqualified
access ever needs a real answer, the right mechanism is the **prelude** (an
injected vocabulary) or an **author-opt-in open module** (the module marks
itself wildcard-importable — the author, who knows it is a coherent vocabulary,
decides, not every consumer). Both are narrower and keep provenance answerable;
both are deferred until a concrete champion appears, and both are addable
backward-compatibly.

## Consequences

Spec + conformance edit (routed to the Spec enclave):

- **§33 §3.2** — drop the `use M` bullet; the import forms become three.
- **§33 §3.3** — remove the "Open ambiguity" rule; simplify the resolution list
  to qualified/aliased/selective (unambiguous by construction) + local-shadows-
  imported. Adjust the closing "every failure" list (drop "ambiguous open").
- **Conformance** — retire the `use-open-ambiguity` case; re-sweep the catalog
  to confirm zero `use` usage as a removal precondition.
- **Grammar / lexer / parser (build fast-follow, not this doc round)** — retire
  the `use` production and, if `use` is a reserved keyword, free it. Confirm the
  keyword's current status when the build item is framed.

**Zero kernel / `trusted_base()` delta** — this is a surface-only removal; it
narrows what the elaborator accepts and touches nothing the kernel sees.

## Alternatives considered

- **Keep `use M` (status quo).** Rejected: a standing provenance hazard against
  humans-read, carrying a §3.3 rule and a conformance case for a use case with
  no champion.
- **Author-opt-in open module.** A better answer than per-consumer `use` *if*
  the vocabulary case ever materializes, but it adds a marker mechanism —
  deferred
  until a concrete need appears (reflect-don't-extend).
- **Restrict `use` to the prelude only.** The prelude is injected, not `use`d,
  so this reduces to removal plus dead syntax.
