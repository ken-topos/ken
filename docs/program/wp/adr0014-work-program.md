# ADR 0014 work program — cross-package resolution + fail-closed collision

Owner: Steward. Design source of truth:
`docs/adr/0014-cross-package-resolution-and-fail-closed-collision.md`
(Status: Accepted). This document decomposes the Accepted ADR into sequenced,
shovel-ready work packages and records their state. It is the build-side
companion to the ADR's decision register.

## Shape of the program

Every ADR mechanism is a **surface/elaboration** concern — the module system
elaborates away to the flat `Σ` with **zero `trusted_base()` delta**. No kernel,
no prelude, no semantics core is touched by this program. Each round is run on
the proven `#29` template:

1. **Spec lane (enclave).** The spec enclave (spec-leader / spec-author /
   conformance-validator) turns the round's MRES decisions into normative spec
   text (§30-surface chapters) and a conformance golden. Merges first.
2. **Build lane (build team).** The owning build team implements against the
   landed spec (predominantly `crates/ken-elaborator` — resolver / modules /
   classes / elab). Merges second.
3. **Conformance.** CV's golden is the acceptance oracle; it is grounded on the
   landed build and merges as the round's tail (doc/conformance-only).

The enclave is a single unit and serializes spec work; the loader is the spine
that N3/N4 depend on. So the rounds run **mostly sequentially**, with N1
independent of the loader and therefore first.

## Work packages

### N1 — Fail-closed duplicate-definition slice (round 1) · size S · **first**

