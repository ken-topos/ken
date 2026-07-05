---
scope: roles/conformance-validator
audience: (see scope README)
source: private memory `delivery-contract-op-list-can-overscope`
---

# Casting the Spec vote on a delivery contract: re-derive scope from source

When casting the independent **Spec vote** on a normative **delivery contract**
(a `§` that fixes *what a WP delivers*), **re-derive SCOPE from source, not the
prose.** A delivery contract that lists an op in its totality/guarantee clause
can silently **OVER-SCOPE past a settled OUT-boundary** (a sibling WP's
deliverable). The check is mechanical: **grep whether each op the contract names
is actually reduced/implemented today, and cross-check the frame's IN/OUT
scope** — never "does the prose read consistent." Live: F1 `§5.2.1(1)` grouped
`leq_int` with `eq_int` in the "compare over the true integers" **F1**
guarantee, but `leq_int` is *registered-but-unreduced* (`eval.rs`, no reduce arm
— the F5 WP's deliverable) while `eq_int` **is** reduced (`eval.rs:676`, F1).
Caught by grepping `eval.rs` against the brief's OUT-scope, **not** by reading
`§5.2.1` (whose clause lumped them). Scope-face of reconcile-don't-cite; sibling
of laundered citation authority (an op-list gains false "in-scope" authority by
sitting in a normative clause no one grepped against the registry row).

**Two co-carries from the same WP (F1 conformance):**

- **A rename that touches a case header must re-run the coverage-map name
  sync.** `grep case-headers` vs `grep coverage-map-ids | sort | diff` — a local
  edit (renaming an AC case in a revision) silently **desyncs a distant
  reference** (the coverage map still named the old id → dangling index;
  spec-author Fidelity caught it). Add the header↔map name-diff to the
  pre-handoff internal-consistency sweep (which already covers input/output +
  mechanism consistency). Same class as a reflow that injects a mid-token space
  in a distant span.

- **Establish-not-preserve is the tell that a hand-feed trap is live.** When the
  deliverable is a *newly-wired producer* into a *pre-existing consumer* (F1's
  `to_rt` BigInt arm — absent today, `eval.rs:212` — feeding the already-green
  `Value::BigInt`/`canonical.rs` store), the consumer is **already green with
  zero WP**, so a case that hand-feeds the consumer re-validates old code. The
  case must **drive the producer** (value arises from a real op → `to_rt`),
  verified by grepping the producer. Specializes conformance hand feeds the
  deliverable.

- **Before voting an erratum "presentation-only / no decision reopened," grep
  EVERY cited `OQ-`/decision-id in `90-open-decisions` — a dangling decision-id
  is, by itself, proof it is NOT presentation-only.** A "presentation-only"
  restatement that *cites a `DECIDED`/closed decision* is the highest-risk kind:
  the framing manufactures settledness, and if the id isn't in the register
  there is nothing to cross-check against. Live (44-capacity-restate
  `dec_5vjsm3nax4c6v`): §3 introduced a *new* positioning decision and cited
  "`OQ-systems-target` closed 2026-07-02" — but
  `OQ-systems-target`/`systems-adjacent` appear NOWHERE on main (register holds
  only `OQ-backend-target`, OPEN, codegen not positioning). Architect grepped
  the register and blocked; I had voted APPROVE on the diff's own
  "operator-closed" prose **without** grepping — I even flagged §3 as "the one
  substantive addition" and *still* didn't check its citation. The Spec/fidelity
  axis owns "does the spec cite settled decisions accurately"; a
  `DECIDED`-framed cite is the exact spot to run the register grep, not defer to
  the citer. Sibling of trust level prose vs locked adr crosscheck.

- **Erratum-review nuance (the FIX side): grounding a fabricated citation means
  REGISTER-the-decision, not delete-the-term — and the reviewer's strip-check
  targets false AUTHORITY/STATUS tokens only, never the grounded content.** When
  a dangling `OQ-<id>`/"closed" cite is fixed by the operator actually ruling,
  the fold (a) authors the real register entry and (b) strips only the
  *fabricated id* + false *status* phrasing ("closed in favour of") + any
  *pulled* wording — while the *positioning term itself* (e.g.
  "systems-adjacent"), now cited to the real entry, MUST survive. A blanket
  term-grep-to-zero false-blocks the correct fold (Architect flagged mine twice:
  exempt the unrelated "closed sum" intern- result, and keep grounded
  "systems-adjacent"). Scope the strip grep to the section + the false-authority
  tokens. Plus a standing **over-claim bar** on any newly-recorded DECIDED
  positioning: an ASPIRATIONAL reach gated on an OPEN dependency (native codegen
  / `OQ-backend-target`) must be marked "directional, not delivered," never
  upgraded to a shipped capability — honesty-about-the- boundary, my blocking
  lane (44-capacity §3 `OQ-domain`).
