# CAT-1-build — constructor classes → code

**Owner:** Language build team. **Branch:** `wp/CAT-1-build` (off `origin/main`).
**Status:** Steward frame — **re-based onto `origin/main @ 5a780f8` post-SURF-1**
(2026-07-05). **Base:** `origin/main @ 5a780f8` (CAT-1 elaboration + the landed
`catalog/packages/Core/Classes/` value-level package + SURF-1's `const`/`fn`/`proc`
migration).
**Sequence:** first build of the catalog campaign (Track A, Language). **Blocks
CAT-2 + CAT-3.** Runs in parallel with **CAT-4-build** (Runtime, independent).

This is the execution wrapper for the merged CAT-1 elaboration
(`docs/program/wp/CAT-1-constructor-classes.md` §"Enclave elaboration" E1–E5 +
`spec/50-stdlib/55-lawful-functors.md`). **Read those first — they are canonical
and on `main`.** This frame is the cross-crate build checklist + hard merge
gates. Treat any "current code state" line as perishable — **verify against the
landed elaborator at pickup.**

## 0. What changed since the original frame (SURF-1 landed) — read first

The original frame based on `24a414b` said "SURF-1's D4 migration *later* sweeps
this WP's `view`→`const`/`fn`." **That already happened** — SURF-1 merged
(`5a780f8`) and the `view` keyword is **retired**. Consequences for this build:

- The purity keywords are `const` (0-param pure value), `fn` (pure fn ≥1 param,
  closed row), `proc` (potentially-effectful / effect-row-polymorphic). **Write
  every new declaration in `const`/`fn`/`proc`; do NOT use `view`** (it no longer
  lexes). `Functor`/`Foldable` law proofs and `map`/`fold` helpers are pure →
  `const`/`fn`.
- `catalog/packages/Core/Classes/LawfulFunctors.ken.md` is **already migrated** to `fn`
  (Semigroup/Monoid over `List Nat` + `Bool`, proofs green). D2 = **keep it
  green** under the extension; no re-migration.
- The bidirectional purity checker is live: a `fn` with a non-empty effect row,
  or a `proc` that is actually pure, is a keyword/signature mismatch. Class-method
  and instance bodies must carry the keyword their purity earns.

Everything below stands; only the base ref and the keyword vocabulary moved.

## 1. Objective

Land the **type-constructor class pattern** as running code: the small
outer-ring elaborator extension that admits a class over `f : Type → Type`, then
`Functor`/`Foldable` as **law-carrying classes proved over inductive carriers**
(`List`/`Option`). The value-level `Semigroup`/`Monoid` half already **landed**
with the elaboration (E1, `lawful_functors.ken`) — this WP does **not** rebuild
it, only keeps it green.

## 2. Fixed inputs — do NOT reopen

Carried verbatim from the merged elaboration (E2/E3/E5 + the Steward bundling
ruling). These are settled; the build executes them.

1. **The extension is FIVE pieces, `ken-elaborator`-only, kernel-untouched** —
   E2's four + the E4 parametric-instance-head (Steward bundled it as a distinct
   5th piece, `evt_71j66044qyyv8`):
   1. **AST** — `ClassDecl` carries an optional param-kind (absent ⇒ `Type0`).
   2. **Parser** — admit `class C (f : K) { … }` binder alongside the bare ident.
   3. **Elab** — replace the 3 hard-coded `Term::ty(Level::Zero)` sites in
      `elab_class_decl` (~`elab.rs:1862–1902`, **verify line numbers at pickup**)
      with the elaborated param-kind (default `Type0`). ~10 lines.
   4. **Instance-side head resolution** — `instance Functor List` resolves to the
      **bare `List` indformer** (`Type0→Type0`), not `List a`.
   5. **Parametric instance head** (E4, bundled) — `instance Monoid (List a)`
      generalizes the free `a` as a `(a : Type0)` Π-layer
      (`inst : (a:Type) → Monoid (List a)`), a `declare_def` of a Π-typed
      instance in `elab_instance_decl` (~`elab.rs:2034`, **verify at pickup**).
      Coherent — the overlap registry keys on `head_type_name "List"`.
2. **Functor law = pointwise, one field** (E3, `55 §5.2`): funext is definitional
   (`obs.rs`), so state one canonical pointwise field per law; the point-free
   equation is definitionally-equal and free — **do NOT proliferate a second law
   field.** id: `(a)(x:f a) → Equal (f a) (map a a (idf a) x) x`; applied-fusion
   per `55 §5.2` verbatim. This is the form CAT-2's Monad inherits.
3. **Laws PROVED over inductive carriers, ZERO `Axiom`.** `Functor`/`Foldable`
   instances for `List`/`Option` discharge by **direct induction on the carrier**
   (pointwise is the normal form → the goal *is* the stated law). An `Axiom` on an
   inductive carrier is a **defect**, not an honest postulate (unlike `Int`'s
   audited-primitive precedent) — automatic gate-fail.
