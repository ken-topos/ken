# Proof-failure diagnostics

> Status: **V4 elaborated** (implementation-ready). Normative for the four
> diagnostic mechanisms and their meaning; concrete serialization is in
> `25-protocol.md`. Contract for WS-V **V4**. This is the feature that most
> differentiates Ken for agentic use: a proof that does not go through yields a
> *structured, machine-readable explanation*, not an opaque error — high value
> for agents.

When the prover (`23`) attempts an obligation `Γ ⊢ φ` (`22`), the **verdict**
(`23 §1.2`) is one of `proved` / `disproved` / `unknown`. For every verdict that
is **not** `proved`, the verification layer emits one or more of **four
diagnostics**, each derived from the **topos/Heyting structure** itself (so they
are principled, not heuristic) and carried over the protocol (`25`).

## The diagnostic contract — fidelity to V3's verdict (the cardinal rule)

A diagnostic **explains the verdict V3 produced; it never re-decides it.** This
is the load-bearing property of the whole chapter and the reason a V4 bug is an
*advisory-UX* regression, never an unsoundness (★★): the kernel already settled
`proved`/not via the certificate (`23 §1.3`, `21 §5.4`); V4 only *renders* the
outcome. Each verdict maps to exactly **one** diagnostic region and disposition:

| V3 verdict (`23 §1.2`) | Carried evidence | Region (§3) | Tag | Actionable reading |
|---|---|---|---|---|
| `proved` | certificate (kernel-checked) | `S_φ` | — | nothing to fix — **no diagnostic** (§7, AC5) |
| `disproved` | countermodel **forcing `¬φ`** | `S_{¬φ}` | **`false`** | **fix** the code or the spec (§1; §4 does **not** apply) |
| `unknown` | typed hole `?h : φ` | `{¬¬φ} ∖ S_φ` | **`unknown`** | **supply more facts** (§1, §2, §4) |

**The cardinal rule (the source pin).** The `false` / `unknown` tag on a
diagnostic is a **projection of V3's verdict**, *not* an independent reading of
the evidence. In particular a Kripke model that merely **fails to force `φ`** is
**not** a refutation: only a model that **forces `¬φ`** tags `false`. Pinning
this here — at the boundary where a procedure's output ("a countermodel") maps
to a verdict ("false or unknown") — closes the silence that would otherwise let
a reader default to "countermodel ⇒ false" (the V3-prover trap, now a standing
carry: an unpinned verdict-mapping is a latent bug). V4 **never relabels** an
`unknown` as `false` or vice versa: it reads V3's verdict field; it does not
recompute the verdict from the model.

The general invariant, load-bearing for the **cross-case consistency sweep**:

> **A classically-valid goal is never tagged `false`** (Glivenko: `¬¬φ` is
> intuitionistically provable iff `φ` is classically provable, so for a
> classically-valid `φ`, `¬φ` is unprovable and **no world forces `¬φ`**). Such
> a goal is `proved` (if intuitionistically valid) or `unknown` (if not — the
> `¬¬φ ⇒ φ` gap, e.g. `p ∨ ¬p`), and `false`/`disproved` is reserved for
> **genuinely refutable** goals where `¬φ` holds (`23 §5`, `16 §1.3`).

## 1. Kripke countermodels — the `false`-vs-`unknown` discriminator

When an FO obligation does not go through, the Kripke embedding (`23 §4`) hands
Z3 the obligation's truth condition `φ#` and, on failure, a **falsifying model
of `φ#`** — a finite **Kripke countermodel** for `φ`:

```
worlds:   w0 ≤ w1 ≤ …            (stages of knowledge; ≤ = information growth)
forcing:  w ⊩ P  for each atom P at each world w   (monotone in ≤)
witness:  the world w* and the subformula of φ not forced there
```

A Kripke model is **richer than a classical counterexample**: forcing is
monotone and the model carries more than two truth values across its worlds.
That extra structure is exactly what separates the two non-`proved` verdicts —
the discriminator V3 computes (`23 §5`) and V4 renders faithfully:

