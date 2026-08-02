# Catalog package subject index

Availability: **current**

Authority: **derived reference**

This path-preserving index is generated from the first heading of every checked
catalog package document. At this revision, the command returns all 39 package
subjects:

```sh
git grep -n '^# ' -- 'catalog/packages/**/*.ken.md'
```

```text
catalog/packages/Application/CommandLine/ArgParse.ken.md:1:# ArgParse
catalog/packages/Application/Configuration/Decoder.ken.md:1:# Application.Configuration.Decoder
catalog/packages/Application/Input/Schema.ken.md:1:# Schema
catalog/packages/Capability/Console/Text.ken.md:1:# `Console` — ordinary text-output helpers
catalog/packages/Capability/Diagnostics/Core.ken.md:1:# Capability.Diagnostics.Core
catalog/packages/Capability/Diagnostics/Render.ken.md:1:# Capability.Diagnostics.Render
catalog/packages/Capability/Filesystem/Authority.ken.md:1:# Filesystem authority manifests
catalog/packages/Capability/Filesystem/Errors.ken.md:1:# `FS` — file-error rendering
catalog/packages/Capability/Filesystem/Path/Posix.ken.md:1:# `Capability.Filesystem.Path.Posix` — byte-preserving lexical paths
catalog/packages/Capability/Formatting/Doc.ken.md:1:# `Capability.Formatting.Doc` — a small lawful document algebra
catalog/packages/Capability/Parsing/Cursor.ken.md:1:# Capability.Parsing.Cursor
catalog/packages/Capability/Parsing/Decoder.ken.md:1:# Capability.Parsing.Decoder
catalog/packages/Capability/Parsing/Numeric.ken.md:1:# `Capability.Parsing.Numeric` — located decimal parsing
catalog/packages/Capability/Parsing/Parsing.ken.md:1:# `parsing` — source artifacts, spans, parsers, and a Boolean grammar
catalog/packages/Capability/Process/Arguments.ken.md:1:# Capability.Process.Arguments
catalog/packages/Capability/Process/Environment.ken.md:1:# Capability.Process.Environment
catalog/packages/Capability/Process/Exit.ken.md:1:# Capability.Process.Exit
catalog/packages/Capability/Process/WorkingDirectory.ken.md:1:# Capability.Process.WorkingDirectory
catalog/packages/Capability/System/Buffer.ken.md:1:# System.Buffer
catalog/packages/Capability/System/IO.ken.md:1:# System.IO
catalog/packages/Capability/System/Resource.ken.md:1:# System.Resource
catalog/packages/Capability/Time/WallClock.ken.md:1:# Capability.Time.WallClock
catalog/packages/Core/Classes/EffectfulClasses.ken.md:1:# `Applicative`, `Monad`, and `Traversable` — effectful constructor classes
catalog/packages/Core/Classes/LawfulClasses.ken.md:1:# `lawful-classes` — `Eq`, `DecEq`, `Ord`
catalog/packages/Core/Classes/LawfulFunctors.ken.md:1:# `lawful-functors` — `Semigroup`, `Monoid`, `Functor`, `Foldable`
catalog/packages/Core/Logic/EmptyDec.ken.md:1:# `Empty` and `Dec` — computational falsity and decidability
catalog/packages/Core/Logic/Transport.ken.md:1:# `transport` — `subst`, `cong`, `cast`, `sym`, `trans`
catalog/packages/Data/Binary/BytesKeys.ken.md:1:# `Data.Binary.BytesKeys` — lawful byte equality
catalog/packages/Data/Collections/Derived.ken.md:1:# `Collections` — derived collection, string, and byte views
catalog/packages/Data/Collections/Map.ken.md:1:# `Map`/`Set` — a proved, pure ordered binary search tree
catalog/packages/Data/Collections/NonEmpty.ken.md:1:# `NonEmpty` — lists with a structural head
catalog/packages/Data/Numeric/Nat/Arithmetic.ken.md:1:# `Nat` arithmetic — canonical operations and free algebraic laws
catalog/packages/Data/Numeric/Nat/Order.ken.md:1:# `Ord Nat` — a lawful total order on `Nat`, and its operations
catalog/packages/Data/Sums/Combinators.ken.md:1:# `Sums` — the `Option`/`Result`/`Either` combinator floor
catalog/packages/Data/Sums/Validation.ken.md:1:# `Validation` — accumulating independent errors
catalog/packages/Data/Text/Codec.ken.md:1:# `Data.Text.Codec` — safe UTF-8 and ASCII views
catalog/packages/Data/Text/StringBijection.ken.md:1:# String/List-Char retraction and injectivity certificate
catalog/packages/Data/Text/StringKeys.ken.md:1:# `Data.Text.StringKeys` — lawful String equality and order
catalog/packages/Tooling/Testing/Property.ken.md:1:# `Tooling.Testing.Property` — deterministic finite property checks
```

The path and heading together are the generated subject identity. The index
does not infer declarations, laws, effects, assurance, platform, maturity, or
dependency facts.
