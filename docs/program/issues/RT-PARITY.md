---
id: RT-PARITY
title: interpreter/native parity erratum (adversary F5 + F6)
status: merged
owner: runtime
size: M
gate: none
depends_on: [SPAN-SEAL]
blocks: []
github: null
origin: evt_7njbntfhre2qx
---

Two seams broke the "interpreter is the reference oracle, native is the
trusted lowering; they must agree" premise: `ReadSome` reification hardcoded
`remaining = 0` on the interpreter side (F5), and a companion native-lowering
mismatch (F6). Neither was a safety hole — both failed closed — but a
differential test would have flagged *native* as wrong when native was
correct.

Confirmed by the Architect's batch ruling `evt_4e5bqa5tes7nm`, which directed
folding F5+F6 into one Runtime parity erratum with a conformance companion.
Held behind `SPAN-SEAL` (a prerequisite, not a parallel task) per that WP's
own "Blocking" note.

**Merged** `e892777c` (PR #800). Verified by content; all three §10 retros in;
adversary notified per the mandatory-on-every-code-merge rule.

Full brief: [`docs/program/wp/rt-parity-interp-native.md`](../wp/rt-parity-interp-native.md).
