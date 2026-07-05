---
scope: enclave
audience: (see scope README)
source: private memory `proof-relevant-inductive-cannot-be-declared-at-omega`
---

# A proof-relevant multi-constructor inductive cannot live directly at Omega

When a spec defines a **predicate**/relation as a `data P : … : Ω` **inductive
with multiple constructors**, check whether it is **proof-relevant** (distinct
derivations are genuinely distinct proofs — a permutation relation
`perm_refl|swap|trans|cons`, a reachability/transitive-closure relation, `∨`,
`∃`). If so, **declaring it directly at Ω is inadmissible and unsound.** Ken's Ω
is **definitionally proof-irrelevant SProp**: `16 §1.3` — *"only genuine
sub-singleton types may enter the strict-prop universe; an unrestricted
`Type → Ω` would admit `Bool`, making `true ≡ false` by Ω-PI and breaking
consistency."* Ω-PI would collapse the inductive's distinct constructors to
definitionally-equal — the consistency break. Ken provides the proof-relevant
connectives in Ω **only via truncation** (`16 §6`: `∨ := ‖P + Q‖`, `∃ := ‖Σ‖`).

**So a proof-relevant relation reaches Ω only two ways:**
1. **Truncation** — `P xs ys := ‖ P_rel xs ys ‖`, where `P_rel : … : Type` is
   the proof-relevant inductive at **Type**, squashed into Ω (`16 §6`; the
   direct `∃ := ‖Σ‖` analog). Keeps the inductive form.
2. **Natively-Ω form** — build it from Ω-connectives with **no** proof-relevant
   choice: e.g. count-equality
   `Perm xs ys := Π x. Eq Nat (count x xs)(count x ys)` (a Π into Ω, `Eq : Ω`) —
   needs `DecEq`/`count`.

A **structural recursion** returning Ω-connectives (`∧`/`∨`-truncated/`Eq`) is
fine — it's not a multi-constructor inductive (e.g.
`isSorted (x::y::r) = (x≤y) ∧ isSorted (y::r)` lands in Ω, *provided* `≤` is
Ω-valued — a `Bool`-valued order leaks it to `Type`; bridge with
`IsTrue (leq a b):Ω`).

**Live (ES1 `§37 §6`, 2026-07-01):** the flagship verified-`sort` refinement
`{ys | isSorted ys ∧ Perm ys xs}` specified `Perm` as
`data Perm : … : Ω := perm_refl|perm_swap|perm_trans|perm_cons` — a
4-constructor proof-relevant inductive **directly at Ω**. Inadmissible: the
kernel rejects it (or Ω-PI breaks consistency), so the headline proof doesn't
type-check. My **CV-Spec blocking finding** — caught by **re-deriving the
Ω-sorts against `16`**, not reconciling the prose. `isSorted` was fine
(recursion over `∧`); only `Perm` (the inductive relation) hit it. Fix:
`‖Perm_rel‖` truncation or count-equality.

