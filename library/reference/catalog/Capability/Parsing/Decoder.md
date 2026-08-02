# Capability.Parsing.Decoder

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Capability/Parsing/Decoder.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Capability/Parsing/Decoder.ken.md` — “Capability.Parsing.Decoder,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares location-generic decoder errors and results, the decoder function type, sequencing and token combinators, and repetition and recursive layers whose fuel derives from cursor remaining input. |
| Law | `authored` | The canonical checked fence declares progress, whole-input consumption, and reject-at-end predicates, then states the implication that progress plus end-only rejection makes `decoder_many` consume all input. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | Every combinator is transparent, structurally recursive on `Nat` fuel, and uses only checked cursor operations; the package adds no axiom or primitive. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