4. **Kernel-untouched.** Zero `ken-kernel` diff, no new `Term`/`Decl`,
   `trusted_base()` byte-unchanged. `class`=record (`33 §5.2`), law=Ω-prop
   (`16 §1`), `IsTrue` bridge — all landed machinery.

## 3. Deliverables

### D1 — the five-piece extension (deliverable 1, its own commit)

Build pieces 1–5 above. Lands **first**, on its own commit, gated distinctly on
**CB1 (kernel-untouched) + CB2 (five-piece boundary) + `cargo test --workspace`
green** before the class packages build on it. Pieces 1–4 (higher-kinded param)
and piece 5 (parametric head) are **independent** — build/verify each on its own
discriminating check.

### D2 — Semigroup/Monoid: keep green (already landed + SURF-1-migrated)

`lawful_functors.ken`'s value-level `Semigroup`/`Monoid` (List Nat/Bool
instances, proved, already `fn`-migrated) landed. **No rebuild** — confirm it
stays green under the extension and add the parametric `instance Monoid (List a)`
(piece 5) with a **generic element-type proof**.

### D3 — Functor / Foldable (gated on D1)

- `class Functor f { map; id-law; fusion-law }` over `f : Type → Type`, pointwise
  laws (§2.2). Instances `Functor List`, `Functor Option` — laws by induction.
- `class Foldable f { … }` — pin the **`foldr`-vs-`foldMap` primary** and the
  exact fold laws with the build (E5 defers this to here). Instances
  `Foldable List`, `Foldable Option`, laws proved.
- CV's held `Functor`/`Foldable` conformance cases (in the merged seed, currently
  red-until-built) go **green** against the extension; CV re-reconciles every
  token on the built diff.

## 4. Hard merge gates (CB1–CB7)

- **CB1 — kernel-untouched.** `git diff origin/main -- crates/ken-kernel/` empty;
  no new `Term`/`Decl`; `trusted_base()` byte-unchanged. Grep-verified.
- **CB2 — five-piece boundary held.** The extension is exactly pieces 1–5; **no
  sixth piece, no kernel touch, no new `Term`/`Decl`**. Anything beyond
  **re-forks to Steward before proceeding** (PIN c, verbatim). Automatic
  gate-fail if a kernel touch or a second axis appears.
- **CB3 — laws proved, zero Axiom.** Grep `lawful_functors.ken`: zero
  `Axiom`/postulate/opaque in law fields; `Functor`/`Foldable` List/Option laws
  are real induction proofs. An Axiom on an inductive carrier fails the gate.
- **CB4 — pointwise, one field.** Functor laws are the single pointwise form of
  `55 §5.2`, character-for-character; **no proliferated point-free field**.
- **CB5 — parametric head genuine.** `instance Monoid (List a)` elaborates (no
  `UnresolvedCon`) **and** carries a proof generic in `a`; coherence intact
  (one `Monoid (List _)`, overlap keys on `head_type_name`).
- **CB6 — higher-kinded classes elaborate end-to-end.** `class Functor f` /
  `class Foldable f` admit `f : Type → Type`; `instance Functor List`/`Option`
  resolve to the bare indformer and type-check (the extension actually works,
  not just compiles).
- **CB7 — workspace green + corpus.** `cargo test --workspace` green; the rosetta
  corpus still passes (**16/0**); the landed Semigroup/Monoid + all prior
  packages unbroken. **Purity-keyword green:** every new decl carries the keyword
  its signature/body earns (no `fn`-with-effects, no over-declared `proc`).

**Any forced deviation → surface to the leader → Steward; don't smuggle it** (the
fs-flip / effect-composition precedent — surfaced deviations are exactly what got
caught and re-affirmed clean).

## 5. Gate & acceptance

- **Architect re-certifies** on the *built* diff: **AC1** (kernel-untouched + the
  five-piece boundary actually held — no sixth piece) + the **pointwise-law form**
  (the exact analog of his effect-composition build re-cert; his build-time
  obligation, queued in E5).
- **Language-QA** independent pass (re-derive CB1–CB7 from the diff, not the
  report) + **Verify-QA** + **CI**.
- **Acceptance:** a `.ken` program declares `Functor`/`Foldable` over `List` and
  `Option`, uses `map`/`fold` through the class, and the laws are proved (not
  postulated); the discriminating conformance cases (broken law → rejected
  right-reason, not `is_err()`) pass; workspace + corpus green.

## 6. Dependencies / sequencing

- **Depends on:** CAT-1 elaboration (merged, PR #287) + landed `lawful-functors`
  package + landed `lawful-classes` (Eq/DecEq/Ord) + SURF-1 (`const`/`fn`/`proc`,
  merged `5a780f8`).
- **Parallel with:** CAT-4-build (Runtime, maps/sets/relations — value-level,
  independent; no code conflict).
- **Blocks:** CAT-2 (Applicative/Monad/Traversable) + CAT-3 (collection laws).
- **Conformance:** the merged CAT-1 seed's Functor/Foldable cases + any added.
