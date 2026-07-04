# CAT-1 — the type-constructor class pattern

**Steward frame → spec enclave (design the abstraction boundary) → build.**
First WP of the **catalog campaign** (`docs/program/06-catalog-campaign.md`;
operator chose catalog-led, 2026-07-04). Establishes law-carrying classes over a
**type constructor `f : Type -> Type`** — the genuinely-new pattern every later
catalog layer (collections, parsers, effects, traversals) leans on. Pattern-
setter ⇒ **first-of-kind, T1 design + review** (`core-catalog` report, Tier C).

Owner: **Language** (the `packages/`/L8 stdlib owner). Design front-loaded to the
enclave — **Architect owns the abstraction-boundary core** (the higher-kinded
class mechanism + how OTT states the laws); spec-author + conformance-validator
assist on surface + discriminating conformance. Gate: enclave elaboration →
merge → Language build → Architect soundness + Language-QA + Verify-QA + CI.
Findings/forks → **Steward**.

## What it is

Extend the landed `packages/lawful-classes` pattern (`Eq`/`DecEq`/`Ord`, over a
**value type** `a : Type`) to the workhorse **constructor classes** and their
value-level algebra companions:

- **`Semigroup`, `Monoid`** — value-level algebra (`<>`/`mappend`, `mempty`;
  associativity + left/right unit). The warm-up: same shape as `Eq`/`Ord` (over
  `a : Type`), no new kind machinery — lands the algebra `Foldable` needs.
- **`Functor`, `Foldable`** — the first classes over `f : Type -> Type`. This is
  where the new pattern is set: a `class` abstracting a **type constructor**, with
  laws that are **equations between `f a` values / functions**.

`Applicative`/`Monad`/`Traversable` are **CAT-2** (fast-follow, depends on this).

## Fixed inputs — pin as settled, do NOT reopen

Cite `spec/50-stdlib/51-lawful-classes.md`, the kernel record/Σ + Ω vocabulary
(`33 §5.2`, `13 §3`, `16 §1`), and the landed `lawful-classes` package.

- **Kernel-untouched (AC1).** Ordinary Ken + the existing surface, exactly like
  `lawful-classes`: a **class is a record** (right-nested Σ, `33 §5.2`); a **law
  is an Ω-proposition** (`16 §1`) bridged by `view IsTrue (b:Bool):Prop = Equal
  Bool b True`. No new kernel former, no new `Term`/`Decl`.
- **Same package discipline** as `lawful-classes` (`packages/README.md`):
  MANIFEST, Ken source, derivation path, declared `trusted_base()` delta. New
  package `packages/lawful-functors/` (or extend the catalog namespace per the
  enclave's naming — coordinate with `06`'s `ken.*` shape).
- **Laws PROVED over inductive carriers, not postulated.** The instances land
  over **inductive** carriers (`List`, `Option`) whose laws are **provable by
  induction ⇒ zero-delta** — the contrast the landed package already draws: `Int`
  instances are *audited-delta* only because `Int` is a K1 primitive (opaque to
  δ, no induction principle). A law field that is `Axiom` on an inductive carrier
  is a **defect**, not an honest postulate.
- **Additive, subsumes nothing.** These classes sit *alongside* `Eq`/`DecEq`/`Ord`
  and the `collections` package; they do not rework them.
- **Ω-cleanliness over truncation** (the `Ord.total` precedent): prefer a
  `Bool`-equation / decidable form that stays Ω-clean; reach for `‖·‖`
  truncation only where a law is genuinely proof-relevant, and justify it.

## The core design question — routed to the enclave (Architect owns)

**Does the landed `class`/record machinery already admit a `Type -> Type`
parameter, or does the higher-kinded abstraction need an (outer-ring) elaborator
extension?** This is the crux and it is the enclave's to ground **first**, on the
landed code (perishable — verify against the elaborator's `class` desugaring at
pickup, do not trust this line):

1. **Grounding (buildability, every axis).** Grep the `class`/instance
   desugaring: can a class quantify over `f : Type -> Type` (a higher-kinded
   record parameter) today, and at what universe level does the record sit? If
   **yes** → CAT-1 is pure package Ken (like `lawful-classes`). If **no** → the
   minimal elaborator extension to admit a constructor parameter is a **pinned
   sub-deliverable of this WP** — still **outer-ring (kernel-untouched)**, but
   more than package code; size it and flag it to Steward. Do not assert either
   branch without the grep ([[buildability-ruling-must-ground-every-axis]],
   [[class-dict-explicit-vs-implicit-abstract-tyvar]]).