**A DECIDABLE operation lets a would-be-proof-relevant law dodge truncation
entirely — state it as a Bool-equation (ES4-classes `51`, 2026-07-01).** The
truncation is only *forced* when the law is a **bare propositional** `∨`/`∃`
(which side / which witness is content). If the underlying operation is
**decidable** (returns `Bool`), state the law as a **Bool-equation** and the
proof-relevance vanishes at the value level. Canonical case: `Ord`'s
**totality** `x ≤ y ∨ y ≤ x` — as a propositional
`IsTrue (leq x y) ∨ IsTrue (leq y x)` it is proof-relevant → needs `‖·‖`; but
with the decidable `leq : a→a→Bool` it states as
`total : IsTrue (leq x y || leq y x) = Eq Bool (leq x y || leq y x) True : Ω` —
**proof-irrelevant, no truncation** (the value-level `||` collapses the "which
side" content before it ever becomes a proof). General rule: **a decidable op
lets every law be a `Bool`-equation / `Dec` form → Ω-clean; the truncation
obligation bites only a law stated as a bare propositional `∨`/`∃`.** So the
design order is: prefer the decidable-operation form (avoids truncation), fall
to truncation only for a genuinely-non-decidable relation. Both the Architect
and CV independently corroborated this on `Ord`'s `total`; reusable for every
lawful class over a decidable op (`DecEq`'s `sound`/`complete`, any future one).

**RANGE-REFINEMENT corollary — a decidable-range refinement `{ x : Int | P x }`
must encode `P` as `IsTrue (Pbool x)`, NEVER as a raw range-`∨` (BUILTINS
registry T1, `18a §5.9`, 2026-07-02).** When a carrier is a *refinement* of an
Int-like base by a decidable predicate — `Char := { c : Int | isScalar c }` with
`isScalar c := (0 ≤ c ≤ 0xD7FF) ∨ (0xE000 ≤ c ≤ 0x10FFFF)` — writing the
predicate as a **raw disjunction `A ∨ B : Ω` is the forbidden direction**: a
disjunction *is* the sum `A + B`, a two-constructor **proof-relevant** type that
cannot sit at Ω (the `Bool → Ω` trap). **Range-DISJOINTNESS does NOT rescue it**
— at most one summand is *inhabited*, but the *type* `A + B` still carries the
injection tag, so it stays relevant (a subtle over-trust: "the ranges don't
overlap so it's basically a subsingleton" is WRONG). The Ω-admissible encoding
is the **Bool-decidable reflection `P x := IsTrue (Pbool x)`**
(`Pbool : Base → Bool` computes the range test with `≤?`/`&&`/`||`; `IsTrue b`
is a genuine subsingleton → proof-irrelevant → Ω), or explicit truncation
`‖A+B‖`. This is **load-bearing, not cosmetic**: the refinement's payoff — Ω-PI
collapsing `{x|P x}`-equality to **base equality** (two `Char`s, same codepoint,
distinct-looking scalar proofs, equal by Ω-PI → **zero-delta `DecEq Char`**) —
holds ONLY if `P` is *actually* proof-irrelevant, which the naive-`∨` is not
(and forcing `A+B` into Ω re-opens the `Bool→Ω` inconsistency). So a
refinement-carrier demoting its ops over the base projection (the
`Char`-over-`Int` / `Decimal`-over-`(coeff,exp)` un-gating) is sound **iff** its
predicate is the `IsTrue` form. Same `IsTrue`-not-`∨` discipline as `Ord.total`
above, one level up (a refinement predicate, not a class law). I wrote the `∨`
form in the registry; both Architect (T1) and CV caught it *with my own Ω-sort
memory* — the encoding, not just the sort, is the AC.

**META (BUILTINS, 2026-07-02): run your OWN authored discipline exhaustively on
your own rows — the adversarial default only bites if run to conclusion.** In
one WP I under-applied three disciplines I had *authored into the same doc*,
each caught by the enclave, each improving the audit: (1) **trust-level** — I
over-classified an evaluator wrong-value (`Decimal` saturate) as a "false kernel
proof" without grepping the emission (kernel `Eq`-at-primitive is neutral, no
`eq→Eq` bridge → nothing transports); (2) **cross-AC derivability (§4.1)** — I
filed `checked_*`/`saturating_*` NATIVE without running my own "derivable given
the *other* ratified ACs" pass (they derive over the conversion set + bignum
`Int`); (3) **Ω-sort** — the range-`∨` above. Tell: the row you *don't*
scrutinize is the one where your own rule already gives the answer. When you
write a soundness rule into a doc, sweep every row against it before surfacing —
authoring the discipline is not applying it. **Fourth angle on the same F4 miss
— a RECEIVED severity ESCALATION deserves the same grep-the-emission
re-derivation as any trust-level claim, MORE so because escalation is the
over-claim direction.** F4 was three-deep: the Architect *escalated* the
`Decimal`-saturate wrong-value to "false kernel proof / `refl`-inhabits
`Eq Decimal` / explosion," I *baked* it into the registry, CV *concurred* — and
one Steward grep of the emission (kernel `Eq`-at-primitive neutral, no `eq→Eq`
reflection lemma, no `ken-interp` dep) caught all three. I re-derived it only
*after* the Steward flagged it; I should have re-derived it *when I inherited
it*, precisely because it RAISED severity to a kernel emergency. Inheriting a
peer's soundness escalation on trust is exactly where over-classification
propagates across gates — re-ground a severity-*raise* at adoption, never
inherit it. The conjunction of independent gates is what caught it (no gate
individually clean); that symmetry is the design working, architect gate can be
skipped review on main + trust level prose vs locked adr crosscheck applied to a
received escalation.

**Fifth angle, DELIVERY-CONTRACT face (WP F1 elaboration, `18a §5.2.1`,
2026-07-02): a prose contract that ELABORATES an existing table can over-claim
what a row DEFERS — sweep the contract against every row's current-state +
verdict column before surfacing.** Authoring the iff-bignum delivery contract, I
grouped `eq_int`/`leq_int` as F1 reductions — but my OWN `18a §5.2` `leq_int`
row three sections up says GAP (F5, registered/unreduced), NATIVE iff arm added:
F1 delivers only the *representation*, not the reduce arm. CV's independent
Spec-vote grep (`eval.rs:1339` arity list has no `leq_int`) caught it. A
native-vs-derived TABLE and its prose ELABORATION are two representations of one
boundary and drift exactly like a conformance corpus drifts from the spec — so
run verdict mapping silence is a latent conformance bug's cross-case consistency
sweep *inward*, across my own doc's sections, not only across the corpus. Same
"run-own-discipline-exhaustively" root as the three above, pointed at intra-doc
cross-section consistency. (Also that WP: grounding the CRATE surface against
landed code, not the brief's perishable anchors, surfaced that `to_rt` has no
`BigInt` arm → the store round-trip is a contract F1 ESTABLISHES-not-preserves →
reshaped AC3 to drive the real producer, dodging conformance hand feeds the
deliverable.)

**The SORT axis has a sibling: the CARRIER-PROVABILITY axis (ES4-classes
erratum, `51 §6`, 2026-07-01).** A **zero-delta lawful instance** (real law
proofs, nothing postulated) needs **BOTH** orthogonal axes: (1) the law's
**sort** — Ω-clean via a decidable op (above); (2) the carrier's **provability**
— the carrier must be **INDUCTIVE (have an eliminator)**, because the law fields
are ∀-quantified (`∀ x. IsTrue (leq x x)`) and are proved **by case-analysis/
induction on the carrier**. `Bool` (real `data`) proves every law by finite
case-split → zero delta. A **primitive** carrier (`Int`/`Float`/`String`/`Char`
— opaque to δ, `int_leq x x` on a *variable* doesn't reduce, and NO induction
principle) **can't prove its ∀-laws** → the only inhabitant is a `postulate` →
`Opaque` → **non-empty `trusted_base_delta`**. So a primitive carrier's lawful
instance is **NOT zero-delta**: it ships as an **audited-delta** (laws
postulated but *structurally visible* in `trusted_base_delta` — the `Opaque`
entry the delta computation cannot miss — the honest FFI/primitive posture,
tested not trusted posture needs reachability precondition), or is deferred
until the primitive gains reduction rules + induction. `Ord Bool` clears both
axes; `Ord Int` clears sort, **fails carrier** — that was the §6 bug.

**A THIRD axis: CONCLUSION-SHAPE — which kernel capability discharges the law
(§51 §6 K5-refine, `16 §1.4`, 2026-07-02).** Even with both axes above cleared
(carrier inductive ✓, laws Ω-clean Bool-equations ✓), a "∀-laws provable by
case-split → zero-delta" claim must ALSO be checked at the **per-branch
obligation *shape***, because that determines *which kernel rule* closes each
branch — and the two capabilities landed as **separate WPs**:
- **Live-`Eq`-conclusion laws** (`refl`/`trans`/`total`, `Eq`'s equivalence
  laws): the per-branch obligation stays a **live `Eq`** (a deferred
  `IsTrue`/`bool_leq`) → discharged by **K4** (Ω-motive elimination of the
  general eliminator, `14 §3`). Zero-delta as soon as K4 lands.
- **Concrete-equality-conclusion laws** (`Ord.antisym`, `DecEq.sound`/
  `complete`): conclude/hypothesize a *concrete* kernel `Eq a x y` whose
  per-branch obligation **whnf-reduces to a concrete `Top`** (same-ctor nullary,
  e.g. `Eq Bool true true ⇝ Top`) or **`Bottom`** (distinct-ctor,
  `Eq Bool true false ⇝ Bottom`, `16 §2.2`). Closing those needs **`Top`-intro
  (`tt`)** and **`Bottom`-elim (`absurd`)** — the **observational fragment**,
  which K4 does NOT provide → gated on **K5** (`16 §1.4`: bare Ω_0 sub-singleton
  constants with direct intro/elim rules; sound because `Bottom` is empty ⇒
  ex-falso vacuous, `Top` a singleton via Ω-PI; typing-admission only,
  `eq_reduce` untouched; **distinct from the forbidden elim-*out*-of-Ω**).
So a *complete* zero-delta `Ord Bool`/`DecEq Bool` is **K5-gated** (`antisym`
mandatory for a total order); K4 alone realizes only the live-`Eq` fragment.
**The tell: an instance claimed complete-zero-delta whose laws touch a
*concrete* kernel `Eq`-conclusion, not just a deferred `IsTrue`.** My own §6
un-stage over-claimed exactly here (checked carrier + sort, not conclusion-
shape) — passed authoring + all four un-stage gates; the Architect caught it
post-merge. Carrier-inductive is necessary, NOT sufficient. The two-branch
`antisym` shape (equal → `tt`, contradictory → `absurd`) IS the K5 fragment.

**WHY concrete-`Top`/`Bottom` is a genuine runtime gate, not a spelling gate —
`Refl` needs a syntactic `Eq` (K5 Fidelity, Architect-grounded, 2026-07-02).** I
nearly mis-recorded this: I inferred a `Top`-collapsed goal `Eq Bool True True`
is reflexivity-provable pre-K5 (`refl True`), which would make the K5 capability
cases green-vs-green at the goal level (only the `tt` *symbol* new). **Wrong** —
Ken's `check.rs::Refl` requires the expected type to **whnf to a syntactic
`Term::Eq`**, but `whnf(Eq Bool True True)` eagerly reduces to `Top` (a `Const`,
`§2.2` same-nullary-ctor), so **`Refl` cannot fire** on the reduced-`Top` goal →
`tt` (`Top`-intro) is **strictly required**, not `Refl`-substitutable. So the
concrete-equality-conclusion laws are a genuine **runtime-provability** gate on
K5 (the goal has no pre-K5 inhabitant), not a canonical-proof/spelling gate. The
dual for the live side: `refl`/`trans`/`total` stay K4-live **precisely
because** their goals route through an *unresolved application*
(`bool_leq`/`bool_or` over abstract carriers) that keeps the `Eq` **live** (not
`Top`-collapsed), so `Refl` fires. THAT is the mechanism behind the live-`Eq` vs
concrete-equality boundary — grounded in `check.rs::Refl`, not prose. Lesson: a
"flip is only symbol-existence / green-vs-green" suspicion on a kernel
capability must be checked against the kernel's actual **admission gate** (here
`check.rs::Refl`'s syntactic-`Eq` requirement), not an armchair OTT model — my
model under-modeled `Refl` and the Architect's source read corrected it. The
Fidelity concern was still *right to raise* (it forced the attribution question
closed against source, option-2 of my ask); it just resolved against my
inference. See trusted by typing guarantee is not kernel proved Q on grounding
trust-level claims in the emission/gate, not the name/model.

**THREE distinct kernel gates for a complete lawful `Bool` instance — and the
un-stage-must-re-derive-per-obligation rule (K5 un-stage, 2026-07-02).** The
full gate mapping, discovered across the ES4-lawproofs build (each escalation
confirmed by the Architect, none a false alarm): **K4** (`Ω`-motive elim) →
`Ord`'s `refl`/`trans`/`total` + `Eq`'s `refl` (goals stay a *live* `Eq` via an
unresolved app like `bool_leq x x`/`bool_eq x x` so `Refl` fires); **K5**
(`Top`-intro/`Bottom`-elim) → `Ord.antisym` + `DecEq.sound`/`complete` (goals
collapse to concrete `Top`/`Bottom`); **K6** (`conv_struct` `Eq`-congruence,
still open) → `Eq`'s `sym`/`trans` (reuse a hypothesis across a missing
congruence arm — a THIRD axis, distinct from both live-`Eq` and `Top`/`Bottom`).
The lesson for the **un-stage** (capability X lands → lift its `(gated: X)`
tags): **re-derive per-obligation WHICH laws fall within X's power, branch by
branch — never name-match "X landed → un-gate everything."** "K5 landed"
un-gates *only* antisym/sound/complete; `Eq` sym/trans stay `(gated: K6)`, and a
§6 line that lumped "`Eq`'s equivalence laws: zero-delta now" was a latent
over-claim the build falsified — the un-stage is the moment to *correct* it, not
propagate it. Both the Architect ("`Eq Bool` fully zero-delta now") and I (the
§6 lump) had to walk back the pre-build optimism; the build's empirical
proof-construction is what pinned the real gate. Also: a "capability landed"
edit can REVEAL a new forward gate (K6) that must be *staged* even as another
(K5) un-stages — an un-stage and a re-stage in the same edit. See composition wp
real producer may be deferred engine on grep-the-real-producer; here the "real
producer" is the landed build's honest `Axiom` attribution (`72e38a5`), which is
ground truth for the gate-state.

