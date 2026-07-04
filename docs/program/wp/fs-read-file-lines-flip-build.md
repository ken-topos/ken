# fs-read-file-lines-flip — Runtime BUILD frame (D1–D5 → VAL2 16/0)

**Steward build frame → Runtime team.** The design is **already elaborated and
merged** (`ce9622dd`, PR #283). This frame does **not** re-design anything — it
turns the two merged spec docs into a **shovel-ready, file-by-file build
checklist** and folds the enclave's **build-verification notes as hard, gating
ACs** so a ~1-year-behind build model executes mechanically without re-opening a
settled call.

Owner: **Runtime** (single team, single branch
`wp/fs-read-file-lines-flip-build`, one merge Decision). Touches
`ken-elaborator` (prelude + effect algebra), `ken-interp` (cap runtime value +
driver decode), `ken-cli` (entry-point mint), the example, and tests. **Kernel
untouched (AC1).** Gate: **Architect** (soundness re-affirm on the built diff) +
**Runtime-QA** + **Verify-QA** + CI. Findings → **Steward**.

## ★ THE AUTHORITATIVE DESIGN — build to THESE, do not re-derive

The full elaborated design is on `main`; read it from your worktree
(`docs/program/wp/`), not from this frame:

- **`fs-read-file-lines-flip.md`** — the frame + **spec-author's D1 (`lines`
  helper) and D2 (enriched-signature manifest)** elaboration, and the
  operator-locked contract. This is the primary build spec.
- **`fs-read-file-lines-flip-conformance.md`** — **CV's D4/D5 conformance
  plan**: the AC3 producer-grep, the AC4 discriminating pair (M-suff /
  M-insuff), the AC2 oracle, the AC5/AC6 faces.

This build frame is the **execution wrapper**: the cross-crate checklist, the
verification-note ACs, and the §2c sequencing. Where this frame and the spec
docs agree, the spec docs are authoritative; where this frame adds a
**build-verification AC**, it is gating. **Line numbers in all three docs are
perishable — verify against the landed code at pickup** (`git show
origin/main:<path>`; this worktree may hold stub crate `lib.rs`s). The spec
grounds against `origin/main@e391d843`; you build on `origin/main@ce9622dd`
(same substrate — `ce9622dd` is docs-only).

## ★ LOCKED — DO NOT REOPEN (carried from the merged spec)

These are **operator-settled or enclave-settled**; a build model must not
relitigate them. Full rationale is in the spec docs — do not re-argue, just
build to them:

1. **Declared-authority manifest (operator, twice).** `main` declares the exact
   authority it needs **on its type**; the CLI grants **precisely that** — never
   full, never ambient at the root. The rejected alternative (CLI mints a
   full-authority root cap) is **do-not-reopen**; do not "simplify" to it.
2. **Design α — `read_bytes` is authority-POLYMORPHIC (forced by AC4).**
   `read_bytes : (a : Auth) -> Cap a -> Bytes -> FS (…)` does **no** static
   sufficiency check. Sufficiency is enforced **only** at the runtime driver
   `authorizes` gate — the sole net. Do **not** add a static minimum-authority
   gate on `read_bytes` (design β): it would reject the insufficient `main` at
   elaboration, contradict AC4, and deaden the runtime net. **α is settled by
   citation, not a fork.**
3. **Kernel / `trusted_base` untouched (AC1, grep-verified).** No `ken-kernel/`
   diff, **no new `Term`/`Decl` variant**, `trusted_base` delta zero. The
   enriched `Cap` is a **prelude/elaborator** construct (a higher-kinded opaque
   former via the same `declare_primitive(…, OpaqueType)` path). `Auth` is a
   checked inductive, **not** a new `trusted_base` member. If any step *seems*
   to need a kernel/`Term`/`Decl` change — **STOP and route to Steward.** It
   does not (the whole flow is outer-ring, mirroring Phase 2).
4. **Authority-only `Cap a`, not `Cap FS a`.** The `Auth` index carries
   authority **level**; the effect stays explicit on the `FS (…)` codomain +
   `CapParam.effect`. Do **not** add an `Eff` index here (`Cap E a`, spec `62
   §2.1`) — it is a deferred additive enrichment, not needed for AC4, and the
   name `FS` already binds the effect **monad** (`view FS`).
5. **Path-scope stays deferred (authority-LEVEL only).** The landed
   `authorizes(cap, _path)` ignores the path. The manifest declares authority
   *level* (`ANone`/`APartial`/`AFull`); do **not** realize path-scope here.
6. **Cap stays unforgeable; `mint` not surface-callable.** The CLI entry point
   (Rust) is the **sole** mint site, minting **exactly** the declared authority.
   No surface intro form for `Cap`/`EvalVal::Cap`.

