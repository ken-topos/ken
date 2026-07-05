---
scope: teams/kernel
audience: (see scope README)
source: private memory `sct-completeness-fix-conservative-construction`
---

# An SCT completeness fix must use conservative construction

**The gate (Ken `sct-completeness` (a) / VAL2 #12, 2026-07-03,
`evt_1fwtnnzrmhjm8`).** A completeness fix to `crates/ken-kernel/src/sct.rs`
(trust root) so it accepts a nested-sub-pattern-split + flat-sibling recursion
the old construction over-rejected. My up-front approach-review + soundness
gate.

**The frame that makes it gateable.** SCT is sound because *every idempotent
self-loop matrix in the composition-set closure has a strict `↓` diagonal* ⟹
every δ-reentry has an infinitely-descending thread ⟹ termination. Completeness
≠ moving the accepting RULE; it = the **matrix construction**
(`enter_method`/`collect_calls`/ `size_rel`) recording more TRUE `Down`/`DownEq`
and fewer `Unknown`s, so more true strict diagonals survive composition. So the
gate splits cleanly:
- **Criterion sacred (grep the diff):** `has_strict_diagonal`'s strict-`↓`
  requirement, the *distinct-triple* composition closure (no union-masking — my
  own K2c counterexample), and `compose_ord` stay **verbatim**; no new
  early-accept / `edges.is_empty()⇒accept` path (sct unapplied self reference
  over accepts).
- **The sole unsafe-conversion vector:** the construction **over-claiming** a
  size relation that isn't provably true. A bogus `Down`/`DownEq` → false strict
  diagonal → admits a divergent program → closed inhabitant of `Bottom`. SCT is
  the *only* termination gate (no conversion-side backstop), so this is a real
  hole, not a wrong-value. **Error-direction is the load-bearing tell:**
  under-record (`?` where a true `Down` exists) = over-reject = SAFE;
  over-record = unsafe. Demand a **per-relation TRUTH argument** for every new
  `Down`/`DownEq` the construction learns to emit — "it makes the target program
  accept" is NOT a truth argument.

**Two mechanisms, two truth-argument shapes (the #12 vs Ackermann split I
ruled).**
- **(a) #12 = a THREADING realignment, no new relation kind.** The post-#5
  match-compiler binds only the split field then defers the siblings/IHs into
  each nested branch; the old `enter_method` (fixed `n_fields+n_ihs` peel)
  mis-aligned. The fix threads the *already-proven* flat-case
  `field_prov=(root,Down)`/IH-`None` rule through the nested-Elim boundary via a
  continuation queue. Truth: a deferred field is a *direct field* of the
  scrutinee → genuinely `< root`, independent of nesting depth. Safe to approve
  as a *strict generalization* (empty continuation = byte-identical old
  behavior).
- **(b) Ackermann = a genuinely NEW mechanism (deferred to its own WP).** A
  reconstruction `C(v0..vn-1)` of a matched scrutinee is its ι-reduct →
  definitionally EQUAL → `DownEq`, **never `Down`** (equality doesn't shrink);
  the exact-raw-fields condition excludes the Up case (`Suc(Suc m2)` →
  `Unknown`). New provenance state + its own from-scratch soundness argument ⟹
  isolate to its own gate (bundling risks (a)'s mechanical greenness masking
  (b)'s subtle hole — two arm producer needs a case per arm /
  one-trust-root-argument-per-gate).

**Two hard verification moves that earned the APPROVE (do these, don't trust the
report):**
1. **Match the IH-count PRODUCER, not a proxy.** The fix re-derived "recursive
   field" locally (`is_recursive_field`) because
   `ConstructorDecl.recursive_positions` is never populated (`declare_inductive`
   always `Vec::new()`). The doc said "mirrors `eval::is_recursive_arg`" — but
   that simpler proxy LACKS the `Pi` arm; the true producer is
   `ken_kernel::inductive::recursive_args` (peels `Π` for W-style), which the
   match-compiler actually uses to build the IH binders. **Under-counting IHs is
   the unsafe direction** (continuation slides `field_prov` onto an IH slot →
   bogus `Down`). Verify the count matches the compiler's *own* producer
   (kernel-qa's sharpening — grep-the-producer-not-the-name).
2. **Run the discriminating tests myself + the isolation flip.** The sharpest is
   a single-self-loop tripwire (`recursing_on_a_deferred_ih_slot_stays_rejected`
   → REJECTS proves no IH gets a bogus `Down`), plus revert-only-`sct.rs` →
   accept-tests flip to `NotTerminating`, reject-tests still reject (clean
   isolation). I built a scratch worktree at the candidate and ran them rather
   than trusting workspace-green.

**Named residual invariant (honesty about the boundary):** the fix's safety
rests on a cross-component coupling — the construction's queue order must match
the match-compiler's actual binder layout at every depth. Documented, not
runtime- enforced. Acceptable because it's the SAME coupling the old code
already carried (a generalization, not a new class) and a per-binder type-check
defense would add more trust-root machinery than it removes. But NAME it + track
the forward-obligation: when the upstream match-compiler's deeper-nesting gap is
fixed, re-verify the actual compiled layout against the construction's
assumption. Sibling of kernel rejects is completeness fix is where soundness
converts (the completeness/ soundness framing this specializes to a size-change
checker).
