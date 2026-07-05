---
scope: enclave
audience: (see scope README)
source: private memory `package-ecosystem-comprehensive-standard-small-contrib`
---

# Package ecosystem: comprehensive standard library, small contrib surface

Operator ("Pat") strategic direction, 2026-07-01 (a "shower thought" but real
policy). Shapes ES4 (standard-package catalog), F3b (registry), and Sec3
(supply-chain).

**The strategy (three layers):** minimal trusted core (built-ins = minimum
generating set) + **broad lawful DERIVED standard library** (comprehensive, "all
anyone needs") + **small third-party surface**.

- **Standard packages are COMPREHENSIVE, not minimal** — unlike the built-in
  set, the catalog aims to cover as much general functionality as possible,
  lawfully and coherently. This does NOT conflict with subsume-don't-proliferate
  or small-auditable-TCB: by the ES1 certified minimality invariant a derived
  package adds ZERO to `trusted_base()`, so the library layer can be maximally
  broad at zero trust cost. "Batteries included" and "small auditable TCB" are
  compatible *because the batteries are all derived*.
- **Contrib is a curated, upstream-able staging tier, kept small** — a contrib
  facility exists, but sufficiently-general functionality is reviewed,
  harmonized, and integrated into standard packages (deduped, held to the same
  laws-PROVED-not-postulated discipline). Contrib is a pipeline INTO standard,
  not a permanent parallel universe.
- **Small third-party universe = supply-chain risk reduction** — making standard
  cover most needs + upstreaming the rest shrinks the attack surface Sec3
  (re-check-on-consume, provenance, `63`) must defend. A comprehensive
  first-party catalog is a FIRST-CLASS supply-chain control. F3b must
  distinguish the governed standard/contrib tier from arbitrary third-party —
  "not npm for Ken."

**Why:** it's `subsume-don't-proliferate` (docs/PRINCIPLES.md) applied to the
ecosystem layer, with a security payoff. The comprehensiveness is *enabled* by
the ES1 zero-TCB-delta invariant — that's the load-bearing reconciliation.

**How to apply:** scope ES4 ambitiously (broad coverage, but non-redundant +
lawful, each entry with its ES1 derivation path — comprehensive ≠ kitchen-sink).
When F3b is scheduled, encode the contrib→standard upstreaming governance + the
tier distinction. Captured in `docs/program/everyday-surface-program.md`
(*Package-ecosystem strategy* section). **Open design Q for ES4/F3b design
time:** is "contrib" a ken-topos-hosted curated area vs. the open third-party
universe? — Steward lean: a curated reviewed staging tier that feeds standard,
arbitrary third-party possible but deliberately a small minority.
