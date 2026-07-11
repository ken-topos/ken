# WP `proof-vocabulary` — make `proof … for` a first-class subject-organizing form

**Owner:** Language team (elaborator) · **Spec enclave** for §33 + soundness ·
**Architect** algorithm review · **Size:** L · **Risk:** medium (touches the
admission path — soundness-bearing half).
**Depends on:** nothing (design fully settled). Supersedes the earlier
`acyclic-forward-reference-elaboration` candidate (absorbed).

## Objective

Make `lemma`/`proof` first-class in Ken's declaration model by wiring them into
the same **signature-first, dependency-ordered, SCC + SCT** elaboration that
`fn`/`const` already use, and by relaxing the two incidental `proof … for`
restrictions that make `proof` a near-dead keyword. Result: recursive and
mutually-recursive proofs (under SCT), forward references, and a genuine
subject-organized proof namespace — `leq_nat::refl/trans/antisym`,
`add::comm/assoc/zero_l`. This is what lets the pedagogic-catalog initiative
retire the `fn _ind` + synonym-wrapper idiom for a single recursive `lemma`, and
what makes `proof … for` usable for real laws.

## Settled design — FIXED inputs

Architect rulings: evt_2evdthp5fey6p, evt_3bvdpr3paf8t7, evt_3d8xkej5gkwv3.

Three changes on **one elaboration seam**. Do not relitigate; cite the rulings.

**(A) Recursive `lemma`/`proof` under SCT — soundness-bearing.**
Wire `lemma`, `proof`, and attached-`proof` decls into the same
dependency-ordered **SCC + SCT** elaboration `fn`/`const` use today (that path is
`ViewDecl`/`LetDecl`-only at `crates/ken-elaborator/src/modules.rs:879`).
- The admission guard **keys on SCT-acceptance, NOT cyclicity**: a self-edge
  (structural induction) and a homogeneous proof↔proof mutual cycle (mutual
  induction) are **both admitted iff SCT accepts**; an SCT-rejected self-reference
  and any non-descending cycle **fail closed** — identical obligation to
  `fn`/`const`.
