# NC22 - Broad Cranelift Lowering for Runtime IR

**Owner:** Runtime-led. **Branch:** `wp/NC22-broad-cranelift-lowering`.
**Size:** XL. **Risk:** high.

## Objective

Lower the supported NC10-NC18 runtime-IR starter subset to Cranelift for native
Ken-only executable generation.

## Scope

In scope:

- Cranelift lowering for supported runtime-IR expressions, declarations,
  closures, data constructors, records, primitive values, and traps;
- fail-closed lanes for unsupported effect, foreign, dependency, or runtime
  shapes;
- backend metadata linking lowered code to runtime-IR artifact identity;
- tests comparing lowered native fragments with runtime-IR evaluator behavior.

Out of scope:

- broad optimization;
- translation validation beyond exact tests and reports;
- library object generation;
- C/Rust interop or foreign-call execution.

## Deliverables

- Cranelift lowering for the starter executable subset.
- Positive fixtures covering the NC13-NC17 core subset.
- Negative fixtures for every NC18 unavailable effect/foreign lane.
- Backend metadata for NC26 reports.

## Acceptance

- Supported starter programs lower to native code without hand-fed runtime IR.
- Unsupported runtime-IR nodes fail before native execution with stable lanes.
- Native behavior is testable against the runtime-IR evaluator.
- The backend does not claim proof or translation validation.

## Guardrails

- Do not make Cranelift output semantic authority.
- Do not silently erase effect or foreign-boundary facts.
- Do not widen the supported subset without a fixture and report lane.
