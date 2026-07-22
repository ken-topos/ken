---
id: CAT-CAPEX
title: "catalog exhibits no checked capability/authority exemplar"
status: draft
owner: steward
size: TBD
gate: none
depends_on: []
blocks: []
github: null
origin: evt_2dgcc89s1yapn
---

**Surfaced by DOC-W1-3, confirmed by measurement, not filed as a doc problem.**
@doc-author grepped the **whole** `catalog/packages/` tree — not just the seven
registered fragments — for any capability-typed signature, `attenuate`, or
authority-lattice code (`Cap_FS`, `: Cap `, `CapParam`, `cap_set`,
`attenuate`):

> **zero hits anywhere in the catalog.**

`visits [...]` effect rows *are* exhibited (`Capability/Console/Text.ken.md`).
But **no checked fragment in the entire catalog shows an explicit capability
token, attenuation, or authority comparison.** `Filesystem/Errors.ken.md`'s
`AFull` is named only in prose — *"the current authority check is coarse and is
not path-confined"* — never as code there or anywhere else.

## Why this is a program fact, not a chapter problem

**@doc-leader already ruled the chapter correctly** (`evt_4b9pp185rmbpm`) and
that resolution needs no revisiting: ch04 teaches the effects half from real
checked fragments, and the capability/authority half from
`spec/60-security/62-authority.md` §7 labelled **unavailable in checked form**,
alongside the one real prose artifact — reusing the pattern ch03 validated for
`tested`. **The honest gap is itself part of what the chapter teaches.**

What that ruling correctly declines to do is fix the underlying asymmetry:

- **`spec/` now carries a behavioral contract for exactly this surface.**
  ABI-REVOKE merged at `9ebebb8e` — revocation lineage, denial identity split
  across two error families, admission/settlement semantics.
- **`catalog/` has no checked program exercising any of it.**

So the normative surface and the exemplar corpus have diverged on capability
and authority specifically. Two independent tracks hit the same wall from
opposite sides in the same hour, which is the signal this is structural rather
than incidental.

## Consequences worth naming before someone rediscovers them

1. **It recurs.** Every later chapter touching authority (DOC-W1 ch06 is
   already flagged in the wave's own brief) meets the identical gap, and each
   author re-derives the same escalation. One tracked fact is cheaper than
   five rediscoveries.
2. **`unavailable` labels accumulate silently.** Each is honest in isolation;
   in aggregate they are a measurement of how far the exemplar corpus trails
   the spec, and nothing currently counts them.
3. **It is a conformance question too, not only a library one.** A normative
   surface with no checked exemplar has weaker evidence behind it than one
   with — worth the Spec enclave's view on whether that matters here.

## ⛔ Not ready, and deliberately unassigned

**No ring should pick this up.** Ordering is unclear (is the exemplar blocked
on `ABI-R3` and the membrane implementation, or can a capability-typed fragment
be written against the landed contract today?), sizing is unknown, and the
build side is capped at two implementation tracks with idle rings being the
**intended** state per operator. **Filed so the fact survives, not to open a
track.** Route to the Architect / Spec enclave for the ordering question when
the current tracks free up.