- **`disproved` → `false`.** Some world forces `¬φ` — i.e. `(¬φ)# := ∀ w' ≥ w.
  ¬ φ#[w']` holds at the model's root (`23 §4`). There is **positive evidence
  against** `φ`: it is **genuinely refutable** (`¬φ` is provable), region
  `S_{¬φ}` (§3). The canonical witness is `p ∧ ¬p` — `¬(p ∧ ¬p)` is an
  intuitionistic theorem, so `¬φ` holds and a world forces it. *Actionable:* the
  spec or the code is wrong — **fix it**. (When the prover also yields a kernel
  proof of `¬φ`, `check(env, Γ, p, ¬φ)` certifies the refutation, `21 §5.1`.)
- **`unknown` → `unknown`.** **No** world forces `φ` *and* **none** forces `¬φ`:
  the information is **absent**. `φ` sits in the `¬¬φ ⇒ φ` gap (§3) — e.g.
  the abstract-atom LEM instance `p ∨ ¬p` (`¬(p ∨ ¬p)` is *false*, so no world
  forces `¬φ`; yet `p ∨ ¬p` is intuitionistically unprovable). *Actionable:* the
  property may well hold but the current facts do not force it — **supply more
  facts** (§2 hole, §4 slice). The discriminating contrast — same abstract atom
  `p`, opposite verdict — is `p ∧ ¬p` (`false`, `¬φ` forced) vs `p ∨ ¬p`
  (`unknown`, neither forced): the discriminator is *is `¬φ` forced*, nothing
  else.

The discriminator is **structural in the model** — *forces `¬φ`* vs *merely
fails to force `φ`* — and it is **V3 that computes it** (the cardinal rule); V4
reads the resulting verdict. This split is invisible to a Boolean counterexample
(which offers only "satisfies / does not satisfy") and is the single most
actionable signal V4 carries: it tells the agent whether to *fix the code* (it
is false) or *supply more facts* (it is unknown). The diagnostic names the
**world `w*` and the atom** whose forcing would close the goal — the seed for
§4's missing hypothesis and §5's `add_precondition`.

**The countermodel diagnostic value** (machine-readable; the wire form is `25`):

```
KripkeCountermodel {
  verdict   : { disproved | unknown }   -- COPIED from V3 (the cardinal rule),
                                        --   never recomputed from the model
  worlds    : [WorldId]                 -- finite, w0 the root
  order     : [(WorldId, WorldId)]      -- the ≤ preorder (reflexive-transitive)
  forcing   : [(WorldId, AtomId)]       -- which atoms hold where (monotone in ≤)
  failure   : { world : WorldId, subformula : FormRef }  -- where φ breaks
}
```

The value carries **no** independent `is_false` flag: the `false`/`unknown` tag
*is* the `verdict` field, projected from V3 (the cardinal rule). `disproved`
holds **iff** some world forces `¬φ`; otherwise `unknown`.

## 2. Typed holes and `unknown` propagation

A V3 `unknown` verdict's evidence is a **typed hole** (`23 §1.2`):

```
  ?h : φ        in context Γ          -- the obligation's goal, located (22 §1)
```

- **It type-checks and runs.** The hole is admitted to the environment as an
  **opaque postulate** of type `φ` (`11 §4`; `declare_postulate`, `18 §4.2`) —
  so it appears in **`trusted_base()`** (`18 §5`, `21 §5.4`): the system is
  *honest* that `φ` is **assumed, not proved**. This is the same
  kernel-structural status as the prover's honesty guard (`23 §1.3`) — a hole
  reads `unknown`, never silently `proved`, because the discriminator is
  postulate membership in the kernel's own `trusted_base()`, not a V-layer flag.
  Shipping a verified artifact means **zero** spec-induced postulates in
  `trusted_base()` (or an explicit, recorded acceptance of the listed ones,
  `21 §5.4`).
