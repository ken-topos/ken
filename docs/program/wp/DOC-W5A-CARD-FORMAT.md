# WP frame — `DOC-W5A-CARD-FORMAT` (Wave 5, slice 1 of 3)

Node: `docs/program/issues/DOC-W5A-CARD-FORMAT.md`. Program:
`docs/program/12-documentation-program.md` §Wave 5. Owner: doc ring
(`doc-leader` + `doc-author`, Librarian as QA).
Authority: the Wave 5 precondition report `docs/program/wave-5-format-capability.md`,
merged at `1f069b43` on Decision `dec_5a1ytj5jymfpz`.

Wave 5 produces one reference card per live catalog package, plus the fact-class
indexes. This slice establishes **the card format** and proves it on the two
smallest areas. Slices 2 and 3 apply it to the remaining 33 packages.

## The fork this slice implements, and what it forecloses

The capability report's `D2` recommended the **mixed fork**, and that is what
this slice builds. Of Wave 5's nine promised fact classes, the report measured:

| disposition | classes | count |
|---|---|---:|
| `generated` | subject | 1 |
| `authored` | declaration/type, law, effect/capability, assurance | 4 |
| `held` | platform, maturity, dependency, reverse-dependency | 4 |

⇒ **Five classes are populated. Four are held, and held is a stated result, not
an omission.** The two catalog facets (platform, maturity) are reserved by
`docs/program/06-catalog-campaign.md:119-121` but uninstantiated; the two
relations (dependency, reverse-dependency) have no package-level projection.

> ### THE HAZARD THIS WHOLE SLICE IS SHAPED AROUND — "none" has two meanings
>
> A card that reads **"Effects: none"** is making one of two entirely different
> claims:
>
> 1. **The package declares no effects** — read off canonical fences, a fact.
> 2. **We could not determine whether it has effects** — an absence of
>    measurement, which looks identical on the page.
>
> The report already caught this in its own sample: `Capability/Filesystem/`
> `Authority.ken.md` has no law declaration, and the report was careful to say
> that "none declared" there is **human-read, not emitted**. That distinction
> survives into the card or the card is worse than no card.
>
> ⇒ **Every card row carries its disposition label**, and the vocabulary has a
> third value beside `generated` and `authored`: **`none-declared`** (we looked
> at the canonical fences and there are none) versus **`held`** (nobody can
> answer this yet). ⛔ A row may never be blank, and it may never render a held
> class as though it were an answered one.
>
> This is the program's own §3 second risk, carried in verbatim as instructed:
> **a generated corpus can be confidently wrong, and where a generated fact
> matters it needs an anchor the generator does not produce.**

## Fixed inputs

Measured at `origin/main = a8df4b7b`.

