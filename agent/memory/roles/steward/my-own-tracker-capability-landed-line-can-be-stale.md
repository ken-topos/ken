---
scope: roles/steward
audience: (see scope README)
source: private memory `my-own-tracker-capability-landed-line-can-be-stale`
---

# Your own tracker's 'capability X landed' line can be stale

Before I write a WP brief/kickoff that asserts "capability X has landed, so
re-author against it," I must **grep the actual reduction/emission site on
`origin/main`** — not trust my own tracker's or checkpoint's "X landed" line. My
own durable notes drift and can **conflate a related-but-different landed
thing** with the target capability.

**Live (2026-07-04, VAL2 Phase 2):** my tracker + the `read-file-lines` kickoff
premise said "#9 FS landed" — but I had conflated **L6 (the `Bytes` type +
`bytes_decode`/`bytes_length`, which DID land)** with the **FS-driver (actual
`read_bytes`/`write_bytes` file-I/O reduction, which was only *framed*, never
built)**. `git show origin/main:crates/ken-interp/src/eval.rs` has zero
`read_bytes` reduction; `FS-driver.md` isn't even on main. I shipped the false
"landed" premise in a build brief; the **implementer caught it by grepping**, I
didn't. The gap (`GAP-fs-read-unwired`) was fully open the whole time.

**How to apply:** a "landed" claim that gates a WP premise gets a **producer
grep on `origin/main`** at the exact site (`eval.rs::apply` for an interp
reduction, `declare_*` for a kernel primitive, the elaborator emission for a
surface feature) — the *name* of a sibling feature landing ("Bytes", "L6", "the
effect type exists") is not evidence the *specific* reduction/op is wired. This
is the WP-premise dual of kernel backed claim grep the emission not the name and
named floor must be grepped not assumed — same rule (grep the mechanism, not the
name), now applied to my *own* status records, which have the same
authority-drift risk as a laundered citation.
