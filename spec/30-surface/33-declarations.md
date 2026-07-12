# Declarations, modules, and constraints

> Status: **normative for the features**; the concrete syntax spelling is
> `OQ-syntax` (proposal-level). §3 modules / imports / name-resolution and §4
> visibility (**private-by-default, settled**) + abstract-export are
> **normative** (ES3); §5's class/constraint core is landed, and §5.5.1 is the
> normative N4 source-world admission contract —
> **typeclasses-as-subobjects-of-the-universe** (Lc). The module system
> **elaborates away** to the kernel's flat append-only `Σ`: **zero
> `trusted_base()` delta** (`30-taxonomy.md §1.1`, the ES1 minimality
> invariant).

## 1. Definitions

A definition's keyword **declares its static purity** — a checked signal, not a
convention (`36 §1.6`; operator ruling, SURF-1). Three keywords, split at purity
and arity:

- **`const c … = e`** — a **pure value** (zero explicit value parameters);
  elaborates to a global definition. Subsumes the former value definition (the
  old nullary `let`). A `const` may still take **implicit** type/level/instance
  parameters — it is a constant *family* (`36 §1.6.3`), e.g. `const nil {A :
  Type} : List A = Nil A`.
- **`fn f … = e`** — a **pure function** (≥1 explicit value parameter);
  elaborates to a global Π/λ definition, gated by SCT (`../10-kernel/17 §4`).
  The verification layer may treat an `fn` as a mathematical function (`36 §1`).
- **`proc p … visits ρ = e`** — a **potentially impure/imperative** definition
  at any arity: it carries an effect row `ρ` (concrete, or a row variable, `36
  §1.5`), or is a `space` operation (`36 §4`). A `proc` is the *only* keyword a
  `visits` row (non-empty, or a variable) may sit on.
- **`def T … = …`** — a **definition**: a base type narrowed by conditions (a
  refinement/`Σ`/Π type abbreviation); a plain alias is the zero-condition case
  (transparent; unfolds by δ). Was spelled `type` before `SURF-def-refinement`;
  `type` is now reserved, not a declaration keyword.
- All top-level definitions are **mutually recursive within a module** if the
  SCT check accepts the group; otherwise the offending recursion is reported
  (`17`). This grouping **includes `lemma` and attached `proof`** declarations
  (`§8`), not only `const`/`fn`: proof declarations enter the **same**
  signatures-first, dependency-ordered SCC + SCT admission run, so a recursive
  or mutually-recursive proof is admitted **iff SCT accepts** it, on identical
  terms to `const`/`fn` (`§8.4`). A recursive cycle that **mixes** a proof
  declaration with a `const`/`fn` is **rejected** (`§8.4`, out of scope this
  iteration).
- Definitions may be **generic** (implicit type/level parameters, `39`): `fn id
  {A : Type} (x : A) : A = x`.

Every top-level definition name in one compilation unit occupies the same flat
namespace, independent of the role that introduced it. A second top-level
definition of a name already defined in that unit is a **hard surface error**
(ADR 0014, MRES-5/MRES-8). Thus `class Eq` and a data constructor named `Eq`
collide and are rejected by the same rule: types are terms, so class and
constructor names do not inhabit separate namespaces (D8-③; MRES-7).
Arity-gated surface sugar remains outside this duplicate-definition rule: the
established `Eq`/`J` sugar may coexist with lower-arity definitions of those
names (MRES-8).

The keyword is **checked bidirectionally** against the signature and the body's
inferred effects (`36 §1.6.2`); a mismatch — an `fn` that performs an effect, a
`proc` that is provably pure, a `const`/`fn` at the wrong arity — is a **hard
error** (`36 §1.6.3`). The single definition keyword `view` is **retired**
(`36 §1.6.4`); the local `let … in …` **expression** (`32 §3`) is unchanged.

## 2. Records (products)

```
record Point { x : Int, y : Int }
record User  { name : String, age : { n : Int | n ≥ 0 } }   -- refined field
```

- Elaborates to right-nested Σ with definitional η (`../10-kernel/13 §3`), so
  field access `p.x`, record literals `{ x = 1, y = 2 }`, punning `{ x, y }`,
  and **functional update** `{ p | y = 3 }` all have their expected definitional
  behaviour.
- Fields may be **dependent** (a later field's type mentions an earlier field)
  and **refined** (carry a proposition) — records are Σ, so this is free.

## 3. Modules and imports

A **module** is a pure surface namespacing + information-hiding device. It
**elaborates away**: after name resolution the kernel sees the single flat,
append-only global environment `Σ` (`../10-kernel/11 §4`) it would see for the
same program written in one fully-qualified namespace. Modules, imports, and
visibility are **surface + elaboration-time only** — they add **no** kernel
feature and **nothing** to `trusted_base()`. The ES1 minimality invariant
(`30-taxonomy.md §1.1`, surface built-in set ≡ `trusted_base()` delta) carries
verbatim: a module program's trusted-base delta is **identical** to its
flattened equivalent's.

### 3.1 Declaring modules

**`module M { … }`** groups declarations under the namespace `M`. A file is an
implicit module named by its path. Modules **nest**
(`module M { module N { … } }` gives `M.N`). A module is an **environment
fragment**: its declarations elaborate into `Σ` in dependency order under their
qualified names, exactly as if written flat.

### 3.2 Importing

Within a module, an `import` brings another module's **exported** names (`§4`)
into scope. Three forms:

- **`import M`** — qualified: `M`'s exports are accessible as `M.foo`, `M.Bar`.
- **`import M as N`** — aliased: the same, under `N.foo` (`M` itself unbound).
- **`import M (foo, Bar)`** — selective: exactly `foo`, `Bar`, brought
  **unqualified**; nothing else of `M`.

A selective item is either a name or a per-name rename. Thus `import M (foo,
Bar as Baz)` brings `foo` unqualified and brings `M.Bar` unqualified under the
name `Baz`; it does not also bind `Bar`. The per-name `as Baz` is inside the
selection list and is distinct from the module alias in `import M as N`.

For in-repo compilation units, dotted module paths and source-file paths obey a
total, role-blind bijection under a catalog root. A path with `N` components
names the unique leaf source file reached through `N - 1` directories: for
example, `import Data.Collections.Map` resolves exactly one of
`Data/Collections/Map.ken` and its `.ken.md` form. A path component is treated
only as a module component; resolution does not depend on whether declarations
in the file are values, types, constructors, classes, or any other role. A path
position is a leaf or a directory, never both. This is the catalog taxonomy
WP's pinned path/import identity and strict leaf-file rule (ADR 0014,
MRES-2/MRES-3(a strict)).

