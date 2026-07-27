---
id: V4-RESIDUAL
title: "The Kripke countermodel is an inert shell: it is never related to `φ` at all — no interpretation of the formula, no recursive forcing evaluator — and V3's prose `description` is stuffed into `FormRef`, a slot meant for a structural subformula reference"
status: active
owner: verify
size: L
gate: G2-G3
depends_on: []
blocks: [SEC1-IFC-R3]
github: null
origin: "Measured by the Steward 2026-07-27 at origin/main 2ebe232c while looking for Verify's next WP after V3-RESIDUAL (PR #1103) merged. V4 is BUILT (diagnostics.rs + protocol.rs + v4_acceptance.rs, 15 tests, no placeholders, all asserting) -- this node is its RESIDUAL, not the WP. Frame docs/program/wp/V4-diagnostics.md if one exists; spec/20-verification/24-diagnostics.md self-declares V4 elaborated / implementation-ready."
---

> ## ⛔ READ FIRST — V4 IS ALREADY BUILT. This node is the RESIDUAL.
>
> ⛔ **Do not implement diagnostics from scratch.** They exist:
> `crates/ken-elaborator/src/diagnostics.rs` (blob **`2d0f5e23`**),
> `src/protocol.rs` (**`ca026c6f`**), and
> `tests/v4_acceptance.rs` (**`3657e841`**, **15 tests**).
>
> ⭐ **The V4 suite is HONEST** — unlike V3's before `V3-RESIDUAL`. All 15 tests
> assert; **zero** are assertion-free, and none needs renaming. ⛔ Do not go
> looking for the V3 placeholder defect here; it is not present.
>
> ⚠ ⛔ **Do NOT sequence this from `spec/SPEC-PROGRESS.md`.** That column is
> unmaintained — 47 of 48 rows say `DRAFT` and `REVISED` has never been used.
> `24-diagnostics.md` itself says *"Status: **V4 elaborated**
> (implementation-ready)."*

## ⭐⭐ The objective — the shape is right and the CONTENT is inert

`KripkeCountermodel` (`diagnostics.rs:78–91`) declares exactly what `24 §1`
asks for:

```rust
pub struct KripkeCountermodel {
    pub verdict: DiagnosticTag,
    pub worlds:  Vec<WorldId>,           // [placeholder — V4-backend]
    pub order:   Vec<(WorldId, WorldId)>,// [placeholder — V4-backend]
    pub forcing: Vec<(WorldId, AtomId)>, // [placeholder — V4-backend]
    pub failure: Option<FailureWitness>, // [placeholder — V4-backend]
}
```

⛔ **But the only constructor fills it with a scaffold.** `from_v3`
(`diagnostics.rs:97–111`), verbatim:

```rust
worlds:  vec![WorldId("w0".to_owned())],  // minimal scaffold
order:   vec![],
forcing: vec![],                          // <-- EMPTY, ALWAYS
failure: Some(FailureWitness {
    world:      WorldId("w0".to_owned()),
    subformula: FormRef(v3.description.clone()),   // <-- PROSE IN A FORMULA SLOT
}),
```

Three separate defects fall out, and ⛔ **they are not the same defect three
times**:

1. ⭐⭐ **Nothing in the emitted model is connected to `φ` at all.** `24 §1` makes
   a refutation *mean* **"a model that forces `¬φ`"**, and the preamble's cardinal
   rule is explicit that *"a Kripke model that merely fails to force `φ` is
   **not** a refutation."* ⇒ The constructor never interprets `φ`, never builds a
   forcing relation over its atoms, and has no evaluator that could decide
   `forces(w, ¬φ)`. **The model is unrelated to the obligation it claims to
   refute.**

   ⛔ **The defect is NOT that `forcing` is empty.** See the AMENDMENT below —
   an empty atomic forcing relation is lawful and is frequently the *correct*
   witness. Do not "fix" this by populating `forcing`.
2. **`order` is empty and `worlds` is a singleton**, so there are no stages of
   knowledge and the monotonicity requirement (`w ⊩ P` monotone in `≤`) is
   vacuous — satisfied by having no content rather than by holding.