**A FOURTH gate + a FOURTH axis — OPERAND-REDUCTION: the eliminator must
actually FIRE on the obligation's term shape (K7, `wp/es4-51-k7-erratum`,
2026-07-02).** Conclusion-shape (above) was necessary but **still not
sufficient**: I un-gated `antisym`/`sound`/`complete` onto K5 as "realizable
now" after checking they conclude a concrete `Top`/`Bottom` (K5's territory) and
don't need K6 — but I NEVER checked whether K5's `absurd` actually **fires**,
i.e. whether the contradictory-branch hypothesis genuinely *reduces* to
`Bottom`. It doesn't on the landed kernel: the hyp routes the carrier through
the instance's own op (`IsTrue (bool_leq True False)` =
`Eq Bool (bool_leq True False) True`), and `bool_leq True False` is a **redex**
— but `ken-kernel/src/obs.rs::eq_at_inductive` peels its operands with
`peel_app` on the **raw** terms and never `whnf`s them first (its sibling
`eq_at_type` does), so the head is `Const bool_leq` not a constructor → `Eq`
stays neutral → no `Bottom` → `absurd` can't fire. That gap is **K7** (a
two-line `whnf`-the-operands completeness fix in the trust root), distinct from
K5 AND K6. So the full gate map is FOUR: **K4** live-`Eq` / **K5**
`Top`/`Bottom`-intro-elim / **K6** `conv_struct` `Eq`-congruence (variable-stuck
operands) / **K7** `eq_at_inductive` operand-`whnf` (redex-wrapped operands).
The build (ES4-lawproofs-remainder) hit the K7 wall, stopped (AC3 discipline),
Architect ruled it a genuine kernel incompleteness. **The sharpened un-stage
rule: a per-obligation re-derivation must verify the gating rule's ELIMINATOR
ACTUALLY FIRES on the obligation's term shape — not merely that the
conclusion-shape matches the capability.** An `absurd`/Bottom-elim obligation
discharges only if the contradictory hypothesis genuinely *reduces* to `Bottom`,
and a hypothesis that wraps the carrier through the instance's own operation is
a **redex** whose reduction is a *separate* kernel-completeness gate
(operand-`whnf`). Tell: an un-stage reasoning "the obligation reduces to
`Top`/`Bottom`" without checking the operands are already `whnf`'d (or that the
kernel `whnf`s them). Twin finding: `16 §8.1`'s *algorithmic* Eq-reduction
recipe says "compute `whnf(A)`" for the TYPE but is silent on `whnf`ing the
OPERANDS before the constructor-head compare — the SAME missing step as my
re-derivation blind spot (a native/self- host port following §8.1 literally
reproduces K7). §2.2's *denotational* rule is whnf-relative by OTT convention
(faithful), so K7 is a one-line §8.1 clarity note + an impl fix, zero semantic
delta. My own §6 un-stage shipped the over-claim to main (`0feb2c8`),
self-caught post-merge against the K7 finding → erratum `wp/es4-51-k7-erratum`
re-attributes off "realizable now via K5" onto "K5 landed + K7 forward; park as
visible `Axiom` pending K7" (parallel to sym/trans pending K6).
Conclusion-shape-match is necessary, operand-reduction-fires is the further
necessary condition.

