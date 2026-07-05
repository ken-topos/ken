---
scope: enclave
audience: (see scope README)
source: private memory `capability-gate-three-state-lifecycle`
---

# A capability-gate has three prose states; the middle one goes stale either way

A conformance/spec claim gated on a forthcoming kernel capability moves through
**three** prose states, not two: **over-claim** (says realizable before the
capability lands) → **interim-park** (honestly "parks as visible `Axiom` pending
X") → **realized** (real proof on main). The **middle state is intrinsically
transient** and goes stale in **both** directions:
- over-claim side: "pending X (forward)" is stale the moment X *merges* — caught
  by decoupling capability-landedness from the real blocker (K7 erratum #38:
  re-key the park reason to the *proof-wiring WP*, not the capability);
- under-claim side: the honest "park pending the wiring" is stale the moment the
  *wiring* lands — caught only by verify-on-main (#39: `9a82745` wired the real
  `Ord Bool`/`DecEq Bool` proofs one commit after the erratum → seed now
  under-claims).

**Why:** in a fast concurrent-merge arc, no phrasing survives — when the
referent lands, "pending it" is stale by definition; only a *prompt coupled
flip* fixes it.

**How to apply:** (1) when authoring an interim-park case, PRE-FILE the
realized-flip task keyed to the named wiring commit, and put a one-line "(flip
to realized when `<WP>` lands)" marker in the seed itself — don't let the flip
be discovered post-merge. (2) Cite a not-yet-merged forward capability **by
capability + spec-section, never the squash-bound branch SHA** (a branch tip
never appears on main; only merged SHAs like K5's `1c84a30` are citable). (3)
Keep interim-net *vehicles* decoupled from the transient park state so they
don't ALSO need flipping (the `absurd-subterm` delta-honesty net re-keyed onto
the structural `collect_consts_in_tb`/`63f3050` construction stayed live
continuously through park→realized). Validated across the K4→K5→K7→remainder
un-stage ladder (#33/#36/#38/#39). Sibling of trust level prose vs locked adr
crosscheck.

**Fourth failure mode — the gate DISSOLVES (customerless reframe), not just
advances (#37 Eq-flip).** A capability-gated field can turn out realizable by a
*different* mechanism, leaving the original gate **customerless**. Do NOT flip
"gate landed → realized" — that mis-attributes the realization to an uninvolved
(often *unbuilt*) capability. The load-bearing check is the **counterfactual,
not the landing**: *would the gate, if landed **soundly**, have even closed this
obligation?* For K6 (Eq Bool sym/trans): a **sound positional** `conv_struct`
Eq-congruence compares `bool_eq x y` vs `bool_eq y x` → stuck args `(x,y)` vs
`(y,x)` → fails; only the **unsound cross-wise** arm (definitional Eq-arg
symmetry, collapses directed Eq) would → hard NO. So K6 is genuinely
customerless; the pair realized by full case-split (K7 reduces `bool_eq` on
concrete args). Re-derive the counterfactual from the mechanism's own rules
(positional-vs-cross-wise) — three-way convergence (my re-derivation +
Architect's ruling + the spec) is what makes a customerless reframe trustworthy.
Then attribute realization to the *real* mechanism and **grep-confirm the false
`"<gate> landed"` phrasing is absent** in seed/README/§6. Also: stacking a
second doc-flip on an UNMERGED first flip that edits the same lines → the
first's squash-merge turns the second into a stale base (merge-tree conflict) →
rebase the second's net-new commits onto post-first-merge main before its own
merge (content-preserving re-parent; re-verify byte-identity at the fresh SHA,
gates carry).
