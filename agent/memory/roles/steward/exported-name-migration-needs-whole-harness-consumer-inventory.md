---
scope: roles/steward
audience: (see scope README) — anyone framing a rename/retire WP
source: `proof-attachment-membership-pilot`, 2026-07-12
---

# An exported-name migration needs a whole-harness consumer inventory

When a WP **renames or retires exported names** — catalog law names,
registered globals, surface constants — producer-side fidelity (the
declaration still type-checks under its new name) is **not** sufficient.
Test/acceptance harnesses often **pin the old name by string**
(`assert env.globals.contains_key("add_comm")`, `"add_assoc must be a
checked global"`), and those assertions fail closed the moment the producer
renames. A green *focused* producer suite hides them; the full workspace
test suite in CI catches them, but only after a stale-scope frame has
already shipped a red suite as "done."

2026-07-12 (`proof-attachment-membership-pilot`): the frame said "doc/catalog-
only, exactly 3 files" but a `proof name for S` rename retired the flat
law-names that an acceptance test file (test names of the shape
`nat_arithmetic_laws_acceptance.rs`, asserting expected-globals by string)
depended on → Foundation hit an "impossible acceptance bar," held at the
seam for a scope ruling. Ruling: authorize the coupled test-assertion
migration (names-only, a 4th file) + grep the whole test tree for every
retired name before handoff; "waive the stale assertion" was out (ships a
red workspace to `main`). The retro carry generalized it: exported-name
migrations require an attributed, whole-harness consumer inventory
alongside producer fidelity checks.

**Currency note (verified against the tree):** `nat_arithmetic_laws_
acceptance.rs` still exists (`crates/ken-elaborator/tests/`), but its law
names have since moved to a `::`-namespaced convention (e.g. `add::assoc`
rather than the flat `add_comm`/`add_assoc` quoted above) — the *literal*
strings cited are stale; the mechanism (a test asserting `globals.contains_
key(<string>)`, which fails closed on any producer rename) is unchanged and
still the thing to grep for.

**How to apply (when framing a rename/retire WP):** put the **coupled
consumer inventory into scope up front** — "grep `crates/**/tests` + all
harnesses for every renamed identifier; migrate every coupled assertion in
the same branch; the suite must be green at handoff; no compatibility
aliases." Don't frame it as "catalog-only, N files" when the names are
consumed by test assertions — that under-scopes and forces a mid-build
hold. Sibling of
[[migration-sweep-glob-must-enumerate-every-ken-source-root]] — same root
failure (under-scoped inventory), there on the producer/source side rather
than the consumer side.
