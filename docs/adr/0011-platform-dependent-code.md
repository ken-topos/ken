# ADR 0011 — Platform-dependent code: interface-selection, manifest ABI, thin FFI

- **Status:** Accepted (orientation — no near-term work scheduled). Ratified by
  the Architect 2026-07-09, ground-checked against the landed corpus. The
  ratification covers the **Ken-side machinery claims and the design
  orientation**; the external-tool syntax (Idris `%foreign`, Agda `COMPILE`, Lean
  `@[extern]`, Backpack, Dune) is explicitly **out of scope** — illustrative, not
  load-bearing, and flagged below to verify against live docs.
- **Date:** 2026-07-09
- **Deciders:** the operator (a defensive-design question dispatched via the
  Steward); Architect ratification of the characterization of Ken's machinery

## Context

This ADR answers a **defensive-design** question, not a scheduled work item: does
Ken's fundamental design correctly accommodate **platform-dependent code**, or
does supporting it later require a language feature we would regret not having
anticipated? The concern was raised to validate the foundation, and is recorded
here so the reasoning survives even though nothing is to be built in the near
term.

"Platform-dependent code" conflates **three distinct concerns**, and the good
designs in the literature keep them separate because each wants a different
mechanism:

1. **Representation / ABI** — byte order (endianness), word and pointer size,
   alignment, struct layout, calling convention. Low-level, FFI-adjacent.
2. **Foreign binding** — actually calling into C/OS code and marshalling values
   across the boundary.
3. **Implementation selection** — one interface, different implementation per
   platform (POSIX vs. Win32 filesystem, path semantics, which syscalls exist,
   which library backs a capability).

The low-level concerns (1)/(2) are expected to dominate, but (3) carries the
richer design literature and is where a dependently-typed language can do better
than the mainstream.

## Survey — how other language families handle it

**Systems / Rust (the well-engineered baseline).** C uses the preprocessor
(`#ifdef __linux__`, `<stdint.h>`, `<endian.h>`) plus autoconf feature detection
— universal but unprincipled (textual, untyped, no interface enforcement). Rust
keeps conditional compilation but makes it compiler-known: `#[cfg(target_os =
"windows")]` applies at item/module granularity so platform-specific
implementations sit behind a uniform interface, and `[target.'cfg(...)'.
dependencies]` selects a whole implementation crate per target (the
`libc` vs. `winapi` split) — "implementation selection" as conditional linking.

**Haskell (three mechanisms, ascending in principle).** CPP (`{-# LANGUAGE CPP
#-}`) — literally the C preprocessor. Cabal conditionals (`if os(windows)`
stanzas) swap modules/deps — the idiomatic impl-selection lever (the `directory`
package conditionally depends on `unix` or `Win32`). `Foreign.Storable` captures
`sizeOf`/`alignment`/`peek`/`poke` so the *instance* encodes platform layout, and
`hsc2hs`/`c2hs` resolve layout by compiling against the **real headers** at build
time (the correct way to handle ABI — never hardcode offsets). **Backpack** (GHC
module signatures) lets a package be written against an abstract signature and
instantiated with a platform implementation — the most principled Haskell option,
underused.

**ML family (the studied gold standard for implementation selection).** SML/OCaml
**functors**: a `PLATFORM` *signature* + platform-specific *structures* + a
functor parametrizing the program over the platform module — statically checked,
no preprocessor. OCaml Dune's **virtual libraries** (`(virtual_modules …)` +
`(implements …)`) make this a build-system feature. **MirageOS** is the
existence proof at scale: a unikernel OS functorized over its device/platform
modules, retargeted by plugging in different implementation structures.

**Dependently-typed languages.** Two patterns recur:

- *Per-backend realization on the declaration.* **Idris 2**: `%foreign
  "C:add,libadd" "javascript:add"` — one abstract foreign declaration carrying a
  list of per-backend specifiers; the active backend picks its binding.
  **Agda**: `{-# COMPILE GHC … #-}` and `{-# COMPILE JS … #-}` pragmas on the same
  definition. **Lean 4**: `@[extern "sym"]` binds to C, with
  `System.Platform.isWindows`/`numBits` for runtime queries. This "abstract decl
  + per-target specifier" is the cleanest observed design for the *binding*
  concern.
- *Prove abstractly, realize by extraction.* **Coq** extracts to OCaml/Haskell;
  `Extract Constant` maps a logical axiom to target code, and the host language's
  functors handle platform-dependence (CompCert-style). **F\*** takes verified
  low-level portability furthest: a `Low*` effect-typed subset extracts to C via
  KaRaMeL, platform primitives are `assume val` with specifications, and
  **HACL\*/EverCrypt** are real *proven* portable crypto libraries. For the
  verified version of the question, F\*/HACL\* is the state of the art.

## Decision — the design orientation for Ken

