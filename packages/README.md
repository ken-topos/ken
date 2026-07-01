# `packages/` — the standard-package catalog (realized Ken source)

> Status: **layout established by ES4-classes** (the first tranche). This tree
> holds the **realized Ken source** for the standard-package catalog; the
> **spec/index** side is `spec/50-stdlib/` (`../spec/50-stdlib/README.md`). A
> package here is *ordinary Ken* over the built-ins — imported, derivable,
> **kernel-re-checked** — never part of `trusted_base()`.

These are the third tier of the surface taxonomy (`../spec/30-surface/30 §5`):
**optional, explicitly imported, out of `trusted_base()`** — not the
always-present prelude (`30 §4`) and not the built-in surface TCB (`30 §3`). The
monolithic L8 stdlib is **dissolved** into this catalog (ES1); each package is
Ken with its **derivation path from the built-ins stated** and a **zero
`trusted_base()` delta`.

## Layout (the pattern every ES4 tranche follows)

One **package per directory**, a module unit in the ES3 sense
(`../spec/30-surface/33 §3` — namespacing + `pub` visibility + abstract export
elaborate to the flat Σ). Each package directory carries:

```
packages/<package>/
  MANIFEST.md      -- name, the spec catalog entry it realizes, its public API,
                   --   and the DERIVATION-PATH + ZERO-trusted_base()-DELTA
                   --   declaration (the entry's contract from 50-stdlib)
  <module>.ken     -- the Ken source: defs / classes / instances (build-authored)
  ...              -- further module units as needed
```

The **discipline that distinguishes these from a typical stdlib** (`50-stdlib
§`): a package's core abstractions **carry their laws as propositions — proved,
not postulated** (`../spec/20-verification/`). A `Monoid` is `(append, empty)`
**plus** real proofs of associativity and the unit laws; an `Ord` instance
carries real total-order proofs (`../spec/50-stdlib/51-lawful-classes.md §5`).
An instance whose law fields are postulated/holed has a **non-empty
`trusted_base_delta`** and is **not** a lawful package entry.

## Tranches

| Package | Spec catalog entry | Status |
|---|---|---|
| lawful classes (`Eq`/`DecEq`/`Ord`) | `../spec/50-stdlib/51-lawful-classes.md` | spec pinned (ES4-classes); build follow-on (Team Language) |

Subsequent ES4 tranches (collection combinators, formatting, …) follow as their
own WPs against this layout + the laws-PROVED discipline — comprehensive but
non-redundant and lawful. The **systems track** (OS-kernel interface, the Ward
CT-codegen obligation) is a **distinct track**, framed separately — not an ES4
tranche.

> **This WP (ES4-classes) is spec+conformance-only.** It establishes this layout
> and the lawful-classes contract; the **`.ken` source** (classes + canonical
> law-carrying instances) is the named **Team-Language build follow-on**, which
> populates `packages/lawful-classes/` against
> `../spec/50-stdlib/51-lawful-classes.md`.
