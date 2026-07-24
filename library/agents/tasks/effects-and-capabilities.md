# Effects and capabilities

## 1. Use when

Use this module to read or author an effectful boundary, effect row, handler, or
authority requirement. Do not use it for FFI/platform design or to infer that
an effect label itself grants authority.

## 2. Prerequisites

Load `../core/read-ken.md` for review or `../core/write-ken.md` for authoring.
Load `../core/toolchain.md` before claiming an effect was executed.

## 3. Current capability

The landed surface uses `proc` for effectful declarations and `visits [...]`
for explicit effect rows. The checker infers used effects and rejects an
omitted escaping effect. Capabilities are value-level authority supplied to
perform effects; handlers interpret effect trees and may provide authority.

## 4. Canonical forms

Read an effectful signature as three separate facts:

```text
proc name inputs : result visits [Effect]
effect row: what may occur
capability input or program manifest: who may authorize it
handler/runtime: how it is interpreted
```

Ground concrete spellings in a checked entry such as
`catalog/packages/Capability/Console/Text.ken.md` rather than reconstructing
them from this summary.

## 5. Invariants and prohibitions

- A row is not a capability, and a capability is not a handler.
- Every performed effect must appear in the declared/inferred row.
- No ambient authority may be assumed.
- Handling an effect removes only the handled effect from the open row.
- Do not invent an effect label, capability constructor, handler, or host
  binding.

## 6. Decision procedure

1. Inventory every operation the body may perform transitively.
2. Derive the required row from that inventory.
3. Identify the capability value or enclosing handler that supplies authority.
4. Check the signature and body together.
5. Run only with an explicit valid entrypoint and host supply.
6. Stop if any effect lacks a landed label, capability path, or supported
   driver.

## 7. Failure signatures

An escaping-effect diagnostic indicates an incomplete row; a missing-capability
diagnostic indicates absent authority; a pure-keyword mismatch indicates
`fn`/`const` was used where `proc` is required; an unknown effect at runtime
indicates a driver/handler gap after checking.

## 8. Validation

Run `ken check` first. For execution, run the same checked program with the
required capability and record the observable effect. Use a negative control
that removes one row member or authority source and require the corresponding
named failure.

## 9. Authority and sources

Normative effect rules are in `spec/30-surface/36-effects.md`; authority rules
are in `spec/60-security/62-authority.md`. Current examples live under
`catalog/packages/Capability/`. Revision: `library/agents/manifest.toml`.

## 10. Known unavailable or partial behavior

The FFI/platform surface is deliberately outside this module. Not every
specified effect or handler has a checked catalog example or a host driver on
every execution path. If no landed capability and driver exist for the
requested effect, refuse to author the boundary instead of inventing syntax or
claiming that a row alone makes it executable.
