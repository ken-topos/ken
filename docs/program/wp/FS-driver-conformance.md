# FS-driver Phase 1 — conformance plan (deliverable 4)

**CV companion to `FS-driver.md`.** Authored by conformance-validator, parallel
to spec-author's semantics (deliverables 1/2/3/5). This is the Phase-1
conformance **plan** — the fixture strategy + discriminating shapes the Phase-2
build wires into `conformance/fs/`; it is **not** the corpus itself (that lands
with Phase 2). It is grounded against spec-author's landed contract (`D1`–`D5`)
and the landed substrate; **cross-refs `D3` for the mechanism these fixtures
discriminate.** Findings → Steward.

## 0. What deliverable 4 must pin

`FS-driver.md`'s AC2/AC3/AC4 turn on conformance evidence. The load-bearing one
is **AC3**, and `D3` fixes its shape: the capability has **two faces**, and each
AC3 arm must state which it pins (the recurring static-vs-runtime effect-AC
split, [[soundness-AC-static-vs-runtime-face]]). Every fixture below keys its
verdict on a **structural** discriminator (a declared `using cap`, a carried
`Cap` token) — never a self-reported flag — so a case cannot pass green-vs-green
under a silently-disabled check (COORDINATION §7).

## 1. Fixture strategy — checked-in, hermetic, real code path

- **Version-controlled fixtures** under `conformance/fs/fixtures/` (e.g.
  `three-lines.txt` = exactly `alpha\nbeta\ngamma\n`). The test reads them
  **through the real driver** (real `ken-interp` reduction → real host read of a
  repo-relative path), **not** a stubbed read and **not** a mock/virtual FS
  (the operator's locked decision). Determinism comes from the fixed *input*,
  not from faking the code path.
- **No nondeterministic surface.** Content only — no `mtime`/`ctime`/size/inode
  assertions, no absolute host paths (repo-relative, resolved against a fixtures
  root the harness passes). Expected output is pinned to the fixture **bytes**.
- **Grep-verifiable no-mock (AC4).** The Phase-2 build must expose, for grep: the
  reduction dispatches to a real `std::fs`-backed driver (the `run_io` file-I/O
  analog), and there is **no** `MockFs`/`VirtualFs`/in-memory shim. AC4 is a
  producer-grep, not a test — consistent with `D5`'s "`eval.rs` FS interception
  calls **no** `std::fs`" (the pure core builds the `ITree`; only the *driver*
  touches the disk).

## 2. AC3 — the two-face capability net (aligned to `D3`)

`D3` delivers **both** faces; deliverable 4 pins the concrete fixtures for each.

### 2a. Static face — is `[FS]` authority *declared*? (LANDED gate)

Grounded on the landed `check.rs::check_capabilities` (`EffectError::
MissingCapability` at **elaboration**). Non-degenerate pair:

| # | Program | Expected |
|---|---|---|
| S1 | `read_bytes cap path` in a decl **with** `using cap : Cap FS` | **elaborates** |
| S2 | same call in a decl with **no** `using cap : Cap FS` / no handler | **elaboration REJECT** — `MissingCapability` |

Discriminates: `[FS]` access requires a *declared* capability at compile time.
This face is already landed + tested; the fixtures re-pin it so a D1 re-type
regression surfaces here.

### 2b. Runtime face — does `authorizes(cap, path)` *gate the syscall*? (NEW)

The load-bearing thread this WP adds. Non-degenerate pair, **same op**, outcome
flips on the carried authority:

| # | Program (all **statically valid** — declare `using cap`) | Expected |
|---|---|---|
| R1 | cap minted/attenuated **sufficient** for the fixture | **reads** the fixture bytes |
| R2 | **same op**, cap attenuated to **insufficient** authority | **`CapabilityDenied`** at the driver, **no read** |

- **The negative must reach the refusal ([[conformance-hand-feeds-the-deliverable]]).**
  R2 **declares `using cap`** so it passes the *static* gate (2a) and actually
  reaches the *runtime* `authorizes` check — otherwise it would fail at
  `MissingCapability` and pin the static face, not the runtime one (a silent
  face-conflation). This is the subtle design `D3` flags: "a no-op `authorizes`
  (always-true) = ambient authority = fails AC3 — the negative must actually
  reach the refusal." So R2's cap is *present and statically sufficient*, only
  *runtime-insufficient*.
- **Phase-1 vs Phase-2 authority (honest capability pin, T1).** `D3`'s ideal R2
  attenuates the cap to **exclude the path** — but the landed
  `capabilities.rs::Authority(u8)` is a **scalar** (`⊥/partial/⊤`) with **no path
  field**, so path-*scoped* exclusion is **not expressible in Phase 1**. So:
  - **R2 (Phase-1, expressible now):** the cap is attenuated to an
    **insufficient authority level** (`AUTH_NONE`, or below the op's required
    level via `check_authority_sufficient`) ⇒ `CapabilityDenied`. This exercises
    the runtime gate with the scalar we have.
  - **R2′ (Phase-2, known-gap-with-reason):** cap authorizing `dir1` must refuse
    a read of `dir2/file` — the true `authorizes(cap, path)` path-scope. Pinned
    to its capability: lands with the `Authority`→path-set realization (`62
    §2.1`), Architect's/Phase-2's representation call. The **contract is fixed**
    (`authorizes` gates the syscall, `attenuate` only narrows, unauthorized ⇒
    `CapabilityDenied`); only the path-scope *spelling* is `(oracle)`-deferred —
    no over-frozen Phase-2 token.

