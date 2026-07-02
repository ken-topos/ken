# Surface minimality — the derivation-path table (ES1)

Format: `../../README.md`. This is the **minimality proof** of the everyday
surface (`docs/program/everyday-surface-program.md`, `cfe5172`): the
load-bearing artifact certifying the invariant

> **The surface built-in set ≡ the surface `trusted_base()` delta.** A
> prelude/standard entry with a Ken-**derivation witness** lands as a
> re-checked `definition` (out of `trusted_base()`); only a genuine
> **irreducibility witness** stays a `postulate`/primitive.

so "built-in vs package" **is** "audited trust-root vs re-checkable Ken" — the
minimality proof is simultaneously a **TCB-hygiene** proof, the surface analog
of Sec4's TB-Sound + TB-Complete (`64 §1.1`/`§1.2`). The table has **two
halves**, each a **direction** of the discriminating check (a one-directional
table proves nothing):

- **Completeness (§C):** every prelude/package feature → a **real Ken
  derivation path** from the built-ins. A feature with **no** path is a
  **hidden built-in** (the TB-Complete omission — an assumption that hid).
- **Irredundancy (§A/§D):** every built-in → its **irreducibility witness**
  (why it *can't* be Ken-defined). A built-in **with** a path is **bloat** (a
  TB-Sound phantom — a needless `trusted_base()` entry).

**Method — real, not asserted.** Each verdict carries a witness grounded
against landed code (`prelude.rs`/`numbers.rs`/`bytes.rs` @ `b97ca5c`, the
kernel). The full *elaboration* witness (each demoted def kernel-checks in the
stated sort + `trusted_base()` shrinks by exactly the named ids) is **ES2's
build-verification** (a real `trusted_base()`-delta assertion — my Sec4 lane);
`ken-cli` is REPL-only and Ω-data/truncation may not be surfaced yet, so here
the witness is the **grounded sort-analysis**, which is what the taxonomy line
needs.

## A. The built-in set (irreducibility witnesses)

The surface TCB — each is **irreducible** (no Ken derivation path); a path
here would be bloat. None found (all four are genuine).

| Built-in | Witness (why not Ken-definable) |
|---|---|
| **Primitive types + literals** — `Int`/`Int8..64`/`UInt8..64`/`Decimal`/`Float`/`Float32` (`numbers.rs reg_ty`), `Char`, `String`/`Bytes` (`bytes.rs`) + literal syntax | parser-produced opaque type constants (`declare_primitive` OpaqueType, `14 §5`); nothing is more primitive. |
| **Audited primitive ops** (`14 §5`) — `reg_binop` (`A→A→A` arith), `reg_cmpop` (`A→A→Bool`), the `String`/`Bytes` prims (`append`/`slice`/`byteLength`/`String↔List Char`) | bottom out in the kernel's audited `PrimReduction::Op` on literals; not expressible as pure Ken (they *are* the machine semantics). |
| **The effect / FFI boundary** — `foreign` + the base `IO`/effect primitive (`[Console]`/`[FS]`; `print_line` foreign) | I/O is not pure Ken — the effect boundary is where the world enters. |
| **Base elaborator syntax** — λ/app/`let`/`match`/annotation/`data`/`view`/`instance`, refinement types, the **operator-infix + fixity** affordance, `if`-sugar, minimal `module`/`import` | the language forms themselves; the parser/elaborator realizes them. (Note: `if` *desugars* to `match`, and operator *semantics* is package — but the **syntactic affordance** to write them is base. Syntax built-in; semantics derivable.) |

## B. The prelude set (the signature-reference closure — AC2)

**Membership rule (normative, checkable):** a type is **prelude** iff it is
named in a **built-in primitive's type signature** — nothing else. The surface
analog of the kernel's closed `is_prelude = {Top, Bottom}` (`64 §1`); the
prelude is a **second minimality target**, not a catch-all. Grounded by
signature-grep of `reg_*` in `numbers.rs`/`bytes.rs`:

| Prelude type | Signature that names it | Derivation (Ken-defined) |
|---|---|---|
| **`Bool`** | `reg_cmpop` result `A → A → Bool` (`numbers.rs:173`) | `data Bool = True \| False` — derivable, but signature-named ⇒ prelude (F1). |
| **`Char`** | `string_to_list_char : String → List Char`, `char_length` (`bytes.rs`) | a scalar type (`35 §2.4`); signature-named ⇒ prelude. |
| **`List`** | `string_to_list_char : String → **List** Char` / `list_char_to_string` | L2 inductive (`data List`); signature-named (via `List Char`) ⇒ prelude. |
| **`Ω` (Omega)** | `reg_novf` no-overflow prop `A → A → Ω₀` (`numbers.rs:190`) | **kernel-provided** (the strict-prop universe, `16 §1`) — a *kernel* built-in, not a surface prelude datatype. |

