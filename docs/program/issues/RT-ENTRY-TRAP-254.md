---
id: RT-ENTRY-TRAP-254
title: "public_source_observes_raw_argv_environment_cwd_bytes_in_field_order exits 1 with an explicit entry trap where it expects 254 — branch-introduced, and the only tip failure that is not the byte-span gap"
status: ready
owner: runtime
size: TBD
gate: none
depends_on: [RT-CONTSRC-PRODUCER-LOCAL]
blocks: []
github: null
origin: Measured at candidate tip b914c7ff (evt_2h8wm2ff99ayq) and provenance-probed (evt_fxgentgrpw6g). Filed by the Steward because it was the one attributed tip failure with no owning node; an unowned failure is what gets lost. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## THE FRAME IS OWED. This node is `draft` and NOT startable.
>
> It is filed to give a measured, branch-introduced failure an owner. It has no
> fixed inputs at a named SHA, no acceptance criteria with controls, and no
> contention check. The Steward owes those before it flips `ready`.
>
> Size is `TBD` deliberately. Nothing measured so far bounds the repair, and a
> guessed size on this campaign has been wrong every time it was guessed.

## What is measured

At candidate tip `b914c7ff`, `px4b_native_production` is **14 passed / 5
failed**. Four failures are the byte-span per-seat gap owned by
`RT-CARRIER-BYTESPAN-OBSERVE`. **This is the fifth, and it is the only one that
is not.**

```
test:      public_source_observes_raw_argv_environment_cwd_bytes_in_field_order
observed:  ken native trap: explicit entry trap, exit Some(1)
expected:  exit Some(254)
```

It is a **runtime trap, not a lowering refusal** — a distinct failure class from
the four per-seat effect refusals.

## Provenance — branch-introduced, with no green/red boundary

| SHA | result |
|---|---|
| `e6b4a13b` merge base | GREEN |
| `3015aafd` main | GREEN |
| `b9189ee9` | GREEN |
| `c7410b79` | RED, but `ken native trap: malformed borrowed process input` |
| `b914c7ff` tip | RED, `ken native trap: explicit entry trap` |

**No last-green/first-red pair exists for this signature.** The test is red
**continuously** from `c7410b79`, so under skip-not-bad discipline every commit
carrying the older trap is a *skip* by construction and no green commit is
adjacent to the explicit-entry region.

The answerable question was **when the signature changed shape**:

| | SHA | subject |
|---|---|---|
| last `malformed borrowed process input` | `fb663bf3` | `D7: re-assert the two pins the aggregate lane narrowed` |
| first `explicit entry trap` | `9cea8a5e` | `D7 checkpoint (RED at this tip): aggregate ownership record, and the population gap it measures` |

Adjacent, verified `9cea8a5e^ == fb663bf3`, and **both endpoints re-probed
directly rather than inherited from the bisect log**.

`9cea8a5e` declares `(RED at this tip)` in its own subject. All three signature
transitions on this branch land on commits that announce themselves. That means
nothing was hidden; it does **not** mean the failure is benign.

## The inference that is BANNED here

**Do not conclude that this belongs to the byte-span family because the test
name contains "bytes."**

The test is `..._observes_raw_argv_environment_cwd_bytes_in_field_order`, and
the temptation to fold it into `RT-CARRIER-BYTESPAN-OBSERVE` on that basis is
exactly the vocabulary inference the Architect refuted on this campaign
(`evt_7v61ed5pn9q3t`) — where a signature was matched against a function that
did not exist at the commit in question, and the words agreed only because they
were generic.

**Measured position:** a byte-span observer **cannot** clear this trap. Byte-span
is a lowering refusal at a host-effect seat; this is a runtime `-1` sentinel.
Whether the two share a root cause is **unmeasured and must be measured, not
argued from the name.**

## First obligations, when this is framed

1. **Attribute the trap.** Which entry path emits `explicit entry trap`, and why
   the observed exit is `1` where the test expects `254`. The `-1` sentinel is
   rendered by `object_linker_packaging.rs` from borrowed-input validation
   paths (Architect, `evt_7v61ed5pn9q3t`) — confirm or refute that this is the
   same emitter.
2. **Decide whether `254` is still the correct expectation**, or whether the
   test encodes a contract the branch legitimately changed. A wrong expectation
   and a wrong runtime are indistinguishable from the exit code alone.
3. Only then size the repair.

## Relationship to the publish decision

If the operator's gate-readiness ruling lands the branch behind
exact-signature residual gates, **this node owns re-enabling its row**, and the
gate must assert this signature exactly so a *different* failure at that row
still reds.
