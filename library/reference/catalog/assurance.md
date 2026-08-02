# Catalog assurance index

Availability: **current**

Authority: **derived reference**

## Population and disposition

This index projects the Assurance row from every landed catalog card. Its
population is 39 of 39 cards: all 39 rows are `authored`, and none is
`none-declared`. Each entry cites the card that owns the complete assurance
summary; this index introduces no stronger proof or trust claim.

| Package card | Disposition | Projected result |
|---|---|---|
| [Application/CommandLine/ArgParse](Application/CommandLine/ArgParse.md) | `authored` | Every declaration is a transparent structural term over the cited checked floor. The package adds no primitive, postulate, `Axiom`, or trusted-base entry, and preserves raw arguments as `Bytes`. |
| [Application/Configuration/Decoder](Application/Configuration/Decoder.md) | `authored` | The package adds no parser, renderer, cached-length carrier, primitive, postulate, `Axiom`, or trusted-base entry; raw values, including invalid UTF-8, remain `Bytes`. |
| [Application/Input/Schema](Application/Input/Schema.md) | `authored` | The schema layer is client-independent and adds no primitive, postulate, `Axiom`, or trusted-base entry; result and issue carriers remain parameterized over client types. |
| [Capability/Console/Text](Capability/Console/Text.md) | `authored` | The helpers are ordinary kernel-checked definitions over the byte-exact Console ABI and add zero `trusted_base()` entries. |
| [Capability/Diagnostics/Core](Capability/Diagnostics/Core.md) | `authored` | All declarations are transparent kernel-checked terms over landed data; the package defines no renderer, axiom, primitive, postulate, or opaque constant. |
| [Capability/Diagnostics/Render](Capability/Diagnostics/Render.md) | `authored` | The renderer is a transparent structural projection into `Capability.Formatting.Doc`; it adds no error carrier, primitive, postulate, `Axiom`, or trusted-base entry. |
| [Capability/Filesystem/Authority](Capability/Filesystem/Authority.md) | `authored` | The checked surface distinguishes `Cap AFull` from `Cap APartial` and deliberately exposes no attenuation, revocation, strengthening, constructor, or management binding. Host-only identity and settlement claims remain outside checked Ken. |
| [Capability/Filesystem/Errors](Capability/Filesystem/Errors.md) | `authored` | Rendering is ordinary package policy rather than host-driver behavior and adds zero entries to `trusted_base()`. |
| [Capability/Filesystem/Path/Posix](Capability/Filesystem/Path/Posix.md) | `authored` | Every operation is transparent over `List UInt8` and existing lawful equality; the package declares no primitive, postulate, opaque constant, or `Axiom`, so its `trusted_base()` delta is zero. |
| [Capability/Formatting/Doc](Capability/Formatting/Doc.md) | `authored` | Every declaration is transparent ordinary Ken with structural recursion and landed equality combinators; the package adds no primitive, postulate, `Axiom`, trusted-base entry, or diagnostic dependency. |
| [Capability/Parsing/Cursor](Capability/Parsing/Cursor.md) | `authored` | All declarations are transparent checked terms over landed `Bytes`, `List`, `Option`, and equality; the package adds no axiom, primitive, or postulate. |
| [Capability/Parsing/Decoder](Capability/Parsing/Decoder.md) | `authored` | Every combinator is transparent, structurally recursive on `Nat` fuel, and uses only checked cursor operations; the package adds no axiom or primitive. |
| [Capability/Parsing/Numeric](Capability/Parsing/Numeric.md) | `authored` | Parsing and formatting are structural checked definitions; the format law avoids the opaque Int/String gap, and the package declares no primitive, postulate, opaque constant, `Axiom`, or trusted-base delta. |
| [Capability/Parsing/Parsing](Capability/Parsing/Parsing.md) | `authored` | The package is ordinary data, a class-backed record, transparent functions, and kernel-checked proofs; it adds no primitive or source-loader semantics, and its `trusted_base()` delta is zero. |
| [Capability/Process/Arguments](Capability/Process/Arguments.md) | `authored` | All declarations are transparent checked terms over landed `ProcessInput`, `List`, `Bytes`, and `ArgLocation`; there is no primitive, postulate, opaque constant, or `Axiom`, and the `trusted_base()` delta is zero. |
| [Capability/Process/Environment](Capability/Process/Environment.md) | `authored` | The declarations are transparent checked terms over landed `ProcessInput`, `List`, `Prod`, and `Bytes`; the package adds no primitive, postulate, opaque constant, `Axiom`, or trusted-base entry. |
| [Capability/Process/Exit](Capability/Process/Exit.md) | `authored` | All four declarations are transparent checked terms over `ExitCode`, `UInt8`, and `Result`; the package adds no datatype, primitive, postulate, opaque constant, `Axiom`, or trusted-base entry. |
| [Capability/Process/WorkingDirectory](Capability/Process/WorkingDirectory.md) | `authored` | The declarations are transparent checked terms over landed `ProcessInput` and `Bytes`; the package adds no primitive, postulate, opaque constant, `Axiom`, or trusted-base entry. |
| [Capability/System/Buffer](Capability/System/Buffer.md) | `authored` | Count and budget witnesses are kernel-checked Ken data; fixed capacity, current-window discipline, and settlement invalidation remain explicitly runtime-enforced rather than restated as proofs. |
| [Capability/System/IO](Capability/System/IO.md) | `authored` | The five proofs are ordinary kernel-checked terms and not axioms or runtime claims; exactly-once settlement and liveness remain explicitly delegated to the runtime boundary. |
| [Capability/System/Resource](Capability/System/Resource.md) | `authored` | The page distinguishes ordinary checked result handling from runtime-enforced handle liveness, settlement, authority denial, and the currently deferred checked-source controlled-trap face. |
| [Capability/Time/WallClock](Capability/Time/WallClock.md) | `authored` | Both declarations are transparent structural definitions; the page keeps host clock movement outside their claim and requires a separate session-shaped design for monotonicity. |
| [Core/Classes/EffectfulClasses](Core/Classes/EffectfulClasses.md) | `authored` | The package states zero trusted-base delta for its law fields: proofs are kernel-checked, with finite `Option` cases and structural `List` induction. Its validation evidence checks the trust posture, discriminating failures, and checked fences. |
| [Core/Classes/LawfulClasses](Core/Classes/LawfulClasses.md) | `authored` | Trust is carrier-specific: `Ord Int` retains four visible `Axiom` law fields; the named integer equality certificate is pre-existing; `Bool` adds zero trust; transported `Char` dictionaries add zero new trust; the lifted structures use checked proofs. |
| [Core/Classes/LawfulFunctors](Core/Classes/LawfulFunctors.md) | `authored` | Every instance has zero trusted-base delta. The law fields are kernel-checked with induction, case splitting, `Proved`, `Refl`, and `cong`; none is postulated. |
| [Core/Logic/EmptyDec](Core/Logic/EmptyDec.md) | `authored` | Standard inductives and package functions add no postulate or primitive. `dec_eq_decides` preserves the trust posture of the supplied `DecEq` instance; the checked `Bool` example has zero added trust. |
| [Core/Logic/Transport](Core/Logic/Transport.md) | `authored` | All five public names are ordinary uses of `J`, `Eq`, and equality reduction. They add zero trusted-base delta, use no recursion, and introduce no eliminator or reduction rule. |
| [Data/Binary/BytesKeys](Data/Binary/BytesKeys.md) | `authored` | The package declares no local trust: it consumes the existing `UInt8` and `Bytes` retraction certificates and adds no equality primitive or kernel certificate. |
| [Data/Collections/Derived](Data/Collections/Derived.md) | `authored` | The package reports zero `trusted_base()` delta: its inductive, definitions, and proofs are checked terms, with no primitive, postulate, or law-field assumption. |
| [Data/Collections/Map](Data/Collections/Map.md) | `authored` | The package reports zero `trusted_base()` delta: every operation and proof is checked, recursion is termination checked, and no `Axiom` occurs in the canonical fences. |
| [Data/Collections/NonEmpty](Data/Collections/NonEmpty.md) | `authored` | The package reports zero `trusted_base()` delta: the carrier is strictly positive, operations are structural, and the sole class law is a checked proof term. |
| [Data/Numeric/Nat/Arithmetic](Data/Numeric/Nat/Arithmetic.md) | `authored` | The operations and proofs are ordinary checked definitions using structural recursion on `Nat`; no trusted declaration or numeric instance is introduced. |
| [Data/Numeric/Nat/Order](Data/Numeric/Nat/Order.md) | `authored` | The package reports zero `trusted_base()` delta: every order law is kernel checked and the entry introduces no `Axiom`, primitive, or postulate. |
| [Data/Sums/Combinators](Data/Sums/Combinators.md) | `authored` | The package reports zero `trusted_base()` delta: the new carrier is positivity checked, every combinator is structural, and every proof is an ordinary term. |
| [Data/Sums/Validation](Data/Sums/Validation.md) | `authored` | The package reports zero `trusted_base()` delta: every class field is a kernel-checked term, with semigroup associativity consumed only in the all-invalid branch. |
| [Data/Text/Codec](Data/Text/Codec.md) | `authored` | Trust delta is zero: every operation is ordinary Ken over landed total byte operations, and the checked fences add no `Axiom`, primitive, postulate, or opaque declaration. |
| [Data/Text/StringBijection](Data/Text/StringBijection.md) | `authored` | The retraction is the one explicit named `axiom` selected at the conversion layer; injectivity is transparent, and no comparator primitive or second certificate is introduced. |
| [Data/Text/StringKeys](Data/Text/StringKeys.md) | `authored` | The package contains no `Axiom`; equality-producing fields cite the separately homed injectivity certificate, and all other fields use dictionary projections and congruence. |
| [Tooling/Testing/Property](Tooling/Testing/Property.md) | `authored` | The checked implementation has zero trusted-base delta and introduces no primitive, postulate, axiom, proof hole, effect, or assumed proposition. Its deterministic finite samples make the first counterexample reproducible. |
