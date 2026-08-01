---
scope: teams/verify
audience: (see scope README)
source: DOC-VALIDATION-BINDING retros, 2026-07-24 — named independently by the
  implementer (as its strongest "held") and by QA (as its "carry"),
  evt_78qng91927xvj. Tier-1 candidate: two seats, one team, one run.
---

# An executable inventory needs a reversible deletion proof at its stored reference

When a mechanism claims *"this registry is the one authoritative list, and every
row in it actually runs,"* the text of the registry is **not evidence**. A list
of tokens that reads like an inventory can still be a **shadow list** — names
sitting beside the real dispatch rather than driving it. It looks identical in a
diff, and a green suite confirms nothing, because the tests pass whether or not
the row is what caused the runner to execute.

> **A registry proves nothing about binding. Only a *break* does.**

## The proof that discriminates

Delete or rename the thing the row names, and show the failure lands **at the
registry's own stored reference** — then restore byte-for-byte:

1. Rename the referenced item (`check_links` → anything), leaving the registry's
   stored reference (`run: check_links`) untouched.
2. Build. The **expected** result is a resolution error **at the registry line**
   — `E0425` in the row itself, not a test assertion failing downstream.
3. Restore byte-for-byte and confirm green.

Step 2 is the whole proof. A failure *anywhere else* — a test assertion, a
runtime check, a missing-file error — means the row is **not** the binding: the
build found the item some other way, and the registry is decoration.

## Why this is stronger than any test you could add

A test asserts the inventory is complete **at the moment it runs**. The
deletion proof shows the inventory **cannot be incomplete** — an unlisted or
misnamed entry stops compiling. That converts a checked property into a
structural one — the same move as making a matcher exhaustive by
construction: make the load-bearing completeness a **compile error**, not an
assertion.

⇒ It also generalizes well past this gate. Any *"X is the single source of
truth"* claim — a dispatch table, a manifest, a capability registry, a
generated-artifact ledger — is testable this way, and it demonstrates the
binding instead of arguing for it.

## How to apply

- **QA:** when a candidate claims an executable inventory, do not accept
  token-text agreement as evidence. Ask for the reversible break, and require
  the failure be **located** — "it went red" is not the claim; "it went red *at
  the registry line*" is.
- **Implementer:** run it before handoff and put the exact error code and
  location in the handoff. It is the cheapest way to make a binding claim
  reviewable.
- **Restore byte-for-byte and re-verify green.** A mutation proof leaves the
  tree mutated; commit the real fix *before* any mutation-proof reset, or the
  reset eats it.
- **The mutation must be injected where the thing is really produced.** If a
  break is caught upstream and can never reach the registry, the proof is dead
  and its pass means nothing — see
  [[a-mutation-that-passes-when-it-should-fail-means-a-stale-input]].

## Ladder status

**Tier-1 (team-local) candidate.** Two seats reached it independently on one WP,
which is one run in one team — short of the promotion bar (≥3 runs, or ≥2 teams
independently). Promote to `build/qa/` when a second team hits it. It is
node-internal (it sharpens what an existing reviewer checks), so it adds no
party, relay, or gate.
