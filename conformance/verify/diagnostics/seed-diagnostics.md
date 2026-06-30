# Proof-failure diagnostics (V4) conformance — seed cases

Format: `../../README.md`. These pin **WS-V V4** — the proof-failure
diagnostics: turning a non-`proved` verdict from the prover (`23 §1.2`) into a
**structured, machine-readable explanation** an agent can act on. For each
verdict that is **not** `proved`, the verification layer emits one or more of
the **four diagnostics** (`24 §1`–§4): a **Kripke countermodel** with the
`false`-vs-`unknown` tag (§1), a **typed hole** with `unknown` runtime
propagation (§2), the **three-region** Heyting decomposition (§3), and the
**slice / missing-hypothesis** bridge (§4) — plus region-tagged
`suggested_actions` (§5), determinism and minimality (§6). Grounded in the
**landed** `24-diagnostics.md` (`§1`–§7), the V3 verdict shapes it consumes (`23
§1.2`/§4/§5), V1's status + honesty model (`21 §5.1`/§5.4), the runtime
`unknown` value + Kleene table (`41 §6`, `42 §4`), and the Heyting structure /
`Decidable` (`16 §1.3`). The prototype is not mounted; none of these required
it.

**The cardinal rule — fidelity to V3's verdict (`24` preamble).** A diagnostic
**explains the verdict V3 produced; it never re-decides it.** The `false` /
`unknown` tag is a **projection of V3's verdict**, *not* an independent reading
of the evidence: `24 false ⟺ V3 disproved`, `24 unknown ⟺ V3 unknown`, `proved ⟹
no diagnostic`. V4 reads V3's `verdict` field; it does **not** recompute the
verdict from the Kripke model. This is the load-bearing property of the whole
chapter and the reason a V4 bug is an **advisory-UX** regression, never an
unsoundness (★★): the kernel already settled `proved`/not via the certificate
(`23 §1.3`, `21 §5.4`).

**The failure mode V4 guards is *infidelity*, not unsoundness (`24 §6`).** V4 is
untrusted — a wrong diagnostic misleads an agent but **cannot** make an unsound
program check (the kernel is unaffected). So the load-bearing class here is
**fidelity**: mislabeling `unknown` as `false` (or vice versa), emitting a
diagnostic for a `proved` goal, or offering an `unknown`-only repair on a
refuted goal. Cases tagged **(fidelity)** encode this V4-specific commitment —
the advisory-layer analog of `(soundness)` — and must never regress. The one
genuinely **kernel-structural** commitment that reappears here is the typed-hole
honesty guard (a hole reads `unknown`, never silently `proved`, by
`trusted_base()` membership — `24 §2`, `23 §1.3`); it is tagged **(soundness)**.

**The Glivenko cross-case invariant (`24` preamble / §3 — the consistency-sweep
anchor).** A **classically-valid** goal is **never** tagged `false` / placed in
`S_{¬φ}`: by Glivenko, `¬¬φ` is intuitionistically provable iff `φ` is
classically provable, so for a classically-valid `φ`, `¬φ` is unprovable and
**no world forces `¬φ`**. Such a goal is `proved` (if intuitionistically valid)
or `unknown` (the `¬¬φ ⇒ φ` gap, e.g. `p ∨ ¬p`), never `disproved`/`false`. The
discriminating pair on shared abstract-atom metatheory is **`p ∧ ¬p` → `false`**
(`¬(p∧¬p)` is a theorem ⇒ `¬φ` forced) vs **`p ∨ ¬p` → `unknown`** (`¬(p∨¬p)` is
*false* ⇒ neither forced); the discriminator is *is `¬φ` forced*, nothing else.
The cases in this class — `unknown-verdict-not-relabeled-false` (A2),
`lem-unknown-no-forcing-world` (B2), `classically-valid-never-in-refuted-region`
(D2) — must **agree** (all `unknown`, never `false`), asserted by the cross-case
sweep, not just per-case verdict-flip.

