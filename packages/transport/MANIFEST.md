# `transport` — `subst`, `cong`, `cast`, `sym`, `trans`

**Spec catalog entry:** `spec/50-stdlib/53-transport.md` (five combinators)
and `spec/30-surface/34-data-match.md §3.4` (the `J` surface former these
combinators are built from). Build WP: `docs/program/wp/surface-transport.md`
(Map Gap A).

## Public API

- **`J motive base eq`** — not a package export, a **surface term-former**
  (`ken-elaborator/src/elab.rs`, `infer_j`), the identity eliminator. Elaborates
  directly to the kernel's existing `Term::J` (already in `trusted_base()`).
  Infer-mode; the motive is user-written and elaborated bidirectionally from
  `eq`'s inferred type (a bare unannotated `\b' _. G[b']` cannot itself be
  `infer`'d — see the doc comment on `infer_j`).
- **`Eq A a b`** — the kernel's native equality *type*, spelled directly
  (`infer_eq`/the `elab_type` companion arm), needed because the prelude's
  `Equal` alias (`prelude.rs:337`) is `declare_def`-monomorphic at `Type0` and
  cannot express `cast`'s `Eq Type A B` (an equality between two *types*,
  one universe level up). Zero new trust — `Term::Eq` already exists and is
  already in `trusted_base()`; this is surface plumbing to reach it directly
  instead of only through the level-fixed `Equal` alias. Needed to spell the
  five combinators' own signatures (the spec listing's `Eq A a b`, not
  `Equal A a b`).
- `view subst`, `view cong`, `view cast`, `view sym`, `view trans` — the five
  combinators, each a non-recursive `view` over `J` (`transport.ken`).

## Derivation path + `trusted_base()` delta

**Zero delta.** `Term::J`/`Term::Cast`/`Term::Eq` all already exist
(`ken-kernel/src/term.rs`) and are already in `trusted_base()` — this package
adds no `declare_primitive`, no `declare_postulate`, no new `Decl`/`Term`
variant, and touches no `crates/ken-kernel/` file. Every combinator is a
non-recursive `view` (SCT-trivial) that reduces through the kernel's existing
`J`/`Cast` typing and reduction rules (`check.rs::infer_j`, `obs.rs::j_nonrefl`).

`Eq` (the type-position surface spelling) is the same kind of plumbing as
`Refl`/`absurd`/`tt` — no new elimination form, no new reduction rule, purely
a way to *write* an existing kernel type former directly (needed because the
`Equal` alias cannot spell a universe-polymorphic use).

## Naming convention

The spec listing (`53-transport.md §2`) uses capitalized math-style names
(`A`, `B`, `P`) for readability. The surface grammar parses any capitalized
identifier as a constructor reference (`ECon`), never checked against local
scope (`resolve.rs::resolve_expr_ctx`), so a bound type/family parameter
referenced inside an expression *body* (not just a type annotation) must be
lowercase — matching every other package in this catalog (`class Eq a`, never
`Eq A`). `transport.ken`'s five combinators are spelled with lowercase
parameter names accordingly; the shape is otherwise verbatim the spec
listing.

## Consumers

`map-verified-laws`'s four comparison-dependent Branch-B laws (preservation,
found-after-insert, locality, agreement) are gated on this package together
with the non-nullary dependent match of Gap B (`52 §7d`). The `Equal`/`Eq`
alias transparency (CV grounding, `evt_19a2qs0gj4ycz`) means the Map's
existing `Equal Bool (leq …) True` order-hypotheses need no migration to be
`J`-transportable.
