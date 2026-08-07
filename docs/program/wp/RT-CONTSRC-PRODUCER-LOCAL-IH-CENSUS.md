# `RT-CONTSRC-PRODUCER-LOCAL` — the IH-requirement census

Measured at exact `5377d2abce42acf6a5652474b7e846c53ca1da20` plus the bounded
fidelity correction that ships with this unit. The instrument was a temporary
probe, reverted before commit; no census machinery is in the tree.

## The unit, stated

**One candidate edge = one call to `exact_continuation_source_environment` from
the specialization-discovery loop**, identified by

```
program fingerprint + consumer owner + continuation origin
                    + producer construct origin + recursive position
                    + closure origin
```

⛔ The program fingerprint is not decoration. `StaticOriginId`s are allocated
**per compile**, so without it two edges in two different fixtures collide on
identity and the census silently undercounts. It is
`source occurrences / function units`.

⛔ **No edge is classified by its first declining or `Open` input.** Every edge's
**full required environment vector** is inspected — all `required_input_count`
positions, each with its verdict and the construct that supplies it. The label
walk that names the supplying construct is written **independently** of the
authority walk, so their agreement on length is a cross-check rather than a
restatement.

## Corpora and totals

| | `ken-runtime` lib | `ken-cli` `rt_parity_native` | both |
|---|---|---|---|
| raw records | 1090 | 60 | 1150 |
| distinct edge identities | 60 | 17 | 77 |
| `(identity, vector)` instances | 66 | 17 | 83 |
| torn / malformed lines | 0 | 0 | 0 |

Both suites were run **single-threaded** and both matched their baselines with
the probe installed — `ken-runtime` 724 passed / 2 failed, parity 1 passed /
6 failed — so the instrument does not perturb what it measures.

**Six identities carry two vectors each.** That is one edge planned under two
ABI configurations (different entry-input counts), not two edges. It is reported
rather than merged, and **every one of the six agrees with itself on
IH-requirement**, so the partition below is unaffected by how they are counted.
The instance count treats them as distinct, which over-counts rather than
under-counts.

## The partition

| | requires ≥1 recursive-IH binder | requires none |
|---|---|---|
| every required position closed | 0 | **78** |
| required environment is empty | 0 | 2 |
| at least one required position not closed | **1** | 2 |
| **total** | **1** | **82** |

**Exactly one edge in the whole measured population requires a recursive-IH
binder.**

```
prog=10/2 consumer=fn0 cont=origin10 construct=origin19 pos=0 closure=origin18
  required=2  0:OPEN[ih-binder]  1:local[case-arg]
```

It is a `ken-runtime` lib edge. It is the *only* place an IH contract would
change an admission verdict.

## Everything that is still not closed, exhaustively

Across all 83 instances there are exactly **three** non-closed required
positions, and each belongs to a different edge:

| supplying construct | verdict | edge |
|---|---|---|
| `ih-binder` | `OPEN` | the one IH edge above |
| `let-value:Construct` | `OPEN` | `prog=12/2 … required=3 … 2:OPEN[let-value:Construct]` |
| `let-value:If` | `AMBIG2` | `prog=15/2 … required=3 … 2:AMBIG2[let-value:If]` |

Neither of the latter two is producer-local-representable: a `Construct` result
is not a `Let`-bound host effect and not a case binder, and the `If` position
joins **two distinct exact sources**, which the walk deliberately refuses to
collapse. Neither is affected by an IH contract.

The two `empty` edges have `required_input_count == 0` — nothing is required, so
they are trivially closed and intern today.

## Every verdict, by supplying construct

All required positions across all instances:

| verdict | supplying construct | count |
|---|---|---|
| `entry` | `entry:0` … `entry:7` | 47 + 39 + 12 + 12 + 12 + 10 + 3 + 3 |
| `local` | `case-arg` | 43 |
| `local` | `effect-result` | 6 |
| `entry` | `let-value:Var` | 6 |
| `OPEN` | `ih-binder` | 1 |
| `OPEN` | `let-value:Construct` | 1 |
| `AMBIG2` | `let-value:If` | 1 |

## Beyond the prior census

The prior figures — **34 case-binder-only, 4 effect-result-plus-case-binder, 1
`Construct`-only** — were taken before `D2` populated anything, when every case
binder and every host-effect result was `Open`. They are **superseded, not
contradicted**: those same positions now read `local[case-arg]` and
`local[effect-result]`, which is why the closed count is 78 rather than a
handful.

Complete enumeration **did** find edges beyond that census: 83 instances against
39 previously-declining edges. The prior census counted only *declining* edges;
this one enumerates the **whole candidate population**, including the edges that
already interned.

The `Construct`-only edge from the prior census survives as the
`let-value:Construct` row above. The `AMBIG2[let-value:If]` edge is **new to
this census** — it was not in the 34/4/1 partition at all.

## The parity corpus, separately

**All 17 parity edges are all-closed and none requires an IH binder.**

That population is the one behind the six failing `D0` rows, and it includes
every closure the `1e` hard stop was about:

| closure | vector |
|---|---|
| `381` (the framed witness) | `required=2  0:local[effect-result]  1:local[case-arg]` |
| `767` | `required=2  0:local[effect-result]  1:local[case-arg]` |
| `855` | `required=2  0:local[effect-result]  1:local[case-arg]` |
| `1086` | `required=2  0:local[effect-result]  1:local[case-arg]` |

`1e` measured those four as `0:OPEN[Let-value:Effect] 1:OPEN[Match-case-binder]`
and stopped on them. **Both of those positions are now closed by `D2`'s two
binding kinds**, and neither is an IH.

⛔ This is a statement about the **environment**, not about the `D0` rows. The
rows are still red and their refusal texts are unchanged; closing an
environment is necessary for admission, and admission is `D4`.

## What this does and does not decide

**MEASURED:** one edge in 83 requires a recursive-IH binder; 80 are already
closed or empty; the remaining two are blocked by constructs an IH contract
would not touch.

**CLAIMED:** the IH contract is not on the path to admitting the parity
population, and is on the path for exactly one lib edge.

**THE GAP:** this census says nothing about whether that one edge *should* be
admitted, nor about what a callable contract would have to say. ⛔ Per the
release, the graph decision is not mine to make — this is the inventory it rests
on.
