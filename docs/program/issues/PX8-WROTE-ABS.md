---
id: PX8-WROTE-ABS
title: "PX8 clause-(a) evidence gap — interpreter capped-short Wrote lacks an absolute oracle; PR-C error identities unreached"
status: draft
owner: TBD
size: TBD
gate: none
depends_on: []
blocks: [PX8]
github: null
origin: architect PX8 closure-property verdict evt_163mfgjs7fkh8 (2026-07-23); Steward-filed (agents cannot create tracked work per COORDINATION §2)
---

Surfaced by the **Architect's PX8 closure-property verdict** (`evt_163mfgjs7fkh8`)
— clause (a) *absolute-not-differential* evidence is **not** discharged for two
value populations on the positioned/partial IO path. This is a clause-(a)
**evidence** gap (the source formulae are presently right; they are not
*asserted absolutely*), distinct from the clause-(a) **behavior** gap in
[[PX8-F-CAP-41]] and the clause-(b) provenance gap in [[PX8-SPAN-PROV]].

## The gap (Architect-grounded, exact anchors)

### A2a — interpreter capped-short `Wrote` has no absolute oracle
Interpreter `ReadSome` has capped-full **and** capped-short absolute assertions;
native has capped-full and capped-short for both `ReadSome` and `Wrote`.
**Interpreter `Wrote` has only capped-full** (`crates/ken-interp/src/eval.rs:6274-6379`).
Its distinct reifier arm is `eval.rs:4981-4997`, and the wrong shortcut
`effective := count` is **green when full** because both yield `remaining == 0`.
There is no interpreter capped-short `Wrote` assertion corresponding to native's
load-bearing `raw 8 / effective 4 / count 2 / remaining 2` case
(`crates/ken-runtime/src/cranelift_backend/lowering/core/tests/effects.rs:425-455`).
The closure condition requires this value asserted **absolutely** against LOCKED
`spec/30-surface/38-ffi-io.md`, and it is not.

### A2b — several PR-C error identities have no independent reaching evidence
`MalformedResource`, `InvalidBounds`, allocation-failure-distinct-from-`BufferLimit`,
unsupported-nonblocking posture, and host-I/O-failure-distinct-from-`Interrupted`
have **no independent reaching evidence** (`conformance/behavioral/buffer-io/
seed-buffer-io.md:619-645`). These are values reified by the positioned/partial
path, so the universal absolute-evidence claim of clause (a) cannot be made yet.

## Disposition / open question

Two admissible closure routes (Architect's verdict):
1. **Add the evidence** — the interpreter capped-short `Wrote` absolute oracle
   (mirroring native's `effects.rs:425-455` case) + independent reaching tests
   for the five error identities, each asserted absolutely against §38.
2. **Narrow the root property normatively** — if some error rows are out of the
   intended positioned/partial closure scope, the *current universal text of the
   PX8 property includes them*, so narrowing is a **spec/normative decision**
   (spec enclave + operator), not a silent scope trim.

⇒ Needs a scoping call (which error rows are in-scope for PX8 closure) before
sizing. The `Wrote` oracle (A2a) is a bounded test addition and is the
shovel-ready core; A2b's error-row set may split by the normative call.
Fix site crate: `ken-interp` (oracle) + conformance (`seed-buffer-io.md`) → **CV
in the review lane.**
