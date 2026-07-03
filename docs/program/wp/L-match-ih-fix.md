# L-match-ih-fix — dependent-`match` over ≥2 same-typed recursive fields

**Steward frame → Team Language.** An **expedited** elaborator correctness fix
(VAL2 finding #5). **Root cause is pinned and the fix direction + soundness
posture are Architect-confirmed** (`evt_zsq8w9g48f8s`); this frame pins them as
fixed inputs. Owner: **language-leader → language-implementer → language-qa.**
Gate: **Architect soundness (4 criteria below) + Language QA + CI.** No spec/CV
vote — this is a completeness bug in the elaborator, `/spec` unchanged, kernel
untouched. Findings → **Steward**.

## Why
A user recursive `data` cannot be `match`ed when one constructor has **≥2 fields
of the same recursive type**: `data Tree = Leaf | Node Tree Char Tree` → `match`
fails `KernelRejected { TypeMismatch }`, even with constant arms. This blocks
binary trees, binary ASTs, any n-ary tree with ≥2 same-typed children —
**foundational** for a language built on inductive data + `match`. Surfaced by
VAL2's `tree-traversal`; currently held behind a `KNOWN-GAP.md`.

## Settled inputs — DO NOT REOPEN
- **Root-cause site** (`elab.rs`; *perishable — verify against the landed code at
  pickup*): `compile_match_matrix`'s `ColKind::Ih` branch (~2364-2394), via the
  `tail_codomain` helper (~2296-2322). For a ctor with 2+ recursive fields the
  elaborator lays out one induction-hypothesis (Ih) column per recursive field as
  flat siblings (`build_ctor_buckets` ~2515-2597: `[Real,Real,Ih,Ih]` for
  `N10 x y`). Each Ih slot's type is computed via `tail_codomain` over
  *everything still pending after that column* — which **wrongly sweeps in the
  next sibling Ih column**. `tail_codomain`'s full-Pi-fold is correct at its real
  use (the whole-method codomain at the split-column call site, ~2475-2480) but
  wrong reused at a single-Ih-binder site. Result: the first Ih slot is
  over-built as `Pi(ret_ty, ret_ty)` instead of flat `ret_ty` → the extra Π makes
  the expected method type stricter → kernel `TypeMismatch`. 0-1 recursive fields
  → no sibling Ih to sweep → `List`/single-field types unaffected.
- **This is a COMPLETENESS bug, provably — NOT soundness** (Architect-confirmed
  from the mechanism, not just the failure mode). The error is an **over-build**:
  it always *adds* structure (folds in the sibling Ih), and adding structure to
  an expected type can only **reject more, never accept more** — there is no
  input on which it under-accepts. The kernel is correctly rejecting a malformed
  `Elim` (fail-closed).
- **Fix direction (Architect-endorsed, matches correct dependent-eliminator
  semantics).** For `N10 : T10→T10→T10` with motive `M`, the method is
  `(l r : T10) → M l → M r → M (N10 l r)` — each Ih binder is **flat** `M <field>`,
  not a Π-chain. In `ColKind::Ih`, compute `ih_ty` as the **flat**
  `weaken(ret_ty, real_depth_so_far)` when the pending tail is itself sibling
  `Ih` columns **from the same constructor**, reserving `tail_codomain`'s
  full-fold for genuinely-outer pending splits. Verify the exact helper/args
  against the landed code; the *shape* (flat binder, not Π-chain) is the fixed
  requirement.
- **Lane:** the fix lives **only** in `ken-elaborator` (`ColKind::Ih` /
  `tail_codomain` scoping). **The kernel `Elim`/motive checker is NOT touched.**

## Deliverable — the scoped fix
In `compile_match_matrix`'s `ColKind::Ih` branch, distinguish "my own
constructor's next Ih sibling" from "the enclosing match's genuine pending tail,"
and build the flat Ih binder type for the former. Minimal, targeted — do not
refactor `tail_codomain`'s legitimate split-column use.

## Acceptance criteria — the Architect's soundness gate (4, load-bearing first)
- **AC1 — Kernel untouched (LOAD-BEARING).** `git diff origin/main --
  crates/ken-kernel/` **empty**; `trusted_base()` unchanged. The fix is
  `ken-elaborator` only. *If the fix ever touches the kernel `Elim`/motive checker
  to swallow the previously-rejected term — STOP and escalate to Steward →
  Architect; that is the only path from this completeness bug to a soundness
  hole.*
- **AC2 — Correct-for-the-right-reason.** A **real structural-recursion test**: a
  tree `size`/`fold` over a ≥2-recursive-field type (`data Tree = Leaf | Node
  Tree Char Tree` or similar) that **uses both IHs** and **computes the right
  value** — not just a constant-arm match. Guards the dual bug of dropping or
  mis-binding an IH (a wrong-but-flat build the kernel would *accept* → a wrong
  value, caught here, not by the kernel).
- **AC3 — No over-correction (discriminating pair).** A valid ≥2-same-field
  `match` **accepts**, AND an ill-typed sibling (e.g. an arm at the wrong type)
  **still rejects**. The fix must not turn the motive machinery permissive.
- **AC4 — No regression.** The scoping condition ("pending tail is sibling Ih
  from the same ctor") must **not misfire** on 0-1-recursive-field types:
  `List`/`MyList`, single-recursive-field `T7`, `1-rec+1-other` `T8` all still
  elaborate. Run the existing match/elaborator suite green + `cargo test
  --workspace`.

## Guardrails (do-not-reopen)
- Kernel checker is **off-limits** — this is fixed by making the elaborator build
  the correct (flat) motive, never by relaxing the kernel.
- Keep `tail_codomain`'s full-fold for its legitimate whole-method-codomain use;
  only the single-Ih-binder reuse is wrong.
- Don't broaden scope beyond the `ColKind::Ih` binder-type computation.

## Gate & sequencing
- **Gate:** **Architect soundness** (AC1-4; he has pre-grounded the site so the
  gate is fast on return) **+ Language QA + CI.** No spec/CV (`/spec` unchanged,
  completeness fix).
- **Lane:** Team Language (owns `ken-elaborator`).
- **Sequencing — EXPEDITED = the next thing after VAL2's close seam**, ahead of
  the operator-queue feature items — **not** interrupt-now (Architect concurs:
  trust root intact, `main` honest while the bug stands since it only *rejects*,
  so no soundness reason to thrash Team Language mid-VAL2). Released at VAL2's
  close (retros in → Handoff-Gate compaction → kickoff), one-WP-per-team. Branch
  cut off the `origin/main` at release. On kickoff, relay the frame to the
  Architect — he pre-grounded `ColKind::Ih` (~2364-2394) / `tail_codomain`
  (~2296-2322) for a fast gate.
- **On land:** un-hold the VAL2 binary-ADT examples (`tree-traversal` and any
  held tree-shaped tasks) — swap their `KNOWN-GAP.md` for real `expected` oracles
  (route back to Team Language as a VAL2 continuation, not part of this fix WP).
