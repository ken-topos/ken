---
id: PX8-ERRID-SCOPE
title: "PX8 clause-(a) A2b — five PR-C error identities have no independent reaching evidence, and whether they are in PX8's closure scope is a normative call"
status: draft
owner: spec-enclave
size: TBD
gate: none
depends_on: []
blocks: [PX8]
github: null
origin: "Split out of PX8-WROTE-ABS by the Steward 2026-07-27 when framing its A2a half. Both halves trace to the Architect's PX8 closure-property verdict evt_163mfgjs7fkh8 (2026-07-23)."
---

⛔ **This is the half of [[PX8-WROTE-ABS]] that cannot be sized yet.** A2a (the
interpreter capped-short `Wrote` absolute oracle) is framed and released;
this is A2b.

## The gap

`conformance/behavioral/buffer-io/seed-buffer-io.md:619-645`. Five PR-C error
identities have **no independent reaching evidence**:

- `MalformedResource`
- `InvalidBounds`
- allocation-failure distinct from `BufferLimit`
- unsupported-nonblocking posture
- host-I/O-failure distinct from `Interrupted`

These are values reified by the positioned/partial IO path, so clause (a)'s
**universal** absolute-evidence claim cannot be made while they are unreached.

## ⛔ Why this is not simply "write five tests"

The Architect's verdict named **two** admissible closure routes, and the second
is normative:

> *if some error rows are out of the intended positioned/partial closure scope,
> the current universal text of the PX8 property includes them, so narrowing is
> a **spec/normative decision** (spec enclave + operator), not a silent scope
> trim.*

⇒ ⭐ **The question that must be answered first is which of the five are in
PX8's closure scope at all** — not how to reach them. Sizing follows the
answer, and the set may split: some reachable and testable, some narrowed out
of the property by an explicit normative edit.

⛔ Do not frame this as an implementation WP before that call. A team handed
"add reaching evidence for five error identities" would silently pick route 1
for all five, which is exactly the trim the Architect ruled against.

## Disposition

Owner is `spec-enclave` because the deliverable is the scoping ruling, not the
tests. Once the in-scope set is fixed, the reaching-evidence work is an
ordinary build WP and can be re-owned.

⚠ **`PX8` does not close until this and [[PX8-WROTE-ABS]] and
[[PX8-F-CAP-41]] all discharge**, and the closure property is re-verified.
