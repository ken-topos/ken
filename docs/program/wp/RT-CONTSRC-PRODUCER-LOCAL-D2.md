# `RT-CONTSRC-PRODUCER-LOCAL` `D2` — both binding kinds, populated

**What landed:** the `ProducerLocal` arm `D1` represented is now **constructed**
by the planner, for both binding kinds, with a derived contract. ⛔ Nothing is
admitted: every candidate whose required environment names one still declines,
so no edge's verdict moves.

## The two kinds, and where each is minted

Both are minted at the site that **introduces an environment entry**, which is
also the only site that can supply a locator.

| kind | minted at | `binding_origin` | `binding_ordinal` | `locator.environment_origin` | `locator.environment_index` |
|---|---|---|---|---|---|
| host-effect result | the `Let` whose bound value is an `Effect` | the `Effect` occurrence | `0` | the `Let`'s **body** | `0` |
| constructor **argument** binder | the `Match` / `ComputationalMatch` case descent | the **case body** occurrence | the binder's index | the same case body | the binder's index |
| recursive **IH** binder | ⛔ not minted — stays `Open`, see the correction below | — | — | — | — |

**The two origins differ for the effect result and coincide for the case
binder, and that is the separation `D1` exists to express.** An `Effect`
*creates* the value; the `Let` body is the scope that *holds* it. A case binder
is introduced by, and held in, one scope.

⭐ **Why the effect result is minted at the `Let` and not in the `Effect` arm.**
A locator names a position in an environment. An effect result consumed without
a binder never enters an environment, so there is no position to name — and it
correctly stays `Open`. Minting in the `Effect` arm would have to invent a
locator for a value that has no home.

⛔ **The case binder's identity is the case body plus an ordinal, never the
match occurrence plus an encoded pair.** Packing a case index and a binder index
into one `u32` is the aliasing this plane refuses everywhere else, and it is
unnecessary: the case body is already the exact static identity of that binder's
scope, so two binders in different cases of one match differ by origin and two
in the same case differ by ordinal.

## The contract, fact by fact

One function derives both kinds. Only the carrier and the referent lifetime
differ per kind; ownership and storage owner are read off `AbiCarrier`'s
existing methods — the same two the entry plane's `abi::slot` reads — so this
record **cannot disagree with the entry plane** about what a carrier implies.

| fact | host-effect result | constructor argument binder |
|---|---|---|
| carrier | `abi::result_carrier` on the `Effect` shape | `abi::result_carrier` on the **scrutinee's** shape |
| ownership | `AbiCarrier::ownership` | `AbiCarrier::ownership` |
| storage owner | `AbiCarrier::storage_owner` | `AbiCarrier::storage_owner` |
| referent affinity | the `Effect` occurrence's own lifetime authority | the scrutinee child's lifetime authority |

⭐ **The argument binder's carrier is read, not chosen.** A constructor argument
binder *preserves the scrutinee's representation* — that is the existing
result-phase rule, stated at both the `Match` and `ComputationalMatch` arms of
`summarize_result_phase` — so the binder's carrier is the carrier the
scrutinee's result travels in, and `abi::result_carrier` is the sole authority
for that. The carrier is then put through `slot_referent_affinity`, the same
admissibility gate an entry slot passes: a continuation source environment
admits `ValueWord` and `GroundValueCarrier` and refuses every convention
carrier, so a scrutinee whose result travels in one **fails closed** rather than
being silently narrowed.

⚠ **The first version of this deliverable (`a5a6ce9b`) asserted a blanket
`ValueWord` for "a case binder" instead.** That was a `D2` invention, and the
Architect blocked it at `evt_9krmbv834z9p`. It is replaced by the reading
above.

⭐ **The binder's referent lifetime is not conservatively floored, because it
does not have to be.** `PlannedReferentLifetime::Persistent` is issued only when
the complete source result is closed over persistent children, so a field of a
persistent scrutinee is persistent *by that type's own definition*. Reading the
scrutinee's lifetime is a read, not a promotion — and the type documents that it
has no promotion operation.

The derivation is measured to be per-binding rather than stamped: on the control
fixture the case binder's scrutinee is persistent and the effect result is
activation-owned, so the two affinities **differ**. A single hardcoded affinity
— the easiest wrong implementation — reds that assertion.

## The correction: a case run is not homogeneous, and the IH half is a hard stop

**The defect in `a5a6ce9b`.** A `ComputationalMatch` case environment is

```
[ recursive IH binders, constructor argument binders, outer environment ]
```

and the two runs are **not** contracted alike. `derive_occurrence_lifetime`
gives every IH `ActivationOwned` and every argument binder the *scrutinee's*
lifetime; `summarize_result_phase` gives IHs a declared-unit-result contract
(`carrier()` or `SPECIALIZED`, on `functionized_units`) and argument binders the
scrutinee's representation. `a5a6ce9b` looped over the combined count and
stamped one contract across both, silently misclassifying the IH prefix.

**The correction.** The loop now splits at `recursive_positions.len()`. The
ordinal still spans the whole run, so the identity stays `(case body, binder
ordinal)` with no new tag and no new enum arm. Arguments get the scrutinee-read
contract above.

**The IH prefix stays `Open` — no contract is claimed for it.** Three
independent reasons, each measured in the current source rather than argued:

