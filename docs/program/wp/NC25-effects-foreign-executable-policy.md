# NC25 - Effects and Foreign-Boundary Executable Policy

**Owner:** Runtime-led, with Security and Verify review. **Branch:**
`wp/NC25-effects-foreign-executable-policy`. **Size:** M. **Risk:** high.

## Objective

Carry NC18 effect and foreign-boundary honesty into native executable emission.
Ken-only executable generation must not accidentally execute host effects or
foreign calls.

## Scope

In scope:

- native-emission policy for runtime-IR effect and foreign-boundary facts;
- explicit unavailable or unsupported lanes for host-effect execution;
- trust-report fields for effect/foreign executable status;
- tests proving effectful or foreign-boundary targets do not silently enter
  native execution.

Out of scope:

- FFI ABI or linker support;
- host-effect implementation;
- policy-as-code enforcement beyond existing metadata;
- library import/export contracts.

## Deliverables

- Native executable policy for effect and foreign-boundary runtime IR.
- Negative fixtures for hidden foreign calls, missing authority, unsupported
  capability, and stale effect metadata.
- Report updates distinguishing represented, unavailable, and unsupported
  native effect execution.

## Acceptance

- Pure supported targets remain native-executable.
- Effectful or foreign-boundary targets either fail before native execution or
  report unavailable using exact NC18 facts.
- No report implies host effects ran unless an exact supported mechanism exists.
- No C/Rust interop surface is defined.

## Guardrails

- Do not launder effects through primitive calls.
- Do not treat unavailable effect execution as successful native codegen.
- Do not add foreign ABI design to this WP.