- **Acyclic forward references** (`lemma`→later `fn`, `lemma`→later `lemma`) are
  **delivered by the signatures-first generalization — NOT "for free"**
  (Architect-corrected, probed on `origin/main`). The current `fn`/`const` seam
  (`modules.rs:879`) walks the SCC-condensation in *source order* and declares
  signatures-before-bodies only *within* one mutual-recursion SCC, so an acyclic
  forward `fn`→`fn` fails today with `UnresolvedCon`. Generalizing
  `elaborate_mutual_group`'s signatures-first discipline from per-SCC to the whole
  declaration scope + elaborating bodies in **topological order** is net-new work
  (this is the absorbed `acyclic-forward-reference-elaboration`/#19 content) — in
  scope, and the mechanism is Phase 1 of the admission algorithm below.
- Soundness rests on two properties (state them, don't assume — enclave
  checklist below): SCT termination is **result-type-agnostic** (it already
  accepts the merged Ω-valued `_ind` fns), and **proof-irrelevance** means Ω-proofs
  are never δ-unfolded in `conv`, so recursion introduces no conversion/reduction
  burden. See memory `recursive-omega-proof-soundness-sct-plus-irrelevance`.

**(B) Telescope relaxation — ZERO-soundness surface.**
Replace `validate_attached_subject_telescope`
(`crates/ken-elaborator/src/elab.rs:5415`) with an **"occurs-applied-in-φ"**
check: `proof p for s` is well-formed iff the subject `s` occurs **applied**
somewhere in the proof's type φ (hypotheses **or** conclusion). Broad reading:
`proof p for s` = "a named property **of** s."
- This has **zero soundness impact**: attachment is namespacing over an
  already-checked theorem (`elab.rs:3714` — `AttachedProof` elaborates via the
  identical `elaborate_checked_theorem` path as `lemma`; the result is a plain
  `declare_def` named `subject::proof_name`; `RAttachedProofRef` at
  `elab.rs:2192/3452` resolves to `RCon("subject::proof_name")`). **Nothing is
  keyed on the telescope** — no coherence/auto-discharge/obligation/rewrite/
  instance hook consumes an attached proof (Architect grep-confirmed).
- **Keep** the "subject must be an already-resolved definition" precondition
  (`elab.rs:5433` — a real precondition).

**(C) Remove the no-sibling-dependency check — soundness-bearing (co-lands with A).**
Delete the check at `crates/ken-elaborator/src/elab.rs:5356-5378` (it collects all
`subject::*` ids and rejects if the proof's type/body mentions any). Under (A)+(B)
it is **redundant AND actively wrong**: proving `s::antisym` via sibling `s::trans`
or mutual sibling induction is natural, and once proof decls go through SCC+SCT,
sibling references are ordinary dependency (acyclic resolve; mutual admitted iff
SCT accepts; non-terminating fail closed). It **must be deleted in the same change
as (A)** or they conflict.

**Out of scope (keep fail-closed narrow):** mixed `fn`↔`proof` mutual *cycles*
(a Type-relevant `fn` is δ-unfolded in `conv` while its Ω-proof partner is not — a
non-trivial interaction with no customer; DEFER). `prop`'s `where`-intros beyond
the v0 seed into full inductive Ω-relations (separate feature). Both stay OUT.

## §33 spec edits (Spec enclave)

- **§33 §1** (mutual recursion): extend "all top-level definitions are mutually
  recursive within a module if the SCT check accepts the group" to **explicitly
  include `lemma`/`proof`** in the SCC/SCT grouping (today read as view/let-only).
- **§33 §8.3 / §8.2** (lemma / proof): replace "body may not self-reference" with
  "a `lemma`/`proof` body **may self-recurse and mutually recurse with other proof
  decls iff the recursion passes SCT**; SCT-rejected proof recursion fails closed."
  Replace the §8.2 **telescope-repeat** rule with the **occurs-applied-in-φ**
  condition, and **remove** the §8.2 **no-sibling-dependency** rule.
- State the **admission invariant** normatively: proof recursion is admitted iff
  SCT-accepted (identical to `fn`/`const`), and **proof-irrelevance is preserved**
  (Ω-proofs are not δ-unfolded in `conv`).

## Admission algorithm — normative pseudocode (Architect, evt_eqvvn2qxykc8)

This is the Architect's trust-root read (every SCT branch, one rejection per
guard) and the artifact the kernel/elaborator review walks. It **replaces the
source-order SCC walk for the whole declaration scope** (fn/const AND
lemma/proof/attached-proof).

```
elaborate_scope(decls):
  # Phase 1 — DECLARE ALL SIGNATURES FIRST (generalizes elaborate_mutual_group's
  # sig-first step from per-SCC to the whole scope; THIS is what delivers forward refs).
  for d in decls:
    ty := elaborate_type(d.signature)          # resolves other decls via their SIGNATURE only
    if is_proof_decl(d):                        # lemma | proof | attached-proof
      ensure_omega_type(ty)                     # [Guard 1] existing proof gate — φ must classify at Ω
      if is_attached(d):
        require subject_is_resolved_def(d.subject)          # [Guard 2a] KEEP (elab.rs:5433 precondition)
        require occurs_applied_in_phi(d.subject, ty)        # [Guard 2b] (B): replaces telescope-repeat
    declare_signature(d.name, ty)              # global visible to ALL bodies (forward refs)

  # Phase 2 — dependency graph over bodies AND types; condense to SCCs.
  G := { d -> e : e occurs in d.body OR d.type }
  SCCs := condense(G)                          # Tarjan; each node = one SCC

  # Phase 3 — elaborate bodies in TOPOLOGICAL order of the SCC-condensation
  # (dependencies' bodies ready for δ-reduction before dependents are checked).
  for scc in topological_order(SCCs):
    recursive := (scc.size > 1) OR has_self_edge(scc)
    if recursive:
      if mixes_proof_and_computational(scc):   # [Guard 3] OUT OF SCOPE — fail closed, EXPLICIT
        reject "mixed fn/proof recursive cycle not supported (WP-deferred)"
      if not sct_accepts(scc):                 # [Guard 4] THE soundness guard — keys on SCT, not cyclicity
        reject sct_diagnostic(scc)             # specific SCT error, not a generic reject
    for d in scc:
      body := check(d.body, signature_of(d))   # kernel-checks body against φ (Ω for proofs); UNCHANGED
      declare_body(d.name, body)
```

**Every SCT branch, one rejection case per guard:**
- **Non-recursive** (acyclic SCC, size 1, no self-edge) → admitted directly, SCT
  not consulted. (Covers `add_zero_r = Refl`, and every forward reference once
  Phase 1 has declared signatures.)
- **[Guard 4 / 4a] Self-recursive, structural descent** (`refl_leq_nat` recursing
  `Suc x2`→`x2`) → `sct_accepts` = true → **ADMIT**. Identical obligation to the
  merged `refl_leq_nat_ind` fn.
- **[Guard 4 / 4b] Self-recursive, NON-descending** (`lemma bad : φ = bad`, no
  decreasing arg) → `sct_accepts` = false → **REJECT (SCT diagnostic)**. The
  load-bearing fail-closed case that keeps a looping "proof" of a false Ω-prop out.
- **[Guard 4 / 4c] Mutual proof cycle, common decreasing measure** (homogeneous
  proof↔proof, SCT finds descent across the cycle) → **ADMIT** (mutual induction).
- **[Guard 4 / 4d] Mutual proof cycle, no decreasing measure** → **REJECT (SCT
  diagnostic)**.
- **[Guard 3] Recursive SCC mixing a proof with a `fn`/`const`** → **REJECT
  explicitly** ("mixed cycle, deferred") — never silently admitted; the
  OUT-of-scope boundary made fail-closed.
- **[Guard 1] Proof decl whose φ is not at Ω** → reject (unchanged
  `ensure_omega_type`).
- **[Guard 2b] Attached `proof p for s` with `s` not occurring applied in φ** →
  reject (surface well-formedness; the theorem is checked identically either way —
  zero soundness).

**Load-bearing invariants the algorithm preserves (enclave review checklist):**
1. **No proof recursion admitted un-SCT-checked** — Guard 4 runs on EVERY
   recursive SCC (self-edge or size>1) before any body in it is declared; no path
   declares a recursive proof body while skipping SCT.
2. **SCT is the sole recursion discriminator** — no categorical cyclicity reject
   anywhere (Guard 4 asks `sct_accepts`, never `size > 1`).
3. **Proof-irrelevance untouched** — Phase 3 `check(...)` is the existing kernel
   check; conv still short-circuits on Ω, so no recursive Ω-proof is δ-unfolded.
   Topological body order only affects `fn`/`const` δ-reduction availability.
4. **Signatures-before-bodies is scope-wide** — the only new structural change; a
   strict generalization of the existing per-SCC discipline, so existing fn/const
   cycles behave identically (criterion 6 regression net covers it).

**Implementation note for the owning team:** topological body order is a
*reordering* of pure definitions (elaboration has no observable side effect beyond
declaring globals), so it is semantics-preserving for existing code — but the
regression suite (criterion 6) is the proof; flag any diagnostic-order or
shadowing-sensitive test that shifts. The Architect will review the as-implemented
SCT wiring (Guard 4 + Guard 3's mixed-cycle rejection especially) at the
kernel/elaborator review.

## Deliverable structure — split by review lane

Author/land the WP so review effort lands where the risk is:

1. **Soundness-bearing core (A + C).** The SCC+SCT wiring of proof decls + the
   no-sibling-check deletion. Requires: (i) Spec-enclave **soundness statement**
   (the checklist below, stated not assumed); (ii) **kernel/elaborator algorithm
   review** — Architect trust-root pseudocode read: walk every SCT branch, one
   rejection case per guard (SCT-rejected self-ref; non-descending mutual cycle;
   admitted structural descent).
2. **Zero-soundness surface (B).** The `validate_attached_subject_telescope` →
   occurs-in-φ swap. Requires: **Architect surface review only** + a conformance
   pair (below). No kernel/Spec soundness review for this half — authoritative,
   because attachment is namespacing over `elaborate_checked_theorem` with nothing
   keyed on the telescope.

## Enclave soundness checklist (state these; all expected to hold)

- (a) SCT's strong-normalization result is **result-type-agnostic** — termination
  from the call graph / structural descent, independent of `Type` vs `Ω` codomain.
- (b) A strongly-normalizing (SCT-accepted) definition of an Ω-prop **is a valid
  proof** — termination is the whole obligation; no extra burden from Ω codomain.
- (c) **Proof-irrelevance is preserved** — admitting recursive Ω-proofs does not
  force `conv` to unfold them (conv short-circuits on Ω); no new reduction burden.
- (d) **Erasure/extraction unaffected** — recursive Ω-proofs carry no computational
  content at use sites and erase as before.

## Acceptance criteria (testable)

1. **Recursion (A):** a self-recursive `lemma` elaborates green —
   ```
   lemma refl_leq_nat (x : Nat) : Equal Bool (leq_nat x x) True =
     match x { Zero ⇒ tt ; Suc x2 ⇒ refl_leq_nat x2 }
   ```
   and a homogeneous proof↔proof mutual pair that passes SCT elaborates, while an
   SCT-**rejected** proof recursion (non-descending self/mutual) **fails closed**
   with the SCT diagnostic (assert the specific error, not bare `is_err`).
2. **Forward refs (A):** a `lemma` stated above the `fn`/`lemma` it invokes
   resolves (no `UnresolvedCon`).
3. **Telescope (B):** `proof refl for leq_nat`, `proof trans for leq_nat`,
   `proof assoc for add`, `proof zero_l for add` all elaborate (subject occurs
   applied in φ); a `proof p for s` whose φ **never** mentions `s` is **rejected**.
   Conformance pair pins both arms.
4. **Sibling refs (C):** `proof antisym for leq_nat` may reference sibling
   `leq_nat::trans`; a mutual sibling pair passing SCT elaborates; the old
   no-sibling rejection is gone.
5. **Zero TCB / soundness invariant:** `trusted_base_delta` unchanged; the
   admission invariant holds (no proof recursion admitted un-SCT-checked); a
   round-trip that Ω-proofs are not δ-unfolded in `conv`.
6. **No regression:** the merged pedagogic files (NatArith/OrdNat) and the full
   elaborator suite stay green; the `_ind`+wrapper idiom can then be *optionally*
   simplified (separate follow-up, not this WP).

## Do-not-reopen guardrails

- The guard keys on **SCT-acceptance, not cyclicity** — do not reintroduce any
  categorical "reject proof cycles" rule (retracted).
- The telescope relaxation is **zero-soundness surface** — do not route it through
  a kernel/Spec soundness gate; it is a `validate_*` swap + conformance.
- **Mixed `fn`↔`proof` cycles and `prop` inductive-relations stay OUT.**
- Keep the "subject is an already-resolved definition" precondition.

## Notes

This collapses the pedagogic prototype's `fn _ind` + synonym-`lemma` pair to a
single recursive `lemma`, and makes `proof … for` the live subject-organizing
form the initiative reached for — at the cost of one validation swap + wiring
proof decls into the SCC+SCT seam `fn`/`const` already use. Sequenced by the
operator; timing is the operator's call. The Architect will review this frame and
the admission *algorithm* at pseudocode level before implementation.
