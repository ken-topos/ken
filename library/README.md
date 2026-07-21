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
| **Read Ken** — understand a program someone else wrote | [`introduction.md`](introduction.md) | current (the full reading curriculum is Wave 1, planned) |
| **Write Ken** — author a checked program | *not yet available here* | planned — Wave 1/3 |
| **Look something up** — a rule, a diagnostic, a CLI flag | *not yet available here* | planned — Wave 4 |
| **Find a package** — browse the catalog by task | [`catalog/packages/`](../catalog/packages/README.md) directly, for now | planned generated portal — Wave 5 |
| **Load agent context** — select product knowledge for a coding agent | [`agent/playbooks/tools/write-ken.md`](../agent/playbooks/tools/write-ken.md), for now | planned — Wave 2 |

A route with no library page yet is **planned**, not silently missing — see
[the Waves table](../docs/program/12-documentation-program.md#4-waves) for
the wave it lands in (`STATUS.md` lists what is landed, not what is
planned). This table itself will gain real links as each wave lands; it
does not point at pages that do not exist.

## What's here today (Wave 0)

Wave 0 lands the **substrate**, not the content: the manifest every page
registers in, the generated status page, and the gates that keep both
honest — never a hand-typed date, never an unregistered page, never a link
or a cited source that has gone stale. [`introduction.md`](introduction.md)
is the first real page built on that substrate.

`catalog/guide/`'s checked literate guides have **not moved yet** — they
stay exactly where they are until their `ken example`/`ken reject` fence
gate lands and passes (Wave 3). The plan for what moves, what stays
canonical, and what becomes a pointer is recorded in
[`docs/program/13-documentation-migration-ledger.md`](../docs/program/13-documentation-migration-ledger.md).

## Scope and authority

- `library/` is explanatory and derived; `spec/` remains the sole normative
  authority (D1).
- Every page declares its authority class and sources in `manifest.toml`.
- Every page labels its capability **current / partial / planned /
  unavailable**; planned syntax never appears in a checked current example.

Full program frame:
[`docs/program/12-documentation-program.md`](../docs/program/12-documentation-program.md).
