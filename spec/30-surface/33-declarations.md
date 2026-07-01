# Declarations, modules, and constraints

> Status: **DRAFT v0**. Proposal-level for syntax; normative for the *features*.
> Modules/imports, definitions, records, visibility, and Ken's constraint
> mechanism — **typeclasses-as-subobjects-of-the-universe**, a from-scratch
> design for open user typeclasses.

## 1. Definitions

- **`view f … = e`** — a (pure) function; elaborates to a global Π/λ definition,
  gated by SCT (`../10-kernel/17 §4`). Effectful variants carry an effect row
  (`36`).
- **`let x : A = e`** — a value definition (a nullary `view`).
- **`type T … = …`** — a type alias or a refinement/`Σ`/Π type abbreviation
  (transparent; unfolds by δ).
- All top-level definitions are **mutually recursive within a module** if the
  SCT check accepts the group; otherwise the offending recursion is reported
  (`17`).
- Definitions may be **generic** (implicit type/level parameters, `39`): `view
  id {A : Type} (x : A) : A = x`.

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

- **`module M { … }`** groups declarations under a namespace; a file is an
  implicit module named by its path.
- **`import M`** brings `M` into scope qualified (`M.foo`); **`import M as N`**
  aliases; **`import M (foo, Bar)`** selectively; **`use M`** opens `M`
  unqualified (use sparingly).
- **Cross-package imports** resolve through the package manager
  (`38`/`../50-stdlib/`): a manifest (`ken.toml`) + lockfile (`ken.lock`) with
  content-addressed, pinned dependencies; content-addressing makes builds
  reproducible — a marketable feature.
- A package/module is itself an environment fragment; the kernel sees a single
  flattened, append-only `Σ` (`../10-kernel/11 §4`).

## 4. Visibility

- Top-level names are **module-private by default**; `pub` exports. (Or the
  inverse default — OQ-syntax.) Exported names form the module's interface.
- A type may be exported **abstractly** (name only, constructors hidden) — the
  surface form of an opaque constant / abstract interface (`../10-kernel/11
  §4`), giving information hiding without a new kernel feature.

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

view nub {A : Type} (xs : List A) : List A  where DecEq A = …   -- a constraint
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
parameterised by the class head `A`:

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

A constraint `where C A` on a `view`/`let` (grammar `32 §1`) elaborates to an
**implicit instance argument** — an implicit `Π` over the class record inserted
ahead of the explicit parameters:

```
view nub {A : Type} (xs : List A) : List A  where DecEq A = …
      ⟶  nub : {A : Type} → {d : DecEq A} → List A → List A
```

The `{d : C A}` binder is threaded like any implicit (`39 §2.2`): at a **use
site** the elaborator inserts a metavariable for `d` and **discharges it by
instance search** (`39 §6`). Inside the body, a class operation `eq x y` is
`d.eq x y` (projection from the resolved dictionary). Multiple constraints
`where C A, D B` insert one implicit each, left to right. The concept is
ordinary dependent-implicit insertion — the *resolution* of `d` is the only new
step, and it lives in `39 §6`.

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

## 6. Fixity and operators

`infixl N op` / `infixr N op` / `infix N op` declare operator fixity (`32 §6`).
Operators are ordinary `view`s with symbolic names; there is nothing special
about them semantically.

## 7. What WS-L must deliver here

Definitions (incl. generic + mutually recursive under SCT), records (dependent +
refined fields, update/pun), the module/import system + package manager with
content-addressed lockfiles, visibility/abstraction, and the
class/instance/constraint mechanism with **lawful** instances and `derive`.
Conformance: `../../conformance/surface/declarations/`.
