# PX8-H — checked-native heterogeneous continuation composition across the recursor

> **Prerequisite for PX8-F, opened by Architect ruling `evt_3tcjvkcsz02fa`
> (2026-07-18).** After PX8-L landed the finite recursive-declaration lowering,
> validating the unchanged held PX8-F `writeAll` fixture as a throwaway overlay
> surfaced a **second, distinct** compiler boundary: the real `writeAll` now
> compiles, links, opens both files, allocates the buffer, and completes
> `FsReadAt(ReadSome { span: 0/6, transferred: 6 })`, then returns a controlled
> `RuntimeTrap(4)` **before any `FsWriteAt`**. Architect independently reproduced
> it and classified it as **NOT the PX8-L mechanism** (neither cyclic
> normalization nor recursive-`DeclarationRef` CFG lowering) but a **missing case
> in the landed PX7-O/P heterogeneous eliminator-frame mechanism.**
>
> **The mechanism (Architect, verbatim):** at the recursor call site,
> `requires_heterogeneous_deforestation` begins with an empty aggregate-IH set; a
> `Call Var(n)` is classified aggregate-producing **only when that variable is
> already named by a surrounding syntactic computational-match judgment.** Here
> the environment resolves that variable to a real `ComputationalRecursorClosure`,
> but the **syntactic call classifier cannot see it**, so the intervening ordinary
> continuation (`after_read` in the fixture) is **not installed before the
> argument is lowered.** The recursor call path then reconstructs only its
> computational frames, and the dynamic `ReadProgress` carrier reaches
> `lower_dynamic_constructor_match` with the **outer checked-`HostIO` cases**, not
> the intervening ordinary cases — so both real alternatives hit the fail-closed
> "no source case" default. The generic defect: **an environment-resolved
> computational recursive hypothesis feeding a known ordinary match before
> returning to outer computational frames.**
>
> **Causal proof of separability (Architect-reproduced):** flipping only the
> generic known-omission return `-4`→success moves the same artifact to normal
> exit 0 while the output stays empty and no write is emitted — so **weakening the
> fail-closed default merely hides the loss**; the fix must **install the missing
> frame**, not relax the default.
>
> **★ The immutable downstream discriminator is `wp/px8f-buffer-io-surface @
> c8b8cdb7` (on origin).** After PX8-H merges, PX8-F rebases onto it and the
> **unchanged** real `writeAll` fixture must compile, link, and perform the real
> writes in **both** the interpreter and native lanes. Do NOT edit that fixture,
> special-case `writeAll`/`after_read`/`ReadProgress`, or weaken the fail-closed
> default (all forbidden by the ruling). Use `c8b8cdb7` only as a **throwaway
> overlay** integration check — never commit its delta into PX8-H.

- **ID:** PX8-H · **Owner:** **Team Runtime** (leader `agt_37reqrd72cg00` /
  implementer `agt_37reqg3nync00` / qa `agt_37reqvb6ce400`) · **Size:** M ·
  **Risk:** High (checked-native compile pipeline; extends the PX7-O/P
  heterogeneous eliminator-frame mechanism and must stay **fail-closed** —
  soundness-adjacent; a compiler capability that only full-workspace CI fully
  exercises).
- **Branch:** `wp/px8h-heterogeneous-continuation-composition` — created from
  **landed PX8-L** (`git branch wp/px8h-heterogeneous-continuation-composition
  origin/main` once PX8-L is merged to `origin/main`; base = the post-PX8-L main,
  fetched, never stale local `main`). One branch, one PR. **Bases on landed
  PX8-L — does NOT publish atomically with it.**
- **Route:** **Architect §14** (extends the heterogeneous eliminator-frame
  mechanism; must preserve constructor identity / binder arity /
  argument-capture-producer-env order / per-frame default / final
  scalar-vs-exit merge, and stay fail-closed — soundness-adjacent) **+ Runtime
  QA**. **+ CV** only if the candidate touches `spec/`/`conformance/` (not
  expected — a compiler/backend mechanism, no surface change). One Decision on
  the tip.
- **Depends on:** **PX8-L merged** (finite recursive-declaration lowering — the
  recursor call must already lower for this composition seam to be reachable).
  **Downstream:** PX8-F resumes only after PX8-H lands.
- **Ownership note (Architect):** Runtime-owned. This extends **Runtime code
  generation** (the PX7-O/P heterogeneous eliminator-frame machinery), **not**
  kernel conversion. **No kernel change.**

## Objective

Extend the checked-native heterogeneous eliminator-frame mechanism so that an
**environment-resolved `ComputationalRecursorClosure` whose result feeds a known
ordinary match before returning to outer computational frames** lowers correctly:
the **intervening ordinary continuation is installed before the recursor
producer's argument is lowered**, and the ordered heterogeneous continuation
(recursive-IH computational frame(s) → intervening ordinary consumer → outer
active frames) is preserved end-to-end. This is a **general language-mechanism**
fix (any env-bound recursive hypothesis feeding an ordinary match), proven on a
**resource-independent** program — **not** a `writeAll`/buffer/`ReadProgress`
patch. PX8-F is unblocked as a downstream consequence, not as this WP's
deliverable.

