# RT-SRCBODY-BIND-ORDER — build the semantic body environment de Bruijn-nearest-first

**A functionized source-body unit records its ABI operands in descriptor order,
which is right, and then installs the same operands into the body environment in
that same order, which is wrong. A declaration body reads its parameters
de Bruijn-nearest-first. So every multi-parameter source body whose body
distinguishes parameter positions binds them permuted. This restores the
conversion the contract already states.**

**Owner:** Team Runtime. **Size:** M.
**Node:** `docs/program/issues/RT-SRCBODY-BIND-ORDER.md`.
**Risk:** medium. The change is small and the contract already exists; the risk
is in the **scope of what it silently corrects** and in two adjacent mechanisms
that must NOT be rewritten with it.

**Authority:** Architect mechanism ruling `evt_7yfs6qxp9hm5b`, on the
`RT-ENTRY-TRAP-254` `D0`-`D9` diagnosis chain.

---

## 1. Base and fixed inputs

**Cut from current `origin/main`**, after the `RT-CONTSRC-PRODUCER-LOCAL`
candidate merges. The publisher squashes, so do not continue
`wp/RT-DECL-CLOSURE-PORT-typed-units`.

**`wp/RT-DECL-CLOSURE-PORT-typed-units` is a FROZEN PUBLISH REF at `21fd46dc`
under an approved Decision. Nothing lands on it.** The prior diagnosis sits on
`wp/RT-ENTRY-TRAP-254-d6` at `c4112237` (a landed comment correction, `D6`).

**Treat every fixed input as perishable. If one turns out false against the
landed code, say so and escalate — do not quietly build around it.**

### The defect, and the contract it violates

| what | where |
|---|---|
| adapter builds the call in source argument order `[ProcessInput, ProgramCaps]` | `object_linker_packaging.rs:797-809` |
| `inputs = arguments in parameter order ++ captures in D3 order` | `core.rs:14898-14976` |
| ABI descriptor: parameter 0, parameter 1, then captures | `planning/static_transition/abi.rs:1551-1585` |
| **the contract: reverse source arguments, then append captures** | `core.rs:14705-14714` |
| **the violation: one slot-order walk fills both `defining_abi_operands` and `env`** | `lowering/units.rs:3701-3790` |

The observed instance: `main(input, caps)` installs `env = [input, caps]` while
the erased body names `input` as `Var(1)` and `caps` as `Var(0)`, so `Var(1)`
reads `ProgramCaps`.

## 2. Deliverables

### `D1` — split the two jobs the slot-order walk is doing

**Keep the physical ABI slot run and `defining_abi_operands` in descriptor
order — unchanged.** Separately construct the semantic body environment as:

```text
reverse(Parameter run) ++ Capture run in D3 order
```

**Two things this must NOT touch, both named by the Architect:**

- **Do not reverse the synthetic process root's two ABI roles.** Its adapter was
  deliberately authored as `Var(0) = ProcessInput`, `Var(1) = Capability`. **The
  correction belongs to source-body units — `CallableDeclaration` and
  `ClosureBody` — not the root `SchedulingEntry`.**
- **Do not rewrite continuation specializations.** Their planner-authored
  case-binder run is a different mechanism.

### `D2` — the generated-context claim, which this change would otherwise falsify

Generated contexts that execute a raw worker body currently claim
**byte-for-byte equivalence** with the raw unit **while also installing
parameter-then-capture order** (`units.rs:2523-2547`).

⇒ **They must use the same source-body binding conversion, or that equivalence
claim becomes false.** Fixing `D1` alone and leaving this makes a committed
claim untrue — which is worse than the original defect, because the claim is
what a reader would rely on.

### `D3` — the four controls

**Exactly these four. The Architect scoped them and the scope is a ban as much
as a list.**

1. **A two-parameter declaration with distinct NONAGGREGATE values that reads
   both positions.** This is the one that proves **the fix is not
   aggregate-shaped** — the whole campaign nearly concluded that aggregates were
   causal, and they are not.
2. **The `ProcessInput` / `ProgramCaps` discriminator** — the row inherited from
   `RT-ENTRY-TRAP-254`.
3. **A root-adapter control proving its fixed ABI-role order was NOT reversed.**
   Without this, `D1`'s exclusion is a promise rather than a mechanism.
4. **Raw-worker versus generated-context equivalence for a distinguishable
   two-parameter body** — the `D2` claim, made checkable.

**Each control asserts its exact expected values, not merely that it passes**, and
each must be seen RED before green. A control whose red was never observed is
not a control.

### `D4` — un-skip the inherited row

Remove the `#[ignore]` on
`public_source_observes_raw_argv_environment_cwd_bytes_in_field_order` **in the
same commit that greens it**. Report `passed / failed / ignored` as three
numbers: a suite still carrying the attribute has discharged nothing.

**Leave the other four `#[ignore]` attributes alone** — they belong to
[[RT-CARRIER-BYTESPAN-OBSERVE]].

## 3. Acceptance criteria

- **`AC-1` (`D1`) — `defining_abi_operands` is byte-identical to before.** The
  physical ABI run does not move. Show it.
- **`AC-2` (`D1`) — the body environment is `reverse(Parameter run) ++ Capture
  run`,** and the root `SchedulingEntry` is provably excluded (`D3` control 3).
- **`AC-3` (`D2`) — the raw-worker / generated-context equivalence claim is TRUE
  after the change**, demonstrated by `D3` control 4 on a body that
  distinguishes its parameters. **A body that cannot distinguish them proves
  nothing** — unary units are invariant under reversal.
- **`AC-4` (`D3`) — all four controls, each with an observed RED before green,
  and each asserting exact values.** Report which operand moved for each.
- **`AC-5` (`D4`) — the inherited row greens by un-ignoring**, three numbers
  reported.
- **`AC-6` — no regression, workspace green IN CI** — never a local
  `--workspace` run (`COORDINATION §12`).

  > **This AC is doing unusual work here and you should expect it to bite.**
  > The affected class is **every activated non-root functionized source-body
  > unit with at least two parameters whose body distinguishes parameter
  > positions**. Anything currently green *because* it was compensating for the
  > permutation will flip. **A test that goes red here is evidence the fix is
  > reaching its population, not evidence of a regression — but you must
  > attribute each one individually and say which.**

## 4. Banned scope

- **Per-argument transfer coordinates.** Ruled out: they are a
  provenance/ownership design change **and they would leave this defect
  intact**. A carried word bypasses `transfer_into_carrier` entirely, so a
  caller occurrence on the common coordinate cannot change which carried word
  occupies slot 0.
- **Any change to `carry_source_call_inputs`, `carry_call_input`,
  `call_declared_unit_target`, or the wrapper at `mod.rs:5958-5978`.** The
  positional pairing there is already correct; `D9` attributed the defect to
  that seam and the Architect refuted it.
- **Reversing the synthetic process root's ABI roles.**
- **Rewriting continuation specializations.**
- **Touching the carrier consumer, match lowering, ownership certificates, the
  common transfer coordinate, the other four ignored rows, or the frozen publish
  branch.** The Architect named this exclusion list; treat it as closed.

## 5. Hard stop

Stop and report, with the concrete unit, if:

- separating the two runs cannot be done without moving `defining_abi_operands`;
- the generated-context equivalence cannot be restored under `D2`, which would
  mean the claim was already false for a reason other than this defect; or
- `AC-6`'s CI run produces a red you cannot attribute to the permutation being
  corrected. **Do not absorb an unattributed red**, and do not re-baseline
  anything to make it pass.
