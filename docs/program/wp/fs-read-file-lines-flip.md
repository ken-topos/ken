# fs-read-file-lines-flip — close VAL2's last gap end-to-end (read-file-lines → 16/0)

**Steward frame → spec enclave (elaborate the manifest mechanism + Cap runtime
rep) → Runtime build.** Named follow-on to FS-driver Phase 2 (`e391d843`, PR
#281), which merged the `[FS]` **substrate** (real `read_bytes` reduction +
`run_io` driver + runtime capability gate + hermetic fixture) and closed the
root gap `GAP-fs-read-unwired`. This WP closes the **last mile**: an actual
`read-file-lines.ken` program that reads the fixture **through a real elaborated
surface `using cap` program** — not the hand-fed `EvalVal::Int` cap the Phase-2
acceptance tests use — flipping the rosetta corpus from **15 PASS / 1 gap → 16
PASS / 0 gap**.

Owner: **Runtime** builds (touches `ken-cli` entry-point + `ken-interp` cap
representation + prelude helper + the example + e2e tests). Gate: enclave
elaboration (Architect soundness on the Cap runtime rep + CLI entry-point;
conformance-validator on the e2e/discriminator plan) → Runtime-QA + Verify-QA +
CI. Findings → **Steward**.

## ★ THE LOCKED DECISION — DO NOT REOPEN (operator, 2026-07-04)

The one genuine design fork this follow-on surfaced was the **top-level
capability entry-point contract** — a top-level `main : using cap : Cap FS -> FS
(...)` elaborates to a real λ expecting a `Cap FS` argument, but the program
**cannot obtain one**: `mint` is deliberately **not surface-callable** (that is
what makes caps unforgeable, the OQ-B lock), and the CLI mints/binds nothing. So
the cap can only appear the way Phase-2's tests inject it — hand-fed as a bare
`EvalVal::Int` at the runtime layer, bypassing the surface entirely.

**The operator ruled the contract: DECLARED-AUTHORITY MANIFEST.**

> `main` **declares the exact authority it needs**; the CLI (the trust root)
> grants **precisely that** — never full authority, never ambient authority at
> the root. Least-privilege at the entry point: `main` cannot receive more
> authority than it declared.

