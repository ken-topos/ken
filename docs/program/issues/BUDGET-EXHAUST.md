---
id: BUDGET-EXHAUST
title: "transfer-budget bound checks are fail-open on variant extension"
status: ready
owner: verify
size: S
gate: none
depends_on: [BUDGET-EFF]
blocks: []
github: null
origin: evt_48r5yshq9p8hp
---

**Adversary finding at `origin/main @ 214bf4de`, raised against the merged
BUDGET-EFF host/interp half.** Latent, **not live** — nothing is exploitable on
`main` today, and the adversary explicitly ranked it below anything that
reproduces now.

## What is sound (attacked and held)

The adversary could not break the landed bound, and the negative result is worth
recording because it is what makes this finding *narrow* rather than alarming:

- `TransferCountV1` is **genuinely inseparable**, not nominally: both fields
  private, `pub(crate) fn new` enforcing `0 < transferred <= effective_request`,
  no setter / `Default` / `Deserialize` / `From`, read-only accessors. **Every**
  construction site in the crate goes through `new` — enumerated, including
  inside the defining module where private fields would permit a struct-literal
  bypass. So `remaining = effective_request - get()` **cannot** underflow,
  because no unvalidated pair can be constructed.
- **Shape rejection is complete over the live variant set.** `ReadSome` requires
  `FsReadAt`, `Wrote` requires `FsWriteAt`, each with
  `effective_request() <= length`; every other pairing returns `Err`. Exactly
  two positions carry `TransferCountV1` as a payload (`effect_v1.rs:2093`,
  `:2100`) and both are handled. **The crafted-inflated-budget trace is closed.**
- **Interp does not trust the wire check** — `eval.rs:4938`/`:4961` re-derive
  `requested` from the request and independently reject `effective > requested`.
  Real defense in depth, not a restatement.

## The finding — `crates/ken-host/src/effect_wire.rs:734`

```rust
_ => Ok(()),        // residual arm of validate_transfer_request_bound
```

**Add a third variant carrying a `TransferCountV1` and it decodes with no bound
on `effective_request` at all.** `new` still proves the intra-carrier invariant,
so nothing looks wrong at the construction site; the outer chain
`effective_request <= raw request.length` simply never runs. **That is exactly
the crafted-trace gap this function was written to close, reopened — with no
compile error and no failing test.**

`agent/COORDINATION.md §7` names this case: *where completeness is load-bearing,
an explicit arm per variant is the safeguard, not a catch-all.*

**The honest counterargument, which the adversary raised themselves:** §7 also
permits a catch-all where the residual is genuinely uniform, and it **is**
uniform today — outcomes with no budget to bound. **But that uniformity is a
property of the current variant set, not of the types.** The match is over
`CanonicalOutcomeV1`, whose variants are not in bijection with *"carries a
budget,"* so nothing holds the residual uniform as the enum grows.

## ★ Why this is worth a WP rather than a comment

**Neither layer would catch it.** Interp's check is *also* hand-written
per-variant, so a new carrier added without its own bound check passes there
too. **There is no layer where a new `TransferCountV1` carrier is forced to
declare its bound** — both are complete-by-inspection, and a reviewer would have
to notice an *absence* rather than see something break. Two independent
defenses that share the same premise are one defense
(the shared-premise trap — a differential check is blind to what both sides
assume).

## Scope

- **The mechanism is the Architect's call, not the reporter's.** The adversary
  named a candidate shape — matching exhaustively on the two small sealed
  progress enums instead of falling through on the outer outcome enum, which
  would make a new carrier a **compile error** — and explicitly declined to
  choose. Route the mechanism decision before implementation.
- Acceptance must include a **planted-extension proof**: add a third
  budget-carrying variant in a scratch tree and confirm it **fails closed**
  *before* the fix, and that the proof is what establishes completeness — not
  the assertion that the match is exhaustive.

  ⛔ **THE PLANTED VARIANT MUST FAIL CLOSED IN BOTH LAYERS — `effect_wire.rs`
  AND `eval.rs:4938`/`:4961`.** The finding is **the conjunction** ("neither
  layer would catch it"), not the wire catch-all alone. A proof scoped to the
  wire arm can go **green with interp unchanged and the finding half-open** —
  interp's per-variant checks are *also* hand-written and would still silently
  accept the planted carrier.

  **If the implementer or the Architect scopes interp out, that is allowed but
  must be explicit** — record the interp residue as a remaining open item
  rather than letting a one-layer proof close a two-layer finding.

  ★ *This correction is the adversary's* (`evt_9vn3rjc5qcvq`), *not mine.* My
  first phrasing named the mechanism (the wire match) and inherited exactly its
  blind spot — the same defect class this issue is about, committed in the
  acceptance criterion written to catch it.
- ⛔ **Do not fold this into BUDGET-EFF's deferred native half.** That half is a
  parity change on a different file set (`cranelift_backend/lowering/`); this is
  a structural-completeness change in `ken-host` + `ken-interp`. Bundling them
  would put two mechanisms behind one flip.
