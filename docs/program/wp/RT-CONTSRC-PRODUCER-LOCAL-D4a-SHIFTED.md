# `RT-CONTSRC-PRODUCER-LOCAL` `D4a` — the shifted-fixture extension

Released by `evt_s2nbzddsag9k` (Runtime leader), on the Architect's confirming
gate `evt_65xkzqppdqdaj` and the Steward's scope ruling `evt_28xx7t69z7j76`.
Built over `D4a` proper at `52422da5`, itself over QA-approved `D3a`
`14b111ae`. Code at `97a4148b`.

This is the second bounded round of `D4a`, not a new node. `D3b`, `D4b`,
the candidate, QA of the node, `D6` closure, `AC-4`,
`#27`/case-emission, the call-result SCC and downstream `D7` all remain
held.

## What was missing, and why the previous round could not supply it

`D4a` proper admitted the census-bound `V` population and produced the first
producer-local availabilities ever to reach lowering. But its single distinct
reaching emission had

```
post_shift_index == locator.environment_index == 0
```

so a real post-shift walk and a plain pass-through of the locator's
introduction index name the same number. The checkpoint's whole purpose — to
supply `D3b` with a case where the two answers differ — was therefore
unmeasured rather than passed.

The first attempt at the extension hard-stopped: the one durable shifted
fixture, `contsrc_d2_both_binding_kinds_fixture`, is shifted *by* a `Let`-bound
`HostOpV1::ConsoleRead`, and `ConsoleRead` sits outside the compile-time
`CRANELIFT_HOST_EFFECT_CONSUMERS_V1`, so lowering refuses it before any
emission seam. Two seats verified that independently. The Steward then
authorized one new fixture, lifting exactly one prohibition — "do not add a
new population member" — and nothing else.

## The correction this round makes to its own predecessor's reading

The previous round reported a hypothesis: that the producer construct is a
`ComputationalMatch` **scrutinee**, evaluated outside its own match's binders,
and that no nesting it tried could shift the value. The first half is true
and is why the `ComputationalMatch`'s own binders do not count. **The
second half was an artefact of the instrument, not of the program.**

The probe recorded one line per emission seam. The seam carries a *vector* of
continuation inputs, and the shifted one is at **ordinal 1**. Re-running the
same fixture shape with a probe that walks every input reports:

| ordinal | binding | locator index | post-shift index |
|---|---|---|---|
| 0 | the enclosing `Match`'s case binder | 0 | 0 |
| 1 | the `Let`-bound host-effect result | 0 | **1** |

The shift was present the whole time. This is the
"first-cause-not-the-set" shape: a probe that reports one member of a vector
reports the first cause and reads as a property of the population.

## The fixture — and it supplies ONLY the population

`d4a_shifted_lowerable_fixture`, additive, in
`crates/ken-runtime/src/cranelift_backend/lowering/core/tests/control.rs`.
`contsrc_d2_both_binding_kinds_fixture` and its `D2b` discriminator are
**untouched**.

Three pieces, each load-bearing:

- a `Let`-bound `ConsoleWrite` result, so the value is a producer-local
  whose locator `environment_index` is `0`. The lane is the only change
  from `D2b`'s shape. `ConsoleIsTerminal` is in the consumer set and is
  **not** a candidate: it returns before seat synthesis and plans no seat
  at all.
- an enclosing `Match` case binder — the intervening binder. It moves the
  value to post-shift index `1` by the time it is emitted.
- the `Match` scrutinee's constructor argument is a **second** `ConsoleWrite`,
  so the locator slot holds a decoy of the same carrier, the same phase and the
  same lowering shape. Only the SSA word differs, which is what forces the
  oracle to be the SSA word rather than any incidental discriminator.

Being built to exhibit a shift and then observed to exhibit one proves nothing
about the derivation. That is the Steward's first constraint, and it is why the
discrimination below is carried by the mutations, not by the fixture.

## The oracle, and why it is independent of the planner

Lowering records the operand it builds at the **binder-creation seat**,
keyed by its own occurrence id, with no environment index in play. The
seam half reads by index. The two join on `binding_origin`, and a wrong
index breaks the join.

There is no planner re-walk, no index arithmetic, no fixture-authored expected
index and no direct construction anywhere in the row.

## The measurement, at the production planner and lowering path

Compiled through `recursive_port_process_compiles` — the same entry the other
controls in that file use, no alternate route.

```
post_shift_index = 1        locator.environment_index = 0
producer_env[1] = HostResult(v246, Ok, Err)   <- creation seat recorded v246
producer_env[0] = HostResult(v466, Ok, Err)   <- the decoy
```

