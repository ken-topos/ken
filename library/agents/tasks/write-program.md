# Write a program

## 1. Use when

Use this module to turn a bounded requirement into checked Ken source. Do not
use it for a catalog package, an unsupported platform boundary, or a request
whose contract is not yet decidable.

## 2. Prerequisites

Load `../core/write-ken.md` and `../core/toolchain.md`. Add
`../core/proof-and-trust.md` for laws and `effects-and-capabilities.md` for any
effectful boundary.

## 3. Current capability

The landed surface supports pure functions, effectful procedures, dependent
result types, explicit claims and proof terms, effect rows, capabilities, and a
checked `proc main` entrypoint.

## 4. Canonical forms

Decompose the requirement into:

```text
inputs -> result type -> laws/obligations -> pure core -> effectful shell
```

Keep pure transformation in `fn` declarations. Put real effects in a `proc`
whose `visits` row and capability supply are explicit.

## 5. Invariants and prohibitions

- Write the result contract before the body.
- Keep required facts in types or proof declarations, not only comments.
- Do not hide effects in a pure helper.
- Do not add assumptions to satisfy an implementation deadline.
- Do not guess a library name; discover packages from `catalog/packages/README.md`
  and verify the selected entry.

## 6. Decision procedure

1. Restate the requirement as inputs, outputs, laws, and allowed authority.
2. Reject ambiguity that changes the contract.
3. Probe unfamiliar syntax.
4. Implement and check the pure core.
5. Add proofs, then the smallest effectful shell.
6. Format, inspect, check, and run only if a real entrypoint exists.
7. Stop if a required operation has no supported form.

## 7. Failure signatures

A parse error points to the probe spelling; a type mismatch points to the
contract/body boundary; an escaping effect points to the row; a missing
capability points to authority supply; and an unclosed proof points to the
stated law. Diagnose the earliest failing layer.

## 8. Validation

Run `ken fmt`, inspect the result, and run `ken check`. For a runnable program,
also run `ken run` with a named input and expected observation. Validate each
law separately and record any trusted-base delta.

## 9. Authority and sources

Language forms come from `spec/30-surface/`; proof status from
`spec/20-verification/`; execution from `spec/40-runtime/`. Current authoring
practice is in `library/guide/`. Revision: `library/agents/manifest.toml`.

## 10. Known unavailable or partial behavior

Do not invent an FFI, platform primitive, package, effect, capability, or proof
when the current corpus does not supply one. If the requirement depends on an
unsupported boundary or an unproved premise, return the checked subset and an
explicit refusal for the remainder.
