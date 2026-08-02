# The Ken library

Ken's product-documentation portal. `library/` is **explanatory and
derived** — `spec/` remains the sole normative authority
(`docs/program/12-documentation-program.md`, decision D1). Where a page
restates a rule for usability, it cites the exact spec section rather than
asserting the rule on its own authority.

**Currency:** every page's grounding revision is recorded in
[`STATUS.md`](STATUS.md), which is generated from a repository revision —
never a hand-typed date. Every page's authority class and sources are
declared in [`manifest.toml`](manifest.toml).

## Five ways in

| If you want to... | Go here | Status |
|---|---|---|
| **Read Ken** — understand a program someone else wrote | [`introduction.md`](introduction.md) | current — six-chapter reading curriculum available |
| **Write Ken** — author a checked program | [`guide/surface-reference.ken.md`](guide/surface-reference.ken.md) | current — checked conceptual guides available |
| **Look something up** — a rule, a diagnostic, a CLI option | [`reference/toolchain/`](reference/toolchain/README.md) | current — toolchain command reference available |
| **Find a package** — browse the catalog by task | [`catalog/packages/`](../catalog/packages/README.md) directly, for now | map only — Wave 5 generated portal |
| **Load agent context** — select product knowledge for a coding agent | [`agents/README.md`](agents/README.md) | current |

A route with no library page yet is **mapped**, not silently missing — see
[the Waves table](../docs/program/12-documentation-program.md#4-waves).
Waves 3–6 are a map, not a commitment; each is framed only after its
predecessor's exit condition is met. This table gains real links only as
pages land; it does not point at pages that do not exist.

<a id="whats-here-today-wave-0"></a>

## What's here today

The library contains the **substrate**, the fragment-based reading curriculum
and exercises, agent product-context packs, and checked conceptual guides.
The substrate includes the manifest every page registers in and the generated
status page. Manual and release-point tools make registration, links, and
attested source revisions reviewable; no live library-validation runner
currently enforces those properties.

The checked literate guides are the
[surface reference](guide/surface-reference.ken.md),
[proof techniques](guide/proof-techniques.ken.md), and
[decomposition and abstraction](guide/decomposition-abstraction.ken.md).
`catalog/guide/` contains compatibility pointers rather than a second copy.

## Scope and authority

- `library/` is explanatory and derived; `spec/` remains the sole normative
  authority (D1).
- Every page declares its authority class and sources in `manifest.toml`.
- Every page labels its capability **current / partial / planned /
  unavailable**; planned syntax never appears in a checked current example.

Full program frame:
[`docs/program/12-documentation-program.md`](../docs/program/12-documentation-program.md).
