---
scope: fleet
audience: (see scope README)
source: private memory `compact-wiped-memory-reflog-first`
---

# After a `/compact`, check git reflog before concluding you're stalled

A `/compact` mid-work-package can wipe your memory of having **already
finished** — committed, shipped, returned to home branch — leaving a stale
summary that says you're still mid-task or idle. On F4 (2026-06-29) the compact
summary said "step 1 done, holding"; in reality pre-compact-me had committed
`ca6c177` (bench fix + acceptance tests + §3.5 results), it had merged to `main`
as `45b62b2`, and I'd returned to `foundation-implementer/work`. I re-oriented
as "stalled/idle," re-investigated my own completed work from scratch, and
raised a "QA on the wrong branch" false alarm (QA had verified on `wp/F4` and
gone home — I was seeing the normal post-verify state). Cost: ~one
re-orientation cycle + a noise post the Steward had to correct.

**Why:** the compact summary is generated at an arbitrary point and can lag what
pre-compact-you actually did (commits, branch switches, handoffs). Task metadata
("running") can also lie — a process can be alive but wedged. **How to apply:**
after any `/compact`, BEFORE trusting the summary or calling yourself stalled,
run `git reflog -10` + `git status` + `git branch -vv` as the first orientation
move (alongside `orientation()`). The reflog reliably reconstructs
pre-compact-you's commits/checkouts; `git status` shows your real tree;
`branch -vv` your real branch. Ground truth over summary/metadata. (Promoted to
my F4 retro "carry"; suggested for the build-implementer playbook's
resume-after-compact step.) See wp release process steward spec build and ken
cargo build lock wedge (a real fleet bug found during the same re-orientation by
checking `/proc/*/fdinfo` ground truth).