3. ⛔ **`FormRef(v3.description)` is a type confusion.** `description` is
   human prose from `Countermodel { description: String }`
   (`prover.rs:55–57`); `FormRef` is the slot for *"the subformula of `φ` not
   forced there."* A consumer that reads `failure.subformula` as a formula
   reference gets an English sentence.

### ⭐⭐⭐ THE LOAD-BEARING PART — why nothing has caught this, and it is not negligence

`KripkeCountermodel::verdict` carries this doc comment, and it is **correct and
load-bearing**:

> *"Copied from V3's verdict (the cardinal rule — never recomputed from
> `worlds`/`forcing`)."*

⭐ `24`'s preamble **requires** exactly that: V4 *explains* a verdict, it never
*re-decides* one, and reading the tag off the model instead of off V3 is the
named "V3-prover trap."

⇒ ⛔ **But that is also precisely why the empty `forcing` is invisible.**
**Nothing in the system ever reads `forcing` to decide anything.** An empty
forcing relation therefore never produces a wrong verdict, never reddens a test,
and never fails a gate. **The structure is inert: it is populated, serialized,
and never eliminated.**

⭐ **This is the general shape to name in the report: a representation node that
nothing consumes cannot be wrong, and cannot be right either.** The correct
architectural rule (`verdict` is copied, never derived) and the defect (the
model is empty) are **compatible**, which is what let a green 15-test suite ship
over it.

## ⚠ This is an ADVISORY-UX gap, NOT an unsoundness. Say so, and act accordingly.

`24`'s preamble is explicit: *"a V4 bug is an **advisory-UX** regression, never
an unsoundness (★★): the kernel already settled `proved`/not via the certificate."*

⇒ ⛔ **Do not treat this as a soundness incident, do not escalate it as one, and
⛔ do not "fix" it by letting the model influence a verdict.** ⭐ But do not
under-rate it either: `24`'s own preamble calls structured diagnostics *"the
feature that most differentiates Ken for agentic use — a proof that does not go
through yields a structured, machine-readable explanation, not an opaque
error."* An agent consuming `forcing: []` and a prose `FormRef` gets the opaque
error the chapter exists to eliminate.

## Fixed inputs (measured at `origin/main = 2ebe232c`; ⛔ re-derive at point of use)

