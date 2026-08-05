# `RT-UNIT-CLOSURE-CONVERT` `D1c` — the pass is not involved

Measurement only. **No production edit.** Base: exact **`a8b66c5c`**. Released by
`evt_64981706twkyr`. The probe was temporary, confined to
`crates/ken-elaborator/src/erasure.rs`, and is reverted.

## The answer: the question's premise does not hold

**`runtime_depth` is not zero at the three failing sites. The three failing units
never reach a `runtime_depth` site at all.**

Of the four dispositions the release names — depth miscomputed, `Var` wrong, both
locally correct, or the pass cannot adjudicate — the measured answer is **the
pass alone cannot adjudicate, because the pass is not on this path.**

## How that was established

All three elaborator construction sites (`erasure.rs:2132`, `:2428`, `:4329`)
were instrumented to report `runtime_depth`, `context_depth`, whether a branch
remap was present, and the per-index `runtime_index` mapping.

| corpus run | `runtime_depth` records emitted |
|---|---|
| **entire `ken-runtime` lib suite** — contains all five failing tests | **0** |
| **`ken-elaborator` lib suite** — 108 tests, all passing | **0** |

⛔ Zero records from the suite that contains the failures. The elaborator is not
on the path that produces them.

## What actually writes the empty list

The failing units' IR is **hand-built Rust test fixtures**:

```rust
// crates/ken-runtime/src/cranelift_backend/test_objects.rs:176 and :220
RuntimeExpr::LexicalClosure {
    captures: Vec::new(),
    params: vec!["response".to_string()],
    body: Box::new(recursive_body),
}
```

`captures: Vec::new()` is a **literal written by the fixture author**, not a
computed result. Across `ken-runtime` there are **92** such empty-capture
construction sites and a handful of non-empty ones (for example
`static_transition.rs:14882`, `captures: vec![leaf()]`).

⇒ For this corpus, the release's third reading is the operative one:
**source-syntax construction is the only population mechanism** — and the
"source syntax" is a Rust fixture, not elaborated Ken.

## ⚠ This corrects `D1b`

`D1b`'s hard stop named *"a corrected depth basis (upstream, different crate)"*
as one of the routes a repair would require. **That was an inference, not a
measurement:** I found the elaborator was the only *production* writer of
`captures` and concluded it was the producer of *these* units without checking.
It is not. `runtime_depth` is irrelevant to the five reds.

⭐ What survives `D1b` unchanged, because it is about the substrate rather than
about these units: `CaptureSlot { ordinal: u32 }` carries no identity; the
elaborator's membership basis is positional depth rather than a free-variable
set; and a `StaticWorker` occupies de Bruijn index 0 in exactly the failing
units.

## The state, stated without a preferred disposition

Measured across `D1`, `D1b` and `D1c`, and deliberately not resolved here:

- the five reds arise from fixtures that declare **no captures** while their
  bodies reference **one enclosing binding** (`index == env_len`, outermost
  position short by exactly one);
- the substrate has **no place to record** which binding that is, and the
  elaborator has **no analysis** that would compute one — both true, and neither
  is the cause of these five;
- the failing environments carry a **non-capture `StaticWorker`** at index 0,
  which is real, unique to them, and not the shortfall's cause.

⛔ **Whether the fixtures are malformed, or whether the substrate should support
a body referencing an enclosing binding the closure does not declare, is a
disposition question and is not answered here.** The release directs that each
result carries a distinct Steward/Architect disposition, so this checkpoint
reports the mechanism and stops.

⚠ **NOT MEASURED, and it bears on the disposition:** whether *elaborated* Ken
programs ever produce a closure whose body outruns its declared captures. The
three elaborator sites were not reached by either suite available here, so their
behaviour is **unmeasured**, not shown to be correct. Answering it needs an
end-to-end corpus (`ken-cli`), which this release does not authorize.

⛔ **Not claimed:** that the substrate is adequate, that the fixtures are wrong,
or that the `Var` indices are right. `D1c` establishes only which mechanism
produced the empty list, and that it is not the one `D1b` named.

## Suite

Unchanged at this base: **730 passed / 7 failed / 1 ignored**, both profiles
clean. `D1c` ends inventory. `D2`, candidate and downstream remain held.
