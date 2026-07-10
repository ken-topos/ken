# WP DS-8c — Traversable composition coherence law (+ Compose `ap_cmp`)

**Owner:** Foundation team. **Steward-framed** from the DS-8 Core valve
(2026-07-10) — the composition law was SIZE-deferred out of DS-8, not
capability-blocked. Buildable immediately; independent follow-on to DS-8 Core
(`main @ 709c55d`, PR #440). **Outer-ring, no kernel/spec/elaborator-src touch.**

## Goal

Close the two pieces DS-8 Core honestly deferred, completing the `Traversable`
showcase for `spec/50-stdlib/56-effectful-classes.md §5.3`:

1. **`Compose g h`'s `ap_cmp`** — the 4th Applicative law of the Compose
   applicative. DS-8 landed its other three (`ap_id`/`ap_hom`/`ap_ich`) +
   `map_coh` + the Functor laws + the `ap_naturality` aux + the `ap_cmp` **LHS
   reductions only** (partial, honestly marked in `EffectfulClasses.ken.md`).
   DS-8c finishes `ap_cmp`.
2. **The `traverse` composition coherence law** (§5.3) — stated over
   `Compose g h`, which *consumes* Compose's `ap_cmp`. This is the load-bearing
   coherence law that makes the Traversable instance a complete lawful showcase.

## Scope

- Both deliverables land in **`catalog/packages/Core/EffectfulClasses.ken.md`**,
  continuing DS-8's construction mode. Critically: the Compose applicative there
  is a **`fn`-synonym / explicit dictionary-passing** construction
  (`compose_kleisli`, `functor_map_of`, `applicative_pure_of`, …), **NOT a real
  `instance Applicative (Compose g h)`** — the real higher-kinded instance head
  stays blocked by the parametric-instance-head kinding gap
  (`elab.rs:3833-3851`, DS-8's Architect finding). **DS-8c works entirely within
  the landed `fn`-synonym scaffolding — do NOT attempt the real instance;** that
  is a separate Language-team elaborator WP, not this one, and DS-8c needs zero
  new capability without it.
- **Size:** ~40–60 lemmas (this is why it was valved out of DS-8 — a SIZE
  trigger, converging, nothing walling). Pure pointwise `Ω`-clean value
  equations, structural throughout.
- **The implementer's recorded 4-stage closing plan** (the DS-8c spec, per
  Architect honesty pin #5) — the composition-law proof:
  1. rewrite `ψ5` pointwise via `aph.map_coh`;
  2. → triple-pointwise `aph.ap_cmp` via `eq_at_pi`;
  3. → lift through the 3 nested `apg` applications;
  4. → reconcile against the free RHS.
  Assemble `ap_cmp` first (it is what step 2 consumes), then the composition law.

## Boundary / constraints (Architect's 5 honesty pins carry from DS-8 Core)

1. **SIZE, not capability.** Frame every entry/commit note as *buildable-now,
   deferred-for-size* — never "gated"/"capability-blocked" (that is DS-5c, a
   genuinely capability-blocked sibling; do not conflate).
2. **TWO things deferred, scope both** — Compose's `ap_cmp` (1 of its 4
   Applicative laws) **and** the traverse composition law that consumes it.
3. **Scope the "lawful" claims to laws actually proved** — the entry states the
   Traversable showcase is complete only once *both* land; no "fully lawful"
   over-claim before then.
4. **No `Axiom`/`Refl`-papering** on `ap_cmp` or the composition law. The
   Architect **greps the tangled code at the gate** — a general `ap_cmp`/
   composition statement closed by a bare `Refl` (rather than the real
   `trans`/`cong`/`eq_at_pi` chain) is a REJECT. `Refl` is fine only where a
   concrete arm genuinely reduces (as the existing per-constructor Option/List
   `ap_cmp` accessors already do).
5. **The 4-stage plan above is the spec** — prove to it.

Plus the standing catalog bar: **zero new `Axiom`, zero `postulate`, zero
`trusted_base()` delta, no `sorry`** anywhere in the tangled fences; the proofs
are ordinary kernel-checked value equations. Outer-ring: **no `crates/**/src`,
no `ken-kernel`, no `Cargo.lock`** delta.

## Acceptance bar (fidelity = chapter 56 §5.3)

- **Compose `ap_cmp` proved** — the general statement closed by a real proof
  term (the `trans`/`cong`/`eq_at_pi` chain), not `Refl`/`Axiom`; the partial
  LHS-reduction markers DS-8 left are removed/completed, not left dangling.
- **Traverse composition law proved** — the §5.3 pointwise equation over
  `Compose g h`, discharged via the 4-stage plan, consuming Compose's `ap_cmp`.
- **Honest entry** — `EffectfulClasses.ken.md`'s design/Findings notes update
  the DS-8 landed/deferred split to "complete," scoped exactly to what is proved;
  no residual "deferred/partial" marker left for either piece.
- Home tests: extend the DS-8 acceptance suite
  (`crates/ken-elaborator/tests/` beside the DS-8 Traversable acceptance test) —
  the composition law and `ap_cmp` elaborate + type-check at the general
  statement. Targeted builds only (`-p <crate> <test>`); full-suite green proven
  in CI at merge.

## Gate

Foundation ring (foundation-leader → foundation-implementer → foundation-qa) →
**@architect fidelity gate** (chapter 56 §5.3 char-for-char on the law
statements + the tangled-code `Axiom`/`Refl`-paper grep, pins 1–5) →
`git_request` to Steward → **CI-gated** merge (real catalog code + acceptance
tests, not doc-only). Outer-ring, no soundness urgency. Own retro; flag every
judgment call. No WP-token identifiers in production/tangled source.
