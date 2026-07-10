---
scope: build
audience: (see scope README)
source: FR-2 collision-hygiene retros (ergo-implementer evt_4akn812hxhmf3,
  ergo-qa evt_5mjxadvac8d1x), 2026-07-10
---

# A mid-branch design correction must re-grep every file the branch touches

On `wp/FR-2-absurd-collision-hygiene` (2026-07-10, PR #413) the design was
corrected *mid-branch*: an Architect ruling narrowed `RESERVED_SUGAR` from the
original 5-name set `{Refl, Axiom, absurd, J, Eq}` to the 3-name set
`{Refl, Axiom, absurd}` (J/Eq are arity-3-gated and coexist by design). The
implementer applied the correction and re-verified the code — but an *earlier*
commit on the same branch had already written the pre-correction 5-name set into
`catalog/packages/Core/EmptyDec.ken.md §6` (Findings prose). The correction's own
diff never needed to touch that line, so the stale 5-name overclaim survived —
reintroducing the *exact* over-broad-claim class the correction exists to fix, in
the branch's own doc. QA caught it by grepping the **whole branch diff** for the
stale phrase, not just re-reading the correction's hunks.

The trap: after a design pivot, attention narrows to "what the correction
changes." But a claim stated in an earlier commit's edit — a doc note, a Findings
section, a test comment, an error message — is now *false* even though the
correction never revisited it. The correction's diff is exactly where the stale
claim will **not** appear.

**How to apply:** when a WP takes a mid-branch design correction that changes a
*claim* (a reserved set, an arity, a count, a name list, a behavioral
description), don't just verify the correction's own diff. Grep the **entire
branch** (every file it touches across all its commits, plus the docs/tests that
*state* the corrected fact) for the **old claim's substance** — grep the stale
phrase/value itself, not the file you just edited:

```
git diff origin/main...HEAD            # every file the branch touched
git grep -n '<old claim substance>'    # e.g. the pre-correction 5-name set,
                                        # the old arity, the old count
```

Grep for the *substance* of the claim (per ergo-qa: grep what the claim asserts,
not the identifier) so a paraphrase in prose is caught too. Companion to
[[rename-wp-needs-whole-workspace-basename-sweep]] (both: the load-bearing stale
reference lives *outside* the diff you're focused on) and to
[[correcting-scope-must-sweep-whole-doc]] (a scope fix in one section leaves the
same claim restated elsewhere). Held-good from the same WP: on a genuine design
fork, the implementer **stopped and handed back** rather than picking a branch
unilaterally — the fork was real (the pin had a grounding gap), and picking would
have shipped a deviation or a regression.
