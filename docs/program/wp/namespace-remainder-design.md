# WP · Namespace remainder — Architect design round (#39 + #36/N5)

**Owner:** Architect (design) · **Consumers:** Spec enclave (spec/conformance
WPs) + Language (resolver/build) as follow-ons · **Size:** M · **Base:**
`origin/main @ 431e36ea` (your worktree is now current here). **Status: READY —
operator-directed 2026-07-13** (Pat: unblock the namespace remainder; the fleet
is running all lanes to use the weekly window, so this is live now, not queued).

**This is DESIGN work.** You produce the ADR-0014 MRES-6 amendment (#39) and the
`#36/N5` re-export design doc + MRES-9 status-flip. The spec, conformance, and
build WPs follow from your artifacts — I frame those once your design lands.
**Read the authoritative sources from `origin/main`** (your worktree was stale
and was reset to current main). **#36/N5's spelling is OPERATOR-FACING — propose
options, do NOT pick; I surface them to Pat.** Hand the artifacts back to me.

---

## Objective

Unblock the two remaining namespace-resolution design pieces so downstream
spec/conformance/build WPs can proceed:

- **#39** — Generalize MRES-6's fail-closed clash rule to the
  currently-unspecified import×import and import×prelude cases.
- **#36 / N5** — Design the re-export surface form deferred by MRES-9,
  including an operator-facing spelling decision.

## Fixed inputs (verified against current `origin/main`)

- **ADR 0015** (`docs/adr/0015-remove-open-import-use.md`, *Accepted*) removed
  `use M` open-import; the three provenance forms `import M`, `import M as N`,
  `import M (…)` remain. Spec + conformance changes **already merged** (commit
  `5674cb88` + N2/N3/N4, §32-currency `106fd60c`). **The spec is CURRENT — this
  is not a re-derivation.** The `use` keyword is **retired**: the re-export form
  cannot spell as `pub use`.
- **ADR 0014** (`docs/adr/0014-cross-package-resolution-and-fail-closed-collision.md`),
  register MRES-1..10:
  - **MRES-6** (local/import clash → `AmbiguousReference`) landed as N3. But
    **§3.3 of `spec/30-surface/33-declarations.md` currently specs ONLY
    local×import and local×prelude** — it is **silent on import×import and
    import×prelude**. #39 closes exactly that gap; contained follow-on to N3.
  - **MRES-9** deferred the re-export form (does not exist yet). The
    canonical-identity invariant and **MRES-4d** (instance-surface carry) are
    recorded but **unbuilt**.

## Mandated deliverable outline

### #39 — MRES-6 general-clash amendment (ADR 0014)

Produce **amendment text for ADR 0014's MRES-6 entry** stating the general rule:

> When **more than one** of {top-level local def, selective/renamed import,
> prelude} binds the same unqualified name, resolution yields
> `AmbiguousReference` — **order-independent** and **fail-closed**.

- Explicitly cover **import×import** and **import×prelude** (the §3.3 gap),
  unifying them with the landed local×import / local×prelude cases rather than
  as special cases.
- State that qualified access (`M.name`) and renamed import (`as N`) remain the
  disambiguation escape hatches.
- Name the follow-ons (enclave §3.3 spec edit + conformance WP + Language
  resolver build). Do **not** author those here.

### #36 / N5 — Re-export design doc + MRES-9 status-flip

Produce a **design doc** (new `docs/` design note or ADR-0014 companion):

1. **Spelling — OPERATOR DECISION.** `use` is retired, so `pub use` is off the
   table. **Surface 2–3 concrete spelling OPTIONS** with trade-offs (e.g.
   `pub import M (…)`, an `export …` form, a re-export attribute/modifier).
   **Do NOT pick** — flag as an operator decision; lay out the options for Pat.
2. **Semantics:**
   - **Canonical-identity collision checking** — distinguish *defined-at*
     identity from *re-exported-as* surface name; a re-export must not
     manufacture a second canonical identity, and a re-exported name entering a
     scope participates in the #39 general-clash rule.
   - **MRES-4d instance-surface carry** — how re-exported instances enter an
     admitting consumer's **direct-use set**.
   - **Grammar (§32)** and **§4 visibility** additions the form requires
     (sketch, not final spec text).
3. **ADR-0014 MRES-9 status-flip** — flip MRES-9 from *deferred* to the decided
   form, cross-referencing the design doc; record MRES-4d and the
   canonical-identity invariant as now-designed.

## Acceptance criteria

- **#39:** ADR-0014 MRES-6 amendment text lands the order-independent,
  fail-closed general rule covering all four clash pairings, explicitly naming
  import×import and import×prelude as the newly-closed cases, and cites the §3.3
  gap it resolves. Follow-on WPs named but not authored.
- **#36/N5:** Design doc presents ≥2 spelling options **flagged as an operator
  decision** (no unilateral pick), specifies canonical-identity collision
  behavior, MRES-4d instance carry, and §32/§4 additions; ADR-0014 MRES-9
  flipped to decided with a doc cross-reference.
- Both artifacts are **design-only** — no spec/conformance/build changes; each
  names the WPs it feeds.

## Note

The spec is **current**, not stale — neither deliverable re-derives it. **#39 is
a contained follow-on to landed N3.** **#36/N5 introduces a new surface form; its
spelling is operator-facing and must be surfaced as options.**
