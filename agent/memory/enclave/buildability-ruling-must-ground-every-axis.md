---
scope: enclave
audience: architect, spec-author, spec-leader, conformance-validator
source: merges former private memories `buildability-claim-ground-every-axis`,
  `buildability-classify-every-capability-axis`, and
  `buildability-ruling-must-ground-every-axis` (three write-ups of the same
  recurring ruling-discipline gap, across Map-container and CAT-1/CAT-2)
related: sizing-a-subsume-fix-enumerate-every-piece (the sizing-a-fix dual),
  named-floor-must-be-grepped-not-assumed,
  isolate-mechanism-from-orthogonal-fail-closed-gates
---

# A "buildable now" ruling must ground every capability axis the proof touches

A spec/conformance review classifying a proof as **buildable now** vs
**deferred** must check **every construction-capability axis the proof
exercises** — not just the one the split is named after, or the one salient axis
under debate. A ruling that correctly clears the axis being discussed can still
be **wrong** because a second, orthogonal capability wall blocks it — and an
actual empirical build attempt is ground truth that a from-one-axis reasoning
ruling cannot substitute for.

**The recurring shape (Map-container, 2026-07-03).** spec-author's original
`/spec` claimed 5 induction proofs were "buildable, this WP" by lifting the
`Ord Bool` case-split idiom onto Map's abstract keys — an over-claim, since
`leq k k'` on an abstract key is stuck and needs a transport/`J` step the
elaborator doesn't have ("Gap A"). That got caught and an erratum was routed.
But **the first fix carried a second over-claim**: it kept `toList`-ordered in
the "buildable now" column because its proof is comparison-free (never touches
the stuck `leq`) — clearing Gap A. It still didn't build: `toList` ordered
inducts over the non-nullary `Tree`/`List` carrier, and `check_match_dependent`
is gated to **nullary constructors only** (`ken-elaborator/src/elab.rs:455`) — a
second, unrelated wall ("Gap B": dependent-motive narrowing, not equality
transport). The Architect's ruling reasoned correctly about Gap A and missed Gap
B entirely, because the reasoning was scoped to the axis under discussion. Only
foundation-implementer's actual build attempt (an empirical 2-line repro with no
`leq` in it at all) surfaced the second wall.

**Why this survives review.** Every review pass — spec-author's own, the
Architect's ruling, CV's grounding — was implicitly scoped to "is the axis under
discussion buildable," and each answered *that* question correctly. None asked
"are there OTHER capability walls this proof might hit, unrelated to the one
we're debating." A reasoning-only ruling structurally cannot see a wall it isn't
looking for; only an actual construction attempt (or an exhaustive per-mechanism
grep) surfaces it.

