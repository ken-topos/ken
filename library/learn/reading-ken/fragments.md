# Reading-Ken fragment set

The `learn/reading-ken/` chapters (DOC-W1, slices 2-5) all teach from this
one wave-wide fragment set, selected here so a later change is a visible
edit to this file — with a readable blast radius — rather than scattered
rework nobody connects (DOC-W1 slice 1, per doc-leader kickoff).

**Selection rule (`docs/program/issues/DOC-W1.md` §3):** real, checked
`catalog/packages/` fragments spanning enough domain variety to avoid the
"unrelated snippets" trap, without building around the catalog's one whole
program (`catalog/examples/CommandLine/Forge.ken.md` — pure spec-data, no
effects) and without substituting an invented toy.

## What "still checks" means, and the mechanism used

A pure-library catalog entry (no `proc main`) is validated with
`ken check <file>`: it runs the same elaboration and literate fence-role
checking `ken run` does, then stops before the IO-drive step
(`docs/program/07-catalog-style-guide.md` §3). **Every row below reports
that mechanism's actual exit code, not an assertion that the file "should"
still check.** Run at `origin/main @ cf91ec5a4ee2b557540f6a894cb9d8825638a634`
(the SHA this wave was released on), from the repository root, against the
`target/debug/ken` binary built from that same revision:

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
   grounds the `proved` reading in chapters 02-03 directly from a real
   entry rather than from prose about one.
2. **`Core/Logic/Transport.ken.md`** — `subst`/`cong`/`cast`/`sym`/`trans`,
   the equality-reasoning vocabulary later proofs in the corpus build on.
   Chapter 02 (types, contracts, proofs).
3. **`Data/Sums/Combinators.ken.md`** — the `Option`/`Result`/`Either`
   combinator floor, each combinator paired with a real proof of its
   defining equation. Ordinary data plus proofs in one small entry —
   chapter 01 (anatomy) and chapter 02.
4. **`Capability/Console/Text.ken.md`** — four small `IO`-effectful helpers
   over the Console capability, `visits [Console]` on every signature.
   Chapter 04 (effects, capabilities, authority).
5. **`Capability/Filesystem/Errors.ken.md`** — capability-authority
   (`AFull`) plus an explicit, honest security-boundary limitation stated
   in its own prose ("the current authority check is coarse and is *not*
   path-confined"). Useful for chapter 04 and for chapter 03's `unknown`/
   `delegated` reading, since the entry states its own boundary rather than
   overclaiming.
6. **`Capability/System/IO.ken.md`** — proof terms over buffer I/O whose own
   text states plainly that "exactly-once settlement and liveness remain
   runtime-enforced, delegated boundary properties." A real entry that
   names its own `delegated` claims in prose — chapter 03 (assurance and
   trust) and chapter 06 (execution, runtime assumptions).
7. **`Tooling/Testing/Property.ken.md`** — a deterministic finite property
   runner over explicit samples, whose own Motivation section draws the
   `tested`-vs-`proved` line explicitly ("Properties here are computations,
   not propositions. They test the executable shadow of a contract without
   assuming or proving that contract."). Chapter 03 and the checked
   exercises (slice 5).

## A domain gap, reported rather than built around

The candidate domain list carried into this slice's kickoff named
Core/Classes and Data/Numeric as intended coverage alongside Core/Logic,
Data/Sums, Capability, and Tooling. **Neither has a fragment that passes
`ken check` standalone today.** Every file tried in both — `Core/Classes/
LawfulFunctors.ken.md`, `Core/Classes/LawfulClasses.ken.md`, `Core/Classes/
EffectfulClasses.ken.md`, `Data/Numeric/Nat/Arithmetic.ken.md`, `Data/
Numeric/Nat/Order.ken.md` — fails with an `UnresolvedCon` (or, for `Order`,
a `KernelRejected`) naming a name defined in a *different* package file
(`cong`, `sym`, `list_append`, `Functor`, `Semigroup`, `OrdResult`). This is
not those entries being broken; it is the documented current limitation
that cross-file `import` resolution has no disk loader yet — "a catalog
entry that needs another package's helper today still inlines it… not
imports it" (`docs/program/07-catalog-style-guide.md` §13). The entries in
those two domains were written assuming (or awaiting) that resolution and
are not currently self-check-able in isolation.

The kickoff's domain list was offered as "candidate domains… not a fixed
list," so this is not a fixed input turning out false — but it is reported
here rather than silently substituted, per the framing traps
(`DOC-W1.md` §6: "re-verify at pickup; if a fixed input turns out false
against the landed corpus, say so"). The substitution made instead —
`Core/Logic` for `Core/Classes`, `Data/Sums` for `Data/Numeric` — still
delivers proofs-over-data and typeclass-adjacent material (Transport's
`cong`/`sym`/`trans` are exactly the combinators the failing files needed
and couldn't reach), so the wave's domain-variety goal is met without a
whole-program crutch or an invented toy. If a later slice specifically
needs a Classes- or Numeric-shaped fragment, that is a scope question for
doc-leader, not something to improvise here.

## Currency of this artifact

This file's own manifest record cites all seven fragment paths above as its
`sources`. Its currency claim rests on the **content-currency** predicate
(`source-currency` in `manifest.toml`): each cited path's tracked git blob
is byte-identical between `library/REVISION` and `HEAD`
(`scripts/gen-doc-status.sh`, landed by `DOC-CURRENCY-ANCHOR`). That
predicate proves the cited *bytes* are unchanged; it does not, by itself,
re-run `ken check` — the exit-code table above is the separate, explicit
record of that mechanism, current as of the SHA stated there. A later
`REVISION` bump re-verifies byte-identity automatically; it does not
re-verify `ken check` automatically, so any slice that bumps `REVISION`
past this file's citations should re-run the table above rather than
assume it still holds.
