# WP N4 — the `program`/`package` abstraction + `admits` admission gate (ADR 0014 R3, MRES-4)

Owner: **spec enclave** (Lane A: spec + conformance) → **Language team**
(Lane B: build). Two lanes, **Lane A first**, Lane B gated on Lane A landing.
Design source of truth: **`docs/adr/0014-cross-package-resolution-and-fail-
closed-collision.md`** — MRES-4 (the `program` abstraction refinement, ~L221–
262), sub-forks **MRES-4a–4f** (~L286–460; **MRES-4f** — cross-package
collision impossible in source, the constructive-coherence theorem), the
**`package` extension** (~L406–496), and **PKG-1..4** (~L498+). Prereqs landed:
N2 loader (`import` graph live)
+ N3 Lane A (unqualified-name resolution normative). Normative surface today:
`spec/30-surface/33-declarations.md` §3–§5 + `30-taxonomy`. Size **L**. Base:
`origin/main` (re-verify cites at pickup).

## Objective

Define the **`program` boundary** over which ADR 0008's "program-wide-stable"
instance coherence is meaningful, and enforce it with a new elaborator
**admission gate**: instances resolve **ambiently inside** an **explicitly
declared** boundary. A `program` (or a self-admitting `package`) lists the
packages whose instances it **admits**; dispatching an instance from an
unadmitted package is a hard `UnadmittedInstance` error. This is the honest
resolution of the ambient-vs-explicit tension — explicit at the *boundary* (the
load-bearing instance channel), canonical *inside* (ADR 0008 soundness
preserved).

## The buildable-now vs deferred boundary — READ FIRST (settled in the ADR)

**Everything is source today** (the N2 loader is source-based). N4 builds the
**source-world** core; the compiled-package machinery is **forward-compat spec
only** (built with the package-manager round). Do **not** build the deferred
items — spec the forward-compat rules normatively where the ADR requires, but
ship **no** manifest/registry/lockfile code.

- **BUILD NOW (Lane B):** `program`/`package` header grammar; the admission gate
  in instance resolution; single-package/`package` **self-admission**;
  provenance diagnostics; coherence over the transitive **source** closure.
