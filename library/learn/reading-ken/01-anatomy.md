# Source File Anatomy

This chapter teaches how to orient yourself in a Ken catalog entry. You will
learn where its narrative, checked definitions, laws, examples, and trust
accounting live, then use those landmarks to read one function signature
without reaching for its body.

Catalog entries are literate `.ken.md` documents. Prose and checked Ken share
one source file, while exact `ken` fences tangle into the module
([§1](../../../docs/program/07-catalog-style-guide.md#1-the-entry-is-a-literate-kenmd-document)).
The prose prepares you to read the checked material; it does not replace or
redefine it.

## Catalog Format

An entry begins with a title and statement of intent, an index, and named
reading paths for readers who need different depths. These are followed by
Motivation, Definition, Using it, Laws & proofs, Design notes, References, and
Trust & derivation, in that order
([§2](../../../docs/program/07-catalog-style-guide.md#2-the-standard-entry-format)).
The former Findings section is omitted because language gaps are tracked
outside reader-facing package documentation
([§5](../../../docs/program/07-catalog-style-guide.md#5-findings--retired-from-the-catalog-entry-2026-07-11)).

The order is useful before you know anything about the package. Motivation
states the problem. Definition contains the canonical code. Using it supplies
checked examples, while Laws & proofs states the machine-checked contract.
Design notes explains important choices, and Trust & derivation records the
assumptions and derivation path.

This curriculum draws from a fixed [fragment
set](fragments.md#why-these-seven-and-what-each-is-for). Each selected entry is
checked as a pure library with `ken check`; the selection therefore teaches
from registered package sources rather than invented snippets.

## Empty and Dec

Open
[`catalog/packages/Core/Logic/EmptyDec.ken.md`](../../../catalog/packages/Core/Logic/EmptyDec.ken.md).
Its title and opening sentence identify `Empty` and `Dec` as the subject.
The index shows the whole entry at a glance, and the named reading paths offer
short routes through it.

Motivation explains the problem before presenting code. A `Bool` records which
side of a decision holds but discards its proof, while a proof-irrelevant
proposition carries proof without providing an inspectable branch. `Dec`
combines an inspectable choice with the corresponding proof or refutation.

Definition first distinguishes standard names from definitions authored by the
package. Exact `ken` fences contain functions such as `absurd_empty`, `yes`,
`no`, and `dec_eq_decides`. The surrounding conceptual sketch uses an
illustrative fence, so it does not become part of the tangled module.

Using it applies those names in checked `ken example` fences. Laws & proofs
then states the computation facts for `decide`: the `Yes` case produces
`True`, and the `No` case produces `False`. The proof clauses close with
`Proved` because the closed terms reduce to the same `Bool` constructor.

Design notes explains why proposition-level disjunction and a homogeneous sum
do not provide the required mixed payloads. It also includes a checked rejection
for the reserved name `absurd`. Trust & derivation reports a zero
`trusted_base()` delta and maps the package back to its sources. By the time you
reach that section, the earlier narrative has already told you what each
checked region is meant to establish.

## Declaration Keywords

Definitions in those checked fences begin with keywords that declare static
purity and arity ([§1](../../../spec/30-surface/33-declarations.md#1-definitions)).
`const` introduces a pure value with no explicit value parameters, and `fn`
introduces a pure function with at least one. `proc … visits ρ` introduces a
potentially impure definition with an explicit effect row, while `def`
introduces a type definition or refinement.

These are checked distinctions, not naming conventions. A keyword that
disagrees with the signature or inferred effects is rejected. In particular,
an effectful body cannot hide behind `fn`, and an effect row belongs only on
`proc`.

The authored definitions in `EmptyDec.ken.md` are `fn` declarations. Each
accepts explicit values and remains pure: `absurd_empty` eliminates an
`Empty`, `yes` and `no` construct decisions, and `dec_eq_decides` converts a
`DecEq` result into a proof-carrying decision. Their keyword gives you that
first classification before you inspect any body.

## Reading a Signature

Next open
[`catalog/packages/Data/Sums/Combinators.ken.md`](../../../catalog/packages/Data/Sums/Combinators.ken.md)
and find `get_or_else`. Read its signature before the definition:

```ken
fn get_or_else (a : Type) (d : a) (x : Option a) : a = ...
```

The keyword says the declaration is a pure function. The parameter `a` makes
it polymorphic over a type; `d` supplies a value of that type; and `x` supplies
an optional value of the same type. The result must be an `a`.

The signature does not say whether the result is `d` or the value inside `x`.
The body supplies that computation by matching on `x`. The two attached proof
clauses state the cases separately: `None` returns the default, and `Some v`
returns `v`.

This reading order scales beyond the example. Start with the catalog landmarks,
classify a declaration by its keyword, then read its parameters and result.
Consult the body to learn the computation and the attached proofs to learn
which equations the package exposes as its checked contract.

After this chapter, you can locate an entry's narrative, definitions, laws,
examples, and trust accounting; classify `const`, `fn`, `proc`, and `def`; and
distinguish what a signature establishes from what remains for its body and
proofs.

---

**Sources.** This explanatory chapter follows the catalog format
([§§1–2](../../../docs/program/07-catalog-style-guide.md#1-the-entry-is-a-literate-kenmd-document),
[§5](../../../docs/program/07-catalog-style-guide.md#5-findings--retired-from-the-catalog-entry-2026-07-11)),
the declaration rules
([§1](../../../spec/30-surface/33-declarations.md#1-definitions)), the selected
[fragment set](fragments.md), and the two catalog entries examined above.