**Capability-gate PROSE has THREE states, and BOTH transitions go stale-on-
arrival — decouple + own-the-flip (K7 erratum close, 2026-07-02).** A parked
capability's spec/seed prose passes through **over-claim → interim-park →
realized**, and the middle state is intrinsically transient, so a "main-honesty"
erratum that lands it can itself be stale on arrival in *both* directions: (1)
**over-claim side** — "needs K7 (forward)/pending K7" goes false the moment K7
merges (K7 was landing *concurrently*). Fix = **decouple the capability axis
from the proof-wiring axis**: "needs the K5+K7 *kernel capability* (cite by
capability + spec §, e.g. `16 §8.1`) — park pending the *ES4-lawproofs-remainder
real-proof wiring*." Honest under either merge order. And **cite a squash-bound
gate by capability, NEVER its branch-tip SHA** — the Integrator squashes, so the
tip (`b7396ae`) is a phantom that never appears on main; only an already-merged
gate's real SHA (K5 `1c84a30`) is safe to name (CV's catch; sibling of check
main via git object store not find). (2) **under-claim side** — the honest "park
pending the wiring" goes stale the instant the wiring lands (`9a82745` wired
real antisym/sound/complete zero-delta proofs — "no `Axiom` remains" — the *same
window* as the erratum + K7). The build/wiring commit (Team Language) correctly
stays in its lane (`.ken` + tests, even fixes its own `.ken` doc comments) and
does NOT touch spec §6 / conformance seed — those are spec-author + CV. So the
**park→realized flip is a DUE, OWNED lifecycle step** that must be **pre-filed
and scheduled to land in the same window as the wiring commit**, or
verify-on-main finds a stale under-claim on main (I caught it as my post-merge
gate; spec-leader had already called the erratum "closed"). Playbook line: when
a build WP will *realize* a parked capability, pre-file the coupled spec/seed
park→realized flip so it merges with the wiring — don't leave it to be
discovered after. The over-claim erratum and the under-claim flip are the two
bookends of the same un-stage; both are spec-author's to own. Extends the
un-stage-must-re-derive-per-obligation rule above.

