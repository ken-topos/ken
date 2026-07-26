# Author a package

## 1. Use when

Use this module to create or revise a canonical literate package entry under
`catalog/packages/`. Do not use it for an ordinary program, generated package
reference, or speculative package that has no checked implementation.

## 2. Prerequisites

Load `../core/write-ken.md`, `../core/proof-and-trust.md`, and
`../core/toolchain.md`. Read `docs/program/07-catalog-style-guide.md` and the
nearest existing package entries in the same catalog domain.

## 3. Current capability

A package is a `.ken.md` literate source whose checked fences tangle to its
implementation. Current entries carry a contents section, motivation,
definition, usage, laws and proofs, design notes, references, and trust and
derivation.

## 4. Canonical forms

Use this order:

```text
front matter and title
Contents
Motivation
Definition
Using it
Laws & proofs
Design notes
references
Trust & derivation
```

The contents section links to every required section. `Findings` is deliberately
absent: style-guide §5 retired it from reader-facing entries in favor of live gap
routing. Choose the catalog path by task/domain. The path and declared module
identity must agree with the catalog convention.

## 5. Invariants and prohibitions

- The literate entry is canonical source, not commentary beside another source.
- Checked fences contain code; surrounding prose explains it.
- Public laws require checked proof terms or an explicit non-proved status.
- Trust accounting names new and inherited assumptions.
- Do not guess a package name or create a parallel package that an existing
  abstraction subsumes.

## 6. Decision procedure

1. Search the catalog by task and inspect neighboring entries.
2. State the package's distinct purpose and non-goals.
3. Design the smallest public API and laws.
4. Check implementation and proofs before polishing prose.
5. Complete trust, examples, validation, and references.
6. Format, check the entire `.ken.md`, and stop if identity or trust remains
   ambiguous.

## 7. Failure signatures

Path/module mismatch indicates identity drift; a fence error names the
authoring layer; a law without a proof term is an assurance gap; an unexplained
trusted-base delta is a hard stop; and an already-existing equivalent package
indicates proliferation.

## 8. Validation

Run `ken fmt` and `ken check` on the exact entry, inspect its formatted prose
and fences, verify every internal link, and record the trusted-base result.
Run only the package's targeted acceptance tests.

## 9. Authority and sources

Package structure and naming come from
`docs/program/07-catalog-style-guide.md` §2, with `Findings` omitted per §5;
language rules come from `spec/`; checked practice comes from `catalog/guide/`
and neighboring entries. Revision: `library/agents/manifest.toml`.

## 10. Known unavailable or partial behavior

Generated package cards and domain modules are not authored here. A catalog
entry checked as a single file may not demonstrate cross-file import behavior.
If reuse needs an unverified package path or the package would require a new
trust boundary, stop and route that design gap instead of publishing a
plausible-looking entry.