1. **No `ResultPhase` → `AbiCarrier` map exists anywhere.** The two vocabularies
   are disjoint by construction: `ResultPhase` records a *representation phase*
   and `AbiCarrier` records an *ABI transport*. `slot_referent_affinity` — the
   authority for which carriers a continuation source environment admits —
   accepts `ValueWord` and `GroundValueCarrier` and refuses the rest, and
   nothing proves an IH is either of those two.
2. **The IH's phase is not edge-local.** It is `carrier()` or `SPECIALIZED`
   depending on `functionized_units`, which is a whole-plan argument to
   `plan_static_transition_graph_with_symbols` and is **not a field of
   `StaticTransitionPlan`**. So `(case body, ordinal)` does not determine the IH
   contract, and the walk cannot reach the fact that would.
3. **An IH is a callable, and the continuation-input vocabulary for a callable
   is `#[cfg(test)]`-only.** `BoundaryUseAvail::Callable` and
   `BoundaryUseNeed::PreserveCallableIdentity` exist solely as test mutations;
   every production projection is `Value` / `PreserveValue`. Representing an IH
   as a continuation source would need those promoted to production, which is an
   additional boundary-use authority.

⛔ Leaving it `Open` is **not** choosing a default carrier — it is declining to
represent, which is the pre-`D2` behaviour and preserves every edge's verdict.
Picking any of `ValueWord`, `ResultWord`, or a new variant is what
`evt_9krmbv834z9p` forbids.

### The discriminator

`contsrc_d2_a_computational_case_run_separates_its_ih_prefix_from_its_arguments`
uses a case with **one recursive position, one ordinary argument binder and a
persistent scrutinee**, and walks to an inner `ComputationalMatch` so it lands
on that case's **own** binder run.

⚠ The `a5a6ce9b` positive could not have caught this: it targeted an inner
`ComputationalMatch` but inspected that occurrence's *incoming* environment, so
it observed an outer ordinary-`Match` binder and a host-effect result and never
an IH at all.

It asserts position 0 is exactly `Open` and position 1 is exactly the
producer-local argument binder with the scrutinee's affinity — so **either
stamping reds it**, which is the property the Architect asked for:

| mutation | result |
|---|---|
| stamp the argument contract across both subruns (the `a5a6ce9b` loop) | **red** — position 0 becomes `Closed` |
| stamp the IH treatment across both subruns | **red** — position 1 becomes `Open` |

Both were run. Only this row reds, which is what makes it a discriminator rather
than a smoke test.

⛔ The row also asserts the scrutinee's lifetime **is** `Persistent`. An
activation-owned scrutinee would give the argument binder the same affinity an
IH's activation-owned treatment produces, and the comparison would then hold for
the wrong reason.

The `a5a6ce9b` outer-`Match`/effect fixture is retained unchanged for the
host-effect result, the locator/binding split, and the admission mutations.

## Represented, not admitted

`exact_continuation_source_environment` declines any candidate whose exact
inputs name a producer-local coordinate.

**This preserves the pre-`D2` population exactly.** Every position that is a
producer-local binding today was `Open` before `D2`, and `Open` already declined
at the take-loop immediately above. The decline moved from *opaque* to a *named
domain*; the outcome did not move at all. That is why the `D0` per-row parity
record and both lib baselines are unchanged.

⛔ **It is not an edge selector.** It tests the coordinate domain, is uniform
over every edge, and consults no corpus, closure identity, first-`Open` reason
or planned-member status — the four forbidden substitutes.

⛔ **It must sit before the validator, not after.** The refusal `D1` installed in
`validate_continuation_source_slot` is a planner **`Err`**, which rejects the
enclosing source program. Declining the *candidate* is the correct boundary, and
it is the one the take-loop already uses for `Open`.

**Promise class: transition sentinel.** `D4` releases admission and deletes this
block; that is the event that retires it.

## The gate is measured, and the first version of this evidence was not

`contsrc_d2_a_producer_local_environment_declines_the_candidate_not_the_program`
is proved to discriminate: with the gate replaced by `if false`, it fails, and
the failure is the `D1` validator refusal — which is the exact evidence that the
gate sits before the validator.

⚠ **Recorded because it nearly shipped as a vacuous row.** The first version of
this fixture put `unit()` in the case body. That case body then required *no*
surrounding environment, so `required_input_count` was zero, `exact_inputs` was
empty, and **no producer-local coordinate ever reached the gate**. Both rows were
green, and stayed green with the gate disabled. The fixture now uses `Var(3)` —
past the two computational binders, onto the enclosing `Match` binder — which is
what makes the required environment non-empty. The `Var` index is load-bearing
and is commented as such at the fixture.

## Baselines

Measured on this commit against the `D0` record at `179af863`, per row:

- `ken-cli` `rt_parity_native`: unchanged — `buffer_freeze` green, six red with
  byte-identical texts, including the five-`ComputationalMatch` / one-`Match`
  split.
- `ken-runtime` lib: the `D1` checkpoint's 721 passed / 2 failed, plus the two
  `D2` controls, with the same two standing reds.
- `ken-elaborator` lib: 108 passed / 0 failed.
- The walk change alone, measured before the two controls were written, was
  already 721 / 2 — so the behaviour-preservation claim is independent of the
  tests that assert it.