MEASURED: exactly one reaching producer-local input has
`post_shift_index != locator.environment_index`; the operand the emitting
context's environment holds at that post-shift index is the **same
Cranelift SSA value** lowering recorded at the creation seat for that
binding's own occurrence; the operand at the locator index is a
**different** SSA value of identical carrier, phase, lowering shape and
constructor pair.

CLAIMED: a consumer indexing this environment with `post_shift_index` obtains
the producer-local value, and one indexing it with the introduction index does
not.

THE GAP: **no consumer indexes it yet.** The emission seam still refuses every
producer-local coordinate, and this round did not touch that refusal. `D4a`
measures the operands such a consumer would read. This is exactly the
boundary the Architect drew: the `D4a` mutation proves the
**instrument**; `D3b`'s own mutation must prove the **consumer**, against
this same fixture, and `D4a` passing does not discharge it.

## Mutation proofs

Two are **committed inside the control** and assert their own flip:

| mutation | effect |
|---|---|
| `UseLocatorIndex` — consume the locator's introduction index | post-shift slot yields `v466`, not the creation-seat `v246` |
| `SwapSlots` — exchange the two slots | post-shift slot yields `v466`; the locator slot yields `v246` |

`SwapSlots` is distinct from `UseLocatorIndex` rather than redundant with it:
both indices stay lawful and in bounds, so it survives a repair that merely
bounds-checks the index.

Three further mutations were run by hand against the committed tree and each
reverted, every one red at a distinct attributed line:

| mutation | red at | what it proves |
|---|---|---|
| drop the intervening binder (`binders: 1` → `0`) | the "exactly one shifted input" population guard | if the fixture ever stops being shifted the control reds loudly instead of passing on the degenerate case — the act-1 gap, closed |
| decoy reverted to `d4a_unit()` | the same-shape assertion | the decoy's carrier/phase/shape match is enforced, so an incidental representation mismatch cannot carry the row |
| creation record keyed on the `Let` instead of its value | the "exactly one operand for this binding origin" guard | the join key is load-bearing; a wrong key is caught, not silently tolerated |

## Census

The census instrument is a temporary probe and no census machinery is in the
tree, so one was rebuilt for this round, run, and reverted byte-clean. It walks
the **full** required vector before any decline, so a record is the whole
verdict set rather than the first cause.

Both runs are the same binary over the `ken-runtime` lib corpus,
single-threaded; the baseline is the identical run with only the new control
skipped, so the two differ in exactly one fixture.

| | without the fixture | with it |
|---|---|---|
| distinct edges (`C`) | 60 | 61 |
| admitted (`V`) | 57 | 58 |
| declined (`R`) | **3** | **3** |

The delta is exactly one row, and it is an **`ADMIT`**:

```
consumer=PredeclaredFunctionId(0) cont=StaticOriginId(6)
construct=StaticOriginId(15) req=2 verdict=ADMIT
vector=[CLOSED-local,CLOSED-local]
```

`R` is unchanged and its three rows are byte-identical across the two runs —
`[CLOSED-entry,CLOSED-entry,AMBIG2]`, `[CLOSED-entry,CLOSED-entry,OPEN]` and
`[OPEN,CLOSED-local]`, which are the three named causes `AMBIG2[let-value:If]`,
`OPEN[let-value:Construct]` and `OPEN[ih-binder]`. **The fixture adds only to
`V`.**

**These counts are not the frame's `C`=83 / `V`=80 / `R`=3 and must not be
read against them.** That census spanned two corpora — `ken-runtime` lib
plus `ken-cli`'s `rt_parity_native` — and counted `(identity, vector)`
instances under a finer fingerprint. This one is the `ken-runtime` lib
corpus under a coarser key. What is comparable, because both sides come
from the same instrument, is the **delta** and the **`R` invariance**;
those are what the release asked for. `D4b` still owns the full re-census
at the new base.

The `rt_parity_native` corpus cannot have moved: this fixture is a
`#[cfg(test)]` fixture inside the `ken-runtime` lib test binary and is not
reachable from `ken-cli` at all.

## Suite

`ken-runtime` lib: **725 passed / 7 failed / 1 ignored** — the `D4a` baseline's
724/7 plus this control, with the identical seven failures (the two standing
`D0` reds and `D4a`'s five deliberate coordinate-refusal reds). No regression.

Per `agent/COORDINATION.md §12` the workspace build, the `--locked` gate and the
conformance suite run in CI, not here.

## Scope held

No alternate lowerer, ABI or lane widening, selector, fallback,
permanent side map, direct construction, or `D3b` production arm. The
observatory is `#[cfg(test)]` throughout and **disarmed by default**, so
no other test records or pays for it and production compiles as if it did
not exist. It observes and returns nothing; the coordinate refusal below
it is untouched and still fires on every one of these inputs.