| input | measured value |
|---|---|
| the population | **39** checked leaf packages under `catalog/packages/`: Application 3, Capability 19, Core 5, Data 11, Tooling 1 |
| this slice's proving set | **Core (5) + Tooling (1) = 6 cards** |
| the capability report | `docs/program/wave-5-format-capability.md`, merged `1f069b43` |
| the one extractable class | H1 subject, via `git grep -n '^# ' -- 'catalog/packages/**/*.ken.md'` — 39 path-preserving titles |
| output root | `library/reference/` (the program's §Wave 5 "Produces" line) |
| the reserved-facet citation | `docs/program/06-catalog-campaign.md:119-121` |

Reproduce, read-only:

```sh
git rev-parse HEAD
find catalog/packages -name '*.ken.md' | wc -l          # must print 39
git grep -n '^# ' -- 'catalog/packages/**/*.ken.md' | wc -l
```

## Deliverables

- **D1 — the card format**, written once as a specimen plus a short authoring
  rule. Nine rows, one per fact class, each carrying a disposition label from
  the closed vocabulary `generated | authored | none-declared | held`. **The
  four held classes appear on every card** with the reason and the owner who
  would unblock them, never omitted.
- **D2 — the generated subject index**, all 39 packages, with the exact command
  that produced it recorded beside it so a reader can replay it. This is the
  only class Wave 5 may present as generated.
- **D3 — six complete cards**: `Core` (5) and `Tooling` (1), every one of the
  nine rows populated or explicitly held.
- **D4 — the held-class disclosure**, written once and linked from every card:
  what platform, maturity, dependency, and reverse-dependency would require,
  who owns each, and the citation. **It reports the gap; it does not propose a
  convention** — that boundary is what the precondition report was blocked on
  and it binds here too.
- **D5 — the authored-rot statement.** The four authored classes go stale when
  the source changes. State, per class, what change stales it. The report
  already measured these; carry them rather than re-deriving.

## Acceptance criteria

- **AC-1 — every row on every card carries a disposition label** from the closed
  vocabulary, and no row is blank.
  *Control:* read the six cards; count rows. 6 cards x 9 classes = 54 labelled
  rows, no exceptions.
- **AC-2 — `none-declared` and `held` are never interchanged.** A class in the
  held four may not appear as `none-declared` on any card, and a genuinely empty
  canonical fence set may not appear as `held`.
  *Control:* the held four are a fixed list; grep each card for those four rows
  and confirm all six say `held`. This is the slice's central claim.
- **AC-3 — `D2`'s subject index replays.** The recorded command, run at the
  candidate, reproduces the index exactly.
  *Control:* run it and diff against the committed index.
- **AC-4 — the subject index covers 39 packages**, not the 6 this slice cards.
  *Control:* line count.
- **AC-5 — `D4` proposes nothing.** No new catalog convention, facet value, or
  schema is introduced; the reserved facets are cited, not instantiated.
  *Control:* read `D4` against the precondition report's own boundary.
- **AC-6 — no card states a fact the report classed as unextractable.** In
  particular no card carries a dependency list or a platform claim.
  *Control:* the held four, per card.

## Banned scope

- **No card for any package outside Core and Tooling.** Those are slices 2 and 3.
  The subject *index* is the one deliverable that spans all 39.
- **No generator, exporter, schema, or `crates/` change.** Four classes are
  blocked on projections that do not exist; building one is a separate program
  item owned by `crates/`, not a documentation slice.
- **No instantiation of the reserved `platform` or `maturity` facets**, and no
  proposal of a convention for them. Report the gap.
- **No dependency or reverse-dependency index**, however partial. The report
  measured prose references in 15 of 39 leaves and ruled that sparse prose
  cannot honestly mean "no dependencies."
- **No test asserting facts about source, catalog, or documentation lines**
  (operator test policy). The controls above are review artifacts and replayable
  commands, not CI gates.
- **No normative claim.** No `library/` page is normative; where a reader needs
  the rule, the card names the spec section rather than restating it.

## Contention

The doc track runs concurrently with build work and touches `library/` and
`docs/program/` only — no `crates/`, so no build lock and no contention with the
runtime ring. **No `cargo` invocation is needed for this slice at all.**

## Sizing

**Size `M`.** Six cards plus a format. The format is the real work and the six
cards exist to prove it survives contact with three different package shapes —
Core is proof-heavy, Tooling is a single package, and both are small enough that
a format defect shows up as rework measured in minutes rather than in 39 files.

⇒ **Commit at these two checkpoints and post the exact SHA at each:**

1. `D1` format specimen plus `D2` subject index — before any card is written.
2. `D3` six cards, `D4` disclosure, `D5` rot statement.

If checkpoint 1 runs past an hour, stop and route; the format is the Steward's
to re-cut.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **A Core or Tooling package cannot be carried through all nine rows** — some
   class has no honest disposition. That is a gap in the report's own
   classification and it belongs to the Librarian.
2. **The format needs a fact the checked source does not carry.** Interface
   fact; it routes rather than being approximated in prose.
3. **A held class turns out to be answerable** for some package. Do not answer
   it for that one package — a card that answers what its siblings hold is worse
   than one that holds uniformly. Report it; the disposition may be wrong.
4. **The subject index does not replay**, or does not yield 39 rows. The one
   generated class is the one that must be mechanically true.
5. **Writing `D4` requires naming a convention that does not exist.** That is
   exactly the hard stop the precondition report hit; the answer was that a
   report may report a gap and may not create one.
