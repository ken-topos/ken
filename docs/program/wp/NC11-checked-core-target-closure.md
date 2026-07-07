# NC11 - Checked-Core Target Closure

**Owner:** Language-led, with Runtime review. **Branch:**
`wp/NC11-checked-core-target-closure`. **Size:** L. **Risk:** high.

## Objective

Compute a deterministic checked-core closure for each selected compiler target.
The closure must include the target body, reachable declarations, metadata,
obligations, assumptions, lowerability facts, unsupported lanes, and package
dependencies needed by later erasure/runtime lowering.

## Scope

NC11 strengthens the package boundary before broader lowering starts. It
answers: "What exact checked-core facts belong to this target?"

In scope:

- target dependency graph over checked-core declarations;
- closure identity and canonical ordering;
- preservation of obligations, assumption/trust metadata, primitive metadata,
  data/class/record metadata, lowerability, and unsupported lanes;
- diagnostics for missing bodies, foreign-only targets, unresolved references,
  and metadata gaps.

Out of scope:

- runtime-IR lowering of the closure;
- native artifacts;
- whole-program optimization.

## Deliverables

- Target-closure data model or report surface.
- Closure computation from `CheckedCorePackage v0`.
- Positive fixtures with multi-declaration target closures.
- Negative fixtures for missing dependencies, stale symbols, dropped metadata,
  unresolved imports, and non-lowerable closure members.
- Documentation of which closure facts later WPs may rely on.

## Acceptance

- The closure is recomputed from the exact package and target selector.
- Closure identity changes when reachable checked-core content changes.
- Every reachable obligation, assumption, trust entry, lowerability fact, and
  unsupported lane survives into the closure report.
- Dropping or altering closure metadata is detected by tests.
- Closure success does not claim runtime lowerability beyond the recorded facts.

## Guardrails

- No hand-maintained dependency tables as semantic evidence.
- No raw-source fallback for missing checked-core facts.
- No new kernel admission rule.
- No promotion of NC8/NC9 validation beyond their bounded surfaces.

## D1 Language Implementation Notes

The D1 implementation stays on the NC10 elaborator/compiler-driver surface:
target closure is recomputed from the validated `CheckedCorePackage v0` and the
exact selected target report. It does not change checked-core schema, kernel
admission, runtime IR, native artifact emission, or validation evidence.

Closure facts exposed to later WPs:

- `TargetClosure.semantic` is the checked-core semantic slice for the target:
  reachable declarations, reachable metadata, obligations, assumptions,
  trust-delta entries, lowerability facts, unsupported entries, and package
  dependency hashes.
- `TargetClosureReport` records package identity, package hashes, closure
  semantic hash, closure identity, reachable declaration symbols, external
  stable symbols, lowerability facts, unsupported lanes, assumptions,
  obligations, trusted-base delta, and the still-unavailable runtime-lowering
  fact.
- Undeclared stable symbols referenced by checked-core declaration bytes remain
  explicit `unresolved_checked_core_symbol` lanes. They are not treated as
  successfully lowered dependencies.
- Reachable unsupported or non-lowerable entries remain closure facts and make
  runtime lowering unavailable for the report; NC11 does not attempt erasure or
  `RuntimeProgram` construction.
