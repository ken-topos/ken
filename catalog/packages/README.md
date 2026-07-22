# `catalog/packages/` — the standard-package catalog (realized Ken source)

> Status: **layout established by ES4-classes** (the first tranche). This tree
> holds the **realized Ken source** for the standard-package catalog; the
> **spec/index** side is `spec/50-stdlib/` (`../spec/50-stdlib/README.md`). A
> package here is *ordinary Ken* over the built-ins — imported, derivable,
> **kernel-re-checked** — never part of `trusted_base()`.

Catalog source and manifests follow the durable style/refinement contract in
`../docs/program/07-catalog-style-guide.md`: functional builds may land rough
but proved, and explicit refinement WPs raise packages to first-party catalog
quality without weakening proofs.

These are the third tier of the surface taxonomy (`../spec/30-surface/30 §5`):
**optional, explicitly imported, out of `trusted_base()`** — not the
always-present prelude (`30 §4`) and not the built-in surface TCB (`30 §3`). The
monolithic L8 stdlib is **dissolved** into this catalog (ES1); each package is
Ken with its **derivation path from the built-ins stated** and a **declared
`trusted_base()` delta** — **zero** on an inductive carrier; an **audited
delta** where a lawful instance's carrier is a primitive
(`51-lawful-classes.md §6`).

## Layout — Section > Domain > optional Subdomain > Package

This tree mirrors the catalog's controlled seven-Section taxonomy
(`../docs/program/06-catalog-campaign.md`
§"Sections, Domains, Subdomains, and Packages") — the
**normative path ⇔ import rule** is
`../docs/program/07-catalog-style-guide.md`; this section is a pointer, not a
second copy:

- `catalog/packages/<Section>/<Domain>/[<Subdomain>/]<Pkg>.ken[.md]` — the
  **Subdomain** level is present only when the Domain has real internal
  structure (`06`).
- `import <Section>.<Domain>.[<Subdomain>.]<Pkg>` is the **identity map** of
  that path —
  N dotted components → (N−1) directories + a leaf file, PascalCase
  throughout, zero transform, module inferred from the file's path (no
  in-file `module` header).
- **Leaf-or-namespace:** a name at a level is either a package (leaf file) or
  a Domain (directory), never both at once.
- A package's own files sit as siblings of its leaf module: a companion
  `<Pkg>.MANIFEST.md` (name, the spec catalog entry it realizes, its public
  API, the DERIVATION-PATH + `trusted_base()`-DELTA declaration) beside a
  bare `<Section>/<Pkg>.ken`, or one shared `MANIFEST.md` inside a Domain
  directory covering every leaf package in it.
- The loader resolves cross-file `import`
  (`spec/30-surface/33-declarations.md:147-158`); within one compilation
  unit, dotted refs already work too. **No landed catalog entry yet
  exercises the cross-file case** — see `07`'s honesty note — a
  corpus-coverage gap, not a capability one.

The **discipline that distinguishes these from a typical stdlib** (`50-stdlib
§`): a package's core abstractions **carry their laws as propositions — proved,
not postulated** (`../spec/20-verification/`). A `Monoid` is `(append, empty)`
**plus** real proofs of associativity and the unit laws; an `Ord` instance on an
**inductive** carrier carries real total-order proofs
(`../spec/50-stdlib/51-lawful-classes.md §5`) — **zero delta**. An
inductive-carrier instance whose law fields are **postulated/holed** (laws it
*could* have proved) has a **non-empty `trusted_base_delta`** and is **not** a
lawful package entry. (A *primitive*-carrier instance is the honest exception:
its laws are unprovable — no eliminator — so it carries them as an **audited
delta**, transparently declared, `51 §6`.)

## Tranches

| Package | Spec catalog entry | Status |
|---|---|---|
| lawful classes (`Eq`/`DecEq`/`Ord`) | `../spec/50-stdlib/51-lawful-classes.md` | **built** (`Core/Classes/LawfulClasses.ken`) — classes + audited-delta `Int` instances landed; the zero-delta inductive-carrier exemplar (e.g. `Ord Bool`) is a forward WP gated on the kernel gaining Ω-motive `Elim` support (Architect ruling) |
| collections (derived `List`/`Nat` floor + string surface) | `../spec/30-surface/37-strings-collections.md` | **built** (`Data/Collections/Derived.ken`) — the 7-combinator `List`/`Nat` floor + `concat`/`slice`/`char_at`/`eq`/`compare` over `String`, zero-`trusted_base()`-delta; `eq`/`compare` ship as functions, not lawful `DecEq String`/`Ord String` instances (needs a not-yet-landed lawful `DecEq Char`) |
| transport (`subst`/`cong`/`cast`/`sym`/`trans` over the `J` former) | `../spec/50-stdlib/53-transport.md` | **built** (`Core/Logic/Transport.ken`) — the `J` surface former (`elab.rs::infer_j`) + five non-recursive `view` combinators, zero-`trusted_base()`-delta (Map Gap A) |

Subsequent ES4 tranches (collection combinators, formatting, …) follow as their
own WPs against this layout + the laws-PROVED discipline — comprehensive but
non-redundant and lawful. The **systems track** (OS-kernel interface, the Ward
CT-codegen obligation) is a **distinct track**, framed separately — not an ES4
tranche.

> **`wp/ES4-classes-build` (Team Language) landed this layout's `.ken` source**
> for the buildable subset: the three class records + the audited-delta `Int`
> instances (`Core/Classes/LawfulClasses.ken`). The zero-delta,
> law-carrying instance over an inductive carrier (`Ord Bool`/a user `data`) —
> AC3's positive arm — is a **named forward WP**, gated on the kernel gaining
> Ω-motive `Elim` support; see `Core/Classes/LawfulClasses.MANIFEST.md`.
