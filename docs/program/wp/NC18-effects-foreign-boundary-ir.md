# NC18 - Effects and Foreign-Boundary Runtime IR

**Owner:** Runtime-led, with Security and Verify review. **Branch:**
`wp/NC18-effects-foreign-boundary-ir`. **Size:** XL. **Risk:** high.

## Objective

Represent Ken effects, capabilities, and foreign-boundary calls in runtime IR
without executing host effects natively. NC18 completes the first broad
Ken-to-Runtime-IR compiler program by making effectful targets explicit rather
than silently unsupported.

## Scope

In scope:

- runtime-IR nodes or metadata for effect operations, capability requirements,
  and foreign-boundary calls;
- explicit unsupported or deferred execution lanes for host effects;
- comparison harness behavior for pure targets and effect targets with an
  available interpreter observation;
- trust-report fields for unavailable native effect execution;
- diagnostics for missing authority, unsupported capability, foreign call, or
  unrepresented effect row.

Out of scope:

- native host-effect execution;
- FFI ABI or linker work;
- policy-as-code enforcement beyond existing language/security facts;
- claiming non-interference, constant-time, or supply-chain guarantees.

## Deliverables

- Runtime-IR representation for effect and foreign-boundary facts.
- Lowering support that records effect/capability/foreign facts from
  checked-core metadata.
- Positive fixtures for pure targets and effect-bearing targets whose effects
  are represented explicitly.
- Negative fixtures for missing capability metadata, hidden foreign call, stale
  authority identity, and unsupported effect row.
- Final NC10-NC18 program report summarizing supported vs unavailable Ken
  constructs.

## Acceptance

- Effect and foreign-boundary facts survive from checked-core target closure into
  runtime IR metadata.
- Unsupported effect execution is explicit and blocks native/backend claims.
- Pure supported targets continue to lower and evaluate through runtime IR.
- Effect-bearing targets either compare against a named interpreter observation
  source or report the comparison unavailable.
- Every report field names exact-run evidence or remains unavailable.

## Guardrails

- Do not execute host effects through the native backend in this program.
- Do not launder missing authority through compiler metadata.
- Do not claim IFC, constant-time, FFI, linker, or native artifact guarantees.
- Do not make a prose table the source of capability or foreign-boundary facts.