### 2c. Trust-level honesty (do not over-claim "kernel-backed")

The **runtime path gate** — `authorizes(cap, path)` and the
`check_authority_sufficient`/`authority_flows_to`/`AttenuationObligation::
is_satisfied()` it rests on (a plain Rust `bool`) — is **trusted Rust-level**
logic in `capabilities.rs`, **no** `declare_postulate`/`Obligation` emission
([[kernel-backed-claim-grep-the-emission-not-the-name]]). **Erratum (my prior
draft over-claimed): this is distinct from `attenuate`'s *static* refinement
obligation, which IS kernel-re-checked** (`discharge_attenuation` →
`declare_postulate`, `capabilities.rs:107,159`; `62 §3`) — that governs
monotone-downward attenuation soundness at **elaboration**, not the runtime
path gate. Do **not** conflate them or let the runtime gate borrow the static
obligation's kernel-backing (grep the emission, not the name — a flat "the cap
checks are trusted" is itself wrong). So AC3's **runtime** enforcement is a
**tested-not-trusted** posture: the discriminating pairs (S1/S2, R1/R2) are the
net; the reachability precondition is that FS ops run only on kernel-admitted
core ([[tested-not-trusted-posture-needs-reachability-precondition]]). Enforced
and conformance-netted — **not** kernel-proved. (Not a soundness hole; the
framing must be precise, matching the Sec-lane trust level, and consistent with
`FS-driver.md` D5.)

## 3. AC2 (end-to-end) + AC5 (failure-surfacing) — two distinct failure modes

`D2`/`D5` surface failure as a **total `Result`**; note that **capability-denial
and I/O-failure are different modes** and the plan pins them distinctly:

- **End-to-end (`read-file-lines` → PASS).** With R1's sufficient cap, the
  rosetta example reads `three-lines.txt` through the real driver,
  `bytes_decode`s, splits on newlines, prints each line → output pinned to the
  fixture bytes. **Dependency flagged, not assumed:** the example needs a
  `splitOn`/`lines : String -> List String` helper **not yet built**
  (`read-file-lines/KNOWN-GAP.md`); pinned as a Phase-2 prerequisite riding the
  `catalog/packages/Data/Collections/Derived.ken.md` floor — AC2's "correct content" is honestly gated on
  it, not silently presumed.
- **I/O-failure surfacing (AC5, distinct from R2's cap-denial).** A
  **statically-valid, runtime-*sufficient*** cap (so the driver *attempts* the
  read) on a **nonexistent** fixture path ⇒ the driver reaches the syscall, gets
  file-not-found, and surfaces a **total `Result Err (IOError)`** the program
  matches; the **pure core stays total**. This is a *different* fixture from R2
  (which is refused **before** the syscall by `authorizes`) — the plan keeps the
  two modes separate so a run distinguishes "denied by capability" from "read
  attempted, I/O failed."
- **EFF6-independence witness (`D3`'s Console-lift note).** The AC2 fixture
  (`read-file-lines`) is authored **FS-only and sequential** — it performs
  `read_bytes` then processes the lines, and its expected behavior asserts **no
  console-commute *equation***. So the fixture is a **conformance witness** of
  `D3`'s claim that the FS path is independent of the deferred EFF6 console-
  commute (`#245`): AC2 passes without the Console-lift. The plan therefore
  **does not** author any FS-`⊗`-Console commute fixture in Phase 1 (that would
  ride the landed `[State]` `Vis (inr o)` pass-through, and only a program
  needing the specific commute *law* would touch EFF6 — out of Phase-1 scope).
  If a future mixed FS+Console fixture is wanted, it rides the pass-through, not
  EFF6 — flagged, not silently assumed.

## 4. Coverage map — every AC has a fixture home

| Frame AC | Conformance evidence (deliverable 4) |
|---|---|
| AC2 (real I/O E2E) | §3 end-to-end fixture + pinned output (gated on `splitOn`) |
| AC3 (cap enforced, both faces) | §2a S1/S2 static (`MissingCapability`) + §2b R1/R2 runtime (`CapabilityDenied`); R2′ path-scope = Phase-2 known-gap |
| AC4 (determinism, no mock) | §1 checked-in fixture + real-code-path + no-`MockFs` grep |
| AC5 (totality) | §3 I/O-failure surfacing (absent path, valid cap → `Result Err`, total) |

## 5. Independence + gate note

Per spec-leader's routing, my deliverable-4 piece is **cross-checked against
spec-author's `D1`–`D5`** (done: §2 aligns to `D3`'s static/runtime split and
its path-scope Phase-2 flag; §3 to `D2`/`D5`; §1/AC4 to `D5`'s no-`std::fs`-in-
`eval` grep). At the merge Decision I will state precisely: my **Spec-fidelity
vote** attests spec-author's `D1`–`D5` (the semantics I did **not** author);
this deliverable-4 plan is **my authored contribution**, for Architect's
soundness review — I do **not** self-review it
([[disclaimed-framing-still-binds-your-own-companion-artifact]]). AC3 seam:
spec-author pins the runtime-enforcement *contract* (`D3`); I pin the
discriminating *fixtures + verdicts* (this file, §2).
