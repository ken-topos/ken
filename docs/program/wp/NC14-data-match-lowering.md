# NC14 - Data and Match Lowering

**Owner:** Runtime-led, with Kernel and Language review. **Branch:**
`wp/NC14-data-match-lowering`. **Size:** L. **Risk:** high.

## Objective

Lower user data constructors and pattern matches from checked core to runtime IR
for the supported executable subset.

## Scope

In scope:

- constructor values and tags;
- branch selection for supported pattern-match forms;
- payload binding;
- impossible or unreachable branch representation as explicit traps where the
  checked-core evidence permits that shape;
- diagnostics for dependent or proof-only match forms that cannot be represented
  in runtime IR yet.

Out of scope:

- changing kernel coverage or positivity rules;
- proving match compilation correctness;
- broad native codegen for matches.

## Deliverables

- Runtime-IR lowering for supported data constructors and matches.
- Runtime-IR evaluator support for generated constructor/match forms.
- Positive fixtures for `Option`, `Result`, and at least one user-defined data
  type.
- Negative fixtures for stale constructor identity, missing branch, impossible
  branch misuse, unsupported dependent motive, and unsupported proof-only match.
- Report lanes for data and match lowerability.

## Acceptance

- Supported source-derived constructor/match programs lower and evaluate through
  runtime IR.
- Interpreter/runtime-IR observations agree for positive fixtures.
- Unsupported dependent or proof-only match shapes reject loudly before backend
  work.
- Constructor identity is package-bound and cannot be confused across packages.

## Guardrails

- Do not weaken kernel admission or coverage.
- Do not turn an impossible branch into arbitrary host behavior.
- Do not claim NC8 certificate coverage for context-sensitive match forms unless
  a later validator recomputes the needed facts.
