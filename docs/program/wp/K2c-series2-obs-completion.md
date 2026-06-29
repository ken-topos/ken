# WP K2c-series-2 — Observational-reduction completeness (the 3 deferred seams)

> **Status:** Steward frame — **awaiting spec-leader elaboration** (queued in the
> spec chain after X1-spec; both behind L5). spec-leader + Architect first **rule
> the scope** (these are conversion-adjacent — fold near `17`/`16`), then
> spec-author elaborates the three seams' exact reduction rules to rigor, then the
> **Kernel team** builds.
>
> **Team:** Kernel · **Deps:** K2 (done), K2c series-1 (done, `7d38b55`) · **Size:**
> M · **Risk:** ★★★ (**trust root** — kernel reduction; seam 2 is "the hard OTT
> core") · ► On the **G1 critical path** via K-api (K-api wants full K2c).

## Objective

Complete the **three observational-reduction seams** K2's build deferred. Each is
**sound today** — it falls back to a stuck/neutral term, never a wrong result —
but **incomplete**: some `cast`/`J`/quotient computations get *stuck* where the
observational theory says they should *reduce*. Series-2 makes them reduce, with
the same soundness bar series-1 met (the Architect's adversarial review + the
kernel conformance corpus).

## The framing that sets the risk level

This is the **trusted kernel**. A bug here is not a rejected program — it is a
**wrong reduction inside the conversion checker**, i.e. a potential unsoundness
(two distinct terms made convertible, or a non-canonical result accepted). ★★★,
the same bar as K2/K2c series-1. The discipline that caught every prior kernel
soundness bug applies in force: **exercise the property, not the obvious case**
(≥2 distinct levels, open/dependent telescopes, eliminators that use the IH); a
deferred/partial check must **gate the reduction** (an un-invoked check while the
redex fires unconditionally is an unsound *accept*, not a sound stuck fallback —
the K2 closed-`Empty` lesson); and add the **adversarial** test the seam would
mis-accept. Conformance cases must be **discriminating** (verdict-flips, or a
structural assertion on the reduct) — no vacuous reject.

## The three seams (from K2's carry-forward, `K2c-conversion.md`)

1. **`cast`-at-inductive index rewrite.** `cast_at_inductive` rebuilds the
   constructor but keeps the family-index *value*, wrapping in `Cast` when the
   index changes (the suc-injectivity / index-equality seam): casting
   `Vec A n → Vec A m` of `vcons n a xs` currently leaves the index `n`. Complete
   the index-rewrite so the cast computes through to the `m`-indexed form per the
   observational `cast` rules (`16 §9`), wrapping sub-casts where the index proof
   demands.
2. **Non-constant-motive `J`-on-non-`refl`** — *the hard OTT core.* `J` reduces
   for constant motives (the headline) and is left **neutral** otherwise; the
   `cong`/`sym` sub-equality construction for a non-constant motive is unfinished.
   Complete it: build the transported sub-equality so `J` computes on a non-`refl`
   `Eq` proof at a dependent motive, per the OTT `Eq`-by-type semantics.
3. **Full quotient `respect`.** `check_respect` raw-well-forms the respect proof
   for **non-Ω** targets (an inline "soundness TODO") rather than verifying the
   full `cong`/`cast` schema. Complete the schema check. (Ω-target cases are
   respect-free per `16 §5` and are already correct — **do not** regress them; and
   recall the **Ω-element-vs-proof** discipline: proof-irrelevance fires on
   `typeOf(A)=Ω_l`, never on `A=Ω_l`.)

## The elaboration this needs (spec-leader → spec-author + Architect)

`16`/`17` state the observational computations normatively, but these three
reductions were deferred as "sound stuck" — their **exact reduction rules** at
the hard cases (index rewrite, non-constant-motive transport, the respect schema)
need elaborating to builder rigor, with the soundness argument for each:
1. The precise rewrite/transport rule and **where the guard sits** (the reduction
   must not fire until the index/respect condition is checked — gate, don't defer).
2. The decidability/termination obligation (these feed conversion, which must
   still halt — reconcile with the series-1 SCT gate).
3. Conformance seeds (`conformance/kernel/conversion/` + `observational/`) that
   are **discriminating**: a correct reduction vs. the exact mis-reduction the
   seam risks must reach **different** observable results (a computed value vs.
   neutral/`Err`), per the verdict-flip discipline. Include the adversarial
   "would this seam inhabit `Empty`?" case for each.

## Acceptance (testable)

1. **`cast` completes:** `cast (Vec A n) (Vec A m) p (vcons n a xs)` reduces to the
   `m`-indexed constructor form (index rewritten, sub-casts where required) — not
   left stuck; and an **ill-justified** index cast stays **neutral/`Err`** (the
   adversarial guard).
2. **`J` computes at a dependent motive:** `J` on a non-`refl` `Eq` with a
   non-constant motive reduces to the transported value; the constant-motive
   headline is unchanged; a malformed transport does **not** reduce.
3. **Quotient `respect` is fully verified:** a non-Ω quotient elim type-checks +
   computes **only** with a valid `cong`/`cast` respect proof; an invalid respect
   proof is **rejected** (the K2 closed-`Empty`-class adversarial case); Ω-target
   cases unchanged.
4. **Conversion still decides + terminates:** the completed reductions preserve
   the series-1 decidability/SCT guarantee — type-checking halts yes/no on the
   corpus.
5. **No regression:** K1/K2/K2c-series-1 suites stay green; the `[K2c]`-tagged
   carry-forward seeds (cast-computes-inductive, eq-inductive-dependent) now
   **reduce** instead of staying stuck.

## Sequencing

Queued **after X1-spec** in the enclave (both behind L5). Unblocks **K-api**
(kernel judgment + public API), which wants full K2c → feeds **G1**. Kernel is
idle and ready. Scope/fold ruling (own WP vs. into K-api) is the Architect +
spec-leader's first step. Build queries: reduction soundness/design → Architect;
behavioral contract → Spec.
