# RT-CARRIED-ORDINARY-COMPOSITION — `D0`/`D1` checkpoint

**Base: `origin/main` = `06e031de964ee26153f7f4811e37d217033e0e28`**, which
carries both halves of the closed predecessor. Cut from the merged object, per
the frame's standing rule.

**This candidate changes no code.** `git diff 06e031de -- crates/` is empty at
the checkpoint SHA. Every instrument was temporary, was run, and was reverted,
and all coordinates below were re-derived by name **after** the revert.

> # THE THREE FINDINGS
>
> **1. Two of the three guard cells are EMPTY, measured independently.**
> `retained_scrutinee_index` and `deferred_constructor_case` were `false` on
> **every one of the 14 arrivals** across both runs — not merely unobserved
> behind an earlier guard.
>
> **2. Only the trailing-suffix cell has members**, and every one is
> `suffix_len = 1`, `suffix_kinds = Active`.
>
> **3. Every firing member's suffix came from the `active.pending`-rebuilt
> successor. ZERO came from the explicit outer tail.** That is `AC-3`,
> discharged by measurement rather than by the refusal string, which cannot
> distinguish them.

## 1. `D0` — the population across the whole guard family

### 1.1 Why the census measures all three predicates per arrival

**The guards are ordered, so only the first one reached is observable.** A
first-refusal-only census therefore cannot separate *"this guard has no
members"* from *"this guard was never reached"* — the two look identical from
outside.

So the instrument sits at the **top of the `Carried x Ordinary` arm, before any
guard can return**, and records **all three predicates for every arrival**, plus
the suffix's exact length, kinds and provenance. That makes the two zeros below
measurements rather than absences of evidence, and it yields the intersections
directly.

**Provenance is measured, not inferred.** `resume_active_continuation` sets a
flag immediately before the single call that composes against a successor frame
rebuilt from `active.pending`; the composed consumer takes-and-clears it at
entry. Taking it means the attribution is exact for the direct call and never
leaks to a nested descendant.

### 1.2 Denominators

| quantity | retained | A-only exclusion |
|---|---:|---:|
| compilations | 619 | 615 |
| arrivals at the `Carried x Ordinary` arm | 6 | 8 |
| distinct tests compiling | 270 | — |
| distinct tests reaching the arm | 6 | — |
| **tests that compile but never reach the arm** | **264** | — |
| malformed census records | 0 | 0 |

### 1.3 Every arrival, all three guards independently — `AC-1`

| retained idx | deferred case | suffix len | provenance | retained run | A-only exclusion |
|---|---|---:|---|---:|---:|
| false | false | 0 | outer | 5 | 5 |
| false | false | 1 | `resume_built` | 1 | 3 |
| **true** | any | any | any | **0** | **0** |
| any | **true** | any | any | **0** | **0** |

**`retained_scrutinee_index` and `deferred_constructor_case` have zero members**
across all 14 arrivals in both runs. Because every arrival was recorded before
any guard could fire, this distinguishes *no members* from *never reached*: the
arm **was** reached, 6 and 8 times respectively, and both predicates were false
every time.

**No intersections exist.** No arrival satisfies more than one guard, so the
ordering never hid a second cell.

### 1.4 The firing members, with suffix detail — `AC-3`

| test | run | suffix len | suffix kinds | provenance |
|---|---|---:|---|---|
| `d8d_the_composed_binding_site_...` | A-only exclusion | 1 | `Active` | **`resume_built`** |
| `px8j_all_three_producer_paths_reach_real_consumers` | A-only exclusion | 1 | `Active` | **`resume_built`** |
| `ccr_d3_the_active_carried_route_is_taken_...` | both | 1 | `Active` | **`resume_built`** |

**Zero members from the explicit outer tail.** The outer-tail guard added by the
predecessor's `D2` still has no witness, exactly as it had none when it was
written.

**The one retained-run firing member is the predecessor's own `D3` control**,
which arms the committed one-variant hook itself. ⇒ In a genuinely unhooked
production run **no guard in this family fires at all**: all 5 remaining
arrivals carry no retained index, no deferred case and an empty suffix, and they
proceed into the delegation.

## 2. `D1` — the partition

### 2.1 The two empty cells

`retained_scrutinee_index` and `deferred_constructor_case` are **measured-at-base
zeros**. There is no member to repair and no evidence about what either would
need. Building a mechanism for either would be a proof over an empty population
— Campaign Trap 3, where every control passes because there is nothing to
quantify over.

**This is a fact about this tree, not a property of the design.** Both guards
stay fail-closed, for the same reason `PendingLet` did at the sibling node.

### 2.2 The trailing-suffix cell

Exact first refusal, identical on both population members:

```
Unsupported(BoundaryCarrier, "a carried producer-call scrutinee reached an
ordinary eliminator with further composed eliminators behind it; the carried
elimination consumes exactly one frame, so the remainder would be silently
dropped")
```

- **Retained run stays green** at **817 / 0 / 4**.
- **Activation denominator:** the arm is reached 8 times under exclusion, 3 of
  them firing this cell, so no refusal is credited to an unreached path.
- **Positive control:** the 5 arrivals with an empty suffix pass **through** the
  guard family into the delegation in the same run — same arm, same phase, and
  the discriminator is the suffix alone.

