# WP `fn-lemma-partition` — make `fn` mean "computes" and `lemma`/`proof` mean "proof-irrelevant" (enforced)

**Owner:** Language team (elaborator gate + prelude migration) ·
**Foundation** advisory on catalog Ω-valued-`fn` sites · **Spec enclave**
for the §33 normative rule · **Architect** surface + fidelity review. ·
**Size:** M · **Risk:** low (zero-soundness surface — a surface gate +
behaviour-preserving relabeling). **Depends on:** `proof-vocabulary`
**change A** (recursive `lemma`/`proof` under SCT) **merged** — HARD gate,
see below. Absorbs `proof-vocabulary`'s deferred `_ind`→recursive-`lemma`
collapse.

## Objective

Complete the proof vocabulary's **fourth rule**: partition the term-definition
space by result sort so the keyword is a reliable signal —

- `fn` / `const` **compute**: their result type must **NOT** classify at Ω.
- `lemma` / `proof` are the **only** Ω-valued (proof-irrelevant) definitions.

This makes the **erasure boundary** a surface invariant you can read off the
keyword: proof-**relevant** constructions (`Or`/`Dec`/`Sigma` → `Type` → stay
`fn`) vs proof-**irrelevant** ones (`Equal`/`IsTrue` → Ω → `lemma`). The
pedagogic prototype (`origin/main @ 288b3979`) already applied exactly this
discrimination **by hand**; this WP **enforces** it, and closes the last "proof
misdressed as computation" gap (today an Ω-valued `fn` like
`refl_leq_nat_ind : Equal Bool (leq_nat x x) True = …` is legal).

## Settled design — FIXED inputs

Architect ruling **evt_6eyh27x6p7h0x** (endorse; grounded on `origin/main`, not
inference), answering the Steward consult **evt_5hcyz9q03y7cz**. Do not
relitigate; cite the ruling.

**(1) The partition is CLEAN — no legitimate Ω-valued `fn` exceptions.**
Grounded in the kernel sort model: `Term::Type(Level)` and `Term::Omega(Level)`
are **distinct universe constructors** (`crates/ken-kernel/src/term.rs:230/237` —
`Omega_l : Type(suc l)`, predicative/level-indexed; Ω is not a member of
`Type l`). The only variable in a sort is `Level`, which ranges over 0/1/2… —
**never over the Type-vs-Ω distinction**. So there is no sort-polymorphic binder
whose result is Ω in one instantiation and `Type` in another; every `fn`/`const`
result type classifies at a **determinate Type-or-Ω sort at definition time**
(the level may be variable; the Type-vs-Ω constructor is not). The check is
therefore `ensure_NOT_omega` = "result sort is `Term::Type(_)`, not
`Term::Omega(_)`" — decidable and exception-free. HK / universe-polymorphic `fn`s
are always `Type l`, never Ω.

**(2) `const` partitions identically; `def`/`prop` are orthogonal.**
An Ω-valued `const` (e.g. the old `const two_leq_three : IsTrue … = tt`) is a
proof → migrates to `lemma`; the pedagogic prototype already did exactly this
`const`→`lemma` migration, so it is proven behaviour-preserving. `def T = A`
defines a **type** (alias/refinement) — even `def MyProp = Equal Bool x True`
names an Ω-*former*, not a proof term. `prop P : Omega where {…}` declares an
Ω-prop **family** (a type former + intros), not a proof term. Neither is in the
term-definition space this rule partitions; both stay untouched.

**(3) Zero-soundness surface — CONFIRMED (grounded).**
`fn`/`const` and `lemma`/`proof` both bottom out in the same kernel
`declare_def` (transparent global) — the kernel carries **no** `fn`-vs-`lemma`
tag. Erasure operates on the **checked core** (`CheckedCorePackage` /
`RuntimeProgram`, `erasure.rs`) — keyword-free — and partitions runtime-target
vs erased declarations by their **Ω sort**, not the surface keyword (today's
Ω-valued `fn`s already erase correctly, which is only possible if erasure is
sort-driven). So `ensure_NOT_omega` is a **pure surface gate**, exactly
symmetric to the existing `ensure_omega_type` (`elab.rs:5396`) that
`lemma`/`proof` already run. Nothing downstream keys on `fn`-vs-`lemma`. Same
category as `proof-vocabulary`'s telescope relaxation — **not** routed through a
kernel/Spec soundness gate.

**(4) Migrate-then-enforce, atomically — the load-bearing sequencing input.**
The gate must **NOT** land before the migration, or it rejects the existing
tree. Land the migration **and** the gate in **one change**, so `origin/main` is
**never** in a rejecting state. Migration scope is **ALL** Ω-result `fn`/`const`
— **catalog AND the Rust-emitted prelude** (grep the *emission* in
`crates/ken-elaborator/src/prelude.rs`, not just `.ken.md` — a prelude
proof-helper defined as an Ω-valued `fn` would trip the gate at bootstrap). And
**verify none is in a mixed `fn`↔`proof` cycle** — those cannot become `lemma`
until `proof-vocabulary`'s deferred mixed-cycle support lands; today's catalog is
clean (the `_ind` helpers are self-recursive/homogeneous), so this is a **guard
to assert, not a blocker**.

## Dependency gate (Steward-held; do not kick early)

