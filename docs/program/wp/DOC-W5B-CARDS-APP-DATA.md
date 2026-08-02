# WP frame — `DOC-W5B-CARDS-APP-DATA` (Wave 5, slice 2 of 3)

Node: `docs/program/issues/DOC-W5B-CARDS-APP-DATA.md`. Program:
`docs/program/12-documentation-program.md` §Wave 5. Owner: doc ring.
Authority: the Wave 5 precondition report, merged `1f069b43`
(`dec_5a1ytj5jymfpz`); the card format established by slice 1.

Slice 1 built the card format and proved it on 6 packages. This slice **applies
it, unchanged**, to Application (3) and Data (11) — **14 cards**.

> ## THE FORMAT IS SETTLED. THIS SLICE APPLIES IT.
>
> Everything slice 1's frame settles binds here and is not re-derived: the
> closed disposition vocabulary `generated | authored | none-declared | held`,
> the four held classes with their citation, the rule that no row is blank, and
> the boundary that a card **reports** a missing convention and never creates
> one.
>
> ⇒ **If the format does not fit a package, that is a finding about the format,
> not licence to vary it.** Route it (hard stop 1). A card that quietly departs
> from the format makes the corpus inconsistent in exactly the way that is
> invisible until someone reads two cards side by side.

## Why these two areas, and why not Capability

Data (11) is the most law- and proof-dense area and Application (3) is the most
end-user-shaped, so together they stress the `law` and `effect/capability` rows
from opposite directions. Capability (19) is half the corpus and goes alone in
slice 3.

## Fixed inputs

Measured at `origin/main = a8df4b7b`.

| input | measured value |
|---|---|
| slice 1 | `DOC-W5A-CARD-FORMAT`, must be `merged` before this starts |
| the card format | slice 1 `D1`, as landed — **not** re-designed here |
| this slice's population | Application 3 + Data 11 = **14** |
| the subject index | slice 1 `D2`, already covering all 39; this slice adds no index rows |
| the held-class disclosure | slice 1 `D4`, linked rather than restated |

```sh
git rev-parse HEAD
git show origin/main:docs/program/issues/DOC-W5A-CARD-FORMAT.md | grep '^status:'
find catalog/packages/Application catalog/packages/Data -name '*.ken.md' | wc -l   # 14
```

## Deliverables

- **D1 — 14 complete cards**, Application and Data, every one of the nine rows
  populated or explicitly held, in slice 1's format.
- **D2 — the per-class disposition tally** across the 14: how many rows landed
  `generated`, `authored`, `none-declared`, `held`. The four held classes must
  total exactly 14 each.
- **D3 — the format-friction record.** Anything about these 14 packages the
  format handled awkwardly, stated as a finding for slice 3 to inherit. **Empty
  is a valid answer** and is worth more than an invented observation.

## Acceptance criteria

- **AC-1 — 14 cards, 126 labelled rows** (14 x 9), none blank.
  *Control:* the card set and a row count.
- **AC-2 — each of the four held classes is `held` on all 14 cards.**
  *Control:* grep the four rows per card; 14 of 14 each. A single card answering
  a held class fails the slice.
- **AC-3 — `none-declared` appears only where the canonical fences are actually
  empty for that class**, with the package named.
  *Control:* for each `none-declared` row, the fence it was read from. This is
  the distinction slice 1 exists to protect; it is also the one that decays
  silently under volume.
- **AC-4 — the format is byte-unchanged.** Slice 1's format document is not
  edited by this slice.
  *Control:* `git rev-parse <candidate>:<format path>` equals its blob on the
  merge base.
- **AC-5 — no card states a dependency, reverse-dependency, platform, or
  maturity fact.**
  *Control:* the held four, per card.

## Banned scope

- **No card outside Application and Data.** Capability is slice 3.
- **No edit to the card format** (`AC-4`). Friction routes as `D3`.
- **No generator, exporter, schema, or `crates/` change.**
- **No instantiation of the reserved facets and no convention proposal.**
- **No test asserting facts about source, catalog, or documentation lines**
  (operator test policy). `D2` and `D3` are review artifacts.
- **No normative claim**; name the spec section instead.

## Contention

`library/` and `docs/program/` only. No build lock, no `cargo`, no contention
with the runtime ring.

## Sizing

**Size `M`.** Fourteen cards against a settled format. The risk is not
difficulty but **uniformity decay** — `AC-3` is the row most likely to be filled
in from memory by card 10.

⇒ **Commit at these two checkpoints and post the exact SHA at each:**

1. Application (3) plus the first Data batch — enough to confirm the format
   holds outside the slice-1 proving set.
2. The remaining Data cards, `D2` tally, `D3` friction record.

If checkpoint 1 runs past an hour, stop and route.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **The format does not fit a package.** Report the package and the row; do not
   vary the format.
2. **A held class turns out to be answerable** for some package. Report it; do
   not answer it for one card while its siblings hold.
3. **A package's canonical fences cannot be distinguished from illustrative
   ones**, so `none-declared` cannot be established honestly. The report already
   flagged that text grep cannot make this distinction; if reading cannot either,
   that is a real finding.
4. **The disposition tally does not come out at 14 per held class** (`AC-2`).
