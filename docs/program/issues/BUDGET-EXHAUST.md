---
id: BUDGET-EXHAUST
title: "transfer-budget bound checks are fail-open on variant extension"
status: merged
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

## ✅ MECHANISM RULED — `dec_7kcbc14ybndbq` RESOLVED (Architect, `evt_1q82xqdrqd2ma`)

**One exhaustive population classifier, two independent fail-closed consumers.**
Against the landed `origin/main @ 9ebebb8e` shape, in
`crates/ken-host/src/effect_v1.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferRequestBoundV1 {
    ReadAt(TransferCountV1),
    WriteAt(TransferCountV1),
}

impl CanonicalOutcomeV1 {
    pub fn transfer_request_bound(&self) -> Option<TransferRequestBoundV1>;
}
```

That method is the **sole population classifier.** Its body exhaustively
enumerates `CanonicalOutcomeV1`, `CanonicalReplyV1`, `ReadProgressV1`,
`WriteProgressV1`, and the current `SemanticErrorV1`, with **no `_`/catch-all at
any of those levels.** Every current non-budget outcome is named and maps to
`None`; `ReadSome → ReadAt(*transferred)`; `Wrote → WriteAt(*transferred)`.

★ **This is the whole point:** it converts *"remember whether a new reply
carries a budget"* from **an absence a reviewer must notice** into **an explicit
classifier arm the compiler demands.**

**Keep TWO validators, not one shared validator:**

1. `effect_wire.rs::validate_transfer_request_bound` exhaustively matches
   `Option<TransferRequestBoundV1>`, correlating `ReadAt` only with
   `CanonicalRequestV1::FsReadAt` and `WriteAt` only with `FsWriteAt`, enforcing
   `effective_request <= length`.
2. `ken-interp/eval.rs` gets **its own private validator** over the same
   classification and the same raw request, called **once at the entry to
   `reify_host_reply_v1`**, before the outcome is consumed. ⛔ **Only after that
   entry gate exists** may the duplicated per-`ReadSome`/`Wrote` outer-bound
   checks be removed.

**Both consumer matches have no wildcard.** A **request-side** `_ => Err`
remains correct — unlike the old outcome-side `_ => Ok`, every residual request
shape is uniformly *rejected*, so extension stays fail-closed. **The classifier
is shared because population is one fact; the validators stay independent
because wire admission and interpreter reification are two boundaries.**
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

### The Architect's staged proof (binding — supersedes the sketch above)

**Acceptance is the adversary-corrected conjunction, proved in two stages.**

**Stage 0 — remove the false-positive source first.** Mechanically fill the
unrelated encoder/decoder/ABI arms for a scratch
`ProbeProgress(TransferCountV1)`, so their *existing* exhaustive matches cannot
manufacture a failure that looks like the one under test. ★ Without this, every
later step could go red for the wrong reason and still read as success.

**Stage 1 — each layer must fail SPECIFICALLY, in order:**

| planted state | targeted check | must fail specifically at |
|---|---|---|
| probe absent from `transfer_request_bound` | `ken-host` | **the classifier** |
| probe mapped to a new `TransferRequestBoundV1::Probe`, both consumers unchanged | `ken-host` | **the wire validator** |
| scratch **wire** arm filled only | `ken-interp` | **the interpreter validator** |
| interpreter arm filled | both | an inflated-effective probe **rejects in both** layer-specific tests |
| planted extension reverted | — | **zero residue** |

**Retain the current `ReadSome` and `Wrote` above-raw negatives** — this is
additive to the landed coverage, not a replacement for it.

⛔ **Do not fold this WP into the native BUDGET-EFF parity half** (Architect,
restating the constraint independently).
- ⛔ **Do not fold this into BUDGET-EFF's deferred native half.** That half is a
  parity change on a different file set (`cranelift_backend/lowering/`); this is
  a structural-completeness change in `ken-host` + `ken-interp`. Bundling them
  would put two mechanisms behind one flip.

## ✅ MERGED — `origin/main @ e7b2a8a5` (PR #870, CI-gated, 11/11 checks green)

Verified by content across all three paths, not by the publisher's exit code:
`effect_v1.rs`, `effect_wire.rs`, `eval.rs` all empty vs `5d485dcf`;
`TransferRequestBoundV1` + `transfer_request_bound()` present at
`effect_v1.rs:2131/:2137`; `validate_transfer_request_bound` at `eval.rs:4865`,
called as the first statement of `reify_host_reply_v1` at `:4899`; the old
`_ => Ok(())` residual arm is **gone** from the wire validator.

Siblings intact across the window: facade still 492 lines, `api.rs`
byte-identical to `b9c23a6b`, DOC-W1-3's library files present.

⚠ **Instrument note.** The merge-base intersection test **self-fires** when run
against **post-merge** `main` — post-merge `main` contains the candidate's own
squash commit, so the candidate's touched-file set is necessarily a subset of
what main gained. It reported all three files as intersecting; re-run against
pre-merge `b9c23a6b` it was empty. **The intersection test is a pre-merge
question about an unmerged candidate** and has no meaning once the candidate is
an ancestor of `main`. Post-landing, use content-emptiness on the candidate's
own paths plus **sibling survival by content**.
