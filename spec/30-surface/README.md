# The surface language

> Status: **DRAFT v0**. Normative for the *constructs and their meaning*; the
> **concrete syntax is a reasonable proposal, explicitly revisable** (OQ-syntax,
> `../90-open-decisions.md`). Contract for WS-L. Resolution is intentionally
> lower than `10-kernel/`/`20-verification/`: this fixes *what the language has*
> and how it elaborates to the core, leaving spelling to be settled with the
> team.

The surface is the language a programmer or agent actually writes. It is also
the **self-hosting substrate** (strategy WS-L), so it is core, not a late skin.
It elaborates (`39-elaboration.md`) to the kernel core (`../10-kernel/`); the
surface adds ergonomics (implicit arguments, inference, sugar, modules), never
new trusted semantics.

## 1. Design stance

- **Clean-room but informed.** Ken's vocabulary is its own; where the
  prototype's shape is proven (effects, modules) Ken adopts the *shape*, not the
  names or code (digest §7).
- **Finish what the prototype left stubbed.** The prototype *parses* sum types
  but lowers them to an opaque base with no constructor/eliminator — the single
  most important thing to get right. Ken has **real** sum types, `match`, and
  exhaustiveness from day one (`34-data-match.md`); this is the prime cautionary
  tale.
- **`Int` from day one.** One surface numeric type lowered to `f64` was the
  prototype's real defect. Ken has `Int`, `Decimal`, and an honestly-named
  `Float` (`35-numbers.md`).
- **Verification is in the surface, not bolted on.** `requires`/`ensures`,
  refinement types, and `prove` (`../20-verification/21-spec-syntax.md`) are
  first-class surface forms.
- **Concrete syntax is OQ.** The forms below are a coherent proposal; bikeshed
  (keywords, layout vs. braces, operator set) is deferred, not load-bearing.

## 2. Core vocabulary (proposal)

| Concept | Surface form | Elaborates to |
|---|---|---|
| Pure function (a morphism) | `view f (x : A) : B = …` | Π + λ (`13`) |
| Type alias / refinement | `type T = …` / `{ x:A | φ }` | core type / Σ (`12 §5`, `34`) |
| Record (product) | `record R { f : A, … }` | right-nested Σ with η (`13 §3`) |
| Sum / inductive | `data D = C1 … | C2 …` | inductive family (`14`) |
| Pattern match | `match e { … }` | `elim_D` (`14 §3`, `39`) |
| Definition / value | `let x : A = e` | global/local def (`11 §4`) |
| Proposition / spec | `requires`/`ensures`/`prove`/`law` | Ω terms + obligations (`20`) |
| Effectful function | `view f (…) visits [E] = …` | effect-rowed type (`36`) |
| Module | `module M { … }` / `import` | namespaced env (`33`) |
| Stateful region | `space S { … }` | the state/effect model (`36`, OQ-Space DECIDED) |

## 3. Chapter map

| File | Subject |
|---|---|
| `31-lexical.md` | Tokens, identifiers, literals, comments, layout |
| `32-grammar.md` | The concrete grammar (declarations, types, expressions, patterns) |
| `33-declarations.md` | Modules, imports, definitions, records, visibility, typeclasses-as-subobjects |
| `34-data-match.md` | Sum types, constructors, `match`, exhaustiveness, `Result`/`Option`, refinements |
| `35-numbers.md` | `Int`/`Decimal`/`Float` — the f64 correction in full |
| `36-effects.md` | Effect tracking (`visits`), capabilities, the state/`space` model |
| `37-strings-collections.md` | `String`, `List`, `Map`, `Set`, iteration |
| `38-ffi-io.md` | `Bytes`, binary I/O, the FFI surface |
| `39-elaboration.md` | Surface → core: implicits, inference, sugar expansion |

## 4. What the surface must deliver (WS-L, ties to G6)

Real types (`Int`, sum types, `Bytes`), `match` + exhaustiveness, modules + a
package manager, an effect system, FFI, and a curated stdlib — enough to write
one realistic verified component end-to-end (G6) and, eventually, to host a
compiler (self-hosting prerequisite). The verification surface (`../20-`) rides
on top. Conformance for surface behavior: `../../conformance/surface/`.
