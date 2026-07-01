# L3b — user-type DecEq/Ord instancing for collections (the §37 §6 gate)

**Steward frame.** WP `L3b`. Surface-completeness WP → **Team Language** (they
built Lc; same `instance_search` family). Conformance-then-build. Owner:
conformance-validator (`/conformance` seed) → Team Language (build) → QA →
merge Decision.

## Why this WP, and why now

L3 shipped the collection **types** + structural equality + **built-in**
DecEq/Ord for the primitive/core types, but explicitly **gated user-type
instancing on L-classes** (§37 §6: *"Full user-type instancing of `DecEq`/`Ord`
(a user `instance DecEq MyType`) depends on L-classes (`33 §5`/`39`) and is
gated there — L3 pins the boundary, it does not deliver user instancing."*).
**Lc landed** (`4aa36c7`) — that gate is now open. L3b delivers the crossing: a
user's `instance DecEq MyType` makes `Map MyType v` / `Set MyType` work, and a
user's `instance Ord MyType` drives the **verified `sort`** and the ordered
Map/Set operations. This is the first step of the surface push toward the
"test language functionality" phase — real programs need user-keyed maps and
sorting of user types.

## Locus

- **Spec:** `spec/30-surface/37-strings-collections.md` is **impl-ready (L3)**;
  §6 already pins the staging boundary, the `DecEq`/`Ord` class shapes, and the
  verified-`sort` refinement. **No spec-hardening in this WP** — L3b removes the
  gate §6 flagged, it does not re-open §37. (If a genuine spec gap surfaces
  mid-build, flag it as a Steward erratum — do not silently patch §37.)
- **Conformance (extend):** `conformance/surface/collections/` (exists from L3).
  CV adds the **user-instancing** discriminating cases (below).
- **Build:** `crates/ken-elaborator` — wire the collection operations that need
  a key/element class (`Map`/`Set` key ops, the verified `sort`, ordered
  Map/Set ops) to resolve `DecEq`/`Ord` for **user types** via Lc's landed
  `instance_search` (`classes.rs:91`), not the built-in-only path.

## Pinned decisions — settled (§37 + Lc/ADR 0008), do NOT reopen

1. **Staging boundary** (§37 §6): L3 = built-in `DecEq`/`Ord` for primitive/core
   types; **L3b = user-type instancing via Lc**. This WP connects collections to
   the landed class resolver; it does not add a kernel rule (§37 banner: "no new
   kernel rule").
2. **`Map`/`Set` are `DecEq`-keyed, canonically ordered abstract types** (§37
   §3, §3.3): content-addressed (kind `0x07`/`0x08`), **key-sorted canonical
   form** → structural O(1) equality. Two maps with the same entries inserted in
   any order are the **same value**.
3. **`DecEq` and `Ord` are STRUCTURE classes** (§37 §6 / `33 §5`, ADR 0008):
   canonical-instance resolver convention (one canonical per (class, head-type),
   orphan/overlap errors) — the Lc policy, already landed. `Ord` carries its
   total-order **law proofs** (reflexivity/antisymmetry/transitivity/totality).
4. **★ The verified `sort` refinement is `{ ys | isSorted ys ∧ Perm ys xs }` —
   BOTH conjuncts** (§37 §6, `34 §5`). `isSorted`-alone is **degenerate**
   (`sort _ = Nil` satisfies it vacuously); the **`Perm ys xs` conjunct is
   load-bearing** — it forces `sort` to actually be a sort. The elaboration
   **emits the conjoined obligation** `isSorted (sort xs) ∧ Perm (sort xs) xs`
   on the result introduction — **emitted, not assumed** (`22 §2.1`, the
   untrusted-layer discipline).

## Acceptance criteria (discriminating pairs; producer-grep gated)

- **AC1 — ★ user `DecEq` instance keys a `Map` (real `instance_search`).** A user
  `data K = …` with a user `instance DecEq K` → `Map K v` construction/lookup
  **resolves the user instance** via Lc's `instance_search` and works; the same
  `data K` **without** a `DecEq K` instance → `Map K v` is a **no-instance
  compile error** (not a silent built-in fallback, not a runtime failure). The
  verdict flips on the presence of the user instance, resolved by the real
  resolver. Producer: `instance_search` (`classes.rs:91`) wired to the `Map` key
  path — **grep that the Map key op calls the real resolver**, not a built-in
  `DecEq`-only table.

- **AC2 — user `Ord` instance drives the verified `sort`.** A user type with a
  user `instance Ord K` → `sort (xs : List K)` type-checks and its
  refinement discharges; the same type **without** `Ord K` → `sort` is a
  no-instance error. Discriminating pair on the real `Ord` resolution + the
  refinement obligation. (Ordered `Map`/`Set` ops likewise resolve user `Ord`.)

- **AC3 — ★ the verified `sort` EMITS the conjoined VC (soundness).** Elaborating
  `sort` emits `isSorted (sort xs) ∧ Perm (sort xs) xs` as the result obligation
  — **both conjuncts**. Discriminating against the degenerate encoding: an
  `isSorted`-only obligation is satisfied by `sort _ = Nil`, so the case must
  observe the **emitted VC structurally** (per `34 §5` / `22 §2.1`) and confirm
  the `Perm` conjunct is present — a `sort` that dropped `Perm` and returned
  `Nil` must **fail** this AC. Producer: grep the **emitted** obligation at the
  `sort` result-introduction site, not an assumed/hand-fed proposition.

- **AC4 — `Map`/`Set` canonical form (structural O(1) equality).** Two `Map K v`
  built by inserting the same (key,value) set in **different orders** are
  **structurally equal** (O(1) slot-id), because the canonical form is
  key-sorted (via the resolved `Ord`/`DecEq`). Discriminating: same entries →
  equal; a differing entry → unequal. Producer: the real canonicalization path,
  not a list-compare.

## Producer-grep gate (HIGH-signal)

The build is net-new wiring; make the producers **real**:
- AC1/AC2 → collection key/sort ops call Lc's `instance_search` (`classes.rs:91`)
  for **user** types. **Not** a built-in `DecEq`/`Ord` table that silently
  handles only primitives (that would pass a primitive-keyed test while a
  user-keyed map fails — the exact built-in-fallback trap).
- AC3 → the **emitted** conjoined VC at the `sort` result site (`34 §5`). Not an
  assumed proposition; the `Perm` conjunct must be in the emission.
- AC4 → the real key-sorted canonicalization.

Grep the producer, not the test. A green case that hand-feeds the instance
dictionary or the VC is green-vs-green.

## Process (`§14` build; conformance-then-build)

1. spec-leader routes **CV** to author the `conformance/surface/collections/`
   user-instancing additions (AC1–AC4 discriminating cases). No `/spec` change →
   the conformance merge gates on **Architect soundness** (the `sort` VC-emission
   AC3 + user-instance resolution) + **spec-author Fidelity** (on the seed).
   Land the conformance.
2. **Team Language** builds the wiring (collection ops → `instance_search` for
   user types) against the merged conformance; **QA** producer-greps the real
   resolver call (not a built-in table) + the emitted `sort` VC; build merge
   Decision on **Architect soundness**. Integrator merges.
3. Retros in → Steward. Unblocks L8 (stdlib over user-keyed collections) and the
   "test language functionality" phase.

## References (verify targets, don't launder)

- `spec/30-surface/37-strings-collections.md` §3 (collection types), §3.3
  (`Map`/`Set` DecEq-keyed), §6 (equality/ordering/verified `sort` + the L-classes
  staging boundary).
- `spec/30-surface/34-data-match.md §5` (refinement types + emitted VC), `22 §2.1`
  (obligation emission).
- `crates/ken-elaborator/src/classes.rs:91` (`instance_search` — the landed Lc
  resolver), `elab.rs:716` (`init_class_env`), `elab.rs:871` (`elab_instance_decl`).
- `spec/30-surface/33-declarations.md §5` + `39 §6` (Lc class/instance semantics,
  landed); ADR 0008 (coherence — DecEq/Ord are structure classes).