- **MRES:** 5 + 7 + 8. **Deps:** none (acts on one unit's own globals).
- **Objective.** A second top-level definition of the same single-namespace name
  in one compilation unit is a hard error; this includes the class-vs-constructor
  collision (`class Eq` vs ctor `Eq`) under the single-flat-namespace
  (types-are-terms) rule. The **arity-gated sugar exclusion** that lets `Eq`/`J`
  legitimately coexist is preserved.
- **Spec lane.** Normative rule in `spec/30-surface/33-declarations.md`: duplicate
  top-level definition = error; class/ctor same-name collision = error; the
  arity-gated-sugar coexistence remains legal. Conformance golden: duplicate
  rejected; class/ctor collision rejected; arity-gated `Eq`/`J` coexistence still
  accepted (the discriminating positive arm — reject/accept must flip, never
  green-vs-green).
- **Build lane.** Generalize the existing `resolve_decl` choke-point guard
  (`check_no_reserved_sugar_collision`, today a 3-name reserved list) to a
  duplicate-definition check over the live globals, preserving the arity-gated
  exclusion. Elaborator-only; no loader, no import system.
- **AC.** duplicate/collision rejected with a specific error variant; arity-gated
  coexistence green; full workspace green; zero kernel/prelude/semantics/Cargo/
  lock delta.

### N2 — In-repo cross-file loader (round 2, spine) · size M

- **MRES:** 1 + 2 + 3(a strict). **Deps:** none hard, but sequenced after N1.
- **Objective.** A cross-file `import` resolves to another in-repo compilation
  unit (today it is `UnboundName`; packages inline helpers). Total, role-blind
  path↔file bijection (strict, per MRES-3a); lazy discovery from import edges;
  **cycle = hard error**; cache on `ElabEnv`. **Multi-catalog forward-compat:**
  the catalog-root anchor is a **plural-ready root list** (round populates one
  root, the resolution API takes the plural form) — multi-root resolution /
  precedence deferred, not precluded.
- **Spec lane.** Loader semantics in `spec/30-surface/33` §3.2 (path anchoring,
  discovery, cycle-error, caching posture); the strict bijection recorded
  normatively. Conformance golden for cross-file resolution + cycle rejection.
- **Build lane.** Loader on `ElabEnv`, discovery from import edges, cycle
  detection, cache. Package manager / content-addressing explicitly **out**
  (couples to supply-chain `63`, a later round).
- **AC.** cross-file import resolves; cycle rejected with a specific error; single
  root works through the plural API; full workspace green; zero trust delta.

### N3 — Import-exclusion + local/import clash error (round 2 fast-follow) · size M

- **MRES:** 6 (operator override). **Deps:** N2 (needs cross-file imports live).
  **⚠ Scope reduced by ADR 0015 (2026-07-12) — see note below; `hiding` dropped.**
- **Objective.** A top-level local definition and an imported name with the same
  spelling is a **clash error** (reversing §3.3's silent local-win), resolvable
  by **positive de-selection** (drop the name from the selective import list) or
  **rename** (`import M (foo as bar)`). Order-independent, raised whether or not
  the name is used (fail-closed, like N1). **Ordinary lexical binder shadowing
  (params / `let` / `λ`, innermost-wins) is untouched** — only the module-level
  definition-vs-import clause reverses.
- **Spec lane.** `spec/30-surface/33` §3.2 per-name rename grammar
  (`import M (foo as bar)`; positive selection / de-selection already exist);
  §3.3 clause reversal (definition-vs-import clash = error). Conformance golden:
  unresolved clash rejected; rename (or de-selection) resolves it (reject→accept
  flip); lexical binder shadowing still accepted.
- **Build lane.** Clash detection in the import-binding path (today `bind_import`
  silently refuses to touch a `locals` name); new **rename** grammar (no
  `hiding`).
- **AC.** latent clash rejected; rename/de-selection resolves; lexical shadowing
  unaffected; full workspace green.
- **★ ADR 0015 reconciliation (2026-07-12).** ADR 0015 removes the open-import
  `use M`. After that, **every unqualified imported name comes from a selective
  `import M (…)`** — so a local/import clash is resolved by simply **not
  selecting** the name (or renaming it). The `import M hiding (…)` grammar
  originally planned here is *exclusion from an open import*; with no open form
  left, it has **no "bring-all" baseline to subtract from** and is **moot** —
  **dropped from N3.** This shrinks N3 to the §3.3 clash-reversal + the rename
  grammar. (Sequencing: the ADR-0015 spec removal lands **before** N3, so N3
  reverses §3.3 on the already-simplified three-form surface.)

### N4 — Program abstraction / `admits` (round 3) · size M/L

- **MRES:** 4 (+ 4a/b/c). **Deps:** N2 (loader / multi-package boundary).
- **Objective.** A distinguished `program` file declares an **`admits`** list of
  packages whose instances enter ambient resolution; using an instance from an
  un-admitted package is a hard **`UnadmittedInstance`** error. Ambient
  resolution **inside** an explicitly declared boundary. **4a:** the runtime entry
  point is a separate (co-locatable) declaration. **4b:** required only for
  multi-package builds (a single package self-admits). **4c:** the program names
  its **direct** instance deps; **transitive instance-flow is automatic**
  (source==compiled equivalence forces it); coherence is checked over the full
  closure (O(total instances), orphan-bounded — no quadratic cost). **Provenance
  diagnostics are a required deliverable.**
- **Spec lane.** `program`/`admits` form + admission semantics; the transitive
  closure rule; provenance diagnostic requirements. Conformance golden:
  admitted-instance resolves; un-admitted rejected; transitive flow accepted.
- **Build lane.** New admission gate in the elaborator composing with the
  existing orphan + overlap checks (`instance_search` O(1); admission = O(1)
  set-membership per resolved instance); provenance diagnostics on resolution /
  `UnadmittedInstance` / coherence collision.
- **AC.** admitted resolves; un-admitted rejected with the specific variant;
  transitive closure admitted; provenance diagnostics present; full workspace
  green; zero trust delta.

### N5 — Re-export / facade `pub use` (round 4, latest) · size S

- **MRES:** 9. **Deps:** N2. **May defer the surface form**; the
  **canonical-identity invariant is recorded now** so the later form is cheap and
  API-drift is prevented. Sequenced last; scope confirmed when public package
  topologies actually need it.
- **★ ADR 0015 note (2026-07-12).** The placeholder spelling `pub use` reuses the
  `use` keyword that ADR 0015 **retires** (grammar/keyword retirement is 0015's
  build fast-follow). Since N5's surface form is **already deferred**, this is a
  **naming constraint, not rework**: when the re-export form is chosen, spell it
  **without `use`** (e.g. `pub import …` / a dedicated re-export form). The
  canonical-identity invariant is spelling-independent and stands regardless.

## Deferred — forward-compat only (NOT built in this program)

These are recorded requirements the earlier rounds must **not preclude**, built
in the **future package-manager round**, out of ADR 0014's buildable scope:

- **Multi-catalog roots** (standard + org + vendor) — N2's plural-ready root list
  leaves the space.
- **Compiled-package instance-manifest** — a compiled package must carry its
  committed canonical instances so a compiled import feeds N4's coherence check
  identically to source. All-source today; the manifest rides the package
  manager.

## Sequencing (Steward discretion)

`N1 → N2 → N3 → N4 → N5`. N1 is independent and shovel-ready (first). N2 is the
spine N3/N4 depend on. N3 (import-exclusion) and N4 (program abstraction) both
land after the loader; N3 is the loader round's fast-follow, N4 the next round.
N5 is last. Each WP runs spec-lane (enclave, merges first) → build-lane (Language
team, merges second) → conformance tail (CV). Builds may overlap across teams
once their spec has merged, but the enclave serializes the spec lanes and the
loader gates N3/N4, so the practical cadence is round-by-round.
