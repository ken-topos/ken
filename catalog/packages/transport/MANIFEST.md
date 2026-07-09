# `transport` — `subst`, `cong`, `cast`, `sym`, `trans`

## Spec catalog entry and build/refinement WP

**Spec catalog entry:** `spec/50-stdlib/53-transport.md` defines the five
derived transport combinators. `spec/30-surface/34-data-match.md §3.4` owns
the `J` surface former that each combinator uses.

**Build WP:** `docs/program/wp/surface-transport.md` (Map Gap A).
**Refinement WP:** `docs/program/wp/catalog-refinement-pilot.md`.

This package is a small, zero-delta catalog package. It exposes ordinary
transport conveniences while keeping the trusted implementation in the existing
kernel equality and `J` machinery.

## Public API

- `subst` — transports a `Type`-valued family along `Eq ty x y`.
- `cong` — maps equality of endpoints through a function.
- `cast` — transports a value along equality between two types.
- `sym` — reverses a propositional equality.
- `trans` — composes two propositional equalities.

The package also relies on two already-trusted surface/kernel constructs:

- **`J motive base eq`** is a surface term former, not a package export. It
  elaborates directly to the kernel's existing `Term::J` path
  (`crates/ken-elaborator/src/elab.rs::infer_j`).
- **`Eq A a b`** is the kernel's native equality type, spelled directly by the
  surface. The prelude `Equal` alias is level-fixed at `Type0`, so it cannot
  express `cast`'s equality between two `Type` values.

## Source map

| Reader task | Where |
|---|---|
| Package contract, proof strategy, trust posture | `transport.ken` package header |
| Lowercase binder convention | `transport.ken` package header; this manifest's "Proof families" section |
| Public operations/proofs | `transport.ken` section "Public transport combinators" |
| `subst` family transport route | `transport.ken`, `subst` |
| `cong` Ω-valued transport route | `transport.ken`, `cong` |
| Raw type transport route | `transport.ken`, `cast` |
| Equality symmetry/composition routes | `transport.ken`, `sym` and `trans` |
| Trust delta | This manifest's "`trusted_base()` delta" section |
| Validation owner/evidence | This manifest's "Validation evidence" section |

No package README is currently needed: the manifest is the durable navigation
surface, and the source is a single short module.

## Derivation path

The five public combinators are all non-recursive wrappers over `J`. `J`
elaborates to the kernel's existing equality eliminator and reduction path
(`Term::J` / `Term::Cast` / `Term::Eq` in `crates/ken-kernel/src/term.rs`).

`Eq` is the direct surface spelling of the native equality type. It is the same
kind of surface plumbing as `Refl` or `absurd`: it makes an existing kernel
type former reachable in source, and it adds no new elimination form or
reduction rule.

## `trusted_base()` delta

**Zero delta.** This package adds no trusted declaration, primitive wrapper,
opaque trusted entry, kernel change, or Cargo change.

Every public name is checked as ordinary Ken source and reduces through the
already-trusted equality machinery. The direct `Eq` spelling is required only
because `cast` needs `Eq Type ty ty2`, while the prelude `Equal` alias is
monomorphic at `Type0`.

## Proof families

- **Family transport (`subst`).** The `J` motive computes the family at the
  transported endpoint.
- **Function congruence (`cong`).** The equality proof is transported through
  the image equality motive in `Omega`.
- **Raw type transport (`cast`).** The equality proof relates two `Type`
  values, so the motive computes a value of the ambient universe.
- **Equality algebra (`sym`, `trans`).** Both are direct `J` eliminations over
  the equality endpoint.

The source uses lowercase binders such as `ty`, `ty2`, and `fam` where the
parameter is referenced in a term body. The spec prose uses math-style
capitalized names (`A`, `B`, `P`), but the current surface parses capitalized
identifiers as constructor references in expression bodies. The lowercase names
preserve the spec shape without changing the public combinator names.

## Consumers and compatibility notes

The public names `subst`, `cong`, `cast`, `sym`, and `trans` are stable and are
preserved by catalog refinement.

`catalog/packages/lawful-functors` uses `cong`/`sym`/`trans` for inductive congruence
steps. The map verified-laws work also depends on the transport package for the
Gap-A route around stuck comparison transport. The `Equal`/`Eq` alias
transparency means existing `Equal Bool (leq ...) True` order hypotheses remain
`J`-transportable without source migration.

## Validation evidence

Primary acceptance coverage is
`scripts/ken-cargo test -p ken-elaborator --test surface_transport_acceptance -- --nocapture`.

Refinement review also checks:

- `git diff --check`;
- no diff under `crates/ken-kernel`, `Cargo.lock`, or `conformance`;
- trust-drift grep over `catalog/packages/transport` for trusted declarations,
  primitive wrappers, and raw proof-relevant data declarations;
- public-name availability for `subst`, `cong`, `cast`, `sym`, and `trans`.
