# Proof-failure diagnostics

> Status: **DRAFT v0**. Normative for the four diagnostic mechanisms and their
> meaning; concrete serialization is in `25-protocol.md`. Contract for WS-V
> **V4**. This is the feature that most differentiates Ken for agentic use: a
> proof that does not go through yields a *structured, machine-readable
> explanation*, not an opaque error (digest §8 — "high value for agents").

When an obligation `Γ ⊢ φ` (`22`) is not discharged, the verification layer
emits one or more of four diagnostics, each derived from the **topos/Heyting
structure** itself (so they are principled, not heuristic). All are carried over
the protocol (`25`).

## 1. Kripke countermodels

When an FO obligation fails, the Kripke embedding (`23 §4`) gives Z3 a
**falsifying model of `φ#`** — i.e. a finite **Kripke countermodel** for `φ`:

```
worlds:   w0 ≤ w1 ≤ …            (stages of knowledge)
forcing:  which atoms hold at each world
failure:  the world w* and the subformula of φ not forced there
```

Why it is richer than a classical counterexample: a Kripke model distinguishes

- **φ is false** — some world forces `¬φ` (there is positive evidence against
  it), from
- **φ is unknown** — no world forces `φ` and none forces `¬φ` (the information
  is simply absent).

This `false` vs `unknown` distinction is invisible to a Boolean counterexample
and is exactly what an agent needs to decide whether to *fix the code* (it's
false) or *supply more facts* (it's unknown). The diagnostic names the **world
and atom** whose change would make `φ` go through — directly actionable.

## 2. Typed holes and `unknown` propagation

An undischarged obligation becomes a **typed hole**:

```
  ?h : φ        in context Γ
```

- The program **still type-checks and runs.** A hole is admitted to the
  environment as a **postulate** of type `φ` (`../10-kernel/11 §4`,
  `../10-kernel/18 §5`) — so it appears in the **trusted base**: the system is
  *honest* that `φ` is *assumed, not proved*. Shipping a verified artifact means
  zero open holes (or an explicit, recorded acceptance of the listed
  postulates).
- **`unknown` propagation (runtime).** Evaluating an expression whose result
  depends on an open hole yields the runtime third value **`unknown`**
  (`../40-runtime/42-evaluation.md`), which propagates through subsequent
  computation (Kleene/Heyting-style: `unknown ∧ false = false`, `unknown ∨ true
  = true`, `unknown` otherwise). So a partially-verified program *runs and shows
  you where the unproven property actually bites* — the Hazel "total error
  localization, program still runs" model the digest cites.
- Holes are **precisely located** (provenance from `22`) and carry their goal +
  context, so an agent (or the REPL "Little Prover" loop, `21 §3`) can pick one
  up and try to fill it without re-deriving where it came from.

This unifies "obligation," "typed hole," and "visible postulate" into one
concept: verification is *incremental*, and what remains unproven is always
explicit.

## 3. The three-region Heyting decomposition

For a predicate `φ : A → Ω` the Heyting structure splits its domain into
**three** subobjects (not two — this is the intuitionistic refinement of
true/false):

```
  S_φ          = { x | φ x }            proved TRUE
  S_{¬φ}       = { x | ¬ φ x }          proved FALSE  (refuted)
  unknown      = { x | ¬¬ φ x } ∖ S_φ   neither — the φ / ¬¬φ gap
```

(In a Boolean algebra the unknown region is empty; in a Heyting algebra it is
the content of `¬¬φ ⇒ φ` failing, `../10-kernel/12 §5.2`.) The diagnostic
reports, for a failing goal or a class of inputs, **which region it lands in**:

- in `S_{¬φ}` → there is a genuine counterexample; **fix the code or the spec**.
- in the `unknown` region → the property may well be true but the current facts
  do not force it; **strengthen the context** (add a hypothesis / lemma) — this
  is where §1's countermodel and §4's slice help.

This three-way verdict (proved / disproved / unknown) is the surface rendering
of the kernel trichotomy (`../10-kernel/12 §5.2`) and is the spine of the
protocol's verdict (`25`).

## 4. Slice contextualization

Sometimes `φ` is not valid in the ambient context but **is** valid under an
extra assumption — categorically, it holds in a **slice** `𝓒/Y` though not in
the base. The diagnostic surfaces that bridge:

```
  φ fails in Γ, but Γ, h : ψ ⊢ φ holds.
  → "Your claim holds once ψ is assumed. Add `requires ψ` (or prove ψ)."
```

- Concretely the prover searches for a **minimal additional hypothesis `ψ`**
  (from the available lemmas, the refinements in scope, or a failed atom of the
  countermodel) that closes the goal, and reports it as the **missing
  precondition**. This converts "unprovable" into "provable *if* you assume ψ" —
  a concrete, often one-line repair.
- This is the most agent-friendly diagnostic: it frequently *is* the fix (add
  the precondition, narrow the input refinement, or discharge `ψ` as a
  sub-lemma).

## 5. Suggested actions

Every diagnostic is accompanied by **`suggested_actions`** — an ordered,
machine-readable list of concrete next steps, derived from §1–§4:

- `add_precondition ψ` (from §4 slice / §1 countermodel) — often with the exact
  `requires` clause to insert.
- `strengthen_refinement {x:A|φ'}` on a parameter.
- `provide_lemma γ` for an open hole (§2), with the lemma's statement.
- `fix_counterexample x` (from §1/§3 `S_{¬φ}`) — the input class that genuinely
  fails.
- `case_split e` / `induct_on x` when the prover stalled on a missing case
  analysis.

Suggested actions are **hints, not commands**: an agent applies, adapts, or
ignores them. They are what turns a verification failure into a *conversation*
the agent can drive (strategy G7).

## 6. Determinism and minimality

- Diagnostics MUST be **deterministic** for a given program + spec + prover
  version (same input → same countermodel, same hole ids), so an agent can diff
  runs.
- They SHOULD be **minimal**: the smallest countermodel, the fewest missing
  hypotheses, the most-local hole — to keep the repair signal sharp.
- Diagnostics are produced from **untrusted** machinery; a wrong diagnostic
  misleads but cannot make an unsound program check (the kernel is unaffected).

## 7. What WS-V must deliver here (V4)

The four mechanisms — Kripke countermodels (false-vs-unknown), typed holes as
visible postulates with `unknown` runtime propagation, the three-region Heyting
decomposition, slice/missing-hypothesis contextualization — plus
`suggested_actions`, determinism, and minimality. Acceptance ties to **G4**:
every failed obligation emits a schema-valid diagnostic (`25`) with a
countermodel or typed hole + suggested actions, and a partially-verified program
runs propagating `unknown`. Conformance:
`../../conformance/verify/diagnostics/`.
