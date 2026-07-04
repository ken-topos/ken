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

**Manifest FORM — the type IS the manifest (operator, 2026-07-04, second ruling).**
The authority declaration lives **on `main`'s signature**, not in a separate
construct: enrich the bare surface `Cap` (`prelude.rs:894`, today a zero-structure
opaque `Cap : Type0`, flagged "NOT the real Cap_FS") into an **effect-and-authority
-indexed type** (e.g. `Cap FS` at a declared authority *level* — none/partial/full
per `Authority`, `capabilities.rs:31`; **not** path-scope, which stays deferred).
So `main`'s type states *both* which effect **and** how much authority it uses —
capability use is explicit and auditable *where it is used*. The CLI reads that
authority off `main`'s type and mints exactly it. Rationale: the effect is already
explicit via the effect row, so the authority belongs on the **same signature**,
not bolted on elsewhere. The rejected alternative (a separate manifest declaration
/ annotation carrying authority away from the signature) is **do-not-reopen**.
D2 realizes this; D3's runtime rep must carry the type-declared authority
(D2↔D3 coupled).

This is an **extension of the OQ-B capability model**, decided by the operator
via AskUserQuestion (twice: the declared-authority contract, then this
type-is-the-manifest form). **Do not relitigate** to the rejected alternative ("CLI
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

### D2 — Manifest = enriched signature + entry-point mint (operator's ruling)

**The manifest lives on `main`'s type (operator, 2026-07-04): enrich the bare
`Cap` into an effect-and-authority-indexed type** so `main`'s signature declares
its exact FS authority *level*. `read_bytes` re-types to take the enriched cap;
the authority ordering (none ⊑ partial ⊑ full) governs whether `main`'s declared
cap satisfies `read_bytes`'s requirement. The **representation** of the authority
index (type-level `Authority` value / refinement / index), how the ordering
flows, and keeping the enriched `Cap` a **prelude/elaborator** construct (AC1: no
new kernel `Term`/`Decl`) are the enclave's elaboration — route back to Steward →
operator **only if the representation itself forks**. Then the CLI
(`ken-cli/src/main.rs::run_file`, the gap site) must, before `run_io`:
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

**M** (grew from S–M on the operator's type-is-the-manifest ruling). The CLI
plumbing (mint-exactly, `apply`, `run_io`) is small and the `lines` helper is
trivial — but D2 now includes a **cap-type enrichment** (bare `Cap` →
effect-and-authority-indexed), a type-level design step, not just plumbing, with
D3 coupled to it. The **design content is front-loaded into the enclave**: the
enriched-cap representation (D2) and its runtime rep (D3) are the coupled judgment
calls a build model should not invent. Still outer-ring — **kernel untouched**
(AC1): the enriched `Cap` is a prelude/elaborator construct.
Risk is contained: kernel-clean by construction, path-scope deferred, and the
one fork (entry-point contract) is already operator-settled. **This is what
closes VAL2 to zero gaps (16/0).**

## Enclave elaboration — D1 (`lines`) + D2 (enriched-signature manifest)

> Authored by **spec-author** (D1 + D2), grounded against `origin/main@e391d843`
> via `git show` (not the stale worktree). D3 (Cap runtime rep) is Architect's
> and is coupled to D2 below; D4/D5 (conformance) are CV's sibling doc
> `fs-read-file-lines-flip-conformance.md`. Line numbers are perishable — verify
> against the landed code, not this doc.

### D1 — `lines : String -> List String` (pure, total, structural)

**Signature (locked).**

```
lines   : String -> List String        -- split on '\n', terminator semantics
splitNL : List Char -> List (List Char) -- the structural worker
```

**Landed basis (all on `origin/main`; the split needs no new primitive):**

- `type Char = { c : Int | isScalar c }` and `charToInt : Char -> Int = c`
  (`decimal_char.rs`) — `Char` is `Int`-backed by refinement erasure.
- `string_to_list_char : String -> List Char`, `list_char_to_string :
  List Char -> String` (`prelude.rs`, `37 §2.3`).
- `eq_int : Int -> Int -> Bool` (`numbers.rs`).
- `data List a = Nil | Cons a (List a)` (`prelude.rs`).
- `bytes_decode : Bytes -> String` (`bytes.rs`, **partial** — `Neutral` on
  invalid UTF-8) — the `Bytes -> String` bridge the example needs, since
  `read_bytes` yields `Bytes`, not `String`. The fixture is valid UTF-8 so it
  reduces; `bytes_decode`'s partiality is orthogonal to `lines`' totality.

**Newline without a Char literal (the key D1 finding).** The surface has **no
string-escape / Char-literal syntax** — `letter-frequency.ken` states this
verbatim ("the surface has no string escape for embedding a literal newline").
So `lines` cannot obtain `'\n'` as a literal, and it doesn't need to: the
newline test compares each char's **scalar** to the plain `Int` literal `10`
(U+000A) — no Char literal, no new prelude constant:

```
view isNewline (c : Char) : Bool = eq_int (charToInt c) 10
```

**Terminator semantics (SEAM-B, locked with CV).** `lines` uses **`str::lines()`
terminator semantics**: `\n` *terminates* a line; a trailing `\n` does **not**
yield a trailing empty line, but a non-newline-terminated final segment **is** a
line. So `lines "alpha\nbeta\ngamma\n"` = `["alpha", "beta", "gamma"]` (exactly
3, matching the fixture and CV's AC2 oracle), **not** `[..., ""]`. Load-bearing:
CV pins her exact-stdout oracle to this. A naive separator-split would leave the
trailing blank — `lines` must drop it.

**Totality.** `splitNL` recurses structurally on the `Cons` tail — the landed
sound-zone shape (SCT-decreasing, identical to `palindrome.ken`'s
`reverseListChar`). `lines` is a non-recursive composition over `splitNL` + a
structural `map` + a `dropTrailingEmpty` pass. All pass SCT; no `Bottom`.
Illustrative worker (verify spelling at build — explicit type args elided for
readability):

```
-- prepend c onto the first (current) segment
view consFirst (c : Char) (acc : List (List Char)) : List (List Char) =
  match acc {
    Nil           => Cons (Cons c Nil) Nil ;
    Cons seg rest => Cons (Cons c seg) rest
  }

-- split on '\n' as a SEPARATOR (a trailing '\n' yields a trailing "")
view splitNL (xs : List Char) : List (List Char) =
  match xs {
    Nil       => Cons Nil Nil ;
    Cons c cs =>
      match isNewline c {
        True  => Cons Nil (splitNL cs) ;
        False => consFirst c (splitNL cs)
      }
  }

-- lines = terminator semantics: separator-split, then drop the single trailing
-- empty segment iff the input ended in '\n'
view lines (s : String) : List String =
  map list_char_to_string (dropTrailingEmpty (splitNL (string_to_list_char s)))
```

**Placement.** `lines`/`splitNL`/`isNewline`/`consFirst`/`dropTrailingEmpty` are
pure/total/no-cap — prelude **or** the example (build's call; the example is
fine since only `read-file-lines.ken` needs them, per the
no-speculative-helper rule `palindrome.ken` cites).

### D2 — enriched-signature manifest (operator ruling realized)

**The decided shape (operator `evt_6wc3sxtv96cfv`, Architect
`evt_fgkd29xbf35q`): the type IS the manifest.** `main`'s signature carries its
authority; the CLI
reads the authority off the type and mints exactly it — never full, never
ambient.

**Enriched `Cap` former — authority-indexed (`Cap a`, not `Cap FS a`).**

```
data Auth = ANone | APartial | AFull  -- finite Type0 enum: the authority level
-- Cap enriched from bare `Cap : Type0` to authority-indexed:
Cap : Auth -> Type0                    -- opaque former; `Cap APartial` is a type

main       : (cap : Cap APartial) -> FS (Result Unit IOError) -- declares partial FS
read_bytes : (a : Auth) -> Cap a -> Bytes -> FS (Result Bytes IOError)
```

**Why authority-only `Cap a`, not `Cap FS a` (grounded spelling decision).** The
operator's reasoning was: *the effect is already explicit via the effect row, so
the **authority** belongs on the same signature.* Two grounded facts make
authority-only the faithful realization:

1. **`FS` is already bound to the effect *monad*** — `view FS (a : Type) : Type
   = ITree FSOp fs_resp a` (`prelude.rs:924`). A `Cap FS a` form would either
   collide on the name `FS` or need a *second* effect-tag type distinct from the
   monad.
2. **The effect is already manifest twice** — via the `FS (...)` codomain monad
   **and** `CapParam { name, effect: "FS" }` (`algebra.rs:16`). Adding an `Eff`
   index would duplicate it. So `Cap a` adds exactly the missing dimension
   (authority) and nothing else; the effect stays where it already is.

A first-class `Eff` index (`Cap E a`, spec `62 §2.1`'s effect-indexed `Cap E`)
is an **additive** later enrichment — it needs an effect-tag type separate from
the `FS` monad name, and is **not** required for AC4. Deferred, flagged, not
built here.

- `Auth` is an ordinary `data` enum (`elaborate_decl`, like `IOError`) — **data,
  not a proposition**: no Ω / proof-relevance concern (the ordering lives in
  Rust at the CLI + driver; no type-level order proof is needed under α, below).
- **The D2↔D3 shared contract (agreed with Architect):** the `Auth` ctor ↔
  `capabilities::Authority` map — `ANone ↔ AUTH_NONE(0)`, `APartial ↔
  AUTH_PARTIAL(1)`, `AFull ↔ AUTH_FULL(2)` (`capabilities.rs:33-35`).

**AC1 — kernel untouched, CONFIRMED (Architect's flagged verify item,
`evt_fgkd29xbf35q` §AC1).** The enriched former registers via the **same**
`declare_primitive(env, vec![], pi_type, OpaqueType)` path with `pi_type = Auth
-> Type0`. `declare_primitive` (`check.rs:1098`) only requires `classify(&ty)`
to succeed; a Π former type classifies fine (a well-formed type at `Type1`), and
`OpaqueType` imposes **no** kind restriction. So the higher-kinded opaque former
is registrable with **zero `ken-kernel` delta, no new `Term`/`Decl` variant** —
`Cap a` is application of an opaque former to an argument. **No STOP
condition.**
`Cap` stays one opaque postulate (its *type* changes bare `Type0` → `Auth ->
Type0`; **no new `trusted_base` member**; `Auth` is a checked inductive, not a
postulate).

**α — `read_bytes` stays authority-POLYMORPHIC; sufficiency is a RUNTIME check
(forced by locked AC4 + SEAM-A; settled by citation, NOT a fork).**

```
read_bytes : (a : Auth) -> Cap a -> Bytes -> FS (Result Bytes IOError)
FSOp       : Auth -> Type0          -- ReadFile : (a) -> Cap a -> Bytes -> FSOp a
fs_resp    : (a : Auth) -> FSOp a -> Type   -- = Result Bytes IOError
```

`read_bytes` is polymorphic in `a` — it accepts a cap at **any** declared level
and does **no** static sufficiency check. Sufficiency (`a ⊒ APartial`) is
enforced **only** at the landed driver `authorizes` gate (`eval.rs:1772`) — the
sole net Architect's Phase-2 isolation-flip proved. This is **α**, and it is
**forced**:

- **SEAM-A (CV, load-bearing).** AC4's insufficient arm is a `main` declaring
  `Cap ANone` that **keeps its cap param** (so it clears `check_capabilities` —
  the FS effect is still declared), gets a **level-0 cap minted + bound**,
  reaches the driver, and is denied at `authorizes` with `CapabilityDenied`.
  *"Declares `ANone`" is NOT "provide no cap"* — the cap is minted and bound; it
  is simply insufficient at the runtime gate. (Distinct from a `main` with
  **no** cap param → `MissingCapability` at *elaboration* — a different,
  wrong-reason arm.) **So D2 admits a bound sub-`PARTIAL` declaration reaching
  the driver —
  SEAM-A confirmed, no fork.**
- A **static** minimum-gate (β: index-subtyping, or an ordering-obligation
  `Sufficient APartial a`) would reject the `ANone` main at **elaboration**, so
  it never reaches the driver — contradicting locked **AC4** ("refused **at the
  driver** with `CapabilityDenied`") and deadening the sole runtime net. AC4
  settles it: **α**. The ordering `ANone ⊑ APartial ⊑ AFull` is consulted by the
  **CLI** (mint-exactly) and the **driver** (permit/deny), never as a
  compile-time gate on `read_bytes`.

**The CLI manifest → mint-exactly → bind sequence (`ken-cli/src/main.rs::
run_file`, the gap site `:133`).** Before `run_io`:

1. **Read the declared authority off `main`'s type.** `elab_env.env.lookup
   (main_id)` → the decl's type (a Π-telescope). Walk to the `using cap`
   Π-domain (head `Cap`), read its `Auth` index argument off the `Cap a`
   application (`a` is a `Term::const_(auth_ctor_id)`), and map the ctor id to a
   `capabilities::Authority` via the shared contract. If `main` has **no** FS
   cap param → mint/bind **no** FS cap.
2. **Mint exactly that.** `capabilities::Cap::mint(authority, "FS")` — never
   `AUTH_FULL`, never ambient (the locked ruling). The level is a **structural
   read of the type**, not a computed value → Architect's non-widenable
   constraint is met by construction: a surface program cannot inflate its own
   type index without rewriting the visible declaration (the audit point).
3. **Bind + run.** `apply(main_term, EvalVal::Cap(minted))` (D3's opaque
   `EvalVal::Cap(capabilities::Cap)`), then `run_io` the resulting `ITree`.

**D2↔D3 join (agreed with Architect, `evt_fgkd29xbf35q` §3).** The runtime cap
value is D3's opaque `EvalVal::Cap(capabilities::Cap)` — the struct carrying
`authority_val` + `effect` — produced by the **sole** CLI mint site, carrying
**exactly** the `Authority` the type index declared; `authorizes` matches the
opaque variant (`_ => false`
fail-closed on anything else). "Granted == declared" (AC4) is thus a *structural
read* of `main`'s type — strictly tighter than the frame's vaguer "declared."

**Build-verification points (grep the producer, don't assume):**

- **Cap-param detection must recognize a `Cap a`-headed param**, not only bare
  `Cap` — the enrichment changes the domain from `Const(Cap)` to `App(Cap, a)`.
  Confirm whatever populates `CapParam` from surface binders keys on the `Cap`
  **head** through the application spine, else `using cap : Cap APartial` stops
  being seen as a cap param and `check_capabilities` regresses.
- `check_capabilities` keys on `CapParam.effect` (the string) — **unaffected**
  by the type-index enrichment; verify it still passes for the re-authored
  example and that `read_bytes_untracked_is_type_error` stays green (the example
  keeps its `using cap` param).
- `FSOp`/`fs_resp`/`ReadFile` thread the `Auth` index (`FSOp a`) — mechanical,
  but re-check the `ReadFile` ctor field type is `Cap a` for the specific `a`.

**Considered + rejected:** (a) a separate manifest *value* declaration —
rejected by Architect's non-widenable constraint (a runtime value, not a
signature property) and superseded by the operator's type-is-the-manifest
ruling; (b) β static sufficiency gate on `read_bytes` — rejected by AC4 +
SEAM-A (see α); (c) `Cap FS a` with a first-class `Eff` index — deferred (the
`FS` name collides with the effect monad, and the effect is already manifest via
the codomain + `CapParam`; additive later, not needed for AC4).

**Scope:** M (per Steward). D1 is trivial and independent. D2 is a cap-type
enrichment coupled to D3, but AC1-clean end-to-end and settled by citation on
every axis (α by AC4; non-widenable by construction; higher-kinded former by
`declare_primitive`) — **no residual fork.**
