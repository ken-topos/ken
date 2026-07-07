# NC11 - Checked-Core Target Closure

**Owner:** Language-led, with Runtime review. **Branch:**
`wp/NC11-checked-core-target-closure`. **Size:** L. **Risk:** high.

## Objective

Compute a deterministic checked-core closure for each selected compiler target.
The closure must include the target body, reachable declarations, metadata,
obligations, assumptions, lowerability facts, unsupported lanes, and package
dependencies needed by later erasure/runtime lowering.

## Scope

NC11 strengthens the package boundary before broader lowering starts. It answers:
"What exact checked-core facts belong to this target?"

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
