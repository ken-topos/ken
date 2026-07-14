# The surface taxonomy: built-in, prelude, standard-package

> Status: **Normative** for the built-in/prelude/package **line** and the
> minimality invariant; the concrete membership lists grow with WS-L (each entry
> settled by the derivation-path table, `../../conformance/surface/taxonomy/`).
> This is the organizing chapter for the everyday surface: it fixes *what must
> be primitively provided* versus *what is ordinary Ken*, mirroring the kernel
> discipline at the surface. It elaborates the operator principle — **built-ins
> = the minimum set from which everything else is built; everything beyond is a
> standard package** — and makes it a **soundness** property, not just an
> ergonomic one.

## 1. The principle — the surface is a small generating core plus derived Ken

The kernel is a small audited trust root; everything else is re-checked. The
surface mirrors this: a **minimal built-in set** (the surface analog of the TCB)
generates the everyday surface, and everything else is **ordinary Ken** the
kernel re-checks. Minimality is not an aesthetic — it is **TB-Sound +
TB-Complete at the surface** (`../60-security/64 §1.1`):

- a "built-in" that is actually Ken-**derivable** is **bloat** — a surface
  over-claim, the over-large-TCB failure (surface TB-Sound: no phantom
  built-in);
- a "package" with **no** derivation path from the built-ins is a **hidden
  built-in** — an assumption that slipped in unlisted (surface TB-Complete: no
  hidden built-in).

### 1.1 The invariant (normative)

> **The surface built-in set ≡ the surface `trusted_base()` delta.** A prelude
> or standard entry that has a Ken-**derivation witness** lands as a re-checked
> `definition` (a `declare_def`, **out** of `trusted_base()`); only an entry
> with a genuine **irreducibility witness** stays a `postulate`/`primitive`
> (**in** `trusted_base()`). So the set of surface built-ins and the audited
> trusted base **coincide** — the surface analog of TB-Complete's
> choke-point-equals-filter (`64 §1.2`).

This makes "built-in vs package" **the same line** as "audited trust-root vs
re-checkable Ken." The derivation-path table (CV's `/conformance` artifact) that
proves the taxonomy *is* simultaneously the TCB-hygiene proof: it drives the
invariant in **both** directions — no built-in has a derivation path
(irredundant) and no package/prelude entry lacks one (complete).

## 2. The three tiers

| Tier | What it is | Trust level | In `trusted_base()`? |
|---|---|---|---|
| **Built-in** | irreducible — cannot be defined in Ken from other built-ins | the **surface TCB**: audited primitives / assumed at the boundary | **yes** (primitive/postulate) |
| **Prelude** | Ken-defined, but **always present** because a built-in's *signature* names it | re-checked `definition` | **no** |
| **Standard package** | Ken-defined, **optional**, explicit `import` | re-checked `definition` | **no** |

The two lower tiers are both **re-checked Ken** (out of the trusted base); they
differ only in **availability**: prelude is always in scope (the primitive layer
depends on it), a package is imported. "Always there" ≠ "irreducible" — a
prelude entry is a definition the kernel re-checks, not a trust-root assumption.

## 3. The built-in set — the surface TCB (irreducible)

Exactly these are primitively provided; each is irreducible (there is no more
primitive Ken to define it from):

- **Primitive types + the literal affordance** — the types
  `Int`/`Float`/`Char`/`String`/`Bytes` (`35`, `37`) are admitted via
  `declare_primitive` (item-2, in the base once); the parser's reading of a
  literal token is base syntax. Each literal *value* is a **primitive-constant
  term** of its type — computed, so **out** of `trusted_base()` (§6: the current
  per-literal `declare_postulate` is the highest-volume hygiene item). The type
  is irreducible; nothing is more primitive than the machine representation the
  primitive ops compute on.
- **Audited primitive operations** (`../10-kernel/14 §5`) — machine
  arithmetic/comparison and the `String`/`Bytes` primitives, each a
  `Decl::Primitive` whose registered `Op` symbol dispatches runtime evaluation
  on values. These bottom out in the interpreter's audited `prim_reduce`
  surface and are **not** Ken-definable. They remain opaque to kernel conversion
  until K3; an operation equation over literals therefore needs a visible proof
  assumption rather than `Refl` today.
- **The effect / FFI boundary** — `foreign` and the base `IO`/effect primitive
  (`[Console]`/`[FS]` etc., `38`, L5/L7). I/O cannot be pure Ken; the boundary
  is a listed assumption.
- **Base elaborator syntax** — the language forms themselves: λ, application,
  `let`, `match`, annotation, `data`/`view`/`instance`, refinement types, the
  **operator-infix + fixity** affordance, **`if`-sugar**, and the **minimal
  `module`/`import`** (F3a). These are how source becomes core; they are not
  values one could define.

Everything else on the everyday surface is one of the two re-checked tiers
below.

## 4. The prelude tier — Ken-defined, always-present, closed

