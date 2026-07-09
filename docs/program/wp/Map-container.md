# Map-container — a proved, pure associative `Map k v` (VAL2 #8 / OQ-A)

**Steward frame → spec enclave (elaborate) → Foundation (build, packages lane).**
Design-settled capability WP closing VAL2 finding #8 (`letter-frequency`: `Map`
is a bare type with zero operations). Product decision **locked by the operator**
(OQ-A, `VAL2-gap-design-OQs.md`, 2026-07-03). Owner: spec-leader elaborates the
API + proof shape; **Foundation** builds it as a `catalog/packages/` stdlib module. Gate:
**/spec + /conformance touching** → spec enclave elaboration, **Architect**
(structure choice + proof soundness + zero-TCB), **Spec review**
(conformance-validator), team QA, CI. Findings → **Steward**.

## The locked decision — DO NOT REOPEN

- **A proved, pure `Map k v` keyed on `Ord k`**, shipped as a **`catalog/packages/`
  stdlib module (zero TCB, NOT a kernel builtin)** — reuses the landed lawful
  `Ord`; subsume-don't-proliferate.
- **Proved, not tested.** The operator prefers proved to tested and set **no
  tested fallback**: the structural invariant + operation-correctness are carried
  as **real proof terms, not `Axiom` stubs** (the lawful-classes discipline —
  [[lawful-class-instances-must-carry-law-proofs]]; the discriminating case must
  **FAIL** against a law-less/stub instance). If the full proof scope proves too
  large for one WP, **propose a decomposition back to Steward** (e.g. a proof
  layer as a tracked follow-on) — do **not** silently ship a tested-only Map.
- **Structure is the Architect's call (means-to-the-end).** The operator: a
  balanced sorted tree over `Ord k` is the natural fit, and **HAMT is a
  SUGGESTION for a *later* fast-map if profiling demands — not more than a
  suggestion, and not this WP.** This frame pins the **GOAL** (proved + pure +
  `Ord`-keyed + zero-TCB + O(log n)); the **representation** (balanced BST/AVL/
  red-black vs. anything else) is Architect's to choose. No `Map`-specific
  side-consult — Architect is the normal in-process reviewer.

**Why HAMT is not the near-term means (context, do not relitigate):** true O(1)
needs *mutation* Ken lacks, so the real choice is between two persistent
structures — HAMT (O(log₃₂ n)) vs. balanced tree (O(log₂ n)); **both logarithmic,
HAMT only a ~constant-factor win**, while the tree adds provable O(log n)
worst-case + ordered iteration/range/min-max + deterministic order and a simpler
invariant to prove. Hence tree-first, HAMT-as-later-fast-map — both proved.

## Mandated deliverable outline (each item resolves to a concrete choice)

1. **Type + representation.** `Map k v` over `Ord k` in `catalog/packages/` (enclave/
   Foundation fix the module path). Representation = **Architect's choice**
   (balanced tree recommended). Zero kernel/`trusted_base` delta.
2. **Core API.** At least `empty`, `insert`, `lookup`, `delete`, `member`, plus
   ordered traversal (`toList`/`fromList`/fold in key order). Enclave fixes exact
   signatures + the `Ord k` constraint plumbing.
3. **Invariant + proof.** The structural invariant (ordering, and balance if the
   chosen structure has one) carried as a **real proof term**, and
   lookup/insert/delete correctness (e.g. `lookup k (insert k v m) = Some v`)
   proved — **not** `Axiom`. Architect scopes exactly which laws are carried in
   this WP vs. a proposed follow-on.
4. **Conformance — drive the REAL producer.** Tests running the operations
   **through the real interpreter** (insert/lookup/delete round-trips, ordered
   iteration, `letter-frequency` shape) — not a hand-fed harness. Grounded
   against the landed producer ([[conformance-hand-feeds-the-deliverable]]).
5. **Perf note.** O(log n) operations; **HAMT is explicitly out of scope** here —
   a named, parked later fast-map, only if profiling demands (also proved).

## Acceptance criteria

- **AC1 — Zero kernel / `trusted_base` delta.** `git diff origin/main --
  crates/ken-kernel/` empty; `trusted_base()` unchanged; `Map` is a `catalog/packages/`
  stdlib module, no kernel builtin, no new kernel variant. Verify by grep, not a
  test.
- **AC2 — Operations correct end-to-end** through the real interpreter (round-
  trips + ordered iteration + the `letter-frequency` shape).
- **AC3 — Proved, not stubbed.** The invariant + operation-correctness are real
  proof terms; a **discriminating test FAILS against a stub/`Axiom` instance**
  (proved-not-tested is the whole point). If any law is deferred, it is a
  **named tracked follow-on**, not a silent gap.
- **AC4 — No regression.** `cargo test --workspace` green; lawful `Ord` and the
  rest of `catalog/packages/` behave identically pre/post.

## Guardrails (do-not-reopen)

- **Proved, not tested** — real proof terms, no `Axiom` stubs; large proof scope
  → propose decomposition to Steward, never drop to tested-only.
- **Structure = Architect's call** — HAMT is a *later* suggestion, not this WP;
  tree-first recommended; either way proved.
- **Zero-TCB** — `catalog/packages/` stdlib, no kernel builtin; reuse landed lawful `Ord`.

## Sequencing

- **Gate:** /spec + /conformance touching → spec enclave elaborates the API +
  proof shape on this WP branch, merges to `main` via the Integrator, then
  Foundation is kicked. Architect (structure + proof soundness + zero-TCB) + Spec
  review (conformance-validator) + team QA + CI.
- **Lane:** Foundation (catalog/packages/collections). Branch off `origin/main`.
- **Relation to siblings:** independent of `[State]`/`[FS]` (no effect surface);
  couples to the lawful-classes proof discipline (reuse landed `Ord`, carry real
  law proofs).
