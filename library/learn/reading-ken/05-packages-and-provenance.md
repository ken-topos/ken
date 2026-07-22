# 05 — Packages and provenance: what a fragment imports and inherits

Chapters [01](01-anatomy.md)–[04](04-effects-capabilities-and-authority.md)
taught you to read a declaration's shape, its contract, its assurance class,
and the authority it requires — all from the fragment sitting in front of
you. This chapter asks where the fragment's own pieces came from: what does
it stand on, where is it filed, and how would it reach another package's
declarations if it needed to? As with chapter 04's authority half, this is
the chapter most tempted to overclaim its own evidence — provenance is
exactly the kind of claim that is easy to state confidently from spec prose
and hard to back with a real, checked instance. Every claim below says
plainly which of those two it is.

## 1. A package's place is its import path

`catalog/packages/` files at `<Section>/<Domain>/[<Subdomain>/]<Pkg>.ken.md`,
and a package's dotted import path is the *same* path spelled with dots
instead of slashes — a mechanical, total identity: an `N`-component dotted
path names the unique leaf file reached through `N − 1` directories
(`catalog/packages/README.md`; `docs/program/07-catalog-style-guide.md`
[§13](../../../docs/program/07-catalog-style-guide.md#13-path--import--the-normative-rule)).
`catalog/packages/Core/Logic/EmptyDec.ken.md` — the fragment chapter
[03](03-assurance-and-trust.md) §2 grounded `proved` in — sits at
`Core/Logic/EmptyDec`, so its import path is `Core.Logic.EmptyDec`; no
lookup table is needed to go from one spelling to the other. A file is an
implicit module named by its own path — there is no in-file `module
Core.Logic.EmptyDec` header to check against, the directory structure *is*
the declaration
(`spec/30-surface/33-declarations.md`
[§3.1](../../../spec/30-surface/33-declarations.md#31-declaring-modules)).

## 2. What a fragment inherits — the derivation path and the `trusted_base()` delta

Chapter [01](01-anatomy.md) §2 pointed you at a selected fragment's Trust &
derivation section without asking what "derivation" meant there. Read now
for what it actually says a fragment stands on. `Data/Sums/Combinators.ken.md`
states it plainly: "`Either` is a checked `data` inductive (kernel-admitted
by positivity) — the only new carrier this package introduces;
`Option`/`Result` are reused from the prelude unchanged." That sentence
names two different kinds of inheritance in one breath — a brand-new
inductive this file itself declares, and two carriers it takes, unchanged,
from elsewhere. `Tooling/Testing/Property.ken.md` states the same shape
about itself: its implementation "derives from the prelude's `List`,
`Result`, `Unit`, `Nat`, `Bytes`, and `UInt8` values" — every one of those
names is inherited, none re-declared. `Core/Logic/Transport.ken.md` and
`Core/Logic/EmptyDec.ken.md` (chapter [03](03-assurance-and-trust.md) §2)
each report a **zero** `trusted_base()` delta by the same discipline: what a
fragment inherits from the built-ins and the prelude costs nothing new,
because the kernel already re-checks it; what a fragment adds new — a
postulate, a primitive, an axiom-backed instance — is exactly what the
delta reports. Provenance, read this way, is not a vague pedigree claim; it
is the fragment's own stated boundary between "reused, already accounted
for" and "new here."

## 3. The tier that boundary sits on

The built-ins/prelude/package distinction that section 2's fragments each
draw on is itself a specified, closed structure, not an informal habit. A
type is **built-in** only if no more primitive Ken could define it — the
surface's own irreducible floor
(`spec/30-surface/30-taxonomy.md`
[§2](../../../spec/30-surface/30-taxonomy.md#2-the-three-tiers),
[§3](../../../spec/30-surface/30-taxonomy.md#3-the-built-in-set--the-surface-tcb-irreducible)).
A type is **prelude** only if a built-in primitive's own type signature
names it — `Bool`, `Char`, and `List`, and nothing else, because those are
the only names the comparison and `String ↔ List Char` primitives mention
(`spec/30-surface/30-taxonomy.md`
[§4](../../../spec/30-surface/30-taxonomy.md#4-the-prelude-tier--ken-defined-always-present-closed)).
Everything else Ken-definable — including every name `Combinators.ken.md`
and `Property.ken.md` reused above — is a **standard package**: optional,
explicitly imported, with its derivation path from the built-ins stated
in-spec
(`spec/30-surface/30-taxonomy.md`
[§5](../../../spec/30-surface/30-taxonomy.md#5-the-standard-package-tier--the-dissolved-stdlib)).
Both re-checked tiers cost nothing in `trusted_base()`; only the built-in
floor does. Read against section 2: a fragment's "derivation path" sentence
is the fragment's own instance of exactly this classification, stated for
its own names rather than argued about in the abstract.

## 4. Import, module, and export are real — and no catalog fragment exercises them yet

`import`, `module`, and `export` are not planned syntax; they are landed,
checked constructs with a real elaborator behind them. `import M` brings
`M`'s exported names into scope under `M.foo`; `import M as N` aliases the
qualifier; `import M (foo, Bar)` brings exactly those names in unqualified,
each optionally renamed
(`spec/30-surface/33-declarations.md`
[§3.2](../../../spec/30-surface/33-declarations.md#32-importing-and-exporting)).
A `program` header may declare which packages it `admits` and which
authority it `capabilities`; these are two independent manifests, one for
instance-dictionary coherence, one for effect authority — a change to one
cannot alter the other
(`spec/30-surface/33-declarations.md`
[§3.2.1](../../../spec/30-surface/33-declarations.md#321-admission-boundary-headers)).
A colliding unqualified name — between a local definition, an import, and
the prelude — is rejected as `AmbiguousReference` before any expression
references it; that check is order-independent and fail-closed
(`spec/30-surface/33-declarations.md`
[§3.3](../../../spec/30-surface/33-declarations.md#33-name-resolution-surface-only-never-reaches-the-kernel)).
And none of it costs anything: a module/import program and its fully
flattened, single-namespace equivalent elaborate to the identical
`trusted_base()` and the identical declaration count — modules, imports,
and visibility are surface-only, adding no kernel feature
(`crates/ken-elaborator/tests/es3_modules_acceptance.rs`,
`module_elaborates_to_identical_flat_sigma`).

**None of this is exercised by a checked `catalog/packages/` fragment
today.** I grepped every file under `catalog/packages/` for `import`,
`module`, `export`, `admits`, `capabilities`, or `program` used as live
code — the only hits anywhere in the tree are prose: `README.md`'s own
description of the rule, and `Data/Sums/Combinators.ken.md`'s references
section naming Haskell's `Data.Either` *module* as an external citation.
No fragment declares or resolves any of these forms. This is not an oversight this chapter papers over — it is the
catalog's own stated boundary, and stating it plainly is the rule, not an
exception: "there is no disk loader yet… a catalog entry that needs
another package's helper today still inlines it… not imports it. State
this plainly in any entry or guide passage that demonstrates the dotted
syntax — don't imply cross-file import works"
(`docs/program/07-catalog-style-guide.md`
[§13](../../../docs/program/07-catalog-style-guide.md#13-path--import--the-normative-rule)).
So: path ⇔ import addressing is real and mechanical (section 1); the
constructs that would resolve an import from one catalog file to another
are real, tested, and zero-cost (this section, above); but no fragment in
this curriculum's reading path, or anywhere else in the catalog, is
checked using them. Label it **unavailable** in checked-fragment form —
the same labelling chapter [03](03-assurance-and-trust.md) §4 used for
`tested` and chapter [04](04-effects-capabilities-and-authority.md) §3 used
for capability tokens.

## 5. A fragment admitting the gap, in its own checked words

`Core/Logic/EmptyDec.ken.md` does not just fail to use `import` — it names,
in its own prose, exactly why, right where it borrows `sym`/`trans`:
"`sym`/`trans` are inlined from `catalog/packages/Core/Logic/Transport.ken`
(self-containment, the same idiom `catalog/guide/proof-techniques.ken.md`
uses for `cong`)." That sentence, and the two small lemma re-declarations
beneath it, are real, current, checked code — the same `ken check`-passing
file chapter [03](03-assurance-and-trust.md) grounded `proved` in — stating
its own provenance honestly: it needed `Transport.ken.md`'s equality
lemmas, cross-file `import` cannot yet deliver them, so it re-declares them
locally rather than silently pretending they came from nowhere. That is
section 4's gap, observed from inside a real fragment rather than argued
from spec prose alone — the strongest instance this curriculum has of a
checked artifact stating its own inheritance limit in its own voice.

## Reader can now answer

- Given a fragment's file path, what is its import path, and what tells you
  that mapping is mechanical rather than a convention you'd need to look up?
- What does a fragment's "derivation path"/`trusted_base()` sentence
  actually distinguish — and which tier (built-in, prelude, standard
  package) does each reused name in it belong to?
- Why does this chapter tell you plainly that no catalog fragment exercises
  `import`/`module`/`export` — rather than teaching those forms as if a
  real cross-package example already existed?
- Where does a real, checked fragment admit, in its own prose, that it
  worked around that exact gap — and how did it work around it?

---

**Grounds this page:**
`catalog/packages/README.md`;
`docs/program/07-catalog-style-guide.md` §13;
`spec/30-surface/30-taxonomy.md` §§2, 3, 4, 5;
`spec/30-surface/33-declarations.md` §§3.1, 3.2, 3.2.1, 3.3.
Authority class: `explanatory` — this page orders and interprets those
sections and the cited fragments' own text; it does not assert a rule they
do not already state. Every citation here rests on the **content-currency**
predicate (`DOC-CURRENCY-ANCHOR`): the cited byte ranges are re-verified
unchanged between `library/REVISION` and `HEAD` by
`scripts/gen-doc-status.sh`, not merely confirmed to name a real ancestor
commit. Section 2's derivation-path claims are grounded in real, current,
checked fragment prose (`Combinators.ken.md`, `Property.ken.md`,
`Transport.ken.md`, `EmptyDec.ken.md`); section 4's zero-cost claim is
grounded in a real producer test
(`es3_modules_acceptance.rs::module_elaborates_to_identical_flat_sigma`),
not fragment prose, because no fragment exercises the mechanism directly.
The import/module/export gap is labelled **unavailable** in checked-fragment
form, per `docs/program/issues/DOC-W1.md` §3's capability-labelling
discipline and `docs/program/07-catalog-style-guide.md` §13's own
instruction not to imply cross-file import works. Fragments cited are drawn
from the already-selected, registered set in [`fragments.md`](fragments.md);
this chapter does not introduce a fresh selection.