**★ AC2 bloat finding — `OrdResult`.** `data OrdResult = Lt | Eq | Gt`
(`prelude.rs:139`) sits in the elaborator prelude, but **no primitive
signature names it** — the comparisons return `Bool` (`reg_cmpop`), not
`OrdResult`. By the membership rule it is **not prelude** (a prelude type no
signature names = the flagged bloat vector). Its origin is a **workaround**:
"`Bool` is an opaque primitive… not pattern-matchable; `sort`/`insert` branch
on `OrdResult` instead" (`prelude.rs:90`). **F1 (`data Bool`) removes the
need** for the branch workaround. **Ruled (`30-taxonomy §6`, `7fa08cd`):
remove `OrdResult` (bloat); the `Ord` package's `Ordering` is the 3-way
`compare` result (standard-package, no primitive returns it). The prelude
closes to exactly **{`Bool`, `Char`, `List`}**. My witness confirms the ruling:
signature-grep finds no `reg_*` naming `OrdResult`. This is the AC2
discriminator firing in the bloat direction.

## C. The standard-package set (completeness — derivation paths)

Every package feature has a **real Ken derivation path** from the built-ins.
No hidden built-in found — **but** each path names the built-in *floor* it
needs (remove that floor and the feature *becomes* a hidden built-in — the
load-bearing observation).

| Package feature | Derivation path from built-ins | Built-in floor |
|---|---|---|
| **operators** (`+ - * % == < >`) | `Ord`/`Eq` **class methods** (Lc, landed — `lawful_classes.ken`) back `== < >`; `+ - * %` bind directly to the audited prim ops (`int_add` etc. via `reg_binop`) + operator-infix syntax — **`class Num`/`instance Num Int` are specified-but-not-built** (named forward obligation, a future `class Num` WP), so `+`/`*` are not yet class-abstracted; user types get `== < >` by writing `Eq`/`Ord` instances | the audited prim op (`reg_binop`/`reg_cmpop`) + operator-infix syntax (base) + Lc |
| **`show`/formatting** (`Int.show`, …) | `Int` `div`/`mod` prims → digit `Char`s (literals) → `List Char` → **`list_char_to_string`** (landed) → concat via **`append`** (landed) | `div`/`mod` prims, Char literals, `list_char_to_string`, `append` (**all landed** `bytes.rs`/`numbers.rs`) |
| **collection combinators** (`map`/`filter`/`fold`/`range`) | total structural recursion over `List`/`elim_List` (L2/L3); `range` = fuel-bounded unfold (`37 §5`, no coinduction) | `data List` + `elim_List` (L2), recursion + SCT |
| **lawful classes** (`Monoid`/`Functor`/`Monad`/`Foldable`) | `class`/`instance` records (Lc, landed) carrying law propositions | Lc (`33 §5`, landed) + Ω (laws) |
| **string manipulation** (`split`/`join`/`pad`/`toUpper`) | over `String↔List Char` (landed conversions) + `append` + the combinators | `String↔List Char` + `append` + combinators |

**Completeness verdict: PASS** — every package feature is derivable; the
built-in *floor* (the audited String/Int prims + the `String↔List Char`
conversions + Lc) is exactly what makes the surface generable. Had `append` or
`list_char_to_string` **not** been landed, `show`/string-manipulation would be
**hidden built-ins** (no path) — that is the check that matters, and it passes
on the landed set.

## D. Irredundancy findings — the prelude postulates (the ★ TCB-hygiene half)

The concrete `trusted_base()` entries in `prelude.rs` (Architect-approved
`evt_5bedyc3zyhr`), each a live trust-root surface. Verdicts + Ω-sort
witnesses:

| Entry | Form | Verdict | Witness / action |
|---|---|---|---|
| **`Equal : Π(A). A→A→Ω`** | `declare_postulate` | **REDUNDANT — shadows a *computing* primitive** | the kernel provides native **`Eq A a b : Ω`** (computes, with `refl`/`J` — `16 §2`, `term.rs`). The postulate forfeits `Eq`'s computation + `J`-elim. **Action: delete, reference `Eq`** (not "define"). |
| **`And : Ω→Ω→Ω`** | `declare_postulate` | **DERIVABLE** | `data And (A B:Ω):Ω := conj (a:A)(b:B)` → **Ω** via both-keyed `sort_sigma` (Σ of two Ω → Ω); or `16 §1.3` derived connectives. |
| **`isSorted : Π(A). List A→Ω`** | `declare_postulate` | **DERIVABLE (★ soundness)** | Ω-recursion `isSorted (x::y::r)= And (x≤y)(isSorted (y::r))`. **Needs a Prop-valued `≤ : A→A→Ω`** — if `Ord` exposes only `Bool` `leq`, add `Le`/`IsTrue (leq a b):Ω` (else it's `Type`, a relevance leak). |
| **`Perm : Π(A). List A→List A→Ω`** | `declare_postulate` | **DERIVABLE (★ soundness)** | **Ω-sort fork:** the inductive relation (`refl\|swap\|trans\|cons`) is proof-**relevant** (`Type`) ⇒ needs **truncation** `∥·∥` to be an Ω predicate; count-equality (`Π x. Eq Nat (count x xs)(count x ys)`) is **natively Ω** but DecEq-dependent. Either is derivable; spec picks the form. |
| **opaque `Bool`** | `declare_primitive` (Opaque) | **DERIVABLE (F1)** | `data Bool = True\|False` (Type) — removes the opaque primitive **and** the `OrdResult` branch-workaround (§B). |
| **`Map`/`Set`** | `declare_postulate` | **runtime — MIS-CLASSIFIED** | NOT derivable (O(1) content-addressed canonical form is heap-backed, `41 §3a`) — but **audited primitives** (`declare_primitive` OpaqueType, item-2, like `String`/`Bytes`), **not assumed axioms** (item-3). Stay in `trusted_base()`, correctly (§E). |
| **`reg_novf` — the no-overflow PREDICATE** (`Fits`/`inBounds : Int → Ω`) | `declare_postulate` (`numbers.rs:190`) | **★ NEW (seed missed) — DERIVABLE (ruled §6)** | the reusable decidable fixed-width **bound predicate** (`(a+b)` within `[MIN_w, MAX_w]` over arbitrary-precision `Int`) → a **definition, out of `trusted_base()`** (`OQ-1a`). Same Prop-`≤`/`IsTrue` bridge as `isSorted`. My signature-grep surfaced it; §6 ruled it a definition. |
| **the L1 per-op no-silent-wrap OBLIGATION-HOLES** (`declare_postulate` goal per fixed-width op) | `declare_postulate` | **★ NOT bloat — a LIVE OBLIGATION (stays)** | the per-operation "no silent wrap" **proof obligation** awaiting prover discharge (`35 §3`, `43 §2`, [[soundness-AC-static-vs-runtime-face]]) — **legitimate trusted-until-discharged**, the overflow soundness net. Making *this* a "definition" would be circular or **eliminate the net**. It **stays** in `trusted_base()` as a live obligation (item-3, but the *good* kind). **The distinction the invariant turns on:** a *derivable* postulate is bloat; a *live proof-obligation* is not — the predicate demotes, the obligation-hole stays. If `reg_novf` does double duty, ES2 **splits** them (predicate→def / obligation→hole). Architect pre-flag `evt_57r42rsx3jx3w`. |
| **numeric + string literals** (`elab.rs:460`/`:503`, `elab_str_lit`) | `declare_postulate` (**per literal**) | **★ HIGHEST-VOLUME hygiene — TERM (ruled §3)** | each literal *value* (`42`, `"…"`, in `num_values`) is a **per-program `trusted_base()` postulate** — an *assumed* value for a *computed* constant. Ruling: a **primitive-constant TERM** (the *type* is item-2, listed once; the value is a real core term), **out of `trusted_base()`** — not a per-literal entry at all. Verified `elab.rs:460` (Architect VAL1 catch `evt_488kj79z0wqd7`). |

**Ω-sort discipline (the relevance-leak check, Architect `evt_5bedyc3zyhr`):**
every predicate demoted to a def must land in **Ω** (proof-irrelevant), not
`Type` — a `Type`-valued "prop" leaks content into the refinement carrier.
`And` ✓ (both- keyed Σ→Ω); `isSorted`/`Perm` per the forks above; `Bool` is
correctly `Type` (matchable data, not a prop).

## E. Trust-class accounting (AC4) — the `trusted_base()` delta

Both `Decl::Opaque` (item-3, **assumed axiom**) and `Decl::Primitive` (item-2,
**audited**) surface in `trusted_base()` (the `matches!(Opaque | Primitive)`
filter, `64 §1.2`, my Sec4 ground) — so the *category* is a
trust-level-honesty distinction, not a listed-or-not one:

The invariant turns on a distinction the accounting must keep sharp:
**`trusted_base()` should contain only genuine irreducibles + audited
primitives + *live proof-obligations*.** A **derivable** postulate is bloat
(demote); a **live obligation** (awaiting discharge) is legitimate (stays).
Three fates:

- **Leave `trusted_base()` entirely** — the *derivable* / *shadowing* entries,
  now re-checked `Decl::Transparent` defs (or a kernel reference / a term):
  `Equal` (→ kernel `Eq`), `And`, `isSorted`, `Perm`, `Bool`, the `reg_novf`
  **predicate** (→ decidable-bound def), and **every literal** (→
  primitive-constant terms). The **assumed-axiom bloat** goes to zero.
- **Stay listed, re-classified `Opaque`→`Primitive`** (item-3 assumed → item-2
  **audited**): `Map`, `Set`. A real admission-path change (the `Decl`
  variant), not a relabel; no trust regression (still listed), but the basis is
  now honestly "audited," not "assumed."
- **Stay listed as *live obligations*** (item-3, the legitimate kind): the L1
  per-op **no-silent-wrap obligation-holes** — trusted-until-discharged, the
  overflow soundness net (`soundness-AC-static-vs-runtime-face`). **Not** bloat;
  removing them would eliminate the net.

**Net:** the surface **assumed-axiom bloat** in `trusted_base()` goes to
**zero**; what legitimately remains is **`Map`/`Set` as audited primitives** +
the **live overflow obligations**. The invariant holds on the real set: **no
built-in has a derivation path** (§A, no bloat) and **no package/prelude
feature lacks one** (§B/§C, no hidden built-in) — and the `trusted_base()`
surface a consumer audits is honestly *irreducibles + audited primitives + live
obligations*, nothing derivable hiding as an assumption.

## Coverage map (AC → sections)

- **AC1** (invariant normative + minimal set exact; both directions): §A
  (irreducibility, no bloat) + §C (completeness, no hidden built-in) — the
  table exercises **bloat** (§B `OrdResult`, §D `Equal`/`And`/…) **and**
  hidden-built-in (§C, none found; the floor named).
- **AC2** (prelude closed by the signature rule): §B — the signature-grep
  closure {`Bool`,`Char`,`List`}, the `OrdResult` bloat finding (ruled
  remove; `Ordering`→package, §6).
- **AC3** (load-bearing predicates specified as definitions): §D — `And`/
  `isSorted`/`Perm` with defining equations + Ω-sort witnesses; the
  verified-`sort` refinement (`37 §6`) unfolds them (green-vs-green against a
  postulate otherwise).
- **AC4** (trust-class rulings exact): §E — the item-2/item-3 line per entry,
  the `trusted_base()` delta; `Equal` delete-for-`Eq`, `Map`/`Set`
  audited-primitive.

## Build-forward (ES2's verification gate)

This is **spec + conformance only** (no crate). **ES2** implements the
`prelude.rs` demotion; its conformance gate is the **elaboration witness** —
producer-grepped, not asserted:
1. Each demoted predicate (`And`/`isSorted`/`Perm`) **kernel-checks as a
   `Decl::Transparent` def in the stated Ω sort** (the relevance-leak check).
2. The **assumed-axiom** entries **leave** `trusted_base()` — `Equal`, `And`,
   `isSorted`, `Perm`, `Bool`, the `reg_novf` **predicate**, and the per-literal
   postulates (→ terms) — a real `trusted_base()`-delta assertion; no entry
   hides, none over-removed.
3. `Map`/`Set` **still appear** but as `Decl::Primitive` (item-2), the
   trust-class corrected.
4. **★ The live overflow obligation-holes still appear** (item-3, the
   legitimate kind) — ES2 must **not** sweep them away with the predicate; the
   split (predicate→def / obligation→hole) is the load-bearing check, or the
   overflow net is lost.
A green ES2 that hand-inserts the def or asserts "it type-checks" without the
`trusted_base()`-delta (both the removals **and** the retained obligations) is
green-vs-green (`conformance-hand-feeds-the-deliverable`).
