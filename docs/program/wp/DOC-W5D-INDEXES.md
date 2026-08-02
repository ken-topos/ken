# WP frame — `DOC-W5D-INDEXES` (Wave 5 closeout)

Node: `docs/program/issues/DOC-W5D-INDEXES.md`. Program:
`docs/program/12-documentation-program.md` §Wave 5. Owner: doc ring.
Authority: the Wave 5 precondition report, merged `1f069b43`
(`dec_5a1ytj5jymfpz`); the card format landed by slice 1 as `8d86e1fe`
(`dec_64hr5chqc7qwz`).

Slices 1-3 deliver 39 cards. **Wave 5's stated Produces also names nine
indexes, and exactly one of them exists.** This WP closes the wave by building
the indexes that can be built and recording, in one place, why the rest cannot.

## What Wave 5 promised and what is landed

`12-documentation-program.md` §Wave 5 Produces: *"One generated reference page
or card per live package, plus subject, declaration/type, law,
effect/capability, assurance, platform, maturity, dependency, and
reverse-dependency indexes."*

Measured at `origin/main = 8c12f5d4` under `library/reference/catalog/`:

| index | state |
|---|---|
| subject | `subjects.md`, landed, covers all 39 |
| declaration/type, law, effect/capability, assurance | **absent** — this WP builds them |
| platform, maturity, dependency, reverse-dependency | **absent, and cannot be honestly built** — this WP records why |

⇒ The wave's exit property is *"the catalog is discoverable both by what a
reader wants to accomplish and by the exact checked abstractions available."*
Per-package cards give the first half. **Discovery by abstraction is a
cross-package question and no artifact answers it today.**

## The three judgments this frame makes, so you do not have to

### 1. An index is a projection OF THE CARDS, not a re-derivation

⛔ **Do not re-read the canonical fences to build an index.** The cards are the
landed, QA-approved reading of those fences. An index derived independently can
disagree with the card it points at, and then a reader who finds a package by
its law and opens its card sees two different answers.

⇒ **Every index row is projected from the corresponding card row**, and cites
the card. If a card row looks wrong, that is a finding to route (hard stop 1) —
not a thing to quietly correct in the index.

### 2. The four held classes get a RECORDED DISPOSITION, not four files

The precondition report measured `platform` and `maturity` as
reserved-but-uninstantiated, and `dependency`/`reverse-dependency` as lacking
any complete package-level projection
(`library/reference/catalog/card-format.md#held-class-disclosure`).

⛔ **Do not create a held-class index file, not even an empty or partial one.**
A `platform` index listing zero packages reads as "measured: none have a
platform," which is false — the facet is uninstantiated, which is a different
claim. A partial reverse-dependency index is worse: the card format already
forbids inverting incomplete data, and an index is exactly where a partial
inversion would look authoritative.

⇒ **`D4` records the disposition once**, pointing at the existing disclosure.
Four absent files with a stated reason is the honest close.

### 3. ⭐ AN EMPTY INDEX IS A VALID AND VALUABLE RESULT — do not pad it

Measured over the 14 landed Application and Data cards:

| class | dispositions |
|---|---|
| Declaration/type | 14 `authored` |
| Law | 11 `authored`, 3 `none-declared` |
| **Effect/capability** | **14 `none-declared`** |
| Assurance | 14 `authored` |

**Every landed card declares no effect or capability.** If Capability's 19 come
back the same way, the effect/capability index has zero entries across the whole
catalog.

⇒ That is a **finding about the catalog**, and stating it plainly is this WP's
most useful output. ⛔ **Do not manufacture entries, do not widen the class to
find something to list, and do not quietly drop the index because it came out
empty.** An index that says "39 of 39 packages declare no effect or capability,
here is the disposition and here is where that would change" is a real
reference page.

⚠ Note the shape: `Capability/` is an area **name**, not an effect declaration.
An empty effect index sitting next to 19 packages under `Capability/` will look
like a mistake. It is not, and `D2` should say so where a reader will hit it.

## Fixed inputs

Measured at `origin/main = 8c12f5d4`.

| input | measured value |
|---|---|
| slices 1-3 | `DOC-W5A-CARD-FORMAT` `merged`, `DOC-W5B-CARDS-APP-DATA` `merged`, `DOC-W5C-CARDS-CAPABILITY` must be `merged` before this starts |
| the card population | 39 — Application 3, Data 11, Capability 19 |
| the card format | `library/reference/catalog/card-format.md`, as landed — **not** edited here |
| the subject index | `library/reference/catalog/subjects.md` — the shape to follow, and not edited here |
| the held-class disclosure | `card-format.md#held-class-disclosure` — linked, never restated |
| slice 3's `D2`/`D3`/`D4` | inputs here: the tally, the 39-package close, the format verdict |

