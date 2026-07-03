# dependent-match-nonnullary — generalize dependent-motive recovery past the nullary gate (Map Gap B)

**Steward frame → Team Language (build). ELABORATOR LANE — build-completeness
against ALREADY-SPECIFIED behavior** (the `sct-completeness` (a) shape: the spec
commits to the general case, the elaborator under-implements it). **NOT an open
design question** — the destination is written: `spec/30-surface/34-data-match.md
§3.2` (Dependent-motive recovery) + `§3.3` (per-branch definitional refinement)
specify the *general*, non-nullary mechanism with **zero nullary qualifier**.
Owner: **Language**. Gate: **Architect approach-review — DONE, APPROVE**
(`evt_6s799tvr9s02a`, ruling grounded against landed `elab.rs`/`check.rs`/
`inductive.rs`) → **Language builds** → **Architect soundness** on the candidate +
**Language QA** + **CI**; **spec-leader confirmed no `/spec` change** (`34` already
covers it, no nullary qualifier). Findings → **Steward**.

Base: `origin/main`. Branch (pre-staged by Steward):
**`wp/dependent-match-nonnullary`**.

## ★ RATIFIED DECOMPOSITION (Architect approach-review `evt_6s799tvr9s02a`) — build to THIS

**VERDICT: sound to build, ELABORATOR-ONLY, kernel-backstopped, fail-closed. Zero
kernel change, zero `trusted_base` delta.** But the decomposition is **NOT "flip
the nullary gate"** — that framing (below, in "The gap") is *superseded by this
section*. The load-bearing work is **induction-hypothesis (IH) slot emission**.

**Why gate-flip alone fails (the mechanism the build must get right):**
`check_match_dependent`'s **field-telescope narrowing is ALREADY written** — it
pushes each `ctor.args[j]`, reconstructs the concrete `Cₖ field̄`, narrows
`expected` by substituting the field-carrying constructor, and checks the arm
against it. **Do NOT rebuild that.** What it **omits** is the IH binders: it wraps
each method in exactly `n` field-lambdas and **no IH lambdas**. But the kernel's
`method_type` (`ken_kernel::inductive`, ~L202) **requires** IH slots — the method
type is `Π(fields) Π(ih₁…ih_p). M (Cₖ field̄)` for `p = recursive_args(c).len()`.
So a naive gate-flip emits `λfields. body : (fields)→M(Cₖ…)` against an expected
`(fields)(ihs)→M(Cₖ…)` ⇒ **kernel `TypeMismatch`** (it peels the field-lambdas,
then expects an IH-`Π` and finds a non-function) ⇒ **REJECTED for every recursive
ctor** (`Cons`/`Node`/`Suc`). (A non-recursive non-nullary ctor like `Some x` has
`p=0` and would already pass — but Map's `Tree`/`List` are **recursive**, so IH
emission is **required, not optional**.)

