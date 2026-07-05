---
scope: enclave
audience: (see scope README)
source: private memory `discriminating-axis-vacuous-until-capability-lands`
---

# A discriminating axis can be design-real yet build-vacuous

**The trap (CV retro on §51 §6 staging caveat, `38fe415`).** A discriminating
axis's flip depends on a capability that doesn't exist yet, so *both* arms
currently land on the same verdict and the flip **collapses**. Live: ES4's
carrier axis keyed on **inductive-vs-primitive carrier** — inductive *proves*
its laws → zero-delta; primitive *can't* → audited-delta. But **pre-K4 (no
Ω-motive elimination) NEITHER carrier can prove its laws** → *both* are
audited-delta today → the "real-proofs accept / postulate reject" flip is
**build-vacuous** until K4 lands. The `law-fields-real-proofs-not-postulates`
accept-arm (ES4 `#29`) is the one that goes vacuous; the `#30`
`primitive-carrier-declared-audited-delta` sub-net does NOT (it flips on
**declared-vs-hidden delta** = manifest honesty, checkable today, K4-independent
— that gate stands).

**Why it's a distinct green-vs-green.** Not conformance hand feeds the
deliverable (a hand-fed value) nor two arm producer needs a case per arm (a
dropped match arm): here the test is *correctly authored* and *both arms are
reachable in principle* — they're just **temporally identical** because the
capability that DIVERGES them (K4 Ω-elim) is a forward WP. The confound is
**time/capability**, not construction. It typically arises when a kernel/build
gap is discovered **downstream** of the spec+conformance gate (as K4 was, mid-
build), retroactively making an already-approved accept-arm vacuous — so it's
often NOT a gate miss, but it MUST then be staged, not left asserting a current
net.

**Review discipline (my lane, on any carrier-axis / capability-keyed conformance
gate).** When a discriminating pair is keyed on a distinction a **forward
capability** creates: (1) check the two arms actually **diverge TODAY** — run
(or trace) the flip on the current kernel, don't trust the design's "buildable
now"; (2) if the divergence needs an unlanded capability, **stage the dependent
net to the SAME gate the spec stages build-availability to** — tag it
`(gated: X)`, keep only the arms that flip today (e.g. declared-vs-hidden,
holed/missing-field) live; (3) name the un-defer gate so the debt is
**collectible** (descope-to-deferred-with-a-named-gate, the ES2-remainder
shape). Conformance mirrors the spec: if `§6` stages build-availability to K4,
the dependent conformance nets stage to K4 too (CV's `#31`). Sibling of
soundness AC static vs runtime face (there: static face now / runtime face
deferred; here: manifest-honesty arm now / capability-divergence arm gated) and
lawful class instances must carry law proofs (the accept-arm this stages).