Resolution takes a **list of catalog roots**. This round populates that list
with exactly one in-repo root; resolution and precedence among multiple roots
remain deferred to the package-manager round (MRES-1(a)/MRES-2). The plural form
is nevertheless the normative resolver input, so adding roots changes the
input data rather than the module-path contract.

The loader discovers compilation units lazily, following `import` edges from
units already being compiled; it does not scan a catalog tree eagerly. Each
unit is loaded and elaborated at most once in a compilation run, and later
imports reuse the per-run cached result. An import edge to a unit already on
the active import chain is the hard surface error **`ImportCycle`**. The
diagnostic names the closed cycle in edge order rooted at the entry unit. The
conformance harness elaborates `A` as its entry unit, so its cycle payload is
`A → B → A`. Cycles are not accepted as recursive module groups (MRES-2).

The in-repo loader discovers the source graph; the `admits` / `program` /
`package` boundary below adds the instance-admission rule over that graph. The
compiled-package manifest and content-addressed package manager remain a later
round. Multi-root precedence is likewise deferred. The module-level clash and
narrower lexical-shadowing rules are specified in §3.3; neither the loader nor
the admission boundary alters either rule.

#### 3.2.1 Admission-boundary headers

An anonymous **`program`** header marks its file as the admission root for a
multi-package build. An anonymous **`package`** header marks its file as a
package admission boundary. The only payload either header may carry in this
round is an `admits` section listing dotted package paths using the same
role-blind path identity as `import`:

```ken
program
admits Core.LawfulClasses, Data.Collections.Map
```

Neither header takes a name token. The file path is the single identity of the
program or package boundary; a spelling such as `program App` or `package Lib`
is a syntax error. The header's presence is the signal, and an ordinary comment
may carry documentary intent without creating a second identity.

`program` and `package` establish elaboration-time instance-admission
boundaries only. They neither designate nor declare a runtime entry point. An
entry declaration is a separate construct that a program file may eventually
co-host; this section defines no entry syntax (MRES-4a/4e).

### 3.3 Name resolution (surface-only; never reaches the kernel)

Resolution is a **surface / elaboration** pass; a name still unresolved after it
is a **surface error** (`24`) — it never reaches the kernel:

- **Qualified / aliased / selective import targets** each identify exactly one
  declaration before collision checking: `M.foo`, `N.foo`, and a selectively
  imported or renamed name retain the imported declaration's identity.
- **Top-level local/import clash.** If one unqualified name is bound both by a
  top-level local definition and by an import, resolution raises the surface
  error **`AmbiguousReference`**. Silent local-wins is not permitted. The check
  is order-independent and fail-closed: it runs whether or not any expression
  references the clashing name. The source must leave the imported name out of
  the selective list or rename it per-name at the import site, so that only one
  declaration retains the original unqualified name.
- **Narrower lexical shadowing.** A `λ`, `let`, parameter, or pattern binder in
  a narrower lexical scope still shadows an outer or imported name. Resolution
  is lexical (innermost wins) and is never a module-level clash error; this
  term-language rule is orthogonal to the preceding top-level rule.
- **Prelude floor.** Prelude names are always-present unqualified bindings and
  cannot be shadowed by top-level local definitions. Such a collision raises
  **`AmbiguousReference`** and is resolved by renaming the local definition.
  There is no form that excludes or renames a prelude binding; keeping this
  floor deliberately small bounds its collision surface.
- Every failure — unresolved name, **`AmbiguousReference`** from a top-level
  clash, or an out-of-scope private name (`§4`) — is a **surface diagnostic**;
  the flattened `Σ` the kernel receives contains only resolved, in-scope
  references.

## 4. Visibility and abstract export

### 4.1 Visibility — private by default, `pub` to export

Top-level names in a module are **module-private by default**; **`pub`** exports
them. The `pub` names form the module's **interface**; a non-`pub` name is
invisible outside its module, and accessing it from outside is a **surface
error** (name not in scope), *not* a kernel error.

**The default is private — settled (was `OQ-syntax`).** Rationale:
private-by-default is the **least-surface, information-hiding-forward** choice —
a module exposes only its intended interface, so the coupling surface a client
depends on is exactly the `pub` set: small and auditable (the module-level echo
of small-auditable-TCB). Accidental exposure requires an explicit `pub`, never
an explicit hide; and it **matches abstract export** (`§4.2`), the same
information-hiding default one tier down (hide the constructors). The inverse
(public-by-default) is **not** taken. `../90-open-decisions.md` records the
visibility default as resolved — no longer part of the `OQ-syntax` iterating
set.

### 4.2 Abstract export — the opaque constant, not a new mechanism

