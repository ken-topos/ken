# WP N3 — import-exclusion + local/import clash-error (ADR 0014 MRES-6)

Owner: **spec enclave** (Lane A: spec + conformance) → **Language team**
(Lane B: build). Two lanes, released in order: **Lane A first** (enclave writes
the normative rule + golden), **Lane B** (Language builds to it) gated on Lane A
landing. Design source of truth: **`docs/adr/0014-cross-package-resolution-and-
fail-closed-collision.md`** — MRES-6 (§ "Local/import name clash", the full
hand-to-enclave design at lines ~574–623) and MRES-10 (prelude precedence,
reconciled). Normative surface today: `spec/30-surface/33-declarations.md`
§3.2 (import forms) + §3.3 (name resolution). Size **M**. Base: `origin/main`
(re-verify cites at pickup).

## Objective

Land **MRES-6** — the operator's opt-A ruling that a top-level local/import name
clash is a **fail-closed error**, resolved only by **explicit** import
specification (positive selection is already the default; add a **per-name
rename** `import M (foo as bar)`). This **reverses** §3.3's current "Local over
imported … never an error" **for the module-level case only**, folds in
MRES-10's **prelude-unshadowable** resolution, and leaves ordinary **lexical
binder shadowing untouched**. `hiding` is **retired as moot** (no bring-all
baseline survives ADR 0015's `use M` removal) — do **not** add any exclusion
grammar.

## Fixed inputs — SETTLED (operator opt-A, Architect-confirmed), do NOT reopen

- **Clash is an ERROR, not silent local-wins** (operator OVERRODE the original
  rec). A name bound both by a **top-level local definition** and by an
  **import** is a clash error unless the import explicitly drops or renames it.
- **Order-independent, latent-clash-is-clash (fail-closed, like MRES-5).** The
  error is raised whether or not the name is *used* — a latent clash is still a
  clash.
- **Scope the reversal PRECISELY — module-level only.** This governs **top-level
  definition vs import** clashes. **Ordinary lexical shadowing is UNTOUCHED**: a
  function parameter, `let`, or `λ` binder in a narrower scope still shadows an
  outer/imported name (innermost wins) — that is the term language, not a
  module-name clash. §3.3's "narrower scope, innermost wins" **survives** for
  lexical binders; only its *module-level local-over-import* clause reverses.
- **Surviving resolutions (no `hiding`):**
  - `import M (foo, Bar)` — positive selection (already exists, §3.2).
    **Exclusion is the default**: a name you do not list is not brought. This
    subsumes what `hiding` did for imports.
  - **`import M (foo as bar)`** — per-name rename (NEW form; `import M as N`
    today aliases the *module*, not a name). Resolves a clash by renaming the
    imported name at the import site.
- **Prelude is UNSHADOWABLE (MRES-10, opt-A).** The prelude is the one
  always-present unqualified baseline (`§30-taxonomy §4`); it is never
  `import`ed, so a prelude name can be neither un-selected nor renamed at an
  import site. **A local definition clashing with a prelude name is a clash
  error resolved by renaming the LOCAL definition; there is no prelude-exclusion
  form.** This makes `hiding` *fully* moot.
- **`hiding` is retired — add NO exclusion grammar.** Not `import M hiding (…)`,
  not any prelude opt-out.

## Lane A — spec + conformance (spec enclave)

### Spec — `spec/30-surface/33-declarations.md` (+ `32-grammar.md`)

1. **§3.2 — add the per-name rename form.** Extend the selective-import bullet:
   `import M (foo, Bar as Baz)` brings `foo` unqualified and `Bar` unqualified
   **under the name `Baz`**. Keep the three existing forms verbatim; the rename
   is a per-*name* modifier inside the selection list, distinct from the
   module-level `import M as N`.
2. **§3.3 — split and reverse the "Local over imported" bullet.** Replace the
   single current bullet ("A name bound in the current module (or a narrower
   scope) **shadows** an imported one … never an error") with **two** rules:
   - **Module-level (REVERSED):** a name bound both by a **top-level local
     definition** and by an **import** is a **clash error** (`AmbiguousReference`
     or a dedicated clash diagnostic — enclave picks the surface error class);
     resolve it by **not selecting** the name or by **renaming** it at the import
     site. Silent local-win is **not permitted**. Order-independent; raised even
     if the name is never referenced.
   - **Lexical (UNCHANGED):** a name bound in a **narrower lexical scope**
     (`λ`/`let`/parameter/pattern binder) still shadows an outer or imported
     name, resolved lexically (innermost wins) — **never an error**. State
     explicitly that this is the term language, orthogonal to the module-level
     clash.
3. **§3.3 (or §4) — prelude-clash clause.** A **top-level local definition**
   whose name collides with a **prelude** name is a **clash error**, resolved by
   **renaming the local definition**; prelude names are the primitive floor and
   are **not shadowable**; there is **no** prelude-exclusion form. (Cross-ref
   MRES-10; note the ergonomic coupling to a deliberately-small prelude.)
4. **`32-grammar.md` — EBNF for the rename form.** Extend the selective-import
   production so each list item is `name` or `name "as" rename`. Do **not** add a
   `hiding` production. §32 sweep for consistency with §3.2.

### Conformance — `conformance/surface/modules/…`

5. **Flip the re-vehicled shadowing case + add the clash/resolution suite.** ADR
   0015 re-vehicled `local-shadows-imported-lexically` through `import M (foo)`
   **preserving the then-current local-wins semantics**, explicitly so that
   "when N3 flips that rule, the case and the rule flip together" (Architect,
   `evt_6zgnnzd0jkq9t`). N3 is that flip. Deliver:
   - **Module-level clash → error.** A top-level local `foo` + `import M (foo)`
     → clash error (the case that was local-wins is now REJECT). Assert the
     specific surface error class, not merely "rejected". Order-independent:
     include a latent (never-referenced) clash arm — still rejected.
   - **Resolution by de-selection.** Same setup, `import M ()` / omit `foo` →
     accepts; the local `foo` is the sole binding.
   - **Resolution by rename.** `import M (foo as bar)` → accepts; `foo` (local)
     and `bar` (imported) are two distinct bindings; a reference to each resolves
     to the right target (distinct `GlobalId`s).
   - **Lexical shadowing still local-wins.** A `λ`/`let`/parameter binder
     shadowing an imported name → **accepts**, innermost wins (the discriminator
     that the reversal did NOT over-reach into the term language).
   - **Prelude clash → error; rename-the-local resolves.** A top-level local
     redefining a registered prelude name → clash error; renaming the local →
     accepts. (No prelude-exclusion form is testable — assert its absence only
     via the grammar, not a positive case.)
   - **Rename grammar parse.** `import M (foo as bar)` parses; `import M hiding
     (…)` is a **syntax** rejection (no such production).

## Lane B — build (Language team; gated on Lane A landing)

Scope: `crates/ken-elaborator` (parser + resolver). Design cites the build site
directly (ADR MRES-6): today `bind_import` **silently refuses to touch a
`locals` name** (`crates/ken-elaborator/src/…/modules.rs:75-77` — re-verify the
path/line at pickup) — that silent refuse is exactly the local-wins behavior N3
reverses.

1. **Parser — the rename form.** Accept `import M (foo as bar)` (per-name rename
   inside the selection list). Reject `import M hiding (…)` as a syntax error
   (no production). Reuse the existing `as` token; distinguish per-name rename
   (inside `( … )`) from module alias (`import M as N`).
2. **Resolver — clash detection (the reversal).** At `bind_import`, a name bound
   both by a top-level local and by an import is a **clash error**
   (`AmbiguousReference` or the enclave-specced class), **unless** the import
   dropped or renamed it. Order-independent and **latent-clash-is-clash**: raise
   even if the name is never referenced (fail-closed, MRES-5 family). Replace the
   silent `locals`-refuse with the error.
3. **Rename binding.** `foo as bar` binds the imported `M.foo` under `bar`
   unqualified; `foo` is then free for a local (or another import).
4. **Prelude clash.** A top-level local colliding with a prelude name is a clash
   error (resolve by renaming the local). Prelude names are not shadowable at the
   top level; **lexical binders still shadow** (do not touch the term-level
   scope resolution).
5. **Do NOT touch lexical shadowing.** The `λ`/`let`/parameter/pattern
   innermost-wins path is out of scope and must remain behaviorally identical.

## Acceptance criteria

- **AC1 (spec).** §3.2 carries the `import M (foo as bar)` rename form; §3.3's
  "Local over imported" is split into a **module-level clash-error** rule and an
  **unchanged lexical-shadowing** rule; the prelude-clash clause is normative
  (unshadowable, rename-the-local, no exclusion form); `32-grammar.md` has the
  rename EBNF and **no** `hiding` production.
- **AC2 (golden).** The conformance suite asserts: module-level clash → error
  (incl. a latent-clash arm); de-selection resolves; rename resolves (distinct
  `GlobalId`s); lexical shadowing still local-wins; prelude clash → error +
  rename-the-local resolves; `import M (foo as bar)` parses and `hiding` is a
  syntax reject. Each case asserts the **specific** surface error class /
  resolved target, not bare accept/reject.
- **AC3 (build, Lane B).** Parser accepts the rename form and rejects `hiding`;
  resolver raises the clash error order-independently and for latent clashes;
  rename binds correctly; prelude clash errors; lexical shadowing is behaviorally
  unchanged. `scripts/ken-cargo test -p ken-elaborator` green **and** the literal
  `cargo build --workspace --locked && cargo test --workspace --locked` green.
- **AC4 (boundary).** Lane A: spec + conformance only, **zero**
  crates/kernel/prelude/Cargo/lock/`trusted_base()` delta. Lane B:
  `crates/ken-elaborator` (+ tests) only, **zero** kernel/prelude/Cargo/lock/
  `trusted_base()` delta (surface resolution is §3.3 "never reaches the kernel").
  `git diff --check` clean each lane.

## Review

Lane A: enclave gates (spec-leader scope/fidelity + CV conformance) then
**Architect-terminal** (he owns ADR 0014 and the MRES-6 reconciliation, and
flagged the N3-owned §3.3 boundary in the ADR-0015 review). Lane B:
**Architect-terminal** (the resolver clash semantics + the untouched-lexical
boundary are his design). Team QA runs AC3 over the literal locked CI as a
first-class gate (the N2 carry). Hand each lane's SHA to Steward; Steward
publishes (Lane A doc-only; Lane B is code — poll CI in background, stop on red).

## Do-not-reopen guardrails

- **Opt-A is settled** — clash is an error; no silent local-wins revival.
- **No `hiding`, no exclusion grammar** — exclusion is default (omit the name);
  no prelude opt-out form.
- **Prelude is unshadowable** — resolve a prelude clash by renaming the local,
  never by excluding the prelude name.
- **Lexical binder shadowing is UNTOUCHED** — the reversal is module-level only;
  do not narrow or error the `λ`/`let`/parameter innermost-wins path.
- **Rename is per-name inside `( … )`** — distinct from the module-level
  `import M as N` alias; do not conflate them.
- **Surface-only** — resolution never reaches the kernel; zero TCB delta.