This is an **extension of the OQ-B capability model**, decided by the operator
via AskUserQuestion. **Do not relitigate** to the rejected alternative ("CLI
mints a *full-authority* root cap and binds it to `main`") — that was explicitly
**not** chosen; ambient/full grant at the root is out of bounds. The contrasting
option the operator rejected is recorded so no build model re-opens it as "the
obvious simplification."

**Also locked (inherited from FS-driver, still binding):**
- **Cap is unforgeable / `mint` is not surface-callable.** A surface program can
  only *narrow* (`attenuate`), never widen or construct authority. The entry
  point is the **only** place a cap is minted, and it mints **exactly** the
  declared authority.
- **Kernel / `trusted_base` untouched.** The whole delta is outer-ring
  (`ken-cli` entry-point, `ken-interp` cap representation, `ken-elaborator`
  prelude/manifest, the example, tests). **Zero `ken-kernel/`, zero
  `trusted_base` delta, no new `Term`/`Decl` variant** — grep-verified, not a
  test (AC1).
- **Totality preserved.** The mint+bind is outer-ring (CLI/driver); the pure
  core stays total; an insufficiently-authorized `main` gets `CapabilityDenied`
  as a **total in-language `Result` value**, never a panic.
- **Path-scope stays deferred (authority-LEVEL only).** The landed
  `authorizes(cap, path)` (`ken-interp/src/eval.rs`) **ignores the path** —
  authority is the scalar `Authority(u8)` (`capabilities.rs`), path-scope is the
  deferred `R2′` / Phase-2 item (FS-driver.md D3). So the manifest this WP builds
  declares **authority-level** (e.g. "read-level FS authority"), **not**
  path-scope. Do **not** expand into path-scope realization here — it stays a
  named residual, consistent with the substrate.

## ★ LANDED SUBSTRATE (fixed inputs — build against THIS, verify at pickup)

Grounded against `origin/main@e391d843`. Line numbers are **perishable — verify
against the landed code, do not trust this frame over it.** Cite via `git show
origin/main:<path>` (this `steward/work` worktree checks out stub crate
`lib.rs`s). The headline: **everything downstream of the cap *value* is built and
tested; the only gap is the entry point that *produces* that value from a
surface `main`.**

- **`read_bytes` is landed at the new signature.** `read_bytes : Cap FS -> Path
  -> FS (Result Bytes IOError)`, a real `view` reducing `read_bytes cap path` to
  `Vis (ReadFile cap path) (λ r. Ret r)` — the cap is carried **inside** the op
  node (`ken-elaborator/src/prelude.rs`, ~the `read_bytes` view registration;
  `bytes.rs` for the effect row). No syscall in `eval`.
- **The `run_io` FS driver arm is landed.** `ken-interp/src/eval.rs`'s `run_io`
  `ReadFile` arm pulls the cap from `op_args`, calls `authorizes(cap, path)`
  (the load-bearing runtime gate — Architect's Phase-2 isolation-flip proved it
  is the *sole* net), and on success does the one real `std::fs::read`. The gate
  decodes the cap as `EvalVal::Int(level)` today and checks `level >=
  READ_BYTES_REQUIRED_AUTHORITY` (`= capabilities::AUTH_PARTIAL`).
- **`Cap` is a surface-unconstructible `OpaqueType` postulate.** No surface
  constructor, no `EvalVal::Cap` variant (`prelude.rs` Cap registration). The
  *only* way a concrete authority reaches the driver today is the Rust-side
  `EvalVal::Int(level)` projection — and that lives **only in the Phase-2
  acceptance test** (`ken-interp/tests/fs_driver_build_capability_acceptance.rs`,
  the `cap_evalval` hand-feed). **Blessing that `EvalVal::Int` projection as
  *the* runtime cap representation vs. adding a real opaque `EvalVal::Cap`
  carrying the `Cap` struct is the D3 decision below — Architect's soundness
  call.**
- **`capabilities::Cap::mint(authority, "FS")` is `pub` in `ken-elaborator`**,
  which `ken-cli` **already depends on** — so minting a cap *at the CLI entry
  point* is reachable (`capabilities.rs`, `mint` ~L60; note its doc: "not
  callable from Ken's surface language" — the entry point is Rust, not surface,
  so this is consistent, not a violation). `attenuate` (monotone-downward) and
  `authority` are the levers the CLI uses to grant **exactly** the declared
  level.
- **`ken_interp::apply` exists** and binds a value to `main`'s `Cap FS`
  Π-parameter — the mechanism the CLI uses to bind the minted cap to `using
  cap`.
- **THE GAP (crisp).** `ken-cli/src/main.rs::run_file` evaluates `main_term =
  Term::const_(main_id, vec![])` and hands it straight to `run_io`. It **never
  mints a `Cap`, never inspects `main`'s type for cap params, never `apply`s an
  argument to `main`.** A `main : using cap : Cap FS -> FS (...)` therefore
  `eval`s to a **closure**, which `run_io` rejects as `NotAnIOTree`. There is no
  top-level cap provider. **This WP builds exactly that provider, per the
  manifest contract.**
- **Static capability face is landed + must stay green.**
  `ken-elaborator/src/effects/check.rs::check_capabilities` rejects a `[FS]` decl
  lacking `using cap : Cap FS` at **elaboration**
  (`read_bytes_untracked_is_type_error`). The re-authored example **has** the
  `using cap` param, so it clears this gate; do not regress it.
- **The hermetic fixture is landed.** `conformance/fs/fixtures/three-lines.txt`
  = `alpha\nbeta\ngamma\n` (checked in, read through the real driver). The
  re-authored example reads **this** fixture.
- **`splitOn`/`lines : String -> List String` does NOT exist.** Confirmed absent
  from `prelude.rs` and all of `crates/` on `origin/main`. It is a straightforward
  pure-total prelude addition (D1).

## Means are the enclave's + Architect's call

This frame fixes the **goal + properties + acceptance**, not the mechanism.
Within the operator's locked manifest contract, the enclave elaborates: **how
`main` declares its exact authority** (the manifest form — a type-level
authority on the `Cap` param, an annotation, or a small manifest construct), the
**Cap runtime representation** (D3, Architect soundness), the CLI's
manifest-read → mint-exactly → bind sequence, and the `lines` helper's shape.
Illustrative Ken/Rust below is tagged *verify/decide against the landed system,
not this line.*

## Mandated deliverable outline (each item resolves to a concrete choice)

### D1 — `lines` / `splitOn` helper (pure, total)

A total `lines : String -> List String` (splitting on `\n`), and/or `splitOn :
Char -> String -> List String` it is built from. Pure, total, structural — no
effect, no capability. Lives in the prelude (or the example, enclave's call).
The re-authored example uses it to turn the read bytes/string into the
line list. **Smallest, most mechanical piece — pin its signature and totality.**

### D2 — Manifest entry-point contract (the operator's ruling, realized)

The CLI (`ken-cli/src/main.rs::run_file`, the gap site) must, before `run_io`:
1. **Read `main`'s declared authority** — the manifest. Inspect `main`'s type for
   its `using cap : Cap E` parameter(s) **and the exact authority each declares**.
2. **Mint precisely that authority** — `Cap::mint(declared_authority, effect)`
   (or mint-then-`attenuate` to the declared level); **never full, never
   ambient**. If `main` declares no FS authority, no FS cap is provided.
3. **Bind** the minted cap to `main`'s `using cap` param via `apply`, then run
   `run_io` on the resulting `ITree`.

*Illustrative only — verify against the landed `main.rs`/manifest form:*

```
-- surface: main declares it needs read-level FS authority (manifest form = enclave's call)
main : using cap : Cap FS -> FS (Result Unit IOError)     -- [verify the manifest spelling]

// ken-cli run_file (Rust, outer ring):
let manifest = read_declared_authority(&main_ty);          // the declaration
let cap = mint_exactly(manifest);                          // precisely declared, never full
let applied = apply(main_term, cap_value(cap));            // bind to `using cap`
run_io(applied, ...);
```

**The load-bearing property (AC4):** the grant is **exactly the declared
authority**. A `main` declaring **sufficient** authority reads the fixture; a
`main` declaring **insufficient** authority (below `READ_BYTES_REQUIRED_AUTHORITY`)
is refused **at the driver** with `CapabilityDenied` — proving the CLI granted
*precisely what was declared*, not a full/ambient cap that would have read
regardless. This is the manifest contract's discriminating pair, now driven by
`main`'s **own declaration**, not a hand-fed attenuation.

### D3 — Cap runtime representation (Architect soundness-lane)

Whether a surface-minted `Cap` flows through `eval`/`run_io` as the current
`EvalVal::Int(level)` projection, **or** a new opaque `EvalVal::Cap(Cap)`
carrying the real `Cap` struct. **Architect's call at elaboration.** The
binding soundness AC: the representation must **not** let a surface program forge
or widen authority — `Cap` stays unforgeable, the runtime value carries
**exactly** the minted authority, and no surface term can fabricate an
`EvalVal::Int`/`EvalVal::Cap` that reads as a wider authority than `mint`
granted. (If `EvalVal::Int` is blessed, state precisely why a surface program
cannot inject one — the `Cap` opaque type has no surface intro, so an `Int`
never *reaches* the cap position except via the CLI mint; if that argument does
not hold, use a real opaque `EvalVal::Cap`.)

### D4 — Re-author `read-file-lines.ken` + delete KNOWN-GAP.md

Replace the `view main : Nat = Zero` placeholder with a real `main` that:
declares its FS authority (manifest, D2), takes `using cap : Cap FS`, calls
`read_bytes cap <fixture-path>`, matches the `Result` (surfacing `IOError`
in-language), and processes the bytes into lines via the D1 helper. **Delete**
`examples/rosetta/read-file-lines/KNOWN-GAP.md` (the residual it names is now
closed). Update the rosetta runner's expected result for `read-file-lines` from
KNOWN-GAP → **PASS**.

### D5 — End-to-end acceptance test (drives a REAL surface program)

An e2e test that elaborates `read-file-lines.ken` and runs it **through the CLI
entry-point path** (manifest → minted-exactly cap → `apply` → `run_io` → driver
→ fixture read → lines), asserting the expected line output (`["alpha", "beta",
"gamma"]` or the chosen shape). **The test must NOT hand-feed the cap** as
`EvalVal::Int(authority)` at the `read_bytes` application — it drives the real
`using cap` program end-to-end (AC3). Plus the D2 least-privilege discriminator
(sufficient-authority `main` reads; insufficient-authority `main` →
`CapabilityDenied`).

## Acceptance criteria (testable)

- **AC1 — kernel untouched.** `git diff origin/main -- crates/ken-kernel/` empty;
  no new `Term`/`Decl` variant; `trusted_base` delta zero. **Grep-verified, not a
  test.**
- **AC2 — corpus 16/0.** `read-file-lines` PASSES in the rosetta runner
  (`crates/ken-cli/tests/rosetta.rs`); `examples/rosetta/read-file-lines/
  KNOWN-GAP.md` deleted; the corpus is **16 PASS / 0 gap**.
- **AC3 — no hand-fed cap (the anti-pattern guard).** The e2e test drives a
  **real elaborated `using cap` program** end-to-end; **no** test constructs the
  cap as `EvalVal::Int(authority)` at the `read_bytes` call site to stand in for
  the surface flow. Verify by grepping the test: the cap originates from the CLI
  manifest→mint path, not a hand-injected `EvalVal`
  ([[conformance-hand-feeds-the-deliverable]]). *(The Phase-2 hand-feed test may
  remain as a driver-gate unit test; AC3 is about the **new** e2e test genuinely
  exercising the surface path.)*
- **AC4 — precisely-declared authority (the manifest contract, load-bearing).** A
  discriminating pair: `main` declaring **sufficient** FS authority reads the
  fixture; `main` declaring **insufficient** authority is refused at the driver
  with `CapabilityDenied`. The outcome flips on `main`'s **declaration** ⇒ the
  CLI grants exactly-declared authority, **never full/ambient**. A CLI that
  minted a full-authority cap would pass the sufficient arm but **fail** the
  insufficient arm (it would read regardless) — so this AC is what pins the
  operator's ruling, not decoration.
- **AC5 — Cap unforgeability preserved.** No surface path constructs or widens a
  `Cap`; `mint` stays non-surface-callable; the D3 runtime representation carries
  exactly the minted authority and no surface term can fabricate a wider one.
  **Grep the producer** (surface intro forms + the eval cap position), not just a
  test.
- **AC6 — totality / fail-closed.** The mint+bind is outer-ring; the pure core is
  untouched; an insufficiently-authorized or missing-file `main` yields a **total
  `Result`** (`CapabilityDenied` / `NotFound`), **never a panic**.

## Guardrails — do not reopen

1. **The manifest contract is operator-locked.** `main` declares exact authority;
   CLI grants precisely that; least-privilege; **never full/ambient at the root.**
   Do not "simplify" to a full-authority root mint (the rejected alternative).
2. **Cap stays unforgeable; `mint` not surface-callable.** Entry point is the
   sole mint site, minting **exactly** the declared authority.
3. **Path-scope stays deferred (authority-level only).** Do not realize path-scope
   here — the manifest declares authority-level; path-scope is the named residual
   (`R2′`/Phase-2), consistent with the landed `authorizes` ignoring path.
4. **Kernel untouched** (AC1). If any step seems to need a kernel/`Term`/`Decl`
   change, **stop and route to Steward** — it does not (the whole flow is
   outer-ring, mirroring how Phase 2 stayed kernel-clean).
5. **The static capability face stays green** (`read_bytes_untracked_is_type_error`
   and `check_capabilities`); the re-author keeps `using cap`, do not regress it.

## Sequencing (§2c)

1. **Enclave elaboration (Phase 1 of this follow-on).** spec-leader assigns:
   spec-author elaborates the **manifest declaration mechanism** (D2 — how `main`
   declares exact authority, within the operator's ruling) + the `lines` helper
   (D1) contract; **Architect** owns the **Cap runtime representation** (D3
   soundness) + the CLI entry-point trust-level; conformance-validator elaborates
   the **e2e + AC3/AC4 discriminator** plan (drive-the-real-program, guard the
   hand-feed). If the enclave finds the manifest mechanism thin enough to fold
   into the build frame directly (no separate spec merge needed), that's their
   call — but the D3 soundness call and the AC4 discriminator design are
   **enclave-owned**, not left to the build model. **If the manifest mechanism
   surfaces a genuine sub-fork** (e.g. it needs new surface syntax that itself
   forks the design), **route back to Steward → operator** — do not pick.
2. Gate Architect + CV → merge the elaborated frame/spec to `main` (Integrator).
3. **Runtime build** (single team, single branch): D1–D5, kernel-clean, one merge
   Decision (Runtime-leader → Integrator), gate Architect + Runtime-QA + Verify-QA
   + CI. Compact-gate the team first (§2c handoff gate, unconditional for build
   teams).

## Size / risk

**S–M.** The Rust plumbing (mint-exactly, `apply`, `run_io`) is small and the
primitives all exist; the `lines` helper is trivial. The **design content is
front-loaded into the enclave**: the manifest mechanism (D2) and the Cap runtime
representation (D3) are the two judgment calls a build model should not invent.
Risk is contained: kernel-clean by construction, path-scope deferred, and the
one fork (entry-point contract) is already operator-settled. **This is what
closes VAL2 to zero gaps (16/0).**