Some Ken-definable types must be present **before** the primitive layer, because
a built-in primitive's **type signature references them**. `Bool` is the
clearest case: the comparison primitives have type `Int → Int → Bool`, so `Bool`
must exist at the primitive layer — yet `Bool` is `data Bool = True | False`,
ordinary Ken (§6, F1). This is the surface analog of the kernel's
`Top`/`Bottom`/`tt` prelude (`64 §1`: Ken-vocabulary excluded from
`trusted_base()` yet always present, the closed `is_prelude` set).

> **Prelude membership rule (normative, checkable).** A type is in the prelude
> **iff it is named in a built-in primitive's type signature, and is not already
> provided by the kernel** — nothing else. The prelude is therefore a **closed,
> minimal** set, mechanically derivable by collecting the type names the
> primitive signatures (`reg_*`) mention. A "prelude" type **no** primitive
> signature names is a **bloat vector** (it is really a package); a primitive
> whose signature names a type **absent** from the prelude is a **gap**.

The prelude is a **second minimality target** — the same TB-Sound discipline
(`is_prelude` is exactly `{Top, Bottom, tt}`, no catch-all) applied at the
surface.
By the rule, today's prelude is the closed set **`{Bool, Char, List}`** (CV's
signature-grep): `Bool` (the comparison primitives `Int → Int → Bool`), and
`Char` **and** `List` (the `String ↔ List Char` conversion primitive names
both). Each is a re-checked `definition`, **out** of `trusted_base()`.
`Ordering` is **not** prelude — no built-in primitive returns it (comparisons
return `Bool`, and 3-way `compare` is an `Ord` **class method**, a package, F2)
— so it is a standard-package type; adding a `compare : A → A → Ordering`
primitive *would* make it prelude, but that enlarges the built-in set for no
minimality gain and is **not** taken. The derivation-path table
(`../../conformance/surface/taxonomy/`) pins the exact closed set and flags any
over-inclusion as bloat (§6, `OrdResult`).

## 5. The standard-package tier — the dissolved stdlib

Everything Ken-definable that **no** primitive signature forces into the prelude
is a **standard package**: optional, explicitly imported, ordinary Ken with its
**derivation path from the built-ins stated in-spec**. The reframed catalog is
`../50-stdlib/README.md` — the lawful classes (`Num`/`Ord`/`Eq`/`Monoid`/
`Functor`/`Monad`/`Foldable`), the collection combinators
(`map`/`filter`/`fold`/ `range`), and formatting (`show`/`split`/`join`/`pad`).
The monolithic **L8 stdlib dissolves** into this catalog
(`docs/program/wp/L8-stdlib-core.md` superseded); its "laws are **proved, not
postulated**" discipline carries to the package builds (ES4).

**The derivation-path discipline (normative).** Every catalog entry states a
real Ken definition path from the built-ins. A catalog entry with **no** path is
a spec bug — a **hidden built-in** — caught jointly with CV's table; a built-in
with a path is **bloat** and must be reclassified. This is the surface
TB-Complete net made a documentation obligation.

## 6. Classification rulings — making the invariant true on the real code

The current `crates/ken-elaborator/src/prelude.rs` violates the invariant: it
carries **derivable** entries as postulates (needless `trusted_base()` entries)
and **mis-classifies** two runtime types. This chapter specifies the
corrections; ES2 implements the `prelude.rs` demotion. Each is a
`trusted_base()`-shrinking move that makes the built-in set and the trusted base
coincide.

- **`Equal` → delete; use the kernel's native `Eq`.** The prelude postulates
  `Equal : Π(A). A → A → Ω`, which **shadows the kernel's computing `Eq`**
  (`../10-kernel/16 §2` — a real former with `refl`/`J`, reduced by recursion).
  An assumed equality over the real one is not merely a phantom `trusted_base()`
  entry — it **forfeits `Eq`'s computational and `J`-elimination behavior**. The
  ruling is **delete and reference `Eq`**, not re-define. (A postulate
  duplicating a real kernel construct is the surface form of the
  name-shadows-the-mechanism trap.)
- **`And` → the derived connective, not a postulate.** Ω-conjunction is already
  a **derived operation** from the K1 formers (`16 §1.3`). The prelude postulate
  is redundant; reference the derived connective (or
  `data And (A B : Ω) : Ω = conj A B`, which lands in Ω by the
  both-components-keyed `sort_sigma`, `13 §4`).
- **`is_sorted` / `Perm` → definitions (see `37 §6`).** These are **not** prelude
  (no primitive signature names them) — they are the verified-`sort` showcase's
  predicates, and they **must be definitions**, specified in `37 §6` (§below).
  As postulates the flagship proof proves nothing.
- **opaque `Bool` → `data Bool = True | False`** (F1). The current opaque
  primitive `Bool` is not matchable, which forced the `OrdResult` branch
  workaround (`prelude.rs`); as a 2-constructor inductive it is derivable,
  matchable, and `if` is `match` sugar (`34`, `42 §2`). Removes a built-in and
  the workaround.
