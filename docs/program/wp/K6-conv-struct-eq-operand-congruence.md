# WP K6 — `conv_struct` Eq-operand convergence (kernel completeness)

**Owner:** Kernel team. **Steward-framed** (2026-07-11), operator-directed
(forward-candidate #2 greenlit). **TRUST-ROOT / TCB change** — this edits
`ken-kernel/src/conv.rs`, the conversion checker. The **Architect designs the
algorithm + its soundness argument** (the front-loaded T1 judgment; this frame
does NOT specify the fix); the **CV cross-checks**; full kernel + elaborator
soundness gate, **not** outer-ring. Base: `origin/main` (re-verify `file:line`
at pickup — the kernel moves).

## What is ALREADY TRUE — verify, do NOT rebuild (grounded 2026-07-11)

The stale mental model this WP must NOT act on: *"conv_struct has no congruence
arm for `Eq` nodes."* **That arm has LANDED.** As of current `origin/main`:

- **`conv.rs:566` — positional `Eq`-congruence arm EXISTS** (landed via
  `obs-eq-termination`): `(Term::Eq(ty1,a1,b1), Term::Eq(ty2,a2,b2)) =>
  conv_struct ty1 ty2 && conv_struct a1 a2 && conv_struct b1 b2`. Two `Eq`
  *types* convert iff their three components convert, recursively. Sound,
  fail-closed (recognises strictly more true equalities, never a false one).
- **`conv.rs:419` — congruence-first fast path EXISTS:** same-`Const`-same-arity
  applications try argument-spine congruence *before* δ-unfolding the head.

So the K6 residual is **NOT a missing arm.** Any frame or commit note that says
"add the missing `Eq` congruence arm" is wrong and must be corrected — the
kernel-implementer must **read `conv.rs:404–579` first** and build on what is
there, not re-add it.

## The residual gap (the actual problem)

The documented failure (`LawfulClasses.ken.md §5`, findings §6 "PARKED K6"): a
transport proof needs `Eq Bool <neutral₁> True ≡ Eq Bool <neutral₂> True` where
`neutral₁`/`neutral₂` are **syntactically distinct but definitionally equal**
neutrals — e.g. a view function `ord_leq_of x y` vs a dictionary projection
`d.leq x y` that *should* δ-reduce to the same normal form. The positional arm
at `:566` recurses into `conv_struct neutral₁ neutral₂`, which **fails**: the two
neutrals do not converge under the current `whnf`/fast-path, so the `Eq` types
are (correctly, given non-convergence) judged non-convertible, and the proof
cannot bridge. Catalog proofs route around this with the **unbundled raw-`leq`
idiom** (every hypothesis shares the *literally identical* `leq` term, so no
operand convergence is ever asked of the kernel).

**Documented customers (both forced to unbundle today):**
1. `Ord Char`-shaped transport (occurrence #1, the original K6 finding).
2. `compare-ord-lexicographic` brick 1 accessor path (occurrence #2 —
   `Eq Bool (ord_leq_of…) True` won't bridge to `Eq Bool (d.leq…) True`).

Two documented customers is what crossed this from "customerless/parked" to an
operator-greenlit WP (Architect `evt_40ydcv9a45yjd` line; Steward judgment log
"forward candidates"). It is **route-aroundable** — no build is *blocked* — so
there is no soundness urgency; the goal is kernel *completeness* (fewer honest
proofs forced into the unbundled workaround).

## Settled inputs — pin, do NOT reopen

- **Fail-closed / soundness is the hard bar.** Any change must recognise
  strictly MORE true equalities and **NEVER accept a false one.** The **unsound
  cross-wise arm** (matching `Eq` operands cross-positionally, `a1↔b2`) that
  would have "helped" the original `Eq Bool` attempt is a **HARD NO** — already
  ruled out (`LawfulClasses §5`). The fix is positional/convergence-only.
- **Additive, non-weakening.** Must not change or remove any existing conversion
  behaviour; the full landed kernel + elaborator suite stays green. Existing
  `:566`/`:419` arms are the foundation, not competition.
- **The unbundled idiom STAYS valid.** This WP adds kernel completeness; it does
  **not** deprecate or require rewriting the catalog's raw-`leq`/explicit-law
  workaround. Landed catalog proofs are untouched. (Reflect-don't-extend: the
  idiom remains the right shape where it reads cleanly; K6 just stops *forcing*
  it.)

## Design seam — ARCHITECT-OWNED, gates the WP (this frame stops here on purpose)

The fix shape is **not specified** because it turns on a grounding call only the
Architect can make against the *current* kernel:

1. **Re-ground the exact current failing term** (post-`obs-eq-termination`) from
   one of the two customer sites. Determine which of two worlds we are in:
   - **(a) genuine completeness gap** — the two neutrals *are* definitionally
     equal and `conv_struct`/`whnf` simply doesn't see it (a δ-unfold ordering,
     projection-reduction, or normalization shortfall). Then design the **sound
     conv_struct/whnf extension** that makes them converge, with a pseudocode
     spec + the soundness argument (*why it can never equate two distinct
     terms*). This is the T1 design judgment the kernel-implementer executes to.
   - **(b) correct rejection** — the operands are genuinely inconvertible in
     those proof shapes (the "definitionally equal" reading was an illusion of
     the surface syntax). Then **K6 closes as won't-fix**: unbundling is the
     correct design, and the frame is retired with that finding recorded. A
     legitimate outcome — do not manufacture a kernel change to avoid it.
2. If (a): fix the **algorithm location** — is this a new `conv_struct` arm, a
   `whnf`/δ-strategy adjustment, or a projection-reduction rule? Architect calls
   it; each has a different soundness surface.

**Flag to the Steward immediately** if grounding shows this is really a
different-lane gap (e.g. an elaborator emission choice, not a kernel one).

## Acceptance criteria

- **AC1 — customer proofs bundle.** At least one of the two documented customer
  proofs (Ord Char accessor path; compare-ord brick 1 bundled-`Ord` soundness)
  that **today requires the unbundled idiom** kernel-checks in its **bundled**
  form after the fix. (If the Architect returns world (b), this AC is replaced by
  a recorded won't-fix finding — see the seam.)
- **AC2 — soundness preserved, adversarially.** A targeted kernel test that the
  new behaviour **accepts** the true operand equality **and REJECTS a crafted
  false one** (two neutrals that are *not* definitionally equal must stay
  non-convertible). Not a green-vs-green: the reject arm must genuinely reject.
- **AC3 — no regression.** Full kernel + elaborator suite green; every existing
  `conv.rs` arm unchanged in behaviour; the fast-path and `:566` arm intact.
- **AC4 — TCB delta reviewed.** The `trusted_base()`/conv delta is exactly the
  Architect-designed change; CV cross-checks the algorithm against the soundness
  argument; the change is documented at the kernel-source level.

## Gate

**Architect algorithm + soundness design (gates everything below)** → Kernel
ring (kernel-leader → kernel-implementer → kernel-qa) → **CV cross-check** (TCB
audit vs the soundness argument) → `git_request` to Steward → **CI-gated** merge
(full suite — this is the trust root, not doc-only). Own the retro; flag every
judgment call. **No WP-token identifiers in kernel source.** Re-verify all
`file:line` cites at pickup.
