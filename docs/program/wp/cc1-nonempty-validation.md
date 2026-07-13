# WP CC1 — `Data.NonEmpty` + `Data.Validation`

**Program II (catalog closure), step 1 of 9.** Owner: **Foundation**. Reviewer:
**Architect** (soundness/design). Size: **S–M**. Branch:
`wp/cc1-nonempty-validation` (off `origin/main @ 10e453aa` — the post-kenfmt-C
reformatted catalog). Thread all CC1 activity in its kickoff thread.

This is the **entry** of the catalog-closure chain (CC1 → … → CC9,
`ken-cli-tooling-work-program.md §Program II`). It lands two small, independent,
immediately-reusable ordinary-Ken packages that every later CC step and the
whole ArgParse/decoder line leans on. **Zero trust delta** — ordinary
kernel-checked Ken; no kernel rule, no primitive, no postulate.

## Fixed inputs (settled — do NOT reopen)

1. **Two packages, literate `.ken.md`, catalog pedagogic style.**
   - `catalog/packages/Data/NonEmpty/NonEmpty.ken.md`
   - `catalog/packages/Data/Validation/Validation.ken.md`
   Follow the landed catalog conventions (`catalog-style-guide.md`): top-down
   `def`/`prop`/`lemma`/`proof`, membership form (`proof … for S`), literate
   fence roles. Both files **must be `ken fmt`-clean** — the strict
   `ken fmt --check` gate is now live (kenfmt C, `origin/main @ 10e453aa`); run
   `ken fmt` before handoff or `cargo test --workspace --locked` fails.

2. **`NonEmpty a` is a genuine non-empty list.** Pin the carrier as an inductive
   with a single cons-onto-a-list constructor (head **plus** an ordinary
   `List a` tail), so "non-empty" is structural, not a runtime invariant. Provide
   exactly the operations with an obvious CC-chain consumer — `head`, `tail`,
   `toList`, `map`, and `append : NonEmpty a → NonEmpty a → NonEmpty a`. **No
   speculative helpers** (no `zip`, `reverse`, `sort`, … until a real consumer
   exists — the report's "extract only on two real consumers" rule).

3. **`NonEmpty` carries a lawful `Semigroup` via `append`.** The accumulation in
   `Validation` is *powered by* this. The `Semigroup (NonEmpty a)` instance
   **must carry its associativity law proof** — a lawful-class instance carries
   proofs, never postulates them (fleet lesson
   `lawful-class-instances-must-carry-law-proofs`;
   `carrier-canonicity-axis-for-lawful-class-laws`). If the landed catalog has no
   `Semigroup` class yet, define the minimal lawful one here (associativity as
   its law); if it exists, instantiate it.

4. **`Validation e a = Invalid e | Valid a`, with an ERROR-ACCUMULATING
   `Applicative` — this is the whole point.** It is shaped like `Either`/`Result`
   but its `Applicative` **accumulates** independent errors through
   `Semigroup e` (`Invalid e1 <*> Invalid e2 = Invalid (e1 <> e2)`), in contrast
   to `Result` + `Monad`, which **short-circuits** on the first error. The
   `Applicative (Validation e)` instance therefore **requires `Semigroup e`** and
   **must carry the Applicative law proofs** (identity, homomorphism,
   interchange, composition) to the catalog's lawful-class standard, plus the
   `Functor` laws.

5. **`Validation` is deliberately NOT a lawful `Monad`.** Accumulation is
   incompatible with the monad first-error `bind`. Provide `Functor` +
   `Applicative` only; add a short prose `note` in the package explaining *why*
   there is no `Monad (Validation e)` (a `bind` would have to pick one error and
   discard the rest, defeating accumulation). Do **not** ship a `Monad` instance.

6. **Generic over `Semigroup e` — do NOT depend on `Diagnostic`.** The canonical
   use is `e = NonEmpty Diagnostic`, but `Diagnostic` is **CC4**, not yet built.
   CC1's `Validation` stays parameterized over any lawful `Semigroup e`; the
   `NonEmpty Diagnostic` specialization lands later. Depend only on `Data.List`,
   the (possibly new) `Semigroup`/`Functor`/`Applicative` classes, and
   `Data.NonEmpty`.

## Mandated deliverable outline

Each section ends in a concrete, implementable choice — not a survey.

1. **`Data.NonEmpty`** — carrier (per fixed input 2), `head`/`tail`/`toList`/
   `map`/`append`, the lawful `Semigroup (NonEmpty a)` with associativity proof.
2. **`Data.Validation`** — `Validation e a` type, `Functor (Validation e)`,
   `Applicative (Validation e)` (accumulating, `Semigroup e` constraint) with law
   proofs, and the no-`Monad` note.
3. **A worked accumulation example** in the `Validation` package (literate
   `example` fence): validate a small record against **≥2 independent checks**
   and show the `Invalid` result carries **all** failures, not just the first —
   the property that distinguishes it from `Result`. Use a trivial
   `Semigroup e` (e.g. `NonEmpty String`) so CC1 stands alone without CC4.

## Acceptance criteria (testable)

- **AC1 — elaborates clean.** Both packages elaborate in the kernel; `cargo build
  --workspace --locked && cargo test --workspace --locked` green on the exact SHA
  (literal, per fleet CI discipline).
- **AC2 — `ken fmt`-clean.** Both files pass the live strict
  `ken fmt --check` corpus gate (they are new catalog files → in the gate's
  scope). Confirm `ken fmt` is a no-op on them.
- **AC3 — non-empty is structural.** There is no way to construct an empty
  `NonEmpty a`; `toList` of any `NonEmpty` is a non-empty `List`.
- **AC4 — Semigroup law proven.** `Semigroup (NonEmpty a)` carries a real
  associativity proof (kernel-checked, not postulated).
- **AC5 — accumulation is real and law-abiding.** `Applicative (Validation e)`
  carries its law proofs; the worked example demonstrates ≥2 independent errors
  accumulating into one `Invalid (e1 <> e2 …)` — explicitly contrasted (in prose)
  with `Result`'s first-error short-circuit.
- **AC6 — no `Monad (Validation e)`** instance exists; the prose note explains
  the incompatibility.
- **AC7 — scope discipline.** Only the two new catalog packages (+ a minimal
  `Semigroup` class iff one is not already landed); no kernel/prelude/`Cargo`/
  lock/`trusted_base` delta; no speculative helpers.

## Do-not-reopen guardrails

- Do not build `Diagnostic`, `Codec`, `Cursor`, or any later-CC piece — CC1 is
  `NonEmpty` + `Validation` only, generic over `Semigroup e`.
- Do not add a `Monad (Validation e)` "for symmetry" — its absence is the design.
- Do not postulate any class law — lawful instances carry proofs.
- Do not add operations without a concrete CC-chain consumer.
- Do not reformat or touch any file outside the two new packages (+ optional
  minimal `Semigroup` class file).

## Review & close

Foundation builds → QA (Foundation) → **Architect** review (soundness/design of
the accumulating applicative + the law proofs) → `git_request` to Steward →
honesty-gate + CI-poll publish → CC1 closed once retros are in. CV is **not** a
required reviewer unless the build touches `spec/` or `conformance/` (CC1 is pure
catalog and should not). CC2 (`Text.Codec`/`Text.Numeric`) follows in the
Foundation ring.
