---
id: RT-ENTRY-TRAP-254
title: "public_source_observes_raw_argv_environment_cwd_bytes_in_field_order exits 1 with an explicit entry trap where it expects 254 — branch-introduced, and the only tip failure that is not the byte-span gap"
status: closed
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSRC-PRODUCER-LOCAL]
blocks: []
superseded_by: RT-SRCBODY-BIND-ORDER
github: null
origin: Measured at candidate tip b914c7ff (evt_2h8wm2ff99ayq) and provenance-probed (evt_fxgentgrpw6g). Filed by the Steward because it was the one attributed tip failure with no owning node; an unowned failure is what gets lost. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## SUPERSEDED and CLOSED 2026-08-06 — the repair is [[RT-SRCBODY-BIND-ORDER]]
>
> **Architect mechanism ruling `evt_7yfs6qxp9hm5b`.** The `D0`-`D9` chain here
> did its job: it found a **general multi-parameter source-body binding
> permutation**, of which this node's single failing row is **one
> discriminator**.
>
> **The defect:** a functionized source-body unit records `defining_abi_operands`
> in ABI descriptor order (correct) and installs the same operands into the body
> environment in that same order (wrong) — a declaration body reads its
> parameters **de Bruijn-nearest-first**. `units.rs:3701-3790`. So
> `main(input, caps)` gives `env = [input, caps]` while the body names `input`
> as `Var(1)`; `Var(1)` reads `ProgramCaps`.
>
> **Closed rather than re-scoped** because this node's title names one test, and
> a general permutation kept under it would understate the scope to every future
> reader. Three of the four required controls are not about this row at all.
>
> **The skipped row moves to the successor** as control 2 of 4, and un-skipping
> it is that node's `D4`. Nothing here is owed.
>
> **`D9`'s own attribution was refuted** and that matters: it named the common
> transfer coordinate as the selector. `call_declared_unit_target` already does
> positional pairing, and a carried word bypasses `transfer_into_carrier`
> entirely — so **per-argument transfer coordinates are banned in the successor**;
> they would have been a design change that left the defect intact.
>
> **`D6` landed** (the stale carried-scrutinee reachability comment) and sits on
> `wp/RT-ENTRY-TRAP-254-d6` at `c4112237`, awaiting a follow-up PR.

> ## DIAGNOSIS DISCHARGED. Recut to instrument, localize, repair. `size: M`.
>
> **`docs/program/wp/RT-ENTRY-TRAP-254.md`, recut 2026-08-06** on the ring's
> return (`evt_29m0gnx2r43jw`) and the Architect's population ruling
> (`evt_m36y2zegby7m`).
>
> **The `-4` is attributed** — `lowering/mod.rs:16468`, `emit_current_trap`,
> `TrapExitAuthority::Root`, via `seal_source_trap_branch`. The borrowed-input
> hypothesis is **REFUTED**. The trap is reached **after** host observation was
> recorded (not after each field was decoded — the ring bounded that correctly).
>
> **The Architect ruled this is NOT an activation** of the source-machine
> carried-match mechanism: the path is a functionized declaration-unit call whose
> `Carried` scrutinee reaches the **generic** `lower_carried_match`, never
> `SourceContinuation::MatchScrutinee`. **The activation gate has NOT fired.**
>
> **What remains, and it reorders the work:** the root process-sentinel discards
> `identity.abi_word()`, so the run does not localize **which** nested ordinary
> match default fired. That discard is not a tidiness item — **it is the missing
> instrument**, and it is now the first deliverable.
>
> **Two things the frame settles that this node had left open.**
>
> **The exit `1` is not the defect and must not be investigated.** The linked
> shim ends `if (value < 0) return 1;`, so **every** negative sentinel collapses
> to exit 1 and the code cannot distinguish `-1` from `-4`. Only the stderr line
> can. The single fact that matters is that the entrypoint returned `-4`.
>
> **`254` IS the correct expectation — this node's second open obligation is
> discharged.** The test sets `K` to the byte `0xfe` under `env_clear()` and
> asserts exit `254`, with a second arm asserting `253` and an `assert_ne!`
> between them. Those are **legitimate non-negative exit codes**: the program
> observes a raw process byte and returns it, and `return (int)value` passes any
> non-negative value straight through. **254 is producible by this shim today.**
> The program is meant to compute and return a byte and traps instead. Do not
> re-open whether the expectation is stale, and do not "fix" the row by changing
> it — that is the cheapest available repair and the frame forbids it.
>
> **The row ships marked `#[ignore]`** under the operator's 2026-08-06 publish
> ruling. A skipped row measures nothing, so `D0` un-skips it first.

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
is a lowering refusal at a host-effect seat; this is a runtime **`-4`** sentinel
from a program that compiled and ran.

**CORRECTED: this paragraph said `-1` until 2026-08-06.** The `-1` attribution
was the Architect's working hypothesis and the ring **refuted** it — `-1` comes
from separate require/validation emitters and `emit_current_trap` emits it zero
times (`evt_29m0gnx2r43jw`).

## First obligations — SUPERSEDED by the frame, and one is discharged

This list was written before the frame. **Read the frame, not this** — kept
because obligation 2 was answered rather than dropped, and a reader who only saw
it disappear would reasonably wonder which way it went.

1. **Attribute the trap.** Live, and it is the frame's `D1`. Confirm or refute
   that `-4` shares the `-1` sentinel's borrowed-input-validation emitter
   (Architect, `evt_7v61ed5pn9q3t`), and report either way.

   **The half of this obligation asking "why the observed exit is `1`" is
   retired.** The shim collapses every negative sentinel to 1, so there is no
   `1`-specific question to answer.
2. **Decide whether `254` is still the correct expectation** — **DISCHARGED.**
   It is correct. `254` and `253` are legitimate non-negative exit codes the
   program returns from an observed process byte, and the shim passes any
   non-negative value through. **Do not re-open it**, and do not repair the row
   by changing the expectation.

   The premise this obligation rested on — *"a wrong expectation and a wrong
   runtime are indistinguishable from the exit code alone"* — was **true and
   not binding**: the test body distinguishes them, and nobody had read it.
3. **Size the repair only on the diagnosis return.** The frame stops at `D2` and
   hands back; the Steward re-cuts.

## Relationship to the publish decision — RESOLVED

The operator ruled on 2026-08-06: land the candidate with the five failing rows
marked `#[ignore]`, restoring them as work allows. **This node owns re-enabling
its row**, and un-skipping it is the frame's `D0` rather than a later
courtesy — a skipped row measures nothing, so nothing may be asserted while the
attribute is present.

The exact signature the Steward would have put in a residual gate is instead
recorded **in the source comment above the test**, which is where whoever
restores the row will actually read it.
