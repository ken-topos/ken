# `transport` — `subst`, `cong`, `cast`, `sym`, `trans`

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Core/Logic/Transport.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Core/Logic/Transport.ken.md` — “`transport` — `subst`, `cong`, `cast`, `sym`, `trans`,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Exports `subst`, `cong`, `cast`, `sym`, and `trans`: five non-recursive wrappers over the surface equality eliminator `J` and native equality `Eq`. |
| Law | `authored` | `cong`, `sym`, and `trans` are theorem declarations whose checked bodies prove the properties they name; the checked `sym_trans_compose` example exercises their composition. No additional internal law is declared. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | All five public names are ordinary uses of `J`, `Eq`, and equality reduction. They add zero trusted-base delta, use no recursion, and introduce no eliminator or reduction rule. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
