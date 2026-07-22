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
names is inherited, none re-declared. Both entries also state, in their own
words, that this reuse costs nothing new: Combinators' own "`trusted_base()`
delta: zero" and Property's own "The declared `trusted_base()` delta is
**zero**" are each a literal, unqualified zero. Read this against chapter
[03](03-assurance-and-trust.md) §2, precisely: not every fragment's zero is
the *same* zero. `Core/Logic/EmptyDec.ken.md` reports something narrower —
"**Zero new trust category**" — and immediately qualifies it: "Instantiating
`dec_eq_decides` at a carrier whose `DecEq` instance has an audited
assumption retains that instance's declared delta." That is not a literal
zero; it is a statement that *this entry itself* adds no new postulate or
primitive, while explicitly preserving whatever delta a caller's own choice
of carrier already carries. Provenance, read this way, is not a vague
pedigree claim, and it is not always the same claim twice — it is each
fragment's own stated boundary between "reused, already accounted for" and
"new here," and that boundary can be a flat zero or a conditional one
depending on what the fragment actually says.

## 3. The tier that boundary sits on

The built-ins/prelude/package distinction that section 2's fragments each
draw on is itself a specified, closed structure, not an informal habit. A
type is **built-in** only if no more primitive Ken could define it — the
surface's own irreducible floor; the built-in set explicitly names
`Int`/`Float`/`Char`/`String`/`Bytes` as the primitively-provided types
(`spec/30-surface/30-taxonomy.md`
[§2](../../../spec/30-surface/30-taxonomy.md#2-the-three-tiers),
[§3](../../../spec/30-surface/30-taxonomy.md#3-the-built-in-set--the-surface-tcb-irreducible)).
A type is **prelude** only if a built-in primitive's own type signature
names it and it is not already provided by the kernel — a closed set the
taxonomy pins today as exactly `{Bool, Char, List}`
(`spec/30-surface/30-taxonomy.md`
[§4](../../../spec/30-surface/30-taxonomy.md#4-the-prelude-tier--ken-defined-always-present-closed)).
Everything else Ken-definable, with no primitive forcing its presence, is a
**standard package**: optional, explicitly imported, with its derivation
path from the built-ins stated in-spec
(`spec/30-surface/30-taxonomy.md`
[§5](../../../spec/30-surface/30-taxonomy.md#5-the-standard-package-tier--the-dissolved-stdlib)).

Read section 2's two fragments against this taxonomy carefully, rather than
taking their own wording as the classification: Property's prose calls
`List`, `Result`, `Unit`, `Nat`, `Bytes`, and `UInt8` all "the prelude's" —
but that is the fragment's own informal shorthand for "the ambient
environment," not the taxonomy's technical membership test. Checked
against the closed set above, `List` genuinely is prelude; `Bytes` is
**built-in**, not prelude — it is one of the five primitively-provided
types named in §3. `Result`, `Unit`, `Nat`, and `UInt8` are not named in
either closed list this taxonomy chapter states, so this page does not
classify them; that is an honest gap in what the cited spec section pins,
not a fact this chapter can derive by itself. The lesson is section 2's
real point, sharpened: read a fragment's own vocabulary as its author's
convenience, and check any technical-sounding word — "prelude" included —
against the section of the normative source that actually defines it
before repeating it as fact.

## 4. Import, module, and export are real, tested, and resolve across real files

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

This is not merely a single-file mechanism: the loader that follows an
`import` edge across two **separate files on disk** is real and tested
today. A real acceptance test writes an `A` file that `import`s a `B` file
as a genuine, separate compilation unit, and elaborating `A` through the
elaborator's `elaborate_module_from_roots` resolves `A`'s reference to
`B`'s real declaration — lazily (an unrelated third file with invalid
syntax is never touched, because nothing imports it) and with a cache, so
loading the same module twice reuses the first result rather than
re-elaborating it
(`spec/30-surface/33-declarations.md`
[§3.2](../../../spec/30-surface/33-declarations.md#32-importing-and-exporting);
`crates/ken-elaborator/tests/n2_in_repo_loader.rs`,
`cross_file_import_resolves_lazily_through_plural_root_api_and_caches`).
Cross-file import is not an unimplemented language feature.

What is true, and worth stating precisely rather than either overclaiming
or underclaiming it: **no fragment in `catalog/packages/` uses `import`,
`module`, `export`, `admits`, `capabilities`, or `program` today**, and the
command this curriculum's fragments are checked with — `ken check`, which
every "still checks" claim in [`fragments.md`](fragments.md) rests on —
elaborates a single file at a time; it does not call the roots-based
loader this section just cited. So a catalog fragment run through `ken
check` alone would not, today, follow a cross-file `import` even though
the underlying mechanism resolving one is real and tested. That is a
**corpus-and-tooling gap** — no fragment is written to exercise the
mechanism, and the one CLI command used to check fragments doesn't invoke
it — not a capability the language lacks. I grepped every file under
`catalog/packages/` for these six forms used as live code: the only hits
anywhere in the tree are prose — `README.md`'s own description of the
path/import rule, and `Data/Sums/Combinators.ken.md`'s references section
naming Haskell's `Data.Either` *module* as an external citation. Label the
absence **unavailable** in checked-fragment form — the same labelling
chapter [03](03-assurance-and-trust.md) §4 used for `tested` and chapter
[04](04-effects-capabilities-and-authority.md) §3 used for capability
tokens — but label it as what it actually is: a gap in what the corpus and
its tooling exercise, not a gap in what the language can do.

## 5. A fragment naming its own borrowing, in its own checked words

`Core/Logic/EmptyDec.ken.md` does not use `import` — it re-declares two
small lemmas it needs, and names exactly what it is doing right where it
does it: "`sym`/`trans` are inlined from
`catalog/packages/Core/Logic/Transport.ken` (self-containment, the same
idiom `catalog/guide/proof-techniques.ken.md` uses for `cong`)." That
sentence, and the two small lemma re-declarations beneath it, are real,
current, checked code — the same `ken check`-passing file chapter
[03](03-assurance-and-trust.md) grounded `proved` in. Read it precisely,
against section 4: the fragment's own word is "self-containment," a style
choice, not a claim that cross-file `import` failed or could not deliver
`Transport`'s lemmas — nothing in this entry says that, and section 4
already showed that claim would be false. What this fragment does show,
honestly, is section 4's corpus gap from the inside: a real, checked entry
that needed another package's declarations and, in a corpus where no
fragment's own checking path exercises cross-file `import`, chose to
restate them locally rather than reach for a mechanism nothing else in the
catalog uses yet. Reading the fragment's own reason correctly — a
convention, not a limitation — matters more than what conclusion its
neighbor sections happen to support.

## Reader can now answer

- Given a fragment's file path, what is its import path, and what tells you
  that mapping is mechanical rather than a convention you'd need to look up?
- What does a fragment's "derivation path"/`trusted_base()` sentence
  actually distinguish — and why can't you take a fragment's own casual
  word for a name's tier without checking it against the taxonomy that
  actually defines tier membership?
- Cross-file `import` resolution is real and tested — so why does this
  chapter still tell you no catalog fragment exercises `import`/`module`/
  `export`, and what precisely is missing if not the mechanism itself?
- Where does a real, checked fragment name its own borrowing from another
  package in its own words — and what does that fragment's own reason for
  it actually claim, versus what a careless reading might assume it claims?

---

**Grounds this page:**
`catalog/packages/README.md` and `docs/program/07-catalog-style-guide.md`
§13 (path/import addressing only — see the note below on what this page
does **not** inherit from either);
`spec/30-surface/30-taxonomy.md` §§2, 3, 4, 5;
`spec/30-surface/33-declarations.md` §§3.1, 3.2, 3.2.1, 3.3.
Authority class: `explanatory` — this page orders and interprets those
sections and the cited fragments' own text; it does not assert a rule they
do not already state. Every citation here rests on the **content-currency**
predicate: the committed ledger, `library/SOURCE-ATTESTATIONS`, binds each
cited path to its exact tracked blob OID as of a Librarian review, and
`scripts/gen-doc-status.sh` compares the current tracked blob for every
cited path against that ledger — not merely confirming `library/REVISION`
names a real ancestor commit (`REVISION` is a provenance/bootstrap anchor
only; it does not re-verify cited bytes). Content currency is necessary
but not sufficient: a citation can be current *and* still not carry the
semantic claim made from it, which is why every claim below is checked
against what its cited source actually says, not just whether the
source's bytes are unchanged. Section 2's
derivation-path and
`trusted_base()` claims are grounded in real, current, checked fragment
prose (`Combinators.ken.md`, `Property.ken.md`, `Transport.ken.md`,
`EmptyDec.ken.md`), each quoted precisely enough to preserve the
distinction between a literal zero delta and EmptyDec's narrower "zero new
trust category." Section 3's tier claims are checked directly against
`30-taxonomy.md`'s own closed lists rather than taken from a fragment's
informal use of "prelude"; where the cited taxonomy section does not pin a
name's tier, this page says so rather than guessing. Section 4's zero-cost
and cross-file-resolution claims are grounded in real producer tests
(`es3_modules_acceptance.rs::module_elaborates_to_identical_flat_sigma`;
`n2_in_repo_loader.rs::cross_file_import_resolves_lazily_through_plural_root_api_and_caches`),
not fragment prose, because no fragment exercises either mechanism
directly. **This page does not cite `docs/program/07-catalog-style-guide.md`
§13 or `catalog/packages/README.md`'s prose for any claim about whether
cross-file `import` resolves** — this page's own import/module/export gap
claim is instead grounded directly in the whole-catalog grep reported in
section 4 and the `ken check` single-file elaboration path
(`crates/ken-cli/src/main.rs::check_file`), and is scoped precisely to
"no fragment exercises this, and the fragment-checking CLI path doesn't
invoke the loader" — not to "the loader doesn't exist." Fragments cited
are drawn from the already-selected, registered set in
[`fragments.md`](fragments.md); this chapter does not introduce a fresh
selection.