**Vocabulary (reconcile, not cite).** The `false`/`unknown` **tag** *is* the
`verdict` field of the diagnostic value, copied from V3 — the
`KripkeCountermodel` value carries **no** independent `is_false` flag (`24 §1`).
A **typed hole** is a `declare_postulate` of `φ` (`18 §4.2`) enumerated by
`trusted_base()` (`18 §5`) — the same honesty mechanism V3 uses. The runtime
**`unknown`** is the third value (`41 §6`), the operational face of an open
hole; its propagation table is `41 §6` **verbatim** (strict positions + `∧`/`∨`
absorption + `¬`), and `24 §2` adds no rule `41 §6` omits. Region names `S_φ` /
`S_{¬φ}` / `unknown` and the characteristic core `{¬¬φ} ∖ S_φ` are `24 §3`'s;
`Decidable P = P + (P → Empty)` is the derived sum (`16 §1.3`), excluded middle
holding only for *decidable* props. The diagnostic **value shapes**
(`KripkeCountermodel`, `TypedHole`) are landed in `24 §1`/§2 — not `(oracle)`;
the **wire serialization** is `25`-owned and **out of scope** here (`24 §7`).

---

## A. The cardinal rule — diagnostic projects V3's verdict, never re-decides

### verify/diagnostics/disproved-verdict-projects-false-tag
- spec: `24` preamble (the cardinal rule); `24 §1`
- given: an obligation whose prover verdict (`23 §1.2`) is **`disproved`**,
  carrying a `KripkeCountermodel` whose `verdict` field is `disproved`.
- expect: the emitted diagnostic's tag is **`false`**, and that tag **is** the
  copied `verdict` field — the `KripkeCountermodel` value carries **no**
  independent `is_false` flag (`24 §1`). The tag is not recomputed from
  `worlds`/`forcing`.
- why: `24` cardinal rule — the tag is a *projection* of V3's verdict.
  Structural (the tag equals the copied verdict field); pairs with the
  relabel-flip A2.

### verify/diagnostics/unknown-verdict-not-relabeled-false (fidelity)
- spec: `24` preamble; `24 §1`/§6
- given: an obligation whose prover verdict is **`unknown`** for the
  classically-valid-but-intuitionistically-unprovable goal `p ∨ ¬p` (no world
  forces `φ`; none forces `¬φ`, since `¬(p ∨ ¬p)` is false).
- expect: the diagnostic tag is **`unknown`**; relabeling it **`false`** is a
  **fidelity bug** (`24 §6`) — rejected. The tag tracks V3's `unknown` verdict,
  never the absence of a forcing world re-read as a refutation.
- why: the cardinal rule + Glivenko. Verdict-flip: correct = `unknown` tag, the
  exact bug ("countermodel ⇒ false") = `false` tag — green-vs-red. Member of the
  cross-case sweep class (with B2, D2). The author-side V3 trap (`p∨¬p` →
  disproved) made into a V4 reject-witness.

### verify/diagnostics/evidence-consumed-unchanged-from-v3 (fidelity)
- spec: `24` preamble; `24 §1`/§2; `24 §7` AC4
- given: a `disproved` verdict carrying a countermodel `M`
  (worlds/order/forcing/failure) and, separately, an `unknown` verdict carrying
  a typed hole `?h : φ` with provenance `prov`.
- expect: the diagnostic reproduces `M` and `?h` **unchanged** — the worlds,
  forcing pairs, `failure.world`, and the `HoleId`/`goal`/`context`/`origin` are
  V3's emitted evidence, **copied not re-derived**. V4 adds the tag/region; it
  does not alter the evidence.
- why: `24 §7` AC4 (fidelity to V3) — the diagnostic *consumes* V3's output. A
  V4 that re-derived the countermodel could diverge from V3's verdict; copying
  forecloses it. Structural (evidence equality).

## B. The `false`-vs-`unknown` discriminator (`24 §1`)

### verify/diagnostics/refuted-goal-false-with-forcing-world
- spec: `24 §1` (the `disproved → false` bullet); `23 §4`/§5; `16 §1.3`
- given: the obligation `Γ ⊢ p ∧ ¬p`; the prover refutes it (`¬(p ∧ ¬p)` is an
  intuitionistic theorem, so `¬φ` holds and some world forces it) → verdict
  `disproved`.
- expect: a `KripkeCountermodel` tagged **`false`**, with `failure.world = w*`
  the world that **forces `¬φ`** (positive evidence against `φ`); region
  `S_{¬φ}` (§3). The actionable reading is *fix the code or spec*;
  `add_precondition` does **not** appear (§4/§5, region-tagged).
- why: `24 §1` — `false` requires a world forcing `¬φ` (genuinely refutable).
  Structural: `failure.world` names the refuting world. Contrast partner of B2
  on the same abstract atom `p`.

