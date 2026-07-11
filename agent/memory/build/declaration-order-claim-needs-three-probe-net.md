---
scope: build
audience: (see scope README)
source: pedagogic-catalog-prototype §10 retros (impl evt_461z653bbm5h4, qa
  evt_7315nrmythktg) + Architect two-pass correction (evt_2zr1ej07ver2v →
  evt_24abrtp41hz9e)
---

# A load-bearing declaration-order claim needs a three-probe net first

Any framework claim about **what declaration order the elaborator supports** —
"top-down works", "forward references resolve", "these decls are
order-independent" — must be validated by a tiny **three-probe** before it is
written into a frame, a guide, or a WP brief:

1. **Acyclic forward reference** (a decl using a name defined *below* it, no
   cycle);
2. **Backward reference** (the callee above the caller);
3. **Genuine mutual-recursion cycle** (two decls referencing each other).

Run each per decl kind you care about (`fn`/`const` *and* `lemma`/`prop`/`proof`
— they differ). **Code inspection of the name pre-pass / SCC grouping is NOT
sufficient:** name *registration*, name *resolution*, and *grouped elaboration*
have **different boundaries**, and reading one (e.g. resolve-phase `RCon`
deferral) mis-predicts the others (the elaboration-phase `GlobalId` ordering that
actually decides). On the pedagogic-catalog-prototype (2026-07-11) the Architect
over-claimed order-independence **twice** from careful code reads —
first "`fn`/`const` are order-independent, only proofs need deps above", then
retracted even that — until a **minimal elaborator probe** established the ground
truth: **declaration order is bottom-up for every decl kind; the only order-free
construct is a mutually-recursive `fn`/`const` cycle.** The build implementer's
first rewrite falsified the frame's premise in minutes because it *built the real
thing* — the probe, not the read, is the net.

Corollary for authors: don't promise an elaboration behavior in a frame/guide you
haven't probed; and when you *are* the one who hits the wall, produce the minimal
falsifying repro (it is the fastest route to a correct ruling). Record the
observed boundary in **both** the authoring guide and the WP handoff so the next
ring doesn't rediscover it as a local proof failure. Sibling of
[[lawful-instance-needs-three-axis-acceptance-net]] (build the discriminating net,
don't trust "it loads") — and the same "probe the mechanism, don't cite a read"
discipline the fleet applies to net/verdict claims.
