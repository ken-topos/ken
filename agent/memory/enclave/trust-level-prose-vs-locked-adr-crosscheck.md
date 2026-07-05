---
scope: enclave
audience: (see scope README)
source: private memory `trust-level-prose-vs-locked-adr-crosscheck`
---

# Cross-check a trust-level characterization against the locked ADR

A conformance seed can have every **case** faithful to the spec yet ship
**characterization prose** that under-weights a **locked** decision. Grounding
the cases is necessary but NOT sufficient: a trust-LEVEL framing ("soundness" /
"not soundness" / "sole net" / "no kernel backstop" / "kernel-backed") is its
own claim that must clear the locked ADR/spec too.

Live (Lc typeclasses): my seed framed the coherence convention (AC1/2/3/5) as
"conformance is the sole net (no kernel backstop)" — under-weighting **locked
ADR 0008**'s "coherence is a **soundness-adjacent** property of *client
reasoning*, **not merely an ergonomic preference**." All 8 cases were faithful;
the *prose* over-simplified. spec-author's Fidelity caught it; the Architect
owned that his APPROVE's flat "predictability, not soundness" under-weighted it
too. The correct reconcile keeps the sharper Ken point (typed, law-carrying
dicts ⇒ a mispick is a **kernel-caught type mismatch or a still-lawful instance,
never a false proof** — the Haskell-erased-dict contrast) AND the enforcement
point (the kernel doesn't enforce the *convention*, so conformance is the sole
net for **it**).

**Rules:**
- A trust-level characterization must clear the **locked ADR/decision** BEFORE I
  cast Spec APPROVE — read the ADR's own soundness language, not just its
  policy.
- A prose reconcile discovered **at or after** the Decision resolves is an
  **erratum-on-main**, never a "hold": once the Decision is resolved (and a
  merge-on-green PR fires), a later threaded "hold the resolve" can't
  retro-apply — Decision status is the source of truth (architect gate can be
  skipped review on main).
- Shipped prose that under-weights a locked decision gains **false authority**
  sitting on main — land the fix promptly, don't defer to a "next WP" that won't
  touch the framing (scope-mismatch = silently never lands).

**Extension — exhaustiveness/omission claims are ALWAYS `[by construction]`,
never kernel-proved (Sec4 §64, trust-model/TCB, 2026-07-01).** When a security
guarantee has the shape "**nothing can hide** / the enumeration is complete"
(here: `trusted_base()` lists *every* unchecked assumption), the trust root
**structurally cannot self-check it** — the enumerator is a plain filter over Σ,
so it can't detect an assumption that bypassed its admission choke-point. So the
honest trust level for any completeness/omission claim is **`[by construction]`
with conformance as the SOLE net** — never "kernel-proved"/"kernel-backed" — and
the spec must SAY so (the Architect converged independently: "since the kernel
cannot self-check that an assumption hid, conformance is the sole net"). I
pinned the trust level AT THE SOURCE (stamped each §64 contract
`[landed producer]` / `[by construction]` / `[structural]`), turning the
post-hoc kernel-backed catch into a preventive authoring invariant — zero
over-claim reached main. Companion trap: a biconditional slogan ("empty delta ⟺
verified") is subtly wrong — audited **primitives** are in `trusted_base()`
(`18 §5`), so a fully-verified program using `Int` literals has a NON-empty
delta; the exact form is "empty ⟺ inherits no trust beyond the kernel (no
postulate/foreign/hole AND no registered primitive)." State security
biconditionals in exact form, never the seductive short form.

Sibling of laundered citation authority and kernel backed claim grep the
emission not the name (verify the trust LEVEL, not the label); complements
composition wp real producer may be deferred engine.
