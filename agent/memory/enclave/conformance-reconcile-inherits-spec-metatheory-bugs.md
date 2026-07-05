---
scope: enclave
audience: (see scope README)
source: private memory `conformance-reconcile-inherits-spec-metatheory-bugs`
---

# Content-reconciling a conformance case inherits the spec's metatheory bugs

Content-verified reconcile (re-read the landed §-body, not the heading —
conformance oracle grounding fallback) guarantees a conformance case **matches**
the spec. It does **not** guarantee the spec is **right**. When the spec's
metatheory is wrong, a faithfully-reconciled case **inherits the bug** — and a
green corpus then licenses the bug fleet-wide.

**K1.5 (2026-06-29, Architect-caught, dec_23t36t2ddt98k).** I authored
`wstyle-inner-elim-stuck-under-binder` asserting the inner W-style eliminator is
"stuck under the binder." It's a false metatheory claim (the inner elim fires
for a constructor-producing `k`; decidability is by finiteness — see eliminator
termination finiteness not stuckness). The landed §7.7/§9.4 said "stuck," I
content-reconciled against it **faithfully**, and so **encoded the same false
property**. The load-bearing ★★★ trust-root Architect review caught it;
spec-author reground the spec on finiteness and I replaced the one false case
with two correctly-scoped ones. The point *for me*: faithful reconcile is
exactly what let the bug through — matching a wrong spec produces a wrong case.

**Two standing gates this promotes (both were catchable on my side):**

1. **Independently re-derive every structural claim lifted from the spec.** A
   reduction outcome, a "stuck / neutral / fires" claim, a termination basis,
   "the head is/ isn't X" — re-derive it from first principles, do **not** just
   transcribe the spec's prose into an `expect:`. Especially **absence/negative
   claims** ("X is stuck" = "*no* reduction fires"), which are the easy ones to
   over-claim. Ask the **disconfirming** question: here, *"is `k b`'s head
   actually `b`-dependent?"* — it isn't for constructor-producing `k`, so
   "stuck" was false. This is the independent-checker duty that
   content-reconcile alone does not discharge (the reviewer is never just the
   transcriber).

