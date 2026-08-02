# `lawful-classes` — `Eq`, `DecEq`, `Ord`

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Core/Classes/LawfulClasses.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Core/Classes/LawfulClasses.ken.md` — “`lawful-classes` — `Eq`, `DecEq`, `Ord`,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares `IsTrue`, `Eq`, `DecEq`, `Ord`, comparison helpers, and registered dictionaries for `Int`, `Bool`, `Char`, `Pair`, and `List` as supported by the checked source. |
| Law | `authored` | The class records carry equality and order laws. Checked finite-case, equality-elimination, transport, and structural proofs discharge the concrete and lifted dictionaries. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | Trust is carrier-specific: `Ord Int` retains four visible `Axiom` law fields; the named integer equality certificate is pre-existing; `Bool` adds zero trust; transported `Char` dictionaries add zero new trust; the lifted structures use checked proofs. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
