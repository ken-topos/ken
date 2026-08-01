---
scope: fleet
audience: (see scope README) — anyone mutation-proving a check, especially
  a new gate they just added
source: Q-CLAIM-CLOSURE, 2026-07-23
---

# A mutation-proof injection point is a reachability tell

When you mutation-prove a check, mutate at the **NATURAL production site**,
never a convenient injection point. If the mutation must be injected past
the real guards to reach your check, the check is unreachable in
production — and the injection point is exactly what hides that.

A generated-side uniqueness check was added in `crates/ken-host/src/lib.rs`
to catch a duplicate ABI fact name in `TARGET_ABI.facts` (currency-checked
2026-07-28: `TARGET_ABI`, `verify_probe`, and `linux_raw_facts()` all still
exist in `crates/ken-host/build.rs` / `build_support.rs` — grep the
symbols rather than trusting a line number). The check was
"mutation-proved" by injecting a duplicate at the **write layer** of
`build.rs` — `let facts = { let mut f = facts; f.push(dup); f };` right
before the canonical-manifest step, after the build script's own guards.
It went red at the new check. Reported as proved.

## The defect

QA mutated at the **natural site** instead — duplicated a line in
`linux_raw_facts()`, the place a real ABI edit would touch. It never
reached the new check: `build.rs`'s `verify_probe` cardinality gate panics
first ("probe emitted N facts; expected the closed inventory of N+1"),
**before** the generated-code write runs, so `TARGET_ABI` never carries a
duplicate. Every count-preserving variant hit a different pre-existing gate
(a boundary-inventory producer/consumer closure check; a "duplicate probe
fact" parse error). **A real duplicate cannot reach the new check at all**
— it is dead in the production pipeline. The claim "the anchor catches
duplicates" had evidence that didn't carry it: the exact defect the WP
existed to close, on the author's own fold. The check was reverted and the
real upstream detectors credited by name.

## The tell, stated as a rule

**If a mutation needs an artificial injection point to reach the code
under test, the code is unreachable in production.** Injecting past the
real guards does not just fail to prove reachability — it actively HIDES
the unreachability, because it manufactures the arrival the real path
denies. The convenient injection point feels like a modeling shortcut; it
is the whole bug.

## How to apply

- **Mutate where the value is really produced / where a real change would
  be made** — the source function, the generator input, the actual call
  site — not a spot chosen because it's close to the check. Reaching for
  `{ let mut x = …; x.push(); x }` right before the assertion is the
  injection-point smell; stop.
- **If the natural mutation is caught upstream and never arrives, the
  check is dead** in the real pipeline. Either it is genuinely unreachable
  (revert it and credit the mechanism that *does* fire, attributed by name
  and reproduced), or the real first line of defense has been found — say
  so.
- **Enumerate what sits between the mutation site and the check.** Each
  guard in between can intercept the mutation; if any always does, the
  check is shadowed. This is the reachability dual of
  [[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]]'s
  freshness axis ("is the harness looking at the CURRENT code") — here the
  question is "can the mutation even ARRIVE at the check on the real
  path."
- **A synthetic unit test of the detached logic does NOT rescue it.** It
  proves the helper, not its reachability in the real pipeline.
  Reachability is the property under doubt; testing the isolated logic
  answers a different question.
- **Same-day companion lesson:** every fix that ADDS a claim introduces a
  fresh thing to overclaim. After writing any corrective claim, re-run
  "does the evidence carry EXACTLY this?" on the new wording before
  handoff — the reviewer will, so do it first. Four review rounds collapse
  toward one.

Related:
[[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]].
And if one checker is defeated repeatedly, suspect its **default branch**
is the thing that is wrong, rather than patching each defeat.