| input | pin |
|---|---|
| diagnostics | `crates/ken-elaborator/src/diagnostics.rs` blob **`2d0f5e23`** — **10** × `[placeholder — V4-backend]` |
| V3 carrier | `crates/ken-elaborator/src/prover.rs` blob **`dafe77fe`** — `Countermodel { description: String }` at `:55–57` |
| wire | `crates/ken-elaborator/src/protocol.rs` blob **`ca026c6f`** — `serialize_countermodel` at `:170`; ⚠ note `:286` emits the literal `"[countermodel pending V4-backend]"` |
| tests | `crates/ken-elaborator/tests/v4_acceptance.rs` blob **`3657e841`** — 15 tests, all asserting |
| spec | `spec/20-verification/24-diagnostics.md` blob **`84b72c45`** — `§1` worlds/forcing/witness; preamble cardinal rule + Glivenko invariant |
| producer | `V3-RESIDUAL` (PR #1103) — `prover.rs::attempt_with_refutation` is the **first real** `Disproved` producer |

## ⛔⛔ AMENDMENT 2026-07-27 — `AC-V4r1` WAS WRONG. Read this before the table.

**Architect, blocking `7c6e92cb` on Decision `dec_7mctdjqs71jrm`.** The original
`AC-V4r1` demanded a **non-empty** `forcing` relation, on the stated ground that
otherwise no world could force `¬φ`. ⛔ **That ground is false under the spec's
own Kripke clause.** For an atomic `P`, a world forces `¬P` **precisely when no
accessible world forces `P`** — so the atomic forcing relation may legitimately
be **empty**, and emptiness is often the correct witness rather than a defect.

⇒ **A non-emptiness requirement is not just weak, it points the implementer at
the wrong artifact.** The first successor obeyed it by inventing an atom *named*
`not({phi:?})` and then defining success as "is that self-generated string
present at the witness world?" — circular, and it would validate identically for
**any** term. ⭐ **An AC that can be discharged by a string the implementation
chose itself is a self-authenticating label, not a property.**

**The requirement is now: `forces(model, witness, Not(φ)) == true` under a real
recursive forcing evaluator implementing the `23 §4` clauses over the supported
formula fragment.** ⛔ Never `!forcing.is_empty()`; ⛔ never the presence of an
atom whose *name* renders `φ`.

### Successor mechanism — binding (Architect)

1. **`FormRef` becomes a structural reference into the obligation formula** — a
   root-relative child path checked against `triple.phi` — ⛔ not a `String`
   rendering. A debug rendering of the whole `Term` has no node identity and
   cannot distinguish repeated subformulas.
2. **Forcing atoms are structural/canonical**, and a side-effect-free
   `forces(world, formula_ref)` evaluator implements the `23 §4` clauses. ⛔ Its
   result is **validation/advisory only** and must never move the copied V3 tag.
3. **Widen the V3 `Countermodel` / shared evidence carrier as needed** so V4
   consumes **actual structural refutation evidence** instead of ignoring
   `countermodel` and reconstructing a model from the verdict. For the present
   ground unequal-`Int` refutation: model the actual equality atom as **unforced
   at every accessible world** (or a structurally linked theory disequality),
   then demonstrate the recursive negation clause at the witness.
4. **Controls.** Keep the contradictory-content / no-tag-move control. ⭐ **Add
   the causal mutation that injects only the string `not({phi:?})` — semantic
   validation must still FAIL.** Plus rejects for: invalid `FormRef` path,
   unknown forcing world, preorder violation, monotonicity violation.

⚠ This remains an **advisory-UX/completeness** block, not a kernel-unsoundness
incident — no verdict becomes falsely `proved`. Exact `7c6e92cb` carries no
Architect approval; hold that branch and fold a fresh successor.

## Acceptance criteria

| AC | claim | control |
|---|---|---|
| `AC-V4r1` | ⭐⭐ **AMENDED — see above.** For a real refutation, **`forces(model, witness, Not(φ))` evaluates to `true`** under a recursive evaluator implementing the `23 §4` clauses, over a model whose atoms are **structurally derived from `φ`**. | ⛔ **`!forcing.is_empty()` does NOT discharge this and is not even necessary** — an empty atomic relation is lawful. ⛔ Neither does an atom named after `φ`. ⭐ **Required causal control: inject *only* the string `not({phi:?})` and show semantic validation still FAILS.** Then assert the specific structural `(world, atom)` facts the negation clause consumes |
| `AC-V4r2` | ⛔ **The cardinal rule is UNTOUCHED:** the `false`/`unknown` tag is still copied from V3's verdict and **never** recomputed from `worlds`/`forcing`. | ⭐ Control: feed a `KripkeCountermodel` whose forcing contradicts the tag and confirm **the tag does not move**. ⛔ If populating the model changes any verdict anywhere, **stop and re-raise** — that is the V3-prover trap and it is a regression, not a fix |
| `AC-V4r3` | ⭐ **STRENGTHENED 2026-07-27.** `failure.subformula` is a **structural** reference into `φ` — a root-relative child path checked against `triple.phi` — carrying node identity. | ⛔ **A `String` of any kind fails this, including `format!("{phi:?}")`.** Reddening on V3's prose proves only "not V3 prose"; it does **not** prove "real `FormRef`" — that mutation passed while the defect was intact one layer up. ⭐ Required: an **invalid-path reject**, and a case with **repeated subformulas** that the reference distinguishes |
| `AC-V4r4` | `order` and `worlds` describe genuine stages, and **forcing is monotone in `≤`**. | Construct a two-world model and assert monotonicity **holds**; then a deliberately non-monotone one and assert it is **rejected**. ⛔ A vacuous pass on the empty relation does not count |
| `AC-V4r5` | ⭐ **The Glivenko invariant holds: a classically-valid goal is NEVER tagged `false`.** | Take a classically-valid, intuitionistically-unprovable goal (`p ∨ ¬p`) and assert the verdict is `unknown` and the tag is **not** `false` (`24` preamble, `23 §5`, `16 §1.3`) |
| `AC-V4r6` | The wire form carries the structured model. | ⛔ `protocol.rs:286`'s literal `"[countermodel pending V4-backend]"` must no longer be reachable for a real refutation. Report whether it survives for any other path |
| `AC-V4r7` | No regression. | v1/v2/v3/v4/t1 and `sec1_acceptance` stay green **in CI**; ⛔ "no-regression" never means a local `--workspace` run |
| `AC-V4r8` | ⛔ **Trusted-base and kernel delta are ZERO.** | V4 renders; it does not decide. `trusted_base()` unchanged, no new primitive, no `Axiom` |

⛔ **`AC-V4r1` + `AC-V4r2` together are what discharge this node.** `r1` alone
invites populating the model by teaching it to decide; `r2` alone is already
true today. ⭐ The point is a model with **real content** whose content is
**still not consulted for the verdict**.

## Scope

**IN:** `KripkeCountermodel` and its construction path, `FailureWitness` /
`FormRef`, the `Countermodel` carrier where it must widen beyond a `String`,
`serialize_countermodel`, and the controls above.

⛔ **OUT:**
- ⛔ **Any path by which the model influences a verdict** — `AC-V4r2`. This is
  the one hard rule.
- ⛔ **Re-implementing V3's refutation arm** — it landed in PR #1103 and is real.
- ⛔ **Typed holes (`24 §2`) and the `unknown` region**, unless a shared type
  forces it — report, do not expand.
- ⛔ **No kernel enlargement, no trusted-base growth.**
- ⛔ `SEC1-IFC-R3` — this node **unblocks** it but does not contain it.

## ⛔ What this does and does NOT unblock downstream

`SEC1-IFC-R3` is `draft` and blocked on **`V3` AND `V4`**. `V3-RESIDUAL` supplied
the `V3` half. ⭐ This node supplies the `V4` half — **and that still does not
make `SEC1-IFC-R3` releasable on its own.** Its `AC-R3b` needs
`product(c, ζ)` and its `AC-R3c` needs a deliberately too-weak `Φ_post` to be
**detected**; neither follows from a countermodel being well-formed. ⛔ Report
what you observe about `sec1_acceptance`'s synthetic verdicts, ⛔ do not re-scope
that node yourself.

## Validation — ⛔ TARGETED ONLY

⛔ **NEVER `--workspace`** (operator, `agent/COORDINATION.md §12`). `-p
ken-elaborator`, `--test v4_acceptance` (plus `v1`/`v2`/`v3`/`t1`/`sec1_acceptance`
for `AC-V4r7`). Workspace, `--locked`, and conformance run **in CI**.

⚠ `ken-cargo` is a single machine-wide `flock`, slots == 1, and **Language,
Runtime and Kernel are all live on it** — coordinate the turn **in-thread**;
⛔ never sample `ps` to decide it is free.

## Clean-room

⛔ SMT/model-extraction patterns in `local/refs/` are **Spec-enclave-only —
never vendored, never consulted by the implementer** (`CLEAN-ROOM.md`).

## Reporting

Return exact SHA/tree/base, and specifically: **the `forces(model, witness,
Not(φ))` evaluation for `AC-V4r1` and the `not({phi:?})`-injection control that
must FAIL**; **the `AC-V4r2` control showing a contradicting model does NOT move
the tag**; the **structural-path** evidence and invalid-path reject for
`AC-V4r3`; the monotonicity pass **and** the non-monotone rejection for
`AC-V4r4`; and whether `protocol.rs:286`'s pending-literal remains reachable.

## ⚠ SYMPTOM INVENTORY

**NEXT PREDICATE CHECK = 3rd entry · NEXT RESEARCH PULL = 3rd hard-stop.**

**Entry 1 (hard-stop 1) — 2026-07-27.** `AC-V4r1` demanded a non-empty `forcing`
relation on a ground the spec's own Kripke clause falsifies; the implementer
satisfied it with a self-named atom and a validator that looked for that same
name. **Disposition:** Architect block on `dec_7mctdjqs71jrm`; AC amended above
to demand a real recursive `forces(…, Not(φ))` plus a causal control that FAILS
on the string injection. ⭐ **The shape to notice: the AC named a proxy
(non-emptiness) instead of the property, and the proxy was satisfiable by an
artifact the implementation authored itself.**
