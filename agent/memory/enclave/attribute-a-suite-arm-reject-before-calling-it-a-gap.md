---
scope: enclave
audience: (see scope README)
source: private memory `attribute-a-suite-arm-reject-before-calling-it-a-gap`
---

# Attribute a suite-arm reject before calling it a gap

Running a discriminating suite (challenge exercises, conformance arms), a REJECT
or a should-PASS-that-failed is **not self-attributing** — the *reason* matters,
and there are four distinct causes that all look like "a gap" at first:

- **(a) authoring bug in the arm** — a typo/wrong form in your own exercise
  masks a working capability. (C3-sound: `Some (MkProd …)` omitted `Some`'s
  explicit type arg → "expected Type 0"; `unfoldUpTo` elaborated fine on its
  own. Fuel-unfold capability was always present.)
- **(b) coincidental reject — wrong reason** — the arm rejects, but for a
  *different* reason than the semantic gate it targets, so the intended gate is
  never reached. A **parser gap** rejecting a `data Perm : Ω` declaration masks
  the Ω proof-irrelevance sort gate; an `UnresolvedCon` masks the soundness
  edge. The absence-assertion discipline's dual: an absence/reject must fire for
  the *right* reason.
- **(c) harness / loading** — the feature is landed but not in the run
  environment. (C1/C6: `DecEq`/`Ord` live in `packages/lawful-classes/`, not the
  CLI's default `ElabEnv`; surface `import` didn't resolve a package path → the
  fix was *prepending* the package, then the real edges appeared.)
- **(d) genuine feature gap** — kernel-capability present, surface can't reach
  it. (Indexed families / quotients `A/R` / Ω-data ctors don't parse; funext IS
  consumed but dependent-`match`-into-`Equal`/Ω doesn't refine.)

**Why:** the CV-challenge suite first run reported several "surface gaps"; the
Steward asked for **attribution drills** on the two unclassifiable arms.
**Minimizing** (strip the arm to the smallest failing/passing piece; run
targeted probes) reclassified them: C3-sound was **(a)** (my typo, not a gap);
C8-sound was a **correction** — funext IS reachable (probes A/B pass with
`\x. Refl`), the residual is a **(d)** dependent-`match`-into-Ω gap, *not*
"funext unreachable" as the first pass reported. Two of the map's entries
flipped under attribution.

**How to apply:** (1) Never report a suite arm as "a gap" from the top-level
error alone — **minimize to attribute** (isolate the sub-term; run the smallest
positive/negative probes; check whether the feature is merely un-loaded). (2)
State the *reason* a reject fires, and whether it's the intended gate or a
coincidental earlier failure. (3) The four buckets need different dispositions:
**(a)** fix the arm; **(b)** the gate is untested — note it, don't credit the
reject; **(c)** load it and re-probe the *real* edge; **(d)** the actual
frontier — document it. Extends green vs green does not confirm a fix and the
absence-assertion "reject for the right reason" rule from the pass direction to
the *reject/gap* direction.