- **SPEC-NOW, BUILD-LATER (forward-compat, no code):** the **compiled-package
  instance manifest** (MRES-4c point 4 — package-manager round); **source==
  compiled** equivalence as it pertains to compiled artifacts; **MRES-4d**
  re-export-carries-instance-surface (**cannot arise until MRES-9/N5 `pub use`
  lands** — record the rule, no build); **PKG-1 explicit package member-module
  list** — record the *semantics* normatively (a package file explicitly
  enumerates its members; identity/root stay path-inferred, MRES-4e/2b) but
  **defer the concrete member-list grammar/spelling** to the package-manager
  round (do NOT mint a member-list keyword; the buildable-now admission gate runs
  over the existing **source graph / path-implicit membership** — N2's MRES-3a
  bijection — which PKG-1's explicit list later refines); content-addressed
  manifest/lockfile/registry; **test-scoped admission (PKG-4 deferred)**.
  (Steward ruling `evt_vyqbdvaejp00` — defer spelling, not concept.)

## Fixed inputs — SETTLED (operator 2026-07-12), do NOT reopen

- **Admission model (MRES-4).** Ambient *resolution* inside an *explicitly
  declared* boundary. `admits` lists package **paths** (MRES-2 dotted
  addressing). The admission check composes with (does **not** replace) the
  existing orphan + overlap coherence checks.
- **Anonymous headers (MRES-4e).** `program` and `package` headers take **no
  name token** — identity is the file path (MRES-2b); the header's **presence**
  is the signal (a `package` header's presence makes the file a compiled
  package). Documentary intent → an ordinary comment, never a header name.
- **Self-admission (MRES-4b).** A single package implicitly **self-admits** (its
  own instances always in play) — single-package / catalog-dev is
  zero-ceremony. A `program` file is required **exactly when ≥2 packages
  contribute instances** across unit boundaries. A `package` file self-admits
  its own deps so a library-with-deps is buildable/testable in isolation.
- **Entry point is SEPARATE (MRES-4a).** Admission is elaboration-time; the
  runtime entry (`main`) is a distinct declaration a `program` file **may**
  co-host. N4 does **not** define the entry declaration — only the admission
  boundary. (If a bare `program` header is all that's needed, do not invent an
  entry syntax.)
- **The two-set distinction (MRES-4c).** `admits` governs two differently-sized
  sets: the **coherence set** = the full transitive closure (mandatory, **no
  filter** — a coherence map with holes is an undetected second-canonical, and
  MRES-6 name-exclusion cannot reach instance-level coherence); the
  **direct-use set** = the explicit `admits` root (+ MRES-4d re-exports, later).
  **The admission gate keys on the direct-use set.** Transitive presence for
  coherence does **not** grant a unit the right to dispatch — reaching for a
  transitively-present instance makes its package a *direct* dependency that
  must be listed; `UnadmittedInstance` names it.
- **Transitive instances flow automatically (MRES-4c).** The program names its
  **direct** instance deps; a direct dep's own committed instance-uses are part
  of the coherent environment automatically (identical source-or-compiled). The
  author does **not** list transitive providers.
- **Provenance is a REQUIRED deliverable (MRES-4).** On resolution, report the
  admitted package an instance came from; on `UnadmittedInstance`, name the
  unadmitted package + the instance. **The two build-now provenance messages are
  resolution-provenance + `UnadmittedInstance`.** (The "name **both** defining
  packages" collision message is **not build-now** — see MRES-4f below; it is a
  compiled-manifest/package-manager-round deliverable.)
- **★ Cross-package collision is impossible in source (MRES-4f — ACCEPTED
  theorem, `origin/main` ADR 0014).** Under §5.3 orphan locality + N2 acyclic
  import + disjoint PKG-1 membership, **at most one package can legally define a
  given `(class, head)` in any single source import graph** (two defining
  packages force `owner(C)→owner(T)→owner(C)`, an `ImportCycle` N2 rejects before
  both register; generalizes to any `(class, head-constructor)` overlap). Source
  coherence is therefore **by construction**. Consequences for this WP: the
  build-now overlap gate is exercised **intra-package** (two `instance C T` in one
  package → §5.5 `OverlappingInstances`); the **two-package** collision + its
  both-package provenance is **removed from the source AC** and **staged to the
  compiled-manifest / package-manager round** (where two independently-compiled
  manifests can collide with no shared acyclic import graph — the PKG-3
  cross-boundary re-check surface). The source suite instead asserts the
  **constructive `ImportCycle` witness** (the two-package same-head attempt
  rejects upstream) as a positive coherence-by-construction case.
- **Scaling is O(total instances), not O(packages²)** (validated on the code:
  `instance_search` = one `HashMap.get` on `(class, head)`, `classes.rs:131/98`;
  overlap = O(1) key test, `elab.rs:4266`). The admission gate adds one O(1)
  set-membership test per resolved instance. Do not introduce a pairwise scan.

## Lane A — spec + conformance (spec enclave)

### Spec — `spec/30-surface/33-declarations.md` (+ `32-grammar.md`, `30-taxonomy`)

1. **`program`/`package` header grammar** (`32-grammar.md` + §33). Anonymous
   `program` and `package` headers, each carrying only an `admits` section:
   `admits` followed by a list of dotted package paths (reuse MRES-2 addressing).
   No name token. Specify the file-role: a `program` file is the admission root
   for a multi-package build; a `package` file makes its file a compiled-package
   boundary that self-admits its deps.
2. **Admission semantics** (§5 instance visibility, extending MRES-4A). State the
   gate: an instance resolved by `instance_search` must have its defining package
   in the **direct-use admitted set** (explicit `admits` root); otherwise
   `UnadmittedInstance`. The gate **composes with** orphan + overlap. Pin the
   **two-set distinction** normatively (coherence set = transitive closure,
   total; direct-use set = explicit admits). Pin **self-admission** (single
   package / `package` file) and the **≥2-package** requirement for a `program`
   file.
3. **Provenance diagnostics** normative — the **two build-now** messages
   (resolution provenance; `UnadmittedInstance` names package+instance). State the
   **MRES-4f** constructive invariant (source coherence is by construction; at
   most one package defines a `(class, head)` in an acyclic source graph) and mark
   the **both-package collision** message as compiled-manifest/package-manager
   round (not build-now). **Restore/retain the locked O(total-instances) scaling
   statement** — the coherence pass is one keyed collision test per structure
   instance, linear in closure instances, **not** O(packages²) (MRES-4f changes
   reachability, not scaling).
4. **Forward-compat rules (SPEC ONLY, mark clearly as package-manager-round /
   post-MRES-9):** the compiled-package **instance manifest** + source==compiled
   equivalence (MRES-4c point 4); **MRES-4d** re-export-carries-instance-surface
   (record the rule; note it cannot arise until MRES-9/N5). State these are
   normative-but-unbuilt so the later round is cheap and drift-free — **do not**
   let them leak into Lane B scope.
5. **`30-taxonomy` touch** if needed for the package-tier / self-admission
   framing (enclave's call; keep minimal).

### Conformance — `conformance/surface/…`

6. **CV golden** for the admission gate. Assert with **specific** error variant /
   resolved provenance, not bare accept/reject:
   - Two instance-providing packages, program admits both → ambient resolution
     succeeds; provenance names the source package.
   - Program's own unit dispatches an instance whose package is **not** in the
     explicit `admits` root (even if transitively present for coherence) →
     `UnadmittedInstance` naming that package + instance.
   - Single package → **self-admits**, zero program file, resolution succeeds.
   - **Intra-package overlap** (two canonical `instance C T` in **one** package)
     → §5.5 `OverlappingInstances`; first registers, second rejects with both
     spans. (This is the source-constructible overlap coverage.)
   - **MRES-4f constructive witness (positive):** a two-package same-`(class,
     head)` attempt forces the closed graph `owner(C)→owner(T)→owner(C)` and
     rejects **upstream** as a specific `ImportCycle` — neither candidate
     registers. Asserts source coherence-by-construction.
   - **Both-package cross-package collision + provenance is RED-UNTIL the
     package-manager / compiled-manifest round** (re-homed there, tied to PKG-3
     cross-boundary re-check); do **not** assert it as a live source case.
   - Admitted-orphan control (admission ≠ orphan gate): pin an **acyclic** source
     graph giving the orphan module the ordinary edges it needs to name the
     external class + head (so it reaches `OrphanInstance`, not `UnboundName`),
     with neither owner importing it back.
   - Anonymous-header discipline: a `program`/`package` header with a name token
     is a **syntax** reject.
   - Mark forward-compat (compiled-manifest / re-export) cases **RED-UNTIL** the
     package-manager round if any are seeded; do not assert unbuilt behavior as
     live.

## Lane B — build (Language team; gated on Lane A landing)

Scope: `crates/ken-elaborator` (parser + instance resolution). Re-verify the
cited code sites at pickup.

1. **Parser** — the anonymous `program`/`package` headers + `admits` path list.
   Reject a name token on either header.
2. **Admission gate** — at instance resolution (`instance_search`,
   `classes.rs:131`), add the O(1) direct-use set-membership test; on miss raise
   **`UnadmittedInstance`** (new surface error) with the package+instance
   provenance. Compose with — do not replace — orphan/overlap (`elab.rs:4266`).
3. **Self-admission** — single package / a `package` file self-admits its own
   deps; a `program` file is required only for ≥2 instance-providing packages.
   Zero-ceremony single-package path must stay green.
4. **Coherence over the source closure** — the coherence check spans the full
   transitive **source** graph (already in the N2 loader's compilation graph);
   one canonical instance per `(class, head)`; O(total instances).
5. **Provenance diagnostics** — implement the **two build-now** messages
   (resolution provenance; `UnadmittedInstance` names package+instance). The
   both-package collision message is **not** built here (MRES-4f: unreachable in
   source; deferred to the package-manager round). Intra-package overlap uses the
   existing §5.5 `OverlappingInstances`.
6. **NO manifest/registry/lockfile code** — the compiled-package path is
   deferred. Source packages are transparent (units join the parent graph); a
   `package` header marks a compiled-package boundary in spec, but the compiled
   *artifact*/manifest is not built here.

## Acceptance criteria

- **AC1 (spec).** §33/§32 carry the anonymous `program`/`package` + `admits`
  grammar; §5 carries the admission gate, the two-set distinction,
  self-admission, and the ≥2-package rule; provenance is normative; the
  forward-compat manifest/re-export rules are recorded and **clearly marked
  unbuilt**.
- **AC2 (golden).** The suite asserts: admitted → ambient success with
  provenance; unadmitted direct-use → `UnadmittedInstance` (named); self-admit
  single package → success; **intra-package overlap → `OverlappingInstances`
  (both spans)**; **the two-package same-head attempt → upstream `ImportCycle`
  (MRES-4f constructive witness)**; named header → syntax reject. Specific
  variants, not bare accept/reject. **The both-package cross-package collision +
  provenance case is RED-UNTIL the package-manager round** (not a live source
  assertion).
- **AC3 (build, Lane B).** Parser accepts anonymous headers + `admits`, rejects
  named headers; admission gate raises `UnadmittedInstance` with provenance and
  composes with orphan/overlap; self-admission keeps single-package green;
  coherence spans the source closure **O(total instances)** (one keyed test per
  instance, not O(packages²)); intra-package overlap reaches §5.5.
  `scripts/ken-cargo test -p ken-elaborator` green **and** literal
  `cargo build --workspace --locked &&
  cargo test --workspace --locked` green.
- **AC4 (boundary).** Lane A: spec + conformance only. Lane B:
  `crates/ken-elaborator` (+ tests) only. **Zero** kernel/prelude/Cargo/lock/
  `trusted_base()` delta each lane (admission is surface/elaboration —
  `UnadmittedInstance` is a *reject*/completeness, never an admittance of
  anything ill-typed; the flattened `Σ` the kernel receives is unchanged).
  **No manifest/registry/lockfile code.** `git diff --check` clean.

## Review

Lane A: enclave gates (spec-leader + CV) then **Architect-terminal** (he owns
ADR 0014; the buildable-now-vs-deferred boundary and the two-set distinction are
his to validate). Lane B: **Architect-terminal** (the admission-gate placement +
compose-not-replace + the O(total) scaling are his design). Team QA runs AC3 over
the literal locked CI (the N2 carry). Hand each lane's SHA to Steward; Steward
publishes (Lane A doc-only; Lane B code → CI-polled).

## Do-not-reopen guardrails

- **Anonymous headers** — no name token; identity is the path.
- **Self-admission** — single package / `package` file self-admits; `program`
  required only for ≥2 instance-providing packages.
- **Two-set distinction** — coherence = transitive closure (total); admission
  gate keys on the explicit direct-use set. Do not filter the coherence set.
- **Compose, don't replace** — the admission gate is additive to orphan/overlap.
- **MRES-4f — cross-package collision is impossible in source** — do NOT restore
  a source-world two-package `OverlappingInstances`/both-package-provenance
  fixture or AC; source coherence is by construction (the `ImportCycle` witness),
  overlap coverage is intra-package, both-package collision is
  package-manager-round only. Keep the O(total-instances) scaling statement.
- **No compiled-package build** — manifest/registry/lockfile and MRES-4d
  re-export-instance-surface are forward-compat spec only (package-manager round
  / post-MRES-9). No code.
- **No entry-point syntax** — MRES-4a keeps entry separate; N4 is admission only.
- **Surface-only** — zero TCB delta; `UnadmittedInstance` is a reject.
