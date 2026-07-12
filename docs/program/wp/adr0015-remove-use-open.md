# WP ADR-0015-spec — remove the open-import form `use M` (spec + conformance)

Owner: **spec enclave** (single lane — spec + conformance; **no build this
round**). Design source of truth:
`docs/adr/0015-remove-open-import-use.md` (Accepted, operator-directed).
Size **S**. Deps: none hard. **Orthogonal to N2**; sequenced **before N3** to
clear §3.3 first (see boundary note). Base: `origin/main @ 3a5cd323`.

## Fixed inputs — SETTLED, do not reopen

Accepted in ADR 0015; the WP implements the removal, it does not relitigate:

- **Remove `use M`** (wildcard open-import) from the normative surface. The
  import forms drop from **four to three**, keeping the three
  **provenance-preserving** forms: `import M` (qualified), `import M as N`
  (aliased), `import M (…)` (selective).
- **Rationale (recorded, not for debate).** Open import is subsumed by selective
  import; its only delta was provenance loss ("where is this symbol defined?").
  Operator ergonomics are served by instance resolution + the injected prelude.
  Removal is the **reversible** direction (adding a form later is a pure
  widening; removing later would be breaking) — so remove now.
- **Zero `use M` in-tree** — the Architect swept `catalog/` / `examples/` /
  prelude and found none. **The enclave must re-run the authoritative sweep as a
  removal precondition** (below) — the removal is only clean if the sweep is
  still empty at build time.
- **Surface-only.** Zero kernel / prelude-semantics / `trusted_base()` / Cargo /
  lock delta. Grammar/lexer/`use`-keyword retirement is a **build fast-follow**
  (a separate later WP), **not** this doc round.

## Deliverable (mandated outline; each item ends in a concrete edit)

### Spec — `spec/30-surface/33-declarations.md`

1. **§3.2 Importing — drop the `use M` bullet.** Change "**Four forms:**" to
   "**Three forms:**" and delete the `use M` bullet (currently: "*`use M` —
   open: all of `M`'s exports, unqualified. Use sparingly (it maximizes the
   ambiguity surface, §3.3).*"). The three remaining bullets stand unchanged.
2. **§3.3 Name resolution — remove the "Open ambiguity" rule.** Delete the whole
   "**Open ambiguity.**" bullet (the `use`-opened same-name ambiguity rule).
   With no open form, cross-module unqualified same-name ambiguity **cannot
   arise** — every unqualified reference now comes from a *selective* import,
   which is unambiguous by construction (or is a duplicate-selective clash, a
   distinct matter not introduced here).
3. **§3.3 failure list — drop "ambiguous open".** In the closing "Every failure
   — unresolved name, ambiguous open, out-of-scope private name — …" sentence,
   remove "ambiguous open" so the failure list matches the reduced surface.
4. **Sweep §33 (and any §3–§4 cross-reference) for residual `use`-open
   mentions** — the "ambiguity surface" phrasing, any example using `use`, any
   prose that presumes the open form — and reconcile them to the three-form
   surface. A generalization ("the import forms") that still enumerates four is
   a fidelity bug.

### Conformance — `conformance/surface/…`

5. **Retire the `use-open-ambiguity` case.** Remove the conformance case that
   exercised open-import ambiguity (it tests a form that no longer exists).
   Confirm no *other* conformance case silently depends on `use M`.
6. **Authoritative removal-precondition sweep.** Re-run the in-tree sweep for
   any `use M` open-import occurrence across `catalog/`, `examples/`, prelude
   sources, spec examples, and conformance fixtures. Record the sweep result in
   the handoff. **If the sweep is non-empty, STOP and route back to Steward** —
   a live `use M` means migration is required before removal (do not silently
   leave a now-illegal form in-tree). Expected: empty.

## Boundary note — do NOT touch what N3 owns

**N3 (import-exclusion, MRES-6) reverses the §3.3 "Local over imported" rule
(local shadows imported → becomes a clash error).** ADR 0015 removes **only**
the "**Open ambiguity**" rule. **Leave the "Local over imported." bullet exactly
as-is** — it is N3's to reverse, and touching it here would collide N3's edit.
Two different §3.3 rules; this WP retires one, N3 reverses the other.

## AC

- §3.2 lists **three** import forms; the `use M` bullet is gone.
- §3.3's "Open ambiguity" rule is removed; the failure list no longer lists
  "ambiguous open"; the "Local over imported" rule is **unchanged**.
- No residual four-form / `use`-open mention anywhere in §33 (or its
  cross-references).
- The `use-open-ambiguity` conformance case is retired; no other case depends on
  `use M`.
- The removal-precondition sweep is recorded and **empty** (else routed back).
- Doc/spec/conformance-only; **zero** crates / kernel / prelude / Cargo / lock /
  `trusted_base()` delta.

## Review

Enclave gates (spec-leader scope/fidelity + CV conformance) then
**Architect-terminal** surface/fidelity review (he authored the ruling and
requested the routing). Hand the SHA to Steward; Steward publishes doc-only.

## Do-not-reopen guardrails

- Do **not** retire the `use` **keyword** from the grammar/lexer here — that is
  the **build fast-follow** WP. (Confirm-only: note in the handoff whether `use`
  is currently a reserved keyword, for that later WP's frame.)
- Do **not** touch the §3.3 "Local over imported" rule (N3).
- Do **not** add any new import/disambiguation form — this WP only **removes**.
- Do **not** change instance resolution or the injected prelude (the ergonomics
  argument cites them but does not modify them).
