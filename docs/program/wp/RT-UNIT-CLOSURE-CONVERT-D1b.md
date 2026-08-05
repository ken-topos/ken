# `RT-UNIT-CLOSURE-CONVERT` `D1b` — the deciding mechanism

A corrected hard stop.

Measurement only. **No production edit.** Base: exact **`bc754c03`** on
`wp/RT-DECL-CLOSURE-PORT-typed-units`. Released by `evt_7426hqs7pzjvc`. All
probes were temporary and are reverted.

## ⚠ This corrects `D1`'s framing on the point it was challenged

`D1` reported *"a free variable with no declared capture slot — absent, not
inert"*. That reading **assumed the `Var` was valid and the captures were
wrong**, and I had not measured the alternative. The release asked me to resolve
that, and the measurement changes the account. The hard stop stands, but for a
better-grounded reason.

## What writes `captures`, and the membership basis

**The membership basis is positional depth, not free variables.** Three
production construction sites in the elaborator, all identical in shape:

```rust
// erasure.rs:2132, :2428, :4329
RuntimeExpr::LexicalClosure {
    captures: (0..runtime_depth).map(|index| RuntimeExpr::Var(index as u32)).collect(),
    params: vec!["arg0".to_string()],
    body: Box::new(body),
}
```

with

```rust
// erasure.rs:4128
fn runtime_depth(&self, source_depth: usize) -> usize {
    (0..source_depth).filter_map(|index| self.runtime_index(index)).max()
        .map_or(0, |index| index + 1)
}
```

⛔ **Nothing inspects the body.** The list is the enclosing runtime environment
materialised by position — "capture everything in scope" — and it is empty
**exactly when `runtime_depth` evaluates to 0** at the construction site. The
only other writer, `erasure.rs:4568`, *rewrites* an existing list under
`shift_runtime_vars`; it does not populate one.

### Which of the three readings holds

**Producer computes free variables and has a gap — NO.** There is no
free-variable computation anywhere to have a gap in.

**Producer copies an input whose source is missing — YES.** It copies
`runtime_depth`, an ambient count derived from the branch remap / context depth,
and that count is what comes back `0`.

**Source-syntax construction is the only population mechanism — PARTLY.** It is
the only *production* population, but it is positional-from-depth rather than
from syntax structure.

⇒ The deciding mechanism is **`runtime_depth == 0` at the closure's construction
site**, not an analysis that ran and missed something.

## The non-capture environment contribution — CONFIRMED

Every `Var` resolution in a closure-body unit was instrumented across the lib
corpus, successes included. 23 distinct rows:

| header captures | resolves | fails |
|---|---|---|
| `captures: 0` | 10 | **2** |
| `captures: 1` | 4 | 0 |
| `captures: 2` | 7 | 0 |

The environments:

```
unit=fn1 index=3 env_len=3 ok=false  env=[worker | carried(v10) | carried(v11)]
unit=fn3 index=1 env_len=2 ok=true   env=[worker | carried(v10)]
unit=fn3 index=2 env_len=2 ok=false  env=[worker | carried(v10)]
```

⭐ **Yes — there is a non-capture contribution, and it is real.** A
`LoweringEnvironmentBinding::StaticWorker` occupies **de Bruijn index 0** in
exactly these units and **nowhere else** in the closure-body population; every
other environment is values only (`carried`, `specialized-*`). The declared slot
run is `Parameter, Result, Control, Trap, Store` — so the environment is **not**
the frame's parameter/capture prefix, and `D1`'s implicit assumption that it was
is wrong.

⛔ **But the worker is not the cause of the shortfall, and this is the
discriminating fact.** Both failures are `index == env_len` — off the end by
**exactly one** — and the missing position is the **outermost** (index 0 is
innermost; the environment prepends). Removing the worker would make the
environment *shorter*, not longer, and would turn a shortfall of one into two.
The same units resolve every index from `0` to `env_len - 1` correctly.

⇒ The bodies reference **exactly one enclosing-scope value** that the environment
does not hold. That is precisely the position a capture would supply, and the
closure declares `captures: 0` because `runtime_depth` was 0.

## ⛔⛔ THE HARD STOP — restated on the measured mechanism

Supplying the missing outermost binding requires knowing **which value it is**.
Nothing at this base can answer that:

- the elaborator never computed it — the basis is a count, not a set;
- the count itself is the defect, and it is produced in `ken-elaborator`, a
  different crate and one plane above this node;
- `CaptureSlot { ordinal: u32 }` has nowhere to record the answer even once
  known, and the release forbids adding an identity field.

⛔ Every locally available route is one of the four banned repairs. **Shifting
`Var`s** would hide the off-by-one the worker's occupancy makes tempting;
**copying the caller tail** would supply the binding without authority;
**fabricating a capture** would invent a slot the descriptor does not declare;
**padding** would enlarge the frame around the gap.

⚠ **`D1c` CORRECTS THE ROUTE NAMED BELOW.** The "corrected depth basis
(upstream, different crate)" clause was an inference: the elaborator is the only
*production* writer of `captures`, and I concluded it produced *these* units
without checking. It does not — the five failing units are hand-built fixtures
(`test_objects.rs:176`, `:220`) whose `captures: Vec::new()` is a literal, and
the elaborator's three construction sites emit **zero** records across the suite
that contains the failures. `runtime_depth` is irrelevant to them. See
`RT-UNIT-CLOSURE-CONVERT-D1c.md`. Everything above about the substrate stands.

### The concrete missing route

**`runtime_depth` returns 0 at the construction site for these closures, while
their bodies reference one enclosing binding.** Whether that is a defect in
`runtime_depth`, in the branch remap it consults, or in the body's index basis is
an elaborator question — measurable there, not here.

⇒ Either a free-variable analysis (**new analysis**) or a corrected depth basis
(**upstream, different crate**) is required, and recording the result needs an
identity-bearing representation (**substrate expansion**). All three are outside
this node: the Steward owns sizing, the Architect owns substrate expansion.

## What is measured and what is not

**MEASURED**: the membership basis and all four writers; that no free-variable
analysis exists; the full closure-body `Var` population with successes; that the
`StaticWorker` contribution is real, unique to the failing units, and *not* the
shortfall's cause; that both failures are short by exactly one outermost binding.

**NOT MEASURED**: why `runtime_depth` is 0 for these three construction sites and
nonzero elsewhere. That requires instrumenting `ken-elaborator`, which this
release does not authorize and which `D1`'s hard stop already routes to sizing.

⚠ **NOT CLAIMED**: that the `Var` index is correct. It may be that the body's
index basis is wrong and the environment is right. This checkpoint establishes
that the two disagree by exactly one outermost position and that the worker
occupies a real, non-capture slot — it does **not** adjudicate which side is
authoritative, and no repair should assume the answer.

## Suite

Unchanged at this base: **730 passed / 7 failed / 1 ignored**, both profiles
clean. No repair attempted; `D2`, candidate and downstream remain held.
