# NC1 - Checked Core Package Contract

**Owner:** Spec enclave. **Branch:** `wp/NC1-checked-core-package`.
**Size:** M. **Risk:** high: this is the compiler's semantic input boundary.

## Objective

Define `CheckedCorePackage v0`, the stable post-elaboration artifact consumed
by the Rust bootstrap compiler.

## Scope

Specify the artifact as a checked-core package, not a surface-source package.
It must include checked declarations, stable symbols, primitive references,
inductive metadata, class/instance metadata, recursion metadata, effect and
capability metadata, obligations, assumptions, trust delta, and hashes.

## Deliverables

- Normative spec text for `CheckedCorePackage v0`.
- Versioning and compatibility rules.
- A required-section table with unsupported-field semantics.
- Examples from existing checked Ken programs.
- Explicit non-goals for native layout and backend lowering.

## Acceptance

- Unsupported artifact versions reject loudly.
- Current checked examples can be represented or have explicit unsupported
  entries.
- Surface/elaborator changes have a documented preserve, bump, or translate
  path.
- The compiler never needs to read raw surface Ken to understand artifact
  meaning.

## Guardrails

No native backend implementation. No final ABI. No claim of compiler
verification. No kernel TCB expansion.
