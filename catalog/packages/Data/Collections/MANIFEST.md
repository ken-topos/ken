# `Data/Collections` — the derived `List`/`Nat` combinator floor + `Map`/`Set`

This Domain directory holds two leaf packages (`catalog-taxonomy-paths-imports`
WP, content-call — the two were already separate files, genuinely separable):
`Collections.ken` (⇔ `import Data.Collections.Collections`, this MANIFEST's
subject below) — the derived `List`/`Nat` combinator floor + string surface —
and `Map.ken` (⇔ `import Data.Collections.Map`) — the proved `Map`/`Set` BST
(`spec/50-stdlib/52-map.md`), which depends on `Collections.ken`'s
`list_append`.

**Spec catalog entry:** `spec/30-surface/37-strings-collections.md` §2.4/§2.5/
§2.5.1/§4.1. Realizes the slice-2 half of the derived string surface (slice 1,
`string_to_list_char`/`list_char_to_string`, is the native round-trip landed on
`main`, `L3-strings-roundtrip`).

## Public API

- `data OrdResult = Lt | Eq | Gt` — the 3-way comparison result. Local to this
  module (ES2 retired the prelude's `OrdResult` as un-referenced bloat, but
  sanctions a local declaration where genuinely needed — the `natCmp`
  precedent, `crates/ken-elaborator/tests/val1_string_literals.rs:334`), and
  **exported** so `compare`'s result type is bindable. **Not** re-promoted to
  the prelude here (would reopen ES2's retirement; a second consumer, e.g.
  verified `sort`, triggers that subsume decision later — Architect ruling
  `evt_1stp9sspm6ag8`).
- The original 7-combinator `List`/`Nat` floor: `list_append`, `nth`, `take`,
  `drop`, `natSub` (saturating monus), `list_eq`, `list_compare` — each a
  termination-checked recursive derived def (`declare_recursive_group` +
  `sct_check` + `declare_def`) over the real generic `Term::Elim` (`34 §3`).
  `list_append` is a **distinct** name from the landed `Bytes`-domain
  `append` (FS-effect, `crates/ken-elaborator/src/bytes.rs`) — verified
  distinct global id, no `[FS]` effect row.
- The CAT-3 D1 structural collection slice: `map`, `filter`, `mem`, `length`,
  `min`, plus proof-returning laws `take_drop_decomposition`, `map_length`,
  and `length_take_min`. The filter membership characterization is deliberately
  held out until its comparator/Iff statement is pinned; no `fn law : Prop =
  ...` wrapper is shipped.
- The CAT-3 D2 verified Bool sorting slice: `bool_and`, `boolLeq`,
  `eqFromOrd`, `count`, package-local comparator-indexed `Perm`, transparent
  generic `insert`/`sort`, and the proved `List Bool` carrier
  `insertTrueBool`/`sortBool` with `sortBoolSorted` and `sortBoolPerm`.
  `Perm` is count/multiset equality over an explicit comparator, never a raw
  proof-relevant `data ... : Ω` family.
- The CAT-3 D3 projection-abstraction slice: capitalized ordinary record
  classes `View`, `Lens`, `Iso`, `Representation`, `RefinementView`,
  `IndexedView`, and `SetoidMorphism`. The concrete lens is the first-component
  lens over `Pair Bool Bool`, with proof-returning `fstLensGetSet`,
  `fstLensSetGet`, and `fstLensSetSet` laws. The set-get/set-set laws use the
  full `Equal (Pair Bool Bool) ...` public shape. Polymorphic `Lens s a` /
  `Iso a b` and quotient-carrier views remain build-later walls; the
  setoid-morphism flavor ships now with field `project`.
- `compareChar : Char -> Char -> OrdResult` — a faithful 3-way repackaging of
  the landed `leqChar`/`eqChar` (`crates/ken-elaborator/src/decimal_char.rs`),
  not a re-derivation of Char comparison.
- The 5 derived string ops: `concat`, `slice`, `charAt`, `eq`, `compare` —
  `String -> String -> …`, routed through the real `string_to_list_char`/
  `list_char_to_string` round-trip. `eq`/`compare` ship as plain **functions**
  (tested-not-trusted), not lawful `DecEq String`/`Ord String` instances — that
  transport needs a lawful `DecEq Char`, not yet landed (a tracked follow-on).

## Derivation path + `trusted_base()` delta

- **Zero new kernel feature, zero `trusted_base()` delta.** Every combinator,
  law, and string op is a `declare_def` (checked, upgraded opaque ->
  transparent on SCT success) or an ordinary `fn`; `OrdResult` is a checked
  `data` inductive (kernel-admitted by positivity), never a
  `declare_primitive`/`declare_postulate`/`declare_opaque`. No native interp
  primitive is added for any of the list combinators/laws or the 5 string ops
  (Approach A, Architect ruling `evt_4k1yqah3yvpds`) — deriving trivially
  structural folds keeps the audited primitive set small
  (subsume-don't-proliferate).
- **Package dependency.** The CAT-3 proof terms use `cong`, so harnesses and
  consumers load `catalog/packages/Core/Transport.ken` before this file. The
  dependency is proof-only and adds no trusted-base delta.
- **SCT sound zone.** Every recursive call is an applied call whose decreasing
  argument is a strict subterm of a matched argument (the `Cons` tail and/or
  the `Suc` predecessor) — none of the 7 lean on the SCT's unapplied-self-
  reference / recursion-through-opaque-map over-accept hole.
- **Deliverability honesty.** `String` is canonical w.r.t. `List Char` (the
  `string_to_list_char`/`list_char_to_string` round-trip is a bijection on
  scalar sequences, ADR 0010 §2), so `DecEq String`/`Ord String` instances are
  *soundly deliverable* later — but that transport additionally needs a
  lawful `DecEq Char`, not yet landed. Filing `eq`/`compare` as proof-carrying
  instances now would over-claim the trust level; this package ships the
  functions only.

## A known runtime-performance characteristic (non-blocking, flagged forward)

`crates/ken-elaborator/tests/l3_strings_surface_acceptance.rs`'s
`derived_string_ops_reduce_over_real_roundtrip` test exercises the pinned
`slice 0 99 "abc" ≡ "abc"` conformance case
(`conformance/surface/collections/seed-collections.md` DS-AC3). Evaluating a
`take`/`drop`-style structural recursion at a unary-`Nat` depth of ~99 costs
noticeably more than linear time in the current `ken-interp` evaluator
(empirically ~O(n^3.5–4) in the recursion depth `n`, not exponential — a
correct value, just slow: this one test takes on the order of a few CPU-minutes
at `n = 99`, versus sub-millisecond at `n <= 40`). This is a pre-existing
characteristic of `ken-interp`'s reduction strategy for deep unary-Nat
recursion under nested `match` (no prior test exercised Nat depths anywhere
near this range), **not** a bug introduced by this package's derived defs (the
combinators are correct and match the spec's mandated shapes exactly), and
**not** a soundness concern (the interpreter is the tested-not-trusted ring —
"a wrong value, never a false proof," and the value here is correct). Flagged
to the language-leader/Architect as a forward-tracked `ken-interp` performance
finding; not a blocker for this WP.
