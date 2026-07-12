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
substrate with an **in-repo cross-file loader**. The package manager and
cross-package imports remain out of scope.

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

Grounding (landed `§`-bodies + landed code, content-reconciled — not the
plan): `33 §3`
(`module`/`import M`/`import M as N`/`import M (foo, Bar)`; the kernel sees a
single flattened `Σ`), `33 §4` (visibility: module-private by default +
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

### surface/modules/local-shadows-imported-lexically
- spec: `33 §3.3` (local-over-imported shadowing, lexical, innermost wins)
- given: a client that selectively imports `M.foo` with `import M (foo)` **and**
  binds a **local** `def foo : Nat := 0` in the same module, then a bare
  reference to `foo`
- expect: the bare `foo` resolves to the **local** binding (innermost wins,
  lexical), **not** an ambiguity error — the local **shadows** the imported
  name
- why: AC3, shadowing is **lexical and never an error** (`33 §3.3`).
  **Discriminating:** with a local present, `foo` resolves to that local even
  though the selective import made `M.foo` available under the same bare name.
  A resolver that treated local-vs-imported shadowing as an error — or let the
  import shadow the local — fails. Pins the innermost-wins rule at the surface,
  never reaching the kernel.

### surface/modules/three-import-forms-resolve-to-one-binding
- spec: `33 §3.2` (the three import forms)
- given: `module M { pub def foo : Nat := 0 }` and three clients — `import M`
  (uses `M.foo`), `import M as N` (uses `N.foo`), `import M (foo)` (uses
  `foo`)
- expect: all three **resolve to the same underlying binding** `M`'s `foo` (the
  same core `GlobalId` in the flattened `Σ`); the alias and selective forms
  are **surface** re-namings of one declaration
- why: AC3, the **accept anchor** — the three import forms are surface
  resolution sugar over one binding. **Discriminating** (the accept side of
  §A/§C's rejects): all three reach the **identical** `Σ` binding, so a
  resolver that produced **distinct** bindings (duplicating the declaration
  per import form) fails — reinforces AC1 (import is re-naming, not
  re-declaration). Drive the **real** resolution, not a hand-constructed
  `M.foo → GlobalId` map.

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

## Coverage map (AC → cases)

- **AC1** (modules add zero to the TCB):
  `module-elaborates-to-identical-flat-sigma` (soundness).
- **AC2** (abstract export = opaque constant):
  `abstract-export-is-the-opaque-constant`,
  `client-match-hidden-ctor-rejected-at-surface` (soundness).
- **AC3** (visibility + resolution surface-only):
  `private-name-access-rejected-at-surface` (soundness),
  `local-shadows-imported-lexically`,
  `three-import-forms-resolve-to-one-binding`.
- **AC4** (visibility default settled): witnessed by
  `private-name-access-rejected-at-surface` (private-by-default); the OQ
  resolution itself is `/spec §33 §4` + `90-open-decisions.md`.
- **N2** (cross-file path resolution + cycle hard-error + plural-ready roots):
  `cross-file-import-resolves-through-single-root-list` and
  `import-cycle-rejected-naming-closed-path`.

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
  `three-import-forms-resolve-to-one-binding` and
  `module-elaborates-to-identical-flat-sigma` agree: every import form
  resolves to **one** underlying `GlobalId`; a form that re-declared per
  import would perturb the flat `Σ` (contradicting AC1).
- **The N2 pair differs only by one import edge.** The same one-entry root
  list, `A` source, `B.value` declaration, strict bijection, and qualified use
  appear in both arms. With no `B → A` edge, `B.value` resolves and accepts;
  with that sole edge, the active stack closes `A → B → A` and the specific
  cycle gate rejects. No other case in this seed changes that verdict.

## Subsumed / not-duplicated (one home per property)

- **`§5` constraints / typeclasses-as-subobjects** are **Lc's**
  (`../classes/seed-classes.md`, `33 §5`, landed) — ES3 touches `§3`/`§4`
  only; the orphan check's **per-module** predicate (`§5.3`) references
  modules but is Lc's home, not re-pinned here.
- **The opaque constant + the flat `Σ` + `trusted_base()`** are the
  **kernel's** (`11 §4`; `../taxonomy/minimality.md` for the delta). ES3
  observes abstract export **as** the opaque constant and modules **as**
  transparent over the flat `Σ`; the mechanisms are the kernel's home.
- **The package manager / cross-package imports / registry** remain a later
  round (`63` supply-chain). N2 resolves only in-repo units under its supplied
  roots. `program` / `package` / `admits`, instance manifests, package-kind
  detection, and instance visibility are N4 and are not asserted here.
- **Local/import clash and shadowing changes** are N3. The N2 pair uses only a
  qualified `B.value`, so it neither changes nor re-pins `§3.3` precedence.
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
