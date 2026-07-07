# NC13 - Core Expression Lowering

**Owner:** Runtime-led, with Language support. **Branch:**
`wp/NC13-core-expression-lowering`. **Size:** L. **Risk:** high.

## Objective

Broaden checked-core to runtime-IR lowering for ordinary core expression forms:
variables, lets, lambdas, applications, closures, captures, and direct calls.

## Scope

NC13 should make simple functional Ken programs lower through the target closure
into executable runtime IR.

In scope:

- lexical variables and environment lookup;
- non-recursive let bindings;
- function values and applications;
- closure capture and invocation;
- direct calls to selected reachable declarations;
- diagnostics for unsupported dependency shapes.

Out of scope:

- pattern matching and data constructors beyond existing seed support;
- recursion and dictionaries;
- native code broadening.

## Deliverables

- Lowering implementation for the scoped core expression set.
- Runtime-IR evaluator support or tests for the generated forms.
- Source-derived positive fixtures for closed function targets.
- Negative fixtures for unbound variables, unsupported captures, arity
  mismatches, and unresolved calls.
- Report updates naming expression-form lowerability lanes.

## Acceptance

- Source-derived closed targets using variables, lets, lambdas, applications,
  closures, and calls lower to runtime IR.
- Runtime-IR evaluation agrees with the landed interpreter for the positive
  fixtures.
- Each unsupported expression shape rejects before backend work with a stable
  lane name.
- Lowering preserves package, target, closure, obligation, assumption, and
  trusted-base metadata.

## Guardrails

- No Cranelift-specific semantics in checked-core lowering.
- No raw-source inspection after checked-core emission.
- No silent closure capture or arity fallback.