2. **How OTT states the Functor laws.** `map id = id` and `map (g ∘ f) = map g ∘
   map f` are **equations between `f a` values (and functions)**. Decide the
   canonical statement form: pointwise `(x : f a) -> Equal (f a) (map idf x) x`
   vs. a function-level `Equal (f a -> f a) (map idf) idf` needing funext — and
   how OTT's observational equality discharges the fusion law. This is the
   pattern CAT-2's Monad laws inherit, so it must be right the first time
   ([[proof-relevant-inductive-cannot-be-declared-at-omega]] — keep law codomains
   Ω-clean; a proof-relevant equation needs the truncation account).

## Mandated deliverable outline (each ends in a concrete, pinned choice)

1. **`Semigroup`/`Monoid`** (value-level). Fields + laws (assoc; left/right
   unit). Concrete canonical instances: `List a` (append monoid — inductive,
   laws proved), and one more (`Nat` additive **or** `Bool` — enclave picks the
   pattern-clearest). *End state: the exact field/law spelling + which instances.*
2. **`Functor`** (over `f : Type -> Type`). `map`; identity + composition laws in
   the form chosen in the design question §2. Canonical instances: `List`,
   `Option` (both inductive ⇒ proved, zero-delta). *End state: the higher-kinded
   class mechanism pinned + the law statement form + proved instances.*
3. **`Foldable`**. `foldr` (and/or `foldMap` via `Monoid`) + the fold laws + the
   `Monoid` coherence. Canonical instances: `List`, `Option`. *End state: the
   fold interface (foldr vs foldMap primary) + laws + instances.*
4. **The reusable template.** A short section in the MANIFEST/spec naming this as
   the constructor-class template CAT-2 (Applicative/Monad/Traversable) extends —
   so the next tranche is mechanical. *End state: the documented pattern.*

## Acceptance criteria (testable)

- **AC1 — kernel-untouched.** No `crates/ken-kernel` diff; `trusted_base()` delta
  is **zero-new** for the inductive-carrier instances (grep, don't trust prose).
  Any elaborator extension from the design-question §1 is outer-ring only.
- **AC2 — Ω-clean classes.** Each class elaborates as a record/Σ; every law field
  is an Ω-proposition (or a justified truncation), no accidental proof-relevance.
- **AC3 — laws proved, not postulated.** Every canonical instance's law fields are
  **proved** (by induction on the inductive carrier), not `Axiom`. A postulate,
  if any, is a grep-able audited delta with written justification (there should be
  none for List/Option).
- **AC4 — discriminating conformance.** Conformance cases that **fail** for a
  wrong instance: a bogus "Functor" breaking the identity law, or a bogus
  "Monoid" breaking a unit law, is rejected (green-vs-green guard — the test must
  fail for the right reason, [[two-arm-producer-needs-a-case-per-arm]]).
- **AC5 — template documented** for CAT-2.
- **AC6 — examples** of accepted use (a real `map`/`fold`/`<>` program) and
  rejected use.

## Do-not-reopen guardrails

- Kernel stays untouched; the class = record / law = Ω-prop pattern from
  `lawful-classes` is settled — do not relitigate it.
- The higher-kinded class **mechanism** is the enclave's to design, but it stays
  **outer-ring**; a kernel change is out of scope and would re-route to a K-series
  WP.
- Instances land over **inductive** carriers so laws are proved; do **not**
  substitute primitive carriers to dodge a hard proof (that silently reintroduces
  audited delta — the opposite of this WP's point).

## Dependencies / sequencing

- **After effect-composition** (in-flight enclave tail) — it exercises the
  Monad/effect interplay CAT-2 builds on and frees the enclave. CAT-1 elaboration
  picks up once effect-composition merges and the enclave is compacted at that
  seam (§2c).
- **Blocks CAT-2** (Applicative/Monad/Traversable) and **CAT-3** (collection laws,
  which need `Foldable`/`Monoid`).
- Base is `origin/main`; re-verify all current-state claims against the landed
  elaborator at pickup (frame is perishable).