### verify/diagnostics/lem-unknown-no-forcing-world (fidelity)
- spec: `24 §1` (the `unknown → unknown` bullet); `23 §4`; `24` preamble
  (Glivenko)
- given: the obligation `Γ ⊢ p ∨ ¬p` (abstract atom `p`); the prover finds
  **no** world forcing `φ` and **none** forcing `¬φ` → verdict `unknown`.
- expect: tag **`unknown`** (the information is absent — the `¬¬φ ⇒ φ` gap),
  with **no** `failure.world` forcing `¬φ`. **Not** `false`.
- why: `24 §1` discriminator — *merely fails to force `φ`* ≠ refutation.
  Verdict-flip against B1: same abstract `p`, `p ∧ ¬p` → `false` vs `p ∨ ¬p` →
  `unknown`; the only discriminator is *is `¬φ` forced*. Cross-case sweep member
  (with A2, D2).

## C. Typed holes and `unknown` propagation (`24 §2`)

### verify/diagnostics/open-hole-typechecks-runs-in-trusted-base (soundness)
- spec: `24 §2`; `23 §1.3`; `18 §4.2`/§5; `21 §5.4`
- given: a program with one undischarged obligation `Γ ⊢ φ` → verdict `unknown`,
  emitted as a typed hole `?h : φ` (`24 §2`).
- expect: the program **type-checks and runs**; `?h` is admitted as an **opaque
  postulate** of `φ` (`declare_postulate`, `18 §4.2`) and its goal is enumerated
  by `trusted_base()` (`18 §5`). The hole reads **`unknown`**, never silently
  `proved` — the discriminator is **postulate membership**, not a V-layer flag.
- why: `24 §2` / `23 §1.3` honesty guard, kernel-structural — the V3 carry
  reused. Soundness-class: a hole reading `proved` would forge discharge.
  Absence-assertion guarded on `trusted_base()` membership (would fail under any
  bug that drops `?h` from the trusted base).

### verify/diagnostics/unknown-absorption-on-known-operand
- spec: `24 §2` (the Kleene table); `41 §6`; `42 §4`
- given: an expression whose result depends on an open hole (value `unknown`)
  combined with a **known** operand via `∧` / `∨`.
- expect: per `41 §6` **verbatim** — `unknown ∧ false = false`, `unknown ∨ true
  = true` (the absorbing operand decides without forcing the hole), and `unknown
  ∧ true = unknown`, `unknown ∨ false = unknown`, `¬ unknown = unknown` (no
  decision).
- why: `24 §2` = `41 §6` (the two files cannot contradict). Discriminating: a
  **strict** evaluator forces the hole and yields `unknown` where absorption
  yields `false`/`true` — green-vs-red on `unknown ∧ false` and `unknown ∨
  true`. Builder note (not asserted): the connective must be **non-strict in the
  absorbing position** (the CBV-laziness carry, `42 §4`).

### verify/diagnostics/unknown-propagates-in-strict-position
- spec: `24 §2` (strict positions); `41 §6`; `42 §4`
- given: an `unknown` value in a **strict** position — as the scrutinee of an
  eliminator, a function applied, a strict-primitive operand, a `cast`/`Eq`
  argument, or a projection.
- expect: the result is **`unknown`** (`apply unknown u = unknown`, `elimReduce
  … unknown = unknown`, `primReduce op (… unknown …) = unknown`, `(a, unknown).2
  = unknown`) — `41 §6`.
- why: `24 §2` strict-position rule — the runtime third value propagates through
  forcing positions. Structural value assertion; the strictness half
  complementing the absorption case. (`⇒` is `Π`/`apply`, covered here, not a
  separate row.)

### verify/diagnostics/hole-free-program-never-unknown (fidelity)
- spec: `24 §2`; `41 §6`; `42 §4` AC4; `24 §7`
- given: a fully-discharged program — **no** open holes (no `unknown`-typed
  postulates in `trusted_base()`).
- expect: evaluation **never** yields the runtime `unknown` from holes (`41 §6`,
  `42 §4`): every result is a definite value. The presence of `unknown`
  **flips** on hole-present vs hole-free.
- why: `42 §4` AC4 — a fully-verified program has no `unknown` residue.
  Discriminating: hole-present ⇒ `unknown`, hole-free ⇒ definite value
  (green-vs-red). Guards against a bug that injects `unknown` spuriously.

## D. The three-region Heyting decomposition (`24 §3`)

