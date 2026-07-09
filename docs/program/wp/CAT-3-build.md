# CAT-3-build — collection laws + views -> code

**Owner:** Language build team. **Branch:** `wp/CAT-3-build` (off
`origin/main`). **Status:** Steward frame. **Base:** `origin/main @ 6e34371`
(CAT-1-build, SURF-1/SURF-2 builds, and CAT-4-build are all landed).
**Sequence:** operator-away priority after CAT-4. CAT-3 is CAT-2-independent
per the merged frame and spec; CAT-2 remains the default consumption order, not
a hard build gate for this WP.

This is the execution wrapper for the merged CAT-3 elaboration:

- `docs/program/wp/CAT-3-collection-laws.md`, especially "Enclave elaboration"
  E1-E8;
- `spec/50-stdlib/57-collections-and-views.md`;
- `spec/50-stdlib/55-lawful-functors.md` for the law-proof grammar inherited
  from CAT-1.

Read those first. This wrapper only pins current-base corrections, the build
deliverables, and the hard gates. Treat any old "current code state" line in
the design frame as perishable and verify it against this branch at pickup.

## 0. Current-base corrections

Several lines in the CAT-3 design were intentionally written before later build
WPs landed. On `origin/main @ 6e34371`, the build starts from this state:

1. **SURF-1 is built.** The `view` declaration keyword is retired in source;
   new declarations must use `const` / `fn` / `proc`. CAT-3's operations and
   proofs are pure, so use `fn` except for any zero-parameter pure value that
   naturally earns `const`. Do not write new `view` keyword declarations.
2. **The `view` family name remains the operator-chosen concept name.** Use
   capitalized type/record names such as `View` / `Lens`, and do not introduce a
   lowercase `view` identifier. The setoid-morphism projection field is
   `project`, per `57 §4.2`.
3. **CAT-1-build is landed.** `catalog/packages/lawful-functors/lawful_functors.ken`
   includes the generic `list_assoc`, `list_left_unit`, `list_right_unit`, and
   the parametric `instance Monoid (List a)`. Reuse these proofs; do not
   re-prove append monoid laws.
4. **CAT-4 is landed but not a dependency.** `catalog/packages/collections/map.ken`
   may offer useful proof style, but CAT-3's target package remains Layer-1
   list/collection laws plus views. Do not migrate CAT-4 map/set laws into this
   WP.
5. **CAT-2 is not a hard gate.** If a proposed CAT-3 proof needs Applicative /
   Monad / Traversable machinery, stop and report the dependency; the intended
   CAT-3 build does not need it.

## 1. Objective

Land the Layer-1 collection law code and the concrete view abstraction with
honest proof surfaces:

- add the missing list-level operations required by CAT-3 where still absent;
- prove the green CAT-3 laws over the landed carriers, with zero new trust
  surface;
- land verified insertion-sort over the `List Bool` carrier with `isSorted` and
  `Perm` proof surfaces;
- land the concrete view/lens records and coherence proofs that the merged
  spec says ship now;
- add focused acceptance coverage that rejects the named wrong surfaces for the
  right reason.

The build is ordinary Ken and elaborator/package work. It must not touch the
kernel.

## 2. Deliverables

### D1 — Structural list operations and laws

In `catalog/packages/collections/collections.ken`, verify which operations are already
present. At frame time, `list_append`, `nth`, `take`, and `drop` are present;
`map`, `filter`, `mem`, `length`, and `min` were absent.

Build the missing operations needed for the CAT-3 green surface, then land:

- `map` length-preservation, if `map`/`length` are built in this WP;
- `filter` membership characterization, if `filter`/`mem` are built in this WP;
- `take`/`drop` decomposition:
  `list_append (take n xs) (drop n xs) = xs`;
- `length (take n xs) = min n (length xs)`, if `length`/`min` are built in this
  WP;
- the append monoid citation/instance path using CAT-1's landed generic proofs.

If an operation is too large to build cleanly, split the law honestly: land the
operation and law surfaces that check now, keep red-until-built surfaces out of
the runtime package, and route a rescope before claiming completion.

