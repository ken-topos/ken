# `packages/` — the standard-package catalog (realized Ken source)

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

## Layout (the pattern every ES4 tranche follows)

One **package per directory**, a module unit in the ES3 sense
(`../spec/30-surface/33 §3` — namespacing + `pub` visibility + abstract export
elaborate to the flat Σ). Each package directory carries:

```
packages/<package>/
  MANIFEST.md      -- name, the spec catalog entry it realizes, its public API,
                   --   and the DERIVATION-PATH + trusted_base()-DELTA
                   --   declaration (the entry's contract from 50-stdlib)
  <module>.ken     -- the Ken source: defs / classes / instances (build-authored)
  ...              -- further module units as needed
```

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
| lawful classes (`Eq`/`DecEq`/`Ord`) | `../spec/50-stdlib/51-lawful-classes.md` | **built** (`lawful-classes/`) — classes + audited-delta `Int` instances landed; the zero-delta inductive-carrier exemplar (e.g. `Ord Bool`) is a forward WP gated on the kernel gaining Ω-motive `Elim` support (Architect ruling) |
| collections (derived `List`/`Nat` floor + string surface) | `../spec/30-surface/37-strings-collections.md` | **built** (`collections/`) — the 7-combinator `List`/`Nat` floor + `concat`/`slice`/`charAt`/`eq`/`compare` over `String`, zero-`trusted_base()`-delta; `eq`/`compare` ship as functions, not lawful `DecEq String`/`Ord String` instances (needs a not-yet-landed lawful `DecEq Char`) |
| transport (`subst`/`cong`/`cast`/`sym`/`trans` over the `J` former) | `../spec/50-stdlib/53-transport.md` | **built** (`transport/`) — the `J` surface former (`elab.rs::infer_j`) + five non-recursive `view` combinators, zero-`trusted_base()`-delta (Map Gap A) |

Subsequent ES4 tranches (collection combinators, formatting, …) follow as their
own WPs against this layout + the laws-PROVED discipline — comprehensive but
non-redundant and lawful. The **systems track** (OS-kernel interface, the Ward
CT-codegen obligation) is a **distinct track**, framed separately — not an ES4
tranche.

> **`wp/ES4-classes-build` (Team Language) landed this layout's `.ken` source**
> for the buildable subset: the three class records + the audited-delta `Int`
> instances (`lawful-classes/lawful_classes.ken`). The zero-delta,
> law-carrying instance over an inductive carrier (`Ord Bool`/a user `data`) —
> AC3's positive arm — is a **named forward WP**, gated on the kernel gaining
> Ω-motive `Elim` support; see `lawful-classes/MANIFEST.md`.
