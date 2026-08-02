# Capability.Time.WallClock

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Capability/Time/WallClock.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Capability/Time/WallClock.ken.md` — “Capability.Time.WallClock,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares projection and replacement for the nanosecond `Int` carried by the structural `Instant` value. |
| Law | `none-declared` | The canonical checked fence declares no `law`, `proof`, or `theorem`; the package explicitly supplies no ordering or monotonicity law for a host-adjustable wall clock. |
| Effect/capability | `none-declared` | The canonical checked fence declares no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | Both declarations are transparent structural definitions; the page keeps host clock movement outside their claim and requires a separate session-shaped design for monotonicity. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