**Mechanism pin (shovel-ready — this IS the build):**
1. **Emit the IH telescope.** After pushing the `n` field types, compute the `p`
   IH types via the kernel's OWN producer **`ken_kernel::inductive::recursive_args`**
   — the *exact* function `method_type` uses (incl. W-style `Π`-branching) — and
   wrap each method in `n` field + `p` IH lambdas **matching `method_type`'s
   `[args, ihs]` order**; **shift the pattern-var de Bruijn indices by `p`**.
   **REUSE `recursive_args`; do NOT re-derive recursive-arg detection** — a
   divergent local re-derivation under-accepts (fail-closed but silently breaks
   completeness): the `sct-completeness` (a) grep-the-producer lesson
   ([[named-floor-must-be-grepped-not-assumed]]). The IH-column machinery already
   exists in the constant-motive path (`compile_match_matrix`'s `ColKind::Ih`);
   **mirror/reuse it** rather than inventing.
2. **IHs are DEAD binders — NO IH-naming surface syntax needed.** Ken's recursion
   is self-recursive-`view` + SCT (the landed `toList`/`insert` shape), **not**
   eliminator-IH: the proof self-recurses (`toListOrdered l`, `toListOrdered r`) —
   SCT-checked structural descent on the subtrees — so the Elim's IH binders are
   **present-but-unused**. This is exactly why `toList`-ordered needs **only** this
   WP (Gap B) and why **no `\(h:Ty).` syntax is required** (see AC5, now DEFER).
3. **SCT interaction — build MUST confirm.** The dependent-motive Elim's
   self-recursive descent must still pass `sct_check` (the motive doesn't change
   the descent structure — same as landed `toList` — **but verify**, since this
   threads provenance through a dependent-motive Elim, the surface that
   `sct-completeness` (a) just hardened).

**SCOPE — non-indexed parameterized families ONLY (`List`/`Tree`).** These have no
result-indices (`infer_match` already builds `Term::Elim{indices:[]}` for them, so
`34 §3.2`'s "generalized over scrutinee **and indices**" is trivial here — zero
indices). Full **indexed-family (GADT)** dependent match additionally needs index
generalization + higher-order pattern unification (`39 §2.3`) +
impossible-branch/forced-arg refinement — a **distinct, materially harder
mechanism, a separate later WP.** Map / `map-verified-laws` do **not** need it; the
old "including indexed families" downstream line (below) **over-scoped** and is
retracted — keep this WP mechanical and bounded to non-indexed.

## The gap — the dependent-match motive builder is nullary-gated

> **⚠ SUPERSEDED BY THE RATIFIED DECOMPOSITION ABOVE.** This section's original
> "extend the narrower builder to ctor-arg telescopes / flip the nullary gate"
> diagnosis is *partly wrong*: per Architect's grounded review, the field-telescope
> narrowing is **already written** — the true gap is **IH-slot emission**. Kept
> below for the repro shapes and the spec grounding (`34 §3.2`/`§3.3`), which stand;
> build to the ratified mechanism, not to this section's implied "flip the gate."

`check_match_dependent` (the per-branch expected-type-narrowing motive builder)
is called **only** for `flat && nullary` scrutinee families; every non-nullary
family (`List a`'s `Cons`, `Tree`'s `Node`) falls to `infer_match`, which builds
a **constant** motive (no per-branch narrowing). Grounded three ways
(foundation-implementer's 2-line repro + spec-author + foundation-leader, all
independent, vs `origin/main`):

- **The gate** (`crates/ken-elaborator/src/elab.rs`, the `check`/`RMatch` arm,
  ~L447-461): `nullary = flat && <all constructors have empty args>`; `if nullary
  { check_match_dependent } else { infer_match }`. The in-code comment names the
  restriction and its reason outright: *"A parameterized/non-nullary family
  (`List a`'s `Cons`, what `isSorted`/`Perm` match on) is left entirely to the
  existing `infer_match` path … because this narrower builder hasn't been
  validated against ctor-argument telescopes yet (a real, contained follow-on)."*
  **This WP is that follow-on.**
- **`infer_match` builds a constant motive** (`elab.rs` ~L2703-2716) —
  `ret_ty` independent of the scrutinee, zero per-branch refinement.
- **Consequence:** hypothesis-narrowing induction over `List`/`Tree` is
  unsupported. A proof that threads `h : P xs` through `match xs { Nil => …;
  Cons x xs2 => … }` fails — `h`'s type never narrows to the `Cons`-branch form
  (minimal `allTrue`/`tailProof` repro, **no `leq` anywhere** — orthogonal to the
  transport gap). **No landed counterexample exists** — `Ord Bool`'s proofs work
  only because the *outer* match is itself nullary-dependent; that precedent
  cannot transpose to `List`/`Tree`.

**This is Map Gap B** — the second, more basic wall (distinct from the
transport/`J` Gap A that `surface-transport` addresses). It blocks `toList`-ordered
+ its lemmas (which never touch `leq`) and compounds with Gap A on the four
comparison-dependent `map-verified-laws` proofs.

## The target is specified — build to `34 §3.2`/`§3.3`, do not redesign

`34 §3.2`: `elim_D` takes a motive `M : (Δ_i) → D Δ_p Δ_i → Type ℓ'`; for a
**dependent** match the elaborator recovers `M` as the expected type
**generalized over the scrutinee `x : D Δ_p ī` and the indices `ī`**, solved by
higher-order pattern unification against the expected type (`39 §2.3`) — *"genuine
ambiguity is a surface error, never a guess"* (`39 §3`). `34 §3.3`: in the `cₖ`
arm the scrutinee is **definitionally** `cₖ field̄` by the ι-rule, and the
verification layer adds the **scrutinee equation** `(_ : Eq A s (cₖ field̄))` to
`Γ` (`../20-verification/22 §3`) — *this is exactly the hypothesis-narrowing* a
threaded `h : P s` needs (it becomes usable at `P (cₖ field̄)` via the equation).
**AC6** (`34 §3.3`) already pins per-branch refinement as required surface
behavior. The build implements the specified `M`-recovery for the **non-nullary**
case; it does not invent a mechanism.

## Soundness posture — FAIL-CLOSED via the kernel backstop (why this is lower-risk than the sibling WPs)

`check_match_dependent` is **elaborator-side**: it builds a motive `M` and emits a
`Term::Elim { motive: M, methods, scrut, … }` that **the kernel type-checks** —
motive well-formedness, each method against `M cₖ`, and the elim's result type
against `expected`. **A mis-built motive is kernel-REJECTED, never
unsound-accepted:** a wrong generalization makes some method fail to check against
`M cₖ`, or makes the elim's result type fail to match `expected` — either way the
kernel/outer-check rejects (over-rejection, the safe direction). The elaborator
**cannot** launder an ill-typed branch past the kernel by mis-generalizing the
motive; the kernel re-derives the per-branch types from `M` itself. So — unlike
`surface-transport` (touches the equality machinery) and `sct-*` (in the kernel) —
this WP's failure mode is *completeness* (a wrong motive rejects valid programs),
not *soundness*. The kernel is the backstop. **The soundness-relevant care is the
telescope substitution**: generalizing/abstracting `M` correctly when the
constructor binds fields (`Cons x xs2`, `Node l k v r`) — de Bruijn under the
arm's field telescope — is the exact thing the nullary gate deferred (the nullary
case has no fields to bind). Architect's approach-review vets that this
generalization matches `34 §3.2` and that any residual error remains
kernel-caught.

## Acceptance criteria

- **AC1 — CAPABILITY: non-nullary dependent induction elaborates AND kernel-checks.**
  The `allTrue`/`tailProof`-shaped repro (thread `h : P xs` through `match xs {
  Nil => …; Cons x xs2 => … }`, no `leq`) **elaborates and kernel-checks**; the
  per-branch scrutinee equation (`34 §3.3`) narrows `h`'s type in the `Cons` arm.
  Include a `Tree`/`Node` shape (the Map carrier), not only `List`.
- **AC2 — SOUNDNESS: mis-typed dependent branch stays REJECTED, kernel-caught.**
  A **discriminating negative**: a dependent `match` whose `Cons`/`Node` arm
  supplies a term of the **wrong** (non-narrowed or wrongly-narrowed) type is
  **kernel-rejected** — grep that the emitted `Term::Elim` is the mechanism (the
  motive is a real `elim_D` motive the kernel checks, not an elaborator-side
  type assertion — [[kernel-backed-claim-grep-the-emission-not-the-name]]). This
  pins the fail-closed posture: the generalization only *adds* provability for
  genuinely well-typed branches, never accepts an ill-typed one. **Also grep the
  diff: NO `crates/ken-kernel/` file touched and NO new `Decl` variant** — this is
  a pure `ken-elaborator` emission change riding the *existing* `infer_elim` (the
  kernel already types dependent elimination over non-nullary families with zero
  nullary qualifier); `trusted_base()` unchanged.
- **AC2b — SCT descent still passes for the dependent-motive Elim.** The real
  `toList`-ordered lemma (self-recursive `view` over `Tree`/`List` subtrees under a
  dependent motive) **passes `sct_check`** — confirm the dependent motive does not
  perturb the provenance/descent structure vs. the landed constant-motive `toList`
  (mechanism-pin #3). If it does, that is a finding → Steward before shipping.
- **AC3 — spec fidelity: matches `34 §3.2`/`§3.3` exactly.** The recovered motive
  is the expected type generalized over the scrutinee + indices (`§3.2`); the
  per-branch scrutinee equation is added to `Γ` (`§3.3`/`22 §3`); genuine motive
  ambiguity is a **surface error, never a guess** (`39 §3`). **spec-leader
  confirms no `/spec` change** — this is implementation catching up to `34`.
- **AC4 — Monotone, no regression.** `cargo test --workspace` green
  ([[kernel-reduction-change-full-workspace-green]] — an elaborator change to a
  live prelude-affecting path is workspace-wide). Every match that elaborated
  before (all nullary-dependent + all `infer_match` cases) **still** elaborates
  identically; the change only routes **more** scrutinee families through the
  dependent path, adding provability without altering existing results.
- **AC5 — annotated-lambda syntax (`\(h:Ty).`): DEFER — do NOT fold** (Architect
  ruled, `evt_6s799tvr9s02a`). Grounded: the proof idiom is a recursive `view` with
  **top-level annotated params** + `match` under a dependent motive that puts the
  hypothesis IN the motive (goal `Ordered m → isSorted (toList m)`), so per-branch
  goal-narrowing delivers the `Node`-shaped hypothesis as a **checked-mode
  unannotated arm binder** (`Node l k v r => \ho. …`, domain from the narrowed
  goal-`Pi` — confirmed against the `RLam`-checked-against-`Pi` path, `elab.rs:390`).
  No inline typed lambda is needed. The parser addition (`parser.rs` ~L980-1009,
  bare binder names only) **stays with the separate surface-syntax WP**; add it
  there only if a concrete `map-verified-laws` proof later proves it necessary.
  **Do not bundle it here.**

## Guardrails (do-not-reopen)

- **Build to `34 §3.2`/`§3.3`, do not redesign.** The mechanism is specified; this
  is implementation-completeness, not a design WP.
- **Fail-closed is the whole soundness story — keep the kernel as backstop.** The
  emitted `Term::Elim` + motive is kernel-re-checked; **never** substitute an
  elaborator-side type assertion for the kernel's motive check (that would be the
  anti-pattern `lawful_classes.ken` rejects). A wrong motive must *reject*, never
  launder.
- **Genuine motive ambiguity is a surface error, never a guess** (`39 §3`).
- **Do not fold the annotated-lambda syntax** — Architect ruled it DEFER (AC5);
  it stays with the separate surface-syntax WP.
- **Do not touch the kernel or add a `Decl` variant** — elaborator-only emission
  riding the existing `infer_elim`; `trusted_base()` unchanged (AC2).
- **Indexed families / GADTs are OUT OF SCOPE** — non-indexed `List`/`Tree` only.
- **REUSE `ken_kernel::inductive::recursive_args`** for IH detection — never
  re-derive it (grep-the-producer).
- **Ground every mechanism claim against landed `elab.rs`/`parser.rs`** — the
  line numbers here are perishable.

## Sequencing

- **Gate:** **Architect approach-review** (vet the telescope motive-generalization
  vs `34 §3.2`; confirm the fail-closed/kernel-backstop posture) → Language builds
  → **Architect soundness** on the candidate + **Language QA** + **CI**;
  **spec-leader confirms** no `/spec` change (`34` already specifies it — à la
  `sct-completeness` (a)/#5).
- **Lane:** Language / `ken-elaborator`. **Sibling to `surface-transport`** (Gap A)
  — the two are **separate WPs, not bundled** (distinct mechanisms/soundness
  surfaces: motive construction here, equality machinery there). Both are in the
  Language/elaborator lane; sequence per capacity. Disjoint from Kernel's
  `sct-reconstruction-descent` and the `[FS]` frame.
- **Downstream:** this WP is **one of the two hard gates for `map-verified-laws`**
  (the 5 deferred Map §5 laws). `toList`-ordered + its lemmas need **only** this
  (Gap B); the four comparison-dependent laws need **both** this and
  `surface-transport`. It also unblocks non-nullary dependent induction for the
  whole verified-`.ken` corpus (any proof by induction over **non-indexed**
  `List`/`Tree`-shaped families) — a foundational, reusable capability, not a
  Map-local patch. **(Indexed families / GADTs are OUT OF SCOPE — a distinct harder
  later WP; see the SCOPE pin in the ratified decomposition.)**