### D2 — Verified insertion sort on `List Bool`

Land the minimal insertion-sort route pinned by `57 §3`:

- `count` with an explicit Bool comparator;
- `Perm` as count/multiset equality, not a raw proof-relevant `data ... : Ω`;
- `eqFromOrd le x y := bool_and (le x y) (le y x)`;
- `isSorted` as a structural `Ω` predicate;
- insertion `sort`, structurally recursive and SCT-clean;
- proof-returning `isSorted (sort le xs)` and
  `Perm eqFromOrd xs (sort le xs)` surfaces for `List Bool`.

Use `List Bool` for the proved carrier because it is the Axiom-free
`DecEq`/`Ord` carrier on current `main`. Do not use `Int` or `Char` as the
proof carrier for the green law, because their primitive law surface is
intentionally audited.

### D3 — Concrete views and lens coherence

Land the concrete now-surface from `57 §4`:

- concrete `Lens` / `View` records as ordinary Ken records;
- concrete first-component lens over `Pair Bool Bool`;
- `get-set`, `set-get`, and `set-set` proof-returning coherence laws, using
  `Refl` where the endpoint remains neutral or non-nullary with a neutral
  component;
- concrete forms for the other now flavors where they are straightforward:
  representation/iso, refinement, indexed, and setoid-morphism with field
  `project`.

Do not build the polymorphic `Lens s a` / `Iso a b` family if it requires a
multi-parameter dependent record/class surface. Do not build quotient-carrier
views if they require surface quotient introduction. Those are design-now,
build-later walls already recorded by CAT-3.

### D4 — Acceptance tests

Add focused tests, preferably in a new CAT-3 acceptance file under
`crates/ken-elaborator/tests/`, that cover:

- package loading and derived-not-primitive status for the new collection/view
  surfaces;
- the honest green surfaces for the proved laws;
- a non-permuting sort rejected at the `Perm` surface;
- a non-ordering sort rejected at the `isSorted` surface;
- lens coherence wrong-witness rejection at the named law surface;
- a trust-surface guard that rejects `Axiom`/postulate/opaque shortcuts for the
  CAT-3 law proofs.

Use specific assertions where the local harness supports them; avoid
green-vs-green checks that would pass before the new code is wired.

## 3. Hard gates

- **CB1 — kernel untouched.** `git diff origin/main...HEAD --
  crates/ken-kernel Cargo.lock` is empty. No new `Term`/`Decl`; no trusted-base
  growth.
- **CB2 — proof-returning law surfaces.** Do not count
  `fn law : Prop = ...` as a proved law. Every named law accepted for this WP
  must return evidence for its proposition unless the handoff explicitly names
  an approved rescope.
- **CB3 — zero new executable trust surface.** No executable `Axiom`,
  postulate, primitive declaration, or raw proof-relevant `data ... : Ω` in the
  CAT-3 package diff.
- **CB4 — `Perm` is Ω-sound.** Use count/multiset equality for the built
  surface; do not introduce raw proof-relevant permutation inductives in `Ω`.
- **CB5 — append monoid reuse.** Reuse CAT-1's `list_assoc`,
  `list_left_unit`, and `list_right_unit`; do not duplicate those proofs.
- **CB6 — view walls held.** Multi-parameter class/record and quotient-carrier
  surface needs are reported as build-later walls, not smuggled into this WP.
- **CB7 — validation green.** At minimum: the focused CAT-3 acceptance test,
  any affected existing package tests, `git diff --check origin/main...HEAD`,
  and the normal Language QA workspace gate.

## 4. Review routing

Language leader routes implementation and QA through the normal build loop.
Architect final review is required before Integrator merge for:

- CB1/CB3 trust and kernel-boundary confirmation;
- CB4 `Perm`/`Ω` soundness;
- CB6 view mechanism boundaries;
- any rescope that changes which CAT-3 laws are counted as built.

If implementation hits a dependent-match, record-telescope, quotient-surface, or
proof-motive wall, stop at the smallest Ken-owned reproducer and route it back
to Steward as a mechanism WP candidate. Do not grind around a lower-layer
blocker with permanent package-level ceremony.