- **`unknown` propagation (runtime).** Evaluating an expression that depends on
  an open hole yields the runtime third value **`unknown`** (`41 §6`, `42 §4`) —
  the *operational face* of the diagnostic: the program **runs**, and `unknown`
  marks exactly where the unproven property bears on a result (the Hazel "total
  error localization, program still runs" model). The static diagnostic ("this
  goal is `unknown`") and the runtime value are the **same third value**.
  Propagation is Kleene/Heyting (`41 §6`, `42 §4`):

```
strict positions propagate (apply / elim-scrutinee / strict prim / cast / Eq /
  projection on unknown ⇒ unknown):
  apply unknown u = unknown          elimReduce … unknown = unknown
  primReduce op (… unknown …) = unknown      (a, unknown).2 = unknown

absorbing connectives short-circuit on the KNOWN operand (the hole is NOT forced):
  unknown ∧ false = false      unknown ∨ true  = true
  unknown ∧ true  = unknown    unknown ∨ false = unknown      ¬ unknown = unknown
```

  This table is **exactly the pinned set of `41 §6`** (the strict positions +
  `∧`/`∨` absorption + `¬`); `24` states no rule `41 §6` omits, so the two
  files cannot contradict. Runtime `⇒` is **not** a separate connective row: a
  propositional `⇒` is `Π`/`apply` (`16 §1.3`), already covered by the
  strict-position `apply unknown u = unknown` (it is not pinned independently).
  The absorbing rows are the eliminator-branch laziness of `42 §4`: an `unknown`
  **scrutinee** propagates, but an `unknown` in an **untaken** arm is discarded
  (`∧false`/`∨true` are decided by the *other*, known operand without forcing
  the hole — the connective is non-strict in the absorbing position, the
  CBV-laziness carry). The discriminating runtime test flips on **hole-present ⇒
  `unknown`** vs **hole-free ⇒ a definite value**; a hole-free program **never**
  yields `unknown` (`41 §6`, `42 §4` AC4).
- Holes are **precisely located** (provenance from `22 §1`) and carry their goal
  + context, so an agent (or the REPL "Little Prover", `21 §3`) can pick one
  up and try to fill it without re-deriving where it came from.

This unifies **obligation** (`22`), **typed hole** (this section), and **visible
postulate** (`18 §5`) into one concept: verification is *incremental*,
and what remains unproven is always explicit — statically (the hole in
`trusted_base()`) and dynamically (the propagating `unknown` value).

**The typed-hole diagnostic value:**

```
TypedHole {
  id       : HoleId           -- stable, deterministic (§6)
  goal     : Form             -- φ
  context  : [Binding]        -- Γ
  origin   : ObligationProv   -- provenance, from 22 §1
}
```

## 3. The three-region Heyting decomposition

For a predicate `φ : A → Ω` the Heyting structure of `Ω` (`16 §1.3`) splits its
domain into **three** subobjects (not two — this is the intuitionistic
refinement of true/false). The regions are **keyed to V3's verdict** (the
cardinal rule), and because the classifier is **total** (`23 §2`) the three are
**exhaustive and disjoint by construction** — every classified goal/point gets
exactly one. Their Heyting *meaning* (the per-input justification) is:

```
  S_φ      proved TRUE   (verdict proved)     meaning: { x | φ x }
  S_{¬φ}   proved FALSE  (verdict disproved)  meaning: { x | ¬ φ x }
  unknown  neither       (verdict unknown)    meaning: residual; its
                                              double-negation-stable core is
                                              { x | ¬¬ φ x } ∖ S_φ
```

- `S_φ` and `S_{¬φ}` are disjoint (`φ ∧ ¬φ ⊢ Empty`); `S_φ ⊆ S_{¬¬φ}` (`φ ⊢
  ¬¬φ`); and `S_{¬φ}` is disjoint from `S_{¬¬φ}` (`¬φ ∧ ¬¬φ ⊢ Empty`).
