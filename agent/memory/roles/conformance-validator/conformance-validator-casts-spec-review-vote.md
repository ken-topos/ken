---
scope: roles/conformance-validator
audience: (see scope README)
source: private memory `conformance-validator-casts-spec-review-vote`;
  decomposed 2026-07-28 — the citation-verification instances (X2/L6/B3/L2/
  Sec2/L7) were split out to
  `spec-crossref-must-resolve-to-content-not-a-number.md` as a distinct,
  orthogonal lesson (a re-derivation technique, not the vote-authority rule
  this file states)
related: spec-crossref-must-resolve-to-content-not-a-number
---

# conformance-validator casts the Spec review vote on every
spec/conformance Decision

**Operator decision, 2026-06-30, effective from the Sec1ct CT-D1 erratum
onward.** The **Spec review vote** on every merge Decision touching
`spec/`+`conformance/` is cast by **me, conformance-validator**
(`agt_37reqfr97xm00`) — *not* the dead `Spec` placeholder (`agt_37rekz81ceg00`:
`participant_type: agent` + `agent_adapter: null`, a non-running `moot init`
template actor, **never** a reviewer) and *not* spec-leader.

**Rationale (structural, model-agnostic):** CV *is* the independent-validation
role by design (re-derive + ground + reconcile-don't-cite), distinct from
spec-author who authors `/spec` — so the Spec gate is the highest-judgment
**independent** check, cast by the validator, not the author or the coordinator.

**The split — and the invariant it preserves (each piece checked by a
non-author):**
- **spec-author** authors `/spec`, never reviews its own work.
- **I** cast the **Spec** vote: attest the spec is correct via by-role
  independent validation (re-derive every structural claim from first
  principles, ground each cross-ref at its target, internal-consistency sweep) —
  not a content-match against the §-body.
- **spec-leader** proposes/assembles/routes the Decision; does **not** cast
  Spec. When assembling, names me as Spec reviewer with a real @mention to my
  actor_id (the dropped-mention failure mode of architect gate can be skipped
  review on main).
- **Architect** = external soundness gate, **always**.
- Independence preserved: **Architect checks my conformance; I check
  spec-author's spec.** When I author a conformance piece in the *same* Decision
  (e.g. Sec1ct CT-D1), I wear both hats — author the conformance edit AND cast
  Spec on the spec-author's spec edit — with the Architect the independent gate
  on my conformance half. No piece is self-reviewed on the axis that matters.

Verify any non-Architect review target is a *running* agent
(`agent_adapter: "mcp"`, recent `last_seen_at`) before trusting a routed vote —
the `moot init` placeholders are dead actors.

**Validated across 3 exercises (2026-06-30): librarian ASCII→Mermaid (caught a
diagram coupling the Architect's fidelity pass missed), Sec1ct CT-D1, L1
numbers.** All three: APPROVE scoped to my lane (semantic fidelity + conformance
validity), soundness explicitly disclaimed to the Architect, Decision left open
until both votes recorded (never resolve on my vote alone, §14).

## The reviewer hat extracts MORE than the author hat, on the same artifact

When I *author* conformance I resolve a spec ambiguity by silently picking the
right semantics and moving on; I don't flag that the *spec* was ambiguous.
Wearing the Spec-reviewer hat forces the **"would a build team conflate this?"**
question, which surfaces author-side terminological/mapping ambiguities my
authoring alone wouldn't. L1: I encoded AC3's degrade as panic/`unknown` (right)
without noticing `35 §3.2` overloads **"checked"** — the *subsumed* runtime face
of an undischarged obligation (panic/`unknown`) vs the explicit
`checked_add → Option T` op class (`None`, no panic) — a conflation risk a build
team could hit. Only the reviewer pass caught it (a verdict mapping silence is a
latent conformance bug instance: one label → two distinct runtime behaviors). So
on a both-hats Decision, run the *reviewer* pass over the spec as its own gate,
not as a by-product of authoring — they catch different defect classes.

## Subsume-don't-proliferate is a CROSS-WP discipline (L6 carry, 2026-06-30)

A kickoff's QA gate can over-scope its literal ask: L6 said "route a real I/O
signature through the actual `36 §1.4` escape check" — but that gate already has
its conformance home in the L5 `surface/effects/seed-effects.md` seed.
Re-pinning it = proliferation; assuming L5 covers L6 = under-coverage. Resolve
by separating the **bug targets** (*L5*: gate fails to check `⊆`; *L6*: an I/O
primitive declared **without** its mandatory row) — the new WP's cases
**reference** the prior home and pin only the **delta** (the operation-row
binding). The independent-checker move: locate the existing home **before**
authoring, assert the delta against it. Generalizes the within-file subsume (one
home per property) to the corpus level.

## The cross-reference-verification discipline lives in its own lesson

Six further instances (X2, L6, B3, Sec2, L7, L2) all found the SAME class of
defect — a spec citation that *resolves* (right file, real section, plausible
neighborhood) but doesn't host the claimed content — via the SAME re-derivation
discipline (open every cited target, read the body, don't trust the heading or
the §-number). That material is orthogonal to *who casts the vote* (this file's
subject) and is consolidated in
[[spec-crossref-must-resolve-to-content-not-a-number]].
