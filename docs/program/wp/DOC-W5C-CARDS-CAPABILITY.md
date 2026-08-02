# WP frame — `DOC-W5C-CARDS-CAPABILITY` (Wave 5, slice 3 of 3)

Node: `docs/program/issues/DOC-W5C-CARDS-CAPABILITY.md`. Program:
`docs/program/12-documentation-program.md` §Wave 5. Owner: doc ring.
Authority: the Wave 5 precondition report, merged `1f069b43`
(`dec_5a1ytj5jymfpz`); the card format landed by slice 1 as `8d86e1fe`
(`dec_64hr5chqc7qwz`).

Slice 1 built the format and proved it on 6 packages. Slice 2 applied it to 14.
This slice applies it, unchanged, to **Capability (19)** and closes Wave 5's
per-package coverage at 6 + 14 + 19 = **39**.

> ## THE FORMAT IS SETTLED. THIS SLICE APPLIES IT AND CLOSES THE SET.
>
> Everything slices 1 and 2 settle binds here and is not re-derived: the closed
> disposition vocabulary `generated | authored | none-declared | held`, the four
> held classes with their citation, the rule that no row is blank, and the
> boundary that a card **reports** a missing convention and never creates one.
>
> ⇒ **If the format does not fit a package, that is a finding about the format,
> not licence to vary it.** Route it (hard stop 1). This is the last slice, so
> the temptation to absorb a mismatch quietly rather than route it is at its
> highest here — there is no later slice to inherit the finding.

## Why Capability is the hardest 19, and what it does to the held rows

Capability is half the corpus and it is the area whose packages most plausibly
*look* platform-specific. `Capability/Filesystem/`, `Capability/Network/`, and
their siblings read like operating-system surfaces, and the pull to write a
`platform` row from that prose is the strongest anywhere in the catalog.

**That pull is exactly what the held disposition exists to refuse.** The
precondition report measured the `platform` facet as reserved-but-uninstantiated
(`docs/program/06-catalog-campaign.md:119-121`), and a package reading like an
OS surface does not instantiate it. A `platform` row inferred from a package
name or from prose is precisely the "confidently wrong generated fact" the
program's §3 second risk names.

The same applies to `dependency`. Capability packages reference each other in
prose more than any other area, and the report already ruled that sparse prose
references cannot honestly mean "no dependencies" — nor can they be assembled
into a partial relation.

⇒ **All four held classes stay held across all 19.** The area's shape is not an
argument for answering them; it is the reason the rule is stated.

## Fixed inputs

Measured at `origin/main = ab6b89fc`.

| input | measured value |
|---|---|
| slice 1 | `DOC-W5A-CARD-FORMAT`, `merged` at `8d86e1fe` |
| slice 2 | `DOC-W5B-CARDS-APP-DATA`, must be `merged` before this starts |
| the card format | `library/reference/catalog/card-format.md`, as landed — **not** re-designed here |
| this slice's population | Capability **19** |
| the subject index | `library/reference/catalog/subjects.md`, already covering all 39; this slice adds no index rows |
| the held-class disclosure | slice 1's `D4`, linked rather than restated |
| slice 2's friction record | slice 2 `D3` — an **input** here, not a rediscovery |

```sh
git rev-parse HEAD
git show origin/main:docs/program/issues/DOC-W5B-CARDS-APP-DATA.md | grep '^status:'
find catalog/packages/Capability -name '*.ken.md' | wc -l   # 19
```

## Deliverables

- **D1 — 19 complete cards**, Capability, every one of the nine rows populated
  or explicitly held, in the landed format.
- **D2 — the per-class disposition tally** across the 19: how many rows landed
  `generated`, `authored`, `none-declared`, `held`. The four held classes must
  total exactly 19 each.
- **D3 — the 39-package close statement.** With this slice merged, every checked
  leaf package has a card. State the final per-class tally across all 39 and
  confirm the subject index still matches the card set exactly.
- **D4 — the format's verdict.** Slices 2 and 3 together applied the format to
  33 packages it was not designed against. State whether it held, citing slice
  2's `D3` and anything this slice found. **"It held, with nothing to report" is
  a valid and valuable answer** — do not manufacture friction to fill it.

## Acceptance criteria

- **AC-1 — 19 cards, 171 labelled rows** (19 x 9), none blank.
  *Control:* the card set and a row count.
- **AC-2 — each of the four held classes is `held` on all 19 cards.**
  *Control:* grep the four rows per card; 19 of 19 each. A single card answering
  a held class fails the slice. This is the AC the area's shape will press on.
- **AC-3 — no `platform` or `dependency` claim is inferred from a package name,
  a directory, or prose.**
  *Control:* the held four per card, plus a read of any card whose package name
  suggests an OS surface. This is `AC-2`'s failure mode stated in the form it
  will actually take here.
- **AC-4 — `none-declared` appears only where the canonical fences are actually
  empty for that class**, with the package and the fence named.
  *Control:* for each `none-declared` row, the fence it was read from.
- **AC-5 — the format is byte-unchanged.** `card-format.md` is not edited by this
  slice.
  *Control:* `git rev-parse <candidate>:library/reference/catalog/card-format.md`
  equals its blob on the merge base.
- **AC-6 — the subject index is byte-unchanged and still matches the card set.**
  39 index rows, 39 cards, same subjects.
  *Control:* the index blob against the merge base, plus a card-to-index
  reconciliation.

## Banned scope

- **No card outside Capability.** Slices 1 and 2 own the other 20.
- **No edit to the card format or the subject index** (`AC-5`, `AC-6`). Friction
  routes as `D4`.
- **No generator, exporter, schema, or `crates/` change.**
- **No instantiation of the reserved facets and no convention proposal**, however
  strongly a Capability package appears to have a platform.
- **No dependency or reverse-dependency index**, however partial.
- **No test asserting facts about source, catalog, or documentation lines**
  (operator test policy). `D2`, `D3`, and `D4` are review artifacts.
- **No normative claim**; name the spec section instead.

## Contention

`library/` and `docs/program/` only. No build lock, no `cargo`, no contention
with the runtime ring.

## Sizing

**Size `M`.** Nineteen cards against a format proved on 20. The work is
mechanical; the risk is uniformity decay across the longest single run in the
wave, and it concentrates on `AC-2`/`AC-3`.

⇒ **Commit at these three checkpoints and post the exact SHA at each:**

1. The first Capability batch — enough to confirm the format holds on the area's
   OS-shaped packages specifically.
2. The remaining cards.
3. `D2` tally, `D3` 39-package close, `D4` format verdict.

If any checkpoint runs past an hour, stop and route.

**Expect to end your turn at each checkpoint.** Post the SHA and wait for the
leader rather than assuming one turn spans all three.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **The format does not fit a package.** Report the package and the row; do not
   vary the format. There is no later slice to carry the finding.
2. **A held class turns out to be answerable** for some package. Report it; do
   not answer it for one card while its siblings hold.
3. **A package's canonical fences cannot be distinguished from illustrative
   ones**, so `none-declared` cannot be established honestly.
4. **The disposition tally does not come out at 19 per held class** (`AC-2`).
5. **The 39-package close does not reconcile** — the card set and the subject
   index disagree on any package.
