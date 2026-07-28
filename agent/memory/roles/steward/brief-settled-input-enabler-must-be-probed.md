---
scope: roles/steward
audience: (see scope README) — anyone authoring a WP frame/brief
source: `pedagogic-catalog-prototype`, 2026-07-11
---

# A brief's "settled input" asserting a mechanism enabler must be probed

When a WP brief pins a **"settled input"** that asserts the
elaborator/toolchain *supports* some capability (e.g. "top-down declaration
order is supported — a `lemma` may be stated above the `fn` it invokes"),
that claim must be **probed on the actual code**, not inferred from a spec
clause plus a code read, before it is written down as a fixed, do-not-reopen
input.

**Why:** on `pedagogic-catalog-prototype` (2026-07-11) I pinned "top-down is
supported" (citing a mutual-recursion clause and `expand_scope`'s name
pre-pass — `crates/ken-elaborator/src/modules.rs`) as a settled input. It
was **false** — declaration order is bottom-up for every decl kind; only
mutual-recursion cycles group. The Architect had asserted the same from a
careful code read and had to retract it twice before a minimal elaborator
probe gave ground truth. The build implementer falsified the brief in
minutes by building the real thing. A false *"what's supported"* in a brief
is worse than a false *"what's broken"*: it sends the ring to build on a
foundation that isn't there.

**How to apply:** distinguish a *decided-design* input (cite `/spec` + the
open-decisions register — fine to pin) from a *mechanism-enabler* input (the
toolchain does X today). For the latter, either (a) get it from a
probe/test, not a read, or (b) tag it "verify against the landed code —
probe before relying." Same shape as
[[frame-must-ground-substrate-obligations-not-just-names]] — a settled
input that names an *enabler* must be probed before it is pinned, whether
the enabler is a toolchain capability or a primitive's producible field.