## ★ BUILD-VERIFICATION NOTES — folded as HARD, gating ACs

These are the enclave's flagged "grep the producer, don't assume" points. Each
is a **gating AC** (BV-*), not advice. They are where a mechanical build
silently regresses a soundness/behavior invariant.

- **BV1 — `using` is NOT a surface keyword (Steward-verified on `main`).** The
  parser has **no `using` keyword** (`crates/ken-parser` — grep confirms none).
  The spec prose `using cap : Cap E` is **documentation shorthand**; the **real
  surface** is a **plain Π binder** `(cap : Cap APartial)`. Author the
  re-authored example and both AC4 arms with the **plain binder**, not a `using`
  keyword. This resolves CV's binder-keyword nit: there is nothing extra to
  spell — a cap param is an ordinary binder whose domain type has head `Cap`.

- **BV2 — cap-param detection must key on the `Cap` HEAD through the app spine
  (Architect note 2, load-bearing).** The enrichment changes a cap-param domain
  from `Const(Cap)` to `App(Cap, a)`. The detection that populates
  `EffectSig.cap_params` from a decl's Π-telescope (feeding
  `check_capabilities`, `crates/ken-elaborator/src/effects/`) **must recognize a
  `Cap a`-headed domain** — i.e. peel the application spine to the `Cap` head —
  else `(cap : Cap APartial)` stops being seen as a cap param and
  `check_capabilities` silently regresses (a `[FS]` decl would elaborate without
  its required cap). **Gate: `read_bytes_untracked_is_type_error`
  (`crates/ken-elaborator/tests/l6_acceptance.rs`) stays GREEN**, and add a
  positive test that `(cap : Cap APartial)` **is** detected as an FS cap param.

- **BV3 — `op_args` index alignment in the `run_io` `ReadFile` arm (Architect
  note 1).** Threading the `Auth` index into `ReadFile : (a) -> Cap a -> Bytes
  -> FSOp a` may **shift the cap off `op_args[0]`** in the driver
  (`ken-interp/src/eval.rs` `run_io` FS arm). The driver's cap-read must track
  the **new** position and **fail closed** (`_ => false` / `NotAnIOTree`) on any
  shape mismatch — under D3's `EvalVal::Cap`. This is a **QA correctness point,
  not a soundness hole** (fail-closed is safe), but a wrong index breaks the
  read path. Gate: the D5 e2e reads the fixture (proves the index is right)
  **and** a malformed-op-args path fails closed, not panics.

- **BV4 — single-effect residual (Architect note 3, carried, do NOT build).**
  `Cap a` (not `Cap E a`) rests cross-effect safety on `CapParam.effect` + the
  runtime effect tag. This is **correct for the single FS effect today**. When a
  **second** cap-gated effect lands, the effect must enter the type (`Cap E a`,
  spec `62 §2.1`). **Record this as a named carried residual in the retro; do
  not build the `Eff` index now** (guardrail 4).

## Cross-crate implementation checklist (D1–D5 → files)

Each item resolves to a concrete edit. Verify every path/line against the landed
code at pickup. Do it in dependency order: **D2-type before D3-runtime before
D2-CLI before D4/D5**; D1 is independent (do it any time).

### D1 — `lines` helper (pure, total) — prelude *or* the example

- Add `lines : String -> List String` (terminator semantics) built from `splitNL
  : List Char -> List (List Char)`, `isNewline`, `consFirst`,
  `dropTrailingEmpty`. **Spec-author locked the exact SCT-safe shapes** (flip
  spec §D1) — transcribe them, verify spelling/type-args at build.
- `isNewline c = eq_int (charToInt c) 10` — **no Char literal** (surface has no
  string-escape; compare the scalar to `Int` `10`).
- **Terminator semantics (SEAM-B, locked):** `lines "alpha\nbeta\ngamma\n" =
  ["alpha","beta","gamma"]` — exactly 3, trailing `\n` yields **no** trailing
  empty segment. `dropTrailingEmpty` enforces this. This pins the AC2 oracle.
- Placement: prelude or the example (your call — the no-speculative-helper rule
  favors the example since only `read-file-lines.ken` needs them). All pieces
  pass SCT (structural recursion on the `Cons` tail); **no `Bottom`.**

### D2a — enriched `Cap` type + `Auth` enum — `ken-elaborator` prelude/algebra

- Add `data Auth = ANone | APartial | AFull` as an ordinary checked inductive
  (via `elaborate_decl`, like `IOError`) — **data, not a proposition**; no Ω /
  no proof-relevance.