2. **Run an internal-consistency pass over the whole seed file before handoff.**
   Do any two cases assert **contradictory behavior on overlapping inputs**? My
   `wstyle-iota-in-conversion` ("a constructor head always fires ι") and
   `wstyle-inner-elim-stuck-under-binder` ("the inner elim on `k b` is stuck")
   contradicted each other on a constructor-producing `k` — visible by reading
   my own file as a whole, no Architect needed. Add it as a pre-handoff
   checklist item alongside the verdict-flip trace (discriminating conformance
   verdict must flip). (K2c-series-2 confirmed this catches *stale sibling
   notes* too: a placeholder "these seams MUST stay sound-stuck" note in a
   neighbouring seed file directly contradicted the new *reducing* cases —
   retire/flip it as part of the same WP.) **Sharpen it to
   *mechanism*-consistency, not just *input*-consistency (V2, 2026-06-30,
   spec-author-caught twice).** When several cases exercise the **same**
   mechanism (postcondition extraction, a reduction rule, an admission gate),
   verify they agree on its **shape** across the parameter that varies —
   straight-line vs branchy vs recursive body, constant vs dependent motive,
   etc. My V2 A2 (`abs`, conditional body) expected a *single* over-the-body
   postcondition obligation while C/D1 (conditional/recursive) expected
   *per-branch* — **unsatisfiable together** (a single over-the-body obligation
   carries no IH, so it can't verify recursion), yet a per-case input/output
   scan passed each individually. The contradiction is only visible by grouping
   cases **by mechanism** and checking the shared mechanism's shape is
   consistent across the varying parameter. (This is the 2nd V2-lane miss
   spec-author caught — the first was the V1→V2 hole-count case counting a
   refined *parameter* as both a hole and a Γ-hypothesis. Same class: a
   within/across-case contradiction my input-by-input pass slid past.) **3rd
   recurrence — verdict-consistency across overlapping *metatheory* (V3,
   2026-06-30, spec-author-caught in Spec review).** My V3 prover seed: A3
   (`¬¬p⇒p`, classically-valid-topos-invalid → **unknown**) and D2 (`p∨¬p`,
   classically-valid-intuit-invalid → **disproved**) gave **opposite verdicts on
   the same metatheory class**. **The per-case verdict-flip check does NOT
   subsume the cross-case sweep** — I *ran* the flip (each case flips correctly
   in isolation), but the flip validates a case **alone**; only grouping cases
   by **shared metatheory class** and asserting **verdict-agreement** across the
   group collides A3 with D2. Run **both** as hard pre-handoff gates: flip (per
   case) **and** sweep (across the class). The reusable invariant that makes
   A3≡D2 obvious: **a classically-valid goal is never `disproved`** — by
   Glivenko (`¬¬φ` intuitionistically provable iff `φ` classically provable ⇒
   `¬φ` unprovable ⇒ **non-refutable**), it is `proved` if intuitionistically
   valid, else `unknown`. Any conformance case mapping a classically-valid goal
   to `disproved` is wrong (a V4 / prover-IPC invariant). Assembly-integrity
   gates (content-drift + diff-scope) were clean both assemblies — this was pure
   **content**-correctness, the axis only the independent re-derivation + the
   sweep catch. **V4 confirmation — apply the prior WP's trap-class as
   pre-authoring lock-point #1 (2026-06-30, shipped clean first-pass).**
   V4-diagnostics hit the **same false/unknown boundary** as V3's D2, but I
   opened the WP by grounding that exact boundary as a **lock-point to the
   spec-author** (`LP-2`: key the diagnostic tag to V3's *verdict*, not an
   independent V4 re-derivation) — so the same bug-class was pinned at the
   **source**, **zero rework** (vs V3's HOLD→re-fold). The sweep is **strongest
   applied at design time** (as a lock-point), not just pre-handoff: 15/15
   first-pass, no Spec-review content catch. **Reusable false-side complement of
   Glivenko:** the `false`/refuted exemplar must be **genuinely refutable**
   (`¬φ` provable, e.g. `p∧¬p` where `¬(p∧¬p)` is a theorem); an **abstract atom
   `p` is `unknown`, not `false`** (`¬p` unprovable ⇒ no world forces `¬φ`) — a
   `p`→`false` case is both **wrong** and **vacuous** (non-`proved` either way).
   Pairs with "classically-valid is never `disproved`" as the two boundary
   invariants of the verdict trichotomy. **T1 (2026-06-30, `1e9448c`): the
   design-time-lock-point protocol generalized beyond metatheory — held 2/2
   (V4→T1) on a *different* trap-class.** T1's trap was **normative-scope**, not
   a verdict boundary (the spec locks a value-set but defers the literal field
   spelling — over-freezing it blocks valid impls); the same pre-authoring
   lock-point (LP-1) closed it, 16/16 first-pass. So the protocol "surface the
   prior/anticipated trap-class as a lock-point before authoring" is **not
   specific to metatheory bugs** — it front-loads correctness for any axis the
   spec under-determines. **Sec1 (2026-06-30, `e4b8837`): held 3/3 (V4→T1→Sec1)
   AND generalized into a *new domain* — relational/2-safety security, not the
   unary verification spine.** The 3 LPs (stage-split observable, honest-limits
   defer-tags, no-laundering absence-gate) **survived an in-flight spec
   sharpening** — the Architect's N1/N2 folded into `61` *under* the seed (after
   my lock-points, before merge) without breaking any of them, because LP-1/2
   had already pinned the exact boundary N1/N2 sharpened. The design-time pin is
   **robust to the spec moving beneath it**. See conformance assert at locked
   granularity and the trusted-layer-enumeration carry in untrusted layer
   backstop hole for omissions.

**The absence-assertion gate (confirmed across K1.5 + K2c-series-2's cast/`J`/
respect — 5 instances).** The highest-risk cases are reliably the
**negative/absence assertions** — "stays stuck", "stays neutral", "rejected". A
*positive* reduction case self-verifies (it computes the asserted value or it
doesn't); an absence case passes **vacuously** when the impl is stuck/rejecting
*coincidentally* — for a different reason than the one the case claims to pin.
So every stuck/neutral/rejected case must (a) **name the exact guard/gate
condition** that makes it absent (e.g. "stuck *because* the §3.2
canonical-index-decomposition gate can't fire on a neutral index" — not just
"stuck"), and (b) pass the disconfirming check *"would this also be
stuck/rejected if the impl had the precise bug this seam targets?"* — i.e. is
the absence **guard-gated** (genuine) or **coincidental** (vacuous). This is the
unifying rule over finiteness-not- stuckness (eliminator termination finiteness
not stuckness) and verdict-must- flip for the whole absence family.

**Why:** a wrong conformance case is the highest-leverage error in the project —
it codifies a bug as the contract. Matching the spec is necessary but not
sufficient; the corpus is *independent* of the spec precisely so it can catch a
spec that is internally wrong. Extends trust root test coverage discipline (a
green corpus is evidence only if each case pins a *true* property) and the
content-reconcile rule of conformance oracle grounding fallback (read the body —
*and now* re-derive the body's claims, don't just match them).

**How to apply:** before handing a conformance seed to the spec-leader, for each
case that asserts a reduction/normalization/stuckness/termination property: (a)
re-derive the claimed outcome on a concrete witness from first principles,
asking what would falsify it; (b) scan the file for any sibling case whose
behavior contradicts it on a shared input class. Fix or narrow before handoff.
The trust-root Architect review is the backstop, not the first line.

**Tooling caveat for the stale-sibling sweep (CT-D1 erratum, `a06b721`).** When
the internal-consistency / "reconcile every site" pass is driven by a **grep**,
a silent **shell-quoting error** (e.g. backtick-`` `Q` `` unquoted) returns
**empty stdout** — which reads as *"no matches / clean,"* not *"the search
broke."* So a partial sweep looks complete. On the CT-D1 erratum my first grep
under-reported (4 of 9 sites); had I trusted it, the case would have asserted
`P`/`tested` while 6 preamble sites still said `Q` (a self-contradicting file).
**Rule: when a completeness grep returns FEWER hits than the concept's
prevalence implies, suspect the tool, not the corpus** — re-run with a varied
pattern (and read the file region) before trusting "all sites reconciled." A
thin hit-count on a pervasive concept is itself the signal.