**Reviewer corollary — the interim-net keying check (#34 Fidelity,
2026-07-02).** When a staging mirror declares net Y `(gated: X)` while keeping
an *adjacent net* live to enforce the posture in the interim, the reviewer's
highest-value independent check is **which law keys the live net** — it must be
provable *entirely within the currently-landed capability*, with **no per-branch
obligation that reduces into the staged capability's territory**. A K4-live net
keyed on a law with a `Bottom`/`Top` branch (i.e. a concrete-`Eq`-conclusion law
like `antisym`) is a **hollow gate** — vacuously green during the interim,
enforcing nothing. Grep the keying law's per-branch obligation *shapes*, not its
tag. Live: CV keyed the audited-delta live net on `total` (always-true on Bool →
every branch reflexivity-provable, no `Bottom`) — genuinely live; had it been
`antisym` the posture would go unenforced until K5. Also a lane-scoping lesson
(scope review vote to my lane): a deep kernel-metatheory doubt that is
verdict-neutral AND in the soundness reviewer's lane (e.g. "does antisym's
*equal* branch strictly need `tt`, or is it reflexivity-provable pre-K5, making
`absurd` the sole strictly-required K5 op?") is NOT a fidelity blocker — note it
at most, don't rat-hole a mirror-fidelity vote on it.