This WP **cannot build until `proof-vocabulary` change A is merged to
`origin/main`**: recursive proofs are today expressible **only** as Ω-valued
`fn`s (the `_ind` helpers), so forbidding Ω-valued `fn` before recursive `lemma`
exists would make them **inexpressible**. A is what lets each `_ind` fn become a
single recursive `lemma`. The Steward releases this WP only once A shows green on
`main`; until then it stays authored-but-held.

## Scope — three co-landing parts (one branch, atomic)

1. **The gate (elaborator).** Add `ensure_NOT_omega` on the `fn`/`const` result
   sort, symmetric to `ensure_omega_type` (`elab.rs:5396`): reject a `fn`/`const`
   whose result type classifies at `Term::Omega(_)`, with a diagnostic that names
   the vocabulary rule ("`fn`/`const` compute; use `lemma`/`proof` for an
   Ω-valued definition").
2. **The migration (catalog + prelude).** Relabel **every** Ω-valued `fn`/`const`
   to `lemma`/`proof`, catalog **and** prelude emission. This **subsumes**
   `proof-vocabulary`'s deferred `_ind`→recursive-`lemma` collapse: each `_ind`
   helper is exactly an Ω-valued `fn` this gate rejects, and (under A) it becomes
   a single recursive `lemma`, retiring the thin-wrapper idiom. Behaviour-
   preserving relabeling of already-checked Ω terms — preserve name + signature +
   proof term at every site (the pedagogic-rewrite fidelity net).
3. **§33 spec edit (Spec enclave).** Transcribe the normative rule: a `fn`/`const`
   result type **must not** classify at Ω; `lemma`/`proof` are the only Ω-valued
   term definitions. A spec edit (state the rule), **not** a soundness review.

## Out of scope (keep fail-closed narrow)

- **Mixed `fn`↔`proof` mutual cycles** — an Ω-valued `fn` caught in a mixed
  cycle cannot migrate to `lemma` until `proof-vocabulary`'s deferred
  mixed-cycle support lands. Assert the catalog + prelude contain none today
  (they do not); do not attempt to support them here.
- `def`/`prop` (orthogonal per input 2). No change.

## Acceptance criteria (testable)

1. **Gate rejects (positive).** A `fn`/`const` with an Ω-classifying result
   type (e.g. `fn f (x : Nat) : Equal Bool (leq_nat x x) True = …`) is
   **rejected** with the vocabulary diagnostic — assert the **specific** error
   variant, not bare `is_err`.
2. **Gate admits (negative).** A proof-relevant `fn` whose result is `Type`
   (`total_leq_nat : Or …`, any `Dec`/`Sigma`-valued `fn`) still
   **elaborates green** — the gate keys on the Ω sort, not on "looks like a
   proof".
3. **`const` arm.** An Ω-valued `const` is rejected identically; a `Type`-valued
   `const` is admitted.
4. **Migration complete & green.** After migration, the **full catalog +
   prelude bootstrap** elaborate green with the gate active — i.e.
   `origin/main` is never in a rejecting state (migrate-then-enforce holds in
   the single branch). No Ω-valued `fn`/`const` remains anywhere (catalog grep
   **and** prelude-emission grep both clean).
5. **`_ind` collapse.** The `proof-vocabulary` `_ind`+thin-wrapper pairs
   (NatArith/OrdNat) become single recursive `lemma`s; each preserves the public
   law's name + signature + proof content (fidelity net).
6. **Zero TCB.** `trusted_base_delta` unchanged; no `Axiom`/`postulate`/`sorry`/
   `declare_`/`trusted_base` added; the gate is a surface `ensure_*`, not a
   kernel change.
7. **Mixed-cycle guard.** A test (or an explicit audit note in the WP close)
   asserts no migrated def sits in a mixed `fn`↔`proof` cycle.

## Do-not-reopen guardrails

- The partition is **exception-free** (input 1) — do not add an "allow Ω-valued
  `fn` for X" escape hatch; there is no legitimate case (grounded in the
  distinct-universe sort model).
- The gate is **zero-soundness surface** (input 3) — do not route it through a
  kernel/Spec **soundness** gate; it is an `ensure_*` swap + a fidelity net +
  the §33 rule transcription.
- **Migrate-then-enforce is atomic** (input 4) — never land the gate in a commit
  that leaves `main` rejecting its own catalog/prelude.
- **Mixed `fn`↔`proof` cycles stay OUT.**

## Review lanes

- **Gate + migration:** Architect **surface + fidelity** review (the gate is
  surface; the migration is behaviour-preserving `fn`→`lemma` relabeling of
  already-checked Ω terms — same net as the pedagogic rewrite: name +
  signature + proof term preserved per site) + a conformance/fidelity pair.
  **No** kernel/Spec soundness gate.
- **§33 rule:** Spec enclave transcribes the normative rule statement (spec edit).

## Notes

This is the capstone of the proof-vocabulary initiative: with A (recursive
`lemma`/`proof`) landed and this gate in place, `fn` provably means "computes"
and `lemma`/`proof` mean "proof-irrelevant" — the erasure boundary is a surface
invariant, not a convention. Ownership across the elaborator gate (Language),
the prelude emission (Language, `prelude.rs` is theirs), and the catalog
migration (Foundation's lane) co-lands in one branch for migrate-then-enforce
atomicity; the Steward finalizes the single owning team at kick-time (when A
lands), pulling the other team's advisory as needed. Sequenced by the operator;
timing is the operator's call.
