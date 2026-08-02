# Catalog effect and capability index

Availability: **current**

Authority: **derived reference**

## Population and disposition

This index projects the Effect/capability row from every landed catalog card.
Its population is 39 of 39 cards: 5 rows are `authored` and 34 are
`none-declared`. The result is near-empty, not empty. In particular, a package
living under `Capability/` does not contribute an entry unless its card row
does; the area name is not widened into a checked effect fact.

| Package card | Disposition | Projected result |
|---|---|---|
| [Application/CommandLine/ArgParse](Application/CommandLine/ArgParse.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Application/Configuration/Decoder](Application/Configuration/Decoder.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value; acquisition is outside this package and its entry points consume supplied `ProcessInput` or entries. |
| [Application/Input/Schema](Application/Input/Schema.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value; client acquisition and policy remain outside this package. |
| [Capability/Console/Text](Capability/Console/Text.md) | `authored` | All four checked procedures return `IO (Result IOError Unit)` and declare `visits [Console]`; failures such as broken pipes remain named result values. |
| [Capability/Diagnostics/Core](Capability/Diagnostics/Core.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Diagnostics/Render](Capability/Diagnostics/Render.md) | `none-declared` | The canonical checked fence declares no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Filesystem/Authority](Capability/Filesystem/Authority.md) | `authored` | Both checked procedures take an explicit `Cap` value, return the authority-indexed `FS` effect, and declare `visits [FS]`; neither mints or wraps a capability. |
| [Capability/Filesystem/Errors](Capability/Filesystem/Errors.md) | `none-declared` | The canonical checked fence declares no effect row, `proc`, `visits`, or capability value; the page's surrounding security-boundary prose is not substituted for a checked declaration. |
| [Capability/Filesystem/Path/Posix](Capability/Filesystem/Path/Posix.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Formatting/Doc](Capability/Formatting/Doc.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Parsing/Cursor](Capability/Parsing/Cursor.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Parsing/Decoder](Capability/Parsing/Decoder.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Parsing/Numeric](Capability/Parsing/Numeric.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Parsing/Parsing](Capability/Parsing/Parsing.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Process/Arguments](Capability/Process/Arguments.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Process/Environment](Capability/Process/Environment.md) | `none-declared` | The canonical checked fence declares no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Process/Exit](Capability/Process/Exit.md) | `none-declared` | The canonical checked fence declares no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/Process/WorkingDirectory](Capability/Process/WorkingDirectory.md) | `none-declared` | The canonical checked fence declares no effect row, `proc`, `visits`, or capability value for this package. |
| [Capability/System/Buffer](Capability/System/Buffer.md) | `authored` | The canonical checked fence consumes constructor-private `BufferSpan` and `TransferCount` boundary values while exposing no pointer, mutable reference, or producer for either carrier. |
| [Capability/System/IO](Capability/System/IO.md) | `authored` | The exact-prefix theorem consumes constructor-private `BufferSpan` and `TransferCount` values; the checked fence exposes no producer for those boundary carriers. |
| [Capability/System/Resource](Capability/System/Resource.md) | `none-declared` | The canonical checked fence declares no effect row, `proc`, `visits`, or capability value; the surrounding bracket and runtime discussion is not substituted for a checked declaration. |
| [Capability/Time/WallClock](Capability/Time/WallClock.md) | `none-declared` | The canonical checked fence declares no effect row, `proc`, `visits`, or capability value for this package. |
| [Core/Classes/EffectfulClasses](Core/Classes/EffectfulClasses.md) | `authored` | `Traversable.traverse` is declared `proc`: its abstract result constructor is fail-closed as potentially effectful. The checked `List` and `Option` implementations are ordinary `fn` values using an explicit `Applicative` dictionary. No capability value is declared. |
| [Core/Classes/LawfulClasses](Core/Classes/LawfulClasses.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Core/Classes/LawfulFunctors](Core/Classes/LawfulFunctors.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Core/Logic/EmptyDec](Core/Logic/EmptyDec.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Core/Logic/Transport](Core/Logic/Transport.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Data/Binary/BytesKeys](Data/Binary/BytesKeys.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Data/Collections/Derived](Data/Collections/Derived.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Data/Collections/Map](Data/Collections/Map.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this pure package. |
| [Data/Collections/NonEmpty](Data/Collections/NonEmpty.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Data/Numeric/Nat/Arithmetic](Data/Numeric/Nat/Arithmetic.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Data/Numeric/Nat/Order](Data/Numeric/Nat/Order.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Data/Sums/Combinators](Data/Sums/Combinators.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Data/Sums/Validation](Data/Sums/Validation.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Data/Text/Codec](Data/Text/Codec.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Data/Text/StringBijection](Data/Text/StringBijection.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Data/Text/StringKeys](Data/Text/StringKeys.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| [Tooling/Testing/Property](Tooling/Testing/Property.md) | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value; the design explicitly omits effects. |