## Fixed inputs — DO NOT REOPEN (Architect ruling `evt_3tcjvkcsz02fa`; settled)

1. **This is a distinct capability from PX8-L.** PX8-L's finite-SCC + recursive
   `DeclarationRef` lowering is banked and green; do NOT re-open or re-couple it.
   PX8-H is the missing PX7-O/P composition case, independently testable.
2. **The fix is to INSTALL the missing frame, never to weaken the default.** The
   fail-closed "no source case" default is being *reached* because a required
   consumer frame is absent. `lower_dynamic_constructor_match` stays fail-closed:
   known omitted alternatives retain their exact source default, unknown tags
   retain malformed-tag handling, **no missing case becomes success.**
3. **Value-aware classification, NOT a broad heuristic.** Recognize the
   env-resolved `ComputationalRecursorClosure` at the call seam (a value-aware
   call seam **or** an explicit checked-host IR composition form is acceptable);
   a broad **"every `Call Var` is an aggregate"** heuristic is **forbidden** (it
   would mis-classify ordinary calls and regress PX7-O's ordinary dynamic lane).
4. **Preserve exact frame fidelity:** constructor identity, binder arity,
   argument → capture → producer-environment order, **each frame's own default**,
   and the final scalar-versus-exit merge discipline.
5. **No special-casing, no new representation, no kernel change.** Do NOT
   special-case `writeAll`, `after_read`, `ReadProgress`, buffers, resources, or
   any operation identity; add **no** dynamic heap aggregate, **no** ABI/wire
   representation, and **no** kernel change.
6. **`c8b8cdb7` is the immutable downstream integration discriminator** — a
   throwaway overlay only; never edited, never committed into PX8-H.

### Fifteenth-stop scope correction — checked scalar join plan

Architect ruling `evt_7sdmvyme8qy50` makes the scalar join contract an
admission-derived input, never an erased-body inference.  The checked native
production transaction must derive a canonical compiler-private join plan from
the exact admitted declaration type and Match motive, bind each answer to the
exact checked-term occurrence, and embed that hash-free plan through the
existing checked-package semantic-metadata path before package hashes are
finalized.  The erased Runtime graph may validate and consume the plan, but may
not invent it.

Each planned site carries an exact occurrence identity, a checked result-type
fingerprint, and one logical answer kind: `Int`, `Bool`, `StructuralNat`, or
`ExitCode`.  `RecursiveBackedge` is compatible bottom-like edge evidence and
never establishes a kind.  The native source machine resolves the plan before
emitting any predecessor, binds it to a fresh dynamic cut and exact outer
cursor, and treats the existing scalar classifiers only as postcondition
checks.  Missing, malformed, stale, transplanted, wrong-version,
identity-inconsistent, unresolved-polymorphic, non-scalar, or uncovered plan
evidence rejects before join-block creation.

This correction authorizes only compiler-private checked-package metadata and
the native-lowering consumer.  It adds no generic Runtime-expression field,
public ABI, interpreter behavior, kernel rule, aggregate join carrier, or
trusted primitive.  Equal-kind adjacent or nested sites remain distinct;
reversing predecessor emission cannot change the planned answer; and `Bool`
versus `ExitCode` must reject even though both use one Cranelift `i64` block
parameter.

## Landed anchors (verify before editing; do not trust frozen line numbers)

- `crates/ken-runtime/src/cranelift_backend.rs` — `requires_heterogeneous_
  deforestation` (empty-aggregate-IH-set start; the syntactic `Call Var`
  aggregate classifier that cannot see an env-resolved closure), the recursor
  call lowering path, and `lower_dynamic_constructor_match` (the fail-closed "no
  source case" / "checked HostIO match had no constructor arm" default). These
  are the PX7-O/P heterogeneous eliminator-frame machinery landed earlier.
- The PX7-O tests (heterogeneous eliminator frames) and PX7-P tests
  (constructor-field composition / known-omitted-vs-unknown-tag) under
  `crates/ken-cli/tests/` — the negatives PX8-H **must preserve**.
- `wp/px8f-buffer-io-surface @ c8b8cdb7` (on origin) — the throwaway overlay that
  reproduces the real-world instance (`FsReadAt` completes, traps before
  `FsWriteAt`). Reproduce as an overlay; never commit its delta.

## Mandated deliverables (each ends in a concrete implementable choice)

- **H-D1 — Value-aware recursor call-seam classification.** At the recursor call
  site, recognize an **environment-resolved `ComputationalRecursorClosure`** so
  the intervening ordinary continuation is installed **before** the argument is
  lowered. **Choice:** a value-aware call seam that consults the resolved
  environment binding, **or** an explicit checked-host IR composition form that
  names the ordered continuation — **not** a syntactic-only classifier and **not**
  a broad "every `Call Var` is an aggregate" widening.
- **H-D2 — Ordered heterogeneous continuation preservation.** Install and thread
  the explicit ordered continuation across the recursor call: **recursive-IH
  computational frame(s) → intervening ordinary consumer → outer active
  computational/ordinary frames.** Retain exact constructor identity, binder
  arity, argument/capture/producer-environment order, each frame's own default,
  and the final scalar-versus-exit merge. **Choice:** extend the existing PX7-O/P
  frame-stack construction to carry the intervening ordinary frame in order,
  rather than reconstructing only the recursor's computational frames.
- **H-D3 — Fail-closed preservation of `lower_dynamic_constructor_match`.** The
  dynamic constructor match stays fail-closed after the fix: known omitted
  alternatives keep their exact source default, unknown tags keep malformed-tag
  handling, and **no missing case becomes success.** **Choice:** the fix installs
  the correct source continuation so the real alternatives resolve to their
  intervening-ordinary cases; the generic default path is **unchanged** and still
  reached only for genuinely omitted/unknown cases.

## Required proofs / discriminators (each independently reaching)

- **H-P1 — Resource-independent composition proof.** A fixture where an
  **environment-bound `ComputationalRecursorClosure`** produces a **dynamic
  two-constructor aggregate**, a **known ordinary continuation consumes both
  payload directions**, and an **outer computational consumer observes distinct
  final results** — compiling bounded and running with **interpreter↔native exact
  agreement** at zero and multiple steps. No buffer/resource/`writeAll` content.
- **H-P2 — Frame-load-bearing discriminator.** Removing **only** the intervening
  ordinary frame must recover the exact missing-source-case/default (proves the
  frame is the fix, not incidental).
- **H-P3 — Default-not-a-fix discriminator.** Weakening that default must
  **still fail** the payload/side-effect oracle (proves relaxing the default does
  not substitute for installing the frame).
- **H-P4 — Fidelity controls.** Distinct **inner-ordinary vs outer** defaults;
  **wrong arity**; **capture/producer-env order**; **final-kind** (scalar vs
  exit) controls — each independently reaching and discriminating.
- **H-P5 — Preserved negatives (no regression).** PX7-O negatives (direct and
  call-returned `HostResult` remain on the ordinary dynamic lane) and PX7-P's
  **known-omitted vs unknown-tag** distinction stay green.
- **H-P6 — Downstream integration discriminator (overlay only).** On a throwaway
  overlay of the **unchanged** PX8-F `c8b8cdb7` fixture, the real `writeAll` now
  performs the real writes and matches the interpreter lane (past the
  `FsWriteAt`). Overlay only — **never committed**; this is PX8-F's evidence,
  checked here to confirm the seam is truly closed.

## Acceptance criteria (testable)

- H-P1 compiles bounded and passes interpreter↔native exact agreement (0 and
  multi-step).
- H-P2/H-P3/H-P4 each behave exactly as specified (frame-removal recovers the
  default; weakened default still fails the oracle; arity/order/kind controls
  discriminate).
- H-P5 preserved — PX7-O ordinary-lane + PX7-P known-omitted-vs-unknown-tag
  negatives stay green.
- H-P6 — the unchanged `c8b8cdb7` overlay performs real writes in both lanes.
- `lower_dynamic_constructor_match` remains fail-closed (grep the emission: no
  missing case returns success; unknown-tag path unchanged).
- **No-regression = GREEN IN CI** — run the targeted `ken-cli` native-compile
  suites the change implicates before release; the full-workspace/`--locked`
  gate runs in CI, never locally.
- Route satisfied: Runtime QA + Architect §14, one Decision on the tip.

## Do-not-reopen guard

No `writeAll`/`after_read`/`ReadProgress`/buffer/resource/operation-identity
special-case; no "every `Call Var` is an aggregate" heuristic; no dynamic heap
aggregate; no ABI/wire representation change; no kernel-conversion change; no
weakening of the fail-closed default; no edit to `c8b8cdb7`/`writeAll`. A kernel
API-contract violation, if independently shown, is a **distinct Kernel defect** —
route separately, never buried here. PX8-L's banked lowering is settled — do not
re-open it.

## Sequencing

`PX8-L (landed) → PX8-H → PX8-F (rebased terminal gate).` PX8-H bases on landed
PX8-L, takes its own Runtime QA + Architect §14 + one Decision, and merges on its
own tip. Only after PX8-H lands does PX8-F rebase onto the combined main and
require the **unchanged** real `writeAll` fixture to compile, link, and perform
real writes in **both** interpreter and native lanes as its native evidence.
`c8b8cdb7` stays the immutable downstream discriminator throughout.