**The trap that let it through (author-side, high-value):** the false "`Ord Int`
is zero-delta lawful" claim passed **authoring + all four review gates**
(CV-Spec, Architect soundness, my Fidelity, extension-confirmation) — because
the AC3 discriminating **corpus tested a *generic* inductive `K`** (realized
with `Bool`/user `data`), so the **specific primitive-carrier case was never
instantiated**. It surfaced only at **build** (producer-grep constructing the
real instance). **Rule: a "zero-delta / real-proofs" claim over a CONCRETE
carrier the spec names (`Int`) must be checked for THAT carrier's kind — a
property true for all *inductive* carriers can be false for the *specific
primitive* the example uses; the generic-`K` corpus won't catch it.** Tell: an
example instance over a carrier the discriminating corpus doesn't actually
instantiate. Sibling of conformance hand feeds the deliverable (the corpus
doesn't exercise the real case) — here the gap is *carrier-kind*, invisible to a
parametric test.

**How to apply:** for any `data … : Ω` in a spec/build, ask *"are the
constructors genuinely distinct proofs?"* If yes → it can't be at Ω directly;
require truncation, a natively-Ω reformulation, or (if the operation is
decidable) the Bool-equation form above. The Ω-sort analog of the Σ-sort trap
sigma sort pi vs sigma over equating (both are "wrong sort at the strict-prop
boundary → Ω-PI unsoundness"); sibling of trusted by typing guarantee is not
kernel proved Q (Ω/relevance discipline). The relevance-leak check the Architect
flagged, one level up (not just `Type`-vs-`Ω` for a *recursion*, but
*admissibility* of an *inductive* at Ω).
