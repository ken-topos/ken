# obs-eq-termination — harden obs-reduction termination on `Eq`/`cast` at a recursive inductive with function-typed sub-structure (Map capstone, law-4 conv/obs fix)

**Steward frame → Kernel build.** Follow-on to the `conv-eq-congruence`
(Gap-conv, `90f39fe`) arc: that arm was **reverted-to-green** (`b79313f`,
net-zero to pre-arm `019b695`) because it exposed a **pre-existing latent
non-termination in the observational reducer**. This WP fixes that root, then
**re-lands the fixed `(Eq,Eq)` arm** so law 4's conclusion can build. Owner:
**Kernel** (implementer + QA). Gate: **Architect** (soundness — vector
pre-cleared below; he grounds the chosen mechanism + the repro at gate time) +
**kernel-qa** + CI. Findings → **Steward**. No `/spec` change (kernel conv/obs
internal). Same reviewer set + no-`/spec` profile as `conv-eq-congruence` /
`sct-completeness`.

## Why this WP exists

Law 4's `toListOrdered` **elaborates + is well-typed** but its `declare_def`
whole-body kernel-recheck **diverges** (unbounded, ~1.1 GB/s, SIGKILL ~12 GB) —
a **fatal usability regression in the trusted checker** (Pat: *"the language is
non-functional with that resource usage"*). Grounded decisively (2026-07-03):

- **Six-round opaque-stub bisection** ruled out every proof term — with all 14
  Map-arc + goal-forming names opaque, the check still diverges ⇒ **the cost is
  in the kernel conversion/obs machinery on the term shape, function-independent**.
- **Isolation-flip:** reverting *only* the 5-line `(Eq,Eq)` `conv_struct` arm
  (`90f39fe`) turns the divergence into a **0.05 s clean reject** ⇒ the arm is
  the **trigger**.
- **printf-depth trace:** monotonic never-popping growth (depth ~1:1 with calls,
  trap at depth 5000 = call #16585, **zero popping**) ⇒ a **genuine infinite
  loop**, not a huge-finite unwind. The cycle re-enters the **same logical
  comparison one weakening-layer deeper each lap** (a **uniform `+7` de Bruijn
  shift per lap**, every lap structurally identical), never recognised as
  in-progress, never memoised.

**Trigger vs. root (the load-bearing distinction).** The `(Eq,Eq)` arm is the
**messenger**: pre-arm, `(Eq,Eq) → _ => false` cut the path at lap 0; the arm
made a **pre-existing latent obs-reducer divergence reachable** (it is a
[[gate-widening-exposes-latent-bugs-in-newly-reachable-code]] instance — a
completeness fix surfacing a latent divergence in deeper machinery, plausibly
interacting with K7's `eq_at_inductive` operand-whnf change). **Divergence ≠
unsoundness**: a hung checker admits *nothing*, the trust root is intact — this
is completeness/usability, not a soundness hole. The fix belongs at the **root
(obs-reduction termination)**, not the messenger arm.

## The bug (root cause — Architect-grounded, exact site pinned against the repro)

A **productive non-normalisation** in the observational reducer on `Eq`/`cast`
at a **recursive inductive whose sub-structure is function-typed**. Reading
(`crates/ken-kernel/src/obs.rs`, `conv.rs`):

- The `+7` **uniform** weakening per lap comes from the **function-type** obs
  paths — `eq_at_pi` (funext) / `cast_at_pi` do `weaken(_, 1)` + a Π/λ binder.
  So the regenerating layer is a **funext step**, not a Cast-telescope.
- **The Cast-telescope reading is REFUTED** (do not target it): `Node`'s fields
  (`Tree k v`, `k`, `v`, `Tree k v`) are **non-dependent**, so in
  `eq_at_inductive` `a_ty_j ≡ b_ty_j`, `convert_type` returns true, the branch
  takes `lhs = a_bar[j]` with **no Cast** — a homogeneous `Tree` emits a
  cast-free `Σ` of field-equalities.
- Most likely driver: the **comparator functions** in the goal (`leq : k → k →
  Bool`, `pairLeq : Pair → Pair → Bool`) — an `Eq`/`cast` at a **function type**
  driving funext, interleaved with the `Eq`-at-`Tree` decomposition, threaded
  through the `convert_type ↔ conv_struct` calls inside `eq_at_inductive` (the
  telescope check) and `cast_reduce` (the regularity check, `obs.rs:314`).

**Perishable — verify against the landed code + the repro, not this description.**
The *exact* recurring goal depends on the elaborated `toListOrdered` term;
Architect pins the precise site against the WP's first deliverable (the repro),
and confirms the chosen fix's soundness there. The **objective** (obs-reduction
must terminate on this shape) and the **soundness envelope** (below) are the
fixed inputs; the exact function/line is discovered in-build.

## The fix (soundness envelope PRE-CLEARED by the Architect — pin the vector against the repro)

Whatever the exact site, the fix **must decide *identical* convertibility** —
the `declare_def` recheck must **COMPLETE and PASS**, never skip/weaken/bypass a
check to "fit" (that would convert a resource wall into a soundness hole,
[[conformance-hand-feeds-the-deliverable]] trust-root integrity). Three
candidate mechanisms, with their soundness cost stated — **prefer a
trivially-sound one; the obligation-bearing one is gated, not casual:**

1. **Memoise `eq_at_inductive` / `cast_reduce`** on the recurring `(ty, a, b)`
   obs-reduction goal → **trivially sound** (caching a pure decision procedure;
   changes cost, not the decision). Preferred if it breaks the cycle.
2. **Leave the recursive-inductive `Eq` conjunct neutral** (don't force the
   non-normalising reduction) → **fail-closed sound** (returns *less*-reduced,
   never a false convertibility). Safe fallback.
3. **Occurs-guard that returns `true` on a recurring `(ty, a, b)` goal** →
   carries **one proof obligation**: recurrence ⟹ the sub-goals are
   **α-equal-modulo-weakening** (the two sides are the *same* term, re-nested).
   **The trace's `+7`-*uniform*-weakening-per-lap is direct evidence this holds**
   (uniform de Bruijn shift = α-equal-modulo-weakening) — so this vector is
   **very likely sound** — but it **must be argued against the repro, not
   asserted from the trace** (Architect grounds it before the final gate).

## Deliverables (mandated outline)

1. **Minimal divergence repro (FIRST deliverable, the regression test).** A
   guarded hanging-test in the kernel that **independently reproduces** the
   `Eq`-at-recursive-inductive-with-function-typed-sub-structure divergence
   (trigger + mechanism), not resting on the `toListOrdered` report. Architect
   co-builds/independently verifies it (this is the natural home for the
   hanging-test infra; a solo build now would risk the shared build lock for no
   schedule gain).
2. **The termination fix** at the pinned site, one of the enveloped mechanisms,
   deciding identical convertibility.
3. **Re-land the fixed `(Eq,Eq)` `conv_struct` arm** (the reverted `90f39fe`
   congruence closure) **+ its `conv_eq_congruence` test** — now that the obs
   reducer terminates, the arm no longer diverges. (Permanent-revert-B — reserve
   law 4 without the congruence — stays **RULED OUT**: law 4's `Pair`-comparator
   ↔ `k`-predicate bridge *needs* the arm; reverting re-walls it.)
4. **Law 4's conclusion** (`consSorted` / `isSortedAppend` / `toListOrdered`)
   **kernel-checks clean and cheap** on the fixed base — the whole point.

## Acceptance criteria (all testable)

- **AC1 — repro flips under ONLY the fix.** The hanging-test **diverges/traps
  pre-fix** and **terminates post-fix** under **only** this change (isolation-
  flip). Land it as a kernel regression test with a bounded guard (depth/time),
  so a regression re-hangs the test, not the CI box.
- **AC2 — identical convertibility (the soundness net).** The fix decides the
  **same** convertibility as the (terminating) specification: (i) full
  `ken-kernel` suite green **and** a **discriminating valid/invalid pair** — a
  genuinely convertible `Eq`-at-recursive-inductive pair **converts**, a
  genuinely non-convertible one is **REJECTED** (not swallowed). (ii) If the
  **occurs-guard-returns-true** mechanism is chosen, the **α-equal-modulo-
  weakening argument is written and Architect-gated** — the "return true on
  recurrence" is *argued sound against the repro*, never assumed. Mechanisms (1)
  memoise / (2) neutral carry no such obligation.
- **AC3 — the recheck COMPLETES, never skipped.** `toListOrdered`'s
  `declare_def` whole-body recheck **runs to a PASS** cheaply (bounded memory,
  no SIGKILL) — "assembles + well-typed" is **not** acceptance; the trusted
  check must succeed. No path skips/weakens/short-circuits the recheck to fit.
- **AC4 — workspace-wide green (blast radius).** `cargo test --workspace` green
  — re-landing the arm is a conv **completeness** change (K7 blast-radius lesson:
  a downstream `.ken`/test proof may now type-check *because* the arm converts
  something). Validate the **whole workspace**, not just `ken-kernel`.
- **AC5 — `trusted_base` unchanged.** No new `declare_*` / `Term` variant /
  kernel decl; `trusted_base()` byte-unchanged; the diff is `ken-kernel`
  (`obs.rs`/`conv.rs`) + kernel tests only. The TCB's **power** does not grow —
  only conv/obs **termination + completeness**. Grep-verify.

## Guardrails (do-not-reopen)

- **Root, not messenger.** Fix obs-reduction termination; do **not** "fix" by
  gutting the `(Eq,Eq)` arm again (that's the revert, already done as interim
  hygiene) — the arm re-lands with the fix.
- **Not the Cast-telescope.** The dependent-telescope reading is **refuted**
  (Node fields non-dependent). Target the **funext / `eq_at_pi`·`cast_at_pi`**
  path + the `Eq`-at-recursive-inductive interaction.
- **Completeness/termination, not soundness-loosening.** If any change would make
  conversion accept a genuinely non-convertible pair, STOP → raise to Steward.
  The only sanctioned changes decide **identical** convertibility.
- **Permanent-revert-B stays OUT.** We fix + re-land the arm; we do not abandon
  the `(Eq,Eq)` congruence and reserve law 4 another way.

## Sequencing

- **Frame authored now** (front-loaded design judgment). **Kick off Kernel after
  the revert (`b79313f`) merges** — the fix branch cuts off the termination-
  guaranteed **post-revert `main`** and re-adds the fixed arm; rebase this frame
  branch onto that `main` at kickoff (trivial — doc add). Frame rides with the
  build (frame + fix + re-landed arm merge together).
- **Gate:** Architect (soundness — envelope pre-cleared; he grounds the chosen
  mechanism + independently verifies the repro reproduces-pre-fix / dies-post-fix
  / decides identical convertibility on the discriminating pair at gate time) +
  kernel-qa + CI. **Parallel-gate** once the diff is confirmed
  `obs.rs`/`conv.rs`-only (the `conv-eq-congruence` / `sct-(a)/(b)` precedent).
- **On merge:** Steward signals Foundation to resume `map-verified-laws` on the
  held branch (rebased onto the fixed `main`) and build **law 4's conclusion**
  end-to-end; the **whole capstone** (law 4 + laws 1/2/3/5 via Tree-Σ v2) lands
  on the fixed base (Foundation holds it as one unit).
- **Lane:** Kernel (`ken-kernel/obs.rs`, `conv.rs`).
