# 01 — Anatomy: orienting in a source file

Each of this curriculum's selected fragments is a **literate `.ken.md`
document** — a catalog package's primary artifact, carrying narrative
prose and checked code in one file, not a separate doc comment bolted onto
a separate source file
(`docs/program/07-catalog-style-guide.md`
[§1](../../../docs/program/07-catalog-style-guide.md#1-the-entry-is-a-literate-kenmd-document)).
This chapter teaches the shape that gives you: where to look first, what
each part is for, and which parts are the checked contract versus which are
prose written to help you get there.

## 1. The shape every entry follows

A standard catalog entry has a fixed **front matter** — an H1 title with a
one-line statement of intent, an index of anchor links, and a set of named
reading paths that route a *Newcomer*, a *Practitioner*, a *Researcher*, and
someone *porting from Haskell/Lean/Agda* to a different depth — followed by
**required sections, in order**: Motivation, Definition, Using it, Laws &
proofs, Design notes, References, Trust & derivation
(`07-catalog-style-guide.md`
[§2](../../../docs/program/07-catalog-style-guide.md#2-the-standard-entry-format)).
Current entries therefore have seven required sections; `Findings` is not
one of them
(`07-catalog-style-guide.md`
[§5](../../../docs/program/07-catalog-style-guide.md#5-findings--retired-from-the-catalog-entry-2026-07-11)).
That fixed order is itself information: it tells you, before you read a
single declaration, where the checked code lives (Definition), where the
checked contract lives (Laws & proofs), and where the trust accounting
lives (Trust & derivation).

## 2. A guided walk through one real entry

`catalog/packages/Core/Logic/EmptyDec.ken.md` — one of this curriculum's
selected fragments
([`fragments.md`](fragments.md)) — follows the shape exactly. Opening it,
in order:

- Its H1 and one-line intent name the entry's subject: `Empty`/`Dec`,
  computational falsity and decidability.
- Its **Motivation** section explains what gap `Empty`/`Dec` closes (a
  `Bool` tells you which side of a decision holds but discards the proof; a
  proof-irrelevant `Ω` disjunction carries the proof but can't tell you
  which side) — before you have seen a single line of code.
- Its **Definition** section is where the checked code begins: the actual
  `data`, `fn`, `class`, and `instance` declarations, in ` ```ken ` fences
  that tangle to a compilable module. This is the canonical source — the
  narrative around it explains, it does not define.
- Its **Using it** section shows the same names applied, in ` ```ken
  example ` fences.
- Its **Laws & proofs** section states and discharges the entry's actual
  contract — here, that `decide` is an honest reflection of `Dec`'s tag,
  proved for the concrete `DecEq Bool` instance.
- Its **Design notes** section explains rejected alternatives (why not
  `Ω`'s `Or`, why not a homogeneous `Sum`) and includes a deliberately
  rejected declaration, in a ` ```ken reject ` fence, showing that
  redeclaring the reserved name `absurd` is a hard error.
- Its **Trust & derivation** section closes with the entry's `trusted_base()`
  delta (zero, here) and a source map back to the sections above.

Nothing in that walk required reading the code first. The narrative
sections tell you what to expect from the checked sections before you reach
them — that ordering is the anatomy this chapter is teaching you to use.

## 3. The declaration keywords you will meet in the code

Inside every Definition section, four keywords carry distinct, checked
meaning — a declaration's keyword **declares its static purity**, and a
mismatch between the keyword and what the body actually does is a hard
error, not a style preference
(`spec/30-surface/33-declarations.md`
[§1](../../../spec/30-surface/33-declarations.md#1-definitions)):

- **`const`** — a pure value (zero explicit value parameters).
- **`fn`** — a pure function (at least one explicit value parameter). Its
  body must in fact be pure; an effectful `fn` is rejected.
- **`proc … visits ρ`** — a potentially impure definition carrying an
  explicit effect row `ρ`. This is the only keyword an effect row may sit
  on.
- **`def`** — a type-level definition: a plain alias, or a base type
  narrowed by conditions.

`EmptyDec.ken.md`'s `absurd_empty`, `yes`, `no`, and `dec_eq_decides` are
all `fn`: each is a pure function over already-constructed values, and
`ken check` on this file is exactly the check that confirms every one of
them keeps that promise.

## 4. Reading exercise: one signature, before its body

Take `get_or_else` from `catalog/packages/Data/Sums/Combinators.ken.md`
(also a selected fragment) — read only its signature first:

```ken
fn get_or_else (a : Type) (d : a) (x : Option a) : a = ...
```

Before reading the body, the signature alone tells you: this is a pure
function (`fn`); it is polymorphic in a type `a`; given a default value and
an `Option a`, it returns an `a`. What it does *not* yet tell you is
*which* value it returns in each case — that is what the Laws & proofs
section states next, as two separate `proof … for get_or_else` clauses, one
per constructor of `Option`. Reading the signature before the body is not
a stylistic habit; it is the discipline the next chapter builds on.

## Reader can now answer

- Where does a catalog entry's checked code live, versus its narrative,
  versus its stated contract, versus its trust accounting?
- Given an unfamiliar declaration's keyword (`const`, `fn`, `proc`, `def`),
  what does the keyword alone promise about that declaration's purity and
  effects?
- Reading a signature before its body: what can you learn from the
  signature alone, and what is still left to the body and its proofs?

---

**Grounds this page:**
`docs/program/07-catalog-style-guide.md` §§1, 2, 5;
`spec/30-surface/33-declarations.md` §1.
Authority class: `explanatory` — this page orders and interprets those
sections and the cited fragments' own text; it does not assert a rule they
do not already state. Fragments cited are drawn from the already-selected,
registered set in [`fragments.md`](fragments.md); this chapter does not
introduce a fresh selection.
