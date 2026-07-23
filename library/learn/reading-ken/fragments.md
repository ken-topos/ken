# Reading-Ken fragment set

The `learn/reading-ken/` chapters all teach from this one fragment set,
recorded here as a named artifact so a later change is a visible edit to
this file — with a readable blast radius — rather than scattered rework
nobody connects.

**Selection rule:** real, checked `catalog/packages/` fragments spanning
enough domain variety to avoid an "unrelated snippets" collection, without
building the curriculum around the catalog's one whole program
(`catalog/examples/CommandLine/Forge.ken.md` — pure spec-data, no effects)
and without substituting an invented toy.

## What "still checks" means, and the mechanism used

A pure-library catalog entry (no `proc main`) is validated with
`ken check <file>`: it runs the same elaboration and literate fence-role
checking `ken run` does, then stops before the IO-drive step
(`docs/program/07-catalog-style-guide.md` §3). **Every row below reports
that mechanism's actual exit code, not an assertion that the file "should"
still check.** Run at `origin/main @ cf91ec5a4ee2b557540f6a894cb9d8825638a634`,
from the repository root, against the `target/debug/ken` binary built from
that same revision:

```
$ ./target/debug/ken check <path>
```

| # | Fragment | Domain | `ken check` result |
|---|---|---|---|
| 1 | `catalog/packages/Core/Logic/EmptyDec.ken.md` | Core / Logic | exit 0 |
| 2 | `catalog/packages/Core/Logic/Transport.ken.md` | Core / Logic | exit 0 |
| 3 | `catalog/packages/Data/Sums/Combinators.ken.md` | Data / Sums | exit 0 |
| 4 | `catalog/packages/Capability/Console/Text.ken.md` | Capability / Console | exit 0 |
| 5 | `catalog/packages/Capability/Filesystem/Errors.ken.md` | Capability / Filesystem | exit 0 |
| 6 | `catalog/packages/Capability/System/IO.ken.md` | Capability / System | exit 0 |
| 7 | `catalog/packages/Tooling/Testing/Property.ken.md` | Tooling / Testing | exit 0 |

## Why these seven, and what each is for

1. **`Core/Logic/EmptyDec.ken.md`** — `Empty`/`Dec`, computational
   decidability. Carries an explicit "Trust & derivation" section, so it
   grounds a `proved` reading directly from a real entry rather than from
   prose about one.
2. **`Core/Logic/Transport.ken.md`** — `subst`/`cong`/`cast`/`sym`/`trans`,
   the equality-reasoning vocabulary later proofs in the corpus build on.
   Types, contracts, and proofs.
3. **`Data/Sums/Combinators.ken.md`** — the `Option`/`Result`/`Either`
   combinator floor, each combinator paired with a real proof of its
   defining equation. Ordinary data plus proofs in one small entry —
   program anatomy and proofs together.
4. **`Capability/Console/Text.ken.md`** — four small `IO`-effectful helpers
   over the Console capability, `visits [Console]` on every signature.
   Effects, capabilities, authority.
5. **`Capability/Filesystem/Errors.ken.md`** — capability-authority
   (`AFull`) plus an explicit, honest security-boundary limitation stated
   in its own prose ("the current authority check is coarse and is *not*
   path-confined"). Useful for authority reading and for an `unknown`/
   `delegated` reading, since the entry states its own boundary rather than
   overclaiming.
6. **`Capability/System/IO.ken.md`** — proof terms over buffer I/O whose own
   text states plainly that "exactly-once settlement and liveness remain
   runtime-enforced, delegated boundary properties." A real entry that
   names its own `delegated` claims in prose — assurance, trust, and
   runtime assumptions.
7. **`Tooling/Testing/Property.ken.md`** — a deterministic finite property
   runner over explicit samples, whose own Motivation section draws the
   `tested`-vs-`proved` line explicitly ("Properties here are computations,
   not propositions. They test the executable shadow of a contract without
   assuming or proving that contract."). Assurance reading and a natural
   source for checked exercises.

## A domain gap, reported rather than built around

Core/Classes and Data/Numeric are two more domains in `catalog/packages/`
with real content, but as of the SHA above, **no fragment in either
domain passes `ken check` standalone.**

Four files fail the same, diagnosable way — an `UnresolvedCon` naming a
name defined in a *different* package file:

| File | Missing name |
|---|---|
| `Core/Classes/LawfulFunctors.ken.md` | `list_append` |
| `Core/Classes/LawfulClasses.ken.md` | `OrdResult` |
| `Core/Classes/EffectfulClasses.ken.md` | `Functor` |
| `Data/Numeric/Nat/Arithmetic.ken.md` | `cong` |

This is not those four entries being broken; it is a corpus-coverage gap,
not a loader capability one. The loader itself resolves cross-file `import`
(`spec/30-surface/33-declarations.md#32-importing-and-exporting`); what's
still true is
narrower — no landed catalog entry yet exercises the cross-file case, so "a
catalog entry that needs another package's helper today still inlines
it… not imports it." These four entries were written assuming (or awaiting)
that usage and are not currently self-check-able in isolation.

A fifth file, `Data/Numeric/Nat/Order.ken.md`, also fails `ken check`, but
**not** the same way: `KernelRejected(TypeMismatch)`, naming no missing
symbol. It is kept out of the table above and out of the loader
explanation — its cause has not been established here, and grouping it
with the other four would overstate what the evidence shows.

The substitution made instead — `Core/Logic` for `Core/Classes`,
`Data/Sums` for `Data/Numeric` — still delivers proofs-over-data and
typeclass-adjacent material (`Transport`'s `cong`/`sym`/`trans` are exactly
among the names the four `UnresolvedCon` failures above needed and
couldn't reach), so the set's domain-variety goal is met without a
whole-program crutch or an invented toy.

## Currency of this artifact

This file's own manifest record cites all seven fragment paths above as its
`sources`. Its currency claim rests on the **content-currency** predicate
(`source-currency` in `manifest.toml`): the committed ledger,
`library/SOURCE-ATTESTATIONS`, binds each cited path to its exact tracked
git blob OID as of the commit a Librarian review last attested it; the
check (`scripts/gen-doc-status.sh`) compares the current tracked blob for
every cited path against that ledger, on exact population as well as exact
OID. That predicate proves the cited *bytes* match what was reviewed; it
does not, by itself, re-run `ken check` — the exit-code table above is the
separate, explicit record of that mechanism, current as of the SHA stated
there. A cited source's bytes changing requires both a fresh Librarian
ledger refresh (a deliberate act, never automatic) and, separately,
re-running the table above — `library/REVISION` is a provenance/bootstrap
anchor only and neither refreshes nor authorizes source bytes.
