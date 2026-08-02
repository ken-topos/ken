# `Sums` — the `Option`/`Result`/`Either` combinator floor

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Data/Sums/Combinators.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Data/Sums/Combinators.ken.md` — “`Sums` — the `Option`/`Result`/`Either` combinator floor,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares the neutral `Either` carrier and structural elimination, mapping, fallback, chaining, and swapping combinators across `Option`, `Result`, and `Either`. |
| Law | `authored` | Every combinator is paired with checked constructor equations, and `swap` has a checked involution proof; each reduces by direct case analysis. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | The package reports zero `trusted_base()` delta: the new carrier is positivity checked, every combinator is structural, and every proof is an ordinary term. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