**Owned fact: the presence of a composed suffix behind a one-frame elimination.**
Nothing about the carrier, the scrutinee or the cases is implicated; the 5
passing arrivals differ from the 3 firing ones only in `eliminators[1..]`.

### 2.3 The candidate for `D2`, and why hard stop 3 does not fire on it

`lower_carried_match` returns a `Result<LoweringOperand, _>` — **it hands back a
value.** So the suffix can be continued by composing that returned value against
`eliminators[1..]`, re-entering the composed consumer exactly as the
specialized path already does, **without `lower_carried_match` expressing
anything beyond cases / default / origin / env.**

⇒ On this evidence the interface does **not** need widening, so the frame's
third hard stop does not fire. **This is a candidate, not a `D2` decision.**
Whether the returned value actually survives that composition — and whether the
refusal advances to a fifth authority — is `D2`'s to measure. The chain's record
on this is that a route being expressible is not evidence it works: that was
measured, not assumed, at each of the last two nodes.

## 3. Hard stops — none fire

1. **Do the three guards need distinct mechanisms?** Not answerable as posed and
   not needed: **two cells have no members.** One cell fires, so there is one
   mechanism to consider, and the question of partitioning three authorities
   does not arise.
2. **Does any repair need a new planner or ABI population?** Nothing in the
   measurement implicates one.
3. **Does continuing the suffix require widening `lower_carried_match`?** Not on
   this evidence — it already returns the value the suffix would consume.
4. **Has the refusal advanced to a fifth authority?** Not yet; only `D2` can
   answer that.

## 4. Bounds, stated rather than implied

- **The census covers the `ken-runtime` lib suite**, which is `#[cfg(test)]`.
  `ken-cli`'s `rt_parity_native` and `px8f_buffer_native`, and `ken-verify`'s
  `px8f_write_partition`, are **not** covered, and those compile real Ken
  programs.
- **Both real population members are hand-built `RuntimeExpr` values** — Campaign
  Trap 1. This proves the arm is reached with this shape and says nothing about
  whether a real Ken program produces it.
- **The two empty cells are empty over this corpus at this base.** That is what
  makes them safe to leave alone; it is not a claim that they are unreachable.
- **The third firing member is the chain's own `D3` control**, not an
  independent program. The independent population is two.
- **CI has not run.** `AC-9` is a CI claim; no local `--workspace` run, per
  `COORDINATION §12`.

---

# `D2` outcome — the suffix is continued, and the refusal advanced a FIFTH time

## What was built

The trailing-suffix cell only. `lower_carried_match`'s returned `LoweringOperand`
is composed against `eliminators[1..]` and re-entered into the same consumer.
**Its interface is untouched** — still exactly cases / default / origin / env —
so hard stop 3 did not fire.

The two empty cells are unchanged and stay fail-closed as measured-at-base zeros.

## Termination is a property of the code, not of an argument

The lexicographic measure is `(active.pending.len(), eliminators.len())`: a
composed re-entry leaves every pending suffix untouched and consumes one
eliminator, and a resume splits `active.pending` into a head and a **strictly
shorter** tail. Both components are non-increasing and one strictly decreases at
every step.

**That argument is stated and deliberately not relied on.** Every measured member
has `suffix_len = 1`, so depth two is unexercised, and a termination argument
whose only witness is the argument is not evidence. A **bounded re-entry depth
fails closed** past the limit. It is expected never to bind; if a real program
ever reaches it, the refusal is the signal to measure that shape and raise the
bound deliberately.

## What was measured

**The fourth wall is gone from both rows** — the trailing-suffix refusal does not
appear anywhere in the excluded run. Both rows then fail **identically** at a
fifth authority:

```
Unsupported(ComputationalMatch, "scrutinee is not a constructor value after
ordinary expression lowering")
```

**Different construct and different owned fact.** The previous four walls were
`BoundaryCarrier` refusals about how a carried operand may cross or be consumed.
This one is a `ComputationalMatch` refusal about the **value shape a scrutinee
has after ordinary lowering** — the specialized path's `Lowered::Constructor`
destructure inside the same consumer.

**It is already a controlled shape elsewhere.** An existing
`RT-RECURSOR-TRANSPORT`-era control pins this exact rendered refusal, in full
equality, as the replay of a suppressed guard. So the successor inherits a
message that is **load-bearing in a committed assertion** — a repair that changes
or removes it will red that control, and that is a feature rather than an
obstacle.

## Status

- Retained suite green at **817 / 0 / 4**; `diff --check` clean; production build
  warning-neutral at 50.
- **`RT-MATCH-RECURSOR-CONSUMERS AC-1` is not discharged and no row closes.** The
  node stops here under the frame's fourth hard stop, armed in advance.
- `AC-5` landed guards intact; `AC-6` zero added `#[ignore]`; `AC-7` both
  variants, both insertions and the hook unchanged; `AC-8` `issues/` untouched.
- `D3` remains subsequent work. Its counters are present and `#[cfg(test)]`-only,
  and per the Steward it must **exclude itself** from population evidence: a
  control that is a member of the population it observes proves the hook is
  reachable, not the shape.
