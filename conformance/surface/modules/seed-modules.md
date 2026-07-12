# Modules & imports conformance — seed cases (ES3)

Format: `../../README.md`. These pin the **module / namespace substrate** of
`spec/30-surface/33-declarations.md §3–4` (ES3, the bounded L4 slice):
`module` / `import` / `pub` / abstract-export are a **surface +
elaboration-time only** device that **elaborates away** to the kernel's
**single flat append-only `Σ`** (`../../spec/10-kernel/11 §4`), with
**abstract export = the existing opaque constant** (`11 §4`: an opaque
constant "is how axioms, FFI signatures, and **abstract interfaces** are
represented"). **Zero new kernel feature, zero `trusted_base()` delta** — the
ES1 minimality invariant (surface built-in set ≡ `trusted_base()` delta,
`../taxonomy/minimality.md`) carries verbatim. N2 extends this same surface
substrate with an **in-repo cross-file loader**. N4 adds the source-world
`program` / `package` admission boundary over that loader. The
content-addressed package manager remains out of scope.

**Build state — RED UNTIL N2 LANE B.** The cross-file accept arm in §D states
the N2 Lane-B target and currently fails because no loader follows an import
path to another file. The cycle arm pins the fail-closed behavior Lane B must
establish once that loader exists. Both are specification/conformance cases in
this Lane-A seed; neither claims the current elaborator implements N2.

The original ES3 module resolver and the kernel mechanisms it rides — the flat
`Σ`, opaque constants, and `trusted_base()` — are landed. N2 adds the missing
file-discovery producer without changing those mechanisms. The cases therefore
retain the ES3 design discriminants and add a controlled cross-file
accept↔cycle-reject pair for Lane B.

**Build state — RED UNTIL N4 LANE B.** The §E source-world cases state N4's
anonymous boundary headers, direct-use admission gate, self-admission,
closure-wide coherence, and provenance contract. No current parser accepts the
new headers, so every §E case is red at N4's surface before Lane B. Compiled
instance manifests and re-export-carried instance surfaces are deliberately not
asserted as live: they remain package-manager/post-MRES-9 work.

Grounding (landed `§`-bodies + landed code, content-reconciled — not the
plan): `33 §3`
(`module`/`import M`/`import M as N`/`import M (foo, Bar as Baz)`; the kernel
sees a single flattened `Σ`), `33 §4` (visibility: module-private by default +
`pub`; abstract export = opaque interface), `11 §4` (the append-only acyclic
flat `Σ`; the opaque constant `c : A` that introduces abstract interfaces),
ES1 `minimality.md` (the `trusted_base()`-delta invariant ES3 must not
perturb), ADR 0014 MRES-1/MRES-2/MRES-3a, and the total role-blind dotted-path
↔ leaf-file bijection in `docs/program/07-catalog-style-guide.md §13`.

## Reading these cases — the ES3 disciplines

**Modules elaborate away — the load-bearing invariant (`33 §3`, AC1).** A
`module`/`import`/`pub`/abstract-export program and its **fully-qualified,
single-namespace equivalent** (every `M.foo` written flat as `M_foo`, no
module) elaborate to the **identical** flat append-only `Σ` — and therefore
the **identical `trusted_base()`**. The witness is **`Σ`-identity /
`trusted_base()`- identity**, not "it type-checks": a module is an environment
fragment resolved at elaboration, invisible to the kernel. The discriminating
direction: a design that put a **kernel-level module or visibility primitive**
into `Σ` (a new `trusted_base()` entry) **fails** the identity; the
elaborates-away form passes. This is the ES1 minimality net pointed at modules
— modules cost the trust root **nothing**.

**Abstract export IS the opaque constant, not a new mechanism (`11 §4`,
AC2).** A type exported name-only (constructors hidden) maps to the
**existing** opaque constant `T : Type` (`11 §4` — the same mechanism as an
FFI signature or an axiom), byte-identical to a hand-written opaque constant.
Information hiding is enforced at **elaboration** (the hidden constructors are
simply **not exported**, so not in scope for a client), **never** by a kernel
"abstract" flag. A client's `match` on a hidden constructor is a **surface**
name-resolution error, not a kernel type error.

**Visibility + resolution are surface-only (`33 §3`/`§4`, AC3/AC4).** Every
resolution failure — unresolved name, private-name access from outside,
ambiguous `use`-open — is a **surface diagnostic that never reaches the
kernel**. The visibility default is **private-by-default + `pub`** (AC4,
settled in `/spec` `33 §4`; the private-name-rejection case (§C) is its
conformance witness).

**Producer-grep (design-time, forward to ES3-build).** Drive the **real
import-resolution** — a case that asserts `M.foo` resolves by **constructing**
the resolved binding itself (rather than exercising the import rule) is
green-vs-green; pin resolution + the visibility rejection against the **stated
rules**, and the `Σ`/`trusted_base()` identity against the **landed** kernel.

## A. Modules elaborate away — zero TCB delta (AC1 ★)

### surface/modules/module-elaborates-to-identical-flat-sigma (soundness)
- spec: `33 §3`/`§3.1` (module = environment fragment → flattened `Σ`),
  `11 §4` (append-only flat `Σ`), ES1 `../taxonomy/minimality.md` (the
  `trusted_base()` invariant)
- given: two programs — (a) `module M { def foo : Nat := 0 }` with `import M`
  and a use of `M.foo`; (b) the **fully-qualified single-namespace
  equivalent** `def M_foo : Nat := 0` with a use of `M_foo` — each elaborated
  to core
- expect: the two produce the **identical** flat append-only `Σ` (same
  declarations, same order — the module is resolved away) and therefore the
  **identical `trusted_base()`** (`11 §4` filter); **no** module / visibility
  / namespace entry appears in `Σ` or the base
- why: (soundness) AC1, the elaborates-away invariant. A module is an
  elaboration-time **environment fragment**; the kernel sees one flat `Σ`
  (`33 §3`, `11 §4`). **Discriminating on `Σ`/`trusted_base()`-identity:** a
  design that admitted a **kernel module or visibility primitive** (a new
  `trusted_base()` entry) would make (a) ≠ (b) — this case **fails** it; the
  elaborates-away form passes. Grounds the ES1 minimality invariant: **adding
  ES3 leaves the surface `trusted_base()` delta unchanged.** Assert the **`Σ`
  / `trusted_base()` equality**, not "both type-check" (that passes
  vacuously).

## B. Abstract export IS the opaque constant (AC2)

### surface/modules/abstract-export-is-the-opaque-constant
- spec: `33 §4` (abstract export), `11 §4` (opaque constant introduces
  abstract interfaces)
- given: a `module M { pub data T = MkT ; … }` exporting `T` **abstractly**
  (name only, `MkT` hidden), vs a hand-written **opaque constant** `T : Type`
- expect: the abstractly-exported `T`'s **kernel representation is
  byte-identical to the opaque constant** — an opaque `T : Type` in `Σ`
  (`11 §4`), δ-blocking, no constructors visible; **no** kernel "abstract"
  flag or new `Decl` variant
- why: AC2, abstract export = the **existing** opaque-constant mechanism
  (`11 §4` — "how … abstract interfaces are represented"). **Discriminating:**
  a design that added a kernel-level "abstract" marker (a new `Decl`/`Σ` form)
  would make the rep **differ** from a plain opaque constant — this case pins
  them **identical**. Information hiding is surface/elaboration, not a kernel
  concept.

### surface/modules/client-match-hidden-ctor-rejected-at-surface (soundness)
- spec: `33 §4` (abstract export hides constructors), `33 §3.3` (resolution)
- given: `module M { pub data T = MkT }` exporting `T` **abstractly**, and a
  **client** module `import M` that attempts `match t { MkT => … }` on a
  `t : T`
- expect: **rejected at the surface** — `MkT` is **not in scope** (not
  exported; the abstract export withholds the constructor), a
  **name-resolution / surface diagnostic**, **not** a kernel type error, and
  the client is **not admitted**
- why: (soundness) AC2, the information-hiding enforcement is **surface**. The
  hidden constructor never enters the client's scope, so the `match` fails
  resolution **before** the kernel. **Discriminating:** the reject is a
  **surface** name error (`MkT` unresolved), **not** a kernel `TypeMismatch` /
  elaboration of a constructor — a design leaking `MkT` into scope (or
  enforcing the hiding by a kernel check) fails. Pairs with the `Σ`-identity
  of the abstract-export case: hiding is real **and** costs the kernel
  nothing.

## C. Visibility + resolution — surface-only, well-defined (AC3/AC4)

### surface/modules/private-name-access-rejected-at-surface (soundness)
- spec: `33 §4.1` (private-by-default + `pub`, settled), `33 §3.3`
  (resolution)
- given: `module M { def secret : Nat := 0 ; pub def api : Nat := 1 }` (no
  `pub` on `secret`), and a **client** `import M` that references
  **`M.secret`**
- expect: **rejected at the surface** — `secret` is **private**
  (module-private by default, not `pub`-exported), an **unresolved /
  not-exported surface diagnostic**, **not** a kernel error; `M.api`
  (exported) **resolves**
- why: (soundness) AC3 + the **AC4 witness** (private-by-default). Visibility
  is a **surface** predicate on the module interface; a private name is not in
  the export set, so a cross-module reference fails resolution **before** the
  kernel. **Discriminating flip:** `M.api` (pub) accepts, `M.secret` (private)
  rejects — keyed on the **`pub` export set**, on the real resolution rule,
  not a hand-fed visibility flag. Confirms the settled **private-by-default**
  default (`33 §4`, `OQ-syntax` resolved).

### surface/modules/top-level-local-import-clash-rejected
- spec: `33 §3.3` (module-level local/import clash, fail-closed)
- given: `M` exports distinct `foo` and `keep`; one client imports `M.foo`
  unqualified with `import M (foo)` and also declares a top-level `foo`. Run
  both declaration orders. In one arm, never reference either `foo` after
  declaring them.
- expect: both arms reject at the surface with the specific
  **`AmbiguousReference`** diagnostic for `foo`, identifying the distinct
  current-module `foo` and `M.foo` sources. The latent, never-referenced arm
  rejects at the same binding-time gate. Neither arm reaches the kernel.
- why: N3 AC2, the module-level reversal. The two declaration orders and the
  unused arm prevent a use-site-only ambiguity check or silent last-writer/local
  precedence from passing. **RED UNTIL N3 LANE B:** current `bind_import`
  silently keeps the local binding instead of raising the clash.

### surface/modules/import-de-selection-leaves-local-sole-binding
- spec: `33 §3.2`/`§3.3` (positive selection; omission resolves a clash)
- given: `M` exports distinct `foo` and `keep`; the client declares a top-level
  `foo`, then selects only `keep` with `import M (keep)`
- expect: **accepted**. Bare `foo` resolves to the client's local `GlobalId`,
  `keep` resolves to `M.keep`, and `M.foo` is not introduced unqualified.
- why: N3 AC2, the accept side of the name-only clash flip. Relative to the
  reject fixture, the imported `foo` is omitted; a resolver that imports the
  whole module despite the positive list still clashes or resolves `foo` to the
  wrong target.

### surface/modules/per-name-rename-resolves-distinct-targets
- spec: `33 §3.2`/`§3.3` (per-name rename resolves a clash)
- given: the same `M` exports distinct `foo` and `keep`; the client declares
  its own top-level `foo` and writes `import M (foo as bar)`, then references
  both bare names
- expect: **accepted**. Bare `foo` resolves to the local declaration's
  `GlobalId`; bare `bar` resolves to the distinct `GlobalId` of `M.foo`. The
  import creates no new declaration and leaves the two target IDs unequal.
- why: N3 AC2, a structural target discriminator. A parser that confuses the
  inner `as` with module aliasing, or a resolver that binds both spellings to
  one target, cannot satisfy both target assertions. **RED UNTIL N3 LANE B.**

### surface/modules/lexical-binder-still-shadows-imported
- spec: `33 §3.3` (narrower lexical scope, innermost wins)
- given: a client selectively imports `M.foo` with `import M (foo)` and defines
  `fn pick (foo : Nat) : Nat = foo`
- expect: **accepted**. The `foo` in `pick`'s body resolves to the parameter
  (the innermost local/de-Bruijn binding), not the imported `M.foo` `GlobalId`;
  no `AmbiguousReference` is raised.
- why: N3 AC2, the negative boundary of the reversal. This differs from the
  module-level reject by moving only the competing binding into a narrower term
  scope. A resolver that over-applies the new clash rule into lexical binders
  rejects and fails this case.

### surface/modules/prelude-clash-rejected-rename-local-resolves
- spec: `33 §3.3`/`§4` (prelude is an unshadowable primitive floor)
- given: paired clients: (a) declare `def Bool = Nat`; (b) instead declare
  `def LocalBool = Bool`, leaving the registered prelude `Bool`
  untouched
- expect: (a) rejects at the surface with **`AmbiguousReference`** for `Bool`,
  identifying the local and prelude sources; (b) accepts, with `LocalBool` and
  the prelude `Bool` resolving to distinct `GlobalId`s. There is no prelude
  exclusion input or positive opt-out arm.
- why: N3 AC2. Renaming only the local changes reject to accept while the
  prelude environment is fixed. A warn-and-allow policy, silent local win, or
  resolver that aliases `LocalBool` to prelude `Bool` fails. **RED UNTIL N3 LANE
  B.**

### surface/modules/per-name-rename-parses-hiding-is-syntax-error
- spec: `32` import EBNF, `33 §3.2` (selection item rename; no `hiding` form)
- given: parse `import M (foo as bar)` and, as the controlled negative, parse
  `import M hiding (foo)` against the same exported-name fixture
- expect: the first reaches import resolution as a selective per-name rename;
  the second rejects specifically with **`ParseError`** at `hiding`, before
  module loading or name resolution. It is not `UnboundName`,
  `AmbiguousReference`, or a later elaboration error.
- why: N3 AC2 pins both grammar orientations. Treating `as` only as a
  module-level alias rejects the positive arm; admitting any exclusion
  production accepts or advances the negative arm. **RED UNTIL N3 LANE B.**

### surface/modules/import-spellings-resolve-to-one-binding
- spec: `33 §3.2` (three import forms plus selective per-name rename)
- given: `module M { pub def foo : Nat := 0 }` and four clients — `import M`
  (uses `M.foo`), `import M as N` (uses `N.foo`), `import M (foo)` (uses
  `foo`), and `import M (foo as bar)` (uses `bar`)
- expect: all four spellings **resolve to the same underlying binding** `M`'s
  `foo` (the same core `GlobalId` in the flattened `Σ`); alias, selection, and
  per-name rename are surface names for one declaration
- why: AC3 plus N3 AC2, the accept anchor. A resolver that duplicates the
  declaration per spelling perturbs the flat `Σ`; one that mistakes per-name
  rename for module aliasing resolves the fourth arm differently. Drive the
  real resolver, not a hand-constructed `M.foo → GlobalId` map. The fourth arm
  is **RED UNTIL N3 LANE B**; the retained three arms remain live.

## D. In-repo cross-file loader (N2; RED UNTIL LANE B)

These two cases are one controlled experiment. The root list, `A` unit, and
exported declaration in `B` are fixed. The reject arm changes only `B` by
adding the back-edge `import A`. Thus the observable flips from acceptance to
the named cycle rejection solely on the acyclic-versus-cyclic import-graph
axis; an implementation that never loads either file cannot make both arms
pass.

The harness supplies the resolver a **list of roots** containing exactly one
entry, `roots = [<fixture-root>]`. The spelling of the future Rust entry point
is not pinned here; its semantic input is pinned as the plural root list. Under
the strict, role-blind bijection, `A` and `B` name the unique leaf files
`<fixture-root>/A.ken.md` and `<fixture-root>/B.ken.md`. No in-file module
header or declaration-kind exception participates in resolution.

The harness designates `A` as the entry unit to elaborate in both arms. The
closed cycle is named in import-edge order rooted at that entry, fixing this
fixture's payload as `A → B → A`.

### surface/modules/cross-file-import-resolves-through-single-root-list

- spec: `33 §3.2` (N2 in-repo loader), ADR 0014 MRES-1/MRES-2/MRES-3a,
  `docs/program/07-catalog-style-guide.md §13` (total path↔file bijection)
- fixture: the resolver receives `roots = [<fixture-root>]`, with exactly these
  files:

  `<fixture-root>/A.ken.md`:

  ```ken
  import B

  const answer : Bool = B.value
  ```

  `<fixture-root>/B.ken.md`:

  ```ken
  pub const value : Bool = True
  ```

- expect: **accepted**. Loading `A` follows its `import B` edge lazily, maps
  `B` to the unique `B.ken.md` leaf under the sole populated entry of the
  plural root list, and resolves `B.value` to that file's exported `value`.
  Each unit is loaded and elaborated once in this run. **RED UNTIL N2 LANE B:**
  before the loader lands, the same fixture fails with `UnboundName` for `B`
  rather than reaching the exported declaration.
- why: this drives the real cross-file producer: the consumer does not declare
  or pre-load `B.value`, and the harness does not hand-feed an export map.
  A singleton-only API, an eager whole-tree scan, a declaration-role-dependent
  path rule, or a resolver that stops at `UnboundName` fails at least one pinned
  observation. The case exercises the plural API with one root without
  specifying multi-root precedence.

### surface/modules/import-cycle-rejected-naming-closed-path

- spec: `33 §3.2` (cycle = hard surface error), ADR 0014 MRES-2
- fixture: keep the preceding root list and `A.ken.md` byte-identical. Keep
  `B`'s exported declaration byte-identical and add only the back-edge:

  `<fixture-root>/B.ken.md`:

  ```ken
  import A

  pub const value : Bool = True
  ```

- expect: **rejected at the surface** with the specific `ImportCycle`
  diagnostic kind and the closed cycle payload **`A → B → A`**. **RED UNTIL
  N2 LANE B at diagnostic granularity:** a loaderless `UnboundName`, a warning,
  silent SCC acceptance, or a bare `is_err` does not satisfy the case. The
  diagnostic must arise from the active import-stack cycle gate before either
  unit is admitted to the flattened `Σ`.
- why: this is an absence/rejection assertion with an exact gate. The active
  load of `A` requests `B`; the active load of `B` requests `A`, so the second
  `A` closes the named cycle. If Lane B had the precise target bug — accepting
  import SCCs or omitting the active-stack check — this arm would not reject at
  that gate. The acyclic arm above disconfirms coincidental `UnboundName` or
  fixture-syntax rejection: identical `A`, root input, and `B.value` accept
  when the sole back-edge is absent.

## E. Source-world program/package admission (N4; RED UNTIL LANE B)

These fixtures use packages delivered from source through the N2 loader. This
describes their delivery form, not ADR 0014's boundary-less “source package.”
A package path is the defining-package identity reported by the oracle.
`import` remains the ordinary-name channel; `admits` is independently the
instance channel. The class and head declarations named below are ordinary
valid §5 declarations, and every non-collision provider satisfies the existing
orphan rule before the N4 gate is exercised.

For each successful dictionary lookup, the harness records the resolved
dictionary `GlobalId`, canonical `(class, head)` key, and defining package. The
literal harness field names are not pinned. Diagnostics likewise carry
structured package paths and the canonical key; matching only rendered prose is
insufficient. Every case in this section is **RED UNTIL N4 LANE B** unless
stated otherwise.

### surface/modules/two-explicit-admits-resolve-ambient-with-provenance

- spec: ADR 0014 MRES-4/MRES-4c; WP N4 AC2 (source-world admission and
  provenance)
- fixture: package `P`, delivered from source, defines class `Render`, head
  `PItem`, and the sole
  canonical `instance Render PItem`. Package `Q`, also delivered from source,
  defines head `QItem` and the sole canonical `instance Render QItem`. The
  program unit is:

  ```ken
  program
  admits P, Q

  import P (Render, PItem)
  import Q (QItem)
  ```

  Two ordinary declarations in that unit independently require
  `Render PItem` and `Render QItem`, forcing both real instance-search paths.
- expect: **accepted**. Both lookups are ambient—no per-use instance import is
  present—and return the unique canonical dictionary. The first lookup records
  `defining_package = P`; the second records `defining_package = Q`. Their
  `GlobalId`s are distinct and both provenance fields are present.
- why: this is the admitted success anchor. Removing `Q` from only the
  `admits` line must not silently keep the second success merely because
  `import Q` makes `QItem` nameable. Conversely, removing `import Q` may make
  the name unavailable but does not change which instances the boundary admits.

### surface/modules/transitive-coherence-does-not-grant-direct-dispatch

- spec: ADR 0014 MRES-4c (coherence set versus direct-use set); WP N4 AC2
- fixture: package `Q` defines the sole canonical `instance QMark QItem`.
  Package `P` is also delivered from source, declares its own boundary, uses
  that instance in one of its units, and has this anonymous package file:

  ```ken
  package
  admits Q
  ```

  The program admits only `P`, while its own unit imports the ordinary names
  from `Q` and directly dispatches `QMark QItem`:

  ```ken
  program
  admits P

  import Q (QMark, QItem)
  ```

- expect: **rejected** at instance dispatch with the specific
  `UnadmittedInstance` variant carrying
  `defining_package = Q` and `instance = (QMark, QItem)`. `Q` is nevertheless
  present in the full coherence closure through `P`; the diagnostic is not
  `UnboundName`, `MissingInstance`, `OrphanInstance`, or an overlap error.
- why: this is the two-set discriminator. A buggy gate keyed on the transitive
  coherence closure accepts; a buggy coherence pass filtered to the explicit
  root never observes `Q`. The correct implementation observes `Q` for total
  coherence but rejects the program unit's direct dispatch because `Q` is not
  in the explicit root. As the controlled accept arm, changing only the program
  line to `admits P, Q` accepts and records `defining_package = Q`.

### surface/modules/single-package-self-admits-without-program

- spec: ADR 0014 MRES-4b and package extension; WP N4 AC2
- fixture: one package `Solo`, delivered from source, contains class `SoloMark`,
  head `SoloItem`, its valid canonical instance, and an ordinary declaration
  that dispatches `SoloMark SoloItem`. There is **no program file**, no synthetic
  `admits Solo`, and no second instance-providing package in the source graph.
- expect: **accepted**. The lookup returns the canonical dictionary with
  `defining_package = Solo`; absence of a program header is not an error.
- why: this pins zero-ceremony self-admission. An implementation that requires a
  program for every build rejects; one that disables admission checking
  globally cannot also satisfy the transitive-unadmitted reject above.

### surface/modules/closure-collision-names-both-defining-packages

- spec: ADR 0014 MRES-4/MRES-4c (total closure coherence composes with
  overlap); `33 §5.3`/`§5.5`
- fixture: package `P` owns class `Render`; package `R` owns head `RItem`.
  `P` declares `instance Render RItem` at the class-owning locus and `R`
  declares a second `instance Render RItem` at the head-owning locus, so each
  declaration separately satisfies the orphan predicate. The anonymous program
  explicitly admits both packages:

  ```ken
  program
  admits P, R
  ```

- expect: **rejected** by the existing `OverlappingInstances` coherence gate
  for `(Render, RItem)`, with both candidate records present and
  `defining_package = P` and `defining_package = R`. No import order, admits
  order, or later use site may choose one silently.
- why: this pins total closure coherence and compose-not-replace. Both packages
  are admitted, so `UnadmittedInstance` is not the answer; admission cannot
  waive the pre-existing one-canonical-instance rule. A gate checking only
  direct dispatch, or filtering the coherence map, accepts until a use and
  fails this binding-time collision oracle.

### surface/modules/admission-does-not-waive-orphan-rejection

- spec: ADR 0014 MRES-4 (admission composes with orphan and overlap);
  `33 §5.3`
- fixture: the program explicitly admits package `Bad`, but `Bad` declares
  `instance Render RItem` in a module that owns neither `Render` nor `RItem`.
  The otherwise-identical control moves that declaration to the module owning
  `RItem` and leaves the program's `admits Bad` line unchanged.
- expect: the first arm rejects at declaration with the specific
  `OrphanInstance` variant and its class/head/declaration provenance; it never
  becomes a registered candidate. The relocated control passes the orphan gate
  and is eligible for N4 admission/resolution.
- why: admission is additive, not a replacement coherence policy. The pair
  changes only the declaration locus, so an implementation treating `admits`
  as permission to register an orphan flips the wrong arm to acceptance.

### surface/modules/boundary-headers-are-anonymous

- spec: ADR 0014 MRES-4e/MRES-4a; WP N4 AC2
- fixture: parse two controlled pairs with the same following `admits` section:
  bare `program` versus `program App`, and bare `package` versus `package Lib`.
  No arm contains or implies an entry-point declaration.
- expect: each bare header reaches its `admits` list. `program App` rejects with
  the specific `ParseError` variant at `App`; `package Lib` rejects with
  `ParseError` at `Lib`, both before package lookup, admission, or instance
  search. Neither `App` nor `Lib` becomes an identity or provenance label.
- why: the only identity is the file/package path and the header's presence is
  the role marker. A parser accepting documentary names creates a second,
  divergent identity source. The bare controls disconfirm a parser that simply
  rejects both new keywords.

## Coverage map (AC → cases)

- **AC1** (modules add zero to the TCB):
  `module-elaborates-to-identical-flat-sigma` (soundness).
- **AC2** (abstract export = opaque constant):
  `abstract-export-is-the-opaque-constant`,
  `client-match-hidden-ctor-rejected-at-surface` (soundness).
- **AC3** (visibility + resolution surface-only):
  `private-name-access-rejected-at-surface` (soundness),
  `import-spellings-resolve-to-one-binding`.
- **AC4** (visibility default settled): witnessed by
  `private-name-access-rejected-at-surface` (private-by-default); the OQ
  resolution itself is `/spec §33 §4` + `90-open-decisions.md`.
- **N2** (cross-file path resolution + cycle hard-error + plural-ready roots):
  `cross-file-import-resolves-through-single-root-list` and
  `import-cycle-rejected-naming-closed-path`.
- **N3** (module clash error + explicit resolution, lexical boundary, prelude
  floor, and grammar): `top-level-local-import-clash-rejected`,
  `import-de-selection-leaves-local-sole-binding`,
  `per-name-rename-resolves-distinct-targets`,
  `lexical-binder-still-shadows-imported`,
  `prelude-clash-rejected-rename-local-resolves`,
  `per-name-rename-parses-hiding-is-syntax-error`, and the renamed arm of
  `import-spellings-resolve-to-one-binding`.
- **N4** (source-world admission boundary):
  `two-explicit-admits-resolve-ambient-with-provenance`,
  `transitive-coherence-does-not-grant-direct-dispatch`,
  `single-package-self-admits-without-program`,
  `closure-collision-names-both-defining-packages`,
  `admission-does-not-waive-orphan-rejection`, and
  `boundary-headers-are-anonymous`.

## Cross-case consistency sweep

- **The kernel never sees a module (`33 §3`, `11 §4`).** AC1 (`Σ`-identity),
  AC2 (abstract = opaque constant, no kernel flag), AC3 (every reject is a
  **surface** diagnostic) are one story: **modules/visibility/abstract-export
  exist only at elaboration; the kernel sees one flat `Σ` and nothing
  module-shaped.** A case asserting a kernel-level module entry, a kernel
  "abstract" flag, or a **kernel** (not surface) visibility error would
  contradict this class.
- **Rejects are surface name-resolution, not kernel type errors.**
  `client-match-on-hidden-constructor-…` and `private-name-access-…` are one
  class: the failure is an **unresolved / not-exported** *surface* diagnostic
  that **never reaches the kernel** — never a `TypeMismatch` or an
  admitted-then-caught kernel state.
- **Import is re-naming, not re-declaration.**
  `import-spellings-resolve-to-one-binding` and
  `module-elaborates-to-identical-flat-sigma` agree: every import form
  resolves to **one** underlying `GlobalId`; a form that re-declared per
  import would perturb the flat `Σ` (contradicting AC1).
- **Module clashes and lexical shadowing are disjoint gates.** A top-level
  local plus unqualified import rejects at binding time, including when unused;
  moving only that local into a function parameter accepts and resolves the
  body structurally to the innermost binder. Neither verdict can be implemented
  by a blanket "locals win" or blanket "same spelling errors" rule.
- **Both explicit module-clash resolutions leave one target per bare name.**
  De-selection leaves local `foo` as the sole bare binding. Per-name rename
  leaves local `foo` and imported `bar` as two names for two distinct target
  IDs. Both agree with import-as-renaming and the flat-`Σ` invariant.
- **Prelude remains present in both prelude arms.** The reject arm conflicts
  with the fixed prelude `Bool`; the accept arm changes only the local spelling.
  No import list, `hiding` form, or prelude opt-out participates.
- **The N2 pair differs only by one import edge.** The same one-entry root
  list, `A` source, `B.value` declaration, strict bijection, and qualified use
  appear in both arms. With no `B → A` edge, `B.value` resolves and accepts;
  with that sole edge, the active stack closes `A → B → A` and the specific
  cycle gate rejects. No other case in this seed changes that verdict.
- **N4 keeps names, admission, and coherence as three distinct gates.**
  `import` makes a package's exported names available; it does not admit the
  package's instances. The explicit root grants direct dispatch; it does not
  filter the transitive coherence closure or waive orphan/overlap. The
  transitive reject and its one-line accept control prove the two N4 sets are
  neither conflated nor disconnected.
- **N4 provenance follows the defining declaration, never the importing unit.**
  Both admitted successes name their distinct provider packages; the
  unadmitted error names `Q`; the collision enumerates `P` and `R`. All four
  observations use structured package-path fields rather than import aliases or
  header labels.

## Subsumed / not-duplicated (one home per property)

- **`§5` constraints / typeclasses-as-subobjects** are **Lc's**
  (`../classes/seed-classes.md`, `33 §5`, landed) — ES3 touches `§3`/`§4`
  only; the orphan check's **per-module** predicate (`§5.3`) references
  modules but is Lc's home, not re-pinned here.
- **The opaque constant + the flat `Σ` + `trusted_base()`** are the
  **kernel's** (`11 §4`; `../taxonomy/minimality.md` for the delta). ES3
  observes abstract export **as** the opaque constant and modules **as**
  transparent over the flat `Σ`; the mechanisms are the kernel's home.
- **The content-addressed package manager / registry / persisted manifests**
  remain a later round (`63` supply-chain). N4 asserts only source-world
  `program` / `package` / `admits`, instance visibility, and provenance.
  Compiled-manifest source-equivalence is normative forward compatibility, not
  a live case in this seed.
- **Re-export-carried instance surfaces** remain post-MRES-9/N5. No N4 fixture
  uses `pub use` or treats a transitive provider as direct through re-export;
  adding that case before the syntax exists would be a vacuous red.
- **Runtime entry selection** is separate from admission (MRES-4a). No fixture
  invents an entry declaration or treats a `program` header as one.
- **The N3 clash/rename suite does not re-pin the loader.** Its fixtures use
  loaded module interfaces but assert only binding-time diagnostics and target
  identities. The N2 pair remains the sole home for root/path traversal and
  cycle behavior.
- **Multi-root precedence** is deferred. The N2 accept case proves only that a
  plural root input with exactly one populated entry resolves; it does not
  choose how two roots compete.

## Build-forward (N2 Lane B)

This Lane A is **spec + conformance only** (no crate). N2 Lane B implements the
in-repo loader. Its producer gate is the real import-edge traversal from the
plural root input: the accept arm flips from `UnboundName` to acceptance, and
the cycle arm rejects specifically at `ImportCycle` with `A → B → A`. The
existing `Σ` / `trusted_base()` identity (AC1), abstract-export identity (AC2),
and visibility diagnostics (AC3) remain unchanged. No hand-constructed export
map satisfies the new pair.

## Build-forward (N3 Lane B)

This N3 addition is conformance-only. Lane B adds the per-name rename parser
arm and replaces `bind_import`'s silent local-wins behavior with the specific
binding-time clash error, including latent clashes and the fixed prelude floor.
It must not change narrower lexical resolution. No N3 case reaches the kernel,
adds a declaration to `Σ`, or changes `trusted_base()`.

## Build-forward (N4 Lane B)

N4 Lane B implements only the source-world boundary. It parses anonymous
headers, forms the explicit direct-use set, retains the unfiltered transitive
coherence closure, applies one package-membership check after real instance
search, and reports defining-package provenance. All §E rejects are
surface/elaboration diagnostics and add nothing to the flat `Σ` or
`trusted_base()`. Compiled manifests, registries, lockfiles, content addressing,
re-export instance surfaces, and test-scoped admission stay unbuilt.
