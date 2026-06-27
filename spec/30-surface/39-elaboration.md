# Elaboration: surface → core

> Status: **DRAFT v0**. Normative for *what elaboration must produce and
> guarantee*; the algorithm is specified to the level WS-L/WS-V need. Contract
> for **V0** (the minimal elaborator) and the foundation the whole surface rests
> on. Elaboration turns the surface language (`31`–`38`) into fully-explicit
> **core terms** (`../10-kernel/`). It is **untrusted**: the kernel re-checks
> everything it emits.

## 1. Role and the trust split

The elaborator is the largest, cleverest part of the front end —
implicit-argument insertion, unification, type inference, instance resolution,
`match` compilation, sugar expansion — and **none of it is trusted**. Its output
is a core term the kernel `check`s (`../10-kernel/18 §4`). Consequences:

- A bug in the elaborator yields an **ill-typed core term** the kernel
  **rejects**, or a *well-typed but unintended* term (caught by tests/specs) —
  **never** an unsound acceptance. This is why the surface can be rich and
  evolve quickly while the trusted base stays tiny (`../00-overview.md §3`).
- The elaborator MAY use unification, metavariables, heuristics, and search; the
  kernel has none of these (`../10-kernel/18 §3`). The two are deliberately
  asymmetric: cleverness outside, certainty inside.

## 2. What elaboration does

1. **Scope & resolution** — resolve names against the module environment (`33`),
   reject unbound/ambiguous references.
2. **Implicit insertion** — insert implicit arguments `{x:A}` (`32`, `33 §1`) at
   uses, creating metavariables for them.
3. **Type inference & unification** — a bidirectional, **Hindley–Milner +
   dependent** elaboration (the prototype has both; digest §7): propagate
   expected types inward (checking) and synthesize where needed (inference),
   solving metavariables by **unification** up to definitional equality
   (`../10-kernel/17`). Higher-order cases use pattern unification; genuine
   ambiguity is a reported error, not a guess.
4. **Universe/level inference** — solve level metavariables (`../10-kernel/12
   §4`), emitting explicit levels to the kernel.
5. **Instance resolution** — discharge `where C A` constraints (`33 §5`) by
   instance search, inserting the found class-record (a proof of subobject
   membership). **Canonical & coherent** (`OQ-classes`): for structure classes
   exactly one canonical instance per (class, head-type) is searchable (orphans
   are rejected at declaration, `33 §5`); search is deterministic and
   structurally bounded (`../10-kernel/17 §4`); two viable candidates is a
   surface error naming both, never a silent pick; overlap is not permitted.
   Property (Ω-valued) classes resolve to any instance — all are equal. A
   wanted-but-non-canonical dictionary is supplied by passing a named instance
   value explicitly (not via search).
6. **`match` compilation** — translate `match` (`34 §3`) into nested `elim_D`
   (`../10-kernel/14 §3`) with the recovered **dependent motive**, and run
   **exhaustiveness + reachability** checking (`34 §4`).
7. **Sugar expansion** — telescopes (`../10-kernel/13 §3`), records → Σ (`33
   §2`), `if` → `elim_Bool`, contracts/refinements → the obligation encoding
   (`../20-verification/21 §6`, `22`), `do`/comprehensions (if any) →
   combinators, numeric literals → `fromInteger`/… (`35 §4`), layout → braces
   (`31 §6`).
8. **Obligation emission** — where a refinement/contract is introduced, emit the
   proof obligation (`../20-verification/22`) and leave a hole/`prove` slot.

## 3. What elaboration must guarantee

- **Well-typed output.** Every emitted core term `check`s in the kernel; if it
  cannot produce one, it reports a precise surface error (not a kernel error).
- **No guessing past ambiguity.** Unsolved metavariables or ambiguous instances
  are surface errors with locations, never silently defaulted (except the
  *declared* defaults: numeric literals `35 §4`, level typical-ambiguity `12
  §4`).
- **Faithful sugar.** Desugaring preserves the surface's intended meaning; the
  round-trip (surface → core → behaviour) matches the surface semantics the
  chapters specify.
- **Totality routing.** Recursive definitions are emitted as eliminator
  applications where structural, else as δ-definitions gated by the kernel's SCT
  (`../10-kernel/17 §4`); a totality failure is surfaced from the kernel's
  verdict.
- **Determinism.** Same surface input → same core output (modulo metavariable
  names), so diagnostics and the protocol (`../20-verification/25`) are stable.

## 4. Errors and diagnostics

- **Surface type errors** (unification failure, unbound name, non-exhaustive
  `match`, ambiguous instance) are reported by the elaborator with source spans
  — these are *L1* errors, distinct from *L2* verification failures
  (`../20-verification/24`).
- The elaborator SHOULD recover and continue (report multiple errors, support
  the LSP), but its *accepted* output is always kernel-checked. Partial programs
  with verification holes still elaborate (the holes are obligations, `22`);
  programs with *type* errors do not (they have no well-typed core image).

## 5. V0 — the minimal elaborator (Phase 1)

For the G1 vertical slice, V0 is a **minimal** elaborator: enough surface to
`parse → elaborate → kernel-check → interpret` a trivial program and a trivial
proof — basic functions, application, a `data` type, `match`, and a literal —
with explicit-enough surface that little inference is needed. The full
inference, instances, and sugar grow in Phase 3 (WS-L). Keeping V0 minimal
de-risks the slice: it proves the *pipeline shape* before the elaborator's
complexity lands.

## 6. What WS-L/WS-V must deliver here (V0, then L-stream)

The elaborator: scope resolution, implicit insertion, bidirectional HM+dependent
inference with unification up to conversion, level inference, instance
resolution, `match`→`elim_D` with exhaustiveness, full sugar expansion, and
obligation emission — all producing kernel-checked core, with stable surface
diagnostics. **V0** delivers the minimal slice (G1); the rest grows with the
surface. Conformance: `../../conformance/surface/elaboration/` —
well-typed-output invariant (every accepted program's core image checks),
ambiguity-is-an-error cases, and `match`-exhaustiveness compilation.
