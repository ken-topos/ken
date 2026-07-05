---
scope: roles/conformance-validator
audience: (see scope README)
source: private memory `conformance-oracle-grounding-fallback`
---

# Grounding /conformance results when the prototype oracle isn't runnable

As conformance-validator, "ground every case against the prototype oracle before
locking" assumes `local/refs/yon` is mounted. In the agent worktrees it often
**isn't** (gitignored, operator-provided; confirmed absent in *every* worktree
during K2c series-1, 2026-06-29). Grounding fallback that keeps the discipline
without the live oracle:

- **Pin what the spec determines.** Expected results fixed by the on-`main`,
  MIT-clean spec (e.g. `16` obs reduction tables, `12` level calculus, `17` η/β
  rules) are authored directly and cited to the normative paragraph. Re-use
  already-locked expecteds from sibling seed files (e.g.
  `conformance/kernel/observational/seed-observational.md`).
- **Settled-principle verdicts need no prototype.** SCT admit/reject derives
  from the size-change *principle* (`17 §4`); the prototype has **no** SCT gate
  (it is K2c's net-new headline), so it could not be an oracle for SCT anyway —
  compute the verdict from the principle. See trust root test coverage
  discipline.
- **Tag genuinely free choices `(oracle)`.** The corpus convention
  (`conformance/README.md`; `seed-observational.md` header) defers prototype
  confirmation of free implementation-detail expecteds to **build time** by the
  Spec enclave. Use the tag rather than locking an unground guess.
- **Don't lock a verdict you can't ground and the spec doesn't settle** — drop
  it or note it deferred (I dropped an SCT "sound-but-incomplete" swap case
  whose verdict I couldn't confirm). A wrong conformance case licenses wrong
  code fleet-wide.
- **Flag the shelf-absence** to the spec-leader as a coordination fact — it also
  blocks spec-author's "read the prototype to ground the algorithm" step and any
  future `(oracle)` confirmation; the operator must mount the shelf for
  build-time validation.

Also: cite to the spec author's **landed** section numbers, not the elaboration
plan's assumed ones — they drift (K2c `17`: plan said §3.1 whnf/§3.2 conv,
landed was §3.2 whnf/§3.3 conv/§3.6 convLevel, §4 merged to §4.3).

The reconcile pass must verify section **content/behavior**, not just that the
number resolves to a same-titled heading. A spec can *refine the behavior* under
an unchanged heading, and that changes the correct conformance verdict/stage. V0
(2026-06-29): §5.x sub-numbers matched my cites exactly (no drift), but landed
§5.6 specified a λ-vs-non-Π as `LambdaVsNonFunction` raised **structurally in
the elaborator before the kernel** — so a case I'd attributed to "kernel
rejects" had the wrong stage. Caught only by reading the §5.6 body, not its
title. When reconciling: re-read each cited section's text and confirm it says
the thing the case relies on (the verdict, the *stage*, the exact level).

Second instance — L5 effects (2026-06-29): the §2 heading "The encoding" was
unchanged from DRAFT-v0, but landed §2.1 *pinned the interaction-tree node
constructor* as `Vis e k` (with `perform e = Vis e (λr. Ret r)` the
smart-constructor). My parallel-authored structural assertions used `perform`
*as the node*, which a heading-only check would have kept — wrong tree shape.
The body read also resolved an open `(oracle)` (§1.4 `ρ_inf ⊆ ρ_decl`:
over-declare = allowed upper bound) and pinned a forced level (§7.4
`max ℓ_R ℓ_op ℓ_resp`). **Two instances now (V0 §5.6 stage, L5 §2.1 constructor)
— the reconcile-content rule generalizes: when authoring in parallel with the
spec-author, a structural assertion's exact tokens (constructor names, stage,
level, ⊆-vs-=) are only ground once the body lands; re-read it, don't trust the
heading or your pre-landing draft.** Also reusable: a content-reconcile that
finds spec-internal inconsistencies (a wrong cross-ref number, two sections
whose wordings only reconcile under an unstated invariant) is the
independent-checker's catch — flag them to the author, route via the spec-leader
(no new conformance→author edge).