- **Enrich `Cap : Type0` → `Cap : Auth -> Type0`** at its registration
  (`prelude.rs`, the `Cap` `declare_primitive`): change the former's type from
  `type0` to `pi(Auth, type0)`, **same** `declare_primitive(env, vec![],
  pi_type, PrimReduction::OpaqueType)` call. **AC1 confirmed by Architect:**
  `classify` accepts the Π former type; `OpaqueType` imposes no kind
  restriction; `Cap APartial ≠ Cap ANone` are distinct stuck neutrals (opaque
  never reduces). **Zero kernel delta**; `Cap` stays one opaque postulate (its
  *type* changes; no new `trusted_base` member).
- Re-type `read_bytes : (a : Auth) -> Cap a -> Bytes -> FS (Result Bytes
  IOError)` (**α, polymorphic** — guardrail 2). Thread the `Auth` index through
  `FSOp : Auth -> Type0`, `ReadFile : (a) -> Cap a -> Bytes -> FSOp a`, `fs_resp
  : (a : Auth) -> FSOp a -> Type` (mechanical; re-check the `ReadFile` ctor
  field is `Cap a`).
- **BV2 applies here** — after enrichment, confirm cap-param detection still
  fires on `App(Cap, a)`.

### D3 — runtime cap value `EvalVal::Cap` + driver decode — `ken-interp`

- Add opaque **`EvalVal::Cap(capabilities::Cap)`** variant (Architect's D3
  ruling — **real opaque struct, NOT `EvalVal::Int(level)`**; SEAM-C resolved).
- **Re-point `authorizes`** (`eval.rs`, ~the landed `:1772` site):
  `EvalVal::Cap(cap) => check_authority_sufficient(cap,
  READ_BYTES_REQUIRED_AUTHORITY, …), _ => false` — read `Authority` off the real
  minted struct; **fail-closed** on anything else. No re-mint from a bare
  scalar.
- **BV3 applies here** — the `run_io` `ReadFile` arm reads the cap from its
  (possibly shifted) `op_args` position, fail-closed on mismatch.
- The `Auth` ctor ↔ `capabilities::Authority` map is the D2↔D3 shared contract:
  `ANone↔AUTH_NONE(0)`, `APartial↔AUTH_PARTIAL(1)`, `AFull↔AUTH_FULL(2)`
  (`capabilities.rs:33-35`); `READ_BYTES_REQUIRED_AUTHORITY = AUTH_PARTIAL = 1`.

### D2b — CLI manifest → mint-exactly → bind — `ken-cli/src/main.rs::run_file`

At the gap site (`run_file`, ~`:133`, before `run_io`):

1. **Read declared authority off `main`'s type.** `lookup(main_id)` → its
   Π-telescope; walk to the cap-param domain (head `Cap` through the app spine —
   **BV2**), read the `Auth` index argument off `Cap a` (`a` is a
   `Term::const_(auth_ctor_id)`), map the ctor id → `capabilities::Authority`
   via the shared contract. **No FS cap param ⇒ mint/bind no FS cap.**
2. **Mint exactly that.** `capabilities::Cap::mint(authority, "FS")` — **never
   `AUTH_FULL`, never ambient** (operator lock). The level is a **structural
   read of the type**, not a computed value → non-widenable by construction.
3. **Bind + run.** `apply(main_term, EvalVal::Cap(minted))`, then `run_io` the
   resulting `ITree`.

### D4 — re-author the example + flip the oracle

- Replace `view main : Nat = Zero` with a real `main : (cap : Cap APartial) ->
  FS (Result Unit IOError)` (**plain binder — BV1**) that: `read_bytes … cap
  <fixture-path>`, matches the `Result` (surfacing `IOError` in-language),
  `bytes_decode` → `lines` (D1) → prints each line.
- **Delete `KNOWN-GAP.md`**; add an **`expected`** file with byte-exact stdout
  `alpha\nbeta\ngamma\n` (AC2 oracle, SEAM-B). Update `rosetta.rs` expectation
  KNOWN-GAP → PASS. Reads the landed hermetic fixture
  `conformance/fs/fixtures/three-lines.txt`.

### D5 — end-to-end acceptance test(s)

- **AC3 (no hand-fed cap):** the e2e enters through `ken-cli`'s `run_file` (or
  the rosetta runner shelling the real CLI binary) so the cap **originates in
  the CLI mint path**. **Grep guard:** the new test contains **no**
  `EvalVal::Cap(…)` / `EvalVal::Int(…)` / `cap_evalval` at the `read_bytes`/
  `apply` site. (The Phase-2 hand-feed test may remain as a driver-unit test,
  updated to `EvalVal::Cap`.)