- **`Map` / `Set` → proved inductives, out of `trusted_base()` (OQ-A
  supersession).** These were originally slated to *stay* in `trusted_base()` as
  **audited runtime primitives** (`declare_primitive` OpaqueType, item-2, like
  `String`/`Bytes`) — correct *given* the premise that they must be the O(1)
  content-addressed, insertion-order-independent heap form (`41 §3a`). Operator
  decision **OQ-A** (2026-07-03) **changes that premise**: it chooses a
  **proved, pure, `Ord k`-keyed program-level tree** (`../50-stdlib/52-map.md`)
  over the runtime-O(1) heap form — accepting O(log n) and extensional identity
  to gain real proofs and zero trust. So `Map`/`Set` become **ordinary inductive
  `data` + `view` defs, re-checked, out of `trusted_base()`** — the opaque
  `Map`/`Set` primitive is **retired**, shrinking `trusted_base()` (a
  net-negative delta; "proved" *requires* a transparent carrier, since an opaque
  primitive has no eliminator and its laws could only be `Axiom`). The
  content-addressed heap form is **parked** as a possible later fast-map
  (the "HAMT-later" analog, `37 §3.2`), proved if it lands. (The item-2/item-3
  audited-vs-assumed accounting distinction below still governs the entries that
  genuinely *stay* runtime, e.g. `Array`/`String`/`Bytes`.)

- **`OrdResult` → remove (bloat).** `data OrdResult = Lt | Eq | Gt`
  (`prelude.rs`) sits in the prelude but **no primitive signature names it**
  (the comparisons return `Bool`), so by the membership rule (§4) it is a
  **bloat vector**. It exists only as a workaround for the opaque `Bool` (not
  matchable); F1's `data Bool` obviates it. Remove it; where a 3-way result is
  wanted, the `Ord` package's `Ordering` (§4/§5) is it.
- **`reg_novf` — split the predicate from the per-operation obligations (only
  one is bloat; Architect pre-flag).** Two distinct things live under the
  `numbers.rs` no-overflow registration (`OQ-1a`), and they must not blur:
  - The no-overflow **predicate** (`Fits`/`inBounds : Int → Ω`, the decidable
    bound `MIN_w ≤ (a +_ℤ b) ≤ MAX_w` over the **unbounded** `Int` primitive) is
    **derivable** → a **definition**, out of `trusted_base()`. The prover / SMT
    bitvector theory is the **discharge engine** for the *defined* predicate
    (re-checked, oracle-not-authority, G3), not a trusted prover-theory atom.
  - The L1 **per-operation "no silent wrap" obligations** (the
    `declare_postulate` goal each fixed-width op emits, awaiting prover
    discharge) are **legitimate live obligation-holes**, **not**
    derivable-postulate bloat: making *these* definitions would be circular or
    **eliminate the overflow net**. An undischarged obligation is an honest
    typed hole in `trusted_base_delta` (`21 §5.2`, the four-way `unknown`
    status) — it **stays** until discharged.

  So `trusted_base()` legitimately holds **genuine irreducibles + live
  obligation-holes**; the demotion target is the **predicate only**, never the
  per-op obligation — the one place "derivable bloat" and "live obligation" must
  not be conflated.

**The rulings above are worked examples; CV's derivation-path table
(`../../conformance/surface/taxonomy/`) is the exhaustive net.** Every
`trusted_base()` entry it surfaces is classified by the **same two rules**: a
**derivable** entry demotes `postulate → definition` (out of the base); a
**genuinely runtime** entry that is *audited* (computed), not *assumed*,
re-classes `item-3 postulate → item-2 primitive` (stays, correctly described).
The highest-volume case is **literals**: today each numeric/string literal in a
program is a per-program `declare_postulate` (`elab.rs`) — an **assumed value
for a computed constant**. A literal is a **primitive-constant term** of its
built-in type (§3), computed, so it belongs **out** of `trusted_base()` entirely
(the base lists the `Int`/`String` *primitive type* once, item-2, never each
literal value). Whatever the table rules, ES2 lands it.

**Net.** Of the surface soundness entries,
`Equal`/`And`/`is_sorted`/`Perm`/`Bool`/`OrdResult` and the `reg_novf`
**predicate** become re-checked definitions or are removed (**out** of
`trusted_base()`), literals become primitive-constant terms (out), and
`Map`/`Set` — under OQ-A — become **proved package inductives, retired from
`trusted_base()`** (out; `../50-stdlib/52-map.md`), the opaque primitive gone.
The **assumed-axiom** surface trusted base shrinks toward **zero** — leaving
only the genuinely-audited primitives (`Array`/`String`/`Bytes`) **and the live
proof obligations** (e.g. the per-op no-silent-wrap holes, honest until
discharged) — and the verified-`sort` showcase (§37 §6) becomes a real proof.
