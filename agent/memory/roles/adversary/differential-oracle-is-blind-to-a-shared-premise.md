---
name: differential-oracle-is-blind-to-a-shared-premise
description: "A differential test between two implementations is a RELATIVE oracle: it cannot detect a defect both sides share. When a WP's gate is a differential, the residual risk IS the shared premise — check each side against the spec, not against each other."
metadata:
  type: feedback
---

A differential/parity test asserts *A agrees with B*. That is a **relative**
oracle. It is structurally incapable of detecting anything **A and B share** —
and the things two implementations of one contract most reliably share are the
contract's own ambiguities. So on any WP whose gate is a differential, the
parity result is near-worthless as evidence of correctness, and the highest-value
hunt is: **pick the axis both sides compute, read BOTH formulas, then check the
agreed value against the spec (an absolute oracle).**

Live case (2026-07-21, RT-PARITY @ `e892777c`): six executable interp↔native
differentials pinned checked buffer-IO narrowing. Parity genuinely held. But
both sides derived `TransferCount.remaining` from the **raw** request length
(interp `eval.rs`, native `cranelift_backend.rs`, from `positioned_bounds`
narrowed *before* dispatch and so blind to capacity), while the host **clamps**
to `effective = min(length, capacity - start)` (`effect_v1.rs`) and validates
the transfer against `effective`. Locked `spec/30-surface/38-ffi-io.md`
required the count be bounded by the *"effective request"*. Capacity-4 buffer,
`length: 8`, 4 bytes read ⇒ both reify `remaining = 4` on a **full** buffer;
spec required 0. All six differentials passed green.

**Root cause was partly the spec**, which is the recurring pattern: the
"effective request" clause and the read contract's own partition table
restated the bound inconsistently (`≤ requested` vs `≤ effective`). Both
implementations read the same table.

**How to apply:** (1) on a parity/differential WP, treat green parity as
*evidence about agreement only*; (2) enumerate the quantities both sides
compute and read each formula verbatim rather than trusting the differential;
(3) check the agreed value against the **normative** text, and when the two
disagree, grep the spec for its *own* inconsistent restatements of the same
bound — a shared implementation defect usually traces to a sentence, not to two
independent mistakes; (4) report parity-holds as a first-class negative so
nobody reads the finding as a parity regression. Sibling of
[[green-vs-green-does-not-confirm-a-fix]] (same blindness, one layer up) and of
[[differential-verify-which-mechanism-is-the-net]].

★ **Currency note (2026-07-28):** this defect has since been fixed on both
sides — the interp regression test
`budget_eff_capped_full_read_reifies_effective_not_raw_remaining` and the
native lowering path (now under
`crates/ken-runtime/src/cranelift_backend/lowering/core.rs` after
RT-NATIVE-FNSPLIT split the former monolithic `cranelift_backend.rs`) both
compute `remaining` from the reply's `effective_request`, and the spec's
partition table reads consistently. The bug is closed; the **method** —
audit the shared premise, not just the agreement — is the durable lesson and
still applies to any future differential gate.
