---
scope: enclave
audience: (see scope README)
source: private memory `spelling-currency-sweep-separate-from-vacuity`
---

# Run a spelling-currency sweep separate from the vacuity check

When a spec pin changes a **signature or a coupling** (not just a value), the
landed conformance corpus can carry two *independent* kinds of debt, and the
obvious check only catches one:

1. **Vacuity** — does a discriminating case go green-vs-green under the change?
2. **Spelling currency** — do the case's *prose spellings* (signature, the named
   coupling, the cited mechanism) still describe the surface the pin now locks?

The trap: a discriminating net can **survive** the pin (stay green, non-vacuous)
while the case *prose* describes a **superseded** surface — and the surviving
net **masks** the stale spelling. So the two checks are orthogonal; passing
vacuity does **not** certify currency.

**An author's "no conformance piece needed" scopes to (1) only** — new
discriminating-case gaps + vacuity. It is **not** a claim about (2). The
**conformance owner** must run the spelling-currency sweep, because the author
(rightly) isn't scoping to existing-case currency and the surviving net hides
the stale coupling from them.

**Rule:** a **signature/coupling-changing spec pin ⇒ the conformance owner
sweeps the landed corpus for cases that *spell* the superseded surface**,
separate from the vacuity check, and reconciles them against the **merged** spec
body (check main via git object store not find lock-against-landed-body) — as a
follow-on, **not** folded into a spec-only branch (the temporal-race discipline,
two arm producer needs a case per arm).

**Disposition (two severities → two fixes).** A changed pin produces two kinds
of stale case, needing different fixes: **(1) stale spelling** — the case's
*subject still exists*, only its surface spelling moved → **mechanical swap**,
stays live (its discriminating net is unchanged). **(2) latent over-claim** —
the pin *deferred the case's subject itself* → a swap would leave it asserting a
property against a now-absent/stub target (**false-green**). For (2):
**descope-to-deferred, don't delete** — keep the case, tag it `(deferred)`, and
**re-point its intent at the future gated path** (the WP that will re-introduce
the subject). Deleting loses the intent + coverage-continuity; swapping leaves a
false-green against a stub. Gate the descope with a **"does deferring lose a
live net?"** check first — confirm the surviving coverage still has a live home
(else it's a silent hole). Live: the two `user-ord-*-sort` cases' subject (the
`Ord`→`sort` coupling) was deferred, and `Ord` landed as an empty stub → I
descoped them to `(deferred)`, re-pointed at the `where Ord a` desugaring WP,
after verifying the `DecEq` path + base `sort-emits` stay live.

Live (ES2-remainder §37 §6, 2026-07-01): the pin changed the verified `sort`
from `where Ord a` to an **explicit comparator** `leq : a → a → Bool` and
**deferred** the lawful-`Ord` class. spec-author said "no conformance piece
needed" — correct on vacuity (AC6's Perm-conjunct-present net + AC7's
`instance_search` verdict-flip are **comparator-independent**, stay green) — but
**three** landed cases still spell `sort where Ord a`
(`sort-emits-issorted-and-perm` + the two L3b AC7 user-`Ord` cases, incl. "the
Ord dictionary carries the total-order law proofs," now deferred content). I
caught it on my CV-Spec APPROVE, tracked it as my own follow-on (not a spec
blocker — the spec is correct). Sibling of conformance hand feeds the
deliverable (verify against the real producer, not the prose) and verdict
mapping silence is a latent conformance bug (a silence the surviving net hides).