All the principled designs share one move: **write against an abstract interface
and select the implementation at the edge.** The preprocessor family
(cpp/`#ifdef`/`cfg`) is the pragmatic baseline — works everywhere, but the
inter-platform interface is enforced only by convention. The principled family
comes in three flavors, and **Ken already possesses all three plus the ABI
discipline** — so platform-dependence rides existing machinery and **Ken should
not grow a preprocessor.**

1. **Implementation selection → a `Platform` capability as a lawful
   record/class.** This is the ML-functor / Backpack / virtual-library idea,
   *strengthened* by dependent types: the interface can carry the **laws** an
   implementation must prove (e.g. read-after-write for a filesystem), not just
   signatures. Implementations are law-proving instances, selected at the top.
   Ken's lawful-class pattern (the catalog's `Eq`/`Ord`/… discipline) is exactly
   this mechanism.
2. **Abstract capability + platform handler → effect rows and handlers.** Ken's
   effect-row machinery (the interaction-tree model) *is* the algebraic-effects
   design: write against an abstract `FileSystem` effect; install a POSIX handler
   or a Win32 handler at the program boundary.
3. **Value-level capability records / dictionary passing** — the same interface
   passed as a value; already available via class dictionaries and `Cap`.
4. **ABI is manifest in types and resolved at the boundary.** Fixed-width types
   (`Int32`, never a native-width `Int`), explicit endianness in every wire/
   serialization format (never native byte order — the direction the `transport`
   package already takes), and binding against real platform headers rather than
   hardcoded layout. Foreign calls are **thin, effect-typed, spec-carrying
   trusted primitives** whose *types pin the representation* — Ken's existing
   opaque-primitive-with-refinement-codomain and effect-tagged (`[FS]`/`[Net]`)
   machinery. If Ken ever gains multiple backends, adopt the Idris-2
   "abstract decl + per-target specifier" model.

**TCB honesty.** Platform primitives are *irreducibly* in the trusted base — one
cannot prove the OS honors its contract. The whole discipline is therefore to
make the platform interface as **small and spec-carrying** as possible and prove
everything *above* it. This is F\*/HACL\*'s posture, and it fits Ken's
small-auditable-TCB principle (`PRINCIPLES §5`) better than any mainstream
language can, because the interface above the boundary is *lawful*, not merely
typed. A platform primitive grows `trusted_base()` — a visible, enumerated,
audited entry (cf. ADR 0009) — never the kernel.

## Consequences

- **No near-term action.** The foundation is validated defensively: when
  platform-dependence is actually needed, it is expressed with machinery Ken
  already has — lawful capability interfaces, effect rows + handlers, class
  dictionaries, `Cap`, `trusted_base()` entries, and refinement-typed primitive
  codomains. No new core language feature is implied.
- **A guardrail on record:** resist a preprocessor / conditional-compilation
  feature as the *primary* mechanism. If an escape hatch is ever needed it should
  be a last resort, not the design.
- **Catalog implication:** a `Platform` interface, when built, is a Capability
  Section entry (a lawful interface with per-platform instances), and its FFI
  surface is an audited `trusted_base()` addition — the same shape as ADR 0009's
  capability-supply spectrum.
- **Specifics to pin before building:** exact current syntax for Idris 2
  `%foreign`, Agda `COMPILE`, Lean `@[extern]`, Backpack signatures, and Dune
  virtual libraries were recorded from knowledge and should be verified against
  live documentation; and whether Ken ever commits to multiple compilation
  backends is itself an open decision that would activate the per-backend
  specifier design.
- **Effectful laws are the real validation target (Architect note).** The
  lawful-interface example carries an *effectful/temporal* law (read-after-write
  for a filesystem), materially heavier than today's landed precedent, which is
  **pure-algebraic** (Eq/Ord associativity etc.). The mechanism — a lawful record
  with proof fields, over effect rows — exists, but *composing* it to prove an
  effectful law over a platform handler is **undemonstrated**, and is the
  concrete validation target when a real `Platform` interface is first built.
  This ADR does not claim effectful laws are already provable the same way
  Eq/Ord laws are; that is the open work the first `Platform` WP must discharge.

## Alternatives considered

- **Preprocessor / conditional compilation as the primary mechanism** (C cpp,
  Haskell CPP, Rust `cfg` as the *only* tool) — rejected as primary: untyped,
  textual, and the inter-platform interface is unenforced. Acceptable only as a
  narrow, last-resort escape hatch.
- **Runtime platform queries alone** (`System.Info`/`System.Platform` style
  branching) — insufficient on its own (it defers all safety to runtime and
  admits no static interface), but a fine *complement* for genuinely
  runtime-discovered facts.
- **Deferring the question entirely** — rejected: the value here is defensive
  confirmation that the fundamental design already accommodates the need, which
  is worth recording precisely because it is *not* near-term.
