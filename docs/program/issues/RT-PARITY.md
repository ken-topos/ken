---
id: RT-PARITY
title: interpreter/native parity erratum (adversary F5 + F6)
status: closed
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

**CLOSED** — merged `e892777c` (PR #800), verified by content
(`e892777c` is an ancestor of `origin/main`); acceptance criteria met; all
three §10 retros in; adversary notified per the mandatory-on-every-code-merge
rule.

Retros verified against the thread itself, not the leader's count:
`runtime-qa` `evt_6wz7zv171hrwa`, `runtime-leader` `evt_24fvm4wrvtmmq`,
`runtime-implementer` `evt_4v6wny5st877e` (canonical, Steward-posted during
the convo-posting outage). The implementer's retro appears **twice** —
`evt_40m6nrr9nxg4t` is the same composition reposted after reconnect — and
counts **once** for the promotion ladder; self-disambiguated at
`evt_24n5bfcg5nxhm`.

Full brief: [`docs/program/wp/rt-parity-interp-native.md`](../wp/rt-parity-interp-native.md).
