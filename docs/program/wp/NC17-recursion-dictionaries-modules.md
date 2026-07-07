# NC17 - Recursion, Dictionaries, and Modules

**Owner:** Language/Runtime joint. **Branch:**
`wp/NC17-recursion-dictionaries-modules`. **Size:** XL. **Risk:** high.

## Objective

Support whole-package executable targets that rely on recursive declarations,
typeclass dictionaries, module/package references, and imported checked-core
dependencies.

## Scope

In scope:

- recursive declaration groups that are already accepted by the kernel and
  elaborator;
- call-graph representation in target closures and runtime IR;
- dictionary/class evidence as runtime records only when the selected fields are
  executable;
- package/module reference identity across checked-core dependencies;
- diagnostics for recursive or dictionary shapes that cannot be lowered yet.

Out of scope:

- changing termination or guardedness rules;
- adding orphan or coherence behavior;
- dynamic package loading;
- native linking.

## Deliverables

- Closure and lowering support for supported recursion and imported references.
- Runtime-IR representation for supported recursive calls.
- Dictionary lowering for executable class fields.
- Positive fixtures over package/module boundaries and class-using programs.
- Negative fixtures for unsupported recursion, non-executable dictionary field,
  stale import identity, missing package dependency, and coherence violation
  attempts.

## Acceptance

- Supported recursive and dictionary-using source-derived targets lower and
  evaluate through runtime IR.
- Package and module references are bound to exact checked-core dependency
  identities.
- Non-executable proof/law dictionary fields stay erased and report-visible.
- Unsupported recursion and dictionary shapes reject before backend work.

## Guardrails

- Do not weaken termination, coherence, orphan, or import checks.
- Do not treat a dictionary as runtime data unless its fields are explicitly
  runtime fields.
- Do not make module/package path strings semantic authority over checked-core
  dependency identity.