- **`unknown` is the residual** — *neither proved nor refuted* — which is what
  §1's "no world forces `φ` and none forces `¬φ`" describes. The set
  `{¬¬φ} ∖ S_φ` is its **characteristic core**, not its full extent: it names
  the goals where `¬¬φ` *is* established but `φ` is not (the canonical
  `¬¬φ ⇒ φ` gap), and `unknown ⊇ {¬¬φ} ∖ S_φ` also covers a goal where even
  `¬¬φ` is not yet forced. Keying the region to the verdict (not to the set
  membership) is what keeps §1 and §3 from diverging on such a point — both
  defer to V3's `unknown`.
- In a **Boolean** algebra the `unknown` core is **empty** (`¬¬φ ⇒ φ`); in a
  **Heyting** algebra it is **nonempty** — exactly the content of `¬¬φ ⇒ φ`
  *failing* (`16 §1.3`: the connectives are intuitionistic, excluded middle
  holds only for *decidable* props as data, `16 §1.3` `Decidable`). This is the
  home of a classically-valid-but-intuitionistically-unprovable goal.

The diagnostic reports, for a failing goal or class of inputs, **which region it
lands in** — *read off V3's verdict* (the cardinal rule), not recomputed:

- in `S_{¬φ}` (verdict `disproved`) → a genuine counterexample; **fix the code
  or the spec** (§1's `false`; §4 does **not** apply — you do not add a
  hypothesis to repair a refuted goal).
- in the `unknown` region (verdict `unknown`) → the property may hold but the
  current facts do not force it; **strengthen the context** (§1's countermodel
  atom, §2's hole, §4's slice).

**The cross-region invariant (Glivenko — the consistency-sweep anchor).** A
**classically-valid** `φ` **cannot** land in `S_{¬φ}` (`¬φ` is unprovable, so no
point is refuted) — it is `S_φ` or `unknown`, never `false`. `p ∨ ¬p` and
`¬¬p ⇒ p` are both in the `unknown` region, **not** `S_{¬φ}`; any diagnostic
placing such a goal in `S_{¬φ}` is a fidelity bug. Conformance groups cases by
this metatheoretic class and asserts verdict-agreement across the group, not
just per-case (`23 §5`).

## 4. Slice contextualization (the missing hypothesis) — `unknown` only

For a goal in the **`unknown`** region (never `S_{¬φ}`, §3), `φ` is not valid in
the ambient context but **is** valid under an extra assumption — categorically,
it holds in a **slice** `𝓒/Y` though not in the base. The diagnostic surfaces
that bridge:

```
  φ fails in Γ, but Γ, h : ψ ⊢ φ holds.
  → "Your claim holds once ψ is assumed. Add `requires ψ` (or prove ψ)."
```

- The normative claim is **sufficiency**: the prover reports an additional
  hypothesis `ψ` such that `Γ, ψ ⊢ φ` makes V3 return **`proved`** — a
  verdict-flip (`unknown` without `ψ` → `proved` with `ψ`). It is found from the
  lemmas and refinements in scope, or §1's unforced countermodel atom (the
  world/atom `w*` whose forcing closes the goal), and reported as the **missing
  precondition**. This converts "unprovable" into "provable *if* you assume ψ" —
  a concrete, often one-line repair. (Reporting the **minimal** such `ψ` is a
  *quality* SHOULD, §6 — not the contract: a sufficient-but-non-minimal `ψ` is
  still correct.)
- It applies **only** to the `unknown` region: a refuted goal (`S_{¬φ}`,
  `false`) is not repaired by adding a hypothesis (it has a genuine
  counterexample — you fix the code/spec, §1/§3). Surfacing a missing hypothesis
  for a `false` goal would contradict §1's discriminator and is a fidelity bug.
- This is the most agent-friendly diagnostic: it frequently *is* the fix (add
  the precondition, narrow the input refinement, or discharge `ψ` as a
  sub-lemma).

## 5. Suggested actions

Every diagnostic is accompanied by **`suggested_actions`** — an ordered,
machine-readable list of concrete next steps, derived from §1–§4 and **tagged
with the region they apply to** (so an `unknown`-only action never appears for
a `false` goal — the §3/§4 fidelity constraint):

