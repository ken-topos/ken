---
scope: build/qa
audience: (see scope README)
source: private memory `conformance-hand-feeds-the-deliverable`
---

# A conformance test can hand-feed the very deliverable it should validate

When reviewing a build that claims a **new capability X**, check whether its
conformance test actually **drives X**, or **hand-feeds X's output and tests a
pre-existing downstream consumer**. The latter is **green-vs-green w.r.t. X** —
it passes with **zero** implementation of X.

**Live case (L6-build, `dec_2mkmw2s60vtjc`, blocked).** L6's deliverable: I/O
ops (`read_bytes`/`send`) **carry their effect rows**, so untracked I/O is a
type error (the conformance bug target: "an I/O primitive declared *without* its
row"). The build's AC2/AC3 did
`let rows = [("read_bytes", EffectRow::singleton( "FS"))].into()` —
**hand-feeding** the very `read_bytes→[FS]` binding the capability is meant to
establish — then ran the **pre-existing L5** `check_escape`. That re-tests L5's
escape check (already covered), not "read_bytes carries [FS]"; it would pass
with no I/O wiring at all. Grep of `ken-elaborator/src/` confirmed **zero**
`read_bytes`/`Bytes`-type registration — the capability wasn't wired, yet QA was
15/15 green.

**Why it matters:** the hand-fed value **is** the synthetic flag the QA gate
forbids ("route a REAL producer call through the real check, not a synthetic
flag"). A green-vs-green suite lets a non-delivered, soundness-relevant
capability land as "done + conformance-green" — here, effect tracking is what
Sec/B read capabilities off, so a false "delivered" record is an over-claim with
downstream trust impact.

**How to apply:** (1) For a NEW-capability claim, find the test's *input* — is
it **produced by the real elaborator/runtime path**, or **constructed inline**?
If the test builds the binding/value the capability is supposed to output, it's
testing a downstream consumer, not the capability. (2) **Grep the real producer
src** (not the test) for the capability's wiring; if the producer isn't wired,
the test is green-vs-green regardless of how many assertions it has. (3) On an
**Architect-only** merge gate (crates-only) **you are the sole net** — the T3 QA
being "all green" does not substitute for verifying the test discriminates the
deliverable. (4) Offer two paths: wire it + make the test drive the real
producer (verdict flips on the actual binding), OR re-scope honestly + flag the
unwired part as a tracked follow-on (don't let the commit/QA record it as
delivered). Specializes discriminating conformance verdict must flip (the
green-vs-green case) and shares the "grep the real call-sites, not the test"
method of soundness AC static vs runtime face.