**The axis checklist (grep each, don't reason only from the salient one):**
1. **Elaboration path** — does the surface `match`/def lower to a well-formed
   core term? (`check_match_dependent`'s nullary gate; `infer_match`'s constant
   motive; the nested-pattern compiler.)
2. **Termination (SCT)** — does the recursion pass `sct_check`? A separate
   fail-closed gate with its own completeness holes.
3. **Transport / propositional rewrite** — is there a stuck comparison needing
   `J`/`cast` to fire?
4. **Dependent induction** — does it narrow a hypothesis/goal per-branch over a
   **non-nullary** family? Blocked outright with no eliminator/`match` escape if
   the mechanism is absent — there is no surface `elim`/`rec` keyword, `match`
   is the only `Term::Elim` path.
5. **Obligation-discharge** — if routed through the refinement/`{x|φ}` prover,
   does it actually *discharge* the obligation, or emit an unproven hole?
   "Emits" is not "discharges."
6. **Surface syntax** — does the idiom need a token the parser lacks? Often
   subsumed by restructuring (e.g. putting the hypothesis in the motive) — grep
   before assuming a parser gap is load-bearing.

**How to apply.**
1. Identify every distinct elaborator mechanism the proof's construction touches
   — clearing one does not clear the others.
2. Grep the construction capability for **each** mechanism independently.
3. Treat an actual build/construction attempt as higher-confidence ground truth
   than a reasoning-only ruling, especially one scoped to a single axis. A
   2-line repro is ground truth a from-one-axis ruling is not.
4. State the verdict as the **conjunction** of every axis; if you can't cheaply
   ground one, say so and defer to an empirical build attempt rather than
   guessing.
5. When a genuine build-completeness gap is found, check whether the spec
   already commits to the general mechanism — if so, the honest diagnosis is
   "the elaborator lags the spec," not "the spec over-asks."
6. Own the correction plainly when a second wall surfaces after you ruled — it
   was yours to un-rule.

**Sharpening 1 — grounding mode, not just ruling mode (CAT-2 Fork E,
2026-07-04).** The same obligation applies when a claim is offered as *grounding
evidence* for someone else's ruling, not just when you are the one ruling. A
"wires instance X / zero new code" claim must run head `X` through
`elab_instance_decl` before asserting — a **parametric instance head** (free
type/effect variables, e.g. `ITree e resp`) elaborates its head in an empty
context and hits `UnresolvedCon`, so it does *not* elaborate even though the
underlying operations denotationally satisfy the relevant laws.
**Denotational-satisfies is not the same axis as surface-elaborates** — this
axis bit twice in successive WPs (CAT-1 then CAT-2) because the first catch
wasn't generalized into "grep `elab_instance_decl` on the literal instance head
every time an instance is claimed buildable."

**Sharpening 2 — per-role axis enumeration in multi-role WPs (CAT-1,
2026-07-04).** In a multi-role WP, axis-enumeration is *per role*. The
core-owner's grounding covers only the axes **they** own; an assisting role's
slice can exercise a distinct axis the core-owner's ruling never touched (a
value-class instance over a parametric carrier hitting a different
`UnresolvedCon` than the higher-kinded-class-parameter question the core owner
ruled on). Do not treat a core-owner's axis-grounding as exhaustive for your own
slice — enumerate and build-probe the axes *your* deliverable touches, even ones
adjacent to the axis already under debate, and surface any new axis as its own
distinct, grounded finding.

**The fix-vector dual: the same failure recurs when ruling on how to *fix* a
reported wall, not just when classifying buildability.** A code-site read that
confirms a gap is *present* never confirms it is *operative* — a fix vector
derived purely from reading the site can be provably inert if the actual failure
runs through a different mechanism. Before ruling any fix-vector for a reported
wall: (1) reproduce the wall on the *exact* base it was hit on (including
scaffolding/helpers — a bare reconstruction on clean main can silently fail to
trigger it); (2) confirm the proposed fix isolation-flips it (red→green under
only that fix, run by you, not left as a downstream gate); (3) probe for an
explicit-idiom workaround that needs no fix at all. A wall *reproducing* is
necessary but not sufficient to trust a site-read fix vector either — the vector
can target a mechanism the bug doesn't actually run through.

**The construct-avoidance corollary.** A reproduced, re-spelling-invariant "real
gap" still needs a *structure-avoidance* test before it earns a fix WP:
"invariant to N re-spellings" proves the error is structural (not a
spelling/witness-shape bug) — it does **not** prove the structure itself is
required. Check whether the whole construct can be routed around (helper lemmas,
tail-peel, a single telescopic motive) before framing a WP that touches the
elaborator/TCB. A route-around still needs its own check: verify the
restructured construct's top-level signature is byte-identical to the one
required (same hypotheses, same conclusion) — a purely-internal weakened helper
is fine only if the law-interface signature is unchanged.

**Third-party reports need the same discipline, decomposed.** When a fix vector
comes from a research agent or literature report rather than your own site-read,
adopt the report's literature/architecture framing (it maps the design space)
but treat every specific code-mechanism or fix-location claim as an *ungrounded*
site-read — grep the named site and isolation-flip the vector yourself before
committing, exactly as if you'd written it. A report's citation-authority makes
a wrong code claim more seductive, not less.

**When a misdiagnosis is found, freeze siblings built on the same
unreproduced-report path** until each is independently reproduced with its
actual probe on the base that carries the scaffolding — one confirmed
misdiagnosis makes structurally-similar siblings suspect, not presumed-real.