```sh
git rev-parse HEAD
git show origin/main:docs/program/issues/DOC-W5C-CARDS-CAPABILITY.md | grep '^status:'
find library/reference/catalog -name '*.md' -path '*/*/*' | wc -l   # expect 39
```

## Deliverables

- **D1 — the four buildable indexes**, one page each, under
  `library/reference/catalog/`: declaration/type, law, effect/capability,
  assurance. Every entry projected from a card row and citing that card.
- **D2 — each index states its own population and dispositions** at the top: how
  many of the 39 contribute an entry, how many are `none-declared`, and what a
  reader should conclude from that. This is where the empty-index explanation
  lives if effect/capability comes out at zero.
- **D3 — the reconciliation.** Every one of the 39 cards is accounted for in
  each of the four indexes, as either an entry or an explicit `none-declared`.
  No card is silently absent from any index.
- **D4 — the held-class disposition record.** One statement covering all four
  held indexes: why each is absent, who owns the missing mechanism, and what
  would have to become true for it to exist. Links the existing disclosure
  rather than restating it. **No held-class index file is created.**
- **D5 — the Wave 5 close statement.** With this merged, Wave 5's Produces is
  discharged: 39 cards, the subject index, four built indexes, four recorded as
  unbuildable. State whether the wave's exit property is met and name anything
  that remains.

## Acceptance criteria

- **AC-1 — four index files exist and no fifth.** Exactly declaration/type, law,
  effect/capability, assurance.
  *Control:* the directory listing. ⛔ **A `platform.md`, `maturity.md`,
  `dependency.md`, or `reverse-dependency.md` file fails this AC outright**,
  empty or not.
- **AC-2 — every index entry cites the card it was projected from**, and its
  disposition matches that card's row for the same class.
  *Control:* per index, a sample reconciled entry-by-entry against its card. A
  single disposition disagreeing with its card fails the seam — that is the
  drift this WP's first judgment exists to prevent.
- **AC-3 — all 39 cards appear in all four indexes**, as an entry or an explicit
  `none-declared`.
  *Control:* the card set against each index's package set, both directions.
- **AC-4 — no index infers a fact the cards do not carry.** Nothing derived from
  a package name, a directory, or prose.
  *Control:* a read of any entry whose content is not traceable to a card row.
- **AC-5 — an empty or near-empty index is stated as a measured result**, with
  its population and the reason, and is not deleted, padded, or widened.
  *Control:* `D2`'s header on each index. This is the positive control for
  judgment 3 — **the failure it catches is an index quietly dropped for coming
  out empty.**
- **AC-6 — `card-format.md` and `subjects.md` are byte-unchanged.**
  *Control:* both blobs against the merge base.
- **AC-7 — no card is edited.** This WP reads the 39 and writes indexes.
  *Control:* the candidate's diff touches no path under an area directory.

## Banned scope

- **No card edits.** A wrong card row is hard stop 1, not an edit here.
- **No edit to the card format, the subject index, or the held-class
  disclosure** (`AC-6`).
- **No held-class index**, however partial, empty, or clearly labelled
  (`AC-1`).
- **No new metadata convention, facet, schema, or generator.** This WP projects
  what the cards already say.
- **No generator, exporter, or `crates/` change.**
- **No test asserting facts about source, catalog, or documentation lines**
  (operator test policy). Every deliverable here is a review artifact.
- **No normative claim**; name the spec section instead.

## Contention

`library/` and `docs/program/` only. No build lock, no `cargo`, no contention
with the runtime ring.

## Sizing

**Size `M`.** 39 cards times 4 classes is 156 projected rows, but the source is
landed and uniform, so the work is aggregation rather than judgment. The risk is
not volume — it is judgment 3, where an index that comes out empty invites
either padding or deletion.

⇒ **Commit at these three checkpoints and post the exact SHA at each:**

1. The declaration/type and law indexes, with their `D2` headers.
2. The effect/capability and assurance indexes. **If effect/capability is empty,
   stop and post that fact with the checkpoint** — it is the wave's most
   interesting measurement and the leader should see it before `D5` is written.
3. `D3` reconciliation, `D4` held-class record, `D5` Wave 5 close.

**Expect to end your turn at each checkpoint.** Post the SHA and wait for the
leader rather than assuming one turn spans all three.

If any checkpoint runs past an hour, stop and route.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **A card row looks wrong** while projecting it. Report the card and the row.
   ⛔ Do not correct it in the index — that creates the disagreement this frame
   is built to prevent.
2. **A class cannot be indexed** from the card rows as they stand, because the
   rows are not comparable across packages. That is a finding about the card
   format and it belongs to slice 1's owner, not to this WP.
3. **A held class turns out to be answerable** for some package. Report it; do
   not create the index.
4. **The 39-package reconciliation fails** — a card missing from an index, or an
   index naming a package with no card.
5. **An index would need a fact no card carries.** Do not go to the fences for
   it; that is judgment 1 and the answer is to route.
