---
scope: fleet
audience: (see scope README)
source: private memory `surface-the-seam-need-not-your-preferred-mechanism`
---

# Surface a cross-author need; leave the mechanism to the owner

The dual of the seam-surfacing discipline (discriminator negative arm must be
expressible and reaching): surfacing a load-bearing cross-author dependency at
plan time is right — but **how you frame its resolution matters.** Surface the
**need/constraint**; do **not** bundle it with your preferred mechanism, even as
"my recommendation."

**Why:** effect-composition D5↔D3 seam. AC3 (generality) needed an **executable
≥2-pairing discriminator**, whose expressibility was contracted by the
Architect's D3 subsumption choice. I surfaced the seam correctly, but I **framed
the executable route as *requiring* option-1** (`run_io` subsumes `run_state`) —
which I recommended — because from the conformance lane that looked like the
only way to make a 2nd effect-pairing top-level-runnable. Architect **rejected**
option-1 (folding a kernel-re-checked fold into trusted Rust = TCB regression)
**yet still delivered the executable route** via a door invisible from my lane:
the **pure-handler role** — the author writes in-source `runState s₀ (…)`, which
peels State via the general `resp_sum` and re-emits the base op for `run_io` to
run. A TCB-clean 2nd pairing I hadn't seen. Had I authored AC3 committed to my
recommended route, the whole generality net would have been mis-specified
against the actual COEXIST mechanism. What saved it: writing AC3 to **both**
routes and holding the collapse until the ruling.

**How to apply:** (1) When you surface a seam, separate two things cleanly — the
**invariant/constraint you need** ("AC3 needs an executable 2nd pairing that
reaches the driver") vs. **a mechanism that would satisfy it**. Post the first
as the load-bearing ask; offer the second only as *one option among possibly
others the owner sees*, never as "the way." (2) A recommendation bundled with
the need has two failure modes: it **anchors** the owner toward your option
(which may carry a cost — here a TCB regression) they'd otherwise route around,
and it makes a **rejection** of your option read as "then the need can't be met"
— when in fact the owner can meet the need a better way. (3) If you author
downstream of the still-open seam, write to **all** live resolutions and hold
the collapse until the ruling — don't let your own recommendation harden into
the draft's default. (4) The owner's lane routinely holds doors invisible from
yours (the pure-handler route was invisible from conformance); trust them to
find the cheapest one **if** you leave the mechanism open. Sibling of enclave
elaborates autonomously no build team pulls (route judgment to the owning edge)
and buildability ruling must ground every axis (a from-one-lane ruling misses an
orthogonal axis).
