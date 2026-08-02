# Catalog declaration and type index

Availability: **current**

Authority: **derived reference**

## Population and disposition

This index projects the Declaration/type row from every landed catalog card.
Its population is 39 of 39 cards: all 39 rows are `authored`, and none is
`none-declared`. Each entry links the card that supplies the complete result;
the index does not re-read or extend the checked package.

| Package card | Disposition | Projected result |
|---|---|---|
| [Application/CommandLine/ArgParse](Application/CommandLine/ArgParse.md) | `authored` | Declares explicit program, command, option, positional, and parsed-result carriers; byte-matching parsers; located diagnostics; accumulating validation; and help derived from the same specification. |
| [Application/Configuration/Decoder](Application/Configuration/Decoder.md) | `authored` | Declares environment/config provenance, plain-`Bytes` lookup, schema-driven accumulating validation, environment and caller-supplied config entry points, and schema help. |
| [Application/Input/Schema](Application/Input/Schema.md) | `authored` | Declares field presence and value-shape vocabularies, schema and issue carriers, parameterized field checks, accumulating validation traversals, and shared help rendering. |
| [Capability/Console/Text](Capability/Console/Text.md) | `authored` | Declares four text-output procedures: `print`, `printLine`, `eprint`, and `eprintLine`; they encode UTF-8, select stdout or stderr, and make newline policy explicit. |
| [Capability/Diagnostics/Core](Capability/Diagnostics/Core.md) | `authored` | Declares source identities, half-open byte ranges, four origin families, stable diagnostic codes, structured diagnostics, projections, and checkable validity predicates. |
| [Capability/Diagnostics/Render](Capability/Diagnostics/Render.md) | `authored` | Declares projections from diagnostic codes and origins plus `diagnostic_to_doc`, which renders an origin-family label and stable code into the document algebra. |
| [Capability/Filesystem/Authority](Capability/Filesystem/Authority.md) | `authored` | Declares `capability_read`, polymorphic in an authority index, and `full_authority_write`, whose `Cap AFull` input makes full authority load-bearing. |
| [Capability/Filesystem/Errors](Capability/Filesystem/Errors.md) | `authored` | Declares total renderers for `IOError` and `FileError`; the stable `Other` label intentionally leaves its `Int` payload available for structured inspection. |
| [Capability/Filesystem/Path/Posix](Capability/Filesystem/Path/Posix.md) | `authored` | Declares structured raw-byte paths, parsing and rendering, joining and parents, validity checks, and lexical normalization without decoding through `String`. |
| [Capability/Formatting/Doc](Capability/Formatting/Doc.md) | `authored` | Declares a six-constructor `Doc` algebra, content and validity projections, deterministic fitting and rendering, structural content rendering, and thin String-boundary helpers. |
| [Capability/Parsing/Cursor](Capability/Parsing/Cursor.md) | `authored` | Declares an explicit carrier/element/location operations dictionary, raw-byte argument locations and cursors, structural remaining-input operations, normalization, and the concrete argument-cursor dictionary. |
| [Capability/Parsing/Decoder](Capability/Parsing/Decoder.md) | `authored` | Declares location-generic decoder errors and results, the decoder function type, sequencing and token combinators, and repetition and recursive layers whose fuel derives from cursor remaining input. |
| [Capability/Parsing/Numeric](Capability/Parsing/Numeric.md) | `authored` | Declares located numeric diagnostics, decimal parsers over structural character lists, a proof-carrying decimal-digit carrier, structural digit formatting, and the thin `show_digits` String wrapper. |
| [Capability/Parsing/Parsing](Capability/Parsing/Parsing.md) | `authored` | Declares checked source artifacts, spans and located values, total parser results and validity predicates, decoder-backed base parsers, and a complete byte-token Boolean grammar with parsing and formatting. |
| [Capability/Process/Arguments](Capability/Process/Arguments.md) | `authored` | Declares raw-byte argv projection and replacement, positional lookup, byte lookup, structural bounds comparison, and checked `ArgLocation` construction. |
| [Capability/Process/Environment](Capability/Process/Environment.md) | `authored` | Declares projection and replacement for the ordered raw-byte key/value environment in `ProcessInput`, preserving arguments and working-directory bytes. |
| [Capability/Process/Exit](Capability/Process/Exit.md) | `authored` | Declares named success and failure exit values plus total policies that map arbitrary outcomes or `Result` values to the landed `ExitCode` ABI. |
| [Capability/Process/WorkingDirectory](Capability/Process/WorkingDirectory.md) | `authored` | Declares raw-byte working-directory projection and replacement over `ProcessInput`, preserving arguments and environment unchanged. |
| [Capability/System/Buffer](Capability/System/Buffer.md) | `authored` | Declares a `BufferWindow` constructor, scalar and structural projections from `BufferSpan` and `TransferCount`, and proofs for positive, request-bounded transfer counts. |
| [Capability/System/IO](Capability/System/IO.md) | `authored` | Declares five theorems about the transparent `writeAll` loop: its call bound, exact-prefix step, complete success, first-error preservation, and all-success result. |
| [Capability/System/Resource](Capability/System/Resource.md) | `authored` | Declares constructors for successful and failed resource bodies and a total Boolean classifier over every `ResourceBracketResult` constructor. |
| [Capability/Time/WallClock](Capability/Time/WallClock.md) | `authored` | Declares projection and replacement for the nanosecond `Int` carried by the structural `Instant` value. |
| [Core/Classes/EffectfulClasses](Core/Classes/EffectfulClasses.md) | `authored` | Declares the `Applicative`, `Monad`, and `Traversable` classes; concrete `Option` and `List` instances; an `Identity` support instance; and the checked helper functions used by their dictionaries. |
| [Core/Classes/LawfulClasses](Core/Classes/LawfulClasses.md) | `authored` | Declares `IsTrue`, `Eq`, `DecEq`, `Ord`, comparison helpers, and registered dictionaries for `Int`, `Bool`, `Char`, `Pair`, and `List` as supported by the checked source. |
| [Core/Classes/LawfulFunctors](Core/Classes/LawfulFunctors.md) | `authored` | Declares the four named classes, their operations and coherence fields, and checked `List`, `Bool`, and `Option` instances with their supporting functions. |
| [Core/Logic/EmptyDec](Core/Logic/EmptyDec.md) | `authored` | The public API is `Empty`, `absurd_empty`, `Dec`, `Yes`, `No`, `decide`, `yes`, `no`, and `dec_eq_decides`. The standard `Empty`/`Dec` display is illustrative; the package-authored wrappers and bridge are checked. |
| [Core/Logic/Transport](Core/Logic/Transport.md) | `authored` | Exports `subst`, `cong`, `cast`, `sym`, and `trans`: five non-recursive wrappers over the surface equality eliminator `J` and native equality `Eq`. |
| [Data/Binary/BytesKeys](Data/Binary/BytesKeys.md) | `authored` | Declares equality decisions and `DecEq` dictionaries for `UInt8` and `Bytes`, transported through the checked integer widening and structural byte-list views. |
| [Data/Collections/Derived](Data/Collections/Derived.md) | `authored` | Declares list and natural-number combinators, verified sorting, projection-abstraction classes, derived string operations, and a structural byte-length fold. |
| [Data/Collections/Map](Data/Collections/Map.md) | `authored` | Declares the ordered tree carrier and map/set operations, keyed deletion and combination, projections, and finite binary-relation operations. |
| [Data/Collections/NonEmpty](Data/Collections/NonEmpty.md) | `authored` | Declares a head-plus-tail `NonEmpty` carrier, total head and tail projections, list conversion, mapping, append, and its `Semigroup` dictionary. |
| [Data/Numeric/Nat/Arithmetic](Data/Numeric/Nat/Arithmetic.md) | `authored` | Declares structurally recursive natural-number `add` and `mul` operations. |
| [Data/Numeric/Nat/Order](Data/Numeric/Nat/Order.md) | `authored` | Declares structural `Nat` ordering, an `Ord Nat` dictionary, three-way `OrdResult`, and `min`, `max`, `sub`, and `compare`. |
| [Data/Sums/Combinators](Data/Sums/Combinators.md) | `authored` | Declares the neutral `Either` carrier and structural elimination, mapping, fallback, chaining, and swapping combinators across `Option`, `Result`, and `Either`. |
| [Data/Sums/Validation](Data/Sums/Validation.md) | `authored` | Declares the `Validation` carrier, mapping, pure, error-accumulating application, and lawful `Functor` and `Applicative` dictionaries parameterized by a `Semigroup`. |
| [Data/Text/Codec](Data/Text/Codec.md) | `authored` | Declares safe UTF-8 decoding, byte-level ASCII classification, and an optional indexed ASCII view that preserves absent-byte results. |
| [Data/Text/StringBijection](Data/Text/StringBijection.md) | `authored` | Declares the single `string_to_list_char_retraction` certificate and derives `string_to_list_char_injective` for consumers of lawful string keys. |
| [Data/Text/StringKeys](Data/Text/StringKeys.md) | `authored` | Declares transported equality and ordering operations plus lawful `DecEq String` and `Ord String` dictionaries over the checked `List Char` views. |
| [Tooling/Testing/Property](Tooling/Testing/Property.md) | `authored` | The public surface is `Gen`, `gen_from_list`, `gen_samples`, `gen_map`, `check`, `gen_bytes`, and three executable witnesses. Results use the ordinary `Result a Unit` carrier. |
