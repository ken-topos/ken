# Catalog law index

Availability: **current**

Authority: **derived reference**

## Population and disposition

This index projects the Law row from every landed catalog card. Its population
is 39 of 39 cards: 27 rows are `authored` and 12 are `none-declared`. A
`none-declared` entry means exactly that the cited card records no law in its
canonical checked fences; it is not an inference from an omitted index entry.

| Package card | Disposition | Projected result |
|---|---|---|
| [Application/CommandLine/ArgParse](Application/CommandLine/ArgParse.md) | `none-declared` | The canonical checked fences declare no `law`, `proof`, or `theorem` for this package. |
| [Application/Configuration/Decoder](Application/Configuration/Decoder.md) | `none-declared` | The canonical checked fences declare no `law`, `proof`, or `theorem` for this package. |
| [Application/Input/Schema](Application/Input/Schema.md) | `none-declared` | The canonical checked fences declare no `law`, `proof`, or `theorem` for this package. |
| [Capability/Console/Text](Capability/Console/Text.md) | `none-declared` | The canonical checked fence declares no `law`, `proof`, or `theorem` for this package. |
| [Capability/Diagnostics/Core](Capability/Diagnostics/Core.md) | `none-declared` | The canonical checked fences declare no `law`, `proof`, or `theorem` for this package. |
| [Capability/Diagnostics/Render](Capability/Diagnostics/Render.md) | `none-declared` | The canonical checked fence declares no `law`, `proof`, or `theorem` for this package. |
| [Capability/Filesystem/Authority](Capability/Filesystem/Authority.md) | `none-declared` | The canonical checked fence declares no `law`, `proof`, or `theorem` for this package. |
| [Capability/Filesystem/Errors](Capability/Filesystem/Errors.md) | `none-declared` | The canonical checked fence declares no `law`, `proof`, or `theorem` for this package. |
| [Capability/Filesystem/Path/Posix](Capability/Filesystem/Path/Posix.md) | `authored` | The canonical checked fences prove parse/render closure for valid paths, validity preservation, normalization, idempotence, and removal of dot segments and absolute dot-dot segments. |
| [Capability/Formatting/Doc](Capability/Formatting/Doc.md) | `authored` | Checked theorems and attached proofs establish content preservation through layout choices, width independence of text tokens, and the render fixed point for an inert `Text` leaf. |
| [Capability/Parsing/Cursor](Capability/Parsing/Cursor.md) | `authored` | Three checked theorems prove argument-origin index and endpoint fidelity; the canonical fence also declares the cursor obligations for successful peek, advancing progress, and valid end positions. |
| [Capability/Parsing/Decoder](Capability/Parsing/Decoder.md) | `authored` | The canonical checked fence declares progress, whole-input consumption, and reject-at-end predicates, then states the implication that progress plus end-only rejection makes `decoder_many` consume all input. |
| [Capability/Parsing/Numeric](Capability/Parsing/Numeric.md) | `authored` | Checked origin-fidelity theorems preserve argument index and endpoints, the digit carrier attaches a validity proof, and `format_digits_roundtrip` proves structural parse/format recovery. |
| [Capability/Parsing/Parsing](Capability/Parsing/Parsing.md) | `authored` | Checked proofs establish source-byte UTF-8 evidence, span-to-origin fidelity, reflexive and zero-left bounds, and valid zero-width spans; parser laws remain explicit predicates rather than postulates. |
| [Capability/Process/Arguments](Capability/Process/Arguments.md) | `authored` | The checked `round_trip` proof shows that projecting arguments after replacement returns the replacement list. |
| [Capability/Process/Environment](Capability/Process/Environment.md) | `authored` | The checked `round_trip` proof shows that projecting the environment after replacement returns the replacement list. |
| [Capability/Process/Exit](Capability/Process/Exit.md) | `none-declared` | The canonical checked fence declares no `law`, `proof`, or `theorem` for this package. |
| [Capability/Process/WorkingDirectory](Capability/Process/WorkingDirectory.md) | `authored` | The checked `round_trip` proof shows that projecting the working directory after replacement returns the replacement bytes. |
| [Capability/System/Buffer](Capability/System/Buffer.md) | `authored` | Checked theorems expose the positivity proposition and the structural equation that splits a request budget into transferred and remaining counts. |
| [Capability/System/IO](Capability/System/IO.md) | `authored` | All five named theorems have checked proof terms in the canonical fence, covering termination, exact-prefix preservation, success completeness, and error behavior. |
| [Capability/System/Resource](Capability/System/Resource.md) | `none-declared` | The canonical checked fence declares no `law`, `proof`, or `theorem` for this package. |
| [Capability/Time/WallClock](Capability/Time/WallClock.md) | `none-declared` | The canonical checked fence declares no `law`, `proof`, or `theorem`; the package explicitly supplies no ordering or monotonicity law for a host-adjustable wall clock. |
| [Core/Classes/EffectfulClasses](Core/Classes/EffectfulClasses.md) | `authored` | Class fields state the applicative and monad laws; checked theorems prove the `Option` and `List` instances. The traversal section proves identity, naturality, and composition, including the composed-applicative support laws. |
| [Core/Classes/LawfulClasses](Core/Classes/LawfulClasses.md) | `authored` | The class records carry equality and order laws. Checked finite-case, equality-elimination, transport, and structural proofs discharge the concrete and lifted dictionaries. |
| [Core/Classes/LawfulFunctors](Core/Classes/LawfulFunctors.md) | `authored` | Associativity, unit, functor identity/fusion, and fold coherence are class fields with checked witnesses: structural induction for `List`, finite cases for `Bool`, and case splitting or reduction for `Option`. |
| [Core/Logic/EmptyDec](Core/Logic/EmptyDec.md) | `authored` | `yes_is_true` and `no_is_false` check `decide`'s two computation facts; the local `DecEq` record carries its soundness and completeness contract. |
| [Core/Logic/Transport](Core/Logic/Transport.md) | `authored` | `cong`, `sym`, and `trans` are theorem declarations whose checked bodies prove the properties they name; the checked `sym_trans_compose` example exercises their composition. No additional internal law is declared. |
| [Data/Binary/BytesKeys](Data/Binary/BytesKeys.md) | `authored` | Checked injectivity, soundness, and completeness theorems discharge both dictionaries. |
| [Data/Collections/Derived](Data/Collections/Derived.md) | `authored` | The canonical fences contain checked structural, length, sorting, projection, and round-trip-derived proofs, including list decomposition and verified `List Bool` permutation/sortedness. |
| [Data/Collections/Map](Data/Collections/Map.md) | `authored` | Checked proofs cover order preservation, lookup after insertion and locality, ordered traversal, associative-list agreement, keyed-operation characterizations, and set algebra. |
| [Data/Collections/NonEmpty](Data/Collections/NonEmpty.md) | `authored` | A checked three-value structural proof lifts list-append associativity to `nonempty_append` and inhabits the semigroup law. |
| [Data/Numeric/Nat/Arithmetic](Data/Numeric/Nat/Arithmetic.md) | `authored` | Checked attached proofs establish zero, successor, identity, commutativity, associativity, and left/right distributivity laws for addition and multiplication. |
| [Data/Numeric/Nat/Order](Data/Numeric/Nat/Order.md) | `authored` | Checked structural proofs establish reflexivity, antisymmetry, transitivity, and totality; the dictionary fields are inhabited by those proofs. |
| [Data/Sums/Combinators](Data/Sums/Combinators.md) | `authored` | Every combinator is paired with checked constructor equations, and `swap` has a checked involution proof; each reduces by direct case analysis. |
| [Data/Sums/Validation](Data/Sums/Validation.md) | `authored` | Checked proofs establish functor identity/fusion and applicative identity, homomorphism, interchange, composition, and map coherence. |
| [Data/Text/Codec](Data/Text/Codec.md) | `authored` | Checked proofs expose the decode definition, preserve absent and present `ascii_view` cases, and carry the existing one-way `BytesRoundTripLaw` without strengthening it. |
| [Data/Text/StringBijection](Data/Text/StringBijection.md) | `authored` | The checked injectivity theorem follows from the named retraction using symmetry, congruence, and transitivity. |
| [Data/Text/StringKeys](Data/Text/StringKeys.md) | `authored` | Checked soundness, completeness, reflexivity, antisymmetry, transitivity, and totality proofs inhabit the dictionary fields. |
| [Tooling/Testing/Property](Tooling/Testing/Property.md) | `none-declared` | The canonical checked fences contain no `law`, `proof`, or `theorem` declaration. The Laws and proofs section instead records executable `Bool` witness constants and explicitly says properties are computations, not propositions. |