A type may be exported **abstractly**: its name is `pub`, its **constructors are
not**. Clients see the type but cannot `match` on or construct its hidden
constructors. This is **exactly** the kernel's existing **opaque constant**
(`../10-kernel/11 §4` — an opaque `c : A` is "how … abstract interfaces are
represented") — information hiding with **no new kernel feature**:

- It is enforced at **elaboration** (surface): the hidden constructors are
  simply **not in scope** at the client, so a client `match` on one is a
  **surface error** (name not in scope), never a kernel rejection.
- The abstract type's **kernel representation is byte-identical** to a
  hand-written opaque constant — there is **no** kernel "abstract" flag and no
  visibility concept in `Σ`. A design that added a kernel-level
  abstract/module/visibility primitive (a new `trusted_base()` entry) is
  **rejected** by this spec; the elaborates-away form is the whole mechanism.

This is the AC1/AC2 invariant made concrete: `module` / `import` / `pub` /
abstract-export cost the trust root **nothing** — surface namespacing +
information-hiding over the unchanged flat `Σ`.

## 5. Constraints — typeclasses as subobjects of the universe

> **Status of this section: impl-ready (Lc).** The surface grammar is `32 §1`
> (`class`/`instance`/`where`/`derive`); the **elaboration shapes** and the
> **coherence policy** are pinned here to algorithm level; the **resolution
> algorithm** (constraint insertion, instance search, its termination metric,
> and the diagnostics) is `39 §6`. **`OQ-classes` DECIDED, ADR 0008.** **No new
> kernel feature** — a class is a **record** (`../10-kernel/13 §3` Σ + η), a law
> is an **Ω** proposition (`../10-kernel/16 §1`), search termination is the
> **landed SCT** (`../10-kernel/17 §4`) via the reified-dictionary definition
> (`39 §6.4`). If the build adds a kernel rule/judgment/"class" former, it has
> **mis-scoped**.

Ken's constraint/trait mechanism is **structure on types**. A "class" is a
structure `C : Type → Type` (its members carve out a subobject of the universe,
`../10-kernel/12 §5`); an "instance" exhibits that a given type carries it. This
"typeclasses-as-subobjects" design is the most category-faithful account of open
user typeclasses.

```
class DecEq (A : Type) {              -- a record of operations + their laws
  eq    : A → A → Bool                 -- (the propositional equality is the
  ok    : (x y : A) → eq x y == true → Eq A x y   --  kernel's Eq, 10-kernel/15)
}
instance DecEq Int { eq = int_eq, ok = … }

fn nub {A : Type} (xs : List A) : List A  where DecEq A = …   -- a constraint
```

### 5.1 The two kinds — and the sort *is* the discriminant

Two kinds of class, split by **where the dictionary lives** — the distinction
that governs coherence (`OQ-classes`, ADR 0008):

- **Property classes** (Ω-valued: `Decidable p`, `IsHom f`). Proof-irrelevant
  (`../10-kernel/16 §1`), so **any two instances are definitionally equal** —
  the subobject framing is literal, and **coherence is free** (the kernel
  guarantees it; *no resolver convention applies*).
- **Structure classes** (`Type`-valued, dictionary with computational content:
  `DecEq`, `Monoid`, `Ord`). Genuinely *many* can exist on one carrier (ℤ under
  `+` and under `×`), so the subobject reading is "∃ a dictionary" and coherence
  is a **resolver convention** (§5.5), not a theorem.

**The discriminant is the class record's kernel-computed *sort*, not an author
flag (`AC4`).** A class elaborates to a right-nested Σ (§5.2); the kernel's
`sort_sigma` is **both-components-keyed** — `sort_sigma(s₁, s₂) = Ω` **iff**
`s₁ = Ω ∧ s₂ = Ω` (`../10-kernel/13 §4`). So the whole record lands in **Ω iff
*every* field is Ω-valued** (a property class), and in **`Type`** the moment
**any** field is relevant/`Type`-valued — an operation like `eq : A → A → Bool`
(a structure class). The elaborator reads the sort off the emitted record; it
does not carry a separate kind tag.

**Soundness note — never force a structure class into Ω (`AC4`, Architect
gate).** A class with a relevant operation field is `Type`-sorted; classifying
it Ω to "get coherence for free" would fire Ω-PI (`16 §1`) on the whole
record and make its *computational* content proof-irrelevant — two
observationally-distinct dictionaries would be definitionally equal, collapsing
the very content the prover's lemmas depend on. This is the Σ-sort trap
(`13 §4`: sending a relevant-carrier Σ to Ω is unsound): the
both-components-keyed `sort_sigma` is what *prevents* it, so the discriminant
must be the real kernel sort, computed over **all** fields — a mis-keyed
(codomain-only) sort is the soundness bug the Architect gates.

### 5.2 Class declaration → a record type

`class C (A : Type) { op₁ : T₁ ; … ; law₁ : P₁ ; … }` elaborates to a **record
type** — the **right-nested Σ** over the field telescope (`../10-kernel/13 §3`),
parameterised by the class head `A`. A class field may also carry an optional
leading purity keyword:

```
class Traversable (f : Type → Type) {
  functor  : Functor f
  foldable : Foldable f
  proc traverse :
    (g : Type → Type) → Applicative g → (a b : Type) →
    (a → g b) → f a → g (f b)
}
```

The field forms are:

```
class_field ::= field_name : type
              | (const | fn | proc) field_name : type
```

An unmarked field is **unclassified** and keeps the pre-SURF-2 behaviour: the
instance field is checked only against its declared type, and projection carries
no extra static-purity signal. A marked field's keyword is a **checked
signature** over the field declaration itself, reusing the definition-level
SURF-1 purity discipline (`§1`, `36 §1.6`): `const`/`fn` fields must be pure
with the corresponding explicit-value-arity classification, and `proc` fields
must be potentially effectful/effect-row-polymorphic under that same
classifier. Because class fields have no separate value-binder list, the
arity/effect telescope is read from the field's **declared type**; implicit
parameters still do not count (`36 §1.6.3(b)`). A later instance expression must
satisfy the stored field classification — by **covariant subsumption**, not
exact equality: a pure witness inhabits a `proc` field, but the reverse (an
effectful witness for a `fn`/`const` field) rejects (`36 §1.6.2`, DS-8b) — but
it cannot redefine what the class field marker meant. The marker is
surface/elaboration metadata attached to the field declaration, not a field of
the class record.

**AC4 non-interference.** A class-field purity marker does **not** enter the
Type/Ω discriminant. The class's sort is still the kernel-computed sort of the
right-nested Σ over the field **types** alone (§5.1): an operation type such as
`traverse : …` is relevant because of its type, not because the author wrote
`proc`; an Ω law field remains Ω because its type is Ω, not because it is
unmarked. The elaborator may store `Some(proc)`/`Some(fn)`/`Some(const)` beside
the field for checking and projection, but it must not feed that marker into
`sort_sigma`, class-kind selection, or instance-coherence policy.

The record type itself is unchanged:

```
C A  ≡  (op₁ : T₁) × … × (opₙ : Tₙ) × (law₁ : P₁) × … × (lawₘ : Pₘ)
```

- **Operation fields** are `Type`-valued (`eq : A → A → Bool`); **law fields**
  are **Ω-valued propositions** (`../20-verification/21 §3` `law`/`verify`;
  e.g. `assoc : (x y z : A) → op x (op y z) == op (op x y) z`). A mixed record —
  relevant ops beside Ω proofs — is well-formed and lands in `Type`
  (`13 §4`); a record whose fields are *all* Ω is itself a proposition (the
  sound Σ-of-Ω-into-Ω case, `16 §1.3`) — that is exactly a **property class**.
- **Definitional η (`13 §2`)** gives the record its expected behaviour: a
  dictionary `d : C A` satisfies `d ≡ (d.op₁, …, d.lawₘ)`, so field projection
  and reconstruction round-trip definitionally — the prover cites `d.assoc`
  directly (`§5.3`, `AC8`).
- The record's **sort** (Ω vs `Type`, §5.1) classifies the class; the kernel
  computes it — no new former, just a Σ over the existing telescope machinery.

### 5.3 Instance declaration → a record value (+ the orphan check)

`instance C T { op₁ = e₁ ; … ; law₁ = p₁ ; … }` elaborates to a **record value**
of type `C T` — a right-nested pair (`13 §2` Σ-Intro) of the operation
implementations **and the law proofs**:

```
inst_C_T : C T  ≡  (e₁ , … , eₙ , p₁ , … , pₘ)
```

Each `pⱼ` is a **real kernel proof** of `Pⱼ[T/A]`, checked at its Σ-Intro
position `B[a/x]` (`13 §2`) — not a stub. A `Monoid Int` instance therefore
carries genuine `assoc`/`unit` proofs the prover can **cite** (`AC8`): the
lawful-by-construction dictionary is the verification win. The value is admitted
through the real `declare_def` path (`../10-kernel/…`, `check.rs`), so the
kernel re-checks the ops *and* the proofs.

**The orphan check — at declaration, per-module (`AC2`).** An `instance C T`
declaration is **accepted only if it mentions its class `C` or its head-type
`T`'s constructor** in the module that declares it; an instance that names
**neither** (an *orphan*) is a **hard error at the declaration site**
(`39 §6.5`). This is a purely **syntactic, per-module** predicate on the
declaration — decidable without whole-program information — and it is what keeps
canonicity (§5.5) *un-break-able by accident*: because every instance is
co-located with either its class or its head-type, the canonical instance for a
`(class, head-type)` pair is discoverable from those two modules alone. The
check is an **elaborator** check (it constrains *where* a well-typed value may
be declared), not a kernel rule.

### 5.4 Constraint `where C A` → an implicit instance argument

A constraint `where C A` on a definition (`fn`/`proc`/`const`, grammar `32 §1`)
elaborates to an **implicit instance argument** — an implicit `Π` over the class
record inserted ahead of the explicit parameters:

```
fn nub {A : Type} (xs : List A) : List A  where DecEq A = …
      ⟶  nub : {A : Type} → {d : DecEq A} → List A → List A
```

The `{d : C A}` binder is threaded like any implicit (`39 §2.2`): at a **use
site** the elaborator inserts a metavariable for `d` and **discharges it by
instance search** (`39 §6`). Inside the body, a class operation `eq x y` is
`d.eq x y` (projection from the resolved dictionary). Multiple constraints
`where C A, D B` (comma-separated, `32 §1`) insert one implicit each, left to
right. The concept is ordinary dependent-implicit insertion — the *resolution*
of `d` is the only new step, and it lives in `39 §6`.

**Multiple constraints — one dictionary per constraint, deterministically
named (shared by the definition path and instances).** This multi-constraint
contract — the deterministic `d<v>` naming, explicit named binders, and
comma-separated list below — is the **shipped surface of every `where`-clause**:
both the **definition path** (`const`/`fn`/`proc`, and the legacy `view`) and the
**instance** `where`-clause (`instance C Head where …`) parse through the **one
shared production** (`32 §1`'s `constraint_clause`; the landed def-path
constraint-binder unification). The definition path additionally retains `;` as a
separator for existing declarations (`37 §6`, L3b) and instances additionally
tolerate a trailing `,` before `{` — spelling compatibilities, not distinct
grammars. The multi-constraint, `d<v>`, and explicit-binder cases below therefore
hold **uniformly on definitions and instances alike**.

On an instance, with more than one constraint each dictionary needs a distinct,
deterministic name, and the singular `d` generalizes. A constraint **of the form
`C v`** — a class applied to a **single type variable** `v` (`32 §1`:
`constraint ::= ConId atype+`) — binds its dictionary as **`d<v>`**, the reserved
prefix `d` immediately followed by that variable's identifier, projected by
explicit `.field`:

```
instance DecEq (Pair a b) where DecEq a, DecEq b { … }
   ⟶  {a b : Type} → {da : DecEq a} → {db : DecEq b} → DecEq (Pair a b)
      -- fields project da.eq / db.eq, da.sound / db.complete
```

- **Uniform, deterministic, projection-preserving.** The name is a pure
  function of the constraint's type argument — `DecEq a → da`, `DecEq b → db`,
  `Ord a → da` — identical for one or many constraints, and identical on the
  definition path and the instance `where`-clause (the shared production above).
  Explicit `.field` projection is unchanged (`da.eq`, `db.complete`);
  only the *name* generalizes. This is the coherent completion of the singular
  model — a named dictionary per constraint, **not** one reserved `d` for many,
  and **not** type-directed member resolution (a distinct, implicit paradigm;
  reflect-don't-extend).
- **Source-order binding (the contract).** Constraints bind **left to right**:
  the i-th `where` constraint is the i-th implicit `Π` position, and `d<v>`
  denotes exactly that one dictionary at that one position. The field body's
  `d<v>` and the elaborated type's `Π`-position must agree by this order; a
  mismatch elaborates a `Σ`-dictionary that fails its declared type and the
  **kernel rejects it** — the elaborator is untrusted, so a naming/position bug
  is **fail-closed** (rejects a good dictionary, never admits a bad one).
- **Singular reconciliation — `d<v>` canonical, bare `d` the retained
  sole-constraint spelling.** The uniform rule makes the single-constraint case
  `d<v>` on **both** paths (`where DecEq a` → `da`, `da.eq`). The reserved bare
  `d` is **retained for the sole-constraint case on the definition path and on
  instances alike** (`37 §6`/`51 §4`, L3b) — so landed catalog proofs that
  project it (`Core/EmptyDec`'s `d.eq`/`d.sound`/`d.complete`,
  `Core/LawfulClasses`'s `d.leq`) stay valid — and it stays available even when
  that sole constraint is written with an explicit named binder (`where (chosen :
  Flag Int)` still admits `d`). With **two or more** constraints, bare `d` is not
  bound — every dictionary is its `d<v>` auto-name or explicit name.
- **Explicit named binders — required wherever the auto-name is unavailable.**
  The `d<v>` auto-name is defined **only** for the single-type-variable form
  `C v`. **Any other grammatical constraint (`32 §1`) takes no auto-name and
  requires an explicit binder** `where (name : C τ)`. Two cases arise:
  - a **compound- or multi-argument** constraint — `where DecEq (List a)`,
    `where C a b` — has no single variable `v` to key on;
  - a **same-variable collision** — `where DecEq a, Ord a`, where two
    constraints would both auto-name `da`.
  In each, the surface **requires** the explicit form — `where (dla : DecEq
  (List a))`; `where (da : DecEq a), (oa : Ord a)` — projected by the same
  explicit `.field` (`dla.sound`, `oa.leq`); a **bare** `where` in these cases is
  a **surface error** (no auto-name exists, or an ambiguous one). The
  named-binder form `where (name : C τ)` is available generally — any dictionary
  may be user-named — and explicit binder names must be **pairwise distinct** (a
  duplicate is a surface error); an unnamed single-type-variable constraint takes
  the deterministic `d<v>`. So the common `d<v>` names stay stable: reaching for
  a compound/multi-arg or a same-variable constraint is a deliberate move to
  explicit names, never a silent rename of an existing `d<v>`.

(This is the `33 §5.4` naming contract the `constrained-instance-elaboration`
capability co-lands against: the Architect ruled the model — deterministic
named per-constraint projection — and deferred this user-visible spelling to
Spec.)

### 5.5 Coherence policy (`OQ-classes`, ADR 0008 — do not reopen)

For **structure** classes, the resolved dictionary is **semantically
load-bearing** (it carries law proofs the prover *uses*), so implicit resolution
is governed by a canonicity convention — the property a client lemma about "the
`Monoid A`" relies on:

- **One canonical instance per `(class, head-type)`** participates in implicit
  search: "the `Ord A`" is a **function of `A`**, stable program-wide (`AC1`).
- **No overlapping instances**; **ambiguity is a compile error naming both**
  candidates (`AC3`), never a silent pick.
- **Orphans are rejected at declaration** (§5.3, `AC2`) — the structural
  precondition that makes canonicity per-module-decidable.

For **property** classes none of this applies: proof irrelevance (`16 §1`) makes
**any** instance do — all are definitionally equal, so resolution may return the
first found and two instances **never** conflict (`AC4`). The policy split is
**exactly** the sort split of §5.1 — the resolver consults the class's sort to
decide whether canonicity is even in play.

**Named instances are first-class values, passed explicitly (`AC5`).** Because
an instance *is* a record value, you may define a **non-canonical**
`byLength : Ord String` and pass it explicitly (`sortBy byLength xs`) — the
dependent-types escape hatch Haskell lacks (no `newtype` gymnastics). Explicit
passing is **ordinary value application** at the dictionary `Π`; it **bypasses
search** and therefore **does not perturb** implicit canonicity: at the same
type, *implicit* `where Ord String` still resolves to the **canonical**
`Ord String`. The resolver may pick only one canonical thing silently; you may
deliberately use any value. That split is the whole point.

#### 5.5.1 Program/package admission and cross-package coherence

Instance resolution is ambient only inside an explicit admission boundary. For
each `program` or `package` boundary, the elaborator computes two distinct
sets:

- The **coherence set** is the unfiltered transitive closure of the boundary's
  complete source graph, seeded by its own units and admitted roots. Every
  structure instance in this closure remains subject to §5.5; property-class
  instances remain proof-irrelevant and do not conflict. The closure is total:
  neither selective import nor any other name-resolution form removes a package
  from it. In one acyclic source graph, the orphan rule constructively ensures
  that at most one package may legally define a given `(class, head-type)` key.
  The existing §5.5 overlap check retains its intra-package duplicate coverage;
  no separate source-world cross-package collision can arise.
- The **direct-use set** contains the boundary's self-admitted package, when
  applicable, plus the packages named by its explicit `admits` section. An
  instance that one of the boundary's own units dispatches must be defined by a
  package in this set. Transitive membership in the coherence set does not grant
  dispatch rights.

After implicit search selects an instance, the elaborator checks its defining
package against the current boundary's direct-use set. A miss is the hard
surface error **`UnadmittedInstance`** and names both the defining package and
the selected instance. Thus reaching directly for an instance that was present
only because an admitted dependency used it makes its provider a direct
instance dependency that must itself be listed in `admits`.

This admission check is additive to the existing rules. The orphan check still
runs at the instance declaration (§5.3), and the §5.5 overlap check still
rejects a second canonical structure instance within the defining package.
Admission cannot make an orphan or overlapping instance acceptable; conversely,
passing orphan and overlap checks does not admit an instance for direct use.

**Self-admission and when a root is required.** A single package implicitly
self-admits its own instances, so single-package and catalog development need
no `program` file. A `package` boundary also self-admits its own package and
uses its `admits` roots as its direct instance dependencies, so a
library-with-dependencies is buildable and testable in isolation. A build that
combines two or more instance-providing packages across unit boundaries and is
not already rooted at a `package` boundary requires a `program` file to make
the direct-use choice explicit. Transitive dependencies used only inside an
admitted package flow into the coherence set automatically and are not repeated
in the parent's `admits` list.

**Provenance is observable.** A successful implicit resolution reports the
defining package alongside the selected instance. `UnadmittedInstance` reports
the unlisted defining package and instance. Diagnostics must retain this
provenance through source loading; package identity is the dotted path fixed by
§3.2.1, never a header label. Both-package collision provenance belongs to the
compiled-manifest/package-manager boundary specified below, where such a
collision can genuinely arise.

**SPEC-NOW / BUILD-LATER package rules.**

The following rules are normative forward-compatibility requirements, but are
**not part of the current source-world implementation round**:

- A package file explicitly enumerates its member modules. Membership is
  explicit, while package identity and its root remain path-inferred
  (MRES-4e/MRES-2b). The concrete member-list grammar and spelling, and the
  corresponding build, are deferred to the package-manager round. Until that
  round, the admission gate operates over the existing N2 path-based source
  graph; no member-list keyword or production is introduced here.
- In the package-manager round, a compiled package records an instance manifest
  containing the canonical instances its own boundary commits to. Loading that
  compiled package and rebuilding it from source must contribute the same
  instance environment and produce the same admission and coherence outcomes.
  The manifest is generated from the package's own `admits`; delivery from
  source never substitutes the parent's boundary for that declared boundary.
  The one `admits` relation targets the package regardless of whether its
  delivery is source or compiled; authors do not select a delivery form.
- A parent trusts an admitted compiled package's manifest for that package's
  internally checked commitments and re-checks only cross-boundary coherence.
  At this admission boundary, a genuine canonical-instance collision across
  packages is a hard error that names both defining packages and the conflicting
  `(class, head-type)` key.
  The kernel still re-checks every instance dictionary value, so there is no
  new TCB. Signed or attested manifest validation belongs to the package-manager
  and supply-chain round.
- When public re-export lands after MRES-9, re-exporting a name also carries the
  instance surface that the re-exported name's public API commits to. Those
  carried instances enter the admitting consumer's direct-use set. A transitive
  instance not carried by a re-export remains coherence-only; direct dispatch
  still requires admitting its defining package.
- Persisted content-addressed manifests, registries, lockfiles, and
  supply-chain validation are package-manager concerns. Test-scoped admission
  is likewise deferred; the package's ordinary `admits` section is the only
  test boundary specified in this round.

No compiled manifest, registry, lockfile, re-export handling, or test-only
admission syntax is required by the current source-world build. These deferred
rules do not widen the direct-use set until their respective surface and
package-manager mechanisms exist.

### 5.6 `derive` — an untrusted, kernel-re-checked candidate

`derive (DecEq, Show)` on a `data`/`record` (grammar `32 §1`) requests an
**elaborator-generated structural instance**. Generation is **untrusted**: the
elaborator builds a candidate instance *value* (structural recursion over the
type's constructors) and emits it through the **real `declare_def` path**, where
the kernel **re-checks** the ops and law proofs like any other instance (§5.3).
A malformed generated instance is therefore **caught by the kernel**, never
admitted by a trusted insertion (`AC7`, soundness) — `derive` is a *convenience
that cannot widen the trusted base*. Which classes are derivable is a fixed
structural list (`DecEq`, `Show`, …, §`39 §6.6`); a class needing non-structural
content is not `derive`-able and must be written by hand.

### 5.7 What is landed vs. net-new (honest scope)

Classes/instances are **entirely net-new** surface + elaborator machinery; the
Lc build creates the class/instance desugaring, the orphan check, and the
search. What it **reuses unchanged** (the real producers the soundness-critical
ACs bottom out in) is all **landed**: the kernel **Σ/record** primitives
(`Term::Sigma`/`Pair`/`Proj`, `13 §2`/`§3`) the class-record/instance-value
target; **Ω** proof-irrelevance (`16 §1`) that makes property coherence free;
the **`declare_def` re-check path** (`check.rs`) every instance — and every
`derive`d candidate (`AC7`) — traverses; and the **landed SCT** (`17 §4`) that
bounds recursive-instance resolution once it is reified as a dictionary
definition (`39 §6.4`, `AC6`). **No new kernel rule, judgment, or former** —
subsume-don't-proliferate.

The program/package admission gate of §5.5.1 is a net-new source/elaboration
layer over the N2 loader graph. Its current build contract stops at anonymous
headers, the constructive source-coherence invariant, direct-use admission,
self-admission, and resolution/admission provenance. Explicit package
membership, compiled manifests, cross-package collision detection and
both-package provenance, public re-export propagation, registries, lockfiles,
and test-scoped admission remain the clearly marked SPEC-NOW / BUILD-LATER rules
above.

## 6. Fixity and operators

`infixl N op` / `infixr N op` / `infix N op` declare operator fixity (`32 §6`).
Operators are ordinary `fn`/`proc`/`const` definitions with symbolic names;
there is nothing special about them semantically.

## 7. What WS-L must deliver here

Definitions (incl. generic + mutually recursive under SCT), records (dependent +
refined fields, update/pun), the module/import system + package manager with
content-addressed lockfiles, visibility/abstraction, and the
class/instance/constraint mechanism with **lawful** instances and `derive`.
Conformance: `../../conformance/surface/declarations/`.

## 8. Named proof claims — `prop`, `lemma`, and attached `proof`

These declarations are surface/elaboration vocabulary over existing checked
terms only. They add no new kernel declaration class, no trusted proof table,
and no ambient proof search. The proof-claim surface has three roles:

- `prop` names a proposition family / claim shape.
- `lemma` names a reusable standalone checked proof theorem.
- `proof <name> for <subject>` names a checked proof term attached to a
  resolved subject API.

### 8.1 Proposition families — `prop`

`prop` is the claim-shape spelling for an `Omega`-valued family. A `prop`
declares a family whose telescope ends in `Omega`; the declaration is private
by default and may be exported with `pub prop`.

The family can carry an optional constructor-style `where` block:

```ken
prop AppendsTo (A : Type) : List A -> List A -> List A -> Omega where {
  nil  : AppendsTo A xs nil xs;
  cons : ...
}
```

The `where` block elaborates to ordinary checked introduction helpers over an
`Omega`-clean encoding. The public proposition remains proof-irrelevant; the
elaborator may keep an internal witness relation, but only the public `Omega`
proposition and checked intro helpers escape. If the elaborator cannot produce
an `Omega`-clean checked encoding, it rejects the `prop`.

Intro helper names are not bare module names. They are addressed canonically
through the family name, as `AppendsTo.nil`, `AppendsTo.cons`, and so on, and
import/export follows the family's visibility. A bare `nil` in the module
namespace is still just the ordinary value constructor if one exists; `prop`
does not reserve that bare space.

### 8.2 Attached proofs — `proof`

Attached proofs are checked proof definitions attached to a resolved subject.

```ken
proof appends for list_append
  (A : Type) (xs : List A) (ys : List A)
  : AppendsTo A xs ys (list_append A xs ys) = ...
```

The canonical path is `subject::proof_name` - for example
`Collections.List.list_append::appends`. The equivalent selector atom is:

```
proof_ref ::= "proof" ident "for" path
```

A `proof_ref` is a primary expression atom. Its subject is exactly one `path`,
so application binds outside the selector: `proof p for s a b` parses as
`((proof p for s) a) b`. The bare form `proof appends for list_append` and the
grouped form `(proof appends for list_append)` produce the identical
`Expr::EAttachedProofRef { subject, proof_name }` expression and desugar to the
same `subject::ident` global, where `ident` is the proof name. Parentheses are
optional grouping, not part of the selector atom.

Subject resolution runs first; attached lookup runs only after the subject is
resolved. A bare `appends` never resolves to the attached proof. This
expression-position atom does not change the declaration-position `proof`
head described above.

An attached `proof p for s` is well-formed iff the subject `s` **occurs
applied** somewhere in the proof's claim type φ — in a hypothesis or the
conclusion. The reading is broad: `proof p for s` names "a checked property
**of** `s`," not a proof whose telescope mirrors the subject's. A claim φ that
never mentions `s` applied is rejected before the proof is used. This is a
**surface well-formedness** condition only: attachment is namespacing over an
already-checked theorem (`§8.4`), so the theorem is checked identically whether
or not the condition holds — it carries **zero soundness weight**. The subject
must still be an **already-resolved definition** (a real precondition; a bare
or unresolved subject rejects). Duplicate proof names on the same resolved
subject reject. The same proof name may appear on different subjects because
the canonical paths differ.

Attached proofs are private by default, just like other declarations. `pub
proof p for s` is exportable only if `s` is exported. Importing an exported
subject makes its exported attached proofs available only through the explicit
attached path (`s::p`, or `M.s::p` under qualified import); it does not
ambiently import `p` as a bare value.

Same-subject attached proofs are **ordinary dependencies of one another**. An
attached proof **may** reference a sibling `subject::q` (directly or through a
helper): proving `s::antisym` via `s::trans`, or a mutually-recursive sibling
pair, is natural and admitted. Because proof declarations now go through the
shared SCC + SCT admission run (`§8.4`), a sibling reference is just an edge in
the dependency graph — an acyclic reference resolves in dependency order, a
mutual sibling cycle is admitted **iff SCT accepts** it, and a non-terminating
one **fails closed** with the SCT diagnostic. (The earlier blanket
no-sibling-dependency rejection is **withdrawn**: under the SCC + SCT run it
was both redundant and wrong.)

### 8.3 Standalone lemmas — `lemma`

`lemma` is the standalone checked proof-definition form. It is a reusable proof
theorem in the ordinary module namespace, parameterized like a function and
instantiated by ordinary application.

```ken
lemma append_nil_right
  (A : Type) (xs : List A)
  : AppendsTo A xs nil xs = ...
```

The result annotation is required and must classify at `Omega`. The body is an
ordinary checked proof term; `lemma` is not a bundled `prop + proof`, not an
attached proof, and not a new kernel concept. If authors want an open
obligation, the existing `prove` path remains the status-bearing form.

A `lemma` (or attached `proof`) body **may self-recurse** — structural
induction on an argument — **and may mutually recurse** with other proof
declarations, **iff the recursion passes SCT** (`../10-kernel/17 §4`), on
identical terms to a recursive `const`/`fn`. Recursion is admitted **iff SCT
accepts**; an SCT-rejected proof recursion — a non-descending self-reference
such as `lemma bad : φ = bad`, or a mutual proof cycle with no decreasing
measure — **fails closed** with the SCT diagnostic, and no such body is ever
declared. The admission run and its soundness are stated in `§8.4`.

`lemma` obeys ordinary module visibility: private by default, `pub lemma` to
export, and imports/shadowing/ambiguity follow the `33 §3-4` module rules. A
lemma is never addressed as `subject::name` unless it is separately declared as
an attached proof, which is a distinct declaration.

Caller use is ordinary application:

```ken
let h1 = append_nil_right A xs
let h2 = (proof appends for list_append) A xs ys
```

Both are ordinary proof terms after resolution, so they can be passed to
transport, rewrite, congruence, induction, or class-law fields.

### 8.4 Admission and soundness of recursive proof claims

Wiring `lemma`/`proof` into the recursive-definition machinery is an
**elaborator admission** change only. It adds **no kernel rule, no trusted
proof table, no ambient proof search, and no `trusted_base()` entry** —
recursive `lemma`/`proof` declarations are admitted by the *same* checks a
recursive `const`/`fn` already passes. This section states the soundness of
that admission normatively; each claim is a property of the admission path, not
of any test.

**The admission path.** All top-level declarations in a scope — `const`/`fn`
and `lemma`/`proof` alike — are admitted through one shared, dependency-ordered
call-graph pass (`§1`):

1. **Dependency-ordered components deliver forward references.** The scope's
   declarations form a call graph — an edge is "the body **or type** of A
   mentions B" — condensed to strongly-connected **components** and processed in
   **dependency order**: every callee component before the callers that depend on
   it, where a component's dependencies are taken as the **union of all its
   members'** out-edges (so a dependency named only by a *non-entry* member of a
   mutual cycle is still ordered first). This ordering — not a scope-wide
   signature pre-pass — is what lets a declaration reference a sibling defined
   **later in source** (a **forward reference**): the sibling's component is
   fully elaborated first. A `lemma`/attached-`proof` type is required to classify
   at `Omega`, and an attached proof's subject must occur applied in its claim
   (`§8.2`); these gates run before that declaration's body in either branch
   below.
2. **SCT is bypassed only for a singleton with no self-edge at all.** A singleton
   component with **no self-edge** is genuinely non-recursive: its signature and
   body elaborate in one step and SCT is **not** consulted. A singleton *with* a
   self-edge is self-recursive and **is** SCT-gated — a **computational**
   (`const`/`fn`) self-recursion takes the **pre-existing singleton recursive
   path**, which SCT-gates it there; a **proof** self-recursion is newly routed
   to the group/SCT seam of point 3. Only the truly acyclic singleton skips SCT.
3. **A recursive component is signatures-first, then SCT-gated, before any body
   is committed.** A component that is a genuine cycle (size > 1) or a proof
   self-reference is **recursive**: **all** its members' signatures are
   pre-admitted **before any member body** — this is the signatures-first step,
   and it is what lets self- and mutual references *within the cycle* resolve —
   then the member bodies are elaborated and kernel-checked against their declared
   types, then **SCT is run on the whole component as one termination problem**.
   Only if SCT **accepts** are the bodies committed (made available for
   reduction); if SCT **rejects**, or any member fails its kernel check, the
   **entire component is rolled back and the declaration is rejected** — no
   recursive proof body is ever committed without passing SCT.

The four soundness properties this preserves:

- **(a) SCT is result-sort-agnostic.** The termination check is the *same*
  check `const`/`fn` use; it reasons about the call graph and structural descent
  of the bodies and **does not branch on whether a definition's codomain is
  `Type` or `Omega`**. An Ω-valued proof is admitted on exactly the termination
  obligation a `Type`-valued function of the same recursive shape would face.
- **(b) An SCT-accepted Ω definition is a valid proof.** For a total type
  theory, the whole obligation on a recursive inhabitant of an `Omega`
  proposition is that it be **strongly normalizing**; SCT-acceptance is that
  guarantee. There is no additional burden from the `Omega` codomain, so an
  SCT-accepted recursive `lemma`/`proof` is a sound proof of its proposition —
  and, conversely, the fail-closed SCT rejection is what keeps a **looping
  "proof"** of a false proposition (`lemma bad : φ = bad`) out.
- **(c) Proof-irrelevance is preserved — Ω proofs are never δ-unfolded in
  conversion.** Admitting recursive Ω-proofs imposes **no new conversion or
  reduction burden**, because the kernel's conversion check short-circuits on
  `Omega` by proof-irrelevance (`../10-kernel/16 §1`): two proofs of the same
  proposition are already convertible without unfolding either. This is
  unchanged by this iteration — the kernel is untouched — so a recursive proof
  body is never δ-unfolded during `conv`, and its recursion introduces no
  non-termination risk into conversion.
- **(d) Erasure and extraction are unchanged.** A proof at `Omega` carries no
  computational content and is erased at its use sites exactly as before;
  admitting proof recursion adds nothing an extractor must handle. This
  iteration touches no erasure or extraction path.

**Explicit out-of-scope boundary — mixed cycles fail closed.** A recursive
component that **mixes** a proof declaration with a computational (`const`/`fn`)
declaration is **rejected explicitly**, never silently admitted. Such a cycle
would couple a δ-unfolded, `Type`-relevant definition with a proof-irrelevant
Ω partner across the same termination measure — a nontrivial interaction
deferred out of this iteration; the boundary is made fail-closed so a program
that reaches for it gets a clear rejection rather than an unchecked admission.
(Homogeneous proof↔proof and computation↔computation cycles are the admitted
cases.)

**Precise boundary.** Everything above is realized in the elaborator's
admission path; the kernel, its conversion/erasure rules, and the trusted base
are untouched. The SCT check, the `Omega`-classification gate, and the per-body
kernel check are **pre-existing** kernel-and-elaborator machinery — this
iteration only routes proof declarations through the same dependency-ordered
SCC + SCT admission path `const`/`fn` already use. No claim here rests on a
test; the tests witness these properties but do not establish them.