- `add_precondition ψ` (`unknown`; from §4 slice / §1 countermodel atom) — often
  with the exact `requires` clause to insert.
- `strengthen_refinement {x:A|φ'}` (`unknown`) on a parameter.
- `provide_lemma γ` (`unknown`; for an open hole, §2) — with the lemma's
  statement.
- `case_split e` / `induct_on x` (`unknown`; prover stalled on a missing case
  analysis — the §2 sub-obligation-descent leaf, `23 §5`).
- `fix_counterexample x` (**`false`**; from §1/§3 `S_{¬φ}`) — the input class
  that genuinely fails.

Suggested actions are **hints, not commands**: an agent applies, adapts, or
ignores them. They are what turns a verification failure into a *conversation*
the agent can drive (strategy G7). They never alter the verdict (the cardinal
rule); they propose how to *change the program/spec* so a re-run reaches
`proved`.

## 6. Determinism and minimality

- Diagnostics MUST be **deterministic** for a given program + spec + prover
  version (same input → same countermodel, same hole ids, same suggested-action
  order), so an agent can diff runs. Hole ids are stable functions of the
  obligation provenance (`22 §1`), not allocation order.
- They SHOULD be **minimal**: the smallest countermodel (fewest worlds/atoms),
  the fewest missing hypotheses, the most-local hole — to keep the repair signal
  sharp. Minimality is a *quality* property (a non-minimal diagnostic is still
  correct), so conformance asserts determinism (hard) and checks minimality as a
  monotone preference, not an equality.
- Diagnostics are produced from **untrusted** machinery; a wrong diagnostic
  misleads but **cannot** make an unsound program check (the kernel is
  unaffected — the cardinal rule and `23 §1.3`). The failure mode V4 must guard
  is **infidelity** (mislabeling `unknown` as `false`, or emitting a diagnostic
  for a `proved` goal), not unsoundness.

## 7. What WS-V must deliver here (V4)

The four diagnostic mechanisms as **structured, machine-readable values** (the
data an agent consumes; the wire serialization is `25`, out of scope):

1. **Kripke countermodels** (§1) with the `false`-vs-`unknown` discriminator
   read faithfully from V3's verdict (`disproved` ⇒ forces `¬φ` ⇒ `false`;
   `unknown` ⇒ no forcing world ⇒ `unknown`).
2. **Typed holes** (§2) as visible postulates in `trusted_base()` with `unknown`
   runtime propagation per the Kleene/Heyting table (`41 §6`, `42 §4`).
3. The **three-region** Heyting decomposition (§3) — proved / false / `unknown`,
   the `unknown` region being the `¬¬φ` gap.
4. **Slice / missing-hypothesis** contextualization (§4), `unknown`-only.

plus `suggested_actions` (§5, region-tagged), determinism and minimality (§6).
The load-bearing properties, all of which conformance pins:

- **Fidelity to V3 (the cardinal rule):** every diagnostic reflects V3's
  *actual* verdict + evidence (countermodel / hole), consumed **unchanged** —
  never relabel `unknown` as `false` (verdict-flip case).
- **`false` vs `unknown` is honest** (§1): a refuted goal → countermodel tagged
  `false`; a `¬¬φ`-gap goal (e.g. `p ∨ ¬p`) → `unknown`, **not** `false`
  (verdict-flip + the cross-case metatheory-consistency sweep, §3 / `23 §5`).
- **Typed holes run** (§2): a program with an open hole type-checks + runs;
  `unknown` propagates per the table; hole-free ⇒ no `unknown`.
- **Three regions partition** (§3) and a fully-`proved` program emits **zero**
  diagnostics (AC5; V3's pipeline unaffected).

Acceptance ties to **G4**: every failed obligation emits a schema-valid
diagnostic (`25`) carrying a countermodel or typed hole + region-tagged
suggested actions, and a partially-verified program runs propagating `unknown`.
Conformance: `../../conformance/verify/diagnostics/`.