### verify/diagnostics/three-regions-partition-keyed-to-verdict
- spec: `24 §3`; `23 §1.2`/§2
- given: three goals with prover verdicts `proved`, `disproved`, `unknown`
  respectively.
- expect: each lands in **exactly one** region — `proved → S_φ`, `disproved →
  S_{¬φ}`, `unknown → unknown` — **read off V3's verdict**, not recomputed. The
  three are **exhaustive and disjoint** because the classifier is **total** (`23
  §2`): no goal is unclassified, none is in two regions.
- why: `24 §3` keys the region to the verdict (inheriting V3's exhaustiveness).
  Structural partition assertion (totality + disjointness). The region is not a
  fresh `{¬¬φ}` set computation that could diverge from the verdict.

### verify/diagnostics/classically-valid-never-in-refuted-region (fidelity)
- spec: `24 §3` (the cross-region invariant); `24` preamble (Glivenko)
- given: the classically-valid goals `p ∨ ¬p` and `¬¬p ⇒ p`, each with prover
  verdict `unknown`.
- expect: both land in the **`unknown`** region (the `¬¬φ` gap; characteristic
  core `{¬¬φ} ∖ S_φ`), **never** `S_{¬φ}`. A diagnostic placing either in
  `S_{¬φ}` / tagging it `false` is a fidelity bug — rejected.
- why: `24 §3` Glivenko invariant — a classically-valid `φ` cannot be refuted
  (`¬φ` unprovable). Cross-case sweep anchor: `p ∨ ¬p` here must agree with B2
  (`unknown`) and A2 (not relabeled `false`) — same metatheoretic class, same
  verdict. Verdict-flip: correct = `unknown` region, the bug = `S_{¬φ}`.

## E. Slice / missing-hypothesis (`24 §4`) and region-tagged actions (`24 §5`)

### verify/diagnostics/slice-missing-hypothesis-sufficiency-flip
- spec: `24 §4` (sufficiency MUST); `24 §5`; `24 §6` (minimality SHOULD)
- given: a goal `φ` in the **`unknown`** region — `Γ ⊢ φ` not discharged — for
  which `Γ, h : ψ ⊢ φ` **is** discharged.
- expect: the diagnostic surfaces `ψ` as the **missing precondition** with
  action `add_precondition ψ` (region `unknown`), and adding `ψ` flips the
  prover verdict **`unknown` → `proved`** (the sufficiency MUST). The case
  asserts the **flip**, **not** that `ψ` is minimal or unique (minimality is a
  `24 §6` SHOULD).
- why: `24 §4` — sufficiency is the contract (`Γ,ψ ⊢ φ` makes V3 `proved`).
  Verdict-flip on adding `ψ`. Per LP-5: asserting only sufficiency keeps the
  corpus from locking a heuristic (minimality) as contract.

### verify/diagnostics/no-slice-action-for-refuted-goal (fidelity)
- spec: `24 §4` (`unknown`-only); `24 §5` (region-tagged actions); `24 §3`
- given: a **refuted** goal (verdict `disproved`, region `S_{¬φ}`, tag `false`).
- expect: the `suggested_actions` contain **no** `unknown`-only action
  (`add_precondition` / `strengthen_refinement` / `provide_lemma` / `case_split`
  / `induct_on`) — only the `false`-region action `fix_counterexample x`.
  Offering `add_precondition` for a refuted goal is a **fidelity bug** (`24
  §4`/§5) — rejected.
- why: `24 §4`/§5 — you do not repair a genuine counterexample by adding a
  hypothesis; §4 is `unknown`-only and actions are region-tagged.
  Discriminating: a `false` goal yields `fix_counterexample`, **never**
  `add_precondition` (green-vs-red on the region-tag). Cross-region partner of
  E1.

## F. Determinism and no regression (`24 §6`/§7)

### verify/diagnostics/fully-proved-program-zero-diagnostics
- spec: `24 §7` AC5; `24` preamble (the `proved → no diagnostic` row)
- given: a program all of whose obligations are `proved` (each cert kernel-
  accepted; nothing in `trusted_base()` from the spec).
- expect: the diagnostic set is **empty** — `|diagnostics| = 0`. There is **no**
  "proved" diagnostic; a `proved` goal carries nothing to fix.
- why: `24` preamble + `24 §7` AC5 — diagnostics are emitted only for
  **non-`proved`** verdicts. Structural (empty set). Discriminating: a bug that
  emitted a redundant diagnostic for a `proved` goal flips `|diagnostics|` from
  0 → ≥1 (green-vs-red). The no-regression backstop (V3's pipeline unaffected).

### verify/diagnostics/deterministic-same-input-same-diagnostic
- spec: `24 §6` (determinism MUST); `22 §1` (hole-id provenance)
- given: the same program + spec + prover version run twice.
- expect: **identical** diagnostics — same countermodel (same worlds/forcing),
  same `HoleId`s (stable functions of the obligation **provenance**, `22 §1`,
  not allocation order), same `suggested_actions` order — so an agent can diff
  runs.
- why: `24 §6` — determinism is a MUST (minimality is the SHOULD, not asserted
  for equality). Structural: byte-identical diagnostic value across runs. Guards
  against allocation-order-dependent hole ids.

---

## Coverage map (frame ACs → cases; `24` §-mechanisms)

- **AC1 `false` vs `unknown` honest** — `refuted-goal-false-with-forcing-world`,
  `lem-unknown-no-forcing-world`, `unknown-verdict-not-relabeled-false` (the §1
  discriminator + the fidelity flip).
- **AC2 typed holes run + propagation** —
  `open-hole-typechecks-runs-in-trusted-base` (the §1.3 honesty guard reused),
  `unknown-absorption-on-known-operand`,
  `unknown-propagates-in-strict-position`, `hole-free-program-never-unknown`
  (the Kleene table `41 §6` verbatim).
- **AC3 three-region partition** — `three-regions-partition-keyed-to-verdict`,
  `classically-valid-never-in-refuted-region` (the `¬¬φ`-gap + Glivenko
  invariant).
- **AC4 fidelity to V3** — `disproved-verdict-projects-false-tag`,
  `evidence-consumed-unchanged-from-v3`, `no-slice-action-for-refuted-goal` (the
  cardinal rule — verdict + evidence + region-tag consumed unchanged).
- **AC5 no regression** — `fully-proved-program-zero-diagnostics`,
  `deterministic-same-input-same-diagnostic`.
- **`24` §-mechanisms** — §1 Kripke countermodel + false/unknown:
  `refuted-goal-false-with-forcing-world`, `lem-unknown-no-forcing-world`,
  `disproved-verdict-projects-false-tag`, `evidence-consumed-unchanged-from-v3`;
  §2 typed holes + propagation: the four C cases; §3 three-region:
  `three-regions-partition-keyed-to-verdict`,
  `classically-valid-never-in-refuted-region`; §4 slice (`unknown`-only):
  `slice-missing-hypothesis-sufficiency-flip`,
  `no-slice-action-for-refuted-goal`; §5 region-tagged actions:
  `no-slice-action-for-refuted-goal`,
  `slice-missing-hypothesis-sufficiency-flip`; §6 determinism:
  `deterministic-same-input-same-diagnostic`.

**The cross-case metatheory-consistency sweep (hard gate, the V3 carry).** The
abstract-atom / `¬¬φ`-gap / classically-valid class —
`unknown-verdict-not-relabeled-false` (A2), `lem-unknown-no-forcing-world` (B2),
`classically-valid-never-in-refuted-region` (D2) — must **agree**: `p ∨ ¬p` (and
`¬¬p ⇒ p`) is **`unknown`**, region `unknown`, **never** `false`/`S_{¬φ}`, by
Glivenko (classically-valid ⇒ never refutable). The contrast witness
`refuted-goal-false-with-forcing-world` (B1, `p ∧ ¬p` → `false`) is the only
member of the abstract-`p` family that is refutable, and the discriminator is
*is `¬φ` forced*. Per-case verdict-flip does **not** subsume this sweep — the
cases are grouped by shared metatheory and asserted consistent.

Build-sequencing: V4 extends the **landed** verification spine — it consumes
V3's verdict + evidence (`23 §1.2`: countermodel forcing `¬φ` / typed hole) and
**re-decides nothing** (the cardinal rule). It adds **no kernel former**: the
diagnostic values (`KripkeCountermodel`, `TypedHole`) are derived data; the
runtime `unknown` is `41 §6`'s third value; the regions are `24 §3` keyed to the
verdict. A V4 bug is **infidelity** (advisory-UX), never unsoundness — the
kernel is untouched (`24 §6`). The **wire serialization** is `25`-owned and out
of scope (`24 §7`).