- **AC4 (precisely-declared, the net):** the **discriminating pair**, identical
  except the declared `Auth` index —
  - **M-suff** `(cap : Cap APartial)` → reads the fixture → line output;
  - **M-insuff** `(cap : Cap ANone)` → **`Err CapabilityDenied`** (assert the
    exact `capabilitydenied_id` payload, **not** a bare
    `NotFound`/`NotAnIOTree`/ panic — SEAM-A: M-insuff **keeps** its cap param,
    clears `check_capabilities`, mints+binds a level-0 cap, reaches the driver,
    denied at `authorizes`). A full-minting CLI passes M-suff but **fails**
    M-insuff — that is what makes the pair load-bearing.
- **AC6 (totality/fail-closed):** a missing-file arm (`main` reading an absent
  path → `Err NotFound`, total, no panic); M-insuff doubles as the fail-closed
  witness.

## Acceptance criteria (testable)

- **AC1 — kernel untouched.** `git diff origin/main -- crates/ken-kernel/`
  **empty**; no new `Term`/`Decl` variant; `trusted_base` delta zero.
  **Grep-verified, not a test.**
- **AC2 — corpus 16/0.** `read-file-lines` PASSES in `crates/ken-cli/tests/
  rosetta.rs`; `KNOWN-GAP.md` deleted; `expected` = `alpha\nbeta\ngamma\n`
  byte-exact; corpus **16 PASS / 0 gap**.
- **AC3 — no hand-fed cap.** Producer-grep passes (cap from CLI mint path, no
  hand-built `EvalVal` at the read site).
  [[conformance-hand-feeds-the-deliverable]]
- **AC4 — precisely-declared authority.** The M-suff/M-insuff pair flips on
  `main`'s **own declaration**; M-insuff → exact `CapabilityDenied` payload at
  the driver. This pins the operator's least-privilege ruling.
- **AC5 — Cap unforgeability preserved.** Grep the producer: no surface intro
  constructs a `Cap`/`EvalVal::Cap`; `mint`/`attenuate` have no surface binding;
  the sole `EvalVal::Cap` reaching the driver is the CLI mint.
- **AC6 — totality / fail-closed.** Insufficient / missing-file → total `Result`
  (`CapabilityDenied` / `NotFound`), **never a panic**. Pure core untouched.
- **BV1 — plain binder**, no `using` keyword.
- **BV2 — cap-param detection keys on the `Cap` head through the app spine;**
  `read_bytes_untracked_is_type_error` green + a positive `Cap
  APartial`-detected test.
- **BV3 — `op_args` index tracked, fail-closed** in the `ReadFile` driver arm.
- **BV4 — single-effect residual recorded** in the retro (not built).
- **No-regression:** `cargo test --workspace` green (not just the touched crates
  — a prelude/effect-algebra change can ripple to downstream `.ken`
  proofs/tests; validate the whole workspace, per the K7 kernel-blast-radius
  lesson applied to the prelude).

## Guardrails — do not reopen

1. Manifest contract operator-locked; never full/ambient root mint.
2. Design **α** (polymorphic `read_bytes`, runtime sufficiency) — do not add a
   static gate (β).
3. Kernel untouched; no new `Term`/`Decl`. Seems-to-need-kernel ⇒ **STOP →
   Steward.**
4. Authority-only `Cap a`; no `Eff` index now (BV4 residual).
5. Path-scope deferred (authority-level only).
6. Cap unforgeable; `mint` non-surface-callable; CLI the sole mint site.

## Sequencing (§2c)

1. **⛔ HANDOFF GATE first (unconditional).** Compact-verify the Runtime team
   (leader + implementer + QA) **before** the kickoff mention — retros in
   (fs-driver Phase-2 retros already IN), no in-flight obligation, quiescent,
   `/compact` each, **verify the ctx drop**, then post. One indivisible act.
2. **Kick off the Runtime leader only** (§2 mention discipline), pointing at the
   two merged spec docs + this build frame on `main`. Leader assigns
   implementer/QA internally.
3. **Runtime builds D1–D5** on `wp/fs-read-file-lines-flip-build`, kernel-clean,
   **one merge Decision** (Runtime-leader → Integrator).
4. **Gate: Architect** (soundness re-affirm on the built diff — AC1/AC5/α
   preserved; the D3 `EvalVal::Cap` decode fail-closed; BV2/BV3 correct) **+
   Runtime-QA + Verify-QA + CI**. Findings → Steward.
5. On merge + retro (BV4 residual recorded) → **read-file-lines PASS → VAL2
   16/0** → close the FS workstream; next is **CV's challenge-suite frontier**
   (already mapped `7d346a47`; residuals logged, not opened per Option A).

## Size / risk

**M.** Outer-ring only, **kernel-clean by construction** (AC1). The design is
fully front-loaded (enclave-elaborated + merged); the build is mechanical
transcription + wiring, with the four **BV** notes as the watch-points. The one
real design fork (entry-point contract) is operator-settled. **This closes VAL2
to zero gaps (16/0).**
